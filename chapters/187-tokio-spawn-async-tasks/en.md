# Day 187: tokio::spawn: Async Tasks

## Trading Analogy

Imagine a trading bot that simultaneously:
- Monitors BTC prices on Binance
- Monitors BTC prices on Kraken
- Checks balances on both exchanges
- Scans for arbitrage opportunities

If we execute these tasks sequentially, we lose precious time — the arbitrage opportunity vanishes while we wait for a response from the first exchange. We need to run all these tasks **in parallel**.

`tokio::spawn` is like hiring multiple trading assistants: each handles their own task, while you (the main thread) coordinate their work and make decisions based on the data received.

## What is tokio::spawn?

`tokio::spawn` creates an **async task** that executes in parallel with other tasks in the Tokio runtime. This is not an OS thread — it's a lightweight task, and you can create thousands of them without significant overhead.

```rust
use tokio::spawn;

#[tokio::main]
async fn main() {
    // Spawn a task in the background
    let handle = spawn(async {
        // Async code runs in parallel
        println!("Task is running!");
        42
    });

    // Wait for completion and get the result
    let result = handle.await.unwrap();
    println!("Result: {}", result);
}
```

## Simple Example: Monitoring Multiple Assets

```rust
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

async fn fetch_price(symbol: &str) -> PriceUpdate {
    // Simulate exchange API request
    sleep(Duration::from_millis(100)).await;

    let price = match symbol {
        "BTC" => 42000.0 + rand_price_offset(),
        "ETH" => 2500.0 + rand_price_offset(),
        "SOL" => 100.0 + rand_price_offset(),
        _ => 0.0,
    };

    PriceUpdate {
        symbol: symbol.to_string(),
        price,
        timestamp: current_timestamp(),
    }
}

fn rand_price_offset() -> f64 {
    // Simple price fluctuation simulation
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % 1000) as f64 / 10.0
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() {
    let symbols = vec!["BTC", "ETH", "SOL"];

    // Spawn parallel tasks for each asset
    let mut handles = vec![];

    for symbol in symbols {
        let handle = tokio::spawn(async move {
            fetch_price(symbol).await
        });
        handles.push(handle);
    }

    // Collect results
    println!("=== Current Prices ===");
    for handle in handles {
        match handle.await {
            Ok(update) => {
                println!("{}: ${:.2}", update.symbol, update.price);
            }
            Err(e) => {
                println!("Error fetching price: {:?}", e);
            }
        }
    }
}
```

## JoinHandle: Managing Tasks

