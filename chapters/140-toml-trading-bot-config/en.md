# Day 140: TOML — Trading Bot Configuration

## Trading Analogy

When you set up a trading bot, you need to configure many parameters: API keys, exchange endpoints, trading pairs, position sizes, risk limits, and strategy parameters. Hardcoding these values directly in your code is dangerous — you'd have to recompile the bot every time you want to change a stop-loss or switch to a different exchange.

**TOML** (Tom's Obvious, Minimal Language) is like a **trading journal template** — a clean, human-readable format for storing configuration that both you and your bot can understand. Just like you might have a checklist before trading: "Check margin, set stop-loss at 2%, max position size is 0.5 BTC" — TOML lets you write these rules in a file that your bot reads at startup.

## What is TOML?

TOML is a configuration file format that's:
- **Human-readable** — easy to edit by hand
- **Strongly typed** — numbers are numbers, strings are strings
- **Hierarchical** — supports nested sections (tables)
- **Native to Rust** — Cargo uses TOML for `Cargo.toml`!

## Setting Up Dependencies

Add `toml` and `serde` to your `Cargo.toml`:

```toml
[dependencies]
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

## Basic TOML Syntax

```toml
# bot_config.toml - Trading Bot Configuration

# Simple key-value pairs
bot_name = "AlphaTrader"
version = "1.0.0"
debug_mode = false

# Numbers
max_position_usd = 10000.0
max_open_trades = 5
risk_percent = 2.0

# Dates (TOML supports ISO 8601)
start_date = 2024-01-01
```

## Parsing Simple Configuration

```rust
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct BotConfig {
    bot_name: String,
    version: String,
    debug_mode: bool,
    max_position_usd: f64,
    max_open_trades: u32,
    risk_percent: f64,
}

fn main() {
    let config_str = r#"
        bot_name = "AlphaTrader"
        version = "1.0.0"
        debug_mode = false
        max_position_usd = 10000.0
        max_open_trades = 5
        risk_percent = 2.0
    "#;

    let config: BotConfig = toml::from_str(config_str).unwrap();

    println!("╔══════════════════════════════════════╗");
    println!("║         BOT CONFIGURATION            ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ Name:           {:>18} ║", config.bot_name);
    println!("║ Version:        {:>18} ║", config.version);
    println!("║ Debug:          {:>18} ║", config.debug_mode);
    println!("║ Max Position:   ${:>16.2} ║", config.max_position_usd);
    println!("║ Max Trades:     {:>18} ║", config.max_open_trades);
    println!("║ Risk %:         {:>17.1}% ║", config.risk_percent);
    println!("╚══════════════════════════════════════╝");
}
```

## TOML Tables — Nested Configuration

Tables in TOML create nested structures, perfect for organizing exchange settings:

```toml
# config.toml

[bot]
name = "AlphaTrader"
version = "1.0.0"

[exchange]
name = "binance"
base_url = "https://api.binance.com"
ws_url = "wss://stream.binance.com:9443"
rate_limit_per_minute = 1200

[trading]
symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
max_position_usd = 10000.0
max_open_trades = 5

[risk]
max_daily_loss_percent = 5.0
max_drawdown_percent = 10.0
stop_loss_percent = 2.0
take_profit_percent = 4.0
```

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    bot: BotSection,
    exchange: ExchangeSection,
    trading: TradingSection,
    risk: RiskSection,
}

#[derive(Debug, Deserialize)]
struct BotSection {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct ExchangeSection {
    name: String,
    base_url: String,
    ws_url: String,
    rate_limit_per_minute: u32,
}

#[derive(Debug, Deserialize)]
struct TradingSection {
    symbols: Vec<String>,
    max_position_usd: f64,
    max_open_trades: u32,
}

#[derive(Debug, Deserialize)]
struct RiskSection {
    max_daily_loss_percent: f64,
    max_drawdown_percent: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
}

fn main() {
    let config_str = r#"
        [bot]
        name = "AlphaTrader"
        version = "1.0.0"

        [exchange]
        name = "binance"
        base_url = "https://api.binance.com"
        ws_url = "wss://stream.binance.com:9443"
        rate_limit_per_minute = 1200

        [trading]
        symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
        max_position_usd = 10000.0
        max_open_trades = 5

        [risk]
        max_daily_loss_percent = 5.0
        max_drawdown_percent = 10.0
        stop_loss_percent = 2.0
        take_profit_percent = 4.0
    "#;

    let config: Config = toml::from_str(config_str).unwrap();

    println!("Bot: {} v{}", config.bot.name, config.bot.version);
    println!("Exchange: {}", config.exchange.name);
    println!("Trading pairs: {:?}", config.trading.symbols);
    println!("Stop Loss: {}%", config.risk.stop_loss_percent);
}
```

## Strategy Configuration with Inline Tables

Inline tables are great for compact, related data:

```toml
# strategies.toml

[strategy]
name = "momentum"
timeframe = "1h"
enabled = true

# Inline tables for indicator parameters
[strategy.indicators]
sma_fast = { period = 10, source = "close" }
sma_slow = { period = 50, source = "close" }
rsi = { period = 14, overbought = 70, oversold = 30 }
```

```rust
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct StrategyConfig {
    strategy: Strategy,
}

#[derive(Debug, Deserialize)]
struct Strategy {
    name: String,
    timeframe: String,
    enabled: bool,
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    sma_fast: SmaConfig,
    sma_slow: SmaConfig,
    rsi: RsiConfig,
}

#[derive(Debug, Deserialize)]
struct SmaConfig {
    period: u32,
    source: String,
}

#[derive(Debug, Deserialize)]
struct RsiConfig {
    period: u32,
    overbought: u32,
    oversold: u32,
}

fn main() {
    let config_str = r#"
        [strategy]
        name = "momentum"
        timeframe = "1h"
        enabled = true

        [strategy.indicators]
        sma_fast = { period = 10, source = "close" }
        sma_slow = { period = 50, source = "close" }
        rsi = { period = 14, overbought = 70, oversold = 30 }
    "#;

    let config: StrategyConfig = toml::from_str(config_str).unwrap();
    let strategy = &config.strategy;

    println!("Strategy: {} ({})", strategy.name, strategy.timeframe);
    println!("SMA Fast: {} periods", strategy.indicators.sma_fast.period);
    println!("SMA Slow: {} periods", strategy.indicators.sma_slow.period);
    println!("RSI: {} (OB: {}, OS: {})",
        strategy.indicators.rsi.period,
        strategy.indicators.rsi.overbought,
        strategy.indicators.rsi.oversold
    );
}
```

## Array of Tables — Multiple Trading Pairs

Use `[[array_name]]` for arrays of tables — perfect for multiple trading pairs:

```toml
# pairs.toml

[[pairs]]
symbol = "BTCUSDT"
enabled = true
max_position = 0.5
stop_loss = 2.0
take_profit = 4.0

[[pairs]]
symbol = "ETHUSDT"
enabled = true
max_position = 5.0
stop_loss = 3.0
take_profit = 6.0

[[pairs]]
symbol = "SOLUSDT"
enabled = false
max_position = 100.0
stop_loss = 5.0
take_profit = 10.0
```

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PairsConfig {
    pairs: Vec<TradingPair>,
}

#[derive(Debug, Deserialize)]
struct TradingPair {
    symbol: String,
    enabled: bool,
    max_position: f64,
    stop_loss: f64,
    take_profit: f64,
}

fn main() {
    let config_str = r#"
        [[pairs]]
        symbol = "BTCUSDT"
        enabled = true
        max_position = 0.5
        stop_loss = 2.0
        take_profit = 4.0

        [[pairs]]
        symbol = "ETHUSDT"
        enabled = true
        max_position = 5.0
        stop_loss = 3.0
        take_profit = 6.0

        [[pairs]]
        symbol = "SOLUSDT"
        enabled = false
        max_position = 100.0
        stop_loss = 5.0
        take_profit = 10.0
    "#;

    let config: PairsConfig = toml::from_str(config_str).unwrap();

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║              TRADING PAIRS CONFIGURATION               ║");
    println!("╠══════════╦═════════╦════════════╦══════════╦═══════════╣");
    println!("║  Symbol  ║ Enabled ║  Max Pos   ║ Stop Loss║Take Profit║");
    println!("╠══════════╬═════════╬════════════╬══════════╬═══════════╣");

    for pair in &config.pairs {
        let enabled = if pair.enabled { "Yes" } else { "No" };
        println!("║ {:>8} ║ {:>7} ║ {:>10.2} ║ {:>7.1}% ║ {:>8.1}% ║",
            pair.symbol, enabled, pair.max_position,
            pair.stop_loss, pair.take_profit);
    }
    println!("╚══════════╩═════════╩════════════╩══════════╩═══════════╝");

    // Filter only enabled pairs
    let active_pairs: Vec<_> = config.pairs
        .iter()
        .filter(|p| p.enabled)
        .collect();

    println!("\nActive pairs: {}", active_pairs.len());
}
```

## Optional Fields with Default Values

Use `Option<T>` and `#[serde(default)]` for optional configuration:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BotConfig {
    name: String,

    // Optional field - will be None if not present
    description: Option<String>,

    // Default value if not present
    #[serde(default = "default_max_retries")]
    max_retries: u32,

    // Default to false if not present
    #[serde(default)]
    paper_trading: bool,

    // Default empty vector
    #[serde(default)]
    notification_emails: Vec<String>,
}

