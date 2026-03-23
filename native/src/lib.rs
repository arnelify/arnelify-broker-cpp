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

//! <img src="https://static.wikia.nocookie.net/arnelify/images/c/c8/Arnelify-logo-2024.png/revision/latest?cb=20240701012515" style="width:336px;" alt="Arnelify Logo" />
//!
//! ![Arnelify Broker for Rust](https://img.shields.io/badge/Arnelify%20Broker%20for%20Rust-0.9.7-yellow)
//! ![Rust](https://img.shields.io/badge/Rust-1.91.1-orange)
//! ![Cargo](https://img.shields.io/badge/Cargo-1.91.1-blue)
//!
//! # About
//! **Arnelify® Broker for Rust** — a multi-language broker with RPC and UMQT support.
//!
//! All supported protocols:
//! | **#** | **Protocol** | **Transport** |
//! | - | - | - |
//! | 1 | TCP2 | UMQT |
//! | 2 | UDP | UMQT |
//!
//! ## Minimal Requirements
//! Important: It's strongly recommended to use in a container that has been built from the gcc v15.2.0 image.
//! * CPU: Apple M1 / Intel Core i7 / AMD Ryzen 7
//! * OS: Debian 11 / MacOS 15 / Windows 10 with <a href="https://learn.microsoft.com/en-us/windows/wsl/install">WSL2</a>.
//! * RAM: 4 GB
//!
//! ## Installation
//! Run in terminal:
//! ```bash
//! cargo add arnelify_broker
//! ```
//!
//! # RPC
//! **RPC** - operates on top of the transport layer, enabling remote function and procedure calls.
//!
//! ## Examples
//!
//! ```rust
//! use arnelify_broker::rpc::RPC;
//! use arnelify_broker::rpc::RPCLogger;
//! use arnelify_broker::rpc::RPCAction;
//! use arnelify_broker::rpc::BrokerBytes;
//! use arnelify_broker::rpc::BrokerCtx;
//! use arnelify_broker::rpc::RPCStream;
//! use std::sync::{Arc, Mutex};
//!
//! type JSON = serde_json::Value;
//!
//! fn main() -> () {
//!   let rpc: RPC = RPC::new();
//!   let rpc_logger: Arc<RPCLogger> = Arc::new(move |_level: &str, message: &str| -> () {
//!     println!("[Arnelify Broker]: {}", message);
//!   });
//!
//!   rpc.logger(rpc_logger);
//!   let rpc_action: Arc<RPCAction> = Arc::new(
//!     move |ctx: Arc<Mutex<BrokerCtx>>,
//!           bytes: Arc<Mutex<BrokerBytes>>,
//!           stream: Arc<Mutex<RPCStream>>|
//!           -> () {
//!       let json: JSON = ctx.lock().unwrap().clone();
//!       let bytes: Vec<u8> = bytes.lock().unwrap().clone();
//!
//!       let stream_lock: std::sync::MutexGuard<'_, RPCStream> = stream.lock().unwrap();
//!       stream_lock.push(&json, &bytes, true);
//!     },
//!   );
//!
//!   rpc.on("connect", rpc_action);
//!
//!   let json: JSON = serde_json::json!({"message": "Hello World"});
//!   let bytes: Vec<u8> = Vec::from(b"Hello World");
//!
//!   let (ctx, bytes) = rpc.send("connect", &json, &bytes, true);
//!
//!   let ctx: JSON = ctx.lock().unwrap().clone();
//!   let bytes: Vec<u8> = bytes.lock().unwrap().clone();
//!
//!   println!("ctx: {}", ctx);
//!   println!("bytes: {:?}", bytes);
//! }
//! ```
//!
//! # UMQT
//! **UMQT (UDP Message Query Transport)** - is a universal WEB3-transport designed for flexible transport-layer communication, supporting two messaging mechanisms: TCP2 and datagrams.
//!
//! ## Configuration
//!
//! | **Option** | **Description** |
//! | - | - |
//! | **BLOCK_SIZE_KB**| The size of the allocated memory used for processing large packets. |
//! | **CERT_PEM**| Path to the TLS cert-file in PEM format. |
//! | **COMPRESSION**| If this option is enabled, the transport will use BROTLI compression. This setting increases CPU resource consumption. The transport will not use compression if the data size exceeds the value of **BLOCK_SIZE_KB**. |
//! | **KEY_PEM**| Path to the TLS private key-file in PEM format. |
//! | **PORT**| Defines which port the server will listen on. |
//! | **THREAD_LIMIT**| Defines the maximum number of threads that will handle requests. |
//!
//! ## Examples
//!
//! ```rust
//! use arnelify_broker::transport::UMQT;
//! use arnelify_broker::transport::UMQTBytes;
//! use arnelify_broker::transport::UMQTConsumer;
//! use arnelify_broker::transport::UMQTLogger;
//! use arnelify_broker::transport::UMQTOpts;
//! use std::sync::{Arc, Mutex};
//!
//! fn main() {
//!   let umqt_opts: UMQTOpts = UMQTOpts {
//!     block_size_kb: 64,
//!     cert_pem: String::from("certs/cert.pem"),
//!     compression: true,
//!     key_pem: String::from("certs/key.pem"),
//!     port: 4433,
//!     thread_limit: 4,
//!   };
//!
//!   let umqt: Arc<UMQT> = Arc::new(UMQT::new(umqt_opts));
//!   let umqt_logger: Arc<UMQTLogger> = Arc::new(move |_level: &str, message: &str| -> () {
//!     println!("[Arnelify Broker]: {}", message);
//!   });
//!
//!   umqt.logger(umqt_logger);
//!   let umqt_consumer: Arc<UMQTConsumer> = Arc::new(move |bytes: Arc<Mutex<UMQTBytes>>| -> () {
//!     let bytes: Vec<u8> = bytes.lock().unwrap().clone();
//!     println!("bytes: {:?}", bytes);
//!   });
//!
//!   umqt.add_server("connect", "127.0.0.1", 4433);
//!   umqt.on("connect", umqt_consumer);
//!   umqt.start();
//! }
//! ```
//!
//! # MIT License
//!
//! This software is licensed under the <a href="https://github.com/arnelify/arnelify-broker-rust/blob/main/LICENSE">MIT License</a>. The original author's name, logo, and the original name of the software must be included in all copies or substantial portions of the software.
//!
//! # Contributing
//!
//! Join us to help improve this software, fix bugs or implement new functionality. Active participation will help keep the software up-to-date, reliable, and aligned with the needs of its users.
//!
//! Run in terminal:
//! ```bash
//! docker compose up -d --build
//! docker ps
//! docker exec -it <CONTAINER ID> bash
//! ```
//! For RPC:
//! ```bash
//! cargo run --bin test_rpc
//! ```
//! For UMQT:
//! ```bash
//! cargo run --bin test_umqt
//! ```
//!
//! # Release Notes
//!
//! Version 0.9.7 — a multi-language broker with RPC and UMQT support.
//!
//! We are excited to introduce the Arnelify Broker for Rust! Please note that this version is raw and still in active development.
//!
//! Change Log:
//!
//! * UMQT support.
//! * Async Runtime & Multi-Threading.
//! * Block processing in "on-the-fly" mode.
//! * BROTLI compression (still in development).
//! * FFI, PYO3 and NEON support.
//! * Significant refactoring and optimizations.
//!
//! Please use this version with caution, as it may contain bugs and unfinished features. We are actively working on improving and expanding the broker's capabilities, and we welcome your feedback and suggestions.
//!
//! # Links
//!
//! * <a href="https://github.com/arnelify/arnelify-pod-cpp">Arnelify POD for C++</a>
//! * <a href="https://github.com/arnelify/arnelify-pod-node">Arnelify POD for NodeJS</a>
//! * <a href="https://github.com/arnelify/arnelify-pod-python">Arnelify POD for Python</a>
//! * <a href="https://github.com/arnelify/arnelify-pod-rust">Arnelify POD for Rust</a>
//! * <a href="https://github.com/arnelify/arnelify-react-native">Arnelify React Native</a>

