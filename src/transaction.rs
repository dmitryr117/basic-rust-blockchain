use std::collections::HashMap;

use crate::txn_input::TransactionInput;
use crate::utils::output_map_to_bytes;
use crate::wallet::Wallet;
use libp2p::identity::PublicKey;
use rand::Rng;
use uuid::Uuid;

pub struct Transaction {
	pub id: Uuid,
	pub amount: usize,
	pub input: TransactionInput,
	pub output_map: HashMap<Vec<u8>, usize>,
}

impl Transaction {
	pub fn new(
		sender_wallet: &Wallet,
		recipient_pk: &Vec<u8>,
		amount: usize,
	) -> Self {
		let id = Self::generate_uuid_v1();
		let output_map =
			Transaction::create_output_map(sender_wallet, recipient_pk, amount);
		let output_bytes = output_map_to_bytes(&output_map);
		let signature = sender_wallet
			.sign(&output_bytes)
			.expect("Failed to generate signature.");
		let input = TransactionInput::new(
			sender_wallet.balance,
			sender_wallet.public_key.clone(),
			signature,
		);
		Self { id, amount, output_map, input }
	}

	pub fn generate_uuid_v1() -> Uuid {
		let mut node_id = [0u8; 6];
		rand::rng().fill(&mut node_id);

		Uuid::now_v1(&node_id)
	}

	pub fn create_output_map(
		sender_wallet: &Wallet,
		recipient_pk: &Vec<u8>,
		amount: usize,
	) -> HashMap<Vec<u8>, usize> {
		let mut output_map: HashMap<Vec<u8>, usize> = HashMap::new();

		output_map.insert(recipient_pk.clone(), amount);
		output_map.insert(
			sender_wallet.public_key.clone(),
			sender_wallet.balance - amount,
		);

		output_map
	}

	pub fn is_valid(&self) -> bool {
		let output_total: usize = self.output_map.values().sum();

		if self.input.amount != output_total {
			let amt = self.input.amount;
			let invalid_address =
				PublicKey::try_decode_protobuf(&self.input.sender_address)
					.expect("Failed to decode address protobuf.")
					.to_peer_id()
					.to_string();
			eprintln!(
				"Invalid transaction data from address: {invalid_address}, {output_total}, {amt}"
			);
			return false;
		}

		let data = output_map_to_bytes(&self.output_map);
		if !Wallet::verify_signature(
			&self.input.sender_address,
			&data,
			&self.input.signature,
		) {
			let invalid_address =
				PublicKey::try_decode_protobuf(&self.input.sender_address)
					.expect("Failed to decode address protobuf.")
					.to_peer_id()
					.to_string();
			eprintln!(
				"Invalid transaction signature from address: {invalid_address}"
			);
			return false;
		}

		true
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		transaction::Transaction, utils::output_map_to_bytes, wallet::Wallet,
	};
	use libp2p::identity::Keypair;
	use pretty_assertions::assert_eq;

	fn before_each() -> (Wallet, Wallet, usize) {
		let sender_wallet = Wallet::new(&Keypair::generate_ed25519());
		let recipient_wallet = Wallet::new(&Keypair::generate_ed25519());
		let amount: usize = 50;

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
