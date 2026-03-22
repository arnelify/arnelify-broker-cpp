#include <iostream>

#include "json.h"
#include "lib.hpp"

int main() {
  RPC rpc;
  RPCLogger rpc_logger = [](const std::string& level,
                            const std::string& message) -> void {
    std::cout << "[Arnelify Server]: " << message << std::endl;
  };

  rpc.logger(rpc_logger);
  RPCAction rpc_action = [](BrokerCtx& ctx, BrokerBytes& bytes,
                            RPCStream& stream) -> void {
    stream.push(ctx, bytes);
  };

  rpc.on("connect", rpc_action);

  const std::string message = "Hello World";
  const std::vector<std::uint8_t> buff(message.begin(), message.end());
  Json::Value json = Json::objectValue;
  json["message"] = message;

  auto [ctx, bytes] = rpc.send("connect", json, buff, true);

  Json::StreamWriterBuilder writer;
  writer["indentation"] = "";
  writer["emitUTF8"] = true;

  std::cout << "ctx: " << Json::writeString(writer, ctx) << std::endl;

  std::cout << "bytes: [ ";
  for (auto b : bytes) std::cout << (int)b << " ";
  std::cout << "]" << std::endl;
}