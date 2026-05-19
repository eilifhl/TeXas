use anyhow::Result;
use std::collections::{HashMap, HashSet};
use libp2p::futures::StreamExt;
use libp2p::{Multiaddr, PeerId, gossipsub, mdns, noise, ping, swarm::SwarmEvent, tcp, yamux};
use tokio::sync::mpsc;

use crate::network::{
    NetworkCommand, NetworkEvent,
    texas_behaviour::{TexasBehaviour, TexasBehaviourEvent},
};

pub async fn run(
    events: mpsc::Sender<NetworkEvent>,
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

    send_event(&events, NetworkEvent::LocalPeerId(swarm.local_peer_id().to_string())).await;

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    let mut mdns_peers: HashMap<PeerId, HashSet<Multiaddr>> = HashMap::new();

    loop {
        tokio::select! {
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        send_event(&events, NetworkEvent::Listening(address.to_string())).await;
                    }
                    SwarmEvent::Behaviour(TexasBehaviourEvent::Mdns(event)) => {
                        handle_mdns_event(
                            event,
                            &mut mdns_peers,
                            &mut swarm.behaviour_mut().gossipsub,
                            &events,
                        )
                        .await;
                    }
                    SwarmEvent::Behaviour(TexasBehaviourEvent::Gossipsub(event)) => {
                        handle_gossipsub_event(event, &events).await;
                    }
                    SwarmEvent::Behaviour(event) => {
                        send_event(&events, NetworkEvent::Log(format!("{event:?}"))).await;
                    }
                    _ => {}
                }
            }
            command = commands.recv() => {
                match command {
                    Some(NetworkCommand::Subscribe { topic }) => {
                        subscribe_topic(&mut swarm.behaviour_mut().gossipsub, &topic, &events)
                            .await?;
                    }
                    Some(NetworkCommand::Publish { topic, payload }) => {
                        publish_message(&mut swarm.behaviour_mut().gossipsub, &topic, payload, &events)
                            .await?;
                    }
                    None => {
                        send_event(
                            &events,
                            NetworkEvent::Log("network command channel closed".to_owned()),
                        )
                        .await;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_mdns_event(
    event: mdns::Event,
    mdns_peers: &mut HashMap<PeerId, HashSet<Multiaddr>>,
    gossipsub: &mut gossipsub::Behaviour,
    events: &mpsc::Sender<NetworkEvent>,
) {
    match event {
        mdns::Event::Discovered(list) => {
            for (peer_id, addr) in list {
                let known_addresses = mdns_peers.entry(peer_id).or_default();
                let is_new_peer = known_addresses.is_empty();

                known_addresses.insert(addr);

                if is_new_peer {
                    gossipsub.add_explicit_peer(&peer_id);
                    send_event(events, NetworkEvent::PeerDiscovered(peer_id.to_string())).await;
                }
            }
        }
        mdns::Event::Expired(list) => {
            for (peer_id, addr) in list {
                let Some(known_addresses) = mdns_peers.get_mut(&peer_id) else {
                    continue;
                };

                known_addresses.remove(&addr);

                if known_addresses.is_empty() {
                    mdns_peers.remove(&peer_id);
                    gossipsub.remove_explicit_peer(&peer_id);
                    send_event(events, NetworkEvent::PeerExpired(peer_id.to_string())).await;
                }
            }
        }
    }
}

async fn subscribe_topic(
    gossipsub: &mut gossipsub::Behaviour,
    topic: &str,
    events: &mpsc::Sender<NetworkEvent>,
) -> Result<()> {
    let topic = gossipsub::IdentTopic::new(topic);
    gossipsub.subscribe(&topic)?;
    send_event(events, NetworkEvent::Subscribed(topic.to_string())).await;
    Ok(())
}

async fn publish_message(
    gossipsub: &mut gossipsub::Behaviour,
    topic: &str,
    payload: Vec<u8>,
    events: &mpsc::Sender<NetworkEvent>,
) -> Result<()> {
    let topic = gossipsub::IdentTopic::new(topic);
    gossipsub.publish(topic.clone(), payload)?;
    send_event(
        events,
        NetworkEvent::Log(format!("published message on {}", topic)),
    )
    .await;
    Ok(())
}

async fn handle_gossipsub_event(event: gossipsub::Event, events: &mpsc::Sender<NetworkEvent>) {
    match event {
        gossipsub::Event::Message { message, .. } => {
            let payload = String::from_utf8_lossy(&message.data).into_owned();
            send_event(
                events,
                NetworkEvent::MessageReceived {
                    topic: message.topic.to_string(),
                    payload,
                },
            )
            .await;
        }
        other => {
            send_event(events, NetworkEvent::Log(format!("{other:?}"))).await;
        }
    }
}

async fn send_event(events: &mpsc::Sender<NetworkEvent>, event: NetworkEvent) {
    match event {
        NetworkEvent::Log(message) => {
            let _ = events.try_send(NetworkEvent::Log(message));
        }
        other => {
            let _ = events.send(other).await;
        }
    }
}
