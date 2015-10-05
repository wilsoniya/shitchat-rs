use std::net::{TcpStream};
use bufstream::BufStream;
use std::sync::{Arc, Mutex};

use chat::ChatServer;

// Client request:
// ===============
//   GET /chat HTTP/1.1
//   Host: server.example.com
//   Upgrade: websocket
//   Connection: Upgrade
//   Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==
//   Sec-WebSocket-Protocol: chat, superchat
//   Sec-WebSocket-Version: 13
//   Origin: http://example.com
//
// Server response:
// ================
//   HTTP/1.1 101 Switching Protocols
//   Upgrade: websocket
//   Connection: Upgrade
//   Sec-WebSocket-Accept: HSmrc0sMlYUkAGmm5OPpG2HaGWk=
//   Sec-WebSocket-Protocol: chat

#[derive(Show)]
pub enum HTTPMethod {GET, PUT, POST}

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
pub struct Header {
    pub key: String,
    pub value: String,
}

impl Header {
    fn from_str(string: &str) -> Header {
        println!("header: {}", string);
        let mut kv = string.splitn(2, ':');
        let key = String::from(kv.next().unwrap());
        let value = String::from(kv.next().unwrap().trim_left());
        Header {
            key: key,
            value: value,
        }
    }
}

pub struct Request {
    pub method: HTTPMethod,
    pub path: String,
    pub protocol: String,
    pub headers: Vec<Header>,
    pub stream: BufStream<TcpStream>,
    pub chat_server: Arc<Mutex<ChatServer>>,
}

impl Request {
    pub fn new(request_lines: &Vec<String>,
               stream: BufStream<TcpStream>,
               chat_server: Arc<Mutex<ChatServer>>) -> Request {
        let first_line = &request_lines[0];

        let frags: &Vec<&str> = &first_line[..]
            .trim()
            .split_whitespace()
            .collect::<Vec<&str>>();
        let (method, path, protocol) = match frags.len() {
            3 => {
                let method = HTTPMethod::from_str(frags[0]);
                (method, String::from(frags[1]), String::from(frags[2]))
            }
            _ => {
                panic!("Malformed request: {}", first_line);
            }
        };

        let headers = request_lines
            .iter()
            .skip(1)
            .map(|h| {Header::from_str(h)})
            .collect::<Vec<Header>>();

        Request{
            method: method,
            path: path,
            protocol: protocol,
            headers: headers,
            stream: stream,
            chat_server: chat_server,
        }
    }

    pub fn get_header(&self, name: &str) -> Option<String> {
        let header = self.headers
            .iter()
            .filter(|h| { &h.key[..] == name })
            .next();
        match header {
            Some(ref h) => Some(h.value.clone()),
            None => None
        }
    }

    pub fn is_websocket(&self) -> bool {
        let connection = self.get_header("Connection");
        let upgrade = self.get_header("Upgrade");

        let connection_string = Some(String::from("Upgrade"));
        let upgrade_string = Some(String::from("websocket"));

        (connection_string, upgrade_string) == (connection, upgrade)
    }
}
