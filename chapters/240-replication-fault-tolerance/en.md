# Day 240: Replication and Fault Tolerance

## Trading Analogy

Imagine you're running a trading desk at a major investment bank. You can't afford to have a single point of failure — if your main trading system goes down during market hours, you could lose millions in missed opportunities or, worse, be unable to close positions during a market crash.

Professional trading firms solve this by running **multiple identical systems** in parallel. If one server fails, another immediately takes over. This is **replication** — keeping multiple copies of your data and systems. Combined with **fault tolerance** — the ability to continue operating when something breaks — this ensures your trading operations never stop.

In the database world:
- **Primary server** = Your main trading desk that handles all orders
- **Replica servers** = Backup desks ready to take over instantly
- **Failover** = When a backup desk becomes the main desk after a failure
- **Data synchronization** = Ensuring all desks have the same portfolio and order information

## Why Replication Matters for Trading Systems

Trading systems require:
1. **High Availability** — Markets don't wait. If your system is down, you miss trades
2. **Data Durability** — You can never lose trade records or position data
3. **Read Scalability** — Multiple services need to read price data simultaneously
4. **Geographic Distribution** — Trade on exchanges worldwide with low latency

## Replication Strategies

### 1. Master-Slave (Primary-Replica) Replication

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

/// Represents a trade record that must be replicated
#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

/// Simulates a database node (primary or replica)
#[derive(Debug)]
struct DatabaseNode {
    name: String,
    is_primary: bool,
    trades: RwLock<HashMap<u64, Trade>>,
    replication_lag_ms: u64,
}

impl DatabaseNode {
    fn new(name: &str, is_primary: bool, lag_ms: u64) -> Self {
        DatabaseNode {
            name: name.to_string(),
            is_primary,
            trades: RwLock::new(HashMap::new()),
            replication_lag_ms: lag_ms,
        }
    }

    fn write_trade(&self, trade: Trade) -> Result<(), String> {
        if !self.is_primary {
            return Err("Cannot write to replica directly!".to_string());
        }

        let mut trades = self.trades.write().unwrap();
        println!("[{}] Writing trade: {:?}", self.name, trade);
        trades.insert(trade.id, trade);
        Ok(())
    }

    fn replicate_trade(&self, trade: Trade) {
        // Simulate replication delay
        thread::sleep(Duration::from_millis(self.replication_lag_ms));

        let mut trades = self.trades.write().unwrap();
        println!("[{}] Replicated trade {} (lag: {}ms)",
            self.name, trade.id, self.replication_lag_ms);
        trades.insert(trade.id, trade);
    }

    fn read_trade(&self, id: u64) -> Option<Trade> {
        let trades = self.trades.read().unwrap();
        trades.get(&id).cloned()
    }

    fn trade_count(&self) -> usize {
        self.trades.read().unwrap().len()
    }
}

/// Trading system with replication
struct ReplicatedTradingSystem {
    primary: Arc<DatabaseNode>,
    replicas: Vec<Arc<DatabaseNode>>,
    next_trade_id: RwLock<u64>,
}

impl ReplicatedTradingSystem {
    fn new() -> Self {
        let primary = Arc::new(DatabaseNode::new("Primary", true, 0));
        let replicas = vec![
            Arc::new(DatabaseNode::new("Replica-1", false, 10)),
            Arc::new(DatabaseNode::new("Replica-2", false, 25)),
            Arc::new(DatabaseNode::new("Replica-3", false, 50)),
        ];

        ReplicatedTradingSystem {
            primary,
            replicas,
            next_trade_id: RwLock::new(1),
        }
    }

