use anyhow::Result;
use log::error;
use std::{
    fmt::format,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

//new ones
const HTTP_200_OK: &str = "HTTP/1.1 200 OK";
const HTTP_404_NOT_FOUND: &str = "HTTP/1.1 404 Not Found";
const HTTP_FORBIDDEN: &str = "HTTP/1.1 403 Invalid Request";

const CONTENT_TYPE_TEXT: &str = "Content-Type: text/plain";

const ERROR_MESSAGE_ECHO: &str = "Message is Required. Example Usage: /echo/{message}";
const ERROR_USER_AGENT_MISSING: &str = "User-Agent header not found";

fn build_success_response(body: &str) -> String {
    format!(
        "{}\r\n{}\r\nContent-Length: {}\r\n\r\n{}",
        HTTP_200_OK,
        CONTENT_TYPE_TEXT,
        body.len(),
        body
    )
}

fn build_error_response(status: &str, body: &str) -> String {
    format!(
        "{}\r\n{}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        CONTENT_TYPE_TEXT,
        body.len(),
        body
    )
}
fn find_header<'a>(headers: &'a [(String, String)], header: &str) -> Option<&'a str> {
    for (key, value) in headers {
        if key == header {
            return Some(value.as_str());
        }
    }
    None
}

fn handle_user_agent(headers: &[(String, String)]) -> String {
    let user_agent = find_header(&headers, "User-Agent");

    match user_agent {
        None => {
            error!("User Agent header Missing");
            build_error_response(HTTP_FORBIDDEN, ERROR_USER_AGENT_MISSING)
        }
        Some(user_agent) => build_success_response(&user_agent),
    }
}
fn handle_request(mut stream: TcpStream) -> Result<()> {
    println!("new connection");

    let mut request_buffer = BufReader::new(&stream);
    let mut request_line = String::new();
    request_buffer.read_line(&mut request_line)?;

    let mut headers: Vec<(String, String)> = Vec::new();

    loop {
        let mut header_line = String::new();
        let next_header = request_buffer.read_line(&mut header_line)?;
        if header_line == "\r\n" || next_header == 0 {
            break;
        }
        let Some((key, value)) = header_line.split_once(": ") else {
            error!("invalid header");
            continue;
        };
        headers.push((key.to_string(), value.to_string()));
    }

    println!("Request Line: {:?}", request_line);
    let path: Vec<&str> = request_line.split_whitespace().collect();

    let response = match path[..] {
        ["GET", path, "HTTP/1.1"] => {
            if path == "/" {
                format!("{}\r\n\r\n", HTTP_200_OK)
            } else if path == "/user-agent" {
                handle_user_agent(&headers)
            } else if path.starts_with("/echo") {
                let message = path.split("/echo/").nth(1);
                println!("Message: {:?}", message);
                match message {
                    Some(mut message) => {
                        if message == "" {
                            message = ERROR_MESSAGE_ECHO;
                        }
                        build_success_response(message)
                    }
                    None => {
                        error!("No message provided");
                        build_error_response("403", ERROR_MESSAGE_ECHO)
                    }
                }
            } else {
                println!("Random path requested");
                build_error_response("404", "Invalid path")
            }
        }
        _ => {
            error!("no path");
            build_error_response("404", "Invalid path")
        }
    };
    stream.write(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    println!("Server Started");
    env_logger::init();
    let listener: TcpListener = TcpListener::bind("127.0.0.1:3001")?;
    let mut handles = Vec::new();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let handle = std::thread::spawn(|| handle_request(stream));
                handles.push(handle);
            }
            Err(e) => {
                println!("Error: {e}");
            }
        }
    }
    for handle in handles {
        handle.join().unwrap()?;
    }
    Ok(())
}
