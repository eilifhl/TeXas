use anyhow::Result;
use std::collections::{HashMap, HashSet};
use libp2p::futures::StreamExt;
use libp2p::{Multiaddr, PeerId, gossipsub, mdns, noise, ping, swarm::SwarmEvent, tcp, yamux};
use tokio::sync::mpsc;

use crate::network::{
    RepaintSignal,
    NetworkCommand, NetworkEvent,
    texas_behaviour::{TexasBehaviour, TexasBehaviourEvent},
};

pub async fn run(
    events: mpsc::Sender<NetworkEvent>,
    mut commands: mpsc::UnboundedReceiver<NetworkCommand>,
    repaint: RepaintSignal,
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

    emit_event(
        &events,
        &repaint,
        NetworkEvent::LocalPeerId(swarm.local_peer_id().to_string()),
    )
    .await;

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    let mut mdns_peers: HashMap<PeerId, HashSet<Multiaddr>> = HashMap::new();

    loop {
        tokio::select! {
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        emit_event(&events, &repaint, NetworkEvent::Listening(address.to_string()))
                            .await;
                    }
                    SwarmEvent::Behaviour(TexasBehaviourEvent::Mdns(event)) => {
                        handle_mdns_event(
                            event,
                            &mut mdns_peers,
                            &mut swarm.behaviour_mut().gossipsub,
                            &events,
                            &repaint,
                        )
                        .await;
                    }
                    SwarmEvent::Behaviour(TexasBehaviourEvent::Gossipsub(event)) => {
                        handle_gossipsub_event(event, &events, &repaint).await;
                    }
                    SwarmEvent::Behaviour(TexasBehaviourEvent::Ping(_)) => {}
                    _ => {}
                }
            }
            command = commands.recv() => {
                match command {
                    Some(NetworkCommand::Subscribe { topic }) => {
                        if let Err(error) = subscribe_topic(
                            &mut swarm.behaviour_mut().gossipsub,
                            &topic,
                            &events,
                            &repaint,
                        )
                        .await
                        {
                            emit_event(&events, &repaint, NetworkEvent::Error(error.to_string()))
                                .await;
                        }
                    }
                    Some(NetworkCommand::Publish { topic, payload }) => {
                        if let Err(error) = publish_message(
                            &mut swarm.behaviour_mut().gossipsub,
                            &topic,
                            payload,
                        )
                        .await
                        {
                            emit_event(&events, &repaint, NetworkEvent::Error(error.to_string()))
                                .await;
                        }
                    }
                    None => {
                        emit_event(
                            &events,
                            &repaint,
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
    repaint: &RepaintSignal,
) {
    match event {
        mdns::Event::Discovered(list) => {
            for (peer_id, addr) in list {
                let known_addresses = mdns_peers.entry(peer_id).or_default();
                let is_new_peer = known_addresses.is_empty();

                known_addresses.insert(addr);

                if is_new_peer {
                    gossipsub.add_explicit_peer(&peer_id);
                    emit_event(events, repaint, NetworkEvent::PeerDiscovered(peer_id.to_string()))
                        .await;
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
                    emit_event(events, repaint, NetworkEvent::PeerExpired(peer_id.to_string()))
                        .await;
                }
            }
        }
    }
}

async fn subscribe_topic(
    gossipsub: &mut gossipsub::Behaviour,
    topic: &str,
    events: &mpsc::Sender<NetworkEvent>,
    repaint: &RepaintSignal,
) -> Result<()> {
    let topic = gossipsub::IdentTopic::new(topic);
    gossipsub.subscribe(&topic)?;
    emit_event(events, repaint, NetworkEvent::Subscribed(topic.to_string())).await;
    Ok(())
}

async fn publish_message(
    gossipsub: &mut gossipsub::Behaviour,
    topic: &str,
    payload: Vec<u8>,
) -> Result<()> {
    let topic = gossipsub::IdentTopic::new(topic);
    gossipsub.publish(topic, payload)?;
    Ok(())
}

async fn handle_gossipsub_event(
    event: gossipsub::Event,
    events: &mpsc::Sender<NetworkEvent>,
    repaint: &RepaintSignal,
) {
    match event {
        gossipsub::Event::Message { message, .. } => {
            emit_event(
                events,
                repaint,
                NetworkEvent::MessageReceived {
                    topic: message.topic.to_string(),
                    payload: message.data,
                },
            )
            .await;
        }
        other => {
            emit_event(events, repaint, NetworkEvent::Log(format!("{other:?}"))).await;
        }
    }
}

async fn emit_event(
    events: &mpsc::Sender<NetworkEvent>,
    repaint: &RepaintSignal,
    event: NetworkEvent,
) {
    let queued = match event {
        NetworkEvent::Log(message) => {
            events.try_send(NetworkEvent::Log(message)).is_ok()
        }
        other => {
            events.send(other).await.is_ok()
        }
    };

    if queued {
        repaint.as_ref()();
    }
}
