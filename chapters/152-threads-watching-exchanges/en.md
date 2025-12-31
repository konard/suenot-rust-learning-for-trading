# Day 152: Threads — Watching Exchanges in Parallel

## Trading Analogy

Imagine you want to monitor prices on multiple exchanges simultaneously: Binance, Bybit, and OKX. If you check them one by one, while you're looking at Binance, the price on OKX might already have changed. It's like having only one monitor and constantly switching between tabs.

**Threads** are like hiring three assistant traders: one watches Binance, another watches Bybit, and the third watches OKX. They all work simultaneously and notify you when they see an interesting price.

In Rust, threads allow you to execute multiple tasks **in parallel**, which is critical for:
- Monitoring multiple exchanges simultaneously
- Processing large volumes of market data
- Calculating complex indicators without blocking the main program
- Quickly reacting to market events

## Why Do We Need Threads in Trading?

```rust
// WITHOUT threads: sequential checking
fn check_prices_sequential() {
    let binance_price = fetch_binance_price(); // 1 second
    let bybit_price = fetch_bybit_price();     // 1 second
    let okx_price = fetch_okx_price();         // 1 second
    // Total: 3 seconds - too slow for trading!
}

// WITH threads: parallel checking
fn check_prices_parallel() {
    // All three requests execute simultaneously
    // Total: ~1 second - exactly what we need!
}
```

## Creating Your First Thread

In Rust, threads are created using `std::thread::spawn`:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("Main trader: Starting exchange monitoring...");

    // Create a thread to monitor Binance
    let binance_handle = thread::spawn(|| {
        for i in 1..=3 {
            println!("Binance: BTC = ${}", 42000 + i * 100);
            thread::sleep(Duration::from_millis(500));
        }
        println!("Binance: Monitoring complete");
    });

    // Main thread continues working
    for i in 1..=3 {
        println!("Main: Analyzing market... step {}", i);
        thread::sleep(Duration::from_millis(300));
    }

    // Wait for Binance thread to complete
    binance_handle.join().unwrap();

    println!("Main trader: All threads finished");
}
```

Program output (order may vary — that's parallelism!):
```
Main trader: Starting exchange monitoring...
Binance: BTC = $42100
Main: Analyzing market... step 1
Main: Analyzing market... step 2
Binance: BTC = $42200
Main: Analyzing market... step 3
Binance: BTC = $42300
Binance: Monitoring complete
Main trader: All threads finished
```

## Multiple Threads: Monitoring Three Exchanges

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("═══════════════════════════════════════");
    println!("    MULTI-EXCHANGE PRICE MONITOR");
    println!("═══════════════════════════════════════\n");

    // Start threads for each exchange
    let binance = thread::spawn(|| {
        monitor_exchange("Binance", 42000.0, 50.0)
    });

    let bybit = thread::spawn(|| {
        monitor_exchange("Bybit", 41980.0, 30.0)
    });

    let okx = thread::spawn(|| {
        monitor_exchange("OKX", 42010.0, 40.0)
    });

    // Wait for all threads to complete
    binance.join().unwrap();
    bybit.join().unwrap();
    okx.join().unwrap();

    println!("\nAll exchange monitoring complete!");
}

fn monitor_exchange(name: &str, base_price: f64, volatility: f64) {
    for tick in 1..=5 {
        // Simulate price changes
        let price_change = ((tick as f64 * 1.5).sin() * volatility) as f64;
        let current_price = base_price + price_change;

        println!("[{}] BTC: ${:.2}", name, current_price);
        thread::sleep(Duration::from_millis(200));
    }
}
```

## Getting Results from a Thread

Threads can return values:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("Fetching prices from exchanges...\n");

    // Each thread returns a price
    let binance_handle = thread::spawn(|| -> f64 {
        thread::sleep(Duration::from_millis(100));
        42150.50  // Simulated received price
    });

    let bybit_handle = thread::spawn(|| -> f64 {
        thread::sleep(Duration::from_millis(150));
        42145.25
    });

    let okx_handle = thread::spawn(|| -> f64 {
        thread::sleep(Duration::from_millis(120));
        42155.75
    });

    // Collect results
    let binance_price = binance_handle.join().unwrap();
    let bybit_price = bybit_handle.join().unwrap();
    let okx_price = okx_handle.join().unwrap();

    // Analyze the received data
    println!("Received prices:");
    println!("  Binance: ${:.2}", binance_price);
    println!("  Bybit:   ${:.2}", bybit_price);
    println!("  OKX:     ${:.2}", okx_price);

    let avg_price = (binance_price + bybit_price + okx_price) / 3.0;
    let best_buy = binance_price.min(bybit_price).min(okx_price);
    let best_sell = binance_price.max(bybit_price).max(okx_price);
    let spread = best_sell - best_buy;

    println!("\nAnalysis:");
    println!("  Average price:    ${:.2}", avg_price);
    println!("  Best buy at:      ${:.2}", best_buy);
    println!("  Best sell at:     ${:.2}", best_sell);
    println!("  Arbitrage spread: ${:.2} ({:.3}%)",
             spread,
             (spread / avg_price) * 100.0);
}
```

## Practical Example: Arbitrage Scanner

```rust
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct ExchangePrice {
    exchange: String,
    symbol: String,
    bid: f64,      // Buy price
    ask: f64,      // Sell price
    timestamp: u64,
}

