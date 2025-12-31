# Day 360: Canary Deployments

## Trading Analogy

Imagine you're a hedge fund manager with a new trading algorithm. You're confident it works well in backtests, but deploying it to manage your entire $100 million portfolio at once is terrifying. What if there's a bug that causes catastrophic losses?

**The Canary Approach in Trading:**
Just like coal miners used canaries to detect dangerous gases, you can use a small portion of your capital to "test the air" before full deployment:

1. **Start Small**: Deploy the new algorithm with just 1% of capital ($1M)
2. **Monitor Closely**: Watch key metrics — P&L, slippage, execution quality
3. **Gradual Increase**: If metrics are good, increase to 5%, then 10%, then 25%
4. **Quick Rollback**: If something goes wrong, instantly revert to the old algorithm

| Phase | Capital Allocation | Risk Level | Action on Problem |
|-------|-------------------|------------|-------------------|
| **Canary** | 1-5% | Minimal | Instant rollback |
| **Early Adopter** | 10-25% | Low | Fast rollback |
| **Majority** | 50-75% | Medium | Controlled rollback |
| **Full Rollout** | 100% | Full | Old version removed |

**Why "Canary"?**
In coal mines, canaries were more sensitive to toxic gases than humans. They would show signs of distress before conditions became dangerous for miners. Similarly, a canary deployment exposes a small portion of traffic to the new version first — if there are problems, they affect only a small subset of users/capital.

## Canary Deployment Fundamentals

### The Canary Deployment Pattern

```rust
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Canary deployment state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanaryState {
    /// No canary active, all traffic to stable
    Stable,
    /// Canary running with specified percentage
    Active { percentage: u8 },
    /// Canary promoted to stable
    Promoted,
    /// Canary rolled back due to issues
    RolledBack,
}

/// Metrics collected during canary deployment
#[derive(Debug, Default)]
pub struct CanaryMetrics {
    // Stable version metrics
    pub stable_requests: AtomicU64,
    pub stable_errors: AtomicU64,
    pub stable_latency_sum_ms: AtomicU64,

    // Canary version metrics
    pub canary_requests: AtomicU64,
    pub canary_errors: AtomicU64,
    pub canary_latency_sum_ms: AtomicU64,
}

impl CanaryMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_stable(&self, success: bool, latency_ms: u64) {
        self.stable_requests.fetch_add(1, Ordering::Relaxed);
        self.stable_latency_sum_ms.fetch_add(latency_ms, Ordering::Relaxed);
        if !success {
            self.stable_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_canary(&self, success: bool, latency_ms: u64) {
        self.canary_requests.fetch_add(1, Ordering::Relaxed);
        self.canary_latency_sum_ms.fetch_add(latency_ms, Ordering::Relaxed);
        if !success {
            self.canary_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn stable_error_rate(&self) -> f64 {
        let requests = self.stable_requests.load(Ordering::Relaxed);
        if requests == 0 { return 0.0; }
        let errors = self.stable_errors.load(Ordering::Relaxed);
        (errors as f64 / requests as f64) * 100.0
    }

    pub fn canary_error_rate(&self) -> f64 {
        let requests = self.canary_requests.load(Ordering::Relaxed);
        if requests == 0 { return 0.0; }
        let errors = self.canary_errors.load(Ordering::Relaxed);
        (errors as f64 / requests as f64) * 100.0
    }

    pub fn stable_avg_latency(&self) -> f64 {
        let requests = self.stable_requests.load(Ordering::Relaxed);
        if requests == 0 { return 0.0; }
        let sum = self.stable_latency_sum_ms.load(Ordering::Relaxed);
        sum as f64 / requests as f64
    }

    pub fn canary_avg_latency(&self) -> f64 {
        let requests = self.canary_requests.load(Ordering::Relaxed);
        if requests == 0 { return 0.0; }
        let sum = self.canary_latency_sum_ms.load(Ordering::Relaxed);
        sum as f64 / requests as f64
    }

    pub fn reset(&self) {
        self.stable_requests.store(0, Ordering::Relaxed);
        self.stable_errors.store(0, Ordering::Relaxed);
        self.stable_latency_sum_ms.store(0, Ordering::Relaxed);
        self.canary_requests.store(0, Ordering::Relaxed);
        self.canary_errors.store(0, Ordering::Relaxed);
        self.canary_latency_sum_ms.store(0, Ordering::Relaxed);
    }
}

/// Canary deployment controller
pub struct CanaryDeployment {
    stable_version: String,
    canary_version: Option<String>,
    canary_percentage: AtomicU8,
    metrics: Arc<CanaryMetrics>,
    started_at: Option<Instant>,
}

impl CanaryDeployment {
    pub fn new(stable_version: String) -> Self {
        CanaryDeployment {
            stable_version,
            canary_version: None,
            canary_percentage: AtomicU8::new(0),
            metrics: Arc::new(CanaryMetrics::new()),
            started_at: None,
        }
    }

    /// Start a canary deployment with initial percentage
    pub fn start_canary(&mut self, version: String, initial_percentage: u8) {
        println!("Starting canary deployment:");
        println!("  Stable version: {}", self.stable_version);
        println!("  Canary version: {}", version);
        println!("  Initial traffic: {}%", initial_percentage);

        self.canary_version = Some(version);
        self.canary_percentage.store(initial_percentage.min(100), Ordering::SeqCst);
        self.metrics.reset();
        self.started_at = Some(Instant::now());
    }

    /// Route a request to either stable or canary
    pub fn route_request(&self, request_id: u64) -> &str {
        let percentage = self.canary_percentage.load(Ordering::Relaxed);

        if percentage == 0 || self.canary_version.is_none() {
            return &self.stable_version;
        }

        // Use request_id for deterministic routing
        let bucket = (request_id % 100) as u8;

        if bucket < percentage {
            self.canary_version.as_ref().unwrap()
        } else {
            &self.stable_version
        }
    }

    /// Check if request should go to canary
    pub fn is_canary_request(&self, request_id: u64) -> bool {
        let percentage = self.canary_percentage.load(Ordering::Relaxed);
        if percentage == 0 { return false; }
        (request_id % 100) as u8 < percentage
    }

    /// Increase canary traffic percentage
    pub fn increase_traffic(&self, new_percentage: u8) -> u8 {
        let old = self.canary_percentage.load(Ordering::Relaxed);
        let new = new_percentage.min(100);
        self.canary_percentage.store(new, Ordering::SeqCst);
        println!("Canary traffic increased: {}% -> {}%", old, new);
        new
    }

    /// Promote canary to stable (100% traffic)
    pub fn promote(&mut self) -> Result<String, &'static str> {
        match self.canary_version.take() {
            Some(version) => {
                println!("Promoting canary {} to stable", version);
                self.stable_version = version.clone();
                self.canary_percentage.store(0, Ordering::SeqCst);
                self.started_at = None;
                Ok(version)
            }
            None => Err("No canary version to promote"),
        }
    }

    /// Rollback canary (0% traffic, remove canary)
    pub fn rollback(&mut self) -> Option<String> {
        if let Some(version) = self.canary_version.take() {
            println!("Rolling back canary version: {}", version);
            self.canary_percentage.store(0, Ordering::SeqCst);
            self.started_at = None;
            Some(version)
        } else {
            None
        }
    }

    /// Get current deployment status
    pub fn status(&self) -> CanaryStatus {
        let percentage = self.canary_percentage.load(Ordering::Relaxed);
        let duration = self.started_at.map(|s| s.elapsed());

        CanaryStatus {
            stable_version: self.stable_version.clone(),
            canary_version: self.canary_version.clone(),
            canary_percentage: percentage,
            duration,
            metrics: Arc::clone(&self.metrics),
        }
    }
}

#[derive(Debug)]
pub struct CanaryStatus {
    pub stable_version: String,
    pub canary_version: Option<String>,
    pub canary_percentage: u8,
    pub duration: Option<Duration>,
    pub metrics: Arc<CanaryMetrics>,
}

impl CanaryStatus {
    pub fn print_report(&self) {
        println!("\n=== Canary Deployment Status ===");
        println!("Stable version: {}", self.stable_version);

        if let Some(ref canary) = self.canary_version {
            println!("Canary version: {} ({}% traffic)", canary, self.canary_percentage);

            if let Some(duration) = self.duration {
                println!("Running for: {:.1} minutes", duration.as_secs_f64() / 60.0);
            }

            println!("\nMetrics Comparison:");
            println!("  Stable  - Requests: {}, Error rate: {:.2}%, Avg latency: {:.1}ms",
                self.metrics.stable_requests.load(Ordering::Relaxed),
                self.metrics.stable_error_rate(),
                self.metrics.stable_avg_latency());
            println!("  Canary  - Requests: {}, Error rate: {:.2}%, Avg latency: {:.1}ms",
                self.metrics.canary_requests.load(Ordering::Relaxed),
                self.metrics.canary_error_rate(),
                self.metrics.canary_avg_latency());
        } else {
            println!("No canary deployment active");
        }
    }
}

fn main() {
    println!("=== Canary Deployment Fundamentals ===\n");

    let mut deployment = CanaryDeployment::new("v1.0.0".to_string());

    // Start canary with 10% traffic
    deployment.start_canary("v1.1.0".to_string(), 10);

    // Simulate requests
    println!("\nSimulating 1000 requests...");
    for i in 0..1000 {
        let is_canary = deployment.is_canary_request(i);
        let version = deployment.route_request(i);

        // Simulate success/failure (canary slightly better in this example)
        let success = if is_canary {
            i % 50 != 0  // 98% success rate
        } else {
            i % 25 != 0  // 96% success rate
        };

        let latency = if is_canary { 45 } else { 50 };

        if is_canary {
            deployment.status().metrics.record_canary(success, latency);
        } else {
            deployment.status().metrics.record_stable(success, latency);
        }
    }

    deployment.status().print_report();

    // Increase traffic
    println!("\n--- Increasing canary traffic ---");
    deployment.increase_traffic(25);

    // Simulate more requests
    for i in 1000..2000 {
        let is_canary = deployment.is_canary_request(i);
        let success = if is_canary { i % 50 != 0 } else { i % 25 != 0 };
        let latency = if is_canary { 45 } else { 50 };

        if is_canary {
            deployment.status().metrics.record_canary(success, latency);
        } else {
            deployment.status().metrics.record_stable(success, latency);
        }
    }

    deployment.status().print_report();

    // Promote canary
    println!("\n--- Promoting canary ---");
    deployment.promote().unwrap();

    deployment.status().print_report();
}
```

