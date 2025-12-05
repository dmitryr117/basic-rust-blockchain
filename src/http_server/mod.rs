pub mod blocks;
pub mod transact;
pub mod transaction_pool;

use std::sync::Arc;

use axum::{Router, routing::get};
use tokio::{
	sync::{RwLock, mpsc},
	task::JoinHandle,
};

use crate::{
	blockchain::Blockchain, channels::AppEvent,
	transaction_pool::TransactionPool, wallet::Wallet,
};

#[derive(Clone)]
pub struct AppState {
	pub wallet: Arc<RwLock<Wallet>>,
	pub blockchain: Arc<RwLock<Blockchain>>,
	pub transaction_pool: Arc<RwLock<TransactionPool>>,
	pub event_tx: Arc<mpsc::UnboundedSender<AppEvent>>,
}

pub fn start_http_server_task(
	port: u32,
	wallet: Arc<RwLock<Wallet>>,
	blockchain: Arc<RwLock<Blockchain>>,
	transaction_pool: Arc<RwLock<TransactionPool>>,
	event_tx: Arc<mpsc::UnboundedSender<AppEvent>>,
) -> JoinHandle<()> {
	tokio::spawn(async move {
		let state = AppState { wallet, blockchain, transaction_pool, event_tx };

		let app: Router = Router::new()
			.nest(
				"/api",
				Router::new()
					.merge(blocks::routes())
					.merge(transact::routes())
					.merge(transaction_pool::routes())
					.route("/", get(hello_world)),
			)
			.with_state(state);

		let listener =
			tokio::net::TcpListener::bind(format!("localhost:{port}"))
				.await
				.expect(&format!("Failed to bind to port {port}"));

		axum::serve(listener, app)
			.await
			.expect("HTTP server failed.");
	})
}

async fn hello_world() -> &'static str {
	"Hello, rust World!"
}
