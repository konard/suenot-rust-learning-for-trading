# Day 175: Thread Pool: Limiting Parallelism

## Trading Analogy

Imagine a trading floor at an exchange. If every trader hired a new assistant for each trade, the floor would quickly become overcrowded, with people bumping into each other and getting in the way. It's much more efficient to have a fixed team of, say, 8 traders who take orders from a queue and execute them as they become available.

This is exactly what a **thread pool** is — a pre-created set of threads that wait for tasks and execute them. Instead of creating a new thread for each task (which is expensive), we reuse existing threads.

In algorithmic trading, thread pools are critically important:
- Simultaneously analyzing 1000 stocks shouldn't create 1000 threads
- Processing market data should be limited by CPU capabilities
- Order execution requires controlled parallelism

## Why Limit Parallelism?

```
Without thread pool:                 With thread pool:
┌─────────────────────┐              ┌─────────────────────┐
│ 1000 tasks          │              │ 1000 tasks          │
│        ↓            │              │        ↓            │
│ 1000 threads!       │              │ Task queue          │
│ (context switching, │              │        ↓            │
│  starvation,        │              │ 8 threads (= CPU)   │
│  resource           │              │ (efficient          │
│  exhaustion)        │              │  resource usage)    │
└─────────────────────┘              └─────────────────────┘
```

## Simple Thread Pool by Hand

First, let's see how a thread pool works internally:

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    println!("Worker {} received a job", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} shutting down", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

fn main() {
    let pool = ThreadPool::new(4);

    // Simulating analysis of 10 stocks
    for i in 0..10 {
        pool.execute(move || {
            let symbol = format!("STOCK_{}", i);
            println!("Analyzing {}", symbol);
            thread::sleep(std::time::Duration::from_millis(100));
            println!("{}: analysis complete", symbol);
        });
    }

    // Pool will automatically wait for completion when going out of scope
}
```

## Using rayon — The Industry Standard

In practice, the `rayon` library is used, which provides a powerful and optimized thread pool:

```rust
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct StockAnalysis {
    symbol: String,
    price: f64,
    sma_20: f64,
    sma_50: f64,
    signal: Signal,
}

#[derive(Debug, Clone)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

fn analyze_stock(symbol: &str, prices: &[f64]) -> StockAnalysis {
    // Calculate moving averages
    let sma_20 = if prices.len() >= 20 {
        prices.iter().rev().take(20).sum::<f64>() / 20.0
    } else {
        prices.iter().sum::<f64>() / prices.len() as f64
    };

    let sma_50 = if prices.len() >= 50 {
        prices.iter().rev().take(50).sum::<f64>() / 50.0
    } else {
        prices.iter().sum::<f64>() / prices.len() as f64
    };

    let current_price = *prices.last().unwrap();

    let signal = if sma_20 > sma_50 && current_price > sma_20 {
        Signal::Buy
    } else if sma_20 < sma_50 && current_price < sma_20 {
        Signal::Sell
    } else {
        Signal::Hold
    };

    StockAnalysis {
        symbol: symbol.to_string(),
        price: current_price,
        sma_20,
        sma_50,
        signal,
    }
}

fn main() {
    // Simulating data for 100 stocks
    let stocks: Vec<(String, Vec<f64>)> = (0..100)
        .map(|i| {
            let symbol = format!("STOCK_{:03}", i);
            let prices: Vec<f64> = (0..100)
                .map(|j| 100.0 + (i as f64 * 0.1) + (j as f64 * 0.01))
                .collect();
            (symbol, prices)
        })
        .collect();

    // Parallel analysis with limited threads (default = CPU cores)
    let results: Vec<StockAnalysis> = stocks
        .par_iter()  // Parallel iterator!
        .map(|(symbol, prices)| analyze_stock(symbol, prices))
        .collect();

    // Output buy signals
    let buy_signals: Vec<_> = results
        .iter()
        .filter(|a| matches!(a.signal, Signal::Buy))
        .collect();

    println!("Found {} buy signals:", buy_signals.len());
    for analysis in buy_signals.iter().take(5) {
        println!(
            "  {} @ ${:.2} (SMA20: {:.2}, SMA50: {:.2})",
            analysis.symbol, analysis.price, analysis.sma_20, analysis.sma_50
        );
    }
}
```

## Configuring Pool Size in rayon

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

fn main() {
    // Create global pool with 4 threads
    ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    let symbols: Vec<String> = (0..20)
        .map(|i| format!("CRYPTO_{}", i))
        .collect();

    // Now parallel operations use only 4 threads
    symbols.par_iter().for_each(|symbol| {
        println!(
            "Thread {:?} processing {}",
            std::thread::current().id(),
            symbol
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    });
}
```

## Local Thread Pools

Sometimes you need multiple independent pools:

```rust
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::sync::Arc;

struct TradingSystem {
    data_pool: rayon::ThreadPool,     // For market data processing
    analysis_pool: rayon::ThreadPool,  // For analysis
    order_pool: rayon::ThreadPool,     // For order handling
}

impl TradingSystem {
    fn new() -> Self {
        TradingSystem {
            // Lots of data — many threads
            data_pool: ThreadPoolBuilder::new()
                .num_threads(8)
                .thread_name(|i| format!("data-worker-{}", i))
                .build()
                .unwrap(),

            // Analysis is CPU-intensive — match core count
            analysis_pool: ThreadPoolBuilder::new()
                .num_threads(num_cpus::get())
                .thread_name(|i| format!("analysis-worker-{}", i))
                .build()
                .unwrap(),

            // Orders are critical — fewer threads, higher priority
            order_pool: ThreadPoolBuilder::new()
                .num_threads(2)
                .thread_name(|i| format!("order-worker-{}", i))
                .build()
                .unwrap(),
        }
    }

    fn process_market_data(&self, data: Vec<f64>) -> Vec<f64> {
        self.data_pool.install(|| {
            data.par_iter()
                .map(|&price| price * 1.0001) // Simulating processing
                .collect()
        })
    }

    fn analyze_positions(&self, positions: Vec<(String, f64)>) -> Vec<String> {
        self.analysis_pool.install(|| {
            positions
                .par_iter()
                .filter(|(_, value)| *value > 10000.0)
                .map(|(symbol, value)| {
                    format!("{}: ${:.2} - requires attention", symbol, value)
                })
                .collect()
        })
    }
}

fn main() {
    let system = TradingSystem::new();

    // Simulating market data
    let prices: Vec<f64> = (0..1000).map(|i| 100.0 + i as f64 * 0.1).collect();

    // Processing in data_pool
    let processed = system.process_market_data(prices);
    println!("Processed {} prices", processed.len());

    // Analyzing positions in analysis_pool
    let positions: Vec<(String, f64)> = vec![
        ("BTC".to_string(), 50000.0),
        ("ETH".to_string(), 3000.0),
        ("SOL".to_string(), 15000.0),
    ];

    let alerts = system.analyze_positions(positions);
    for alert in alerts {
        println!("{}", alert);
    }
}
```

## Thread Pool for Order Processing

```rust
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
use std::collections::VecDeque;
use rayon::prelude::*;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: f64,
}

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct ExecutionResult {
    order_id: u64,
    status: ExecutionStatus,
    filled_price: Option<f64>,
}

#[derive(Debug, Clone)]
enum ExecutionStatus {
    Filled,
    PartiallyFilled,
    Rejected(String),
}

struct OrderExecutor {
    order_counter: AtomicU64,
    execution_log: Arc<Mutex<Vec<ExecutionResult>>>,
}

impl OrderExecutor {
    fn new() -> Self {
        OrderExecutor {
            order_counter: AtomicU64::new(0),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn execute_order(&self, order: &Order) -> ExecutionResult {
        // Simulating order validation and execution
        std::thread::sleep(std::time::Duration::from_millis(10));

        let result = if order.quantity > 1000.0 {
            ExecutionResult {
                order_id: order.id,
                status: ExecutionStatus::Rejected(
                    "Order size too large".to_string()
                ),
                filled_price: None,
            }
        } else {
            // Simulating slippage
            let slippage = match order.side {
                OrderSide::Buy => 1.001,
                OrderSide::Sell => 0.999,
            };

            ExecutionResult {
                order_id: order.id,
                status: ExecutionStatus::Filled,
                filled_price: Some(order.price * slippage),
            }
        };

        self.execution_log.lock().unwrap().push(result.clone());
        result
    }

    fn process_batch(&self, orders: Vec<Order>) -> Vec<ExecutionResult> {
        // Parallel processing with limited parallelism
        orders
            .par_iter()
            .map(|order| self.execute_order(order))
            .collect()
    }

    fn get_stats(&self) -> (usize, usize, usize) {
        let log = self.execution_log.lock().unwrap();
        let filled = log.iter()
            .filter(|r| matches!(r.status, ExecutionStatus::Filled))
            .count();
        let partial = log.iter()
            .filter(|r| matches!(r.status, ExecutionStatus::PartiallyFilled))
            .count();
        let rejected = log.iter()
            .filter(|r| matches!(r.status, ExecutionStatus::Rejected(_)))
            .count();
        (filled, partial, rejected)
    }
}

fn main() {
    // Limit to 4 threads for order processing
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    let executor = OrderExecutor::new();

    // Create a batch of orders
    let orders: Vec<Order> = (0..50)
        .map(|i| Order {
            id: i,
            symbol: if i % 2 == 0 { "BTC" } else { "ETH" }.to_string(),
            side: if i % 3 == 0 { OrderSide::Sell } else { OrderSide::Buy },
            quantity: 10.0 + (i as f64 * 50.0),
            price: 42000.0 + (i as f64 * 10.0),
        })
        .collect();

    println!("Processing {} orders...", orders.len());

    let start = std::time::Instant::now();
    let results = executor.process_batch(orders);
    let elapsed = start.elapsed();

    println!("Processed in {:?}", elapsed);

    let (filled, partial, rejected) = executor.get_stats();
    println!("\nResults:");
    println!("  Filled: {}", filled);
    println!("  Partial: {}", partial);
    println!("  Rejected: {}", rejected);

    // Show first 5 results
    println!("\nExecution examples:");
    for result in results.iter().take(5) {
        match &result.status {
            ExecutionStatus::Filled => {
                println!(
                    "  Order {}: filled @ ${:.2}",
                    result.order_id,
                    result.filled_price.unwrap()
                );
            }
            ExecutionStatus::Rejected(reason) => {
                println!("  Order {}: rejected - {}", result.order_id, reason);
            }
            ExecutionStatus::PartiallyFilled => {
                println!("  Order {}: partially filled", result.order_id);
            }
        }
    }
}
```