fn main() {
    println!("╔═════════════════════════════════════════════════╗");
    println!("║         ARBITRAGE SCANNER v1.0                  ║");
    println!("║    Finding Arbitrage Opportunities              ║");
    println!("╚═════════════════════════════════════════════════╝\n");

    let start = Instant::now();

    // Fetch prices from all exchanges in parallel
    let binance = thread::spawn(|| fetch_exchange_price("Binance", 42100.0, 42105.0));
    let bybit = thread::spawn(|| fetch_exchange_price("Bybit", 42095.0, 42102.0));
    let okx = thread::spawn(|| fetch_exchange_price("OKX", 42110.0, 42118.0));
    let kraken = thread::spawn(|| fetch_exchange_price("Kraken", 42090.0, 42098.0));

    // Collect results
    let prices = vec![
        binance.join().unwrap(),
        bybit.join().unwrap(),
        okx.join().unwrap(),
        kraken.join().unwrap(),
    ];

    let elapsed = start.elapsed();
    println!("Data fetched in {:?}\n", elapsed);

    // Display all prices
    println!("┌─────────────┬────────────┬────────────┬────────────┐");
    println!("│ Exchange    │ Bid        │ Ask        │ Spread     │");
    println!("├─────────────┼────────────┼────────────┼────────────┤");
    for price in &prices {
        let spread = price.ask - price.bid;
        println!("│ {:11} │ ${:9.2} │ ${:9.2} │ ${:9.2} │",
                 price.exchange, price.bid, price.ask, spread);
    }
    println!("└─────────────┴────────────┴────────────┴────────────┘\n");

    // Find arbitrage opportunities
    find_arbitrage(&prices);
}

fn fetch_exchange_price(exchange: &str, bid: f64, ask: f64) -> ExchangePrice {
    // Simulate network delay
    thread::sleep(Duration::from_millis(50 + (bid as u64 % 50)));

    ExchangePrice {
        exchange: exchange.to_string(),
        symbol: "BTC/USDT".to_string(),
        bid,
        ask,
        timestamp: 1234567890,
    }
}

fn find_arbitrage(prices: &[ExchangePrice]) {
    println!("Searching for arbitrage opportunities...\n");

    let mut opportunities_found = false;

    for buy_from in prices {
        for sell_to in prices {
            if buy_from.exchange == sell_to.exchange {
                continue;
            }

            // Buy at ask, sell at bid
            let buy_price = buy_from.ask;
            let sell_price = sell_to.bid;
            let profit_percent = ((sell_price - buy_price) / buy_price) * 100.0;

            if profit_percent > 0.0 {
                opportunities_found = true;
                println!("ARBITRAGE FOUND!");
                println!("  Buy on {} at ${:.2}", buy_from.exchange, buy_price);
                println!("  Sell on {} at ${:.2}", sell_to.exchange, sell_price);
                println!("  Profit: {:.4}%\n", profit_percent);
            }
        }
    }

    if !opportunities_found {
        println!("No arbitrage opportunities found.");
        println!("(This is normal — markets are usually efficient)");
    }
}
```

## Threads and Ownership: A Key Concept

In Rust, it's important to understand that each thread takes **ownership** of the data passed to it:

```rust
use std::thread;

fn main() {
    let portfolio = vec!["BTC", "ETH", "SOL"];

    // move — transfers ownership of the vector to the thread
    let handle = thread::spawn(move || {
        println!("Portfolio in thread: {:?}", portfolio);
        // portfolio now belongs to this thread
    });

    // This line won't compile:
    // println!("Portfolio in main: {:?}", portfolio);
    // error: value borrowed after move

    handle.join().unwrap();
}
```

To share data between threads, we'll need special tools (`Arc`, `Mutex`), which we'll learn about in upcoming chapters.

## Handling Panics in Threads

Threads can panic, and this won't terminate the main program:

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        // Simulate a critical error
        panic!("Exchange connection error!");
    });

    // join() returns Err if the thread panicked
    match handle.join() {
        Ok(_) => println!("Thread completed successfully"),
        Err(e) => {
            println!("Thread terminated with an error!");
            // We can try to extract the message
            if let Some(msg) = e.downcast_ref::<&str>() {
                println!("Reason: {}", msg);
            }
        }
    }

    println!("Main thread continues running");
}
```

## What We Learned

| Concept | Description | Trading Analogy |
|---------|-------------|-----------------|
| `thread::spawn` | Creating a new thread | Hiring an assistant trader |
| `handle.join()` | Waiting for thread completion | Waiting for assistant's report |
| `move` closure | Transferring ownership to thread | Giving your data to assistant |
| Return value | Thread returns a result | Assistant reports the price |
| Panic handling | Thread can fail safely | Assistant can make mistakes |

## Practice Exercises

### Exercise 1: Parallel Indicator Calculation
Create a program that calculates three indicators for a price array in parallel:
- Simple Moving Average (SMA)
- Maximum
- Minimum

### Exercise 2: Multi-Cryptocurrency Monitoring
Write a program where each thread "monitors" a separate cryptocurrency (BTC, ETH, SOL, DOT) and returns the "latest price".

### Exercise 3: Execution Timer
Create a function that runs a task in a separate thread and measures its execution time.

### Exercise 4: Request Pool
Write a program that creates 10 threads, each "making an API request" (sleep for random time) and returns a result.

## Homework

1. **Arbitrage Scanner**: Extend the arbitrage scanner example by adding:
   - Profit calculation in absolute numbers for a $10,000 position
   - Exchange fee consideration (0.1% maker, 0.1% taker)
   - Filtering opportunities with profit > 0.05%

2. **Parallel Backtest**: Write a program that:
   - Loads historical data (can be simulated)
   - Tests three different strategies in parallel
   - Returns results for each strategy

3. **Multi-Timeframe Analysis**: Create a program where:
   - One thread analyzes 1-minute candles
   - Second thread — 5-minute candles
   - Third — 15-minute candles
   - Main thread collects and displays results

## Navigation

[← Previous day](../151-project-historical-data-loader/en.md) | [Next day →](../153-thread-spawn/en.md)
