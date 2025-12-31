# Day 338: Graceful Shutdown

## Trading Analogy

Imagine you're managing a trading firm and it's the end of the workday. You have two ways to close the office:

**Hard Shutdown:**
You simply turn off the lights and leave. Traders abandon unfinished orders, open positions are left unattended, data is not saved. The next day you'll arrive to chaos: lost trades, unsynchronized positions, potential losses.

**Graceful Shutdown:**
You announce that the office is closing in 15 minutes. Traders:
1. Stop accepting new orders
2. Complete processing of current orders
3. Close or hedge open positions
4. Save all trade data
5. Disconnect from exchanges
6. Shut down systems in the correct order

| Criterion | Hard Shutdown | Graceful Shutdown |
|-----------|---------------|-------------------|
| **Analogy** | Pull the plug | Announce closing |
| **Data** | May be lost | Saved |
| **Orders** | Abandoned | Completed/cancelled |
| **Connections** | Broken | Closed properly |
| **State** | Unknown | Consistent |

## Why This Matters for Trading Systems

Trading systems are particularly sensitive to improper shutdown:

1. **Open positions** — unclosed positions can lead to losses
2. **Unfinished orders** — may execute or hang indefinitely
3. **Data loss** — trade history is critically important
4. **Connection state** — exchanges may block accounts after abrupt disconnection
5. **Database integrity** — incomplete transactions

## Basic Graceful Shutdown Pattern

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::thread;

/// Flag to signal shutdown
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Trading worker
struct TradingWorker {
    name: String,
    processing: AtomicBool,
}

impl TradingWorker {
    fn new(name: &str) -> Self {
        TradingWorker {
            name: name.to_string(),
            processing: AtomicBool::new(false),
        }
    }

    /// Process orders with shutdown flag checking
    fn process_orders(&self) {
        println!("[{}] Starting order processing", self.name);

        for i in 1..=10 {
            // Check shutdown flag before each operation
            if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                println!("[{}] Shutdown signal received, stopping new order intake", self.name);
                break;
            }

            self.processing.store(true, Ordering::SeqCst);
            println!("[{}] Processing order #{}", self.name, i);

            // Simulate order processing
            thread::sleep(Duration::from_millis(100));

            self.processing.store(false, Ordering::SeqCst);
        }

        println!("[{}] Processing completed", self.name);
    }

    /// Check if there's active work
    fn is_processing(&self) -> bool {
        self.processing.load(Ordering::SeqCst)
    }
}

