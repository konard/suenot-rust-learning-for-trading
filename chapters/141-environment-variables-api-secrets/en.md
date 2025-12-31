# Day 141: Environment Variables — API Secrets

## Trading Analogy

Imagine you have a safe deposit box at a bank. You don't carry the key around in plain sight — you keep it in a secure place. API keys for exchanges (Binance, Bybit, Coinbase) are your "keys to the vault" with money. Storing them directly in code is like writing your password on a sticky note and putting it on your monitor.

**Environment variables** are the secure way to store secrets: API keys, passwords, tokens. They live in the operating system, not in your code.

## Why This Is Critical

```rust
// ❌ NEVER DO THIS!
const API_KEY: &str = "sk-abc123xyz789";
const API_SECRET: &str = "super-secret-key";

fn main() {
    // If you push this code to GitHub —
    // bots will find the keys within seconds and drain your account!
}
```

**Real cases:**
- A trader lost $50,000 in 2 minutes after accidentally committing API keys
- Bots scan GitHub 24/7 looking for exchange keys
- Key leak = full access to your account

## Reading Environment Variables

### Basic Approach: std::env

```rust
use std::env;

fn main() {
    // Get exchange API key
    match env::var("BINANCE_API_KEY") {
        Ok(key) => println!("API Key loaded: {}...", &key[..8]),
        Err(_) => println!("BINANCE_API_KEY is not set!"),
    }
}
```

### Safe Presence Check

```rust
use std::env;

fn main() {
    // Check all required variables before starting the bot
    let required_vars = [
        "EXCHANGE_API_KEY",
        "EXCHANGE_API_SECRET",
        "TELEGRAM_BOT_TOKEN",
    ];

    let mut missing = Vec::new();

    for var in &required_vars {
        if env::var(var).is_err() {
            missing.push(*var);
        }
    }

    if !missing.is_empty() {
        println!("Error: missing environment variables:");
        for var in &missing {
            println!("  - {}", var);
        }
        std::process::exit(1);
    }

    println!("All variables loaded. Bot is ready.");
}
```

### Default Values

```rust
use std::env;

fn main() {
    // For non-critical settings, use default values
    let trading_mode = env::var("TRADING_MODE")
        .unwrap_or_else(|_| String::from("paper")); // paper trading by default

    let max_position_size: f64 = env::var("MAX_POSITION_SIZE")
        .unwrap_or_else(|_| String::from("1000.0"))
        .parse()
        .unwrap_or(1000.0);

    let log_level = env::var("LOG_LEVEL")
        .unwrap_or_else(|_| String::from("info"));

    println!("Mode: {}", trading_mode);
    println!("Max position: ${}", max_position_size);
    println!("Log level: {}", log_level);
}
```

## Trading Bot Configuration Structure

```rust
use std::env;

struct ExchangeConfig {
    api_key: String,
    api_secret: String,
    testnet: bool,
}

struct TradingConfig {
    max_position_usd: f64,
    risk_per_trade_percent: f64,
    allowed_symbols: Vec<String>,
}

struct BotConfig {
    exchange: ExchangeConfig,
    trading: TradingConfig,
    telegram_token: Option<String>,
}

impl BotConfig {
    fn from_env() -> Result<Self, String> {
        // Required variables
        let api_key = env::var("EXCHANGE_API_KEY")
            .map_err(|_| "EXCHANGE_API_KEY is not set")?;

        let api_secret = env::var("EXCHANGE_API_SECRET")
            .map_err(|_| "EXCHANGE_API_SECRET is not set")?;

        // Optional with defaults
        let testnet = env::var("USE_TESTNET")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true); // Testnet by default!

        let max_position_usd: f64 = env::var("MAX_POSITION_USD")
            .unwrap_or_else(|_| String::from("100.0"))
            .parse()
            .map_err(|_| "MAX_POSITION_USD must be a number")?;

        let risk_per_trade: f64 = env::var("RISK_PER_TRADE_PERCENT")
            .unwrap_or_else(|_| String::from("1.0"))
            .parse()
            .map_err(|_| "RISK_PER_TRADE_PERCENT must be a number")?;

        // Parse symbol list from environment variable
        let symbols_str = env::var("ALLOWED_SYMBOLS")
            .unwrap_or_else(|_| String::from("BTCUSDT,ETHUSDT"));
        let allowed_symbols: Vec<String> = symbols_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // Fully optional
        let telegram_token = env::var("TELEGRAM_BOT_TOKEN").ok();

        Ok(BotConfig {
            exchange: ExchangeConfig {
                api_key,
                api_secret,
                testnet,
            },
            trading: TradingConfig {
                max_position_usd,
                risk_per_trade_percent: risk_per_trade,
                allowed_symbols,
            },
            telegram_token,
        })
    }
}

fn main() {
    match BotConfig::from_env() {
        Ok(config) => {
            println!("Configuration loaded:");
            println!("  API Key: {}...", &config.exchange.api_key[..8.min(config.exchange.api_key.len())]);
            println!("  Testnet: {}", config.exchange.testnet);
            println!("  Max position: ${}", config.trading.max_position_usd);
            println!("  Risk per trade: {}%", config.trading.risk_per_trade_percent);
            println!("  Symbols: {:?}", config.trading.allowed_symbols);
            println!("  Telegram: {}", if config.telegram_token.is_some() { "configured" } else { "not configured" });
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    }
}
```

