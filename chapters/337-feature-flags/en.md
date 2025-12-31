# Day 337: Feature Flags: Enabling Features

## Trading Analogy

Imagine you're developing a trading platform that serves thousands of traders. You want to introduce a new high-frequency trading algorithm, but deploying it to everyone at once is risky. What if it has a bug that causes massive losses?

**Without Feature Flags:**
You deploy the new algorithm to all users. If something goes wrong, you need to rollback the entire deployment, affecting everyone and potentially losing money during the downtime.

**With Feature Flags:**
You deploy the code but keep the new algorithm hidden behind a switch. First, you enable it only for your internal testing team. Then, you gradually roll it out to 1%, 10%, 50% of users, monitoring performance at each step. If issues arise, you simply flip the switch — no deployment needed.

| Scenario | Without Flags | With Flags |
|----------|---------------|------------|
| **New strategy testing** | Deploy to all or none | Enable for select accounts |
| **Bug discovered** | Full rollback | Disable instantly |
| **A/B testing** | Complex infrastructure | Built-in capability |
| **Regional rollout** | Multiple deployments | Configuration change |
| **Emergency disable** | Redeploy old version | Flip a switch |

## What Are Feature Flags?

Feature flags (also called feature toggles) are a technique for modifying system behavior without changing code. They allow you to:

1. **Gradually roll out features** — enable for a percentage of users
2. **A/B test** — compare different implementations
3. **Kill switch** — instantly disable problematic features
4. **Environment-specific behavior** — different features for dev/staging/prod
5. **User-specific features** — premium features, beta testers

## Basic Feature Flag Implementation

```rust
use std::collections::HashMap;
use std::sync::RwLock;

/// Simple feature flag storage
pub struct FeatureFlags {
    flags: RwLock<HashMap<String, bool>>,
}

impl FeatureFlags {
    pub fn new() -> Self {
        FeatureFlags {
            flags: RwLock::new(HashMap::new()),
        }
    }

    /// Set a feature flag value
    pub fn set(&self, feature: &str, enabled: bool) {
        let mut flags = self.flags.write().unwrap();
        flags.insert(feature.to_string(), enabled);
    }

    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        let flags = self.flags.read().unwrap();
        *flags.get(feature).unwrap_or(&false)
    }

    /// Load flags from configuration
    pub fn load_from_config(config: &HashMap<String, bool>) -> Self {
        let flags = FeatureFlags::new();
        for (key, value) in config {
            flags.set(key, *value);
        }
        flags
    }
}

/// Trading system with feature flags
struct TradingSystem {
    flags: FeatureFlags,
}

impl TradingSystem {
    fn new(flags: FeatureFlags) -> Self {
        TradingSystem { flags }
    }

    fn execute_trade(&self, symbol: &str, quantity: f64, price: f64) {
        println!("Executing trade: {} {} @ ${:.2}", quantity, symbol, price);

        // New risk management feature behind a flag
        if self.flags.is_enabled("advanced_risk_check") {
            self.perform_advanced_risk_check(symbol, quantity, price);
        }

        // New smart order routing behind a flag
        if self.flags.is_enabled("smart_order_routing") {
            self.route_to_best_exchange(symbol, quantity, price);
        } else {
            self.route_to_default_exchange(symbol, quantity, price);
        }

        // Experimental ML predictions
        if self.flags.is_enabled("ml_price_prediction") {
            let prediction = self.get_ml_prediction(symbol);
            println!("  ML prediction: ${:.2}", prediction);
        }
    }

    fn perform_advanced_risk_check(&self, symbol: &str, quantity: f64, price: f64) {
        let position_value = quantity * price;
        println!("  Advanced risk check: position value ${:.2}", position_value);

        if position_value > 100000.0 {
            println!("  WARNING: Large position detected!");
        }
    }

    fn route_to_best_exchange(&self, symbol: &str, _quantity: f64, _price: f64) {
        println!("  Smart routing: finding best exchange for {}", symbol);
    }

    fn route_to_default_exchange(&self, symbol: &str, _quantity: f64, _price: f64) {
        println!("  Default routing: sending to primary exchange for {}", symbol);
    }

    fn get_ml_prediction(&self, symbol: &str) -> f64 {
        // Simulated ML prediction
        match symbol {
            "BTCUSDT" => 51234.56,
            "ETHUSDT" => 3456.78,
            _ => 100.0,
        }
    }
}

fn main() {
    println!("=== Basic Feature Flags Demo ===\n");

    // Configure feature flags
    let flags = FeatureFlags::new();
    flags.set("advanced_risk_check", true);
    flags.set("smart_order_routing", false);
    flags.set("ml_price_prediction", true);

    let system = TradingSystem::new(flags);

    // Execute trades with different feature combinations
    system.execute_trade("BTCUSDT", 0.5, 50000.0);
    println!();

    // Dynamically enable smart routing
    println!("--- Enabling smart order routing ---\n");
    system.flags.set("smart_order_routing", true);

    system.execute_trade("ETHUSDT", 10.0, 3000.0);
}
```

