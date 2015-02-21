use std::old_io::timer::sleep;
use std::time::duration::Duration; 
use std::str::from_utf8;

use sha1::Sha1;
use rustc_serialize::base64::ToBase64;
use rustc_serialize::base64::STANDARD;

use ws;
use http::Request;

static DOCUMENT: &'static str = include_str!("index.html");
static WS_GUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

fn verify_key(key: &String) -> String {
    let mut key = key.clone();
    key.push_str(WS_GUID);
    let mut sha = Sha1::new();
    sha.update(key.as_bytes());
    let digest: Vec<u8> = sha.digest();
    digest.as_slice().to_base64(STANDARD)
}


pub fn index(request: &mut Request) -> (String, u32) {
    (String::from_str(DOCUMENT), 200)
}

pub fn ws(request: &mut Request) -> (String, u32) {
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

    while true {
        let data = ws::read_stream(&mut request.stream); 
        ws::write_stream(&mut request.stream, &data);
        println!("{}", from_utf8(data.as_slice()).unwrap());
    } 

    (String::from_str("fin"), 200)
} 

pub fn error_404(request: &mut Request) -> (String, u32) {
    (String::from_str("Not Found"), 404)
}

pub fn error_500(request: &mut Request) -> (String, u32) {
    (String::from_str("Internal Server Error"), 500)
}
