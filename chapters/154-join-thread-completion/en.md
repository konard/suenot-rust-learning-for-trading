# Day 154: join: Waiting for Thread Completion

## Trading Analogy

Imagine you've sent several analysts to gather data from different exchanges. One is analyzing Binance, another Coinbase, and the third Kraken. You cannot make a trading decision until all analysts return with their results. The `join()` method is like waiting for each analyst to return: you block your work until a specific thread completes its task.

**Without join:** You make decisions based on incomplete data — dangerous!
**With join:** You wait for all analysts and make an informed decision.

## What is join?

When we create a thread with `thread::spawn()`, it returns a `JoinHandle<T>`. This is a "handle" for controlling the thread:

```rust
use std::thread;

fn main() {
    // spawn returns JoinHandle
    let handle = thread::spawn(|| {
        println!("Working in thread!");
        42  // Return value
    });

    // join() blocks until the thread completes and returns Result
    let result = handle.join();

    match result {
        Ok(value) => println!("Thread returned: {}", value),
        Err(_) => println!("Thread panicked!"),
    }
}
```

## Basic Example: Fetching Prices from Exchanges

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Collecting Prices from Exchanges ===\n");

    // Spawn threads to fetch prices
    let binance_handle = thread::spawn(|| {
        println!("[Binance] Connecting...");
        thread::sleep(Duration::from_millis(100));
        let price = 42150.50;
        println!("[Binance] BTC price: ${:.2}", price);
        price
    });

    let coinbase_handle = thread::spawn(|| {
        println!("[Coinbase] Connecting...");
        thread::sleep(Duration::from_millis(150));
        let price = 42148.75;
        println!("[Coinbase] BTC price: ${:.2}", price);
        price
    });

    let kraken_handle = thread::spawn(|| {
        println!("[Kraken] Connecting...");
        thread::sleep(Duration::from_millis(80));
        let price = 42152.00;
        println!("[Kraken] BTC price: ${:.2}", price);
        price
    });

    // Wait for all threads to complete
    let binance_price = binance_handle.join().expect("Binance thread crashed");
    let coinbase_price = coinbase_handle.join().expect("Coinbase thread crashed");
    let kraken_price = kraken_handle.join().expect("Kraken thread crashed");

    // Now we have all the data
    let avg_price = (binance_price + coinbase_price + kraken_price) / 3.0;
    println!("\n=== Results ===");
    println!("Average BTC price: ${:.2}", avg_price);
}
```

## Why is join Important?

### Without join — Program may exit before threads complete

```rust
use std::thread;
use std::time::Duration;

fn main() {
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(1));
        println!("This may NOT print!");
    });

    println!("Main is exiting...");
    // Program exits, thread is killed!
}
```

### With join — Guaranteed waiting

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_secs(1));
        println!("This WILL print!");
    });

    println!("Main is waiting...");
    handle.join().unwrap();
    println!("Thread completed!");
}
```

## Error Handling: Result from join

`join()` returns `Result<T, Box<dyn Any + Send>>`:
- `Ok(value)` — thread completed successfully and returned a value
- `Err(panic_info)` — thread panicked

```rust
use std::thread;

fn main() {
    // Thread with successful completion
    let success_handle = thread::spawn(|| {
        "Success!"
    });

    // Thread with panic
    let panic_handle = thread::spawn(|| {
        panic!("Something went wrong!");
    });

    // Handle results
    match success_handle.join() {
        Ok(msg) => println!("Successful thread: {}", msg),
        Err(_) => println!("Thread crashed!"),
    }

    match panic_handle.join() {
        Ok(_) => println!("This won't print"),
        Err(_) => println!("Panicked thread handled safely"),
    }

    println!("Program continues running!");
}
```

## Practical Example: Parallel Portfolio Analysis