## Compile-Time Feature Flags with Cargo

Rust's Cargo supports compile-time feature flags through `Cargo.toml`:

```toml
# Cargo.toml
[package]
name = "trading_system"
version = "0.1.0"
edition = "2021"

[features]
default = ["basic_indicators"]

# Trading features
basic_indicators = []
advanced_indicators = ["basic_indicators"]
ml_predictions = ["dep:ndarray"]
real_time_data = ["dep:tokio", "dep:tokio-tungstenite"]
paper_trading = []
live_trading = []
backtesting = []

# Performance features
simd_optimization = []
parallel_processing = ["dep:rayon"]

# Debugging features
detailed_logging = ["dep:tracing"]
performance_metrics = []

[dependencies]
ndarray = { version = "0.15", optional = true }
tokio = { version = "1.0", features = ["full"], optional = true }
tokio-tungstenite = { version = "0.20", optional = true }
rayon = { version = "1.8", optional = true }
tracing = { version = "0.1", optional = true }
```

Using compile-time flags in code:

```rust
/// Price indicator calculations with feature flags
pub struct Indicators;

impl Indicators {
    /// Simple Moving Average — always available
    pub fn sma(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        let sum: f64 = prices[prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }

    /// Exponential Moving Average — requires advanced_indicators feature
    #[cfg(feature = "advanced_indicators")]
    pub fn ema(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = Self::sma(&prices[..period], period)?;

        for price in &prices[period..] {
            ema = (price - ema) * multiplier + ema;
        }

        Some(ema)
    }

    /// Bollinger Bands — requires advanced_indicators feature
    #[cfg(feature = "advanced_indicators")]
    pub fn bollinger_bands(prices: &[f64], period: usize, std_dev: f64) -> Option<(f64, f64, f64)> {
        if prices.len() < period {
            return None;
        }

        let slice = &prices[prices.len() - period..];
        let sma = slice.iter().sum::<f64>() / period as f64;

        let variance = slice.iter()
            .map(|p| (p - sma).powi(2))
            .sum::<f64>() / period as f64;
        let std = variance.sqrt();

        Some((sma - std_dev * std, sma, sma + std_dev * std))
    }

    /// ML-based prediction — requires ml_predictions feature
    #[cfg(feature = "ml_predictions")]
    pub fn predict_next_price(prices: &[f64]) -> f64 {
        // Simplified linear regression prediction
        let n = prices.len() as f64;
        let sum_x: f64 = (0..prices.len()).map(|i| i as f64).sum();
        let sum_y: f64 = prices.iter().sum();
        let sum_xy: f64 = prices.iter().enumerate()
            .map(|(i, p)| i as f64 * p)
            .sum();
        let sum_x2: f64 = (0..prices.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;

        slope * n + intercept
    }
}

/// Strategy execution with feature-gated behavior
pub struct Strategy {
    name: String,
}

impl Strategy {
    pub fn new(name: &str) -> Self {
        Strategy { name: name.to_string() }
    }

    pub fn analyze(&self, prices: &[f64]) {
        println!("Strategy '{}' analyzing {} prices", self.name, prices.len());

        // Basic SMA always available
        if let Some(sma) = Indicators::sma(prices, 20) {
            println!("  SMA(20): {:.2}", sma);
        }

        // Advanced indicators only with feature
        #[cfg(feature = "advanced_indicators")]
        {
            if let Some(ema) = Indicators::ema(prices, 20) {
                println!("  EMA(20): {:.2}", ema);
            }

            if let Some((lower, middle, upper)) = Indicators::bollinger_bands(prices, 20, 2.0) {
                println!("  Bollinger Bands: {:.2} | {:.2} | {:.2}", lower, middle, upper);
            }
        }

        // ML predictions only with feature
        #[cfg(feature = "ml_predictions")]
        {
            let prediction = Indicators::predict_next_price(prices);
            println!("  ML Prediction: {:.2}", prediction);
        }

        // Show which features are enabled
        #[cfg(feature = "detailed_logging")]
        {
            println!("  [DEBUG] Analysis completed with detailed logging");
        }
    }

    #[cfg(feature = "paper_trading")]
    pub fn execute_paper_trade(&self, symbol: &str, side: &str, quantity: f64, price: f64) {
        println!(
            "[PAPER] {} {} {} @ ${:.2}",
            side, quantity, symbol, price
        );
    }

    #[cfg(feature = "live_trading")]
    pub fn execute_live_trade(&self, symbol: &str, side: &str, quantity: f64, price: f64) {
        println!(
            "[LIVE] Executing {} {} {} @ ${:.2}",
            side, quantity, symbol, price
        );
        // In real implementation: connect to exchange API
    }
}

fn main() {
    println!("=== Compile-Time Feature Flags ===\n");

    // Show which features are compiled in
    println!("Compiled features:");

    #[cfg(feature = "basic_indicators")]
    println!("  - basic_indicators");

    #[cfg(feature = "advanced_indicators")]
    println!("  - advanced_indicators");

    #[cfg(feature = "ml_predictions")]
    println!("  - ml_predictions");

    #[cfg(feature = "paper_trading")]
    println!("  - paper_trading");

    #[cfg(feature = "live_trading")]
    println!("  - live_trading");

    #[cfg(feature = "parallel_processing")]
    println!("  - parallel_processing");

    println!();

    // Generate sample price data
    let prices: Vec<f64> = (0..100)
        .map(|i| 50000.0 + (i as f64 * 0.1).sin() * 1000.0)
        .collect();

    let strategy = Strategy::new("TrendFollower");
    strategy.analyze(&prices);

    println!();

    // Conditional execution based on features
    #[cfg(feature = "paper_trading")]
    strategy.execute_paper_trade("BTCUSDT", "BUY", 0.1, 50000.0);

    #[cfg(feature = "live_trading")]
    strategy.execute_live_trade("BTCUSDT", "BUY", 0.1, 50000.0);

    #[cfg(not(any(feature = "paper_trading", feature = "live_trading")))]
    println!("No trading mode enabled. Use --features paper_trading or live_trading");
}
```

