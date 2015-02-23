use std::old_io::{TcpStream, BufferedStream, Stream};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::str::from_utf8;
use std::rand;
use std::collections::HashMap;

use http::Request;
use ws;

#[derive(Clone, Debug)]
pub enum ChatEvent {
    TextMessage{
        user: Option<String>, 
        message: String,
        client_id: i64,
    },
    UserHangup{
        user: Option<String>,
        client_id: i64,
    }, 
    UsernameRegistration{
        name: String,
        client_id: i64,
    }
}

#[derive(Clone)]
pub struct ChatClient {
    pub name: Option<String>,
    pub msg_tx: Sender<ChatEvent>,
    pub client_id: i64,
    server: Arc<Mutex<ChatServer>>,
}

impl ChatClient {
    pub fn run(request: Request) {
        let (tx, rx) = channel::<ChatEvent>();
        let local_tx = tx.clone();
        let client = ChatClient { 
            name: None, msg_tx: tx, server: request.chat_server.clone(), 
            client_id: rand::random()};
        let stream: TcpStream = request.stream.into_inner();
        let stream2: TcpStream = stream.clone();
        let buf_stream = BufferedStream::new(stream);
        let buf_stream2 = BufferedStream::new(stream2);

        // create server listener thread
        client.start_server_listener(rx, buf_stream);
        request.chat_server.lock().unwrap().add_client(client.clone());

        // start listening to client via stream
        // this function blocks until the user hangs up
        client.start_client_listener(buf_stream2);

        // when client hangs up, kill the server listener thread
        {
            let mut server = request.chat_server.lock().unwrap();
            server.handle_client_event(
                ChatEvent::UserHangup{
                    user: client.name.clone(),
                    client_id: client.client_id.clone(),
                });
            server.rm_client(&client);
        }
    }

    fn start_server_listener
        <T: Stream + Send + 'static>(&self, rx: Receiver<ChatEvent>, 
                                     stream: BufferedStream<T>) {
        let mut stream = stream;
        let client = self.clone();
        thread::spawn(move || {
            for event in rx.iter() {
                match event {
                    ChatEvent::TextMessage{client_id: client_id, 
                        message: message, ..} => {
                        let text = format!("client_id: {}; message: {}", 
                                           client_id, message);
                        ws::write_stream(&mut stream, &text.into_bytes());
                    },
                    ChatEvent::UserHangup{client_id: client_id, ..} => {
                        if client_id == client.client_id {
                            // case: this client has hung up; harakiri
                            break;
                        }
                    },
                    ChatEvent::UsernameRegistration{name: name, ..} => {
                        // pass for now
                    }
                }
            }
        });
    }

    fn start_client_listener<T: Stream>(&self, stream: BufferedStream<T>) {
        let mut stream = stream;
        while true {
            match ws::read_stream(&mut stream) {
                Ok(data) => {
                    let message = String::from_str(from_utf8(&data[..]).unwrap());
                    let event = ChatEvent::TextMessage{
                        user: self.name.clone(), 
                        message: message,
                        client_id: self.client_id.clone(),
                    };
                    self.server.lock().unwrap().handle_client_event(event);
                },
                Err(e) => {
                    // case: user hung up; return and bail
                    break;
                }
            }
        }
    }

    pub fn send_event(&self, event: ChatEvent) {
        self.msg_tx.send(event); 
    }
}

pub struct ChatServer {
    clients: HashMap<i64, ChatClient>,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {clients: HashMap::new()}
    }

    pub fn add_client(&mut self, client: ChatClient) {
        self.clients.insert(client.client_id, client);
    }

    pub fn rm_client(&mut self, client: &ChatClient) {
        let _ = self.clients.remove(&client.client_id);
    }

    pub fn handle_client_event(&self, event: ChatEvent) {
        println!("handling client event: {:?}", event);
        for client in self.clients.values() {
            client.send_event(event.clone()) 
        } 
    }
}