fn main() {
    println!("=== Basic Graceful Shutdown ===\n");

    let worker = Arc::new(TradingWorker::new("OrderProcessor"));
    let worker_clone = Arc::clone(&worker);

    // Start worker in a separate thread
    let handle = thread::spawn(move || {
        worker_clone.process_orders();
    });

    // Simulate system operation
    thread::sleep(Duration::from_millis(350));

    // Initiate graceful shutdown
    println!("\n>>> Initiating graceful shutdown...\n");
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);

    // Wait for worker to finish
    handle.join().unwrap();

    println!("\n>>> System shut down gracefully");
}
```

## Handling Unix Signals (SIGTERM, SIGINT)

In production, it's important to handle system signals:

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};

/// Graceful shutdown manager
struct ShutdownManager {
    shutdown_tx: broadcast::Sender<()>,
    is_shutting_down: AtomicBool,
}

impl ShutdownManager {
    fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        ShutdownManager {
            shutdown_tx,
            is_shutting_down: AtomicBool::new(false),
        }
    }

    /// Get subscription to shutdown event
    fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Initiate shutdown
    fn initiate(&self) {
        if !self.is_shutting_down.swap(true, Ordering::SeqCst) {
            println!("[ShutdownManager] Initiating graceful shutdown...");
            let _ = self.shutdown_tx.send(());
        }
    }

    fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::SeqCst)
    }
}

/// Trading service
struct TradingService {
    name: String,
    shutdown_rx: broadcast::Receiver<()>,
}

impl TradingService {
    fn new(name: &str, shutdown_manager: &ShutdownManager) -> Self {
        TradingService {
            name: name.to_string(),
            shutdown_rx: shutdown_manager.subscribe(),
        }
    }

    async fn run(&mut self) {
        println!("[{}] Service started", self.name);

        loop {
            tokio::select! {
                // Main work
                _ = self.do_work() => {
                    // Work done, continue
                }
                // Shutdown signal received
                _ = self.shutdown_rx.recv() => {
                    println!("[{}] Shutdown signal received", self.name);
                    break;
                }
            }
        }

        // Perform cleanup
        self.cleanup().await;
        println!("[{}] Service terminated", self.name);
    }

    async fn do_work(&self) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        // Simulate market data processing
    }

    async fn cleanup(&self) {
        println!("[{}] Performing cleanup...", self.name);
        // Close connections, save data, etc.
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::main]
async fn main() {
    println!("=== Unix Signal Handling ===\n");

    let shutdown_manager = Arc::new(ShutdownManager::new());

    // Start signal handler
    let shutdown_for_signals = Arc::clone(&shutdown_manager);
    tokio::spawn(async move {
        // Wait for SIGTERM or SIGINT (Ctrl+C)
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("\n>>> Received SIGINT (Ctrl+C)");
            }
            // For Unix systems, you can add SIGTERM:
            // _ = async {
            //     let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
            //     sigterm.recv().await
            // } => {
            //     println!("\n>>> Received SIGTERM");
            // }
        }
        shutdown_for_signals.initiate();
    });

    // Create trading services
    let mut market_data = TradingService::new("MarketData", &shutdown_manager);
    let mut order_executor = TradingService::new("OrderExecutor", &shutdown_manager);

    // Start services
    let md_handle = tokio::spawn(async move {
        market_data.run().await;
    });

    let oe_handle = tokio::spawn(async move {
        order_executor.run().await;
    });

    // Simulate system operation (in reality this would be a long process)
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Initiate shutdown programmatically (for demonstration)
    shutdown_manager.initiate();

    // Wait for all services to finish with timeout
    let shutdown_timeout = Duration::from_secs(10);
    match timeout(shutdown_timeout, async {
        let _ = md_handle.await;
        let _ = oe_handle.await;
    }).await {
        Ok(_) => println!("\n>>> All services terminated gracefully"),
        Err(_) => println!("\n>>> Timeout! Forcing shutdown"),
    }
}
```

## Multi-Phase Shutdown for Trading Systems

