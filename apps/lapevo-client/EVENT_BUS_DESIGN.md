# Event Bus Design for client-rs

This document outlines the design for a Rust event bus implementation, mirroring the functionality of the Python event bus in `racing-coach-core`.

## Table of Contents

1. [Design Goals](#design-goals)
2. [Core Architecture](#core-architecture)
3. [Event Type Management](#event-type-management)
4. [Bus-Level Filtering](#bus-level-filtering)
5. [Handler Typing Options](#handler-typing-options)
6. [Handler Publishing Alternatives](#handler-publishing-alternatives)
7. [Handler Execution Model](#handler-execution-model)
8. [EventBus Structure](#eventbus-structure)
9. [Error Handling](#error-handling)
10. [Comparison Matrices](#comparison-matrices)
11. [Resolved Design Decisions](#resolved-design-decisions)
12. [File Structure](#file-structure)
13. [References](#references)

---

## Design Goals

- **Performance**: High throughput for telemetry data (hundreds of Hz)
- **Memory Efficiency**: Avoid unnecessary copies of large payloads
- **Elegant API**: Handlers receive only their event type, no manual filtering
- **Flexibility**: Support both sync and async handlers, handler chaining
- **Tokio-native**: Built on tokio primitives

---

## Core Architecture

### Foundation: `tokio::sync::broadcast`

The event bus is built on `tokio::sync::broadcast` because it:

- Supports multiple receivers consuming the same events in parallel
- Has sync `send()` (no await needed for publishing)
- Has both async `recv()` and sync `blocking_recv()` for receiving
- Is well-maintained and battle-tested
- Handles slow receivers gracefully (lagging receivers drop old messages)

### Memory Efficiency: Arc-Wrapped Payloads

To avoid cloning large telemetry data for each receiver:

```rust
// BAD: Clones entire TelemetryFrame for each receiver
broadcast::channel::<TelemetryFrame>(100);

// GOOD: Only clones the Arc pointer (atomic refcount increment)
broadcast::channel::<Arc<TelemetryFrame>>(100);
```

All event payloads should be wrapped in `Arc<T>`. The broadcast channel clones `T` for each receiver, but `Arc::clone()` is just an atomic increment—effectively free.

---

## Event Type Management

The broadcast channel is generic over a single type. To support multiple event types with filtered subscriptions, there are three main approaches:

### Option A: Enum-Based Events (Recommended for Implementation)

Define all events as variants of a single enum:

```rust
use std::sync::Arc;

#[derive(Clone)]
pub enum Event {
    TelemetryFrame(Arc<TelemetryFrame>),
    LapComplete(Arc<LapAndSession>),
    SessionStart(Arc<SessionStart>),
    SessionEnd(Arc<SessionEnd>),
    MetricsExtracted(Arc<MetricsAndSession>),
    LapUploadSuccess(Arc<LapUploadResult>),
    LapUploadFailed(Arc<LapUploadResult>),
}
```

**Pros**:
- Zero-cost dispatch (enum discriminant comparison is one CPU instruction)
- Compile-time exhaustiveness checking
- Idiomatic Rust

**Cons**:
- Closed set: adding new event types requires modifying the enum
- With single-channel approach, each receiver gets all events over the wire

### Option B: Per-Type Channel Registry (TypeId-Keyed)

Maintain separate broadcast channels for each event type using `TypeId`:

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};

pub struct EventBus {
    channels: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    capacity: usize,
}

impl EventBus {
    pub async fn publish<E: Event>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        let channels = self.channels.read().await;

        if let Some(boxed) = channels.get(&type_id) {
            if let Some(tx) = boxed.downcast_ref::<broadcast::Sender<E>>() {
                let _ = tx.send(event);
            }
        }
    }

    pub async fn subscribe<E: Event>(&self) -> broadcast::Receiver<E> {
        let type_id = TypeId::of::<E>();
        let mut channels = self.channels.write().await;

        let sender = channels
            .entry(type_id)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel::<E>(self.capacity);
                Box::new(tx)
            })
            .downcast_ref::<broadcast::Sender<E>>()
            .expect("type mismatch");

        sender.subscribe()
    }
}
```

**Pros**:
- Open for extension (add new event types without modifying core)
- Natural filtering (receivers only get events they subscribed to)
- No filtering overhead in handler tasks

**Cons**:
- Runtime TypeId lookup (fast, but not zero-cost)
- More complex implementation
- Type safety relies on TypeId correctness

---

## Bus-Level Filtering

The single-channel enum approach sends all events to all receivers, requiring client-side filtering. For high-frequency events like telemetry, this adds overhead. **Discriminant-keyed channels** provide bus-level filtering while keeping enum type safety.

### Discriminant-Keyed Channels (Recommended)

Create a separate broadcast channel for each event kind:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Discriminant enum for efficient channel routing (no payload)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventKind {
    TelemetryFrame,
    LapComplete,
    SessionStart,
    SessionEnd,
    MetricsExtracted,
    LapUploadSuccess,
    LapUploadFailed,
}

impl EventKind {
    /// Returns an iterator over all event kinds
    pub fn all() -> impl Iterator<Item = EventKind> {
        [
            EventKind::TelemetryFrame,
            EventKind::LapComplete,
            EventKind::SessionStart,
            EventKind::SessionEnd,
            EventKind::MetricsExtracted,
            EventKind::LapUploadSuccess,
            EventKind::LapUploadFailed,
        ]
        .into_iter()
    }
}

impl Event {
    pub fn kind(&self) -> EventKind {
        match self {
            Event::TelemetryFrame(_) => EventKind::TelemetryFrame,
            Event::LapComplete(_) => EventKind::LapComplete,
            Event::SessionStart(_) => EventKind::SessionStart,
            Event::SessionEnd(_) => EventKind::SessionEnd,
            Event::MetricsExtracted(_) => EventKind::MetricsExtracted,
            Event::LapUploadSuccess(_) => EventKind::LapUploadSuccess,
            Event::LapUploadFailed(_) => EventKind::LapUploadFailed,
        }
    }
}

#[derive(Clone)]
pub struct EventBus {
    inner: Arc<EventBusInner>,
}

struct EventBusInner {
    channels: HashMap<EventKind, broadcast::Sender<Event>>,
    capacity: usize,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let channels = EventKind::all()
            .map(|kind| {
                let (tx, _) = broadcast::channel(capacity);
                (kind, tx)
            })
            .collect();

        Self {
            inner: Arc::new(EventBusInner { channels, capacity }),
        }
    }

    /// Publish an event to the appropriate channel
    pub fn publish(&self, event: Event) {
        let kind = event.kind();
        if let Some(tx) = self.inner.channels.get(&kind) {
            let _ = tx.send(event);
        }
    }

    /// Subscribe to a specific event kind
    pub fn subscribe(&self, kind: EventKind) -> broadcast::Receiver<Event> {
        self.inner
            .channels
            .get(&kind)
            .expect("all event kinds should have channels")
            .subscribe()
    }

    /// Subscribe to multiple event kinds
    pub fn subscribe_many(&self, kinds: &[EventKind]) -> Vec<broadcast::Receiver<Event>> {
        kinds.iter().map(|k| self.subscribe(*k)).collect()
    }
}
```

### Trade-offs: Single Channel vs Discriminant-Keyed

| Aspect | Single Channel | Discriminant-Keyed |
|--------|---------------|-------------------|
| Filtering | Client-side (match in handler) | Bus-level (per-kind channels) |
| Memory | One channel buffer | One buffer per event kind |
| Performance | One match per recv | Direct dispatch, no matching |
| Complexity | Simple | Moderate |
| High-frequency events | All handlers receive all frames | Only subscribed handlers receive |

**Recommendation**: Use discriminant-keyed channels when you have high-frequency events (like telemetry at 60+ Hz) to avoid wasting cycles on events handlers don't care about.

---

## Handler Typing Options

### Option A: Full Event Enum (Simpler)

Handler receives the full `Event` enum and matches internally:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Which event kind this handler processes
    fn handles(&self) -> EventKind;

    /// Process the event
    async fn handle(&self, event: Event, ctx: &HandlerContext);
}

struct LapHandler;

#[async_trait]
impl EventHandler for LapHandler {
    fn handles(&self) -> EventKind {
        EventKind::LapComplete
    }

    async fn handle(&self, event: Event, ctx: &HandlerContext) {
        // Must match to extract payload
        let Event::LapComplete(lap) = event else { return };

        // Process lap data...
        let metrics = extract_metrics(&lap).await;

        // Publish follow-up event
        ctx.bus.publish(Event::MetricsExtracted(Arc::new(metrics)));
    }
}
```

**Pros**:
- Simple trait definition
- Single dispatch path
- Easy to understand

**Cons**:
- Handler must match internally
- Risk of handling wrong variant if `handles()` doesn't match implementation

### Option B: Typed Payload Handler (Stricter)

Handler is generic over payload type; bus extracts payload before calling:

```rust
use async_trait::async_trait;
use std::sync::Arc;

/// Marker trait for event payloads
pub trait EventPayload: Clone + Send + Sync + 'static {
    fn event_kind() -> EventKind;
}

impl EventPayload for LapAndSession {
    fn event_kind() -> EventKind {
        EventKind::LapComplete
    }
}

#[async_trait]
pub trait TypedHandler<P: EventPayload>: Send + Sync {
    async fn handle(&self, payload: Arc<P>, ctx: &HandlerContext);
}

struct LapHandler;

#[async_trait]
impl TypedHandler<LapAndSession> for LapHandler {
    async fn handle(&self, lap: Arc<LapAndSession>, ctx: &HandlerContext) {
        // Payload is already the correct type - no matching needed
        let metrics = extract_metrics(&lap).await;
        ctx.bus.publish(Event::MetricsExtracted(Arc::new(metrics)));
    }
}
```

**Pros**:
- Type-safe at compile time
- No matching required in handler
- Handler can only receive its declared payload type

**Cons**:
- More complex registry implementation
- Need type-erased storage for heterogeneous handlers
- Requires additional trait bounds

### Recommendation

Start with **Option A (Full Event Enum)** for simplicity. The match statement is a minor cost, and the implementation is straightforward. Consider Option B if you find yourself frequently making mistakes with mismatched `handles()` declarations.

---

## Handler Publishing Alternatives

Handlers often need to publish follow-up events (e.g., after extracting metrics from a lap, publish `MetricsExtracted`). There are several patterns for enabling this:

### Pattern 1: EventBus in Context (Recommended)

Pass a clone of the EventBus in the handler context:

```rust
use std::time::Instant;

#[derive(Clone)]
pub struct HandlerContext {
    pub bus: EventBus,
    pub timestamp: Instant,
}

impl HandlerContext {
    pub fn publish(&self, event: Event) {
        self.bus.publish(event);
    }
}

// Handler can publish anytime during execution
async fn handle(&self, event: Event, ctx: &HandlerContext) {
    let result = process(event).await;
    ctx.publish(Event::ResultReady(Arc::new(result)));

    // Can publish multiple events
    ctx.publish(Event::ProcessingComplete(Arc::new(stats)));
}
```

**Pros**:
- Direct and intuitive
- Matches Python `HandlerContext` pattern
- Handler can publish at any point during execution

**Cons**:
- Handlers coupled to EventBus type
- Slightly harder to unit test (need to mock bus)

### Pattern 2: Return Events (Functional)

Handler returns events to be published:

```rust
#[async_trait]
pub trait EventHandler: Send + Sync {
    fn handles(&self) -> EventKind;
    async fn handle(&self, event: Event) -> Vec<Event>;
}

// In the event bus runner
let returned_events = handler.handle(event).await;
for event in returned_events {
    self.publish(event);
}
```

**Pros**:
- Pure handlers with no side effects
- Very easy to unit test
- No dependency on EventBus in handler

**Cons**:
- Batched publishing (events published after handler completes)
- Can't publish mid-handler (e.g., for progress updates)
- Slightly more boilerplate

### Pattern 3: mpsc Channel (Decoupled)

Each handler receives a channel sender:

```rust
use tokio::sync::mpsc;

pub struct HandlerContext {
    event_tx: mpsc::Sender<Event>,
}

impl HandlerContext {
    pub async fn publish(&self, event: Event) {
        self.event_tx.send(event).await.ok();
    }
}

// Bus aggregates from all handler channels
```

**Pros**:
- Fully decoupled from EventBus implementation
- Built-in backpressure support
- Clean separation of concerns

**Cons**:
- More infrastructure to manage
- Additional async overhead for sending
- Channel capacity management

### Pattern 4: Actor Model

Each handler is an independent actor:

```rust
use tokio::sync::mpsc;

struct LapActor {
    bus_tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
}

impl LapActor {
    async fn run(mut self) {
        while let Some(event) = self.rx.recv().await {
            if let Event::LapComplete(lap) = event {
                let metrics = extract_metrics(&lap).await;
                self.bus_tx.send(Event::MetricsExtracted(Arc::new(metrics))).await.ok();
            }
        }
    }
}
```

**Pros**:
- Maximum isolation between handlers
- Supervision trees possible (restart failed handlers)
- Natural backpressure per-actor

**Cons**:
- Most boilerplate
- Requires actor runtime or manual implementation
- Overkill for simple handlers

### Recommendation

Use **Pattern 1 (EventBus in Context)** for simplicity and parity with the Python implementation. It's the most straightforward and covers most use cases well.

---

## Handler Execution Model

### Parallel Execution

Each handler subscription spawns its own tokio task. All handlers for the same event type run in parallel automatically:

```rust
// These handlers run concurrently when TelemetryFrame is published
bus.on_telemetry_frame(|frame| async move {
    process_for_display(frame).await;
});

bus.on_telemetry_frame(|frame| async move {
    save_to_buffer(frame).await;
});
```

### Handler Registry

A centralized registry for managing handlers:

```rust
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct HandlerRegistry {
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }

    pub fn register(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    pub async fn run(&self, bus: EventBus) -> Vec<JoinHandle<()>> {
        let mut tasks = vec![];

        for handler in &self.handlers {
            let kind = handler.handles();
            let mut rx = bus.subscribe(kind);
            let handler = Arc::clone(handler);
            let ctx = HandlerContext {
                bus: bus.clone(),
                timestamp: std::time::Instant::now(),
            };

            tasks.push(tokio::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(event) => {
                            handler.handle(event, &ctx).await;
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("Handler lagged, dropped {n} events");
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }));
        }

        tasks
    }
}
```

### Sync vs Async Handlers

Support both patterns:

```rust
impl EventBus {
    /// Async handler - runs directly on tokio runtime
    pub fn on_telemetry_frame_async<F, Fut>(&self, handler: F) -> JoinHandle<()>
    where
        F: Fn(Arc<TelemetryFrame>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let mut rx = self.subscribe(EventKind::TelemetryFrame);
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if let Event::TelemetryFrame(data) = event {
                    handler(data).await;
                }
            }
        })
    }

    /// Sync handler - runs on tokio's blocking thread pool
    pub fn on_telemetry_frame_sync<F>(&self, handler: F) -> JoinHandle<()>
    where
        F: Fn(Arc<TelemetryFrame>) + Send + Sync + 'static,
    {
        let mut rx = self.subscribe(EventKind::TelemetryFrame);
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if let Event::TelemetryFrame(data) = event {
                    let handler = handler.clone();
                    tokio::task::spawn_blocking(move || handler(data))
                        .await
                        .ok();
                }
            }
        })
    }
}
```

---

## EventBus Structure

Complete EventBus implementation with discriminant-keyed channels:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct EventBus {
    inner: Arc<EventBusInner>,
}

struct EventBusInner {
    channels: HashMap<EventKind, broadcast::Sender<Event>>,
    capacity: usize,
}

#[derive(Clone)]
pub struct HandlerContext {
    pub bus: EventBus,
    pub timestamp: Instant,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let channels = EventKind::all()
            .map(|kind| {
                let (tx, _) = broadcast::channel(capacity);
                (kind, tx)
            })
            .collect();

        Self {
            inner: Arc::new(EventBusInner { channels, capacity }),
        }
    }

    /// Publish an event (sync - no await needed)
    pub fn publish(&self, event: Event) {
        let kind = event.kind();
        if let Some(tx) = self.inner.channels.get(&kind) {
            let _ = tx.send(event);
        }
    }

    /// Subscribe to a specific event kind
    pub fn subscribe(&self, kind: EventKind) -> broadcast::Receiver<Event> {
        self.inner
            .channels
            .get(&kind)
            .expect("all event kinds have channels")
            .subscribe()
    }

    /// Get a raw receiver for all events (useful for debugging)
    pub fn subscribe_all(&self) -> Vec<broadcast::Receiver<Event>> {
        self.inner
            .channels
            .values()
            .map(|tx| tx.subscribe())
            .collect()
    }

    /// Current capacity setting
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }
}
```

