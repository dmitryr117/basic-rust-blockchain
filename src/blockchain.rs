use std::collections::BTreeSet;

use uuid::Uuid;

use crate::{
	block::{Block, BlockTr},
	config::{MINING_REWARD, REWARD_INPUT_ADDRESS},
	traits::BinarySerializable,
	transaction::Transaction,
	utils::cryptohash,
	wallet::Wallet,
};

// Also nbeed to load chain from file system if it exists.
pub trait BlockchainTr {
	fn add_block(&mut self, data: Vec<Transaction>);
	fn replace_chain(&mut self, new_chain: Vec<Block>) -> Result<(), String>;
	fn valid_transaction_data(&self, blockchain: &Blockchain) -> bool;
}

#[derive(Debug, Clone)]
pub struct Blockchain {
	pub chain: Vec<Block>,
}

impl Blockchain {
	pub fn new() -> Self {
		Self { chain: vec![Block::genesis()] }
	}

	pub fn is_valid_chain(chain: &Vec<Block>) -> bool {
		let first_block = chain.first().unwrap();
		let genesis = Block::genesis();

		if *first_block != genesis {
			return false;
		}

		for idx in 0..chain.len() {
			if idx == 0 {
				continue;
			}
			let block = chain.get(idx).unwrap();
			let (timestamp, last_hash, hash, data, nonce, difficulty) = (
				block.timestamp,
				&block.last_hash,
				&block.hash,
				&block.data,
				block.nonce,
				block.difficulty,
			);

			let last_block = chain.get(idx - 1).unwrap();
			let actual_last_hash = &last_block.hash;

			let difficulty_delta = last_block.difficulty.abs_diff(difficulty);

			if actual_last_hash != last_hash || difficulty_delta > 1 {
				return false;
			}

			let last_hash = hex::encode(last_hash);
			let data_bytes = Block::data_to_bytes(data);
			let validated_hash = cryptohash(
				&data_bytes,
				&last_hash,
				timestamp,
				nonce,
				difficulty,
			);

			if *hash != validated_hash {
				return false;
			}
		}
		true
	}
}

impl BlockchainTr for Blockchain {
	fn add_block(&mut self, data: Vec<Transaction>) {
		let last_block = self.chain.last().unwrap();
		let new_block = Block::mine_block(data, last_block);
		self.chain.push(new_block);
	}

	fn replace_chain(&mut self, new_chain: Vec<Block>) -> Result<(), String> {
		if new_chain.len() <= self.chain.len() {
			return Err(format!(
				"New chain too short. Old len: {}, new len: {}",
				self.chain.len(),
				new_chain.len()
			));
		}

		if !Blockchain::is_valid_chain(&new_chain) {
			return Err(format!("New chain is Invalid!"));
		}
		self.chain = new_chain;
		Ok(())
	}

	fn valid_transaction_data(&self, blockchain: &Blockchain) -> bool {
		for (idx, block) in blockchain.chain.iter().enumerate() {
			if idx == 0 {
				continue;
			}
			let mut reward_txn_count = 0;
			let mut txn_set: BTreeSet<Uuid> = BTreeSet::new();

			for txn in &block.data {
				if txn.input.sender_address == REWARD_INPUT_ADDRESS {
					reward_txn_count += 1;
					if reward_txn_count > 1 {
						eprintln!("Miner reward exceeds limit.");
						return false;
					}

					if *txn.output_map.values().next().unwrap() != MINING_REWARD
					{
						eprintln!("Miner reward amount invalid.");
						return false;
					}
				} else {
					if !txn.is_valid() {
						eprintln!("Invalid transaction");
						return false;
					}
					let true_balance = Wallet::calculate_balance(
						&self.chain,
						&txn.input.sender_address,
					);

					println!("TB: {}, {}", true_balance, txn.input.amount);

					if txn.input.amount != true_balance {
						eprintln!("Invalid input amount.");
						return false;
					}

					if let Some(_) = txn_set.get(&txn.id) {
						return false;
					} else {
						txn_set.insert(txn.id);
					}
				}
			}
		}
		true
	}
}

impl BinarySerializable for Blockchain {
	fn to_bytes(
		&self,
	) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
		let config = bincode::config::standard();
		match bincode::serde::encode_to_vec(&self.chain, config) {
			Ok(bytes) => Ok(bytes),
			Err(err) => Err(Box::new(err)),
		}
	}

	fn from_bytes(
		bytes: &[u8],
	) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
		let config = bincode::config::standard();
		match bincode::serde::decode_from_slice(bytes, config) {
			Ok(data) => {
				let chain: Vec<Block> = data.0;
				return Ok(Self { chain });
			}
			Err(err) => Err(Box::new(err)),
		}
	}
}

#[cfg(test)]
mod test_blockchain {
	use crate::config::REWARD_INPUT_ADDRESS;

	use super::*;
	use pretty_assertions::assert_eq;

	#[test]
	fn contains_chain_vec() {
		let blockchain = Blockchain::new();
		assert!(blockchain.chain.len() > 0);
	}

	#[test]
	fn starts_with_genesis() {
		let blockchain = Blockchain::new();
		assert_eq!(*(blockchain.chain.first().unwrap()), Block::genesis());
	}

	#[test]
	fn adds_new_block_to_chain() {
		let new_data = vec![
			Transaction::new_reward_txn(
				&REWARD_INPUT_ADDRESS,
				&REWARD_INPUT_ADDRESS,
				50,
			),
			Transaction::new_reward_txn(
				&REWARD_INPUT_ADDRESS,
				&REWARD_INPUT_ADDRESS,
				50,
			),
		];
		let mut blockchain = Blockchain::new();
		blockchain.add_block(new_data.clone());
		assert_eq!(blockchain.chain.last().unwrap().data, new_data);
	}
}
