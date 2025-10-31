use std::usize;

use crate::{
	config::{
		GENESIS_DATA, GENESIS_DIFFICULTY, GENESIS_HASH, GENESIS_LAST_HASH, GENESIS_NONCE,
		GENESIS_TS, MINE_RATE, MINE_RATE_DELTA,
	},
	crypto_hash::cryptohash,
};
use chrono::Utc;

pub trait BlockTr<T> {
	fn adjust_difficulty(last_block: &T, ms_time: usize) -> usize;
	fn genesis() -> T;
	fn mine_block(data: Vec<String>, last_block: &T) -> T;
	fn is_valid_bit_hash(hash: &[u8], difficulty: usize) -> bool;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
	pub timestamp: usize,
	pub last_hash: Vec<u8>,
	pub hash: Vec<u8>,
	pub data: Vec<String>,
	pub nonce: usize,
	pub difficulty: usize,
}

impl Block {
	pub fn new(
		timestamp: usize,
		last_hash: Vec<u8>,
		hash: Vec<u8>,
		data: Vec<String>,
		nonce: usize,
		difficulty: usize,
	) -> Self {
		Self {
			timestamp,
			last_hash,
			hash,
			data,
			nonce,
			difficulty,
		}
	}
}

impl BlockTr<Block> for Block {
	fn genesis() -> Self {
		let data = GENESIS_DATA.iter().map(|item| item.to_string()).collect();
		Self::new(
			GENESIS_TS,
			GENESIS_LAST_HASH.to_vec(),
			GENESIS_HASH.to_vec(),
			data,
			GENESIS_NONCE,
			GENESIS_DIFFICULTY,
		)
	}

	fn mine_block(data: Vec<String>, last_block: &Block) -> Block {
		let mut ms_time = Utc::now().timestamp_millis() as usize;
		let last_hash = hex::encode(&last_block.hash);
		let difficulty = Self::adjust_difficulty(last_block, ms_time);
		let mut nonce = 0;
		let mut new_hash: Vec<u8>;

		loop {
			nonce += 1;
			ms_time = Utc::now().timestamp_millis() as usize;
			new_hash = cryptohash(&data, &last_hash, ms_time, nonce, difficulty);
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

	fn adjust_difficulty(last_block: &Block, ms_time: usize) -> usize {
		let diff = (last_block.timestamp as isize - ms_time as isize).abs() as usize;
		let mut new_difficulty: usize;
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
		new_difficulty = if new_difficulty < 1 {
			1
		} else {
			new_difficulty
		};
		new_difficulty
	}

	fn is_valid_bit_hash(hash: &[u8], difficulty: usize) -> bool {
		let full_bytes = difficulty / 8;
		let bits = difficulty % 8;

		// check full zero bytes
		if hash.iter().take(full_bytes).any(|&b| b != 0) {
			return false;
		}

		if bits > 0 {
			if let Some(&byte) = hash.get(full_bytes) {
				let mask = 0xFFu8 << (8 - bits);
				if byte & mask != 0 {
					return false;
				}
			}
		}

		true
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{blockchain::{Blockchain}, config::{GENESIS_NONCE, MINE_RATE}};
	use pretty_assertions::assert_eq;

	#[test]
	fn test_new_block() {
		let timestamp = 1234;
		let last_hash = vec![1, 2, 3, 4];
		let hash = vec![1, 2, 3, 4];
		let data = vec![String::from("data")];

		let new_block = Block::new(
			timestamp,
			last_hash.clone(),
			hash.clone(),
			data.clone(),
			1,
			1,
		);

		let comp_block = Block {
			timestamp,
			last_hash,
			hash,
			data,
			nonce: 1,
			difficulty: 1,
		};

		assert_eq!(new_block, comp_block);
	}

	#[test]
	fn test_genesis() {
		let genesis_block = Block::genesis();

		let genesis_data = GENESIS_DATA.iter().map(|item| item.to_string()).collect();
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

	fn init_mined_block() -> (Block, Block) {
		let last_block = Block::genesis();
		let data = vec![String::from("Mined Data")];
		let mined_block = Block::mine_block(data, &last_block);
		(last_block, mined_block)
	}

	#[test]
	fn test_mine_block() {
		let (last_block, mined_block) = init_mined_block();

		assert_eq!(last_block.hash, mined_block.last_hash);
		assert_eq!(vec![String::from("Mined Data")], mined_block.data);
	}

	#[test]
	fn hash_matches_difficulty() {
		let (_, mined_block) = init_mined_block();

		// -------------- Need to adjust this to work with bit zeros instead of byte zeros.
		let difficulty = mined_block.difficulty as usize;
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
		let data: Vec<String> = data.iter().map(|item| item.to_string()).collect();
		let expected = vec![
			String::from("abc"),
			String::from("bcd"),
			String::from("cdf"),
		];
		assert_eq!(expected, data);
	}

	#[test]
	fn increase_difficulty_if_mined_too_fast() {
		let (_, mined_block) = init_mined_block();
		let ms_time = mined_block.timestamp + MINE_RATE - 100;
		let new_difficulty = Block::adjust_difficulty(&mined_block, ms_time);
		assert_eq!(new_difficulty, mined_block.difficulty + 1);
	}

	#[test]
	fn decrease_difficulty_if_mined_too_slow() {
		let (_, mut mined_block) = init_mined_block();
		let ms_time = mined_block.timestamp + MINE_RATE + 100;
		mined_block.difficulty = 2; // make sure that last block is > 1 for test
		let new_difficulty = Block::adjust_difficulty(&mined_block, ms_time);
		assert_eq!(new_difficulty, mined_block.difficulty - 1);
	}

	#[test]
	fn adjust_difficulty_low_limit() {
		let (_, mut mined_block) = init_mined_block();
		let ms_time = mined_block.timestamp + MINE_RATE + 100;
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

		let (genesis_block, _) = init_mined_block();

		let last_hash_hex = hex::encode(&genesis_block.hash);
		let timestamp = Utc::now().timestamp_millis() as usize;
		let data: Vec<String> = vec![String::from("test")];
		let nonce = 0;
		let difficulty = 1;

		let new_hash = cryptohash(&data, &last_hash_hex, timestamp, nonce, difficulty);
		let bad_block = Block::new(timestamp, genesis_block.hash.clone(), new_hash, data, nonce, difficulty);
		blockchain.chain.push(bad_block);

		let is_valid = Blockchain::is_valid_chain(&blockchain.chain);
		assert_eq!(is_valid, false);
	}
}
