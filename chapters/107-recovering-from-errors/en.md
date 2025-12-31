# Day 107: Recovering from Errors — Building a Resilient Trading System

## Trading Analogy

In real trading, errors are inevitable: exchanges may be unavailable, APIs return unexpected responses, data may be corrupted. Professional traders don't just handle errors — they **recover** from them: switch to backup exchanges, use cached data, or defer operations until service recovery.

## The `?` Operator — Elegant Error Propagation

The `?` operator allows concise error propagation up the call stack:

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};

fn main() {
    match load_trading_config("config.json") {
        Ok(config) => println!("Config loaded: {:?}", config),
        Err(e) => eprintln!("Failed to load config: {}", e),
    }
}

#[derive(Debug)]
struct TradingConfig {
    max_position_size: f64,
    risk_per_trade: f64,
    stop_loss_percent: f64,
}

fn load_trading_config(path: &str) -> Result<TradingConfig, io::Error> {
    let file = File::open(path)?;  // If error — return immediately
    let reader = BufReader::new(file);

    let mut lines = reader.lines();

    let max_position = lines.next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing max_position"))??
        .parse::<f64>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let risk = lines.next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing risk"))??
        .parse::<f64>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let stop_loss = lines.next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing stop_loss"))??
        .parse::<f64>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(TradingConfig {
        max_position_size: max_position,
        risk_per_trade: risk,
        stop_loss_percent: stop_loss,
    })
}
```

## Result Combinators — Processing Chains

### `map` and `map_err` — Transforming Value or Error

```rust
fn main() {
    let price_str = "42500.50";

    // map transforms the Ok value
    let price_with_fee: Result<f64, _> = price_str
        .parse::<f64>()
        .map(|price| price * 1.001);  // Add 0.1% fee

    println!("Price with fee: {:?}", price_with_fee);

    // map_err transforms the error
    let price: Result<f64, String> = "invalid"
        .parse::<f64>()
        .map_err(|e| format!("Price parsing failed: {}", e));

    println!("Parse result: {:?}", price);
}
```

### `and_then` — Chaining Operations

```rust
fn main() {
    let result = validate_price("42500.50")
        .and_then(|price| validate_quantity("0.5").map(|qty| (price, qty)))
        .and_then(|(price, qty)| calculate_order_value(price, qty));

    match result {
        Ok(value) => println!("Order value: ${:.2}", value),
        Err(e) => println!("Error: {}", e),
    }
}

fn validate_price(s: &str) -> Result<f64, String> {
    let price = s.parse::<f64>()
        .map_err(|_| format!("Invalid price: {}", s))?;

    if price <= 0.0 {
        return Err("Price must be positive".to_string());
    }
    if price > 1_000_000.0 {
        return Err("Price exceeds maximum".to_string());
    }
    Ok(price)
}

fn validate_quantity(s: &str) -> Result<f64, String> {
    let qty = s.parse::<f64>()
        .map_err(|_| format!("Invalid quantity: {}", s))?;

    if qty <= 0.0 {
        return Err("Quantity must be positive".to_string());
    }
    Ok(qty)
}

fn calculate_order_value(price: f64, qty: f64) -> Result<f64, String> {
    let value = price * qty;
    if value < 10.0 {
        return Err("Minimum order value is $10".to_string());
    }
    Ok(value)
}
```

### `unwrap_or` and `unwrap_or_else` — Default Values

```rust
fn main() {
    // unwrap_or — static default value
    let price = get_price("BTC")
        .unwrap_or(0.0);
    println!("BTC price: {}", price);

    // unwrap_or_else — lazy default computation
    let price = get_price("UNKNOWN")
        .unwrap_or_else(|e| {
            eprintln!("Warning: {}, using cached price", e);
            get_cached_price("UNKNOWN")
        });
    println!("Unknown asset price: {}", price);

    // unwrap_or_default — type's default value
    let count: i32 = "invalid".parse().unwrap_or_default();
    println!("Count: {}", count);  // 0
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42500.0),
        "ETH" => Ok(2250.0),
        _ => Err(format!("Unknown symbol: {}", symbol)),
    }
}

