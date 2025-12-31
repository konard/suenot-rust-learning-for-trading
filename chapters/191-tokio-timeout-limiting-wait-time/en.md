# Day 191: tokio::timeout: Limiting Wait Time

## Trading Analogy

Imagine you're sending a market order to an exchange. In an ideal world, the exchange responds instantly: "Order executed!" But what if the exchange is overloaded? Or the network is slow? Or the exchange server is hanging?

In real trading, you never want to wait forever:
- **Price request**: If the exchange doesn't respond within 500ms — the price is already stale, use another source
- **Order placement**: If there's no response within 2 seconds — assume the order wasn't placed and make a decision
- **Order cancellation**: It's critically important to know the result quickly — the market doesn't wait

`tokio::timeout` is your "patience timer". It says: "Wait for the result, but no longer than X time. If time runs out — return an error, and I'll decide what to do next."

## What is tokio::timeout?

`tokio::timeout` is a function from the Tokio library that wraps any asynchronous operation (Future) and limits its execution time. If the operation doesn't complete within the specified time, it's interrupted and an `Elapsed` error is returned.

```rust
use tokio::time::{timeout, Duration};

async fn example() {
    // Wait for the result for a maximum of 5 seconds
    let result = timeout(Duration::from_secs(5), some_async_operation()).await;

    match result {
        Ok(value) => println!("Operation completed: {:?}", value),
        Err(_) => println!("Timeout! Operation didn't complete in time"),
    }
}
```

## Basic Syntax

```rust
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() {
    // timeout returns Result<T, Elapsed>
    // where T is the result of the inner Future
    let result = timeout(
        Duration::from_secs(2),  // Maximum wait time
        async {
            // Your asynchronous operation
            tokio::time::sleep(Duration::from_secs(1)).await;
            "Done!"
        }
    ).await;

    match result {
        Ok(value) => println!("Success: {}", value),
        Err(elapsed) => println!("Timeout: {:?}", elapsed),
    }
}
```

## Example: Price Request with Timeout

```rust
use tokio::time::{timeout, Duration};
use std::collections::HashMap;

// Simulating price request from an exchange
async fn fetch_price(exchange: &str, symbol: &str) -> f64 {
    // Simulating different response times for different exchanges
    let delay = match exchange {
        "fast_exchange" => 100,
        "normal_exchange" => 500,
        "slow_exchange" => 2000,
        _ => 1000,
    };

    tokio::time::sleep(Duration::from_millis(delay)).await;

    // Return "price"
    match (exchange, symbol) {
        ("fast_exchange", "BTC/USDT") => 42150.50,
        ("normal_exchange", "BTC/USDT") => 42148.25,
        ("slow_exchange", "BTC/USDT") => 42145.00,
        _ => 42000.0,
    }
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USDT";
    let max_wait = Duration::from_millis(300); // Maximum 300ms for response

    let exchanges = vec!["fast_exchange", "normal_exchange", "slow_exchange"];
    let mut prices: HashMap<&str, Option<f64>> = HashMap::new();

    for exchange in exchanges {
        println!("Requesting {} price from {}...", symbol, exchange);

        let result = timeout(max_wait, fetch_price(exchange, symbol)).await;

        match result {
            Ok(price) => {
                println!("  {} responded: ${:.2}", exchange, price);
                prices.insert(exchange, Some(price));
            }
            Err(_) => {
                println!("  {} — TIMEOUT! Exchange didn't respond within {:?}", exchange, max_wait);
                prices.insert(exchange, None);
            }
        }
    }

    // Use prices from exchanges that responded in time
    let valid_prices: Vec<f64> = prices
        .values()
        .filter_map(|p| *p)
        .collect();

    if !valid_prices.is_empty() {
        let avg_price: f64 = valid_prices.iter().sum::<f64>() / valid_prices.len() as f64;
        println!("\nAverage price (from {} sources): ${:.2}", valid_prices.len(), avg_price);
    } else {
        println!("\nNo exchange responded in time!");
    }
}
```

## Example: Order Placement with Timeout