## Runtime Feature Flags with Percentage Rollout

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Feature flag with rollout percentage
#[derive(Clone, Debug)]
pub struct FeatureConfig {
    pub enabled: bool,
    pub rollout_percentage: u8,  // 0-100
    pub allowed_users: Vec<String>,
    pub blocked_users: Vec<String>,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        FeatureConfig {
            enabled: false,
            rollout_percentage: 0,
            allowed_users: vec![],
            blocked_users: vec![],
        }
    }
}

/// Advanced feature flag system with user targeting
pub struct FeatureFlagSystem {
    configs: RwLock<HashMap<String, FeatureConfig>>,
}

impl FeatureFlagSystem {
    pub fn new() -> Self {
        FeatureFlagSystem {
            configs: RwLock::new(HashMap::new()),
        }
    }

    /// Configure a feature
    pub fn configure(&self, feature: &str, config: FeatureConfig) {
        let mut configs = self.configs.write().unwrap();
        configs.insert(feature.to_string(), config);
    }

    /// Check if feature is enabled for a specific user
    pub fn is_enabled_for_user(&self, feature: &str, user_id: &str) -> bool {
        let configs = self.configs.read().unwrap();

        let config = match configs.get(feature) {
            Some(c) => c,
            None => return false,
        };

        // Feature globally disabled
        if !config.enabled {
            return false;
        }

        // Check blocked users first
        if config.blocked_users.contains(&user_id.to_string()) {
            return false;
        }

        // Check allowed users
        if config.allowed_users.contains(&user_id.to_string()) {
            return true;
        }

        // Percentage rollout based on user hash
        if config.rollout_percentage >= 100 {
            return true;
        }

        let user_bucket = self.get_user_bucket(user_id, feature);
        user_bucket < config.rollout_percentage
    }

    /// Get consistent bucket (0-99) for user/feature combination
    fn get_user_bucket(&self, user_id: &str, feature: &str) -> u8 {
        let mut hasher = DefaultHasher::new();
        format!("{}:{}", user_id, feature).hash(&mut hasher);
        (hasher.finish() % 100) as u8
    }

    /// Enable feature for percentage of users
    pub fn enable_for_percentage(&self, feature: &str, percentage: u8) {
        let mut configs = self.configs.write().unwrap();
        let config = configs.entry(feature.to_string()).or_insert(FeatureConfig::default());
        config.enabled = true;
        config.rollout_percentage = percentage.min(100);
    }

    /// Get enabled users count for a feature (for monitoring)
    pub fn get_rollout_stats(&self, feature: &str, all_users: &[&str]) -> (usize, usize) {
        let enabled_count = all_users
            .iter()
            .filter(|u| self.is_enabled_for_user(feature, u))
            .count();
        (enabled_count, all_users.len())
    }
}

/// Trading strategy with gradual rollout
struct TradingStrategy {
    name: String,
    flags: FeatureFlagSystem,
}

impl TradingStrategy {
    fn new(name: &str) -> Self {
        let flags = FeatureFlagSystem::new();

        // Configure features with rollout percentages
        flags.configure("new_entry_logic", FeatureConfig {
            enabled: true,
            rollout_percentage: 25,  // Only 25% of users
            allowed_users: vec!["beta_tester_1".to_string()],
            blocked_users: vec![],
        });

        flags.configure("experimental_exit", FeatureConfig {
            enabled: true,
            rollout_percentage: 10,  // Only 10% of users
            allowed_users: vec![],
            blocked_users: vec!["risk_averse_user".to_string()],
        });

        TradingStrategy {
            name: name.to_string(),
            flags,
        }
    }

