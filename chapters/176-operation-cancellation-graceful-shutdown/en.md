# Day 176: Operation Cancellation: Graceful Shutdown

## Trading Analogy

Imagine you've launched a trading bot that processes market data, places orders, and monitors positions. Suddenly, you receive a critical notification from the exchange — system maintenance in 5 minutes. What do you do?

**Bad option:** Just kill the process. Result:
- Open orders left hanging on the exchange
- Positions remain unclosed
- Data may be lost
- Balance not saved

**Good option (Graceful Shutdown):**
1. Receive the stop signal
2. Stop accepting new orders
3. Cancel all active orders on the exchange
4. Wait for all positions to close (or close them at market price)
5. Save state to the database
6. Properly close all connections

This is **graceful shutdown** — a controlled termination where all operations complete correctly, resources are released, and the system remains in a consistent state.

## Why Do We Need Graceful Shutdown?

In real trading systems, graceful shutdown is critically important:

| Scenario | Without graceful shutdown | With graceful shutdown |
|----------|--------------------------|------------------------|
| Deploying new version | Active orders lost | All orders safely cancelled |
| Server maintenance | Position mismatch | Positions saved to DB |
| Critical error | Hanging WebSocket connections | All connections closed |
| Manual stop | Resource leaks | Memory and files released |

## Signals in Unix/Linux

Main signals for graceful shutdown:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // Flag for stop signal
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown_requested);

    // Ctrl+C (SIGINT) handler
    ctrlc::set_handler(move || {
        println!("\nReceived stop signal (Ctrl+C)...");
        shutdown_clone.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    println!("Trading bot started. Press Ctrl+C to stop.");

    // Main loop
    while !shutdown_requested.load(Ordering::SeqCst) {
        println!("Processing market data...");
        thread::sleep(Duration::from_secs(1));
    }

    println!("Executing graceful shutdown...");
    // Proper shutdown logic here
    println!("Bot stopped.");
}
```

**Note:** To compile, add to `Cargo.toml`:
```toml
[dependencies]
ctrlc = "3.4"
```

## Pattern: Cancellation Token

In multithreaded applications, it's convenient to use a cancellation token:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Token for coordinating operation cancellation
#[derive(Clone)]
struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    fn new() -> Self {
        CancellationToken {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if cancellation was requested
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Request cancellation
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

/// Market data collector
fn market_data_collector(token: CancellationToken, symbol: &str) {
    println!("[{}] Data collector started", symbol);

    while !token.is_cancelled() {
        // Simulate receiving data
        println!("[{}] Quote received: ${:.2}", symbol, 42000.0 + rand_price());
        thread::sleep(Duration::from_millis(500));
    }

    println!("[{}] Data collector stopped", symbol);
}

/// Order executor
fn order_executor(token: CancellationToken) {
    println!("[OrderExecutor] Started");

    let mut pending_orders = 5; // Simulating active orders

    while !token.is_cancelled() || pending_orders > 0 {
        if token.is_cancelled() {
            // Cancel remaining orders
            println!("[OrderExecutor] Cancelling order... (remaining: {})", pending_orders);
            pending_orders -= 1;
            thread::sleep(Duration::from_millis(200));
        } else {
            // Normal operation
            println!("[OrderExecutor] Waiting for new orders...");
            thread::sleep(Duration::from_secs(1));
        }
    }

    println!("[OrderExecutor] All orders cancelled, stopping");
}

fn rand_price() -> f64 {
    // Simple "randomness" for the example
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64
}

fn main() {
    let token = CancellationToken::new();

    // Start data collectors
    let btc_token = token.clone();
    let btc_collector = thread::spawn(move || {
        market_data_collector(btc_token, "BTC");
    });

    let eth_token = token.clone();
    let eth_collector = thread::spawn(move || {
        market_data_collector(eth_token, "ETH");
    });

    // Start order executor
    let executor_token = token.clone();
    let executor = thread::spawn(move || {
        order_executor(executor_token);
    });

    // Simulate work
    thread::sleep(Duration::from_secs(3));

    // Initiate graceful shutdown
    println!("\n=== Initiating graceful shutdown ===\n");
    token.cancel();

    // Wait for all threads to complete
    btc_collector.join().unwrap();
    eth_collector.join().unwrap();
    executor.join().unwrap();

    println!("\n=== All components stopped ===");
}
```

## Channels for Shutdown Coordination

For more flexible coordination, we use channels:

```rust
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Commands for worker control
enum WorkerCommand {
    ProcessOrder { id: u64, symbol: String, quantity: f64 },
    Shutdown,
}

/// Worker results
enum WorkerResult {
    OrderProcessed { id: u64, success: bool },
    ShutdownComplete { pending_orders: usize },
}

fn order_worker(
    commands: Receiver<WorkerCommand>,
    results: Sender<WorkerResult>,
) {
    let mut pending = Vec::new();

    loop {
        match commands.recv() {
            Ok(WorkerCommand::ProcessOrder { id, symbol, quantity }) => {
                println!("Processing order #{}: {} x {}", id, symbol, quantity);
                pending.push(id);

                // Simulate processing
                thread::sleep(Duration::from_millis(100));

                pending.retain(|&x| x != id);
                let _ = results.send(WorkerResult::OrderProcessed { id, success: true });
            }

            Ok(WorkerCommand::Shutdown) => {
                println!("Shutdown command received, cancelling {} orders...", pending.len());

                // Cancel all active orders
                for order_id in &pending {
                    println!("  Cancelling order #{}", order_id);
                    thread::sleep(Duration::from_millis(50));
                }

                let _ = results.send(WorkerResult::ShutdownComplete {
                    pending_orders: pending.len(),
                });
                break;
            }

            Err(_) => {
                // Channel closed
                println!("Command channel closed, terminating");
                break;
            }
        }
    }
}

fn main() {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    // Start the worker
    let worker = thread::spawn(move || {
        order_worker(cmd_rx, result_tx);
    });

    // Send some orders
    for i in 1..=5 {
        cmd_tx.send(WorkerCommand::ProcessOrder {
            id: i,
            symbol: "BTC".to_string(),
            quantity: 0.1 * i as f64,
        }).unwrap();
    }

    // Receive results
    for _ in 0..5 {
        if let Ok(WorkerResult::OrderProcessed { id, success }) = result_rx.recv() {
            println!("Order #{} processed: {}", id, if success { "success" } else { "error" });
        }
    }

    // Initiate graceful shutdown
    println!("\n=== Sending shutdown command ===\n");
    cmd_tx.send(WorkerCommand::Shutdown).unwrap();

    // Wait for shutdown confirmation
    if let Ok(WorkerResult::ShutdownComplete { pending_orders }) = result_rx.recv() {
        println!("Shutdown complete, {} orders cancelled", pending_orders);
    }

    worker.join().unwrap();
}
```

## Practical Example: Trading Bot with Graceful Shutdown

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Order status
#[derive(Debug, Clone)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

/// Order
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    quantity: f64,
    price: f64,
    status: OrderStatus,
}

