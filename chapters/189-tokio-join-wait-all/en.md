# Day 189: tokio::join!: Wait for All

## Trading Analogy

Imagine you're an algo-trader, and before the market opens, you need to gather data from multiple sources simultaneously:
- Current prices from Binance
- Current prices from Kraken
- News from financial portals
- Your portfolio data

If you fetch them sequentially — each request takes ~500ms, totaling 2 seconds. But if you launch all requests **simultaneously** and wait until **all** complete — it takes just ~500ms! This is exactly what `tokio::join!` does — it runs multiple async operations in parallel and waits for **all** of them to complete.

In real trading, this is critically important:
- For arbitrage, you need prices from all exchanges at the same time
- For portfolio monitoring, you need to update all positions at once
- For market analysis, you need to gather data on all assets in parallel

## What is tokio::join!?

`tokio::join!` is a macro that:
1. Runs multiple async expressions **concurrently**
2. Waits until **all** of them complete
3. Returns a tuple with results from all operations

```
                 join!
                   |
      +------------+------------+
      |            |            |
      v            v            v
  Future 1     Future 2     Future 3
      |            |            |
      v            v            v
  Result 1     Result 2     Result 3
      |            |            |
      +------------+------------+
                   |
                   v
          (Result1, Result2, Result3)
```

## Why Not Sequential await?

Let's examine the problem with an example:

```rust
use tokio::time::{sleep, Duration};

async fn get_binance_price() -> f64 {
    sleep(Duration::from_millis(500)).await; // Simulating request
    42000.0
}

async fn get_kraken_price() -> f64 {
    sleep(Duration::from_millis(500)).await;
    42050.0
}

// SLOW: sequential execution (~1000ms)
async fn fetch_prices_sequential() -> (f64, f64) {
    let binance = get_binance_price().await;  // 500ms
    let kraken = get_kraken_price().await;    // another 500ms
    (binance, kraken)
}

// FAST: parallel execution (~500ms)
async fn fetch_prices_parallel() -> (f64, f64) {
    tokio::join!(
        get_binance_price(),
        get_kraken_price()
    )
}
```

## Basic Syntax

```rust
use tokio;

#[tokio::main]
async fn main() {
    // Basic usage
    let (result1, result2, result3) = tokio::join!(
        async_operation_1(),
        async_operation_2(),
        async_operation_3()
    );

    // With biased mode (fixed polling order)
    let (result1, result2) = tokio::join!(
        biased;
        priority_operation(),
        secondary_operation()
    );
}
```

## Practical Example: Fetching Data from Multiple Exchanges

```rust
use tokio::time::{sleep, Duration, Instant};

#[derive(Debug, Clone)]
struct PriceData {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

async fn fetch_binance_price(symbol: &str) -> PriceData {
    // Simulating Binance API request
    sleep(Duration::from_millis(150)).await;

    PriceData {
        exchange: "Binance".to_string(),
        symbol: symbol.to_string(),
        bid: 42000.0,
        ask: 42010.0,
        timestamp: 1234567890,
    }
}

async fn fetch_kraken_price(symbol: &str) -> PriceData {
    // Simulating Kraken API request
    sleep(Duration::from_millis(200)).await;

    PriceData {
        exchange: "Kraken".to_string(),
        symbol: symbol.to_string(),
        bid: 42005.0,
        ask: 42015.0,
        timestamp: 1234567890,
    }
}

async fn fetch_coinbase_price(symbol: &str) -> PriceData {
    // Simulating Coinbase API request
    sleep(Duration::from_millis(180)).await;

    PriceData {
        exchange: "Coinbase".to_string(),
        symbol: symbol.to_string(),
        bid: 41995.0,
        ask: 42008.0,
        timestamp: 1234567890,
    }
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USD";
    let start = Instant::now();

    // Fetch prices from all exchanges simultaneously
    let (binance, kraken, coinbase) = tokio::join!(
        fetch_binance_price(symbol),
        fetch_kraken_price(symbol),
        fetch_coinbase_price(symbol)
    );

    let elapsed = start.elapsed();

    println!("Prices fetched in {:?}:", elapsed);
    println!("  Binance: bid={}, ask={}", binance.bid, binance.ask);
    println!("  Kraken:  bid={}, ask={}", kraken.bid, kraken.ask);
    println!("  Coinbase: bid={}, ask={}", coinbase.bid, coinbase.ask);

    // Find best prices for arbitrage
    let best_bid = [binance.bid, kraken.bid, coinbase.bid]
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    let best_ask = [binance.ask, kraken.ask, coinbase.ask]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);

    println!("\nBest bid: {}, best ask: {}", best_bid, best_ask);
    println!("Spread: {:.2}", best_ask - best_bid);
}
```

