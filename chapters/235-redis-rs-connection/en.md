# Day 235: redis-rs: Connecting to Redis

## Trading Analogy

Imagine you're a trader arriving at the exchange every morning. Before you can start trading, you need to:
1. Connect to the trading terminal
2. Authenticate
3. Verify the connection is stable
4. Only then can you submit orders

Connecting to Redis works exactly the same way. **Redis** is an ultra-fast in-memory database that in trading is used for:
- Caching current prices (instant access)
- Storing user sessions
- Order queues (pub/sub)
- Temporary storage of calculations (risk metrics, P&L)

The **redis-rs** library is the official Rust client for working with Redis. Today we'll learn how to establish a connection — the first and most crucial step.

## Adding the Dependency

First, add `redis` to your `Cargo.toml`:

```toml
[dependencies]
redis = "0.25"
```

If you need async support (recommended for high-performance trading systems):

```toml
[dependencies]
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }
tokio = { version = "1", features = ["full"] }
```

## Basic Synchronous Connection

```rust
use redis::{Client, Commands, Connection};

fn main() -> redis::RedisResult<()> {
    // Create a client (analogous to setting up a trading terminal)
    let client = Client::open("redis://127.0.0.1:6379/")?;

    // Establish connection (analogous to logging into the exchange)
    let mut con: Connection = client.get_connection()?;

    println!("Successfully connected to Redis!");

    // Test operation — store current BTC price
    con.set("btc:price", 42500.50)?;

    // Read the price back
    let price: f64 = con.get("btc:price")?;
    println!("Current BTC price: ${:.2}", price);

    Ok(())
}
```

## Connection String Format

Redis supports several URL formats:

```rust
use redis::Client;

fn main() -> redis::RedisResult<()> {
    // Basic connection (localhost)
    let _client1 = Client::open("redis://127.0.0.1/")?;

    // With port specified
    let _client2 = Client::open("redis://127.0.0.1:6379/")?;

    // With password (for secured servers)
    let _client3 = Client::open("redis://:mypassword@127.0.0.1:6379/")?;

    // With database selection (0-15)
    let _client4 = Client::open("redis://127.0.0.1:6379/2")?;

    // With username and password (Redis 6+)
    let _client5 = Client::open("redis://trading_user:secure_pass@127.0.0.1:6379/")?;

    // Unix socket (for local high-performance systems)
    let _client6 = Client::open("unix:///var/run/redis/redis.sock")?;

    // TLS/SSL connection
    let _client7 = Client::open("rediss://127.0.0.1:6379/")?;

    println!("All connection formats are valid!");

    Ok(())
}
```

## Handling Connection Errors

In trading systems, proper error handling is critical:

```rust
use redis::{Client, Commands, RedisError};
use std::time::Duration;
use std::thread;

#[derive(Debug)]
struct TradingDataClient {
    client: Client,
    max_retries: u32,
    retry_delay: Duration,
}

impl TradingDataClient {
    fn new(url: &str) -> Result<Self, RedisError> {
        let client = Client::open(url)?;
        Ok(TradingDataClient {
            client,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        })
    }

    fn connect_with_retry(&self) -> Result<redis::Connection, RedisError> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.client.get_connection() {
                Ok(con) => {
                    println!("Connection established (attempt {})", attempt);
                    return Ok(con);
                }
                Err(e) => {
                    println!(
                        "Connection error (attempt {}/{}): {}",
                        attempt, self.max_retries, e
                    );
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        thread::sleep(self.retry_delay);
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    fn store_price(&self, symbol: &str, price: f64) -> Result<(), RedisError> {
        let mut con = self.connect_with_retry()?;
        let key = format!("price:{}", symbol);
        con.set(&key, price)?;
        println!("Stored price {}: ${:.2}", symbol, price);
        Ok(())
    }

    fn get_price(&self, symbol: &str) -> Result<f64, RedisError> {
        let mut con = self.connect_with_retry()?;
        let key = format!("price:{}", symbol);
        let price: f64 = con.get(&key)?;
        Ok(price)
    }
}

fn main() {
    match TradingDataClient::new("redis://127.0.0.1:6379/") {
        Ok(client) => {
            // Store prices
            if let Err(e) = client.store_price("BTC", 42500.0) {
                eprintln!("Error storing BTC: {}", e);
            }

            if let Err(e) = client.store_price("ETH", 2250.0) {
                eprintln!("Error storing ETH: {}", e);
            }

            // Read prices
            match client.get_price("BTC") {
                Ok(price) => println!("BTC price: ${:.2}", price),
                Err(e) => eprintln!("Error reading BTC: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Failed to create client: {}", e);
        }
    }
}
```

