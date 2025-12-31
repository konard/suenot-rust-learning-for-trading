# Day 358: Horizontal Scaling

## Trading Analogy

Imagine you're running a high-frequency trading firm. Initially, you have one powerful server handling all orders. But as your business grows, you face problems:

- During peak trading hours, the server can't handle the load
- If the server fails — all orders stop
- Adding more power to a single server becomes too expensive

**Horizontal Scaling in Trading:**
Instead of buying one $1 million supercomputer, you can:

1. **Split by instruments**: One cluster for cryptocurrencies, another for stocks
2. **Split by functions**: Separate clusters for order execution, market analysis, risk management
3. **Geographic distribution**: Servers closer to exchanges for minimal latency

| Approach | Cost | Fault Tolerance | Complexity |
|----------|------|-----------------|------------|
| **Vertical** | High | Low (single server) | Low |
| **Horizontal** | Medium | High (many servers) | High |
| **Hybrid** | Optimal | High | Medium |

**Why Horizontal Scaling?**
In trading, horizontal scaling is critical:
- **Fault tolerance**: If one server fails, others continue working
- **Elasticity**: Easy to add capacity during high volatility
- **Cost efficiency**: 10 servers at $10k each are cheaper than one $1M with the same power

## Horizontal Scaling Fundamentals

### Load Balancing for Orders

