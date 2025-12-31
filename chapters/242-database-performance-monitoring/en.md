# Day 242: Database Performance Monitoring

## Trading Analogy

Imagine you're running a trading platform with millions of trades per day. Every millisecond of delay when querying the database is potentially lost profit or, worse, an incorrectly executed order. Database performance monitoring is like watching your trading terminal: you track latency (how fast queries execute), throughput (how many queries are processed per second), and resource utilization (is the system overloaded).

In algorithmic trading, the database stores:
- Price and candlestick history
- Trade log journal
- Portfolio and position state
- Trading strategy configurations
- Risk management metrics

Slow database = slow trading = losses.

## What is Database Performance Monitoring?

Database performance monitoring includes:

1. **Query execution time measurement** — how long each query takes
2. **Connection pool tracking** — are connections being used efficiently
3. **Cache monitoring** — what percentage of queries hit the cache
4. **Slow query analysis** — identifying problematic queries
5. **Resource metrics** — CPU, memory, disk, network

## Basic Metrics Structure

```rust
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Database query performance metrics
#[derive(Debug)]
pub struct QueryMetrics {
    /// Total number of executed queries
    pub total_queries: AtomicU64,
    /// Number of successful queries
    pub successful_queries: AtomicU64,
    /// Number of failed queries
    pub failed_queries: AtomicU64,
    /// Total execution time of all queries (in microseconds)
    pub total_duration_us: AtomicU64,
    /// Minimum query time (in microseconds)
    pub min_duration_us: AtomicU64,
    /// Maximum query time (in microseconds)
    pub max_duration_us: AtomicU64,
}

impl QueryMetrics {
    pub fn new() -> Self {
        QueryMetrics {
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            total_duration_us: AtomicU64::new(0),
            min_duration_us: AtomicU64::new(u64::MAX),
            max_duration_us: AtomicU64::new(0),
        }
    }

    /// Records a successful query metric
    pub fn record_success(&self, duration: Duration) {
        let duration_us = duration.as_micros() as u64;

        self.total_queries.fetch_add(1, Ordering::Relaxed);
        self.successful_queries.fetch_add(1, Ordering::Relaxed);
        self.total_duration_us.fetch_add(duration_us, Ordering::Relaxed);

        // Update minimum
        self.min_duration_us.fetch_min(duration_us, Ordering::Relaxed);
        // Update maximum
        self.max_duration_us.fetch_max(duration_us, Ordering::Relaxed);
    }

    /// Records a failed query metric
    pub fn record_failure(&self, duration: Duration) {
        let duration_us = duration.as_micros() as u64;

        self.total_queries.fetch_add(1, Ordering::Relaxed);
        self.failed_queries.fetch_add(1, Ordering::Relaxed);
        self.total_duration_us.fetch_add(duration_us, Ordering::Relaxed);
    }

    /// Returns average query time
    pub fn average_duration(&self) -> Duration {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            return Duration::ZERO;
        }
        let avg_us = self.total_duration_us.load(Ordering::Relaxed) / total;
        Duration::from_micros(avg_us)
    }

    /// Returns success rate percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            return 100.0;
        }
        let successful = self.successful_queries.load(Ordering::Relaxed);
        (successful as f64 / total as f64) * 100.0
    }
}

fn main() {
    let metrics = Arc::new(QueryMetrics::new());

    // Simulate database queries
    for i in 0..100 {
        let start = Instant::now();

        // Simulating DB query
        std::thread::sleep(Duration::from_micros(100 + (i % 50) * 10));

        let duration = start.elapsed();

        if i % 10 == 0 {
            metrics.record_failure(duration);
        } else {
            metrics.record_success(duration);
        }
    }

    println!("=== DB Performance Metrics ===");
    println!("Total queries: {}", metrics.total_queries.load(Ordering::Relaxed));
    println!("Successful: {}", metrics.successful_queries.load(Ordering::Relaxed));
    println!("Failed: {}", metrics.failed_queries.load(Ordering::Relaxed));
    println!("Average time: {:?}", metrics.average_duration());
    println!("Success rate: {:.2}%", metrics.success_rate());
}
```