## Automated Canary Analysis for Trading Systems

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Trading-specific canary metrics
#[derive(Debug)]
pub struct TradingCanaryMetrics {
    // Order execution metrics
    pub stable_orders: AtomicU64,
    pub canary_orders: AtomicU64,
    pub stable_fills: AtomicU64,
    pub canary_fills: AtomicU64,
    pub stable_rejects: AtomicU64,
    pub canary_rejects: AtomicU64,

    // P&L tracking (stored in cents, offset by 1B for negative values)
    pub stable_pnl_cents: AtomicU64,
    pub canary_pnl_cents: AtomicU64,

    // Latency (microseconds)
    pub stable_latency_sum_us: AtomicU64,
    pub canary_latency_sum_us: AtomicU64,

    // Slippage (basis points * 100)
    pub stable_slippage_sum: AtomicU64,
    pub canary_slippage_sum: AtomicU64,
}

impl TradingCanaryMetrics {
    const PNL_OFFSET: i64 = 1_000_000_000_00; // $1B offset in cents

    pub fn new() -> Self {
        TradingCanaryMetrics {
            stable_orders: AtomicU64::new(0),
            canary_orders: AtomicU64::new(0),
            stable_fills: AtomicU64::new(0),
            canary_fills: AtomicU64::new(0),
            stable_rejects: AtomicU64::new(0),
            canary_rejects: AtomicU64::new(0),
            stable_pnl_cents: AtomicU64::new(Self::PNL_OFFSET as u64),
            canary_pnl_cents: AtomicU64::new(Self::PNL_OFFSET as u64),
            stable_latency_sum_us: AtomicU64::new(0),
            canary_latency_sum_us: AtomicU64::new(0),
            stable_slippage_sum: AtomicU64::new(0),
            canary_slippage_sum: AtomicU64::new(0),
        }
    }