## The .env File and dotenv Library

In practice, variables are often stored in a `.env` file:

```bash
# .env file (DO NOT COMMIT TO GIT!)
EXCHANGE_API_KEY=your-api-key-here
EXCHANGE_API_SECRET=your-secret-here
USE_TESTNET=true
MAX_POSITION_USD=500.0
RISK_PER_TRADE_PERCENT=2.0
ALLOWED_SYMBOLS=BTCUSDT,ETHUSDT,SOLUSDT
```

```rust
// To use .env file, add to Cargo.toml:
// [dependencies]
// dotenv = "0.15"

use std::env;

fn load_dotenv() {
    // Simple .env loader implementation without external dependencies
    if let Ok(contents) = std::fs::read_to_string(".env") {
        for line in contents.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();

                // Set variable only if not already set
                if env::var(key).is_err() {
                    env::set_var(key, value);
                }
            }
        }
    }
}

fn main() {
    load_dotenv();

    // Now you can use variables from .env
    if let Ok(key) = env::var("EXCHANGE_API_KEY") {
        println!("Key loaded from .env");
    }
}
```

## Masking Secrets in Logs

```rust
use std::env;

/// Masks a secret string, showing only first and last characters
fn mask_secret(secret: &str, visible_chars: usize) -> String {
    if secret.len() <= visible_chars * 2 {
        return "*".repeat(secret.len());
    }

    let start = &secret[..visible_chars];
    let end = &secret[secret.len() - visible_chars..];
    let middle = "*".repeat(secret.len() - visible_chars * 2);

    format!("{}{}{}", start, middle, end)
}

/// Safe API configuration output
fn log_api_config(api_key: &str, api_secret: &str) {
    println!("API Configuration:");
    println!("  Key:    {}", mask_secret(api_key, 4));
    println!("  Secret: {}", mask_secret(api_secret, 2));
}

fn main() {
    let api_key = env::var("API_KEY")
        .unwrap_or_else(|_| String::from("test-key-12345678"));
    let api_secret = env::var("API_SECRET")
        .unwrap_or_else(|_| String::from("super-secret-password-123"));

    log_api_config(&api_key, &api_secret);

    // Masking examples
    println!("\nMasking examples:");
    println!("  'abcdefghij' -> '{}'", mask_secret("abcdefghij", 2));
    println!("  'short' -> '{}'", mask_secret("short", 2));
}
```

## API Key Validation

