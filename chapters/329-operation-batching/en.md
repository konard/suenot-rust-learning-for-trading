# Day 329: Operation Batching

## Trading Analogy

Imagine you're running a trading bot that monitors 100 cryptocurrency pairs. Every second, the bot receives price updates and must:
- Calculate the new portfolio value
- Update trailing stop-loss for each position
- Check entry/exit conditions
- Write data to logs

Without batching, each price update triggers 4 operations. With 100 pairs and 10 updates per second, that's 4000 operations per second!

**Operation batching** is like an exchange's matching system:
- Instead of instantly executing each order, the exchange accumulates orders
- At regular intervals (e.g., every 100ms), "matching" occurs
- All accumulated orders are processed in a single batch

The result:
- Reduced system load
- Fewer lock contentions
- Optimized network and disk usage

## When to Use Batching?

| Scenario | Trading Example | Benefit |
|----------|----------------|---------|
| **Many small writes** | Logging each tick | Reduced I/O operations |
| **Network requests** | Sending orders to exchange | Lower latency and overhead |
| **Database updates** | Saving trade history | Fewer transactions |
| **Multi-asset calculations** | Portfolio recalculation | Efficient CPU usage |

## Simple Batcher for Trading Operations

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Trading operation for batching
#[derive(Debug, Clone)]
enum TradeOperation {
    UpdatePrice { symbol: String, price: f64 },
    PlaceOrder { symbol: String, side: String, quantity: f64 },
    CancelOrder { order_id: u64 },
    UpdateStopLoss { symbol: String, new_price: f64 },
}

/// Operation batcher with configurable flush conditions
struct OperationBatcher {
    buffer: VecDeque<TradeOperation>,
    max_size: usize,
    max_wait: Duration,
    last_flush: Instant,
    total_batches: u64,
    total_operations: u64,
}

impl OperationBatcher {
    fn new(max_size: usize, max_wait_ms: u64) -> Self {
        OperationBatcher {
            buffer: VecDeque::new(),
            max_size,
            max_wait: Duration::from_millis(max_wait_ms),
            last_flush: Instant::now(),
            total_batches: 0,
            total_operations: 0,
        }
    }

    /// Add operation to buffer
    fn add(&mut self, op: TradeOperation) -> Option<Vec<TradeOperation>> {
        self.buffer.push_back(op);

        // Check flush conditions
        if self.should_flush() {
            Some(self.flush())
        } else {
            None
        }
    }

    /// Check if buffer should be flushed
    fn should_flush(&self) -> bool {
        self.buffer.len() >= self.max_size
            || self.last_flush.elapsed() >= self.max_wait
    }

    /// Flush buffer and return accumulated operations
    fn flush(&mut self) -> Vec<TradeOperation> {
        let ops: Vec<_> = self.buffer.drain(..).collect();
        self.last_flush = Instant::now();
        self.total_batches += 1;
        self.total_operations += ops.len() as u64;
        ops
    }

    /// Force flush (e.g., on shutdown)
    fn force_flush(&mut self) -> Vec<TradeOperation> {
        if !self.buffer.is_empty() {
            self.flush()
        } else {
            Vec::new()
        }
    }

    /// Statistics
    fn stats(&self) -> (u64, u64, f64) {
        let avg = if self.total_batches > 0 {
            self.total_operations as f64 / self.total_batches as f64
        } else {
            0.0
        };
        (self.total_batches, self.total_operations, avg)
    }
}

fn process_batch(batch: &[TradeOperation]) {
    println!("  Processing batch of {} operations:", batch.len());
    for op in batch {
        match op {
            TradeOperation::UpdatePrice { symbol, price } => {
                println!("    - Price {}: ${:.2}", symbol, price);
            }
            TradeOperation::PlaceOrder { symbol, side, quantity } => {
                println!("    - Order: {} {} {:.4}", side, quantity, symbol);
            }
            TradeOperation::CancelOrder { order_id } => {
                println!("    - Cancel order #{}", order_id);
            }
            TradeOperation::UpdateStopLoss { symbol, new_price } => {
                println!("    - Stop-loss {}: ${:.2}", symbol, new_price);
            }
        }
    }
}

