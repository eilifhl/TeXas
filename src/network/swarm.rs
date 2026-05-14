use std::error::Error;

use libp2p::{
  identity,
  PeerId,
  yamux,
  tcp,
  ping,
  noise,
};

pub async fn start() -> Result<(), Box<dyn Error>> {
  //let key_pair = identity::Keypair::generate_ed25519();
  //let peer_id = PeerId::from(key_pair.public());

  //println!("local peer id: {peer_id}");
  
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

  Ok(())
}
