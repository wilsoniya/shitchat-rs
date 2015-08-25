//#![feature(old_io)]
//#![feature(collections)]

mod http;
mod views;
mod routes;
mod ws;
mod server;
mod chat;

extern crate sha1;
extern crate rustc_serialize;
extern crate bufstream;
extern crate rand;
extern crate byteorder;


fn main() {
    server::server("127.0.0.1", 8080);
}
