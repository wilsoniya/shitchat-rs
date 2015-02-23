use std::old_io::{TcpListener, TcpStream, BufferedStream};
use std::old_io::{Acceptor, Listener};
use std::thread;
use std::sync::{Arc, Mutex};

use http;
use routes;
use chat::ChatServer;

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

fn handle_client(stream: TcpStream, chat_server: Arc<Mutex<ChatServer>>) {
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
                    request_lines.push(
                        String::from_str(line.as_slice().trim()));
                }
            }
        }
    }

    let mut request = http::Request::new(&request_lines, 
                                         BufferedStream::new(stream2), 
                                         chat_server);
    println!("{}\n", request_lines.connect("\n"));
    let (response, status) = routes::route_request(request); 

    let _ = buf.write_all(
        render_response(response.as_slice(), status).as_bytes());
}

pub fn server(bind_addr: &str, port: u16) -> () {
    let addr_str = format!("{}:{}", bind_addr, port);
    let addr = addr_str.as_slice();
    println!("listening on {}", addr);
    let listener = TcpListener::bind(addr);
    let mut acceptor = listener.listen();
    let chat_server = Arc::new(Mutex::new(ChatServer::new()));

    for conn in acceptor.incoming() {
        match conn {
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
