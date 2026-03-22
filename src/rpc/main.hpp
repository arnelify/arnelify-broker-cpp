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

#ifndef ARNELIFY_BROKER_RPC_HPP
#define ARNELIFY_BROKER_RPC_HPP

#include <iostream>
#include <functional>
#include <unordered_map>
#include <mutex>
#include <sstream>
#include <optional>

#include "json.h"
#include "main.h"

using BrokerCtx = Json::Value;
using BrokerBytes = std::vector<std::uint8_t>;
using BrokerConsumer = std::function<void(const BrokerBytes)>;

using BrokerConsumerHandler =
    std::function<void(const std::string& topic, const BrokerConsumer& cb)>;

using BrokerProducer = std::function<void(
    const std::string& topic, const std::uint8_t* bytes, std::size_t len)>;

std::tuple<BrokerCtx, BrokerBytes> deserialize(const char* response) {
  if (!response) throw std::runtime_error("Null data pointer");
  std::string data(response);
  auto pos1 = data.find('+');
  auto pos2 = data.find(':');

  int ctx_len = std::stoi(data.substr(0, pos1));
  int bytes_len = std::stoi(data.substr(pos1 + 1, pos2 - pos1 - 1));

  Json::Value ctx;
  Json::CharReaderBuilder reader;
  std::string errs;
  std::istringstream s(data.substr(pos2 + 1, ctx_len));
  if (!Json::parseFromStream(reader, s, &ctx, &errs)) {
    throw std::runtime_error("JSON parse error: " + errs);
  }

  std::vector<std::uint8_t> bytes;
  const char* bytes_start = data.data() + pos2 + 1 + ctx_len;
  bytes.insert(bytes.end(), bytes_start, bytes_start + bytes_len);
  return {ctx, bytes};
}

class RPCStream {
 private:
  const int id;
  const int rpc_id;

 public:
  RPCStream(const int stream_id, const int rpc_id)
      : id(stream_id), rpc_id(rpc_id) {}
  ~RPCStream() {}

  void push(Json::Value& json, const std::vector<std::uint8_t>& bytes,
            const bool is_reliable = true) {
    Json::StreamWriterBuilder writer;
    writer["indentation"] = "";
    writer["emitUTF8"] = true;

    const std::string c_json = Json::writeString(writer, json);
    const char* c_bytes = nullptr;
    int c_bytes_len = 0;

    if (!bytes.empty()) {
      c_bytes = reinterpret_cast<const char*>(bytes.data());
      c_bytes_len = static_cast<int>(bytes.size());
    }

    rpc_push(this->id, c_json.c_str(), c_bytes, c_bytes_len,
             is_reliable ? 1 : 0);
  }

  void push_bytes(const std::vector<std::uint8_t>& bytes,
                  const bool is_reliable = true) {
    const char* c_bytes = nullptr;
    int c_bytes_len = 0;

    if (!bytes.empty()) {
      c_bytes = reinterpret_cast<const char*>(bytes.data());
      c_bytes_len = static_cast<int>(bytes.size());
    }

    rpc_push_bytes(this->id, c_bytes, c_bytes_len, is_reliable ? 1 : 0);
  }

  void push_json(Json::Value& json, const bool is_reliable = true) {
    Json::StreamWriterBuilder writer;
    writer["indentation"] = "";
    writer["emitUTF8"] = true;

    const std::string c_json = Json::writeString(writer, json);
    rpc_push_json(this->id, c_json.c_str(), is_reliable ? 1 : 0);
  }

  const std::tuple<BrokerCtx, BrokerBytes> send(
      const std::string& topic, Json::Value& json,
      const std::vector<std::uint8_t>& bytes, const bool is_reliable = true) {
    Json::StreamWriterBuilder writer;
    writer["indentation"] = "";
    writer["emitUTF8"] = true;

    const std::string c_json = Json::writeString(writer, json);
    const char* c_bytes = nullptr;
    int c_bytes_len = 0;

    if (!bytes.empty()) {
      c_bytes = reinterpret_cast<const char*>(bytes.data());
      c_bytes_len = static_cast<int>(bytes.size());
    }

    const char* response = rpc_send(this->rpc_id, topic.c_str(), c_json.c_str(),
                                    c_bytes, c_bytes_len, is_reliable ? 1 : 0);
    return deserialize(response);
  }

  const std::tuple<BrokerCtx, BrokerBytes> send_bytes(
      const std::string& topic, const std::vector<std::uint8_t>& bytes,
      const bool is_reliable = true) {
    const char* c_bytes = nullptr;
    int c_bytes_len = 0;

    if (!bytes.empty()) {
      c_bytes = reinterpret_cast<const char*>(bytes.data());
      c_bytes_len = static_cast<int>(bytes.size());
    }

    const char* response = rpc_send_bytes(this->rpc_id, topic.c_str(), c_bytes,
                                          c_bytes_len, is_reliable ? 1 : 0);
    return deserialize(response);
  }

