use crate::{config::*, types::TelemetryData};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot, Notify, Semaphore};
use tokio::time::{interval, sleep};
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub struct DeviceStatus {
    pub device_id: u32,
    pub uptime_seconds: u64,
    #[allow(dead_code)]
    pub last_telemetry_sent: u64,
    pub battery_level: u8,
}

pub async fn device_task(
    device_id: u32,
    tx: mpsc::Sender<TelemetryData>,
    mut health_check_rx: mpsc::Receiver<oneshot::Sender<DeviceStatus>>,
    semaphore: Arc<Semaphore>,
    shutdown: Arc<Notify>,
) {
    info!("📱 Device {} started", device_id);
    
    let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut last_telemetry_sent = start_time;
    let mut current_battery = 80 + rand::random::<u8>() % 20;
    
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
                        warn!("⚠️ Device {} BLOCKED - all {} bandwidth slots occupied, backing off {}ms", 
                              device_id, DEVICE_BANDWIDTH_SLOTS, DEVICE_BACKOFF_MS);
                        // Backpressure: exponential backoff
                        sleep(Duration::from_millis(DEVICE_BACKOFF_MS)).await;
                        // Try to acquire with blocking wait
                        match semaphore.acquire().await {
                            Ok(permit) => {
                                warn!("✅ Device {} acquired bandwidth slot after wait", device_id);
                                permit
                            }
                            Err(_) => {
                                error!("📱 Device {} semaphore closed", device_id);
                                break;
                            }
                        }
                    }
                };
                
                // Update battery level occasionally
                if rand::random::<f64>() < 0.1 {
                    current_battery = current_battery.saturating_sub(1);
                }
                
                // Generate realistic telemetry data
                let telemetry = TelemetryData {
                    device_id,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    temperature: 18.0 + rand::random::<f64>() * 15.0, // 18-33°C
                    humidity: 30.0 + rand::random::<f64>() * 40.0,    // 30-70%
                    battery_level: current_battery,
                };
                
                last_telemetry_sent = telemetry.timestamp;
                
                // Send via mpsc to aggregator (many devices → one aggregator)
                // Try non-blocking first to detect channel pressure
                match tx.try_send(telemetry.clone()) {
                    Ok(_) => {
                        info!("📱 Device {} sent: temp={:.1}°C, humidity={:.1}%", 
                              device_id, telemetry.temperature, telemetry.humidity);
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        warn!("⚠️ Device {} blocked - telemetry channel full ({}), waiting...", 
                              device_id, TELEMETRY_CHANNEL_CAPACITY);
                        // Fall back to blocking send
                        match tx.send(telemetry.clone()).await {
                            Ok(_) => {
                                info!("📱 Device {} sent after wait: temp={:.1}°C, humidity={:.1}%", 
                                      device_id, telemetry.temperature, telemetry.humidity);
                            }
                            Err(_) => {
                                error!("📱 Device {} failed to send telemetry - channel closed", device_id);
                                break;
                            }
                        }
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        error!("📱 Device {} failed to send telemetry - channel closed", device_id);
                        break;
                    }
                }
                
                drop(permit); // Release semaphore permit
            }
            
            // oneshot: Handle health check requests
            Some(response_tx) = health_check_rx.recv() => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                let status = DeviceStatus {
                    device_id,
                    uptime_seconds: now - start_time,
                    last_telemetry_sent,
                    battery_level: current_battery,
                };
                
                // oneshot: Send response back (single use, one-time communication)
                if let Err(_) = response_tx.send(status) {
                    warn!("📱 Device {} health check response failed - receiver dropped", device_id);
                }
            }
            _ = shutdown.notified() => {
                info!("📱 Device {} shutting down", device_id);
                break;
            }
        }
    }
}