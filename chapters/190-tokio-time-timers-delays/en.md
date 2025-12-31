# Day 190: tokio::time: Timers and Delays

## Trading Analogy

Imagine you're working as a trader on an exchange. Time is your most important tool:

- **Delay (sleep)** — like a pause between price checks. You don't want to spam the exchange with requests every millisecond, so you wait a second before the next request.
- **Timeout** — like a time-based stop-loss. If the exchange doesn't respond within 5 seconds — something went wrong.
- **Interval** — like regular portfolio monitoring. Every 10 seconds you check your current positions.
- **Instant** — like a precise timestamp for measuring order execution speed.

In asynchronous programming, working with time differs from synchronous code: instead of blocking the thread, we "suspend" the task, allowing other tasks to work.

## tokio::time Basics

`tokio::time` provides a set of tools for working with time in asynchronous code:

| Tool | Purpose | Trading Analogy |
|------|---------|-----------------|
| `sleep` | Pause for a given time | Wait before next API request |
| `timeout` | Limit operation time | Cancel order if not filled in N seconds |
| `interval` | Periodic execution | Check portfolio every 10 seconds |
| `Instant` | Precise time measurement | Measure execution speed |

## sleep: Asynchronous Pause

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Fetching BTC price...");
    let price = get_btc_price().await;
    println!("BTC price: ${}", price);

    // Wait 1 second before next request (rate limiting)
    println!("Waiting before next request...");
    sleep(Duration::from_secs(1)).await;

    println!("Fetching price again...");
    let new_price = get_btc_price().await;
    println!("New BTC price: ${}", new_price);
}

async fn get_btc_price() -> f64 {
    // Simulating API request
    sleep(Duration::from_millis(100)).await;
    42_000.50
}
```

### Key Difference from std::thread::sleep

```rust
use tokio::time::{sleep, Duration};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let start = Instant::now();

    // These tasks run in PARALLEL!
    let (price1, price2, price3) = tokio::join!(
        fetch_price_with_delay("BTC", 1000),
        fetch_price_with_delay("ETH", 1500),
        fetch_price_with_delay("SOL", 800),
    );

    println!("BTC: ${}, ETH: ${}, SOL: ${}", price1, price2, price3);
    println!("Total time: {:?}", start.elapsed());
    // Will output ~1.5 seconds, not 3.3 seconds!
}

async fn fetch_price_with_delay(symbol: &str, delay_ms: u64) -> f64 {
    // Simulating network delay
    sleep(Duration::from_millis(delay_ms)).await;

    match symbol {
        "BTC" => 42_000.0,
        "ETH" => 2_200.0,
        "SOL" => 95.0,
        _ => 0.0,
    }
}
```

**Key distinction:**
- `std::thread::sleep` blocks the entire thread — nobody else can work
- `tokio::time::sleep` yields control to the runtime — other tasks continue working

## timeout: Limiting Wait Time

Timeout is critical in trading — if the exchange isn't responding, you need to make a quick decision.

```rust
use tokio::time::{timeout, Duration, sleep};

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

#[tokio::main]
async fn main() {
    let order = Order {
        id: 1,
        symbol: "BTC".to_string(),
        price: 42_000.0,
        quantity: 0.1,
    };

    println!("Sending order #{} to exchange...", order.id);

    // Give 5 seconds for order execution
    match timeout(Duration::from_secs(5), execute_order(&order)).await {
        Ok(Ok(trade_id)) => {
            println!("Order executed! Trade ID: {}", trade_id);
        }
        Ok(Err(e)) => {
            println!("Execution error: {}", e);
        }
        Err(_) => {
            println!("TIMEOUT! Order not executed within 5 seconds.");
            println!("Cancelling order #{}", order.id);
            cancel_order(order.id).await;
        }
    }
}

async fn execute_order(order: &Order) -> Result<u64, String> {
    // Simulating slow exchange (6 seconds — longer than timeout)
    sleep(Duration::from_secs(6)).await;
    Ok(12345)
}

async fn cancel_order(order_id: u64) {
    println!("Order {} cancelled", order_id);
}
```

### Timeout with Retry Logic

```rust
use tokio::time::{timeout, Duration, sleep};

async fn fetch_price_with_retry(symbol: &str, max_retries: u32) -> Result<f64, String> {
    for attempt in 1..=max_retries {
        println!("Attempt {} to get {} price...", attempt, symbol);

        match timeout(Duration::from_secs(2), fetch_price(symbol)).await {
            Ok(Ok(price)) => {
                return Ok(price);
            }
            Ok(Err(e)) => {
                println!("API error: {}", e);
            }
            Err(_) => {
                println!("Timeout fetching price");
            }
        }

        if attempt < max_retries {
            // Exponential backoff between attempts
            let delay = Duration::from_millis(100 * 2u64.pow(attempt - 1));
            println!("Waiting {:?} before next attempt...", delay);
            sleep(delay).await;
        }
    }

    Err(format!("Failed to get {} price after {} attempts", symbol, max_retries))
}