```rust
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;

/// State of an order processing worker node
#[derive(Debug, Clone)]
pub struct WorkerNode {
    pub id: String,
    pub address: String,
    pub capacity: u64,
    pub current_load: Arc<AtomicU64>,
    pub is_healthy: bool,
    pub latency_ms: f64,
}

impl WorkerNode {
    pub fn new(id: &str, address: &str, capacity: u64) -> Self {
        WorkerNode {
            id: id.to_string(),
            address: address.to_string(),
            capacity,
            current_load: Arc::new(AtomicU64::new(0)),
            is_healthy: true,
            latency_ms: 0.0,
        }
    }

    pub fn load_percentage(&self) -> f64 {
        let load = self.current_load.load(Ordering::Relaxed);
        (load as f64 / self.capacity as f64) * 100.0
    }

    pub fn available_capacity(&self) -> u64 {
        let load = self.current_load.load(Ordering::Relaxed);
        self.capacity.saturating_sub(load)
    }
}

/// Load balancing strategy
#[derive(Debug, Clone, Copy)]
pub enum LoadBalancingStrategy {
    /// Round-robin selection
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Weighted by capacity
    WeightedCapacity,
    /// Based on latency
    LowestLatency,
    /// Hash-based binding (for consistency)
    HashBased,
}

/// Load balancer for trading orders
pub struct OrderLoadBalancer {
    workers: Vec<WorkerNode>,
    strategy: LoadBalancingStrategy,
    round_robin_counter: AtomicUsize,
    orders_routed: AtomicU64,
}

impl OrderLoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        OrderLoadBalancer {
            workers: Vec::new(),
            strategy,
            round_robin_counter: AtomicUsize::new(0),
            orders_routed: AtomicU64::new(0),
        }
    }

    pub fn add_worker(&mut self, worker: WorkerNode) {
        println!("Added worker node: {} ({})", worker.id, worker.address);
        self.workers.push(worker);
    }

    pub fn remove_worker(&mut self, worker_id: &str) {
        self.workers.retain(|w| w.id != worker_id);
        println!("Removed worker node: {}", worker_id);
    }

    /// Select a worker node for an order
    pub fn route_order(&self, order_id: &str, symbol: &str) -> Option<&WorkerNode> {
        let healthy_workers: Vec<_> = self.workers.iter()
            .filter(|w| w.is_healthy && w.available_capacity() > 0)
            .collect();

        if healthy_workers.is_empty() {
            return None;
        }

        self.orders_routed.fetch_add(1, Ordering::Relaxed);

        let selected = match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let idx = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
                healthy_workers[idx % healthy_workers.len()]
            }
            LoadBalancingStrategy::LeastConnections => {
                healthy_workers.iter()
                    .min_by_key(|w| w.current_load.load(Ordering::Relaxed))
                    .copied()
                    .unwrap()
            }
            LoadBalancingStrategy::WeightedCapacity => {
                healthy_workers.iter()
                    .max_by(|a, b| {
                        a.available_capacity().cmp(&b.available_capacity())
                    })
                    .copied()
                    .unwrap()
            }
            LoadBalancingStrategy::LowestLatency => {
                healthy_workers.iter()
                    .min_by(|a, b| {
                        a.latency_ms.partial_cmp(&b.latency_ms).unwrap()
                    })
                    .copied()
                    .unwrap()
            }
            LoadBalancingStrategy::HashBased => {
                // Consistent hashing by symbol for cache locality
                let hash = symbol.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize));
                healthy_workers[hash % healthy_workers.len()]
            }
        };

        // Increase load on selected node
        selected.current_load.fetch_add(1, Ordering::Relaxed);

        Some(selected)
    }

    /// Release resource on node after order processing
    pub fn release_order(&self, worker_id: &str) {
        if let Some(worker) = self.workers.iter().find(|w| w.id == worker_id) {
            worker.current_load.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Get load balancer statistics
    pub fn stats(&self) -> LoadBalancerStats {
        let total_capacity: u64 = self.workers.iter().map(|w| w.capacity).sum();
        let total_load: u64 = self.workers.iter()
            .map(|w| w.current_load.load(Ordering::Relaxed))
            .sum();
        let healthy_count = self.workers.iter().filter(|w| w.is_healthy).count();

        LoadBalancerStats {
            total_workers: self.workers.len(),
            healthy_workers: healthy_count,
            total_capacity,
            total_load,
            utilization_pct: if total_capacity > 0 {
                (total_load as f64 / total_capacity as f64) * 100.0
            } else {
                0.0
            },
            orders_routed: self.orders_routed.load(Ordering::Relaxed),
        }
    }

    pub fn print_status(&self) {
        println!("\n=== Load Balancer Status ===");
        println!("Strategy: {:?}", self.strategy);

        for worker in &self.workers {
            let status = if worker.is_healthy { "✓" } else { "✗" };
            println!("  {} {} - Load: {:.1}% ({}/{}), Latency: {:.1}ms",
                status,
                worker.id,
                worker.load_percentage(),
                worker.current_load.load(Ordering::Relaxed),
                worker.capacity,
                worker.latency_ms);
        }

        let stats = self.stats();
        println!("\nTotal utilization: {:.1}%", stats.utilization_pct);
        println!("Total orders routed: {}", stats.orders_routed);
    }
}

#[derive(Debug)]
pub struct LoadBalancerStats {
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub total_capacity: u64,
    pub total_load: u64,
    pub utilization_pct: f64,
    pub orders_routed: u64,
}

fn main() {
    println!("=== Load Balancing for Orders ===\n");

    let mut balancer = OrderLoadBalancer::new(LoadBalancingStrategy::LeastConnections);

    // Add worker nodes
    balancer.add_worker(WorkerNode {
        id: "worker-1".to_string(),
        address: "10.0.0.1:8080".to_string(),
        capacity: 1000,
        current_load: Arc::new(AtomicU64::new(0)),
        is_healthy: true,
        latency_ms: 1.5,
    });

    balancer.add_worker(WorkerNode {
        id: "worker-2".to_string(),
        address: "10.0.0.2:8080".to_string(),
        capacity: 1500,
        current_load: Arc::new(AtomicU64::new(0)),
        is_healthy: true,
        latency_ms: 2.0,
    });

    balancer.add_worker(WorkerNode {
        id: "worker-3".to_string(),
        address: "10.0.0.3:8080".to_string(),
        capacity: 800,
        current_load: Arc::new(AtomicU64::new(0)),
        is_healthy: true,
        latency_ms: 1.2,
    });

    // Simulate order routing
    println!("\nRouting orders...");
    for i in 0..20 {
        let order_id = format!("order-{}", i);
        let symbol = if i % 2 == 0 { "BTCUSDT" } else { "ETHUSDT" };

        if let Some(worker) = balancer.route_order(&order_id, symbol) {
            println!("Order {} ({}) -> {}", order_id, symbol, worker.id);
        }
    }

    balancer.print_status();

    // Release some orders
    println!("\nReleasing orders...");
    for i in 0..10 {
        let worker_id = format!("worker-{}", (i % 3) + 1);
        balancer.release_order(&worker_id);
    }

    balancer.print_status();
}
```

## Data Partitioning

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Partitioning strategy
#[derive(Debug, Clone, Copy)]
pub enum PartitionStrategy {
    /// Hash-based partitioning
    Hash,
    /// Range-based partitioning
    Range,
    /// List-based partitioning
    List,
    /// Consistent hashing
    ConsistentHash,
}

