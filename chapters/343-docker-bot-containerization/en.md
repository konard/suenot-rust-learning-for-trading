# Day 343: Docker: Bot Containerization

## Trading Analogy

Imagine you work at a major investment firm. You have a trading bot that works perfectly on your computer. But how do you move it to an exchange server or the cloud?

**The Problem:** Your computer has specific library versions, Rust version, and configurations. The server might have everything set up differently. It's like a trader who's used to one terminal being forced to use another — buttons are in different places, hotkeys don't work.

**Docker Solution:** A container is like a personal trading terminal you can take anywhere. Inside the container — everything is configured exactly as you need: library versions, configuration, environment variables. Run it on any server — it works the same way.

| Analogy | Docker Concept |
|---------|----------------|
| Trading terminal in a suitcase | Docker container |
| Terminal setup instructions | Dockerfile |
| Warehouse of ready terminals | Docker Registry (Docker Hub) |
| Terminal image on disk | Docker Image |
| Running terminal | Docker Container |

## What is Docker?

Docker is a platform for developing, shipping, and running applications in isolated containers. A container includes everything needed for the application to run: code, runtime, libraries, environment variables.

### Key Concepts

```
┌─────────────────────────────────────────────────────────────┐
│                         Docker Host                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  Container 1    │  │  Container 2    │  │  Container 3 │ │
│  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌────────┐  │ │
│  │  │Trading Bot│  │  │  │ Database  │  │  │  │ Redis  │  │ │
│  │  └───────────┘  │  │  └───────────┘  │  │  └────────┘  │ │
│  │  Rust 1.75     │  │  │  PostgreSQL   │  │  │  Redis 7   │ │
│  │  Alpine Linux  │  │  │  Alpine Linux │  │  │  Alpine    │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│                                                              │
│                    Docker Engine                             │
└─────────────────────────────────────────────────────────────┘
                              │
                    Host Operating System
```

## Installing Docker

### Linux (Ubuntu/Debian)

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add user to docker group
sudo usermod -aG docker $USER

# Verify
docker --version
```

### macOS and Windows

Download Docker Desktop from [docker.com](https://www.docker.com/products/docker-desktop/).

## Creating a Dockerfile for Trading Bot

Let's start with a simple trading bot and create a Dockerfile for it:

### Project Structure

```
trading-bot/
├── Cargo.toml
├── src/
│   └── main.rs
├── Dockerfile
└── .dockerignore
```

### Cargo.toml

```toml
[package]
name = "trading-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### src/main.rs — Simple Trading Bot

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tracing::{info, warn, error, Level};
use tracing_subscriber::FmtSubscriber;

/// Market data
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume: f64,
    timestamp: u64,
}

/// Order
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    id: String,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

/// Bot configuration from environment variables
#[derive(Debug)]
struct BotConfig {
    /// Trading symbols
    symbols: Vec<String>,
    /// Maximum position size
    max_position_size: f64,
    /// Entry threshold (spread percentage)
    entry_threshold: f64,
    /// API key (in production - from secrets)
    api_key: String,
    /// Operating mode: live or paper
    mode: String,
}

impl BotConfig {
    /// Load configuration from environment variables
    fn from_env() -> Self {
        let symbols = env::var("BOT_SYMBOLS")
            .unwrap_or_else(|_| "BTCUSDT,ETHUSDT".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let max_position_size = env::var("BOT_MAX_POSITION")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse()
            .unwrap_or(1.0);

        let entry_threshold = env::var("BOT_ENTRY_THRESHOLD")
            .unwrap_or_else(|_| "0.1".to_string())
            .parse()
            .unwrap_or(0.1);

        let api_key = env::var("BOT_API_KEY")
            .unwrap_or_else(|_| "demo_key".to_string());

        let mode = env::var("BOT_MODE")
            .unwrap_or_else(|_| "paper".to_string());

        BotConfig {
            symbols,
            max_position_size,
            entry_threshold,
            api_key,
            mode,
        }
    }
}

/// Trading engine
struct TradingEngine {
    config: BotConfig,
    positions: HashMap<String, f64>,
    orders: Vec<Order>,
    order_counter: u64,
}

impl TradingEngine {
    fn new(config: BotConfig) -> Self {
        TradingEngine {
            config,
            positions: HashMap::new(),
            orders: Vec::new(),
            order_counter: 0,
        }
    }

