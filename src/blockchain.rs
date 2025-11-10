use crate::{
	block::{Block, BlockTr},
	crypto_hash::cryptohash,
};

// Also nbeed to load chain from file system if it exists.
pub trait BlockchainTr {
	fn add_block(&mut self, data: Vec<String>);
	fn replace_chain(&mut self, new_chain: Vec<Block>);
	fn chain_to_bytes(
		chain: &Vec<Block>,
	) -> Result<Vec<u8>, bincode::error::EncodeError>;
	fn chain_from_bytes(
		bytes: &[u8],
	) -> Result<Vec<Block>, bincode::error::DecodeError>;
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
			let validated_hash =
				cryptohash(data, &last_hash, timestamp, nonce, difficulty);

			if *hash != validated_hash {
				return false;
			}
		}
		true
	}
}

impl BlockchainTr for Blockchain {
	fn add_block(&mut self, data: Vec<String>) {
		let last_block = self.chain.last().unwrap();
		let new_block = Block::mine_block(data, last_block);
		self.chain.push(new_block);
	}

	fn replace_chain(&mut self, new_chain: Vec<Block>) {
		if new_chain.len() < self.chain.len() {
			eprintln!(
				"New chain too short. Old len: {}, new len: {}",
				self.chain.len(),
				new_chain.len()
			);
			return;
		}

		if !Blockchain::is_valid_chain(&new_chain) {
			eprintln!("New chain in Invalid!");
			return;
		}
		self.chain = new_chain;
	}

	fn chain_to_bytes(
		chain: &Vec<Block>,
	) -> Result<Vec<u8>, bincode::error::EncodeError> {
		let config = bincode::config::standard();
		bincode::encode_to_vec(chain, config)
	}

	fn chain_from_bytes(
		bytes: &[u8],
	) -> Result<Vec<Block>, bincode::error::DecodeError> {
		let config = bincode::config::standard();
		let (chain, _bytes_read) = bincode::decode_from_slice(bytes, config)?;
		Ok(chain)
	}
}

#[cfg(test)]
mod test_blockchain {
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
		let new_data = vec![String::from("foo"), String::from("bar")];
		let mut blockchain = Blockchain::new();
		blockchain.add_block(new_data.clone());
		assert_eq!(blockchain.chain.last().unwrap().data, new_data);
	}
}
