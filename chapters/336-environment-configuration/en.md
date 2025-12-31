# Day 336: Environment Configuration

## Trading Analogy

Imagine you have a trading bot that operates in three modes:

**Development:**
You test new strategies on historical data. Using test API keys, small volumes, detailed logging. Errors are normal, you're learning.

**Staging:**
Bot is connected to the exchange testnet. Trading with "paper" money, but in real-time. You verify everything works before going to production.

**Production:**
Real money, real trades. Minimal logging, maximum performance, strict risk limits.

| Environment | API | Volumes | Logging | Risk |
|-------------|-----|---------|---------|------|
| **Development** | Test | Minimal | Detailed | Zero |
| **Staging** | Testnet | Medium | Moderate | Low |
| **Production** | Real | Working | Minimal | Real |

## Environment Variables

The primary way to configure in Rust — environment variables:

```rust
use std::env;

/// Trading bot configuration
#[derive(Debug, Clone)]
struct TradingConfig {
    /// Current environment
    environment: Environment,
    /// Exchange API key
    api_key: String,
    /// API secret
    api_secret: String,
    /// Exchange base URL
    api_base_url: String,
    /// Maximum position size
    max_position_size: f64,
    /// Maximum risk per trade in percent
    max_risk_percent: f64,
    /// Log level
    log_level: String,
}

#[derive(Debug, Clone, PartialEq)]
enum Environment {
    Development,
    Staging,
    Production,
}

impl TradingConfig {
    /// Loads configuration from environment variables
    fn from_env() -> Result<Self, String> {
        // Determine environment
        let env_str = env::var("TRADING_ENV")
            .unwrap_or_else(|_| "development".to_string());

        let environment = match env_str.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        };

        // Load API keys
        let api_key = env::var("TRADING_API_KEY")
            .map_err(|_| "TRADING_API_KEY is not set")?;

        let api_secret = env::var("TRADING_API_SECRET")
            .map_err(|_| "TRADING_API_SECRET is not set")?;

        // URL depends on environment
        let api_base_url = match environment {
            Environment::Production => {
                env::var("TRADING_API_URL")
                    .unwrap_or_else(|_| "https://api.exchange.com".to_string())
            }
            Environment::Staging => {
                "https://testnet.exchange.com".to_string()
            }
            Environment::Development => {
                "http://localhost:8080".to_string()
            }
        };

        // Risk limits depend on environment
        let (max_position_size, max_risk_percent) = match environment {
            Environment::Production => (10000.0, 2.0),
            Environment::Staging => (1000.0, 5.0),
            Environment::Development => (100.0, 10.0),
        };

        // Log level
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| {
            match environment {
                Environment::Production => "warn".to_string(),
                Environment::Staging => "info".to_string(),
                Environment::Development => "debug".to_string(),
            }
        });

        Ok(TradingConfig {
            environment,
            api_key,
            api_secret,
            api_base_url,
            max_position_size,
            max_risk_percent,
            log_level,
        })
    }

    /// Checks if live trading is allowed
    fn is_live_trading_allowed(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Returns maximum order size for current environment
    fn max_order_size(&self) -> f64 {
        self.max_position_size * (self.max_risk_percent / 100.0)
    }
}

fn main() {
    // For demonstration, set variables manually
    env::set_var("TRADING_ENV", "staging");
    env::set_var("TRADING_API_KEY", "test_key_12345");
    env::set_var("TRADING_API_SECRET", "test_secret_67890");

    match TradingConfig::from_env() {
        Ok(config) => {
            println!("=== Trading Bot Configuration ===\n");
            println!("Environment: {:?}", config.environment);
            println!("API URL: {}", config.api_base_url);
            println!("Max position: ${:.2}", config.max_position_size);
            println!("Max risk: {:.1}%", config.max_risk_percent);
            println!("Max order: ${:.2}", config.max_order_size());
            println!("Log level: {}", config.log_level);
            println!("\nLive trading: {}",
                if config.is_live_trading_allowed() { "YES" } else { "NO" }
            );
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
        }
    }
}
```

## The dotenv Library

For local development it's convenient to use `.env` files:

```rust
use std::env;
use std::fs;
use std::collections::HashMap;

/// Simple .env file parser
fn load_dotenv(path: &str) -> Result<HashMap<String, String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;

    let mut vars = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse KEY=VALUE
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            // Remove quotes if present
            let value = value.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            vars.insert(key, value);
        }
    }

    Ok(vars)
}

/// Loads .env file and sets environment variables
fn init_dotenv() {
    // Determine which file to load
    let env_file = match env::var("TRADING_ENV").ok() {
        Some(e) if e == "production" => ".env.production",
        Some(e) if e == "staging" => ".env.staging",
        _ => ".env.development",
    };

    // Try environment-specific file first, then general .env
    let files_to_try = [env_file, ".env"];

    for file in files_to_try {
        if let Ok(vars) = load_dotenv(file) {
            println!("Loaded config from: {}", file);
            for (key, value) in vars {
                // Don't overwrite already set variables
                if env::var(&key).is_err() {
                    env::set_var(&key, &value);
                }
            }
            break;
        }
    }
}

/// Exchange connection configuration
#[derive(Debug)]
struct ExchangeConfig {
    name: String,
    api_key: String,
    api_secret: String,
    base_url: String,
    rate_limit: u32,      // requests per second
    timeout_ms: u64,
}

impl ExchangeConfig {
    fn from_env(prefix: &str) -> Result<Self, String> {
        let get_var = |suffix: &str| -> Result<String, String> {
            let key = format!("{}_{}", prefix, suffix);
            env::var(&key).map_err(|_| format!("{} is not set", key))
        };

        let get_var_or_default = |suffix: &str, default: &str| -> String {
            let key = format!("{}_{}", prefix, suffix);
            env::var(&key).unwrap_or_else(|_| default.to_string())
        };

        Ok(ExchangeConfig {
            name: prefix.to_lowercase(),
            api_key: get_var("API_KEY")?,
            api_secret: get_var("API_SECRET")?,
            base_url: get_var_or_default("BASE_URL", "https://api.exchange.com"),
            rate_limit: get_var_or_default("RATE_LIMIT", "10")
                .parse()
                .unwrap_or(10),
            timeout_ms: get_var_or_default("TIMEOUT_MS", "5000")
                .parse()
                .unwrap_or(5000),
        })
    }
}

fn main() {
    println!("=== Example .env File ===\n");

    // Create a test .env file
    let env_content = r#"
# Trading bot settings
TRADING_ENV=staging

# Binance
BINANCE_API_KEY=binance_key_123
BINANCE_API_SECRET=binance_secret_456
BINANCE_BASE_URL=https://testnet.binance.vision
BINANCE_RATE_LIMIT=20

# Kraken
KRAKEN_API_KEY=kraken_key_789
KRAKEN_API_SECRET=kraken_secret_012
KRAKEN_RATE_LIMIT=15

# General settings
LOG_LEVEL=debug
MAX_CONCURRENT_ORDERS=5
"#;

    // Write test file
    fs::write(".env.test", env_content).unwrap();

    // Load it
    if let Ok(vars) = load_dotenv(".env.test") {
        println!("Loaded {} variables:\n", vars.len());
        for (key, value) in &vars {
            // Hide secrets
            let display_value = if key.contains("SECRET") {
                "***".to_string()
            } else {
                value.clone()
            };
            println!("  {} = {}", key, display_value);
        }

        // Set variables
        for (key, value) in vars {
            env::set_var(&key, &value);
        }
    }

    println!("\n=== Exchange Configurations ===\n");

    // Load exchange configurations
    for exchange in ["BINANCE", "KRAKEN"] {
        match ExchangeConfig::from_env(exchange) {
            Ok(config) => {
                println!("{}: URL={}, Rate Limit={}/s",
                    config.name, config.base_url, config.rate_limit);
            }
            Err(e) => println!("{}: Error - {}", exchange, e),
        }
    }

    // Remove test file
    let _ = fs::remove_file(".env.test");
}
```

## Configuration Files (TOML, JSON)

For complex configurations use structured files:

```rust
use std::fs;
use std::collections::HashMap;

/// Trading strategy configuration
#[derive(Debug, Clone)]
struct StrategyConfig {
    name: String,
    enabled: bool,
    symbols: Vec<String>,
    parameters: HashMap<String, f64>,
    risk_limits: RiskLimits,
}

#[derive(Debug, Clone)]
struct RiskLimits {
    max_position_pct: f64,
    max_drawdown_pct: f64,
    max_daily_loss: f64,
    stop_loss_pct: f64,
}

/// Full system configuration
#[derive(Debug)]
struct SystemConfig {
    environment: String,
    strategies: Vec<StrategyConfig>,
    exchanges: HashMap<String, ExchangeSettings>,
    database_url: String,
    metrics_port: u16,
}

#[derive(Debug)]
struct ExchangeSettings {
    enabled: bool,
    api_key_env: String,
    api_secret_env: String,
    base_url: String,
}

/// Simple TOML parser (simplified version)
fn parse_simple_toml(content: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section [section]
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].to_string();
            continue;
        }

        // Key = value
        if let Some((key, value)) = line.split_once('=') {
            let full_key = if current_section.is_empty() {
                key.trim().to_string()
            } else {
                format!("{}.{}", current_section, key.trim())
            };

            let value = value.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            result.insert(full_key, value);
        }
    }

    result
}

fn main() {
    println!("=== Configuration via TOML ===\n");

    // Example TOML configuration
    let toml_content = r#"
# Trading system configuration

environment = "staging"
database_url = "postgres://localhost/trading"
metrics_port = 9090

[exchange.binance]
enabled = true
api_key_env = "BINANCE_API_KEY"
api_secret_env = "BINANCE_API_SECRET"
base_url = "https://testnet.binance.vision"

[exchange.kraken]
enabled = false
api_key_env = "KRAKEN_API_KEY"
api_secret_env = "KRAKEN_API_SECRET"
base_url = "https://api.kraken.com"

[strategy.momentum]
enabled = true
symbols = "BTCUSDT,ETHUSDT,SOLUSDT"
lookback_period = 20
entry_threshold = 0.02
exit_threshold = 0.01

[strategy.momentum.risk]
max_position_pct = 10.0
max_drawdown_pct = 5.0
max_daily_loss = 1000.0
stop_loss_pct = 2.0

[strategy.mean_reversion]
enabled = true
symbols = "BTCUSDT"
window_size = 50
std_multiplier = 2.0

[strategy.mean_reversion.risk]
max_position_pct = 5.0
max_drawdown_pct = 3.0
max_daily_loss = 500.0
stop_loss_pct = 1.5
"#;

    // Parse TOML
    let config = parse_simple_toml(toml_content);

    println!("Environment: {}", config.get("environment").unwrap_or(&"unknown".to_string()));
    println!("Database: {}", config.get("database_url").unwrap_or(&"".to_string()));
    println!("Metrics port: {}", config.get("metrics_port").unwrap_or(&"".to_string()));

    println!("\n--- Exchanges ---");
    for exchange in ["binance", "kraken"] {
        let enabled = config.get(&format!("exchange.{}.enabled", exchange))
            .map(|s| s == "true")
            .unwrap_or(false);
        let url = config.get(&format!("exchange.{}.base_url", exchange))
            .unwrap_or(&"N/A".to_string())
            .clone();
        println!("{}: {} ({})", exchange, if enabled { "ON" } else { "OFF" }, url);
    }

    println!("\n--- Strategies ---");
    for strategy in ["momentum", "mean_reversion"] {
        let enabled = config.get(&format!("strategy.{}.enabled", strategy))
            .map(|s| s == "true")
            .unwrap_or(false);
        let symbols = config.get(&format!("strategy.{}.symbols", strategy))
            .unwrap_or(&"".to_string())
            .clone();
        let max_pos = config.get(&format!("strategy.{}.risk.max_position_pct", strategy))
            .unwrap_or(&"0".to_string())
            .clone();

        if enabled {
            println!("{}: symbols=[{}], max_position={}%",
                strategy, symbols, max_pos);
        }
    }
}
```

## Configuration via Builder Pattern

For flexible configuration use the Builder pattern:

```rust
use std::env;
use std::collections::HashMap;

/// Trading engine configuration
#[derive(Debug, Clone)]
struct TradingEngine {
    name: String,
    environment: String,
    exchanges: Vec<String>,
    strategies: Vec<String>,
    risk_limits: RiskConfig,
    logging: LogConfig,
    features: HashMap<String, bool>,
}

#[derive(Debug, Clone, Default)]
struct RiskConfig {
    max_position_size: f64,
    max_daily_loss: f64,
    max_drawdown: f64,
    position_sizing: String,
}

#[derive(Debug, Clone, Default)]
struct LogConfig {
    level: String,
    file_path: Option<String>,
    json_format: bool,
}

/// Builder for creating configuration
#[derive(Default)]
struct TradingEngineBuilder {
    name: Option<String>,
    environment: Option<String>,
    exchanges: Vec<String>,
    strategies: Vec<String>,
    risk_limits: RiskConfig,
    logging: LogConfig,
    features: HashMap<String, bool>,
}

impl TradingEngineBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    fn environment(mut self, env: &str) -> Self {
        self.environment = Some(env.to_string());
        self
    }

    fn add_exchange(mut self, exchange: &str) -> Self {
        self.exchanges.push(exchange.to_string());
        self
    }

    fn add_strategy(mut self, strategy: &str) -> Self {
        self.strategies.push(strategy.to_string());
        self
    }

    fn max_position_size(mut self, size: f64) -> Self {
        self.risk_limits.max_position_size = size;
        self
    }

    fn max_daily_loss(mut self, loss: f64) -> Self {
        self.risk_limits.max_daily_loss = loss;
        self
    }

    fn max_drawdown(mut self, drawdown: f64) -> Self {
        self.risk_limits.max_drawdown = drawdown;
        self
    }

    fn position_sizing(mut self, method: &str) -> Self {
        self.risk_limits.position_sizing = method.to_string();
        self
    }

    fn log_level(mut self, level: &str) -> Self {
        self.logging.level = level.to_string();
        self
    }

    fn log_to_file(mut self, path: &str) -> Self {
        self.logging.file_path = Some(path.to_string());
        self
    }

    fn json_logging(mut self, enabled: bool) -> Self {
        self.logging.json_format = enabled;
        self
    }

    fn feature(mut self, name: &str, enabled: bool) -> Self {
        self.features.insert(name.to_string(), enabled);
        self
    }

    /// Creates configuration based on environment variables
    fn from_env(mut self) -> Self {
        // Environment
        if let Ok(env) = env::var("TRADING_ENV") {
            self.environment = Some(env);
        }

        // Risk limits from environment
        if let Ok(val) = env::var("MAX_POSITION_SIZE") {
            if let Ok(size) = val.parse() {
                self.risk_limits.max_position_size = size;
            }
        }

        if let Ok(val) = env::var("MAX_DAILY_LOSS") {
            if let Ok(loss) = val.parse() {
                self.risk_limits.max_daily_loss = loss;
            }
        }

        // Logging
        if let Ok(level) = env::var("LOG_LEVEL") {
            self.logging.level = level;
        }

        self
    }

    fn build(self) -> Result<TradingEngine, String> {
        let name = self.name.ok_or("Engine name not specified")?;
        let environment = self.environment.unwrap_or_else(|| "development".to_string());

        if self.exchanges.is_empty() {
            return Err("No exchanges specified".to_string());
        }

        Ok(TradingEngine {
            name,
            environment,
            exchanges: self.exchanges,
            strategies: self.strategies,
            risk_limits: self.risk_limits,
            logging: self.logging,
            features: self.features,
        })
    }
}

/// Preset configurations for different environments
impl TradingEngineBuilder {
    fn development() -> Self {
        Self::new()
            .environment("development")
            .max_position_size(100.0)
            .max_daily_loss(50.0)
            .max_drawdown(10.0)
            .position_sizing("fixed")
            .log_level("debug")
            .feature("paper_trading", true)
            .feature("backtesting", true)
            .feature("live_trading", false)
    }

    fn staging() -> Self {
        Self::new()
            .environment("staging")
            .max_position_size(1000.0)
            .max_daily_loss(200.0)
            .max_drawdown(5.0)
            .position_sizing("percent")
            .log_level("info")
            .feature("paper_trading", true)
            .feature("backtesting", true)
            .feature("live_trading", false)
    }

    fn production() -> Self {
        Self::new()
            .environment("production")
            .max_position_size(10000.0)
            .max_daily_loss(500.0)
            .max_drawdown(3.0)
            .position_sizing("kelly")
            .log_level("warn")
            .json_logging(true)
            .feature("paper_trading", false)
            .feature("backtesting", false)
            .feature("live_trading", true)
    }
}

fn main() {
    println!("=== Builder Pattern for Configuration ===\n");

    // Create development configuration
    let dev_engine = TradingEngineBuilder::development()
        .name("DevBot")
        .add_exchange("binance_testnet")
        .add_strategy("momentum")
        .add_strategy("mean_reversion")
        .build()
        .unwrap();

    println!("Development:\n{:#?}\n", dev_engine);

    // Production configuration
    let prod_engine = TradingEngineBuilder::production()
        .name("ProdBot")
        .add_exchange("binance")
        .add_exchange("kraken")
        .add_strategy("momentum")
        .log_to_file("/var/log/trading/bot.log")
        .from_env()  // Override from environment
        .build()
        .unwrap();

    println!("Production:\n{:#?}", prod_engine);
}
```

