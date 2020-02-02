use std::net;
use std::io::{Read, Write};
use std::{error::Error, task::{Context, Poll}};
use async_std::{io, task};
use futures::{future, prelude::*};
use serde::{Serialize, Deserialize};

fn main () -> Result<(), Box<dyn Error>> {

    let mut peer_ips: Vec<String> = Vec::new();

    let listener = net::TcpListener::bind(beacon::DISCOVERY_ADDRESS).unwrap();

    task::block_on(future::poll_fn(move |cx: &mut Context| {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let ip = stream.peer_addr().unwrap().ip().to_string();
                    println!("New Connection: {}", ip);
                    if !peer_ips.contains(&ip) {
                        peer_ips.push(ip);
                    }
                    stream.write(&bincode::serialize(&peer_ips).unwrap());
                }
                Err(_) =>{
                    println!("bad");
                }
            }
        }
        Poll::Pending
    }))
}
