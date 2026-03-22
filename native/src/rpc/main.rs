// MIT LICENSE
//
// COPYRIGHT (R) 2025 ARNELIFY. AUTHOR: TARON SARKISYAN
//
// PERMISSION IS HEREBY GRANTED, FREE OF CHARGE, TO ANY PERSON OBTAINING A COPY
// OF THIS SOFTWARE AND ASSOCIATED DOCUMENTATION FILES (THE "SOFTWARE"), TO DEAL
// IN THE SOFTWARE WITHOUT RESTRICTION, INCLUDING WITHOUT LIMITATION THE RIGHTS
// TO USE, COPY, MODIFY, MERGE, PUBLISH, DISTRIBUTE, SUBLICENSE, AND/OR SELL
// COPIES OF THE SOFTWARE, AND TO PERMIT PERSONS TO WHOM THE SOFTWARE IS
// FURNISHED TO DO SO, SUBJECT TO THE FOLLOWING CONDITIONS:
//
// THE ABOVE COPYRIGHT NOTICE AND THIS PERMISSION NOTICE SHALL BE INCLUDED IN ALL
// COPIES OR SUBSTANTIAL PORTIONS OF THE SOFTWARE.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{
  collections::HashMap,
  io::{Error, ErrorKind},
  sync::{Arc, Mutex, OnceLock, mpsc},
  time::{SystemTime, UNIX_EPOCH},
};

use tokio::runtime::{Builder, Runtime};

static MAP: OnceLock<
  Mutex<HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>>,
> = OnceLock::new();

pub fn generate_request_id() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_nanos()
}

pub type BrokerBytes = Vec<u8>;
pub type BrokerCtx = serde_json::Value;
pub type JSON = serde_json::Value;

struct RPCReq {
  has_body: bool,
  has_meta: bool,

  buff: Vec<u8>,
  binary: Vec<u8>,

  compression: Option<String>,
  binary_length: usize,
  json_length: usize,

  request_id: u128,
  ctx: BrokerCtx,
}

impl RPCReq {
  pub fn new() -> Self {
    Self {
      has_body: false,
      has_meta: false,

      buff: Vec::new(),
      binary: Vec::new(),

      compression: None,
      binary_length: 0,
      json_length: 0,

      request_id: 0,
      ctx: serde_json::json!({
        "topic": JSON::Null,
        "payload": {},
        "request_id": 0,
        "reliable": false
      }),
    }
  }

  pub fn add(&mut self, block: &[u8]) -> () {
    self.buff.extend_from_slice(block);
  }

  pub fn get_bytes(&self) -> Vec<u8> {
    self.binary.to_vec()
  }

  pub fn get_ctx(&self) -> BrokerCtx {
    self.ctx.clone()
  }

  pub fn get_request_id(&self) -> u128 {
    self.request_id
  }

  fn read_meta(&mut self, meta_end: usize) -> Result<u8, Error> {
    let meta_bytes: &[u8] = &self.buff[..meta_end];
    let pos: usize = match meta_bytes.iter().position(|&b| b == b'+') {
      Some(v) => v,
      None => return Err(Error::new(ErrorKind::InvalidData, "Missing '+' in meta")),
    };

    let json_length: &str = match std::str::from_utf8(&meta_bytes[..pos]) {
      Ok(s) => s,
      Err(_) => {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid UTF-8."));
      }
    };

    self.json_length = match json_length.parse() {
      Ok(n) => n,
      Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Invalid meta.")),
    };

    let binary_length: &str = match std::str::from_utf8(&meta_bytes[pos + 1..]) {
      Ok(s) => s,
      Err(_) => {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid UTF-8."));
      }
    };

    self.binary_length = match binary_length.parse() {
      Ok(n) => n,
      Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Invalid meta.")),
    };

    Ok(1)
  }