/// Position
#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

/// Trading bot state
struct TradingBot {
    running: AtomicBool,
    shutdown_requested: AtomicBool,
    next_order_id: AtomicU64,
    orders: Mutex<HashMap<u64, Order>>,
    positions: Mutex<HashMap<String, Position>>,
    cash_balance: Mutex<f64>,
}

impl TradingBot {
    fn new(initial_cash: f64) -> Arc<Self> {
        Arc::new(TradingBot {
            running: AtomicBool::new(true),
            shutdown_requested: AtomicBool::new(false),
            next_order_id: AtomicU64::new(1),
            orders: Mutex::new(HashMap::new()),
            positions: Mutex::new(HashMap::new()),
            cash_balance: Mutex::new(initial_cash),
        })
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    fn request_shutdown(&self) {
        println!("[Bot] Graceful shutdown requested");
        self.shutdown_requested.store(true, Ordering::SeqCst);
    }

    fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Create a new order
    fn place_order(&self, symbol: &str, quantity: f64, price: f64) -> Option<u64> {
        if self.is_shutdown_requested() {
            println!("[Bot] Rejecting new order — shutdown in progress");
            return None;
        }

        let id = self.next_order_id.fetch_add(1, Ordering::SeqCst);
        let order = Order {
            id,
            symbol: symbol.to_string(),
            quantity,
            price,
            status: OrderStatus::Pending,
        };

        let mut orders = self.orders.lock().unwrap();
        orders.insert(id, order);
        println!("[Bot] Order #{} created: {} x {} @ ${:.2}", id, symbol, quantity, price);

        Some(id)
    }

    /// Cancel all active orders
    fn cancel_all_orders(&self) -> Vec<u64> {
        let mut orders = self.orders.lock().unwrap();
        let mut cancelled = Vec::new();

        for (id, order) in orders.iter_mut() {
            if matches!(order.status, OrderStatus::Pending) {
                order.status = OrderStatus::Cancelled;
                cancelled.push(*id);
                println!("[Bot] Order #{} cancelled", id);
            }
        }

        cancelled
    }

    /// Close all positions at market price
    fn close_all_positions(&self) {
        let mut positions = self.positions.lock().unwrap();
        let mut cash = self.cash_balance.lock().unwrap();

        for (symbol, position) in positions.drain() {
            // Simulate closing position
            let market_price = position.avg_price * 1.001; // Small slippage
            let revenue = position.quantity * market_price;
            *cash += revenue;
            println!(
                "[Bot] Position {} closed: {} x ${:.2} = ${:.2}",
                symbol, position.quantity, market_price, revenue
            );
        }
    }

    /// Get count of active orders
    fn pending_orders_count(&self) -> usize {
        let orders = self.orders.lock().unwrap();
        orders
            .values()
            .filter(|o| matches!(o.status, OrderStatus::Pending))
            .count()
    }

    /// Save state (simulation)
    fn save_state(&self) {
        let orders = self.orders.lock().unwrap();
        let positions = self.positions.lock().unwrap();
        let cash = self.cash_balance.lock().unwrap();

        println!("[Bot] Saving state:");
        println!("  - Orders: {}", orders.len());
        println!("  - Positions: {}", positions.len());
        println!("  - Balance: ${:.2}", *cash);
        // In reality, this would write to a database
    }

    /// Execute graceful shutdown
    fn graceful_shutdown(&self) {
        println!("\n=== Starting graceful shutdown ===\n");

        // Step 1: Stop accepting new orders (already done via shutdown_requested)
        println!("[Shutdown] Step 1: Stopped accepting new orders");

        // Step 2: Cancel all active orders
        println!("[Shutdown] Step 2: Cancelling active orders...");
        let cancelled = self.cancel_all_orders();
        println!("[Shutdown] Cancelled {} orders", cancelled.len());

        // Simulate waiting for cancellation confirmation from exchange
        thread::sleep(Duration::from_millis(500));

        // Step 3: Close all positions
        println!("[Shutdown] Step 3: Closing positions...");
        self.close_all_positions();

        // Step 4: Save state
        println!("[Shutdown] Step 4: Saving state...");
        self.save_state();

        // Step 5: Stop the bot
        println!("[Shutdown] Step 5: Stopping all components...");
        self.stop();

        println!("\n=== Graceful shutdown complete ===\n");
    }
}

/// Trading strategy simulation
fn trading_strategy(bot: Arc<TradingBot>) {
    println!("[Strategy] Strategy started");

    let mut tick = 0;
    while bot.is_running() {
        if !bot.is_shutdown_requested() {
            tick += 1;
            // Simulate trading decisions
            if tick % 3 == 0 {
                bot.place_order("BTC", 0.1, 42000.0 + tick as f64 * 10.0);
            }
        }
        thread::sleep(Duration::from_millis(300));
    }

    println!("[Strategy] Strategy stopped");
}

/// Market data handler simulation
fn market_data_handler(bot: Arc<TradingBot>) {
    println!("[MarketData] Data handler started");

    while bot.is_running() {
        if !bot.is_shutdown_requested() {
            // Simulate receiving quotes
            println!("[MarketData] BTC: ${:.2}", 42000.0);
        }
        thread::sleep(Duration::from_millis(500));
    }

    println!("[MarketData] Data handler stopped");
}

fn main() {
    let bot = TradingBot::new(100_000.0);

    // Add initial position for demonstration
    {
        let mut positions = bot.positions.lock().unwrap();
        positions.insert("ETH".to_string(), Position {
            symbol: "ETH".to_string(),
            quantity: 10.0,
            avg_price: 2500.0,
        });
        let mut cash = bot.cash_balance.lock().unwrap();
        *cash -= 25_000.0; // Reduce by position cost
    }

    // Start components
    let bot_clone1 = Arc::clone(&bot);
    let strategy = thread::spawn(move || trading_strategy(bot_clone1));

    let bot_clone2 = Arc::clone(&bot);
    let market_data = thread::spawn(move || market_data_handler(bot_clone2));

    // Simulate work
    thread::sleep(Duration::from_secs(2));

    // Initiate graceful shutdown
    bot.request_shutdown();
    bot.graceful_shutdown();

    // Wait for all threads to complete
    strategy.join().unwrap();
    market_data.join().unwrap();

    println!("All bot components successfully stopped!");
}
```

## Timeout for Graceful Shutdown

It's important to limit the time for graceful shutdown to prevent the system from hanging:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct ShutdownCoordinator {
    shutdown_requested: Arc<AtomicBool>,
    timeout: Duration,
}

impl ShutdownCoordinator {
    fn new(timeout: Duration) -> Self {
        ShutdownCoordinator {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            timeout,
        }
    }

    fn token(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown_requested)
    }

    fn initiate_shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
    }

    fn wait_for_workers(&self, handles: Vec<thread::JoinHandle<()>>) -> bool {
        let start = Instant::now();

        for handle in handles {
            let remaining = self.timeout.saturating_sub(start.elapsed());

            if remaining.is_zero() {
                println!("WARNING: Graceful shutdown timeout expired!");
                println!("Some threads will be forcefully terminated.");
                return false;
            }

            // In practice, we need join with timeout, which std doesn't support directly
            // We use a simple approach: the thread should check the token and terminate itself
            match handle.join() {
                Ok(_) => println!("Thread successfully terminated"),
                Err(_) => println!("Error terminating thread"),
            }
        }

        true
    }
}

fn worker(id: usize, shutdown: Arc<AtomicBool>) {
    println!("Worker {} started", id);

    while !shutdown.load(Ordering::SeqCst) {
        // Simulate work
        thread::sleep(Duration::from_millis(100));
    }

    // Simulate cleanup (may take time)
    println!("Worker {}: performing cleanup...", id);
    thread::sleep(Duration::from_millis(300));

    println!("Worker {} stopped", id);
}

fn main() {
    let coordinator = ShutdownCoordinator::new(Duration::from_secs(5));

    // Start workers
    let mut handles = Vec::new();
    for i in 0..3 {
        let token = coordinator.token();
        handles.push(thread::spawn(move || worker(i, token)));
    }

    // Work
    thread::sleep(Duration::from_secs(1));

    // Start shutdown
    println!("\n=== Starting graceful shutdown with 5 second timeout ===\n");
    coordinator.initiate_shutdown();

    if coordinator.wait_for_workers(handles) {
        println!("\nAll workers terminated within timeout");
    } else {
        println!("\nSome workers did not terminate in time");
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Graceful Shutdown | Controlled termination with proper resource cleanup |
| Cancellation Token | Atomic flag for coordinating cancellation between threads |
| SIGINT/SIGTERM | Unix signals for requesting program termination |
| Channels for shutdown | Using `mpsc` to coordinate termination |
| Shutdown timeout | Limiting time for graceful shutdown |
| Shutdown phases | Stop accepting → cancel operations → save → stop |

## Homework

1. **Simple shutdown**: Create a program with three threads that:
   - Properly handles Ctrl+C
   - Sends a stop signal to all threads
   - Waits for each thread to complete
   - Prints statistics for each thread's work

2. **Exchange gateway**: Implement an `ExchangeGateway` struct with methods:
   - `connect()` — connect to the exchange
   - `submit_order(order)` — submit an order
   - `shutdown()` — graceful shutdown that:
     - Cancels all active orders
     - Waits for cancellation confirmation (with timeout)
     - Closes the connection

3. **Shutdown coordinator**: Create a `ShutdownCoordinator` that:
   - Registers multiple components
   - On shutdown, calls them in a specific order (reverse registration order)
   - Logs the completion time for each component
   - Has an overall timeout for the entire process

4. **Checkpoint system**: Implement a system that:
   - Periodically saves the trading bot's state
   - Saves a final checkpoint on shutdown
   - Restores state from the last checkpoint on startup

## Navigation

[← Previous day](../175-thread-pool-limiting-parallelism/en.md) | [Next day →](../177-pattern-producer-consumer/en.md)