    pub fn record_stable_order(&self, filled: bool, pnl_cents: i64, latency_us: u64, slippage_bp: f64) {
        self.stable_orders.fetch_add(1, Ordering::Relaxed);
        if filled {
            self.stable_fills.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stable_rejects.fetch_add(1, Ordering::Relaxed);
        }

        // Update P&L
        if pnl_cents >= 0 {
            self.stable_pnl_cents.fetch_add(pnl_cents as u64, Ordering::Relaxed);
        } else {
            self.stable_pnl_cents.fetch_sub((-pnl_cents) as u64, Ordering::Relaxed);
        }

        self.stable_latency_sum_us.fetch_add(latency_us, Ordering::Relaxed);
        self.stable_slippage_sum.fetch_add((slippage_bp * 100.0) as u64, Ordering::Relaxed);
    }

    pub fn record_canary_order(&self, filled: bool, pnl_cents: i64, latency_us: u64, slippage_bp: f64) {
        self.canary_orders.fetch_add(1, Ordering::Relaxed);
        if filled {
            self.canary_fills.fetch_add(1, Ordering::Relaxed);
        } else {
            self.canary_rejects.fetch_add(1, Ordering::Relaxed);
        }

        if pnl_cents >= 0 {
            self.canary_pnl_cents.fetch_add(pnl_cents as u64, Ordering::Relaxed);
        } else {
            self.canary_pnl_cents.fetch_sub((-pnl_cents) as u64, Ordering::Relaxed);
        }

        self.canary_latency_sum_us.fetch_add(latency_us, Ordering::Relaxed);
        self.canary_slippage_sum.fetch_add((slippage_bp * 100.0) as u64, Ordering::Relaxed);
    }

    pub fn stable_fill_rate(&self) -> f64 {
        let orders = self.stable_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let fills = self.stable_fills.load(Ordering::Relaxed);
        (fills as f64 / orders as f64) * 100.0
    }

    pub fn canary_fill_rate(&self) -> f64 {
        let orders = self.canary_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let fills = self.canary_fills.load(Ordering::Relaxed);
        (fills as f64 / orders as f64) * 100.0
    }

    pub fn stable_pnl(&self) -> f64 {
        let cents = self.stable_pnl_cents.load(Ordering::Relaxed) as i64;
        (cents - Self::PNL_OFFSET) as f64 / 100.0
    }

    pub fn canary_pnl(&self) -> f64 {
        let cents = self.canary_pnl_cents.load(Ordering::Relaxed) as i64;
        (cents - Self::PNL_OFFSET) as f64 / 100.0
    }

    pub fn stable_avg_latency_ms(&self) -> f64 {
        let orders = self.stable_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let sum = self.stable_latency_sum_us.load(Ordering::Relaxed);
        (sum as f64 / orders as f64) / 1000.0
    }

    pub fn canary_avg_latency_ms(&self) -> f64 {
        let orders = self.canary_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let sum = self.canary_latency_sum_us.load(Ordering::Relaxed);
        (sum as f64 / orders as f64) / 1000.0
    }

    pub fn stable_avg_slippage_bp(&self) -> f64 {
        let orders = self.stable_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let sum = self.stable_slippage_sum.load(Ordering::Relaxed);
        (sum as f64 / orders as f64) / 100.0
    }

    pub fn canary_avg_slippage_bp(&self) -> f64 {
        let orders = self.canary_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let sum = self.canary_slippage_sum.load(Ordering::Relaxed);
        (sum as f64 / orders as f64) / 100.0
    }
}

/// Canary analysis decision
#[derive(Debug, Clone, PartialEq)]
pub enum CanaryDecision {
    /// Continue monitoring, not enough data
    Continue,
    /// Increase traffic percentage
    IncreaseTraffic(u8),
    /// Promote canary to stable
    Promote,
    /// Rollback canary immediately
    Rollback(String),
}

/// Thresholds for automatic canary decisions
#[derive(Debug, Clone)]
pub struct CanaryThresholds {
    /// Minimum requests before making decisions
    pub min_requests: u64,
    /// Maximum allowed error rate increase (percentage points)
    pub max_error_rate_increase: f64,
    /// Maximum allowed latency increase (percentage)
    pub max_latency_increase_pct: f64,
    /// Maximum allowed slippage increase (basis points)
    pub max_slippage_increase_bp: f64,
    /// Minimum P&L per order compared to stable (as ratio)
    pub min_pnl_ratio: f64,
    /// Traffic increase steps
    pub traffic_steps: Vec<u8>,
}

impl Default for CanaryThresholds {
    fn default() -> Self {
        CanaryThresholds {
            min_requests: 100,
            max_error_rate_increase: 1.0,   // 1 percentage point
            max_latency_increase_pct: 20.0, // 20% slower is acceptable
            max_slippage_increase_bp: 0.5,  // 0.5 basis points
            min_pnl_ratio: 0.8,             // Canary P&L should be at least 80% of stable
            traffic_steps: vec![5, 10, 25, 50, 75, 100],
        }
    }
}

/// Automated canary analyzer for trading systems
pub struct TradingCanaryAnalyzer {
    metrics: Arc<TradingCanaryMetrics>,
    thresholds: CanaryThresholds,
    current_percentage: u8,
    is_halted: AtomicBool,
}

impl TradingCanaryAnalyzer {
    pub fn new(thresholds: CanaryThresholds) -> Self {
        TradingCanaryAnalyzer {
            metrics: Arc::new(TradingCanaryMetrics::new()),
            thresholds,
            current_percentage: 0,
            is_halted: AtomicBool::new(false),
        }
    }

    pub fn metrics(&self) -> Arc<TradingCanaryMetrics> {
        Arc::clone(&self.metrics)
    }

    pub fn set_percentage(&mut self, pct: u8) {
        self.current_percentage = pct;
    }

