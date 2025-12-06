use bs58;
use cryptochain::wallet::Wallet;

use cryptochain::config::STARTING_BALANCE;
use libp2p::identity::{Keypair, PublicKey};
use sha3::{Digest, Sha3_256};

use pretty_assertions::assert_eq;

#[test]
fn test_default_balance() {
	let wallet = Wallet::new(&Keypair::generate_ed25519());
	assert_eq!(wallet.balance, STARTING_BALANCE);
}

#[test]
fn test_has_public_key() {
	let keypair = Keypair::generate_ed25519();
	let wallet = Wallet::new(&keypair);

	let pubkey = keypair.public().encode_protobuf();

	assert_eq!(wallet.public_key, pubkey);
}

fn build_address_for_test(public_key: &PublicKey) -> String {
	let pubkey_bytes = public_key.encode_protobuf();
	let mut hasher = Sha3_256::new();
	hasher.update(&pubkey_bytes);
	let hash = hasher.finalize();
	bs58::encode(hash).into_string()
}

#[test]
fn test_derive_address() {
	let keypair = Keypair::generate_ed25519();
	let pubkey = keypair.public();

	let standard = Wallet::derive_address(&keypair);
	let comparator = build_address_for_test(&pubkey);

	println!("Public address: {standard}");

	assert_eq!(standard, comparator);
}

mod test_verify_signature {

	use super::*;
	use pretty_assertions::assert_eq;

	#[test]
	fn test_sign_data_verify_signature() {
		let data = "FooBar".as_bytes();
		let wallet = Wallet::new(&Keypair::generate_ed25519());

		let signed_data = wallet.sign(data).unwrap();

		let verified =
			Wallet::verify_signature(&wallet.public_key, data, &signed_data);
		assert_eq!(verified, true);
	}

	#[test]
	fn test_sign_data_invalid_signature() {
		let data = "FooBar".as_bytes();
		let wallet = Wallet::new(&Keypair::generate_ed25519());
		let wrong_wallet = Wallet::new(&Keypair::generate_ed25519());

		let wrong_signed_data = wrong_wallet.sign(data).unwrap();

		let verified = Wallet::verify_signature(
			&wallet.public_key,
			data,
			&wrong_signed_data,
		);
		assert_eq!(verified, false);
	}
}

mod test_create_transaction {
	use super::*;
	use cryptochain::blockchain::Blockchain;
	use pretty_assertions::assert_eq;

	fn before_each() -> (Blockchain, u32, Wallet, Wallet) {
		let blockchain = Blockchain::new();
		let amount: u32 = 50;
		let recipient = Wallet::new(&Keypair::generate_ed25519());
		let wallet = Wallet::new(&Keypair::generate_ed25519());

		(blockchain, amount, recipient, wallet)
	}

	#[test]
	fn create_transaction_amount_exceeds_balance() {
		let (blockchain, _amount, recipient, mut wallet) = before_each();

		let res = wallet.create_transaction(
			999_999,
			&recipient.public_key,
			&blockchain,
		);
		assert_eq!(res.is_err(), true);
	}

	#[test]
	fn match_transaction_input_with_wallet() {
		let (blockchain, amount, recipient, mut wallet) = before_each();
		let transaction = wallet
			.create_transaction(amount, &recipient.public_key, &blockchain)
			.unwrap();

		assert_eq!(transaction.input.sender_address, wallet.public_key);
	}

	#[test]
	fn output_recipient_amount() {
		let (blockchain, amount, recipient, mut wallet) = before_each();
		let transaction = wallet
			.create_transaction(amount, &recipient.public_key, &blockchain)
			.unwrap();

		let txn_recipient_output_map_value = transaction
			.output_map
			.get(&recipient.public_key)
			.unwrap();

		assert_eq!(*txn_recipient_output_map_value, amount);
	}

	#[test]
	fn chain_is_passed() {}
}

mod test_calculate_balance {
	use super::*;
	use cryptochain::{
		blockchain::{Blockchain, BlockchainTr},
		config::STARTING_BALANCE,
		wallet::Wallet,
	};
	use pretty_assertions::assert_eq;

