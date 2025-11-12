use std::collections::HashMap;

use crate::wallet::Wallet;
use libp2p::identity::PublicKey;
use rand::Rng;
use uuid::Uuid;

pub struct Transaction {
	id: Uuid,
	sender_wallet: Wallet,
	recipient_wallet: Wallet,
	amount: usize,
}

impl Transaction {
	pub fn new(
		sender_wallet: &Wallet,
		recipient_wallet: &Wallet,
		amount: usize,
	) -> Self {
		let id = Self::generate_uuid_v1();

		Self {
			id,
			sender_wallet: sender_wallet.clone(),
			recipient_wallet: recipient_wallet.clone(),
			amount,
		}
	}

	pub fn generate_uuid_v1() -> Uuid {
		let mut node_id = [0u8; 6];
		rand::rng().fill(&mut node_id);

		Uuid::now_v1(&node_id)
	}

	pub fn create_output_map(&self) -> HashMap<PublicKey, usize> {
		let mut output_map: HashMap<PublicKey, usize> = HashMap::new();
		output_map.insert(
			self.recipient_wallet.public_key.clone(),
			self.recipient_wallet.balance + self.amount,
		);
		output_map.insert(
			self.sender_wallet.public_key.clone(),
			self.sender_wallet.balance - self.amount,
		);

		output_map
	}
}

#[cfg(test)]
mod tests {
	use crate::{transaction::Transaction, wallet::Wallet};
	use libp2p::identity::Keypair;

	fn before_each() -> (Wallet, Wallet, usize) {
		let kp1 = Keypair::generate_ed25519();
		let sender_wallet = Wallet::new(&kp1);

		let kp2 = Keypair::generate_ed25519();
		let recipient_wallet = Wallet::new(&kp2);
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
		let transaction =
			Transaction::new(&sender_wallet, &recipient_wallet, amount);

		let txn_id_bytes = &transaction.id.into_bytes();

		assert!(txn_id_bytes.len() == 16);
	}

	#[test]
	fn output_amount_to_recipient() {
		let (sender_wallet, recipient_wallet, amount) = before_each();
		let transaction =
			Transaction::new(&sender_wallet, &recipient_wallet, amount);

		let output_map = transaction.create_output_map();

		let recipient_amount_comparator = recipient_wallet.balance + amount;

		let txn_value = output_map
			.get(&transaction.recipient_wallet.public_key)
			.unwrap();

		assert_eq!(*txn_value, recipient_amount_comparator);
	}

	#[test]
	fn output_amount_to_sender() {
		let (sender_wallet, recipient_wallet, amount) = before_each();
		let transaction =
			Transaction::new(&sender_wallet, &recipient_wallet, amount);

		let output_map = transaction.create_output_map();

		let sender_amount_comparator = sender_wallet.balance - amount;

		let txn_value = output_map
			.get(&transaction.sender_wallet.public_key)
			.unwrap();

		assert_eq!(*txn_value, sender_amount_comparator);
	}
}