    /// Analyze current metrics and return decision
    pub fn analyze(&self) -> CanaryDecision {
        // Check if halted
        if self.is_halted.load(Ordering::Relaxed) {
            return CanaryDecision::Continue;
        }

        let canary_requests = self.metrics.canary_orders.load(Ordering::Relaxed);
        let stable_requests = self.metrics.stable_orders.load(Ordering::Relaxed);

        // Not enough data yet
        if canary_requests < self.thresholds.min_requests {
            return CanaryDecision::Continue;
        }

        // Check rejection rate
        let stable_fill_rate = self.metrics.stable_fill_rate();
        let canary_fill_rate = self.metrics.canary_fill_rate();
        let fill_rate_diff = stable_fill_rate - canary_fill_rate;

        if fill_rate_diff > self.thresholds.max_error_rate_increase {
            return CanaryDecision::Rollback(format!(
                "Fill rate degraded: stable {:.1}% vs canary {:.1}%",
                stable_fill_rate, canary_fill_rate
            ));
        }

        // Check latency
        let stable_latency = self.metrics.stable_avg_latency_ms();
        let canary_latency = self.metrics.canary_avg_latency_ms();

        if stable_latency > 0.0 {
            let latency_increase = ((canary_latency - stable_latency) / stable_latency) * 100.0;
            if latency_increase > self.thresholds.max_latency_increase_pct {
                return CanaryDecision::Rollback(format!(
                    "Latency increased by {:.1}%: stable {:.2}ms vs canary {:.2}ms",
                    latency_increase, stable_latency, canary_latency
                ));
            }
        }

        // Check slippage
        let stable_slippage = self.metrics.stable_avg_slippage_bp();
        let canary_slippage = self.metrics.canary_avg_slippage_bp();
        let slippage_diff = canary_slippage - stable_slippage;

        if slippage_diff > self.thresholds.max_slippage_increase_bp {
            return CanaryDecision::Rollback(format!(
                "Slippage increased: stable {:.2}bp vs canary {:.2}bp",
                stable_slippage, canary_slippage
            ));
        }

        // Check P&L (per order)
        if stable_requests > 0 && canary_requests > 0 {
            let stable_pnl_per_order = self.metrics.stable_pnl() / stable_requests as f64;
            let canary_pnl_per_order = self.metrics.canary_pnl() / canary_requests as f64;

            if stable_pnl_per_order > 0.0 {
                let pnl_ratio = canary_pnl_per_order / stable_pnl_per_order;
                if pnl_ratio < self.thresholds.min_pnl_ratio {
                    return CanaryDecision::Rollback(format!(
                        "P&L degraded: stable ${:.2}/order vs canary ${:.2}/order (ratio: {:.2})",
                        stable_pnl_per_order, canary_pnl_per_order, pnl_ratio
                    ));
                }
            }
        }

        // All checks passed - determine next step
        let next_step = self.thresholds.traffic_steps.iter()
            .find(|&&step| step > self.current_percentage);

        match next_step {
            Some(&100) => CanaryDecision::Promote,
            Some(&step) => CanaryDecision::IncreaseTraffic(step),
            None => CanaryDecision::Promote,
        }
    }

    /// Halt canary analysis (manual intervention)
    pub fn halt(&self) {
        self.is_halted.store(true, Ordering::Relaxed);
        println!("Canary analysis HALTED - manual intervention required");
    }

    /// Resume canary analysis
    pub fn resume(&self) {
        self.is_halted.store(false, Ordering::Relaxed);
        println!("Canary analysis RESUMED");
    }

    /// Generate detailed report
    pub fn report(&self) -> String {
        let mut report = String::new();

        report.push_str("\n╔══════════════════════════════════════════════════╗\n");
        report.push_str("║       TRADING CANARY DEPLOYMENT REPORT           ║\n");
        report.push_str("╠══════════════════════════════════════════════════╣\n");

        let stable_orders = self.metrics.stable_orders.load(Ordering::Relaxed);
        let canary_orders = self.metrics.canary_orders.load(Ordering::Relaxed);

        report.push_str(&format!("║ Traffic Split: Stable {}% / Canary {}%          \n",
            100 - self.current_percentage, self.current_percentage));
        report.push_str(&format!("║ Orders: Stable {} / Canary {}                   \n",
            stable_orders, canary_orders));

        report.push_str("╠══════════════════════════════════════════════════╣\n");
        report.push_str("║ METRIC          │ STABLE      │ CANARY     │ DIFF \n");
        report.push_str("╠══════════════════════════════════════════════════╣\n");

        // Fill Rate
        let stable_fill = self.metrics.stable_fill_rate();
        let canary_fill = self.metrics.canary_fill_rate();
        report.push_str(&format!("║ Fill Rate       │ {:>6.1}%     │ {:>6.1}%    │ {:>+.1}%\n",
            stable_fill, canary_fill, canary_fill - stable_fill));

        // Latency
        let stable_lat = self.metrics.stable_avg_latency_ms();
        let canary_lat = self.metrics.canary_avg_latency_ms();
        report.push_str(&format!("║ Latency (ms)    │ {:>7.2}     │ {:>7.2}    │ {:>+.2}\n",
            stable_lat, canary_lat, canary_lat - stable_lat));

        // Slippage
        let stable_slip = self.metrics.stable_avg_slippage_bp();
        let canary_slip = self.metrics.canary_avg_slippage_bp();
        report.push_str(&format!("║ Slippage (bp)   │ {:>7.2}     │ {:>7.2}    │ {:>+.2}\n",
            stable_slip, canary_slip, canary_slip - stable_slip));

        // P&L
        let stable_pnl = self.metrics.stable_pnl();
        let canary_pnl = self.metrics.canary_pnl();
        report.push_str(&format!("║ Total P&L       │ ${:>9.2}  │ ${:>9.2} │ ${:>+.2}\n",
            stable_pnl, canary_pnl, canary_pnl - stable_pnl));

        report.push_str("╚══════════════════════════════════════════════════╝\n");

        // Decision
        let decision = self.analyze();
        report.push_str(&format!("\nDecision: {:?}\n", decision));

        report
    }
}