/// Data partition
#[derive(Debug)]
pub struct Partition<V> {
    pub id: u32,
    pub data: RwLock<HashMap<String, V>>,
    pub item_count: RwLock<usize>,
}

impl<V: Clone> Partition<V> {
    pub fn new(id: u32) -> Self {
        Partition {
            id,
            data: RwLock::new(HashMap::new()),
            item_count: RwLock::new(0),
        }
    }

    pub fn insert(&self, key: String, value: V) {
        let mut data = self.data.write().unwrap();
        let is_new = !data.contains_key(&key);
        data.insert(key, value);
        if is_new {
            *self.item_count.write().unwrap() += 1;
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        self.data.read().unwrap().get(key).cloned()
    }

    pub fn count(&self) -> usize {
        *self.item_count.read().unwrap()
    }
}

/// Trading data partitioner
pub struct TradingDataPartitioner<V> {
    partitions: Vec<Arc<Partition<V>>>,
    strategy: PartitionStrategy,
    symbol_to_partition: RwLock<HashMap<String, u32>>,
}

impl<V: Clone> TradingDataPartitioner<V> {
    pub fn new(partition_count: u32, strategy: PartitionStrategy) -> Self {
        let partitions = (0..partition_count)
            .map(|id| Arc::new(Partition::new(id)))
            .collect();

        TradingDataPartitioner {
            partitions,
            strategy,
            symbol_to_partition: RwLock::new(HashMap::new()),
        }
    }

    /// Compute hash for key
    fn hash_key(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Determine partition for key
    pub fn get_partition(&self, key: &str) -> Arc<Partition<V>> {
        let partition_id = match self.strategy {
            PartitionStrategy::Hash => {
                let hash = Self::hash_key(key);
                (hash % self.partitions.len() as u64) as u32
            }
            PartitionStrategy::Range => {
                // Partition by first letter of symbol
                let first_char = key.chars().next().unwrap_or('A');
                let range = (first_char as u32 - 'A' as u32) % self.partitions.len() as u32;
                range
            }
            PartitionStrategy::List => {
                // Use cache for symbol to partition binding
                let cache = self.symbol_to_partition.read().unwrap();
                if let Some(&partition_id) = cache.get(key) {
                    partition_id
                } else {
                    drop(cache);
                    // Assign by hash if not in list
                    let hash = Self::hash_key(key);
                    let partition_id = (hash % self.partitions.len() as u64) as u32;
                    self.symbol_to_partition.write().unwrap().insert(key.to_string(), partition_id);
                    partition_id
                }
            }
            PartitionStrategy::ConsistentHash => {
                // Consistent hashing with virtual nodes
                let hash = Self::hash_key(key);
                // Simplified version: use modulo
                (hash % self.partitions.len() as u64) as u32
            }
        };

        Arc::clone(&self.partitions[partition_id as usize])
    }

    /// Insert data
    pub fn insert(&self, key: String, value: V) {
        let partition = self.get_partition(&key);
        partition.insert(key, value);
    }

    /// Get data
    pub fn get(&self, key: &str) -> Option<V> {
        let partition = self.get_partition(key);
        partition.get(key)
    }

    /// Partition statistics
    pub fn stats(&self) -> PartitionStats {
        let counts: Vec<usize> = self.partitions.iter()
            .map(|p| p.count())
            .collect();

        let total: usize = counts.iter().sum();
        let max = *counts.iter().max().unwrap_or(&0);
        let min = *counts.iter().min().unwrap_or(&0);
        let avg = if counts.is_empty() { 0.0 } else { total as f64 / counts.len() as f64 };

        // Imbalance coefficient (standard deviation)
        let variance = counts.iter()
            .map(|&c| (c as f64 - avg).powi(2))
            .sum::<f64>() / counts.len() as f64;
        let std_dev = variance.sqrt();

        PartitionStats {
            partition_count: self.partitions.len(),
            total_items: total,
            items_per_partition: counts,
            max_items: max,
            min_items: min,
            avg_items: avg,
            imbalance_ratio: if min > 0 { max as f64 / min as f64 } else { 0.0 },
            std_deviation: std_dev,
        }
    }

    pub fn print_stats(&self) {
        let stats = self.stats();

        println!("\n=== Partitioning Statistics ===");
        println!("Strategy: {:?}", self.strategy);
        println!("Partition count: {}", stats.partition_count);
        println!("Total items: {}", stats.total_items);
        println!("Average per partition: {:.1}", stats.avg_items);
        println!("Min/Max: {} / {}", stats.min_items, stats.max_items);
        println!("Imbalance ratio: {:.2}", stats.imbalance_ratio);
        println!("Standard deviation: {:.2}", stats.std_deviation);

        println!("\nDistribution:");
        for (i, count) in stats.items_per_partition.iter().enumerate() {
            let bar_len = (*count as f64 / stats.max_items.max(1) as f64 * 20.0) as usize;
            let bar = "█".repeat(bar_len);
            println!("  Partition {}: {:>5} {}", i, count, bar);
        }
    }
}

#[derive(Debug)]
pub struct PartitionStats {
    pub partition_count: usize,
    pub total_items: usize,
    pub items_per_partition: Vec<usize>,
    pub max_items: usize,
    pub min_items: usize,
    pub avg_items: f64,
    pub imbalance_ratio: f64,
    pub std_deviation: f64,
}

/// Trading trade for example
#[derive(Debug, Clone)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

fn main() {
    println!("=== Trading Data Partitioning ===\n");

    // Create partitioner with 4 partitions
    let partitioner: TradingDataPartitioner<Trade> =
        TradingDataPartitioner::new(4, PartitionStrategy::Hash);

    // Simulate trade insertions
    let symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "XRPUSDT",
                   "SOLUSDT", "DOTUSDT", "MATICUSDT", "LINKUSDT", "AVAXUSDT"];

