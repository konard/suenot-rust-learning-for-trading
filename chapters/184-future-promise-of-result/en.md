# Day 184: Future: Promise of Result

## Trading Analogy

Imagine you place a limit order to buy Bitcoin at $40,000. The order is sent to the exchange, but it will only execute when the price reaches your target. You've received a **promise** of a result — a "receipt" that says: "When the conditions are met, you'll get your Bitcoin."

In Rust, this is called a **Future** — a promise that sometime in the future we'll receive a result. A Future is like a limit order: it represents work that isn't completed yet, but will be completed in the future.

Real trading examples of Futures:
- **Limit order** — waits for target price to be reached
- **API request to exchange** — waits for server response
- **Historical data download** — waits for data transfer over network
- **WebSocket connection** — waits for connection establishment

## What is a Future?

A Future in Rust is a trait that represents an asynchronous computation. Instead of blocking the thread while waiting for a result, a Future allows the program to continue doing other work.

```rust
// Simplified Future trait definition
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

// Poll is an enum with two variants
pub enum Poll<T> {
    Ready(T),    // Result is ready
    Pending,     // Still working, call back later
}
```

It's like calling the exchange and asking "Is my order filled?":
- `Ready(result)` — "Yes, here's your Bitcoin!"
- `Pending` — "Not yet, call back later"

## Simple Future Example

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// A Future that waits for target price to be reached
struct PriceTarget {
    target_price: f64,
    current_price: f64,
    started: Instant,
    timeout: Duration,
}

impl PriceTarget {
    fn new(target_price: f64, current_price: f64) -> Self {
        PriceTarget {
            target_price,
            current_price,
            started: Instant::now(),
            timeout: Duration::from_secs(10),
        }
    }

    fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;
    }
}

impl Future for PriceTarget {
    type Output = Result<f64, String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check timeout
        if self.started.elapsed() > self.timeout {
            return Poll::Ready(Err("Timeout: price target not reached".to_string()));
        }