	fn before_each() -> (Blockchain, Wallet, Wallet, Wallet, Wallet) {
		let blockchain = Blockchain::new();
		let sender_1 = Wallet::new(&Keypair::generate_ed25519());
		let sender_2 = Wallet::new(&Keypair::generate_ed25519());
		let recipient_1 = Wallet::new(&Keypair::generate_ed25519());
		let recipient_2 = Wallet::new(&Keypair::generate_ed25519());

		return (blockchain, sender_1, sender_2, recipient_1, recipient_2);
	}

	#[test]
	fn test_no_outputs_for_wallet() {
		let (blockchain, _sender_1, _sender_2, recipient_1, _recipient_2) =
			before_each();
		let wallet_balance = Wallet::calculate_balance(
			&blockchain.chain,
			&recipient_1.public_key,
		);
		assert_eq!(wallet_balance, STARTING_BALANCE);
	}

	#[test]
	fn test_with_outputs_for_wallet() {
		let (amount_1, amount_2) = (50, 60);

		let (
			mut blockchain,
			mut sender_1,
			mut sender_2,
			recipient_1,
			_recipient_2,
		) = before_each();

		let txn_1 = sender_1
			.create_transaction(amount_1, &recipient_1.public_key, &blockchain)
			.unwrap();
		let txn_2 = sender_2
			.create_transaction(amount_2, &recipient_1.public_key, &blockchain)
			.unwrap();

		blockchain.add_block(vec![txn_1, txn_2]);

		let wallet_balance = Wallet::calculate_balance(
			&blockchain.chain,
			&recipient_1.public_key,
		);

		let expected_wallet_balance = STARTING_BALANCE + amount_1 + amount_2;

		assert_eq!(wallet_balance, expected_wallet_balance);
	}

	mod test_avoid_double_count {
		use super::*;
		use cryptochain::{
			config::REWARD_INPUT_ADDRESS, transaction::Transaction,
		};
		use pretty_assertions::assert_eq;

		#[test]
		fn test_balance_and_wallet_made_transaction() {
			let (
				mut blockchain,
				mut sender_1,
				_sender_2,
				recipient_1,
				_recipient_2,
			) = super::before_each();

			let recent_txn = sender_1
				.create_transaction(50, &recipient_1.public_key, &blockchain)
				.unwrap();

			blockchain.add_block(vec![recent_txn.clone()]);

			let balance = Wallet::calculate_balance(
				&blockchain.chain,
				&sender_1.public_key,
			);

			let expected = recent_txn
				.output_map
				.get(&sender_1.public_key)
				.unwrap();

			assert_eq!(*expected, balance);
		}

		#[test]
		fn test_outputs_next_to_recent() {
			let (
				mut blockchain,
				mut wallet_1,
				mut wallet_2,
				recipient_1,
				_recipient_2,
			) = super::before_each();

			let recent_txn = wallet_1
				.create_transaction(100, &recipient_1.public_key, &blockchain)
				.unwrap();

			let same_block_txn = Transaction::new_reward_txn(
				&wallet_1.public_key,
				&REWARD_INPUT_ADDRESS,
				50,
			);
			blockchain
				.add_block(vec![recent_txn.clone(), same_block_txn.clone()]);

			let next_block_transaction = wallet_2
				.create_transaction(75, &wallet_1.public_key, &blockchain)
				.unwrap();
			blockchain.add_block(vec![next_block_transaction.clone()]);

			let wallet_balance = Wallet::calculate_balance(
				&blockchain.chain,
				&wallet_1.public_key,
			);

			let mut expected_balance = *recent_txn
				.output_map
				.get(&wallet_1.public_key)
				.unwrap();
			expected_balance += *same_block_txn
				.output_map
				.get(&wallet_1.public_key)
				.unwrap();
			expected_balance += *next_block_transaction
				.output_map
				.get(&wallet_1.public_key)
				.unwrap();

			assert_eq!(expected_balance, wallet_balance);
		}
	}
}