  const std::tuple<BrokerCtx, BrokerBytes> send_json(
      const std::string& topic, Json::Value& json,
      const bool is_reliable = true) {
    Json::StreamWriterBuilder writer;
    writer["indentation"] = "";
    writer["emitUTF8"] = true;

    const std::string c_json = Json::writeString(writer, json);
    const char* response = rpc_send_json(this->rpc_id, topic.c_str(),
                                         c_json.c_str(), is_reliable ? 1 : 0);
    return deserialize(response);
  }
};

using RPCLogger = std::function<void(const std::string&, const std::string&)>;
using RPCAction = std::function<void(BrokerCtx&, BrokerBytes&, RPCStream&)>;

static std::unordered_map<int, RPCLogger> ARNELIFY_BROKER_RPC_LOGGERS;
static std::mutex ARNELIFY_BROKER_RPC_LOGGERS_MTX;

static std::unordered_map<int, std::unordered_map<std::string, RPCAction>>
    ARNELIFY_BROKER_RPC_ACTIONS;
static std::mutex ARNELIFY_BROKER_RPC_ACTIONS_MTX;

static std::unordered_map<int, BrokerConsumerHandler>
    ARNELIFY_BROKER_RPC_CONSUMERS;
static std::mutex ARNELIFY_BROKER_RPC_CONSUMERS_MTX;

static std::unordered_map<int, BrokerProducer> ARNELIFY_BROKER_RPC_PRODUCERS;
static std::mutex ARNELIFY_BROKER_RPC_PRODUCERS_MTX;

static std::unordered_map<int, BrokerConsumer>
    ARNELIFY_BROKER_RPC_BROKER_CONSUMERS;
static std::mutex ARNELIFY_BROKER_RPC_BROKER_CONSUMERS_MTX;

class RPC {
 private:
  int id;

