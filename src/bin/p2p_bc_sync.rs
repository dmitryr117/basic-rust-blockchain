/**
 * Testing libp2p communicator singleton class with terminal chat.
 */
use futures::StreamExt;
use libp2p::{gossipsub, mdns, swarm::SwarmEvent};
use std::{error::Error, str::FromStr, sync::Arc, time::Duration};
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;
use tokio::time::interval;

use cryptochain::{
	blockchain::{Blockchain, BlockchainTr},
	comms_debounce::debounce,
	p2p_mdns_bc_coms::{self, P2PBehaviourEvent, TopicEnum},
};

// Should have initialization script, and continuous event loop.

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let blockchain = Arc::new(Mutex::new(Blockchain::new()));
	let connection = p2p_mdns_bc_coms::P2PConnection::global().await;
	tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

	println!("\nType messages to send. 'exit' to quit.");
	println!("=================\n");

	let mut stdin = io::BufReader::new(io::stdin()).lines();
	let topic =
		Arc::new(gossipsub::IdentTopic::new(TopicEnum::Blockchain.to_string()));
	let mut heartbeat = interval(Duration::from_millis(100));

	/*
	 * In loop have to do a few concurrent things.
	 * later also have to refactor all this into separate concurrent async functions inside main application loop.
	 * 1 - get new blocks from network
	 * 2 - post new blocks to network
	 * 3 - get verification of block from network
	 * 4 - post verification of block to network (may include rayon to spawn parallel process for block validation)
	 * 5 - get transactions from network.
	 * 6 - post transactions to network
	 * 7 - get chain replacement from network
	 * 8 - post chain replacement to network
	 * 9 - PoW block mining process with rayon for parallel processing.
	 * 10. - add a global event blocker to prevent from mining anything until required number of blocks has been downloaded into history when the chain is out of sync.
	 */

	let debounced_chain_broadcast = {
		let topic = topic.clone();
		let blockchain = blockchain.clone();
		let connection = connection.clone();

		debounce(Duration::from_millis(10000), move || {
			let topic = topic.clone();
			let blockchain = blockchain.clone();
			let connection = connection.clone();

			async move {
				connection
					.broadcast_chain(&topic, &*blockchain.lock().await)
					.await;
			}
		})
	};

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
						match connection.publish(&topic, input.as_bytes()).await {
							Ok(_) => println!("Message sent!"),
							Err(e) => eprintln!("Failed to send: {}", e),
						}
					} else if !input.is_empty() {
						println!("No connected peers yet. Waiting for automatic discovery...");
					}
				}
			},
			event = async {
				// Only lock swarm long enough to get one event
				let event = {
					let mut swarm = connection.swarm.lock().await;
					swarm.select_next_some().await
				};
				event // Lock released before returning from the async block
			} => {
				match event {
					SwarmEvent::Behaviour(P2PBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
						let topic: &String = &message.topic.to_string();

						if let Ok(topic_enum) = TopicEnum::from_str(&topic) {
							match topic_enum {
								TopicEnum::Blockchain => {
									// chain replacement.
									if let Ok(new_chain) = Blockchain::chain_from_bytes(&message.data) {
										blockchain.lock().await.replace_chain(new_chain);
									}
								},
								_ => {
									println!("Unknown channel message.");
								}
							}
						}
						println!(">! {:#?}", message);
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

						debounced_chain_broadcast();

						// let topic = gossipsub::IdentTopic::new(TopicEnum::Blockchain.to_string());
						// let blockchain_guard = blockchain.lock().await;
						// if let Ok(bytes_chain) = Blockchain::chain_to_bytes(&*blockchain_guard) {
						// 	match connection.publish(&topic, &bytes_chain).await {
						// 		Ok(_) => println!("Debounced blockchain published!"),
						// 		Err(e) => println!("Failed to send: {}", e),
						// 	}
						// }
					}
					SwarmEvent::ConnectionClosed { peer_id, .. } => {
						connection.closed_connection(&peer_id).await;
					}
					_ => {}
				}
			},
			_ = heartbeat.tick() => {}
			// Do nothing, just yield to other tasks
		}
		// tokio::task::yield_now().await;
	}

	Ok(())
}