pub mod rpc;
pub mod transport;

pub use rpc::{
  BrokerBytes, BrokerConsumer, BrokerConsumerHandler, BrokerCtx, BrokerProducer, RPC, RPCAction,
  RPCLogger, RPCStream,
};

pub use transport::{UMQT, UMQTBytes, UMQTConsumer, UMQTLogger, UMQTOpts};

use std::{
  collections::HashMap,
  convert::TryFrom,
  ffi::{CStr, CString},
  os::raw::{c_char, c_int},
  slice,
  sync::{
    Arc, Mutex, MutexGuard, OnceLock,
    atomic::{AtomicI32, Ordering},
  },
};

use serde_json::Value as JSON;

type RPCStreams = HashMap<i32, Arc<Mutex<RPCStream>>>;
type Handler =
  extern "C" fn(c_id: c_int, c_topic: *const c_char, c_bytes: *const c_char, c_bytes_len: c_int);
type HandlerWithTransport = extern "C" fn(
  c_id: c_int,
  c_stream_id: c_int,
  c_topic: *const c_char,
  c_json: *const c_char,
  c_bytes: *const c_char,
  c_bytes_len: c_int,
);

type ProducerHandler =
  extern "C" fn(c_id: c_int, c_topic: *const c_char, c_bytes: *const c_char, c_bytes_len: c_int);
