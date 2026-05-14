mod network;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  network::start().await?;
  Ok(())
}