**Output:**
```
Prices fetched in ~200ms:    (not ~530ms with sequential execution!)
  Binance: bid=42000, ask=42010
  Kraken:  bid=42005, ask=42015
  Coinbase: bid=41995, ask=42008

Best bid: 42005, best ask: 42008
Spread: 3.00
```

## Portfolio Monitoring with tokio::join!

```rust
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

#[derive(Debug, Clone)]
struct PositionStatus {
    symbol: String,
    quantity: f64,
    current_price: f64,
    pnl: f64,
    pnl_percent: f64,
}

async fn get_current_price(symbol: &str) -> f64 {
    // Simulating price fetch
    sleep(Duration::from_millis(100)).await;

    match symbol {
        "BTC" => 42500.0,
        "ETH" => 2250.0,
        "SOL" => 95.0,
        _ => 0.0,
    }
}

async fn calculate_position_status(position: Position) -> PositionStatus {
    let current_price = get_current_price(&position.symbol).await;
    let current_value = position.quantity * current_price;
    let cost_basis = position.quantity * position.avg_price;
    let pnl = current_value - cost_basis;
    let pnl_percent = (pnl / cost_basis) * 100.0;

    PositionStatus {
        symbol: position.symbol,
        quantity: position.quantity,
        current_price,
        pnl,
        pnl_percent,
    }
}

#[tokio::main]
async fn main() {
    let positions = vec![
        Position { symbol: "BTC".to_string(), quantity: 0.5, avg_price: 40000.0 },
        Position { symbol: "ETH".to_string(), quantity: 5.0, avg_price: 2000.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, avg_price: 80.0 },
    ];

    println!("Updating portfolio status...\n");

    // Update all positions simultaneously
    let (btc_status, eth_status, sol_status) = tokio::join!(
        calculate_position_status(positions[0].clone()),
        calculate_position_status(positions[1].clone()),
        calculate_position_status(positions[2].clone())
    );

    let all_statuses = vec![btc_status, eth_status, sol_status];

    println!("{:<6} {:>10} {:>12} {:>12} {:>10}",
        "Asset", "Qty", "Price", "PnL", "PnL %");
    println!("{}", "-".repeat(54));

    let mut total_pnl = 0.0;
    for status in &all_statuses {
        println!("{:<6} {:>10.4} {:>12.2} {:>12.2} {:>9.2}%",
            status.symbol,
            status.quantity,
            status.current_price,
            status.pnl,
            status.pnl_percent
        );
        total_pnl += status.pnl;
    }

    println!("{}", "-".repeat(54));
    println!("Total PnL: ${:.2}", total_pnl);
}
```

## Error Handling with tokio::join!

Important to understand: `tokio::join!` waits for **all** operations to complete, even if some fail with an error. For early termination on error, use `tokio::try_join!`.

### tokio::join! — Wait for All, Even with Errors

```rust
use tokio::time::{sleep, Duration};

async fn fetch_price(exchange: &str) -> Result<f64, String> {
    sleep(Duration::from_millis(100)).await;

    match exchange {
        "binance" => Ok(42000.0),
        "kraken" => Err("Connection timeout".to_string()),
        "coinbase" => Ok(42050.0),
        _ => Err("Unknown exchange".to_string()),
    }
}

#[tokio::main]
async fn main() {
    // join! waits for ALL results, even if there are errors
    let (binance, kraken, coinbase) = tokio::join!(
        fetch_price("binance"),
        fetch_price("kraken"),
        fetch_price("coinbase")
    );

    println!("Binance: {:?}", binance);   // Ok(42000.0)
    println!("Kraken: {:?}", kraken);     // Err("Connection timeout")
    println!("Coinbase: {:?}", coinbase); // Ok(42050.0)

    // Handle results individually
    let valid_prices: Vec<f64> = [binance, kraken, coinbase]
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    if !valid_prices.is_empty() {
        let avg = valid_prices.iter().sum::<f64>() / valid_prices.len() as f64;
        println!("Average price (excluding errors): {:.2}", avg);
    }
}
```