  fn read_body(&mut self) -> Result<Option<u8>, Error> {
    if !self.has_meta {
      let meta_end: usize = match self.buff.iter().position(|&b| b == b':') {
        Some(pos) => pos,
        None => {
          if self.buff.len() > 8192 {
            self.buff.clear();
            return Err(Error::new(
              ErrorKind::InvalidData,
              "The maximum size of the meta has been exceeded.",
            ));
          }

          return Ok(None);
        }
      };

      match self.read_meta(meta_end) {
        Ok(_) => {}
        Err(e) => {
          self.buff.clear();
          return Err(e);
        }
      }

      self.has_meta = true;
      self.buff.drain(..=meta_end);
    }

    if self.json_length != 0 && self.buff.len() >= self.json_length {
      let message: &str = match std::str::from_utf8(&self.buff[..self.json_length]) {
        Ok(v) => v,
        Err(_) => {
          self.buff.clear();
          return Err(Error::new(ErrorKind::InvalidInput, "Invalid UTF-8."));
        }
      };

      let json: serde_json::Value = match serde_json::from_str(message) {
        Ok(v) => v,
        Err(_) => {
          self.buff.clear();
          return Err(Error::new(ErrorKind::InvalidInput, "Invalid JSON."));
        }
      };

      if json.get("topic").is_none()
        || json.get("request_id").is_none()
        || json.get("payload").is_none()
      {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid message."));
      }

      self.request_id = json["request_id"]
        .as_str()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

      self.ctx = json["payload"].clone();
      self.buff.drain(..self.json_length);
      if self.binary_length == 0 {
        self.has_body = true;
        return Ok(Some(1));
      }
    }

    if self.binary_length != 0 && self.buff.len() >= self.binary_length {
      self.binary = self.buff[..self.binary_length].to_vec();

      self.has_body = true;
      self.buff.drain(..self.binary_length);
      return Ok(Some(1));
    }

    Ok(None)
  }

  pub fn read_block(&mut self) -> Result<Option<u8>, Error> {
    if !self.has_body {
      match self.read_body() {
        Ok(Some(_)) => {}
        Ok(None) => return Ok(None),
        Err(e) => return Err(e),
      }
    }

    Ok(Some(1))
  }

  pub fn reset(&mut self) -> () {
    self.has_body = false;
    self.has_meta = false;

    self.binary.clear();

    self.compression = None;
    self.binary_length = 0;
    self.json_length = 0;
    self.ctx = serde_json::json!({
      "topic": JSON::Null,
      "payload": {},
      "request_id": 0,
      "reliable": false
    });
  }
}

pub struct RPCStream {
  cb_send: Arc<dyn Fn(Vec<u8>) + Send + Sync>,
  topic: String,
  request_id: u128,
}

impl RPCStream {
  pub fn new() -> Self {
    Self {
      cb_send: Arc::new(|bytes: Vec<u8>| {
        println!("{:?}", bytes);
      }),
      topic: String::new(),
      request_id: 0,
    }
  }

  pub fn on_send(&mut self, cb: Arc<dyn Fn(Vec<u8>) + Send + Sync>) -> () {
    self.cb_send = cb;
  }

  pub fn push(&self, payload: &JSON, bytes: &[u8], is_reliable: bool) -> () {
    let json: JSON = serde_json::json!({
        "topic": self.topic,
        "request_id": self.request_id.to_string(),
        "payload": payload,
        "reliable": is_reliable
    });

    let mut buff: Vec<u8> = Vec::new();
    let message: String = serde_json::to_string(&json).unwrap();
    let meta: String = format!("{}+{}:", message.len(), bytes.len());
    buff.extend_from_slice(meta.as_bytes());
    buff.extend_from_slice(message.as_bytes());
    buff.extend_from_slice(bytes);

    (self.cb_send)(buff);
  }

