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
