//! Event bus implementation

use crate::{Event, EventError};
use tokio::sync::broadcast;

/// Event bus for publish-subscribe communication
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event
    pub fn publish(&self, event: Event) -> Result<(), EventError> {
        self.sender
            .send(event)
            .map(|_| ())
            .map_err(|e| EventError::PublishFailed(e.to_string()))
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}