  pub fn push_bytes(&self, bytes: &[u8], is_reliable: bool) -> () {
    let json: JSON = serde_json::json!({
        "topic": self.topic,
        "request_id": self.request_id.to_string(),
        "payload": "<bytes>",
        "reliable": is_reliable
    });

    let mut buff: Vec<u8> = Vec::new();
    let message: String = serde_json::to_string(&json).unwrap();
    let meta: String = format!("{}+{}:", message.len(), bytes.len());
    buff.extend_from_slice(meta.as_bytes());
    buff.extend_from_slice(message.as_bytes());
    buff.extend_from_slice(bytes);

    (self.cb_send)(buff);
  }

  pub fn push_json(&self, payload: &JSON, is_reliable: bool) -> () {
    let json: JSON = serde_json::json!({
        "topic": self.topic,
        "request_id": self.request_id.to_string(),
        "payload": payload,
        "reliable": is_reliable
    });

    let mut buff: Vec<u8> = Vec::new();
    let message: String = serde_json::to_string(&json).unwrap();
    let meta: String = format!("{}+{}:", message.len(), 0);
    buff.extend_from_slice(meta.as_bytes());
    buff.extend_from_slice(message.as_bytes());

    (self.cb_send)(buff);
  }

  pub fn send(
    &self,
    topic: &str,
    payload: &JSON,
    bytes: &[u8],
    is_reliable: bool,
  ) -> (Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>) {
    let (tx, rx) = mpsc::channel();
    let request_id: u128 = generate_request_id();
    let req_topic: String = format!("req:{}", topic);

    {
      let map: &Mutex<
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = MAP.get_or_init(|| Mutex::new(HashMap::new()));
      let mut map_lock: std::sync::MutexGuard<
        '_,
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = map.lock().unwrap();
      map_lock.insert(request_id, tx);
    }

    let json: JSON = serde_json::json!({
        "topic": req_topic,
        "request_id": request_id.to_string(),
        "payload": payload,
        "reliable": is_reliable
    });

    let mut buff: Vec<u8> = Vec::new();
    let message: String = serde_json::to_string(&json).unwrap();
    let meta: String = format!("{}+{}:", message.len(), bytes.len());
    buff.extend_from_slice(meta.as_bytes());
    buff.extend_from_slice(message.as_bytes());
    buff.extend_from_slice(bytes);

    (self.cb_send)(buff);

    rx.recv().unwrap()
  }

  pub fn send_bytes(
    &self,
    topic: &str,
    bytes: &[u8],
    is_reliable: bool,
  ) -> (Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>) {
    let (tx, rx) = mpsc::channel();
    let request_id: u128 = generate_request_id();
    let req_topic: String = format!("req:{}", topic);

    {
      let map: &Mutex<
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = MAP.get_or_init(|| Mutex::new(HashMap::new()));
      let mut map_lock: std::sync::MutexGuard<
        '_,
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = map.lock().unwrap();
      map_lock.insert(request_id, tx);
    }

    let json: JSON = serde_json::json!({
        "topic": req_topic,
        "request_id": request_id.to_string(),
        "payload": "<bytes>",
        "reliable": is_reliable
    });

    let mut buff: Vec<u8> = Vec::new();
    let message: String = serde_json::to_string(&json).unwrap();
    let meta: String = format!("{}+{}:", message.len(), bytes.len());
    buff.extend_from_slice(meta.as_bytes());
    buff.extend_from_slice(message.as_bytes());
    buff.extend_from_slice(bytes);

    (self.cb_send)(buff);

    rx.recv().unwrap()
  }

  pub fn send_json(
    &self,
    topic: &str,
    payload: &JSON,
    is_reliable: bool,
  ) -> (Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>) {
    let (tx, rx) = mpsc::channel();
    let request_id: u128 = generate_request_id();
    let req_topic: String = format!("req:{}", topic);

    {
      let map: &Mutex<
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = MAP.get_or_init(|| Mutex::new(HashMap::new()));
      let mut map_lock: std::sync::MutexGuard<
        '_,
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = map.lock().unwrap();
      map_lock.insert(request_id, tx);
    }

    let json: JSON = serde_json::json!({
        "topic": req_topic,
        "request_id": request_id.to_string(),
        "payload": payload,
        "reliable": is_reliable
    });

    let mut buff: Vec<u8> = Vec::new();
    let message: String = serde_json::to_string(&json).unwrap();
    let meta: String = format!("{}+0:", message.len());
    buff.extend_from_slice(meta.as_bytes());
    buff.extend_from_slice(message.as_bytes());

    (self.cb_send)(buff);

    rx.recv().unwrap()
  }

