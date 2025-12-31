# Day 182: Sync vs Async: Blocking vs Non-Blocking

## Trading Analogy

Imagine two types of traders on an exchange:

**Synchronous (blocking) trader**: Sends an order to the exchange and freezes, staring at the screen until they receive execution confirmation. Can't do anything while waiting — no chart analysis, no checking other positions, just waiting.

**Asynchronous (non-blocking) trader**: Sends an order and immediately switches to analyzing the next instrument. When confirmation arrives — processes it and continues working. Can simultaneously track dozens of orders without wasting a second.

In real trading, the second approach is critical:
- Exchange APIs can respond with 50-500ms latency
- Need to track quotes across many instruments
- Can't miss trading signals while waiting for exchange responses

## What is Synchronous Code?

Synchronous (blocking) code executes sequentially. Each operation waits for the previous one to complete:

```rust
use std::thread;
use std::time::Duration;

fn fetch_price_sync(symbol: &str) -> f64 {
    println!("[{}] Fetching {} price...",
             chrono::Local::now().format("%H:%M:%S%.3f"), symbol);

    // Simulate network request — thread is BLOCKED
    thread::sleep(Duration::from_millis(500));

    // Return "price"
    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2200.0,
        "SOL" => 95.0,
        _ => 0.0,
    }
}

fn main() {
    let start = std::time::Instant::now();

    // Sequential requests — each waits for the previous
    let btc = fetch_price_sync("BTC");
    let eth = fetch_price_sync("ETH");
    let sol = fetch_price_sync("SOL");

    println!("\nPrices: BTC=${}, ETH=${}, SOL=${}", btc, eth, sol);
    println!("Total time: {:?}", start.elapsed());
    // Result: ~1500ms (3 requests × 500ms)
}
```

**Problem**: If the exchange responds in 500ms, fetching prices for 10 instruments takes 5 seconds. The price could move during that time!

## What is Asynchronous Code?

Asynchronous (non-blocking) code allows starting an operation and doing other things while waiting for the result:

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price_async(symbol: &str) -> f64 {
    println!("[{}] Fetching {} price...",
             chrono::Local::now().format("%H:%M:%S%.3f"), symbol);

    // Simulate network request — thread is NOT blocked!
    sleep(Duration::from_millis(500)).await;

    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2200.0,
        "SOL" => 95.0,
        _ => 0.0,
    }
}

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();

    // Parallel requests — all execute simultaneously
    let (btc, eth, sol) = tokio::join!(
        fetch_price_async("BTC"),
        fetch_price_async("ETH"),
        fetch_price_async("SOL"),
    );

    println!("\nPrices: BTC=${}, ETH=${}, SOL=${}", btc, eth, sol);
    println!("Total time: {:?}", start.elapsed());
    // Result: ~500ms (all requests in parallel)
}
```

**Advantage**: 3 requests in the time of 1. With 10 instruments, that's 500ms instead of 5 seconds!

## How async/await Works in Rust

### Future — A Promise of a Result

A `Future` in Rust is a computation that may not be complete yet:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// Simplified Future implementation for understanding
struct PriceFuture {
    symbol: String,
    completed: bool,
}

impl Future for PriceFuture {
    type Output = f64;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            // Result is ready
            Poll::Ready(42000.0)
        } else {
            // Not ready yet, wake me later
            self.completed = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
```

### async fn Creates a Future

```rust
// This function:
async fn get_balance() -> f64 {
    100.0
}

// Is equivalent to:
fn get_balance() -> impl Future<Output = f64> {
    async { 100.0 }
}
```

### .await Suspends Execution

```rust
async fn trading_workflow() {
    // .await says: "Wait for the result, but don't block the thread"
    let price = fetch_price_async("BTC").await;

    // Continue only when price is ready
    if price > 41000.0 {
        place_order("BTC", "BUY", 0.1).await;
    }
}
```

## Comparing Approaches

```rust
use tokio::time::{sleep, Duration};
use std::thread;

// Synchronous approach — blocks the thread
fn sync_market_scan() {
    let symbols = vec!["BTC", "ETH", "SOL", "AVAX", "MATIC"];
    let start = std::time::Instant::now();

    for symbol in &symbols {
        // Each request blocks for 200ms
        thread::sleep(Duration::from_millis(200));
        println!("{}: checked", symbol);
    }

    println!("Synchronous scan: {:?}", start.elapsed());
    // ~1000ms
}

// Asynchronous approach — doesn't block
async fn async_market_scan() {
    let symbols = vec!["BTC", "ETH", "SOL", "AVAX", "MATIC"];
    let start = std::time::Instant::now();

    let futures: Vec<_> = symbols.iter().map(|symbol| {
        async move {
            sleep(Duration::from_millis(200)).await;
            println!("{}: checked", symbol);
            symbol
        }
    }).collect();

    // All requests execute in parallel
    futures::future::join_all(futures).await;

    println!("Asynchronous scan: {:?}", start.elapsed());
    // ~200ms
}
```

