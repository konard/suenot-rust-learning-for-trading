# Day 326: Async vs Threading: Choosing Model

## Trading Analogy

Imagine you're managing a trading floor with multiple workstations. You have two approaches to handle incoming orders:

**Threading Model:**
You hire several traders. Each trader sits at their own workstation and processes their orders independently. When one trader is waiting for a trade confirmation from the exchange, they just sit and wait — their workstation is occupied.

**Async Model:**
You have one trader, but they work very efficiently. When they send an order to the exchange and wait for a response, they don't sit idle — they switch to another order. When the response for the first order arrives, they return to it.

| Criteria | Threading | Async |
|----------|-----------|-------|
| **Analogy** | Multiple traders | One multi-tasking trader |
| **Waiting for I/O** | Blocks thread | Switches to another task |
| **Memory** | ~2-8 MB per thread | ~1-4 KB per task |
| **Context switching** | Expensive (OS) | Cheap (runtime) |
| **Best for** | CPU-intensive tasks | I/O-intensive tasks |

## When to Choose Threading?

Threading is suitable for **CPU-bound** tasks where real parallel CPU processing is needed:

```rust
use std::thread;
use std::time::Instant;
use std::sync::{Arc, Mutex};

/// Moving average calculation — CPU-intensive operation
fn calculate_sma(prices: &[f64], window: usize) -> Vec<f64> {
    prices
        .windows(window)
        .map(|w| w.iter().sum::<f64>() / window as f64)
        .collect()
}

/// RSI calculation — also requires CPU
fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    let mut rsi_values = Vec::new();
    let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

        let rs = if avg_loss != 0.0 {
            avg_gain / avg_loss
        } else {
            100.0
        };
        rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
    }

    rsi_values
}

fn main() {
    // Generate data for 100 instruments
    let instruments: Vec<Vec<f64>> = (0..100)
        .map(|i| {
            (0..10000)
                .map(|j| 100.0 + (i as f64 * 0.01) + (j as f64 * 0.001).sin())
                .collect()
        })
        .collect();

    println!("=== Comparing Threading vs Sequential ===\n");

    // Sequential calculation
    let start = Instant::now();
    let mut sequential_results = Vec::new();
    for prices in &instruments {
        let sma = calculate_sma(prices, 20);
        let rsi = calculate_rsi(prices, 14);
        sequential_results.push((sma, rsi));
    }
    let sequential_time = start.elapsed();
    println!("Sequential: {:?}", sequential_time);

    // Multi-threaded calculation
    let start = Instant::now();
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // Split into chunks for threads
    let chunk_size = instruments.len() / 4;
    for chunk in instruments.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let results = Arc::clone(&results);

        let handle = thread::spawn(move || {
            let mut local_results = Vec::new();
            for prices in &chunk {
                let sma = calculate_sma(prices, 20);
                let rsi = calculate_rsi(prices, 14);
                local_results.push((sma, rsi));
            }
            results.lock().unwrap().extend(local_results);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let threaded_time = start.elapsed();
    println!("Multi-threaded (4 threads): {:?}", threaded_time);

    println!(
        "\nSpeedup: {:.2}x",
        sequential_time.as_secs_f64() / threaded_time.as_secs_f64()
    );
}
```

## When to Choose Async?

Async is suitable for **I/O-bound** tasks where the program spends most of its time waiting for external events:

```rust
use std::time::Duration;

// Simulating async request to exchange
async fn fetch_price(exchange: &str, symbol: &str) -> Result<f64, String> {
    // Simulating network delay
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Return "price" based on hash for demonstration
    let hash = exchange.len() + symbol.len();
    Ok(50000.0 + (hash as f64 * 100.0))
}

async fn fetch_order_book(exchange: &str, symbol: &str) -> Result<(Vec<f64>, Vec<f64>), String> {
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Simulating order book
    let bids = vec![49900.0, 49850.0, 49800.0];
    let asks = vec![50100.0, 50150.0, 50200.0];
    Ok((bids, asks))
}

async fn submit_order(exchange: &str, symbol: &str, side: &str, price: f64, qty: f64)
    -> Result<String, String>
{
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(format!("ORDER-{}-{}-{}", exchange, symbol, price as u64))
}

#[tokio::main]
async fn main() {
    use std::time::Instant;

    println!("=== Async: Parallel Requests to Exchanges ===\n");

    let exchanges = ["binance", "kraken", "coinbase", "bybit"];
    let symbol = "BTCUSDT";

    // Sequential requests
    let start = Instant::now();
    for exchange in &exchanges {
        let price = fetch_price(exchange, symbol).await.unwrap();
        println!("{}: ${:.2}", exchange, price);
    }
    println!("Sequential: {:?}\n", start.elapsed());

    // Parallel requests with async
    let start = Instant::now();
    let futures: Vec<_> = exchanges
        .iter()
        .map(|exchange| async move {
            let price = fetch_price(exchange, symbol).await?;
            Ok::<_, String>((exchange.to_string(), price))
        })
        .collect();

    let results = futures::future::join_all(futures).await;
    for result in results {
        if let Ok((exchange, price)) = result {
            println!("{}: ${:.2}", exchange, price);
        }
    }
    println!("Parallel (async): {:?}", start.elapsed());
}
```

