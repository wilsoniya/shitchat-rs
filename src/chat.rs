use std::net::{TcpStream};
use std::io::{Read, Write};
use bufstream::BufStream;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::str::from_utf8;
use rand;
use std::collections::{HashMap, HashSet};

use rustc_serialize::json;

use http::Request;
use ws;

#[derive(Clone, Debug, RustcEncodable)]
struct ClientIdUsername {
    client_id: i64,
    username: String,
}

#[derive(Clone, Debug, RustcEncodable)]
pub enum ServerMessage {
    /// messages from the server
    TextMessage{
        message: String,
        client_id: i64,
    },
    UserHangup{
        client_id: i64,
    },
    UsernameRegistration{
        name: String,
        client_id: i64,
    },
    ClientAcknowledgement{
        client_id: i64,
    },
    UsernameInUse{
        name: String,
    },
    ClientIdUsernameMappings{
        client_id_usernames: Vec<ClientIdUsername>,
    },
}

#[derive(Debug, RustcDecodable)]
pub enum ClientMessage {
    /// messages from clients
    TextMessage{
        message: String,
    },
    UsernameRegistration{
        name: String,
    }
}

#[derive(Clone)]
pub struct ChatClient {
    pub name: Option<String>,
    pub client_id: i64,
    msg_tx: Sender<ServerMessage>,
    server: Arc<Mutex<ChatServer>>,
}

impl ChatClient {
    pub fn run(request: Request) {
        let (tx, rx) = channel::<ServerMessage>();
        let local_tx = tx.clone();
        let mut client = ChatClient {
            name: None, msg_tx: tx, server: request.chat_server.clone(),
            client_id: rand::random()};
        let stream: TcpStream = request.stream.into_inner().unwrap();
        let stream2: TcpStream = stream.try_clone().unwrap();
        let buf_stream = BufStream::new(stream);
        let buf_stream2 = BufStream::new(stream2);

        // create server listener thread
        client.start_server_listener(rx, buf_stream);
        request.chat_server.lock().unwrap().add_client(client.clone());

        // start listening to client via stream
        // this function blocks until the user hangs up
        client.start_client_listener(buf_stream2);

        // when client hangs up, kill the server listener thread
        {
            let mut server = request.chat_server.lock().unwrap();
            server.dispatch_message(
                ServerMessage::UserHangup{
                    client_id: client.client_id.clone(),
                });
            server.rm_client(&client);
        }
    }

    fn start_server_listener
        <T: Read + Write + Send + 'static>(&self, rx: Receiver<ServerMessage>,
                                     stream: BufStream<T>) {
        let mut stream = stream;
        let client = self.clone();
        thread::spawn(move || {
            for msg in rx.iter() {
                match msg {
                    ServerMessage::UserHangup{client_id: client_id, ..}
                        if client_id == client.client_id => {
                            // case: client has signaled it's time to stop
                            return
                    },
                    _ => (),
                }

                let text = json::encode(&msg).unwrap();
                ws::write_stream(&mut stream, &text.into_bytes());
            }
        });
    }

    fn start_client_listener<T: Read + Write>(&mut self, mut stream: BufStream<T>) {
        loop {
            match ws::read_stream(&mut stream) {
                Ok(data) => {
                    let message = match from_utf8(&data[..]) {
                        Ok(message) => message,
                        Err(e) => break
                    };
                    match json::decode(message) {
                        Ok(msg) => {
                            match msg {
                                ClientMessage::UsernameRegistration{name: ref name} => {
                                    self.name = Some(name.clone());
                                },
                                _ => (),
                            }
                            self.server.lock().unwrap()
                                .handle_client_msg(msg, self.client_id);
                        }
                        Err(e) => {
                            println!("Bad message from client: {} {}",
                                     message.trim(), e);
                        }
                    }
                },
                Err(e) => {
                    // case: user hung up; return and bail
                    break;
                }
            }
        }
    }

    pub fn send_msg(&self, event: ServerMessage) {
        self.msg_tx.send(event);
    }
}

pub struct ChatServer {
    clients: HashMap<i64, ChatClient>,
    client_usernames: HashSet<String>,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {
            clients: HashMap::new(),
            client_usernames: HashSet::new(),
        }
    }

    pub fn add_client(&mut self, client: ChatClient) {
        let client_id = client.client_id;

        // send ack message to new client
        let ack_msg = ServerMessage::ClientAcknowledgement{
            client_id: client_id};
        client.send_msg(ack_msg);

        let mut cid_usernames = Vec::new();
        for (k, v) in self.clients.iter() {
            match v.name {
                Some(ref name) => {
                    cid_usernames.push(ClientIdUsername{
                        client_id: k.clone(),
                        username: name.clone(),
                    });
                },
                None => ()
            }
        }

        let cid_username_msg = ServerMessage::ClientIdUsernameMappings{
            client_id_usernames: cid_usernames,
        };

        client.send_msg(cid_username_msg);

        self.clients.insert(client.client_id, client);
        println!("client joined: {} ({} total clients)", client_id,
                 self.clients.len());

    }

    pub fn rm_client(&mut self, client: &ChatClient) {
        let _ = self.clients.remove(&client.client_id);
        println!("client left: {} ({} total clients)", &client.client_id,
                 self.clients.len());

        match client.name {
            Some(ref username) => {
                self.client_usernames.remove(username);
            },
            None => (),
        }
    }

    pub fn register_username(&mut self, username: &String) -> bool {
        if self.client_usernames.contains(username) {
            false
        } else {
            self.client_usernames.insert(username.clone());
            true
        }
    }

    pub fn dispatch_message(&self, msg: ServerMessage) {
        println!("client msg: {:?}", msg);
        for client in self.clients.values() {
            client.send_msg(msg.clone())
        }
    }

    pub fn handle_client_msg(&mut self, msg: ClientMessage, client_id: i64) {
        let server_msg = match msg {
            ClientMessage::TextMessage{message: message} => {
                ServerMessage::TextMessage{message: message,
                                           client_id: client_id}
            },
            ClientMessage::UsernameRegistration{name: name} => {
                let username_opt;

                {
                    let client = self.clients.get(&client_id).unwrap();
                    username_opt = client.name.clone();
                }

                match username_opt {
                    Some(ref username) if username == &name => {
                        println!("Client reregisered username {}", name);
                        return;
                    },
                    _ => (),
                };

                if !self.register_username(&name) {
                    // case: username is in use
                    let msg = ServerMessage::UsernameInUse{name: name.clone()};
                    let client = self.clients.get(&client_id).unwrap();
                    client.send_msg(msg);
                    return;
                } else {
                    // case: username is free
                    match username_opt {
                        Some(ref username) if username == &name => {
                            self.client_usernames.remove(username);
                        },
                        _ => (),
                    };
                    let mut client = self.clients.get_mut(&client_id).unwrap();
                    client.name = Some(name.clone());
                    println!("Client {} is now called {}", client.client_id,
                             name);
                }
                ServerMessage::UsernameRegistration{
                    name: name, client_id: client_id}
            }
        };
        self.dispatch_message(server_msg);
    }
}
