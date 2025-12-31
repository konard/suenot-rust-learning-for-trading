# Day 117: Errors in Async Code (Preview)

## Trading Analogy

Imagine you have several trading bots running simultaneously: one monitors BTC price, another watches ETH, and a third sends orders to the exchange. Each bot works **asynchronously** — without blocking the others. But what happens when one of the bots encounters an error?

If the bot monitoring BTC loses connection to the exchange — the other bots should continue working. But we must **properly handle** the error: log it, try to reconnect, or notify the trader.

In synchronous code, an error stops execution. In asynchronous code — an error in one task shouldn't "crash" all the others. This is the main challenge of error handling in async.

## Why Are Errors in Async More Complex?

### 1. Deferred Execution

In async code, a function doesn't execute immediately — it returns a `Future`:

```rust
use std::future::Future;

// Synchronous code — executes immediately
fn sync_fetch_price() -> Result<f64, String> {
    // Works right now
    Ok(42000.0)
}

// Async code — returns a "promise" of result
async fn async_fetch_price() -> Result<f64, String> {
    // Will only execute when we .await
    Ok(42000.0)
}

#[tokio::main]
async fn main() {
    // Without .await nothing happens!
    let future = async_fetch_price(); // Just created a Future

    // Now we execute
    let result = future.await;
    println!("Price: {:?}", result);
}
```

### 2. Errors Can Occur at Any Moment

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price_with_timeout() -> Result<f64, String> {
    // Request can fail at any point during wait
    sleep(Duration::from_secs(1)).await;

    // Or here
    let price = simulate_api_call().await?;

    // Or here
    Ok(price)
}

async fn simulate_api_call() -> Result<f64, String> {
    // Simulate random error
    if rand::random::<bool>() {
        Err("Connection lost".to_string())
    } else {
        Ok(42000.0)
    }
}
```

### 3. Parallel Tasks

When multiple tasks execute simultaneously, each can finish with an error:

```rust
use tokio::join;

async fn fetch_btc_price() -> Result<f64, String> {
    Ok(42000.0)
}

async fn fetch_eth_price() -> Result<f64, String> {
    Err("ETH API unavailable".to_string())
}

async fn fetch_sol_price() -> Result<f64, String> {
    Ok(95.0)
}

#[tokio::main]
async fn main() {
    // All three requests execute in parallel
    let (btc, eth, sol) = join!(
        fetch_btc_price(),
        fetch_eth_price(),
        fetch_sol_price()
    );

    // Each result needs separate handling
    println!("BTC: {:?}", btc);  // Ok(42000.0)
    println!("ETH: {:?}", eth);  // Err("ETH API unavailable")
    println!("SOL: {:?}", sol);  // Ok(95.0)
}
```

## The ? Operator in Async Functions

The `?` operator works the same as in regular functions:

```rust
use tokio::fs;

async fn load_portfolio() -> Result<Portfolio, Box<dyn std::error::Error>> {
    // ? propagates error upward
    let content = fs::read_to_string("portfolio.json").await?;
    let portfolio: Portfolio = serde_json::from_str(&content)?;
    Ok(portfolio)
}

#[derive(Debug, serde::Deserialize)]
struct Portfolio {
    btc: f64,
    eth: f64,
}
```

## Error Handling with tokio::spawn

When a task is launched via `tokio::spawn`, errors are wrapped in `JoinError`:

```rust
use tokio::task::JoinError;

async fn risky_trade() -> Result<f64, String> {
    Err("Trade failed: insufficient balance".to_string())
}

#[tokio::main]
async fn main() {
    // spawn returns JoinHandle
    let handle = tokio::spawn(async {
        risky_trade().await
    });

    // Result is Result<Result<f64, String>, JoinError>
    match handle.await {
        Ok(inner_result) => {
            // Task completed, check inner Result
            match inner_result {
                Ok(profit) => println!("Profit: ${:.2}", profit),
                Err(e) => println!("Trade error: {}", e),
            }
        }
        Err(join_error) => {
            // Task panicked or was cancelled
            println!("Task failed: {:?}", join_error);
        }
    }
}
```

### Simplifying with flatten

```rust
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        fetch_price().await
    });

    // Use pattern matching for simplification
    match handle.await {
        Ok(Ok(price)) => println!("Price: ${}", price),
        Ok(Err(e)) => println!("API error: {}", e),
        Err(e) => println!("Task panic: {:?}", e),
    }
}