### tokio::try_join! — Abort on First Error

```rust
use tokio::time::{sleep, Duration};

async fn fetch_critical_price(exchange: &str) -> Result<f64, String> {
    sleep(Duration::from_millis(100)).await;

    match exchange {
        "binance" => Ok(42000.0),
        "kraken" => Err("API key expired".to_string()),
        "coinbase" => Ok(42050.0),
        _ => Err("Unknown exchange".to_string()),
    }
}

#[tokio::main]
async fn main() {
    // try_join! aborts on FIRST error
    let result = tokio::try_join!(
        fetch_critical_price("binance"),
        fetch_critical_price("kraken"),
        fetch_critical_price("coinbase")
    );

    match result {
        Ok((binance, kraken, coinbase)) => {
            println!("All prices fetched successfully!");
            println!("Binance: {}, Kraken: {}, Coinbase: {}",
                binance, kraken, coinbase);
        }
        Err(e) => {
            println!("Error fetching prices: {}", e);
            // Can implement fallback logic here
        }
    }
}
```

## Advanced Example: Trading Aggregator

```rust
use tokio::time::{sleep, Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct OrderBook {
    exchange: String,
    symbol: String,
    bids: Vec<(f64, f64)>, // (price, quantity)
    asks: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
struct MarketNews {
    source: String,
    headline: String,
    sentiment: f64, // -1.0 to 1.0
}

#[derive(Debug, Clone)]
struct TradingSignal {
    symbol: String,
    action: String,
    confidence: f64,
}

async fn fetch_order_book(exchange: &str, symbol: &str) -> OrderBook {
    sleep(Duration::from_millis(150)).await;

    OrderBook {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        bids: vec![(42000.0, 1.5), (41990.0, 2.0), (41980.0, 3.0)],
        asks: vec![(42010.0, 1.0), (42020.0, 2.5), (42030.0, 1.8)],
    }
}

async fn fetch_news(symbol: &str) -> Vec<MarketNews> {
    sleep(Duration::from_millis(200)).await;

    vec![
        MarketNews {
            source: "CryptoNews".to_string(),
            headline: format!("{} shows steady growth", symbol),
            sentiment: 0.7,
        },
        MarketNews {
            source: "TradingView".to_string(),
            headline: format!("{} technical analysis indicates trend continuation", symbol),
            sentiment: 0.5,
        },
    ]
}

async fn calculate_signals(order_books: &[OrderBook], news: &[MarketNews]) -> Vec<TradingSignal> {
    sleep(Duration::from_millis(50)).await;

    // Simple analysis based on spread and news
    let avg_sentiment: f64 = news.iter().map(|n| n.sentiment).sum::<f64>()
        / news.len() as f64;

    let mut signals = Vec::new();

    for book in order_books {
        let best_bid = book.bids.first().map(|(p, _)| *p).unwrap_or(0.0);
        let best_ask = book.asks.first().map(|(p, _)| *p).unwrap_or(0.0);
        let spread_percent = (best_ask - best_bid) / best_bid * 100.0;

        let action = if avg_sentiment > 0.5 && spread_percent < 0.05 {
            "BUY"
        } else if avg_sentiment < -0.5 {
            "SELL"
        } else {
            "HOLD"
        };

        signals.push(TradingSignal {
            symbol: book.symbol.clone(),
            action: action.to_string(),
            confidence: avg_sentiment.abs() * (1.0 - spread_percent),
        });
    }

    signals
}

#[tokio::main]
async fn main() {
    let symbol = "BTC/USD";
    let start = Instant::now();

    println!("Aggregating market data for {}...\n", symbol);

    // Fetch data from all sources in parallel
    let (binance_book, kraken_book, news) = tokio::join!(
        fetch_order_book("Binance", symbol),
        fetch_order_book("Kraken", symbol),
        fetch_news(symbol)
    );

    let order_books = vec![binance_book.clone(), kraken_book.clone()];

    // Calculate signals
    let signals = calculate_signals(&order_books, &news).await;

    let elapsed = start.elapsed();

    println!("=== Market Data (fetched in {:?}) ===\n", elapsed);

    for book in &order_books {
        println!("{} - {}", book.exchange, book.symbol);
        println!("  Best bid: {:.2}", book.bids[0].0);
        println!("  Best ask: {:.2}", book.asks[0].0);
        println!("  Spread: {:.2}\n", book.asks[0].0 - book.bids[0].0);
    }

    println!("=== News ===\n");
    for news_item in &news {
        println!("  [{}] {} (sentiment: {:.2})",
            news_item.source,
            news_item.headline,
            news_item.sentiment);
    }

    println!("\n=== Trading Signals ===\n");
    for signal in &signals {
        println!("  {} {} (confidence: {:.2}%)",
            signal.symbol,
            signal.action,
            signal.confidence * 100.0);
    }
}
```

