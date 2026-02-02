use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::broadcast;

pub use broadcast::error::SendError;
pub use broadcast::Receiver;

/// Trait that user-defined event enums must implement.
///
/// The `Kind` associated type is the discriminant enum used for channel routing.
/// This design separates "what kind of event" from "what data does it carry".
pub trait EventLike: Clone + Send + Sync + 'static {
    /// The discriminant type used for routing.
    /// Must be Copy + Hash + Eq so it can be used as HashMap keys.
    type Kind: Copy + Hash + Eq + Send + Sync + std::fmt::Debug + 'static;

    /// Get the discriminant for this event instance.
    fn kind(&self) -> Self::Kind;

    /// Return an iterator over ALL possible event kinds.
    /// This is used at bus construction time to pre-create channels.
    fn all_kinds() -> impl Iterator<Item = Self::Kind>;
}

/// Event bus with discriminant-keyed channels for efficient routing.
///
/// # Type Parameters
/// - `E`: The event type, must implement `EventLike`
#[derive(Clone)]
pub struct EventBus<E: EventLike> {
    inner: Arc<EventBusInner<E>>,
}

struct EventBusInner<E: EventLike> {
    channels: HashMap<E::Kind, broadcast::Sender<E>>,
    capacity: usize,
}

impl<E: EventLike> EventBus<E> {
    /// Create a new event bus with the given channel capacity per event kind.
    pub fn new(capacity: usize) -> Self {
        let channels = E::all_kinds()
            .map(|kind| {
                let (tx, _) = broadcast::channel(capacity);
                (kind, tx)
            })
            .collect();

        Self {
            inner: Arc::new(EventBusInner { channels, capacity }),
        }
    }

    /// Publish an event to the appropriate channel (sync - no await needed).
    ///
    /// Returns the number of receivers that received the event,
    /// or an error if the channel has no subscribers.
    pub fn publish(&self, event: E) -> Result<usize, SendError<E>> {
        let kind = event.kind();
        self.inner
            .channels
            .get(&kind)
            .expect("all event kinds should have channels initialized")
            .send(event)
    }

    /// Subscribe to a specific event kind.
    ///
    /// Returns a `Receiver` that will receive all events of the given kind.
    pub fn subscribe(&self, kind: E::Kind) -> Receiver<E> {
        self.inner
            .channels
            .get(&kind)
            .expect("all event kinds should have channels initialized")
            .subscribe()
    }

    /// Get the configured channel capacity.
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }
}
