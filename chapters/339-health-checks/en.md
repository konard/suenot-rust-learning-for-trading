# Day 339: Health Checks

## Trading Analogy

Imagine you're managing a trading center with multiple branches. Every morning you send an inspector to check each branch:

- **Is the store open?** — basic availability check
- **Is the cash register working?** — critical systems check
- **Is there inventory in stock?** — dependencies check
- **Is the internet working for terminals?** — external connections check

In trading systems, Health Checks work the same way:

| Trading Analogy | Health Check Component |
|-----------------|------------------------|
| Store is open | Service is running and responding |
| Cash register works | Database is available |
| Inventory in stock | Data cache is fresh |
| Internet works | Exchange API is available |
| Staff on duty | Workers are processing tasks |

## Why Do We Need Health Checks?

In production systems, Health Checks are critical for:

1. **Load Balancer** — routes traffic only to healthy instances
2. **Kubernetes** — decides when to restart a pod
3. **Monitoring** — alerts about problems before they escalate
4. **CI/CD** — verifies that deployment succeeded

## Types of Health Checks

### 1. Liveness Probe — "Is the service alive?"

The simplest check: does the service respond at all.

```rust
use axum::{routing::get, Router, Json};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct LivenessResponse {
    status: String,
    timestamp: u64,
}

/// Liveness endpoint — simply confirms the service is running
async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "alive".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health/live", get(liveness));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Service running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 2. Readiness Probe — "Ready to accept requests?"

Checks that all dependencies are working.

```rust
use axum::{routing::get, Router, Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trading system health state
#[derive(Clone)]
struct TradingSystemHealth {
    database_connected: bool,
    exchange_api_available: bool,
    market_data_fresh: bool,
    order_queue_healthy: bool,
}

impl TradingSystemHealth {
    fn is_ready(&self) -> bool {
        self.database_connected
            && self.exchange_api_available
            && self.market_data_fresh
            && self.order_queue_healthy
    }
}

#[derive(Serialize)]
struct ReadinessResponse {
    ready: bool,
    checks: ReadinessChecks,
}

#[derive(Serialize)]
struct ReadinessChecks {
    database: CheckResult,
    exchange_api: CheckResult,
    market_data: CheckResult,
    order_queue: CheckResult,
}

#[derive(Serialize)]
struct CheckResult {
    healthy: bool,
    message: String,
}

async fn readiness(
    health: Arc<RwLock<TradingSystemHealth>>,
) -> impl IntoResponse {
    let state = health.read().await;

    let response = ReadinessResponse {
        ready: state.is_ready(),
        checks: ReadinessChecks {
            database: CheckResult {
                healthy: state.database_connected,
                message: if state.database_connected {
                    "Connected".to_string()
                } else {
                    "Connection failed".to_string()
                },
            },
            exchange_api: CheckResult {
                healthy: state.exchange_api_available,
                message: if state.exchange_api_available {
                    "API responding".to_string()
                } else {
                    "API timeout".to_string()
                },
            },
            market_data: CheckResult {
                healthy: state.market_data_fresh,
                message: if state.market_data_fresh {
                    "Data fresh (<5s)".to_string()
                } else {
                    "Stale data (>30s)".to_string()
                },
            },
            order_queue: CheckResult {
                healthy: state.order_queue_healthy,
                message: if state.order_queue_healthy {
                    "Queue processing".to_string()
                } else {
                    "Queue blocked".to_string()
                },
            },
        },
    };

    let status = if state.is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response))
}

