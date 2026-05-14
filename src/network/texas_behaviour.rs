use libp2p::{gossipsub, mdns, ping, swarm::NetworkBehaviour};

#[derive(NetworkBehaviour)]
pub struct TexasBehaviour {
    pub ping: ping::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
}
