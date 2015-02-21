
use std::old_io::{TcpStream, BufferedStream};

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
}

impl Request {
    pub fn new(request_lines: &Vec<String>, 
               stream: BufferedStream<TcpStream>) -> Request {
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
}
