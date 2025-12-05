/**
 * Testing libp2p communicator singleton class with terminal chat.
 */
use futures::StreamExt;
use libp2p::{gossipsub, mdns, swarm::SwarmEvent};
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::task::JoinHandle;
use tokio::time::interval;
use uuid::Uuid;

use crate::channels::AppEvent;
use crate::traits::BinarySerializable;
use crate::transaction::Transaction;
use crate::transaction_miner::TransactionMiner;
use crate::transaction_pool::TransactionPool;
use crate::{
	blockchain::{Blockchain, BlockchainTr},
	comms_debounce::Debouncer,
	p2p_mdns_bc_coms::{self, P2PBehaviourEvent, TopicEnum},
};

// Should have initialization script, and continuous event loop.
pub fn start_p2p_task(
	blockchain: Arc<RwLock<Blockchain>>,
	transaction_pool: Arc<RwLock<TransactionPool>>,
	transaction_miner: Arc<Mutex<TransactionMiner>>,
	_event_tx: Arc<mpsc::UnboundedSender<AppEvent>>,
	mut event_rx: mpsc::UnboundedReceiver<AppEvent>,
) -> JoinHandle<()> {
	tokio::spawn(async move {
		let connection = p2p_mdns_bc_coms::P2PConnection::global().await;
		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

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
						Some(AppEvent::BroadcastTransaction(data)) => {
							let txn_pool = transaction_pool.read().await;
							if let Ok(arr) = data.as_slice().try_into() {
								let uuid = Uuid::from_bytes_le(arr);
								if let Some(transaction) = txn_pool.transaction_map.get(&uuid) {
									if let Ok(txn_bytes) = transaction.to_bytes() {
										connection.broadcast(&txn_topic, Some(txn_bytes), "Published transaction.", "Cannot publish transaction").await;
									}
								}
							}
						}
						Some(AppEvent::BroadcastChain) => {
							let blockchain_read = blockchain.read().await;
							if let Ok(chain_bytes) = blockchain_read.to_bytes() {
								connection
									.broadcast(
										&chain_topic,
										Some(chain_bytes),
										"Blockchain sent.",
										"Blockchain send failed",
									)
									.await;
							}
						}
						Some(AppEvent::MineTransactions) => {
							let miner = transaction_miner.lock().await;
							miner.mine_transactions().await;
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
											let mut blockchain_write = blockchain.write().await;
											match blockchain_write.replace_chain(new_chain.chain) {
												Ok(()) => {
													// Do transaction cleanup
													let mut transaction_pool_write = transaction_pool.write().await;
													transaction_pool_write.clear_blockchain_transactions(&blockchain_write);
												},
												Err(err) => {
													println!("Failed to replace chain. {}", err);
												}
											}
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
				let blockchain_read = blockchain.read().await;
				let transaction_pool_read = transaction_pool.read().await;

				if let Ok(blockchain_bytes) = blockchain_read.to_bytes() {
					connection
						.broadcast(
							&chain_topic,
							Some(blockchain_bytes),
							"Blockchain sent.",
							"Blockchain send failed",
						)
						.await;
				}

				if let Ok(transaction_pool_bytes) =
					transaction_pool_read.to_bytes()
				{
					connection
						.broadcast(
							&txn_pool_topic,
							Some(transaction_pool_bytes),
							"Blockchain sent.",
							"Blockchain send failed",
						)
						.await;
				}
			}
		}
	})
}