    fn execute_trade(&self, symbol: &str, price: f64, quantity: f64) -> Result<u64, String> {
        // Get next trade ID
        let trade_id = {
            let mut id = self.next_trade_id.write().unwrap();
            let current = *id;
            *id += 1;
            current
        };

        let trade = Trade {
            id: trade_id,
            symbol: symbol.to_string(),
            price,
            quantity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Write to primary
        self.primary.write_trade(trade.clone())?;

        // Asynchronously replicate to all replicas
        for replica in &self.replicas {
            let replica = Arc::clone(replica);
            let trade = trade.clone();
            thread::spawn(move || {
                replica.replicate_trade(trade);
            });
        }

        Ok(trade_id)
    }

    fn read_trade_from_replica(&self, id: u64, replica_index: usize) -> Option<Trade> {
        if replica_index < self.replicas.len() {
            self.replicas[replica_index].read_trade(id)
        } else {
            None
        }
    }
}

fn main() {
    let system = ReplicatedTradingSystem::new();

    println!("=== Executing trades on Primary ===\n");

    // Execute some trades
    let trade1 = system.execute_trade("BTC/USD", 42000.0, 0.5).unwrap();
    let trade2 = system.execute_trade("ETH/USD", 2800.0, 5.0).unwrap();
    let trade3 = system.execute_trade("SOL/USD", 95.0, 100.0).unwrap();

    println!("\nTrade IDs: {}, {}, {}", trade1, trade2, trade3);

    // Wait for replication
    println!("\n=== Waiting for replication ===\n");
    thread::sleep(Duration::from_millis(100));

    // Verify replication
    println!("\n=== Verifying replication ===\n");
    println!("Primary trade count: {}", system.primary.trade_count());
    for (i, replica) in system.replicas.iter().enumerate() {
        println!("Replica-{} trade count: {}", i + 1, replica.trade_count());
    }
}
```

### 2. Synchronous vs Asynchronous Replication

```rust
use std::sync::{Arc, Mutex, Barrier};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct OrderUpdate {
    order_id: u64,
    status: String,
    filled_quantity: f64,
}

/// Synchronous replication - waits for all replicas to confirm
struct SyncReplication {
    replicas: Vec<Arc<Mutex<Vec<OrderUpdate>>>>,
}

impl SyncReplication {
    fn new(replica_count: usize) -> Self {
        let replicas = (0..replica_count)
            .map(|_| Arc::new(Mutex::new(Vec::new())))
            .collect();
        SyncReplication { replicas }
    }

    fn replicate(&self, update: OrderUpdate) -> Duration {
        let start = Instant::now();

        // Use a barrier to synchronize all replica writes
        let barrier = Arc::new(Barrier::new(self.replicas.len() + 1));
        let mut handles = vec![];

        for (i, replica) in self.replicas.iter().enumerate() {
            let replica = Arc::clone(replica);
            let update = update.clone();
            let barrier = Arc::clone(&barrier);

            handles.push(thread::spawn(move || {
                // Simulate network latency (varies by replica)
                thread::sleep(Duration::from_millis(10 + (i as u64 * 15)));

                let mut data = replica.lock().unwrap();
                data.push(update);

                // Signal completion
                barrier.wait();
            }));
        }

        // Wait for all replicas
        barrier.wait();

        for handle in handles {
            handle.join().unwrap();
        }

        start.elapsed()
    }
}

/// Asynchronous replication - returns immediately, replicates in background
struct AsyncReplication {
    replicas: Vec<Arc<Mutex<Vec<OrderUpdate>>>>,
    pending_count: Arc<Mutex<u64>>,
}

impl AsyncReplication {
    fn new(replica_count: usize) -> Self {
        let replicas = (0..replica_count)
            .map(|_| Arc::new(Mutex::new(Vec::new())))
            .collect();
        AsyncReplication {
            replicas,
            pending_count: Arc::new(Mutex::new(0)),
        }
    }

    fn replicate(&self, update: OrderUpdate) -> Duration {
        let start = Instant::now();

        {
            let mut pending = self.pending_count.lock().unwrap();
            *pending += self.replicas.len() as u64;
        }

        for (i, replica) in self.replicas.iter().enumerate() {
            let replica = Arc::clone(replica);
            let update = update.clone();
            let pending_count = Arc::clone(&self.pending_count);

            thread::spawn(move || {
                // Simulate network latency
                thread::sleep(Duration::from_millis(10 + (i as u64 * 15)));

                let mut data = replica.lock().unwrap();
                data.push(update);

                let mut pending = pending_count.lock().unwrap();
                *pending -= 1;
            });
        }

        // Return immediately without waiting
        start.elapsed()
    }

