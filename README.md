# bare-socket

An HTTP server built on raw TCP sockets using only the Rust standard library. No HTTP libraries or async runtimes are used. The server operates directly on `std::net::TcpListener` and `std::io::BufReader`.

## Architecture

A `TcpListener` accepts connections on port 3001. Each connection is dispatched to a thread via `std::thread::spawn`. The thread reads the socket with a `BufReader`, parses the HTTP request line and headers manually, matches the path against a set of routes, and writes a response back. Thread handles are collected in a `Vec` and joined on shutdown.

This is a thread-per-connection model with no thread pool or async runtime.

Request parsing works by splitting the request line on whitespace into method, path, and version components. Headers are read line by line until a blank line is reached, then split on `": "` and stored as key-value pairs. Responses are constructed as formatted strings with explicit `Content-Length` values computed from the body byte length.

## Routes

| Path | Description |
|------|-------------|
| `GET /` | Returns 200 OK |
| `GET /echo/{message}` | Echoes the message as `text/plain` |
| `GET /user-agent` | Returns the `User-Agent` header value |

Status codes: 200, 403, 404.

## Running

```sh
cargo run
```

## Limitations

- HTTP/1.1 only, no persistent connections
- GET only, no request body parsing
- Thread per connection, no pooling
- No TLS

---

The project structure follows the CodeCrafters "Build Your Own HTTP Server" challenge. This was written as a first Rust project to learn about socket-level programming and manual HTTP parsing.