## Async Connection with Tokio

For high-performance trading systems, the async approach is recommended:

```rust
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    // Create client
    let client = Client::open("redis://127.0.0.1:6379/")?;

    // Async connection with multiplexing
    let mut con: MultiplexedConnection = client.get_multiplexed_async_connection().await?;

    println!("Async connection established!");

    // Async operations
    con.set("btc:price", 42500.50).await?;
    con.set("eth:price", 2250.75).await?;

    let btc: f64 = con.get("btc:price").await?;
    let eth: f64 = con.get("eth:price").await?;

    println!("BTC: ${:.2}, ETH: ${:.2}", btc, eth);

    Ok(())
}
```

## Connection Manager for Trading Applications

`ConnectionManager` automatically reconnects on connection drops — critical for 24/7 trading systems:

```rust
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use std::time::Duration;

struct MarketDataCache {
    manager: ConnectionManager,
}

impl MarketDataCache {
    async fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;
        Ok(MarketDataCache { manager })
    }

    async fn update_price(&mut self, symbol: &str, price: f64) -> redis::RedisResult<()> {
        let key = format!("market:{}:price", symbol);
        let timestamp_key = format!("market:{}:updated", symbol);

        // Store price and update time
        self.manager.set(&key, price).await?;
        self.manager.set(&timestamp_key, chrono::Utc::now().timestamp()).await?;

        // Set TTL (time to live) — 60 seconds
        self.manager.expire::<_, ()>(&key, 60).await?;
        self.manager.expire::<_, ()>(&timestamp_key, 60).await?;

        Ok(())
    }

    async fn get_price(&mut self, symbol: &str) -> redis::RedisResult<Option<f64>> {
        let key = format!("market:{}:price", symbol);
        self.manager.get(&key).await
    }

    async fn update_order_book(
        &mut self,
        symbol: &str,
        bids: &[(f64, f64)],  // (price, quantity)
        asks: &[(f64, f64)],
    ) -> redis::RedisResult<()> {
        let bids_key = format!("orderbook:{}:bids", symbol);
        let asks_key = format!("orderbook:{}:asks", symbol);

        // Clear old data
        self.manager.del::<_, ()>(&bids_key).await?;
        self.manager.del::<_, ()>(&asks_key).await?;

        // Add bids (sorted set: score = price)
        for (price, qty) in bids {
            let member = format!("{}:{}", price, qty);
            self.manager.zadd::<_, _, _, ()>(&bids_key, member, *price).await?;
        }

        // Add asks
        for (price, qty) in asks {
            let member = format!("{}:{}", price, qty);
            self.manager.zadd::<_, _, _, ()>(&asks_key, member, *price).await?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let mut cache = MarketDataCache::new("redis://127.0.0.1:6379/").await?;

    // Update market data
    cache.update_price("BTC", 42500.0).await?;
    cache.update_price("ETH", 2250.0).await?;

    // Update order book
    cache.update_order_book(
        "BTC",
        &[(42490.0, 1.5), (42480.0, 2.3), (42470.0, 5.0)],
        &[(42510.0, 1.2), (42520.0, 3.1), (42530.0, 4.5)],
    ).await?;

    // Read price
    if let Some(price) = cache.get_price("BTC").await? {
        println!("Cached BTC price: ${:.2}", price);
    }

    Ok(())
}
```

## Checking Connection Status

```rust
use redis::{Client, Commands, Connection};

fn check_redis_connection(con: &mut Connection) -> bool {
    match redis::cmd("PING").query::<String>(con) {
        Ok(response) => response == "PONG",
        Err(_) => false,
    }
}

fn get_redis_info(con: &mut Connection) -> redis::RedisResult<()> {
    // Get server information
    let info: String = redis::cmd("INFO").arg("server").query(con)?;

    println!("=== Redis Server Information ===");
    for line in info.lines() {
        if line.starts_with("redis_version") ||
           line.starts_with("connected_clients") ||
           line.starts_with("used_memory_human") {
            println!("{}", line);
        }
    }

    Ok(())
}

fn main() -> redis::RedisResult<()> {
    let client = Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_connection()?;

    if check_redis_connection(&mut con) {
        println!("Redis is available!");
        get_redis_info(&mut con)?;
    } else {
        eprintln!("Redis is unavailable!");
    }

    Ok(())
}
```

