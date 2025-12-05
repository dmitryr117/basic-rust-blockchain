use bincode::{Decode, Encode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::BTreeMap;

use crate::{utils::output_map_to_bytes, wallet::Wallet};

#[serde_as]
#[derive(
	Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Decode, Encode,
)]
pub struct TransactionInput {
	pub timestamp: i64,
	pub amount: u32,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub sender_address: Vec<u8>,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub signature: Vec<u8>,
}

impl TransactionInput {
	pub fn new(
		sender_wallet: &Wallet,
		output_map: &BTreeMap<Vec<u8>, u32>,
	) -> Self {
		let timestamp = Utc::now().timestamp_millis();
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

	pub fn new_reward_input(sender_pk: &[u8]) -> Self {
		let timestamp = Utc::now().timestamp_millis();

		Self {
			timestamp,
			amount: 0,
			sender_address: sender_pk.to_vec().clone(),
			signature: vec![],
		}
	}
}