    fn generate_signal(&self, user_id: &str, price: f64, sma: f64) -> &str {
        println!("\nGenerating signal for user: {}", user_id);

        let signal = if self.flags.is_enabled_for_user("new_entry_logic", user_id) {
            println!("  Using NEW entry logic (feature enabled)");
            // New logic: more aggressive entry
            if price < sma * 0.98 {
                "STRONG_BUY"
            } else if price < sma {
                "BUY"
            } else if price > sma * 1.02 {
                "STRONG_SELL"
            } else if price > sma {
                "SELL"
            } else {
                "HOLD"
            }
        } else {
            println!("  Using OLD entry logic (feature disabled)");
            // Old logic: conservative
            if price < sma {
                "BUY"
            } else if price > sma {
                "SELL"
            } else {
                "HOLD"
            }
        };

        // Check exit logic feature
        if self.flags.is_enabled_for_user("experimental_exit", user_id) {
            println!("  Experimental exit logic: ENABLED");
        } else {
            println!("  Experimental exit logic: DISABLED");
        }

        println!("  Signal: {}", signal);
        signal
    }
}

fn main() {
    println!("=== Feature Flags with Percentage Rollout ===\n");

    let strategy = TradingStrategy::new("MomentumStrategy");

    // Simulate different users
    let users = [
        "user_001", "user_002", "user_003", "user_004", "user_005",
        "beta_tester_1", "risk_averse_user", "user_100", "user_200",
    ];

    let price = 50000.0;
    let sma = 50500.0;

    for user in &users {
        strategy.generate_signal(user, price, sma);
    }

    // Show rollout statistics
    println!("\n=== Rollout Statistics ===");
    let user_refs: Vec<&str> = users.iter().copied().collect();

    let (enabled, total) = strategy.flags.get_rollout_stats("new_entry_logic", &user_refs);
    println!("new_entry_logic: {}/{} users ({:.0}%)", enabled, total, enabled as f64 / total as f64 * 100.0);

    let (enabled, total) = strategy.flags.get_rollout_stats("experimental_exit", &user_refs);
    println!("experimental_exit: {}/{} users ({:.0}%)", enabled, total, enabled as f64 / total as f64 * 100.0);

    // Demonstrate gradual rollout
    println!("\n=== Gradual Rollout Simulation ===");
    let many_users: Vec<String> = (0..1000).map(|i| format!("user_{:04}", i)).collect();
    let user_refs: Vec<&str> = many_users.iter().map(|s| s.as_str()).collect();

    for percentage in [10, 25, 50, 75, 100] {
        strategy.flags.enable_for_percentage("gradual_feature", percentage);
        let (enabled, total) = strategy.flags.get_rollout_stats("gradual_feature", &user_refs);
        println!(
            "At {}% rollout: {}/{} users actually enabled ({:.1}%)",
            percentage, enabled, total, enabled as f64 / total as f64 * 100.0
        );
    }
}
```

## Environment-Based Feature Flags

```rust
use std::env;
use std::collections::HashMap;

/// Environment type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    fn from_env() -> Self {
        match env::var("TRADING_ENV").as_deref() {
            Ok("production") | Ok("prod") => Environment::Production,
            Ok("staging") | Ok("stage") => Environment::Staging,
            _ => Environment::Development,
        }
    }
}

/// Environment-aware feature flags
pub struct EnvFeatureFlags {
    environment: Environment,
    overrides: HashMap<String, bool>,
}

impl EnvFeatureFlags {
    pub fn new() -> Self {
        EnvFeatureFlags {
            environment: Environment::from_env(),
            overrides: HashMap::new(),
        }
    }

    pub fn with_environment(environment: Environment) -> Self {
        EnvFeatureFlags {
            environment,
            overrides: HashMap::new(),
        }
    }

    /// Override a feature flag
    pub fn set_override(&mut self, feature: &str, enabled: bool) {
        self.overrides.insert(feature.to_string(), enabled);
    }

    /// Check if feature is enabled based on environment
    pub fn is_enabled(&self, feature: &str) -> bool {
        // Check overrides first
        if let Some(&enabled) = self.overrides.get(feature) {
            return enabled;
        }

        // Environment-based defaults
        match (feature, self.environment) {
            // Debug features only in development
            ("debug_logging", Environment::Development) => true,
            ("debug_logging", _) => false,

            ("verbose_errors", Environment::Development) => true,
            ("verbose_errors", Environment::Staging) => true,
            ("verbose_errors", Environment::Production) => false,

            // Paper trading in dev/staging, real trading in production
            ("paper_trading", Environment::Production) => false,
            ("paper_trading", _) => true,

            ("live_trading", Environment::Production) => true,
            ("live_trading", _) => false,

            // Risk limits stricter in production
            ("relaxed_risk_limits", Environment::Development) => true,
            ("relaxed_risk_limits", _) => false,

            // Performance features in production
            ("query_caching", Environment::Production) => true,
            ("query_caching", Environment::Staging) => true,
            ("query_caching", Environment::Development) => false,

            // Experimental features only in development
            ("experimental_algorithm", Environment::Development) => true,
            ("experimental_algorithm", _) => false,

            // Default: disabled
            _ => false,
        }
    }

    pub fn environment(&self) -> Environment {
        self.environment
    }
}

/// Order executor with environment-aware behavior
struct OrderExecutor {
    flags: EnvFeatureFlags,
}

