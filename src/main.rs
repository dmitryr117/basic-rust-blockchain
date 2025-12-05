use cryptochain::channels::create_unbounded_channel;
use cryptochain::transaction_miner::TransactionMiner;
use cryptochain::transaction_pool::TransactionPool;
use cryptochain::wallet::Wallet;
use libp2p::identity::Keypair;
use std::env;
/**
 * Testing libp2p communicator singleton class with terminal chat.
 */
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use cryptochain::blockchain::Blockchain;
use cryptochain::http_server::start_http_server_task;
use cryptochain::p2p_task::start_p2p_task;

// Should have initialization script, and continuous event loop.

#[tokio::main]
async fn main() {
	let args: Vec<String> = env::args().collect();

	let mut port: u32 = 3005;

	if args.len() > 1 {
		port = args[1].parse().expect("Port must be a number.");
	}

	let (event_tx, event_rx) = create_unbounded_channel();
	let event_tx = Arc::new(event_tx);
	let blockchain = Arc::new(RwLock::new(Blockchain::new()));
	let wallet =
		Arc::new(RwLock::new(Wallet::new(&Keypair::generate_ed25519())));
	let transaction_pool = Arc::new(RwLock::new(TransactionPool::new()));
	let transaction_miner = Arc::new(Mutex::new(TransactionMiner::new(
		blockchain.clone(),
		transaction_pool.clone(),
		wallet.clone(),
		event_tx.clone(),
	)));

	let p2p_handle = start_p2p_task(
		blockchain.clone(),
		transaction_pool.clone(),
		transaction_miner.clone(),
		event_tx.clone(),
		event_rx,
	);
	let http_server_handle = start_http_server_task(
		port,
		wallet.clone(),
		blockchain.clone(),
		transaction_pool.clone(),
		event_tx.clone(),
	);

	tokio::select! {
		_ = p2p_handle => {},
		_ = http_server_handle => {},
		_ = tokio::signal::ctrl_c() => {
			println!("Shutting down...");
			std::process::exit(0);
		}
	}

	// std::future::pending::<()>().await;
}