## Configuration Validation

It's critical to validate configuration at startup:

```rust
use std::env;

#[derive(Debug)]
struct ValidatedConfig {
    environment: String,
    api_key: String,
    max_position: f64,
    risk_per_trade: f64,
    allowed_symbols: Vec<String>,
}

#[derive(Debug)]
enum ConfigError {
    MissingRequired(String),
    InvalidValue { field: String, value: String, reason: String },
    SecurityRisk(String),
    InconsistentConfig(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingRequired(field) =>
                write!(f, "Required field missing: {}", field),
            ConfigError::InvalidValue { field, value, reason } =>
                write!(f, "Invalid value {}: '{}' - {}", field, value, reason),
            ConfigError::SecurityRisk(msg) =>
                write!(f, "Security risk: {}", msg),
            ConfigError::InconsistentConfig(msg) =>
                write!(f, "Inconsistent configuration: {}", msg),
        }
    }
}

struct ConfigValidator {
    errors: Vec<ConfigError>,
    warnings: Vec<String>,
}

impl ConfigValidator {
    fn new() -> Self {
        ConfigValidator {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn validate_required(&mut self, key: &str) -> Option<String> {
        match env::var(key) {
            Ok(value) if !value.is_empty() => Some(value),
            _ => {
                self.errors.push(ConfigError::MissingRequired(key.to_string()));
                None
            }
        }
    }

    fn validate_positive_f64(&mut self, key: &str, default: f64) -> f64 {
        match env::var(key) {
            Ok(value) => match value.parse::<f64>() {
                Ok(num) if num > 0.0 => num,
                Ok(num) => {
                    self.errors.push(ConfigError::InvalidValue {
                        field: key.to_string(),
                        value: num.to_string(),
                        reason: "must be positive".to_string(),
                    });
                    default
                }
                Err(_) => {
                    self.errors.push(ConfigError::InvalidValue {
                        field: key.to_string(),
                        value,
                        reason: "is not a number".to_string(),
                    });
                    default
                }
            },
            Err(_) => default,
        }
    }

    fn validate_range(&mut self, key: &str, min: f64, max: f64, default: f64) -> f64 {
        let value = self.validate_positive_f64(key, default);
        if value < min || value > max {
            self.errors.push(ConfigError::InvalidValue {
                field: key.to_string(),
                value: value.to_string(),
                reason: format!("must be in range [{}, {}]", min, max),
            });
            default
        } else {
            value
        }
    }

    fn validate_api_key(&mut self, key: &str) -> Option<String> {
        match env::var(key) {
            Ok(value) => {
                // Check key format
                if value.len() < 16 {
                    self.errors.push(ConfigError::InvalidValue {
                        field: key.to_string(),
                        value: "***".to_string(),
                        reason: "too short (minimum 16 characters)".to_string(),
                    });
                    return None;
                }

                // Warn about test keys in production
                if env::var("TRADING_ENV").unwrap_or_default() == "production" {
                    if value.starts_with("test") || value.starts_with("demo") {
                        self.errors.push(ConfigError::SecurityRisk(
                            format!("{} looks like a test key in production", key)
                        ));
                    }
                }

                Some(value)
            }
            Err(_) => {
                self.errors.push(ConfigError::MissingRequired(key.to_string()));
                None
            }
        }
    }

    fn validate_symbols(&mut self, key: &str) -> Vec<String> {
        match env::var(key) {
            Ok(value) => {
                let symbols: Vec<String> = value
                    .split(',')
                    .map(|s| s.trim().to_uppercase())
                    .filter(|s| !s.is_empty())
                    .collect();

                // Check symbol format
                for symbol in &symbols {
                    if !symbol.chars().all(|c| c.is_alphanumeric()) {
                        self.warnings.push(
                            format!("Symbol '{}' contains special characters", symbol)
                        );
                    }
                }

                if symbols.is_empty() {
                    self.warnings.push("Symbol list is empty".to_string());
                }

                symbols
            }
            Err(_) => {
                self.warnings.push(
                    format!("{} not set, using default symbols", key)
                );
                vec!["BTCUSDT".to_string()]
            }
        }
    }

    fn cross_validate(&mut self, environment: &str, max_position: f64, risk: f64) {
        // Check limit consistency
        if environment == "production" && max_position > 50000.0 {
            self.warnings.push(
                "Very high position limit for production".to_string()
            );
        }

        if environment == "development" && risk > 10.0 {
            self.errors.push(ConfigError::InconsistentConfig(
                "Risk > 10% for development — possible error".to_string()
            ));
        }

        // Production requires strict limits
        if environment == "production" && risk > 5.0 {
            self.errors.push(ConfigError::SecurityRisk(
                "Risk > 5% in production is prohibited by policy".to_string()
            ));
        }
    }

    fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    fn print_report(&self) {
        if !self.warnings.is_empty() {
            println!("=== Warnings ===");
            for warning in &self.warnings {
                println!("  WARN: {}", warning);
            }
            println!();
        }

        if !self.errors.is_empty() {
            println!("=== Errors ===");
            for error in &self.errors {
                println!("  ERROR: {}", error);
            }
            println!();
        }
    }
}

fn load_validated_config() -> Result<ValidatedConfig, Vec<ConfigError>> {
    let mut validator = ConfigValidator::new();

    // Load and validate
    let environment = validator.validate_required("TRADING_ENV")
        .unwrap_or_else(|| "development".to_string());

    let api_key = validator.validate_api_key("TRADING_API_KEY")
        .unwrap_or_default();

    let max_position = validator.validate_positive_f64("MAX_POSITION", 1000.0);
    let risk_per_trade = validator.validate_range("RISK_PER_TRADE", 0.1, 10.0, 2.0);
    let allowed_symbols = validator.validate_symbols("ALLOWED_SYMBOLS");

    // Cross-validation
    validator.cross_validate(&environment, max_position, risk_per_trade);

    // Print report
    validator.print_report();

    if validator.is_valid() {
        Ok(ValidatedConfig {
            environment,
            api_key,
            max_position,
            risk_per_trade,
            allowed_symbols,
        })
    } else {
        Err(validator.errors)
    }
}

fn main() {
    println!("=== Configuration Validation ===\n");

    // Set test variables
    env::set_var("TRADING_ENV", "staging");
    env::set_var("TRADING_API_KEY", "staging_key_1234567890");
    env::set_var("MAX_POSITION", "5000");
    env::set_var("RISK_PER_TRADE", "3.0");
    env::set_var("ALLOWED_SYMBOLS", "BTCUSDT, ETHUSDT, SOLUSDT");

    match load_validated_config() {
        Ok(config) => {
            println!("Configuration loaded successfully!");
            println!("Environment: {}", config.environment);
            println!("Max position: ${:.2}", config.max_position);
            println!("Risk per trade: {:.1}%", config.risk_per_trade);
            println!("Symbols: {:?}", config.allowed_symbols);
        }
        Err(errors) => {
            println!("Configuration is invalid! {} errors:", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
            std::process::exit(1);
        }
    }
}
```

