#![feature(old_io)] 

extern crate "rustc-serialize" as rustc_serialize;
extern crate "sha1" as sha1;

use std::old_io::{TcpListener, TcpStream, BufferedStream};
use std::old_io::{Acceptor, Listener};
use std::thread;
use std::iter::IteratorExt;
use std::old_io::timer::sleep;
use std::time::duration::Duration; 

use rustc_serialize::base64::ToBase64;
use rustc_serialize::base64::STANDARD;

use sha1::Sha1;

static document: &'static str = include_str!("index.html");
static ws_guid: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Show)]
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

#[derive(Show)]
struct Header {
    key: String,
    value: String,
}

impl Header {
    fn from_str(string: &str) -> Header {
        let mut kv = string.splitn(1, ':');
        let key = String::from_str(kv.next().unwrap());
        let value = String::from_str(kv.next().unwrap().trim_left());
        Header {
            key: key, 
            value: value,
        }
    }
}

struct Request {
    method: HTTPMethod,
    path: String,
    protocol: String,
    headers: Vec<Header>, 
    stream: BufferedStream<TcpStream>,
}

impl Request {
    fn new(request_lines: &Vec<String>, stream: BufferedStream<TcpStream>) -> Request {
        let first_line = request_lines[0].clone();

        let frags: Vec<&str> = first_line.as_slice().trim().split_str(" ").collect();
        let (method, path, protocol) = match frags.len() {
            3 => {
                let method = HTTPMethod::from_str(frags[0]);
                (method, String::from_str(frags[1]), String::from_str(frags[2])) 
            }
            _ => {
                panic!("Malformed request: {}", first_line);
            }
        }; 

        let headers = request_lines
            .as_slice()[1..]
            .iter()
            .map(|h| {Header::from_str(h)})
            .collect();

        Request{
            method: method, 
            path: path,
            protocol: protocol,
            headers: headers,
            stream: stream,
        }
    }

    fn get_header(&self, name: &str) -> Option<String> {
        let header = self.headers
            .iter()
            .filter(|h| { h.key.as_slice() == name })
            .next();
        match header {
            Some(ref h) => Some(h.value.clone()),
            None => None
        }
    }
}


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

fn index(request: &mut Request) -> (String, u32) {
    (String::from_str(document), 200)
}

fn verify_key(key: &String) -> String {
    let mut key = key.clone();
    key.push_str(ws_guid);
    let mut sha = Sha1::new();
    sha.update(key.as_bytes());
    let digest: Vec<u8> = sha.digest();
    digest.as_slice().to_base64(STANDARD)
}

fn ws(request: &mut Request) -> (String, u32) {
    let ws_key = request.get_header("Sec-WebSocket-Key").unwrap();
    let accept_key = verify_key(&ws_key);
    let accept_key_header = format!("Sec-WebSocket-Accept: {}", accept_key);

    let mut response: Vec<&str> = Vec::new(); 

    response.push("HTTP/1.1 101 Switching Protocols");
    response.push("Upgrade: websocket");
    response.push("Connection: Upgrade");
    response.push(accept_key_header.as_slice());
    response.push("Sec-WebSocket-Protocol: chat");
    response.push("");
    response.push("");

    let res_str = response.connect("\r\n");
    print!("{}", res_str);

    request.stream.write_all(res_str.as_slice().as_bytes()); 
    request.stream.flush();

    sleep(Duration::seconds(1));

    request.stream.write_str("farrrrrrrt");
    println!("farrrrrrrt\r\n\r\n");

    sleep(Duration::seconds(1));

    for line in request.stream.lines() {
        print!("about to print a line...");
        match line {
            Ok(line) => {
                println!("got line! {}", line);
            }
            Err(e) => {
                println!("Error while getting line: {}", e);
            }
        }
    } 

    (String::from_str("fin"), 200)
} 

fn error_404(request: &mut Request) -> (String, u32) {
    (String::from_str("Not Found"), 404)
}

fn error_500(request: &mut Request) -> (String, u32) {
    (String::from_str("Internal Server Error"), 500)
}

fn route_request(request: &mut Request) -> (String, u32) {
    let view_fn: fn(&mut Request) -> (String, u32) = match (&request.method, request.path.as_slice()) {
        (&HTTPMethod::GET, "/") => index, 
        (&HTTPMethod::GET, "/ws/") => ws, 
        _ => error_404,
    };

    view_fn(request)
}

fn handle_client(mut stream: TcpStream) {
    let mut stream2 = stream.clone();
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

    let mut request = Request::new(&request_lines, BufferedStream::new(stream2));
    println!("{}\n", request_lines.connect("\n"));
    let (response, status) = route_request(&mut request); 

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