    println!("Inserting trades...");
    for i in 0..1000 {
        let symbol = symbols[i % symbols.len()];
        let trade = Trade {
            id: format!("trade-{}", i),
            symbol: symbol.to_string(),
            price: 50000.0 + (i as f64 * 0.1),
            quantity: 0.1 + (i as f64 * 0.001),
            timestamp: 1700000000 + i as u64,
        };

        partitioner.insert(trade.id.clone(), trade);
    }

    partitioner.print_stats();

    // Test data retrieval
    println!("\nData retrieval test:");
    for i in [0, 100, 500, 999] {
        let key = format!("trade-{}", i);
        if let Some(trade) = partitioner.get(&key) {
            println!("  {} -> {} @ ${:.2}", key, trade.symbol, trade.price);
        }
    }

    // Compare strategies
    println!("\n=== Partitioning Strategy Comparison ===");

    for strategy in [PartitionStrategy::Hash, PartitionStrategy::Range,
                     PartitionStrategy::ConsistentHash] {
        let part: TradingDataPartitioner<Trade> =
            TradingDataPartitioner::new(4, strategy);

        for i in 0..1000 {
            let symbol = symbols[i % symbols.len()];
            let trade = Trade {
                id: format!("trade-{}", i),
                symbol: symbol.to_string(),
                price: 50000.0,
                quantity: 0.1,
                timestamp: 1700000000,
            };
            part.insert(trade.id.clone(), trade);
        }

        let stats = part.stats();
        println!("\n{:?}:", strategy);
        println!("  Imbalance: {:.2}, Std. dev.: {:.2}",
            stats.imbalance_ratio, stats.std_deviation);
    }
}
```

## Distributed Market Data Processing

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Market data tick
#[derive(Debug, Clone)]
pub struct MarketTick {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub timestamp: u64,
}

/// Data processing worker node
pub struct DataProcessingNode {
    pub id: String,
    pub assigned_symbols: Vec<String>,
    pub ticks_processed: AtomicU64,
    pub last_tick_time: AtomicU64,
    processor: Box<dyn Fn(&MarketTick) -> ProcessingResult + Send + Sync>,
}

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub symbol: String,
    pub signal: Option<TradeSignal>,
    pub latency_us: u64,
}

#[derive(Debug, Clone)]
pub enum TradeSignal {
    Buy { price: f64, size: f64 },
    Sell { price: f64, size: f64 },
}

impl DataProcessingNode {
    pub fn new<F>(id: &str, processor: F) -> Self
    where
        F: Fn(&MarketTick) -> ProcessingResult + Send + Sync + 'static,
    {
        DataProcessingNode {
            id: id.to_string(),
            assigned_symbols: Vec::new(),
            ticks_processed: AtomicU64::new(0),
            last_tick_time: AtomicU64::new(0),
            processor: Box::new(processor),
        }
    }

    pub fn assign_symbol(&mut self, symbol: String) {
        if !self.assigned_symbols.contains(&symbol) {
            self.assigned_symbols.push(symbol);
        }
    }

    pub fn process_tick(&self, tick: &MarketTick) -> ProcessingResult {
        self.ticks_processed.fetch_add(1, Ordering::Relaxed);
        self.last_tick_time.store(tick.timestamp, Ordering::Relaxed);
        (self.processor)(tick)
    }

    pub fn stats(&self) -> NodeStats {
        NodeStats {
            id: self.id.clone(),
            symbols_count: self.assigned_symbols.len(),
            ticks_processed: self.ticks_processed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
pub struct NodeStats {
    pub id: String,
    pub symbols_count: usize,
    pub ticks_processed: u64,
}

/// Distributed processing coordinator
pub struct DistributedProcessor {
    nodes: Vec<Arc<DataProcessingNode>>,
    symbol_to_node: HashMap<String, usize>,
    total_ticks: AtomicU64,
    start_time: Instant,
}

impl DistributedProcessor {
    pub fn new() -> Self {
        DistributedProcessor {
            nodes: Vec::new(),
            symbol_to_node: HashMap::new(),
            total_ticks: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn add_node(&mut self, node: DataProcessingNode) {
        let node_idx = self.nodes.len();
        let node = Arc::new(node);

        // Register node symbols
        for symbol in &node.assigned_symbols {
            self.symbol_to_node.insert(symbol.clone(), node_idx);
        }

        self.nodes.push(node);
    }

    /// Assign symbols to nodes evenly
    pub fn distribute_symbols(&mut self, symbols: &[String]) {
        if self.nodes.is_empty() {
            return;
        }

        for (i, symbol) in symbols.iter().enumerate() {
            let node_idx = i % self.nodes.len();
            self.symbol_to_node.insert(symbol.clone(), node_idx);

            println!("Symbol {} assigned to node {}", symbol, self.nodes[node_idx].id);
        }
    }

    /// Process tick on appropriate node
    pub fn process_tick(&self, tick: &MarketTick) -> Option<ProcessingResult> {
        self.total_ticks.fetch_add(1, Ordering::Relaxed);

        if let Some(&node_idx) = self.symbol_to_node.get(&tick.symbol) {
            let node = &self.nodes[node_idx];
            Some(node.process_tick(tick))
        } else {
            // Symbol not assigned, use first available node
            if !self.nodes.is_empty() {
                Some(self.nodes[0].process_tick(tick))
            } else {
                None
            }
        }
    }

    /// Get statistics for all nodes
    pub fn get_stats(&self) -> ClusterStats {
        let node_stats: Vec<NodeStats> = self.nodes.iter()
            .map(|n| n.stats())
            .collect();

        let total_ticks = self.total_ticks.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();
        let throughput = if elapsed.as_secs_f64() > 0.0 {
            total_ticks as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        ClusterStats {
            node_count: self.nodes.len(),
            total_symbols: self.symbol_to_node.len(),
            total_ticks_processed: total_ticks,
            throughput_per_second: throughput,
            uptime: elapsed,
            nodes: node_stats,
        }
    }

    pub fn print_stats(&self) {
        let stats = self.get_stats();

        println!("\n=== Distributed Processor Statistics ===");
        println!("Nodes: {}", stats.node_count);
        println!("Symbols: {}", stats.total_symbols);
        println!("Total ticks: {}", stats.total_ticks_processed);
        println!("Throughput: {:.0} ticks/sec", stats.throughput_per_second);
        println!("Uptime: {:.2}s", stats.uptime.as_secs_f64());

        println!("\nPer-node statistics:");
        for node in &stats.nodes {
            println!("  {} - Symbols: {}, Ticks: {}",
                node.id, node.symbols_count, node.ticks_processed);
        }
    }
}

#[derive(Debug)]
pub struct ClusterStats {
    pub node_count: usize,
    pub total_symbols: usize,
    pub total_ticks_processed: u64,
    pub throughput_per_second: f64,
    pub uptime: Duration,
    pub nodes: Vec<NodeStats>,
}

fn main() {
    println!("=== Distributed Market Data Processing ===\n");

    let mut processor = DistributedProcessor::new();

    // Create processing nodes with different logic
    let node1 = DataProcessingNode::new("crypto-processor", |tick| {
        let start = Instant::now();

        // Simple momentum logic
        let spread = tick.ask - tick.bid;
        let signal = if spread < 0.0001 * tick.bid {
            Some(TradeSignal::Buy {
                price: tick.ask,
                size: tick.bid_size.min(1.0)
            })
        } else {
            None
        };

        ProcessingResult {
            symbol: tick.symbol.clone(),
            signal,
            latency_us: start.elapsed().as_micros() as u64,
        }
    });

    let node2 = DataProcessingNode::new("forex-processor", |tick| {
        let start = Instant::now();

        // Forex logic
        let mid_price = (tick.bid + tick.ask) / 2.0;
        let signal = if tick.bid_size > tick.ask_size * 1.5 {
            Some(TradeSignal::Buy {
                price: tick.ask,
                size: 10000.0
            })
        } else if tick.ask_size > tick.bid_size * 1.5 {
            Some(TradeSignal::Sell {
                price: tick.bid,
                size: 10000.0
            })
        } else {
            None
        };

        ProcessingResult {
            symbol: tick.symbol.clone(),
            signal,
            latency_us: start.elapsed().as_micros() as u64,
        }
    });

    processor.add_node(node1);
    processor.add_node(node2);

    // Distribute symbols
    let symbols: Vec<String> = vec![
        "BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT",
        "EURUSD", "GBPUSD", "USDJPY", "AUDUSD"
    ].into_iter().map(String::from).collect();

    processor.distribute_symbols(&symbols);

    // Simulate tick processing
    println!("\nProcessing ticks...");
    for i in 0..1000 {
        let symbol = &symbols[i % symbols.len()];
        let tick = MarketTick {
            symbol: symbol.clone(),
            bid: 50000.0 + (i as f64 * 0.01),
            ask: 50001.0 + (i as f64 * 0.01),
            bid_size: 1.5 + (i as f64 % 10.0) * 0.1,
            ask_size: 1.2 + (i as f64 % 8.0) * 0.1,
            timestamp: 1700000000 + i as u64,
        };

        if let Some(result) = processor.process_tick(&tick) {
            if result.signal.is_some() && i < 10 {
                println!("Tick {} -> {:?}", tick.symbol, result.signal);
            }
        }
    }

    processor.print_stats();
}
```

## State Synchronization in Distributed Systems

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// State version for consistency control
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateVersion(pub u64);

impl StateVersion {
    pub fn new() -> Self {
        StateVersion(0)
    }