    /// Process market data
    fn process_market_data(&mut self, data: &MarketData) {
        let spread = (data.ask - data.bid) / data.bid * 100.0;

        info!(
            symbol = %data.symbol,
            bid = data.bid,
            ask = data.ask,
            spread = format!("{:.4}%", spread),
            "Received market data"
        );

        // Simple strategy: buy on tight spread
        if spread < self.config.entry_threshold {
            let current_position = self.positions.get(&data.symbol).unwrap_or(&0.0);

            if *current_position < self.config.max_position_size {
                self.place_order(&data.symbol, OrderSide::Buy, data.ask, 0.1);
            }
        }
    }

    /// Place order
    fn place_order(&mut self, symbol: &str, side: OrderSide, price: f64, quantity: f64) {
        self.order_counter += 1;
        let order_id = format!("ORD-{:06}", self.order_counter);

        let order = Order {
            id: order_id.clone(),
            symbol: symbol.to_string(),
            side: side.clone(),
            price,
            quantity,
            status: OrderStatus::Pending,
        };

        info!(
            order_id = %order_id,
            symbol = %symbol,
            side = ?side,
            price = price,
            quantity = quantity,
            "Order placed"
        );

        // In paper mode - execute immediately
        if self.config.mode == "paper" {
            self.execute_order(&order);
        }

        self.orders.push(order);
    }

    /// Execute order
    fn execute_order(&mut self, order: &Order) {
        let position = self.positions.entry(order.symbol.clone()).or_insert(0.0);

        match order.side {
            OrderSide::Buy => *position += order.quantity,
            OrderSide::Sell => *position -= order.quantity,
        }

        info!(
            order_id = %order.id,
            new_position = *position,
            "Order executed"
        );
    }

    /// Get current positions
    fn get_positions(&self) -> &HashMap<String, f64> {
        &self.positions
    }