fn main() {
    println!("=== Automated Canary Analysis for Trading ===\n");

    let thresholds = CanaryThresholds {
        min_requests: 50,
        ..Default::default()
    };

    let mut analyzer = TradingCanaryAnalyzer::new(thresholds);
    analyzer.set_percentage(10);

    let metrics = analyzer.metrics();

    // Simulate trading activity
    println!("Simulating trading with 10% canary traffic...\n");

    for i in 0..500 {
        let is_canary = (i % 10) == 0;  // ~10% to canary

        // Simulate order execution
        let filled = if is_canary {
            i % 12 != 0  // 91.7% fill rate for canary
        } else {
            i % 11 != 0  // 90.9% fill rate for stable
        };

        // Simulate P&L (canary slightly better)
        let pnl = if is_canary {
            if filled { 150 } else { -50 }  // Better P&L
        } else {
            if filled { 120 } else { -60 }
        };

        // Simulate latency (canary faster)
        let latency_us = if is_canary { 45_000 } else { 52_000 };

        // Simulate slippage
        let slippage_bp = if is_canary { 0.8 } else { 1.2 };

        if is_canary {
            metrics.record_canary_order(filled, pnl, latency_us, slippage_bp);
        } else {
            metrics.record_stable_order(filled, pnl, latency_us, slippage_bp);
        }
    }

    // Print report and get decision
    println!("{}", analyzer.report());

    // Simulate increasing traffic
    match analyzer.analyze() {
        CanaryDecision::IncreaseTraffic(pct) => {
            println!("\n--- Increasing canary traffic to {}% ---", pct);
            analyzer.set_percentage(pct);

            // More trading...
            for i in 500..1000 {
                let is_canary = (i % 4) == 0;  // ~25% to canary
                let filled = if is_canary { i % 12 != 0 } else { i % 11 != 0 };
                let pnl = if is_canary {
                    if filled { 150 } else { -50 }
                } else {
                    if filled { 120 } else { -60 }
                };
                let latency_us = if is_canary { 45_000 } else { 52_000 };
                let slippage_bp = if is_canary { 0.8 } else { 1.2 };

                if is_canary {
                    metrics.record_canary_order(filled, pnl, latency_us, slippage_bp);
                } else {
                    metrics.record_stable_order(filled, pnl, latency_us, slippage_bp);
                }
            }

            println!("{}", analyzer.report());
        }
        decision => {
            println!("Unexpected decision: {:?}", decision);
        }
    }
}
```

## Gradual Rollout Strategy

```rust
use std::time::{Duration, Instant};

/// Rollout phase configuration
#[derive(Debug, Clone)]
pub struct RolloutPhase {
    pub name: String,
    pub traffic_percentage: u8,
    pub min_duration: Duration,
    pub min_requests: u64,
    pub success_criteria: SuccessCriteria,
}

/// Success criteria for advancing to next phase
#[derive(Debug, Clone)]
pub struct SuccessCriteria {
    pub max_error_rate: f64,
    pub max_latency_p99_ms: f64,
    pub min_throughput: f64,
}

impl Default for SuccessCriteria {
    fn default() -> Self {
        SuccessCriteria {
            max_error_rate: 1.0,      // 1% max errors
            max_latency_p99_ms: 100.0, // 100ms p99 latency
            min_throughput: 100.0,     // 100 requests/second minimum
        }
    }
}

/// Gradual rollout manager for trading systems
pub struct GradualRollout {
    phases: Vec<RolloutPhase>,
    current_phase: usize,
    phase_started: Option<Instant>,
    phase_requests: u64,
    phase_errors: u64,
    latencies: Vec<f64>,
}

impl GradualRollout {
    /// Create a standard trading system rollout
    pub fn trading_standard() -> Self {
        let criteria = SuccessCriteria::default();

        GradualRollout {
            phases: vec![
                RolloutPhase {
                    name: "Canary".to_string(),
                    traffic_percentage: 1,
                    min_duration: Duration::from_secs(300),  // 5 minutes
                    min_requests: 100,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Early Adopter".to_string(),
                    traffic_percentage: 5,
                    min_duration: Duration::from_secs(600),  // 10 minutes
                    min_requests: 500,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Limited".to_string(),
                    traffic_percentage: 10,
                    min_duration: Duration::from_secs(900),  // 15 minutes
                    min_requests: 1000,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Expanding".to_string(),
                    traffic_percentage: 25,
                    min_duration: Duration::from_secs(1200), // 20 minutes
                    min_requests: 2500,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Majority".to_string(),
                    traffic_percentage: 50,
                    min_duration: Duration::from_secs(1800), // 30 minutes
                    min_requests: 5000,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Final".to_string(),
                    traffic_percentage: 100,
                    min_duration: Duration::from_secs(0),
                    min_requests: 0,
                    success_criteria: criteria,
                },
            ],
            current_phase: 0,
            phase_started: None,
            phase_requests: 0,
            phase_errors: 0,
            latencies: Vec::new(),
        }
    }

    /// Create an aggressive rollout for lower-risk changes
    pub fn aggressive() -> Self {
        let criteria = SuccessCriteria {
            max_error_rate: 2.0,
            max_latency_p99_ms: 200.0,
            min_throughput: 50.0,
        };

        GradualRollout {
            phases: vec![
                RolloutPhase {
                    name: "Quick Test".to_string(),
                    traffic_percentage: 5,
                    min_duration: Duration::from_secs(60),
                    min_requests: 50,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Ramp Up".to_string(),
                    traffic_percentage: 25,
                    min_duration: Duration::from_secs(120),
                    min_requests: 200,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Full".to_string(),
                    traffic_percentage: 100,
                    min_duration: Duration::from_secs(0),
                    min_requests: 0,
                    success_criteria: criteria,
                },
            ],
            current_phase: 0,
            phase_started: None,
            phase_requests: 0,
            phase_errors: 0,
            latencies: Vec::new(),
        }
    }

