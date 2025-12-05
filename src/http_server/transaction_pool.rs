use std::collections::BTreeMap;

use axum::{Json, Router, extract::State, routing::get};
use uuid::Uuid;

use crate::{http_server::AppState, transaction::Transaction};

pub fn routes() -> Router<AppState> {
	Router::new().route("/transaction-pool-map", get(get_transaction_pool))
}

async fn get_transaction_pool(
	State(state): State<AppState>,
) -> Json<BTreeMap<Uuid, Transaction>> {
	let transaction_pool = state.transaction_pool.read().await;
	Json(transaction_pool.transaction_map.to_owned())
}