async fn fetch_price(symbol: &str) -> Result<f64, String> {
    // Simulating unstable API
    sleep(Duration::from_millis(500)).await;

    // 30% chance of error
    if rand::random::<f32>() < 0.3 {
        return Err("API temporarily unavailable".to_string());
    }

    Ok(42_000.0)
}
```

## interval: Periodic Tasks

Intervals are perfect for regular monitoring in trading.

```rust
use tokio::time::{interval, Duration, Instant};

#[derive(Debug)]
struct Portfolio {
    btc_balance: f64,
    eth_balance: f64,
    usdt_balance: f64,
}

#[tokio::main]
async fn main() {
    // Create a 5-second interval
    let mut interval = interval(Duration::from_secs(5));

    let portfolio = Portfolio {
        btc_balance: 1.5,
        eth_balance: 10.0,
        usdt_balance: 50_000.0,
    };

    println!("Starting portfolio monitoring...");

    // Monitor for 30 seconds (6 ticks)
    for tick_number in 1..=6 {
        // tick() waits until next interval
        interval.tick().await;

        let btc_price = get_price("BTC").await;
        let eth_price = get_price("ETH").await;

        let total_value = portfolio.btc_balance * btc_price
            + portfolio.eth_balance * eth_price
            + portfolio.usdt_balance;

        println!(
            "[Tick {}] Portfolio value: ${:.2}",
            tick_number, total_value
        );
    }

    println!("Monitoring finished");
}

async fn get_price(symbol: &str) -> f64 {
    // Simulating price fetch with small variation
    let base_price = match symbol {
        "BTC" => 42_000.0,
        "ETH" => 2_200.0,
        _ => 1.0,
    };

    // Add random variation ±1%
    let variation = (rand::random::<f64>() - 0.5) * 0.02;
    base_price * (1.0 + variation)
}
```

### MissedTickBehavior: Handling Missed Ticks

```rust
use tokio::time::{interval, Duration, MissedTickBehavior, sleep};

#[tokio::main]
async fn main() {
    // Interval every 100ms
    let mut price_check = interval(Duration::from_millis(100));

    // What to do if processing takes longer than interval?
    // Burst — execute missed ticks as fast as possible
    // Delay — skip and continue from new interval (default)
    // Skip — skip missed ticks
    price_check.set_missed_tick_behavior(MissedTickBehavior::Skip);

    for i in 1..=10 {
        price_check.tick().await;
        println!("Price check #{}", i);

        // Sometimes processing takes longer
        if i == 3 {
            println!("Long processing...");
            sleep(Duration::from_millis(350)).await; // Will miss ~3 ticks
        }
    }
}
```

## Instant: Precise Time Measurement

In trading, milliseconds matter — you need to precisely measure latencies.

```rust
use tokio::time::{Instant, sleep, Duration};

#[derive(Debug)]
struct OrderExecutionMetrics {
    order_id: u64,
    submission_time: Duration,
    confirmation_time: Duration,
    total_latency: Duration,
}

#[tokio::main]
async fn main() {
    let metrics = measure_order_latency().await;

    println!("=== Order #{} Execution Metrics ===", metrics.order_id);
    println!("Submission time: {:?}", metrics.submission_time);
    println!("Confirmation time: {:?}", metrics.confirmation_time);
    println!("Total latency: {:?}", metrics.total_latency);

    // Alert if latency is too high
    if metrics.total_latency > Duration::from_millis(500) {
        println!("WARNING: High latency! Check your connection.");
    }
}

async fn measure_order_latency() -> OrderExecutionMetrics {
    let start = Instant::now();

    // Phase 1: Submit order
    submit_order().await;
    let submission_time = start.elapsed();

    // Phase 2: Wait for confirmation
    let confirmation_start = Instant::now();
    wait_for_confirmation().await;
    let confirmation_time = confirmation_start.elapsed();

    OrderExecutionMetrics {
        order_id: 1,
        submission_time,
        confirmation_time,
        total_latency: start.elapsed(),
    }
}

async fn submit_order() {
    sleep(Duration::from_millis(50)).await;
}

async fn wait_for_confirmation() {
    sleep(Duration::from_millis(150)).await;
}
```

## Practical Example: Rate Limiter for Exchange API

Exchanges limit the number of requests. Here's how to properly implement rate limiting:

```rust
use tokio::time::{sleep, Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;

struct RateLimiter {
    requests_per_second: u32,
    last_request: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(requests_per_second: u32) -> Self {
        RateLimiter {
            requests_per_second,
            last_request: Mutex::new(Instant::now()),
            min_interval: Duration::from_secs(1) / requests_per_second,
        }
    }

    async fn wait(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();

        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            sleep(wait_time).await;
        }

        *last = Instant::now();
    }
}

struct ExchangeClient {
    rate_limiter: Arc<RateLimiter>,
}

