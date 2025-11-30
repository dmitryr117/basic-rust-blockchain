use crate::{
	blockchain::Blockchain,
	constants::{U32_SIZE, UUID_SIZE},
	traits::BinarySerializable,
	transaction::Transaction,
};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
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

	pub fn update_transaction_pool(
		&mut self,
		transaction_pool: TransactionPool,
	) {
		for (uuid, transaction) in transaction_pool.transaction_map {
			if !self.transaction_map.contains_key(&uuid) {
				self.transaction_map.insert(uuid, transaction);
			}
		}
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

	pub fn get_valid_transactions(&self) -> HashMap<&Uuid, &Transaction> {
		let mut valid_transactions: HashMap<&Uuid, &Transaction> =
			HashMap::new();
		for (uuid, txn) in self.transaction_map.iter() {
			if txn.is_valid() {
				valid_transactions.insert(uuid, txn);
			}
		}
		valid_transactions
	}

	pub fn clear(&mut self) {
		self.transaction_map.clear();
	}

	pub fn clear_blockchain_transactions(&mut self, blockchain: &Blockchain) {
		self.transaction_map.retain(|uuid, _| {
			!blockchain
				.chain
				.iter()
				.rev()
				.any(|block| uuid.to_string() == block.data[0])
		});
	}
}

impl BinarySerializable for TransactionPool {
	fn to_bytes(
		&self,
	) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
		let mut bytes: Vec<u8> = Vec::new();
		for (uuid, txn) in &self.transaction_map {
			bytes.extend(uuid.to_bytes_le());
			let txn_bytes = txn.to_bytes()?;
			bytes.extend((txn_bytes.len() as u32).to_le_bytes());
			bytes.extend(txn_bytes);
		}
		Ok(bytes)
	}

	fn from_bytes(
		bytes: &[u8],
	) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
		let mut transaction_map: HashMap<Uuid, Transaction> = HashMap::new();
		let mut cursor: usize = 0;

		loop {
			if bytes.len() <= cursor {
				break;
			}
			// decodee uuid
			if bytes.len() < cursor + UUID_SIZE {
				return Err("Insufficient bytes for uuid.".into());
			}
			let uuid_bytes: [u8; UUID_SIZE] =
				bytes[cursor..cursor + UUID_SIZE].try_into()?;
			let uuid = Uuid::from_bytes_le(uuid_bytes);

			cursor += UUID_SIZE;

			// transaction size
			if bytes.len() < cursor + U32_SIZE {
				return Err("Insufficient bytes for txn_size.".into());
			}
			let txn_size_bytes: [u8; U32_SIZE] =
				bytes[cursor..cursor + U32_SIZE].try_into()?;
			let txn_size = u32::from_le_bytes(txn_size_bytes);

			cursor += U32_SIZE;

			if bytes.len() < cursor + txn_size as usize {
				return Err("Insufficient bytes for transaction".into());
			}
			let transaction = Transaction::from_bytes(
				&bytes[cursor..cursor + txn_size as usize],
			)?;

			transaction_map.insert(uuid, transaction);

			cursor += txn_size as usize;
		}
		let mut transaction_pool = TransactionPool::new();
		transaction_pool.transaction_map = transaction_map;
		Ok(transaction_pool)
	}
}

#[cfg(test)]
mod test_transaction_pool {
	use libp2p::identity::Keypair;

	use crate::{
		transaction::Transaction, transaction_pool::TransactionPool,
		wallet::Wallet,
	};

	const AMOUNT: u32 = 50;

	fn before_each() -> (TransactionPool, Transaction, Wallet) {
		let transaction_pool = TransactionPool::new();
		let sender_wallet = Wallet::new(&Keypair::generate_ed25519());
		let recipient_wallet = Wallet::new(&Keypair::generate_ed25519());
		let amount: u32 = AMOUNT;
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

	mod get_valid_transactions {
		use crate::utils::output_map_to_bytes;

		use super::*;
		use pretty_assertions::assert_eq;

		fn before_each() -> TransactionPool {
			let (mut transaction_pool, _transaction, sender_wallet) =
				super::before_each();
			for i in 0..10 {
				let recipient_wallet =
					Wallet::new(&Keypair::generate_ed25519());
				let recipient_pk = &recipient_wallet.public_key;
				let amount = 60;
				let mut transaction =
					Transaction::new(&sender_wallet, &recipient_pk, amount);
				if i % 3 == 0 {
					transaction.input.amount = 999999;
				} else if i % 3 == 1 {
					let invalid_wallet =
						Wallet::new(&Keypair::generate_ed25519());
					let output_map_bytes =
						output_map_to_bytes(&transaction.output_map);

					transaction.input.signature = invalid_wallet
						.sign(&output_map_bytes)
						.expect("Unable to sign");
				}
				transaction_pool.set_transaction(transaction);
			}
			transaction_pool
		}

		#[test]
		fn get_valid_transactions() {
			let transaction_pool = before_each();

			let valid_transactions = transaction_pool.get_valid_transactions();

			assert_eq!(valid_transactions.len(), 3);
		}
	}

	mod test_byte_encode_decode {
		use super::*;
		use crate::traits::BinarySerializable;
		use pretty_assertions::assert_eq;

		#[test]
		fn test_encode_decode() {
			let (transaction_pool, _, _) = super::before_each();

			let bytes = transaction_pool.to_bytes().unwrap();
			let decoded = TransactionPool::from_bytes(&bytes).unwrap();

			assert_eq!(transaction_pool, decoded);
		}
	}

	mod test_clear_transactions {
		use std::collections::HashMap;

		use crate::{
			blockchain::{Blockchain, BlockchainTr},
			transaction::Transaction,
			wallet::Wallet,
		};
		use libp2p::identity::Keypair;
		use pretty_assertions::assert_eq;
		use uuid::Uuid;

		#[test]
		fn test_clear_all_transactions() {
			let (mut transaction_pool, _, _) = super::before_each();
			transaction_pool.clear();

			assert_eq!(transaction_pool.transaction_map.len(), 0);
		}

		#[test]
		fn test_clear_blockchain_transactions() {
			let (mut transaction_pool, _, _) = super::before_each();
			let mut blockchain = Blockchain::new();
			let mut expected_txn_map: HashMap<Uuid, Transaction> =
				HashMap::new();

			for i in 0..6 {
				let sender_wallet = Wallet::new(&Keypair::generate_ed25519());
				let recipient_wallet =
					Wallet::new(&Keypair::generate_ed25519());
				let transaction = Transaction::new(
					&sender_wallet,
					&recipient_wallet.public_key,
					50,
				);
				// DR - Need to change later as blockchain will need to record transaction in dirrerent format.
				let txn_uuid = transaction.id;
				transaction_pool.set_transaction(transaction.clone());
				if i % 2 == 0 {
					let data = vec![txn_uuid.to_string()];
					blockchain.add_block(data);
				} else {
					expected_txn_map.insert(txn_uuid, transaction);
				}
			}
			transaction_pool.clear_blockchain_transactions(&blockchain);

			assert_eq!(transaction_pool.transaction_map.len(), 3);
			assert_eq!(expected_txn_map.len(), 3);
			assert_eq!(transaction_pool.transaction_map, expected_txn_map);
		}
	}
}
