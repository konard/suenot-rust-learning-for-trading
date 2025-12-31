# Day 183: async/await Syntax

## Trading Analogy

Imagine you're running a trading bot that needs to simultaneously:
- Fetch quotes from 5 different exchanges
- Track your open positions
- Check risk limits
- Send notifications

**Synchronous approach:** The bot sequentially queries each exchange, waits for a response, then moves to the next one... While waiting for Binance's response, the price on Kraken has already changed!

**Asynchronous approach with async/await:** The bot sends all requests at once and processes responses as they arrive. While waiting for one exchange, we work with data from another.

It's like an experienced trader watching multiple monitors simultaneously rather than looking at them one by one.

## What is async/await?

`async` and `await` are syntactic sugar for working with asynchronous code in Rust:

- **async** — declares a function or block as asynchronous
- **await** — suspends execution until the Future produces a result

```rust
// Synchronous function — blocks the thread
fn fetch_price_sync() -> f64 {
    // Waiting for response, doing nothing
    42000.0
}

// Asynchronous function — returns a Future
async fn fetch_price_async() -> f64 {
    // Allows other tasks to execute while waiting
    42000.0
}
```

## Basic Example: Fetching Asset Price

```rust
use tokio::time::{sleep, Duration};

// Simulating an API request to an exchange
async fn fetch_btc_price(exchange: &str) -> f64 {
    println!("[{}] Requesting BTC price...", exchange);

    // Simulating network delay (1 second)
    sleep(Duration::from_millis(1000)).await;

    // Return "price" depending on exchange
    match exchange {
        "Binance" => 42150.0,
        "Kraken" => 42145.0,
        "Coinbase" => 42155.0,
        _ => 42000.0,
    }
}

#[tokio::main]
async fn main() {
    println!("Fetching BTC price from Binance...");

    let price = fetch_btc_price("Binance").await;

    println!("BTC price: ${:.2}", price);
}
```

## Parallel Execution with tokio::join!

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price(symbol: &str, exchange: &str) -> f64 {
    println!("[{}] Requesting {}...", exchange, symbol);
    sleep(Duration::from_millis(500)).await;

    match (symbol, exchange) {
        ("BTC", "Binance") => 42150.0,
        ("BTC", "Kraken") => 42145.0,
        ("ETH", "Binance") => 2250.0,
        ("ETH", "Kraken") => 2248.0,
        _ => 0.0,
    }
}

#[tokio::main]
async fn main() {
    println!("=== Sequential Execution ===");
    let start = std::time::Instant::now();

    let btc_binance = fetch_price("BTC", "Binance").await;
    let btc_kraken = fetch_price("BTC", "Kraken").await;
    let eth_binance = fetch_price("ETH", "Binance").await;
    let eth_kraken = fetch_price("ETH", "Kraken").await;

    println!("Sequential took: {:?}", start.elapsed());

    println!("\n=== Parallel Execution ===");
    let start = std::time::Instant::now();

    // tokio::join! executes all futures in parallel
    let (btc_b, btc_k, eth_b, eth_k) = tokio::join!(
        fetch_price("BTC", "Binance"),
        fetch_price("BTC", "Kraken"),
        fetch_price("ETH", "Binance"),
        fetch_price("ETH", "Kraken")
    );

    println!("Parallel took: {:?}", start.elapsed());

    println!("\nPrices:");
    println!("BTC Binance: ${:.2}", btc_b);
    println!("BTC Kraken: ${:.2}", btc_k);
    println!("ETH Binance: ${:.2}", eth_b);
    println!("ETH Kraken: ${:.2}", eth_k);
}
```

## Error Handling in async Functions

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct ExchangeError {
    exchange: String,
    message: String,
}

async fn fetch_price_safe(symbol: &str, exchange: &str) -> Result<f64, ExchangeError> {
    println!("[{}] Requesting {}...", exchange, symbol);
    sleep(Duration::from_millis(300)).await;

    // Simulate random errors
    if exchange == "OfflineExchange" {
        return Err(ExchangeError {
            exchange: exchange.to_string(),
            message: "Exchange unavailable".to_string(),
        });
    }

    match (symbol, exchange) {
        ("BTC", "Binance") => Ok(42150.0),
        ("BTC", "Kraken") => Ok(42145.0),
        ("ETH", "Binance") => Ok(2250.0),
        _ => Err(ExchangeError {
            exchange: exchange.to_string(),
            message: format!("Symbol {} not found", symbol),
        }),
    }
}

#[tokio::main]
async fn main() {
    // Handling Result with ? operator
    match get_best_btc_price().await {
        Ok(price) => println!("Best BTC price: ${:.2}", price),
        Err(e) => println!("Error: {:?}", e),
    }
}

async fn get_best_btc_price() -> Result<f64, ExchangeError> {
    let binance_price = fetch_price_safe("BTC", "Binance").await?;
    let kraken_price = fetch_price_safe("BTC", "Kraken").await?;

    Ok(binance_price.min(kraken_price))
}
```