```rust
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, broadcast};
use tokio::time::{Duration, timeout};
use std::collections::HashMap;

/// Shutdown phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShutdownPhase {
    Running,           // System is running
    StopNewOrders,     // Stop accepting new orders
    CancelPending,     // Cancel pending orders
    ClosePositions,    // Close open positions
    SaveState,         // Save state
    Disconnect,        // Disconnect from exchanges
    Complete,          // Completion
}

/// Order in the system
#[derive(Debug, Clone)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
}

/// Position
#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

/// Trading engine with graceful shutdown
struct TradingEngine {
    phase: Arc<RwLock<ShutdownPhase>>,
    pending_orders: Arc<RwLock<Vec<Order>>>,
    open_positions: Arc<RwLock<HashMap<String, Position>>>,
    shutdown_tx: broadcast::Sender<ShutdownPhase>,
}

impl TradingEngine {
    fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);

        // Create test data
        let mut positions = HashMap::new();
        positions.insert("BTCUSDT".to_string(), Position {
            symbol: "BTCUSDT".to_string(),
            quantity: 0.5,
            entry_price: 50000.0,
        });
        positions.insert("ETHUSDT".to_string(), Position {
            symbol: "ETHUSDT".to_string(),
            quantity: 5.0,
            entry_price: 3000.0,
        });

        let pending_orders = vec![
            Order {
                id: "ORD-001".to_string(),
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                quantity: 0.1,
                price: 49000.0,
                status: "PENDING".to_string(),
            },
            Order {
                id: "ORD-002".to_string(),
                symbol: "ETHUSDT".to_string(),
                side: "SELL".to_string(),
                quantity: 1.0,
                price: 3100.0,
                status: "PENDING".to_string(),
            },
        ];

        TradingEngine {
            phase: Arc::new(RwLock::new(ShutdownPhase::Running)),
            pending_orders: Arc::new(RwLock::new(pending_orders)),
            open_positions: Arc::new(RwLock::new(positions)),
            shutdown_tx,
        }
    }

    fn subscribe(&self) -> broadcast::Receiver<ShutdownPhase> {
        self.shutdown_tx.subscribe()
    }

    async fn set_phase(&self, phase: ShutdownPhase) {
        *self.phase.write().await = phase;
        let _ = self.shutdown_tx.send(phase);
        println!("[TradingEngine] Transitioning to phase: {:?}", phase);
    }

    async fn current_phase(&self) -> ShutdownPhase {
        *self.phase.read().await
    }

    /// Attempt to place an order
    async fn place_order(&self, order: Order) -> Result<(), String> {
        let phase = self.current_phase().await;

        if phase != ShutdownPhase::Running {
            return Err(format!(
                "New orders not accepted, system in phase {:?}",
                phase
            ));
        }

        self.pending_orders.write().await.push(order.clone());
        println!("[TradingEngine] Order accepted: {}", order.id);
        Ok(())
    }

    /// Cancel all pending orders
    async fn cancel_all_pending(&self) -> Vec<String> {
        let mut orders = self.pending_orders.write().await;
        let cancelled: Vec<String> = orders.iter().map(|o| o.id.clone()).collect();

        for order in orders.iter_mut() {
            order.status = "CANCELLED".to_string();
            println!("[TradingEngine] Order cancelled: {}", order.id);
        }

        orders.clear();
        cancelled
    }

    /// Close all positions
    async fn close_all_positions(&self) -> Vec<(String, f64)> {
        let mut positions = self.open_positions.write().await;
        let mut closed = Vec::new();

        for (symbol, position) in positions.iter() {
            // Simulate closing at market price
            let market_price = position.entry_price * 1.01; // Approximate price
            let pnl = (market_price - position.entry_price) * position.quantity;

            println!(
                "[TradingEngine] Closing position {}: {} @ ${:.2}, P&L: ${:.2}",
                symbol, position.quantity, market_price, pnl
            );

            closed.push((symbol.clone(), pnl));
        }

        positions.clear();
        closed
    }

    /// Save state
    async fn save_state(&self) {
        println!("[TradingEngine] Saving state...");

        // Simulate saving to file/DB
        tokio::time::sleep(Duration::from_millis(100)).await;

        let orders = self.pending_orders.read().await;
        let positions = self.open_positions.read().await;

        println!(
            "[TradingEngine] State saved: {} orders, {} positions",
            orders.len(),
            positions.len()
        );
    }

    /// Full graceful shutdown
    async fn graceful_shutdown(&self, shutdown_timeout: Duration) -> Result<(), String> {
        println!("\n=== Starting Graceful Shutdown ===\n");

        // Phase 1: Stop accepting new orders
        self.set_phase(ShutdownPhase::StopNewOrders).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Phase 2: Cancel pending orders
        self.set_phase(ShutdownPhase::CancelPending).await;
        let cancelled = self.cancel_all_pending().await;
        println!("Orders cancelled: {}", cancelled.len());

        // Phase 3: Close positions
        self.set_phase(ShutdownPhase::ClosePositions).await;
        let closed = self.close_all_positions().await;
        let total_pnl: f64 = closed.iter().map(|(_, pnl)| pnl).sum();
        println!("Positions closed: {}, Total P&L: ${:.2}", closed.len(), total_pnl);

        // Phase 4: Save state
        self.set_phase(ShutdownPhase::SaveState).await;
        self.save_state().await;

        // Phase 5: Disconnect
        self.set_phase(ShutdownPhase::Disconnect).await;
        println!("[TradingEngine] Disconnecting from exchanges...");
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Completion
        self.set_phase(ShutdownPhase::Complete).await;

        println!("\n=== Graceful Shutdown Complete ===");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    println!("=== Multi-Phase Trading System Shutdown ===\n");

    let engine = Arc::new(TradingEngine::new());
    let engine_clone = Arc::clone(&engine);

    // Subscribe to phase changes
    let mut phase_rx = engine.subscribe();
    tokio::spawn(async move {
        while let Ok(phase) = phase_rx.recv().await {
            println!("[Monitor] Current phase: {:?}", phase);
        }
    });

    // Attempt to place orders
    println!("--- Attempting to place orders ---");

    let order = Order {
        id: "ORD-003".to_string(),
        symbol: "SOLUSDT".to_string(),
        side: "BUY".to_string(),
        quantity: 10.0,
        price: 100.0,
        status: "NEW".to_string(),
    };

    match engine.place_order(order).await {
        Ok(_) => println!("Order placed successfully"),
        Err(e) => println!("Error: {}", e),
    }

    // Initiate shutdown
    tokio::time::sleep(Duration::from_millis(200)).await;

    let shutdown_result = engine_clone
        .graceful_shutdown(Duration::from_secs(30))
        .await;

    match shutdown_result {
        Ok(_) => println!("\nSystem shut down gracefully"),
        Err(e) => println!("\nShutdown error: {}", e),
    }

    // Attempt to place order after shutdown
    println!("\n--- Attempting to place order after shutdown ---");
    let order = Order {
        id: "ORD-004".to_string(),
        symbol: "SOLUSDT".to_string(),
        side: "BUY".to_string(),
        quantity: 10.0,
        price: 100.0,
        status: "NEW".to_string(),
    };

    match engine.place_order(order).await {
        Ok(_) => println!("Order placed successfully"),
        Err(e) => println!("Expected error: {}", e),
    }
}
```

