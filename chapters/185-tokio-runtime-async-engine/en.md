# Day 185: tokio Runtime: Async Engine

## Trading Analogy

Imagine a trading floor at an exchange. The traditional approach (synchronous) is when one trader processes one order at a time: receives order → waits for counterparty confirmation → logs it → only then takes the next one. If the counterparty takes 5 minutes to respond — the trader just sits and waits.

**tokio runtime** is like a modern electronic trading floor with a smart task dispatcher. Instead of waiting, it says: "While we're waiting for a BTC response, let's process the ETH order. And while ETH is thinking — let's check SOL status". One "trader" (thread) can juggle hundreds of orders, because most of the time they're not working, but waiting for network responses.

In real trading, this is critically important for:
- Connecting to 10 exchanges simultaneously
- Receiving quotes for 100 trading pairs
- Sending orders without blocking data reception
- Monitoring positions while other operations are running

## What is tokio runtime?

**tokio** is an asynchronous runtime for Rust. A runtime is an "engine" that:

1. **Schedules tasks** — decides which async function to execute now
2. **Manages threads** — creates a thread pool for task execution
3. **Handles I/O** — efficiently waits for data from network, files
4. **Provides timers** — for delays and timeouts

```rust
// Future is a promise of a result in the future
// tokio runtime is the one who fulfills these promises

async fn fetch_price(symbol: &str) -> f64 {
    // This doesn't block the thread — tokio will switch to another task
    tokio::time::sleep(Duration::from_millis(100)).await;
    42000.0
}
```

## tokio Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      tokio Runtime                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Reactor   │  │  Scheduler  │  │ Timer Wheel │         │
│  │  (I/O poll) │  │ (task queue)│  │  (delays)   │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
├─────────────────────────────────────────────────────────────┤
│  Thread Pool                                                 │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐               │
│  │Thread 1│ │Thread 2│ │Thread 3│ │Thread N│               │
│  │ Tasks  │ │ Tasks  │ │ Tasks  │ │ Tasks  │               │
│  └────────┘ └────────┘ └────────┘ └────────┘               │
└─────────────────────────────────────────────────────────────┘
```

## Installing tokio

Add to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

Available features:
- `rt` — basic runtime
- `rt-multi-thread` — multi-threaded runtime
- `time` — timers and delays
- `net` — network operations
- `io-util` — I/O utilities
- `sync` — synchronization primitives
- `macros` — `#[tokio::main]` and `#[tokio::test]` macros
- `full` — all features (convenient for development)

## Creating runtime manually

```rust
use tokio::runtime::Runtime;
use std::time::Duration;

fn main() {
    // Create runtime manually
    let rt = Runtime::new().unwrap();

    // Execute async code
    rt.block_on(async {
        println!("Starting trading bot...");

        // Simulate price fetching
        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("BTC price: $42,000");
    });

    println!("Runtime finished");
}
```

## Runtime Types

### Single-threaded runtime

For simple applications or when full control is needed:

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn main() {
    // Single-threaded runtime
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("Single-threaded mode");
        println!("Perfect for a simple bot with one exchange");

        let price = fetch_price("BTC").await;
        println!("Price: ${:.2}", price);
    });
}

async fn fetch_price(_symbol: &str) -> f64 {
    tokio::time::sleep(Duration::from_millis(100)).await;
    42000.0
}
```

### Multi-threaded runtime

For high-performance applications:

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn main() {
    // Multi-threaded runtime
    let rt = Builder::new_multi_thread()
        .worker_threads(4)  // 4 worker threads
        .thread_name("trading-worker")
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("Multi-threaded mode — 4 threads");
        println!("Perfect for a multi-exchange bot");

        // Parallel price fetching
        let (btc, eth, sol) = tokio::join!(
            fetch_price("BTC"),
            fetch_price("ETH"),
            fetch_price("SOL")
        );

        println!("BTC: ${:.2}", btc);
        println!("ETH: ${:.2}", eth);
        println!("SOL: ${:.2}", sol);
    });
}

async fn fetch_price(symbol: &str) -> f64 {
    tokio::time::sleep(Duration::from_millis(100)).await;
    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        "SOL" => 100.0,
        _ => 0.0,
    }
}
```

## Practical Example: Price Monitor

