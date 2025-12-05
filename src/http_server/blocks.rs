use axum::{Json, Router, extract::State, http::StatusCode, routing::get};

use crate::{block::Block, http_server::AppState};

pub fn routes() -> Router<AppState> {
	Router::new().route("/blocks", get(get_blocks))
}

async fn get_blocks(
	State(state): State<AppState>,
) -> Result<Json<Vec<Block>>, (StatusCode, String)> {
	let blockchain = state.blockchain.read().await;
	Ok(Json(blockchain.chain.clone()))
}
