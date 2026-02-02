use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::bus::{EventBus, EventLike};

/// Context passed to handlers - allows publishing follow-up events.
#[derive(Clone)]
pub struct HandlerContext<E: EventLike> {
    bus: EventBus<E>,
}

impl<E: EventLike> HandlerContext<E> {
    pub(crate) fn new(bus: EventBus<E>) -> Self {
        Self { bus }
    }

    /// Publish a follow-up event.
    ///
    /// Errors are ignored (only happens if channel has no subscribers).
    pub fn publish(&self, event: E) {
        let _ = self.bus.publish(event);
    }

    /// Publish with error handling if caller needs to know about failures.
    pub fn try_publish(&self, event: E) -> Result<usize, broadcast::error::SendError<E>> {
        self.bus.publish(event)
    }
}

/// Trait for event handlers.
///
/// Handlers are spawned as tokio tasks and receive events from a single channel.
#[async_trait]
pub trait EventHandler<E: EventLike>: Send + Sync {
    /// Which event kind this handler processes.
    fn handles(&self) -> E::Kind;

    /// Process an event.
    ///
    /// The handler can emit follow-up events via the context.
    async fn handle(&self, event: E, ctx: &HandlerContext<E>);

    /// Handler name for logging and debugging.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Registry for managing handler lifecycle.
pub struct HandlerRegistry<E: EventLike> {
    handlers: Vec<Arc<dyn EventHandler<E>>>,
    cancel_token: CancellationToken,
}

impl<E: EventLike> HandlerRegistry<E> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            cancel_token: CancellationToken::new(),
        }
    }

    /// Register a handler.
    pub fn register<H: EventHandler<E> + 'static>(&mut self, handler: H) {
        self.handlers.push(Arc::new(handler));
    }

    /// Spawn all handler tasks, returns join handles.
    pub fn run(&self, bus: EventBus<E>) -> Vec<JoinHandle<()>> {
        self.handlers
            .iter()
            .map(|handler| {
                let kind = handler.handles();
                let mut rx = bus.subscribe(kind);
                let handler = Arc::clone(handler);
                let bus_clone = bus.clone();
                let token = self.cancel_token.clone();
                let handler_name = handler.name();

                tokio::spawn(async move {
                    info!("Handler {} started, listening for {:?}", handler_name, kind);

                    let mut events_received: u64 = 0;
                    let mut events_lagged: u64 = 0;

                    loop {
                        tokio::select! {
                            _ = token.cancelled() => {
                                info!(
                                    "Handler {} shutting down - received: {}, lagged: {}",
                                    handler_name, events_received, events_lagged
                                );
                                break;
                            }
                            result = rx.recv() => {
                                match result {
                                    Ok(event) => {
                                        events_received += 1;
                                        let ctx = HandlerContext::new(bus_clone.clone());
                                        handler.handle(event, &ctx).await;
                                    }
                                    Err(broadcast::error::RecvError::Lagged(n)) => {
                                        events_lagged += n;
                                        warn!(
                                            "Handler {} lagged, dropped {} events (total lagged: {})",
                                            handler_name, n, events_lagged
                                        );
                                    }
                                    Err(broadcast::error::RecvError::Closed) => {
                                        info!(
                                            "Handler {} channel closed - received: {}, lagged: {}",
                                            handler_name, events_received, events_lagged
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                    }
                })
            })
            .collect()
    }

    /// Signal all handlers to shut down.
    pub fn shutdown(&self) {
        info!("Signaling handler shutdown");
        self.cancel_token.cancel();
    }
}

impl<E: EventLike> Default for HandlerRegistry<E> {
    fn default() -> Self {
        Self::new()
    }
}
