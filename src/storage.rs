use crate::{config::*, types::TelemetryBatch};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Notify, Semaphore};
use tokio::time::sleep;
use tracing::{info, warn, error};

pub async fn storage_writer_task(
    mut batch_rx: mpsc::Receiver<TelemetryBatch>,
    db_pool: Arc<Semaphore>,
    shutdown: Arc<Notify>,
) {
    info!("💾 Storage writer started");
    
    loop {
        tokio::select! {
            // Receive batches from aggregator
            Some(batch) = batch_rx.recv() => {
                // Semaphore: Simulate limited DB connection pool
                let permit = match db_pool.try_acquire() {
                    Ok(permit) => permit,
                    Err(_) => {
                        warn!("⚠️ DB POOL EXHAUSTED - all {} connections busy, batch {} BLOCKED", 
                              DB_CONNECTION_POOL_SIZE, batch.batch_id);
                        warn!("🔄 Storage writer waiting for DB connection to become available...");
                        // Wait for a connection to become available
                        let permit = db_pool.acquire().await.unwrap();
                        info!("✅ DB connection acquired after wait for batch {}", batch.batch_id);
                        permit
                    }
                };
                
                info!("💾 Writing batch {} ({} records) to storage", 
                      batch.batch_id, batch.data.len());
                
                // Simulate database write operation with realistic timing
                let write_duration = Duration::from_millis(
                    STORAGE_WRITE_BASE_MS + (batch.data.len() as u64 * STORAGE_WRITE_PER_RECORD_MS)
                );
                sleep(write_duration).await;
                
                info!("💾 Batch {} written successfully in {:?}", 
                      batch.batch_id, write_duration);
                
                // Simulate occasional write failures for realism
                if rand::random::<f64>() < STORAGE_FAILURE_PROBABILITY {
                    error!("💾 Simulated write failure for batch {} - would retry in production", batch.batch_id);
                    // In production: implement retry logic, dead letter queue, circuit breaker
                } else {
                    info!("💾 Batch {} committed to storage successfully", batch.batch_id);
                }
                
                drop(permit); // Release DB connection
            }
            
            _ = shutdown.notified() => {
                info!("💾 Storage writer shutting down");
                break;
            }
        }
    }
}