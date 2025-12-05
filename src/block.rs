use std::usize;

use crate::{
	config::{
		GENESIS_DIFFICULTY, GENESIS_HASH, GENESIS_LAST_HASH, GENESIS_NONCE,
		GENESIS_TS, MINE_RATE, MINE_RATE_DELTA,
	},
	traits::BinarySerializable,
	transaction::Transaction,
	utils::cryptohash,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub trait BlockTr<T> {
	fn adjust_difficulty(last_block: &T, ms_time: i64) -> u32;
	fn genesis() -> T;
	fn mine_block(data: Vec<Transaction>, last_block: &T) -> T;
	fn data_to_bytes(data: &Vec<Transaction>) -> Vec<u8>;
	fn is_valid_bit_hash(hash: &[u8], difficulty: u32) -> bool;
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Block {
	pub timestamp: i64,
	pub last_hash: Vec<u8>,
	pub hash: Vec<u8>,
	pub data: Vec<Transaction>,
	pub nonce: u32,
	pub difficulty: u32,
}

impl Block {
	pub fn new(
		timestamp: i64,
		last_hash: Vec<u8>,
		hash: Vec<u8>,
		data: Vec<Transaction>,
		nonce: u32,
		difficulty: u32,
	) -> Self {
		Self { timestamp, last_hash, hash, data, nonce, difficulty }
	}
}

impl BlockTr<Block> for Block {
	fn genesis() -> Self {
		let data = vec![];
		Self::new(
			GENESIS_TS,
			GENESIS_LAST_HASH.to_vec(),
			GENESIS_HASH.to_vec(),
			data,
			GENESIS_NONCE,
			GENESIS_DIFFICULTY,
		)
	}

	fn mine_block(data: Vec<Transaction>, last_block: &Block) -> Block {
		let mut ms_time = Utc::now().timestamp_millis();
		let last_hash = hex::encode(&last_block.hash);
		let difficulty: u32 = Self::adjust_difficulty(last_block, ms_time);
		let mut nonce: u32 = 0;
		let mut new_hash: Vec<u8>;

		// transaction vector to bytes
		// let mut txn_bytes: Vec<u8> = Vec::new();
		// data.iter().for_each(|item| {
		// 	if let Ok(data_bytes) = item.to_bytes() {
		// 		txn_bytes.extend(data_bytes);
		// 	}
		// });
		let txn_bytes = Self::data_to_bytes(&data);

		loop {
			nonce += 1;
			ms_time = Utc::now().timestamp_millis();
			new_hash =
				cryptohash(&txn_bytes, &last_hash, ms_time, nonce, difficulty);
			// let sector = new_hash.get(0..difficulty).unwrap();
			// let comparator: Vec<u8> = vec![0; difficulty as usize];

			if Self::is_valid_bit_hash(&new_hash, difficulty) {
				break;
			}
		}
		Self::new(
			ms_time,
			last_block.hash.clone(),
			new_hash,
			data,
			nonce,
			difficulty,
		)
	}

	fn adjust_difficulty(last_block: &Block, ms_time: i64) -> u32 {
		let diff: u32 =
			(last_block.timestamp as isize - ms_time as isize).abs() as u32;
		let mut new_difficulty: u32;
		if diff > MINE_RATE + MINE_RATE_DELTA {
			// decrease difficulty
			new_difficulty = last_block.difficulty - 1;
		} else if diff < MINE_RATE - MINE_RATE_DELTA {
			// increase difficulty
			new_difficulty = last_block.difficulty + 1;
		} else {
			// keep difficulty the same
			new_difficulty = last_block.difficulty;
		}

		// check new difficulty not less then 1
		new_difficulty = if new_difficulty < 1 { 1 } else { new_difficulty };
		new_difficulty
	}

	fn is_valid_bit_hash(hash: &[u8], difficulty: u32) -> bool {
		let full_bytes = difficulty / 8;
		let bits = difficulty % 8;

		// check full zero bytes
		if hash
			.iter()
			.take(full_bytes as usize)
			.any(|&b| b != 0)
		{
			return false;
		}

		if bits > 0 {
			if let Some(&byte) = hash.get(full_bytes as usize) {
				let mask = 0xFFu8 << (8 - bits);
				if byte & mask != 0 {
					return false;
				}
			}
		}

		true
	}

	fn data_to_bytes(data: &Vec<Transaction>) -> Vec<u8> {
		let mut txn_bytes: Vec<u8> = Vec::new();
		data.into_iter().for_each(|item| {
			if let Ok(data_bytes) = item.to_bytes() {
				txn_bytes.extend(data_bytes);
			}
		});
		txn_bytes
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		blockchain::Blockchain,
		config::{GENESIS_NONCE, MINE_RATE, REWARD_INPUT_ADDRESS},
	};
	use pretty_assertions::assert_eq;

	#[test]
	fn test_new_block() {
		let timestamp = 1234;
		let last_hash = vec![1, 2, 3, 4];
		let hash = vec![1, 2, 3, 4];
		let data = vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			50,
		)];

		let new_block = Block::new(
			timestamp,
			last_hash.clone(),
			hash.clone(),
			data.clone(),
			1,
			1,
		);

		let comp_block =
			Block { timestamp, last_hash, hash, data, nonce: 1, difficulty: 1 };

		assert_eq!(new_block, comp_block);
	}

	#[test]
	fn test_genesis() {
		let genesis_block = Block::genesis();

		let genesis_data = vec![];
		let comp_block = Block {
			timestamp: GENESIS_TS,
			last_hash: GENESIS_LAST_HASH.to_vec(),
			hash: GENESIS_HASH.to_vec(),
			data: genesis_data,
			nonce: GENESIS_NONCE,
			difficulty: GENESIS_DIFFICULTY,
		};

		assert_eq!(genesis_block, comp_block);
	}

	fn init_mined_block() -> (Block, Block, Vec<Transaction>) {
		let last_block = Block::genesis();
		let data = vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			50,
		)];
		let mined_block = Block::mine_block(data.clone(), &last_block);
		(last_block, mined_block, data)
	}

	#[test]
	fn test_mine_block() {
		let (last_block, mined_block, data) = init_mined_block();

		assert_eq!(last_block.hash, mined_block.last_hash);
		assert_eq!(data, mined_block.data);
	}

	#[test]
	fn hash_matches_difficulty() {
		let (_, mined_block, _) = init_mined_block();

		// -------------- Need to adjust this to work with bit zeros instead of byte zeros.
		let difficulty = mined_block.difficulty;
		// let sector = mined_block.hash.get(0..difficulty).unwrap();
		// let comparator: Vec<u8> = vec![0; difficulty as usize];
		let is_valid = Block::is_valid_bit_hash(&mined_block.hash, difficulty);

		// assert_eq!(sector, comparator);
		assert_eq!(is_valid, true);
	}

	#[test]
	fn test_black_data_sorting() {
		let mut data = vec!["bcd", "cdf", "abc"];
		data.sort();
		let data: Vec<String> =
			data.iter().map(|item| item.to_string()).collect();
		let expected =
			vec![String::from("abc"), String::from("bcd"), String::from("cdf")];
		assert_eq!(expected, data);
	}

	#[test]
	fn increase_difficulty_if_mined_too_fast() {
		let (_, mined_block, _) = init_mined_block();
		let ms_time = mined_block.timestamp + MINE_RATE as i64 - 100;
		let new_difficulty = Block::adjust_difficulty(&mined_block, ms_time);
		assert_eq!(new_difficulty, mined_block.difficulty + 1);
	}

	#[test]
	fn decrease_difficulty_if_mined_too_slow() {
		let (_, mut mined_block, _) = init_mined_block();
		let ms_time = mined_block.timestamp + MINE_RATE as i64 + 100;
		mined_block.difficulty = 2; // make sure that last block is > 1 for test
		let new_difficulty = Block::adjust_difficulty(&mined_block, ms_time);
		assert_eq!(new_difficulty, mined_block.difficulty - 1);
	}

	#[test]
	fn adjust_difficulty_low_limit() {
		let (_, mut mined_block, _) = init_mined_block();
		let ms_time = mined_block.timestamp + MINE_RATE as i64 + 100;
		mined_block.difficulty = 1; // make sure that last block is > 1 for test
		let new_difficulty = Block::adjust_difficulty(&mined_block, ms_time);
		assert_eq!(new_difficulty, 1);
	}

	#[test]
	fn valid_bit_hash_false_full_bytes() {
		let hash: i32 = 0x00ffffff;
		let bytes = hash.to_be_bytes();
		let is_valid = Block::is_valid_bit_hash(&bytes, 10);
		assert_eq!(is_valid, false);
	}

	#[test]
	fn valid_bit_hash_true_full_bytes() {
		let hash: i32 = 0x0000ffff;
		let bytes = hash.to_be_bytes();
		let is_valid = Block::is_valid_bit_hash(&bytes, 16);
		assert_eq!(is_valid, true);
	}

	#[test]
	fn valid_bit_hash_partial_false() {
		let hash: i32 = 0x0002ffff;
		let bytes = hash.to_be_bytes();
		let is_valid = Block::is_valid_bit_hash(&bytes, 15);
		assert_eq!(is_valid, false);
	}

	#[test]
	fn valid_bit_hash_partial_true() {
		let hash: i32 = 0x0001ffff;
		let bytes = hash.to_be_bytes();
		let is_valid = Block::is_valid_bit_hash(&bytes, 15);
		assert_eq!(is_valid, true);
	}

	#[test]
	fn jumped_difficulty() {
		let mut blockchain = Blockchain::new();

		let (genesis_block, _, _) = init_mined_block();

		let last_hash_hex = hex::encode(&genesis_block.hash);
		let timestamp = Utc::now().timestamp_millis() as i64;
		let data = vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			50,
		)];
		let nonce = 0;
		let difficulty = 1;

		let data_bytes = Block::data_to_bytes(&data);

		let new_hash = cryptohash(
			&data_bytes,
			&last_hash_hex,
			timestamp,
			nonce,
			difficulty,
		);
		let bad_block = Block::new(
			timestamp,
			genesis_block.hash.clone(),
			new_hash,
			data,
			nonce,
			difficulty,
		);
		blockchain.chain.push(bad_block);

		let is_valid = Blockchain::is_valid_chain(&blockchain.chain);
		assert_eq!(is_valid, false);
	}
}
