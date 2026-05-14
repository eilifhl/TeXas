use std::error::Error;
use libp2p::futures::StreamExt;

use libp2p::{
  yamux,
  tcp,
  ping,
  mdns,
  noise,
  gossipsub,
  swarm::SwarmEvent,
};

use crate::network::texas_behaviour::TexasBehaviour;

pub async fn start() -> Result<(), Box<dyn Error>> {
  let mut swarm = libp2p::SwarmBuilder::with_new_identity()
    .with_tokio()
    .with_tcp(
      tcp::Config::default(),
      noise::Config::new,
      yamux::Config::default,
    )?
    .with_behaviour(|key| {
      let peer_id = key.public().to_peer_id();

      let ping = ping::Behaviour::default();

      let mdns = mdns::tokio::Behaviour::new(
        mdns::Config::default(),
        peer_id,
      )?;

      let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(key.clone()),
        gossipsub::Config::default(),
      )?;

      Ok(TexasBehaviour{ping, mdns, gossipsub})
    })?
    .build();

  let peer_id = swarm.local_peer_id();
  println!("local peer id: {peer_id}");

  swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

  loop {
    match swarm.select_next_some().await {
      SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
      SwarmEvent::Behaviour(event) => println!("{event:?}"),
      _ => {}
    }
  }
}