---

## Error Handling

### Lagging Receivers

When a receiver can't keep up, `recv()` returns `RecvError::Lagged(n)` indicating `n` messages were dropped. For telemetry, this is acceptable—we want fresh data, not backlog:

```rust
loop {
    match rx.recv().await {
        Ok(event) => { /* handle */ },
        Err(broadcast::error::RecvError::Lagged(n)) => {
            tracing::warn!("Handler lagged, dropped {n} events");
            // Continue receiving - position is now at oldest available
        },
        Err(broadcast::error::RecvError::Closed) => break,
    }
}
```

### Send Errors

`send()` only fails if there are no receivers, which is usually fine to ignore:

```rust
let _ = tx.send(event);  // Ignore if no subscribers
```

---

## Comparison Matrices

### Event Type Management Approaches

| Aspect | Single Channel (Enum) | Discriminant-Keyed | TypeId-Keyed |
|--------|----------------------|-------------------|--------------|
| Type Safety | Compile-time | Compile-time | Runtime |
| Filtering | Client-side | Bus-level | Bus-level |
| Extensibility | Closed (modify enum) | Closed (modify enum) | Open |
| Performance | One match/recv | Direct dispatch | TypeId lookup |
| Complexity | Low | Medium | High |
| Memory | One buffer | N buffers (N = event kinds) | Dynamic |

