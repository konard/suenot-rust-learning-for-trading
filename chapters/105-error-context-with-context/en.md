# Day 105: Error Context — with_context()

## Trading Analogy

Imagine receiving a message from your broker: "Order execution failed". This is not helpful! Now imagine a different message: "Order execution failed for BUY 0.5 BTC @ $42,000 on Binance exchange: insufficient funds, balance $15,000". The second message contains **context** — all the information needed to understand and fix the problem.

The `with_context()` method in Rust does exactly this — it adds context to errors, transforming cryptic messages into informative ones.

## Why Do We Need Error Context?

```rust
// Bad: error without context
Err("Failed to parse price")

// Good: error with context
Err("Failed to parse price for BTCUSDT from API response: invalid format 'N/A'")
```

Without context:
- Unclear which asset caused the problem
- Unknown data source
- No debugging information

With context:
- Know exactly where the error occurred
- See the input data
- Can quickly fix the issue

## The anyhow Library

`with_context()` is a method from the popular `anyhow` library, which simplifies error handling:

```toml
# Cargo.toml
[dependencies]
anyhow = "1.0"
```

## Basic Usage of with_context()

```rust
use anyhow::{Context, Result};
use std::fs;

fn main() -> Result<()> {
    let config = load_trading_config("config.json")?;
    println!("Loaded config: {:?}", config);
    Ok(())
}

fn load_trading_config(path: &str) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to load trading config from '{}'", path))
}
```

Without context, the error looks like this:
```
Error: No such file or directory (os error 2)
```

With context:
```
Error: Failed to load trading config from 'config.json'

Caused by:
    No such file or directory (os error 2)
```

## Trading Examples

### 1. Parsing Market Data

```rust
use anyhow::{Context, Result};

#[derive(Debug)]
struct MarketData {
    symbol: String,
    price: f64,
    volume: f64,
}

fn parse_market_data(symbol: &str, data: &str) -> Result<MarketData> {
    let parts: Vec<&str> = data.split(',').collect();

    if parts.len() < 2 {
        anyhow::bail!("Invalid data format for {}: expected 'price,volume'", symbol);
    }

    let price: f64 = parts[0]
        .trim()
        .parse()
        .with_context(|| format!(
            "Failed to parse price for {}: invalid value '{}'",
            symbol, parts[0]
        ))?;

    let volume: f64 = parts[1]
        .trim()
        .parse()
        .with_context(|| format!(
            "Failed to parse volume for {}: invalid value '{}'",
            symbol, parts[1]
        ))?;

    Ok(MarketData {
        symbol: symbol.to_string(),
        price,
        volume,
    })
}

fn main() -> Result<()> {
    // Successful parsing
    let btc = parse_market_data("BTCUSDT", "42000.50,1234.5")?;
    println!("BTC: {:?}", btc);

    // Error with context
    let invalid = parse_market_data("ETHUSDT", "N/A,500.0");
    if let Err(e) = invalid {
        println!("Error: {:?}", e);
    }

    Ok(())
}
```

### 2. Order Validation

```rust
use anyhow::{Context, Result, ensure};

#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn validate_order(order: &Order, balance: f64, min_order_size: f64) -> Result<()> {
    // Check symbol
    ensure!(
        !order.symbol.is_empty(),
        "Order validation failed: symbol cannot be empty"
    );

    // Check price
    ensure!(
        order.price > 0.0,
        "Order validation failed for {}: price must be positive, got {}",
        order.symbol, order.price
    );

    // Check quantity
    ensure!(
        order.quantity >= min_order_size,
        "Order validation failed for {}: quantity {} is below minimum {}",
        order.symbol, order.quantity, min_order_size
    );

    // Check balance
    let required = order.price * order.quantity;
    ensure!(
        required <= balance,
        "Order validation failed for {}: required ${:.2} exceeds balance ${:.2}",
        order.symbol, required, balance
    );

    Ok(())
}

fn submit_order(order: Order, balance: f64) -> Result<String> {
    validate_order(&order, balance, 0.001)
        .with_context(|| format!(
            "Cannot submit {} order for {} @ ${:.2}",
            order.side, order.symbol, order.price
        ))?;

    Ok(format!("Order submitted: {:?}", order))
}

fn main() -> Result<()> {
    let order = Order {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        price: 42000.0,
        quantity: 0.5,
    };

    // Attempt with insufficient balance
    match submit_order(order, 10000.0) {
        Ok(msg) => println!("{}", msg),
        Err(e) => println!("Error:\n{:?}", e),
    }

    Ok(())
}
```

