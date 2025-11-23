use std::collections::HashMap;

use crate::constants::{U32_SIZE, UUID_SIZE};
use crate::txn_input::TransactionInput;
use crate::utils::output_map_to_bytes;
use crate::wallet::Wallet;
use libp2p::identity::PublicKey;
use rand::Rng;
use serde::Serialize;
use serde_with::serde_as;
use uuid::Uuid;

#[serde_as]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Transaction {
	pub id: Uuid,
	pub amount: u32,
	pub input: TransactionInput,
	#[serde_as(as = "HashMap<serde_with::hex::Hex, _>")]
	pub output_map: HashMap<Vec<u8>, u32>,
}

impl Transaction {
	pub fn new(
		sender_wallet: &Wallet,
		recipient_pk: &Vec<u8>,
		amount: u32,
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
		amount: u32,
	) -> HashMap<Vec<u8>, u32> {
		let mut output_map: HashMap<Vec<u8>, u32> = HashMap::new();

		output_map.insert(recipient_pk.clone(), amount);
		output_map.insert(
			sender_wallet.public_key.clone(),
			sender_wallet.balance - amount,
		);

		output_map
	}

	pub fn is_valid(&self) -> bool {
		let output_total: u32 = self.output_map.values().sum();

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
		next_recipient: &Vec<u8>,
		next_amount: u32,
	) -> Result<(), &str> {
		let output_balance = *self
			.output_map
			.get(&sender_wallet.public_key)
			.unwrap();

		if next_amount > output_balance {
			return Err("Insufficient wallet balance.");
		}

		let mut recipient_amount = next_amount;
		if self.output_map.contains_key(next_recipient) {
			recipient_amount = *self
				.output_map
				.get(next_recipient)
				.expect("Unable to get amount value.")
				+ recipient_amount;
		}
		self.output_map
			.insert(next_recipient.clone(), recipient_amount);

		self.output_map.insert(
			sender_wallet.public_key.clone(),
			output_balance - next_amount,
		);

		self.input = TransactionInput::new(sender_wallet, &self.output_map);

		Ok(())
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
		let mut bytes: Vec<u8> = Vec::new();
		let config = bincode::config::standard();
		bytes.extend(self.id.to_bytes_le());
		bytes.extend(self.amount.to_le_bytes());
		let input_bytes = bincode::encode_to_vec(&self.input, config)?;
		bytes.extend((input_bytes.len() as u32).to_le_bytes());
		bytes.extend(input_bytes);
		let output_map_bytes =
			bincode::encode_to_vec(&self.output_map, config)?;
		bytes.extend((output_map_bytes.len() as u32).to_le_bytes());
		bytes.extend(output_map_bytes);
		Ok(bytes)
	}

	pub fn from_bytes(
		bytes: Vec<u8>,
	) -> Result<Self, Box<dyn std::error::Error>> {
		let config = bincode::config::standard();
		let mut cursor: usize = 0;

		if bytes.len() < cursor + UUID_SIZE {
			return Err("Insufficient bytes for uuid.".into());
		}
		let uuid_bytes: [u8; UUID_SIZE] =
			bytes[cursor..cursor + UUID_SIZE].try_into()?;
		let id = Uuid::from_bytes_le(uuid_bytes);

		cursor += UUID_SIZE;

		if bytes.len() < cursor + U32_SIZE {
			return Err("Insufficient bytes for amount.".into());
		}
		let amount_bytes: [u8; U32_SIZE] =
			bytes[cursor..cursor + U32_SIZE].try_into()?;
		let amount = u32::from_le_bytes(amount_bytes);

		cursor += U32_SIZE;

		if bytes.len() < cursor + U32_SIZE {
			return Err("Insufficient bytes for input size.".into());
		}

		let input_size_bytes: [u8; U32_SIZE] =
			bytes[cursor..cursor + U32_SIZE].try_into()?;
		let input_size = u32::from_le_bytes(input_size_bytes);

		cursor += U32_SIZE;

		if bytes.len() < cursor + input_size as usize {
			return Err("Insufficient bytes for input.".into());
		}

		let input_bytes: Vec<u8> =
			bytes[cursor..cursor + input_size as usize].try_into()?;
		let (input, _bytes): (TransactionInput, usize) =
			bincode::decode_from_slice(&input_bytes, config)?;

		cursor += input_size as usize;

		if bytes.len() < cursor + U32_SIZE {
			return Err("Insufficient bytes for output map size.".into());
		}

		let output_map_size_bytes: [u8; U32_SIZE] =
			bytes[cursor..cursor + U32_SIZE].try_into()?;
		let output_map_size = u32::from_le_bytes(output_map_size_bytes);

		cursor += U32_SIZE;

		if bytes.len() < cursor + output_map_size as usize {
			return Err("Insufficient bytes for output map.".into());
		}

		let output_map_bytes: Vec<u8> =
			bytes[cursor..cursor + output_map_size as usize].try_into()?;
		let (output_map, _bytes): (HashMap<Vec<u8>, u32>, usize) =
			bincode::decode_from_slice(&output_map_bytes, config)?;

		Ok(Self { id, amount, input, output_map })
	}
}
