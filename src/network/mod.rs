mod swarm;
mod texas_behaviour;

use anyhow::Result;
use tokio::{runtime::Handle, sync::mpsc};

pub const DEFAULT_DOCUMENT_TOPIC: &str = "texas/document/main";

pub enum NetworkEvent {
    LocalPeerId(String),
    Listening(String),
    PeerDiscovered(String),
    PeerExpired(String),
    Subscribed(String),
    MessageReceived { topic: String, payload: String },
    Log(String),
    Error(String),
}

pub enum NetworkCommand {
    Subscribe { topic: String },
    Publish { topic: String, payload: Vec<u8> },
}

pub struct NetworkHandle {
    receiver: mpsc::UnboundedReceiver<NetworkEvent>,
    command_sender: mpsc::UnboundedSender<NetworkCommand>,
}

impl NetworkHandle {
    pub fn poll_events(&mut self) -> Vec<NetworkEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        events
    }

    pub fn subscribe(&self, topic: impl Into<String>) -> Result<()> {
        self.command_sender
            .send(NetworkCommand::Subscribe {
                topic: topic.into(),
            })
            .map_err(|error| anyhow::anyhow!("failed to queue subscribe command: {error}"))
    }

    pub fn publish(&self, topic: impl Into<String>, payload: Vec<u8>) -> Result<()> {
        self.command_sender
            .send(NetworkCommand::Publish {
                topic: topic.into(),
                payload,
            })
            .map_err(|error| anyhow::anyhow!("failed to queue publish command: {error}"))
    }
}

pub fn start(runtime: &Handle) -> Result<NetworkHandle> {
    let (event_sender, receiver) = mpsc::unbounded_channel();
    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let service_sender = event_sender.clone();

    runtime.spawn(async move {
        if let Err(error) = swarm::run(service_sender, command_receiver).await {
            let _ = event_sender.send(NetworkEvent::Error(error.to_string()));
        }
    });

    Ok(NetworkHandle {
        receiver,
        command_sender,
    })
}
