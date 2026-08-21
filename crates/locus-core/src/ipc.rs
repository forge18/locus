//! Frequency-appropriate core-to-webview transports.

use tokio::sync::broadcast;

use crate::telemetry::Event;

/// High-frequency terminal bytes. Tauri forwards each item through a `Channel`, never an event.
#[derive(Clone, Debug)]
pub struct PtyChannel {
    sender: broadcast::Sender<Vec<u8>>,
}

impl PtyChannel {
    pub fn new(capacity: usize) -> Self { let (sender, _) = broadcast::channel(capacity); Self { sender } }
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> { self.sender.subscribe() }
    pub fn send(&self, bytes: &[u8]) { let _ = self.sender.send(bytes.to_vec()); }
}

/// High-frequency normalized records. Tauri forwards each item through a `Channel`, never an event.
#[derive(Clone, Debug)]
pub struct EventChannel {
    sender: broadcast::Sender<Event>,
}

impl EventChannel {
    pub fn new(capacity: usize) -> Self { let (sender, _) = broadcast::channel(capacity); Self { sender } }
    pub fn subscribe(&self) -> broadcast::Receiver<Event> { self.sender.subscribe() }
    pub fn send(&self, event: Event) { let _ = self.sender.send(event); }
}

/// Notifications are intentionally separate from streams: only low-frequency state changes emit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notification { RunFinished, TaskMoved, GuardrailTripped }

pub fn may_emit(notification: Notification) -> bool {
    matches!(notification, Notification::RunFinished | Notification::TaskMoved | Notification::GuardrailTripped)
}

#[cfg(test)]
mod pty_channel {
    use super::PtyChannel;
    #[tokio::test]
    async fn delivers_bytes_without_an_event_transport() { let stream = PtyChannel::new(1); let mut rx = stream.subscribe(); stream.send(b"pty"); assert_eq!(rx.recv().await.unwrap(), b"pty"); }
}

#[cfg(test)]
mod event_channel {
    use super::EventChannel;
    use crate::telemetry::{Event, EventVerb};
    #[tokio::test]
    async fn delivers_normalized_events_without_an_event_transport() { let stream = EventChannel::new(1); let mut rx = stream.subscribe(); let event = Event { run_id: "run".into(), seq: 1, ts: "2026-01-01T00:00:00Z".into(), verb: EventVerb::Assistant, text: None, tool: None, args: None, usage: None, raw: serde_json::json!({}) }; stream.send(event.clone()); assert_eq!(rx.recv().await.unwrap(), event); }
}

#[cfg(test)]
mod emit_is_low_frequency {
    use super::{may_emit, Notification};
    #[test]
    fn permits_only_lifecycle_notifications() { assert!(may_emit(Notification::RunFinished)); assert!(may_emit(Notification::TaskMoved)); assert!(may_emit(Notification::GuardrailTripped)); }
}
