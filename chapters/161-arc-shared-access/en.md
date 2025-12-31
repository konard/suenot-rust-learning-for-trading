# Day 161: Arc: Shared Access Between Threads

## Trading Analogy

Imagine a trading platform with multiple modules:
- Price analysis module
- Risk management module
- Order execution module
- Monitoring module

All these modules need access to **the same market data** — current price, price history, order book state. Each module runs in its own thread for maximum performance.

In Rust, regular references (`&T`) cannot outlive the thread that created them. For safe shared ownership of data across threads, we need **Arc** — Atomic Reference Counted.

## The Problem: Data Between Threads

```rust
use std::thread;

fn main() {
    let market_data = vec![42000.0, 42100.0, 42050.0];

    // ERROR! market_data cannot be used in another thread
    // let handle = thread::spawn(|| {
    //     println!("Price: {}", market_data[0]);
    // });

    // This won't compile: `market_data` doesn't implement `Send`
}
```

## Solution with Arc

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    // Wrap data in Arc
    let market_data = Arc::new(vec![42000.0, 42100.0, 42050.0]);

    // Create Arc clones (not data clones!) for each thread
    let data_for_analyzer = Arc::clone(&market_data);
    let data_for_risk = Arc::clone(&market_data);

    let analyzer = thread::spawn(move || {
        let avg: f64 = data_for_analyzer.iter().sum::<f64>()
            / data_for_analyzer.len() as f64;
        println!("Analyzer: Average price = ${:.2}", avg);
    });

    let risk_manager = thread::spawn(move || {
        let max = data_for_risk.iter().cloned().fold(0.0_f64, f64::max);
        let min = data_for_risk.iter().cloned().fold(f64::MAX, f64::min);
        println!("Risk: Range = ${:.2} - ${:.2}", min, max);
    });

    // Wait for threads to complete
    analyzer.join().unwrap();
    risk_manager.join().unwrap();

    // Original Arc is still available
    println!("Main: {} prices available", market_data.len());
}
```

Output:
```
Analyzer: Average price = $42050.00
Risk: Range = $42000.00 - $42100.00
Main: 3 prices available
```

## How Arc Works

```rust
use std::sync::Arc;

fn main() {
    let price = Arc::new(42000.0_f64);
    println!("Reference count: {}", Arc::strong_count(&price)); // 1

    let price2 = Arc::clone(&price);
    println!("Reference count: {}", Arc::strong_count(&price)); // 2

    {
        let price3 = Arc::clone(&price);
        println!("Reference count: {}", Arc::strong_count(&price)); // 3
    } // price3 goes out of scope

    println!("Reference count: {}", Arc::strong_count(&price)); // 2

    // Data is freed only when count = 0
}
```

## Arc vs Rc

| Characteristic | `Rc<T>` | `Arc<T>` |
|----------------|---------|----------|
| Thread safety | No | Yes |
| Performance | Faster | Slower (atomic operations) |
| Use case | Single thread | Multiple threads |
| Trait | `!Send`, `!Sync` | `Send + Sync` (if T: Send + Sync) |

## Practical Example: Multi-Exchange Monitor

```rust
use std::sync::Arc;
use std::thread;
use std::collections::HashMap;

#[derive(Debug)]
struct ExchangeData {
    name: String,
    btc_price: f64,
    eth_price: f64,
    volume_24h: f64,
}

fn main() {
    // Data from multiple exchanges
    let exchanges = Arc::new(vec![
        ExchangeData {
            name: "Binance".to_string(),
            btc_price: 42000.0,
            eth_price: 2500.0,
            volume_24h: 1_000_000.0,
        },
        ExchangeData {
            name: "Coinbase".to_string(),
            btc_price: 42050.0,
            eth_price: 2510.0,
            volume_24h: 500_000.0,
        },
        ExchangeData {
            name: "Kraken".to_string(),
            btc_price: 41980.0,
            eth_price: 2495.0,
            volume_24h: 300_000.0,
        },
    ]);

    // Module 1: Find best BTC price
    let data_for_btc = Arc::clone(&exchanges);
    let btc_analyzer = thread::spawn(move || {
        let best = data_for_btc
            .iter()
            .min_by(|a, b| a.btc_price.partial_cmp(&b.btc_price).unwrap())
            .unwrap();
        println!("Best BTC price: {} @ ${:.2}", best.name, best.btc_price);
        (best.name.clone(), best.btc_price)
    });

    // Module 2: Find best ETH price
    let data_for_eth = Arc::clone(&exchanges);
    let eth_analyzer = thread::spawn(move || {
        let best = data_for_eth
            .iter()
            .min_by(|a, b| a.eth_price.partial_cmp(&b.eth_price).unwrap())
            .unwrap();
        println!("Best ETH price: {} @ ${:.2}", best.name, best.eth_price);
        (best.name.clone(), best.eth_price)
    });

    // Module 3: Liquidity analysis
    let data_for_volume = Arc::clone(&exchanges);
    let volume_analyzer = thread::spawn(move || {
        let total: f64 = data_for_volume.iter().map(|e| e.volume_24h).sum();
        let by_exchange: HashMap<String, f64> = data_for_volume
            .iter()
            .map(|e| (e.name.clone(), e.volume_24h / total * 100.0))
            .collect();
        println!("Total 24h volume: ${:.0}", total);
        for (name, share) in &by_exchange {
            println!("  {}: {:.1}%", name, share);
        }
        by_exchange
    });

    // Collect results
    let best_btc = btc_analyzer.join().unwrap();
    let best_eth = eth_analyzer.join().unwrap();
    let volume_shares = volume_analyzer.join().unwrap();

    println!("\n=== SUMMARY ===");
    println!("Buy BTC on {} @ ${:.2}", best_btc.0, best_btc.1);
    println!("Buy ETH on {} @ ${:.2}", best_eth.0, best_eth.1);
}
```

## Arc with Structs

```rust
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct Portfolio {
    assets: Vec<(String, f64, f64)>, // (symbol, quantity, price)
}