```rust
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
struct Order {
    id: Option<u64>,
    symbol: String,
    side: String,
    quantity: f64,
    price: Option<f64>,
}

#[derive(Debug)]
enum OrderResult {
    Filled { order_id: u64, fill_price: f64 },
    PartialFill { order_id: u64, filled_qty: f64, remaining: f64 },
    Pending { order_id: u64 },
    Rejected { reason: String },
}

// Simulating exchange API
async fn place_order_api(order: Order) -> OrderResult {
    // Simulating network and processing delay
    tokio::time::sleep(Duration::from_millis(800)).await;

    // Simulating different results
    OrderResult::Filled {
        order_id: 12345,
        fill_price: 42100.0,
    }
}

async fn place_order_with_timeout(order: Order, max_wait: Duration) -> Result<OrderResult, String> {
    println!("Placing order: {:?}", order);

    match timeout(max_wait, place_order_api(order.clone())).await {
        Ok(result) => {
            println!("Order processed: {:?}", result);
            Ok(result)
        }
        Err(_) => {
            // Timeout! Order might have been placed, but response wasn't received
            Err(format!(
                "Timeout while placing {} {} order. \
                WARNING: Order might have been placed! Check open orders.",
                order.side, order.symbol
            ))
        }
    }
}

#[tokio::main]
async fn main() {
    let order = Order {
        id: None,
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        quantity: 0.5,
        price: Some(42000.0),
    };

    // Scenario 1: Sufficient timeout
    println!("=== Scenario 1: 2 second timeout ===");
    match place_order_with_timeout(order.clone(), Duration::from_secs(2)).await {
        Ok(result) => println!("Success: {:?}", result),
        Err(e) => println!("Error: {}", e),
    }

    println!();

    // Scenario 2: Too short timeout
    println!("=== Scenario 2: 100ms timeout ===");
    match place_order_with_timeout(order.clone(), Duration::from_millis(100)).await {
        Ok(result) => println!("Success: {:?}", result),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Example: Fetching Data from Multiple Exchanges with Timeout

```rust
use tokio::time::{timeout, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct MarketData {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume_24h: f64,
}

async fn fetch_market_data(exchange: &str, symbol: &str) -> MarketData {
    // Simulating different delays for different exchanges
    let delay = match exchange {
        "binance" => 150,
        "kraken" => 300,
        "coinbase" => 450,
        "huobi" => 1500, // Slow exchange
        _ => 500,
    };

    tokio::time::sleep(Duration::from_millis(delay)).await;

    // Generate different prices for different exchanges
    let base_price = 42000.0;
    let spread = match exchange {
        "binance" => 10.0,
        "kraken" => 15.0,
        "coinbase" => 20.0,
        "huobi" => 25.0,
        _ => 12.0,
    };

    MarketData {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        bid: base_price - spread / 2.0,
        ask: base_price + spread / 2.0,
        last_price: base_price,
        volume_24h: 1_000_000.0,
    }
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USDT";
    let exchanges = vec!["binance", "kraken", "coinbase", "huobi"];
    let data_timeout = Duration::from_millis(500);

    println!("Collecting {} market data from {} exchanges...", symbol, exchanges.len());
    println!("Timeout: {:?}\n", data_timeout);

    let mut results: HashMap<String, Option<MarketData>> = HashMap::new();

    // Request data in parallel
    let mut handles = vec![];
    for exchange in &exchanges {
        let exchange = exchange.to_string();
        let symbol = symbol.to_string();

        handles.push(tokio::spawn(async move {
            let result = timeout(
                data_timeout,
                fetch_market_data(&exchange, &symbol)
            ).await;

            (exchange, result)
        }));
    }

    // Collect results
    for handle in handles {
        let (exchange, result) = handle.await.unwrap();

        match result {
            Ok(data) => {
                println!("✓ {}: bid={:.2}, ask={:.2}, spread={:.2}",
                    data.exchange, data.bid, data.ask, data.ask - data.bid);
                results.insert(exchange, Some(data));
            }
            Err(_) => {
                println!("✗ {}: TIMEOUT", exchange);
                results.insert(exchange, None);
            }
        }
    }

    // Analyze collected data
    let valid_data: Vec<&MarketData> = results
        .values()
        .filter_map(|d| d.as_ref())
        .collect();

    if !valid_data.is_empty() {
        let best_bid = valid_data.iter().map(|d| d.bid).fold(f64::MIN, f64::max);
        let best_ask = valid_data.iter().map(|d| d.ask).fold(f64::MAX, f64::min);

        println!("\n=== Aggregated Data ===");
        println!("Best bid: ${:.2}", best_bid);
        println!("Best ask: ${:.2}", best_ask);
        println!("Cross-exchange spread: ${:.2}", best_ask - best_bid);
    }
}
```

## timeout vs timeout_at

Tokio provides two versions of timeout:

```rust
use tokio::time::{timeout, timeout_at, Duration, Instant};

#[tokio::main]
async fn main() {
    // timeout: relative time from the current moment
    let result1 = timeout(
        Duration::from_secs(5),
        some_operation()
    ).await;

    // timeout_at: absolute moment in time (deadline)
    let deadline = Instant::now() + Duration::from_secs(5);
    let result2 = timeout_at(
        deadline,
        some_operation()
    ).await;
}

async fn some_operation() -> &'static str {
    tokio::time::sleep(Duration::from_secs(1)).await;
    "done"
}
```

## Example: Trading Bot with Timeouts

```rust
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
struct TradingConfig {
    price_fetch_timeout: Duration,
    order_timeout: Duration,
    cancel_timeout: Duration,
}