## Tokio CancellationToken

Tokio provides a convenient primitive for graceful shutdown:

```rust
use tokio_util::sync::CancellationToken;
use tokio::time::{Duration, interval};
use std::sync::Arc;

/// Market data service
async fn market_data_service(token: CancellationToken, symbol: String) {
    println!("[MarketData:{}] Started", symbol);

    let mut price = 50000.0;
    let mut tick_interval = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("[MarketData:{}] Cancellation signal received, shutting down...", symbol);
                break;
            }
            _ = tick_interval.tick() => {
                // Simulate price update
                price += (rand_simple() - 0.5) * 100.0;
                // Here would be update processing
            }
        }
    }

    // Cleanup
    println!("[MarketData:{}] Cleanup completed", symbol);
}

/// Order executor service
async fn order_executor_service(token: CancellationToken) {
    println!("[OrderExecutor] Started");

    let mut order_count = 0;

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("[OrderExecutor] Cancellation signal received");
                println!("[OrderExecutor] Finishing processing of {} orders...", order_count);
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                order_count += 1;
                // Simulate order processing
            }
        }
    }

    // Finalization
    println!("[OrderExecutor] All orders processed");
}

/// Pseudo-random for demonstration
fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

#[tokio::main]
async fn main() {
    println!("=== Tokio CancellationToken ===\n");

    // Create parent token
    let parent_token = CancellationToken::new();

    // Create child tokens for different services
    let md_btc_token = parent_token.child_token();
    let md_eth_token = parent_token.child_token();
    let executor_token = parent_token.child_token();

    // Start services
    let md_btc = tokio::spawn(market_data_service(md_btc_token, "BTCUSDT".to_string()));
    let md_eth = tokio::spawn(market_data_service(md_eth_token, "ETHUSDT".to_string()));
    let executor = tokio::spawn(order_executor_service(executor_token));

    // Work for some time
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Initiate shutdown
    println!("\n>>> Initiating shutdown via CancellationToken\n");
    parent_token.cancel();

    // Wait for all services to complete
    let _ = tokio::join!(md_btc, md_eth, executor);

    println!("\n>>> All services terminated");
}
```