#[tokio::main]
async fn main() {
    // Initialize health state
    let health = Arc::new(RwLock::new(TradingSystemHealth {
        database_connected: true,
        exchange_api_available: true,
        market_data_fresh: true,
        order_queue_healthy: true,
    }));

    let health_for_route = Arc::clone(&health);

    let app = Router::new()
        .route("/health/ready", get(move || {
            let h = Arc::clone(&health_for_route);
            async move { readiness(h).await }
        }));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Readiness endpoint: http://{}/health/ready", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### 3. Startup Probe — "Has initialization completed?"

Especially important for trading systems that need to load historical data.

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Trading system startup state
struct StartupState {
    config_loaded: AtomicBool,
    historical_data_loaded: AtomicBool,
    strategies_initialized: AtomicBool,
    exchange_connections_established: AtomicBool,
    ready_for_trading: AtomicBool,
}

impl StartupState {
    fn new() -> Self {
        StartupState {
            config_loaded: AtomicBool::new(false),
            historical_data_loaded: AtomicBool::new(false),
            strategies_initialized: AtomicBool::new(false),
            exchange_connections_established: AtomicBool::new(false),
            ready_for_trading: AtomicBool::new(false),
        }
    }

    fn is_started(&self) -> bool {
        self.config_loaded.load(Ordering::Relaxed)
            && self.historical_data_loaded.load(Ordering::Relaxed)
            && self.strategies_initialized.load(Ordering::Relaxed)
            && self.exchange_connections_established.load(Ordering::Relaxed)
    }

    fn startup_progress(&self) -> StartupProgress {
        StartupProgress {
            config_loaded: self.config_loaded.load(Ordering::Relaxed),
            historical_data_loaded: self.historical_data_loaded.load(Ordering::Relaxed),
            strategies_initialized: self.strategies_initialized.load(Ordering::Relaxed),
            exchange_connections: self.exchange_connections_established.load(Ordering::Relaxed),
            ready_for_trading: self.ready_for_trading.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
struct StartupProgress {
    config_loaded: bool,
    historical_data_loaded: bool,
    strategies_initialized: bool,
    exchange_connections: bool,
    ready_for_trading: bool,
}

/// Simulate trading system startup
async fn simulate_startup(state: Arc<StartupState>) {
    println!("=== Trading System Startup ===\n");

    // Phase 1: Load configuration
    println!("[1/5] Loading configuration...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    state.config_loaded.store(true, Ordering::Relaxed);
    println!("      Configuration loaded");

    // Phase 2: Load historical data
    println!("[2/5] Loading historical data...");
    for i in 1..=5 {
        tokio::time::sleep(Duration::from_millis(300)).await;
        println!("      Loaded {} of 5 instruments", i);
    }
    state.historical_data_loaded.store(true, Ordering::Relaxed);
    println!("      Historical data loaded");

    // Phase 3: Initialize strategies
    println!("[3/5] Initializing trading strategies...");
    tokio::time::sleep(Duration::from_millis(400)).await;
    state.strategies_initialized.store(true, Ordering::Relaxed);
    println!("      Strategies initialized");

    // Phase 4: Connect to exchanges
    println!("[4/5] Establishing exchange connections...");
    tokio::time::sleep(Duration::from_millis(600)).await;
    state.exchange_connections_established.store(true, Ordering::Relaxed);
    println!("      Connections established");

    // Phase 5: Ready for trading
    println!("[5/5] Final verification...");
    tokio::time::sleep(Duration::from_millis(200)).await;
    state.ready_for_trading.store(true, Ordering::Relaxed);
    println!("      System ready for trading!");

    println!("\n=== System Fully Started ===");
}

#[tokio::main]
async fn main() {
    let state = Arc::new(StartupState::new());
    let state_clone = Arc::clone(&state);

    // Start initialization process
    let startup_handle = tokio::spawn(simulate_startup(state_clone));

    // Monitor startup progress
    let state_monitor = Arc::clone(&state);
    let monitor_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let progress = state_monitor.startup_progress();

            if state_monitor.is_started() {
                println!("\n[Health Check] Startup complete: {:?}", progress);
                break;
            } else {
                println!("[Health Check] Startup in progress: {:?}", progress);
            }
        }
    });

    startup_handle.await.unwrap();
    monitor_handle.await.unwrap();
}
```

## Comprehensive Health Check System for Trading Bot

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Health check status
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component check result
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub latency_ms: u64,
    pub last_check: u64,
}

/// Overall system health report
#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Component health check trait
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> ComponentHealth;
}

/// Database connection check
pub struct DatabaseHealthCheck {
    connection_string: String,
}

impl DatabaseHealthCheck {
    pub fn new(connection_string: &str) -> Self {
        DatabaseHealthCheck {
            connection_string: connection_string.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        "database"
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();

        // Simulate database connection check
        tokio::time::sleep(Duration::from_millis(10)).await;

        let latency = start.elapsed().as_millis() as u64;
        let (status, message) = if latency < 100 {
            (HealthStatus::Healthy, "Connection OK".to_string())
        } else if latency < 500 {
            (HealthStatus::Degraded, format!("Slow response: {}ms", latency))
        } else {
            (HealthStatus::Unhealthy, "Connection timeout".to_string())
        };

        ComponentHealth {
            name: self.name().to_string(),
            status,
            message,
            latency_ms: latency,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Exchange connection check
pub struct ExchangeHealthCheck {
    exchange_name: String,
    api_endpoint: String,
}

impl ExchangeHealthCheck {
    pub fn new(exchange_name: &str, api_endpoint: &str) -> Self {
        ExchangeHealthCheck {
            exchange_name: exchange_name.to_string(),
            api_endpoint: api_endpoint.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for ExchangeHealthCheck {
    fn name(&self) -> &str {
        &self.exchange_name
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();

        // Simulate exchange API request
        tokio::time::sleep(Duration::from_millis(50)).await;

        let latency = start.elapsed().as_millis() as u64;

        ComponentHealth {
            name: format!("exchange_{}", self.exchange_name),
            status: HealthStatus::Healthy,
            message: format!("API {} responding", self.api_endpoint),
            latency_ms: latency,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Market data freshness check
pub struct MarketDataHealthCheck {
    max_staleness_seconds: u64,
    last_update: Arc<RwLock<Instant>>,
}

impl MarketDataHealthCheck {
    pub fn new(max_staleness_seconds: u64) -> Self {
        MarketDataHealthCheck {
            max_staleness_seconds,
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn update_timestamp(&self) {
        let mut last = self.last_update.write().await;
        *last = Instant::now();
    }
}

#[async_trait::async_trait]
impl HealthCheck for MarketDataHealthCheck {
    fn name(&self) -> &str {
        "market_data"
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();
        let last_update = self.last_update.read().await;
        let staleness = last_update.elapsed().as_secs();

        let (status, message) = if staleness < self.max_staleness_seconds {
            (HealthStatus::Healthy, format!("Data fresh ({}s ago)", staleness))
        } else if staleness < self.max_staleness_seconds * 2 {
            (HealthStatus::Degraded, format!("Data aging ({}s ago)", staleness))
        } else {
            (HealthStatus::Unhealthy, format!("Data stale ({}s ago)", staleness))
        };

        ComponentHealth {
            name: self.name().to_string(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Order queue health check
pub struct OrderQueueHealthCheck {
    queue_size: Arc<RwLock<usize>>,
    max_queue_size: usize,
}

impl OrderQueueHealthCheck {
    pub fn new(max_queue_size: usize) -> Self {
        OrderQueueHealthCheck {
            queue_size: Arc::new(RwLock::new(0)),
            max_queue_size,
        }
    }

    pub async fn set_queue_size(&self, size: usize) {
        let mut q = self.queue_size.write().await;
        *q = size;
    }
}

#[async_trait::async_trait]
impl HealthCheck for OrderQueueHealthCheck {
    fn name(&self) -> &str {
        "order_queue"
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();
        let queue_size = *self.queue_size.read().await;
        let queue_percent = (queue_size as f64 / self.max_queue_size as f64) * 100.0;

        let (status, message) = if queue_percent < 50.0 {
            (HealthStatus::Healthy, format!("Queue OK ({}/{})", queue_size, self.max_queue_size))
        } else if queue_percent < 80.0 {
            (HealthStatus::Degraded, format!("Queue filling ({:.0}%)", queue_percent))
        } else {
            (HealthStatus::Unhealthy, format!("Queue critical ({:.0}%)", queue_percent))
        };

        ComponentHealth {
            name: self.name().to_string(),
            status,
            message,
            latency_ms: start.elapsed().as_millis() as u64,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Health check manager
pub struct HealthCheckManager {
    checks: Vec<Box<dyn HealthCheck>>,
    start_time: Instant,
    version: String,
}

impl HealthCheckManager {
    pub fn new(version: &str) -> Self {
        HealthCheckManager {
            checks: Vec::new(),
            start_time: Instant::now(),
            version: version.to_string(),
        }
    }

    pub fn add_check(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    pub async fn run_all_checks(&self) -> HealthReport {
        let mut components = Vec::new();
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for check in &self.checks {
            let result = check.check().await;
            match result.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                _ => {}
            }
            components.push(result);
        }

        let overall_status = if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthReport {
            overall_status,
            components,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            version: self.version.clone(),
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Health Check System for Trading Bot ===\n");

    // Create health check manager
    let mut manager = HealthCheckManager::new("1.0.0");

    // Add checks
    manager.add_check(Box::new(DatabaseHealthCheck::new("postgres://localhost/trading")));
    manager.add_check(Box::new(ExchangeHealthCheck::new("binance", "api.binance.com")));
    manager.add_check(Box::new(ExchangeHealthCheck::new("kraken", "api.kraken.com")));

    let market_data_check = Arc::new(MarketDataHealthCheck::new(5));
    manager.add_check(Box::new(MarketDataHealthCheck::new(5)));

    let order_queue_check = Arc::new(OrderQueueHealthCheck::new(1000));
    manager.add_check(Box::new(OrderQueueHealthCheck::new(1000)));

    // Run checks
    let report = manager.run_all_checks().await;

    println!("Overall Status: {:?}", report.overall_status);
    println!("Uptime: {}s", report.uptime_seconds);
    println!("Version: {}\n", report.version);

    println!("Components:");
    for component in &report.components {
        println!(
            "  {} [{:?}]: {} ({}ms)",
            component.name, component.status, component.message, component.latency_ms
        );
    }

    // Output in JSON format
    println!("\n=== JSON Response ===");
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
```

## Kubernetes Integration

```rust
use axum::{routing::get, Router, Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Kubernetes probe configuration
#[derive(Clone)]
pub struct K8sProbeConfig {
    /// Startup timeout (for startup probe)
    pub startup_timeout_seconds: u64,
    /// Liveness failure threshold
    pub liveness_failure_threshold: u32,
    /// Readiness failure threshold
    pub readiness_failure_threshold: u32,
}

/// Application state for K8s
pub struct AppState {
    pub started: bool,
    pub ready: bool,
    pub live: bool,
    pub consecutive_failures: u32,
}

#[derive(Serialize)]
struct ProbeResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

/// Startup probe — checks if initialization completed
async fn startup_probe(state: Arc<RwLock<AppState>>) -> impl IntoResponse {
    let app = state.read().await;

    if app.started {
        (StatusCode::OK, Json(ProbeResponse {
            status: "started".to_string(),
            details: None,
        }))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(ProbeResponse {
            status: "starting".to_string(),
            details: Some("Initialization in progress".to_string()),
        }))
    }
}

/// Liveness probe — checks if process is hung
async fn liveness_probe(state: Arc<RwLock<AppState>>) -> impl IntoResponse {
    let app = state.read().await;

    if app.live {
        (StatusCode::OK, Json(ProbeResponse {
            status: "alive".to_string(),
            details: None,
        }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ProbeResponse {
            status: "dead".to_string(),
            details: Some("Process unresponsive".to_string()),
        }))
    }
}

/// Readiness probe — checks readiness to accept traffic
async fn readiness_probe(state: Arc<RwLock<AppState>>) -> impl IntoResponse {
    let app = state.read().await;

    if app.ready {
        (StatusCode::OK, Json(ProbeResponse {
            status: "ready".to_string(),
            details: None,
        }))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(ProbeResponse {
            status: "not_ready".to_string(),
            details: Some("Dependencies unavailable".to_string()),
        }))
    }
}

/*
Example Kubernetes manifest:

apiVersion: v1
kind: Pod
metadata:
  name: trading-bot
spec:
  containers:
  - name: trading-bot
    image: trading-bot:latest
    ports:
    - containerPort: 8080

    # Startup probe — allows time for initialization
    startupProbe:
      httpGet:
        path: /health/startup
        port: 8080
      initialDelaySeconds: 5
      periodSeconds: 5
      failureThreshold: 30  # 30 * 5 = 150 seconds for startup

    # Liveness probe — restarts hung pod
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
      periodSeconds: 10
      failureThreshold: 3

    # Readiness probe — removes pod from Service
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8080
      periodSeconds: 5
      failureThreshold: 2
*/

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(AppState {
        started: false,
        ready: false,
        live: true,
        consecutive_failures: 0,
    }));

    let state_startup = Arc::clone(&state);
    let state_liveness = Arc::clone(&state);
    let state_readiness = Arc::clone(&state);

    let app = Router::new()
        .route("/health/startup", get(move || {
            let s = Arc::clone(&state_startup);
            async move { startup_probe(s).await }
        }))
        .route("/health/live", get(move || {
            let s = Arc::clone(&state_liveness);
            async move { liveness_probe(s).await }
        }))
        .route("/health/ready", get(move || {
            let s = Arc::clone(&state_readiness);
            async move { readiness_probe(s).await }
        }));

    // Simulate startup process
    let state_init = Arc::clone(&state);
    tokio::spawn(async move {
        println!("Starting initialization...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        {
            let mut app = state_init.write().await;
            app.started = true;
            println!("Initialization complete");
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        {
            let mut app = state_init.write().await;
            app.ready = true;
            println!("Ready to accept traffic");
        }
    });

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("K8s probes available:");
    println!("  Startup:   http://{}/health/startup", addr);
    println!("  Liveness:  http://{}/health/live", addr);
    println!("  Readiness: http://{}/health/ready", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Deep Health Check for Critical Systems

```rust
use std::time::{Duration, Instant};
use serde::Serialize;

/// Deep trading system health check
#[derive(Debug, Serialize)]
pub struct DeepHealthCheck {
    pub timestamp: u64,
    pub overall_healthy: bool,
    pub checks: DeepChecks,
    pub performance: PerformanceMetrics,
    pub trading_status: TradingStatus,
}

#[derive(Debug, Serialize)]
pub struct DeepChecks {
    pub database_write_test: CheckDetail,
    pub database_read_test: CheckDetail,
    pub exchange_order_test: CheckDetail,
    pub market_data_freshness: CheckDetail,
    pub strategy_execution: CheckDetail,
    pub risk_limits: CheckDetail,
}

#[derive(Debug, Serialize)]
pub struct CheckDetail {
    pub passed: bool,
    pub latency_ms: u64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub avg_order_latency_ms: f64,
    pub orders_per_second: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct TradingStatus {
    pub trading_enabled: bool,
    pub active_positions: u32,
    pub pending_orders: u32,
    pub daily_pnl: f64,
    pub risk_utilization_percent: f64,
}

impl DeepHealthCheck {
    pub async fn run() -> Self {
        let start = Instant::now();

        // Database write check
        let db_write = Self::check_database_write().await;

        // Database read check
        let db_read = Self::check_database_read().await;

        // Exchange API check (test order)
        let exchange_test = Self::check_exchange_api().await;

        // Market data freshness check
        let market_data = Self::check_market_data().await;

        // Strategy execution check
        let strategy = Self::check_strategy_execution().await;

        // Risk limits check
        let risk = Self::check_risk_limits().await;

        let all_passed = db_write.passed
            && db_read.passed
            && exchange_test.passed
            && market_data.passed
            && strategy.passed
            && risk.passed;

        DeepHealthCheck {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            overall_healthy: all_passed,
            checks: DeepChecks {
                database_write_test: db_write,
                database_read_test: db_read,
                exchange_order_test: exchange_test,
                market_data_freshness: market_data,
                strategy_execution: strategy,
                risk_limits: risk,
            },
            performance: Self::collect_performance_metrics(),
            trading_status: Self::get_trading_status(),
        }
    }

    async fn check_database_write() -> CheckDetail {
        let start = Instant::now();
        // Simulate test row write
        tokio::time::sleep(Duration::from_millis(5)).await;

        CheckDetail {
            passed: true,
            latency_ms: start.elapsed().as_millis() as u64,
            message: "Write test passed".to_string(),
        }
    }

    async fn check_database_read() -> CheckDetail {
        let start = Instant::now();
        // Simulate test row read
        tokio::time::sleep(Duration::from_millis(3)).await;

        CheckDetail {
            passed: true,
            latency_ms: start.elapsed().as_millis() as u64,
            message: "Read test passed".to_string(),
        }
    }

    async fn check_exchange_api() -> CheckDetail {
        let start = Instant::now();
        // Simulate API check (get balance)
        tokio::time::sleep(Duration::from_millis(50)).await;

        CheckDetail {
            passed: true,
            latency_ms: start.elapsed().as_millis() as u64,
            message: "API responding, balance retrieved".to_string(),
        }
    }

    async fn check_market_data() -> CheckDetail {
        let start = Instant::now();
        // Simulate data freshness check
        let staleness_seconds = 2;

        CheckDetail {
            passed: staleness_seconds < 5,
            latency_ms: start.elapsed().as_millis() as u64,
            message: format!("Data age: {}s", staleness_seconds),
        }
    }

    async fn check_strategy_execution() -> CheckDetail {
        let start = Instant::now();
        // Simulate strategy execution check
        tokio::time::sleep(Duration::from_millis(10)).await;

        CheckDetail {
            passed: true,
            latency_ms: start.elapsed().as_millis() as u64,
            message: "Strategy engine responsive".to_string(),
        }
    }

    async fn check_risk_limits() -> CheckDetail {
        let start = Instant::now();
        let risk_percent = 45.0;

        CheckDetail {
            passed: risk_percent < 80.0,
            latency_ms: start.elapsed().as_millis() as u64,
            message: format!("Risk utilization: {:.1}%", risk_percent),
        }
    }

    fn collect_performance_metrics() -> PerformanceMetrics {
        PerformanceMetrics {
            avg_order_latency_ms: 45.5,
            orders_per_second: 150.0,
            memory_usage_mb: 512.0,
            cpu_usage_percent: 35.0,
        }
    }

    fn get_trading_status() -> TradingStatus {
        TradingStatus {
            trading_enabled: true,
            active_positions: 5,
            pending_orders: 12,
            daily_pnl: 1250.50,
            risk_utilization_percent: 45.0,
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Deep Health Check for Trading System ===\n");

    let report = DeepHealthCheck::run().await;

    println!("Overall Healthy: {}", report.overall_healthy);
    println!("\nChecks:");
    println!("  Database Write: {} ({} ms)",
        if report.checks.database_write_test.passed { "PASS" } else { "FAIL" },
        report.checks.database_write_test.latency_ms);
    println!("  Database Read: {} ({} ms)",
        if report.checks.database_read_test.passed { "PASS" } else { "FAIL" },
        report.checks.database_read_test.latency_ms);
    println!("  Exchange API: {} ({} ms)",
        if report.checks.exchange_order_test.passed { "PASS" } else { "FAIL" },
        report.checks.exchange_order_test.latency_ms);
    println!("  Market Data: {} - {}",
        if report.checks.market_data_freshness.passed { "PASS" } else { "FAIL" },
        report.checks.market_data_freshness.message);
    println!("  Strategy: {} ({} ms)",
        if report.checks.strategy_execution.passed { "PASS" } else { "FAIL" },
        report.checks.strategy_execution.latency_ms);
    println!("  Risk Limits: {} - {}",
        if report.checks.risk_limits.passed { "PASS" } else { "FAIL" },
        report.checks.risk_limits.message);

    println!("\nPerformance:");
    println!("  Avg Order Latency: {:.1} ms", report.performance.avg_order_latency_ms);
    println!("  Orders/sec: {:.0}", report.performance.orders_per_second);
    println!("  Memory: {:.0} MB", report.performance.memory_usage_mb);
    println!("  CPU: {:.0}%", report.performance.cpu_usage_percent);

    println!("\nTrading Status:");
    println!("  Trading Enabled: {}", report.trading_status.trading_enabled);
    println!("  Active Positions: {}", report.trading_status.active_positions);
    println!("  Pending Orders: {}", report.trading_status.pending_orders);
    println!("  Daily PnL: ${:.2}", report.trading_status.daily_pnl);
    println!("  Risk Utilization: {:.1}%", report.trading_status.risk_utilization_percent);

    println!("\n=== JSON Response ===");
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Liveness Probe** | Basic "is the process alive" check |
| **Readiness Probe** | Check for readiness to accept traffic |
| **Startup Probe** | Check for initialization completion |
| **Deep Health Check** | Comprehensive check of all components |
| **Health Status** | Healthy / Degraded / Unhealthy |
| **Component Health** | Status of individual system component |

## Practical Exercises

1. **WebSocket Health Check**: Create a health check for WebSocket connection to exchange:
   - Check connection activity
   - Measure latency (ping/pong)
   - Subscription status for instruments
   - Automatic reconnection

2. **Cascade Health Check**: Implement cascading dependency check:
   - If DB unavailable → system unhealthy
   - If exchange API slow → system degraded
   - If cache stale → warning
   - Aggregate status of all dependencies

3. **Health Check Dashboard**: Create a web interface for monitoring:
   - Display status of all components
   - Check history for the last hour
   - Latency graphs
   - Alerts on status changes

4. **Self-Healing Health Check**: Implement a self-healing system:
   - Upon detecting a problem → attempt automatic fix
   - Restart hung components
   - Switch to backup connections
   - Log all actions

## Homework

1. **Production Health Check System**: Develop a full-featured health check system:
   - Support for HTTP and gRPC endpoints
   - Custom checks via plugin system
   - Integration with Prometheus for metrics
   - Alerting via webhook
   - Support for graceful degradation

2. **Multi-Region Health Checker**: Create a system for multi-region health checking:
   - Check availability from different locations
   - Compare latency between regions
   - Automatic failover on problems
   - Dashboard with geographic map

3. **Trading-Specific Health Checks**: Implement specialized checks:
   - Order book synchronization check
   - Exchange balance validation
   - Test order execution verification
   - Slippage monitoring
   - API rate limit checking

4. **Health Check Automation**: Create an automation system:
   - Generate health check code from OpenAPI spec
   - Automatic dependency discovery
   - CI/CD integration for pre-deploy checks
   - Canary deployment with health verification

## Navigation

[← Previous day](../338-graceful-shutdown/en.md) | [Next day →](../340-metrics-prometheus/en.md)
