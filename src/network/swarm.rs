use std::error::Error;

use libp2p::{
  identity,
  PeerId,
};

pub async fn start() -> Result<(), Box<dyn Error>> {
  let key_pair = identity::Keypair::generate_ed25519();
  let peer_id = PeerId::from(key_pair.public());

  println!("local peer id: {peer_id}");

  Ok(())
}
