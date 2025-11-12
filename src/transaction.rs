pub struct Transaction {}

impl Transaction {}

#[cfg(test)]
mod tests {
	use crate::wallet::Wallet;
	use libp2p::identity::Keypair;

	fn before_each() -> (Wallet, Wallet, i32) {
		let kp1 = Keypair::generate_ed25519();
		let sender_wallet = Wallet::new(&kp1);

		let kp2 = Keypair::generate_ed25519();
		let recipient_wallet = Wallet::new(&kp2);
		let amount = 50;

		(sender_wallet, recipient_wallet, amount)
	}

	#[test]
	fn test_has_txn_id() {
		let (sender_wallet, recipient_wallet, amount) = before_each();

		let transaction = Transaction::new(sender_wallet, recipient, amount);
	}

	mod output_map_tests {
		use super::*;

		#[test]
		fn output_amount_to_recipient() {
			let (sender_wallet, recipient_wallet, amount) = before_each();
			let transaction =
				Transaction::new(sender_wallet, recipient, amount);

			assert_eq!(transaction.outputMap(recipient_wallet), amount);
		}

		#[test]
		fn output_amount_to_sender() {
			let (sender_wallet, recipient_wallet, amount) = before_each();
			let transaction =
				Transaction::new(sender_wallet, recipient, amount);

			let balance = sender_wallet.balance - amount;
			assert_eq!(transaction.outputMap(sender_wallet), amount);
		}
	}
}
