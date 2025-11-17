use bs58;
use hex;

use crate::config::STARTING_BALANCE;
use libp2p::identity::{Keypair, PublicKey, SigningError};
use sha3::{Digest, Sha3_256};

#[derive(Clone, Debug)]
pub struct Wallet {
	// **ALERT!**  This has to be removed in production. Transactions have
	//to be signed locally before being submitted into the system
	pub keypair: Keypair,
	pub public_key: Vec<u8>,
	pub balance: usize,
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

	pub fn get_peer_id(&self) -> String {
		let public_key = PublicKey::try_decode_protobuf(&self.public_key)
			.expect("Failed to dekode PK protobuf");
		public_key.to_peer_id().to_string()
	}

	pub fn verify_signature(
		public_key: &Vec<u8>,
		data: &[u8],
		signature: &[u8],
	) -> bool {
		let pk = PublicKey::try_decode_protobuf(public_key)
			.expect("Failed to decode PK protobuf");
		pk.verify(data, signature)
	}

	pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SigningError> {
		self.keypair.sign(data)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use libp2p::identity::Keypair;
	use pretty_assertions::assert_eq;

	#[test]
	fn test_default_balance() {
		let keypair = Keypair::generate_ed25519();
		let wallet = Wallet::new(&keypair);
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

	#[test]
	fn test_sign_data_verify_signature() {
		let data = "FooBar".as_bytes();

		let keypair = Keypair::generate_ed25519();
		let wallet = Wallet::new(&keypair);

		let signed_data = wallet.sign(data).unwrap();

		let verified =
			Wallet::verify_signature(&wallet.public_key, data, &signed_data);
		assert_eq!(verified, true);
	}

	#[test]
	fn test_sign_data_invalid_signature() {
		let data = "FooBar".as_bytes();

		let keypair = Keypair::generate_ed25519();
		let wallet = Wallet::new(&keypair);

		let keypair = Keypair::generate_ed25519();
		let wrong_wallet = Wallet::new(&keypair);

		let wrong_signed_data = wrong_wallet.sign(data).unwrap();

		let verified = Wallet::verify_signature(
			&wallet.public_key,
			data,
			&wrong_signed_data,
		);
		assert_eq!(verified, false);
	}
}
