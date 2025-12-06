use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{http_server::AppState, wallet::Wallet};

pub fn routes() -> Router<AppState> {
	Router::new().route("/wallet-info", get(wallet_info))
}

#[serde_as]
#[derive(Serialize, Deserialize)]
struct WalletInfo {
	#[serde_as(as = "serde_with::hex::Hex")]
	pub public_key: Vec<u8>,
	pub balance: u32,
}

impl WalletInfo {
	fn new(public_key: Vec<u8>, balance: u32) -> Self {
		Self { public_key, balance }
	}
}

async fn wallet_info(
	State(state): State<AppState>,
) -> Result<Json<WalletInfo>, (StatusCode, String)> {
	let wallet = state.wallet.read().await;
	let blockchain = state.blockchain.read().await;
	let wallet_balance =
		Wallet::calculate_balance(&blockchain.chain, &wallet.public_key);
	let wallet_info =
		WalletInfo::new(wallet.public_key.clone(), wallet_balance);
	Ok(Json(wallet_info))
}