fn get_cached_price(_symbol: &str) -> f64 {
    0.0  // Placeholder for cached price
}
```

## Retry Pattern — Repeated Attempts

```rust
use std::thread;
use std::time::Duration;

fn main() {
    match fetch_price_with_retry("BTC", 3) {
        Ok(price) => println!("BTC price: ${:.2}", price),
        Err(e) => eprintln!("Failed after retries: {}", e),
    }
}

fn fetch_price_with_retry(symbol: &str, max_retries: u32) -> Result<f64, String> {
    let mut last_error = String::new();

    for attempt in 1..=max_retries {
        match fetch_price_from_api(symbol) {
            Ok(price) => {
                println!("Success on attempt {}", attempt);
                return Ok(price);
            }
            Err(e) => {
                last_error = e;
                if attempt < max_retries {
                    let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                    println!("Attempt {} failed, retrying in {:?}...", attempt, delay);
                    thread::sleep(delay);
                }
            }
        }
    }

    Err(format!("All {} attempts failed. Last error: {}", max_retries, last_error))
}

fn fetch_price_from_api(symbol: &str) -> Result<f64, String> {
    // Simulating unstable API
    static mut CALL_COUNT: u32 = 0;
    unsafe {
        CALL_COUNT += 1;
        if CALL_COUNT < 3 {
            return Err("Connection timeout".to_string());
        }
    }

    match symbol {
        "BTC" => Ok(42500.0),
        "ETH" => Ok(2250.0),
        _ => Err(format!("Unknown symbol: {}", symbol)),
    }
}
```

## Fallback Pattern — Backup Data Sources

```rust
fn main() {
    let price = get_price_with_fallback("BTC");
    println!("BTC price: ${:.2}", price);
}

fn get_price_with_fallback(symbol: &str) -> f64 {
    // Try primary source
    get_price_primary(symbol)
        .or_else(|e| {
            eprintln!("Primary source failed: {}", e);
            get_price_secondary(symbol)
        })
        .or_else(|e| {
            eprintln!("Secondary source failed: {}", e);
            get_cached_price_result(symbol)
        })
        .unwrap_or_else(|e| {
            eprintln!("All sources failed: {}. Using emergency default.", e);
            get_emergency_default_price(symbol)
        })
}

fn get_price_primary(_symbol: &str) -> Result<f64, String> {
    Err("Primary exchange unavailable".to_string())
}

fn get_price_secondary(_symbol: &str) -> Result<f64, String> {
    Err("Secondary exchange rate limited".to_string())
}

fn get_cached_price_result(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),  // Cached price
        _ => Err("No cached price available".to_string()),
    }
}

fn get_emergency_default_price(symbol: &str) -> f64 {
    match symbol {
        "BTC" => 40000.0,
        "ETH" => 2000.0,
        _ => 0.0,
    }
}
```

## Graceful Degradation

```rust
fn main() {
    let analysis = analyze_market_with_degradation("BTC");
    println!("{:?}", analysis);
}

#[derive(Debug)]
struct MarketAnalysis {
    symbol: String,
    price: Option<f64>,
    volume: Option<f64>,
    rsi: Option<f64>,
    recommendation: String,
    confidence: f64,
    data_quality: DataQuality,
}

#[derive(Debug)]
enum DataQuality {
    Full,      // All data available
    Partial,   // Partial data
    Degraded,  // Minimal data
}

