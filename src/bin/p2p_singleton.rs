/**
 * Testing libp2p communicator singleton class with terminal chat.
 */
use futures::StreamExt;
use libp2p::{gossipsub, mdns, swarm::SwarmEvent};
use std::{error::Error, str::FromStr};
use tokio::io::{self, AsyncBufReadExt};

use cryptochain::p2p_mdns_singleton::{self, P2PBehaviourEvent, TopicEnum};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let connection = p2p_mdns_singleton::P2PConnection::global().await;
	tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

	println!("\nType messages to send. 'exit' to quit.");
	println!("=================\n");

	let mut stdin = io::BufReader::new(io::stdin()).lines();
	let topic = gossipsub::IdentTopic::new(TopicEnum::Blockchain.to_string());

	loop {
		tokio::select! {
			line = stdin.next_line() => {
				if let Ok(Some(input)) = line {
					let input = input.trim();

					if input == "exit" {
						break;
					}
					let mut swarm = connection.swarm.lock().await;

					if connection.connected_peers.read().await.len() > 0 && !input.is_empty() {
						match swarm.behaviour_mut().gossipsub.publish(topic.clone(), input.as_bytes()) {
							Ok(_) => println!("Message sent!"),
							Err(e) => eprintln!("Failed to send: {}", e),
						}
					} else if !input.is_empty() {
						println!("No connected peers yet. Waiting for automatic discovery...");
					}
				}
			},
			event = async {
				let mut swarm = connection.swarm.lock().await;
				swarm.select_next_some().await
			} => {
				match event {
					SwarmEvent::NewListenAddr { address, .. } => {
						println!("* Listening on: {}", address);
					}
					SwarmEvent::Behaviour(P2PBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
						println!(">! {:#?}", message);

						let topic: &String = &message.topic.to_string();
						let text = String::from_utf8_lossy(&message.data);

						if let Ok(topic_enum) = TopicEnum::from_str(&topic) {
							match topic_enum {
								TopicEnum::Blockchain => {
									// this is where chain replacement is.
									let text = String::from_utf8_lossy(&message.data);
									println!(">> {text}");
								},
								_ => {
									println!("Unknown channel message.");
								}
							}
						}

						println!(">! {:#?}", message);
						println!(">> {}, {}", text, topic);
					}
					SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
						let mut swarm = connection.swarm.lock().await;
						for (peer_id, addr) in list {
							println!("* Discovered peer: {} at {}", peer_id, addr);
							// Force connection to discovered peer
							if let Err(e) = swarm.dial(addr) {
								eprintln!("Failed to connect to {}: {}", peer_id, e);
							}
						}
					}
					SwarmEvent::Behaviour(p2p_mdns_singleton::P2PBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
						let mut connected_peers = connection.connected_peers.write().await;
						for (peer_id, _addr) in list {
							println!("* Peer expired: {}", peer_id);
							// connected_peers = connected_peers.saturating_sub(1);
							connected_peers.remove(&peer_id);
						}
					}
					SwarmEvent::ConnectionEstablished { peer_id, .. } => {
						// peer deduplication
						let mut connected_peers = connection.connected_peers.write().await;
						let mut swarm = connection.swarm.lock().await;
						let count = connected_peers.entry(peer_id).and_modify(|c| *c += 1).or_insert(1);

						if *count == 1 {
							println!("* New peer: {} ({} unique peers)", peer_id, connected_peers.len());
						} else {
							println!("* Additional connection to: {} ({} total connections)", peer_id, count);
						}
						// IMPORTANT: When we connect to a new peer, make sure gossipsub knows about it
						// This helps with topic propagation and mesh formation
						// Add this to all connections, but keep peer itself deduplicated.
						swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
					}
					SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
						if let Some(peer) = peer_id {
							eprintln!("Failed to connect to {}: {}", peer, error);
						} else {
							eprintln!("Failed to connect: {}", error);
						}
					}
					SwarmEvent::Dialing { peer_id, .. } => {
						println!("Attempting to connect to: {:?}", peer_id);
					}
					SwarmEvent::ConnectionClosed { peer_id, .. } => {
						let mut connected_peers = connection.connected_peers.write().await;
						// connected_peers = connected_peers.saturating_sub(1);
						if let Some(count) = connected_peers.get_mut(&peer_id) {
							*count -= 1;
							if *count == 0 {
								connected_peers.remove(&peer_id);
								println!("Peer disconnected: {} ({} peers left)", peer_id, connected_peers.len());
							} else {
								println!("Connection closed: {} ({} remain)", peer_id, count);
							}
						}
					}
					_ => {}
				}
			}
		}
	}

	Ok(())
}