### 3. Loading Portfolio with Context Chain

```rust
use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(Debug)]
struct Portfolio {
    positions: HashMap<String, f64>,
}

fn load_portfolio_file(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read portfolio file: {}", path))
}

fn parse_portfolio(content: &str) -> Result<Portfolio> {
    let mut positions = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let parts: Vec<&str> = line.split(':').collect();

        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid format at line {}: expected 'SYMBOL:QUANTITY'",
                line_num + 1
            );
        }

        let symbol = parts[0].trim().to_string();
        let quantity: f64 = parts[1]
            .trim()
            .parse()
            .with_context(|| format!(
                "Failed to parse quantity at line {} for symbol {}",
                line_num + 1, symbol
            ))?;

        positions.insert(symbol, quantity);
    }

    Ok(Portfolio { positions })
}

fn load_portfolio(path: &str) -> Result<Portfolio> {
    let content = load_portfolio_file(path)
        .with_context(|| "Failed to load portfolio")?;

    parse_portfolio(&content)
        .with_context(|| format!("Failed to parse portfolio from {}", path))
}

fn main() -> Result<()> {
    match load_portfolio("portfolio.txt") {
        Ok(portfolio) => println!("Portfolio: {:?}", portfolio),
        Err(e) => {
            // Error chain with full context
            println!("Error chain:");
            for (i, cause) in e.chain().enumerate() {
                println!("  {}: {}", i, cause);
            }
        }
    }

    Ok(())
}
```

### 4. Risk Calculation with Context

```rust
use anyhow::{Context, Result, ensure};

#[derive(Debug)]
struct RiskMetrics {
    var_95: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
}

fn calculate_returns(prices: &[f64]) -> Result<Vec<f64>> {
    ensure!(
        prices.len() >= 2,
        "Need at least 2 prices to calculate returns, got {}",
        prices.len()
    );

    let returns: Vec<f64> = prices
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();

    Ok(returns)
}

fn calculate_var(returns: &[f64], confidence: f64) -> Result<f64> {
    ensure!(
        !returns.is_empty(),
        "Cannot calculate VaR on empty returns"
    );
    ensure!(
        confidence > 0.0 && confidence < 1.0,
        "Confidence level must be between 0 and 1, got {}",
        confidence
    );

    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let index = ((1.0 - confidence) * sorted.len() as f64) as usize;
    Ok(sorted[index])
}

fn calculate_risk_metrics(symbol: &str, prices: &[f64]) -> Result<RiskMetrics> {
    let returns = calculate_returns(prices)
        .with_context(|| format!("Failed to calculate returns for {}", symbol))?;

    let var_95 = calculate_var(&returns, 0.95)
        .with_context(|| format!("Failed to calculate VaR for {}", symbol))?;

    // Simplified calculation for example
    let max_drawdown = returns.iter().cloned().fold(0.0_f64, f64::min).abs();

    let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns.iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / returns.len() as f64;
    let std_dev = variance.sqrt();

    let sharpe_ratio = if std_dev > 0.0 { mean / std_dev } else { 0.0 };

    Ok(RiskMetrics {
        var_95,
        max_drawdown,
        sharpe_ratio,
    })
}

fn main() -> Result<()> {
    let btc_prices = vec![42000.0, 42500.0, 41800.0, 43000.0, 42200.0, 43500.0];

    match calculate_risk_metrics("BTCUSDT", &btc_prices) {
        Ok(metrics) => {
            println!("Risk Metrics for BTCUSDT:");
            println!("  VaR 95%: {:.4}", metrics.var_95);
            println!("  Max Drawdown: {:.4}", metrics.max_drawdown);
            println!("  Sharpe Ratio: {:.4}", metrics.sharpe_ratio);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Test with insufficient data
    let insufficient_data = vec![42000.0];
    match calculate_risk_metrics("ETHUSDT", &insufficient_data) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Expected error:\n{:?}", e),
    }

    Ok(())
}
```

## context() vs with_context()

```rust
use anyhow::{Context, Result};

fn example() -> Result<()> {
    // context() — static message (faster)
    let _data = std::fs::read_to_string("config.json")
        .context("Failed to read config")?;

    // with_context() — dynamic message (created on error)
    let path = "data.json";
    let _data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path))?;

    Ok(())
}
```

**When to use:**
- `context()` — when the message doesn't depend on variables
- `with_context()` — when you need to include dynamic data

## Practical Exercise 1: Exchange API Client