impl Default for TradingConfig {
    fn default() -> Self {
        TradingConfig {
            price_fetch_timeout: Duration::from_millis(500),
            order_timeout: Duration::from_secs(2),
            cancel_timeout: Duration::from_secs(1),
        }
    }
}

struct TradingBot {
    config: TradingConfig,
    position: f64,
    balance: f64,
}

impl TradingBot {
    fn new(config: TradingConfig) -> Self {
        TradingBot {
            config,
            position: 0.0,
            balance: 10000.0,
        }
    }

    async fn get_price(&self, symbol: &str) -> Result<f64, String> {
        // Simulating price request
        async fn fetch(symbol: &str) -> f64 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            42000.0 + (symbol.len() as f64 * 10.0)
        }

        timeout(self.config.price_fetch_timeout, fetch(symbol))
            .await
            .map_err(|_| format!("Timeout while fetching {} price", symbol))
    }

    async fn place_order(&mut self, side: &str, quantity: f64, price: f64) -> Result<u64, String> {
        // Simulating order placement
        async fn submit(_: &str, _: f64, _: f64) -> u64 {
            tokio::time::sleep(Duration::from_millis(800)).await;
            rand::random::<u64>() % 1000000
        }

        let order_id = timeout(
            self.config.order_timeout,
            submit(side, quantity, price)
        )
        .await
        .map_err(|_| "Timeout while placing order".to_string())?;

        // Update position
        match side {
            "BUY" => {
                self.position += quantity;
                self.balance -= quantity * price;
            }
            "SELL" => {
                self.position -= quantity;
                self.balance += quantity * price;
            }
            _ => {}
        }

        Ok(order_id)
    }

    async fn cancel_order(&self, order_id: u64) -> Result<bool, String> {
        // Simulating order cancellation
        async fn cancel(_: u64) -> bool {
            tokio::time::sleep(Duration::from_millis(300)).await;
            true
        }

        timeout(self.config.cancel_timeout, cancel(order_id))
            .await
            .map_err(|_| format!("Timeout while canceling order {}", order_id))
    }

    fn status(&self) -> String {
        format!("Position: {:.4} BTC, Balance: ${:.2}", self.position, self.balance)
    }
}