impl OrderExecutor {
    fn new(flags: EnvFeatureFlags) -> Self {
        OrderExecutor { flags }
    }

    fn execute_order(&self, symbol: &str, side: &str, quantity: f64, price: f64) {
        println!("\n=== Order Execution ===");
        println!("Environment: {:?}", self.flags.environment());
        println!("Order: {} {} {} @ ${:.2}", side, quantity, symbol, price);

        // Debug logging only in development
        if self.flags.is_enabled("debug_logging") {
            println!("[DEBUG] Order details:");
            println!("  - Symbol: {}", symbol);
            println!("  - Side: {}", side);
            println!("  - Quantity: {}", quantity);
            println!("  - Price: {}", price);
            println!("  - Notional: ${:.2}", quantity * price);
        }

        // Risk checks
        let max_order_value = if self.flags.is_enabled("relaxed_risk_limits") {
            1_000_000.0  // $1M in dev
        } else {
            100_000.0    // $100K in production
        };

        let order_value = quantity * price;
        if order_value > max_order_value {
            println!("ORDER REJECTED: Value ${:.2} exceeds limit ${:.2}", order_value, max_order_value);
            return;
        }

        // Execute based on trading mode
        if self.flags.is_enabled("paper_trading") {
            println!("[PAPER TRADE] Simulated execution");
            println!("  Fill price: ${:.2}", price);
            println!("  Status: FILLED (simulated)");
        } else if self.flags.is_enabled("live_trading") {
            println!("[LIVE TRADE] Sending to exchange...");
            println!("  Exchange: Primary");
            println!("  Status: PENDING");
        } else {
            println!("[NO TRADING MODE] Order not executed");
        }

        // Verbose error handling
        if self.flags.is_enabled("verbose_errors") {
            println!("[TRACE] Order processing completed without errors");
        }
    }
}

