use anyhow::Result;
use libp2p::futures::StreamExt;
use libp2p::{gossipsub, mdns, noise, ping, swarm::SwarmEvent, tcp, yamux};
use tokio::sync::mpsc;

use crate::network::{
    NetworkCommand, NetworkEvent,
    texas_behaviour::{TexasBehaviour, TexasBehaviourEvent},
};

pub async fn run(
    events: mpsc::UnboundedSender<NetworkEvent>,
    mut commands: mpsc::UnboundedReceiver<NetworkCommand>,
) -> Result<()> {
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
            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub::Config::default(),
            )?;

            Ok(TexasBehaviour {
                ping,
                mdns,
                gossipsub,
            })
        })?
        .build();

    let _ = events.send(NetworkEvent::LocalPeerId(swarm.local_peer_id().to_string()));

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        tokio::select! {
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        let _ = events.send(NetworkEvent::Listening(address.to_string()));
                    }
                    SwarmEvent::Behaviour(TexasBehaviourEvent::Mdns(event)) => {
                        handle_mdns_event(event, &mut swarm.behaviour_mut().gossipsub, &events);
                    }
                    SwarmEvent::Behaviour(TexasBehaviourEvent::Gossipsub(event)) => {
                        handle_gossipsub_event(event, &events);
                    }
                    SwarmEvent::Behaviour(event) => {
                        let _ = events.send(NetworkEvent::Log(format!("{event:?}")));
                    }
                    _ => {}
                }
            }
            command = commands.recv() => {
                match command {
                    Some(NetworkCommand::Subscribe { topic }) => {
                        subscribe_topic(&mut swarm.behaviour_mut().gossipsub, &topic, &events)?;
                    }
                    Some(NetworkCommand::Publish { topic, payload }) => {
                        publish_message(&mut swarm.behaviour_mut().gossipsub, &topic, payload, &events)?;
                    }
                    None => {
                        let _ = events.send(NetworkEvent::Log("network command channel closed".to_owned()));
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn handle_mdns_event(
    event: mdns::Event,
    gossipsub: &mut gossipsub::Behaviour,
    events: &mpsc::UnboundedSender<NetworkEvent>,
) {
    match event {
        mdns::Event::Discovered(list) => {
            for (peer_id, _addr) in list {
                gossipsub.add_explicit_peer(&peer_id);
                let _ = events.send(NetworkEvent::PeerDiscovered(peer_id.to_string()));
            }
        }
        mdns::Event::Expired(list) => {
            for (peer_id, _addr) in list {
                gossipsub.remove_explicit_peer(&peer_id);
                let _ = events.send(NetworkEvent::PeerExpired(peer_id.to_string()));
            }
        }
    }
}

fn subscribe_topic(
    gossipsub: &mut gossipsub::Behaviour,
    topic: &str,
    events: &mpsc::UnboundedSender<NetworkEvent>,
) -> Result<()> {
    let topic = gossipsub::IdentTopic::new(topic);
    gossipsub.subscribe(&topic)?;
    let _ = events.send(NetworkEvent::Subscribed(topic.to_string()));
    Ok(())
}

fn publish_message(
    gossipsub: &mut gossipsub::Behaviour,
    topic: &str,
    payload: Vec<u8>,
    events: &mpsc::UnboundedSender<NetworkEvent>,
) -> Result<()> {
    let topic = gossipsub::IdentTopic::new(topic);
    gossipsub.publish(topic.clone(), payload)?;
    let _ = events.send(NetworkEvent::Log(format!("published message on {}", topic)));
    Ok(())
}

fn handle_gossipsub_event(event: gossipsub::Event, events: &mpsc::UnboundedSender<NetworkEvent>) {
    match event {
        gossipsub::Event::Message { message, .. } => {
            let payload = String::from_utf8_lossy(&message.data).into_owned();
            let _ = events.send(NetworkEvent::MessageReceived {
                topic: message.topic.to_string(),
                payload,
            });
        }
        other => {
            let _ = events.send(NetworkEvent::Log(format!("{other:?}")));
        }
    }
}