## Hybrid Approach: Async + Threading

Real trading systems often use a combination of both approaches:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Market data — updated asynchronously
#[derive(Clone, Debug)]
struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume: f64,
}

/// Trading signal — result of CPU-intensive calculations
#[derive(Debug, Clone)]
struct TradingSignal {
    symbol: String,
    action: String,  // "BUY", "SELL", "HOLD"
    confidence: f64,
    price_target: f64,
}

/// Market data cache with thread-safe access
struct MarketDataCache {
    data: Arc<RwLock<HashMap<String, MarketData>>>,
}

impl MarketDataCache {
    fn new() -> Self {
        MarketDataCache {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn update(&self, market_data: MarketData) {
        let mut data = self.data.write().await;
        data.insert(market_data.symbol.clone(), market_data);
    }

    async fn get(&self, symbol: &str) -> Option<MarketData> {
        let data = self.data.read().await;
        data.get(symbol).cloned()
    }

    async fn get_all(&self) -> Vec<MarketData> {
        let data = self.data.read().await;
        data.values().cloned().collect()
    }
}

/// CPU-intensive analysis — runs in a separate thread
fn analyze_market_data(data: Vec<MarketData>) -> Vec<TradingSignal> {
    // Simulating complex calculations
    data.iter()
        .map(|md| {
            // Simple strategy based on spread
            let spread_pct = (md.ask - md.bid) / md.bid * 100.0;

            let (action, confidence) = if spread_pct < 0.1 {
                ("BUY", 0.8)
            } else if spread_pct > 0.5 {
                ("SELL", 0.7)
            } else {
                ("HOLD", 0.5)
            };

            TradingSignal {
                symbol: md.symbol.clone(),
                action: action.to_string(),
                confidence,
                price_target: md.last_price * if action == "BUY" { 1.02 } else { 0.98 },
            }
        })
        .collect()
}

#[tokio::main]
async fn main() {
    println!("=== Hybrid Approach: Async + Threading ===\n");

    let cache = Arc::new(MarketDataCache::new());

    // Async data update (simulating WebSocket)
    let cache_clone = Arc::clone(&cache);
    let update_task = tokio::spawn(async move {
        let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "ADAUSDT"];

        for (i, symbol) in symbols.iter().enumerate() {
            let md = MarketData {
                symbol: symbol.to_string(),
                bid: 50000.0 + (i as f64 * 1000.0),
                ask: 50010.0 + (i as f64 * 1000.0),
                last_price: 50005.0 + (i as f64 * 1000.0),
                volume: 1000000.0,
            };
            cache_clone.update(md).await;
            println!("Updated data: {}", symbol);
        }
    });

    // Wait for data loading
    update_task.await.unwrap();

    // Get all data for analysis
    let all_data = cache.get_all().await;
    println!("\nLoaded {} instruments", all_data.len());

    // CPU-intensive analysis in a separate thread
    let signals = tokio::task::spawn_blocking(move || {
        println!("Starting analysis in separate thread...");
        analyze_market_data(all_data)
    })
    .await
    .unwrap();

    println!("\n=== Trading Signals ===");
    for signal in signals {
        println!(
            "{}: {} (confidence: {:.0}%, target: ${:.2})",
            signal.symbol, signal.action, signal.confidence * 100.0, signal.price_target
        );
    }
}
```

## Comparing Models

### Memory Overhead

```rust
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};

static TASK_COUNT: AtomicUsize = AtomicUsize::new(0);

