use std::net::{TcpListener, TcpStream};
use std::io::BufRead;
use std::io::{Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::io::Result;

use bufstream::BufStream;

use http;
use routes;
use chat::ChatServer;

fn render_response(content: &str, status: u32) -> String {
    let status_line = format!("HTTP/1.1 {} OK", status);
    let content_len = format!("Content-Length: {}", content.len());

    let mut response: Vec<&str> = Vec::new();
    response.push(&status_line[..]);
    response.push("Connection: Close");
    response.push(&content_len[..]);
    response.push("");
    response.push(content);
    response.connect("\r\n")
}

fn handle_client(stream: TcpStream, chat_server: Arc<Mutex<ChatServer>>) {
    let stream2 = stream.try_clone().unwrap();
    let mut buf = BufStream::new(stream);
    let mut request_lines = Vec::new();

    loop {
        let mut line = String::new();
        match buf.read_line(&mut line) {
            Err(e) => {
                println!("break: {}", e);
                break;
            }
            Ok(_) => {
                if (&line[..]).trim().len() == 0 {
                    break;
                } else {
                    request_lines.push(
                        String::from((&line[..]).trim()));
                }
            }
        }
    }

    let mut request = http::Request::new(&request_lines,
                                         BufStream::new(stream2),
                                         chat_server);
    println!("{}\n", request_lines.connect("\n"));
    let (response, status) = routes::route_request(request);

    let _ = buf.write_all(
        render_response(&response[..], status).as_bytes());
}

pub fn server(bind_addr: &str, port: u16) {
    let addr_str = format!("{}:{}", bind_addr, port);
    let addr = &addr_str[..];
    println!("listening on {}", addr);
    let mut listener = TcpListener::bind(addr).unwrap();
//  let (mut acceptor, _) = try!(listener.accept());
    let chat_server = Arc::new(Mutex::new(ChatServer::new()));

    for stream in listener.incoming() {
        match stream {
            Err(e) => {
                println!("{}", e);
            }
            Ok(stream) => {
                let cs = chat_server.clone();
                thread::spawn(move|| {
                    handle_client(stream, cs)
                });
            }
        }
    }
}