fn main() {
    let mut batcher = OperationBatcher::new(5, 100); // max 5 ops or 100ms

    println!("=== Trading Operation Batching ===\n");

    // Simulate operation stream
    let operations = vec![
        TradeOperation::UpdatePrice { symbol: "BTC".to_string(), price: 42500.0 },
        TradeOperation::UpdatePrice { symbol: "ETH".to_string(), price: 2500.0 },
        TradeOperation::UpdateStopLoss { symbol: "BTC".to_string(), new_price: 42000.0 },
        TradeOperation::PlaceOrder { symbol: "SOL".to_string(), side: "BUY".to_string(), quantity: 10.0 },
        TradeOperation::UpdatePrice { symbol: "SOL".to_string(), price: 100.0 },
        TradeOperation::CancelOrder { order_id: 12345 },
        TradeOperation::UpdatePrice { symbol: "BTC".to_string(), price: 42550.0 },
    ];

    for op in operations {
        if let Some(batch) = batcher.add(op) {
            process_batch(&batch);
        }
    }

    // Flush remaining operations
    let remaining = batcher.force_flush();
    if !remaining.is_empty() {
        println!("\nRemaining operations:");
        process_batch(&remaining);
    }

    let (batches, ops, avg) = batcher.stats();
    println!("\n=== Statistics ===");
    println!("Total batches: {}", batches);
    println!("Total operations: {}", ops);
    println!("Average per batch: {:.1}", avg);
}
```

## Batching with Priorities

In trading, some operations are more important than others:

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::time::Instant;

/// Operation priority
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Priority {
    Critical = 3,  // Stop-loss, liquidation
    High = 2,      // Order placement
    Normal = 1,    // Data updates
    Low = 0,       // Logging
}

impl Priority {
    fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Operation with priority
#[derive(Debug, Clone)]
struct PrioritizedOp {
    priority: Priority,
    operation: String,
    created_at: Instant,
    sequence: u64,  // For stable sorting
}

impl Eq for PrioritizedOp {}

impl PartialEq for PrioritizedOp {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Ord for PrioritizedOp {
    fn cmp(&self, other: &Self) -> Ordering {
        // First by priority (higher = better)
        // On equal priority — by time (earlier = better)
        match self.priority.as_u8().cmp(&other.priority.as_u8()) {
            Ordering::Equal => other.sequence.cmp(&self.sequence),
            other => other,
        }
    }
}

impl PartialOrd for PrioritizedOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Batcher with priority support
struct PriorityBatcher {
    heap: BinaryHeap<PrioritizedOp>,
    max_size: usize,
    sequence: u64,
}

impl PriorityBatcher {
    fn new(max_size: usize) -> Self {
        PriorityBatcher {
            heap: BinaryHeap::new(),
            max_size,
            sequence: 0,
        }
    }

    fn add(&mut self, priority: Priority, operation: String) {
        self.sequence += 1;
        self.heap.push(PrioritizedOp {
            priority,
            operation,
            created_at: Instant::now(),
            sequence: self.sequence,
        });
    }

    fn flush(&mut self) -> Vec<PrioritizedOp> {
        let count = self.heap.len().min(self.max_size);
        let mut result = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(op) = self.heap.pop() {
                result.push(op);
            }
        }
        result
    }

    fn len(&self) -> usize {
        self.heap.len()
    }
}

fn main() {
    let mut batcher = PriorityBatcher::new(5);

    println!("=== Priority Batching ===\n");

    // Add operations in random order
    batcher.add(Priority::Normal, "Update BTC price".to_string());
    batcher.add(Priority::Low, "Write log".to_string());
    batcher.add(Priority::Critical, "Execute stop-loss!".to_string());
    batcher.add(Priority::High, "Place order".to_string());
    batcher.add(Priority::Normal, "Update ETH price".to_string());
    batcher.add(Priority::Critical, "Liquidation warning!".to_string());
    batcher.add(Priority::Low, "Send metrics".to_string());

    println!("Total operations in queue: {}\n", batcher.len());

    // Process batch (in priority order)
    let batch = batcher.flush();
    println!("Processing batch ({} operations):", batch.len());
    for (i, op) in batch.iter().enumerate() {
        println!("  {}. [{:?}] {}", i + 1, op.priority, op.operation);
    }

    println!("\nRemaining in queue: {}", batcher.len());
}
```

## Order Batching for Exchange

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

