use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
	channels::{AppEvent, AppMessage},
	constants,
	http_server::AppState,
	transaction::Transaction,
};

#[derive(Debug, Deserialize, Validate)]
struct TransactionDto {
	#[validate(required)]
	amount: Option<u32>,
	#[validate(required)]
	recipient: Option<String>,
}

pub fn routes() -> Router<AppState> {
	Router::new().route("/transact", post(transact))
}

async fn broadcast_txn(state: &AppState, uuid: &Uuid) {
	if let Ok(_) =
		state
			.event_tx
			.send(AppEvent::BroadcastMessage(AppMessage::new(
				constants::BROADCAST_TXN_POOL.to_string(),
				Some(uuid.to_bytes_le().to_vec()),
			))) {};
}

async fn transact(
	State(state): State<AppState>,
	Json(payload): Json<TransactionDto>,
) -> Result<Json<Transaction>, (StatusCode, String)> {
	// Transaction signing has to happen on client when system becomes operational.
	// Transactions should be only submitted via API. But will mod it later.
	let wallet = state.wallet.read().await;
	let amount = payload.amount.expect("Unable to load amount.");
	let recipient_hex_address = payload
		.recipient
		.expect("Unable to load recipient address.");
	let recipient_vec_address = hex::decode(recipient_hex_address.clone())
		.expect("Unable to decode recipient hex address.");

	let mut transaction_pool = state.transaction_pool.write().await;

	let existing_transaction =
		transaction_pool.existing_transaction_mut(&wallet.public_key);

	match existing_transaction {
		Some(transaction) => {
			let update_result =
				transaction.update(&wallet, &recipient_vec_address, amount);
			match update_result {
				Ok(()) => {
					broadcast_txn(&state, &transaction.id).await;
					Ok(Json(transaction.clone()))
				}
				Err(err) => Err((
					StatusCode::BAD_REQUEST,
					format!("Invalid transaction: {}", err),
				)),
			}
		}
		None => {
			let txn_result =
				wallet.create_transaction(amount, &recipient_vec_address);
			match txn_result {
				Ok(transaction) => {
					transaction_pool.set_transaction(transaction.clone());
					broadcast_txn(&state, &transaction.id).await;
					Ok(Json(transaction))
				}
				Err(err) => Err((
					StatusCode::BAD_REQUEST,
					format!("Invalid transaction: {}", err),
				)),
			}
		}
	}
}
