# Day 344: Docker Compose: Local Environment

## Trading Analogy

Imagine you're setting up a trading desk. You need:
- The trading bot itself (executes strategy)
- A database (stores trade history)
- Redis (caches real-time prices)
- Grafana (monitors profit and loss)
- Prometheus (collects performance metrics)

Starting each service manually is like managing a trading floor where every trader uses different tools and systems. **Docker Compose** is your operations manager, ensuring all systems start correctly, can communicate with each other, and shut down gracefully.

| Concept | Trading Analogy |
|---------|-----------------|
| **Container** | Individual trading terminal with configured environment |
| **Service** | Trading function (execution, risk management, data) |
| **Network** | Trading desk internal communication system |
| **Volume** | Secure storage for trade history and configuration |
| **Compose file** | Operational checklist for starting the entire desk |

## Docker Compose Basics

Docker Compose allows you to define multi-container applications in a single YAML file.

### Basic docker-compose.yml Structure

```yaml
# docker-compose.yml
version: '3.8'

services:
  # Trading bot — main application
  trading-bot:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: trading-bot
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgres://trader:secret@postgres:5432/trading
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    networks:
      - trading-network
    volumes:
      - ./config:/app/config:ro
      - trading-logs:/app/logs
    restart: unless-stopped

  # PostgreSQL for trade history
  postgres:
    image: postgres:15-alpine
    container_name: trading-postgres
    environment:
      POSTGRES_USER: trader
      POSTGRES_PASSWORD: secret
      POSTGRES_DB: trading
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    networks:
      - trading-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U trader -d trading"]
      interval: 10s
      timeout: 5s
      retries: 5

  # Redis for real-time price caching
  redis:
    image: redis:7-alpine
    container_name: trading-redis
    command: redis-server --appendonly yes
    volumes:
      - redis-data:/data
    networks:
      - trading-network
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

networks:
  trading-network:
    driver: bridge

volumes:
  postgres-data:
  redis-data:
  trading-logs:
```

### Dockerfile for Rust Application

```dockerfile
# Dockerfile
# Build stage
FROM rust:1.75-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Build application
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Final image
FROM alpine:3.19

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

# Copy built binary
COPY --from=builder /app/target/release/trading-bot /app/trading-bot

# Create non-root user
RUN addgroup -S trader && adduser -S trader -G trader
USER trader

EXPOSE 8080

CMD ["./trading-bot"]
```

## Trading Infrastructure with Docker Compose

### Complete Algo-Trading Stack

```yaml
# docker-compose.trading.yml
version: '3.8'

services:
  # === Core Services ===

  trading-bot:
    build:
      context: ./trading-bot
      dockerfile: Dockerfile
    container_name: algo-trading-bot
    environment:
      - RUST_LOG=info,trading_bot=debug
      - DATABASE_URL=postgres://trader:${DB_PASSWORD}@postgres:5432/trading
      - REDIS_URL=redis://redis:6379
      - EXCHANGE_API_KEY=${EXCHANGE_API_KEY}
      - EXCHANGE_API_SECRET=${EXCHANGE_API_SECRET}
    env_file:
      - .env
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - trading-network
    volumes:
      - ./config:/app/config:ro
      - trading-logs:/app/logs
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 512M

  # Market data service
  market-data:
    build:
      context: ./market-data
      dockerfile: Dockerfile
    container_name: market-data-service
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
      - SYMBOLS=BTCUSDT,ETHUSDT,SOLUSDT
    depends_on:
      redis:
        condition: service_healthy
    networks:
      - trading-network
    restart: unless-stopped

  # Risk management service
  risk-manager:
    build:
      context: ./risk-manager
      dockerfile: Dockerfile
    container_name: risk-manager
    environment:
      - RUST_LOG=info,risk_manager=debug
      - DATABASE_URL=postgres://trader:${DB_PASSWORD}@postgres:5432/trading
      - MAX_POSITION_SIZE=10000
      - MAX_DAILY_LOSS=1000
    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - trading-network
    restart: unless-stopped

  # === Data Storage ===

  postgres:
    image: postgres:15-alpine
    container_name: trading-postgres
    environment:
      POSTGRES_USER: trader
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: trading
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./sql/init.sql:/docker-entrypoint-initdb.d/01-init.sql:ro
      - ./sql/schema.sql:/docker-entrypoint-initdb.d/02-schema.sql:ro
    networks:
      - trading-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U trader -d trading"]
      interval: 10s
      timeout: 5s
      retries: 5
    deploy:
      resources:
        limits:
          memory: 1G

  redis:
    image: redis:7-alpine
    container_name: trading-redis
    command: redis-server --appendonly yes --maxmemory 256mb --maxmemory-policy allkeys-lru
    volumes:
      - redis-data:/data
    networks:
      - trading-network
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  # === Monitoring ===

  prometheus:
    image: prom/prometheus:v2.48.0
    container_name: trading-prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    networks:
      - trading-network
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana:10.2.2
    container_name: trading-grafana
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    depends_on:
      - prometheus
    networks:
      - trading-network
    ports:
      - "3000:3000"

  # Alert manager for notifications
  alertmanager:
    image: prom/alertmanager:v0.26.0
    container_name: trading-alertmanager
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'
    volumes:
      - ./monitoring/alertmanager.yml:/etc/alertmanager/alertmanager.yml:ro
    networks:
      - trading-network
    ports:
      - "9093:9093"

networks:
  trading-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16

volumes:
  postgres-data:
  redis-data:
  prometheus-data:
  grafana-data:
  trading-logs:
```