```rust
use anyhow::{Context, Result, bail};
use std::collections::HashMap;

struct ExchangeClient {
    name: String,
    connected: bool,
}

impl ExchangeClient {
    fn new(name: &str) -> Self {
        ExchangeClient {
            name: name.to_string(),
            connected: false,
        }
    }

    fn connect(&mut self) -> Result<()> {
        // Simulate connection
        self.connected = true;
        Ok(())
    }

    fn get_price(&self, symbol: &str) -> Result<f64> {
        if !self.connected {
            bail!("Not connected to exchange");
        }

        // Simulate price retrieval
        let prices: HashMap<&str, f64> = [
            ("BTCUSDT", 42000.0),
            ("ETHUSDT", 2200.0),
        ].into_iter().collect();

        prices.get(symbol)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Symbol not found"))
            .with_context(|| format!(
                "Failed to get price for {} from {}",
                symbol, self.name
            ))
    }

    fn place_order(&self, symbol: &str, side: &str, quantity: f64) -> Result<String> {
        if !self.connected {
            bail!("Not connected to exchange");
        }

        let price = self.get_price(symbol)
            .with_context(|| format!(
                "Cannot place {} order: failed to get current price",
                side
            ))?;

        Ok(format!(
            "Order placed on {}: {} {} {} @ ${:.2}",
            self.name, side, quantity, symbol, price
        ))
    }
}

fn main() -> Result<()> {
    let mut client = ExchangeClient::new("Binance");
    client.connect()?;

    // Successful order
    let result = client.place_order("BTCUSDT", "BUY", 0.1)?;
    println!("{}", result);

    // Order with unknown symbol
    match client.place_order("UNKNOWN", "BUY", 0.1) {
        Ok(r) => println!("{}", r),
        Err(e) => println!("Error:\n{:?}", e),
    }

    Ok(())
}
```

## Practical Exercise 2: Trade History Parser

```rust
use anyhow::{Context, Result};

#[derive(Debug)]
struct Trade {
    timestamp: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn parse_trade_line(line: &str, line_num: usize) -> Result<Trade> {
    let parts: Vec<&str> = line.split(',').collect();

    if parts.len() != 5 {
        anyhow::bail!(
            "Invalid format at line {}: expected 5 fields, got {}",
            line_num, parts.len()
        );
    }

    let timestamp: u64 = parts[0]
        .trim()
        .parse()
        .with_context(|| format!("Invalid timestamp at line {}: '{}'", line_num, parts[0]))?;

    let symbol = parts[1].trim().to_string();
    let side = parts[2].trim().to_string();

    let price: f64 = parts[3]
        .trim()
        .parse()
        .with_context(|| format!(
            "Invalid price at line {} for {}: '{}'",
            line_num, symbol, parts[3]
        ))?;

    let quantity: f64 = parts[4]
        .trim()
        .parse()
        .with_context(|| format!(
            "Invalid quantity at line {} for {}: '{}'",
            line_num, symbol, parts[4]
        ))?;

    Ok(Trade { timestamp, symbol, side, price, quantity })
}

fn parse_trade_history(content: &str) -> Result<Vec<Trade>> {
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(i, line)| {
            parse_trade_line(line, i + 1)
                .with_context(|| "Failed to parse trade history")
        })
        .collect()
}

fn main() -> Result<()> {
    let valid_data = r#"
1703980800,BTCUSDT,BUY,42000.0,0.5
1703984400,BTCUSDT,SELL,42500.0,0.5
1703988000,ETHUSDT,BUY,2200.0,2.0
"#;

    let trades = parse_trade_history(valid_data)?;
    for trade in &trades {
        println!("{:?}", trade);
    }

    // Test with error
    let invalid_data = "1703980800,BTCUSDT,BUY,invalid,0.5";
    match parse_trade_history(invalid_data) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("\nExpected error:\n{:?}", e),
    }

    Ok(())
}
```

## Practical Exercise 3: Strategy Manager

