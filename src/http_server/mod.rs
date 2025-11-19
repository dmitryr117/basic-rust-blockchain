pub mod transact;

use std::sync::Arc;

use axum::{Router, routing::get};
use tokio::{sync::RwLock, task::JoinHandle};

use crate::{transaction_pool::TransactionPool, wallet::Wallet};

#[derive(Clone)]
pub struct AppState {
	pub wallet: Arc<RwLock<Wallet>>,
	pub transaction_pool: Arc<RwLock<TransactionPool>>,
}

pub fn start_http_server_task(
	wallet: Arc<RwLock<Wallet>>,
	transaction_pool: Arc<RwLock<TransactionPool>>,
) -> JoinHandle<()> {
	tokio::spawn(async move {
		let state = AppState { wallet, transaction_pool };

		let app: Router = Router::new()
			.merge(transact::routes())
			.route("/", get(hello_world))
			.with_state(state);

		let listener = tokio::net::TcpListener::bind("localhost:3005")
			.await
			.expect("Failed to bind to port 3005");

		axum::serve(listener, app)
			.await
			.expect("HTTP server failed.");
	})
}

async fn hello_world() -> &'static str {
	"Hello, rust World!"
}
