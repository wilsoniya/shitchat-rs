use std::old_io::{TcpStream, BufferedStream, Stream};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::str::from_utf8;

use http::Request;
use ws;

#[derive(Clone, Debug)]
pub enum ChatEvent {
    TextMessage{user: String, message: String},
    UserHangup{user: String}, 
}

#[derive(Clone)]
pub struct ChatClient {
    pub name: String,
    pub msg_tx: Sender<ChatEvent>,
    server: Arc<Mutex<ChatServer>>,
}

impl ChatClient {
    pub fn run(name: &str, request: Request) {
        let (tx, rx) = channel::<ChatEvent>();
        let local_tx = tx.clone();
        let client = ChatClient {
            name: String::from_str(name), msg_tx: tx, 
            server: request.chat_server.clone()};
        let stream: TcpStream = request.stream.into_inner();
        let stream2: TcpStream = stream.clone();
        let buf_stream = BufferedStream::new(stream);
        let buf_stream2 = BufferedStream::new(stream2);

        // create server listener thread
        client.start_server_listener(rx, buf_stream);
        request.chat_server.lock().unwrap().add_client(client.clone());

        // start listening to client via stream
        client.start_client_listener(buf_stream2);

        // when client hangs up, kill the server listener thread
        request.chat_server.lock().unwrap().handle_client_event(
            ChatEvent::UserHangup{user: client.name.clone()});
    }

    fn start_server_listener
        <T: Stream + Send + 'static>(&self, rx: Receiver<ChatEvent>, 
                                     stream: BufferedStream<T>) {
        let mut stream = stream;
        let client = self.clone();
        thread::spawn(move || {
            for event in rx.iter() {
                match event {
                    ChatEvent::TextMessage{user: user, message: message} => {
                        let text = format!("user: {}; message: {}", user, 
                                           message);
                        ws::write_stream(&mut stream, &text.into_bytes());
                    },
                    ChatEvent::UserHangup{user:user} => {
                        if user == client.name {
                            // case: this client has hung up; harakiri
                            break;
                        }
                    }
                }
            }
        });
    }

    fn start_client_listener<T: Stream>(&self, stream: BufferedStream<T>) {
        let mut stream = stream;
        while true {
            let data = ws::read_stream(&mut stream);
            let message = String::from_str(from_utf8(&data[..]).unwrap());
            let event = ChatEvent::TextMessage{
                user: self.name.clone(), message: message};
            self.server.lock().unwrap().handle_client_event(event);
        }
    }

    pub fn send_event(&self, event: ChatEvent) {
        self.msg_tx.send(event); 
    }
}

pub struct ChatServer {
    clients: Vec<ChatClient>
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {clients: Vec::new()}
    }

    pub fn add_client(&mut self, client: ChatClient) {
        self.clients.push(client);
    }

    pub fn rm_client(&self, client: ChatClient) { }

    pub fn handle_client_event(&self, event: ChatEvent) {
        println!("handling client event: {:?}", event);
        for client in self.clients.iter() {
             client.send_event(event.clone()) 
        } 
    }
}