```rust
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration, interval};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct PriceMonitor {
    prices: Arc<RwLock<Vec<PriceData>>>,
}

impl PriceMonitor {
    fn new() -> Self {
        PriceMonitor {
            prices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn fetch_price(&self, symbol: &str) -> f64 {
        // Simulate network request
        sleep(Duration::from_millis(50)).await;

        // Return "random" price
        let base = match symbol {
            "BTC" => 42000.0,
            "ETH" => 2500.0,
            "SOL" => 100.0,
            _ => 50.0,
        };

        // Add small variation
        let variation = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() % 100) as f64;

        base + variation
    }

    async fn update_price(&self, symbol: &str) {
        let price = self.fetch_price(symbol).await;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let data = PriceData {
            symbol: symbol.to_string(),
            price,
            timestamp,
        };

        let mut prices = self.prices.write().await;

        // Update or add
        if let Some(existing) = prices.iter_mut().find(|p| p.symbol == symbol) {
            *existing = data;
        } else {
            prices.push(data);
        }
    }

    async fn get_all_prices(&self) -> Vec<PriceData> {
        self.prices.read().await.clone()
    }
}

fn main() {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let monitor = Arc::new(PriceMonitor::new());

        println!("=== Cryptocurrency Price Monitor ===\n");

        // Start price updates for multiple symbols
        let symbols = vec!["BTC", "ETH", "SOL"];

        // Initial parallel fetch of all prices
        let mut handles = vec![];
        for symbol in &symbols {
            let monitor = Arc::clone(&monitor);
            let sym = symbol.to_string();
            handles.push(tokio::spawn(async move {
                monitor.update_price(&sym).await;
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Display prices
        let prices = monitor.get_all_prices().await;
        for price in &prices {
            println!("{}: ${:.2}", price.symbol, price.price);
        }

        println!("\n=== Periodic Update (3 cycles) ===\n");

        // Periodic updates
        let mut interval = interval(Duration::from_secs(1));

        for cycle in 1..=3 {
            interval.tick().await;

            println!("Cycle {}:", cycle);

            // Update all prices in parallel
            let mut handles = vec![];
            for symbol in &symbols {
                let monitor = Arc::clone(&monitor);
                let sym = symbol.to_string();
                handles.push(tokio::spawn(async move {
                    monitor.update_price(&sym).await;
                }));
            }

            for handle in handles {
                handle.await.unwrap();
            }

            // Display updated prices
            let prices = monitor.get_all_prices().await;
            for price in &prices {
                println!("  {}: ${:.2}", price.symbol, price.price);
            }
        }

        println!("\nMonitoring completed");
    });
}
```

## Example: Trading Engine with Runtime

```rust
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use std::time::Duration;

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
enum OrderResult {
    Filled { order_id: u64, fill_price: f64 },
    Rejected { order_id: u64, reason: String },
}

async fn process_order(order: Order) -> OrderResult {
    // Simulate order processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Simple logic: buys always succeed, sells — 80%
    let success = match order.side {
        OrderSide::Buy => true,
        OrderSide::Sell => order.id % 5 != 0, // every 5th is rejected
    };

    if success {
        OrderResult::Filled {
            order_id: order.id,
            fill_price: order.price,
        }
    } else {
        OrderResult::Rejected {
            order_id: order.id,
            reason: "Insufficient liquidity".to_string(),
        }
    }
}

fn main() {
    // Create runtime with trading-optimized settings
    let rt = Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("order-processor")
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("=== Trading Engine Started ===\n");

        // Channel for sending orders
        let (tx, mut rx) = mpsc::channel::<Order>(100);

        // Task for processing orders
        let processor = tokio::spawn(async move {
            let mut results = Vec::new();

            while let Some(order) = rx.recv().await {
                println!("Processing order #{}: {:?} {} {:.4} @ ${:.2}",
                    order.id, order.side, order.symbol,
                    order.quantity, order.price);

                let result = process_order(order).await;

                match &result {
                    OrderResult::Filled { order_id, fill_price } => {
                        println!("  ✓ Order #{} filled at ${:.2}",
                            order_id, fill_price);
                    }
                    OrderResult::Rejected { order_id, reason } => {
                        println!("  ✗ Order #{} rejected: {}",
                            order_id, reason);
                    }
                }

                results.push(result);
            }

            results
        });

        // Send orders
        let orders = vec![
            Order { id: 1, symbol: "BTC".into(), side: OrderSide::Buy,
                    price: 42000.0, quantity: 0.1 },
            Order { id: 2, symbol: "ETH".into(), side: OrderSide::Buy,
                    price: 2500.0, quantity: 1.0 },
            Order { id: 3, symbol: "BTC".into(), side: OrderSide::Sell,
                    price: 42100.0, quantity: 0.05 },
            Order { id: 4, symbol: "SOL".into(), side: OrderSide::Buy,
                    price: 100.0, quantity: 10.0 },
            Order { id: 5, symbol: "ETH".into(), side: OrderSide::Sell,
                    price: 2550.0, quantity: 0.5 },
        ];

        println!("Sending {} orders...\n", orders.len());

        for order in orders {
            tx.send(order).await.unwrap();
        }

        // Close channel so processor finishes
        drop(tx);

        // Wait for processing to complete
        let results = processor.await.unwrap();

        println!("\n=== Summary ===");
        let filled = results.iter()
            .filter(|r| matches!(r, OrderResult::Filled { .. }))
            .count();
        let rejected = results.len() - filled;

        println!("Filled: {}", filled);
        println!("Rejected: {}", rejected);
    });

    println!("\nTrading engine stopped");
}
```