### Handler Publishing Patterns

| Pattern | Testability | Coupling | Complexity | Mid-Handler Publish |
|---------|-------------|----------|------------|---------------------|
| Context w/ Bus | Medium | High | Low | Yes |
| Return Events | High | None | Low | No |
| mpsc Channel | High | Low | Medium | Yes |
| Actor Model | High | None | High | Yes |

### Handler Typing Options

| Aspect | Full Event Enum | Typed Payload |
|--------|-----------------|---------------|
| Simplicity | Higher | Lower |
| Type Safety | Runtime (match) | Compile-time |
| Registry Complexity | Low | High |
| Boilerplate | Less | More |

---

## Resolved Design Decisions

### Macro Strategy

**Decision**: Start with manual implementations. Add `macro_rules!` only if boilerplate exceeds 3-4 event types.

Rationale: The repetitive code is straightforward, and manual implementations make debugging easier. We can add macros later without breaking changes.

### Handler Registration Style

**Decision**: Use trait-based registration (Option C from original open questions):

```rust
#[async_trait]
pub trait EventHandler: Send + Sync {
    fn handles(&self) -> EventKind;
    async fn handle(&self, event: Event, ctx: &HandlerContext);
}

// Registration
registry.register(Arc::new(LapHandler));
registry.register(Arc::new(MetricsHandler));
```

