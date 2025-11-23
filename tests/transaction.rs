mod transaction_tests {
	use cryptochain::{
		transaction::Transaction, utils::output_map_to_bytes, wallet::Wallet,
	};
	use libp2p::identity::Keypair;
	use pretty_assertions::assert_eq;

	fn before_each() -> (Wallet, Wallet, u32) {
		let sender_wallet = Wallet::new(&Keypair::generate_ed25519());
		let recipient_wallet = Wallet::new(&Keypair::generate_ed25519());
		let amount: u32 = 50;

		(sender_wallet, recipient_wallet, amount)
	}

	#[test]
	fn test_has_generate_txn_id() {
		let txn_id = Transaction::generate_uuid_v1();
		let txn_id_bytes = txn_id.into_bytes();

		assert!(txn_id_bytes.len() == 16);
	}

	#[test]
	fn test_has_txn_id() {
		let (sender_wallet, recipient_wallet, amount) = before_each();
		let transaction = Transaction::new(
			&sender_wallet,
			&recipient_wallet.public_key,
			amount,
		);

		let txn_id_bytes = &transaction.id.into_bytes();

		assert!(txn_id_bytes.len() == 16);
	}

	#[test]
	fn output_amount_to_recipient() {
		let (sender_wallet, recipient_wallet, amount) = before_each();
		let transaction = Transaction::new(
			&sender_wallet,
			&recipient_wallet.public_key,
			amount,
		);

		// let recipient_amount_comparator = recipient_wallet.balance + amount;

		let txn_value = transaction
			.output_map
			.get(&recipient_wallet.public_key)
			.unwrap();

		assert_eq!(*txn_value, amount);
	}

	#[test]
	fn output_amount_to_sender() {
		let (sender_wallet, recipient_wallet, amount) = before_each();
		let transaction = Transaction::new(
			&sender_wallet,
			&recipient_wallet.public_key,
			amount,
		);

		let sender_amount_comparator = sender_wallet.balance - amount;

		let txn_value = transaction
			.output_map
			.get(&sender_wallet.public_key)
			.unwrap();

		assert_eq!(*txn_value, sender_amount_comparator);
	}

	#[test]
	fn sets_sender_wallet_balance() {
		let (sender_wallet, recipient_wallet, amount) = before_each();
		let sender_amount_pre = sender_wallet.balance;
		let transaction = Transaction::new(
			&sender_wallet,
			&recipient_wallet.public_key,
			amount,
		);

		let sender_amount = transaction
			.output_map
			.get(&sender_wallet.public_key)
			.unwrap();

		let sender_remaining = sender_amount_pre - amount;
		assert_eq!(sender_remaining, *sender_amount)
	}

	#[test]
	fn sets_address_to_sender_pk() {
		let (sender_wallet, recipient_wallet, amount) = before_each();
		let transaction = Transaction::new(
			&sender_wallet,
			&recipient_wallet.public_key,
			amount,
		);
		assert_eq!(transaction.input.sender_address, sender_wallet.public_key)
	}

	#[test]
	fn signs_the_input() {
		let (sender_wallet, recipient_wallet, amount) = before_each();
		let transaction = Transaction::new(
			&sender_wallet,
			&recipient_wallet.public_key,
			amount,
		);

		let output_map = transaction.output_map;
		let output_bytes = output_map_to_bytes(&output_map);
		let signature = sender_wallet
			.sign(&output_bytes)
			.expect("Failed to generate signature.");

		assert_eq!(
			Wallet::verify_signature(
				&sender_wallet.public_key,
				&output_bytes,
				&signature
			),
			true
		)
	}

	mod test_is_valid {
		use super::*;
		use pretty_assertions::assert_eq;

		#[test]
		fn transaction_is_valid() {
			let (sender_wallet, recipient_wallet, amount) = before_each();
			let transaction = Transaction::new(
				&sender_wallet,
				&recipient_wallet.public_key,
				amount,
			);

			assert_eq!(transaction.is_valid(), true);
		}

		#[test]
		fn transaction_invalid_hashmap() {
			let (sender_wallet, recipient_wallet, amount) = before_each();

			let mut transaction = Transaction::new(
				&sender_wallet,
				&recipient_wallet.public_key,
				amount,
			);

			transaction
				.output_map
				.insert(sender_wallet.public_key, 999999);

			assert_eq!(transaction.is_valid(), false);
		}

		#[test]
		fn transaction_invalid_signature() {
			let (sender_wallet, recipient_wallet, amount) = before_each();

			let mut transaction = Transaction::new(
				&sender_wallet,
				&recipient_wallet.public_key,
				amount,
			);

			let wallet = Wallet::new(&Keypair::generate_ed25519());
			let output_bytes = output_map_to_bytes(&transaction.output_map);

			transaction.input.signature = wallet.sign(&output_bytes).unwrap();

			assert_eq!(transaction.is_valid(), false);
		}
	}

	mod test_update_amount_valid {
		use super::*;
		use pretty_assertions::assert_eq;

		fn before_each()
		-> (Wallet, Wallet, Transaction, Vec<u8>, Vec<u8>, u32, u32, u32) {
			let (sender_wallet, recipient_wallet, amount) =
				super::before_each();
			let mut transaction = Transaction::new(
				&sender_wallet,
				&recipient_wallet.public_key,
				amount,
			);

			let original_signature = transaction.input.signature.clone();
			let original_sender_output = *transaction
				.output_map
				.get(&sender_wallet.public_key)
				.unwrap();

			let next_recipient =
				Wallet::new(&Keypair::generate_ed25519()).public_key;
			let next_amount: u32 = 80;

			transaction
				.update(&sender_wallet, &next_recipient, next_amount)
				.unwrap();

			(
				sender_wallet,
				recipient_wallet,
				transaction,
				original_signature,
				next_recipient,
				amount,
				original_sender_output,
				next_amount,
			)
		}

		#[test]
		fn outputs_amount_to_next_recipient() {
			let (
				_sender_wallet,
				_recipient_wallet,
				transaction,
				_original_signature,
				next_recipient,
				_amount,
				_original_sender_output,
				next_amount,
			) = before_each();

			let next_recipient_amount = transaction
				.output_map
				.get(&next_recipient)
				.unwrap();

			assert_eq!(*next_recipient_amount, next_amount);
		}

		#[test]
		fn subtracts_from_sender_output_amount() {
			let (
				sender_wallet,
				_recipient_wallet,
				transaction,
				_original_signature,
				_next_recipient,
				_amount,
				original_sender_output,
				next_amount,
			) = before_each();

			let next_recipient_amount = transaction
				.output_map
				.get(&sender_wallet.public_key)
				.unwrap();

			assert_eq!(
				*next_recipient_amount,
				original_sender_output - next_amount
			);
		}

		#[test]
		fn maintains_total_output_matching_input() {
			let (
				_sender_wallet,
				_recipient_wallet,
				transaction,
				_original_signature,
				_next_recipient,
				_amount,
				_original_sender_output,
				_next_amount,
			) = before_each();

			let amount_sum: u32 = transaction.output_map.values().sum();

			assert_eq!(amount_sum, transaction.input.amount);
		}

		#[test]
		fn resigns_transaction() {
			let (
				_sender_wallet,
				_recipient_wallet,
				transaction,
				original_signature,
				_next_recipient,
				_amount,
				_original_sender_output,
				_next_amount,
			) = before_each();

			assert_ne!(transaction.input.signature, original_signature);
		}
	}

	mod test_update_amount_invalid {
		use super::*;
		use pretty_assertions::assert_eq;

		#[test]
		fn returns_error() {
			let (sender_wallet, recipient_wallet, amount) =
				super::before_each();
			let mut transaction = Transaction::new(
				&sender_wallet,
				&recipient_wallet.public_key,
				amount,
			);

			let next_recipient =
				Wallet::new(&Keypair::generate_ed25519()).public_key;
			let next_amount: u32 = 999999;

			let res = transaction.update(
				&sender_wallet,
				&next_recipient,
				next_amount,
			);

			assert_eq!(res.is_err(), true);
		}

		#[test]
		fn adds_amount_for_same_recipient() {
			let (sender_wallet, recipient_wallet, amount) =
				super::before_each();
			let mut transaction = Transaction::new(
				&sender_wallet,
				&recipient_wallet.public_key,
				amount,
			);

			let next_recipient = &recipient_wallet.public_key;
			let next_amount: u32 = 100;

			let first_recipient_amount = *transaction
				.output_map
				.get(next_recipient)
				.unwrap();

			let first_sender_amount = *transaction
				.output_map
				.get(&sender_wallet.public_key)
				.unwrap();

			transaction
				.update(&sender_wallet, &next_recipient, next_amount)
				.unwrap();

			let total_recipient_amount = *transaction
				.output_map
				.get(next_recipient)
				.unwrap();

			let total_sender_amount = *transaction
				.output_map
				.get(&sender_wallet.public_key)
				.unwrap();

			assert_eq!(
				total_recipient_amount,
				first_recipient_amount + next_amount
			);

			assert_eq!(total_sender_amount, first_sender_amount - next_amount);
		}
	}

	mod test_byte_encode_decode {
		use super::*;
		use cryptochain::traits::BinarySerializable;
		use pretty_assertions::assert_eq;

		#[test]
		fn test_encode_decode() {
			let (sender_wallet, recipient_wallet, amount) =
				super::before_each();
			let transaction = Transaction::new(
				&sender_wallet,
				&recipient_wallet.public_key,
				amount,
			);

			let bytes = transaction.to_bytes().unwrap();
			let decoded = Transaction::from_bytes(&bytes).unwrap();

			assert_eq!(transaction, decoded);
		}
	}
}
