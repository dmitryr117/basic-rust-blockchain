use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub struct AppMessage {
	pub action: String,
	pub uuid: Uuid,
}

impl AppMessage {
	pub fn new(action: String, uuid: Uuid) -> Self {
		Self { action, uuid }
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
