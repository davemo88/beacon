use async_std::{io, task};
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
use std::{error::Error, task::{Context, Poll}};

pub fn chat_example() -> Result<(), Box<dyn Error>> {
//    env_logger::init();

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    println!("local peer id: {:?}", local_peer_id);

    let transport =  libp2p::build_development_transport(local_key)?;

    let floodsub_topic = floodsub::TopicBuilder::new("beacon").build();

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
                println!("Received: '{:?}' from {:?}:", String::from_utf8_lossy(&message.data), message.source);
            }
        }
    }

    let mut swarm = {
        let mdns = task::block_on(Mdns::new())?;
        let mut behavior = BeaconBehavior {
            floodsub: Floodsub::new(local_peer_id.clone()),
            mdns,
        };

        behavior.floodsub.subscribe(floodsub_topic.clone());
        Swarm::new(transport, behavior, local_peer_id)
    };

    if let Some(to_dial) = std::env::args().nth(1) {
        let addr: Multiaddr = to_dial.parse()?;
        Swarm::dial_addr(&mut swarm, addr)?;
        println!("Dialed {:?}", to_dial)
    }

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context| {
        loop {
            match stdin.try_poll_next_unpin(cx)? {
                Poll::Ready(Some(line)) => swarm.floodsub.publish(&floodsub_topic, line.as_bytes()),
                Poll::Ready(None) => panic!("Stdin closed"),
                Poll::Pending => break
            }
        }
        loop
        {
            match swarm.poll_next_unpin(cx)
            {
                Poll::Ready(Some(event)) => println!("{:?}", event),
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => {
                    if !listening {
                        for addr in Swarm::listeners(&swarm) {
                            println!("Listending on {:?}", addr);
                            listening = true;
                        }
                    }
                    break
                }
            }
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