## Load Control with Semaphore

Sometimes you need even finer control:

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
struct RateLimitedClient {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl RateLimitedClient {
    fn new(max_concurrent: usize) -> Self {
        RateLimitedClient {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    async fn fetch_price(&self, symbol: &str) -> Result<f64, String> {
        // Wait for semaphore permit
        let _permit = self.semaphore.acquire().await.unwrap();

        println!(
            "Fetching {} price (active requests: {})",
            symbol,
            self.max_concurrent - self.semaphore.available_permits()
        );

        // Simulating API request
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Permit is automatically released here
        Ok(42000.0 + rand::random::<f64>() * 1000.0)
    }
}

#[tokio::main]
async fn main() {
    // Maximum 3 concurrent API requests
    let client = RateLimitedClient::new(3);

    let symbols = vec!["BTC", "ETH", "SOL", "ADA", "DOT", "LINK", "AVAX", "MATIC"];

    let mut handles = vec![];

    for symbol in symbols {
        let client = client.clone();
        let symbol = symbol.to_string();

        handles.push(tokio::spawn(async move {
            match client.fetch_price(&symbol).await {
                Ok(price) => println!("{}: ${:.2}", symbol, price),
                Err(e) => println!("{}: error - {}", symbol, e),
            }
        }));
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }
}
```

## Comparing Approaches

| Approach | When to Use | Advantages |
|----------|-------------|------------|
| Manual ThreadPool | Learning, simple cases | Full control |
| rayon | CPU-bound tasks | Automatic parallelism |
| tokio + Semaphore | Async I/O | Concurrency control |
| Multiple pools | Different priorities | Task isolation |

## What We Learned

| Concept | Description |
|---------|-------------|
| Thread Pool | A set of reusable threads for executing tasks |
| Work Stealing | Threads steal tasks from each other (rayon) |
| Limiting Parallelism | Control over the number of concurrent operations |
| rayon | Library for parallel iterators |
| Semaphore | Limiting concurrency in async code |

## Practice Exercises

1. **Simple Thread Pool**: Implement your own ThreadPool with an `execute()` method and proper shutdown via Drop.

2. **Parallel Stock Screener**: Using rayon, create a function that:
   - Takes a list of 1000 stock symbols
   - Calculates technical indicators for each
   - Returns the top-10 by some criterion

3. **Rate-limited API Client**: Implement a client with limitations:
   - Maximum 10 requests per second
   - Maximum 3 concurrent requests
   - Queue for pending requests

4. **Priority Pools**: Create a system with three pools:
   - High Priority (2 threads) — for critical operations
   - Normal (4 threads) — for regular tasks
   - Background (2 threads) — for background computations

## Homework

1. **Backtesting with Thread Pool**: Write a program that tests a trading strategy in parallel across 100 different instruments. Use rayon to limit parallelism to the number of CPU cores.

2. **Position Monitoring**: Create a system that checks PnL for 50 open positions every second. Limit parallel computations to 8 threads and collect processing time statistics.

3. **Concurrent Exchange**: Implement an `ExchangeSimulator` struct that:
   - Has a pool for processing incoming orders
   - Has a pool for order matching
   - Has a pool for sending notifications
   - Each pool is limited to a different number of threads

4. **Adaptive Pool**: Create a pool that:
   - Starts with 2 threads
   - Scales up to 8 if the task queue grows
   - Scales back down under low load
   - Logs size changes

## Navigation

[← Previous day](../174-scoped-threads/en.md) | [Next day →](../176-work-stealing/en.md)
