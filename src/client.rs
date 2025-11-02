use crate::{config::*, types::SystemSnapshot};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, watch, Notify};
use tokio::time::sleep;
use tracing::{info, warn, error};

pub async fn client_task(
    client_id: u32,
    mut broadcast_rx: broadcast::Receiver<SystemSnapshot>,
    snapshot_rx: watch::Receiver<SystemSnapshot>,
    shutdown: Arc<Notify>,
) {
    info!("👤 Client {} connected", client_id);
    
    // On connect, get latest snapshot from watch channel
    let current_snapshot = snapshot_rx.borrow().clone();
    info!("👤 Client {} got initial snapshot: {} devices, {} total messages", 
          client_id, current_snapshot.active_devices, current_snapshot.total_messages);
    
    loop {
        tokio::select! {
            // Receive real-time updates via broadcast
            result = broadcast_rx.recv() => {
                match result {
                    Ok(snapshot) => {
                        info!("👤 Client {} received update: {} devices active, avg temps: {}", 
                              client_id, 
                              snapshot.active_devices,
                              snapshot.device_metrics.values()
                                  .map(|m| format!("{:.1}°C", m.avg_temperature))
                                  .collect::<Vec<_>>()
                                  .join(", "));
                        
                        // Simulate random client disconnection/reconnection
                        if rand::random::<f64>() < CLIENT_DISCONNECT_PROBABILITY {
                            info!("👤 Client {} temporarily disconnecting", client_id);
                            let reconnect_delay = CLIENT_RECONNECT_DELAY_BASE_MS + 
                                (rand::random::<u64>() % CLIENT_RECONNECT_DELAY_VARIANCE_MS);
                            sleep(Duration::from_millis(reconnect_delay)).await;
                            
                            // On reconnect, get latest state from watch channel
                            let latest = snapshot_rx.borrow().clone();
                            info!("👤 Client {} reconnected, caught up to {} total messages", 
                                  client_id, latest.total_messages);
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("⚠️ CLIENT LAG - Client {} fell behind by {} messages (buffer size: {})", 
                              client_id, n, BROADCAST_CHANNEL_CAPACITY);
                        warn!("🔄 Client {} missed updates due to slow processing - using watch channel to resync", client_id);
                        // Get current state from watch channel to catch up
                        let current = snapshot_rx.borrow().clone();
                        info!("✅ Client {} resynced: {} total messages now", 
                              client_id, current.total_messages);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        error!("👤 Client {} - broadcast channel closed unexpectedly", client_id);
                        break;
                    }
                }
            }
            
            _ = shutdown.notified() => {
                info!("👤 Client {} shutting down", client_id);
                break;
            }
        }
    }
}