/// Result of batch order submission
#[derive(Debug)]
struct BatchResult {
    successful: Vec<u64>,
    failed: Vec<(u64, String)>,
    latency_ms: u64,
}

/// Order batcher with symbol grouping
struct OrderBatcher {
    orders: HashMap<String, Vec<Order>>,
    max_orders_per_symbol: usize,
    max_total_orders: usize,
    total_orders: usize,
    last_send: Instant,
    send_interval: Duration,
}

impl OrderBatcher {
    fn new(max_per_symbol: usize, max_total: usize, interval_ms: u64) -> Self {
        OrderBatcher {
            orders: HashMap::new(),
            max_orders_per_symbol: max_per_symbol,
            max_total_orders: max_total,
            total_orders: 0,
            last_send: Instant::now(),
            send_interval: Duration::from_millis(interval_ms),
        }
    }

    /// Add order to batch
    fn add_order(&mut self, order: Order) -> Option<HashMap<String, Vec<Order>>> {
        let symbol = order.symbol.clone();

        let orders = self.orders.entry(symbol.clone()).or_insert_with(Vec::new);
        orders.push(order);
        self.total_orders += 1;

        // Check send conditions
        if self.should_send(&symbol) {
            Some(self.flush())
        } else {
            None
        }
    }

    fn should_send(&self, symbol: &str) -> bool {
        let symbol_count = self.orders.get(symbol).map(|v| v.len()).unwrap_or(0);

        symbol_count >= self.max_orders_per_symbol
            || self.total_orders >= self.max_total_orders
            || self.last_send.elapsed() >= self.send_interval
    }

    fn flush(&mut self) -> HashMap<String, Vec<Order>> {
        let result = std::mem::take(&mut self.orders);
        self.total_orders = 0;
        self.last_send = Instant::now();
        result
    }

    /// Get buffer statistics
    fn buffer_stats(&self) -> (usize, usize) {
        (self.orders.len(), self.total_orders)
    }
}

/// Simulate sending batch to exchange
fn send_order_batch(orders: &HashMap<String, Vec<Order>>) -> BatchResult {
    let start = Instant::now();
    let mut successful = Vec::new();
    let mut failed = Vec::new();

    for (symbol, order_list) in orders {
        println!("  Sending {} orders for {}:", order_list.len(), symbol);
        for order in order_list {
            // Simulation: 95% success rate
            if order.id % 20 != 0 {
                successful.push(order.id);
                let price_str = order.price
                    .map(|p| format!(" @ ${:.2}", p))
                    .unwrap_or_else(|| " (market)".to_string());
                println!("    ✓ #{}: {:?} {:.4} {}{}",
                    order.id, order.side, order.quantity, symbol, price_str);
            } else {
                failed.push((order.id, "Insufficient balance".to_string()));
                println!("    ✗ #{}: Error - Insufficient balance", order.id);
            }
        }
    }

    BatchResult {
        successful,
        failed,
        latency_ms: start.elapsed().as_millis() as u64,
    }
}