fn default_max_retries() -> u32 {
    3
}

fn main() {
    // Minimal config - missing optional fields
    let minimal = r#"
        name = "SimpleBot"
    "#;

    let config: BotConfig = toml::from_str(minimal).unwrap();

    println!("Name: {}", config.name);
    println!("Description: {:?}", config.description);
    println!("Max retries: {}", config.max_retries);
    println!("Paper trading: {}", config.paper_trading);
    println!("Emails: {:?}", config.notification_emails);

    // Full config
    let full = r#"
        name = "AdvancedBot"
        description = "Multi-strategy trading bot"
        max_retries = 5
        paper_trading = true
        notification_emails = ["trader@example.com", "alerts@example.com"]
    "#;

    let config: BotConfig = toml::from_str(full).unwrap();
    println!("\n--- Full Config ---");
    println!("{:#?}", config);
}
```

## Writing Configuration to TOML

Save your bot's state or generate config files:

```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Portfolio {
    last_updated: String,
    total_value_usd: f64,
    positions: Vec<Position>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

fn main() {
    let portfolio = Portfolio {
        last_updated: "2024-01-15T10:30:00Z".to_string(),
        total_value_usd: 52450.0,
        positions: vec![
            Position {
                symbol: "BTC".to_string(),
                quantity: 0.5,
                entry_price: 42000.0,
                current_price: 43500.0,
            },
            Position {
                symbol: "ETH".to_string(),
                quantity: 10.0,
                entry_price: 2200.0,
                current_price: 2350.0,
            },
        ],
    };

    // Serialize to TOML string
    let toml_string = toml::to_string_pretty(&portfolio).unwrap();
    println!("Generated TOML:\n{}", toml_string);

    // Save to file
    // fs::write("portfolio.toml", &toml_string).unwrap();
}
```

Output:
```toml
last_updated = "2024-01-15T10:30:00Z"
total_value_usd = 52450.0

[[positions]]
symbol = "BTC"
quantity = 0.5
entry_price = 42000.0
current_price = 43500.0

[[positions]]
symbol = "ETH"
quantity = 10.0
entry_price = 2200.0
current_price = 2350.0
```

## Error Handling for Configuration

Always validate your configuration:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RiskConfig {
    max_position_usd: f64,
    stop_loss_percent: f64,
    max_daily_loss_percent: f64,
}

impl RiskConfig {
    fn validate(&self) -> Result<(), String> {
        if self.max_position_usd <= 0.0 {
            return Err("max_position_usd must be positive".to_string());
        }
        if self.stop_loss_percent <= 0.0 || self.stop_loss_percent > 100.0 {
            return Err("stop_loss_percent must be between 0 and 100".to_string());
        }
        if self.max_daily_loss_percent <= 0.0 || self.max_daily_loss_percent > 100.0 {
            return Err("max_daily_loss_percent must be between 0 and 100".to_string());
        }
        Ok(())
    }
}

fn load_config(toml_str: &str) -> Result<RiskConfig, String> {
    // Parse TOML
    let config: RiskConfig = toml::from_str(toml_str)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Validate values
    config.validate()?;

    Ok(config)
}

fn main() {
    // Valid config
    let valid = r#"
        max_position_usd = 10000.0
        stop_loss_percent = 2.0
        max_daily_loss_percent = 5.0
    "#;

    match load_config(valid) {
        Ok(config) => println!("Config loaded: {:?}", config),
        Err(e) => println!("Error: {}", e),
    }

    // Invalid config - negative stop loss
    let invalid = r#"
        max_position_usd = 10000.0
        stop_loss_percent = -2.0
        max_daily_loss_percent = 5.0
    "#;

    match load_config(invalid) {
        Ok(config) => println!("Config loaded: {:?}", config),
        Err(e) => println!("Validation error: {}", e),
    }

    // Parse error - wrong type
    let parse_error = r#"
        max_position_usd = "not a number"
        stop_loss_percent = 2.0
        max_daily_loss_percent = 5.0
    "#;

    match load_config(parse_error) {
        Ok(config) => println!("Config loaded: {:?}", config),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

## Practical Example: Complete Trading Bot Configuration

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct TradingBotConfig {
    bot: BotInfo,
    exchange: ExchangeConfig,
    trading: TradingConfig,
    risk: RiskManagement,
    strategies: Vec<StrategyConfig>,
    notifications: NotificationConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct BotInfo {
    name: String,
    version: String,
    #[serde(default)]
    paper_trading: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExchangeConfig {
    name: String,
    api_key_env: String,  // Environment variable name, not actual key!
    secret_key_env: String,
    testnet: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct TradingConfig {
    symbols: Vec<String>,
    base_currency: String,
    timeframe: String,
    max_open_positions: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct RiskManagement {
    max_position_percent: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
    max_daily_loss_percent: f64,
    max_drawdown_percent: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct StrategyConfig {
    name: String,
    enabled: bool,
    weight: f64,
    parameters: StrategyParameters,
}

#[derive(Debug, Deserialize, Serialize)]
struct StrategyParameters {
    fast_period: Option<u32>,
    slow_period: Option<u32>,
    signal_period: Option<u32>,
    rsi_period: Option<u32>,
    rsi_overbought: Option<f64>,
    rsi_oversold: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NotificationConfig {
    #[serde(default)]
    telegram_enabled: bool,
    telegram_bot_token_env: Option<String>,
    telegram_chat_id: Option<String>,
    #[serde(default)]
    email_enabled: bool,
    email_addresses: Vec<String>,
}

fn main() {
    let config_str = r#"
        [bot]
        name = "AlphaTrader Pro"
        version = "2.0.0"
        paper_trading = true

        [exchange]
        name = "binance"
        api_key_env = "BINANCE_API_KEY"
        secret_key_env = "BINANCE_SECRET_KEY"
        testnet = true

        [trading]
        symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"]
        base_currency = "USDT"
        timeframe = "4h"
        max_open_positions = 3

        [risk]
        max_position_percent = 10.0
        stop_loss_percent = 2.0
        take_profit_percent = 6.0
        max_daily_loss_percent = 5.0
        max_drawdown_percent = 15.0

        [[strategies]]
        name = "macd_crossover"
        enabled = true
        weight = 0.6
        [strategies.parameters]
        fast_period = 12
        slow_period = 26
        signal_period = 9

        [[strategies]]
        name = "rsi_reversal"
        enabled = true
        weight = 0.4
        [strategies.parameters]
        rsi_period = 14
        rsi_overbought = 70.0
        rsi_oversold = 30.0

        [notifications]
        telegram_enabled = true
        telegram_bot_token_env = "TELEGRAM_BOT_TOKEN"
        telegram_chat_id = "123456789"
        email_enabled = false
        email_addresses = []
    "#;

    let config: TradingBotConfig = toml::from_str(config_str).unwrap();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║             TRADING BOT CONFIGURATION                    ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    println!("\n📊 Bot Info:");
    println!("   Name: {} v{}", config.bot.name, config.bot.version);
    println!("   Mode: {}", if config.bot.paper_trading { "Paper Trading" } else { "Live Trading" });

    println!("\n🏦 Exchange:");
    println!("   Name: {}", config.exchange.name);
    println!("   Testnet: {}", config.exchange.testnet);

    println!("\n📈 Trading:");
    println!("   Symbols: {:?}", config.trading.symbols);
    println!("   Timeframe: {}", config.trading.timeframe);
    println!("   Max positions: {}", config.trading.max_open_positions);

    println!("\n⚠️  Risk Management:");
    println!("   Max position: {}%", config.risk.max_position_percent);
    println!("   Stop loss: {}%", config.risk.stop_loss_percent);
    println!("   Take profit: {}%", config.risk.take_profit_percent);
    println!("   Max daily loss: {}%", config.risk.max_daily_loss_percent);

    println!("\n🎯 Strategies:");
    for strategy in &config.strategies {
        let status = if strategy.enabled { "✓" } else { "✗" };
        println!("   {} {} (weight: {:.0}%)", status, strategy.name, strategy.weight * 100.0);
    }

    println!("\n🔔 Notifications:");
    println!("   Telegram: {}", if config.notifications.telegram_enabled { "Enabled" } else { "Disabled" });
    println!("   Email: {}", if config.notifications.email_enabled { "Enabled" } else { "Disabled" });
}
```

## What We Learned

| TOML Feature | Syntax | Use Case |
|--------------|--------|----------|
| Key-value | `key = "value"` | Simple settings |
| Table | `[section]` | Grouped settings |
| Nested table | `[section.subsection]` | Hierarchical config |
| Inline table | `{ key = "value" }` | Compact structures |
| Array | `[1, 2, 3]` | Lists of values |
| Array of tables | `[[items]]` | Multiple similar objects |
| Comments | `# comment` | Documentation |

## Best Practices

1. **Never store secrets in TOML files** — use environment variable names instead
2. **Validate configuration at startup** — catch errors early
3. **Use defaults for optional settings** — `#[serde(default)]`
4. **Document your config format** — add comments explaining each option
5. **Version your config format** — include a version field for migrations

## Homework

1. Create a TOML config for a multi-exchange trading bot that supports Binance, Bybit, and OKX with different settings for each

2. Write a configuration validator that checks:
   - Stop loss is less than take profit
   - Total strategy weights sum to 1.0
   - At least one trading symbol is configured

3. Implement a config merger that takes a default config and overlays user overrides

4. Create a TOML config for backtesting with:
   - Date range for historical data
   - List of strategies to test
   - Performance metrics to calculate

## Navigation

[← Previous day](../139-time-formatting/en.md) | [Next day →](../141-environment-variables-api-secrets/en.md)