## Runtime for Different Scenarios

### Scenario 1: Simple Bot with One Exchange

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn main() {
    // Single-threaded — enough for one exchange
    let rt = Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("Simple Binance bot");

        loop {
            let price = fetch_binance_price("BTCUSDT").await;
            println!("BTC: ${:.2}", price);

            if should_trade(price) {
                execute_trade("BUY", 0.01).await;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
            break; // Exit after first iteration for this example
        }
    });
}

async fn fetch_binance_price(_symbol: &str) -> f64 {
    tokio::time::sleep(Duration::from_millis(50)).await;
    42000.0
}

fn should_trade(_price: f64) -> bool {
    false // Trading decision logic
}

async fn execute_trade(side: &str, amount: f64) {
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("Executed {} {}", side, amount);
}
```

### Scenario 2: Multi-Exchange Arbitrage

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn main() {
    // Multi-threaded — for parallel exchange work
    let rt = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("arbitrage")
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        println!("Arbitrage Bot\n");

        // Fetch prices from all exchanges in parallel
        let (binance, bybit, okx) = tokio::join!(
            fetch_exchange_price("Binance", "BTC"),
            fetch_exchange_price("Bybit", "BTC"),
            fetch_exchange_price("OKX", "BTC")
        );

        println!("Binance: ${:.2}", binance);
        println!("Bybit:   ${:.2}", bybit);
        println!("OKX:     ${:.2}", okx);

        // Look for arbitrage
        let min_price = binance.min(bybit).min(okx);
        let max_price = binance.max(bybit).max(okx);
        let spread = (max_price - min_price) / min_price * 100.0;

        println!("\nSpread: {:.2}%", spread);

        if spread > 0.1 {
            println!("Arbitrage opportunity found!");
        } else {
            println!("Arbitrage not profitable");
        }
    });
}

async fn fetch_exchange_price(exchange: &str, _symbol: &str) -> f64 {
    // Simulate different response times for exchanges
    let delay = match exchange {
        "Binance" => 50,
        "Bybit" => 80,
        "OKX" => 60,
        _ => 100,
    };

    tokio::time::sleep(Duration::from_millis(delay)).await;

    // Different prices on different exchanges
    match exchange {
        "Binance" => 42000.0,
        "Bybit" => 42010.0,
        "OKX" => 41995.0,
        _ => 42000.0,
    }
}
```

## Production Runtime Configuration

```rust
use tokio::runtime::Builder;
use std::time::Duration;

fn create_production_runtime() -> tokio::runtime::Runtime {
    Builder::new_multi_thread()
        // Thread count = number of CPU cores
        .worker_threads(num_cpus::get())
        // Thread names for debugging
        .thread_name("trading-runtime")
        // Stack size (default is 2MB)
        .thread_stack_size(3 * 1024 * 1024)
        // Enable all features
        .enable_all()
        // Build the runtime
        .build()
        .expect("Failed to create tokio runtime")
}

fn main() {
    let rt = create_production_runtime();

    rt.block_on(async {
        println!("Production runtime started");
        println!("Threads: {}", num_cpus::get());

        // Your trading code here
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("Work completed");
    });
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| tokio runtime | Engine for executing async code |
| Runtime::new() | Create runtime with default settings |
| Builder | Fine-tune runtime configuration |
| new_current_thread() | Single-threaded runtime |
| new_multi_thread() | Multi-threaded runtime |
| block_on() | Execute Future in synchronous context |
| worker_threads() | Configure number of threads |

## Homework

1. **Basic runtime**: Create a program that uses manual runtime creation to fetch prices for three cryptocurrencies in parallel. Measure execution time and compare with sequential fetching.

2. **Thread configuration**: Create two runtimes — single-threaded and multi-threaded. Run 10 parallel tasks (simulating price fetching with 100ms delay) in each. Compare execution times.

3. **Trading server**: Implement a `TradingServer` struct with methods:
   - `new(threads: usize)` — create with specified thread count
   - `run(&self, handler: F)` — run with handler
   - `shutdown(self)` — graceful shutdown

   The server should accept orders via a channel and process them in parallel.

4. **Profiling**: Add callbacks to the runtime for monitoring:
   - Number of active tasks
   - Work time of each thread
   - Memory usage

   Output statistics after work completion.

## Navigation

[← Previous day](../184-future-promise-result/en.md) | [Next day →](../186-tokio-main-entry-point/en.md)