    fn pending_replications(&self) -> u64 {
        *self.pending_count.lock().unwrap()
    }
}

fn main() {
    let update = OrderUpdate {
        order_id: 12345,
        status: "FILLED".to_string(),
        filled_quantity: 100.0,
    };

    println!("=== Synchronous Replication ===\n");
    let sync_replication = SyncReplication::new(3);
    let sync_time = sync_replication.replicate(update.clone());
    println!("Sync replication took: {:?}", sync_time);
    println!("All replicas confirmed before returning\n");

    println!("=== Asynchronous Replication ===\n");
    let async_replication = AsyncReplication::new(3);
    let async_time = async_replication.replicate(update.clone());
    println!("Async replication returned in: {:?}", async_time);
    println!("Pending replications: {}", async_replication.pending_replications());

    // Wait and check again
    thread::sleep(Duration::from_millis(100));
    println!("After waiting - Pending: {}", async_replication.pending_replications());

    println!("\n=== Trade-offs ===");
    println!("Sync: Slower ({:?}) but guarantees durability", sync_time);
    println!("Async: Faster ({:?}) but may lose data on failure", async_time);
}
```

## Fault Tolerance Patterns

### 1. Health Checks and Failover

```rust
use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum NodeStatus {
    Healthy,
    Degraded,
    Failed,
}

#[derive(Debug)]
struct TradingNode {
    id: String,
    is_alive: AtomicBool,
    last_heartbeat: RwLock<Instant>,
    processed_orders: AtomicU64,
    status: RwLock<NodeStatus>,
}

impl TradingNode {
    fn new(id: &str) -> Self {
        TradingNode {
            id: id.to_string(),
            is_alive: AtomicBool::new(true),
            last_heartbeat: RwLock::new(Instant::now()),
            processed_orders: AtomicU64::new(0),
            status: RwLock::new(NodeStatus::Healthy),
        }
    }

    fn heartbeat(&self) {
        *self.last_heartbeat.write().unwrap() = Instant::now();
    }

    fn process_order(&self) -> Result<u64, String> {
        if !self.is_alive.load(Ordering::SeqCst) {
            return Err(format!("Node {} is down!", self.id));
        }

        let order_num = self.processed_orders.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(order_num)
    }

    fn simulate_failure(&self) {
        self.is_alive.store(false, Ordering::SeqCst);
        *self.status.write().unwrap() = NodeStatus::Failed;
    }

    fn recover(&self) {
        self.is_alive.store(true, Ordering::SeqCst);
        self.heartbeat();
        *self.status.write().unwrap() = NodeStatus::Healthy;
    }
}

struct FaultTolerantCluster {
    nodes: Vec<Arc<TradingNode>>,
    primary_index: RwLock<usize>,
    heartbeat_timeout: Duration,
}

impl FaultTolerantCluster {
    fn new(node_count: usize) -> Self {
        let nodes = (0..node_count)
            .map(|i| Arc::new(TradingNode::new(&format!("node-{}", i))))
            .collect();

        FaultTolerantCluster {
            nodes,
            primary_index: RwLock::new(0),
            heartbeat_timeout: Duration::from_millis(100),
        }
    }

    fn get_primary(&self) -> Arc<TradingNode> {
        let index = *self.primary_index.read().unwrap();
        Arc::clone(&self.nodes[index])
    }