```rust
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct AssetAnalysis {
    symbol: String,
    price: f64,
    change_24h: f64,
    signal: String,
}

fn analyze_asset(symbol: &str, price: f64) -> AssetAnalysis {
    // Simulate analysis
    thread::sleep(Duration::from_millis(50));

    let change_24h = match symbol {
        "BTC" => 2.5,
        "ETH" => -1.2,
        "SOL" => 5.8,
        _ => 0.0,
    };

    let signal = if change_24h > 3.0 {
        "STRONG BUY"
    } else if change_24h > 0.0 {
        "BUY"
    } else if change_24h > -3.0 {
        "HOLD"
    } else {
        "SELL"
    };

    AssetAnalysis {
        symbol: symbol.to_string(),
        price,
        change_24h,
        signal: signal.to_string(),
    }
}

fn main() {
    println!("=== Parallel Portfolio Analysis ===\n");

    let start = std::time::Instant::now();

    // Launch analysis of each asset in a separate thread
    let btc_handle = thread::spawn(|| {
        analyze_asset("BTC", 42150.0)
    });

    let eth_handle = thread::spawn(|| {
        analyze_asset("ETH", 2250.0)
    });

    let sol_handle = thread::spawn(|| {
        analyze_asset("SOL", 98.50)
    });

    // Collect results
    let analyses = vec![
        btc_handle.join().expect("BTC analysis crashed"),
        eth_handle.join().expect("ETH analysis crashed"),
        sol_handle.join().expect("SOL analysis crashed"),
    ];

    let elapsed = start.elapsed();

    // Print results
    println!("╔═══════════════════════════════════════════════╗");
    println!("║           PORTFOLIO ANALYSIS                  ║");
    println!("╠═══════════════════════════════════════════════╣");

    for analysis in &analyses {
        println!("║ {:>6} | ${:>10.2} | {:>+6.1}% | {:>10} ║",
            analysis.symbol,
            analysis.price,
            analysis.change_24h,
            analysis.signal
        );
    }

    println!("╠═══════════════════════════════════════════════╣");
    println!("║ Analysis time: {:>6.2?}                       ║", elapsed);
    println!("╚═══════════════════════════════════════════════╝");
}
```

## Waiting for Multiple Threads

### Method 1: Sequential join

```rust
use std::thread;

fn main() {
    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                println!("Thread {} working", i);
                i * 10
            })
        })
        .collect();

    // Wait for each thread in order
    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    println!("Results: {:?}", results);
}
```

### Method 2: Collecting results in a loop

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Parallel Indicator Calculation ===\n");

    let indicators = vec!["SMA", "EMA", "RSI", "MACD", "BB"];

    let handles: Vec<_> = indicators
        .iter()
        .map(|&name| {
            thread::spawn(move || {
                // Simulate calculation
                thread::sleep(Duration::from_millis(50));
                let value = match name {
                    "SMA" => 42100.0,
                    "EMA" => 42050.0,
                    "RSI" => 65.5,
                    "MACD" => 125.0,
                    "BB" => 42200.0,
                    _ => 0.0,
                };
                (name, value)
            })
        })
        .collect();

    println!("All indicators launched, waiting for results...\n");

    for handle in handles {
        match handle.join() {
            Ok((name, value)) => println!("{}: {:.2}", name, value),
            Err(_) => println!("Calculation error!"),
        }
    }
}
```

## Return Values from Threads

Threads can return any type that implements `Send`:

```rust
use std::thread;
use std::collections::HashMap;

fn main() {
    // Return simple value
    let num_handle = thread::spawn(|| 42);

    // Return String
    let str_handle = thread::spawn(|| String::from("Result"));

    // Return Vec
    let vec_handle = thread::spawn(|| vec![1, 2, 3, 4, 5]);

    // Return HashMap
    let map_handle = thread::spawn(|| {
        let mut prices = HashMap::new();
        prices.insert("BTC", 42000.0);
        prices.insert("ETH", 2200.0);
        prices
    });

    println!("Number: {}", num_handle.join().unwrap());
    println!("String: {}", str_handle.join().unwrap());
    println!("Vector: {:?}", vec_handle.join().unwrap());
    println!("HashMap: {:?}", map_handle.join().unwrap());
}
```

## Practical Example: Monitoring Multiple Trading Pairs

```rust
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    bid: f64,
    ask: f64,
    spread: f64,
}

fn monitor_pair(symbol: &str, base_price: f64) -> PriceUpdate {
    thread::sleep(Duration::from_millis(100));

    let spread_pct = 0.0005; // 0.05%
    let bid = base_price * (1.0 - spread_pct);
    let ask = base_price * (1.0 + spread_pct);

    PriceUpdate {
        symbol: symbol.to_string(),
        bid,
        ask,
        spread: ask - bid,
    }
}