## Practical Example: Trading Bot

```rust
use tokio::time::{sleep, Duration, interval};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

// Async market data fetching
async fn fetch_market_data(symbol: &str) -> MarketData {
    // Simulate API request
    sleep(Duration::from_millis(100)).await;

    MarketData {
        symbol: symbol.to_string(),
        price: 42000.0 + (rand::random::<f64>() * 1000.0),
        volume: 1000.0 + (rand::random::<f64>() * 500.0),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    }
}

// Async order submission
async fn submit_order(order: Order) -> Result<String, String> {
    // Simulate sending to exchange
    sleep(Duration::from_millis(50)).await;

    Ok(format!("ORDER-{}", rand::random::<u32>()))
}

// Async strategy
async fn momentum_strategy(
    data: MarketData,
    prev_price: Option<f64>,
) -> Option<Order> {
    let Some(prev) = prev_price else {
        return None;
    };

    let change_pct = (data.price - prev) / prev * 100.0;

    // Buy on > 0.5% rise
    if change_pct > 0.5 {
        Some(Order {
            symbol: data.symbol,
            side: "BUY".to_string(),
            quantity: 0.01,
            price: data.price,
        })
    }
    // Sell on > 0.5% drop
    else if change_pct < -0.5 {
        Some(Order {
            symbol: data.symbol,
            side: "SELL".to_string(),
            quantity: 0.01,
            price: data.price,
        })
    } else {
        None
    }
}

// Main trading bot loop
async fn trading_bot() {
    let symbols = vec!["BTC", "ETH", "SOL"];
    let mut prev_prices: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    let mut interval = interval(Duration::from_secs(1));

    for _ in 0..10 { // 10 iterations for demo
        interval.tick().await;

        // Fetch data for all instruments IN PARALLEL
        let data_futures: Vec<_> = symbols.iter()
            .map(|s| fetch_market_data(s))
            .collect();

        let market_data = futures::future::join_all(data_futures).await;

        // Analyze each instrument
        for data in market_data {
            let prev = prev_prices.get(&data.symbol).copied();

            if let Some(order) = momentum_strategy(data.clone(), prev).await {
                // Submit order asynchronously
                match submit_order(order).await {
                    Ok(id) => println!("Order placed: {}", id),
                    Err(e) => println!("Error: {}", e),
                }
            }

            prev_prices.insert(data.symbol.clone(), data.price);
        }
    }
}
```

## When to Use Sync vs Async?

### Use synchronous code when:

```rust
// 1. Computation without I/O
fn calculate_sma(prices: &[f64], period: usize) -> f64 {
    prices.iter()
        .rev()
        .take(period)
        .sum::<f64>() / period as f64
}

// 2. Simple scripts without parallelism
fn main() {
    let data = vec![100.0, 101.0, 102.0, 101.5, 103.0];
    let sma = calculate_sma(&data, 3);
    println!("SMA(3) = {}", sma);
}

// 3. CPU-intensive tasks
fn backtest_strategy(historical_data: &[f64]) -> f64 {
    // Heavy computation is better in regular threads
    historical_data.iter()
        .map(|p| complex_calculation(*p))
        .sum()
}
```

### Use asynchronous code when:

```rust
// 1. Network operations
async fn fetch_orderbook(symbol: &str) -> OrderBook {
    let response = reqwest::get(format!(
        "https://api.exchange.com/orderbook/{}", symbol
    )).await.unwrap();

    response.json().await.unwrap()
}

// 2. Many parallel I/O operations
async fn monitor_portfolio(symbols: Vec<String>) {
    let mut handles = vec![];

    for symbol in symbols {
        handles.push(tokio::spawn(async move {
            loop {
                let price = fetch_price(&symbol).await;
                update_portfolio(&symbol, price).await;
                sleep(Duration::from_secs(1)).await;
            }
        }));
    }

    futures::future::join_all(handles).await;
}

// 3. Servers and connection handlers
async fn websocket_handler(ws: WebSocket) {
    while let Some(msg) = ws.recv().await {
        // Process messages from exchange
        process_market_update(msg).await;
    }
}
```

## Common Mistakes

### 1. Blocking Code in Async Context

```rust
// BAD: thread::sleep blocks the entire runtime
async fn bad_example() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks!
}

// GOOD: tokio::time::sleep doesn't block
async fn good_example() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

### 2. CPU-Intensive Tasks in Async

```rust
// BAD: heavy computation blocks async runtime
async fn bad_backtest() {
    let result = heavy_computation(); // Blocks for seconds!
    result
}