fn main() {
    println!("=== Environment-Based Feature Flags ===\n");

    // Test different environments
    for env in [Environment::Development, Environment::Staging, Environment::Production] {
        println!("\n==================================================");
        println!("Testing in {:?} environment", env);
        println!("==================================================");

        let flags = EnvFeatureFlags::with_environment(env);
        let executor = OrderExecutor::new(flags);

        executor.execute_order("BTCUSDT", "BUY", 0.5, 50000.0);
    }

    // Test with overrides
    println!("\n==================================================");
    println!("Testing with overrides in Production");
    println!("==================================================");

    let mut flags = EnvFeatureFlags::with_environment(Environment::Production);
    flags.set_override("debug_logging", true);  // Force debug in production
    flags.set_override("paper_trading", true);  // Force paper trading

    let executor = OrderExecutor::new(flags);
    executor.execute_order("ETHUSDT", "SELL", 10.0, 3000.0);
}
```

## Feature Flag Service Pattern

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Feature flag with metadata
#[derive(Clone, Debug)]
pub struct Feature {
    pub name: String,
    pub enabled: bool,
    pub description: String,
    pub owner: String,
    pub created_at: Instant,
    pub last_modified: Instant,
    pub tags: Vec<String>,
}

/// Centralized feature flag service
pub struct FeatureFlagService {
    features: Arc<RwLock<HashMap<String, Feature>>>,
    cache_ttl: Duration,
    last_refresh: RwLock<Instant>,
}

impl FeatureFlagService {
    pub fn new(cache_ttl: Duration) -> Self {
        FeatureFlagService {
            features: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            last_refresh: RwLock::new(Instant::now()),
        }
    }

    /// Register a new feature flag
    pub fn register(&self, name: &str, enabled: bool, description: &str, owner: &str, tags: Vec<&str>) {
        let mut features = self.features.write().unwrap();
        let now = Instant::now();

        features.insert(name.to_string(), Feature {
            name: name.to_string(),
            enabled,
            description: description.to_string(),
            owner: owner.to_string(),
            created_at: now,
            last_modified: now,
            tags: tags.iter().map(|s| s.to_string()).collect(),
        });
    }

    /// Check if feature is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.maybe_refresh();

        let features = self.features.read().unwrap();
        features.get(name).map(|f| f.enabled).unwrap_or(false)
    }

    /// Toggle a feature
    pub fn toggle(&self, name: &str) -> Option<bool> {
        let mut features = self.features.write().unwrap();

        if let Some(feature) = features.get_mut(name) {
            feature.enabled = !feature.enabled;
            feature.last_modified = Instant::now();
            Some(feature.enabled)
        } else {
            None
        }
    }

    /// Enable a feature
    pub fn enable(&self, name: &str) -> bool {
        self.set_enabled(name, true)
    }

    /// Disable a feature
    pub fn disable(&self, name: &str) -> bool {
        self.set_enabled(name, false)
    }

    fn set_enabled(&self, name: &str, enabled: bool) -> bool {
        let mut features = self.features.write().unwrap();

        if let Some(feature) = features.get_mut(name) {
            feature.enabled = enabled;
            feature.last_modified = Instant::now();
            true
        } else {
            false
        }
    }

    /// Get all features with a specific tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<Feature> {
        let features = self.features.read().unwrap();
        features
            .values()
            .filter(|f| f.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Get feature summary
    pub fn summary(&self) -> FeatureSummary {
        let features = self.features.read().unwrap();

        let total = features.len();
        let enabled = features.values().filter(|f| f.enabled).count();
        let by_owner = features.values().fold(HashMap::new(), |mut acc, f| {
            *acc.entry(f.owner.clone()).or_insert(0) += 1;
            acc
        });

        FeatureSummary {
            total,
            enabled,
            disabled: total - enabled,
            by_owner,
        }
    }

    /// Simulate cache refresh (in real app, would fetch from remote config)
    fn maybe_refresh(&self) {
        let last = *self.last_refresh.read().unwrap();
        if last.elapsed() > self.cache_ttl {
            // In real implementation: fetch from config server
            *self.last_refresh.write().unwrap() = Instant::now();
        }
    }

    /// List all features
    pub fn list_all(&self) -> Vec<Feature> {
        let features = self.features.read().unwrap();
        features.values().cloned().collect()
    }
}

#[derive(Debug)]
pub struct FeatureSummary {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub by_owner: HashMap<String, usize>,
}

/// Trading platform using feature flag service
struct TradingPlatform {
    flags: Arc<FeatureFlagService>,
}

impl TradingPlatform {
    fn new(flags: Arc<FeatureFlagService>) -> Self {
        TradingPlatform { flags }
    }

    fn process_market_data(&self, symbol: &str, price: f64) {
        println!("\nProcessing market data: {} @ ${:.2}", symbol, price);

        if self.flags.is_enabled("real_time_alerts") {
            println!("  [ALERT] Price update received");
        }

        if self.flags.is_enabled("price_analytics") {
            println!("  [ANALYTICS] Recording price for analysis");
        }

        if self.flags.is_enabled("ml_scoring") {
            println!("  [ML] Calculating opportunity score...");
        }
    }

    fn execute_strategy(&self, symbol: &str) {
        println!("\nExecuting strategy for {}", symbol);

        if self.flags.is_enabled("stop_loss_v2") {
            println!("  Using enhanced stop-loss logic");
        }

        if self.flags.is_enabled("dynamic_position_sizing") {
            println!("  Using dynamic position sizing");
        }

        if self.flags.is_enabled("multi_exchange_routing") {
            println!("  Checking prices across exchanges");
        }
    }
}

fn main() {
    println!("=== Feature Flag Service Pattern ===\n");

    // Create feature flag service
    let service = Arc::new(FeatureFlagService::new(Duration::from_secs(60)));

    // Register features
    service.register(
        "real_time_alerts",
        true,
        "Send real-time price alerts",
        "alerts-team",
        vec!["notifications", "real-time"],
    );

    service.register(
        "price_analytics",
        true,
        "Record price data for analytics",
        "data-team",
        vec!["analytics", "data"],
    );

    service.register(
        "ml_scoring",
        false,
        "ML-based opportunity scoring",
        "ml-team",
        vec!["ml", "experimental"],
    );

    service.register(
        "stop_loss_v2",
        true,
        "Enhanced stop-loss with trailing",
        "trading-team",
        vec!["trading", "risk"],
    );

    service.register(
        "dynamic_position_sizing",
        false,
        "Adjust position size based on volatility",
        "trading-team",
        vec!["trading", "risk", "experimental"],
    );

    service.register(
        "multi_exchange_routing",
        true,
        "Route orders to best exchange",
        "execution-team",
        vec!["execution", "optimization"],
    );

    // Show summary
    println!("Feature Flag Summary:");
    let summary = service.summary();
    println!("  Total: {}", summary.total);
    println!("  Enabled: {}", summary.enabled);
    println!("  Disabled: {}", summary.disabled);
    println!("  By owner: {:?}", summary.by_owner);

    // List experimental features
    println!("\nExperimental features:");
    for feature in service.get_by_tag("experimental") {
        println!(
            "  - {} ({}): {}",
            feature.name,
            if feature.enabled { "ON" } else { "OFF" },
            feature.description
        );
    }

    // Create platform and process data
    let platform = TradingPlatform::new(Arc::clone(&service));

    platform.process_market_data("BTCUSDT", 50000.0);
    platform.execute_strategy("BTCUSDT");

    // Toggle a feature
    println!("\n--- Enabling ML scoring ---");
    service.enable("ml_scoring");

    platform.process_market_data("ETHUSDT", 3000.0);

    // Disable a feature
    println!("\n--- Disabling real-time alerts ---");
    service.disable("real_time_alerts");

    platform.process_market_data("SOLUSDT", 100.0);
}
```

## A/B Testing with Feature Flags

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// A/B test variant
#[derive(Clone, Debug)]
pub enum Variant {
    Control,
    Treatment(String),
}

/// A/B test configuration
#[derive(Clone, Debug)]
pub struct ABTest {
    pub name: String,
    pub variants: Vec<(String, u8)>,  // (variant_name, percentage)
    pub metrics: Vec<String>,
}

