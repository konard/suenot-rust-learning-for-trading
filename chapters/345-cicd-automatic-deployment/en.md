# Day 345: CI/CD: Automatic Deployment

## Trading Analogy

Imagine you've developed a new trading strategy. Before running it with real money, you want to make sure it works correctly:

**Manual Deployment (old approach):**
Each time you manually check the strategy code, run tests, copy files to the server, restart the bot. This takes a lot of time and is prone to errors — you might forget to run tests or copy the wrong file.

**Automatic CI/CD (modern approach):**
You have an automated pipeline. When you make changes to the code:
1. **CI (Continuous Integration)** — tests run automatically, code is verified
2. **CD (Continuous Deployment)** — if everything passes, code is automatically deployed to the server

| Stage | Trading Analogy | Action |
|-------|-----------------|--------|
| **Commit** | Record strategy changes | Save code |
| **Build** | Compile the trading bot | Build application |
| **Test** | Run backtest on historical data | Run tests |
| **Stage** | Test on demo account | Deploy to staging |
| **Deploy** | Run on live account | Deploy to production |

## CI/CD Basics for Rust

### GitHub Actions Structure

GitHub Actions is a popular CI/CD tool. Configuration is stored in `.github/workflows/`:

```yaml
# .github/workflows/ci.yml
name: Trading Bot CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Stage 1: Code checking
  lint:
    name: Lint & Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  # Stage 2: Testing
  test:
    name: Test
    runs-on: ubuntu-latest
    needs: lint
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Run doc tests
        run: cargo test --doc

  # Stage 3: Build
  build:
    name: Build Release
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Build release
        run: cargo build --release

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: trading-bot
          path: target/release/trading-bot

  # Stage 4: Deploy
  deploy:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: build
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: trading-bot

      - name: Deploy to server
        run: |
          echo "Deploying trading bot to production..."
          # Deployment commands here
```

### Trading Bot Example with CI/CD

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Deployment configuration
#[derive(Debug, Clone)]
pub struct DeployConfig {
    pub environment: Environment,
    pub api_endpoint: String,
    pub max_position_size: f64,
    pub risk_limit_pct: f64,
    pub enable_trading: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl DeployConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let env = std::env::var("DEPLOY_ENV")
            .unwrap_or_else(|_| "development".to_string());

        let environment = match env.as_str() {
            "production" => Environment::Production,
            "staging" => Environment::Staging,
            _ => Environment::Development,
        };

        let api_endpoint = std::env::var("API_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let max_position_size: f64 = std::env::var("MAX_POSITION_SIZE")
            .unwrap_or_else(|_| "1000.0".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("MAX_POSITION_SIZE"))?;

        let risk_limit_pct: f64 = std::env::var("RISK_LIMIT_PCT")
            .unwrap_or_else(|_| "2.0".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("RISK_LIMIT_PCT"))?;

        // In production, trading is only enabled explicitly
        let enable_trading = std::env::var("ENABLE_TRADING")
            .map(|v| v == "true")
            .unwrap_or(environment != Environment::Production);

        Ok(DeployConfig {
            environment,
            api_endpoint,
            max_position_size,
            risk_limit_pct,
            enable_trading,
        })
    }

    /// Validate configuration before deployment
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_position_size <= 0.0 {
            return Err(ConfigError::InvalidValue("max_position_size must be positive"));
        }

        if self.risk_limit_pct <= 0.0 || self.risk_limit_pct > 100.0 {
            return Err(ConfigError::InvalidValue("risk_limit_pct must be between 0 and 100"));
        }

