# Day 229: Connection Pool — Database Connection Pooling

## Trading Analogy

Imagine you're managing a trading desk with 100 traders. Each trader needs a direct phone line to the exchange to send orders. But renting a dedicated line is expensive, and establishing a connection takes 30 seconds.

**Bad approach**: Every time a trader needs to send an order, they call the exchange, wait 30 seconds for connection, send the order in 1 second, and hang up. With 1000 orders per day, that's 8+ hours just waiting for connections!

**Good approach**: You rent 10 permanent lines. When a trader needs to send an order, they grab a free line from the "pool", send the order, and return the line to the pool. The next trader can use the same line instantly!

This is a **Connection Pool** — a set of pre-established database connections that an application reuses instead of creating a new connection for each request.

## Why Do We Need Connection Pooling?

In high-frequency trading systems, every millisecond counts:

| Operation | Time without pool | Time with pool |
|-----------|-------------------|----------------|
| TCP connection | 1-5 ms | 0 ms |
| TLS handshake | 10-50 ms | 0 ms |
| Database authentication | 5-20 ms | 0 ms |
| Query execution | 1-10 ms | 1-10 ms |
| **Total** | **17-85 ms** | **1-10 ms** |

With 10,000 requests per second, pooling saves up to **750 seconds** of CPU time every second!

## Core Concepts

### Connection Pool Parameters

```rust
use std::time::Duration;

struct PoolConfig {
    // Minimum number of connections (always kept open)
    min_connections: u32,

    // Maximum number of connections
    max_connections: u32,

    // Timeout waiting for a free connection
    connection_timeout: Duration,

    // Connection lifetime (to protect against memory leaks)
    max_lifetime: Duration,

    // Idle time before closing excess connection
    idle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            min_connections: 5,
            max_connections: 20,
            connection_timeout: Duration::from_secs(30),
            max_lifetime: Duration::from_secs(1800), // 30 minutes
            idle_timeout: Duration::from_secs(600),  // 10 minutes
        }
    }
}
```

## Example with SQLx — A Popular Async Driver

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::time::Duration;

#[derive(Debug, Clone)]
struct Trade {
    id: i64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

async fn create_trading_pool() -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .min_connections(5)           // Minimum 5 connections
        .max_connections(50)          // Maximum 50 connections
        .acquire_timeout(Duration::from_secs(3))  // Acquire timeout
        .idle_timeout(Duration::from_secs(600))   // Close idle connections
        .max_lifetime(Duration::from_secs(1800))  // Maximum lifetime
        .connect("postgres://trader:secret@localhost/trading_db")
        .await?;

    println!("Connection pool created: min=5, max=50");
    Ok(pool)
}

async fn record_trade(pool: &PgPool, trade: &Trade) -> Result<i64, sqlx::Error> {
    // Connection is automatically acquired from pool
    let row = sqlx::query(
        r#"
        INSERT INTO trades (symbol, price, quantity, side, timestamp)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
    )
    .bind(&trade.symbol)
    .bind(trade.price)
    .bind(trade.quantity)
    .bind(&trade.side)
    .bind(trade.timestamp)
    .fetch_one(pool) // Connection returned to pool after execution
    .await?;

    Ok(row.get("id"))
}

