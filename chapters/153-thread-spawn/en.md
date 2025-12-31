# Day 153: std::thread::spawn — Starting a Thread

## Trading Analogy

Imagine you're a single trader watching multiple exchanges simultaneously: Binance, Coinbase, Kraken. If you check them one by one — while looking at Binance, you might miss a great price on Kraken.

The solution? Hire **multiple analysts** (threads), each watching their own exchange **in parallel**. The `std::thread::spawn` function is like hiring a new analyst for a specific task.

## Basic Thread Spawning

```rust
use std::thread;

fn main() {
    // Spawn a new thread for price monitoring
    thread::spawn(|| {
        println!("Thread: Monitoring BTC price...");
        println!("Thread: Current price: $42,500");
    });

    println!("Main thread: Continuing work");
}
```

**Important:** The main thread may finish before the child thread! Until we learn about `join`, the output may be incomplete.

## Monitoring Multiple Exchanges

```rust
use std::thread;
use std::time::Duration;

fn main() {
    // Thread for monitoring Binance
    thread::spawn(|| {
        for i in 1..=3 {
            println!("[Binance] Check #{}: BTC = $42,{:03}", i, 500 + i * 10);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Thread for monitoring Coinbase
    thread::spawn(|| {
        for i in 1..=3 {
            println!("[Coinbase] Check #{}: BTC = $42,{:03}", i, 520 + i * 5);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Main thread waits a bit to see the output
    thread::sleep(Duration::from_millis(500));
    println!("[Main] Monitoring complete");
}
```

## Passing Data to Threads

### Cloning Data

```rust
use std::thread;

fn main() {
    let symbol = String::from("BTC/USDT");
    let initial_price = 42000.0;

    // Clone data for the thread
    let symbol_clone = symbol.clone();

    thread::spawn(move || {
        println!("Analyzing pair: {}", symbol_clone);
        println!("Initial price: ${:.2}", initial_price);

        // Simulation of analysis
        let target = initial_price * 1.02;  // +2%
        println!("Target: ${:.2}", target);
    });

    println!("Main thread: Working with {}", symbol);
    thread::sleep(std::time::Duration::from_millis(100));
}
```

### The move Closure

```rust
use std::thread;

fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

    // move transfers ownership of prices to the thread
    let handle = thread::spawn(move || {
        let sum: f64 = prices.iter().sum();
        let avg = sum / prices.len() as f64;
        println!("Average price: ${:.2}", avg);
        avg  // Return value
    });

    // prices is no longer accessible here!
    // println!("{:?}", prices);  // Compilation error

    // Wait for completion and get the result
    let average = handle.join().unwrap();
    println!("Received from thread: ${:.2}", average);
}
```

## Trading Signals from Multiple Sources

```rust
use std::thread;
use std::time::Duration;

fn main() {
    // Thread for technical analysis
    let ta_thread = thread::spawn(|| {
        println!("[TA] Analyzing RSI and MACD...");
        thread::sleep(Duration::from_millis(150));
        let signal = "BUY";  // Analysis result
        println!("[TA] Signal: {}", signal);
        signal
    });

    // Thread for volume analysis
    let volume_thread = thread::spawn(|| {
        println!("[Volume] Analyzing volumes...");
        thread::sleep(Duration::from_millis(100));
        let signal = "NEUTRAL";
        println!("[Volume] Signal: {}", signal);
        signal
    });

    // Thread for news analysis
    let news_thread = thread::spawn(|| {
        println!("[News] Checking news...");
        thread::sleep(Duration::from_millis(200));
        let signal = "BUY";
        println!("[News] Signal: {}", signal);
        signal
    });

    // Collect results
    let ta_signal = ta_thread.join().unwrap();
    let volume_signal = volume_thread.join().unwrap();
    let news_signal = news_thread.join().unwrap();

    println!("\n=== Final Analysis ===");
    println!("TA: {}, Volume: {}, News: {}", ta_signal, volume_signal, news_signal);
}
```

## Parallel Indicator Calculation

```rust
use std::thread;

fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    let slice = &prices[prices.len() - period..];
    Some(slice.iter().sum::<f64>() / period as f64)
}

fn calculate_rsi(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period + 1 {
        return None;
    }

    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in (prices.len() - period)..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses += change.abs();
        }
    }

    let avg_gain = gains / period as f64;
    let avg_loss = losses / period as f64;

    if avg_loss == 0.0 {
        return Some(100.0);
    }

    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}

fn main() {
    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
    ];

    let prices_for_sma = prices.clone();
    let prices_for_rsi = prices.clone();

    // Parallel indicator calculation
    let sma_handle = thread::spawn(move || {
        println!("[SMA] Calculating SMA-10...");
        calculate_sma(&prices_for_sma, 10)
    });

    let rsi_handle = thread::spawn(move || {
        println!("[RSI] Calculating RSI-14...");
        calculate_rsi(&prices_for_rsi, 14)
    });

    // Get results
    let sma = sma_handle.join().unwrap();
    let rsi = rsi_handle.join().unwrap();

    println!("\n=== Indicators ===");
    match sma {
        Some(v) => println!("SMA-10: ${:.2}", v),
        None => println!("SMA-10: Not enough data"),
    }
    match rsi {
        Some(v) => println!("RSI-14: {:.2}", v),
        None => println!("RSI-14: Not enough data"),
    }
}
```

## Parallel Order Processing

