use crate::{config::*, types::TelemetryData};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Notify, Semaphore};
use tokio::time::{interval, sleep};
use tracing::{info, warn, error};

pub async fn device_task(
    device_id: u32,
    tx: mpsc::Sender<TelemetryData>,
    semaphore: Arc<Semaphore>,
    shutdown: Arc<Notify>,
) {
    info!("📱 Device {} started", device_id);
    
    // Each device has slightly different intervals to create realistic variance
    let base_interval = DEVICE_BASE_INTERVAL_MS + (device_id as u64 * DEVICE_INTERVAL_VARIANCE_MS);
    let mut interval = interval(Duration::from_millis(base_interval));
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Semaphore: Limit concurrent device operations (bandwidth/slot limits)
                let permit = match semaphore.try_acquire() {
                    Ok(permit) => permit,
                    Err(_) => {
                        warn!("📱 Device {} waiting for bandwidth slot", device_id);
                        // Backpressure: exponential backoff
                        sleep(Duration::from_millis(DEVICE_BACKOFF_MS)).await;
                        continue;
                    }
                };
                
                // Generate realistic telemetry data
                let telemetry = TelemetryData {
                    device_id,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    temperature: 18.0 + rand::random::<f64>() * 15.0, // 18-33°C
                    humidity: 30.0 + rand::random::<f64>() * 40.0,    // 30-70%
                    battery_level: (80 + rand::random::<u8>() % 20),  // 80-99%
                };
                
                // Send via mpsc to aggregator (many devices → one aggregator)
                match tx.send(telemetry.clone()).await {
                    Ok(_) => {
                        info!("📱 Device {} sent: temp={:.1}°C, humidity={:.1}%", 
                              device_id, telemetry.temperature, telemetry.humidity);
                    }
                    Err(_) => {
                        error!("📱 Device {} failed to send telemetry - channel closed", device_id);
                        break;
                    }
                }
                
                drop(permit); // Release semaphore permit
            }
            _ = shutdown.notified() => {
                info!("📱 Device {} shutting down", device_id);
                break;
            }
        }
    }
}