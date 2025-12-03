use tokio::sync::mpsc;

#[derive(Debug)]
pub struct AppMessage {
	pub action: String,
	pub data: Option<Vec<u8>>,
}

impl AppMessage {
	pub fn new(action: String, data: Option<Vec<u8>>) -> Self {
		Self { action, data }
	}
}

#[derive(Debug)]
pub enum AppEvent {
	BroadcastMessage(AppMessage),
	SyncBlockchain,
}

pub fn create_unbounded_channel()
-> (mpsc::UnboundedSender<AppEvent>, mpsc::UnboundedReceiver<AppEvent>) {
	mpsc::unbounded_channel::<AppEvent>()
}
