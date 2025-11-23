use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
	blockchain::Blockchain, transaction_pool::TransactionPool, wallet::Wallet,
};

pub struct TransactionMiner {
	pub blockchain: Arc<RwLock<Blockchain>>,
	pub transaction_pool: Arc<RwLock<TransactionPool>>,
	pub wallet: Arc<RwLock<Wallet>>,
	pub pubsub: (), // will be called dirrerent.
}

impl TransactionMiner {
	pub fn new(
		blockchain: Arc<RwLock<Blockchain>>,
		transaction_pool: Arc<RwLock<TransactionPool>>,
		wallet: Arc<RwLock<Wallet>>,
	) -> Self {
		Self { blockchain, transaction_pool, wallet, pubsub: () }
	}

	pub fn mine_transactions() {
		// get valid transactions from txn pool

		// generate miners reward

		// add a block to blockchain

		// broadcast updated blockchain

		// clear the pool
	}
}