type Logger = extern "C" fn(c_id: c_int, c_level: *const c_char, c_message: *const c_char);
type ConsumerHandler = extern "C" fn(
  c_id: c_int,
  c_topic: *const c_char,
  c_cb_id: c_int,
  c_cb: extern "C" fn(c_id: c_int, c_bytes: *const c_char, c_bytes_len: c_int),
);

static RPC_MAP: OnceLock<Mutex<HashMap<c_int, Arc<RPC>>>> = OnceLock::new();
static RPC_ID: OnceLock<Mutex<c_int>> = OnceLock::new();
static RPC_CONSUMER_ID: AtomicI32 = AtomicI32::new(1);
static RPC_CONSUMERS: OnceLock<Mutex<HashMap<c_int, Arc<rpc::BrokerConsumer>>>> = OnceLock::new();
static RPC_STREAM_ID: AtomicI32 = AtomicI32::new(1);
static RPC_STREAMS: OnceLock<Mutex<RPCStreams>> = OnceLock::new();

static UMQT_MAP: OnceLock<Mutex<HashMap<c_int, Arc<UMQT>>>> = OnceLock::new();
static UMQT_ID: OnceLock<Mutex<c_int>> = OnceLock::new();

fn get_str(opts: &JSON, key: &str) -> String {
  opts
    .get(key)
    .and_then(JSON::as_str)
    .expect(&format!(
      "[Arnelify Broker]: Rust FFI error: '{}' missing or not a string.",
      key
    ))
    .to_string()
}

fn get_u64(opts: &JSON, key: &str) -> u64 {
  opts.get(key).and_then(JSON::as_u64).expect(&format!(
    "[Arnelify Broker]: Rust FFI error: '{}' missing or not a u64.",
    key
  ))
}

fn get_usize(opts: &JSON, key: &str) -> usize {
  let val: u64 = get_u64(opts, key);
  usize::try_from(val).expect(&format!(
    "[Arnelify Broker]: Rust FFI error: '{}' out of usize range.",
    key
  ))
}

fn get_u16(opts: &JSON, key: &str) -> u16 {
  let val: u64 = get_u64(opts, key);
  u16::try_from(val).expect(&format!(
    "[Arnelify Broker]: Rust FFI error: '{}' out of u16 range.",
    key
  ))
}