    pub fn increment(&self) -> Self {
        StateVersion(self.0 + 1)
    }
}

/// Trading portfolio state
#[derive(Debug, Clone)]
pub struct PortfolioState {
    pub positions: HashMap<String, Position>,
    pub cash_balance: f64,
    pub version: StateVersion,
    pub last_updated: u64,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
    pub unrealized_pnl: f64,
}

/// State change
#[derive(Debug, Clone)]
pub struct StateChange {
    pub change_id: u64,
    pub version: StateVersion,
    pub operation: StateOperation,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum StateOperation {
    UpdatePosition { symbol: String, quantity: f64, price: f64 },
    UpdateCash { amount: f64 },
    ClosePosition { symbol: String, price: f64 },
}

/// State replica on a node
pub struct StateReplica {
    pub node_id: String,
    pub state: RwLock<PortfolioState>,
    pub pending_changes: RwLock<Vec<StateChange>>,
    pub applied_version: AtomicU64,
}

impl StateReplica {
    pub fn new(node_id: &str) -> Self {
        StateReplica {
            node_id: node_id.to_string(),
            state: RwLock::new(PortfolioState {
                positions: HashMap::new(),
                cash_balance: 100_000.0,
                version: StateVersion::new(),
                last_updated: 0,
            }),
            pending_changes: RwLock::new(Vec::new()),
            applied_version: AtomicU64::new(0),
        }
    }

