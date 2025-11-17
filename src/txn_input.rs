use chrono::Utc;
use std::collections::HashMap;

use crate::{utils::output_map_to_bytes, wallet::Wallet};

pub struct TransactionInput {
	pub timestamp: usize,
	pub amount: usize,
	pub sender_address: Vec<u8>,
	pub signature: Vec<u8>,
}

impl TransactionInput {
	pub fn new(
		sender_wallet: &Wallet,
		output_map: &HashMap<Vec<u8>, usize>,
	) -> Self {
		let timestamp = Utc::now().timestamp_millis() as usize;
		let output_bytes = output_map_to_bytes(&output_map);
		let signature = sender_wallet
			.sign(&output_bytes)
			.expect("Failed to generate signature.");

		Self {
			timestamp,
			amount: sender_wallet.balance,
			sender_address: sender_wallet.public_key.clone(),
			signature,
		}
	}
}