## Secrets and Security

Never store secrets in code:

```rust
use std::env;
use std::fs;

/// Secure secret loader
struct SecretLoader {
    environment: String,
}

impl SecretLoader {
    fn new() -> Self {
        let environment = env::var("TRADING_ENV")
            .unwrap_or_else(|_| "development".to_string());
        SecretLoader { environment }
    }

    /// Loads secret from different sources depending on environment
    fn load_secret(&self, name: &str) -> Result<String, String> {
        // 1. First check environment variables
        if let Ok(value) = env::var(name) {
            return Ok(value);
        }

        // 2. In production — try secret files (Docker secrets, Kubernetes)
        if self.environment == "production" {
            let secret_path = format!("/run/secrets/{}", name.to_lowercase());
            if let Ok(value) = fs::read_to_string(&secret_path) {
                return Ok(value.trim().to_string());
            }
        }

        // 3. For development — try local secrets file
        if self.environment == "development" {
            let local_path = format!(".secrets/{}", name.to_lowercase());
            if let Ok(value) = fs::read_to_string(&local_path) {
                return Ok(value.trim().to_string());
            }
        }

        Err(format!("Secret {} not found", name))
    }

    /// Masks secret for logging
    fn mask_secret(secret: &str) -> String {
        if secret.len() <= 8 {
            return "****".to_string();
        }
        let visible = &secret[..4];
        format!("{}****", visible)
    }
}

/// Configuration with secrets
struct SecureConfig {
    api_key: String,
    api_secret: String,
    database_password: String,
}

impl SecureConfig {
    fn load() -> Result<Self, String> {
        let loader = SecretLoader::new();

        Ok(SecureConfig {
            api_key: loader.load_secret("TRADING_API_KEY")?,
            api_secret: loader.load_secret("TRADING_API_SECRET")?,
            database_password: loader.load_secret("DATABASE_PASSWORD")?,
        })
    }

    /// Safe output for logging
    fn log_safe(&self) -> String {
        format!(
            "SecureConfig {{ api_key: {}, api_secret: {}, db_password: {} }}",
            SecretLoader::mask_secret(&self.api_key),
            SecretLoader::mask_secret(&self.api_secret),
            SecretLoader::mask_secret(&self.database_password),
        )
    }
}

fn main() {
    println!("=== Secure Secret Loading ===\n");

    // Create directory for secrets
    let _ = fs::create_dir(".secrets");

    // Create test secret files
    fs::write(".secrets/trading_api_key", "dev_api_key_12345678").unwrap();
    fs::write(".secrets/trading_api_secret", "dev_secret_87654321").unwrap();
    fs::write(".secrets/database_password", "dev_db_pass_secure").unwrap();

    // Set environment
    env::set_var("TRADING_ENV", "development");

    match SecureConfig::load() {
        Ok(config) => {
            println!("Secrets loaded!");
            println!("Log-safe output: {}", config.log_safe());

            // Never do this in real code!
            // println!("API Key: {}", config.api_key);
        }
        Err(e) => {
            println!("Error loading secrets: {}", e);
        }
    }

    // Remove test files
    let _ = fs::remove_dir_all(".secrets");
}
```

