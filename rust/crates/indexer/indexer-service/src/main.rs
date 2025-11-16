use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    serve, Json, Router,
};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tokio::signal;
use tokio::net::TcpListener;
use tracing::{info, warn};

#[derive(Clone)]
struct AppConfig {
    /// In production this would be read from env/config file.
    bind_addr: String,
    /// Network selection can be threaded through to upstream block sources.
    network: String,
}

#[derive(Clone, Serialize)]
struct TxsResponse {
    txs: Vec<TxRecord>,
}

#[derive(Clone, Serialize)]
struct TxRecord {
    height: u32,
    raw_tx: String,
    txid: String,
}

#[derive(Clone, Default)]
struct InMemoryTxStore {
    records: Arc<Vec<TxRecord>>,
}

impl InMemoryTxStore {
    fn new(records: Vec<TxRecord>) -> Self {
        Self {
            records: Arc::new(records),
        }
    }

    fn list_range(&self, from: Option<u32>, to: Option<u32>, limit: usize) -> Vec<TxRecord> {
        self.records
            .iter()
            .filter(|tx| match (from, to) {
                (Some(start), Some(end)) => tx.height >= start && tx.height <= end,
                (Some(start), None) => tx.height >= start,
                (None, Some(end)) => tx.height <= end,
                (None, None) => true,
            })
            .take(limit)
            .cloned()
            .collect()
    }
}

#[derive(Deserialize, Debug)]
struct RangeQuery {
    from_height: Option<u32>,
    to_height: Option<u32>,
    limit: Option<usize>,
}

#[derive(Clone)]
struct AppState {
    store: InMemoryTxStore,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let config = AppConfig {
        bind_addr: "0.0.0.0:8080".to_string(),
        network: "mainnet".into(),
    };

    // Production TODO: Replace bootstrap data with real ingestion that tails a Zcash source
    // (lightwalletd gRPC or zcashd RPC) and persists to durable storage.
    let store = InMemoryTxStore::new(bootstrap_sample_data());
    let state = AppState { store };

    let app = Router::new()
        .route("/api/txs", get(list_txs))
        .with_state(state);

    info!(
        "Starting indexer-service on {} (network {})",
        config.bind_addr, config.network
    );
    let listener = TcpListener::bind(&config.bind_addr).await?;
    serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn list_txs(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(500).min(5_000);
    let txs = state
        .store
        .list_range(query.from_height, query.to_height, limit);
    Json(TxsResponse { txs })
}

fn bootstrap_sample_data() -> Vec<TxRecord> {
    // Production TODO: remove this and stream from chain source; ensure tx bytes are validated.
    let dummy_tx_bytes = b"placeholder-zcash-tx";
    vec![TxRecord {
        height: 0,
        raw_tx: STANDARD.encode(dummy_tx_bytes),
        txid: hex::encode(blake2b_simd::Params::new().hash_length(32).to_state().finalize()),
    }]
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(
            "indexer_service=info".parse().unwrap_or_else(|_| "info".parse().unwrap()),
        ))
        .init();
}

async fn shutdown_signal() {
    // Production TODO: add health endpoints and structured shutdown hooks to close DB connections.
    #[cfg(unix)]
    {
        tokio::select! {
            _ = signal::ctrl_c() => {},
            _ = sigterm() => {},
        }
    }

    #[cfg(not(unix))]
    {
        let _ = signal::ctrl_c().await;
    }
    warn!("Shutdown signal received, stopping server");
}

#[cfg(unix)]
async fn sigterm() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm =
        signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
    sigterm.recv().await;
}