    /// Apply change to local state
    pub fn apply_change(&self, change: &StateChange) -> Result<(), &'static str> {
        let current_version = self.applied_version.load(Ordering::SeqCst);

        // Check version ordering
        if change.version.0 != current_version + 1 {
            return Err("Version gap detected, synchronization required");
        }

        let mut state = self.state.write().unwrap();

        match &change.operation {
            StateOperation::UpdatePosition { symbol, quantity, price } => {
                let position = state.positions.entry(symbol.clone()).or_insert(Position {
                    symbol: symbol.clone(),
                    quantity: 0.0,
                    avg_price: 0.0,
                    unrealized_pnl: 0.0,
                });

                // Recalculate average price
                let total_value = position.quantity * position.avg_price + quantity * price;
                let new_quantity = position.quantity + quantity;

                if new_quantity.abs() > 0.0001 {
                    position.avg_price = total_value / new_quantity;
                    position.quantity = new_quantity;
                } else {
                    state.positions.remove(symbol);
                }
            }
            StateOperation::UpdateCash { amount } => {
                state.cash_balance += amount;
            }
            StateOperation::ClosePosition { symbol, price } => {
                if let Some(position) = state.positions.remove(symbol) {
                    let pnl = (price - position.avg_price) * position.quantity;
                    state.cash_balance += position.quantity * price;
                    println!("  Position {} closed, P&L: ${:.2}", symbol, pnl);
                }
            }
        }