```rust
use std::thread;
use std::time::Duration;

struct Order {
    id: u32,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

fn process_order(order: Order) -> Result<String, String> {
    println!("[Order #{}] Processing {} {} @ ${:.2}...",
             order.id, order.side, order.symbol, order.price);

    // Simulate processing delay
    thread::sleep(Duration::from_millis(100));

    // Simulate successful execution
    Ok(format!("Order #{} executed: {} {} {} @ ${:.2}",
               order.id, order.side, order.quantity, order.symbol, order.price))
}

fn main() {
    let orders = vec![
        Order { id: 1, symbol: "BTC/USDT".into(), side: "BUY".into(), quantity: 0.5, price: 42000.0 },
        Order { id: 2, symbol: "ETH/USDT".into(), side: "SELL".into(), quantity: 2.0, price: 2800.0 },
        Order { id: 3, symbol: "SOL/USDT".into(), side: "BUY".into(), quantity: 10.0, price: 95.0 },
    ];

    let mut handles = vec![];

    // Spawn a thread for each order
    for order in orders {
        let handle = thread::spawn(move || {
            process_order(order)
        });
        handles.push(handle);
    }

    // Collect results
    println!("\n=== Results ===");
    for handle in handles {
        match handle.join().unwrap() {
            Ok(msg) => println!("✓ {}", msg),
            Err(e) => println!("✗ Error: {}", e),
        }
    }
}
```

## Naming Threads (for debugging)

```rust
use std::thread;

fn main() {
    let builder = thread::Builder::new()
        .name("price-monitor".into())
        .stack_size(32 * 1024);  // 32 KB stack

    let handle = builder.spawn(|| {
        let thread = thread::current();
        println!("Thread '{}' started", thread.name().unwrap_or("unnamed"));
        println!("Monitoring prices...");
    }).unwrap();

    handle.join().unwrap();
}
```

## Practical Example: Multi-Exchange Arbitrage

```rust
use std::thread;
use std::time::Duration;

struct ExchangePrice {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
}

fn fetch_price(exchange: &str, symbol: &str) -> ExchangePrice {
    // Simulate fetching price from exchange
    thread::sleep(Duration::from_millis(50 + (exchange.len() * 10) as u64));

    let base_price = 42000.0;
    let spread = match exchange {
        "Binance" => 10.0,
        "Coinbase" => 25.0,
        "Kraken" => 15.0,
        _ => 20.0,
    };

    let offset = match exchange {
        "Binance" => 0.0,
        "Coinbase" => 50.0,
        "Kraken" => -30.0,
        _ => 0.0,
    };

    ExchangePrice {
        exchange: exchange.to_string(),
        symbol: symbol.to_string(),
        bid: base_price + offset - spread / 2.0,
        ask: base_price + offset + spread / 2.0,
    }
}

fn main() {
    let exchanges = vec!["Binance", "Coinbase", "Kraken"];
    let symbol = "BTC/USDT";

    let mut handles = vec![];

    println!("Fetching prices from {} exchanges in parallel...\n", exchanges.len());

    for exchange in exchanges {
        let sym = symbol.to_string();
        let handle = thread::spawn(move || {
            fetch_price(exchange, &sym)
        });
        handles.push(handle);
    }

    let mut prices: Vec<ExchangePrice> = vec![];
    for handle in handles {
        prices.push(handle.join().unwrap());
    }

    // Display prices
    println!("{:<12} {:>12} {:>12} {:>12}", "Exchange", "Bid", "Ask", "Spread");
    println!("{}", "-".repeat(50));

    for p in &prices {
        println!("{:<12} ${:>10.2} ${:>10.2} ${:>10.2}",
                 p.exchange, p.bid, p.ask, p.ask - p.bid);
    }

    // Find arbitrage opportunities
    println!("\n=== Arbitrage Opportunities ===");

    for i in 0..prices.len() {
        for j in 0..prices.len() {
            if i != j {
                let profit = prices[j].bid - prices[i].ask;
                if profit > 0.0 {
                    println!("Buy on {} @ ${:.2}, sell on {} @ ${:.2} = ${:.2} profit",
                             prices[i].exchange, prices[i].ask,
                             prices[j].exchange, prices[j].bid,
                             profit);
                }
            }
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `thread::spawn` | Creates a new thread of execution |
| `move` closure | Transfers ownership of data to the thread |
| `JoinHandle` | Handle for waiting on thread completion |
| `thread::sleep` | Pauses thread for a duration |
| `thread::current()` | Gets information about the current thread |
| `Builder` | Advanced thread configuration |

## Important Points

1. **Data must be `Send`** — the type must be safe to transfer between threads
2. **Ownership is transferred** — after `move`, data is inaccessible in the original thread
3. **Clone if needed** — `clone()` allows using data in multiple threads
4. **Main thread may finish first** — use `join()` to wait for completion

## Homework

1. **Multi-Exchange Monitor:** Create a program that fetches prices from 5 different "exchanges" in parallel and finds the best price for buying and selling

2. **Parallel Backtest:** Write a function that runs a backtest of one strategy on multiple time periods in parallel and collects the results

3. **Alert System:** Create a system where one thread generates random prices while other threads check various conditions (level crossings, volatility, volume)

4. **Trade History Processing:** Write a program that processes trade history split into "chunks" in parallel and aggregates the results

## Navigation

[← Day 152: Threads: Watching Exchanges in Parallel](../152-threads-watching-exchanges/en.md) | [Day 154: join: Waiting for Thread Completion →](../154-thread-join/en.md)
