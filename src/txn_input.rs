use chrono::Utc;

pub struct TransactionInput {
	pub timestamp: usize,
	pub amount: usize,
	pub sender_address: Vec<u8>,
	pub signature: Vec<u8>,
}

impl TransactionInput {
	pub fn new(
		amount: usize,
		sender_address: Vec<u8>,
		signature: Vec<u8>,
	) -> Self {
		let timestamp = Utc::now().timestamp_millis() as usize;
		Self { timestamp, amount, sender_address, signature }
	}
}
