//! In-process event delivery for the Locus core.

use tokio::sync::broadcast;

/// Broadcasts events to all subscribers in this process.
#[derive(Clone)]
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