fn get_bool(opts: &JSON, key: &str) -> bool {
  opts.get(key).and_then(JSON::as_bool).expect(&format!(
    "[Arnelify Broker]: Rust FFI error: '{}' missing or not a bool.",
    key
  ))
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_create() -> c_int {
  let map: &Mutex<HashMap<c_int, Arc<RPC>>> = RPC_MAP.get_or_init(|| Mutex::new(HashMap::new()));
  let id: &Mutex<c_int> = RPC_ID.get_or_init(|| Mutex::new(0));
  let c_id: c_int = {
    let mut c: MutexGuard<'_, c_int> = id.lock().unwrap();
    *c += 1;
    *c
  };

  {
    let rpc: RPC = RPC::new();
    map.lock().unwrap().insert(c_id, Arc::new(rpc));
  }

  c_id
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_destroy(c_id: c_int) {
  if let Some(map) = RPC_MAP.get() {
    map.lock().unwrap().remove(&c_id);
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_logger(c_id: c_int, c_cb: Logger) {
  let rpc_logger: Arc<RPCLogger> = Arc::new(move |level: &str, message: &str| {
    let c_level: CString = CString::new(level).unwrap();
    let c_message: CString = CString::new(message).unwrap();
    c_cb(c_id, c_level.as_ptr(), c_message.as_ptr());
  });

  if let Some(map) = RPC_MAP.get() {
    let rpc: Option<Arc<RPC>> = {
      let rpc_lock: MutexGuard<'_, HashMap<i32, Arc<RPC>>> = map.lock().unwrap();
      rpc_lock.get(&c_id).cloned()
    };

    if let Some(rpc) = rpc {
      rpc.logger(rpc_logger);
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_on(c_id: c_int, c_topic: *const c_char, c_cb: HandlerWithTransport) {
  let topic: &str = match unsafe { CStr::from_ptr(c_topic) }.to_str() {
    Ok(s) => s,
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in rpc_on: Invalid UTF-8 in 'c_topic'.");
      return;
    }
  };

  let rpc_action: Arc<RPCAction> = Arc::new(
    move |ctx: Arc<Mutex<BrokerCtx>>,
          bytes: Arc<Mutex<BrokerBytes>>,
          stream: Arc<Mutex<RPCStream>>| {
      let c_stream_id: i32 = RPC_STREAM_ID.fetch_add(1, Ordering::Relaxed);

      RPC_STREAMS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .insert(c_stream_id, stream);

      let json: String = {
        let ctx_lock: MutexGuard<'_, BrokerCtx> = ctx.lock().unwrap();
        serde_json::to_string(&*ctx_lock).unwrap()
      };

      let bytes: Vec<u8> = {
        let bytes_lock: MutexGuard<'_, BrokerBytes> = bytes.lock().unwrap();
        bytes_lock.clone()
      };

      let c_topic: CString = CString::new(topic).unwrap();
      let c_json: CString = CString::new(json).unwrap();
      let c_bytes: *const c_char = bytes.as_ptr() as *const c_char;

      c_cb(
        c_id,
        c_stream_id,
        c_topic.as_ptr(),
        c_json.as_ptr(),
        c_bytes,
        bytes.len() as c_int,
      );

      if let Some(map) = RPC_STREAMS.get() {
        map.lock().unwrap().remove(&c_stream_id);
      }
    },
  );

  if let Some(map) = RPC_MAP.get() {
    let rpc: Option<Arc<RPC>> = {
      let rpc_lock: MutexGuard<'_, HashMap<i32, Arc<RPC>>> = map.lock().unwrap();
      rpc_lock.get(&c_id).cloned()
    };

    if let Some(rpc) = rpc {
      rpc.on(topic, rpc_action);
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_push(
  c_stream_id: c_int,
  c_json: *const c_char,
  c_bytes: *const c_char,
  c_bytes_len: c_int,
  c_is_reliable: c_int,
) {
  let json: JSON = match unsafe { CStr::from_ptr(c_json) }.to_str() {
    Ok(s) => match serde_json::from_str(s) {
      Ok(json) => json,
      Err(_) => {
        println!("[Arnelify Broker]: Rust FFI error in rpc_push: Invalid JSON in 'c_json'.");
        return;
      }
    },
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in rpc_push: Invalid UTF-8 in 'c_json'.");
      return;
    }
  };

  let bytes: &[u8] = if c_bytes.is_null() || c_bytes_len <= 0 {
    &[]
  } else {
    unsafe { std::slice::from_raw_parts(c_bytes as *const u8, c_bytes_len as usize) }
  };

  let is_reliable: bool = c_is_reliable == 1;
  if let Some(map) = RPC_STREAMS.get() {
    let stream: Option<Arc<Mutex<RPCStream>>> = {
      let streams: MutexGuard<'_, HashMap<i32, Arc<Mutex<RPCStream>>>> = map.lock().unwrap();
      streams.get(&c_stream_id).cloned()
    };

    match stream {
      Some(stream) => {
        let stream_lock: std::sync::MutexGuard<'_, RPCStream> = stream.lock().unwrap();
        stream_lock.push(&json, &bytes, is_reliable);
      }
      None => {
        println!("[Arnelify Broker]: Rust FFI error in rpc_push: No stream found.");
      }
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_push_bytes(
  c_stream_id: c_int,
  c_bytes: *const c_char,
  c_bytes_len: c_int,
  c_is_reliable: c_int,
) {
  let bytes: &[u8] = if c_bytes.is_null() || c_bytes_len <= 0 {
    &[]
  } else {
    unsafe { std::slice::from_raw_parts(c_bytes as *const u8, c_bytes_len as usize) }
  };

  let is_reliable: bool = c_is_reliable == 1;
  if let Some(map) = RPC_STREAMS.get() {
    let stream: Option<Arc<Mutex<RPCStream>>> = {
      let streams: MutexGuard<'_, HashMap<i32, Arc<Mutex<RPCStream>>>> = map.lock().unwrap();
      streams.get(&c_stream_id).cloned()
    };

    match stream {
      Some(stream) => {
        let stream_lock: std::sync::MutexGuard<'_, RPCStream> = stream.lock().unwrap();
        stream_lock.push_bytes(&bytes, is_reliable);
      }
      None => {
        println!("[Arnelify Broker]: Rust FFI error in rpc_push_bytes: No stream found.");
      }
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_push_json(c_stream_id: c_int, c_json: *const c_char, c_is_reliable: c_int) {
  let json: JSON = match unsafe { CStr::from_ptr(c_json) }.to_str() {
    Ok(s) => match serde_json::from_str(s) {
      Ok(json) => json,
      Err(_) => {
        println!("[Arnelify Broker]: Rust FFI error in rpc_push_json: Invalid JSON in 'c_json'.");
        return;
      }
    },
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in rpc_push_json: Invalid UTF-8 in 'c_json'.");
      return;
    }
  };

  let is_reliable: bool = c_is_reliable == 1;
  if let Some(map) = RPC_STREAMS.get() {
    let stream: Option<Arc<Mutex<RPCStream>>> = {
      let streams: MutexGuard<'_, HashMap<i32, Arc<Mutex<RPCStream>>>> = map.lock().unwrap();
      streams.get(&c_stream_id).cloned()
    };

    match stream {
      Some(stream) => {
        let stream_lock: std::sync::MutexGuard<'_, RPCStream> = stream.lock().unwrap();
        stream_lock.push_json(&json, is_reliable);
      }
      None => {
        println!("[Arnelify Broker]: Rust FFI error in rpc_push_json: No stream found.");
      }
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_send(
  c_id: c_int,
  c_topic: *const c_char,
  c_json: *const c_char,
  c_bytes: *const c_char,
  c_bytes_len: c_int,
  c_is_reliable: c_int,
) -> *const c_char {
  let topic: &str = match unsafe { CStr::from_ptr(c_topic) }.to_str() {
    Ok(s) => s,
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in rpc_send: Invalid UTF-8 in 'c_topic'.");
      return std::ptr::null();
    }
  };

  let json: JSON = match unsafe { CStr::from_ptr(c_json) }.to_str() {
    Ok(s) => match serde_json::from_str(s) {
      Ok(json) => json,
      Err(_) => {
        println!("[Arnelify Broker]: Rust FFI error in rpc_send: Invalid JSON in 'c_json'.");
        return std::ptr::null();
      }
    },
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in rpc_send: Invalid UTF-8 in 'c_json'.");
      return std::ptr::null();
    }
  };

  let bytes: &[u8] = if c_bytes.is_null() || c_bytes_len <= 0 {
    &[]
  } else {
    unsafe { std::slice::from_raw_parts(c_bytes as *const u8, c_bytes_len as usize) }
  };

  let is_reliable: bool = c_is_reliable == 1;
  if let Some(map) = RPC_MAP.get() {
    let rpc: Option<Arc<RPC>> = {
      let rpc_lock: MutexGuard<'_, HashMap<i32, Arc<RPC>>> = map.lock().unwrap();
      rpc_lock.get(&c_id).cloned()
    };

    if let Some(rpc) = rpc {
      let (ctx, bytes) = rpc.send(&topic, &json, &bytes, is_reliable);
      let mut buff: Vec<u8> = Vec::new();

      let ctx: BrokerCtx = ctx.lock().unwrap().clone();
      let ctx_vec: Vec<u8> = serde_json::to_vec(&ctx).unwrap_or_default();
      let bytes: Vec<u8> = bytes.lock().unwrap().clone();
      let meta: String = format!("{}+{}:", ctx_vec.len(), bytes.len());

      buff.extend_from_slice(meta.as_bytes());
      buff.extend_from_slice(&ctx_vec);
      buff.extend_from_slice(&bytes);

      let c_result: CString = match CString::new(buff) {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
      };

      return c_result.into_raw() as *const c_char;
    }
  }

  return std::ptr::null();
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_send_bytes(
  c_id: c_int,
  c_topic: *const c_char,
  c_bytes: *const c_char,
  c_bytes_len: c_int,
  c_is_reliable: c_int,
) -> *const c_char {
  let topic: &str = match unsafe { CStr::from_ptr(c_topic) }.to_str() {
    Ok(s) => s,
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in rpc_send_bytes: Invalid UTF-8 in 'c_topic'.");
      return std::ptr::null();
    }
  };

  let bytes: &[u8] = if c_bytes.is_null() || c_bytes_len <= 0 {
    &[]
  } else {
    unsafe { std::slice::from_raw_parts(c_bytes as *const u8, c_bytes_len as usize) }
  };

  let is_reliable: bool = c_is_reliable == 1;
  if let Some(map) = RPC_MAP.get() {
    let rpc: Option<Arc<RPC>> = {
      let rpc_lock: MutexGuard<'_, HashMap<i32, Arc<RPC>>> = map.lock().unwrap();
      rpc_lock.get(&c_id).cloned()
    };

    if let Some(rpc) = rpc {
      let (ctx, bytes) = rpc.send_bytes(&topic, &bytes, is_reliable);
      let mut buff: Vec<u8> = Vec::new();

      let ctx: BrokerCtx = ctx.lock().unwrap().clone();
      let ctx_vec: Vec<u8> = serde_json::to_vec(&ctx).unwrap_or_default();
      let bytes: Vec<u8> = bytes.lock().unwrap().clone();
      let meta: String = format!("{}+{}:", ctx_vec.len(), bytes.len());

      buff.extend_from_slice(meta.as_bytes());
      buff.extend_from_slice(&ctx_vec);
      buff.extend_from_slice(&bytes);

      let c_result: CString = match CString::new(buff) {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
      };

      return c_result.into_raw() as *const c_char;
    }
  }

  return std::ptr::null();
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_send_json(
  c_id: c_int,
  c_topic: *const c_char,
  c_json: *const c_char,
  c_is_reliable: c_int,
) -> *const c_char {
  let topic: &str = match unsafe { CStr::from_ptr(c_topic) }.to_str() {
    Ok(s) => s,
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in rpc_send_bytes: Invalid UTF-8 in 'c_topic'.");
      return std::ptr::null();
    }
  };

  let json: JSON = match unsafe { CStr::from_ptr(c_json) }.to_str() {
    Ok(s) => match serde_json::from_str(s) {
      Ok(json) => json,
      Err(_) => {
        println!("[Arnelify Broker]: Rust FFI error in rpc_send: Invalid JSON in 'c_json'.");
        return std::ptr::null();
      }
    },
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in rpc_send: Invalid UTF-8 in 'c_json'.");
      return std::ptr::null();
    }
  };

  let is_reliable: bool = c_is_reliable == 1;
  if let Some(map) = RPC_MAP.get() {
    let rpc: Option<Arc<RPC>> = {
      let rpc_lock: MutexGuard<'_, HashMap<i32, Arc<RPC>>> = map.lock().unwrap();
      rpc_lock.get(&c_id).cloned()
    };

    if let Some(rpc) = rpc {
      let (ctx, bytes) = rpc.send_json(&topic, &json, is_reliable);
      let mut buff: Vec<u8> = Vec::new();

      let ctx: BrokerCtx = ctx.lock().unwrap().clone();
      let ctx_vec: Vec<u8> = serde_json::to_vec(&ctx).unwrap_or_default();
      let bytes: Vec<u8> = bytes.lock().unwrap().clone();
      let meta: String = format!("{}+{}:", ctx_vec.len(), bytes.len());

      buff.extend_from_slice(meta.as_bytes());
      buff.extend_from_slice(&ctx_vec);
      buff.extend_from_slice(&bytes);

      let c_result: CString = match CString::new(buff) {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
      };

      return c_result.into_raw() as *const c_char;
    }
  }

  return std::ptr::null();
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_set_consumer(c_id: c_int, c_cb: ConsumerHandler) {
  let consumer_handler: Arc<BrokerConsumerHandler> =
    Arc::new(move |topic: &str, cb: Arc<BrokerConsumer>| {
      let c_consumer_id: c_int = RPC_CONSUMER_ID.fetch_add(1, Ordering::Relaxed);

      RPC_CONSUMERS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .insert(c_consumer_id, cb);

      extern "C" fn broker_consumer(
        c_consumer_id: c_int,
        c_bytes: *const c_char,
        c_bytes_len: c_int,
      ) {
        let bytes: &[u8] = if c_bytes.is_null() || 0 >= c_bytes_len {
          &[]
        } else {
          unsafe { slice::from_raw_parts(c_bytes as *const u8, c_bytes_len as usize) }
        };

        if let Some(map) = RPC_CONSUMERS.get() {
          let consumer: Option<Arc<BrokerConsumer>> = {
            let consumers: MutexGuard<'_, HashMap<i32, Arc<BrokerConsumer>>> = map.lock().unwrap();
            consumers.get(&c_consumer_id).cloned()
          };

          if let Some(consumer) = consumer {
            consumer(bytes);
          }
        }
      }

      let c_topic: CString = CString::new(topic).unwrap();
      c_cb(c_id, c_topic.as_ptr(), c_consumer_id, broker_consumer);
    });

  if let Some(map) = RPC_MAP.get() {
    let rpc: Option<Arc<RPC>> = {
      let rpc_lock: MutexGuard<'_, HashMap<i32, Arc<RPC>>> = map.lock().unwrap();
      rpc_lock.get(&c_id).cloned()
    };

    if let Some(rpc) = rpc {
      rpc.set_consumer(consumer_handler);
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn rpc_set_producer(c_id: c_int, c_cb: ProducerHandler) {
  let broker_producer: Arc<BrokerProducer> = Arc::new(move |topic: &str, bytes: &[u8]| {
    let c_topic: CString = CString::new(topic).unwrap();
    let c_bytes: *const c_char = bytes.as_ptr() as *const c_char;
    let c_bytes_len: i32 = bytes.len() as c_int;
    c_cb(c_id, c_topic.as_ptr(), c_bytes, c_bytes_len);
  });

  if let Some(map) = RPC_MAP.get() {
    let rpc: Option<Arc<RPC>> = {
      let rpc_lock: MutexGuard<'_, HashMap<i32, Arc<RPC>>> = map.lock().unwrap();
      rpc_lock.get(&c_id).cloned()
    };

    if let Some(rpc) = rpc {
      rpc.set_producer(broker_producer);
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn umqt_add_server(
  c_id: c_int,
  c_topic: *const c_char,
  c_host: *const c_char,
  c_port: c_int,
) {
  let topic: &str = match unsafe { CStr::from_ptr(c_topic) }.to_str() {
    Ok(s) => s,
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in umqt_add_server: Invalid UTF-8 in 'c_topic'.");
      return;
    }
  };

  let host: &str = match unsafe { CStr::from_ptr(c_host) }.to_str() {
    Ok(s) => s,
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in umqt_add_server: Invalid UTF-8 in 'c_topic'.");
      return;
    }
  };

  if let Some(map) = UMQT_MAP.get() {
    let umqt: Option<Arc<UMQT>> = {
      let umqt_lock: MutexGuard<'_, HashMap<i32, Arc<UMQT>>> = map.lock().unwrap();
      umqt_lock.get(&c_id).cloned()
    };

    if let Some(umqt) = umqt {
      umqt.add_server(topic, host, c_port as u16);
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn umqt_create(c_opts: *const c_char) -> c_int {
  let opts: JSON = match unsafe { CStr::from_ptr(c_opts) }.to_str() {
    Ok(s) => match serde_json::from_str(s) {
      Ok(json) => json,
      Err(_) => {
        println!("[Arnelify Broker]: Rust FFI error in umqt_create: Invalid JSON in 'c_opts'.");
        return 0;
      }
    },
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in umqt_create: Invalid UTF-8 in 'c_opts'.");
      return 0;
    }
  };

  let map: &Mutex<HashMap<c_int, Arc<UMQT>>> = UMQT_MAP.get_or_init(|| Mutex::new(HashMap::new()));
  let id: &Mutex<c_int> = UMQT_ID.get_or_init(|| Mutex::new(0));
  let c_id: c_int = {
    let mut c: MutexGuard<'_, c_int> = id.lock().unwrap();
    *c += 1;
    *c
  };

  let umqt_opts: UMQTOpts = UMQTOpts {
    block_size_kb: get_usize(&opts, "block_size_kb"),
    cert_pem: get_str(&opts, "cert_pem"),
    compression: get_bool(&opts, "compression"),
    key_pem: get_str(&opts, "key_pem"),
    port: get_u16(&opts, "port"),
    thread_limit: get_u64(&opts, "thread_limit"),
  };

  {
    let umqt: UMQT = UMQT::new(umqt_opts);
    map.lock().unwrap().insert(c_id, Arc::new(umqt));
  }

  c_id
}

#[unsafe(no_mangle)]
pub extern "C" fn umqt_destroy(c_id: c_int) {
  if let Some(map) = UMQT_MAP.get() {
    map.lock().unwrap().remove(&c_id);
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn umqt_logger(c_id: c_int, c_cb: Logger) {
  let umqt_logger: Arc<UMQTLogger> = Arc::new(move |level: &str, message: &str| {
    let c_level: CString = CString::new(level).unwrap();
    let c_message: CString = CString::new(message).unwrap();
    c_cb(c_id, c_level.as_ptr(), c_message.as_ptr());
  });

  if let Some(map) = UMQT_MAP.get() {
    let umqt: Option<Arc<UMQT>> = {
      let umqt_lock: MutexGuard<'_, HashMap<i32, Arc<UMQT>>> = map.lock().unwrap();
      umqt_lock.get(&c_id).cloned()
    };

    if let Some(umqt) = umqt {
      umqt.logger(umqt_logger);
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn umqt_on(c_id: c_int, c_topic: *const c_char, c_cb: Handler) {
  let topic: &str = match unsafe { CStr::from_ptr(c_topic) }.to_str() {
    Ok(s) => s,
    Err(_) => {
      println!("[Arnelify Broker]: Rust FFI error in umqt_on: Invalid UTF-8 in 'c_topic'.");
      return;
    }
  };

  let umqt_consumer: Arc<UMQTConsumer> = Arc::new(move |bytes: Arc<Mutex<BrokerBytes>>| {
    let bytes: BrokerBytes = {
      let bytes_lock: MutexGuard<'_, BrokerBytes> = bytes.lock().unwrap();
      bytes_lock.clone()
    };

    let c_topic: *const c_char = topic.as_ptr() as *const c_char;
    let c_bytes: *const c_char = bytes.as_ptr() as *const c_char;
    c_cb(c_id, c_topic, c_bytes, bytes.len() as c_int);
  });

  if let Some(map) = UMQT_MAP.get() {
    let umqt: Option<Arc<UMQT>> = {
      let umqt_lock: MutexGuard<'_, HashMap<i32, Arc<UMQT>>> = map.lock().unwrap();
      umqt_lock.get(&c_id).cloned()
    };

    if let Some(umqt) = umqt {
      umqt.on(topic, umqt_consumer);
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn umqt_send(
  c_id: c_int,
  c_topic: *const c_char,
  c_bytes: *const c_char,
  c_bytes_len: c_int,
  c_is_reliable: c_int,
) {
  if let Some(map) = UMQT_MAP.get() {
    let umqt: Option<Arc<UMQT>> = {
      let umqt_lock: MutexGuard<'_, HashMap<i32, Arc<UMQT>>> = map.lock().unwrap();
      umqt_lock.get(&c_id).cloned()
    };

    if let Some(umqt) = umqt {
      let is_reliable: bool = c_is_reliable == 1;
      let topic: &str = match unsafe { CStr::from_ptr(c_topic) }.to_str() {
        Ok(s) => s,
        Err(_) => {
          println!("[Arnelify Broker]: Rust FFI error in umqt_send: Invalid UTF-8 in 'c_topic'.");
          return;
        }
      };

      let bytes: &[u8] = if c_bytes.is_null() || c_bytes_len <= 0 {
        &[]
      } else {
        unsafe { slice::from_raw_parts(c_bytes as *const u8, c_bytes_len as usize) }
      };

      umqt.send(topic, &bytes, is_reliable);
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn umqt_start(c_id: c_int) {
  if let Some(map) = UMQT_MAP.get() {
    let umqt: Option<Arc<UMQT>> = {
      let umqt_lock: MutexGuard<'_, HashMap<i32, Arc<UMQT>>> = map.lock().unwrap();
      umqt_lock.get(&c_id).cloned()
    };

    if let Some(umqt) = umqt {
      umqt.start();
    }
  }
}

#[unsafe(no_mangle)]
pub extern "C" fn umqt_stop(c_id: c_int) {
  if let Some(map) = UMQT_MAP.get() {
    let umqt: Option<Arc<UMQT>> = {
      let umqt_lock: MutexGuard<'_, HashMap<i32, Arc<UMQT>>> = map.lock().unwrap();
      umqt_lock.get(&c_id).cloned()
    };

    if let Some(umqt) = umqt {
      umqt.stop();
    }
  }
}
