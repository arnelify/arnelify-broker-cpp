#include <iostream>
#include <thread>

#include "json.h"
#include "lib.hpp"

int main() {
  UMQTOpts umqt_opts(
      /* block_size_kb */ 64,
      /* cert_pem */ "certs/cert.pem",
      /* compression */ true,
      /* key_pem */ "certs/key.pem",
      /* port */ 4433,
      /* thread_limit */ 4);

  UMQT umqt(umqt_opts);
  UMQTLogger umqt_logger = [](const std::string& level,
                              const std::string& message) -> void {
    std::cout << "[Arnelify Broker]: " << message << std::endl;
  };

  umqt.logger(umqt_logger);
  UMQTConsumer umqt_consumer = [](UMQTBytes& bytes) -> void {
    std::string s(bytes.begin(), bytes.end());
    std::cout << "received: " << s << std::endl;
  };

  umqt.add_server("connect", "127.0.0.1", 4433);
  umqt.on("connect", umqt_consumer);
  umqt.start();
}