```rust
use std::env;

#[derive(Debug)]
enum ApiKeyError {
    Missing(String),
    TooShort { key_name: String, min_length: usize },
    InvalidFormat(String),
}

fn validate_binance_key(key: &str) -> Result<(), ApiKeyError> {
    // Binance API keys are typically 64 characters
    if key.len() < 64 {
        return Err(ApiKeyError::TooShort {
            key_name: String::from("BINANCE_API_KEY"),
            min_length: 64,
        });
    }

    // Check that key contains only valid characters
    if !key.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ApiKeyError::InvalidFormat(
            String::from("Key must contain only letters and numbers")
        ));
    }

    Ok(())
}

fn load_and_validate_api_keys() -> Result<(String, String), ApiKeyError> {
    let api_key = env::var("BINANCE_API_KEY")
        .map_err(|_| ApiKeyError::Missing(String::from("BINANCE_API_KEY")))?;

    let api_secret = env::var("BINANCE_API_SECRET")
        .map_err(|_| ApiKeyError::Missing(String::from("BINANCE_API_SECRET")))?;

    validate_binance_key(&api_key)?;
    validate_binance_key(&api_secret)?;

    Ok((api_key, api_secret))
}

fn main() {
    match load_and_validate_api_keys() {
        Ok((key, secret)) => {
            println!("API keys loaded and validated");
            println!("Key length: {}, Secret length: {}", key.len(), secret.len());
        }
        Err(ApiKeyError::Missing(name)) => {
            eprintln!("Error: variable {} is not set", name);
        }
        Err(ApiKeyError::TooShort { key_name, min_length }) => {
            eprintln!("Error: {} is too short (min {} characters)", key_name, min_length);
        }
        Err(ApiKeyError::InvalidFormat(msg)) => {
            eprintln!("Format error: {}", msg);
        }
    }
}
```

## Working with Different Environments

```rust
use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    fn from_env() -> Self {
        match env::var("RUST_ENV").as_deref() {
            Ok("production") | Ok("prod") => Environment::Production,
            Ok("staging") | Ok("stage") => Environment::Staging,
            _ => Environment::Development,
        }
    }

    fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    fn api_base_url(&self) -> &'static str {
        match self {
            Environment::Development => "https://testnet.binance.vision",
            Environment::Staging => "https://testnet.binance.vision",
            Environment::Production => "https://api.binance.com",
        }
    }

    fn default_max_position(&self) -> f64 {
        match self {
            Environment::Development => 100.0,    // $100 for development
            Environment::Staging => 1000.0,       // $1000 for staging
            Environment::Production => 10000.0,   // $10000 for production
        }
    }
}

fn main() {
    let env = Environment::from_env();

    println!("Current environment: {:?}", env);
    println!("API URL: {}", env.api_base_url());
    println!("Default max position: ${}", env.default_max_position());

    if env.is_production() {
        println!("\n⚠️  WARNING: Running in PRODUCTION mode!");
        println!("    All trades will be REAL!");
    }
}
```

## Practical Example: Complete Trading Bot Config