// Adding a simple random number generator
mod rand {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random<T>() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

#[tokio::main]
async fn main() {
    let config = TradingConfig::default();
    let mut bot = TradingBot::new(config);

    println!("=== Trading Bot with Timeouts ===\n");
    println!("Initial state: {}\n", bot.status());

    // Get price
    match bot.get_price("BTC/USDT").await {
        Ok(price) => {
            println!("Current price: ${:.2}", price);

            // Place order
            match bot.place_order("BUY", 0.1, price).await {
                Ok(order_id) => {
                    println!("Order placed: #{}", order_id);
                    println!("New state: {}", bot.status());
                }
                Err(e) => println!("Order error: {}", e),
            }
        }
        Err(e) => println!("Price error: {}", e),
    }
}
```

## Timeout Error Handling

```rust
use tokio::time::{timeout, Duration, error::Elapsed};

#[derive(Debug)]
enum TradingError {
    Timeout(String),
    NetworkError(String),
    ApiError(String),
}

impl From<Elapsed> for TradingError {
    fn from(_: Elapsed) -> Self {
        TradingError::Timeout("Operation exceeded time limit".to_string())
    }
}

async fn robust_api_call<T, F, Fut>(
    operation_name: &str,
    max_duration: Duration,
    operation: F,
) -> Result<T, TradingError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, TradingError>>,
{
    match timeout(max_duration, operation()).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(e),
        Err(elapsed) => {
            println!("⚠ Timeout in '{}': {:?}", operation_name, elapsed);
            Err(TradingError::Timeout(format!(
                "{} didn't complete within {:?}",
                operation_name, max_duration
            )))
        }
    }
}

#[tokio::main]
async fn main() {
    let result = robust_api_call(
        "Get balance",
        Duration::from_secs(2),
        || async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok::<_, TradingError>(1000.0_f64)
        }
    ).await;

    match result {
        Ok(balance) => println!("Balance: ${:.2}", balance),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Practical Exercises

### Exercise 1: Quote Request with Timeout
Write a function `get_best_quote` that:
- Requests prices from three exchanges in parallel
- Sets a 200ms timeout on each request
- Returns the best (lowest) buy price from the received responses
- Handles the case when no exchange responded in time

### Exercise 2: Order with Retry
Implement a function `place_order_with_retry` that:
- Attempts to place an order with a 1-second timeout
- On timeout, makes up to 3 retry attempts
- Increases the timeout on each attempt (1s, 2s, 3s)
- Logs each attempt

### Exercise 3: Order Cancellation with Guarantee
Create a function `safe_cancel_order` that:
- Attempts to cancel an order with a timeout
- On timeout, checks the order status (filled/active/cancelled)
- Returns a clear result about the order's fate

## Homework

1. **Price Aggregator with Timeouts**: Create a `PriceAggregator` struct that:
   - Stores a list of exchanges to poll
   - Has configurable timeout for each exchange
   - Returns VWAP (Volume Weighted Average Price) from all responding exchanges
   - Logs which exchanges didn't respond in time

2. **Trading Engine with SLA**: Implement a trading engine that:
   - Guarantees a response to any operation within 5 seconds
   - Uses short timeouts for critical operations (cancel, stop-loss)
   - Sends an alert when SLA is exceeded (simulate via println)

3. **Parallel Requests with Shared Deadline**: Write a function that:
   - Sends 5 parallel requests
   - All requests must complete within 1 second (shared deadline)
   - Uses `timeout_at` for a single end time
   - Returns all successful results

4. **Graceful Degradation**: Implement a system that:
   - Falls back to a backup data source on primary source timeout
   - Tracks statistics of successful/unsuccessful requests
   - Automatically increases timeouts on frequent errors

## What We Learned

| Concept | Description |
|---------|-------------|
| `tokio::timeout` | Limits the execution time of an asynchronous operation |
| `Duration` | Structure for representing a time span |
| `Elapsed` | Error returned when timeout expires |
| `timeout_at` | Version with absolute time (deadline) |
| Graceful degradation | Pattern for smooth degradation on errors |

## Navigation

[← Previous day](../190-tokio-time-timers-delays/en.md) | [Next day →](../192-tokio-interval-periodic-tasks/en.md)