## tokio::select! — First Ready Result

In trading, we often need the fastest response:

```rust
use tokio::time::{sleep, Duration};

async fn fetch_from_binance() -> f64 {
    sleep(Duration::from_millis(150)).await;
    42150.0
}

async fn fetch_from_kraken() -> f64 {
    sleep(Duration::from_millis(100)).await;
    42145.0
}

async fn fetch_from_coinbase() -> f64 {
    sleep(Duration::from_millis(200)).await;
    42155.0
}

#[tokio::main]
async fn main() {
    println!("Waiting for first response from any exchange...");

    let price = tokio::select! {
        price = fetch_from_binance() => {
            println!("Binance responded first!");
            price
        }
        price = fetch_from_kraken() => {
            println!("Kraken responded first!");
            price
        }
        price = fetch_from_coinbase() => {
            println!("Coinbase responded first!");
            price
        }
    };

    println!("Received price: ${:.2}", price);
}
```

## Practical Example: Portfolio Monitoring

```rust
use tokio::time::{sleep, Duration, interval};

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

#[derive(Debug)]
struct PositionStatus {
    symbol: String,
    current_price: f64,
    pnl: f64,
    pnl_percent: f64,
}

async fn fetch_current_price(symbol: &str) -> f64 {
    // Simulate API request
    sleep(Duration::from_millis(100)).await;

    match symbol {
        "BTC" => 42500.0 + (rand_simple() * 1000.0 - 500.0),
        "ETH" => 2300.0 + (rand_simple() * 100.0 - 50.0),
        "SOL" => 95.0 + (rand_simple() * 10.0 - 5.0),
        _ => 100.0,
    }
}

// Simple pseudo-random number generator
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

async fn check_position(position: &Position) -> PositionStatus {
    let current_price = fetch_current_price(&position.symbol).await;
    let pnl = (current_price - position.entry_price) * position.quantity;
    let pnl_percent = ((current_price / position.entry_price) - 1.0) * 100.0;

    PositionStatus {
        symbol: position.symbol.clone(),
        current_price,
        pnl,
        pnl_percent,
    }
}

async fn monitor_portfolio(positions: Vec<Position>) {
    println!("=== Portfolio Monitor ===\n");

    // Check all positions in parallel
    let mut futures = Vec::new();
    for position in &positions {
        futures.push(check_position(position));
    }

    let results = futures::future::join_all(futures).await;

    let mut total_pnl = 0.0;

    for status in results {
        let sign = if status.pnl >= 0.0 { "+" } else { "" };
        println!(
            "{}: ${:.2} | PnL: {}${:.2} ({}{:.2}%)",
            status.symbol,
            status.current_price,
            sign,
            status.pnl,
            sign,
            status.pnl_percent
        );
        total_pnl += status.pnl;
    }

    println!("\nTotal PnL: ${:.2}", total_pnl);
}

#[tokio::main]
async fn main() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 0.5, entry_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 5.0, entry_price: 2200.0 },
        Position { symbol: "SOL".to_string(), quantity: 50.0, entry_price: 90.0 },
    ];

    monitor_portfolio(portfolio).await;
}
```

## Timeouts for Trading Operations

```rust
use tokio::time::{timeout, Duration};

async fn place_order(symbol: &str, side: &str, quantity: f64) -> Result<String, String> {
    // Simulate order placement
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(format!("ORDER-{}-{}-{}", symbol, side, quantity))
}

async fn place_order_with_timeout(
    symbol: &str,
    side: &str,
    quantity: f64,
    timeout_ms: u64
) -> Result<String, String> {
    let order_future = place_order(symbol, side, quantity);

    match timeout(Duration::from_millis(timeout_ms), order_future).await {
        Ok(result) => result,
        Err(_) => Err(format!(
            "Timeout placing order {} {} {}",
            side, quantity, symbol
        )),
    }
}

#[tokio::main]
async fn main() {
    // Successful order
    match place_order_with_timeout("BTC", "BUY", 0.1, 1000).await {
        Ok(order_id) => println!("Order placed: {}", order_id),
        Err(e) => println!("Error: {}", e),
    }

    // Order with timeout
    match place_order_with_timeout("ETH", "SELL", 1.0, 100).await {
        Ok(order_id) => println!("Order placed: {}", order_id),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Periodic Tasks: Price Updates

```rust
use tokio::time::{interval, Duration};

