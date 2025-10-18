use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SystemMetrics {
    pub messages_processed: Arc<AtomicU64>,
    pub batches_written: Arc<AtomicU64>,
    pub client_lag_events: Arc<AtomicU64>,
    pub backpressure_events: Arc<AtomicU64>,
    pub storage_failures: Arc<AtomicU64>,
}

#[allow(dead_code)]
impl SystemMetrics {
    pub fn new() -> Self {
        Self {
            messages_processed: Arc::new(AtomicU64::new(0)),
            batches_written: Arc::new(AtomicU64::new(0)),
            client_lag_events: Arc::new(AtomicU64::new(0)),
            backpressure_events: Arc::new(AtomicU64::new(0)),
            storage_failures: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment_messages(&self) {
        self.messages_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_batches(&self) {
        self.batches_written.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_lag_events(&self) {
        self.client_lag_events.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_backpressure(&self) {
        self.backpressure_events.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_storage_failures(&self) {
        self.storage_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn print_summary(&self) {
        println!("\n📊 SYSTEM METRICS SUMMARY:");
        println!("   Messages processed: {}", self.messages_processed.load(Ordering::Relaxed));
        println!("   Batches written: {}", self.batches_written.load(Ordering::Relaxed));
        println!("   Client lag events: {}", self.client_lag_events.load(Ordering::Relaxed));
        println!("   Backpressure events: {}", self.backpressure_events.load(Ordering::Relaxed));
        println!("   Storage failures: {}", self.storage_failures.load(Ordering::Relaxed));
    }
}