`tokio::spawn` returns a `JoinHandle<T>` — a task handle that allows you to:
- Wait for completion (`.await`)
- Cancel the task (`.abort()`)
- Check completion status

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Spawn a long-running task
    let handle = tokio::spawn(async {
        println!("Starting market monitoring...");

        for i in 1..=10 {
            sleep(Duration::from_millis(100)).await;
            println!("Monitoring cycle #{}", i);
        }

        "Monitoring complete"
    });

    // Wait a bit and cancel
    sleep(Duration::from_millis(350)).await;
    println!("Cancelling monitoring!");
    handle.abort();

    // Check the result
    match handle.await {
        Ok(result) => println!("Result: {}", result),
        Err(e) if e.is_cancelled() => println!("Task cancelled"),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Example: Arbitrage Scanner

```rust
use tokio::time::{sleep, Duration, timeout};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ExchangePrice {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
}

#[derive(Debug)]
struct ArbitrageOpportunity {
    symbol: String,
    buy_exchange: String,
    sell_exchange: String,
    buy_price: f64,
    sell_price: f64,
    profit_percent: f64,
}

async fn fetch_exchange_price(exchange: &str, symbol: &str) -> ExchangePrice {
    // Simulate different exchange latencies
    let delay = match exchange {
        "Binance" => 50,
        "Kraken" => 80,
        "Coinbase" => 100,
        _ => 150,
    };
    sleep(Duration::from_millis(delay)).await;

    // Simulate different prices across exchanges
    let base_price = match symbol {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        _ => 100.0,
    };

    let spread = match exchange {
        "Binance" => 0.001,   // 0.1% spread
        "Kraken" => 0.002,    // 0.2% spread
        "Coinbase" => 0.003,  // 0.3% spread
        _ => 0.005,
    };

    // Add random offset for each exchange
    let exchange_offset = match exchange {
        "Binance" => 0.0,
        "Kraken" => base_price * 0.002,  // Kraken slightly higher
        "Coinbase" => -base_price * 0.001, // Coinbase slightly lower
        _ => 0.0,
    };

    let mid_price = base_price + exchange_offset;

    ExchangePrice {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        bid: mid_price * (1.0 - spread / 2.0),
        ask: mid_price * (1.0 + spread / 2.0),
    }
}

fn find_arbitrage(prices: &[ExchangePrice]) -> Vec<ArbitrageOpportunity> {
    let mut opportunities = vec![];

    // Compare all exchange pairs
    for buy_price in prices {
        for sell_price in prices {
            if buy_price.exchange == sell_price.exchange {
                continue;
            }

            // Buy at ask on one exchange, sell at bid on another
            let profit = sell_price.bid - buy_price.ask;
            let profit_percent = (profit / buy_price.ask) * 100.0;

            if profit_percent > 0.0 {
                opportunities.push(ArbitrageOpportunity {
                    symbol: buy_price.symbol.clone(),
                    buy_exchange: buy_price.exchange.clone(),
                    sell_exchange: sell_price.exchange.clone(),
                    buy_price: buy_price.ask,
                    sell_price: sell_price.bid,
                    profit_percent,
                });
            }
        }
    }

    opportunities.sort_by(|a, b| b.profit_percent.partial_cmp(&a.profit_percent).unwrap());
    opportunities
}

#[tokio::main]
async fn main() {
    let exchanges = vec!["Binance", "Kraken", "Coinbase"];
    let symbol = "BTC";

    println!("=== Arbitrage Scanner ===\n");
    println!("Fetching {} prices from all exchanges...\n", symbol);

    // Fetch prices from all exchanges in parallel
    let mut handles = vec![];

    for exchange in &exchanges {
        let exchange = exchange.to_string();
        let symbol = symbol.to_string();

        let handle = tokio::spawn(async move {
            // Timeout for the request
            timeout(
                Duration::from_secs(1),
                fetch_exchange_price(&exchange, &symbol)
            ).await
        });

        handles.push(handle);
    }

    // Collect results
    let mut prices = vec![];

    for handle in handles {
        match handle.await {
            Ok(Ok(price)) => {
                println!("{}: bid=${:.2}, ask=${:.2}",
                    price.exchange, price.bid, price.ask);
                prices.push(price);
            }
            Ok(Err(_)) => println!("Request timeout"),
            Err(e) => println!("Task error: {:?}", e),
        }
    }

    // Analyze arbitrage opportunities
    println!("\n=== Arbitrage Opportunities ===\n");

    let opportunities = find_arbitrage(&prices);

    if opportunities.is_empty() {
        println!("No arbitrage opportunities found");
    } else {
        for opp in opportunities.iter().take(3) {
            println!("Buy on {} at ${:.2}", opp.buy_exchange, opp.buy_price);
            println!("Sell on {} at ${:.2}", opp.sell_exchange, opp.sell_price);
            println!("Profit: {:.4}%\n", opp.profit_percent);
        }
    }
}
```

## Handling Panics in Tasks

If a task panics, the panic doesn't propagate to other tasks — it's isolated. However, `JoinHandle::await` will return an error:

```rust
use tokio::time::{sleep, Duration};

async fn risky_price_fetch(symbol: &str) -> f64 {
    if symbol == "INVALID" {
        panic!("Unknown symbol!");
    }
    42000.0
}

#[tokio::main]
async fn main() {
    let symbols = vec!["BTC", "INVALID", "ETH"];

    let mut handles = vec![];

    for symbol in symbols {
        let handle = tokio::spawn(async move {
            risky_price_fetch(symbol).await
        });
        handles.push((symbol, handle));
    }

    for (symbol, handle) in handles {
        match handle.await {
            Ok(price) => println!("{}: ${:.2}", symbol, price),
            Err(e) if e.is_panic() => {
                println!("{}: PANIC in task!", symbol);
            }
            Err(e) => println!("{}: error {:?}", symbol, e),
        }
    }

    println!("\nProgram continues running!");
}
```

## Example: Multi-threaded Trading Bot

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, interval};

#[derive(Debug, Clone)]
enum TradingSignal {
    Buy { symbol: String, price: f64 },
    Sell { symbol: String, price: f64 },
    Hold { symbol: String },
}

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

// Task: fetch market data
async fn market_data_task(
    symbol: String,
    tx: mpsc::Sender<MarketData>,
) {
    let mut ticker = interval(Duration::from_millis(500));
    let mut price = match symbol.as_str() {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        _ => 100.0,
    };

    for i in 0..10 {
        ticker.tick().await;

        // Simulate price movement
        let change = (i as f64 % 3.0 - 1.0) * 10.0;
        price += change;

        let data = MarketData {
            symbol: symbol.clone(),
            price,
            volume: 1000.0 + (i as f64 * 100.0),
            timestamp: current_timestamp(),
        };

        if tx.send(data.clone()).await.is_err() {
            println!("[{}] Channel closed", symbol);
            break;
        }

        println!("[{}] Price: ${:.2}", symbol, data.price);
    }
}

