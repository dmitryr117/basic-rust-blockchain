use crate::transaction::Transaction;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct TransactionPool {
	pub transaction_map: HashMap<Uuid, Transaction>,
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

	const AMOUNT: usize = 50;

	fn before_each() -> (TransactionPool, Transaction, Wallet) {
		let transaction_pool = TransactionPool::new();
		let sender_wallet = Wallet::new(&Keypair::generate_ed25519());
		let recipient_wallet = Wallet::new(&Keypair::generate_ed25519());
		let amount: usize = AMOUNT;
		let transaction = Transaction::new(
			&sender_wallet,
			&recipient_wallet.public_key,
			amount,
		);

		(transaction_pool, transaction, sender_wallet)
	}

	mod set_transaction {
		use super::*;
		use pretty_assertions::assert_eq;

		#[test]
		fn add_transaction() {
			let (mut transaction_pool, transaction, _) = before_each();

			transaction_pool.set_transaction(transaction.clone());

			let txn = transaction_pool
				.transaction_map
				.get(&transaction.id)
				.unwrap();
			assert_eq!(*txn, transaction);
		}
	}

	mod existing_transaction_mut {
		use super::*;
		use pretty_assertions::assert_eq;

		fn before_each() -> (TransactionPool, Transaction, Wallet) {
			let (mut transaction_pool, transaction, sender_wallet) =
				super::before_each();
			transaction_pool.set_transaction(transaction.clone());
			(transaction_pool, transaction, sender_wallet)
		}

		#[test]
		fn existing_transaction_mut() {
			let (mut transaction_pool, transaction, sender_wallet) =
				before_each();
			let txn = transaction_pool
				.existing_transaction_mut(&sender_wallet.public_key)
				.expect("Transaction should exist, but got None");
			assert_eq!(*txn, transaction)
		}
	}
}
