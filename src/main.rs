#![feature(old_io)] 

mod http;
mod views;
mod routes;
mod ws;

extern crate "sha1" as sha1;
extern crate "rustc-serialize" as rustc_serialize;


use std::old_io::{TcpListener, TcpStream, BufferedStream};
use std::old_io::{Acceptor, Listener};
use std::thread;

fn render_response(content: &str, status: u32) -> String {
    let status_line = format!("HTTP/1.1 {} OK", status);
    let content_len = format!("Content-Length: {}", content.len());

    let mut response: Vec<&str> = Vec::new();
    response.push(status_line.as_slice());
    response.push("Connection: Close");
    response.push(content_len.as_slice());
    response.push("");
    response.push(content); 
    response.connect("\r\n")
} 

fn handle_client(stream: TcpStream) {
    let stream2 = stream.clone();
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

    let mut request = http::Request::new(&request_lines, BufferedStream::new(stream2));
    println!("{}\n", request_lines.connect("\n"));
    let (response, status) = routes::route_request(&mut request); 

    buf.write_all(render_response(response.as_slice(), status).as_bytes());
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
}