// Task: analyze and generate signals
async fn signal_generator_task(
    mut rx: mpsc::Receiver<MarketData>,
    signal_tx: mpsc::Sender<TradingSignal>,
) {
    let mut last_prices: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    while let Some(data) = rx.recv().await {
        let signal = if let Some(&last_price) = last_prices.get(&data.symbol) {
            let change_percent = ((data.price - last_price) / last_price) * 100.0;

            if change_percent > 0.02 {
                TradingSignal::Sell {
                    symbol: data.symbol.clone(),
                    price: data.price,
                }
            } else if change_percent < -0.02 {
                TradingSignal::Buy {
                    symbol: data.symbol.clone(),
                    price: data.price,
                }
            } else {
                TradingSignal::Hold {
                    symbol: data.symbol.clone(),
                }
            }
        } else {
            TradingSignal::Hold {
                symbol: data.symbol.clone(),
            }
        };

        last_prices.insert(data.symbol, data.price);

        if signal_tx.send(signal).await.is_err() {
            break;
        }
    }
}

// Task: execute trading signals
async fn execution_task(
    mut signal_rx: mpsc::Receiver<TradingSignal>,
) {
    let mut position: std::collections::HashMap<String, f64> =
        std::collections::HashMap::new();

    while let Some(signal) = signal_rx.recv().await {
        match signal {
            TradingSignal::Buy { symbol, price } => {
                let qty = 0.1;
                *position.entry(symbol.clone()).or_insert(0.0) += qty;
                println!(">>> BUY {} at ${:.2} (position: {:.2})",
                    symbol, price, position.get(&symbol).unwrap_or(&0.0));
            }
            TradingSignal::Sell { symbol, price } => {
                if let Some(pos) = position.get_mut(&symbol) {
                    if *pos > 0.0 {
                        let qty = (*pos).min(0.1);
                        *pos -= qty;
                        println!(">>> SELL {} at ${:.2} (position: {:.2})",
                            symbol, price, pos);
                    }
                }
            }
            TradingSignal::Hold { symbol } => {
                // Do nothing
            }
        }
    }

    println!("\n=== Final Positions ===");
    for (symbol, qty) in &position {
        println!("{}: {:.4}", symbol, qty);
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() {
    println!("=== Trading Bot Started ===\n");

    // Channels for inter-task communication
    let (data_tx, data_rx) = mpsc::channel::<MarketData>(100);
    let (signal_tx, signal_rx) = mpsc::channel::<TradingSignal>(100);

    // Spawn data collection tasks
    let btc_tx = data_tx.clone();
    let btc_task = tokio::spawn(async move {
        market_data_task("BTC".to_string(), btc_tx).await;
    });

    let eth_tx = data_tx.clone();
    let eth_task = tokio::spawn(async move {
        market_data_task("ETH".to_string(), eth_tx).await;
    });

    // Close the original sender
    drop(data_tx);

    // Spawn signal analyzer
    let signal_task = tokio::spawn(async move {
        signal_generator_task(data_rx, signal_tx).await;
    });

    // Spawn executor
    let execution_task = tokio::spawn(async move {
        execution_task(signal_rx).await;
    });

    // Wait for all tasks to complete
    let _ = tokio::join!(btc_task, eth_task, signal_task, execution_task);

    println!("\n=== Trading Bot Stopped ===");
}
```

## tokio::spawn vs async block

| Characteristic | `tokio::spawn` | async block |
|----------------|----------------|-------------|
| Execution | Parallel | Sequential (by default) |
| Data requirements | `'static + Send` | Any references |
| Cancellation | Via `.abort()` | Only via drop |
| Panic | Isolated | Propagates |

```rust
#[tokio::main]
async fn main() {
    let data = vec![1, 2, 3];

    // async block — can use reference to data
    let result1 = async {
        data.iter().sum::<i32>()
    }.await;

    // spawn — data must be 'static
    let data_owned = data.clone();
    let handle = tokio::spawn(async move {
        data_owned.iter().sum::<i32>()
    });
    let result2 = handle.await.unwrap();

    println!("Results: {} and {}", result1, result2);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `tokio::spawn` | Create a parallel async task |
| `JoinHandle` | Handle for managing the task |
| `.await` on JoinHandle | Wait for task completion |
| `.abort()` | Cancel a running task |
| Panic isolation | Panic in one task doesn't affect others |
| `'static + Send` | Requirements for data in spawned tasks |

## Homework

1. **Parallel Price Monitor**: Create a program that monitors prices of 5 different cryptocurrencies in parallel. Each task should update the price every 500ms. The main thread should collect all updates and print a summary table every 2 seconds.

2. **Arbitrage Bot with Timeouts**: Extend the arbitrage scanner from the example:
   - Add 5 exchanges
   - Set different timeouts for each exchange
   - If an exchange doesn't respond — skip it and continue
   - Log all timeouts and errors

3. **Risk Management System**: Implement a system with three parallel tasks:
   - **Price Monitor**: watches the price and sends updates
   - **Risk Calculator**: receives updates and calculates current position risk
   - **Alert System**: receives risk level and sends alerts if risk exceeds threshold

4. **Graceful Shutdown**: Modify the trading bot:
   - Add cancellation signal handling (use `tokio::select!` and `tokio_util::sync::CancellationToken`)
   - When signal is received, all tasks should gracefully complete their work
   - Save the final state before exiting

## Navigation

[← Previous day](../186-tokio-main-entry-point/en.md) | [Next day →](../188-join-select-await/en.md)
