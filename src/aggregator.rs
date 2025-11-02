use crate::{config::*, types::{DeviceMetrics, SystemSnapshot, TelemetryBatch, TelemetryData}};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{broadcast, mpsc, watch, Notify};
use tracing::{info, warn, error};

pub struct Aggregator {
    // mpsc: Receive telemetry from many devices
    telemetry_rx: mpsc::Receiver<TelemetryData>,
    // watch: Share latest system snapshot
    snapshot_tx: watch::Sender<SystemSnapshot>,
    // broadcast: Send updates to many clients
    broadcast_tx: broadcast::Sender<SystemSnapshot>,
    // mpsc: Send batches to storage (bounded for backpressure)
    storage_tx: mpsc::Sender<TelemetryBatch>,
    shutdown: Arc<Notify>,
    
    // Internal state
    device_metrics: HashMap<u32, DeviceMetrics>,
    total_messages: u64,
    batch_buffer: Vec<TelemetryData>,
    batch_id: u64,
}

impl Aggregator {
    pub fn new(
        telemetry_rx: mpsc::Receiver<TelemetryData>,
        snapshot_tx: watch::Sender<SystemSnapshot>,
        broadcast_tx: broadcast::Sender<SystemSnapshot>,
        storage_tx: mpsc::Sender<TelemetryBatch>,
        shutdown: Arc<Notify>,
    ) -> Self {
        Self {
            telemetry_rx,
            snapshot_tx,
            broadcast_tx,
            storage_tx,
            shutdown,
            device_metrics: HashMap::new(),
            total_messages: 0,
            batch_buffer: Vec::new(),
            batch_id: 0,
        }
    }
    
    pub async fn run(&mut self) {
        info!("🔄 Aggregator started");
        
        // Batch timer - send batches at configured interval
        let mut batch_interval = tokio::time::interval(std::time::Duration::from_secs(BATCH_FLUSH_INTERVAL_SECS));
        
        loop {
            tokio::select! {
                // Receive telemetry from devices via mpsc
                Some(telemetry) = self.telemetry_rx.recv() => {
                    self.process_telemetry(telemetry).await;
                }
                
                // Batch timer - send accumulated data to storage
                _ = batch_interval.tick() => {
                    self.flush_batch().await;
                }
                
                // Shutdown signal
                _ = self.shutdown.notified() => {
                    info!("🔄 Aggregator shutting down");
                    self.flush_batch().await; // Final batch
                    break;
                }
            }
        }
    }
    
    async fn process_telemetry(&mut self, telemetry: TelemetryData) {
        self.total_messages += 1;
        self.batch_buffer.push(telemetry.clone());
        
        info!("🔄 Aggregator processed telemetry from device {}", telemetry.device_id);
        
        // Update device metrics
        let metrics = self.device_metrics
            .entry(telemetry.device_id)
            .or_insert(DeviceMetrics {
                device_id: telemetry.device_id,
                last_seen: telemetry.timestamp,
                avg_temperature: telemetry.temperature,
                avg_humidity: telemetry.humidity,
                message_count: 0,
            });
        
        // Update running averages
        let count = metrics.message_count as f64;
        metrics.avg_temperature = (metrics.avg_temperature * count + telemetry.temperature) / (count + 1.0);
        metrics.avg_humidity = (metrics.avg_humidity * count + telemetry.humidity) / (count + 1.0);
        metrics.message_count += 1;
        metrics.last_seen = telemetry.timestamp;
        
        // Create system snapshot
        let snapshot = SystemSnapshot {
            active_devices: self.device_metrics.len(),
            total_messages: self.total_messages,
            device_metrics: self.device_metrics.clone(),
            last_update: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Share via watch channel (latest state)
        let _ = self.snapshot_tx.send(snapshot.clone());
        
        // Broadcast to clients (one → many)
        match self.broadcast_tx.send(snapshot) {
            Ok(count) => info!("📡 Broadcasted update to {} clients", count),
            Err(_) => warn!("📡 No clients listening to broadcasts"),
        }
    }
    
    async fn flush_batch(&mut self) {
        if self.batch_buffer.is_empty() {
            return;
        }
        
        self.batch_id += 1;
        let batch = TelemetryBatch {
            batch_id: self.batch_id,
            data: self.batch_buffer.drain(..).collect(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        info!("📦 Flushing batch {} with {} records", batch.batch_id, batch.data.len());
        
        // Send to storage via bounded mpsc (backpressure simulation)
        match self.storage_tx.try_send(batch) {
            Ok(_) => info!("📦 Batch sent to storage"),
            Err(mpsc::error::TrySendError::Full(dropped_batch)) => {
                warn!("⚠️ STORAGE BACKPRESSURE - queue full ({} capacity), batch {} with {} records BLOCKED", 
                      STORAGE_CHANNEL_CAPACITY, dropped_batch.batch_id, dropped_batch.data.len());
                // Try blocking send to show the backpressure effect
                match self.storage_tx.send(dropped_batch).await {
                    Ok(_) => {
                        warn!("✅ Batch finally sent to storage after backpressure wait");
                    }
                    Err(_) => {
                        error!("📦 Storage channel closed during backpressure wait");
                    }
                }
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                error!("📦 Storage channel closed unexpectedly");
            }
        }
    }
}