    /// Start the rollout
    pub fn start(&mut self) {
        self.phase_started = Some(Instant::now());
        self.phase_requests = 0;
        self.phase_errors = 0;
        self.latencies.clear();

        let phase = &self.phases[self.current_phase];
        println!("Starting rollout phase '{}' at {}% traffic",
            phase.name, phase.traffic_percentage);
    }

    /// Record a request result
    pub fn record_request(&mut self, success: bool, latency_ms: f64) {
        self.phase_requests += 1;
        if !success {
            self.phase_errors += 1;
        }
        self.latencies.push(latency_ms);
    }

    /// Get current traffic percentage
    pub fn current_percentage(&self) -> u8 {
        self.phases[self.current_phase].traffic_percentage
    }

    /// Check if ready to advance to next phase
    pub fn check_advance(&mut self) -> RolloutAction {
        let phase = &self.phases[self.current_phase];

        // Check minimum duration
        if let Some(started) = self.phase_started {
            if started.elapsed() < phase.min_duration {
                return RolloutAction::Wait(format!(
                    "Waiting for minimum duration: {:.0}s remaining",
                    (phase.min_duration - started.elapsed()).as_secs_f64()
                ));
            }
        } else {
            return RolloutAction::NotStarted;
        }

        // Check minimum requests
        if self.phase_requests < phase.min_requests {
            return RolloutAction::Wait(format!(
                "Waiting for minimum requests: {} / {}",
                self.phase_requests, phase.min_requests
            ));
        }

        // Check success criteria
        let error_rate = (self.phase_errors as f64 / self.phase_requests as f64) * 100.0;
        if error_rate > phase.success_criteria.max_error_rate {
            return RolloutAction::Rollback(format!(
                "Error rate {:.2}% exceeds threshold {:.2}%",
                error_rate, phase.success_criteria.max_error_rate
            ));
        }

        // Check p99 latency
        if !self.latencies.is_empty() {
            let mut sorted = self.latencies.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p99_idx = (sorted.len() as f64 * 0.99) as usize;
            let p99 = sorted.get(p99_idx.min(sorted.len() - 1)).copied().unwrap_or(0.0);

            if p99 > phase.success_criteria.max_latency_p99_ms {
                return RolloutAction::Rollback(format!(
                    "P99 latency {:.2}ms exceeds threshold {:.2}ms",
                    p99, phase.success_criteria.max_latency_p99_ms
                ));
            }
        }

        // All criteria met - advance to next phase
        if self.current_phase < self.phases.len() - 1 {
            self.current_phase += 1;
            self.phase_started = Some(Instant::now());
            self.phase_requests = 0;
            self.phase_errors = 0;
            self.latencies.clear();

            let next_phase = &self.phases[self.current_phase];
            RolloutAction::Advance(next_phase.traffic_percentage)
        } else {
            RolloutAction::Complete
        }
    }

    /// Get current phase name
    pub fn current_phase_name(&self) -> &str {
        &self.phases[self.current_phase].name
    }

    /// Print rollout status
    pub fn print_status(&self) {
        let phase = &self.phases[self.current_phase];

        println!("\n=== Rollout Status ===");
        println!("Phase: {} ({}/{})", phase.name, self.current_phase + 1, self.phases.len());
        println!("Traffic: {}%", phase.traffic_percentage);
        println!("Requests: {} / {} required", self.phase_requests, phase.min_requests);

        if self.phase_requests > 0 {
            let error_rate = (self.phase_errors as f64 / self.phase_requests as f64) * 100.0;
            println!("Error rate: {:.2}% (max: {:.2}%)", error_rate, phase.success_criteria.max_error_rate);
        }

        if let Some(started) = self.phase_started {
            let elapsed = started.elapsed();
            let remaining = phase.min_duration.saturating_sub(elapsed);
            println!("Duration: {:.0}s elapsed, {:.0}s remaining",
                elapsed.as_secs_f64(), remaining.as_secs_f64());
        }
    }
}

#[derive(Debug)]
pub enum RolloutAction {
    NotStarted,
    Wait(String),
    Advance(u8),
    Rollback(String),
    Complete,
}

fn main() {
    println!("=== Gradual Rollout Strategy ===\n");

    let mut rollout = GradualRollout::trading_standard();
    rollout.start();

    // Simulate going through phases
    for phase_num in 0..4 {
        println!("\n--- Simulating Phase {} ---", phase_num + 1);
        rollout.print_status();

        // Simulate requests for this phase
        let requests_needed = 150 + (phase_num * 200) as u64;
        for i in 0..requests_needed {
            let success = i % 100 != 0;  // 99% success rate
            let latency = 30.0 + (i % 40) as f64;  // 30-70ms latency
            rollout.record_request(success, latency);
        }

        // Check if we can advance
        match rollout.check_advance() {
            RolloutAction::Advance(pct) => {
                println!("\nAdvancing to {}% traffic", pct);
            }
            RolloutAction::Wait(reason) => {
                println!("\nWaiting: {}", reason);
                // In real implementation, would wait and retry
                // For demo, force advance by resetting timing
            }
            RolloutAction::Rollback(reason) => {
                println!("\nROLLBACK: {}", reason);
                break;
            }
            RolloutAction::Complete => {
                println!("\nRollout COMPLETE!");
                break;
            }
            RolloutAction::NotStarted => {
                println!("\nRollout not started");
            }
        }
    }

    rollout.print_status();
}
```

## Integration with Trading Bot Architecture

```rust
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::collections::HashMap;

/// Version identifier for trading strategies
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StrategyVersion {
    pub name: String,
    pub version: String,
    pub hash: String,
}

impl StrategyVersion {
    pub fn new(name: &str, version: &str) -> Self {
        let hash = format!("{:x}", version.as_bytes().iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64)));
        StrategyVersion {
            name: name.to_string(),
            version: version.to_string(),
            hash,
        }
    }

    pub fn full_id(&self) -> String {
        format!("{}-{}-{}", self.name, self.version, &self.hash[..8])
    }
}

/// Trading strategy interface
pub trait TradingStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &StrategyVersion;
    fn generate_signal(&self, symbol: &str, price: f64, volume: f64) -> Signal;
    fn calculate_position_size(&self, signal: &Signal, balance: f64) -> f64;
}

