use cryptochain::p2p_task::start_p2p_task;
/**
 * Testing libp2p communicator singleton class with terminal chat.
 */
use std::sync::Arc;
use tokio::sync::RwLock;

use cryptochain::blockchain::Blockchain;

// Should have initialization script, and continuous event loop.

#[tokio::main]
async fn main() {
	let blockchain = Arc::new(RwLock::new(Blockchain::new()));

	let p2p_handle = start_p2p_task(blockchain.clone());

	tokio::select! {
		// Send blockchain sync event instead of text arg. Need to do some work, and add
		_ = p2p_handle => {},
		_ = tokio::signal::ctrl_c() => {
			println!("Shutting down...");
			std::process::exit(0);
		}
	}

	// std::future::pending::<()>().await;
}