## Shutdown with Timeouts and Forced Termination

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{Duration, timeout};

/// Task that may hang
async fn potentially_slow_task(id: u32, slow: bool) -> Result<u32, String> {
    println!("[Task-{}] Starting execution", id);

    if slow {
        // Simulate hanging task
        tokio::time::sleep(Duration::from_secs(60)).await;
    } else {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("[Task-{}] Completed", id);
    Ok(id)
}

/// Task manager with graceful shutdown
struct TaskManager {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl TaskManager {
    fn new(max_concurrent: usize) -> Self {
        TaskManager {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    async fn shutdown_with_timeout(
        &self,
        tasks: Vec<tokio::task::JoinHandle<Result<u32, String>>>,
        graceful_timeout: Duration,
        force_timeout: Duration,
    ) {
        println!("\n=== Starting shutdown ===");
        println!("Graceful timeout: {:?}", graceful_timeout);
        println!("Force timeout: {:?}", force_timeout);

        // Phase 1: Graceful shutdown
        println!("\n[Phase 1] Waiting for tasks to complete...");

        match timeout(graceful_timeout, async {
            for (i, task) in tasks.into_iter().enumerate() {
                match task.await {
                    Ok(Ok(id)) => println!("  Task {} completed successfully", id),
                    Ok(Err(e)) => println!("  Task {} completed with error: {}", i, e),
                    Err(_) => println!("  Task {} was cancelled", i),
                }
            }
        }).await {
            Ok(_) => {
                println!("\n[Result] All tasks completed gracefully");
                return;
            }
            Err(_) => {
                println!("\n[Phase 1] Timeout! Proceeding to forced shutdown");
            }
        }

        // Phase 2: Force shutdown
        println!("\n[Phase 2] Forcing shutdown...");

        // In reality, there would be additional actions here:
        // - Cancel tasks
        // - Save state
        // - Log incomplete operations

        tokio::time::sleep(force_timeout).await;
        println!("[Phase 2] Forced shutdown complete");
    }
}

#[tokio::main]
async fn main() {
    println!("=== Shutdown with Timeouts ===\n");

    let manager = TaskManager::new(5);

    // Create tasks (one will "hang")
    let mut tasks = Vec::new();

    for i in 0..5 {
        let slow = i == 2; // Task 2 will be slow
        let handle = tokio::spawn(potentially_slow_task(i, slow));
        tasks.push(handle);
    }

    // Give time to start execution
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Start shutdown
    manager.shutdown_with_timeout(
        tasks,
        Duration::from_millis(500),  // Graceful timeout
        Duration::from_millis(100),  // Force timeout
    ).await;

    println!("\n>>> Shutdown complete");
}
```

## Pattern: Shutdown Hooks for Trading Systems

```rust
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::collections::VecDeque;

/// Hook function type
type ShutdownHook = Box<dyn Fn() + Send + Sync>;

/// Hook priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HookPriority {
    Critical = 0,   // Execute first (save data)
    High = 1,       // Close positions
    Normal = 2,     // Cancel orders
    Low = 3,        // Disconnect from exchanges
    Cleanup = 4,    // Resource cleanup
}

/// Registered hook
struct RegisteredHook {
    name: String,
    priority: HookPriority,
    hook: ShutdownHook,
}

/// Shutdown hook manager
struct ShutdownHookManager {
    hooks: RwLock<Vec<RegisteredHook>>,
}

impl ShutdownHookManager {
    fn new() -> Self {
        ShutdownHookManager {
            hooks: RwLock::new(Vec::new()),
        }
    }

    /// Register a hook
    async fn register<F>(&self, name: &str, priority: HookPriority, hook: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let registered = RegisteredHook {
            name: name.to_string(),
            priority,
            hook: Box::new(hook),
        };

        let mut hooks = self.hooks.write().await;
        hooks.push(registered);

        // Sort by priority
        hooks.sort_by(|a, b| a.priority.cmp(&b.priority));

        println!("[HookManager] Registered hook '{}' with priority {:?}", name, priority);
    }

    /// Execute all hooks
    async fn run_all(&self) {
        let hooks = self.hooks.read().await;

        println!("\n[HookManager] Executing {} hooks...\n", hooks.len());

        for hook in hooks.iter() {
            println!("[HookManager] Executing '{}' ({:?})...", hook.name, hook.priority);
            (hook.hook)();
        }

        println!("\n[HookManager] All hooks executed");
    }
}

#[tokio::main]
async fn main() {
    println!("=== Shutdown Hooks for Trading System ===\n");

    let hook_manager = Arc::new(ShutdownHookManager::new());

    // Register hooks in different order
    hook_manager.register(
        "disconnect_exchanges",
        HookPriority::Low,
        || println!("  -> Disconnecting from exchanges")
    ).await;

    hook_manager.register(
        "save_state",
        HookPriority::Critical,
        || println!("  -> Saving critical state")
    ).await;

    hook_manager.register(
        "close_positions",
        HookPriority::High,
        || println!("  -> Closing open positions")
    ).await;

    hook_manager.register(
        "cancel_orders",
        HookPriority::Normal,
        || println!("  -> Cancelling pending orders")
    ).await;

    hook_manager.register(
        "cleanup_temp_files",
        HookPriority::Cleanup,
        || println!("  -> Cleaning up temporary files")
    ).await;

    // Simulate operation
    println!("System running...");
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Execute shutdown
    println!("\n>>> Initiating shutdown");
    hook_manager.run_all().await;

    println!("\n>>> System terminated");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Graceful Shutdown** | Proper termination with state preservation |
| **Hard Shutdown** | Immediate termination without cleanup |
| **Shutdown phases** | Sequential termination stages |
| **CancellationToken** | Tokio primitive for task cancellation |
| **Shutdown hooks** | Functions called during shutdown |
| **Shutdown timeout** | Time limit for graceful shutdown |
| **Signal handling** | Processing SIGTERM, SIGINT |

## Practical Exercises

1. **Graceful Shutdown for WebSocket Client**: Create a system that:
   - Connects to multiple exchanges via WebSocket
   - Properly closes all connections during shutdown
   - Saves the last received data
   - Logs all disconnection stages

2. **Order Manager with Shutdown**: Implement an order manager:
   - Tracks all active orders
   - Cancels pending orders during shutdown
   - Waits for cancellation confirmation with timeout
   - Logs uncancelled orders

3. **Position Closer**: Create a service:
   - Tracks open positions
   - Closes positions at market during shutdown
   - Calculates final P&L
   - Generates closing report

4. **State Persister**: Implement a persistence system:
   - Periodically saves state
   - Makes final save during shutdown
   - Verifies data integrity
   - Supports recovery after crash

## Homework

1. **Complete Trading System with Graceful Shutdown**: Develop a system:
   - Market data service with subscriptions
   - Order execution with order queue
   - Position tracking with P&L calculation
   - Risk manager with limits
   - Multi-phase shutdown with all stages
   - Handling of all system signals
   - Timeouts for each phase
   - Logging of all operations

2. **Fault-tolerant Shutdown**: Implement shutdown that:
   - Continues operation when individual components fail
   - Has fallback for critical operations
   - Saves error information
   - Allows partial recovery
   - Sends alerts on problems

3. **Distributed Shutdown Coordinator**: Create a coordinator:
   - Manages shutdown of multiple services
   - Considers dependencies between services
   - Coordinates termination order
   - Handles unavailable services
   - Generates overall report

4. **Shutdown Testing Framework**: Develop a framework:
   - Simulates various shutdown scenarios
   - Tests all execution paths
   - Verifies data preservation
   - Measures time for each phase
   - Generates coverage reports

## Navigation

[← Previous day](../337-*/en.md) | [Next day →](../339-*/en.md)