impl Portfolio {
    fn total_value(&self) -> f64 {
        self.assets.iter().map(|(_, qty, price)| qty * price).sum()
    }

    fn position_values(&self) -> Vec<(String, f64)> {
        self.assets
            .iter()
            .map(|(sym, qty, price)| (sym.clone(), qty * price))
            .collect()
    }
}

fn main() {
    let portfolio = Arc::new(Portfolio {
        assets: vec![
            ("BTC".to_string(), 0.5, 42000.0),
            ("ETH".to_string(), 5.0, 2500.0),
            ("SOL".to_string(), 100.0, 95.0),
        ],
    });

    // Thread 1: Calculate total value
    let p1 = Arc::clone(&portfolio);
    let total_handle = thread::spawn(move || {
        let total = p1.total_value();
        println!("Portfolio total: ${:.2}", total);
        total
    });

    // Thread 2: Analyze positions
    let p2 = Arc::clone(&portfolio);
    let positions_handle = thread::spawn(move || {
        let positions = p2.position_values();
        for (sym, value) in &positions {
            println!("{}: ${:.2}", sym, value);
        }
        positions
    });

    let total = total_handle.join().unwrap();
    let positions = positions_handle.join().unwrap();

    // Calculate allocations
    println!("\n--- Allocation ---");
    for (sym, value) in positions {
        println!("{}: {:.1}%", sym, value / total * 100.0);
    }
}
```

## Arc with Large Data

Arc is especially useful for large data structures that are expensive to clone:

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    // Large volume of historical data
    let historical_prices: Arc<Vec<f64>> = Arc::new(
        (0..1_000_000)
            .map(|i| 40000.0 + (i as f64 * 0.001).sin() * 2000.0)
            .collect()
    );

    println!("Historical data size: {} prices", historical_prices.len());

    let mut handles = vec![];

    // Launch 4 analysis threads
    for thread_id in 0..4 {
        let data = Arc::clone(&historical_prices);
        let chunk_size = data.len() / 4;
        let start = thread_id * chunk_size;
        let end = if thread_id == 3 { data.len() } else { start + chunk_size };

        let handle = thread::spawn(move || {
            let chunk = &data[start..end];
            let avg: f64 = chunk.iter().sum::<f64>() / chunk.len() as f64;
            let max = chunk.iter().cloned().fold(0.0_f64, f64::max);
            let min = chunk.iter().cloned().fold(f64::MAX, f64::min);

            println!(
                "Thread {}: chunk [{}-{}], avg=${:.2}, range=${:.2}-${:.2}",
                thread_id, start, end, avg, min, max
            );

            (avg, min, max)
        });

        handles.push(handle);
    }

    // Collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    let global_avg = results.iter().map(|(avg, _, _)| avg).sum::<f64>() / 4.0;
    let global_min = results.iter().map(|(_, min, _)| *min).fold(f64::MAX, f64::min);
    let global_max = results.iter().map(|(_, _, max)| *max).fold(0.0_f64, f64::max);

    println!("\n=== GLOBAL STATS ===");
    println!("Average: ${:.2}", global_avg);
    println!("Range: ${:.2} - ${:.2}", global_min, global_max);
}
```

## Important: Arc Provides Only Immutable Access

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let price = Arc::new(42000.0_f64);
    let price_clone = Arc::clone(&price);

    let handle = thread::spawn(move || {
        // Can read
        println!("Price: ${}", *price_clone);

        // CANNOT modify!
        // *price_clone = 43000.0; // ERROR!
    });

    handle.join().unwrap();

    // For modification, you need Arc<Mutex<T>> - covered in the next chapter
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Arc<T>` | Atomic Reference Counted — thread-safe smart pointer |
| `Arc::new(data)` | Create Arc with data |
| `Arc::clone(&arc)` | Create new pointer (not a data clone!) |
| `Arc::strong_count()` | Current reference count |
| `*arc` | Access data (read-only) |
| `Arc` vs `Rc` | Arc for multithreading, Rc for single thread |

## Practical Exercises

1. **Multi-analyzer prices**: Create a program that launches 3 threads to calculate SMA (Simple Moving Average), EMA (Exponential), and RSI in parallel on the same historical data.

2. **Portfolio tracker**: Implement a system where multiple threads calculate portfolio metrics in parallel: total value, unrealized P&L, sector exposure.

3. **Order book aggregator**: Write a program that receives order book data from multiple exchanges and finds the best bid/ask and calculates the spread in parallel threads.

## Homework

1. Implement a multithreaded volatility calculator:
   - Load historical data into `Arc<Vec<f64>>`
   - Calculate daily volatility in one thread
   - Calculate weekly volatility in another thread
   - Find maximum drawdown in a third thread

2. Create a portfolio monitoring system:
   - Store portfolio data in `Arc<Portfolio>`
   - Launch a thread to calculate VaR (Value at Risk)
   - Launch a thread to calculate Sharpe Ratio
   - Launch a thread to check limits

3. Write an arbitrage scanner:
   - Store exchange prices in `Arc<HashMap<String, ExchangeData>>`
   - Search for arbitrage opportunities between exchange pairs in parallel
   - Output found spreads and potential profit

4. Implement a backtester with parallel analysis:
   - Historical data in `Arc<Vec<Candle>>`
   - Launch multiple threads with different strategy parameters
   - Collect results and find optimal parameters

## Navigation

[← Previous day](../160-mutex-one-trader/en.md) | [Next day →](../162-arc-mutex-shared-mutable/en.md)