        // Additional checks for production
        if self.environment == Environment::Production {
            if self.api_endpoint.contains("localhost") {
                return Err(ConfigError::InvalidValue("localhost not allowed in production"));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidValue(&'static str),
    MissingEnvVar(&'static str),
}

/// Trading bot with hot reload support
pub struct TradingBot {
    config: Arc<RwLock<DeployConfig>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    version: String,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
}

impl Position {
    pub fn unrealized_pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }
}

impl TradingBot {
    pub fn new(config: DeployConfig) -> Self {
        TradingBot {
            config: Arc::new(RwLock::new(config)),
            positions: Arc::new(RwLock::new(HashMap::new())),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Health check for deployment
    pub async fn health_check(&self) -> HealthStatus {
        let config = self.config.read().await;
        let positions = self.positions.read().await;

        let total_pnl: f64 = positions.values()
            .map(|p| p.unrealized_pnl())
            .sum();

        HealthStatus {
            healthy: true,
            version: self.version.clone(),
            environment: format!("{:?}", config.environment),
            open_positions: positions.len(),
            total_unrealized_pnl: total_pnl,
            trading_enabled: config.enable_trading,
        }
    }

    /// Hot reload configuration without stopping the bot
    pub async fn hot_reload_config(&self, new_config: DeployConfig) -> Result<(), ConfigError> {
        new_config.validate()?;

        let mut config = self.config.write().await;

        println!("=== Hot Configuration Reload ===");
        println!("Old: {:?}", config.environment);
        println!("New: {:?}", new_config.environment);

        *config = new_config;

        Ok(())
    }

    /// Graceful shutdown — safe termination
    pub async fn graceful_shutdown(&self) -> ShutdownResult {
        println!("=== Starting graceful shutdown ===");

        let positions = self.positions.read().await;

        if !positions.is_empty() {
            println!("Open positions: {}", positions.len());
            println!("Closing all positions...");

            // In a real system, there would be exchange API calls here
            for (symbol, pos) in positions.iter() {
                println!("  Closing {}: {} @ {:.2}", symbol, pos.quantity, pos.current_price);
            }
        }

        println!("=== Shutdown completed ===");

        ShutdownResult {
            positions_closed: positions.len(),
            success: true,
        }
    }
}

#[derive(Debug)]
pub struct HealthStatus {
    pub healthy: bool,
    pub version: String,
    pub environment: String,
    pub open_positions: usize,
    pub total_unrealized_pnl: f64,
    pub trading_enabled: bool,
}

#[derive(Debug)]
pub struct ShutdownResult {
    pub positions_closed: usize,
    pub success: bool,
}

fn main() {
    println!("=== CI/CD: Automatic Deployment ===\n");

    // Demonstrate configuration loading
    std::env::set_var("DEPLOY_ENV", "staging");
    std::env::set_var("MAX_POSITION_SIZE", "5000.0");
    std::env::set_var("RISK_LIMIT_PCT", "1.5");

    let config = DeployConfig::from_env().expect("Failed to load config");
    println!("Loaded configuration: {:?}", config.environment);
    println!("  API: {}", config.api_endpoint);
    println!("  Max position: ${:.2}", config.max_position_size);
    println!("  Risk limit: {:.1}%", config.risk_limit_pct);
    println!("  Trading: {}", if config.enable_trading { "enabled" } else { "disabled" });

    // Validation
    match config.validate() {
        Ok(()) => println!("\nConfiguration is valid!"),
        Err(e) => println!("\nValidation error: {:?}", e),
    }
}
```

## Blue-Green Deployment

Blue-Green deployment is a strategy where two identical environments are maintained:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Blue-Green deployment for trading system
pub struct BlueGreenDeployment {
    blue_active: Arc<AtomicBool>,
    blue_version: String,
    green_version: String,
}

#[derive(Debug, Clone)]
pub struct DeploymentStatus {
    pub active_env: &'static str,
    pub active_version: String,
    pub standby_version: String,
}

impl BlueGreenDeployment {
    pub fn new(initial_version: String) -> Self {
        BlueGreenDeployment {
            blue_active: Arc::new(AtomicBool::new(true)),
            blue_version: initial_version.clone(),
            green_version: initial_version,
        }
    }

    /// Deploy new version to standby environment
    pub fn deploy_to_standby(&mut self, new_version: String) -> Result<(), DeployError> {
        println!("Deploying version {} to standby environment...", new_version);

        if self.blue_active.load(Ordering::SeqCst) {
            // Blue is active, deploy to Green
            self.green_version = new_version;
            println!("  -> Deployed to Green");
        } else {
            // Green is active, deploy to Blue
            self.blue_version = new_version;
            println!("  -> Deployed to Blue");
        }

        Ok(())
    }

    /// Switch traffic to new version
    pub fn switch_traffic(&self) -> DeploymentStatus {
        let was_blue = self.blue_active.fetch_xor(true, Ordering::SeqCst);

        if was_blue {
            println!("Traffic switched: Blue -> Green");
        } else {
            println!("Traffic switched: Green -> Blue");
        }

        self.status()
    }

    /// Rollback to previous version
    pub fn rollback(&self) -> DeploymentStatus {
        println!("Rolling back to previous version...");
        self.switch_traffic()
    }

    /// Current deployment status
    pub fn status(&self) -> DeploymentStatus {
        let blue_active = self.blue_active.load(Ordering::SeqCst);

        DeploymentStatus {
            active_env: if blue_active { "Blue" } else { "Green" },
            active_version: if blue_active {
                self.blue_version.clone()
            } else {
                self.green_version.clone()
            },
            standby_version: if blue_active {
                self.green_version.clone()
            } else {
                self.blue_version.clone()
            },
        }
    }
}

#[derive(Debug)]
pub enum DeployError {
    ValidationFailed(String),
    DeploymentFailed(String),
}

fn main() {
    println!("=== Blue-Green Deployment ===\n");

    let mut deployment = BlueGreenDeployment::new("v1.0.0".to_string());

    println!("Initial status: {:?}\n", deployment.status());

    // Deploy new version
    deployment.deploy_to_standby("v1.1.0".to_string()).unwrap();
    println!("After deployment: {:?}\n", deployment.status());

    // Switch traffic
    let status = deployment.switch_traffic();
    println!("After switch: {:?}\n", status);

    // Rollback (if something went wrong)
    let status = deployment.rollback();
    println!("After rollback: {:?}", status);
}
```

## Canary Deployment

Canary deployment is a gradual rollout of a new version to a portion of traffic:

```rust
use std::collections::HashMap;

/// Canary deployment — gradual rollout
pub struct CanaryDeployment {
    stable_version: String,
    canary_version: Option<String>,
    canary_percentage: u8,  // 0-100
    metrics: DeployMetrics,
}

#[derive(Debug, Default)]
pub struct DeployMetrics {
    pub stable_requests: u64,
    pub canary_requests: u64,
    pub stable_errors: u64,
    pub canary_errors: u64,
    pub stable_latency_sum_ms: u64,
    pub canary_latency_sum_ms: u64,
}

impl DeployMetrics {
    pub fn stable_error_rate(&self) -> f64 {
        if self.stable_requests == 0 {
            0.0
        } else {
            self.stable_errors as f64 / self.stable_requests as f64 * 100.0
        }
    }

    pub fn canary_error_rate(&self) -> f64 {
        if self.canary_requests == 0 {
            0.0
        } else {
            self.canary_errors as f64 / self.canary_requests as f64 * 100.0
        }
    }

    pub fn stable_avg_latency(&self) -> f64 {
        if self.stable_requests == 0 {
            0.0
        } else {
            self.stable_latency_sum_ms as f64 / self.stable_requests as f64
        }
    }

    pub fn canary_avg_latency(&self) -> f64 {
        if self.canary_requests == 0 {
            0.0
        } else {
            self.canary_latency_sum_ms as f64 / self.canary_requests as f64
        }
    }
}

impl CanaryDeployment {
    pub fn new(stable_version: String) -> Self {
        CanaryDeployment {
            stable_version,
            canary_version: None,
            canary_percentage: 0,
            metrics: DeployMetrics::default(),
        }
    }

    /// Start canary with specified traffic percentage
    pub fn start_canary(&mut self, version: String, percentage: u8) {
        println!(
            "Starting canary: version {} with {}% traffic",
            version, percentage
        );
        self.canary_version = Some(version);
        self.canary_percentage = percentage.min(100);
        self.metrics = DeployMetrics::default();
    }

    /// Determine which version handles the request
    pub fn route_request(&self) -> &str {
        let random: u8 = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() % 100) as u8;

        if let Some(ref canary) = self.canary_version {
            if random < self.canary_percentage {
                return canary;
            }
        }
        &self.stable_version
    }

    /// Increase canary traffic percentage
    pub fn increase_canary(&mut self, new_percentage: u8) {
        let old = self.canary_percentage;
        self.canary_percentage = new_percentage.min(100);
        println!(
            "Canary traffic increased: {}% -> {}%",
            old, self.canary_percentage
        );
    }

    /// Promote canary to stable
    pub fn promote_canary(&mut self) -> Result<(), &'static str> {
        match &self.canary_version {
            Some(version) => {
                println!("Promoting canary {} to stable", version);
                self.stable_version = version.clone();
                self.canary_version = None;
                self.canary_percentage = 0;
                Ok(())
            }
            None => Err("No canary version to promote"),
        }
    }

    /// Rollback canary
    pub fn rollback_canary(&mut self) {
        if self.canary_version.is_some() {
            println!("Rolling back canary version");
            self.canary_version = None;
            self.canary_percentage = 0;
        }
    }

    /// Check metrics for automatic promotion decision
    pub fn should_promote(&self) -> bool {
        // Promote if canary is not worse than stable
        let error_diff = self.metrics.canary_error_rate() - self.metrics.stable_error_rate();
        let latency_diff = self.metrics.canary_avg_latency() - self.metrics.stable_avg_latency();

        // Allow slight degradation
        error_diff < 0.5 && latency_diff < 10.0
    }

    /// Check metrics for automatic rollback decision
    pub fn should_rollback(&self) -> bool {
        // Rollback if canary is significantly worse
        let error_rate = self.metrics.canary_error_rate();
        let stable_error_rate = self.metrics.stable_error_rate();

        // Rollback if errors are twice as many or more than 5%
        error_rate > stable_error_rate * 2.0 || error_rate > 5.0
    }

    /// Record request for demonstration
    pub fn record_request(&mut self, is_canary: bool, success: bool, latency_ms: u64) {
        if is_canary {
            self.metrics.canary_requests += 1;
            self.metrics.canary_latency_sum_ms += latency_ms;
            if !success {
                self.metrics.canary_errors += 1;
            }
        } else {
            self.metrics.stable_requests += 1;
            self.metrics.stable_latency_sum_ms += latency_ms;
            if !success {
                self.metrics.stable_errors += 1;
            }
        }
    }
}

fn main() {
    println!("=== Canary Deployment ===\n");

    let mut deployment = CanaryDeployment::new("v1.0.0".to_string());

    // Start canary with 10% traffic
    deployment.start_canary("v1.1.0".to_string(), 10);

    // Simulate requests
    println!("\nSimulating 100 requests...");
    for i in 0..100 {
        let version = deployment.route_request();
        let is_canary = version == "v1.1.0";
        let success = i % 20 != 0;  // 95% success rate
        let latency = if is_canary { 45 } else { 50 };  // Canary slightly faster
        deployment.record_request(is_canary, success, latency);
    }

    // Analyze metrics
    println!("\nMetrics:");
    println!(
        "  Stable: {} requests, {:.1}% errors, {:.1}ms latency",
        deployment.metrics.stable_requests,
        deployment.metrics.stable_error_rate(),
        deployment.metrics.stable_avg_latency()
    );
    println!(
        "  Canary: {} requests, {:.1}% errors, {:.1}ms latency",
        deployment.metrics.canary_requests,
        deployment.metrics.canary_error_rate(),
        deployment.metrics.canary_avg_latency()
    );

    // Decision on promotion
    if deployment.should_rollback() {
        println!("\nDecision: ROLLBACK (metrics degraded)");
        deployment.rollback_canary();
    } else if deployment.should_promote() {
        println!("\nDecision: PROMOTE (metrics acceptable)");
        deployment.promote_canary().unwrap();
    } else {
        println!("\nDecision: Increase canary traffic");
        deployment.increase_canary(25);
    }
}
```

## Docker and Kubernetes for Trading Systems

### Dockerfile for Rust Bot

```dockerfile
# Dockerfile
# Stage 1: Build
FROM rust:1.75 as builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build application
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/trading-bot /usr/local/bin/

# Non-root user for security
RUN useradd -m -u 1000 trader
USER trader

ENV RUST_LOG=info

ENTRYPOINT ["trading-bot"]
```

### Kubernetes Manifests

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: trading-bot
  labels:
    app: trading-bot
spec:
  replicas: 2
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 0
      maxSurge: 1
  selector:
    matchLabels:
      app: trading-bot
  template:
    metadata:
      labels:
        app: trading-bot
    spec:
      containers:
      - name: trading-bot
        image: trading-bot:latest
        ports:
        - containerPort: 8080
        env:
        - name: DEPLOY_ENV
          value: "production"
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 15
          periodSeconds: 20
---
apiVersion: v1
kind: Service
metadata:
  name: trading-bot
spec:
  selector:
    app: trading-bot
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

## Health Checks and Readiness Probes

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service health state
#[derive(Debug, Clone)]
pub struct HealthState {
    pub ready: bool,
    pub live: bool,
    pub exchange_connected: bool,
    pub database_connected: bool,
    pub last_heartbeat: std::time::Instant,
}

impl Default for HealthState {
    fn default() -> Self {
        HealthState {
            ready: false,
            live: true,
            exchange_connected: false,
            database_connected: false,
            last_heartbeat: std::time::Instant::now(),
        }
    }
}

/// Health check component
pub struct HealthChecker {
    state: Arc<RwLock<HealthState>>,
    startup_timeout_secs: u64,
    heartbeat_timeout_secs: u64,
}

impl HealthChecker {
    pub fn new() -> Self {
        HealthChecker {
            state: Arc::new(RwLock::new(HealthState::default())),
            startup_timeout_secs: 30,
            heartbeat_timeout_secs: 60,
        }
    }

    /// Liveness check — is the process alive?
    pub async fn liveness(&self) -> LivenessResponse {
        let state = self.state.read().await;

        let heartbeat_age = state.last_heartbeat.elapsed().as_secs();
        let is_live = state.live && heartbeat_age < self.heartbeat_timeout_secs;

        LivenessResponse {
            status: if is_live { "ok" } else { "error" },
            live: is_live,
            heartbeat_age_secs: heartbeat_age,
        }
    }

    /// Readiness check — ready to accept traffic?
    pub async fn readiness(&self) -> ReadinessResponse {
        let state = self.state.read().await;

        let is_ready = state.ready
            && state.exchange_connected
            && state.database_connected;

        ReadinessResponse {
            status: if is_ready { "ok" } else { "not_ready" },
            ready: is_ready,
            checks: HealthChecks {
                exchange: state.exchange_connected,
                database: state.database_connected,
            },
        }
    }

    /// Update component status
    pub async fn update_component(&self, component: &str, healthy: bool) {
        let mut state = self.state.write().await;

        match component {
            "exchange" => state.exchange_connected = healthy,
            "database" => state.database_connected = healthy,
            _ => {}
        }

        state.last_heartbeat = std::time::Instant::now();

        // Ready if all components are healthy
        state.ready = state.exchange_connected && state.database_connected;
    }

    /// Mark service as unhealthy
    pub async fn mark_unhealthy(&self, reason: &str) {
        let mut state = self.state.write().await;
        state.live = false;
        println!("Service marked unhealthy: {}", reason);
    }
}

#[derive(Debug)]
pub struct LivenessResponse {
    pub status: &'static str,
    pub live: bool,
    pub heartbeat_age_secs: u64,
}

#[derive(Debug)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub ready: bool,
    pub checks: HealthChecks,
}

#[derive(Debug)]
pub struct HealthChecks {
    pub exchange: bool,
    pub database: bool,
}

#[tokio::main]
async fn main() {
    println!("=== Health Checks Demo ===\n");

    let checker = HealthChecker::new();

    // Initial state
    println!("Initial state:");
    println!("  Liveness: {:?}", checker.liveness().await);
    println!("  Readiness: {:?}", checker.readiness().await);

    // Simulate startup
    println!("\nStarting components...");
    checker.update_component("database", true).await;
    println!("  Database connected");

    checker.update_component("exchange", true).await;
    println!("  Exchange connected");

    // Check after startup
    println!("\nAfter startup:");
    println!("  Liveness: {:?}", checker.liveness().await);
    println!("  Readiness: {:?}", checker.readiness().await);

    // Simulate exchange disconnection
    println!("\nExchange disconnected...");
    checker.update_component("exchange", false).await;

    println!("After disconnection:");
    println!("  Liveness: {:?}", checker.liveness().await);
    println!("  Readiness: {:?}", checker.readiness().await);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **CI (Continuous Integration)** | Automatic build and testing on every commit |
| **CD (Continuous Deployment)** | Automatic deployment after successful CI |
| **Blue-Green Deployment** | Two identical environments for safe switching |
| **Canary Deployment** | Gradual rollout to a portion of traffic |
| **Health Checks** | Liveness and readiness probes |
| **Graceful Shutdown** | Proper termination with position closing |
| **Hot Reload** | Configuration update without stopping |

## Practical Exercises

1. **CI Pipeline for Trading Bot**: Create a GitHub Actions workflow that:
   - Checks code formatting
   - Runs clippy with maximum checks
   - Executes unit and integration tests
   - Builds Docker image
   - Pushes image to registry

2. **Blue-Green Controller**: Implement a system that:
   - Manages two environments (blue/green)
   - Automatically switches traffic
   - Monitors metrics after switching
   - Performs automatic rollback on issues

3. **Canary with Automatic Analysis**: Create a system:
   - Automatically increases canary traffic percentage
   - Analyzes metrics in real-time
   - Makes promote/rollback decisions
   - Notifies on issues

4. **Health Check System**: Develop a component:
   - Checks all dependencies (database, exchange, cache)
   - Supports liveness and readiness probes
   - Collects health metrics
   - Integrates with Kubernetes

## Homework

1. **Full CI/CD Pipeline**: Set up a pipeline that:
   - Triggers on push to develop and main
   - Includes all code quality checks
   - Deploys to staging on develop
   - Deploys to production on main with manual approval
   - Sends notifications to Slack/Telegram
   - Stores artifacts and test reports

2. **Versioning System**: Implement:
   - Semantic versioning for releases
   - Automatic changelog updates
   - Git tags for releases
   - Version display in health check
   - Configuration compatibility checking

3. **Disaster Recovery Plan**: Create a system:
   - Automatic backup of positions and state
   - Recovery from backup on startup
   - Data integrity verification
   - Read-only mode on issues
   - Recovery procedure documentation

4. **Deployment Monitoring**: Develop a dashboard:
   - Deployment and rollback times
   - Deployment frequency by environment
   - Success metrics (MTTR, MTBF)
   - Anomaly alerts
   - Prometheus/Grafana integration

## Navigation

[← Previous day](../326-async-vs-threading/en.md) | [Next day →](../346-*/en.md)