        // Check if target price is reached
        if self.current_price >= self.target_price {
            Poll::Ready(Ok(self.current_price))
        } else {
            // Not ready yet — ask to wake us up later
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// Usage with async/await (requires runtime like tokio)
async fn wait_for_price() {
    println!("Waiting for target price...");
    // In real code, PriceTarget would be updated from external source
}
```

## async/await — Syntactic Sugar for Future

The `async` and `await` keywords are syntactic sugar that makes working with Futures convenient:

```rust
use tokio::time::{sleep, Duration};

/// An async function is a function that returns a Future
async fn fetch_bitcoin_price() -> f64 {
    // Simulate API request to exchange
    println!("Fetching BTC price...");
    sleep(Duration::from_millis(100)).await;

    // Return the "received" price
    42_150.75
}

async fn fetch_ethereum_price() -> f64 {
    println!("Fetching ETH price...");
    sleep(Duration::from_millis(150)).await;

    2_850.30
}

async fn calculate_portfolio_value(btc_amount: f64, eth_amount: f64) -> f64 {
    // await suspends execution until the Future is ready
    let btc_price = fetch_bitcoin_price().await;
    let eth_price = fetch_ethereum_price().await;

    let btc_value = btc_amount * btc_price;
    let eth_value = eth_amount * eth_price;

    println!("BTC: {} × ${:.2} = ${:.2}", btc_amount, btc_price, btc_value);
    println!("ETH: {} × ${:.2} = ${:.2}", eth_amount, eth_price, eth_value);

    btc_value + eth_value
}

#[tokio::main]
async fn main() {
    let total = calculate_portfolio_value(0.5, 10.0).await;
    println!("Total portfolio value: ${:.2}", total);
}
```

## Parallel Future Execution

One of the main advantages of Futures is the ability to execute multiple operations in parallel:

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price(symbol: &str) -> (String, f64) {
    // Simulate different response times for different exchanges
    let delay = match symbol {
        "BTC" => 100,
        "ETH" => 150,
        "SOL" => 80,
        _ => 200,
    };

    sleep(Duration::from_millis(delay)).await;

    let price = match symbol {
        "BTC" => 42_000.0,
        "ETH" => 2_800.0,
        "SOL" => 95.0,
        _ => 0.0,
    };

    (symbol.to_string(), price)
}

#[tokio::main]
async fn main() {
    // ❌ Sequential execution — slow!
    println!("=== Sequential ===");
    let start = std::time::Instant::now();

    let btc = fetch_price("BTC").await;
    let eth = fetch_price("ETH").await;
    let sol = fetch_price("SOL").await;

    println!("Time: {:?}", start.elapsed());
    println!("BTC: ${}, ETH: ${}, SOL: ${}", btc.1, eth.1, sol.1);

    // ✅ Parallel execution with join! — fast!
    println!("\n=== Parallel with join! ===");
    let start = std::time::Instant::now();

    let (btc, eth, sol) = tokio::join!(
        fetch_price("BTC"),
        fetch_price("ETH"),
        fetch_price("SOL")
    );

    println!("Time: {:?}", start.elapsed());
    println!("BTC: ${}, ETH: ${}, SOL: ${}", btc.1, eth.1, sol.1);
}
```

## Future as Return Type

Functions with `async` return a type that implements `Future`:

```rust
use std::future::Future;

/// Explicit return type specification
fn fetch_order_status(order_id: u64) -> impl Future<Output = String> {
    async move {
        // Simulate API request
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        if order_id % 2 == 0 {
            "filled".to_string()
        } else {
            "pending".to_string()
        }
    }
}

/// Shorter form is also available
async fn fetch_order_status_short(order_id: u64) -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    if order_id % 2 == 0 {
        "filled".to_string()
    } else {
        "pending".to_string()
    }
}

#[tokio::main]
async fn main() {
    let status1 = fetch_order_status(42).await;
    let status2 = fetch_order_status_short(43).await;

    println!("Order 42: {}", status1);
    println!("Order 43: {}", status2);
}
```

## Error Handling in Futures

Futures often return `Result` for error handling:

```rust
use tokio::time::{sleep, timeout, Duration};

#[derive(Debug)]
enum TradingError {
    Timeout,
    NetworkError(String),
    InsufficientFunds,
    OrderRejected(String),
}

async fn place_order(
    symbol: &str,
    side: &str,
    quantity: f64,
    price: f64,
) -> Result<u64, TradingError> {
    println!("Placing order: {} {} {} @ ${}", side, quantity, symbol, price);

    // Simulate network request
    sleep(Duration::from_millis(100)).await;

    // Check conditions (simulation)
    if quantity > 100.0 {
        return Err(TradingError::InsufficientFunds);
    }

    if price < 0.0 {
        return Err(TradingError::OrderRejected("Invalid price".to_string()));
    }

    // Success — return order ID
    Ok(12345)
}

async fn place_order_with_timeout(
    symbol: &str,
    side: &str,
    quantity: f64,
    price: f64,
) -> Result<u64, TradingError> {
    // Wrap in timeout
    match timeout(Duration::from_secs(5), place_order(symbol, side, quantity, price)).await {
        Ok(result) => result,
        Err(_) => Err(TradingError::Timeout),
    }
}

#[tokio::main]
async fn main() {
    // Successful order
    match place_order_with_timeout("BTC", "buy", 0.1, 42000.0).await {
        Ok(order_id) => println!("Order placed: #{}", order_id),
        Err(e) => println!("Error: {:?}", e),
    }

    // Order with error
    match place_order_with_timeout("BTC", "buy", 1000.0, 42000.0).await {
        Ok(order_id) => println!("Order placed: #{}", order_id),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Practical Example: Multi-Exchange Monitoring

```rust
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct PriceQuote {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

async fn fetch_from_binance(symbol: &str) -> PriceQuote {
    sleep(Duration::from_millis(50)).await;
    PriceQuote {
        exchange: "Binance".to_string(),
        symbol: symbol.to_string(),
        bid: 42_000.0,
        ask: 42_010.0,
        timestamp: 1234567890,
    }
}

async fn fetch_from_kraken(symbol: &str) -> PriceQuote {
    sleep(Duration::from_millis(75)).await;
    PriceQuote {
        exchange: "Kraken".to_string(),
        symbol: symbol.to_string(),
        bid: 41_995.0,
        ask: 42_015.0,
        timestamp: 1234567890,
    }
}

async fn fetch_from_coinbase(symbol: &str) -> PriceQuote {
    sleep(Duration::from_millis(60)).await;
    PriceQuote {
        exchange: "Coinbase".to_string(),
        symbol: symbol.to_string(),
        bid: 42_005.0,
        ask: 42_020.0,
        timestamp: 1234567890,
    }
}

async fn find_best_price(symbol: &str) -> (PriceQuote, PriceQuote) {
    // Query all exchanges in parallel
    let (binance, kraken, coinbase) = tokio::join!(
        fetch_from_binance(symbol),
        fetch_from_kraken(symbol),
        fetch_from_coinbase(symbol)
    );

    let quotes = vec![binance, kraken, coinbase];

    // Best price for buying (minimum ask)
    let best_buy = quotes.iter()
        .min_by(|a, b| a.ask.partial_cmp(&b.ask).unwrap())
        .unwrap()
        .clone();

    // Best price for selling (maximum bid)
    let best_sell = quotes.iter()
        .max_by(|a, b| a.bid.partial_cmp(&b.bid).unwrap())
        .unwrap()
        .clone();

    (best_buy, best_sell)
}

async fn check_arbitrage_opportunity(symbol: &str) -> Option<f64> {
    let (best_buy, best_sell) = find_best_price(symbol).await;

    println!("Best buy: {} @ ${:.2}", best_buy.exchange, best_buy.ask);
    println!("Best sell: {} @ ${:.2}", best_sell.exchange, best_sell.bid);

    let spread = best_sell.bid - best_buy.ask;
    let spread_percent = (spread / best_buy.ask) * 100.0;

    if spread > 0.0 {
        println!("Arbitrage opportunity: ${:.2} ({:.4}%)", spread, spread_percent);
        Some(spread)
    } else {
        println!("No arbitrage: spread ${:.2}", spread);
        None
    }
}

#[tokio::main]
async fn main() {
    println!("=== Searching for Arbitrage Opportunities ===\n");

    let start = std::time::Instant::now();

    check_arbitrage_opportunity("BTC").await;

    println!("\nExecution time: {:?}", start.elapsed());
}
```

## Future States

A Future can be in different states:

```
┌─────────────────────────────────────────────────────────────┐
│                        Future                                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌─────────┐    poll()     ┌─────────┐                     │
│   │ Created │ ─────────────>│ Pending │                     │
│   └─────────┘               └────┬────┘                     │
│                                  │                           │
│                                  │ poll() returns Pending    │
│                                  v                           │
│                            ┌─────────┐                       │
│                            │ Waiting │ <──────┐              │
│                            └────┬────┘        │              │
│                                 │             │              │
│                                 │ waker.wake()│              │
│                                 v             │              │
│                            ┌─────────┐        │              │
│                            │  Poll   │────────┘              │
│                            │  Again  │                       │
│                            └────┬────┘                       │
│                                 │                            │
│                                 │ poll() returns Ready       │
│                                 v                            │
│                            ┌─────────┐                       │
│                            │  Ready  │                       │
│                            │(Done)   │                       │
│                            └─────────┘                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Future | Trait representing an asynchronous computation |
| Poll::Ready | Result is ready |
| Poll::Pending | Result not ready yet, need to wait |
| async | Keyword for creating asynchronous functions |
| await | Suspends execution until Future is ready |
| tokio::join! | Executes multiple Futures in parallel |
| timeout | Limits wait time for a Future |

## Practical Exercises

1. **Parallel Price Query**: Write a function that queries prices of 5 cryptocurrencies in parallel and returns them in a HashMap.

2. **Order Timeout**: Create an order placement function with timeout. If the order is not confirmed within 5 seconds, return an error.

3. **First Response**: Using `tokio::select!`, write a function that returns the price from the first exchange to respond.

4. **Retry Logic**: Create a wrapper that retries failed requests up to 3 times with exponential backoff.

## Homework

1. **Price Aggregator**: Create a `PriceAggregator` struct that:
   - Connects to multiple "exchanges" (simulated)
   - Queries prices in parallel
   - Returns best bid/ask
   - Logs timing for each request

2. **Trading Bot with Futures**: Implement an async function `execute_strategy` that:
   - Fetches current prices (Future)
   - Analyzes entry conditions
   - Places an order (Future)
   - Waits for confirmation (Future with timeout)
   - Returns trade result or error

3. **Monitoring with Cancellation**: Create an infinite price monitoring loop that can be stopped via `tokio::select!` when receiving a cancellation signal.

4. **Custom Future**: Implement your own Future `DelayedOrder` that:
   - Takes an order and a delay
   - Returns `Poll::Pending` until the delay expires
   - Returns `Poll::Ready(order)` when it's time to execute

## Navigation

[← Previous day](../183-async-await-syntax/en.md) | [Next day →](../185-tokio-runtime-async-engine/en.md)