  pub fn set_request_id(&mut self, request_id: u128) -> () {
    self.request_id = request_id;
  }

  pub fn set_topic(&mut self, topic: &str) -> () {
    self.topic = String::from(topic);
  }
}

pub type BrokerConsumer = dyn Fn(&[u8]) + Send + Sync;
pub type BrokerConsumerHandler = dyn Fn(&str, Arc<BrokerConsumer>) + Send + Sync;
pub type BrokerProducer = dyn Fn(&str, &[u8]) + Send + Sync;

pub type RPCAction =
  dyn Fn(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>, Arc<Mutex<RPCStream>>) + Send + Sync;
pub type RPCLogger = dyn Fn(&str, &str) + Send + Sync;

pub struct RPC {
  cb_logger: Arc<Mutex<Arc<RPCLogger>>>,
  cb_consumer: Arc<Mutex<Arc<BrokerConsumerHandler>>>,
  cb_producer: Arc<Mutex<Arc<BrokerProducer>>>,
}

impl RPC {
  pub fn new() -> Self {
    let rt: Arc<Runtime> = Arc::new(
      Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap(),
    );

    let consumers: Arc<Mutex<HashMap<String, Arc<BrokerConsumer>>>> =
      Arc::new(Mutex::new(HashMap::new()));
    let c_consumers: Arc<Mutex<HashMap<String, Arc<BrokerConsumer>>>> = Arc::clone(&consumers);
    let p_consumers: Arc<Mutex<HashMap<String, Arc<BrokerConsumer>>>> = Arc::clone(&consumers);

    Self {
      cb_logger: Arc::new(Mutex::new(Arc::new(move |_level: &str, message: &str| {
        println!("[Arnelify Broker]: {}", message);
      }))),
      cb_consumer: Arc::new(Mutex::new(Arc::new(
        move |topic: &str, cb: Arc<BrokerConsumer>| {
          let mut map: std::sync::MutexGuard<'_, HashMap<String, Arc<BrokerConsumer>>> =
            c_consumers.lock().unwrap();
          map.insert(String::from(topic), cb);
        },
      ))),
      cb_producer: Arc::new(Mutex::new(Arc::new(move |topic: &str, bytes: &[u8]| {
        let map: std::sync::MutexGuard<'_, HashMap<String, Arc<BrokerConsumer>>> =
          p_consumers.lock().unwrap();
        if let Some(consumer) = map.get(topic) {
          let bytes: BrokerBytes = bytes.to_vec();
          let consumer_safe: Arc<BrokerConsumer> = Arc::clone(consumer);
          rt.spawn(async move {
            consumer_safe(&bytes);
          });
        }
      }))),
    }
  }

  pub fn logger(&self, cb: Arc<RPCLogger>) -> () {
    let mut logger_lock: std::sync::MutexGuard<'_, Arc<RPCLogger>> = self.cb_logger.lock().unwrap();
    *logger_lock = cb;
  }