### Environment Variables File

```bash
# .env
# Database credentials
DB_PASSWORD=your_secure_password_here

# Exchange API keys (don't commit to git!)
EXCHANGE_API_KEY=your_api_key
EXCHANGE_API_SECRET=your_api_secret

# Monitoring
GRAFANA_PASSWORD=admin_password

# Environment settings
ENVIRONMENT=development
LOG_LEVEL=debug
```

## Rust Code for Docker Service Integration

### Application Configuration

```rust
use std::env;
use serde::Deserialize;

/// Trading bot configuration from environment variables
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub exchange_api_key: String,
    pub exchange_api_secret: String,
    pub log_level: String,
    pub max_position_size: f64,
    pub max_daily_loss: f64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            redis_url: env::var("REDIS_URL")
                .map_err(|_| ConfigError::Missing("REDIS_URL"))?,
            exchange_api_key: env::var("EXCHANGE_API_KEY")
                .map_err(|_| ConfigError::Missing("EXCHANGE_API_KEY"))?,
            exchange_api_secret: env::var("EXCHANGE_API_SECRET")
                .map_err(|_| ConfigError::Missing("EXCHANGE_API_SECRET"))?,
            log_level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
            max_position_size: env::var("MAX_POSITION_SIZE")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000.0),
            max_daily_loss: env::var("MAX_DAILY_LOSS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000.0),
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Missing(&'static str),
    Invalid(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(var) => write!(f, "Missing environment variable: {}", var),
            ConfigError::Invalid(var) => write!(f, "Invalid variable value: {}", var),
        }
    }
}
```

### Service Health Checking

```rust
use std::time::Duration;
use tokio::time::timeout;

/// Service health checker
pub struct HealthChecker {
    database_url: String,
    redis_url: String,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub database: ServiceStatus,
    pub redis: ServiceStatus,
    pub overall: bool,
}

#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Healthy,
    Unhealthy(String),
    Unknown,
}

impl HealthChecker {
    pub fn new(database_url: String, redis_url: String) -> Self {
        HealthChecker {
            database_url,
            redis_url,
        }
    }

    /// Check health of all services
    pub async fn check_all(&self) -> HealthStatus {
        let (db_status, redis_status) = tokio::join!(
            self.check_database(),
            self.check_redis()
        );

        let overall = matches!(db_status, ServiceStatus::Healthy)
            && matches!(redis_status, ServiceStatus::Healthy);

        HealthStatus {
            database: db_status,
            redis: redis_status,
            overall,
        }
    }

    async fn check_database(&self) -> ServiceStatus {
        // Simulate PostgreSQL connection check
        match timeout(Duration::from_secs(5), self.ping_database()).await {
            Ok(Ok(())) => ServiceStatus::Healthy,
            Ok(Err(e)) => ServiceStatus::Unhealthy(e),
            Err(_) => ServiceStatus::Unhealthy("Connection timeout".to_string()),
        }
    }

    async fn check_redis(&self) -> ServiceStatus {
        // Simulate Redis connection check
        match timeout(Duration::from_secs(5), self.ping_redis()).await {
            Ok(Ok(())) => ServiceStatus::Healthy,
            Ok(Err(e)) => ServiceStatus::Unhealthy(e),
            Err(_) => ServiceStatus::Unhealthy("Connection timeout".to_string()),
        }
    }

    async fn ping_database(&self) -> Result<(), String> {
        // In real code this would connect to PostgreSQL
        // sqlx::PgPool::connect(&self.database_url).await
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    async fn ping_redis(&self) -> Result<(), String> {
        // In real code this would connect to Redis
        // redis::Client::open(&self.redis_url)?.get_connection()
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    println!("=== Trading Infrastructure Health Check ===\n");

    let checker = HealthChecker::new(
        "postgres://trader:secret@localhost:5432/trading".to_string(),
        "redis://localhost:6379".to_string(),
    );

    let status = checker.check_all().await;

    println!("Database: {:?}", status.database);
    println!("Redis: {:?}", status.redis);
    println!("Overall status: {}", if status.overall { "Healthy" } else { "Issues detected" });
}
```

