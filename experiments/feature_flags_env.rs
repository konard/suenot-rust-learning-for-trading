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
