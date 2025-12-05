use std::collections::BTreeMap;

use sha3::{Digest, Sha3_256};

pub fn cryptohash(
	data: &[u8],
	last_hash: &str,
	timestamp: i64,
	nonce: u32,
	difficulty: u32,
) -> Vec<u8> {
	let mut hasher = Sha3_256::new();
	// let data = data.join(":");
	// let new_data =
	// 	format!("{data}:{last_hash}:{timestamp}:{nonce}:{difficulty}");
	let mut new_data: Vec<u8> = Vec::new();
	new_data.extend(data);
	new_data.extend(last_hash.as_bytes());
	new_data.extend(timestamp.to_le_bytes());
	new_data.extend(nonce.to_le_bytes());
	new_data.extend(difficulty.to_be_bytes());
	hasher.update(new_data);
	hasher.finalize().to_vec()
}

pub fn output_map_to_bytes(output_map: &BTreeMap<Vec<u8>, u32>) -> Vec<u8> {
	let config = bincode::config::standard();
	bincode::encode_to_vec(output_map, config)
		.expect("Output bytes failed to encode.")
}

#[cfg(test)]
mod test {
	use super::*;
	use pretty_assertions::assert_eq;

	#[test]
	fn test_crypto_hash() {
		let expected_hash =
			"bb2444b6656b87e461c214189ae14603cafe5fe332391b3cebe99a9dab2fdd9b";
		let mydata = "Test_data_string".as_bytes();
		let result = cryptohash(mydata, "my_hash", 1234, 1, 1);
		let hexval = hex::encode(&result);
		assert_eq!(hexval, expected_hash);
	}
}