### Price Cache with Redis

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Price data
#[derive(Debug, Clone)]
pub struct PriceData {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: f64,
    pub timestamp: Instant,
}

/// Local price cache with TTL (Redis simulation)
pub struct PriceCache {
    cache: HashMap<String, PriceData>,
    ttl: Duration,
}

impl PriceCache {
    pub fn new(ttl_seconds: u64) -> Self {
        PriceCache {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Save price to cache
    pub fn set(&mut self, symbol: &str, bid: f64, ask: f64, last: f64, volume: f64) {
        let data = PriceData {
            symbol: symbol.to_string(),
            bid,
            ask,
            last,
            volume,
            timestamp: Instant::now(),
        };
        self.cache.insert(symbol.to_string(), data);
    }

    /// Get price from cache
    pub fn get(&self, symbol: &str) -> Option<&PriceData> {
        self.cache.get(symbol).filter(|data| {
            data.timestamp.elapsed() < self.ttl
        })
    }

    /// Get spread
    pub fn get_spread(&self, symbol: &str) -> Option<f64> {
        self.get(symbol).map(|data| data.ask - data.bid)
    }

    /// Cleanup expired entries
    pub fn cleanup_expired(&mut self) {
        self.cache.retain(|_, data| {
            data.timestamp.elapsed() < self.ttl
        });
    }

    /// Get all active symbols
    pub fn get_active_symbols(&self) -> Vec<String> {
        self.cache
            .iter()
            .filter(|(_, data)| data.timestamp.elapsed() < self.ttl)
            .map(|(symbol, _)| symbol.clone())
            .collect()
    }
}

fn main() {
    println!("=== Price Cache Demo ===\n");

    let mut cache = PriceCache::new(60); // 60 second TTL

    // Simulate price updates from exchange
    let prices = vec![
        ("BTCUSDT", 50000.0, 50010.0, 50005.0, 1500.5),
        ("ETHUSDT", 3000.0, 3002.0, 3001.0, 5000.0),
        ("SOLUSDT", 100.0, 100.1, 100.05, 10000.0),
    ];

    for (symbol, bid, ask, last, volume) in prices {
        cache.set(symbol, bid, ask, last, volume);
        println!("Updated price: {} - bid: {}, ask: {}", symbol, bid, ask);
    }

    println!("\n=== Cache Status ===");
    for symbol in cache.get_active_symbols() {
        if let Some(data) = cache.get(&symbol) {
            println!(
                "{}: ${:.2} (spread: ${:.2})",
                symbol,
                data.last,
                cache.get_spread(&symbol).unwrap_or(0.0)
            );
        }
    }
}
```

## Profiles for Different Environments

### Development

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  trading-bot:
    build:
      context: .
      dockerfile: Dockerfile.dev
    environment:
      - RUST_LOG=debug
      - RUST_BACKTRACE=1
    volumes:
      - ./src:/app/src:ro          # Mount source code for hot-reload
      - ./target:/app/target        # Cache builds
    ports:
      - "8080:8080"                 # Debug port
      - "9229:9229"                 # Debugger port

  postgres:
    ports:
      - "5432:5432"                 # External DB access

  redis:
    ports:
      - "6379:6379"                 # External Redis access

  # Development tools
  adminer:
    image: adminer:4
    ports:
      - "8081:8080"
    networks:
      - trading-network

  redis-commander:
    image: rediscommander/redis-commander:latest
    environment:
      - REDIS_HOSTS=local:redis:6379
    ports:
      - "8082:8081"
    networks:
      - trading-network
```

### Production

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  trading-bot:
    image: ${REGISTRY}/trading-bot:${VERSION}
    environment:
      - RUST_LOG=info
    deploy:
      replicas: 2
      update_config:
        parallelism: 1
        delay: 30s
        failure_action: rollback
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
        window: 120s
      resources:
        limits:
          cpus: '4'
          memory: 4G
        reservations:
          cpus: '2'
          memory: 2G
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  postgres:
    deploy:
      resources:
        limits:
          memory: 2G
    # Don't expose ports in production!
```

### Running with Profiles

```bash
# Development
docker compose -f docker-compose.yml -f docker-compose.dev.yml up

# Production
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# With specific services
docker compose up trading-bot postgres redis

# Rebuild after changes
docker compose build --no-cache trading-bot
docker compose up -d trading-bot
```

## Monitoring and Logging

### Prometheus Configuration

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

rule_files:
  - "alerts/*.yml"

scrape_configs:
  - job_name: 'trading-bot'
    static_configs:
      - targets: ['trading-bot:8080']
    metrics_path: /metrics
    scrape_interval: 5s

  - job_name: 'market-data'
    static_configs:
      - targets: ['market-data:8080']

  - job_name: 'risk-manager'
    static_configs:
      - targets: ['risk-manager:8080']

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
```

### Trading Alert Rules

```yaml
# monitoring/alerts/trading.yml
groups:
  - name: trading_alerts
    rules:
      - alert: HighDrawdown
        expr: trading_drawdown_percent > 5
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "High drawdown: {{ $value }}%"
          description: "Drawdown exceeded 5%. Attention required."

      - alert: OrderExecutionSlow
        expr: trading_order_execution_seconds > 2
        for: 30s
        labels:
          severity: warning
        annotations:
          summary: "Slow order execution"
          description: "Order execution time: {{ $value }} seconds"

      - alert: MarketDataStale
        expr: time() - trading_last_price_update_timestamp > 30
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Stale market data"
          description: "Data hasn't been updated for more than 30 seconds"

      - alert: HighMemoryUsage
        expr: container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Container is using more than 85% of memory"
```

### Rust Code for Metrics Export

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Trading bot metrics for Prometheus
pub struct TradingMetrics {
    // Counters
    orders_total: AtomicU64,
    orders_filled: AtomicU64,
    orders_rejected: AtomicU64,

    // Histograms (simplified version)
    execution_times_ms: Arc<std::sync::Mutex<Vec<u64>>>,

    // Current values
    current_pnl: AtomicU64,        // Stored as f64 bits
    current_drawdown: AtomicU64,
    position_count: AtomicU64,

    // Last update time
    last_update: Arc<std::sync::Mutex<Instant>>,
}

impl TradingMetrics {
    pub fn new() -> Self {
        TradingMetrics {
            orders_total: AtomicU64::new(0),
            orders_filled: AtomicU64::new(0),
            orders_rejected: AtomicU64::new(0),
            execution_times_ms: Arc::new(std::sync::Mutex::new(Vec::new())),
            current_pnl: AtomicU64::new(0),
            current_drawdown: AtomicU64::new(0),
            position_count: AtomicU64::new(0),
            last_update: Arc::new(std::sync::Mutex::new(Instant::now())),
        }
    }

    /// Record a new order
    pub fn record_order(&self, filled: bool, execution_time_ms: u64) {
        self.orders_total.fetch_add(1, Ordering::Relaxed);

        if filled {
            self.orders_filled.fetch_add(1, Ordering::Relaxed);
        } else {
            self.orders_rejected.fetch_add(1, Ordering::Relaxed);
        }

        if let Ok(mut times) = self.execution_times_ms.lock() {
            times.push(execution_time_ms);
            // Keep only last 1000 measurements
            if times.len() > 1000 {
                times.remove(0);
            }
        }
    }

    /// Update PnL
    pub fn update_pnl(&self, pnl: f64) {
        self.current_pnl.store(pnl.to_bits(), Ordering::Relaxed);
        if let Ok(mut last) = self.last_update.lock() {
            *last = Instant::now();
        }
    }

    /// Update drawdown
    pub fn update_drawdown(&self, drawdown_percent: f64) {
        self.current_drawdown.store(
            (drawdown_percent * 100.0) as u64,
            Ordering::Relaxed
        );
    }

    /// Generate metrics in Prometheus format
    pub fn to_prometheus_format(&self) -> String {
        let pnl = f64::from_bits(self.current_pnl.load(Ordering::Relaxed));
        let drawdown = self.current_drawdown.load(Ordering::Relaxed) as f64 / 100.0;

        let avg_execution = if let Ok(times) = self.execution_times_ms.lock() {
            if times.is_empty() {
                0.0
            } else {
                times.iter().sum::<u64>() as f64 / times.len() as f64
            }
        } else {
            0.0
        };

        format!(
            r#"# HELP trading_orders_total Total number of orders
# TYPE trading_orders_total counter
trading_orders_total {}

# HELP trading_orders_filled Number of filled orders
# TYPE trading_orders_filled counter
trading_orders_filled {}

# HELP trading_orders_rejected Number of rejected orders
# TYPE trading_orders_rejected counter
trading_orders_rejected {}

# HELP trading_pnl_usd Current profit/loss in USD
# TYPE trading_pnl_usd gauge
trading_pnl_usd {:.2}

# HELP trading_drawdown_percent Current drawdown percentage
# TYPE trading_drawdown_percent gauge
trading_drawdown_percent {:.2}

# HELP trading_order_execution_avg_ms Average order execution time
# TYPE trading_order_execution_avg_ms gauge
trading_order_execution_avg_ms {:.2}
"#,
            self.orders_total.load(Ordering::Relaxed),
            self.orders_filled.load(Ordering::Relaxed),
            self.orders_rejected.load(Ordering::Relaxed),
            pnl,
            drawdown,
            avg_execution
        )
    }
}

fn main() {
    println!("=== Trading Bot Metrics Demo ===\n");

    let metrics = TradingMetrics::new();

    // Simulate trading activity
    for i in 0..10 {
        let filled = i % 3 != 0;  // 70% successful orders
        let execution_time = 50 + (i * 10) as u64;
        metrics.record_order(filled, execution_time);
    }

    metrics.update_pnl(1234.56);
    metrics.update_drawdown(2.5);

    println!("{}", metrics.to_prometheus_format());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Docker Compose** | Orchestration of multi-container applications |
| **Services** | Container definitions and their dependencies |
| **Networks** | Isolated communication between services |
| **Volumes** | Persistent data storage |
| **Health checks** | Service health verification |
| **Profiles** | Configurations for different environments |
| **Environment variables** | Configuration without code changes |
| **Monitoring** | Prometheus + Grafana for metrics |

## Practical Exercises

1. **Basic Trading Stack**: Create a docker-compose.yml with:
   - Rust trading bot application
   - PostgreSQL for storing trades
   - Redis for price caching
   - Health checks for all services
   - Named volumes for data

2. **Multi-Environment Setup**: Configure profiles for:
   - Development with hot-reload and debug ports
   - Testing with isolated databases
   - Production with resource limits

3. **PnL Monitoring**: Add:
   - Prometheus for collecting metrics
   - Grafana with profit/loss dashboard
   - Alerts for high drawdown

4. **Service Mesh**: Create microservices architecture:
   - Separate market data service
   - Order execution service
   - Risk management service
   - Shared Redis for communication

## Homework

1. **Complete Trading Stack**: Implement infrastructure with:
   - At least 3 Rust microservices
   - PostgreSQL with replication
   - Redis cluster
   - Full monitoring stack
   - Automatic backups
   - Logging to ELK/Loki

2. **CI/CD Pipeline**: Set up:
   - Automatic image building
   - Tests in Docker containers
   - Deployment with docker compose
   - Rollback on failed health checks
   - Telegram notifications on issues

3. **Fault Tolerance**: Add:
   - PostgreSQL replication (primary/replica)
   - Redis Sentinel for high availability
   - Nginx as load balancer
   - Automatic restart of failed services
   - Graceful shutdown for trading services

4. **Security**: Implement:
   - Secrets management (Docker secrets or Vault)
   - Isolated networks for different services
   - Read-only file systems where possible
   - Non-root users in containers
   - Image vulnerability scanning

## Navigation

[← Previous day](../343-*/en.md) | [Next day →](../345-*/en.md)
