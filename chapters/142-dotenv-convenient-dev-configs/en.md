# Day 142: dotenv: Convenient Dev Configs

## Trading Analogy

Imagine every trader in the office has a personal notebook with secrets: exchange API keys, passwords, position limits. When the trader sits down at their workstation, they pull out this notebook and configure their terminal. A `.env` file is like that "notebook" for your trading bot: all secrets and settings in one place, but not in the code.

## Why dotenv?

In the previous chapter, we learned about environment variables. But typing them every time in the terminal is inconvenient:

```bash
BINANCE_API_KEY=abc123 BINANCE_SECRET=xyz789 RISK_PERCENT=2 cargo run
```

A `.env` file lets you write all variables once and automatically load them at startup.

## Adding dotenv

Add the dependency to `Cargo.toml`:

```toml
[dependencies]
dotenvy = "0.15"
```

> **Note:** We use `dotenvy` — an actively maintained fork of the original `dotenv` library.

## Creating the .env File

Create a `.env` file in your project root:

```env
# Exchange connection settings
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET=your_secret_here

# Risk management settings
MAX_POSITION_SIZE=1000.0
RISK_PERCENT=2.0
MAX_DAILY_LOSS=500.0

# Strategy settings
SMA_FAST_PERIOD=10
SMA_SLOW_PERIOD=20
RSI_OVERSOLD=30
RSI_OVERBOUGHT=70

# Operating mode
TRADING_MODE=paper
LOG_LEVEL=debug
```

**Important:** Add `.env` to `.gitignore`! Secrets should never be committed to Git.

```gitignore
.env
.env.local
.env.*.local
```

## Basic Usage

```rust
use std::env;

fn main() {
    // Load variables from .env
    dotenvy::dotenv().ok();

    // Read trading settings
    let api_key = env::var("BINANCE_API_KEY")
        .expect("BINANCE_API_KEY must be set");

    let risk_percent: f64 = env::var("RISK_PERCENT")
        .unwrap_or_else(|_| "1.0".to_string())
        .parse()
        .expect("RISK_PERCENT must be a valid number");

    println!("API Key: {}...", &api_key[..8.min(api_key.len())]);
    println!("Risk: {}%", risk_percent);
}
```

## Configuration Structure

For convenience, let's create a structure holding all settings:

```rust
use std::env;

#[derive(Debug)]
struct TradingConfig {
    // API credentials
    api_key: String,
    api_secret: String,

    // Risk management
    max_position_size: f64,
    risk_percent: f64,
    max_daily_loss: f64,

    // Strategy parameters
    sma_fast: usize,
    sma_slow: usize,
    rsi_oversold: f64,
    rsi_overbought: f64,

    // Mode
    is_paper_trading: bool,
}

impl TradingConfig {
    fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        Ok(TradingConfig {
            api_key: env::var("BINANCE_API_KEY")
                .map_err(|_| "BINANCE_API_KEY not set")?,

            api_secret: env::var("BINANCE_SECRET")
                .map_err(|_| "BINANCE_SECRET not set")?,

            max_position_size: env::var("MAX_POSITION_SIZE")
                .unwrap_or_else(|_| "1000.0".to_string())
                .parse()
                .map_err(|_| "Invalid MAX_POSITION_SIZE")?,

            risk_percent: env::var("RISK_PERCENT")
                .unwrap_or_else(|_| "2.0".to_string())
                .parse()
                .map_err(|_| "Invalid RISK_PERCENT")?,

            max_daily_loss: env::var("MAX_DAILY_LOSS")
                .unwrap_or_else(|_| "500.0".to_string())
                .parse()
                .map_err(|_| "Invalid MAX_DAILY_LOSS")?,

            sma_fast: env::var("SMA_FAST_PERIOD")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| "Invalid SMA_FAST_PERIOD")?,

            sma_slow: env::var("SMA_SLOW_PERIOD")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .map_err(|_| "Invalid SMA_SLOW_PERIOD")?,

            rsi_oversold: env::var("RSI_OVERSOLD")
                .unwrap_or_else(|_| "30.0".to_string())
                .parse()
                .map_err(|_| "Invalid RSI_OVERSOLD")?,

            rsi_overbought: env::var("RSI_OVERBOUGHT")
                .unwrap_or_else(|_| "70.0".to_string())
                .parse()
                .map_err(|_| "Invalid RSI_OVERBOUGHT")?,

            is_paper_trading: env::var("TRADING_MODE")
                .unwrap_or_else(|_| "paper".to_string())
                .to_lowercase() == "paper",
        })
    }
}

fn main() {
    match TradingConfig::from_env() {
        Ok(config) => {
            println!("Trading Configuration Loaded:");
            println!("  Paper trading: {}", config.is_paper_trading);
            println!("  Max position: ${:.2}", config.max_position_size);
            println!("  Risk: {}%", config.risk_percent);
            println!("  SMA periods: {}/{}", config.sma_fast, config.sma_slow);
            println!("  RSI levels: {}/{}", config.rsi_oversold, config.rsi_overbought);
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    }
}
```