fn analyze_market_with_degradation(symbol: &str) -> MarketAnalysis {
    let price = fetch_price(symbol).ok();
    let volume = fetch_volume(symbol).ok();
    let rsi = calculate_rsi(symbol).ok();

    let (recommendation, confidence, quality) = match (&price, &volume, &rsi) {
        (Some(p), Some(v), Some(r)) => {
            // Full analysis
            let rec = if *r < 30.0 { "BUY" } else if *r > 70.0 { "SELL" } else { "HOLD" };
            let conf = calculate_confidence(*p, *v, *r);
            (rec.to_string(), conf, DataQuality::Full)
        }
        (Some(p), _, Some(r)) => {
            // Partial analysis without volume
            let rec = if *r < 30.0 { "WEAK_BUY" } else if *r > 70.0 { "WEAK_SELL" } else { "HOLD" };
            (rec.to_string(), 0.5, DataQuality::Partial)
        }
        (Some(_), _, _) => {
            // Only price available
            ("HOLD".to_string(), 0.2, DataQuality::Degraded)
        }
        _ => {
            // No data
            ("NO_DATA".to_string(), 0.0, DataQuality::Degraded)
        }
    };

    MarketAnalysis {
        symbol: symbol.to_string(),
        price,
        volume,
        rsi,
        recommendation,
        confidence,
        data_quality: quality,
    }
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42500.0),
        _ => Err("Price unavailable".to_string()),
    }
}

fn fetch_volume(_symbol: &str) -> Result<f64, String> {
    Err("Volume data unavailable".to_string())
}

fn calculate_rsi(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(45.0),
        _ => Err("RSI calculation failed".to_string()),
    }
}

fn calculate_confidence(price: f64, volume: f64, rsi: f64) -> f64 {
    let price_factor = if price > 0.0 { 0.3 } else { 0.0 };
    let volume_factor = if volume > 0.0 { 0.3 } else { 0.0 };
    let rsi_factor = if rsi >= 0.0 && rsi <= 100.0 { 0.4 } else { 0.0 };
    price_factor + volume_factor + rsi_factor
}
```

## Error Context — Adding Information

```rust
fn main() {
    match execute_trade("BTC", 0.5, 42500.0) {
        Ok(order_id) => println!("Trade executed: {}", order_id),
        Err(e) => eprintln!("Trade failed:\n{}", e),
    }
}

#[derive(Debug)]
struct TradeError {
    context: Vec<String>,
    source: String,
}

impl std::fmt::Display for TradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error: {}", self.source)?;
        for (i, ctx) in self.context.iter().rev().enumerate() {
            writeln!(f, "  {}: {}", i + 1, ctx)?;
        }
        Ok(())
    }
}

impl TradeError {
    fn new(source: impl Into<String>) -> Self {
        TradeError {
            context: vec![],
            source: source.into(),
        }
    }

    fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }
}

fn execute_trade(symbol: &str, quantity: f64, price: f64) -> Result<String, TradeError> {
    validate_balance(price * quantity)
        .map_err(|e| e.with_context(format!("executing trade for {} {}", quantity, symbol)))?;

    submit_order(symbol, quantity, price)
        .map_err(|e| e.with_context("submitting order to exchange"))?;

    Ok("ORD-12345".to_string())
}

fn validate_balance(required: f64) -> Result<(), TradeError> {
    let balance = 10000.0;  // Simulated balance
    if required > balance {
        return Err(TradeError::new(format!(
            "Insufficient balance: need ${:.2}, have ${:.2}",
            required, balance
        )).with_context("validating account balance"));
    }
    Ok(())
}

fn submit_order(_symbol: &str, _quantity: f64, _price: f64) -> Result<(), TradeError> {
    // Simulating error
    Err(TradeError::new("Exchange connection timeout")
        .with_context("connecting to exchange API"))
}
```

## State Recovery After Failure

```rust
fn main() {
    let mut portfolio = Portfolio::new(100000.0);

    // Create checkpoint before risky operation
    let checkpoint = portfolio.create_checkpoint();

    match portfolio.execute_trades(vec![
        ("BTC", 0.5, 42500.0),
        ("ETH", 10.0, 2250.0),
        ("INVALID", 1.0, 100.0),  // This will cause an error
    ]) {
        Ok(_) => println!("All trades executed successfully"),
        Err(e) => {
            eprintln!("Trade execution failed: {}. Rolling back...", e);
            portfolio.restore_checkpoint(checkpoint);
            println!("Portfolio restored to checkpoint");
        }
    }

    println!("Final balance: ${:.2}", portfolio.balance);
}

