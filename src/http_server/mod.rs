use axum::{Router, routing::get};
use tokio::task::JoinHandle;

pub fn start_http_server_task() -> JoinHandle<()> {
	tokio::spawn(async move {
		let app: Router = Router::new().route("/", get(hello_world));

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
