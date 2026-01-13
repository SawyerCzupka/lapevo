//! Metrics computation and upload handler.
//!
//! This handler subscribes to `LapComplete` events, computes corner and braking
//! metrics using the algorithms in the `algs` module, and uploads the metrics
//! to the server.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::algs::{extract_lap_metrics, CornerDetectionMode, CornerSegment, ExtractConfig};
use crate::api::{ServerAPIClient, TrackBoundaryListResponse, TrackBoundaryResponse};
use crate::events::{LapCompletePayload, RacingEvent, RacingEventKind};
use eventbus::{EventHandler, HandlerContext};

/// Handler that computes and uploads lap metrics.
///
/// On receiving a `LapComplete` event, this handler:
/// 1. Fetches corner segments from the server (if not cached)
/// 2. Extracts braking and corner metrics from the lap telemetry
/// 3. Uploads the computed metrics to the server
pub struct MetricsHandler {
    client: Arc<ServerAPIClient>,
    /// Cache of corner segments per track_id.
    corner_segments_cache: Mutex<HashMap<i32, CachedTrackData>>,
}

/// Cached data for a track.
struct CachedTrackData {
    boundary_id: Uuid,
    segments: Vec<CornerSegment>,
    boundary: Option<TrackBoundaryResponse>,
}

impl MetricsHandler {
    pub fn new(client: Arc<ServerAPIClient>) -> Self {
        Self {
            client,
            corner_segments_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Fetch and cache corner segments for a track.
    async fn get_track_data(&self, track_id: i32) -> Option<CachedTrackData> {
        // Check cache first
        {
            let cache = self.corner_segments_cache.lock().await;
            if let Some(data) = cache.get(&track_id) {
                return Some(CachedTrackData {
                    boundary_id: data.boundary_id,
                    segments: data.segments.clone(),
                    boundary: data.boundary.clone(),
                });
            }
        }

        // Fetch from server
        let track_data = self.fetch_track_data(track_id).await;

        // Cache the result (even if empty)
        if let Some(ref data) = track_data {
            let mut cache = self.corner_segments_cache.lock().await;
            cache.insert(track_id, CachedTrackData {
                boundary_id: data.boundary_id,
                segments: data.segments.clone(),
                boundary: data.boundary.clone(),
            });
        }

        track_data
    }

    /// Fetch track boundary and corner segments from the server.
    async fn fetch_track_data(&self, track_id: i32) -> Option<CachedTrackData> {
        // 1. Fetch list of track boundaries
        let boundaries: TrackBoundaryListResponse = match self.client.fetch_track_boundaries().await
        {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Failed to fetch track boundaries: {}", e);
                return None;
            }
        };

        // 2. Find boundary for this track
        let boundary_summary = boundaries
            .boundaries
            .iter()
            .find(|b| b.track_id == track_id)?;

        let boundary_id = boundary_summary.id;
        debug!("Found track boundary {} for track_id {}", boundary_id, track_id);

        // 3. Fetch full boundary data (for lateral position calculation)
        let boundary: Option<TrackBoundaryResponse> =
            match self.client.fetch_track_boundary(boundary_id).await {
                Ok(b) => Some(b),
                Err(e) => {
                    warn!("Failed to fetch track boundary details: {}", e);
                    None
                }
            };

        // 4. Fetch corner segments
        let segments: Vec<CornerSegment> =
            match self.client.fetch_corner_segments(boundary_id).await {
                Ok(resp) => resp.corners.into_iter().map(CornerSegment::from).collect(),
                Err(e) => {
                    debug!("No corner segments for track (will use auto-detect): {}", e);
                    Vec::new()
                }
            };

        Some(CachedTrackData {
            boundary_id,
            segments,
            boundary,
        })
    }
}

#[async_trait]
impl EventHandler<RacingEvent> for MetricsHandler {
    fn handles(&self) -> RacingEventKind {
        RacingEventKind::LapComplete
    }

    fn name(&self) -> &'static str {
        "MetricsHandler"
    }

    async fn handle(&self, event: RacingEvent, _ctx: &HandlerContext<RacingEvent>) {
        let RacingEvent::LapComplete(payload) = event else {
            return;
        };

        let LapCompletePayload {
            lap_number,
            lap_time,
            frames,
            lap_id,
            session,
            ..
        } = payload;

        // Skip if no frames
        if frames.is_empty() {
            debug!("Skipping metrics for lap {} - no frames", lap_number);
            return;
        }

        info!(
            "Computing metrics for lap {} ({} frames, lap_id: {})",
            lap_number,
            frames.len(),
            lap_id
        );

        // Fetch track data (corner segments and boundary)
        let track_data = self.get_track_data(session.track_id).await;

        let (segments, boundary) = match track_data {
            Some(data) => {
                debug!(
                    "Using {} corner segments for track {}",
                    data.segments.len(),
                    session.track_id
                );
                (Some(data.segments), data.boundary)
            }
            None => {
                debug!("No track data available, using auto-detect for corners");
                (None, None)
            }
        };

        // Extract metrics
        let config = ExtractConfig::default();
        let metrics_result = extract_lap_metrics(
            &frames,
            Some(lap_number),
            lap_time,
            &config,
            segments.as_deref(),
            boundary.as_ref(),
            CornerDetectionMode::SegmentsWithFallback,
        );

        let metrics = match metrics_result {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to extract metrics for lap {}: {}", lap_number, e);
                return;
            }
        };

        info!(
            "Lap {} metrics: {} braking zones, {} corners, max speed: {:.1} m/s",
            lap_number, metrics.total_braking_zones, metrics.total_corners, metrics.max_speed
        );

        // Upload metrics to server
        match self.client.upload_lap_metrics(&metrics, lap_id).await {
            Ok(response) => {
                info!(
                    "Uploaded metrics for lap {} (metrics_id: {})",
                    lap_number, response.lap_metrics_id
                );
            }
            Err(e) => {
                error!("Failed to upload metrics for lap {}: {}", lap_number, e);
                // Continue processing - don't block future laps
            }
        }
    }
}
