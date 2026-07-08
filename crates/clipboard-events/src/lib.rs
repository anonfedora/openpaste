//! OpenPaste Events Module
//!
//! This module provides an event bus for internal communication using publish-subscribe pattern.

pub mod bus;
pub mod event;

pub use bus::EventBus;
pub use event::Event;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Event publish failed: {0}")]
    PublishFailed(String),
    #[error("Event subscribe failed: {0}")]
    SubscribeFailed(String),
}
