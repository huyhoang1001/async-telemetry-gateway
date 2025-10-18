mod aggregator;
mod client;
mod config;
mod device;
mod metrics;
mod storage;
mod types;

use aggregator::Aggregator;
use client::client_task;
use config::*;
use device::device_task;
use metrics::SystemMetrics;
use storage::storage_writer_task;
use types::{SystemSnapshot, TelemetryBatch, TelemetryData};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, watch, Notify, Semaphore};
use tokio::task::JoinSet;
use tokio::time::sleep;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize custom tracing logger
    custom_tracing_logger::init();

    info!("🚀 Starting Telemetry Gateway");

    // Shutdown coordination via Notify
    let shutdown = Arc::new(Notify::new());
    
    // System metrics tracking
    let _metrics = SystemMetrics::new();
    
    // Channel setup
    // mpsc: Devices → Aggregator (many to one, bounded for backpressure)
    let (telemetry_tx, telemetry_rx) = mpsc::channel::<TelemetryData>(TELEMETRY_CHANNEL_CAPACITY);
    
    // watch: Latest system snapshot sharing
    let initial_snapshot = SystemSnapshot {
        active_devices: 0,
        total_messages: 0,
        device_metrics: HashMap::new(),
        last_update: 0,
    };
    let (snapshot_tx, snapshot_rx) = watch::channel(initial_snapshot);
    
    // broadcast: Aggregator → Clients (one to many)
    let (broadcast_tx, _) = broadcast::channel::<SystemSnapshot>(BROADCAST_CHANNEL_CAPACITY);
    
    // mpsc: Aggregator → Storage (bounded for backpressure simulation)
    let (storage_tx, storage_rx) = mpsc::channel::<TelemetryBatch>(STORAGE_CHANNEL_CAPACITY);
    
    // Semaphores for concurrency control
    let device_semaphore = Arc::new(Semaphore::new(DEVICE_BANDWIDTH_SLOTS)); // Bandwidth/slot limits
    let db_pool = Arc::new(Semaphore::new(DB_CONNECTION_POOL_SIZE)); // DB connection pool
    
    // JoinSet: Track all spawned tasks for graceful shutdown
    let mut join_set = JoinSet::new();
    
    // Spawn aggregator task
    let mut aggregator = Aggregator::new(
        telemetry_rx,
        snapshot_tx,
        broadcast_tx.clone(),
        storage_tx,
        shutdown.clone(),
    );
    join_set.spawn(async move {
        aggregator.run().await;
    });
    
    // Spawn storage writer task
    join_set.spawn({
        let db_pool = db_pool.clone();
        let shutdown = shutdown.clone();
        async move {
            storage_writer_task(storage_rx, db_pool, shutdown).await;
        }
    });
    
    // Spawn device tasks
    for device_id in 1..=DEVICE_COUNT {
        let tx = telemetry_tx.clone();
        let semaphore = device_semaphore.clone();
        let shutdown_clone = shutdown.clone();
        
        join_set.spawn(async move {
            device_task(device_id, tx, semaphore, shutdown_clone).await;
        });
    }
    
    // Spawn client tasks
    for client_id in 1..=CLIENT_COUNT {
        let broadcast_rx = broadcast_tx.subscribe();
        let snapshot_rx = snapshot_rx.clone();
        let shutdown_clone = shutdown.clone();
        
        join_set.spawn(async move {
            client_task(client_id, broadcast_rx, snapshot_rx, shutdown_clone).await;
        });
    }
    
    info!("⏰ System running for 10 seconds...");
    info!("📊 Watch for device backpressure, storage batching, and client updates");
    
    // Run the system for configured duration
    sleep(Duration::from_secs(SYSTEM_RUNTIME_SECS)).await;
    
    info!("🛑 Initiating graceful shutdown...");
    
    // Trigger shutdown for all tasks via Notify
    shutdown.notify_waiters();
    
    // Wait for all tasks to complete using JoinSet
    let mut completed = 0;
    while let Some(result) = join_set.join_next().await {
        completed += 1;
        if let Err(e) = result {
            error!("Task failed during shutdown: {:?}", e);
        }
    }
    
    info!("✅ All {} tasks completed gracefully", completed);
    
    // Print system metrics
    // metrics.print_summary(); // TODO: Integrate metrics tracking
    
    // Print summary of async primitives demonstrated
    println!("\n🎯 ASYNC PRIMITIVES IN TELEMETRY GATEWAY:");
    println!("📨 mpsc::channel     - Devices→Aggregator (bounded, backpressure)");
    println!("📨 mpsc::channel     - Aggregator→Storage (batching, bounded)");
    println!("📡 broadcast::channel - Aggregator→Clients (real-time updates)");
    println!("👁️  watch::channel    - Latest system snapshot sharing");
    println!("🚀 tokio::spawn      - Concurrent tasks for each component");
    println!("🎯 JoinSet           - Track and gracefully shutdown all tasks");
    println!("🚦 Semaphore         - Device bandwidth limits & DB connection pool");
    println!("🔔 Notify            - Coordinate shutdown across all components");
    println!("⏱️  tokio::select!    - Handle multiple async operations per task");
    println!("\n💡 Key Learnings:");
    println!("   • Bounded channels create natural backpressure");
    println!("   • watch channels provide consistent state snapshots");
    println!("   • broadcast handles subscriber churn gracefully");
    println!("   • Semaphores control resource contention realistically");
    
    Ok(())
}