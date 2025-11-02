// System configuration constants
pub const DEVICE_COUNT: u32 = 7;
pub const CLIENT_COUNT: u32 = 4;
pub const SYSTEM_RUNTIME_SECS: u64 = 10;

// Channel capacities - reduced to show backpressure
pub const TELEMETRY_CHANNEL_CAPACITY: usize = 5;  // Very small to cause blocking
pub const STORAGE_CHANNEL_CAPACITY: usize = 2;    // Tiny to force batching delays
pub const BROADCAST_CHANNEL_CAPACITY: usize = 10; // Small to show lag warnings

// Resource limits - very restrictive
pub const DEVICE_BANDWIDTH_SLOTS: usize = 2;      // Only 2 devices can send at once
pub const DB_CONNECTION_POOL_SIZE: usize = 1;     // Single DB connection

// Timing constants - faster sending to create pressure
pub const DEVICE_BASE_INTERVAL_MS: u64 = 300;     // Send every 300ms instead of 1000ms
pub const DEVICE_INTERVAL_VARIANCE_MS: u64 = 50;   // Less variance for more consistent pressure
pub const BATCH_FLUSH_INTERVAL_SECS: u64 = 2;
pub const CLIENT_DISCONNECT_PROBABILITY: f64 = 0.03;
pub const STORAGE_FAILURE_PROBABILITY: f64 = 0.05;

// Backpressure and retry
pub const DEVICE_BACKOFF_MS: u64 = 100;
pub const CLIENT_RECONNECT_DELAY_BASE_MS: u64 = 500;
pub const CLIENT_RECONNECT_DELAY_VARIANCE_MS: u64 = 1000;
pub const STORAGE_WRITE_BASE_MS: u64 = 200;
pub const STORAGE_WRITE_PER_RECORD_MS: u64 = 10;