    fn check_health(&self) -> HashMap<String, NodeStatus> {
        let mut health = HashMap::new();
        let now = Instant::now();

        for node in &self.nodes {
            let last_hb = *node.last_heartbeat.read().unwrap();
            let status = if !node.is_alive.load(Ordering::SeqCst) {
                NodeStatus::Failed
            } else if now.duration_since(last_hb) > self.heartbeat_timeout {
                NodeStatus::Degraded
            } else {
                NodeStatus::Healthy
            };

            *node.status.write().unwrap() = status.clone();
            health.insert(node.id.clone(), status);
        }

        health
    }

    fn failover(&self) -> Result<(), String> {
        let current_primary = *self.primary_index.read().unwrap();

        // Find next healthy node
        for i in 1..self.nodes.len() {
            let candidate_index = (current_primary + i) % self.nodes.len();
            let candidate = &self.nodes[candidate_index];

            if candidate.is_alive.load(Ordering::SeqCst) {
                *self.primary_index.write().unwrap() = candidate_index;
                println!("FAILOVER: {} is now primary", candidate.id);
                return Ok(());
            }
        }

        Err("No healthy nodes available for failover!".to_string())
    }

    fn process_order(&self) -> Result<u64, String> {
        let primary = self.get_primary();

        match primary.process_order() {
            Ok(order_num) => Ok(order_num),
            Err(_) => {
                println!("Primary failed! Initiating failover...");
                self.failover()?;
                // Retry on new primary
                self.get_primary().process_order()
            }
        }
    }
}

fn main() {
    let cluster = Arc::new(FaultTolerantCluster::new(3));

    println!("=== Fault Tolerant Trading Cluster ===\n");

    // Process some orders normally
    for i in 1..=5 {
        match cluster.process_order() {
            Ok(num) => println!("Order {} processed (total: {})", i, num),
            Err(e) => println!("Order {} failed: {}", i, e),
        }
    }

    println!("\n=== Health Check ===");
    let health = cluster.check_health();
    for (node, status) in &health {
        println!("{}: {:?}", node, status);
    }

    // Simulate primary failure
    println!("\n=== Simulating Primary Failure ===\n");
    cluster.get_primary().simulate_failure();

    // Try to process more orders (should trigger failover)
    for i in 6..=10 {
        match cluster.process_order() {
            Ok(num) => println!("Order {} processed (total: {})", i, num),
            Err(e) => println!("Order {} failed: {}", i, e),
        }
    }

    println!("\n=== Final Health Check ===");
    let health = cluster.check_health();
    for (node, status) in &health {
        println!("{}: {:?}", node, status);
    }
}
```

### 2. Write-Ahead Log (WAL) for Durability

```rust
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter, BufReader, BufRead};
use std::path::Path;

#[derive(Debug, Clone)]
struct TradeEntry {
    sequence: u64,
    trade_id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

impl TradeEntry {
    fn to_wal_format(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}\n",
            self.sequence, self.trade_id, self.symbol,
            self.side, self.price, self.quantity
        )
    }

    fn from_wal_format(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.trim().split('|').collect();
        if parts.len() != 6 {
            return None;
        }

        Some(TradeEntry {
            sequence: parts[0].parse().ok()?,
            trade_id: parts[1].parse().ok()?,
            symbol: parts[2].to_string(),
            side: parts[3].to_string(),
            price: parts[4].parse().ok()?,
            quantity: parts[5].parse().ok()?,
        })
    }
}

struct WriteAheadLog {
    file: BufWriter<File>,
    sequence: u64,
}

impl WriteAheadLog {
    fn new(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        // Count existing entries to get sequence number
        let sequence = if Path::new(path).exists() {
            let reader = BufReader::new(File::open(path)?);
            reader.lines().count() as u64
        } else {
            0
        };

        Ok(WriteAheadLog {
            file: BufWriter::new(file),
            sequence,
        })
    }

    fn append(&mut self, mut entry: TradeEntry) -> std::io::Result<u64> {
        self.sequence += 1;
        entry.sequence = self.sequence;

        let data = entry.to_wal_format();
        self.file.write_all(data.as_bytes())?;
        self.file.flush()?; // Ensure durability

        Ok(self.sequence)
    }

