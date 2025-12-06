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
	use cryptochain::{
		blockchain::{Blockchain, BlockchainTr},
		wallet::Wallet,
	};
	use libp2p::identity::Keypair;
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
		match blockchain.replace_chain(new_chain.chain) {
			Ok(_) => panic!("Blockchain was replaced"),
			Err(_) => assert_eq!(blockchain.chain, original_chain),
		}
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

		match blockchain.replace_chain(new_chain.chain) {
			Ok(_) => panic!("Blockchain was replaced"),
			Err(_) => assert_eq!(blockchain.chain, original_chain),
		}
	}

	#[test]
	fn when_chain_is_longer_and_valid_replace() {
		let (mut blockchain, mut new_chain) = before_each();

		let wallet_1 = Wallet::new(&Keypair::generate_ed25519());
		let wallet_2 = Wallet::new(&Keypair::generate_ed25519());

		let txn = Transaction::new(&wallet_1, &wallet_2.public_key, 50);
		// need to chack might be an attack vector.
		new_chain.add_block(vec![txn.clone()]);
		new_chain.add_block(vec![txn.clone()]);
		new_chain.add_block(vec![txn.clone()]);

		blockchain
			.replace_chain(new_chain.chain.clone())
			.expect("Unable to replace chain");
		assert_eq!(blockchain.chain, new_chain.chain);
	}
}

mod test_valid_txn_data {
	use std::collections::BTreeMap;

	use chrono::Utc;
	use cryptochain::{
		blockchain::{Blockchain, BlockchainTr},
		config::REWARD_INPUT_ADDRESS,
		transaction::Transaction,
		utils::output_map_to_bytes,
		wallet::Wallet,
	};
	use libp2p::identity::Keypair;
	use pretty_assertions::assert_eq;

	fn before_each()
	-> (Blockchain, Blockchain, Wallet, Transaction, Transaction) {
		let blockchain = Blockchain::new();
		let new_chain = Blockchain::new();
		let mut wallet = Wallet::new(&Keypair::generate_ed25519());
		let recipient = Wallet::new(&Keypair::generate_ed25519());
		let txn = wallet
			.create_transaction(65, &recipient.public_key, &blockchain)
			.unwrap();
		let reword_txn = Transaction::new_reward_txn(
			&wallet.public_key,
			&REWARD_INPUT_ADDRESS,
			5,
		);

		(blockchain, new_chain, wallet, txn, reword_txn)
	}

	#[test]
	fn test_txn_data_is_valid() {
		let (blockchain, new_chain, _wallet, _txn, _reward_txn) = before_each();
		let valid = blockchain.valid_transaction_data(&new_chain.chain);

		assert_eq!(true, valid);
	}

	#[test]
	fn test_txn_data_has_multiple_rewards() {
		let (blockchain, mut new_chain, _wallet, txn, reward_txn) =
			before_each();
		new_chain.add_block(vec![txn, reward_txn.clone(), reward_txn]);

		let valid = blockchain.valid_transaction_data(&new_chain.chain);

		assert_eq!(false, valid);
	}

	#[test]
	fn test_txn_data_has_malformed_output_map_not_reward() {
		let (blockchain, mut new_chain, wallet, mut txn, reward_txn) =
			before_each();
		txn.output_map.insert(wallet.public_key, 999_999);
		new_chain.add_block(vec![txn, reward_txn]);

		let valid = blockchain.valid_transaction_data(&new_chain.chain);
		assert_eq!(false, valid);
	}

	#[test]
	fn test_txn_data_has_malformed_output_map_is_reward() {
		let (blockchain, mut new_chain, wallet, txn, mut reward_txn) =
			before_each();
		reward_txn
			.output_map
			.insert(wallet.public_key, 999_999);
		new_chain.add_block(vec![txn, reward_txn]);

		let valid = blockchain.valid_transaction_data(&new_chain.chain);

		assert_eq!(false, valid);
	}

	#[test]
	fn test_txn_data_has_malformed_input() {
		let (blockchain, mut new_chain, mut wallet_1, _txn, reward_txn) =
			before_each();
		wallet_1.balance = 9000;
		let wallet_2 = Wallet::new(&Keypair::generate_ed25519());

		let mut evil_map: BTreeMap<Vec<u8>, u32> = BTreeMap::new();
		evil_map.insert(wallet_1.public_key.clone(), 8900);
		evil_map.insert(wallet_2.public_key.clone(), 100);

		let mut evil_txn =
			Transaction::new(&wallet_1, &wallet_2.public_key, 100);
		evil_txn.output_map = evil_map.clone();
		evil_txn.input.amount = wallet_1.balance;
		evil_txn.input.timestamp = Utc::now().timestamp_millis();
		evil_txn.input.sender_address = wallet_1.public_key.clone();
		let output_bytes = output_map_to_bytes(&evil_map);
		evil_txn.input.signature = wallet_1.sign(&output_bytes).unwrap();

		new_chain.add_block(vec![evil_txn, reward_txn]);

		let valid = blockchain.valid_transaction_data(&new_chain.chain);

		assert_eq!(false, valid);
	}

	#[test]
	fn test_txn_data_has_identical_transactions() {
		let (blockchain, mut new_chain, _wallet, txn, reward_txn) =
			before_each();

		new_chain.add_block(vec![
			txn.clone(),
			txn.clone(),
			txn.clone(),
			reward_txn,
		]);

		let valid = blockchain.valid_transaction_data(&new_chain.chain);

		assert_eq!(false, valid);
	}
}