fn thread_memory_demo() {
    println!("=== Memory: Threads vs Async ===\n");

    // Threads: each takes ~2-8 MB of stack
    println!("Creating 10 threads...");
    let mut handles = vec![];

    for i in 0..10 {
        let handle = thread::spawn(move || {
            // Each thread has its own stack
            let local_data: [u8; 1024] = [0; 1024];
            thread::sleep(std::time::Duration::from_millis(100));
            local_data[0] + i as u8
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
    println!("Threads completed (each used ~2MB of stack)\n");

    // Async: tasks take minimal memory
    println!("Creating 10000 async tasks...");
    // In real code this would be:
    // let runtime = tokio::runtime::Runtime::new().unwrap();
    // runtime.block_on(async {
    //     let futures: Vec<_> = (0..10000)
    //         .map(|_| async { tokio::time::sleep(Duration::from_millis(100)).await })
    //         .collect();
    //     futures::future::join_all(futures).await;
    // });
    println!("Async tasks use ~KB of memory each");
}

fn main() {
    thread_memory_demo();
}
```

### Model Selection Guide

| Scenario | Recommended Model | Reason |
|----------|-------------------|--------|
| WebSocket connections to exchanges | Async | I/O-bound, lots of waiting |
| REST API requests | Async | Network latency |
| Indicator calculations | Threading | CPU-bound |
| Strategy backtesting | Threading | Intensive computations |
| Order processing | Async | I/O operations with exchange |
| Big data analysis | Threading | CPU-bound |
| Event-driven trading | Async | Reacting to events |
| Monte-Carlo simulations | Threading | Computationally intensive |

## Pattern: Actor Model for Trading Systems

Actor Model fits well for organizing trading systems:

```rust
use tokio::sync::mpsc;
use std::collections::HashMap;

/// Messages for market data actor
#[derive(Debug, Clone)]
enum MarketMessage {
    PriceUpdate { symbol: String, price: f64 },
    GetPrice { symbol: String, response: mpsc::Sender<Option<f64>> },
    Subscribe { symbol: String },
}

/// Messages for order actor
#[derive(Debug)]
enum OrderMessage {
    PlaceOrder { symbol: String, side: String, price: f64, qty: f64 },
    CancelOrder { order_id: String },
    GetOpenOrders { response: mpsc::Sender<Vec<String>> },
}

/// Market data actor
async fn market_data_actor(mut rx: mpsc::Receiver<MarketMessage>) {
    let mut prices: HashMap<String, f64> = HashMap::new();
    let mut subscriptions: Vec<String> = Vec::new();

    println!("[MarketData Actor] Started");

    while let Some(msg) = rx.recv().await {
        match msg {
            MarketMessage::PriceUpdate { symbol, price } => {
                prices.insert(symbol.clone(), price);
                println!("[MarketData] Update: {} = ${:.2}", symbol, price);
            }
            MarketMessage::GetPrice { symbol, response } => {
                let price = prices.get(&symbol).copied();
                let _ = response.send(price).await;
            }
            MarketMessage::Subscribe { symbol } => {
                if !subscriptions.contains(&symbol) {
                    subscriptions.push(symbol.clone());
                    println!("[MarketData] Subscribed to: {}", symbol);
                }
            }
        }
    }
}

/// Order manager actor
async fn order_manager_actor(
    mut rx: mpsc::Receiver<OrderMessage>,
    market_tx: mpsc::Sender<MarketMessage>,
) {
    let mut orders: Vec<String> = Vec::new();
    let mut order_counter = 0u64;

    println!("[OrderManager Actor] Started");

    while let Some(msg) = rx.recv().await {
        match msg {
            OrderMessage::PlaceOrder { symbol, side, price, qty } => {
                order_counter += 1;
                let order_id = format!("ORD-{:06}", order_counter);
                orders.push(order_id.clone());

                println!(
                    "[OrderManager] New order: {} {} {} {} @ ${:.2}",
                    order_id, side, qty, symbol, price
                );

                // Request current price through market data actor
                let (resp_tx, mut resp_rx) = mpsc::channel(1);
                let _ = market_tx.send(MarketMessage::GetPrice {
                    symbol: symbol.clone(),
                    response: resp_tx,
                }).await;

                if let Some(Some(current_price)) = resp_rx.recv().await {
                    let diff = (price - current_price).abs() / current_price * 100.0;
                    println!(
                        "[OrderManager] Current price: ${:.2}, deviation: {:.2}%",
                        current_price, diff
                    );
                }
            }
            OrderMessage::CancelOrder { order_id } => {
                orders.retain(|id| id != &order_id);
                println!("[OrderManager] Order cancelled: {}", order_id);
            }
            OrderMessage::GetOpenOrders { response } => {
                let _ = response.send(orders.clone()).await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Actor Model for Trading System ===\n");

    // Create channels for actors
    let (market_tx, market_rx) = mpsc::channel::<MarketMessage>(100);
    let (order_tx, order_rx) = mpsc::channel::<OrderMessage>(100);

    // Start actors
    let market_handle = tokio::spawn(market_data_actor(market_rx));
    let order_handle = tokio::spawn(order_manager_actor(order_rx, market_tx.clone()));

    // Simulate system operation
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Subscribe to instruments
    market_tx.send(MarketMessage::Subscribe {
        symbol: "BTCUSDT".to_string()
    }).await.unwrap();

    // Update prices
    market_tx.send(MarketMessage::PriceUpdate {
        symbol: "BTCUSDT".to_string(),
        price: 50000.0
    }).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Place orders
    order_tx.send(OrderMessage::PlaceOrder {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        price: 49900.0,
        qty: 0.1,
    }).await.unwrap();

    order_tx.send(OrderMessage::PlaceOrder {
        symbol: "BTCUSDT".to_string(),
        side: "SELL".to_string(),
        price: 50100.0,
        qty: 0.1,
    }).await.unwrap();

    // Allow time for processing
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Get list of open orders
    let (resp_tx, mut resp_rx) = mpsc::channel(1);
    order_tx.send(OrderMessage::GetOpenOrders { response: resp_tx }).await.unwrap();

    if let Some(orders) = resp_rx.recv().await {
        println!("\nOpen orders: {:?}", orders);
    }

    // Close channels to terminate actors
    drop(market_tx);
    drop(order_tx);

    let _ = market_handle.await;
    let _ = order_handle.await;

    println!("\nActors terminated");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Threading** | Parallel execution across multiple CPU cores |
| **Async** | Cooperative multitasking for I/O operations |
| **CPU-bound** | Tasks limited by computational power |
| **I/O-bound** | Tasks limited by input/output speed |
| **spawn_blocking** | Running CPU tasks from async context |
| **Actor Model** | Pattern for state isolation through messages |
| **Hybrid approach** | Combining async for I/O and threading for CPU |

## Practical Exercises

1. **Parallel Data Loader**: Create a system that:
   - Asynchronously loads historical data from multiple exchanges
   - Uses threads for parallel data processing
   - Combines results and saves in a unified format
   - Shows loading progress

2. **Distributed Indicator Calculator**: Implement a system:
   - Uses thread pool for heavy indicator calculations
   - Asynchronously receives market data
   - Caches calculation results
   - Updates indicators on new data

3. **Performance Monitor**: Create a tool:
   - Compares execution time of async vs threading
   - Measures memory usage
   - Visualizes results
   - Provides recommendations for model selection

4. **Event-driven Trading Bot**: Implement a bot:
   - Uses async for event processing
   - Applies threading for complex calculations
   - Manages state through actors
   - Logs all operations

## Homework

1. **Async vs Threading Benchmark**: Write a test that:
   - Creates 1000 tasks with varying I/O to CPU ratios
   - Measures execution time for both models
   - Finds the tipping point where one model outperforms the other
   - Generates report with graphs
   - Provides recommendations based on results

2. **Trading System with Hot Switching**: Implement a system:
   - Allows switching between async and threading at runtime
   - Preserves state during switching
   - Measures performance of both modes
   - Automatically selects optimal mode
   - Logs switching reasons

3. **Worker Pool for Analysis**: Create a pool:
   - Dynamically scales under load
   - Distributes CPU tasks across threads
   - Uses async for coordination
   - Collects performance metrics
   - Has graceful shutdown mechanism

4. **Exchange Simulator**: Develop a simulator:
   - Async handling of client connections
   - Threading for matching engine
   - Actor model for component isolation
   - Testing under high load
   - Operation latency monitoring

## Navigation

[← Previous day](../319-memory-tracking-leaks/en.md) | [Next day →](../352-publishing-crates-io/en.md)
