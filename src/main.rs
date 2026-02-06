use clap::Parser;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod sniffer;
mod state;
mod storage;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Network interface to capture on (e.g., eth0). Auto-detects if not provided.
    #[arg(short, long)]
    interface: Option<String>,

    /// Port to serve the API on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Database path
    #[arg(long, default_value = "traffic.db")]
    db_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Signal handler
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    // Channels
    let (tx, rx) = mpsc::channel(10000);

    // State & Storage
    let traffic_state = Arc::new(state::TrafficState::new());
    let storage = Arc::new(storage::Storage::new(&args.db_path)?);

    // Spawn Writer Task
    let storage_clone = storage.clone();
    tokio::spawn(async move {
        storage_clone.run_writer(rx).await;
    });
    

    let tx_clone = tx.clone();
    let interface = args.interface.clone();
    let running_sniffer = running.clone();
    let traffic_state_clone = traffic_state.clone();
    
    std::thread::spawn(move || {
        sniffer::start_sniffer(interface, tx_clone, running_sniffer, traffic_state_clone);
    });

    
    // API
    let app_state = Arc::new(api::AppState {
        traffic: traffic_state.clone(),
        storage: storage.clone(),
    });

    let app = api::router(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    tracing::info!("Server running on http://0.0.0.0:{}", args.port);
    axum::serve(listener, app).await?;

    Ok(())
}
