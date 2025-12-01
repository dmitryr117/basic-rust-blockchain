use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
	blockchain::{Blockchain, BlockchainTr},
	config::{MINING_REWARD, REWARD_INPUT_ADDRESS},
	transaction::Transaction,
	transaction_pool::TransactionPool,
	wallet::Wallet,
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

	pub async fn mine_transactions(&self) {
		// get valid transactions from txn pool
		let valid_transactions = self
			.transaction_pool
			.read()
			.await
			.get_valid_transactions();

		// generate miners reward
		let miner_pk = &self.wallet.read().await.public_key;
		Transaction::new_reward_txn(
			miner_pk,
			&REWARD_INPUT_ADDRESS,
			MINING_REWARD,
		);

		// add a block to blockchain
		let blockchain = &mut self.blockchain.write().await;
		blockchain.add_block(valid_transactions);

		// broadcast updated blockchain

		// clear the pool
		self.transaction_pool.write().await.clear();
	}
}