#[derive(Debug, Clone)]
pub enum Signal {
    Buy { strength: f64, target: f64, stop_loss: f64 },
    Sell { strength: f64, target: f64, stop_loss: f64 },
    Hold,
}

/// Stable version of momentum strategy (v1.0)
pub struct MomentumStrategyV1 {
    version: StrategyVersion,
}

impl MomentumStrategyV1 {
    pub fn new() -> Self {
        MomentumStrategyV1 {
            version: StrategyVersion::new("Momentum", "1.0.0"),
        }
    }
}

impl TradingStrategy for MomentumStrategyV1 {
    fn name(&self) -> &str { "Momentum" }
    fn version(&self) -> &StrategyVersion { &self.version }

    fn generate_signal(&self, _symbol: &str, price: f64, volume: f64) -> Signal {
        // Simple momentum logic
        let momentum = volume / 1000.0;
        if momentum > 1.5 {
            Signal::Buy {
                strength: (momentum - 1.0).min(1.0),
                target: price * 1.02,
                stop_loss: price * 0.98,
            }
        } else if momentum < 0.5 {
            Signal::Sell {
                strength: (1.0 - momentum).min(1.0),
                target: price * 0.98,
                stop_loss: price * 1.02,
            }
        } else {
            Signal::Hold
        }
    }

    fn calculate_position_size(&self, signal: &Signal, balance: f64) -> f64 {
        match signal {
            Signal::Buy { strength, .. } | Signal::Sell { strength, .. } => {
                balance * 0.01 * strength  // 1% of balance * signal strength
            }
            Signal::Hold => 0.0,
        }
    }
}

/// Canary version with improved logic (v2.0)
pub struct MomentumStrategyV2 {
    version: StrategyVersion,
}

impl MomentumStrategyV2 {
    pub fn new() -> Self {
        MomentumStrategyV2 {
            version: StrategyVersion::new("Momentum", "2.0.0"),
        }
    }
}

impl TradingStrategy for MomentumStrategyV2 {
    fn name(&self) -> &str { "Momentum" }
    fn version(&self) -> &StrategyVersion { &self.version }

    fn generate_signal(&self, _symbol: &str, price: f64, volume: f64) -> Signal {
        // Improved momentum logic with adaptive thresholds
        let momentum = volume / 1000.0;
        let volatility_factor = (price / 50000.0).min(2.0);  // Adjust for price level

        let buy_threshold = 1.3 * volatility_factor;
        let sell_threshold = 0.7 / volatility_factor;

        if momentum > buy_threshold {
            Signal::Buy {
                strength: ((momentum - buy_threshold) / buy_threshold).min(1.0),
                target: price * 1.025,  // Slightly higher target
                stop_loss: price * 0.985,  // Tighter stop loss
            }
        } else if momentum < sell_threshold {
            Signal::Sell {
                strength: ((sell_threshold - momentum) / sell_threshold).min(1.0),
                target: price * 0.975,
                stop_loss: price * 1.015,
            }
        } else {
            Signal::Hold
        }
    }

    fn calculate_position_size(&self, signal: &Signal, balance: f64) -> f64 {
        match signal {
            Signal::Buy { strength, .. } | Signal::Sell { strength, .. } => {
                // More aggressive sizing in v2
                balance * 0.015 * strength  // 1.5% of balance * signal strength
            }
            Signal::Hold => 0.0,
        }
    }
}

/// Strategy router with canary support
pub struct StrategyRouter {
    stable: Arc<dyn TradingStrategy>,
    canary: Option<Arc<dyn TradingStrategy>>,
    canary_percentage: AtomicU8,
    is_canary_enabled: AtomicBool,
}

impl StrategyRouter {
    pub fn new(stable: Arc<dyn TradingStrategy>) -> Self {
        println!("Initializing router with stable strategy: {}",
            stable.version().full_id());

        StrategyRouter {
            stable,
            canary: None,
            canary_percentage: AtomicU8::new(0),
            is_canary_enabled: AtomicBool::new(false),
        }
    }

    pub fn deploy_canary(&mut self, canary: Arc<dyn TradingStrategy>, percentage: u8) {
        println!("Deploying canary strategy: {} at {}%",
            canary.version().full_id(), percentage);

        self.canary = Some(canary);
        self.canary_percentage.store(percentage.min(100), Ordering::SeqCst);
        self.is_canary_enabled.store(true, Ordering::SeqCst);
    }

    pub fn route(&self, account_id: u64) -> Arc<dyn TradingStrategy> {
        if !self.is_canary_enabled.load(Ordering::Relaxed) {
            return Arc::clone(&self.stable);
        }

        if let Some(ref canary) = self.canary {
            let percentage = self.canary_percentage.load(Ordering::Relaxed);
            let bucket = (account_id % 100) as u8;

            if bucket < percentage {
                return Arc::clone(canary);
            }
        }

        Arc::clone(&self.stable)
    }

    pub fn promote_canary(&mut self) -> Result<(), &'static str> {
        if let Some(canary) = self.canary.take() {
            println!("Promoting canary {} to stable", canary.version().full_id());
            self.stable = canary;
            self.canary_percentage.store(0, Ordering::SeqCst);
            self.is_canary_enabled.store(false, Ordering::SeqCst);
            Ok(())
        } else {
            Err("No canary to promote")
        }
    }

    pub fn rollback_canary(&mut self) -> Option<String> {
        if let Some(canary) = self.canary.take() {
            let version_id = canary.version().full_id();
            println!("Rolling back canary: {}", version_id);
            self.canary_percentage.store(0, Ordering::SeqCst);
            self.is_canary_enabled.store(false, Ordering::SeqCst);
            Some(version_id)
        } else {
            None
        }
    }

    pub fn increase_canary_traffic(&self, new_percentage: u8) {
        let old = self.canary_percentage.load(Ordering::Relaxed);
        self.canary_percentage.store(new_percentage.min(100), Ordering::SeqCst);
        println!("Canary traffic: {}% -> {}%", old, new_percentage.min(100));
    }
}