```rust
use std::env;
use std::collections::HashMap;

#[derive(Debug)]
struct TradingBotConfig {
    // Exchange
    exchange_name: String,
    api_key: String,
    api_secret: String,
    is_testnet: bool,

    // Trading
    symbols: Vec<String>,
    max_position_usd: f64,
    max_daily_trades: u32,
    risk_percent: f64,

    // Notifications
    telegram_chat_id: Option<String>,
    telegram_bot_token: Option<String>,

    // Additional settings
    extra: HashMap<String, String>,
}

impl TradingBotConfig {
    fn from_env() -> Result<Self, String> {
        // Required
        let exchange_name = env::var("EXCHANGE")
            .unwrap_or_else(|_| String::from("binance"));

        let api_key = env::var("API_KEY")
            .map_err(|_| "API_KEY is required")?;

        let api_secret = env::var("API_SECRET")
            .map_err(|_| "API_SECRET is required")?;

        // With defaults
        let is_testnet = env::var("TESTNET")
            .map(|v| v == "true" || v == "1" || v == "yes")
            .unwrap_or(true);

        let symbols: Vec<String> = env::var("TRADING_SYMBOLS")
            .unwrap_or_else(|_| String::from("BTCUSDT"))
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .collect();

        let max_position_usd: f64 = env::var("MAX_POSITION_USD")
            .unwrap_or_else(|_| String::from("100"))
            .parse()
            .unwrap_or(100.0);

        let max_daily_trades: u32 = env::var("MAX_DAILY_TRADES")
            .unwrap_or_else(|_| String::from("10"))
            .parse()
            .unwrap_or(10);

        let risk_percent: f64 = env::var("RISK_PERCENT")
            .unwrap_or_else(|_| String::from("1.0"))
            .parse()
            .unwrap_or(1.0)
            .min(5.0); // Max 5% risk

        // Optional
        let telegram_chat_id = env::var("TELEGRAM_CHAT_ID").ok();
        let telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN").ok();

        // Collect additional settings with BOT_ prefix
        let extra: HashMap<String, String> = env::vars()
            .filter(|(k, _)| k.starts_with("BOT_"))
            .map(|(k, v)| (k.strip_prefix("BOT_").unwrap().to_string(), v))
            .collect();

        Ok(TradingBotConfig {
            exchange_name,
            api_key,
            api_secret,
            is_testnet,
            symbols,
            max_position_usd,
            max_daily_trades,
            risk_percent,
            telegram_chat_id,
            telegram_bot_token,
            extra,
        })
    }

    fn print_summary(&self) {
        println!("╔═══════════════════════════════════════════╗");
        println!("║         TRADING BOT CONFIGURATION         ║");
        println!("╠═══════════════════════════════════════════╣");
        println!("║ Exchange:     {:>27} ║", self.exchange_name);
        println!("║ Mode:         {:>27} ║", if self.is_testnet { "TESTNET" } else { "PRODUCTION" });
        println!("║ API Key:      {:>27} ║", format!("{}...", &self.api_key[..8.min(self.api_key.len())]));
        println!("║ Symbols:      {:>27} ║", self.symbols.join(", "));
        println!("║ Max Position: {:>26}$ ║", self.max_position_usd);
        println!("║ Max Trades:   {:>27} ║", self.max_daily_trades);
        println!("║ Risk:         {:>26}% ║", self.risk_percent);
        println!("║ Telegram:     {:>27} ║",
            if self.telegram_bot_token.is_some() { "Enabled" } else { "Disabled" });
        println!("╚═══════════════════════════════════════════╝");

        if !self.extra.is_empty() {
            println!("\nAdditional settings:");
            for (key, value) in &self.extra {
                println!("  {}: {}", key, value);
            }
        }
    }
}

fn main() {
    println!("Loading configuration...\n");

    match TradingBotConfig::from_env() {
        Ok(config) => {
            config.print_summary();

            if !config.is_testnet {
                println!("\n🚨 WARNING: PRODUCTION MODE! 🚨");
                println!("All trades will be REAL!");
            }
        }
        Err(e) => {
            eprintln!("❌ Configuration load error: {}", e);
            eprintln!("\nExample environment variable setup:");
            eprintln!("  export API_KEY=\"your-api-key\"");
            eprintln!("  export API_SECRET=\"your-api-secret\"");
            eprintln!("  export TESTNET=true");
            std::process::exit(1);
        }
    }
}
```

## What We Learned

| Concept | Description | Example |
|---------|-------------|---------|
| `env::var()` | Read variable | `env::var("API_KEY")` |
| `unwrap_or_else` | Default value | `var.unwrap_or_else(\|_\| default)` |
| `.env` file | Local storage | Add to `.gitignore`! |
| Masking | Hide in logs | `key[..4] + "***"` |
| Validation | Format check | Length, characters, format |

## Security Rules

1. **NEVER** commit secrets to Git
2. Add `.env` to `.gitignore`
3. Use different keys for dev/prod
4. Mask secrets in logs
5. Default to testnet mode

## Homework

1. Write a function `load_multi_exchange_config()` that loads configuration for multiple exchanges (Binance, Bybit, Coinbase) from variables with prefixes

2. Create an API key validator that checks key formats for different exchanges (each has its own rules)

3. Implement a "secret vault" system with encryption: keys are stored encrypted, and a master password from an environment variable is needed for decryption

4. Write a security check utility that scans `.rs` project files for hardcoded secrets (strings that look like API keys)

## Navigation

[← Previous day](../140-file-output-trade-exports/en.md) | [Next day →](../142-file-reading-historical-data/en.md)