Rationale: Trait-based handlers are more flexible than closures for complex handlers that need state, and they're easier to test in isolation.

### Graceful Shutdown

**Decision**: Use `tokio_util::sync::CancellationToken`:

```rust
use tokio_util::sync::CancellationToken;

pub struct HandlerRegistry {
    handlers: Vec<Arc<dyn EventHandler>>,
    cancel_token: CancellationToken,
}

impl HandlerRegistry {
    pub async fn run(&self, bus: EventBus) -> Vec<JoinHandle<()>> {
        let mut tasks = vec![];

        for handler in &self.handlers {
            let kind = handler.handles();
            let mut rx = bus.subscribe(kind);
            let handler = Arc::clone(handler);
            let ctx = HandlerContext {
                bus: bus.clone(),
                timestamp: std::time::Instant::now(),
            };
            let token = self.cancel_token.clone();

            tasks.push(tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = token.cancelled() => break,
                        result = rx.recv() => {
                            match result {
                                Ok(event) => handler.handle(event, &ctx).await,
                                Err(broadcast::error::RecvError::Lagged(n)) => {
                                    tracing::warn!("Handler lagged, dropped {n} events");
                                }
                                Err(broadcast::error::RecvError::Closed) => break,
                            }
                        }
                    }
                }
            }));
        }

        tasks
    }

    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }
}
```