/// Trading bot with canary deployment support
pub struct TradingBot {
    router: StrategyRouter,
    accounts: HashMap<u64, AccountState>,
}

#[derive(Debug)]
pub struct AccountState {
    pub balance: f64,
    pub positions: HashMap<String, f64>,
}

impl TradingBot {
    pub fn new(stable_strategy: Arc<dyn TradingStrategy>) -> Self {
        let router = StrategyRouter::new(stable_strategy);

        // Create sample accounts
        let mut accounts = HashMap::new();
        for i in 0..10 {
            accounts.insert(i, AccountState {
                balance: 100_000.0,
                positions: HashMap::new(),
            });
        }

        TradingBot { router, accounts }
    }

    pub fn deploy_canary(&mut self, strategy: Arc<dyn TradingStrategy>, percentage: u8) {
        self.router.deploy_canary(strategy, percentage);
    }

    pub fn process_market_data(&self, symbol: &str, price: f64, volume: f64) {
        println!("\n--- Processing {} @ ${:.2} (vol: {:.0}) ---", symbol, price, volume);

        for (&account_id, account) in &self.accounts {
            let strategy = self.router.route(account_id);
            let signal = strategy.generate_signal(symbol, price, volume);
            let position_size = strategy.calculate_position_size(&signal, account.balance);

            match signal {
                Signal::Buy { strength, target, stop_loss } if position_size > 0.0 => {
                    println!("Account {}: {} [{}] BUY ${:.2} (str: {:.2}, tgt: {:.2}, sl: {:.2})",
                        account_id, strategy.version().version, symbol,
                        position_size, strength, target, stop_loss);
                }
                Signal::Sell { strength, target, stop_loss } if position_size > 0.0 => {
                    println!("Account {}: {} [{}] SELL ${:.2} (str: {:.2}, tgt: {:.2}, sl: {:.2})",
                        account_id, strategy.version().version, symbol,
                        position_size, strength, target, stop_loss);
                }
                _ => {
                    // No action or hold
                }
            }
        }
    }

    pub fn increase_canary(&self, percentage: u8) {
        self.router.increase_canary_traffic(percentage);
    }

    pub fn promote(&mut self) -> Result<(), &'static str> {
        self.router.promote_canary()
    }

    pub fn rollback(&mut self) -> Option<String> {
        self.router.rollback_canary()
    }
}

fn main() {
    println!("=== Trading Bot Canary Deployment ===\n");

    // Create trading bot with stable strategy
    let stable = Arc::new(MomentumStrategyV1::new());
    let mut bot = TradingBot::new(stable);

    // Process some market data with stable only
    println!("=== Running with stable strategy only ===");
    bot.process_market_data("BTCUSDT", 50000.0, 2000.0);

    // Deploy canary
    println!("\n=== Deploying canary strategy at 20% ===");
    let canary = Arc::new(MomentumStrategyV2::new());
    bot.deploy_canary(canary, 20);

    // Process more market data
    bot.process_market_data("BTCUSDT", 50100.0, 2200.0);

    // Increase canary traffic
    println!("\n=== Increasing canary to 50% ===");
    bot.increase_canary(50);

    bot.process_market_data("BTCUSDT", 50200.0, 1800.0);

    // Promote canary
    println!("\n=== Promoting canary to stable ===");
    bot.promote().unwrap();

    bot.process_market_data("BTCUSDT", 50300.0, 2100.0);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Canary Deployment** | Gradually releasing new version to a subset of traffic |
| **Traffic Splitting** | Routing requests between stable and canary versions |
| **Metrics Comparison** | Comparing error rates, latencies, P&L between versions |
| **Automatic Analysis** | Making promotion/rollback decisions based on metrics |
| **Gradual Rollout** | Step-by-step traffic increase with validation at each step |
| **Quick Rollback** | Instant reversion to stable version on problems |
| **Strategy Versioning** | Identifying and tracking different strategy versions |

## Practical Exercises

1. **Order Routing Canary**: Implement a system that:
   - Routes a percentage of orders through a new order routing algorithm
   - Compares execution quality (slippage, fill rate, latency)
   - Automatically rolls back if execution quality degrades
   - Logs all routing decisions for audit

2. **Risk Engine Canary**: Create a canary system for risk management:
   - Tests new risk calculations on subset of accounts
   - Compares risk limits with production engine
   - Alerts on significant differences
   - Validates new risk models before full rollout

3. **Multi-Exchange Canary**: Build a deployment system that:
   - Tests new exchange connectors with small order sizes
   - Monitors connection stability and latency
   - Compares order execution across exchanges
   - Supports per-exchange rollback

4. **Strategy A/B with Canary**: Implement a hybrid system:
   - Runs A/B tests within canary traffic
   - Tracks performance metrics per variant
   - Promotes winning variant to stable
   - Provides statistical confidence for decisions

## Homework

1. **Complete Canary Pipeline**: Build an end-to-end system that:
   - Integrates with CI/CD for automatic canary deployments
   - Monitors trading-specific metrics (P&L, Sharpe ratio, drawdown)
   - Supports scheduled traffic increases
   - Implements automatic rollback on anomalies
   - Sends alerts to Slack/PagerDuty on issues
   - Maintains deployment history and audit logs

2. **Machine Learning Canary**: Create a system for deploying ML models:
   - Compares prediction accuracy between versions
   - Monitors feature drift and model degradation
   - Supports shadow mode (predictions without execution)
   - Validates model latency in production
   - Implements gradual traffic shifting based on performance

3. **Global Canary Orchestration**: Design a multi-region system:
   - Deploys canary per geographic region
   - Monitors region-specific metrics
   - Supports region-by-region rollout
   - Handles timezone-aware trading schedules
   - Coordinates rollback across all regions

4. **Chaos Engineering Integration**: Combine canary with chaos testing:
   - Injects failures during canary phase
   - Tests error handling and recovery
   - Validates circuit breakers and fallbacks
   - Measures system resilience under stress
   - Generates reliability reports

## Navigation

[← Previous day](../354-production-logging/en.md) | [Next day →](../361-*/en.md)