## Different Files for Different Environments

Create separate files for different environments:

```
.env                # Default values (can be committed)
.env.development    # For development
.env.production     # For production
.env.local          # Local overrides (don't commit!)
```

```rust
use std::env;

fn load_environment_config() {
    // First load base .env
    dotenvy::dotenv().ok();

    // Determine current environment
    let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    // Load environment-specific file
    let env_file = format!(".env.{}", environment);
    dotenvy::from_filename(&env_file).ok();

    // Local overrides have highest priority
    dotenvy::from_filename(".env.local").ok();

    println!("Loaded configuration for: {}", environment);
}

fn main() {
    load_environment_config();

    let trading_mode = env::var("TRADING_MODE").unwrap_or_else(|_| "paper".to_string());
    println!("Trading mode: {}", trading_mode);
}
```

## Validating Configuration at Startup

```rust
use std::env;

struct ExchangeConfig {
    api_key: String,
    api_secret: String,
    base_url: String,
}

struct RiskConfig {
    max_position_usd: f64,
    max_loss_percent: f64,
    max_orders_per_minute: u32,
}

fn validate_config() -> Result<(ExchangeConfig, RiskConfig), Vec<String>> {
    dotenvy::dotenv().ok();

    let mut errors = Vec::new();

    // Check required fields
    let api_key = env::var("API_KEY").ok();
    let api_secret = env::var("API_SECRET").ok();

    if api_key.is_none() {
        errors.push("API_KEY is required".to_string());
    }
    if api_secret.is_none() {
        errors.push("API_SECRET is required".to_string());
    }

    // Check numeric values
    let max_position: Result<f64, _> = env::var("MAX_POSITION_USD")
        .unwrap_or_else(|_| "10000.0".to_string())
        .parse();

    if let Err(_) = max_position {
        errors.push("MAX_POSITION_USD must be a valid number".to_string());
    }

    let max_loss: Result<f64, _> = env::var("MAX_LOSS_PERCENT")
        .unwrap_or_else(|_| "5.0".to_string())
        .parse();

    if let Err(_) = max_loss {
        errors.push("MAX_LOSS_PERCENT must be a valid number".to_string());
    }

    // Check valid ranges
    if let Ok(loss) = max_loss {
        if loss < 0.0 || loss > 100.0 {
            errors.push("MAX_LOSS_PERCENT must be between 0 and 100".to_string());
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok((
        ExchangeConfig {
            api_key: api_key.unwrap(),
            api_secret: api_secret.unwrap(),
            base_url: env::var("API_BASE_URL")
                .unwrap_or_else(|_| "https://api.exchange.com".to_string()),
        },
        RiskConfig {
            max_position_usd: max_position.unwrap(),
            max_loss_percent: max_loss.unwrap(),
            max_orders_per_minute: env::var("MAX_ORDERS_PER_MINUTE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        },
    ))
}

fn main() {
    println!("Validating trading bot configuration...\n");

    match validate_config() {
        Ok((exchange, risk)) => {
            println!("Configuration valid!");
            println!("Exchange URL: {}", exchange.base_url);
            println!("Max position: ${:.2}", risk.max_position_usd);
            println!("Max loss: {:.1}%", risk.max_loss_percent);
            println!("Rate limit: {} orders/min", risk.max_orders_per_minute);
        }
        Err(errors) => {
            eprintln!("Configuration errors found:");
            for error in errors {
                eprintln!("  - {}", error);
            }
            std::process::exit(1);
        }
    }
}
```

## Example: Trading Bot Configuration File

Example of a complete `.env` file for a trading bot:

```env
# ╔════════════════════════════════════════════════════════════════╗
# ║               TRADING BOT CONFIGURATION                        ║
# ╚════════════════════════════════════════════════════════════════╝

# ── Environment ──────────────────────────────────────────────────
APP_ENV=development
LOG_LEVEL=debug

# ── Exchange API ─────────────────────────────────────────────────
EXCHANGE=binance
API_KEY=your_api_key_here
API_SECRET=your_secret_key_here
API_PASSPHRASE=                     # For exchanges that require it

# ── Trading Mode ─────────────────────────────────────────────────
TRADING_MODE=paper                  # paper | live
DRY_RUN=true                        # true = log orders, don't execute

# ── Risk Management ──────────────────────────────────────────────
MAX_POSITION_SIZE_USD=1000.0        # Maximum single position
RISK_PER_TRADE_PERCENT=2.0          # % of capital per trade
MAX_DAILY_LOSS_USD=500.0            # Stop trading after this loss
MAX_OPEN_POSITIONS=5                # Maximum concurrent positions

# ── Strategy: SMA Crossover ──────────────────────────────────────
STRATEGY=sma_crossover
SMA_FAST_PERIOD=10
SMA_SLOW_PERIOD=20
MIN_VOLUME_USD=100000               # Minimum 24h volume to trade

# ── Strategy: RSI ────────────────────────────────────────────────
RSI_PERIOD=14
RSI_OVERSOLD=30
RSI_OVERBOUGHT=70

# ── Symbols ──────────────────────────────────────────────────────
TRADING_SYMBOLS=BTC/USDT,ETH/USDT,SOL/USDT
QUOTE_CURRENCY=USDT

# ── Execution ────────────────────────────────────────────────────
ORDER_TYPE=limit                    # limit | market
SLIPPAGE_TOLERANCE_PERCENT=0.5
MAX_RETRIES=3
RETRY_DELAY_MS=1000

# ── Notifications ────────────────────────────────────────────────
TELEGRAM_BOT_TOKEN=
TELEGRAM_CHAT_ID=
NOTIFY_ON_TRADE=true
NOTIFY_ON_ERROR=true

# ── Database ─────────────────────────────────────────────────────
DATABASE_URL=sqlite:./trading_bot.db
```

