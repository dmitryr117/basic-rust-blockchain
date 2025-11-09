use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

/// Returns a debounced version of an async function.
/// Each call resets the timer; only the last one actually runs.
pub fn debounce<F, Fut>(
	delay: Duration,
	func: F,
) -> impl Fn() + Send + Sync + 'static
where
	F: Fn() -> Fut + Send + Sync + 'static,
	Fut: std::future::Future<Output = ()> + Send + 'static,
{
	let func = Arc::new(func);
	let state: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> =
		Arc::new(Mutex::new(None));

	move || {
		let func = func.clone();
		let state = state.clone();

		tokio::spawn(async move {
			// Cancel previous timer
			if let Some(handle) = state.lock().await.take() {
				handle.abort();
			}

			// Start new timer
			let handle = tokio::spawn(async move {
				sleep(delay).await;
				(func)().await;
			});

			*state.lock().await = Some(handle);
		});
	}
}
