/**
 * Testing libp2p communicator singleton class with terminal chat.
 */
use futures::StreamExt;
use libp2p::{gossipsub, mdns, swarm::SwarmEvent};
use std::{error::Error, str::FromStr};
use tokio::io::{self, AsyncBufReadExt};

use cryptochain::p2p_mdns_bc_coms::{self, P2PBehaviourEvent, TopicEnum};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let connection = p2p_mdns_bc_coms::P2PConnection::global().await;
	tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

	println!("\nType messages to send. 'exit' to quit.");
	println!("=================\n");

	let mut stdin = io::BufReader::new(io::stdin()).lines();
	let topic = gossipsub::IdentTopic::new(TopicEnum::Blockchain.to_string());

	loop {
		tokio::select! {
			// Send blockchain sync event instead of text arg. Need to do some work, and add
			line = stdin.next_line() => {
				if let Ok(Some(input)) = line {
					let input = input.trim();
					if input == "exit" {
						break;
					}
					if connection.get_connected_peers_len().await > 0 && !input.is_empty() {
						match connection.publish(topic.clone(), input.as_bytes()).await {
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
					SwarmEvent::NewListenAddr { address, .. } => {
						println!("* Listening on: {}", address);
					}
					SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
						if let Some(peer) = peer_id {
							eprintln!("Failed to connect to {}: {}", peer, error);
						} else {
							eprintln!("Failed to connect: {}", error);
						}
					}
					SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
						connection.dial_discovered_peers(list).await;
					}
					SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
						connection.remove_peer(list).await;
					}
					SwarmEvent::ConnectionEstablished { peer_id, .. } => {
						connection.add_connected_peer(&peer_id).await;
					}
					SwarmEvent::ConnectionClosed { peer_id, .. } => {
						connection.closed_connection(&peer_id).await;
					}
					_ => {}
				}
			}
		}
	}

	Ok(())
}