#[derive(Debug)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

async fn fetch_and_log_price(symbol: &str) -> PriceUpdate {
    let price = match symbol {
        "BTC" => 42000.0 + (rand_simple() * 100.0),
        "ETH" => 2200.0 + (rand_simple() * 10.0),
        _ => 100.0,
    };

    PriceUpdate {
        symbol: symbol.to_string(),
        price,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

#[tokio::main]
async fn main() {
    let mut price_interval = interval(Duration::from_secs(1));
    let mut update_count = 0;

    println!("Starting price monitoring (5 updates)...\n");

    loop {
        price_interval.tick().await;

        let (btc, eth) = tokio::join!(
            fetch_and_log_price("BTC"),
            fetch_and_log_price("ETH")
        );

        println!("[{}] BTC: ${:.2} | ETH: ${:.2}",
            btc.timestamp, btc.price, eth.price);

        update_count += 1;
        if update_count >= 5 {
            break;
        }
    }

    println!("\nMonitoring complete.");
}
```

## Spawn: Background Execution

```rust
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;

#[derive(Debug)]
enum TradeSignal {
    Buy { symbol: String, price: f64 },
    Sell { symbol: String, price: f64 },
}

async fn price_monitor(symbol: String, tx: mpsc::Sender<TradeSignal>) {
    let mut last_price = 42000.0;

    for _ in 0..5 {
        sleep(Duration::from_millis(500)).await;

        // Simulate price change
        let new_price = last_price + (rand_simple() * 200.0 - 100.0);

        // Generate signal on significant change
        if new_price > last_price * 1.001 {
            let _ = tx.send(TradeSignal::Buy {
                symbol: symbol.clone(),
                price: new_price,
            }).await;
        } else if new_price < last_price * 0.999 {
            let _ = tx.send(TradeSignal::Sell {
                symbol: symbol.clone(),
                price: new_price,
            }).await;
        }

        last_price = new_price;
    }
}

fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<TradeSignal>(100);

    // Start monitors in background
    let btc_tx = tx.clone();
    let eth_tx = tx.clone();

    tokio::spawn(async move {
        price_monitor("BTC".to_string(), btc_tx).await;
    });

    tokio::spawn(async move {
        price_monitor("ETH".to_string(), eth_tx).await;
    });

    // Close original sender
    drop(tx);

    println!("Waiting for trade signals...\n");

    while let Some(signal) = rx.recv().await {
        match signal {
            TradeSignal::Buy { symbol, price } => {
                println!("[BUY]  {} @ ${:.2}", symbol, price);
            }
            TradeSignal::Sell { symbol, price } => {
                println!("[SELL] {} @ ${:.2}", symbol, price);
            }
        }
    }

    println!("\nAll monitors completed.");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `async fn` | Declares an asynchronous function returning a Future |
| `.await` | Suspends execution until the Future is ready |
| `tokio::join!` | Executes multiple futures in parallel |
| `tokio::select!` | Returns the first ready result |
| `tokio::spawn` | Runs a task in the background |
| `timeout` | Limits waiting time |
| `interval` | Creates a periodic timer |

## Homework

1. **Arbitrage Scanner**: Create an async function that fetches BTC prices from 5 exchanges in parallel and finds arbitrage opportunities (difference > 0.5%).

2. **Order with Retries**: Implement a `place_order_with_retry` function that:
   - Attempts to place an order
   - On error, retries up to 3 times with exponential backoff
   - Uses timeout for each attempt

3. **Streaming Position Monitor**: Write a program that:
   - Every second fetches prices for a portfolio of 10 assets
   - Calculates total PnL
   - Sends a notification when PnL changes by more than 5%

4. **Concurrent Execution Engine**: Create an `OrderExecutor` struct with a method:
   ```rust
   async fn execute_batch(&self, orders: Vec<Order>) -> Vec<Result<OrderResult, Error>>
   ```
   That executes up to 5 orders in parallel (use Semaphore).

## Navigation

[← Previous day](../182-tokio-runtime/en.md) | [Next day →](../184-futures-and-streams/en.md)