    fn recover(path: &str) -> std::io::Result<Vec<TradeEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let entries: Vec<TradeEntry> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| TradeEntry::from_wal_format(&line))
            .collect();

        Ok(entries)
    }
}

struct DurableTradingEngine {
    wal: WriteAheadLog,
    trades: Vec<TradeEntry>,
    next_trade_id: u64,
}

impl DurableTradingEngine {
    fn new(wal_path: &str) -> std::io::Result<Self> {
        // First, try to recover from existing WAL
        let trades = if Path::new(wal_path).exists() {
            println!("Recovering from WAL...");
            WriteAheadLog::recover(wal_path)?
        } else {
            vec![]
        };

        let next_trade_id = trades.iter()
            .map(|t| t.trade_id)
            .max()
            .unwrap_or(0) + 1;

        println!("Recovered {} trades, next ID: {}", trades.len(), next_trade_id);

        Ok(DurableTradingEngine {
            wal: WriteAheadLog::new(wal_path)?,
            trades,
            next_trade_id,
        })
    }

    fn execute_trade(&mut self, symbol: &str, side: &str, price: f64, quantity: f64)
        -> std::io::Result<u64>
    {
        let trade_id = self.next_trade_id;
        self.next_trade_id += 1;

        let entry = TradeEntry {
            sequence: 0, // Will be set by WAL
            trade_id,
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            quantity,
        };

        // Write to WAL FIRST (before in-memory update)
        let seq = self.wal.append(entry.clone())?;
        println!("Trade {} written to WAL (seq: {})", trade_id, seq);

        // Then update in-memory state
        self.trades.push(entry);

        Ok(trade_id)
    }

    fn get_trades(&self) -> &[TradeEntry] {
        &self.trades
    }
}