// GOOD: offload to separate thread
async fn good_backtest() {
    let result = tokio::task::spawn_blocking(|| {
        heavy_computation()
    }).await.unwrap();
    result
}
```

### 3. Forgotten .await

```rust
async fn forgotten_await() {
    let future = fetch_price("BTC"); // Returns Future, not f64!

    // Error: future will never execute
    // println!("Price: {}", future); // Won't compile

    // Correct:
    let price = future.await;
    println!("Price: {}", price);
}
```

## Practical Example: Multi-Exchange Arbitrage

```rust
use tokio::time::{timeout, Duration};
use futures::future::join_all;

#[derive(Debug, Clone)]
struct ExchangePrice {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
}

async fn fetch_binance_price(symbol: &str) -> Result<ExchangePrice, String> {
    tokio::time::sleep(Duration::from_millis(80)).await;
    Ok(ExchangePrice {
        exchange: "Binance".to_string(),
        symbol: symbol.to_string(),
        bid: 42000.0,
        ask: 42005.0,
    })
}

async fn fetch_kraken_price(symbol: &str) -> Result<ExchangePrice, String> {
    tokio::time::sleep(Duration::from_millis(120)).await;
    Ok(ExchangePrice {
        exchange: "Kraken".to_string(),
        symbol: symbol.to_string(),
        bid: 42010.0,
        ask: 42020.0,
    })
}

async fn fetch_coinbase_price(symbol: &str) -> Result<ExchangePrice, String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(ExchangePrice {
        exchange: "Coinbase".to_string(),
        symbol: symbol.to_string(),
        bid: 41995.0,
        ask: 42008.0,
    })
}

async fn find_arbitrage(symbol: &str) -> Option<(String, String, f64)> {
    // Fetch prices from all exchanges IN PARALLEL with timeout
    let fetch_timeout = Duration::from_millis(500);

    let results = join_all(vec![
        timeout(fetch_timeout, fetch_binance_price(symbol)),
        timeout(fetch_timeout, fetch_kraken_price(symbol)),
        timeout(fetch_timeout, fetch_coinbase_price(symbol)),
    ]).await;

    // Collect successful results
    let prices: Vec<ExchangePrice> = results
        .into_iter()
        .filter_map(|r| r.ok().and_then(|inner| inner.ok()))
        .collect();

    if prices.len() < 2 {
        return None;
    }

    // Find best buy and sell prices
    let best_bid = prices.iter()
        .max_by(|a, b| a.bid.partial_cmp(&b.bid).unwrap())?;
    let best_ask = prices.iter()
        .min_by(|a, b| a.ask.partial_cmp(&b.ask).unwrap())?;

    // Arbitrage: buy low, sell high
    let spread = best_bid.bid - best_ask.ask;
    let spread_pct = spread / best_ask.ask * 100.0;

    if spread > 0.0 {
        Some((
            format!("Buy on {} @ {}", best_ask.exchange, best_ask.ask),
            format!("Sell on {} @ {}", best_bid.exchange, best_bid.bid),
            spread_pct,
        ))
    } else {
        None
    }
}

#[tokio::main]
async fn main() {
    println!("Searching for arbitrage opportunities...\n");

    let symbols = vec!["BTC", "ETH", "SOL"];

    // Search for arbitrage across all pairs in parallel
    let arb_futures: Vec<_> = symbols.iter()
        .map(|s| find_arbitrage(s))
        .collect();

    let results = join_all(arb_futures).await;

    for (symbol, result) in symbols.iter().zip(results) {
        match result {
            Some((buy, sell, pct)) => {
                println!("{}: Arbitrage found! (+{:.3}%)", symbol, pct);
                println!("  {}", buy);
                println!("  {}", sell);
            }
            None => {
                println!("{}: No arbitrage found", symbol);
            }
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Synchronous code | Executes sequentially, blocks the thread |
| Asynchronous code | Non-blocking, allows parallel execution |
| `async fn` | Creates a function returning a Future |
| `.await` | Suspends execution until result is ready |
| `Future` | A value that will be available in the future |
| `tokio::join!` | Runs multiple Futures in parallel |
| `tokio::spawn` | Creates an independent async task |

## Homework

1. **Parallel Market Scanner**: Write a program that:
   - Fetches prices for 10 instruments in parallel
   - Calculates daily change for each
   - Outputs top-3 gainers and top-3 losers
   - Compare timing of sync vs async versions

2. **Quote Streaming**: Implement an async stream that:
   - Generates random price changes every 100ms
   - Calculates moving average (SMA) in real-time
   - Sends a signal when price crosses the SMA

3. **Multi-Exchange Connector**: Create an `ExchangeConnector` struct that:
   - Connects to multiple "exchanges" asynchronously
   - Tracks connection state
   - Automatically reconnects on disconnect
   - Aggregates data from all exchanges

4. **Async Backtester**: Implement a system that:
   - Loads historical data asynchronously
   - Runs multiple strategies in parallel
   - Collects results and compares their performance

## Navigation

[← Previous day](../181-tokio-runtime/en.md) | [Next day →](../183-async-await-syntax/en.md)
