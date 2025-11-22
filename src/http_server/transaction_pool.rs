use axum::{Json, Router, extract::State, routing::get};

use crate::{http_server::AppState, transaction_pool::TransactionPool};

pub fn routes() -> Router<AppState> {
	Router::new().route("/transaction_pool", get(get_transaction_pool))
}

async fn get_transaction_pool(
	State(state): State<AppState>,
) -> Json<TransactionPool> {
	let transaction_pool = state.transaction_pool.read().await;
	Json(transaction_pool.to_owned())
}
