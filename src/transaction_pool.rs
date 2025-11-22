use crate::transaction::Transaction;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct TransactionPool {
	transaction_map: HashMap<Uuid, Transaction>,
}

impl TransactionPool {
	pub fn new() -> Self {
		Self { transaction_map: HashMap::new() }
	}

	pub fn set_transaction(&mut self, transaction: Transaction) {
		self.transaction_map
			.insert(transaction.id, transaction);
	}

	pub fn existing_transaction_mut(
		&mut self,
		input_address: &Vec<u8>,
	) -> Option<&mut Transaction> {
		let opt = self
			.transaction_map
			.iter_mut()
			.find(|txn| txn.1.input.sender_address == *input_address);
		match opt {
			Some((_, transaction)) => Some(transaction),
			None => None,
		}
	}
}

#[cfg(test)]
mod test_transaction_pool {
	use libp2p::identity::Keypair;

	use crate::{
		transaction::Transaction, transaction_pool::TransactionPool,
		wallet::Wallet,
	};

	mod set_transaction {
		use super::*;
		use pretty_assertions::assert_eq;

		fn before_each() -> (TransactionPool, Transaction) {
			let transaction_pool = TransactionPool::new();
			let sender_wallet = Wallet::new(&Keypair::generate_ed25519());
			let recipient_wallet = Wallet::new(&Keypair::generate_ed25519());
			let amount: usize = 50;
			let transaction = Transaction::new(
				&sender_wallet,
				&recipient_wallet.public_key,
				amount,
			);

			(transaction_pool, transaction)
		}

		#[test]
		fn add_transaction() {
			let (mut transaction_pool, transaction) = before_each();

			transaction_pool.set_transaction(transaction.clone());

			let txn = transaction_pool
				.transaction_map
				.get(&transaction.id)
				.unwrap();
			// wikll have issues here
			assert_eq!(*txn, transaction);
		}
	}
}