fn main() -> std::io::Result<()> {
    let wal_path = "/tmp/trading_wal.log";

    // Clean start for demo
    if Path::new(wal_path).exists() {
        std::fs::remove_file(wal_path)?;
    }

    println!("=== Write-Ahead Log Demo ===\n");

    // First run - create some trades
    {
        let mut engine = DurableTradingEngine::new(wal_path)?;

        engine.execute_trade("BTC/USD", "BUY", 42000.0, 1.0)?;
        engine.execute_trade("ETH/USD", "SELL", 2800.0, 10.0)?;
        engine.execute_trade("SOL/USD", "BUY", 95.0, 50.0)?;

        println!("\nTrades in memory: {}", engine.get_trades().len());
    }

    println!("\n=== Simulating Crash and Recovery ===\n");

    // Second run - recover from WAL
    {
        let engine = DurableTradingEngine::new(wal_path)?;

        println!("\nRecovered trades:");
        for trade in engine.get_trades() {
            println!("  {:?}", trade);
        }
    }

    // Cleanup
    std::fs::remove_file(wal_path)?;

    Ok(())
}
```

### 3. Quorum-Based Consistency

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    version: u64,
}

#[derive(Debug)]
struct QuorumNode {
    id: String,
    prices: RwLock<HashMap<String, PriceUpdate>>,
    latency_ms: u64,
}

impl QuorumNode {
    fn new(id: &str, latency_ms: u64) -> Self {
        QuorumNode {
            id: id.to_string(),
            prices: RwLock::new(HashMap::new()),
            latency_ms,
        }
    }

    fn write(&self, update: PriceUpdate) -> bool {
        thread::sleep(Duration::from_millis(self.latency_ms));

        let mut prices = self.prices.write().unwrap();

        // Only accept if version is higher
        if let Some(existing) = prices.get(&update.symbol) {
            if existing.version >= update.version {
                return false;
            }
        }

        prices.insert(update.symbol.clone(), update);
        true
    }

    fn read(&self, symbol: &str) -> Option<PriceUpdate> {
        thread::sleep(Duration::from_millis(self.latency_ms));

        let prices = self.prices.read().unwrap();
        prices.get(symbol).cloned()
    }
}

struct QuorumCluster {
    nodes: Vec<Arc<QuorumNode>>,
    write_quorum: usize,  // W
    read_quorum: usize,   // R
}

impl QuorumCluster {
    fn new(node_count: usize, write_quorum: usize, read_quorum: usize) -> Self {
        assert!(write_quorum + read_quorum > node_count,
            "W + R must be greater than N for consistency!");

        let nodes = (0..node_count)
            .map(|i| Arc::new(QuorumNode::new(
                &format!("node-{}", i),
                10 + (i as u64 * 5), // Varying latencies
            )))
            .collect();

        QuorumCluster {
            nodes,
            write_quorum,
            read_quorum,
        }
    }

    fn write(&self, update: PriceUpdate) -> Result<usize, String> {
        let mut handles = vec![];

        for node in &self.nodes {
            let node = Arc::clone(node);
            let update = update.clone();
            handles.push(thread::spawn(move || {
                node.write(update)
            }));
        }

        let mut success_count = 0;
        for handle in handles {
            if handle.join().unwrap() {
                success_count += 1;
            }
        }

        if success_count >= self.write_quorum {
            Ok(success_count)
        } else {
            Err(format!("Write quorum not reached: {} < {}",
                success_count, self.write_quorum))
        }
    }

    fn read(&self, symbol: &str) -> Option<PriceUpdate> {
        let mut handles = vec![];

        for node in &self.nodes {
            let node = Arc::clone(node);
            let symbol = symbol.to_string();
            handles.push(thread::spawn(move || {
                node.read(&symbol)
            }));
        }

        let mut results: Vec<PriceUpdate> = vec![];
        for handle in handles {
            if let Some(update) = handle.join().unwrap() {
                results.push(update);
            }

            // Return as soon as we have read quorum
            if results.len() >= self.read_quorum {
                break;
            }
        }

        if results.len() >= self.read_quorum {
            // Return the value with highest version
            results.into_iter().max_by_key(|u| u.version)
        } else {
            None
        }
    }
}

fn main() {
    println!("=== Quorum-Based Consistency ===\n");
    println!("N=5, W=3, R=3 (W+R > N ensures consistency)\n");

    let cluster = QuorumCluster::new(5, 3, 3);

    // Write price updates
    let updates = vec![
        PriceUpdate { symbol: "BTC/USD".into(), price: 42000.0, version: 1 },
        PriceUpdate { symbol: "BTC/USD".into(), price: 42100.0, version: 2 },
        PriceUpdate { symbol: "ETH/USD".into(), price: 2800.0, version: 1 },
    ];

    for update in updates {
        println!("Writing: {} = ${} (v{})", update.symbol, update.price, update.version);
        match cluster.write(update) {
            Ok(count) => println!("  Success: {} nodes acknowledged\n", count),
            Err(e) => println!("  Failed: {}\n", e),
        }
    }

    // Read prices
    println!("=== Reading Prices ===\n");

    for symbol in &["BTC/USD", "ETH/USD", "SOL/USD"] {
        match cluster.read(symbol) {
            Some(update) => {
                println!("{}: ${} (version {})", symbol, update.price, update.version);
            }
            None => println!("{}: Not found or quorum not reached", symbol),
        }
    }

    println!("\n=== Quorum Properties ===");
    println!("- Read quorum (R=3) overlaps with write quorum (W=3)");
    println!("- This guarantees reading the latest written value");
    println!("- Can tolerate 2 node failures for reads and writes");
}
```

## Practical Example: Resilient Order Management System