/// A/B testing system for trading strategies
pub struct ABTestingSystem {
    tests: RwLock<HashMap<String, ABTest>>,
    assignments: RwLock<HashMap<(String, String), String>>,  // (test, user) -> variant
    results: RwLock<HashMap<(String, String), Vec<f64>>>,    // (test, variant) -> metric values
}

impl ABTestingSystem {
    pub fn new() -> Self {
        ABTestingSystem {
            tests: RwLock::new(HashMap::new()),
            assignments: RwLock::new(HashMap::new()),
            results: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new A/B test
    pub fn create_test(&self, name: &str, variants: Vec<(&str, u8)>, metrics: Vec<&str>) {
        let test = ABTest {
            name: name.to_string(),
            variants: variants.iter().map(|(n, p)| (n.to_string(), *p)).collect(),
            metrics: metrics.iter().map(|s| s.to_string()).collect(),
        };

        let mut tests = self.tests.write().unwrap();
        tests.insert(name.to_string(), test);
    }

    /// Get variant for user (deterministic based on user_id)
    pub fn get_variant(&self, test_name: &str, user_id: &str) -> Option<String> {
        // Check if already assigned
        let key = (test_name.to_string(), user_id.to_string());
        {
            let assignments = self.assignments.read().unwrap();
            if let Some(variant) = assignments.get(&key) {
                return Some(variant.clone());
            }
        }

        // Get test config
        let tests = self.tests.read().unwrap();
        let test = tests.get(test_name)?;

        // Assign based on hash
        let bucket = self.get_bucket(user_id, test_name);
        let mut cumulative = 0u8;

        for (variant_name, percentage) in &test.variants {
            cumulative += percentage;
            if bucket < cumulative {
                let mut assignments = self.assignments.write().unwrap();
                assignments.insert(key, variant_name.clone());
                return Some(variant_name.clone());
            }
        }

        None
    }

    fn get_bucket(&self, user_id: &str, test_name: &str) -> u8 {
        let mut hasher = DefaultHasher::new();
        format!("{}:{}", user_id, test_name).hash(&mut hasher);
        (hasher.finish() % 100) as u8
    }

    /// Record metric for a variant
    pub fn record_metric(&self, test_name: &str, variant: &str, value: f64) {
        let key = (test_name.to_string(), variant.to_string());
        let mut results = self.results.write().unwrap();
        results.entry(key).or_insert_with(Vec::new).push(value);
    }

    /// Get test results
    pub fn get_results(&self, test_name: &str) -> HashMap<String, TestResults> {
        let tests = self.tests.read().unwrap();
        let results = self.results.read().unwrap();

        let test = match tests.get(test_name) {
            Some(t) => t,
            None => return HashMap::new(),
        };

        let mut variant_results = HashMap::new();

        for (variant_name, _) in &test.variants {
            let key = (test_name.to_string(), variant_name.clone());
            let values = results.get(&key).cloned().unwrap_or_default();

            let count = values.len();
            let mean = if count > 0 {
                values.iter().sum::<f64>() / count as f64
            } else {
                0.0
            };

            let std_dev = if count > 1 {
                let variance = values.iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / (count - 1) as f64;
                variance.sqrt()
            } else {
                0.0
            };

            variant_results.insert(variant_name.clone(), TestResults {
                variant: variant_name.clone(),
                count,
                mean,
                std_dev,
                values,
            });
        }

        variant_results
    }
}

#[derive(Debug)]
pub struct TestResults {
    pub variant: String,
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub values: Vec<f64>,
}

/// Trading strategy with A/B testing
struct TradingStrategyAB {
    ab_system: ABTestingSystem,
}

impl TradingStrategyAB {
    fn new() -> Self {
        let ab_system = ABTestingSystem::new();

        // Create A/B tests for different strategy components
        ab_system.create_test(
            "entry_timing",
            vec![("immediate", 50), ("wait_confirmation", 50)],
            vec!["profit_pct", "win_rate"],
        );

        ab_system.create_test(
            "position_size",
            vec![("fixed_1pct", 33), ("fixed_2pct", 33), ("dynamic", 34)],
            vec!["total_return", "max_drawdown"],
        );

        ab_system.create_test(
            "stop_loss_type",
            vec![("fixed", 50), ("trailing", 50)],
            vec!["avg_loss", "win_rate"],
        );

        TradingStrategyAB { ab_system }
    }

    fn execute_trade(&self, user_id: &str, symbol: &str, signal_strength: f64) {
        println!("\n=== Executing trade for {} ===", user_id);
        println!("Symbol: {}, Signal: {:.2}", symbol, signal_strength);

        // Get variants for this user
        let entry_variant = self.ab_system.get_variant("entry_timing", user_id)
            .unwrap_or("immediate".to_string());
        let size_variant = self.ab_system.get_variant("position_size", user_id)
            .unwrap_or("fixed_1pct".to_string());
        let stop_variant = self.ab_system.get_variant("stop_loss_type", user_id)
            .unwrap_or("fixed".to_string());

        println!("A/B variants:");
        println!("  - Entry: {}", entry_variant);
        println!("  - Position size: {}", size_variant);
        println!("  - Stop loss: {}", stop_variant);

        // Simulate trade execution with variant-specific logic
        let entry_delay = match entry_variant.as_str() {
            "immediate" => 0,
            "wait_confirmation" => 100,
            _ => 0,
        };

        let position_pct = match size_variant.as_str() {
            "fixed_1pct" => 1.0,
            "fixed_2pct" => 2.0,
            "dynamic" => signal_strength * 2.0,
            _ => 1.0,
        };

        let stop_distance = match stop_variant.as_str() {
            "fixed" => 2.0,
            "trailing" => 1.5,
            _ => 2.0,
        };

        println!("Trade parameters:");
        println!("  - Entry delay: {}ms", entry_delay);
        println!("  - Position: {:.1}% of portfolio", position_pct);
        println!("  - Stop distance: {:.1}%", stop_distance);

        // Simulate outcome and record metrics
        let profit_pct = (signal_strength - 0.5) * 4.0 + (rand_simple() - 0.5);
        let win = profit_pct > 0.0;

        println!("Outcome: {:.2}% ({})", profit_pct, if win { "WIN" } else { "LOSS" });

        // Record results for A/B analysis
        self.ab_system.record_metric("entry_timing", &entry_variant, profit_pct);
        self.ab_system.record_metric("position_size", &size_variant, profit_pct);
        self.ab_system.record_metric("stop_loss_type", &stop_variant, if win { 1.0 } else { 0.0 });
    }

    fn print_ab_results(&self) {
        println!("\n=== A/B Test Results ===\n");

        for test_name in ["entry_timing", "position_size", "stop_loss_type"] {
            println!("Test: {}", test_name);

            let results = self.ab_system.get_results(test_name);
            for (variant, data) in &results {
                println!(
                    "  {}: n={}, mean={:.3}, std={:.3}",
                    variant, data.count, data.mean, data.std_dev
                );
            }
            println!();
        }
    }
}

/// Simple pseudo-random for demo (not for production)
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

fn main() {
    println!("=== A/B Testing with Feature Flags ===");

    let strategy = TradingStrategyAB::new();

    // Simulate trades for multiple users
    let users = ["user_001", "user_002", "user_003", "user_004", "user_005",
                 "user_006", "user_007", "user_008", "user_009", "user_010"];

    for user in &users {
        let signal = 0.3 + (rand_simple() * 0.4);  // Random signal 0.3-0.7
        strategy.execute_trade(user, "BTCUSDT", signal);
    }

    // Print A/B test results
    strategy.print_ab_results();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Feature Flags** | Runtime switches for enabling/disabling features |
| **Compile-time Flags** | Cargo features that exclude code at compile time |
| **Percentage Rollout** | Gradually enabling features for user subsets |
| **Environment Flags** | Different behavior for dev/staging/production |
| **Kill Switch** | Ability to instantly disable problematic features |
| **A/B Testing** | Comparing feature variants with metrics |
| **Feature Service** | Centralized management of all feature flags |

## Practical Exercises

1. **Risk Control Feature Flags**: Create a system that:
   - Has multiple risk check levels (basic, standard, strict)
   - Uses feature flags to enable/disable each level
   - Allows quick disable of trading in emergencies
   - Logs all flag changes with timestamps

2. **Strategy A/B Testing**: Implement a system:
   - Tests two versions of entry signal logic
   - Tracks win rate and profit for each variant
   - Automatically rolls out winning variant
   - Provides statistical significance metrics

3. **Environment-Aware Deployment**: Build a configuration:
   - Uses compile-time flags for dev dependencies
   - Uses runtime flags for feature toggles
   - Different exchange connections per environment
   - Automatic flag sync from config server

4. **Feature Dashboard**: Create a monitoring tool:
   - Lists all active feature flags
   - Shows percentage of users in each variant
   - Displays metrics for each feature
   - Allows toggling features via API

## Homework

1. **Multi-Tenant Feature Flags**: Implement a system that:
   - Supports per-account feature configuration
   - Allows premium features only for paid accounts
   - Has beta program with early access features
   - Tracks feature usage per tenant
   - Generates billing reports based on features used

2. **Canary Deployment System**: Create a system that:
   - Automatically increases rollout percentage
   - Monitors error rates and rollback on threshold
   - Sends alerts on anomalies during rollout
   - Keeps audit log of all deployments
   - Supports instant rollback to previous state

3. **Feature Flag Testing Framework**: Build a framework that:
   - Tests all feature flag combinations
   - Verifies no conflicts between flags
   - Generates coverage report for flags
   - Validates flag dependencies
   - Checks for stale/unused flags

4. **Trading Strategy Lab**: Develop a system that:
   - Runs multiple strategy variants simultaneously
   - Uses feature flags to control allocation
   - Compares performance in real-time
   - Automatically promotes best performers
   - Provides detailed analytics dashboard

## Navigation

[← Previous day](../326-async-vs-threading/en.md) | [Next day →](../338-*/en.md)
