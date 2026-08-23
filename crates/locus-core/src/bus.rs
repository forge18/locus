//! In-process event fan-out.
//!
//! PLAN.md §Process topology lists the event bus beside the store, not inside it: only
//! the cross-process half (`store::bus::PostgresBus`, over LISTEN/NOTIFY) needs Postgres.
//!
//! Five modules each had their own copy of this before it moved here.

use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct InProcessBus<T> {
    sender: broadcast::Sender<T>,
}

impl<T: Clone> InProcessBus<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }

    /// Sends an event to current subscribers, returning their count.
    pub fn publish(&self, event: T) -> usize {
        self.sender.send(event).unwrap_or(0)
    }

    /// Whether two handles fan out to the same subscribers.
    pub fn same_channel(&self, other: &Self) -> bool {
        self.sender.same_channel(&other.sender)
    }
}

#[cfg(test)]
mod in_process {
    use super::InProcessBus;

    #[tokio::test]
    async fn broadcasts_to_every_subscriber() {
        let bus = InProcessBus::new(4);
        let mut first = bus.subscribe();
        let mut second = bus.subscribe();

        assert_eq!(bus.publish("run.completed"), 2);
        assert_eq!(first.recv().await.expect("first event"), "run.completed");
        assert_eq!(second.recv().await.expect("second event"), "run.completed");
    }
}
