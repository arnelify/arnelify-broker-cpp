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

#ifndef ARNELIFY_BROKER_RPC_H
#define ARNELIFY_BROKER_RPC_H

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef void (*rpc_logger_t)(const int c_id, const char* c_level,
                             const char* c_message);

typedef void (*rpc_action_t)(const int c_id, const int c_stream_id,
                             const char* c_topic, const char* c_json,
                             const char* c_bytes, const int c_bytes_len);

typedef void (*rpc_broker_consumer_t)(const int c_consumer_id,
                                      const char* c_bytes,
                                      const int c_bytes_len);

typedef void (*rpc_consumer_handler_t)(const int c_id, const char* c_topic,
                                       const int c_consumer_id,
                                       rpc_broker_consumer_t c_cb);

typedef void (*rpc_producer_t)(const int c_id, const char* c_topic,
                               const char* c_bytes, const int c_bytes_len);

int rpc_create();

void rpc_destroy(const int c_id);

void rpc_logger(const int c_id, rpc_logger_t c_cb);

void rpc_on(const int c_id, const char* c_topic, rpc_action_t c_cb);

void rpc_push(const int c_stream_id, const char* c_json, const char* c_bytes,
              const int c_bytes_len, const int c_is_reliable);

void rpc_push_bytes(const int c_stream_id, const char* c_bytes,
                    const int c_bytes_len, const int c_is_reliable);

void rpc_push_json(const int c_stream_id, const char* c_json,
                   const int c_is_reliable);

void rpc_set_compression(const int c_stream_id, const char* c_algorithm);

const char* rpc_send(const int c_id, const char* c_topic, const char* c_json,
                     const char* c_bytes, const int c_bytes_len,
                     const int c_is_reliable);

const char* rpc_send_bytes(const int c_id, const char* c_topic,
                           const char* c_bytes, const int c_bytes_len,
                           const int c_is_reliable);

const char* rpc_send_json(const int c_id, const char* c_topic,
                          const char* c_json, const int c_is_reliable);

void rpc_set_consumer(const int c_id, rpc_consumer_handler_t c_cb);
void rpc_set_producer(const int c_id, rpc_producer_t c_cb);

#ifdef __cplusplus
}
#endif

#endif