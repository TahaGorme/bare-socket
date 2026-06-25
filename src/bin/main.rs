use anyhow::Result;
use log::error;
use std::{
    fmt::format,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream}, ptr::read,
};

//new ones
const HTTP_200_OK: &str = "HTTP/1.1 200 OK";
const HTTP_404_NOT_FOUND: &str = "HTTP/1.1 404 Not Found";
const HTTP_FORBIDDEN: &str = "HTTP/1.1 403 Invalid Request";

const CONTENT_TYPE_TEXT: &str = "Content-Type: text/plain";

const ERROR_MESSAGE_ECHO: &str = "Message is Required. Example Usage: /echo/{message}";
const ERROR_USER_AGENT_MISSING: &str = "User-Agent header not found";


struct HttpRequest{
    method: String,
    path: String,
    version: String,
    headers: Vec<(String,String)>,
}

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

fn handle_echo(path: &str) -> String {
    let message = path.strip_prefix("/echo/");
    let content = match message {
        Some(msg) if !msg.is_empty() => msg,
        _ => ERROR_MESSAGE_ECHO,
    };
    build_success_response(content)
}


fn parse_request(stream: &mut TcpStream)->Result<HttpRequest>{
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    let mut headers: Vec<(String, String)> = Vec::new();

    loop {
        let mut header_line = String::new();
        let next_header = reader.read_line(&mut header_line)?;
        if header_line == "\r\n" || next_header == 0 {
            break;
        }
        let Some((key, value)) = header_line.split_once(": ") else {
            error!("invalid header");
            continue;
        };
        headers.push((key.to_string(), value.to_string()));
    }

    // println!("Request Line: {:?}", request_line);
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() <3{
        return Err(anyhow::anyhow!("Invalid request"));
    }

    Ok(HttpRequest{
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        version: parts[2].to_string(),
        headers
    })
}
fn handle_request(mut stream: TcpStream) -> Result<()> {
    println!("new connection");
    let request = parse_request(&mut stream)?;
    let response = match request.path.as_str(){

            "/" => format!("{}\r\n\r\n", HTTP_200_OK),
              "/user-agent" => handle_user_agent(&request.headers),
            path if path.starts_with("/echo") =>
                handle_echo(path),
               _=> build_error_response(HTTP_404_NOT_FOUND, "Invalid path")


    };
    stream.write_all(response.as_bytes())?;
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