```rust
use anyhow::{Context, Result, ensure};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Strategy {
    name: String,
    enabled: bool,
    parameters: HashMap<String, f64>,
}

struct StrategyManager {
    strategies: HashMap<String, Strategy>,
}

impl StrategyManager {
    fn new() -> Self {
        StrategyManager {
            strategies: HashMap::new(),
        }
    }

    fn add_strategy(&mut self, strategy: Strategy) -> Result<()> {
        ensure!(
            !strategy.name.is_empty(),
            "Strategy name cannot be empty"
        );

        ensure!(
            !self.strategies.contains_key(&strategy.name),
            "Strategy '{}' already exists", strategy.name
        );

        self.strategies.insert(strategy.name.clone(), strategy);
        Ok(())
    }

    fn get_parameter(&self, strategy_name: &str, param: &str) -> Result<f64> {
        let strategy = self.strategies
            .get(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("Strategy not found"))
            .with_context(|| format!("Cannot get parameter '{}' from '{}'", param, strategy_name))?;

        strategy.parameters
            .get(param)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Parameter not found"))
            .with_context(|| format!(
                "Parameter '{}' not configured for strategy '{}'",
                param, strategy_name
            ))
    }

    fn execute_strategy(&self, strategy_name: &str, price: f64) -> Result<String> {
        let strategy = self.strategies
            .get(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("Strategy not found"))
            .with_context(|| format!("Cannot execute strategy '{}'", strategy_name))?;

        ensure!(
            strategy.enabled,
            "Strategy '{}' is disabled", strategy_name
        );

        let threshold = self.get_parameter(strategy_name, "threshold")
            .with_context(|| format!("Cannot execute '{}': missing required parameter", strategy_name))?;

        let signal = if price > threshold { "BUY" } else { "SELL" };

        Ok(format!(
            "Strategy '{}' signal: {} (price: {:.2}, threshold: {:.2})",
            strategy_name, signal, price, threshold
        ))
    }
}

fn main() -> Result<()> {
    let mut manager = StrategyManager::new();

    let mut params = HashMap::new();
    params.insert("threshold".to_string(), 42000.0);
    params.insert("stop_loss".to_string(), 0.02);

    manager.add_strategy(Strategy {
        name: "BTC_Momentum".to_string(),
        enabled: true,
        parameters: params,
    })?;

    // Successful execution
    let signal = manager.execute_strategy("BTC_Momentum", 42500.0)?;
    println!("{}", signal);

    // Error: non-existent strategy
    match manager.execute_strategy("Unknown", 42500.0) {
        Ok(s) => println!("{}", s),
        Err(e) => println!("Error:\n{:?}", e),
    }

    Ok(())
}
```

## Best Practices

### 1. Add Context at Module Boundaries

```rust
// In API module
fn fetch_price(symbol: &str) -> Result<f64> {
    internal_fetch(symbol)
        .with_context(|| format!("API: Failed to fetch price for {}", symbol))
}

// In strategy module
fn evaluate_signal(symbol: &str) -> Result<Signal> {
    let price = fetch_price(symbol)
        .with_context(|| format!("Strategy: Cannot evaluate signal for {}", symbol))?;
    // ...
}
```

### 2. Include Relevant Data

```rust
// Bad
.with_context(|| "Failed to process order")

// Good
.with_context(|| format!(
    "Failed to process {} order for {} @ ${:.2}",
    order.side, order.symbol, order.price
))
```

### 3. Use Chain for Error Path Tracking

```rust
fn main() -> Result<()> {
    if let Err(e) = run_trading_bot() {
        eprintln!("Trading bot error:");
        for (i, cause) in e.chain().enumerate() {
            eprintln!("  {}: {}", i, cause);
        }
    }
    Ok(())
}
```

## What We Learned

| Method | Usage | Example |
|--------|-------|---------|
| `context()` | Static message | `.context("Failed to load")` |
| `with_context()` | Dynamic message | `.with_context(\|\| format!("Failed for {}", x))` |
| `ensure!()` | Condition check | `ensure!(x > 0, "Must be positive")` |
| `bail!()` | Immediate error return | `bail!("Invalid state")` |
| `.chain()` | Iterate error chain | `for cause in e.chain()` |

## Homework

1. Create a function `load_exchange_config(path: &str) -> Result<ExchangeConfig>` that loads exchange configuration from a file with detailed error context for each step (file reading, JSON parsing, field validation)

2. Implement `execute_trading_strategy(name: &str, market_data: &MarketData) -> Result<Vec<Order>>` where each step of the strategy adds its own context to potential errors

3. Write a function `reconcile_positions(local: &Portfolio, remote: &Portfolio) -> Result<ReconciliationReport>` that compares positions and returns detailed mismatch errors with context

4. Create an exchange API wrapper with methods `get_balance()`, `place_order()`, `cancel_order()`, where each method adds informative context to network and API errors

## Navigation

[← Previous day](../104-error-handling-best-practices/en.md) | [Next day →](../106-custom-error-types/en.md)
