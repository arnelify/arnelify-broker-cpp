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

#ifndef ARNELIFY_BROKER_UMQT_H
#define ARNELIFY_BROKER_UMQT_H

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef void (*umqt_handler_t)(const int c_id, const char* c_topic, const char* c_bytes,
                               const int c_bytes_len);

typedef void (*umqt_logger_t)(const int c_id, const char* c_level,
                              const char* c_message);

int umqt_end(int c_id);

void umqt_add_server(int c_id, const char* c_topic, const char* c_host,
                     int c_port);

int umqt_create(const char* c_opts);

void umqt_destroy(const int c_id);

void umqt_logger(const int c_id, umqt_logger_t c_cb);

void umqt_on(const int c_id, const char* c_topic, umqt_handler_t c_cb);

void umqt_send(const int c_id, const char* c_topic, const char* c_bytes,
               const int c_bytes_len, const int c_is_reliable);

void umqt_start(const int c_id);

void umqt_stop(const int c_id);

#ifdef __cplusplus
}
#endif

#endif