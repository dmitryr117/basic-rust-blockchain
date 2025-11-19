use libp2p::identity::Keypair;
use std::sync::Arc;
use tokio::sync::RwLock;

use cryptochain::blockchain::Blockchain;
use cryptochain::http_server::start_http_server_task;
use cryptochain::p2p_task::start_p2p_task;
use cryptochain::transaction_pool::TransactionPool;
use cryptochain::wallet::Wallet;

/**
 * Testing libp2p communicator singleton class with terminal chat.
 */

// Should have initialization script, and continuous event loop.

#[tokio::main]
async fn main() {
	let blockchain = Arc::new(RwLock::new(Blockchain::new()));
	let wallet =
		Arc::new(RwLock::new(Wallet::new(&Keypair::generate_ed25519())));
	let transaction_pool = Arc::new(RwLock::new(TransactionPool::new()));

	let p2p_handle =
		start_p2p_task(blockchain.clone(), transaction_pool.clone());
	let http_server_handle =
		start_http_server_task(wallet.clone(), transaction_pool.clone());

	tokio::select! {
		// Send blockchain sync event instead of text arg. Need to do some work, and add
		_ = p2p_handle => {},
		_ = http_server_handle => {},
		_ = tokio::signal::ctrl_c() => {
			println!("Shutting down...");
			std::process::exit(0);
		}
	}

	// std::future::pending::<()>().await;
}