fn main() {
    let mut batcher = OrderBatcher::new(3, 10, 500);

    println!("=== Order Batching ===\n");

    // Create order stream
    let orders = vec![
        Order { id: 1, symbol: "BTCUSDT".to_string(), side: OrderSide::Buy, quantity: 0.1, price: Some(42500.0) },
        Order { id: 2, symbol: "ETHUSDT".to_string(), side: OrderSide::Buy, quantity: 2.0, price: Some(2500.0) },
        Order { id: 3, symbol: "BTCUSDT".to_string(), side: OrderSide::Sell, quantity: 0.05, price: Some(43000.0) },
        Order { id: 4, symbol: "SOLUSDT".to_string(), side: OrderSide::Buy, quantity: 10.0, price: None },
        Order { id: 5, symbol: "BTCUSDT".to_string(), side: OrderSide::Buy, quantity: 0.2, price: Some(42000.0) },
        Order { id: 20, symbol: "ETHUSDT".to_string(), side: OrderSide::Sell, quantity: 1.0, price: Some(2600.0) }, // will fail
        Order { id: 7, symbol: "BTCUSDT".to_string(), side: OrderSide::Sell, quantity: 0.15, price: Some(43500.0) },
    ];

    for order in orders {
        println!("Added order #{} for {}", order.id, order.symbol);

        if let Some(batch) = batcher.add_order(order) {
            println!("\n--- Sending batch to exchange ---");
            let result = send_order_batch(&batch);
            println!("\nResult:");
            println!("  Successful: {} orders", result.successful.len());
            println!("  Failed: {} orders", result.failed.len());
            println!("  Latency: {}ms\n", result.latency_ms);
        }
    }

    // Check remaining orders
    let (symbols, total) = batcher.buffer_stats();
    if total > 0 {
        println!("\nRemaining in buffer: {} orders across {} symbols", total, symbols);
    }
}
```

## Database Write Batching

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

/// Write statistics
#[derive(Debug, Default)]
struct WriteStats {
    total_writes: u64,
    total_records: u64,
    total_time_ms: u64,
}

impl WriteStats {
    fn add(&mut self, records: u64, time_ms: u64) {
        self.total_writes += 1;
        self.total_records += records;
        self.total_time_ms += time_ms;
    }

    fn avg_batch_size(&self) -> f64 {
        if self.total_writes > 0 {
            self.total_records as f64 / self.total_writes as f64
        } else {
            0.0
        }
    }

    fn avg_write_time(&self) -> f64 {
        if self.total_writes > 0 {
            self.total_time_ms as f64 / self.total_writes as f64
        } else {
            0.0
        }
    }
}

/// Trade batcher for database writes
struct TradeBatcher {
    buffer: Vec<Trade>,
    max_size: usize,
    max_wait: Duration,
    last_flush: Instant,
    stats: WriteStats,
}

impl TradeBatcher {
    fn new(max_size: usize, max_wait_ms: u64) -> Self {
        TradeBatcher {
            buffer: Vec::with_capacity(max_size),
            max_size,
            max_wait: Duration::from_millis(max_wait_ms),
            last_flush: Instant::now(),
            stats: WriteStats::default(),
        }
    }

    fn add(&mut self, trade: Trade) {
        self.buffer.push(trade);

        if self.should_flush() {
            self.flush();
        }
    }

    fn should_flush(&self) -> bool {
        self.buffer.len() >= self.max_size
            || (!self.buffer.is_empty() && self.last_flush.elapsed() >= self.max_wait)
    }

    fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        let start = Instant::now();
        let count = self.buffer.len();

        // Simulate batch INSERT into database
        self.simulate_db_write();

        let elapsed = start.elapsed().as_millis() as u64;
        self.stats.add(count as u64, elapsed);

        println!(
            "[DB] Wrote {} trades in {}ms (total: {})",
            count, elapsed, self.stats.total_records
        );

        self.buffer.clear();
        self.last_flush = Instant::now();
    }

    fn simulate_db_write(&self) {
        // In reality, this would be a batch INSERT
        // INSERT INTO trades (id, symbol, price, quantity, timestamp) VALUES
        //   ($1, $2, $3, $4, $5), ($6, $7, $8, $9, $10), ...

        // Simulate delay: 5ms base + 0.1ms per record
        let delay_ms = 5 + (self.buffer.len() as u64 / 10);
        std::thread::sleep(Duration::from_millis(delay_ms));
    }

    fn force_flush(&mut self) {
        self.flush();
    }

    fn get_stats(&self) -> &WriteStats {
        &self.stats
    }
}

fn main() {
    let mut batcher = TradeBatcher::new(100, 1000); // max 100 records or 1 second

    println!("=== Trade Write Batching ===\n");

    // Simulate trade stream
    println!("Generating 500 trades...\n");

    for i in 0..500 {
        let trade = Trade {
            id: i,
            symbol: if i % 3 == 0 { "BTCUSDT" } else if i % 3 == 1 { "ETHUSDT" } else { "SOLUSDT" }.to_string(),
            price: 42500.0 + (i as f64 * 0.1),
            quantity: 0.1 + (i as f64 * 0.001),
            timestamp: 1700000000 + i,
        };
        batcher.add(trade);
    }

    // Flush remaining
    batcher.force_flush();

    let stats = batcher.get_stats();
    println!("\n=== Statistics ===");
    println!("Total DB writes: {}", stats.total_writes);
    println!("Total trades: {}", stats.total_records);
    println!("Average batch size: {:.1}", stats.avg_batch_size());
    println!("Average write time: {:.1}ms", stats.avg_write_time());

    // Comparison: without batching would be 500 writes at ~5ms = 2500ms
    // With batching: 5 writes at ~10ms = 50ms
    println!("\nTime savings: ~{}x",
        (stats.total_records as f64 * 5.0) / stats.total_time_ms as f64);
}
```

