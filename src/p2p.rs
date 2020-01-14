use async_std::{io, task};
use futures::{prelude::*};
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

#[derive(NetworkBehaviour)]
struct BeaconBehavior<TSubstream: AsyncRead + AsyncWrite> {
    floodsub: floodsub::Floodsub<TSubstream>,
    mdns: Mdns<TSubstream>,
}

impl<TSubstream: AsyncRead + AsyncWrite> NetworkBehaviourEventProcess<FloodsubEvent> for BeaconBehavior<TSubstream> {
    fn inject_event(&mut self, message: FloodsubEvent) {
        if let FloodsubEvent::Message(message) = message {
            println!("Received: '{:?}' from {:?}:", String::from_utf8_lossy(&message.data), message.source);
        }
    }
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

fn p2p_fn() {
    println!("p2p_fn");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p() -> Result<(), Box<dyn Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        println!("local peer id: {:?}", local_peer_id);

        let transport =  libp2p::build_development_transport(local_key);

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

        Ok(())
    }

}