## Feature Flags for Trading

Managing features through configuration:

```rust
use std::env;
use std::collections::HashMap;

/// Feature flags for trading system
#[derive(Debug, Clone)]
struct FeatureFlags {
    flags: HashMap<String, bool>,
    environment: String,
}

impl FeatureFlags {
    fn new() -> Self {
        let environment = env::var("TRADING_ENV")
            .unwrap_or_else(|_| "development".to_string());

        let mut flags = HashMap::new();

        // Set defaults based on environment
        match environment.as_str() {
            "production" => {
                flags.insert("live_trading".to_string(), true);
                flags.insert("paper_trading".to_string(), false);
                flags.insert("debug_logging".to_string(), false);
                flags.insert("experimental_strategies".to_string(), false);
                flags.insert("margin_trading".to_string(), false);
                flags.insert("auto_compound".to_string(), true);
            }
            "staging" => {
                flags.insert("live_trading".to_string(), false);
                flags.insert("paper_trading".to_string(), true);
                flags.insert("debug_logging".to_string(), true);
                flags.insert("experimental_strategies".to_string(), true);
                flags.insert("margin_trading".to_string(), false);
                flags.insert("auto_compound".to_string(), true);
            }
            _ => {
                flags.insert("live_trading".to_string(), false);
                flags.insert("paper_trading".to_string(), true);
                flags.insert("debug_logging".to_string(), true);
                flags.insert("experimental_strategies".to_string(), true);
                flags.insert("margin_trading".to_string(), true);
                flags.insert("auto_compound".to_string(), false);
            }
        }

        // Override from environment variables
        for (flag, _) in flags.clone().iter() {
            let env_key = format!("FEATURE_{}", flag.to_uppercase());
            if let Ok(value) = env::var(&env_key) {
                let enabled = value == "1" || value.to_lowercase() == "true";
                flags.insert(flag.clone(), enabled);
            }
        }

        FeatureFlags { flags, environment }
    }

    fn is_enabled(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    fn require(&self, flag: &str) -> Result<(), String> {
        if self.is_enabled(flag) {
            Ok(())
        } else {
            Err(format!("Feature '{}' is disabled in '{}' environment",
                flag, self.environment))
        }
    }
}

/// Trading engine with feature flags
struct TradingEngine {
    features: FeatureFlags,
    balance: f64,
}

impl TradingEngine {
    fn new(initial_balance: f64) -> Self {
        TradingEngine {
            features: FeatureFlags::new(),
            balance: initial_balance,
        }
    }

    fn execute_trade(&mut self, symbol: &str, side: &str, amount: f64) -> Result<String, String> {
        // Check if we can trade
        if self.features.is_enabled("live_trading") {
            println!("[LIVE] Executing real trade: {} {} {}", side, amount, symbol);
            // Real trading logic
            Ok(format!("LIVE-ORDER-{}", chrono_mock()))
        } else if self.features.is_enabled("paper_trading") {
            println!("[PAPER] Simulating trade: {} {} {}", side, amount, symbol);
            // Update virtual balance
            if side == "BUY" {
                self.balance -= amount * 50000.0; // approximate price
            } else {
                self.balance += amount * 50000.0;
            }
            Ok(format!("PAPER-ORDER-{}", chrono_mock()))
        } else {
            Err("Neither live_trading nor paper_trading is enabled".to_string())
        }
    }

    fn enable_margin(&mut self) -> Result<(), String> {
        self.features.require("margin_trading")?;
        println!("Margin trading activated");
        Ok(())
    }

    fn run_experimental_strategy(&self, name: &str) -> Result<(), String> {
        self.features.require("experimental_strategies")?;
        println!("Running experimental strategy: {}", name);
        Ok(())
    }

    fn status(&self) {
        println!("\n=== Engine Status ===");
        println!("Balance: ${:.2}", self.balance);
        println!("\nEnabled features:");
        for (flag, enabled) in &self.features.flags {
            let status = if *enabled { "ON" } else { "OFF" };
            println!("  {}: {}", flag, status);
        }
    }
}

fn chrono_mock() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn main() {
    println!("=== Feature Flags for Trading ===\n");

    // Set staging environment
    env::set_var("TRADING_ENV", "staging");
    // Enable experimental feature
    env::set_var("FEATURE_EXPERIMENTAL_STRATEGIES", "true");

    let mut engine = TradingEngine::new(10000.0);

    // Status
    engine.status();

    println!("\n=== Operations ===\n");

    // Try to trade
    match engine.execute_trade("BTCUSDT", "BUY", 0.1) {
        Ok(order_id) => println!("Order created: {}", order_id),
        Err(e) => println!("Error: {}", e),
    }

    // Try to enable margin (should fail in staging)
    match engine.enable_margin() {
        Ok(_) => println!("Margin enabled"),
        Err(e) => println!("Margin unavailable: {}", e),
    }

    // Experimental strategy (should work)
    match engine.run_experimental_strategy("quantum_prediction") {
        Ok(_) => println!("Strategy started"),
        Err(e) => println!("Strategy unavailable: {}", e),
    }

    engine.status();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Environment Variables** | Primary configuration method via `std::env` |
| **dotenv** | `.env` files for local development |
| **TOML/JSON** | Structured configuration files |
| **Builder Pattern** | Flexible configuration creation |
| **Validation** | Configuration checking at startup |
| **Secrets** | Secure API key storage |
| **Feature Flags** | Managing features through configuration |

## Practical Exercises

1. **Multi-Exchange Configurator**: Create a system that:
   - Loads configurations for 5+ exchanges
   - Supports different formats (env, TOML, JSON)
   - Validates API keys for each exchange
   - Shows connection status

2. **Dynamic Strategy Switcher**: Implement a system:
   - Loads strategy list from config
   - Allows enabling/disabling strategies without restart
   - Tracks configuration changes in real-time
   - Logs all configuration changes

3. **A/B Testing System**: Create a tool:
   - Distributes trades between strategy versions
   - Configured through feature flags
   - Collects metrics for each version
   - Generates comparison reports

4. **Configuration Migration**: Implement a utility:
   - Converts old config format to new
   - Validates compatibility
   - Creates backup of old configuration
   - Shows diff of changes

## Homework

1. **Configuration Engine**: Write a system that:
   - Supports configuration inheritance (base → environment)
   - Allows overriding settings via CLI arguments
   - Outputs final configuration with source of each value
   - Exports config to different formats
   - Has dry-run mode for verification without startup

2. **Secrets Manager**: Implement a system:
   - Encrypts local secrets with master password
   - Rotates API keys on schedule
   - Integrates with HashiCorp Vault (or its emulation)
   - Audit logs all secret accesses
   - Supports emergency key rotation

3. **Configuration Server**: Create a service:
   - Centrally stores configurations for all bots
   - Supports configuration versioning
   - Allows rollback of changes
   - Sends notifications on changes
   - REST API for management

4. **Configuration Testing**: Develop a framework:
   - Automatically tests all feature flag combinations
   - Checks environment compatibility
   - Finds conflicting settings
   - Generates compatibility matrix
   - Integrates with CI/CD

## Navigation

[← Day 326: Async vs Threading](../326-async-vs-threading/en.md) | [Day 337 →](../337-*/en.md)
