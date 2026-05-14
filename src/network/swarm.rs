use std::error::Error;
use libp2p::futures::StreamExt;

use libp2p::{
  yamux,
  tcp,
  ping,
  noise,
  swarm::SwarmEvent,
};

pub async fn start() -> Result<(), Box<dyn Error>> {
  let mut swarm = libp2p::SwarmBuilder::with_new_identity()
    .with_tokio()
    .with_tcp(
      tcp::Config::default(),
      noise::Config::new,
      yamux::Config::default,
    )?
    .with_behaviour(|_| ping::Behaviour::default())?
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
