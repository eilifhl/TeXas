mod swarm;
mod texas_behaviour;

use anyhow::Result;
use tokio::{runtime::Handle, sync::mpsc};

pub enum NetworkEvent {
    LocalPeerId(String),
    Listening(String),
    PeerDiscovered(String),
    PeerExpired(String),
    Log(String),
    Error(String),
}

pub struct NetworkHandle {
    receiver: mpsc::UnboundedReceiver<NetworkEvent>,
}

impl NetworkHandle {
    pub fn poll_events(&mut self) -> Vec<NetworkEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        events
    }
}

pub fn start(runtime: &Handle) -> Result<NetworkHandle> {
    let (sender, receiver) = mpsc::unbounded_channel();
    let service_sender = sender.clone();

    runtime.spawn(async move {
        if let Err(error) = swarm::run(service_sender).await {
            let _ = sender.send(NetworkEvent::Error(error.to_string()));
        }
    });

    Ok(NetworkHandle { receiver })
}