## Connection Pool for Multi-threaded Systems

For trading systems with multiple threads, use a connection pool:

```rust
use redis::{Client, Commands};
use std::sync::Arc;
use std::thread;

fn main() -> redis::RedisResult<()> {
    let client = Arc::new(Client::open("redis://127.0.0.1:6379/")?);

    let mut handles = vec![];

    // Simulate multiple trading threads
    for i in 0..4 {
        let client_clone = Arc::clone(&client);

        let handle = thread::spawn(move || {
            // Each thread gets its own connection
            let mut con = client_clone.get_connection().expect("Connection error");

            let symbol = match i {
                0 => "BTC",
                1 => "ETH",
                2 => "SOL",
                _ => "DOGE",
            };

            // Simulate price update
            let price = 1000.0 * (i as f64 + 1.0);
            let key = format!("price:{}", symbol);

            let _: () = con.set(&key, price).expect("Write error");
            println!("Thread {}: {} = ${:.2}", i, symbol, price);

            // Read back
            let stored: f64 = con.get(&key).expect("Read error");
            println!("Thread {}: Read {} = ${:.2}", i, symbol, stored);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread error");
    }

    println!("All threads completed successfully!");

    Ok(())
}
```

## Configuration via Environment Variables

```rust
use redis::Client;
use std::env;

#[derive(Debug)]
struct RedisConfig {
    host: String,
    port: u16,
    password: Option<String>,
    database: u8,
}

impl RedisConfig {
    fn from_env() -> Self {
        RedisConfig {
            host: env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("REDIS_PORT")
                .unwrap_or_else(|_| "6379".to_string())
                .parse()
                .unwrap_or(6379),
            password: env::var("REDIS_PASSWORD").ok(),
            database: env::var("REDIS_DATABASE")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
        }
    }

    fn to_url(&self) -> String {
        match &self.password {
            Some(pass) => format!(
                "redis://:{}@{}:{}/{}",
                pass, self.host, self.port, self.database
            ),
            None => format!(
                "redis://{}:{}/{}",
                self.host, self.port, self.database
            ),
        }
    }
}

fn main() -> redis::RedisResult<()> {
    let config = RedisConfig::from_env();
    println!("Redis configuration: {:?}", config);

    let client = Client::open(config.to_url())?;
    let _con = client.get_connection()?;

    println!("Connection successful!");

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Client::open()` | Create a Redis client with URL |
| `get_connection()` | Synchronous connection |
| `get_multiplexed_async_connection()` | Async connection |
| `ConnectionManager` | Automatic reconnection |
| URL Format | `redis://[user:pass@]host:port/db` |
| Error Handling | Retries, timeouts |
| Connection Pool | For multi-threaded applications |

## Exercises

1. **Basic Connection**: Create a program that connects to Redis, stores the current price of three cryptocurrencies (BTC, ETH, SOL), and reads them back.

2. **Error Handling**: Modify the program from exercise 1 to properly handle the situation when Redis is unavailable (display a clear message instead of panicking).

3. **Configuration**: Implement connection through environment variables (`REDIS_URL` or separate `REDIS_HOST`, `REDIS_PORT`, `REDIS_PASSWORD`).

4. **Multi-threading**: Create 4 threads, each writing and reading prices for different assets. Ensure all operations complete correctly.

## Homework

1. **Connection Monitoring**: Create a `RedisHealthChecker` struct that:
   - Periodically checks Redis availability (PING)
   - Logs response time
   - Notifies (via console output) when there are connection problems

2. **Quote Cache**: Implement a `QuoteCache` struct with methods:
   - `new(redis_url: &str)` — creation with Redis connection
   - `update_quote(symbol: &str, bid: f64, ask: f64)` — update a quote
   - `get_quote(symbol: &str) -> Option<(f64, f64)>` — get a quote
   - `get_spread(symbol: &str) -> Option<f64>` — calculate spread

3. **Graceful Shutdown**: Implement a trading application that:
   - Connects to Redis at startup
   - Handles the termination signal (Ctrl+C)
   - Properly closes the connection before exiting

4. **Benchmark**: Measure Redis performance:
   - Single read/write time
   - Throughput (operations per second)
   - Compare synchronous and asynchronous approaches

## Navigation

[← Day 234: Redis: Cache and Queues](../234-redis-cache-queues/en.md) | [Day 236: Redis: Caching Latest Prices →](../236-redis-caching-latest-prices/en.md)