fn main() {
    println!("=== Trading Pair Monitoring ===\n");

    let pairs = vec![
        ("BTC/USDT", 42000.0),
        ("ETH/USDT", 2200.0),
        ("SOL/USDT", 98.0),
        ("XRP/USDT", 0.55),
    ];

    // Launch monitoring for each pair in a separate thread
    let handles: Vec<_> = pairs
        .into_iter()
        .map(|(symbol, price)| {
            thread::spawn(move || monitor_pair(symbol, price))
        })
        .collect();

    // Collect updates
    let updates: Vec<PriceUpdate> = handles
        .into_iter()
        .filter_map(|h| h.join().ok())
        .collect();

    // Print results
    println!("╔════════════════════════════════════════════════════╗");
    println!("║   Pair     │    Bid     │    Ask     │   Spread   ║");
    println!("╠════════════════════════════════════════════════════╣");

    for update in &updates {
        println!("║ {:>9} │ {:>10.4} │ {:>10.4} │ {:>10.4} ║",
            update.symbol,
            update.bid,
            update.ask,
            update.spread
        );
    }

    println!("╚════════════════════════════════════════════════════╝");

    // Find pair with minimum spread
    if let Some(best) = updates.iter().min_by(|a, b| {
        a.spread.partial_cmp(&b.spread).unwrap()
    }) {
        println!("\nBest spread: {} (${:.4})", best.symbol, best.spread);
    }
}
```

## Practical Exercises

### Exercise 1: Parallel Moving Average Calculation

Write a program that calculates SMAs for different periods in parallel:

```rust
use std::thread;

fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    let sum: f64 = prices[prices.len() - period..].iter().sum();
    Some(sum / period as f64)
}

fn main() {
    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    let periods = vec![3, 5, 7, 10];

    // TODO: Launch calculation of each SMA in a separate thread
    // and print the results
}
```

### Exercise 2: Arbitrage Scanner

Create a program that fetches prices from different "exchanges" in parallel and finds arbitrage opportunities:

```rust
use std::thread;
use std::time::Duration;

fn get_price(exchange: &str, symbol: &str) -> f64 {
    thread::sleep(Duration::from_millis(50));
    // Simulate different prices
    match (exchange, symbol) {
        ("Binance", "BTC") => 42000.0,
        ("Coinbase", "BTC") => 42050.0,
        ("Kraken", "BTC") => 41980.0,
        _ => 0.0,
    }
}

fn main() {
    // TODO: Fetch BTC prices from all exchanges in parallel
    // Find arbitrage opportunity (difference > 0.1%)
}
```

### Exercise 3: Error Handling in Threads

Write a program that safely handles situations where some threads crash:

```rust
use std::thread;

fn fetch_price(exchange: &str) -> f64 {
    if exchange == "BadExchange" {
        panic!("Exchange unavailable!");
    }
    42000.0
}

fn main() {
    let exchanges = vec!["Binance", "BadExchange", "Coinbase", "Kraken"];

    // TODO: Launch threads for each exchange
    // Handle crashes safely
    // Calculate average price from successful results only
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `JoinHandle<T>` | Returned by `spawn()`, allows waiting for thread completion |
| `.join()` | Blocks current thread until target thread completes |
| `Result<T, E>` | join returns Result — can handle panics |
| `.unwrap()` / `.expect()` | Extract value with panic on error |
| Parallel collection | Launch multiple threads and collect results |

## Homework

1. **Parallel Backtest**: Create a program that runs trading strategy backtests on different timeframes in parallel (1m, 5m, 15m, 1h). Each thread should return backtest results (profit, number of trades, win rate).

2. **Multi-Exchange Orderbook**: Write a system that fetches orderbooks from 5 exchanges in parallel and aggregates them into a single "virtual" orderbook. Use `join()` for synchronization.

3. **Fault-Tolerant Loader**: Create a data loader that attempts to fetch an asset's price from 3 exchanges in parallel. If one exchange crashes (panic), the others should continue working. Return the first successful price or an error if all fail.

4. **Parallel Risk Calculation**: Write a program that calculates portfolio risk metrics in parallel: VaR, Sharpe Ratio, Max Drawdown, Beta. Combine results into a final risk report.

## Navigation

[← Previous day](../153-thread-spawn/en.md) | [Next day →](../155-move-closures/en.md)
