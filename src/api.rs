use crate::state::TrafficState;
use crate::storage::Storage;
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct AppState {
    pub traffic: Arc<TrafficState>,
    pub storage: Arc<Storage>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    active_connections: usize,
    total_packets: u64,
}

#[derive(Deserialize)]
pub struct HistoryParams {
    limit: Option<usize>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/live", get(get_live_stats))
        .route("/api/history", get(get_history))
        .route("/api/health", get(get_health))
        .with_state(state)
}

async fn get_health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        active_connections: state.traffic.active_connections.load(std::sync::atomic::Ordering::Relaxed),
        total_packets: state.traffic.total_packets.load(std::sync::atomic::Ordering::Relaxed),
    })
}

async fn get_live_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    // Return a snapshot of current connections
    // Limiting to top 50 for performance
    let mut connections: Vec<_> = state.traffic.connections
        .iter()
        .map(|entry| {
            let (key, stats) = entry.pair();
            serde_json::json!({
                "connection": key,
                "stats": stats
            })
        })
        .collect();
        
    // Sort by recent activity/packets (naive sort)
    connections.sort_by(|a, b| {
         let count_a = a["stats"]["packets_count"].as_u64().unwrap_or(0);
         let count_b = b["stats"]["packets_count"].as_u64().unwrap_or(0);
         count_b.cmp(&count_a)
    });
    
    connections.truncate(50);

    Json(serde_json::json!({
        "connections": connections,
        "total_packets": state.traffic.total_packets.load(std::sync::atomic::Ordering::Relaxed),
        "total_bytes": state.traffic.total_bytes.load(std::sync::atomic::Ordering::Relaxed),
    }))
}

async fn get_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryParams>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(100).min(1000);
    match state.storage.query_history(limit) {
        Ok(data) => Json(serde_json::json!(data)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