Rationale: `CancellationToken` is the standard tokio pattern for cooperative cancellation. It's clean, composable, and works well with `tokio::select!`.

---

## File Structure

Suggested module organization:

```
src/
├── lib.rs
├── main.rs
├── events/
│   ├── mod.rs           # Re-exports
│   ├── bus.rs           # EventBus implementation
│   ├── types.rs         # Event enum, EventKind, payload types
│   ├── handler.rs       # EventHandler trait, HandlerContext, registry
│   └── handlers/        # Concrete handler implementations
│       ├── mod.rs
│       ├── telemetry.rs
│       ├── lap.rs
│       └── metrics.rs
└── ...
```

---

## Dependencies

Current `Cargo.toml` already includes the necessary dependency:

```toml
tokio = { version = "1.48.0", features = ["full"] }
```

Additional recommended dependencies:

```toml
async-trait = "0.1"         # For async trait methods
tokio-util = "0.7"          # For CancellationToken
tracing = "0.1"             # For structured logging
```

---

## References

- [tokio::sync::broadcast docs](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)
- [Python event bus implementation](../../libs/racing-coach-core/src/racing_coach_core/events/base.py)
- [Implementing an Event Bus using Rust](https://blog.digital-horror.com/blog/event-bus-in-tokio/)
- [Types Over Strings: Extensible Architectures in Rust](https://willcrichton.net/notes/types-over-strings/)
- [CQRS and Event Sourcing using Rust](https://doc.rust-cqrs.org/)
- [Bevy ECS Events](https://bevy-cheatbook.github.io/programming/events.html)
