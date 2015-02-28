#![feature(old_io)] 
#![feature(collections)]

mod http;
mod views;
mod routes;
mod ws;
mod server;
mod chat;

extern crate "sha1" as sha1;
extern crate "rustc-serialize" as rustc_serialize; 
extern crate serialize;

fn main() {
    server::server("127.0.0.1", 8080);
}
