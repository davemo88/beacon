use async_std::{io, task};
use std::fs;
use hex;
use futures::{future, prelude::*};
use libp2p::{
    Multiaddr,
    PeerId,
    Swarm,
    NetworkBehaviour,
    identity,
    floodsub::{self, Floodsub, FloodsubEvent},
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess
};
use serde::{Serialize, Deserialize};
use std::{error::Error, task::{Context, Poll}};
use std::io::{Read, Write};
use std::net;

#[path = "beacon.rs"] mod beacon;
use crate::beacon::Command;

#[derive(NetworkBehaviour)]
struct BeaconBehavior<TSubstream: AsyncRead + AsyncWrite> {
    floodsub: Floodsub<TSubstream>,
    mdns: Mdns<TSubstream>,
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<MdnsEvent> for BeaconBehavior<TSubstream> {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) =>
                for (peer, _) in list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            MdnsEvent::Expired(list) =>
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
        }
    }
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<FloodsubEvent> for BeaconBehavior<TSubstream> {
    fn inject_event(&mut self, message: FloodsubEvent) {
        if let FloodsubEvent::Message(message) = message {
            let m: beacon::Message = bincode::deserialize(&message.data).unwrap();
            if m.pubkey.verify(&m.get_bytes_to_sign(), &m.sig).is_ok() {
                println!("Beacon '{:?}' is now {:?}", hex::encode(m.pubkey.as_bytes()), m.beacon_state);
            }
            else
            {
                println!("Message from Beacon '{:?}' has bad sig", m.pubkey);
            }


        }
    }
}



fn get_command(mut stream: net::TcpStream) -> Command {
    let mut v = Vec::new();
    stream.read_to_end(&mut v).unwrap();
    let c: beacon::Command = bincode::deserialize(&v).unwrap();
    c
}

#[cfg(test)]
mod tests {
//    use super::*;
//
//    #[test]
//    fn test_p2p() -> Result<(), Box<dyn Error>> {
//    }
}

fn main() ->Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    println!("local peer id: {:?}", local_peer_id);

    let transport = libp2p::build_tcp_ws_secio_mplex_yamux(local_key)?;

    let floodsub_topic = floodsub::TopicBuilder::new("beacon").build();

    let mut swarm = {
        let mdns = task::block_on(Mdns::new())?;
        let mut behavior = BeaconBehavior {
            floodsub: Floodsub::new(local_peer_id.clone()),
            mdns,
        };

        behavior.floodsub.subscribe(floodsub_topic.clone());
        Swarm::new(transport, behavior, local_peer_id)
    };

    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

    let cli_listener = net::TcpListener::bind(beacon::CLI_TCP_ADDRESS).unwrap();

// look for peers from the discovery server
    match net::TcpStream::connect(beacon::DISCOVERY_ADDRESS) {
        Ok(mut stream) => {
            let mut v = Vec::new();
            stream.read_to_end(&mut v).unwrap();
            let peer_ips: Vec<String> = bincode::deserialize(&v).unwrap();
            println!("discovery peer ids: {:?}", peer_ips);
            for ip in peer_ips {
                let addr: Multiaddr = format!("/ip4/{}",ip).parse()?;
                match Swarm::dial_addr(&mut swarm, addr) {
                    Ok(()) => (),
                    Err(_) => println!("couldn't dial {:?}", ip),
                }
            }
            
        }
        Err(e) => println!("couldn't connect to discovery"),
    }

    let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context| {
        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(event)) => println!("{:?}", event),
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => {
                    if !listening {
                        for addr in Swarm::listeners(&swarm) {
                            println!("Listening on {:?}", addr);
                            listening = true;
                        }
                    }
                    break
                }
            }
        }
// here we put checking for commands over the unix socket e.g. from the cli
        for stream in cli_listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    match get_command(stream) {
                        Command::Create(name) => {
                            let b = beacon::Beacon::new(&name);
                            b.save().unwrap();
                            println!("created beacon {}", name);
//                            stream.write("b")
                        },
                        Command::Delete(name) => {
                            beacon::Beacon::delete(&name).unwrap();
                            println!("deleted beacon {}", name);

                        },
                        Command::Broadcast(name,state) => {
                            let b = beacon::Beacon::load(&name);
                            let m = b.create_message(state);
                            swarm.floodsub.publish(&floodsub_topic, bincode::serialize(&m).unwrap());
                            println!("broadcast state {:?} on beacon {}", state, name);
                        },
//                        Command::Subscribe { name: String, pubkey: String } => {
//                            let bs = beacon::BeaconSub {
//                                name: name,
//                                pubkey: pubkey
//                            }
//                        }
//                  //      Command::Unsubscribe(name),
//                  //      Command::ListBeacons(),
//                  //      Command::ListSubs(),
                        _ => println!("unrecognized command"),
                    //            
                    }
                }
                Err(err) => {
                    break;
                }
            }
        }

        Poll::Pending
    })) 
}
