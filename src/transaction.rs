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
		let input = TransactionInput::new(sender_wallet, &output_map);
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

	pub fn update(
		&mut self,
		sender_wallet: &Wallet,
		next_recipient: Vec<u8>,
		next_amount: usize,
	) -> Result<(), ()> {
		let output_balance = *self
			.output_map
			.get(&sender_wallet.public_key)
			.unwrap();

		if next_amount > output_balance {
			return Err(());
		}

		let mut recipient_amount = next_amount;
		if self.output_map.contains_key(&next_recipient) {
			recipient_amount = *self
				.output_map
				.get(&next_recipient)
				.expect("Unable to get amount value.")
				+ recipient_amount;
		}
		self.output_map
			.insert(next_recipient, recipient_amount);

		self.output_map.insert(
			sender_wallet.public_key.clone(),
			output_balance - next_amount,
		);

		self.input = TransactionInput::new(sender_wallet, &self.output_map);

		Ok(())
	}
}
