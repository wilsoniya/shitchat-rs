use std::old_io::{TcpStream, BufferedStream};
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

pub struct Request {
    pub method: HTTPMethod,
    pub path: String,
    pub protocol: String,
    pub headers: Vec<Header>, 
    pub stream: BufferedStream<TcpStream>,
    pub chat_server: Arc<Mutex<ChatServer>>,
}

impl Request {
    pub fn new(request_lines: &Vec<String>, 
               stream: BufferedStream<TcpStream>, 
               chat_server: Arc<Mutex<ChatServer>>) -> Request {
        let first_line = request_lines[0].clone();

        let frags: Vec<&str> = first_line
            .as_slice()
            .trim()
            .split_str(" ")
            .collect();
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
            chat_server: chat_server,
        }
    }

    pub fn get_header(&self, name: &str) -> Option<String> {
        let header = self.headers
            .iter()
            .filter(|h| { h.key.as_slice() == name })
            .next();
        match header {
            Some(ref h) => Some(h.value.clone()),
            None => None
        }
    }

    pub fn is_websocket(&self) -> bool {
        let connection = self.get_header("Connection");
        let upgrade = self.get_header("Upgrade");

        let connection_string = Some(String::from_str("Upgrade"));
        let upgrade_string = Some(String::from_str("websocket"));

        (connection_string, upgrade_string) == (connection, upgrade) 
    }
}
