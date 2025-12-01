use cryptochain::{
	blockchain::Blockchain, config::REWARD_INPUT_ADDRESS,
	transaction::Transaction,
};

mod is_valid_chain {
	use super::*;
	use pretty_assertions::assert_eq;

	#[test]
	fn when_chain_does_not_start_with_genesis() {
		// is valid chain returns false
		let mut blockchain = Blockchain::new();

		let data = vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			50,
		)];

		if let Some(genesis) = blockchain.chain.first_mut() {
			genesis.data = data;
		}

		assert_eq!(Blockchain::is_valid_chain(&blockchain.chain), false);
	}
}

mod chain_starts_with_genesis_block {
	use super::*;
	use cryptochain::{blockchain::Blockchain, blockchain::BlockchainTr};
	use pretty_assertions::assert_eq;

	fn before_each() -> Blockchain {
		let mut blockchain = Blockchain::new();
		blockchain.add_block(vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			10,
		)]);
		blockchain.add_block(vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			20,
		)]);
		blockchain.add_block(vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			30,
		)]);

		blockchain
	}

	#[test]
	fn and_last_hash_reference_has_changed() {
		// is valid chain returns false
		let mut blockchain = before_each();

		if let Some(block) = blockchain.chain.get_mut(2) {
			block.last_hash = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
		}

		assert_eq!(Blockchain::is_valid_chain(&blockchain.chain), false);
	}

	#[test]
	fn chain_has_block_with_invalid_field() {
		// is valid chain returns false
		let mut blockchain = before_each();

		if let Some(block) = blockchain.chain.get_mut(2) {
			block.data = vec![Transaction::new_reward_txn(
				&REWARD_INPUT_ADDRESS,
				&REWARD_INPUT_ADDRESS,
				101,
			)]
		}

		assert_eq!(Blockchain::is_valid_chain(&blockchain.chain), false);
	}

	#[test]
	fn chain_containe_only_valid_blocks() {
		// is valid chain returns true
		let blockchain = before_each();
		assert_eq!(Blockchain::is_valid_chain(&blockchain.chain), true);
	}
}

mod chain_replacement {
	use super::*;
	use cryptochain::blockchain::{Blockchain, BlockchainTr};
	use pretty_assertions::assert_eq;

	fn before_each() -> (Blockchain, Blockchain) {
		let blockchain = Blockchain::new();
		let new_chain = Blockchain::new();
		(blockchain, new_chain)
	}

	#[test]
	fn when_chain_is_shorter_do_not_replace() {
		let (mut blockchain, mut new_chain) = before_each();
		let original_chain = blockchain.chain.clone();
		if let Some(block) = new_chain.chain.first_mut() {
			block.data = vec![Transaction::new_reward_txn(
				&REWARD_INPUT_ADDRESS,
				&REWARD_INPUT_ADDRESS,
				101,
			)];
		}
		blockchain.replace_chain(new_chain.chain);
		assert_eq!(blockchain.chain, original_chain);
	}

	#[test]
	fn when_chain_is_longer_and_invalid_do_not_replace() {
		let (mut blockchain, mut new_chain) = before_each();

		let original_chain = blockchain.chain.clone();

		new_chain.add_block(vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			10,
		)]);
		new_chain.add_block(vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			20,
		)]);
		new_chain.add_block(vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			30,
		)]);

		// make chain invalid by mutating one of block hashes
		if let Some(block) = new_chain.chain.get_mut(2) {
			block.last_hash = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
		}

		blockchain.replace_chain(new_chain.chain);
		assert_eq!(blockchain.chain, original_chain);
	}

	#[test]
	fn when_chain_is_longer_and_valid_replace() {
		let (mut blockchain, mut new_chain) = before_each();

		new_chain.add_block(vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			10,
		)]);
		new_chain.add_block(vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			20,
		)]);
		new_chain.add_block(vec![Transaction::new_reward_txn(
			&REWARD_INPUT_ADDRESS,
			&REWARD_INPUT_ADDRESS,
			30,
		)]);

		blockchain.replace_chain(new_chain.chain.clone());
		assert_eq!(blockchain.chain, new_chain.chain);
	}
}
