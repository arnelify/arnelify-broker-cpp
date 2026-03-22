<img src="https://static.wikia.nocookie.net/arnelify/images/c/c8/Arnelify-logo-2024.png/revision/latest?cb=20240701012515" style="width:336px;" alt="Arnelify Logo" />

![Arnelify Broker for C++](https://img.shields.io/badge/Arnelify%20Broker%20for%20C++-0.9.6-yellow) ![C++](https://img.shields.io/badge/C++-2b-red) ![G++](https://img.shields.io/badge/G++-15.2.0-blue) ![C-Lang](https://img.shields.io/badge/CLang-19.1.7-blue)

## 🚀 About

**Arnelify® Broker for C & C++** — a multi-language broker with RPC and UMQT support.

All supported protocols:
| **#** | **Protocol** | **Transport** |
| - | - | - |
| 1 | TCP2 | UMQT |
| 2 | UDP | UMQT |

## 📋 Minimal Requirements
> Important: It's strongly recommended to use in a container that has been built from the gcc v15.2.0 image.
* CPU: Apple M1 / Intel Core i7 / AMD Ryzen 7
* OS: Debian 11 / MacOS 15 / Windows 10 with <a href="https://learn.microsoft.com/en-us/windows/wsl/install">WSL2</a>.
* RAM: 4 GB

## 📦 Installation
Run in terminal:
```bash
git clone git@github.com:arnelify/arnelify-broker-cpp.git
```
## 🎉 RPC
**RPC** - operates on top of the transport layer, enabling remote function and procedure calls.

### 📚 Examples

```cpp
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
```
## 🎉 UMQT
**UMQT (UDP Message Query Transport)** - is a universal WEB3-transport designed for flexible transport-layer communication, supporting two messaging mechanisms: TCP2 and datagrams.

### 📚 Configuration

| **Option** | **Description** |
| - | - |
| **BLOCK_SIZE_KB**| The size of the allocated memory used for processing large packets. |
| **CERT_PEM**| Path to the TLS cert-file in PEM format. |
| **COMPRESSION**| If this option is enabled, the transport will use BROTLI compression if the client application supports it. This setting increases CPU resource consumption. The transport will not use compression if the data size exceeds the value of **BLOCK_SIZE_KB**. |
| **KEY_PEM**| Path to the TLS private key-file in PEM format. |
| **PORT**| Defines which port the transport will listen on. |
| **THREAD_LIMIT**| Defines the maximum number of threads that will handle requests. |

### 📚 Examples

```cpp
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
```

## ⚖️ MIT License
This software is licensed under the <a href="https://github.com/arnelify/arnelify-broker-cpp/blob/main/LICENSE">MIT License</a>. The original author's name, logo, and the original name of the software must be included in all copies or substantial portions of the software.

## 🛠️ Contributing
Join us to help improve this software, fix bugs or implement new functionality. Active participation will help keep the software up-to-date, reliable, and aligned with the needs of its users.

VSCode Configs (optional):
```json
"includePath": [
  "/opt/homebrew/Cellar/jsoncpp/1.9.6/include/json",
  "${workspaceFolder}/src"
],
```
Run in terminal:
```bash
docker compose up -d --build
docker ps
docker exec -it <CONTAINER ID> bash
make build
```
For RPC:
```bash
make test_rpc
```
For UMQT:
```bash
make test_umqt
```

## ⭐ Release Notes
Version 0.9.6 — a multi-language broker with RPC and UMQT support.

We are excited to introduce the Arnelify Broker for NodeJS! Please note that this version is raw and still in active development.

Change log:

* UMQT support.
* Async Runtime & Multi-Threading.
* Block processing in "on-the-fly" mode.
* BROTLI compression (still in development).
* FFI, PYO3 and NEON support.
* Significant refactoring and optimizations.

Please use this version with caution, as it may contain bugs and unfinished features. We are actively working on improving and expanding the broker's capabilities, and we welcome your feedback and suggestions.

## 🔗 Links

* <a href="https://github.com/arnelify/arnelify-pod-cpp">Arnelify POD for C++</a>
* <a href="https://github.com/arnelify/arnelify-pod-node">Arnelify POD for NodeJS</a>
* <a href="https://github.com/arnelify/arnelify-pod-python">Arnelify POD for Python</a>
* <a href="https://github.com/arnelify/arnelify-pod-rust">Arnelify POD for Rust</a>
* <a href="https://github.com/arnelify/arnelify-react-native">Arnelify React Native</a>