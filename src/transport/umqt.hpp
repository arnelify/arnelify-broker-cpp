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

#ifndef ARNELIFY_BROKER_UMQT_HPP
#define ARNELIFY_BROKER_UMQT_HPP

#include <iostream>
#include <functional>
#include <unordered_map>
#include <mutex>
#include <sstream>
#include <optional>

#include "json.h"
#include "umqt.h"

using UMQTBytes = std::vector<std::uint8_t>;
using JSON = Json::Value;

struct UMQTOpts {
  const std::size_t block_size_kb;
  const std::string cert_pem;
  const bool compression;
  const std::string key_pem;
  const std::uint16_t port;
  const std::uint64_t thread_limit;

  UMQTOpts(const std::size_t block_size_kb, const std::string& cert_pem,
           const bool compression, const std::string& key_pem,
           const std::uint16_t port, const std::uint64_t thread_limit)
      : block_size_kb(block_size_kb),
        cert_pem(cert_pem),
        compression(compression),
        key_pem(key_pem),
        port(port),
        thread_limit(thread_limit) {}
};

using UMQTLogger = std::function<void(const std::string&, const std::string&)>;
using UMQTConsumer = std::function<void(UMQTBytes&)>;

static std::unordered_map<int, UMQTLogger> ARNELIFY_BROKER_UMQT_LOGGERS;
static std::mutex ARNELIFY_BROKER_UMQT_LOGGERS_MTX;

static std::unordered_map<int, std::unordered_map<std::string, UMQTConsumer>>
    ARNELIFY_BROKER_UMQT_CONSUMERS;
static std::mutex ARNELIFY_BROKER_UMQT_CONSUMERS_MTX;

class UMQT {
 private:
  int id;
  UMQTOpts opts;

  static void logger_adapter(const int id, const char* c_level,
                             const char* c_message) {
    UMQTLogger cb = [](const std::string&, const std::string& message) {
      std::cout << message << std::endl;
    };

    {
      std::lock_guard lock(ARNELIFY_BROKER_UMQT_LOGGERS_MTX);
      auto it = ARNELIFY_BROKER_UMQT_LOGGERS.find(id);
      if (it == ARNELIFY_BROKER_UMQT_LOGGERS.end()) return;
      cb = it->second;
    }

    if (!c_level || !c_message) return;
    cb(std::string(c_level), std::string(c_message));
  }

  static void consumer_adapter(const int id, const char* c_topic,
                               const char* c_bytes, const int c_bytes_len) {
    std::unordered_map<std::string, UMQTConsumer> consumers;
    {
      std::lock_guard lock(ARNELIFY_BROKER_UMQT_CONSUMERS_MTX);
      auto it = ARNELIFY_BROKER_UMQT_CONSUMERS.find(id);
      if (it == ARNELIFY_BROKER_UMQT_CONSUMERS.end()) return;
      consumers = it->second;
    }

    auto it = consumers.find(c_topic);
    if (it == consumers.end()) return;

    UMQTBytes bytes;
    if (c_bytes && c_bytes_len > 0) {
      bytes.assign(c_bytes, c_bytes + c_bytes_len);
    }

    it->second(bytes);
  }

 public:
  explicit UMQT(const UMQTOpts& o) : id(0), opts(o) {
    Json::Value opts = Json::objectValue;

    opts["block_size_kb"] = static_cast<Json::UInt64>(this->opts.block_size_kb);
    opts["cert_pem"] = this->opts.cert_pem;
    opts["compression"] = this->opts.compression;
    opts["key_pem"] = this->opts.key_pem;
    opts["port"] = static_cast<Json::UInt>(this->opts.port);
    opts["thread_limit"] = static_cast<Json::UInt64>(this->opts.thread_limit);

    Json::StreamWriterBuilder writer;
    writer["indentation"] = "";
    writer["emitUTF8"] = true;

    this->id = umqt_create(Json::writeString(writer, opts).c_str());
  }

  ~UMQT() {
    {
      std::lock_guard lock(ARNELIFY_BROKER_UMQT_LOGGERS_MTX);
      ARNELIFY_BROKER_UMQT_LOGGERS.erase(id);
    }

    {
      std::lock_guard lock(ARNELIFY_BROKER_UMQT_CONSUMERS_MTX);
      ARNELIFY_BROKER_UMQT_CONSUMERS.erase(id);
    }

    umqt_destroy(id);
  }

  void add_server(const std::string& topic, const std::string& host,
                  const int port) {
    umqt_add_server(this->id, topic.c_str(), host.c_str(), port);
  }

  void logger(const UMQTLogger& cb) {
    {
      std::lock_guard lock(ARNELIFY_BROKER_UMQT_LOGGERS_MTX);
      ARNELIFY_BROKER_UMQT_LOGGERS[id] = cb;
    }
    umqt_logger(id, &UMQT::logger_adapter);
  }

  void on(const std::string& topic, const UMQTConsumer& cb) {
    {
      std::lock_guard lock(ARNELIFY_BROKER_UMQT_CONSUMERS_MTX);
      ARNELIFY_BROKER_UMQT_CONSUMERS[id][topic] = cb;
    }

    umqt_on(id, topic.c_str(), &UMQT::consumer_adapter);
  }

  void send(const std::string& topic, const UMQTBytes& bytes,
            const bool is_reliable) {
    const char* c_bytes = nullptr;
    int c_bytes_len = 0;

    if (!bytes.empty()) {
      c_bytes = reinterpret_cast<const char*>(bytes.data());
      c_bytes_len = static_cast<int>(bytes.size());
    }

    umqt_send(this->id, topic.c_str(), c_bytes, c_bytes_len, is_reliable);
  }

  void start() { umqt_start(id); }
  void stop() { umqt_stop(id); }
};

#endif