async fn fetch_price() -> Result<f64, String> {
    Ok(42000.0)
}
```

## tokio::try_join! — Abort on First Error

When you need to execute multiple async operations and abort everything on first error:

```rust
use tokio::try_join;

async fn fetch_btc_price() -> Result<f64, String> {
    Ok(42000.0)
}

async fn fetch_eth_price() -> Result<f64, String> {
    Err("ETH API down".to_string())
}

async fn fetch_sol_price() -> Result<f64, String> {
    Ok(95.0)
}

#[tokio::main]
async fn main() {
    // try_join! aborts on first error
    let result = try_join!(
        fetch_btc_price(),
        fetch_eth_price(),
        fetch_sol_price()
    );

    match result {
        Ok((btc, eth, sol)) => {
            println!("All prices: BTC={}, ETH={}, SOL={}", btc, eth, sol);
        }
        Err(e) => {
            // Got error — remaining tasks are cancelled
            println!("Failed to fetch prices: {}", e);
        }
    }
}
```

## tokio::select! — Racing Tasks

`select!` allows waiting for the first completed task:

```rust
use tokio::{select, time::{sleep, Duration}};

async fn fetch_from_binance() -> Result<f64, String> {
    sleep(Duration::from_millis(100)).await;
    Ok(42000.0)
}

async fn fetch_from_kraken() -> Result<f64, String> {
    sleep(Duration::from_millis(150)).await;
    Ok(42050.0)
}

async fn timeout_task() {
    sleep(Duration::from_secs(5)).await;
}

#[tokio::main]
async fn main() {
    select! {
        result = fetch_from_binance() => {
            match result {
                Ok(price) => println!("Binance: ${}", price),
                Err(e) => println!("Binance error: {}", e),
            }
        }
        result = fetch_from_kraken() => {
            match result {
                Ok(price) => println!("Kraken: ${}", price),
                Err(e) => println!("Kraken error: {}", e),
            }
        }
        _ = timeout_task() => {
            println!("All requests timed out");
        }
    }
}
```

## Pattern: Error Handling in Price Monitoring

```rust
use tokio::time::{interval, Duration};

struct PriceMonitor {
    symbol: String,
    retry_count: u32,
    max_retries: u32,
}