## Comparison: join! vs select! vs spawn

| Feature | `join!` | `select!` | `spawn` |
|---------|---------|-----------|---------|
| Waits for | All operations | First completed | Doesn't wait |
| Returns | All results | One result | JoinHandle |
| Parallelism | Single thread | Single thread | Multiple threads |
| Use case | Data aggregation | Timeouts, cancellation | Background tasks |

## Practical Exercises

### Exercise 1: Multi-Exchange Arbitrage Scanner

Create a function that simultaneously fetches prices from 5 exchanges and finds arbitrage opportunities:

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct ArbitrageOpportunity {
    buy_exchange: String,
    sell_exchange: String,
    symbol: String,
    buy_price: f64,
    sell_price: f64,
    profit_percent: f64,
}

// Your implementation here
async fn find_arbitrage(symbol: &str) -> Vec<ArbitrageOpportunity> {
    todo!()
}
```

### Exercise 2: Parallel Limit Order Updates

Implement a function that simultaneously updates multiple limit orders across different exchanges:

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct LimitOrder {
    id: String,
    exchange: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct UpdateResult {
    order_id: String,
    success: bool,
    new_price: Option<f64>,
    error: Option<String>,
}

// Your implementation here
async fn update_orders_parallel(orders: Vec<LimitOrder>, new_prices: Vec<f64>)
    -> Vec<UpdateResult> {
    todo!()
}
```

### Exercise 3: Timeout for Group of Operations

Implement a function that fetches data from multiple exchanges with a shared timeout:

```rust
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
struct ExchangeData {
    exchange: String,
    price: Option<f64>,
    volume: Option<f64>,
}

// Your implementation here
async fn fetch_with_timeout(
    exchanges: Vec<&str>,
    symbol: &str,
    max_wait: Duration
) -> Result<Vec<ExchangeData>, String> {
    todo!()
}
```

### Exercise 4: Continuous Monitoring with Intervals

Implement a system that updates data from all sources every N seconds simultaneously:

```rust
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
struct MarketSnapshot {
    timestamp: u64,
    prices: Vec<(String, f64)>,
    volumes: Vec<(String, f64)>,
}

// Your implementation here
async fn run_market_monitor(
    exchanges: Vec<String>,
    symbol: String,
    update_interval: Duration,
    callback: impl Fn(MarketSnapshot)
) {
    todo!()
}
```

## Homework

1. **Liquidity Aggregator**: Create a system that collects order book data from 3+ exchanges simultaneously and builds a combined order book with best prices.

2. **Risk Monitoring**: Implement parallel checking of all portfolio positions for risk limit violations. Each check should fetch the current price and calculate potential loss.

3. **Arbitrage Bot**: Write a bot that:
   - Fetches prices from all exchanges every 100ms in parallel
   - Finds arbitrage opportunities
   - When an opportunity is found, simultaneously sends orders to both exchanges
   - Uses `try_join!` to cancel if one exchange fails

4. **Notification System**: Create a system that simultaneously sends notifications through different channels (Telegram, Email, SMS) when an alert triggers. Use `join!` to wait for confirmation from all channels.

## What We Learned

| Concept | Description |
|---------|-------------|
| `tokio::join!` | Wait for all async operations to complete |
| Parallel data fetching | Simultaneous requests to multiple sources |
| `biased` mode | Fixed polling order for futures |
| `tokio::try_join!` | Abort on first error |
| Result aggregation | Processing the tuple of results from join! |
| Concurrency vs parallelism | join! provides concurrency, spawn provides parallelism |

## Navigation

[← Previous day](../188-tokio-select-first-respond/en.md) | [Next day →](../190-tokio-time-timers-delays/en.md)
