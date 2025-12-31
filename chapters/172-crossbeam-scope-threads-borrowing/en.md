# Day 172: crossbeam Scope: Threads with Borrowing

## Trading Analogy

Imagine you're a trading floor manager who needs to urgently send a task to analysts: "Analyze this list of assets". With regular threads (`std::thread::spawn`), you would need to **copy** the list for each analyst — expensive for a large portfolio.

With `crossbeam::scope`, you can simply **show** analysts the list on a shared screen — they all look at the same screen, do their work, and only when EVERYONE finishes, you switch the screen to another task. No copying, complete safety — nobody leaves the meeting until everyone is done.

In the world of threads, this solves the main problem: how to let threads **borrow** local variables from the stack instead of requiring `'static` lifetime.

## The Problem with Standard Threads

```rust
use std::thread;

fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 42200.0];

    // THIS WON'T COMPILE!
    let handle = thread::spawn(|| {
        // Error: `prices` does not live long enough
        println!("First price: {}", prices[0]);
    });

    handle.join().unwrap();
}
```

The compiler rightfully complains: it cannot guarantee that `prices` will exist while the thread is running. What if the thread outlives main()?

Classic solutions:
- `move` — move ownership into the thread (but then can't use in other threads)
- `Arc::clone()` — clone a smart pointer (overhead)

## crossbeam::scope — The Elegant Solution

```rust
use crossbeam::thread;

fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 42200.0];
    let volumes = vec![100, 250, 150, 300];

    // The scope guarantees:
    // all threads will finish BEFORE exiting the block
    thread::scope(|s| {
        // Thread 1: analyzes prices
        s.spawn(|_| {
            let avg_price: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
            println!("Average price: ${:.2}", avg_price);
        });

        // Thread 2: analyzes volumes
        s.spawn(|_| {
            let total_volume: i32 = volumes.iter().sum();
            println!("Total volume: {} lots", total_volume);
        });

        // Thread 3: uses BOTH arrays
        s.spawn(|_| {
            let weighted_price: f64 = prices.iter()
                .zip(volumes.iter())
                .map(|(p, v)| p * *v as f64)
                .sum::<f64>() / volumes.iter().sum::<i32>() as f64;
            println!("Volume-weighted price: ${:.2}", weighted_price);
        });

    }).unwrap(); // All threads have finished

    // prices and volumes are still available here!
    println!("Total prices analyzed: {}", prices.len());
}
```

## How It Works

```
main()                          scope
  |                               |
  v                               v
prices: [...]  ─────────────────> s.spawn() sees prices
volumes: [...]  ────────────────> s.spawn() sees volumes
  |                               |
  |   +-- Thread 1 ──────────────>|
  |   +-- Thread 2 ──────────────>|
  |   +-- Thread 3 ──────────────>|
  |                               |
  |<────── ALL threads join ──────+
  |
  v
prices and volumes still alive!
```

The key point: `scope` guarantees that all spawned threads will complete **before** returning from the scope. This allows the compiler to be certain that borrowed data lives long enough.

## Practical Example: Parallel Portfolio Analysis

```rust
use crossbeam::thread;
use std::collections::HashMap;

#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    current_price: f64,
}

impl Position {
    fn pnl(&self) -> f64 {
        (self.current_price - self.avg_price) * self.quantity
    }

    fn pnl_percent(&self) -> f64 {
        ((self.current_price / self.avg_price) - 1.0) * 100.0
    }
}

fn main() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.5, avg_price: 40000.0, current_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 15.0, avg_price: 2800.0, current_price: 2650.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, avg_price: 95.0, current_price: 110.0 },
        Position { symbol: "DOGE".to_string(), quantity: 50000.0, avg_price: 0.08, current_price: 0.09 },
    ];

    // Use scope for parallel analysis
    let results = thread::scope(|s| {
        // Thread 1: Calculate total PnL
        let pnl_handle = s.spawn(|_| {
            let total_pnl: f64 = portfolio.iter()
                .map(|p| p.pnl())
                .sum();
            total_pnl
        });

        // Thread 2: Find best position
        let best_handle = s.spawn(|_| {
            portfolio.iter()
                .max_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
                .map(|p| (p.symbol.clone(), p.pnl_percent()))
        });

        // Thread 3: Find worst position
        let worst_handle = s.spawn(|_| {
            portfolio.iter()
                .min_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
                .map(|p| (p.symbol.clone(), p.pnl_percent()))
        });

        // Thread 4: Calculate total portfolio value
        let value_handle = s.spawn(|_| {
            portfolio.iter()
                .map(|p| p.current_price * p.quantity)
                .sum::<f64>()
        });

        // Collect results (all joins happen inside scope)
        (
            pnl_handle.join().unwrap(),
            best_handle.join().unwrap(),
            worst_handle.join().unwrap(),
            value_handle.join().unwrap(),
        )
    }).unwrap();

    println!("=== Portfolio Analysis ===");
    println!("Total PnL: ${:.2}", results.0);
    if let Some((symbol, pct)) = results.1 {
        println!("Best position: {} ({:+.2}%)", symbol, pct);
    }
    if let Some((symbol, pct)) = results.2 {
        println!("Worst position: {} ({:+.2}%)", symbol, pct);
    }
    println!("Portfolio value: ${:.2}", results.3);
}
```

## Nested Spawning

Sometimes a thread needs to create additional threads. For this, you can use nested scopes — the inner scope will have access to data from the outer context:

```rust
use crossbeam::thread;

fn main() {
    let exchanges = vec!["Binance", "Kraken", "Coinbase"];
    let symbols = vec!["BTC", "ETH", "SOL"];

    // Outer scope for main tasks
    thread::scope(|s| {
        // First thread: analyzer
        s.spawn(|_| {
            println!("Analyzer started");

            // Nested scope for parallel exchange requests
            thread::scope(|inner_s| {
                for exchange in &exchanges {
                    for symbol in &symbols {
                        inner_s.spawn(move |_| {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            println!("[{}] {} = $42000.00", exchange, symbol);
                        });
                    }
                }
            }).unwrap();

            println!("All exchange data received");
        });

        // Second thread: monitoring (runs in parallel with the first)
        s.spawn(|_| {
            println!("Monitoring {} exchanges with {} symbols",
                     exchanges.len(), symbols.len());
        });
    }).unwrap();

    println!("All data collected!");
}
```

## Comparison with std::thread::scope

Since Rust 1.63, `std::thread::scope` is available in the standard library. It's similar to crossbeam, but there are differences:

```rust
use std::thread;

fn main() {
    let prices = vec![42000.0, 42100.0];

    // std::thread::scope (Rust 1.63+)
    thread::scope(|s| {
        s.spawn(|| {
            println!("Price: {}", prices[0]);
        });
    });

    println!("After scope: {:?}", prices);
}
```

| Feature | crossbeam::scope | std::thread::scope |
|---------|------------------|-------------------|
| Availability | Any Rust version | Rust 1.63+ |
| Nested threads | More convenient (scope parameter) | Possible, but less convenient |
| Return values | Via join() | Via join() |
| Panic handling | unwrap() on result | Panic propagates |
| Performance | Optimized | Standard |

## Error Handling in Scope

```rust
use crossbeam::thread;

fn main() {
    let orders = vec![
        ("BTC", 1.0, 42000.0),
        ("ETH", 10.0, 2800.0),
        ("INVALID", -1.0, 0.0), // Invalid order
    ];

    let result = thread::scope(|s| {
        let handles: Vec<_> = orders.iter()
            .map(|(symbol, qty, price)| {
                s.spawn(move |_| {
                    if *qty <= 0.0 {
                        Err(format!("Invalid quantity for {}", symbol))
                    } else {
                        Ok(format!("{}: {} x ${}", symbol, qty, price))
                    }
                })
            })
            .collect();

        // Collect results
        handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });

    match result {
        Ok(results) => {
            for r in results {
                match r {
                    Ok(msg) => println!("Success: {}", msg),
                    Err(e) => println!("Error: {}", e),
                }
            }
        }
        Err(e) => println!("Thread panic: {:?}", e),
    }
}
```

## Example: Parallel Technical Indicator Calculation

```rust
use crossbeam::thread;

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    prices.windows(period)
        .map(|w| w.iter().sum::<f64>() / period as f64)
        .collect()
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() {
        return vec![];
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = vec![prices[0]];

    for price in &prices[1..] {
        let new_ema = (price - ema.last().unwrap()) * multiplier + ema.last().unwrap();
        ema.push(new_ema);
    }

    ema
}

fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let changes: Vec<f64> = prices.windows(2)
        .map(|w| w[1] - w[0])
        .collect();

    let mut rsi = Vec::new();

    for i in period..changes.len() {
        let window = &changes[i - period..i];
        let gains: f64 = window.iter().filter(|&&x| x > 0.0).sum();
        let losses: f64 = window.iter().filter(|&&x| x < 0.0).map(|x| x.abs()).sum();

        let rs = if losses == 0.0 { 100.0 } else { gains / losses };
        let rsi_value = 100.0 - (100.0 / (1.0 + rs));
        rsi.push(rsi_value);
    }

    rsi
}

fn main() {
    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
    ];

    let indicators = thread::scope(|s| {
        let sma_handle = s.spawn(|_| {
            ("SMA(5)", calculate_sma(&prices, 5))
        });

        let ema_handle = s.spawn(|_| {
            ("EMA(5)", calculate_ema(&prices, 5))
        });

        let rsi_handle = s.spawn(|_| {
            ("RSI(14)", calculate_rsi(&prices, 14))
        });

        vec![
            sma_handle.join().unwrap(),
            ema_handle.join().unwrap(),
            rsi_handle.join().unwrap(),
        ]
    }).unwrap();

    println!("=== Technical Indicators ===");
    for (name, values) in indicators {
        if let Some(last) = values.last() {
            println!("{}: {:.2}", name, last);
        } else {
            println!("{}: insufficient data", name);
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `crossbeam::scope` | Creates a scope where threads can borrow local data |
| Completion guarantee | All threads are guaranteed to finish before exiting scope |
| Borrowing | Threads can borrow `&T` and `&mut T` from the stack |
| Nested threads | Scope parameter is passed for creating child threads |
| Return values | `s.spawn().join()` returns the thread's result |
| std::thread::scope | Standard library equivalent (Rust 1.63+) |

## Homework

1. **Parallel Candlestick Analysis**: Write a program that:
   - Has a vector of OHLCV data (Open, High, Low, Close, Volume)
   - Spawns 4 threads, each calculating: average Open, maximum High, minimum Low, total Volume
   - Uses `crossbeam::scope` to borrow the data

2. **Multi-Portfolio Monitoring**: Create a `Portfolio` struct and a vector of portfolios. Using scope:
   - Spawn a separate thread for each portfolio
   - Each thread computes: total value, PnL, number of positions
   - Collect results and print a summary table

3. **Parallel Backtest**: Given historical data:
   - Split it into N parts
   - Spawn N threads, each testing a strategy on its portion
   - Combine results into overall statistics

4. **Performance Comparison**: Write a benchmark:
   - The same calculation done sequentially
   - Using `crossbeam::scope`
   - Using `std::thread::scope`
   - Compare execution times for different data sizes

## Navigation

[← Previous day](../171-crossbeam-channels-faster-mpsc/en.md) | [Next day →](../173-rayon-parallel-iterators/en.md)
