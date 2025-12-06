use bs58;
use hex;

use crate::{
	block::Block, blockchain::Blockchain, config::STARTING_BALANCE,
	transaction::Transaction,
};
use libp2p::identity::{Keypair, PublicKey, SigningError};
use sha3::{Digest, Sha3_256};

#[derive(Clone, Debug)]
pub struct Wallet {
	// **ALERT!**  This has to be removed in production. Transactions have
	//to be signed locally before being submitted into the system
	pub keypair: Keypair,
	pub public_key: Vec<u8>,
	pub balance: u32,
}

impl Wallet {
	pub fn new(keypair: &Keypair) -> Self {
		// let priv_key = keypair.
		let public_key = keypair.public().encode_protobuf();

		let hex_pub_key = hex::encode(public_key.clone());

		println!("Hex pub key: {hex_pub_key}");

		Self {
			balance: STARTING_BALANCE,
			public_key,
			keypair: keypair.clone(), // will need to remove in production
		}
	}

	// need to think more about this. Has to get wallet balance based on value stored in ledger.
	// pub fn from_private_key(hex_private: &str) -> Result<(), Box<dyn Error>> {
	// 	let bytes = hex::decode(hex_private)?;
	// 	let keypair = Keypair::from_protobuf_encoding(&bytes)?;
	// 	let address = Self::derive_address(&keypair);

	// 	Ok(())
	// }

	pub fn derive_address(keypair: &Keypair) -> String {
		let public_key = keypair.public();
		let pubkey_bytes = public_key.encode_protobuf();
		let mut hasher = Sha3_256::new();
		hasher.update(&pubkey_bytes);
		let hash = hasher.finalize();
		bs58::encode(hash).into_string()
	}

	pub fn export_pk(keypair: &Keypair) -> String {
		hex::encode(
			keypair
				.to_protobuf_encoding()
				.expect("Failed to export private key."),
		)
	}

	// pub fn get_peer_id(&self) -> String {
	// 	let public_key = PublicKey::try_decode_protobuf(&self.public_key)
	// 		.expect("Failed to dekode PK protobuf");
	// 	public_key.to_peer_id().to_string()
	// }

	pub fn verify_signature(
		public_key: &Vec<u8>,
		data: &[u8],
		signature: &[u8],
	) -> bool {
		let pk = PublicKey::try_decode_protobuf(public_key)
			.expect("Failed to decode PK protobuf");
		pk.verify(data, signature)
	}

	pub fn calculate_balance(chain: &Vec<Block>, address: &[u8]) -> u32 {
		let mut conducted_txn = false;
		let mut outputs_total = 0;

		let mut block_iter = chain.iter().rev().peekable();

		while let Some(block) = block_iter.next() {
			if block_iter.peek().is_none() {
				break;
			}
			for txn in &block.data {
				if txn.input.sender_address == address {
					conducted_txn = true;
				}

				if let Some(address_output) = txn.output_map.get(address) {
					outputs_total += address_output;
				}
			}

			if conducted_txn {
				break;
			}
		}

		if conducted_txn {
			outputs_total
		} else {
			STARTING_BALANCE + outputs_total
		}
	}

	pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SigningError> {
		self.keypair.sign(data)
	}

	pub fn create_transaction(
		&mut self,
		amount: u32,
		recipient: &Vec<u8>,
		blockchain: &Blockchain,
	) -> Result<Transaction, &str> {
		if blockchain.chain.len() > 1 {
			self.balance =
				Wallet::calculate_balance(&blockchain.chain, &recipient);
		}

		if self.balance < amount {
			return Err("Insufficient ballance.");
		}
		Ok(Transaction::new(self, recipient, amount))
	}
}