## Asynchronous Batching with Channels

```rust
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum BatchCommand {
    Add(String),
    Flush,
    Shutdown,
}

#[derive(Debug)]
struct BatchProcessor {
    sender: Sender<BatchCommand>,
}

impl BatchProcessor {
    fn new(batch_size: usize, flush_interval_ms: u64) -> Self {
        let (sender, receiver) = mpsc::channel();

        // Start background processor
        thread::spawn(move || {
            Self::worker(receiver, batch_size, flush_interval_ms);
        });

        BatchProcessor { sender }
    }

    fn worker(receiver: Receiver<BatchCommand>, batch_size: usize, flush_interval_ms: u64) {
        let mut buffer: Vec<String> = Vec::with_capacity(batch_size);
        let flush_interval = Duration::from_millis(flush_interval_ms);

        loop {
            // Try to receive command with timeout
            match receiver.recv_timeout(flush_interval) {
                Ok(BatchCommand::Add(item)) => {
                    buffer.push(item);
                    if buffer.len() >= batch_size {
                        Self::process_batch(&mut buffer);
                    }
                }
                Ok(BatchCommand::Flush) => {
                    if !buffer.is_empty() {
                        Self::process_batch(&mut buffer);
                    }
                }
                Ok(BatchCommand::Shutdown) => {
                    if !buffer.is_empty() {
                        Self::process_batch(&mut buffer);
                    }
                    println!("[Worker] Shutting down");
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout — flush by time
                    if !buffer.is_empty() {
                        Self::process_batch(&mut buffer);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    println!("[Worker] Channel closed");
                    break;
                }
            }
        }
    }

    fn process_batch(buffer: &mut Vec<String>) {
        println!("[Worker] Processing batch of {} items:", buffer.len());
        for item in buffer.iter() {
            println!("  - {}", item);
        }
        buffer.clear();
    }

    fn add(&self, item: String) {
        let _ = self.sender.send(BatchCommand::Add(item));
    }

    fn flush(&self) {
        let _ = self.sender.send(BatchCommand::Flush);
    }

    fn shutdown(&self) {
        let _ = self.sender.send(BatchCommand::Shutdown);
    }
}

fn main() {
    println!("=== Asynchronous Batching ===\n");

    let processor = BatchProcessor::new(3, 500); // batch=3, interval=500ms

    // Add items
    processor.add("BTC update: $42500".to_string());
    processor.add("ETH update: $2500".to_string());
    processor.add("SOL update: $100".to_string()); // Triggers size-based flush

    thread::sleep(Duration::from_millis(100));

    processor.add("BUY BTC order".to_string());

    // Wait for time-based flush
    thread::sleep(Duration::from_millis(600));

    processor.add("SELL ETH order".to_string());

    // Force flush
    processor.flush();

    thread::sleep(Duration::from_millis(100));

    // Shutdown
    processor.shutdown();
    thread::sleep(Duration::from_millis(100));

    println!("\nDone!");
}
```

## Batching with Deduplication

In trading, duplicate data often arrives:

```rust
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

/// Batcher with deduplication — keeps only the latest value for each key
struct DeduplicatingBatcher {
    // For each symbol, store only the latest update
    updates: HashMap<String, PriceUpdate>,
    // Order of addition to preserve sequence
    order: Vec<String>,
    seen: HashSet<String>,
    max_size: usize,
    last_flush: Instant,
    max_wait: Duration,

    // Statistics
    total_received: u64,
    total_deduplicated: u64,
}

impl DeduplicatingBatcher {
    fn new(max_size: usize, max_wait_ms: u64) -> Self {
        DeduplicatingBatcher {
            updates: HashMap::new(),
            order: Vec::new(),
            seen: HashSet::new(),
            max_size,
            last_flush: Instant::now(),
            max_wait: Duration::from_millis(max_wait_ms),
            total_received: 0,
            total_deduplicated: 0,
        }
    }

    fn add(&mut self, update: PriceUpdate) -> Option<Vec<PriceUpdate>> {
        self.total_received += 1;

        let symbol = update.symbol.clone();

        // If update for this symbol already exists — it's a duplicate
        if self.updates.contains_key(&symbol) {
            self.total_deduplicated += 1;
        } else {
            // New symbol — add to order
            self.order.push(symbol.clone());
            self.seen.insert(symbol.clone());
        }

        // Always overwrite with latest value
        self.updates.insert(symbol, update);

        if self.should_flush() {
            Some(self.flush())
        } else {
            None
        }
    }

    fn should_flush(&self) -> bool {
        self.updates.len() >= self.max_size
            || self.last_flush.elapsed() >= self.max_wait
    }

    fn flush(&mut self) -> Vec<PriceUpdate> {
        // Return in order of addition
        let result: Vec<_> = self.order.drain(..)
            .filter_map(|s| self.updates.remove(&s))
            .collect();

        self.seen.clear();
        self.last_flush = Instant::now();
        result
    }

    fn dedup_rate(&self) -> f64 {
        if self.total_received > 0 {
            (self.total_deduplicated as f64 / self.total_received as f64) * 100.0
        } else {
            0.0
        }
    }
}

fn main() {
    let mut batcher = DeduplicatingBatcher::new(5, 100);

    println!("=== Batching with Deduplication ===\n");

    // Simulate price update stream (many duplicates)
    let updates = vec![
        PriceUpdate { symbol: "BTC".to_string(), price: 42500.0, timestamp: 1 },
        PriceUpdate { symbol: "ETH".to_string(), price: 2500.0, timestamp: 1 },
        PriceUpdate { symbol: "BTC".to_string(), price: 42510.0, timestamp: 2 }, // duplicate
        PriceUpdate { symbol: "BTC".to_string(), price: 42520.0, timestamp: 3 }, // duplicate
        PriceUpdate { symbol: "SOL".to_string(), price: 100.0, timestamp: 1 },
        PriceUpdate { symbol: "ETH".to_string(), price: 2510.0, timestamp: 2 }, // duplicate
        PriceUpdate { symbol: "BTC".to_string(), price: 42530.0, timestamp: 4 }, // duplicate
        PriceUpdate { symbol: "DOGE".to_string(), price: 0.08, timestamp: 1 },
        PriceUpdate { symbol: "XRP".to_string(), price: 0.5, timestamp: 1 },
    ];

    println!("Incoming stream ({} updates):\n", updates.len());

    for update in updates {
        println!("  Received: {} @ ${:.2} (ts={})",
            update.symbol, update.price, update.timestamp);

        if let Some(batch) = batcher.add(update) {
            println!("\n--- Processing batch ---");
            for u in &batch {
                println!("  Final value: {} @ ${:.2}", u.symbol, u.price);
            }
            println!();
        }
    }

    println!("\n=== Statistics ===");
    println!("Total received: {}", batcher.total_received);
    println!("Deduplicated: {}", batcher.total_deduplicated);
    println!("Deduplication rate: {:.1}%", batcher.dedup_rate());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Batching** | Grouping operations for batch processing |
| **Flush conditions** | By size, by time, or by event |
| **Priorities** | Critical operations processed first |
| **Deduplication** | Eliminating duplicate data in batch |
| **Asynchronous** | Background processing via channels |
| **Write batching** | Grouping database writes for I/O optimization |

## Practical Exercises

1. **Adaptive Batcher**: Implement a batcher that automatically increases batch size under high load and decreases under low load.

2. **Batching with Backpressure**: Create a system that slows down incoming data acceptance when the processor can't keep up.

3. **Batching Metrics**: Add metrics collection: average time in buffer, fill percentage, forced flush count.

## Homework

1. **Smart Order Batcher**: Implement a batcher that groups orders not only by time, but also by symbol and direction (BUY/SELL) to minimize API calls.

2. **Batching with Retry**: Create a system that automatically retries batch submission on failure with exponential backoff.

3. **Distributed Batching**: Implement a batcher that works across multiple nodes and coordinates submission through leader election.

4. **WebSocket Batching**: Create a system that groups outgoing WebSocket messages to reduce frame count.

## Navigation

[← Previous day](../328-*/en.md) | [Next day →](../330-*/en.md)
