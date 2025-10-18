use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub device_id: u32,
    pub timestamp: u64,
    pub temperature: f64,
    pub humidity: f64,
    pub battery_level: u8,
}

#[derive(Debug, Clone)]
pub struct DeviceMetrics {
    #[allow(dead_code)]
    pub device_id: u32,
    pub last_seen: u64,
    pub avg_temperature: f64,
    pub avg_humidity: f64,
    pub message_count: u64,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub active_devices: usize,
    pub total_messages: u64,
    pub device_metrics: HashMap<u32, DeviceMetrics>,
    #[allow(dead_code)]
    pub last_update: u64,
}

#[derive(Debug, Clone)]
pub struct TelemetryBatch {
    pub batch_id: u64,
    pub data: Vec<TelemetryData>,
    #[allow(dead_code)]
    pub created_at: u64,
}