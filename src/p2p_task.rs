/**
 * Testing libp2p communicator singleton class with terminal chat.
 */
use futures::StreamExt;
use libp2p::{gossipsub, mdns, swarm::SwarmEvent};
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::{RwLock, mpsc};
use tokio::task::JoinHandle;
use tokio::time::interval;

use crate::channels::AppEvent;
use crate::traits::BinarySerializable;
use crate::transaction::Transaction;
use crate::transaction_pool::TransactionPool;
use crate::{
	blockchain::{Blockchain, BlockchainTr},
	comms_debounce::Debouncer,
	constants,
	p2p_mdns_bc_coms::{self, P2PBehaviourEvent, TopicEnum},
};

// Should have initialization script, and continuous event loop.
pub fn start_p2p_task(
	blockchain: Arc<RwLock<Blockchain>>,
	transaction_pool: Arc<RwLock<TransactionPool>>,
	mut event_rx: mpsc::UnboundedReceiver<AppEvent>,
) -> JoinHandle<()> {
	tokio::spawn(async move {
		let connection = p2p_mdns_bc_coms::P2PConnection::global().await;
		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

		println!("\nType messages to send. 'exit' to quit.");
		println!("=================\n");

		let mut stdin = io::BufReader::new(io::stdin()).lines();

		let chain_topic = Arc::new(gossipsub::IdentTopic::new(
			TopicEnum::Blockchain.to_string(),
		));
		let txn_topic = Arc::new(gossipsub::IdentTopic::new(
			TopicEnum::Transaction.to_string(),
		));
		let txn_pool_topic = Arc::new(gossipsub::IdentTopic::new(
			TopicEnum::TransactionPool.to_string(),
		));

		let mut heartbeat = interval(Duration::from_millis(100));
		let mut debouncer_brodcast_chain =
			Debouncer::new(Duration::from_secs(10));

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
							match connection.publish(&chain_topic, input.as_bytes()).await {
								Ok(_) => println!("Message sent!"),
								Err(e) => eprintln!("Failed to send: {}", e),
							}
						} else if !input.is_empty() {
							println!("No connected peers yet. Waiting for automatic discovery...");
						}
					}
				},
				event_channel = event_rx.recv() => {
					match event_channel {
						Some(AppEvent::BroadcastMessage(message)) => {
							if message.action == constants::BROADCAST_TXN_POOL {
								let txn_pool = transaction_pool.read().await;
								if let Some(transaction) = txn_pool.transaction_map.get(&message.uuid) {
									if let Ok(encoded_txn) = transaction.to_bytes() {
										match connection.publish(&txn_topic, &encoded_txn).await {
											Ok(_) => println!("Transaction published!"),
											Err(e) => println!("Failed to send: {}", e),
										}
									}
								}
							}
							println!("Message {message:?}")
						}
						_ => {
							continue;
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
										if let Ok(new_chain) = Blockchain::from_bytes(&message.data) {
											blockchain.write().await.replace_chain(new_chain);
										}
									},
									TopicEnum::Transaction => {
										if let Ok(transaction) = Transaction::from_bytes(&message.data) {
											let mut txn_pool = transaction_pool.write().await;
											txn_pool.set_transaction(transaction);
										}
									}
									TopicEnum::TransactionPool => {
										if let Ok(incoming_txn_pool) = TransactionPool::from_bytes(&message.data) {
											let mut txn_pool = transaction_pool.write().await;
											txn_pool.update_transaction_pool(incoming_txn_pool);
										}
									}
								}
							}
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
							debouncer_brodcast_chain.on_event();
						}
						SwarmEvent::ConnectionClosed { peer_id, .. } => {
							connection.closed_connection(&peer_id).await;
						}
						_ => {}
					}
				},
				_ = heartbeat.tick() => {} // unblock timed tasks by heartbeat. other continuous option: tokio::task::yield_now().await;
			}
			if debouncer_brodcast_chain.check() {
				let blockchain_quard = blockchain.read().await;
				if let Ok(bytes_chain) =
					Blockchain::to_bytes(&blockchain_quard.chain)
				{
					match connection
						.publish(&chain_topic, &bytes_chain)
						.await
					{
						Ok(_) => println!("Debounced blockchain published!"),
						Err(e) => println!("Failed to send: {}", e),
					}
				}
				let txn_pool = transaction_pool.read().await;
				if let Ok(bytes_txn_pool) = txn_pool.to_bytes() {
					match connection
						.publish(&txn_pool_topic, &bytes_txn_pool)
						.await
					{
						Ok(_) => {
							println!("Debounced transaction pool published!")
						}
						Err(e) => {
							println!("Failed to send transaction pool: {}", e)
						}
					}
				}
			}
		}
	})
}