## Connection Pool Monitoring

In trading systems, it's important to efficiently manage database connections:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Connection pool metrics
#[derive(Debug)]
pub struct ConnectionPoolMetrics {
    /// Current number of active connections
    pub active_connections: AtomicUsize,
    /// Maximum number of connections
    pub max_connections: usize,
    /// Number of threads waiting for a connection
    pub waiting_count: AtomicUsize,
    /// Total number of acquired connections
    pub connections_acquired: AtomicU64,
    /// Total wait time for connections (microseconds)
    pub total_wait_time_us: AtomicU64,
    /// Number of connection timeouts
    pub connection_timeouts: AtomicU64,
}

impl ConnectionPoolMetrics {
    pub fn new(max_connections: usize) -> Self {
        ConnectionPoolMetrics {
            active_connections: AtomicUsize::new(0),
            max_connections,
            waiting_count: AtomicUsize::new(0),
            connections_acquired: AtomicU64::new(0),
            total_wait_time_us: AtomicU64::new(0),
            connection_timeouts: AtomicU64::new(0),
        }
    }

    /// Pool utilization percentage
    pub fn utilization(&self) -> f64 {
        let active = self.active_connections.load(Ordering::Relaxed);
        (active as f64 / self.max_connections as f64) * 100.0
    }

    /// Average connection wait time
    pub fn average_wait_time(&self) -> Duration {
        let acquired = self.connections_acquired.load(Ordering::Relaxed);
        if acquired == 0 {
            return Duration::ZERO;
        }
        let avg_us = self.total_wait_time_us.load(Ordering::Relaxed) / acquired;
        Duration::from_micros(avg_us)
    }
}

/// Trait for monitored connection pool
pub trait MonitoredPool {
    fn acquire(&self) -> Result<Connection, PoolError>;
    fn release(&self, conn: Connection);
    fn metrics(&self) -> &ConnectionPoolMetrics;
}

#[derive(Debug)]
pub struct Connection {
    id: u64,
    created_at: Instant,
}

#[derive(Debug)]
pub enum PoolError {
    Timeout,
    Exhausted,
}

/// Simulated connection pool for trading system
pub struct TradingDatabasePool {
    metrics: ConnectionPoolMetrics,
    next_conn_id: AtomicU64,
}

impl TradingDatabasePool {
    pub fn new(max_connections: usize) -> Self {
        TradingDatabasePool {
            metrics: ConnectionPoolMetrics::new(max_connections),
            next_conn_id: AtomicU64::new(0),
        }
    }

    pub fn acquire(&self) -> Result<Connection, PoolError> {
        let start = Instant::now();

        // Increment waiting counter
        self.metrics.waiting_count.fetch_add(1, Ordering::Relaxed);

        // Check if connections are available
        let current = self.metrics.active_connections.load(Ordering::Relaxed);
        if current >= self.metrics.max_connections {
            self.metrics.waiting_count.fetch_sub(1, Ordering::Relaxed);
            self.metrics.connection_timeouts.fetch_add(1, Ordering::Relaxed);
            return Err(PoolError::Exhausted);
        }

        // Acquire connection
        self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);
        self.metrics.waiting_count.fetch_sub(1, Ordering::Relaxed);
        self.metrics.connections_acquired.fetch_add(1, Ordering::Relaxed);

        let wait_time = start.elapsed();
        self.metrics.total_wait_time_us.fetch_add(
            wait_time.as_micros() as u64,
            Ordering::Relaxed
        );

        let id = self.next_conn_id.fetch_add(1, Ordering::Relaxed);

        Ok(Connection {
            id,
            created_at: Instant::now(),
        })
    }

    pub fn release(&self, _conn: Connection) {
        self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn metrics(&self) -> &ConnectionPoolMetrics {
        &self.metrics
    }
}

