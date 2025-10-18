# Async Telemetry Gateway

A production-ready async Rust application demonstrating all major Tokio primitives working together in a realistic telemetry ingestion system.

## 🎯 Overview

This telemetry gateway simulates a real-world IoT system that collects sensor data from multiple devices, aggregates it, stores it in batches, and broadcasts real-time updates to connected clients. It showcases how different async primitives solve specific architectural challenges.

## 🚀 Features

- **Multi-device simulation** - 7 concurrent sensor devices sending temperature/humidity data
- **Real-time aggregation** - Central hub processing and maintaining running averages
- **Batch storage** - Efficient database writes with configurable batch sizes
- **Live client updates** - 4 concurrent clients receiving real-time broadcasts
- **Graceful shutdown** - Clean termination of all concurrent tasks
- **Backpressure handling** - Bounded channels prevent memory overflow
- **Resource management** - Semaphore-controlled bandwidth and connection pooling

## 🔧 Async Primitives Demonstrated

| Primitive | Usage | Purpose |
|-----------|-------|---------|
| `mpsc::channel` | Device→Aggregator, Aggregator→Storage | Fan-in data collection, batching pipeline |
| `broadcast::channel` | Aggregator→Clients | Real-time updates to multiple subscribers |
| `watch::channel` | System state sharing | Consistent snapshots across components |
| `Semaphore` | Device bandwidth, DB connections | Resource contention control |
| `Notify` | Shutdown coordination | Cross-component signaling |
| `JoinSet` | Task management | Structured concurrency and cleanup |
| `tokio::select!` | Event handling | Multiple async operations per task |

## 📊 System Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Device 1  │───▶│              │───▶│   Storage   │
│   Device 2  │───▶│  Aggregator  │───▶│   Writer    │
│   Device N  │───▶│              │───▶│             │
└─────────────┘    └──────┬───────┘    └─────────────┘
                          │
                          ▼ (broadcast)
                   ┌─────────────┐
                   │  Client 1   │
                   │  Client 2   │
                   │  Client N   │
                   └─────────────┘
```

## 🛠️ Quick Start

### Prerequisites
- Rust 1.70+ with Cargo
- Windows/Linux/macOS

### Build & Run
```bash
# Clone and navigate
cd async-sensor-hub

# Build optimized release
cargo build --release

# Run the telemetry gateway
cargo run --release
```

The system runs for 10 seconds, demonstrating all async patterns with structured logging output.

## 📈 Expected Output

```
🚀 Starting Telemetry Gateway
📊 Watch for device backpressure, storage batching, and client updates

📱 Device 1 sent: temp=20.3°C, humidity=55.7%
🔄 Aggregator processed telemetry from device 1
📡 Broadcasted update to 4 clients
👤 Client 1 received update: 1 devices active, avg temps: 20.3°C
💾 Writing batch 1 (1 records) to storage
```

## 🔧 Configuration

All system parameters are centralized in `src/config.rs`:

```rust
pub const TELEMETRY_CHANNEL_CAPACITY: usize = 50;    // Device→Aggregator
pub const STORAGE_CHANNEL_CAPACITY: usize = 10;      // Aggregator→Storage  
pub const BROADCAST_CAPACITY: usize = 100;           // Real-time updates
pub const STORAGE_BATCH_SIZE: usize = 10;            // Batch optimization
pub const DEVICE_BANDWIDTH_LIMIT: usize = 3;         // Concurrent sends
```

## 📁 Project Structure

```
src/
├── main.rs         # Orchestration & graceful shutdown
├── config.rs       # Centralized configuration
├── types.rs        # Data structures
├── device.rs       # Sensor simulation
├── aggregator.rs   # Central processing hub
├── storage.rs      # Batch storage writer
├── client.rs       # Real-time subscribers
└── metrics.rs      # System metrics framework
```

## 🎓 Learning Outcomes

This project demonstrates:

- **Channel Selection**: When to use mpsc vs broadcast vs watch
- **Backpressure Design**: How bounded channels create natural flow control
- **Resource Management**: Semaphore patterns for realistic constraints
- **Structured Concurrency**: JoinSet for clean task lifecycle management
- **Error Handling**: Graceful degradation and recovery patterns
- **Configuration Management**: Centralized, maintainable system parameters

## 📝 Blog Post

See `blog.md` for a detailed technical explanation of how these async primitives work together in real-world systems.

## 🔍 Key Metrics

During a typical 10-second run:
- **~40 telemetry records** processed
- **5 storage batches** written
- **4 concurrent clients** served
- **Zero data loss** with bounded channels
- **Clean shutdown** of all 13 tasks

## 📄 License

MIT License - see LICENSE file for details.