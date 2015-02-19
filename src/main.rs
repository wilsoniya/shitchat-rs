#![feature(old_io)] 

use std::old_io::{TcpListener, TcpStream, BufferedStream};
use std::old_io::{Acceptor, Listener};
use std::thread;
use std::iter::IteratorExt;

static document: &'static str = include_str!("index.html");

enum HTTPMethod {GET, PUT, POST}

impl HTTPMethod {
    fn from_str(string: &str) -> HTTPMethod {
        match string {
            "GET" => HTTPMethod::GET,
            "PUT" => HTTPMethod::PUT,
            "POST" => HTTPMethod::POST,
            _ => panic!("{} is not a valid method name", string),
        } 
    }
}

struct Header<'a> {
    key: &'a str,
    value: &'a str,
}

struct Request<'a> {
    method: HTTPMethod,
    path: &'a str,
    protocol: &'a str,
    headers: Vec<Header<'a>>, 
}

impl<'a> Request<'a> {
    fn new(request_lines: &'a Vec<String>) -> Request<'a> {
        let first_line: &'a String = &request_lines[0];

        let frags: Vec<&str> = first_line.as_slice().trim().split_str(" ").collect();
        let (method, path, protocol) = match frags.len() {
            3 => {
                let method = HTTPMethod::from_str(frags[0]);
                (method, frags[1], frags[2]) 
            }
            _ => {
                panic!("Malformed request: {}", first_line);
            }
        }; 

        Request{
            method: method, 
            path: path,
            protocol: protocol,
            headers: Vec::new(),
        }
    }
}


fn render_response(content: &str) -> String {
    let content_len = format!("Content-Length: {}", content.len());

    let mut response: Vec<&str> = Vec::new();
    response.push("HTTP/1.1 200 OK");
    response.push("Server: SHITCHAT/666");
    response.push("Connection: Close");
    response.push(content_len.as_slice());
    response.push("");
    response.push(content); 
    response.connect("\r\n")
} 

fn handle_client(mut stream: TcpStream) {
    let mut buf = BufferedStream::new(stream);
    let mut request_lines = Vec::new();

    loop {
        match buf.read_line() {
            Err(e) => {
                println!("break: {}", e);
                break;
            }
            Ok(line) => {
                if line.as_slice().trim().len() == 0 {
                    break;
                } else {
                    request_lines.push(String::from_str(line.as_slice().trim()));
                }
            }
        }
    }

    let request = Request::new(&request_lines);

    println!("{}\n", request_lines.connect("\n"));

    buf.write_all(render_response(document).as_bytes());
    drop(buf);
}

fn server() -> () {
    let addr = "127.0.0.1:8080";
    println!("listening on {}", addr);
    let listener = TcpListener::bind(addr);
    let mut acceptor = listener.listen();

    for conn in acceptor.incoming() {
        match conn {
            Err(e) => {
                println!("{}", e);
            }
            Ok(stream) => {
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
        }
    }

    drop(acceptor);
}

fn main() {
    server();
    println!("Hello, world!");
}