        state.version = change.version;
        state.last_updated = change.timestamp;
        self.applied_version.store(change.version.0, Ordering::SeqCst);

        Ok(())
    }

    /// Get current version
    pub fn current_version(&self) -> StateVersion {
        StateVersion(self.applied_version.load(Ordering::SeqCst))
    }

    /// Get state snapshot
    pub fn snapshot(&self) -> PortfolioState {
        self.state.read().unwrap().clone()
    }
}

/// State synchronization coordinator
pub struct StateCoordinator {
    replicas: Vec<Arc<StateReplica>>,
    change_log: RwLock<Vec<StateChange>>,
    next_change_id: AtomicU64,
    next_version: AtomicU64,
}

impl StateCoordinator {
    pub fn new() -> Self {
        StateCoordinator {
            replicas: Vec::new(),
            change_log: RwLock::new(Vec::new()),
            next_change_id: AtomicU64::new(1),
            next_version: AtomicU64::new(1),
        }
    }

    pub fn add_replica(&mut self, replica: Arc<StateReplica>) {
        println!("Added replica: {}", replica.node_id);
        self.replicas.push(replica);
    }

    /// Propose a state change
    pub fn propose_change(&self, operation: StateOperation) -> Result<StateVersion, &'static str> {
        let change_id = self.next_change_id.fetch_add(1, Ordering::SeqCst);
        let version = StateVersion(self.next_version.fetch_add(1, Ordering::SeqCst));

        let change = StateChange {
            change_id,
            version,
            operation,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Apply to all replicas
        let mut success_count = 0;
        for replica in &self.replicas {
            if replica.apply_change(&change).is_ok() {
                success_count += 1;
            }
        }

        // Require quorum (majority)
        let quorum = (self.replicas.len() / 2) + 1;
        if success_count >= quorum {
            self.change_log.write().unwrap().push(change);
            Ok(version)
        } else {
            Err("Failed to achieve quorum")
        }
    }

    /// Synchronize a lagging replica
    pub fn sync_replica(&self, replica: &StateReplica) -> Result<u64, &'static str> {
        let current_version = replica.current_version();
        let changes = self.change_log.read().unwrap();

        let mut applied = 0;
        for change in changes.iter() {
            if change.version.0 > current_version.0 {
                replica.apply_change(change)?;
                applied += 1;
            }
        }

        Ok(applied)
    }

    /// Check consistency of all replicas
    pub fn check_consistency(&self) -> ConsistencyReport {
        let versions: Vec<(String, StateVersion)> = self.replicas.iter()
            .map(|r| (r.node_id.clone(), r.current_version()))
            .collect();

        let max_version = versions.iter().map(|(_, v)| v.0).max().unwrap_or(0);
        let min_version = versions.iter().map(|(_, v)| v.0).min().unwrap_or(0);

        ConsistencyReport {
            replica_count: self.replicas.len(),
            versions: versions.clone(),
            is_consistent: max_version == min_version,
            version_lag: max_version - min_version,
        }
    }

    pub fn print_status(&self) {
        println!("\n=== State Synchronization Status ===");

        let report = self.check_consistency();
        println!("Replicas: {}", report.replica_count);
        println!("Consistency: {}", if report.is_consistent { "✓" } else { "✗" });
        println!("Version lag: {}", report.version_lag);

        println!("\nVersions per replica:");
        for (node_id, version) in &report.versions {
            println!("  {} -> v{}", node_id, version.0);
        }

        // Show state of first replica
        if let Some(replica) = self.replicas.first() {
            let state = replica.snapshot();
            println!("\nCurrent portfolio state:");
            println!("  Cash: ${:.2}", state.cash_balance);
            println!("  Positions:");
            for (symbol, pos) in &state.positions {
                println!("    {} - {} @ ${:.2}", symbol, pos.quantity, pos.avg_price);
            }
        }
    }
}