```rust
use std::sync::{Arc, RwLock, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    status: OrderStatus,
    created_at: u64,
}

#[derive(Debug)]
struct ReplicaState {
    orders: RwLock<HashMap<u64, Order>>,
    last_sync: RwLock<Instant>,
    is_healthy: AtomicBool,
}

impl ReplicaState {
    fn new() -> Self {
        ReplicaState {
            orders: RwLock::new(HashMap::new()),
            last_sync: RwLock::new(Instant::now()),
            is_healthy: AtomicBool::new(true),
        }
    }
}

struct ResilientOrderSystem {
    primary: Arc<ReplicaState>,
    replicas: Vec<Arc<ReplicaState>>,
    next_order_id: AtomicU64,
    sync_interval: Duration,
}

impl ResilientOrderSystem {
    fn new(replica_count: usize) -> Self {
        let replicas = (0..replica_count)
            .map(|_| Arc::new(ReplicaState::new()))
            .collect();

        ResilientOrderSystem {
            primary: Arc::new(ReplicaState::new()),
            replicas,
            next_order_id: AtomicU64::new(1),
            sync_interval: Duration::from_millis(50),
        }
    }

    fn place_order(&self, symbol: &str, price: f64, quantity: f64) -> Result<u64, String> {
        // Check primary health
        if !self.primary.is_healthy.load(Ordering::SeqCst) {
            return Err("Primary is unhealthy, rejecting new orders".to_string());
        }

        let order_id = self.next_order_id.fetch_add(1, Ordering::SeqCst);

        let order = Order {
            id: order_id,
            symbol: symbol.to_string(),
            price,
            quantity,
            status: OrderStatus::Pending,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Write to primary
        {
            let mut orders = self.primary.orders.write().unwrap();
            orders.insert(order_id, order.clone());
        }

        // Asynchronous replication
        for replica in &self.replicas {
            let replica = Arc::clone(replica);
            let order = order.clone();
            thread::spawn(move || {
                if replica.is_healthy.load(Ordering::SeqCst) {
                    let mut orders = replica.orders.write().unwrap();
                    orders.insert(order.id, order);
                    *replica.last_sync.write().unwrap() = Instant::now();
                }
            });
        }

        Ok(order_id)
    }

    fn update_order_status(&self, order_id: u64, status: OrderStatus) -> Result<(), String> {
        // Update primary
        {
            let mut orders = self.primary.orders.write().unwrap();
            if let Some(order) = orders.get_mut(&order_id) {
                order.status = status.clone();
            } else {
                return Err(format!("Order {} not found", order_id));
            }
        }

        // Replicate status update
        for replica in &self.replicas {
            let replica = Arc::clone(replica);
            let status = status.clone();
            thread::spawn(move || {
                if replica.is_healthy.load(Ordering::SeqCst) {
                    let mut orders = replica.orders.write().unwrap();
                    if let Some(order) = orders.get_mut(&order_id) {
                        order.status = status;
                    }
                }
            });
        }

        Ok(())
    }

    fn get_order(&self, order_id: u64) -> Option<Order> {
        // Try primary first
        if self.primary.is_healthy.load(Ordering::SeqCst) {
            let orders = self.primary.orders.read().unwrap();
            if let Some(order) = orders.get(&order_id) {
                return Some(order.clone());
            }
        }

        // Fallback to replicas
        for replica in &self.replicas {
            if replica.is_healthy.load(Ordering::SeqCst) {
                let orders = replica.orders.read().unwrap();
                if let Some(order) = orders.get(&order_id) {
                    return Some(order.clone());
                }
            }
        }

        None
    }

    fn get_replication_status(&self) -> Vec<(usize, bool, Duration)> {
        self.replicas.iter().enumerate().map(|(i, replica)| {
            let is_healthy = replica.is_healthy.load(Ordering::SeqCst);
            let lag = replica.last_sync.read().unwrap().elapsed();
            (i, is_healthy, lag)
        }).collect()
    }

    fn simulate_replica_failure(&self, index: usize) {
        if index < self.replicas.len() {
            self.replicas[index].is_healthy.store(false, Ordering::SeqCst);
        }
    }

    fn recover_replica(&self, index: usize) {
        if index < self.replicas.len() {
            // Copy all data from primary to replica
            let primary_orders = self.primary.orders.read().unwrap();
            let mut replica_orders = self.replicas[index].orders.write().unwrap();

            *replica_orders = primary_orders.clone();

            self.replicas[index].is_healthy.store(true, Ordering::SeqCst);
            *self.replicas[index].last_sync.write().unwrap() = Instant::now();
        }
    }
}

fn main() {
    println!("=== Resilient Order Management System ===\n");

    let system = Arc::new(ResilientOrderSystem::new(3));

    // Place some orders
    println!("Placing orders...\n");
    let order1 = system.place_order("BTC/USD", 42000.0, 1.0).unwrap();
    let order2 = system.place_order("ETH/USD", 2800.0, 5.0).unwrap();
    let order3 = system.place_order("SOL/USD", 95.0, 100.0).unwrap();

    thread::sleep(Duration::from_millis(50));

    println!("Orders placed: {}, {}, {}", order1, order2, order3);

    // Check replication status
    println!("\nReplication Status:");
    for (i, healthy, lag) in system.get_replication_status() {
        println!("  Replica {}: healthy={}, lag={:?}", i, healthy, lag);
    }

    // Update order status
    println!("\nUpdating order statuses...");
    system.update_order_status(order1, OrderStatus::Filled).unwrap();
    system.update_order_status(order2, OrderStatus::Cancelled).unwrap();

    thread::sleep(Duration::from_millis(50));

    // Simulate failure
    println!("\n=== Simulating Replica 0 Failure ===\n");
    system.simulate_replica_failure(0);

    // Place more orders (should still work)
    let order4 = system.place_order("DOGE/USD", 0.08, 10000.0).unwrap();
    println!("Order {} placed despite replica failure", order4);

    // Check replication status again
    println!("\nReplication Status after failure:");
    for (i, healthy, lag) in system.get_replication_status() {
        println!("  Replica {}: healthy={}, lag={:?}", i, healthy, lag);
    }

    // Recover the failed replica
    println!("\n=== Recovering Replica 0 ===\n");
    system.recover_replica(0);

    println!("Replication Status after recovery:");
    for (i, healthy, lag) in system.get_replication_status() {
        println!("  Replica {}: healthy={}, lag={:?}", i, healthy, lag);
    }

    // Verify all orders are readable
    println!("\n=== Verifying Order Data ===\n");
    for id in [order1, order2, order3, order4] {
        if let Some(order) = system.get_order(id) {
            println!("Order {}: {} {} @ ${} - {:?}",
                order.id, order.quantity, order.symbol, order.price, order.status);
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Replication | Maintaining multiple copies of data across nodes |
| Primary-Replica | One node handles writes, others serve reads |
| Synchronous Replication | Wait for all replicas before confirming write |
| Asynchronous Replication | Write returns immediately, replicate in background |
| Fault Tolerance | Ability to continue operation despite failures |
| Failover | Automatic switch to backup when primary fails |
| Write-Ahead Log (WAL) | Log writes before applying for crash recovery |
| Quorum | Require majority agreement for consistency |
| Health Checks | Continuous monitoring of node availability |

## Homework

1. **Multi-Region Replication**: Implement a trading system with replicas in three "regions" (simulated with different latencies). Ensure that orders placed in one region are eventually consistent across all regions. Add logic to prefer reading from the closest replica.

2. **Conflict Resolution**: Create a system where two replicas can receive conflicting updates (e.g., two different prices for the same symbol). Implement "last-write-wins" using timestamps and a "version vector" approach. Compare the results.

3. **Automatic Failover with Leader Election**: Implement a simple leader election algorithm where nodes vote for a new primary when the current one fails. Use heartbeats to detect failure and ensure only one node becomes the new leader.

4. **Consistent Hashing for Order Distribution**: Build a system that distributes orders across multiple nodes using consistent hashing. When a node fails, orders should be redistributed to remaining nodes with minimal data movement.

## Navigation

[← Previous day](../239-clickhouse-big-data/en.md) | [Next day →](../241-backups-preserving-history/en.md)
