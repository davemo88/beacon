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
use std::os::unix::net::UnixListener;

use crate::beacon;

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

pub fn beacon_p2p() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    println!("local peer id: {:?}", local_peer_id);

    let transport =  libp2p::build_development_transport(local_key)?;

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

    let mut my_beacons: Vec<beacon::Beacon> = Vec::new();

// broadcast if a beacon name is passed
    if let Some(beacon_name) = std::env::args().nth(1) {
        let name: String = beacon_name.parse()?;
        let b = beacon::Beacon::new(&name);
        my_beacons.push(b);
    }

//    if let Some(to_dial) = std::env::args().nth(1) {
//        let addr: Multiaddr = to_dial.parse()?;
//        Swarm::dial_addr(&mut swarm, addr)?;
//        println!("Dialed {:?}", to_dial)
//    }


// main task
// check for new messages from p2p channel subscriptions
// check for new commands from the cli

    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

// TODO: clean this up properly somewhere else
    let socket = std::path::Path::new(beacon::SOCKET_PATH);
    if socket.exists() {
        fs::remove_file(&socket).unwrap();
    }
    let cli_listener = UnixListener::bind(socket).unwrap();

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
// need to read command as bytes, deserialize, match on enum, call function, send response
                }
                Err(err) => {
                    break;
                }
            }
        }
        if my_beacons.len() > 0 {
            let m = my_beacons[0].create_message(true);
            swarm.floodsub.publish(&floodsub_topic, bincode::serialize(&m).unwrap());
        }
        Poll::Pending
    }))
}

#[cfg(test)]
mod tests {
//    use super::*;
//
//    #[test]
//    fn test_p2p() -> Result<(), Box<dyn Error>> {
//    }
}


fn main() {
    beacon_p2p().unwrap();
}