    /// Print statistics
    fn print_stats(&self) {
        info!("=== Trading Bot Statistics ===");
        info!("Mode: {}", self.config.mode);
        info!("Symbols: {:?}", self.config.symbols);
        info!("Total orders: {}", self.orders.len());
        info!("Current positions:");
        for (symbol, size) in &self.positions {
            info!("  {}: {:.4}", symbol, size);
        }
    }
}

/// Simulate receiving market data
fn simulate_market_data(symbol: &str, base_price: f64) -> MarketData {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Add some volatility
    let noise = ((timestamp % 100) as f64 - 50.0) / 1000.0;
    let price = base_price * (1.0 + noise);

    MarketData {
        symbol: symbol.to_string(),
        bid: price * 0.9999,
        ask: price * 1.0001,
        last_price: price,
        volume: 1000.0 + (timestamp % 500) as f64,
        timestamp,
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .json()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set logging subscriber");

    info!("Starting trading bot...");

    // Load configuration
    let config = BotConfig::from_env();
    info!(config = ?config, "Configuration loaded");

    // Create trading engine
    let mut engine = TradingEngine::new(config);

    // Base prices for simulation
    let base_prices: HashMap<&str, f64> = [
        ("BTCUSDT", 50000.0),
        ("ETHUSDT", 3000.0),
    ]
    .into_iter()
    .collect();

    // Main loop
    let mut iteration = 0;
    loop {
        iteration += 1;
        info!(iteration = iteration, "--- Iteration ---");

        // Process each symbol
        for symbol in &engine.config.symbols.clone() {
            if let Some(&base_price) = base_prices.get(symbol.as_str()) {
                let market_data = simulate_market_data(symbol, base_price);
                engine.process_market_data(&market_data);
            } else {
                warn!(symbol = %symbol, "Unknown symbol, skipping");
            }
        }

        // Print stats every 5 iterations
        if iteration % 5 == 0 {
            engine.print_stats();
        }

        // Wait before next iteration
        tokio::time::sleep(Duration::from_secs(2)).await;

        // For demonstration — exit after 20 iterations
        if iteration >= 20 {
            info!("Reached iteration limit, shutting down");
            break;
        }
    }

    info!("Trading bot finished");
    engine.print_stats();
}
```

### Dockerfile — Multi-stage Build

```dockerfile
# ==========================================
# Stage 1: Build (Builder)
# ==========================================
FROM rust:1.75-alpine AS builder

# Install required packages for building
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

# Create working directory
WORKDIR /app

# Copy dependency files for layer caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy main.rs for dependency compilation
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build only dependencies (this will be cached)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Rebuild with actual code
RUN touch src/main.rs && cargo build --release

# ==========================================
# Stage 2: Runtime (minimal image)
# ==========================================
FROM alpine:3.19 AS runtime

# Install minimal dependencies
RUN apk add --no-cache ca-certificates tzdata

# Create unprivileged user
RUN addgroup -S trading && adduser -S bot -G trading

# Working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/trading-bot /app/trading-bot

# Change ownership
RUN chown -R bot:trading /app

# Switch to unprivileged user
USER bot

# Default environment variables
ENV BOT_MODE=paper \
    BOT_SYMBOLS=BTCUSDT,ETHUSDT \
    BOT_MAX_POSITION=1.0 \
    BOT_ENTRY_THRESHOLD=0.1 \
    RUST_LOG=info

# Entry point
ENTRYPOINT ["/app/trading-bot"]
```

### .dockerignore — Exclude Unnecessary Files

```
# Build artifacts
target/
Cargo.lock

# Git
.git/
.gitignore

# IDE
.idea/
.vscode/
*.swp
*.swo

# Documentation and tests (for production image)
docs/
tests/
benches/
examples/

# Docker files
Dockerfile*
docker-compose*.yml
.dockerignore

# Logs and temporary files
*.log
*.tmp
.env.local
```

## Building and Running the Container

```bash
# Build image
docker build -t trading-bot:latest .

# View created image
docker images | grep trading-bot

# Run container
docker run --name my-bot \
    -e BOT_MODE=paper \
    -e BOT_SYMBOLS=BTCUSDT,ETHUSDT \
    -e BOT_MAX_POSITION=0.5 \
    trading-bot:latest

# Run in background
docker run -d --name my-bot-daemon \
    -e BOT_MODE=paper \
    trading-bot:latest

# View logs
docker logs -f my-bot-daemon

# Stop container
docker stop my-bot-daemon

# Remove container
docker rm my-bot-daemon
```

## Docker Image Optimization

### Image Size Comparison

```rust
// Example script for analyzing image size
use std::process::Command;

fn get_image_size(image_name: &str) -> String {
    let output = Command::new("docker")
        .args(["images", image_name, "--format", "{{.Size}}"])
        .output()
        .expect("Failed to execute docker command");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn main() {
    let images = vec![
        ("rust:latest", "Base Rust image"),
        ("rust:slim", "Slim Rust image"),
        ("rust:alpine", "Alpine Rust image"),
        ("trading-bot:latest", "Our optimized bot"),
    ];

    println!("=== Docker Image Size Comparison ===\n");

    for (image, description) in images {
        let size = get_image_size(image);
        println!("{}: {} ({})", description, size, image);
    }
}
```

### Typical Sizes:

| Image | Size | Description |
|-------|------|-------------|
| `rust:latest` | ~1.4 GB | Full Rust toolchain |
| `rust:slim` | ~800 MB | Without extra utilities |
| `rust:alpine` | ~600 MB | Alpine-based |
| Our bot (multi-stage) | ~15-30 MB | Binary only |

## Working with Secrets

### Secure API Key Handling

```bash
# Bad: key visible in command history and docker inspect
docker run -e BOT_API_KEY=super_secret_key trading-bot:latest

# Better: from file
echo "super_secret_key" > api_key.txt
docker run --env-file .env trading-bot:latest

# Even better: Docker secrets (in Swarm/Kubernetes)
echo "super_secret_key" | docker secret create bot_api_key -
```

### .env File for Development

```env
# .env
BOT_MODE=paper
BOT_SYMBOLS=BTCUSDT,ETHUSDT
BOT_MAX_POSITION=1.0
BOT_ENTRY_THRESHOLD=0.1
BOT_API_KEY=dev_key_12345
```

## Health Checks in Docker

```dockerfile
# Add to Dockerfile
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1
```

### Adding Health Endpoint to the Bot

```rust
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;

/// Simple HTTP server for health checks
async fn health_server() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await.expect("Failed to start health server");

    info!("Health server started on {}", addr);

    loop {
        if let Ok((mut socket, _)) = listener.accept().await {
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"healthy\"}";
            let _ = socket.write_all(response.as_bytes()).await;
        }
    }
}

// In main() add:
// tokio::spawn(health_server());
```

## Logging for Containers

### Structured JSON Logs

```rust
use tracing_subscriber::fmt::format::JsonFields;

fn setup_logging() {
    // JSON logs for Docker/Kubernetes
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .with_current_span(false)
        .with_span_list(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set logger");
}
```

Example output:

```json
{"timestamp":"2024-01-15T10:30:00.123Z","level":"INFO","message":"Received market data","symbol":"BTCUSDT","bid":50000.5,"ask":50001.0}
{"timestamp":"2024-01-15T10:30:00.125Z","level":"INFO","message":"Order placed","order_id":"ORD-000001","symbol":"BTCUSDT","side":"Buy","price":50001.0}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Docker Image** | Immutable template for creating containers |
| **Docker Container** | Running instance of an image |
| **Dockerfile** | Instructions for building an image |
| **Multi-stage build** | Multi-stage build for size minimization |
| **.dockerignore** | Excluding files from build context |
| **Environment variables** | Configuration through environment variables |
| **Health checks** | Container health verification |
| **JSON logging** | Structured logs for containerized environments |

## Practical Exercises

1. **Image Optimization**: Modify the Dockerfile to use `scratch` instead of `alpine` as the base image. What size did you get? What issues arose?

2. **File-based Configuration**: Add support for loading configuration from a TOML file mounted into the container:
   ```bash
   docker run -v $(pwd)/config.toml:/app/config.toml trading-bot:latest
   ```

3. **Graceful Shutdown**: Implement proper bot shutdown when receiving `SIGTERM` signal (Docker sends this on `docker stop`):
   ```rust
   tokio::select! {
       _ = trading_loop() => {}
       _ = tokio::signal::ctrl_c() => {
           info!("Received shutdown signal");
       }
   }
   ```

4. **Monitoring Metrics**: Add a `/metrics` endpoint with Prometheus-format metrics:
   - Number of processed orders
   - Current positions
   - Bot uptime

## Homework

1. **Full Dockerfile with Caching**: Create a Dockerfile that:
   - Uses cargo-chef for optimal dependency caching
   - Includes health check
   - Has separate stages for dev and prod
   - Final image weighs less than 20 MB

2. **Multi-architecture Build**: Set up image building for ARM64 and AMD64:
   ```bash
   docker buildx build --platform linux/amd64,linux/arm64 -t trading-bot:multi .
   ```

3. **Local Registry**: Deploy a local Docker Registry and publish your image there:
   ```bash
   docker run -d -p 5000:5000 registry:2
   docker tag trading-bot:latest localhost:5000/trading-bot:latest
   docker push localhost:5000/trading-bot:latest
   ```

4. **CI/CD Integration**: Write a GitHub Actions workflow that:
   - Builds Docker image on every push
   - Runs tests inside the container
   - Publishes image to GitHub Container Registry
   - Scans image for vulnerabilities using Trivy

5. **Advanced Logging**: Implement a logging system that:
   - Outputs JSON logs to stdout
   - Rotates logs by size (when writing to file)
   - Includes request_id for tracing
   - Masks sensitive data (API keys)

## Navigation

[← Previous day](../342-*/en.md) | [Next day →](../344-*/en.md)
