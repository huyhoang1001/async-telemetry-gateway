// System configuration constants
pub const DEVICE_COUNT: u32 = 7;
pub const CLIENT_COUNT: u32 = 4;
pub const SYSTEM_RUNTIME_SECS: u64 = 10;

// Channel capacities
pub const TELEMETRY_CHANNEL_CAPACITY: usize = 50;
pub const STORAGE_CHANNEL_CAPACITY: usize = 10;
pub const BROADCAST_CHANNEL_CAPACITY: usize = 100;

// Resource limits
pub const DEVICE_BANDWIDTH_SLOTS: usize = 5;
pub const DB_CONNECTION_POOL_SIZE: usize = 2;

// Timing constants
pub const DEVICE_BASE_INTERVAL_MS: u64 = 1000;
pub const DEVICE_INTERVAL_VARIANCE_MS: u64 = 200;
pub const BATCH_FLUSH_INTERVAL_SECS: u64 = 2;
pub const CLIENT_DISCONNECT_PROBABILITY: f64 = 0.03;
pub const STORAGE_FAILURE_PROBABILITY: f64 = 0.05;

// Backpressure and retry
pub const DEVICE_BACKOFF_MS: u64 = 100;
pub const CLIENT_RECONNECT_DELAY_BASE_MS: u64 = 500;
pub const CLIENT_RECONNECT_DELAY_VARIANCE_MS: u64 = 1000;
pub const STORAGE_WRITE_BASE_MS: u64 = 200;
pub const STORAGE_WRITE_PER_RECORD_MS: u64 = 10;