#[derive(Debug)]
pub struct ConsistencyReport {
    pub replica_count: usize,
    pub versions: Vec<(String, StateVersion)>,
    pub is_consistent: bool,
    pub version_lag: u64,
}

fn main() {
    println!("=== State Synchronization ===\n");

    let mut coordinator = StateCoordinator::new();

    // Create replicas
    let replica1 = Arc::new(StateReplica::new("node-us-east"));
    let replica2 = Arc::new(StateReplica::new("node-eu-west"));
    let replica3 = Arc::new(StateReplica::new("node-asia"));

    coordinator.add_replica(Arc::clone(&replica1));
    coordinator.add_replica(Arc::clone(&replica2));
    coordinator.add_replica(Arc::clone(&replica3));

    // Simulate trading operations
    println!("\nExecuting trading operations...");

    // Buy BTC
    coordinator.propose_change(StateOperation::UpdateCash { amount: -50000.0 }).unwrap();
    coordinator.propose_change(StateOperation::UpdatePosition {
        symbol: "BTCUSDT".to_string(),
        quantity: 1.0,
        price: 50000.0,
    }).unwrap();

    // Buy ETH
    coordinator.propose_change(StateOperation::UpdateCash { amount: -3000.0 }).unwrap();
    coordinator.propose_change(StateOperation::UpdatePosition {
        symbol: "ETHUSDT".to_string(),
        quantity: 1.0,
        price: 3000.0,
    }).unwrap();

    // Partial BTC sale
    coordinator.propose_change(StateOperation::UpdatePosition {
        symbol: "BTCUSDT".to_string(),
        quantity: -0.5,
        price: 52000.0,
    }).unwrap();
    coordinator.propose_change(StateOperation::UpdateCash { amount: 26000.0 }).unwrap();

    coordinator.print_status();

    // Check consistency
    let report = coordinator.check_consistency();
    println!("\nConsistency check:");
    if report.is_consistent {
        println!("✓ All replicas consistent at version v{}",
            report.versions.first().map(|(_, v)| v.0).unwrap_or(0));
    } else {
        println!("✗ Version divergence detected!");
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Load Balancing** | Distributing requests across worker nodes |
| **Partitioning** | Splitting data into parts for parallel processing |
| **Distributed Processing** | Processing data on multiple nodes simultaneously |
| **State Synchronization** | Maintaining data consistency across replicas |
| **Quorum** | Minimum number of nodes to confirm an operation |
| **Versioning** | Tracking changes to ensure ordering |
| **Consistent Hashing** | Data distribution with minimal redistribution |

## Practical Exercises

1. **Priority-Based Load Balancer**: Implement a system that:
   - Assigns priorities to different order types
   - Routes VIP clients to dedicated nodes
   - Handles urgent orders without queuing
   - Balances load based on instrument type

2. **Time-Based Partitioner**: Create a partitioning system:
   - Splits data by time windows (seconds, minutes, hours)
   - Supports hot and cold partitions
   - Archives old data automatically
   - Optimizes queries by time range

3. **Fault-Tolerant Cluster**: Build a processing cluster:
   - Detects node failures via heartbeat
   - Automatically redistributes load
   - Recovers nodes after failure
   - Logs all events for audit

4. **Geo-Distributed System**: Implement a system:
   - Replicates data between regions
   - Routes requests to nearest data center
   - Handles network partitions between regions
   - Resolves conflicts on data divergence

## Homework

1. **Complete Trading Cluster**: Build a system that:
   - Scales horizontally as load grows
   - Supports adding nodes without downtime
   - Balances load across exchanges
   - Ensures portfolio consistency
   - Monitors health of all components
   - Automatically recovers after failures

2. **Stream Processing Platform**: Create a platform:
   - Processes millions of ticks per second
   - Distributes computations across nodes
   - Aggregates results in real-time
   - Supports backpressure on overload
   - Scales on demand

3. **Distributed Order Book**: Implement:
   - Order book partitioning by symbol
   - Replication for fault tolerance
   - Fast order matching on each node
   - Synchronization between partitions
   - Atomic cross-partition operations

4. **Auto-Scaling Orchestrator**: Design a system:
   - Monitors cluster load metrics
   - Makes scaling decisions
   - Adds/removes nodes automatically
   - Forecasts load based on historical data
   - Optimizes infrastructure costs

## Navigation

[← Previous day](../357-disaster-recovery/en.md) | [Next day →](../359-kubernetes-orchestration/en.md)