  pub fn on(&self, topic: &str, cb: Arc<RPCAction>) -> () {
    let req_logger: Arc<Mutex<Arc<RPCLogger>>> = Arc::clone(&self.cb_logger);
    let res_logger: Arc<Mutex<Arc<RPCLogger>>> = Arc::clone(&self.cb_logger);
    let req_producer: Arc<Mutex<Arc<BrokerProducer>>> = Arc::clone(&self.cb_producer);
    let p_req_topic: String = String::from(format!("req:{}", topic));
    let c_res_topic: String = String::from(format!("res:{}", topic));
    let p_res_topic: String = String::from(format!("res:{}", topic));
    let cb: Arc<RPCAction> = Arc::clone(&cb);

    let req_consumer: Arc<BrokerConsumer> = Arc::new(move |bytes: &[u8]| {
      let mut req: RPCReq = RPCReq::new();
      req.add(bytes);

      match req.read_block() {
        Ok(Some(_)) => {
          let ctx: Arc<Mutex<BrokerCtx>> = Arc::new(Mutex::new(req.get_ctx()));
          let bytes: Arc<Mutex<BrokerBytes>> = Arc::new(Mutex::new(req.get_bytes()));
          let stream: Arc<Mutex<RPCStream>> = Arc::new(Mutex::new(RPCStream::new()));
          {
            let mut stream_lock: std::sync::MutexGuard<'_, RPCStream> = stream.lock().unwrap();
            stream_lock.set_topic(&c_res_topic);
            stream_lock.set_request_id(req.get_request_id());
            let topic_safe: String = c_res_topic.clone();
            let producer_safe: Arc<BrokerProducer> = {
              let producer_lock: std::sync::MutexGuard<'_, Arc<BrokerProducer>> =
                req_producer.lock().unwrap();
              Arc::clone(&*producer_lock)
            };

            stream_lock.on_send(Arc::new(move |bytes_safe: Vec<u8>| {
              (producer_safe)(&topic_safe, &bytes_safe);
            }));
          }

          req.reset();

          // Broker Action
          cb(ctx, bytes, stream);
        }
        Ok(None) => {}
        Err(e) => {
          let logger_lock: std::sync::MutexGuard<'_, Arc<RPCLogger>> = req_logger.lock().unwrap();
          logger_lock("warning", &format!("Block read error: {}", e));
        }
      }
    });

    let res_consumer: Arc<BrokerConsumer> = Arc::new(move |bytes: &[u8]| {
      let mut req: RPCReq = RPCReq::new();
      req.add(bytes);

      match req.read_block() {
        Ok(Some(_)) => {
          let ctx: Arc<Mutex<BrokerCtx>> = Arc::new(Mutex::new(req.get_ctx()));
          let bytes: Arc<Mutex<BrokerBytes>> = Arc::new(Mutex::new(req.get_bytes()));
          let request_id: u128 = req.get_request_id();
          req.reset();

          if let Some(map) = MAP.get() {
            let mut map_lock: std::sync::MutexGuard<
              '_,
              HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
            > = map.lock().unwrap();
            if let Some(tx) = map_lock.remove(&request_id) {
              let _ = tx.send((ctx, bytes));
            }
          }
        }
        Ok(None) => {}
        Err(e) => {
          let logger_lock: std::sync::MutexGuard<'_, Arc<RPCLogger>> = res_logger.lock().unwrap();
          logger_lock("warning", &format!("Block read error: {}", e));
        }
      }
    });

    let consumer_lock: std::sync::MutexGuard<'_, Arc<BrokerConsumerHandler>> =
      self.cb_consumer.lock().unwrap();
    (consumer_lock)(&p_req_topic, req_consumer);
    (consumer_lock)(&p_res_topic, res_consumer);
  }

  pub fn send(
    &self,
    topic: &str,
    payload: &JSON,
    bytes: &[u8],
    is_reliable: bool,
  ) -> (Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>) {
    let (tx, rx) = mpsc::channel();
    let request_id: u128 = generate_request_id();
    let req_topic: String = format!("req:{}", topic);

    {
      let map: &Mutex<
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = MAP.get_or_init(|| Mutex::new(HashMap::new()));
      let mut map_lock: std::sync::MutexGuard<
        '_,
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = map.lock().unwrap();
      map_lock.insert(request_id, tx);
    }

    let json: JSON = serde_json::json!({
        "topic": req_topic,
        "request_id": request_id.to_string(),
        "payload": payload,
        "reliable": is_reliable
    });

    let mut buff: Vec<u8> = Vec::new();
    let message: String = serde_json::to_string(&json).unwrap();
    let meta: String = format!("{}+{}:", message.len(), bytes.len());
    buff.extend_from_slice(meta.as_bytes());
    buff.extend_from_slice(message.as_bytes());
    buff.extend_from_slice(bytes);

    {
      let producer_lock: std::sync::MutexGuard<'_, Arc<BrokerProducer>> =
        self.cb_producer.lock().unwrap();
      (producer_lock)(&req_topic, &buff);
    }

    rx.recv().unwrap()
  }

