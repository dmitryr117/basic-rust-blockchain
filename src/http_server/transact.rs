use axum::{Json, Router, extract::State, routing::post};
use serde::Deserialize;
use validator::Validate;

use crate::http_server::AppState;

#[derive(Debug, Deserialize, Validate)]
struct TransactionDto {
	#[validate(required)]
	amount: Option<usize>,
	#[validate(required)]
	recipient: Option<String>,
}

pub fn routes() -> Router<AppState> {
	Router::new().route("/transact", post(transact))
}

async fn transact(
	State(state): State<AppState>,
	Json(payload): Json<TransactionDto>,
) -> String {
	// Transaction signing has to happen on client when system becomes operational.
	// Transactions should be only submitted via API. But will mod it later.
	let wallet = state.wallet.read().await;
	let amount = payload.amount.expect("Unable to load amount.");
	let recipient_hex_address = payload
		.recipient
		.expect("Unable to load recipient address.");
	let pecipient_vec_address = hex::decode(recipient_hex_address.clone())
		.expect("Unable to decode recipient hex address.");

	let transaction = wallet
		.create_transaction(amount, &pecipient_vec_address)
		.unwrap();

	let mut transaction_pool = state.transaction_pool.write().await;
	transaction_pool.set_transaction(transaction);

	println!("transaction_pool: {:#?}", transaction_pool);

	format!(
		"Created transaction: sender: {}, amount: {} recipient: {}",
		hex::encode(wallet.public_key.clone()),
		payload.amount.unwrap(),
		recipient_hex_address,
	)
}