  static void action_adapter(const int id, const int stream_id,
                             const char* c_topic, const char* c_ctx,
                             const char* c_bytes, const int c_bytes_len) {
    std::unordered_map<std::string, RPCAction> actions;
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_ACTIONS_MTX);
      auto it = ARNELIFY_BROKER_RPC_ACTIONS.find(id);
      if (it == ARNELIFY_BROKER_RPC_ACTIONS.end()) return;
      actions = it->second;
    }

    Json::Value json;
    Json::CharReaderBuilder reader;
    std::string errs;
    std::stringstream ss(c_ctx);
    if (!parseFromStream(reader, ss, &json, &errs)) {
      std::cout
          << "[Arnelify Server]: C++ FFI error in rpc_action_adapter: Invalid "
             "JSON in 'c_ctx'."
          << std::endl;
      exit(1);
    }

    auto it = actions.find(c_topic);
    if (it == actions.end()) return;

    BrokerBytes bytes;
    if (c_bytes && c_bytes_len > 0) {
      bytes.assign(c_bytes, c_bytes + c_bytes_len);
    }

    RPCStream stream(stream_id, id);
    it->second(json, bytes, stream);
  }

  static void consumer_adapter(const int c_id, const char* c_topic,
                               const int c_consumer_id,
                               void (*broker_consumer)(const int, const char*,
                                                       const int)) {
    BrokerConsumer consumer = [broker_consumer,
                               c_consumer_id](const std::vector<std::uint8_t>& bytes) {
      const char* c_bytes = reinterpret_cast<const char*>(bytes.data());
      int c_bytes_len = static_cast<int>(bytes.size());
      broker_consumer(c_consumer_id, c_bytes, c_bytes_len);
    };

    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_BROKER_CONSUMERS_MTX);
      ARNELIFY_BROKER_RPC_BROKER_CONSUMERS[c_consumer_id] = consumer;
    }

    BrokerConsumerHandler handler;
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_CONSUMERS_MTX);
      auto it = ARNELIFY_BROKER_RPC_CONSUMERS.find(c_id);
      if (it == ARNELIFY_BROKER_RPC_CONSUMERS.end()) return;
      handler = it->second;
    }

    const std::string topic = c_topic;
    handler(topic, consumer);
  }

  static void logger_adapter(const int id, const char* c_level,
                             const char* c_message) {
    RPCLogger cb = [](const std::string&, const std::string& message) {
      std::cout << message << std::endl;
    };

    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_LOGGERS_MTX);
      auto it = ARNELIFY_BROKER_RPC_LOGGERS.find(id);
      if (it == ARNELIFY_BROKER_RPC_LOGGERS.end()) return;
      cb = it->second;
    }

    if (!c_level || !c_message) return;
    cb(std::string(c_level), std::string(c_message));
  }

  static void producer_adapter(const int c_id, const char* c_topic,
                               const char* c_bytes, const int c_bytes_len) {
    BrokerProducer producer;
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_PRODUCERS_MTX);
      auto it = ARNELIFY_BROKER_RPC_PRODUCERS.find(c_id);
      if (it == ARNELIFY_BROKER_RPC_PRODUCERS.end()) return;
      producer = it->second;
    }

    const std::uint8_t* bytes = reinterpret_cast<const std::uint8_t*>(c_bytes);
    std::size_t bytes_len = 0;
    if (c_bytes && c_bytes_len > 0) {
      bytes_len = static_cast<std::size_t>(c_bytes_len);
    }

    const std::string topic = c_topic;
    producer(topic, bytes, bytes_len);
  }

 public:
  explicit RPC() : id(0) {
    this->id = rpc_create();
  }

  ~RPC() {
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_LOGGERS_MTX);
      ARNELIFY_BROKER_RPC_LOGGERS.erase(this->id);
    }
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_ACTIONS_MTX);
      ARNELIFY_BROKER_RPC_ACTIONS.erase(this->id);
    }
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_CONSUMERS_MTX);
      ARNELIFY_BROKER_RPC_CONSUMERS.erase(this->id);
    }
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_PRODUCERS_MTX);
      ARNELIFY_BROKER_RPC_PRODUCERS.erase(this->id);
    }

    rpc_destroy(this->id);
  }

  void logger(const RPCLogger& cb) {
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_LOGGERS_MTX);
      ARNELIFY_BROKER_RPC_LOGGERS[id] = cb;
    }
    rpc_logger(id, &RPC::logger_adapter);
  }

  void on(const std::string& topic, const RPCAction& cb) {
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_ACTIONS_MTX);
      ARNELIFY_BROKER_RPC_ACTIONS[id][topic] = cb;
    }
    rpc_on(id, topic.c_str(), &RPC::action_adapter);
  }

  const std::tuple<BrokerCtx, BrokerBytes> send(
      const std::string& topic, Json::Value& json,
      const std::vector<std::uint8_t>& bytes, const bool is_reliable = true) {
    Json::StreamWriterBuilder writer;
    writer["indentation"] = "";
    writer["emitUTF8"] = true;

    const std::string c_json = Json::writeString(writer, json);
    const char* c_bytes = nullptr;
    int c_bytes_len = 0;

    if (!bytes.empty()) {
      c_bytes = reinterpret_cast<const char*>(bytes.data());
      c_bytes_len = static_cast<int>(bytes.size());
    }

    const char* response = rpc_send(this->id, topic.c_str(), c_json.c_str(),
                                    c_bytes, c_bytes_len, is_reliable ? 1 : 0);
    return deserialize(response);
  }

  const std::tuple<BrokerCtx, BrokerBytes> send_bytes(
      const std::string& topic, const std::vector<std::uint8_t>& bytes,
      const bool is_reliable = true) {
    const char* c_bytes = nullptr;
    int c_bytes_len = 0;

    if (!bytes.empty()) {
      c_bytes = reinterpret_cast<const char*>(bytes.data());
      c_bytes_len = static_cast<int>(bytes.size());
    }

    const char* response = rpc_send_bytes(this->id, topic.c_str(), c_bytes,
                                          c_bytes_len, is_reliable ? 1 : 0);
    return deserialize(response);
  }

  const std::tuple<BrokerCtx, BrokerBytes> send_json(
      const std::string& topic, Json::Value& json,
      const bool is_reliable = true) {
    Json::StreamWriterBuilder writer;
    writer["indentation"] = "";
    writer["emitUTF8"] = true;

    const std::string c_json = Json::writeString(writer, json);
    const char* response = rpc_send_json(this->id, topic.c_str(),
                                         c_json.c_str(), is_reliable ? 1 : 0);
    return deserialize(response);
  }

  void set_consumer(const BrokerConsumerHandler& cb) {
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_CONSUMERS_MTX);
      ARNELIFY_BROKER_RPC_CONSUMERS[id] = cb;
    }
    rpc_set_consumer(id, &RPC::consumer_adapter);
  }

  void set_producer(const BrokerProducer& cb) {
    {
      std::lock_guard lock(ARNELIFY_BROKER_RPC_PRODUCERS_MTX);
      ARNELIFY_BROKER_RPC_PRODUCERS[id] = cb;
    }
    rpc_set_producer(id, &RPC::producer_adapter);
  }
};

#endif