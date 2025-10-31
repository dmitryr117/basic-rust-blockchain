use sha3::{Digest, Sha3_256};

pub fn cryptohash(data: &Vec<String>, last_hash: &str, timestamp: i64) -> Vec<u8> {
	let mut hasher = Sha3_256::new();
	let data = data.join(":");
	let new_data = format!("{data}:{last_hash}:{timestamp}");
	hasher.update(new_data);
	hasher.finalize().to_vec()
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_crypto_hash() {
		let expected_hash = "c58e6888d439dbefc904c8f3d356c3b8206eb038e89e3d0d9a16d9276e268f74";
		let mydata = vec![String::from("mydata")];
		let result = cryptohash(&mydata, "my_hash", 1234);
		let hexval = hex::encode(&result);
		assert_eq!(hexval, expected_hash);
	}
}