struct Portfolio {
    balance: f64,
    positions: Vec<(String, f64)>,
}

struct PortfolioCheckpoint {
    balance: f64,
    positions: Vec<(String, f64)>,
}

impl Portfolio {
    fn new(balance: f64) -> Self {
        Portfolio {
            balance,
            positions: vec![],
        }
    }

    fn create_checkpoint(&self) -> PortfolioCheckpoint {
        PortfolioCheckpoint {
            balance: self.balance,
            positions: self.positions.clone(),
        }
    }

    fn restore_checkpoint(&mut self, checkpoint: PortfolioCheckpoint) {
        self.balance = checkpoint.balance;
        self.positions = checkpoint.positions;
    }

    fn execute_trades(&mut self, trades: Vec<(&str, f64, f64)>) -> Result<(), String> {
        for (symbol, qty, price) in trades {
            self.execute_single_trade(symbol, qty, price)?;
        }
        Ok(())
    }

    fn execute_single_trade(&mut self, symbol: &str, qty: f64, price: f64) -> Result<(), String> {
        if symbol == "INVALID" {
            return Err(format!("Unknown symbol: {}", symbol));
        }

        let cost = qty * price;
        if cost > self.balance {
            return Err(format!("Insufficient balance for {} {}", qty, symbol));
        }

        self.balance -= cost;
        self.positions.push((symbol.to_string(), qty));
        println!("Executed: {} {} @ ${:.2}", qty, symbol, price);
        Ok(())
    }
}
```

## Practical Exercises

### Exercise 1: API with Retry

```rust
// Implement a function that makes API requests with exponential backoff
fn fetch_with_exponential_backoff<T, F>(
    operation: F,
    max_retries: u32,
    base_delay_ms: u64,
) -> Result<T, String>
where
    F: Fn() -> Result<T, String>,
{
    // Your code here
    todo!()
}
```

### Exercise 2: Multi-source Price Feed

```rust
// Implement price fetching from multiple sources with priority
fn get_best_price(symbol: &str) -> Result<(f64, &'static str), String> {
    // Try:
    // 1. Primary exchange (priority 1)
    // 2. Secondary exchange (priority 2)
    // 3. Aggregator API (priority 3)
    // Return price and source name
    todo!()
}
```

### Exercise 3: Transaction with Rollback

```rust
// Implement a transaction system with automatic rollback
struct TransactionManager {
    // Your structure
}

impl TransactionManager {
    fn begin(&mut self) -> Transaction { todo!() }
}

struct Transaction {
    // Your structure
}

impl Transaction {
    fn execute<F: FnOnce() -> Result<(), String>>(&mut self, op: F) -> Result<(), String> {
        todo!()
    }
    fn commit(self) -> Result<(), String> { todo!() }
    fn rollback(self) { todo!() }
}
```

## Recovery Patterns

| Pattern | When to Use | Example |
|---------|-------------|---------|
| Retry | Temporary failures | Network timeouts |
| Fallback | Alternative sources | Backup exchange |
| Circuit Breaker | Cascade failure protection | Service isolation |
| Checkpoint | Atomic operations | Batch transactions |
| Graceful Degradation | Partial availability | Cached data |

## Homework

1. **Circuit Breaker**: Implement the Circuit Breaker pattern to protect against API overload. If 5 consecutive requests fail — switch to "open" state for 30 seconds.

2. **Order Recovery System**: Create a system that saves incomplete orders to a file on failure and recovers them on restart.

3. **Multi-Exchange Aggregator**: Write an aggregator that fetches prices from multiple exchanges in parallel, ignores failed responses, and returns the average price from successful ones.

4. **Health Check System**: Implement a health-check system that monitors the status of various services (exchange, database, API) and adapts system behavior based on availability.

## Navigation

[← Previous day](../106-custom-error-types/en.md) | [Next day →](../108-error-handling-best-practices/en.md)