  pub fn send_bytes(
    &self,
    topic: &str,
    bytes: &[u8],
    is_reliable: bool,
  ) -> (Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>) {
    let (tx, rx) = mpsc::channel();
    let request_id: u128 = generate_request_id();
    let req_topic: String = format!("req:{}", topic);

    {
      let map: &Mutex<
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = MAP.get_or_init(|| Mutex::new(HashMap::new()));
      let mut map_lock: std::sync::MutexGuard<
        '_,
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = map.lock().unwrap();
      map_lock.insert(request_id, tx);
    }

    let json: JSON = serde_json::json!({
        "topic": req_topic,
        "request_id": request_id.to_string(),
        "payload": "<bytes>",
        "reliable": is_reliable
    });

    let mut buff: Vec<u8> = Vec::new();
    let message: String = serde_json::to_string(&json).unwrap();
    let meta: String = format!("{}+{}:", message.len(), bytes.len());
    buff.extend_from_slice(meta.as_bytes());
    buff.extend_from_slice(message.as_bytes());
    buff.extend_from_slice(bytes);

    {
      let producer_lock: std::sync::MutexGuard<'_, Arc<BrokerProducer>> =
        self.cb_producer.lock().unwrap();
      (producer_lock)(&req_topic, &buff);
    }

    rx.recv().unwrap()
  }

  pub fn send_json(
    &self,
    topic: &str,
    payload: &JSON,
    is_reliable: bool,
  ) -> (Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>) {
    let (tx, rx) = mpsc::channel();
    let request_id: u128 = generate_request_id();
    let req_topic: String = format!("req:{}", topic);

    {
      let map: &Mutex<
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = MAP.get_or_init(|| Mutex::new(HashMap::new()));
      let mut map_lock: std::sync::MutexGuard<
        '_,
        HashMap<u128, mpsc::Sender<(Arc<Mutex<BrokerCtx>>, Arc<Mutex<BrokerBytes>>)>>,
      > = map.lock().unwrap();
      map_lock.insert(request_id, tx);
    }

    let json: JSON = serde_json::json!({
      "topic": req_topic,
      "request_id": request_id.to_string(),
      "payload": payload,
      "reliable": is_reliable,
    });

    let mut buff: Vec<u8> = Vec::new();
    let message: String = serde_json::to_string(&json).unwrap();
    let meta: String = format!("{}+0:", message.len());
    buff.extend_from_slice(meta.as_bytes());
    buff.extend_from_slice(message.as_bytes());

    {
      let producer_lock: std::sync::MutexGuard<'_, Arc<BrokerProducer>> =
        self.cb_producer.lock().unwrap();
      (producer_lock)(&req_topic, &buff);
    }

    rx.recv().unwrap()
  }

  pub fn set_consumer(&self, cb: Arc<BrokerConsumerHandler>) -> () {
    let mut consumer_lock: std::sync::MutexGuard<'_, Arc<BrokerConsumerHandler>> =
      self.cb_consumer.lock().unwrap();
    *consumer_lock = cb;
  }

  pub fn set_producer(&self, cb: Arc<BrokerProducer>) -> () {
    let mut producer_lock: std::sync::MutexGuard<'_, Arc<BrokerProducer>> =
      self.cb_producer.lock().unwrap();
    *producer_lock = cb;
  }
}
