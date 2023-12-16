use serde_json::Value;
use std::fmt::Debug;
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{Read, Write};

pub enum Method {
    GET,
    POST,
    DELETE,
    PUT,
    HEAD,
    PATCH,
    OPTIONS,
}

impl ToString for Method {
    fn to_string(&self) -> String {
        match self {
            Method::GET => "GET".to_string(),
            Method::POST => "POST".to_string(),
            Method::DELETE => "DELETE".to_string(),
            Method::PUT => "PUT".to_string(),
            Method::HEAD => "HEAD".to_string(),
            Method::PATCH => "PATCH".to_string(),
            Method::OPTIONS => "OPTIONS".to_string(),
        }
    }
}

pub fn send_request<A: ToSocketAddrs + Debug>(method: Method, url: A, path: &str, body: Value) -> Result<Value, ()> {
    let mut stream = match TcpStream::connect(&url) {
        Ok(stream) => stream,
        Err(_) => return Err(()),
    };

    let url_string = url.to_socket_addrs().unwrap().last().unwrap().to_string();

    let request = format!(
        "{} {path} HTTP/1.1\r\nHost: {url_string}\r\nContent-Length: {}\r\n\r\n{}",
        method.to_string(),
        body.to_string().len(),
        body.to_string()
    );



    if let Err(_) = stream.write(request.as_bytes()) {
        return Err(());
    };
    if let Err(_) = stream.flush() {
        return Err(());
    };

    let mut buffer = [0; 4096];
    let bytes = match stream.read(&mut buffer) {
        Ok(bytes) => bytes,
        Err(_) => return Err(()),   
    };

    let req_as_string = String::from_utf8_lossy(&buffer[..bytes]).to_string();
    // if req_as_string.is_empty() {
    //     // return Err(());
    //     return Ok(serde_json::json!({}));
    // }

    let splitted_req = req_as_string.split("\r\n\r\n").collect::<Vec<&str>>();


    let body = req_as_string.split("\r\n\r\n").collect::<Vec<&str>>()[1];

    if let Ok(json_body) = serde_json::from_str(body) {
        return Ok(json_body);
    }
    return Ok(serde_json::json!({}));
}