## Security

### Never Commit Secrets!

```rust
use std::env;

fn mask_secret(secret: &str) -> String {
    if secret.len() <= 4 {
        "*".repeat(secret.len())
    } else {
        format!("{}...{}", &secret[..2], &secret[secret.len()-2..])
    }
}

fn print_safe_config() {
    dotenvy::dotenv().ok();

    let api_key = env::var("API_KEY").unwrap_or_else(|_| "NOT_SET".to_string());
    let api_secret = env::var("API_SECRET").unwrap_or_else(|_| "NOT_SET".to_string());

    println!("API Key: {}", mask_secret(&api_key));
    println!("API Secret: {}", mask_secret(&api_secret));
}

fn main() {
    print_safe_config();
}
```

### Create a .env.example Template

Create a `.env.example` file with empty values for documentation:

```env
# Copy this file to .env and fill in your values
API_KEY=
API_SECRET=
TRADING_MODE=paper
RISK_PERCENT=2.0
```

## Practical Example: Multi-Exchange Config

```rust
use std::env;
use std::collections::HashMap;

#[derive(Debug)]
struct ExchangeCredentials {
    name: String,
    api_key: String,
    api_secret: String,
    is_testnet: bool,
}

fn load_exchange_configs() -> Vec<ExchangeCredentials> {
    dotenvy::dotenv().ok();

    let exchanges = ["BINANCE", "BYBIT", "OKX"];
    let mut configs = Vec::new();

    for exchange in exchanges {
        let key_var = format!("{}_API_KEY", exchange);
        let secret_var = format!("{}_API_SECRET", exchange);
        let testnet_var = format!("{}_TESTNET", exchange);

        if let (Ok(key), Ok(secret)) = (env::var(&key_var), env::var(&secret_var)) {
            configs.push(ExchangeCredentials {
                name: exchange.to_lowercase(),
                api_key: key,
                api_secret: secret,
                is_testnet: env::var(&testnet_var)
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(true),
            });
        }
    }

    configs
}

fn main() {
    let exchanges = load_exchange_configs();

    println!("Loaded {} exchange configurations:", exchanges.len());
    for ex in &exchanges {
        println!(
            "  {} ({}): API key {}...{}",
            ex.name,
            if ex.is_testnet { "testnet" } else { "mainnet" },
            &ex.api_key[..4],
            &ex.api_key[ex.api_key.len()-4..]
        );
    }
}
```

Example `.env` file for this:

```env
# Binance
BINANCE_API_KEY=abc123...
BINANCE_API_SECRET=xyz789...
BINANCE_TESTNET=true

# Bybit
BYBIT_API_KEY=def456...
BYBIT_API_SECRET=uvw012...
BYBIT_TESTNET=true

# OKX (not configured yet, will be skipped)
# OKX_API_KEY=
# OKX_API_SECRET=
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `.env` file | File with environment variables |
| `dotenvy` | Library for loading `.env` |
| `.env.example` | Template for documentation |
| Different environments | `.env.development`, `.env.production` |
| Validation | Checking config at startup |
| Security | Don't commit secrets, mask output |

## Practical Exercises

1. **Position Config:** Create a `.env` file with settings for position size calculation (balance, risk, commission) and a program that loads and validates them.

2. **Multi-Environment:** Create three files `.env.development`, `.env.staging`, `.env.production` with different settings and a program that loads the right one based on `APP_ENV` variable.

3. **Safe Output:** Write a `print_config_summary()` function that prints all settings but masks secret values (API keys, passwords).

## Homework

1. Create a complete configuration file for a trading bot with all necessary parameters: exchange, strategy, risk management, notifications.

2. Write a `TradingBotConfig` struct with a `from_env()` method that loads all parameters from `.env` with validation and default values.

3. Implement a `validate_trading_config()` function that checks:
   - All required fields are filled
   - Numeric values are within valid ranges
   - API keys have proper format
   - Returns a list of all found errors

4. Create a module for working with multiple exchange configs simultaneously, with the ability to switch between them.

## Navigation

[← Previous day](../141-environment-variables-api-secrets/en.md) | [Next day →](../143-command-line-arguments-clap/en.md)
