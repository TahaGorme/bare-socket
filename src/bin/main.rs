use anyhow::Result;
use log::error;
use std::result::Result::Ok;
use std::{
    // any, error,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

const SUCCESS_BASE: &str = "HTTP/1.1 200 OK";
const SUCCESS_RESPONSE: &str = "HTTP/1.1 200 OK\r\n";
const ERROR_RESPONSE: &str = "HTTP/1.1 404 Not Found\r\n";
const ERROR_403: &str = "HTTP/1.1 403 Invalid Request\r\n";
const ERROR_MESSAGE: &str = "Message is Required. Example Usage: /echo/{message}";


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
                SUCCESS_RESPONSE.to_string()
            } else if path == "/user-agent" {
                let mut user_agent = None;
                for (key, value) in headers {
                    if key == "User-Agent" {
                        user_agent = Some(value);
                        break;
                    }
                }
                match user_agent {
                    None => {
                        error!("User agent not found in header");
                        ERROR_403.to_string()
                    }
                    Some(user_agent) => format!(
                        "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        SUCCESS_BASE,
                        user_agent.len(),
                        user_agent
                    ),
                }
            } else if path.starts_with("/echo") {
                let message = path.split("/echo/").nth(1);
                println!("Message: {:?}", message);
                match message {
                    Some(mut message) => {
                        if message == "" {
                            message = ERROR_MESSAGE;
                        }
                        format!(
                            "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                            SUCCESS_BASE,
                            message.len(),
                            message
                        )
                    }
                    None => {
                        error!("No message provided");
                        format!(
                            "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                            SUCCESS_BASE,
                            ERROR_MESSAGE.len(),
                            ERROR_MESSAGE
                        )
                    }
                }
            } else {
                println!("Random path requested");
                ERROR_RESPONSE.to_string()
            }
        }
        _ => {
            error!("no path");
            ERROR_RESPONSE.to_string()
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
