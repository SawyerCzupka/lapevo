//! Extensions for the pitwall crate
//!
//! Provides accelerated replay functionality using pitwall's public API.

mod accelerated_connection;
mod accelerated_provider;

pub use accelerated_connection::AcceleratedReplayConnection;