async fn get_recent_trades(pool: &PgPool, symbol: &str, limit: i32) -> Result<Vec<Trade>, sqlx::Error> {
    let trades = sqlx::query_as!(
        Trade,
        r#"
        SELECT id, symbol, price, quantity, side, timestamp
        FROM trades
        WHERE symbol = $1
        ORDER BY timestamp DESC
        LIMIT $2
        "#,
        symbol,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(trades)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_trading_pool().await?;

    // Simulating high load — 1000 parallel requests
    let mut handles = vec![];

    for i in 0..1000 {
        let pool = pool.clone(); // Cloning pool is cheap (it's an Arc)

        handles.push(tokio::spawn(async move {
            let trade = Trade {
                id: 0,
                symbol: "BTC/USD".to_string(),
                price: 42000.0 + (i as f64 * 0.1),
                quantity: 0.01,
                side: if i % 2 == 0 { "buy" } else { "sell" }.to_string(),
                timestamp: chrono::Utc::now(),
            };

            record_trade(&pool, &trade).await
        }));
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.await?.is_ok() {
            success_count += 1;
        }
    }

    println!("Successfully recorded {} out of 1000 trades", success_count);

    // Pool statistics
    println!("Pool size: {}", pool.size());
    println!("Idle connections: {}", pool.num_idle());

    Ok(())
}
```

## Implementing Your Own Simple Pool

To understand the mechanics, let's create a simplified pool:

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// Simulated database connection
struct DatabaseConnection {
    id: u32,
    created_at: Instant,
}

impl DatabaseConnection {
    fn new(id: u32) -> Self {
        // Simulate connection establishment delay
        std::thread::sleep(Duration::from_millis(100));
        println!("Connection #{} created", id);

        DatabaseConnection {
            id,
            created_at: Instant::now(),
        }
    }

    fn execute(&self, query: &str) -> Result<String, String> {
        // Simulate query execution
        std::thread::sleep(Duration::from_millis(10));
        Ok(format!("Connection #{}: executed query '{}'", self.id, query))
    }

    fn is_healthy(&self) -> bool {
        // Check if connection hasn't expired
        self.created_at.elapsed() < Duration::from_secs(300)
    }
}

struct ConnectionPool {
    connections: Mutex<VecDeque<DatabaseConnection>>,
    available: Condvar,
    max_size: u32,
    current_size: Mutex<u32>,
    next_id: Mutex<u32>,
}

impl ConnectionPool {
    fn new(initial_size: u32, max_size: u32) -> Arc<Self> {
        let pool = Arc::new(ConnectionPool {
            connections: Mutex::new(VecDeque::new()),
            available: Condvar::new(),
            max_size,
            current_size: Mutex::new(0),
            next_id: Mutex::new(0),
        });

        // Create initial connections
        for _ in 0..initial_size {
            let conn = pool.create_connection();
            pool.connections.lock().unwrap().push_back(conn);
        }

        pool
    }

    fn create_connection(&self) -> DatabaseConnection {
        let mut next_id = self.next_id.lock().unwrap();
        let mut current_size = self.current_size.lock().unwrap();

        *next_id += 1;
        *current_size += 1;

        DatabaseConnection::new(*next_id)
    }

    fn get(&self, timeout: Duration) -> Option<PooledConnection> {
        let start = Instant::now();
        let mut connections = self.connections.lock().unwrap();

        loop {
            // Try to get an existing connection
            if let Some(conn) = connections.pop_front() {
                if conn.is_healthy() {
                    return Some(PooledConnection {
                        connection: Some(conn),
                        pool: self,
                    });
                }
                // Unhealthy connection — decrease counter and try next
                *self.current_size.lock().unwrap() -= 1;
                continue;
            }

            // Can we create a new connection?
            let current = *self.current_size.lock().unwrap();
            if current < self.max_size {
                drop(connections); // Release lock before long operation
                let conn = self.create_connection();
                return Some(PooledConnection {
                    connection: Some(conn),
                    pool: self,
                });
            }

            // Wait for a connection to be released
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return None; // Timeout
            }

            let (guard, timeout_result) = self.available
                .wait_timeout(connections, remaining)
                .unwrap();
            connections = guard;

            if timeout_result.timed_out() {
                return None;
            }
        }
    }

    fn return_connection(&self, conn: DatabaseConnection) {
        if conn.is_healthy() {
            self.connections.lock().unwrap().push_back(conn);
            self.available.notify_one();
        } else {
            *self.current_size.lock().unwrap() -= 1;
        }
    }

    fn stats(&self) -> (u32, usize) {
        let current = *self.current_size.lock().unwrap();
        let available = self.connections.lock().unwrap().len();
        (current, available)
    }
}

// RAII wrapper for automatic connection return
struct PooledConnection<'a> {
    connection: Option<DatabaseConnection>,
    pool: &'a ConnectionPool,
}

impl<'a> PooledConnection<'a> {
    fn execute(&self, query: &str) -> Result<String, String> {
        self.connection.as_ref().unwrap().execute(query)
    }
}

impl<'a> Drop for PooledConnection<'a> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            self.pool.return_connection(conn);
        }
    }
}

fn main() {
    let pool = ConnectionPool::new(3, 10);

    println!("\n=== Initial pool state ===");
    let (total, available) = pool.stats();
    println!("Total connections: {}, available: {}", total, available);

    // Simulate trading requests
    let pool = Arc::new(pool);
    let mut handles = vec![];

    for i in 0..20 {
        let pool = Arc::clone(&pool);

        handles.push(std::thread::spawn(move || {
            let trade_query = format!(
                "INSERT INTO trades (symbol, price) VALUES ('BTC', {})",
                42000.0 + i as f64
            );

            match pool.get(Duration::from_secs(5)) {
                Some(conn) => {
                    let result = conn.execute(&trade_query);
                    println!("Thread {}: {:?}", i, result);
                    // Connection automatically returned to pool on drop
                }
                None => {
                    println!("Thread {}: connection timeout!", i);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\n=== Final pool state ===");
    let (total, available) = Arc::try_unwrap(pool)
        .map(|p| p.stats())
        .unwrap_or((0, 0));
    println!("Total connections: {}, available: {}", total, available);
}
```

## Connection Pool in a Trading System

```rust
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

// Configuration for different data types
struct TradingPoolManager {
    // Pool for recording trades — high throughput
    trades_pool: PgPool,

    // Pool for reading market data — many readers
    market_data_pool: PgPool,

    // Pool for risk management — critical operations
    risk_pool: PgPool,
}

impl TradingPoolManager {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Pool for trades — optimized for writes
        let trades_pool = PgPoolOptions::new()
            .min_connections(10)
            .max_connections(100)
            .acquire_timeout(Duration::from_millis(500))
            .connect(database_url)
            .await?;

        // Pool for market data — more connections for reading
        let market_data_pool = PgPoolOptions::new()
            .min_connections(20)
            .max_connections(200)
            .acquire_timeout(Duration::from_millis(100))
            .connect(database_url)
            .await?;

        // Pool for risk management — fewer but more reliable
        let risk_pool = PgPoolOptions::new()
            .min_connections(5)
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(2)) // Longer timeout for critical ops
            .connect(database_url)
            .await?;

        Ok(TradingPoolManager {
            trades_pool,
            market_data_pool,
            risk_pool,
        })
    }

    // Fast trade recording
    async fn record_trade(&self, symbol: &str, price: f64, qty: f64) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "INSERT INTO trades (symbol, price, quantity) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(symbol)
        .bind(price)
        .bind(qty)
        .fetch_one(&self.trades_pool)
        .await
    }

    // Fast market data reading
    async fn get_latest_price(&self, symbol: &str) -> Result<f64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT price FROM market_data WHERE symbol = $1 ORDER BY timestamp DESC LIMIT 1"
        )
        .bind(symbol)
        .fetch_one(&self.market_data_pool)
        .await
    }

    // Critical limit checks
    async fn check_position_limit(&self, symbol: &str, new_qty: f64) -> Result<bool, sqlx::Error> {
        let current: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(quantity), 0) FROM positions WHERE symbol = $1"
        )
        .bind(symbol)
        .fetch_one(&self.risk_pool)
        .await?;

        let limit: f64 = sqlx::query_scalar(
            "SELECT max_position FROM risk_limits WHERE symbol = $1"
        )
        .bind(symbol)
        .fetch_one(&self.risk_pool)
        .await?;

        Ok(current + new_qty <= limit)
    }

    // Statistics for all pools
    fn print_stats(&self) {
        println!("=== Connection Pool Statistics ===");
        println!("Trades Pool: size={}, idle={}",
            self.trades_pool.size(),
            self.trades_pool.num_idle());
        println!("Market Data Pool: size={}, idle={}",
            self.market_data_pool.size(),
            self.market_data_pool.num_idle());
        println!("Risk Pool: size={}, idle={}",
            self.risk_pool.size(),
            self.risk_pool.num_idle());
    }
}
```

## Pool Health Monitoring

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct PoolMetrics {
    connections_acquired: AtomicU64,
    connections_released: AtomicU64,
    connection_timeouts: AtomicU64,
    total_wait_time_ms: AtomicU64,
    queries_executed: AtomicU64,
}

impl PoolMetrics {
    fn new() -> Self {
        PoolMetrics {
            connections_acquired: AtomicU64::new(0),
            connections_released: AtomicU64::new(0),
            connection_timeouts: AtomicU64::new(0),
            total_wait_time_ms: AtomicU64::new(0),
            queries_executed: AtomicU64::new(0),
        }
    }

    fn record_acquire(&self, wait_time_ms: u64) {
        self.connections_acquired.fetch_add(1, Ordering::Relaxed);
        self.total_wait_time_ms.fetch_add(wait_time_ms, Ordering::Relaxed);
    }

    fn record_release(&self) {
        self.connections_released.fetch_add(1, Ordering::Relaxed);
    }

    fn record_timeout(&self) {
        self.connection_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    fn record_query(&self) {
        self.queries_executed.fetch_add(1, Ordering::Relaxed);
    }

    fn report(&self) -> PoolHealthReport {
        let acquired = self.connections_acquired.load(Ordering::Relaxed);
        let total_wait = self.total_wait_time_ms.load(Ordering::Relaxed);

        PoolHealthReport {
            total_acquired: acquired,
            total_released: self.connections_released.load(Ordering::Relaxed),
            total_timeouts: self.connection_timeouts.load(Ordering::Relaxed),
            avg_wait_time_ms: if acquired > 0 { total_wait / acquired } else { 0 },
            total_queries: self.queries_executed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
struct PoolHealthReport {
    total_acquired: u64,
    total_released: u64,
    total_timeouts: u64,
    avg_wait_time_ms: u64,
    total_queries: u64,
}

impl PoolHealthReport {
    fn is_healthy(&self) -> bool {
        // Alert if more than 1% of requests timed out
        let timeout_rate = if self.total_acquired > 0 {
            (self.total_timeouts as f64) / (self.total_acquired as f64)
        } else {
            0.0
        };

        // Alert if average wait time > 100 ms
        timeout_rate < 0.01 && self.avg_wait_time_ms < 100
    }

    fn print(&self) {
        println!("=== Pool Health Report ===");
        println!("Connections acquired: {}", self.total_acquired);
        println!("Connections released: {}", self.total_released);
        println!("Timeouts: {}", self.total_timeouts);
        println!("Average wait time: {} ms", self.avg_wait_time_ms);
        println!("Queries executed: {}", self.total_queries);
        println!("Status: {}", if self.is_healthy() { "HEALTHY" } else { "NEEDS ATTENTION" });
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Connection Pool | A set of reusable database connections |
| min_connections | Minimum connections kept open |
| max_connections | Maximum connections in the pool |
| acquire_timeout | Timeout for getting a connection from pool |
| idle_timeout | Idle time before closing a connection |
| max_lifetime | Maximum lifetime of a connection |
| Pool Metrics | Monitoring health and performance |

## Practical Exercises

### Exercise 1: Basic Pool
Implement a simple connection pool with the ability to:
- Set min/max size
- Get a connection with timeout
- Automatically return via RAII

### Exercise 2: Monitoring
Add metrics collection to the pool:
- Number of successful/failed acquisitions
- Average connection wait time
- Current pool utilization

### Exercise 3: Pool Separation
Create a system with multiple pools for different operations:
- Pool for recording trades (high throughput)
- Pool for reading history (many readers)
- Pool for critical operations (guaranteed access)

### Exercise 4: Health Check
Implement a connection health check mechanism:
- Periodic connection ping
- Automatic removal of "dead" connections
- Restore minimum connection count

## Homework

1. **Adaptive Pool**: Create a pool that automatically increases size under high load and decreases when idle. Use wait time metrics for decision making.

2. **Priority Pool**: Implement a pool where critical operations (e.g., risk management) get connections first, even if the regular queue is full.

3. **Fault-Tolerant Pool**: Create a pool with support for multiple database servers (primary + replicas). Automatically switch to replica for reads when primary fails.

4. **Benchmark**: Write a performance test comparing:
   - Creating a new connection for each request
   - Using a pool with different settings (min=1/max=10, min=5/max=50)

   Measure throughput and latency at 100, 1000, and 10,000 parallel requests.

## Navigation

[← Previous day](../228-database-transactions/en.md) | [Next day →](../230-connection-timeout-handling/en.md)
