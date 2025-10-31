use sha3::{Digest, Sha3_256};

pub fn cryptohash(data: &[String], last_hash: &str, timestamp: usize, nonce: usize, difficulty: usize) -> Vec<u8> {
	let mut hasher = Sha3_256::new();
	let data = data.join(":");
	let new_data = format!("{data}:{last_hash}:{timestamp}:{nonce}:{difficulty}");
	hasher.update(new_data);
	hasher.finalize().to_vec()
}

#[cfg(test)]
mod test {
	use super::*;
	use pretty_assertions::assert_eq;

	#[test]
	fn test_crypto_hash() {
		let expected_hash = "b3a631a9a270c4e28788ff9e6eea9f3f26b08fa2911b9f9bf36bb693bed43bda";
		let mydata = vec![String::from("mydata")];
		let result = cryptohash(&mydata, "my_hash", 1234, 1, 1);
		let hexval = hex::encode(&result);
		assert_eq!(hexval, expected_hash);
	}
}
