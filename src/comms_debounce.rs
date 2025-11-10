use std::time::{Duration, Instant};

pub struct Debouncer {
	last_event: Option<Instant>,
	delay: Duration,
	pending_execution: bool,
}

impl Debouncer {
	pub fn new(delay: Duration) -> Self {
		Self { last_event: None, delay, pending_execution: false }
	}

	pub fn on_event(&mut self) {
		self.last_event = Some(Instant::now());
		self.pending_execution = true;
	}

	pub fn check(&mut self) -> bool {
		if self.pending_execution {
			if let Some(last_event) = self.last_event {
				if last_event.elapsed() >= self.delay {
					self.pending_execution = false;
					return true;
				}
			}
		}
		false
	}
}