impl ExchangeClient {
    fn new(max_requests_per_second: u32) -> Self {
        ExchangeClient {
            rate_limiter: Arc::new(RateLimiter::new(max_requests_per_second)),
        }
    }

    async fn get_price(&self, symbol: &str) -> f64 {
        // Wait if needed to respect rate limit
        self.rate_limiter.wait().await;

        // Make request
        println!("Fetching {} price", symbol);
        sleep(Duration::from_millis(50)).await; // Simulating network request

        42_000.0
    }

    async fn get_order_book(&self, symbol: &str) -> Vec<(f64, f64)> {
        self.rate_limiter.wait().await;

        println!("Fetching {} order book", symbol);
        sleep(Duration::from_millis(100)).await;

        vec![(42_000.0, 1.5), (41_999.0, 2.0), (41_998.0, 0.5)]
    }
}

#[tokio::main]
async fn main() {
    // Exchange allows 5 requests per second
    let client = ExchangeClient::new(5);

    let start = Instant::now();

    // Make 10 requests — should take ~2 seconds
    for i in 1..=10 {
        client.get_price("BTC").await;
        println!("Request {} completed, elapsed {:?}", i, start.elapsed());
    }

    println!("All requests completed in {:?}", start.elapsed());
}
```

## Practical Example: Monitoring with Timeouts

```rust
use tokio::time::{interval, timeout, Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: Instant,
}

#[derive(Debug)]
enum TradingEvent {
    PriceUpdate(PriceUpdate),
    ConnectionLost,
    Reconnected,
}

async fn price_monitor(tx: mpsc::Sender<TradingEvent>) {
    let symbols = vec!["BTC", "ETH", "SOL"];
    let mut check_interval = interval(Duration::from_secs(1));

    loop {
        check_interval.tick().await;

        for symbol in &symbols {
            // Give 500ms to fetch price
            let result = timeout(
                Duration::from_millis(500),
                fetch_price_from_exchange(symbol)
            ).await;

            match result {
                Ok(Ok(price)) => {
                    let update = PriceUpdate {
                        symbol: symbol.to_string(),
                        price,
                        timestamp: Instant::now(),
                    };
                    tx.send(TradingEvent::PriceUpdate(update)).await.ok();
                }
                Ok(Err(_)) | Err(_) => {
                    println!("Problem fetching {} price", symbol);
                    tx.send(TradingEvent::ConnectionLost).await.ok();
                }
            }
        }
    }
}

async fn fetch_price_from_exchange(symbol: &str) -> Result<f64, String> {
    // Simulating network request
    tokio::time::sleep(Duration::from_millis(100)).await;

    match symbol {
        "BTC" => Ok(42_000.0 + rand::random::<f64>() * 100.0),
        "ETH" => Ok(2_200.0 + rand::random::<f64>() * 10.0),
        "SOL" => Ok(95.0 + rand::random::<f64>() * 2.0),
        _ => Err("Unknown symbol".to_string()),
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);

    // Start price monitor in separate task
    tokio::spawn(price_monitor(tx));

    // Process events for 10 seconds
    let deadline = Instant::now() + Duration::from_secs(10);

    while Instant::now() < deadline {
        match timeout(Duration::from_secs(2), rx.recv()).await {
            Ok(Some(event)) => {
                match event {
                    TradingEvent::PriceUpdate(update) => {
                        println!(
                            "{}: ${:.2}",
                            update.symbol, update.price
                        );
                    }
                    TradingEvent::ConnectionLost => {
                        println!("Connection lost!");
                    }
                    TradingEvent::Reconnected => {
                        println!("Connection restored");
                    }
                }
            }
            Ok(None) => {
                println!("Channel closed");
                break;
            }
            Err(_) => {
                println!("No events for 2 seconds");
            }
        }
    }

    println!("Monitoring finished");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `sleep` | Asynchronous pause without blocking the thread |
| `timeout` | Limiting operation execution time |
| `interval` | Periodic task execution |
| `Instant` | Precise elapsed time measurement |
| `MissedTickBehavior` | Strategy for handling missed intervals |
| Rate Limiting | Controlling API request frequency |

## Homework

1. **Smart Price Monitor**: Create a system that:
   - Checks BTC price every 5 seconds
   - If price changes more than 1% — outputs an alert
   - If API doesn't respond 3 times in a row — switches to reduced frequency mode (every 30 seconds)

2. **Rate Limiter with Burst**: Modify `RateLimiter` to:
   - Allow "accumulating" unused requests (up to 10)
   - Send a burst of accumulated requests when needed

3. **Latency Measurement**: Write a system that:
   - Measures average, minimum, and maximum API latency
   - Keeps statistics for the last 100 requests
   - Outputs a report every 30 seconds

4. **Timeout with Graceful Degradation**: Implement a price fetching function that:
   - First tries the fast API (100ms timeout)
   - On failure, tries the backup API (500ms timeout)
   - On complete failure, returns the last known price

## Navigation

[← Previous day](../189-tokio-join/en.md) | [Next day →](../191-tokio-timeout/en.md)