impl PriceMonitor {
    fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            retry_count: 0,
            max_retries: 3,
        }
    }

    async fn fetch_price(&self) -> Result<f64, String> {
        // Simulate API request
        if rand::random::<f64>() < 0.3 {
            Err(format!("{} API error", self.symbol))
        } else {
            Ok(42000.0 * rand::random::<f64>())
        }
    }

    async fn run(&mut self) {
        let mut ticker = interval(Duration::from_secs(1));

        loop {
            ticker.tick().await;

            match self.fetch_price().await {
                Ok(price) => {
                    println!("[{}] Price: ${:.2}", self.symbol, price);
                    self.retry_count = 0; // Reset counter on success
                }
                Err(e) => {
                    self.retry_count += 1;
                    println!("[{}] Error (attempt {}): {}",
                        self.symbol, self.retry_count, e);

                    if self.retry_count >= self.max_retries {
                        println!("[{}] Max retries reached, stopping", self.symbol);
                        break;
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut monitor = PriceMonitor::new("BTC");
    monitor.run().await;
}
```

## Pattern: Task Cancellation on Error

```rust
use tokio::sync::watch;

async fn price_fetcher(symbol: &str, mut shutdown: watch::Receiver<bool>) {
    loop {
        tokio::select! {
            _ = async {
                // Simulate price fetching
                tokio::time::sleep(Duration::from_secs(1)).await;
                match fetch_price(symbol).await {
                    Ok(price) => println!("[{}] ${:.2}", symbol, price),
                    Err(e) => println!("[{}] Error: {}", symbol, e),
                }
            } => {}

            _ = shutdown.changed() => {
                println!("[{}] Shutdown signal received", symbol);
                break;
            }
        }
    }
}

async fn fetch_price(symbol: &str) -> Result<f64, String> {
    Ok(42000.0)
}

use tokio::time::Duration;

#[tokio::main]
async fn main() {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Launch multiple monitors
    let btc_handle = tokio::spawn(price_fetcher("BTC", shutdown_rx.clone()));
    let eth_handle = tokio::spawn(price_fetcher("ETH", shutdown_rx));

    // Wait 3 seconds and send shutdown signal
    tokio::time::sleep(Duration::from_secs(3)).await;
    let _ = shutdown_tx.send(true);

    // Wait for tasks to complete
    let _ = tokio::join!(btc_handle, eth_handle);
    println!("All monitors stopped");
}
```

## Error Handling with anyhow in Async

```rust
use anyhow::{Context, Result, bail};

async fn place_order(symbol: &str, quantity: f64, price: f64) -> Result<String> {
    if quantity <= 0.0 {
        bail!("Quantity must be positive");
    }

    let order_id = submit_to_exchange(symbol, quantity, price)
        .await
        .context("Failed to submit order to exchange")?;

    confirm_order(&order_id)
        .await
        .context(format!("Failed to confirm order {}", order_id))?;

    Ok(order_id)
}

async fn submit_to_exchange(symbol: &str, quantity: f64, price: f64) -> Result<String> {
    // Simulate sending to exchange
    Ok(format!("ORD-{}-{}", symbol, rand::random::<u32>()))
}

async fn confirm_order(order_id: &str) -> Result<()> {
    // Simulate confirmation
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    match place_order("BTC", 0.5, 42000.0).await {
        Ok(id) => println!("Order placed: {}", id),
        Err(e) => {
            // anyhow provides full error chain
            println!("Order failed: {:?}", e);
        }
    }
    Ok(())
}
```

## Practical Example: Robust Exchange Client

```rust
use std::time::Duration;
use tokio::time::sleep;

struct ExchangeClient {
    name: String,
    max_retries: u32,
    retry_delay: Duration,
}

#[derive(Debug)]
enum ExchangeError {
    ConnectionFailed(String),
    RateLimited,
    InvalidResponse(String),
    Timeout,
}

impl std::fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            Self::Timeout => write!(f, "Request timed out"),
        }
    }
}

impl std::error::Error for ExchangeError {}

impl ExchangeClient {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }

    async fn fetch_price(&self, symbol: &str) -> Result<f64, ExchangeError> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.try_fetch_price(symbol).await {
                Ok(price) => return Ok(price),
                Err(e) => {
                    println!("[{}] Attempt {}/{} failed: {}",
                        self.name, attempt, self.max_retries, e);
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        // Exponential backoff
                        let delay = self.retry_delay * (2_u32.pow(attempt - 1));
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn try_fetch_price(&self, symbol: &str) -> Result<f64, ExchangeError> {
        // Simulate various errors
        let random: f64 = rand::random();

        if random < 0.2 {
            Err(ExchangeError::ConnectionFailed("Network error".to_string()))
        } else if random < 0.3 {
            Err(ExchangeError::RateLimited)
        } else if random < 0.35 {
            Err(ExchangeError::Timeout)
        } else {
            Ok(42000.0 + (random * 1000.0))
        }
    }
}

#[tokio::main]
async fn main() {
    let client = ExchangeClient::new("Binance");

    match client.fetch_price("BTCUSDT").await {
        Ok(price) => println!("BTC price: ${:.2}", price),
        Err(e) => println!("Failed after all retries: {}", e),
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `async fn` with `Result` | Async functions return `Future<Output = Result<T, E>>` |
| `?` in async | Works the same as in sync code |
| `tokio::spawn` + errors | Returns `Result<T, JoinError>` — double Result |
| `try_join!` | Aborts execution on first error |
| `join!` | Waits for all tasks, each can have its own error |
| `select!` | Waits for first completed task |
| Retry with backoff | Repeat requests with increasing delay |
| Graceful shutdown | Use channels for proper stopping |

## Homework

1. Write an async function `fetch_prices_concurrent(symbols: Vec<&str>) -> HashMap<String, Result<f64, String>>` that fetches prices for all symbols in parallel and returns the result for each.

2. Implement a `PriceAggregator` that:
   - Queries price from multiple exchanges in parallel
   - Returns average price if at least 2 exchanges responded
   - Returns error if fewer than 2 exchanges responded

3. Create an async function `execute_with_timeout<T, E>(future: impl Future<Output = Result<T, E>>, timeout: Duration) -> Result<T, String>` that adds timeout to any async operation.

4. Write an `OrderExecutor` with methods:
   - `place_order()` — places order with retry logic
   - `cancel_order()` — cancels order
   - `wait_for_fill()` — waits for execution with timeout

## Navigation

[← Previous day](../116-documenting-errors/en.md) | [Next day →](../118-fail-fast/en.md)
