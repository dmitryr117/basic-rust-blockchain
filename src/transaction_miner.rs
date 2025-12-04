use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use crate::{
	blockchain::{Blockchain, BlockchainTr},
	channels::AppEvent,
	config::{MINING_REWARD, REWARD_INPUT_ADDRESS},
	transaction::Transaction,
	transaction_pool::TransactionPool,
	wallet::Wallet,
};

pub struct TransactionMiner {
	pub blockchain: Arc<RwLock<Blockchain>>,
	pub transaction_pool: Arc<RwLock<TransactionPool>>,
	pub wallet: Arc<RwLock<Wallet>>,
	pub event_tx: Arc<mpsc::UnboundedSender<AppEvent>>, // will be called dirrerent.
}

impl TransactionMiner {
	pub fn new(
		blockchain: Arc<RwLock<Blockchain>>,
		transaction_pool: Arc<RwLock<TransactionPool>>,
		wallet: Arc<RwLock<Wallet>>,
		event_tx: Arc<mpsc::UnboundedSender<AppEvent>>,
	) -> Self {
		Self { blockchain, transaction_pool, wallet, event_tx }
	}

	pub async fn mine_transactions(&self) {
		println!("Mine transaction. 01");
		// get valid transactions from txn pool
		let mut valid_transactions = self
			.transaction_pool
			.write()
			.await
			.get_valid_transactions();

		// generate miners reward
		let miner_pk = &self.wallet.read().await.public_key;
		let reward_txn = Transaction::new_reward_txn(
			miner_pk,
			&REWARD_INPUT_ADDRESS,
			MINING_REWARD,
		);
		println!("Mine transaction. 02");
		valid_transactions.push(reward_txn);

		// add a block to blockchain
		let blockchain = &mut self.blockchain.write().await;
		blockchain.add_block(valid_transactions);

		// broadcast updated blockchain
		if let Ok(_) = &self.event_tx.send(AppEvent::ClearTransactionPool) {};
		println!("Mine transaction. 03");
		// clear the pool
		self.transaction_pool.write().await.clear();
	}
}