use std::sync::atomic::AtomicU64;

fn main() {
    let pool = Arc::new(TradingDatabasePool::new(10));

    println!("=== Trading Load Simulation on Connection Pool ===\n");

    // Simulate trading operations
    let mut handles = vec![];

    for trader_id in 0..5 {
        let pool = Arc::clone(&pool);
        let handle = std::thread::spawn(move || {
            for order_id in 0..20 {
                match pool.acquire() {
                    Ok(conn) => {
                        // Simulate DB query for order processing
                        std::thread::sleep(Duration::from_millis(5));
                        println!(
                            "Trader {}: order {} processed (connection #{})",
                            trader_id, order_id, conn.id
                        );
                        pool.release(conn);
                    }
                    Err(PoolError::Exhausted) => {
                        println!(
                            "Trader {}: order {} delayed - pool exhausted",
                            trader_id, order_id
                        );
                    }
                    Err(PoolError::Timeout) => {
                        println!(
                            "Trader {}: order {} timeout",
                            trader_id, order_id
                        );
                    }
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let metrics = pool.metrics();
    println!("\n=== Final Pool Metrics ===");
    println!("Connections acquired: {}", metrics.connections_acquired.load(Ordering::Relaxed));
    println!("Timeouts: {}", metrics.connection_timeouts.load(Ordering::Relaxed));
    println!("Average wait time: {:?}", metrics.average_wait_time());
    println!("Current utilization: {:.1}%", metrics.utilization());
}
```

## Slow Query Tracking

For trading systems, it's critical to identify slow queries:

```rust
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

/// Information about a slow query
#[derive(Debug, Clone)]
pub struct SlowQuery {
    pub query: String,
    pub duration: Duration,
    pub timestamp: SystemTime,
    pub context: String, // e.g., "order_execution" or "price_fetch"
}

/// Slow query tracker for trading system
pub struct SlowQueryTracker {
    /// Threshold for slow query
    threshold: Duration,
    /// Recent slow queries (bounded buffer)
    slow_queries: Mutex<VecDeque<SlowQuery>>,
    /// Maximum number of stored queries
    max_queries: usize,
}

impl SlowQueryTracker {
    pub fn new(threshold: Duration, max_queries: usize) -> Self {
        SlowQueryTracker {
            threshold,
            slow_queries: Mutex::new(VecDeque::with_capacity(max_queries)),
            max_queries,
        }
    }

    /// Records a query if it's slow
    pub fn record(&self, query: &str, duration: Duration, context: &str) {
        if duration >= self.threshold {
            let slow_query = SlowQuery {
                query: query.to_string(),
                duration,
                timestamp: SystemTime::now(),
                context: context.to_string(),
            };

            let mut queries = self.slow_queries.lock().unwrap();

            if queries.len() >= self.max_queries {
                queries.pop_front();
            }

            queries.push_back(slow_query);
        }
    }

    /// Returns all slow queries
    pub fn get_slow_queries(&self) -> Vec<SlowQuery> {
        self.slow_queries.lock().unwrap().iter().cloned().collect()
    }

    /// Returns the slowest query
    pub fn get_slowest(&self) -> Option<SlowQuery> {
        self.slow_queries
            .lock()
            .unwrap()
            .iter()
            .max_by_key(|q| q.duration)
            .cloned()
    }

    /// Returns the count of slow queries
    pub fn count(&self) -> usize {
        self.slow_queries.lock().unwrap().len()
    }
}

/// Monitoring manager for trading DB
pub struct TradingDbMonitor {
    slow_query_tracker: SlowQueryTracker,
}

impl TradingDbMonitor {
    pub fn new() -> Self {
        TradingDbMonitor {
            // 10ms threshold for trading systems — already slow!
            slow_query_tracker: SlowQueryTracker::new(
                Duration::from_millis(10),
                100
            ),
        }
    }

    /// Executes a query with monitoring
    pub fn execute_query<F, T>(&self, query: &str, context: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        self.slow_query_tracker.record(query, duration, context);

        result
    }

    pub fn report(&self) {
        println!("\n=== Slow Query Report ===");
        println!("Total slow queries: {}", self.slow_query_tracker.count());

        if let Some(slowest) = self.slow_query_tracker.get_slowest() {
            println!("\nSlowest query:");
            println!("  Context: {}", slowest.context);
            println!("  Query: {}", slowest.query);
            println!("  Duration: {:?}", slowest.duration);
        }

        println!("\nRecent slow queries:");
        for sq in self.slow_query_tracker.get_slow_queries().iter().take(5) {
            println!(
                "  [{:?}] {} - {:?}",
                sq.context, sq.query, sq.duration
            );
        }
    }
}

fn main() {
    let monitor = TradingDbMonitor::new();

    // Simulate trading queries
    let trading_queries = vec![
        ("SELECT * FROM prices WHERE symbol = 'BTCUSDT' ORDER BY time DESC LIMIT 1", "price_fetch", 5),
        ("INSERT INTO orders (symbol, side, price, qty) VALUES ('BTCUSDT', 'BUY', 42000, 0.1)", "order_insert", 15),
        ("SELECT * FROM positions WHERE portfolio_id = 1", "position_check", 8),
        ("UPDATE balances SET amount = amount - 4200 WHERE currency = 'USDT'", "balance_update", 25),
        ("SELECT * FROM candles WHERE symbol = 'BTCUSDT' AND timeframe = '1h' ORDER BY time DESC LIMIT 100", "candle_fetch", 50),
        ("SELECT AVG(price) FROM trades WHERE symbol = 'BTCUSDT' AND time > NOW() - INTERVAL '1 hour'", "vwap_calc", 35),
        ("INSERT INTO trade_log (order_id, fill_price, qty, fee) VALUES (12345, 42001.5, 0.1, 0.42)", "trade_log", 3),
    ];

    println!("=== Trading Query Simulation ===\n");

    for (query, context, delay_ms) in trading_queries {
        monitor.execute_query(query, context, || {
            // Simulate query execution
            std::thread::sleep(Duration::from_millis(delay_ms));
            println!("Executed: {} ({:?})", context, Duration::from_millis(delay_ms));
        });
    }

    monitor.report();
}
```

## Metrics Aggregation by Query Type

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Statistics for a specific query type
#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    pub count: u64,
    pub total_duration_us: u64,
    pub min_duration_us: u64,
    pub max_duration_us: u64,
    pub errors: u64,
}

impl QueryStats {
    pub fn new() -> Self {
        QueryStats {
            count: 0,
            total_duration_us: 0,
            min_duration_us: u64::MAX,
            max_duration_us: 0,
            errors: 0,
        }
    }

    pub fn record(&mut self, duration: Duration, success: bool) {
        let duration_us = duration.as_micros() as u64;

        self.count += 1;
        self.total_duration_us += duration_us;
        self.min_duration_us = self.min_duration_us.min(duration_us);
        self.max_duration_us = self.max_duration_us.max(duration_us);

        if !success {
            self.errors += 1;
        }
    }

    pub fn average_us(&self) -> u64 {
        if self.count == 0 { 0 } else { self.total_duration_us / self.count }
    }

    pub fn error_rate(&self) -> f64 {
        if self.count == 0 { 0.0 } else { (self.errors as f64 / self.count as f64) * 100.0 }
    }
}

/// Query categories in a trading system
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum QueryCategory {
    PriceFetch,      // Fetching quotes
    OrderInsert,     // Creating orders
    OrderUpdate,     // Updating orders
    TradeLog,        // Logging trades
    BalanceCheck,    // Checking balances
    PositionQuery,   // Position queries
    HistoricalData,  // Historical data
    RiskMetrics,     // Risk metrics
}

/// Metrics aggregator by category
pub struct QueryMetricsAggregator {
    stats: Mutex<HashMap<QueryCategory, QueryStats>>,
}

impl QueryMetricsAggregator {
    pub fn new() -> Self {
        QueryMetricsAggregator {
            stats: Mutex::new(HashMap::new()),
        }
    }

    pub fn record(&self, category: QueryCategory, duration: Duration, success: bool) {
        let mut stats = self.stats.lock().unwrap();
        stats.entry(category)
            .or_insert_with(QueryStats::new)
            .record(duration, success);
    }

    pub fn get_stats(&self, category: &QueryCategory) -> Option<QueryStats> {
        self.stats.lock().unwrap().get(category).cloned()
    }

    pub fn report(&self) {
        let stats = self.stats.lock().unwrap();

        println!("\n{:=<70}", "");
        println!("{:^70}", "TRADING DB PERFORMANCE REPORT");
        println!("{:=<70}", "");
        println!(
            "{:<20} {:>10} {:>12} {:>12} {:>10}",
            "Category", "Queries", "Avg (us)", "Max (us)", "Error %"
        );
        println!("{:-<70}", "");

        for (category, stat) in stats.iter() {
            println!(
                "{:<20} {:>10} {:>12} {:>12} {:>10.2}",
                format!("{:?}", category),
                stat.count,
                stat.average_us(),
                stat.max_duration_us,
                stat.error_rate()
            );
        }
        println!("{:=<70}", "");
    }
}

fn main() {
    let aggregator = Arc::new(QueryMetricsAggregator::new());

    // Simulate a trading day
    let test_data = vec![
        (QueryCategory::PriceFetch, 100, 2, false),
        (QueryCategory::PriceFetch, 150, 1, false),
        (QueryCategory::PriceFetch, 120, 3, false),
        (QueryCategory::OrderInsert, 5000, 15, false),
        (QueryCategory::OrderInsert, 4500, 12, false),
        (QueryCategory::OrderInsert, 6000, 20, true),
        (QueryCategory::BalanceCheck, 200, 5, false),
        (QueryCategory::BalanceCheck, 180, 4, false),
        (QueryCategory::TradeLog, 3000, 8, false),
        (QueryCategory::TradeLog, 2500, 6, false),
        (QueryCategory::HistoricalData, 50000, 150, false),
        (QueryCategory::HistoricalData, 45000, 120, false),
        (QueryCategory::RiskMetrics, 10000, 35, false),
        (QueryCategory::PositionQuery, 500, 8, false),
    ];

    println!("=== Trading Operations Simulation ===\n");

    for (category, duration_us, _expected_ms, is_error) in test_data {
        let duration = Duration::from_micros(duration_us);
        aggregator.record(category.clone(), duration, !is_error);

        if is_error {
            println!("ERROR: {:?}", category);
        }
    }

    aggregator.report();
}
```

## Real-Time Monitoring

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Real-time metrics for trading system
pub struct RealTimeMetrics {
    // Counters for current second
    queries_current_second: AtomicU64,
    // Counters for previous second (for QPS calculation)
    queries_per_second: AtomicU64,
    // Total latency for current second
    latency_sum_us: AtomicU64,
    // Flag to stop background thread
    running: AtomicBool,
}

impl RealTimeMetrics {
    pub fn new() -> Arc<Self> {
        let metrics = Arc::new(RealTimeMetrics {
            queries_current_second: AtomicU64::new(0),
            queries_per_second: AtomicU64::new(0),
            latency_sum_us: AtomicU64::new(0),
            running: AtomicBool::new(true),
        });

        // Start background thread to update QPS
        let metrics_clone = Arc::clone(&metrics);
        thread::spawn(move || {
            while metrics_clone.running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(1));

                // Move current second counter to QPS
                let current = metrics_clone.queries_current_second.swap(0, Ordering::Relaxed);
                metrics_clone.queries_per_second.store(current, Ordering::Relaxed);

                // Reset latency
                metrics_clone.latency_sum_us.store(0, Ordering::Relaxed);
            }
        });

        metrics
    }

    pub fn record_query(&self, duration: Duration) {
        self.queries_current_second.fetch_add(1, Ordering::Relaxed);
        self.latency_sum_us.fetch_add(
            duration.as_micros() as u64,
            Ordering::Relaxed
        );
    }

    pub fn get_qps(&self) -> u64 {
        self.queries_per_second.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Trading DB monitoring dashboard
pub struct TradingDbDashboard {
    metrics: Arc<RealTimeMetrics>,
    start_time: Instant,
}

impl TradingDbDashboard {
    pub fn new() -> Self {
        TradingDbDashboard {
            metrics: RealTimeMetrics::new(),
            start_time: Instant::now(),
        }
    }

    pub fn record(&self, duration: Duration) {
        self.metrics.record_query(duration);
    }

    pub fn display(&self) {
        let uptime = self.start_time.elapsed();
        let qps = self.metrics.get_qps();

        println!("\n+------------------------------------------+");
        println!("|       TRADING DB MONITORING              |");
        println!("+------------------------------------------+");
        println!("| Uptime: {:>10.1}s                      |", uptime.as_secs_f64());
        println!("| QPS:    {:>10}                        |", qps);
        println!("| Status: {:>10}                        |",
            if qps > 100 { "HIGH LOAD" }
            else if qps > 10 { "NORMAL" }
            else { "LOW" }
        );
        println!("+------------------------------------------+");
    }

    pub fn stop(&self) {
        self.metrics.stop();
    }
}

fn main() {
    let dashboard = Arc::new(TradingDbDashboard::new());

    println!("=== Starting Trading DB Monitoring ===");

    // Simulate load
    let dashboard_clone = Arc::clone(&dashboard);
    let load_handle = thread::spawn(move || {
        for i in 0..500 {
            let duration = Duration::from_micros(100 + (i % 100) * 10);
            dashboard_clone.record(duration);
            thread::sleep(Duration::from_millis(5));
        }
    });

    // Display dashboard every second
    for _ in 0..5 {
        thread::sleep(Duration::from_secs(1));
        dashboard.display();
    }

    load_handle.join().unwrap();
    dashboard.stop();

    println!("\nMonitoring stopped.");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| QueryMetrics | Structure for collecting query execution metrics |
| ConnectionPoolMetrics | Monitoring connection pool utilization |
| SlowQueryTracker | Identifying and logging slow queries |
| QueryCategory | Categorizing queries for aggregation |
| RealTimeMetrics | Real-time metrics (QPS) |
| AtomicU64 | Atomic counters for multithreaded access |

## Exercises

1. **Extended Metrics**: Add percentile calculations (p50, p95, p99) to `QueryMetrics` for query execution time. Use a histogram to store the distribution.

2. **Alerts**: Implement an alert system that sends notifications when:
   - QPS drops below a certain threshold
   - Error rate exceeds 5%
   - Average query time exceeds 100ms

3. **Metrics Export**: Create a function to export metrics in Prometheus format (text metrics format).

4. **Visualization**: Implement a simple ASCII visualization of database load as a graph in the terminal.

## Homework

1. **Complete Monitoring System**: Create a comprehensive database monitoring system for a trading platform that includes:
   - All metric types from the lesson
   - Automatic anomaly detection
   - Scheduled reports
   - Metrics history for the last hour

2. **Load Simulator**: Write a program that simulates various trading load patterns:
   - Market open (activity spike)
   - Normal trading
   - High volatility (many orders)
   - Market close

3. **Query Optimizer**: Based on collected metrics, create an analyzer that:
   - Groups similar slow queries
   - Suggests index creation
   - Identifies queries that are candidates for caching

4. **Multithreaded Stress Test**: Implement a connection pool stress test with varying thread counts and analyze how metrics change as load increases.

## Navigation

[← Previous day](../241-database-connection-pools/en.md) | [Next day →](../243-database-query-optimization/en.md)
