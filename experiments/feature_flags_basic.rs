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

    fn perform_advanced_risk_check(&self, _symbol: &str, quantity: f64, price: f64) {
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
