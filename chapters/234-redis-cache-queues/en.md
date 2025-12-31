# Day 234: Redis: Cache and Queues

## Trading Analogy

Imagine you're working at an exchange and need instant access to current asset prices. Querying the main database every time is too slow. Instead, you use a **cache** — fast memory where the latest prices are stored. It's like keeping a board with current quotes right in front of you, rather than running to the archives each time.

Now imagine a stream of orders: thousands of buy and sell requests arrive every second. They can't be processed chaotically — you need a **queue** that guarantees each order will be processed in the correct sequence. Redis is like a super-fast notepad that can both cache data and organize queues.

## What is Redis?

Redis (Remote Dictionary Server) is:
- **In-memory storage** — data is stored in RAM
- **Key-value database** — simple storage by keys
- **Message broker** — supports pub/sub and queues
- **Ultra-fast** — latencies are measured in microseconds

### Why is Redis Important for Trading?

| Use Case | Benefit |
|----------|---------|
| Price caching | Instant access to quotes |
| Order queue | Guaranteed processing order |
| User sessions | Fast authentication |
| Rate limiting | API overload protection |
| Pub/Sub | Real-time update broadcasting |

## Connecting to Redis from Rust

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Basic Connection

```rust
use redis::AsyncCommands;
use redis::Client;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    // Connect to Redis
    let client = Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_multiplexed_async_connection().await?;

    // Simple key operations
    con.set("btc_price", 42000.50_f64).await?;
    let price: f64 = con.get("btc_price").await?;
    println!("BTC price: ${}", price);

    // Set with TTL (Time To Live)
    con.set_ex("eth_price", 2800.25_f64, 60).await?; // Expires in 60 seconds
    println!("ETH price set with 60 second TTL");

    Ok(())
}
```

## Caching Market Data

### Structure for Storing Quotes

```rust
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MarketTick {
    symbol: String,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume_24h: f64,
    timestamp: u64,
}

impl MarketTick {
    fn new(symbol: &str, bid: f64, ask: f64, last_price: f64, volume: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        MarketTick {
            symbol: symbol.to_string(),
            bid,
            ask,
            last_price,
            volume_24h: volume,
            timestamp,
        }
    }

    fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    fn spread_percent(&self) -> f64 {
        (self.spread() / self.last_price) * 100.0
    }
}

struct PriceCache {
    client: redis::Client,
}

impl PriceCache {
    fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(PriceCache { client })
    }

    async fn set_tick(&self, tick: &MarketTick, ttl_seconds: u64) -> redis::RedisResult<()> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = format!("tick:{}", tick.symbol);
        let json = serde_json::to_string(tick).unwrap();

        con.set_ex(&key, json, ttl_seconds).await?;

        // Also store individual fields for quick access
        let price_key = format!("price:{}", tick.symbol);
        con.set_ex(&price_key, tick.last_price, ttl_seconds).await?;

        Ok(())
    }

    async fn get_tick(&self, symbol: &str) -> redis::RedisResult<Option<MarketTick>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = format!("tick:{}", symbol);

        let json: Option<String> = con.get(&key).await?;

        Ok(json.and_then(|j| serde_json::from_str(&j).ok()))
    }

    async fn get_price(&self, symbol: &str) -> redis::RedisResult<Option<f64>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = format!("price:{}", symbol);

        con.get(&key).await
    }

    async fn get_multiple_prices(&self, symbols: &[&str]) -> redis::RedisResult<Vec<Option<f64>>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let keys: Vec<String> = symbols.iter().map(|s| format!("price:{}", s)).collect();

        redis::cmd("MGET")
            .arg(&keys)
            .query_async(&mut con)
            .await
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let cache = PriceCache::new("redis://127.0.0.1:6379/")?;

    // Cache quotes
    let btc_tick = MarketTick::new("BTCUSDT", 41990.0, 42010.0, 42000.0, 50000.0);
    let eth_tick = MarketTick::new("ETHUSDT", 2795.0, 2805.0, 2800.0, 100000.0);

    cache.set_tick(&btc_tick, 5).await?; // 5 second TTL for volatile data
    cache.set_tick(&eth_tick, 5).await?;

    println!("BTC spread: {:.4}%", btc_tick.spread_percent());
    println!("ETH spread: {:.4}%", eth_tick.spread_percent());

    // Get from cache
    if let Some(tick) = cache.get_tick("BTCUSDT").await? {
        println!("From cache: {} @ ${}", tick.symbol, tick.last_price);
    }

    // Batch price retrieval
    let prices = cache.get_multiple_prices(&["BTCUSDT", "ETHUSDT", "SOLUSDT"]).await?;
    println!("Prices: {:?}", prices);

    Ok(())
}
```

## Order Queues with Redis

### Simple Queue with LPUSH/RPOP

```rust
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Order {
    id: String,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum OrderType {
    Market,
    Limit,
}

struct OrderQueue {
    client: redis::Client,
    queue_name: String,
}

impl OrderQueue {
    fn new(redis_url: &str, queue_name: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(OrderQueue {
            client,
            queue_name: queue_name.to_string(),
        })
    }

    // Add order to the end of the queue
    async fn push(&self, order: &Order) -> redis::RedisResult<i64> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(order).unwrap();

        con.rpush(&self.queue_name, json).await
    }

    // Add priority order to the front of the queue
    async fn push_priority(&self, order: &Order) -> redis::RedisResult<i64> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(order).unwrap();

        con.lpush(&self.queue_name, json).await
    }

    // Get order from the front of the queue (non-blocking)
    async fn pop(&self) -> redis::RedisResult<Option<Order>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let json: Option<String> = con.lpop(&self.queue_name, None).await?;

        Ok(json.and_then(|j| serde_json::from_str(&j).ok()))
    }

    // Blocking get (waits until timeout)
    async fn pop_blocking(&self, timeout_seconds: f64) -> redis::RedisResult<Option<Order>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;

        let result: Option<(String, String)> = redis::cmd("BLPOP")
            .arg(&self.queue_name)
            .arg(timeout_seconds)
            .query_async(&mut con)
            .await?;

        Ok(result.and_then(|(_, json)| serde_json::from_str(&json).ok()))
    }

    // Get queue length
    async fn len(&self) -> redis::RedisResult<i64> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.llen(&self.queue_name).await
    }

    // Peek at orders without removing
    async fn peek(&self, count: isize) -> redis::RedisResult<Vec<Order>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let items: Vec<String> = con.lrange(&self.queue_name, 0, count - 1).await?;

        Ok(items
            .into_iter()
            .filter_map(|j| serde_json::from_str(&j).ok())
            .collect())
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let queue = OrderQueue::new("redis://127.0.0.1:6379/", "orders:pending")?;

    // Create test orders
    let orders = vec![
        Order {
            id: "ord_001".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 0.5,
            price: Some(41000.0),
            timestamp: 1700000001,
        },
        Order {
            id: "ord_002".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            quantity: 10.0,
            price: None,
            timestamp: 1700000002,
        },
    ];

    // Add to queue
    for order in &orders {
        let queue_len = queue.push(order).await?;
        println!("Order {} added, queue length: {}", order.id, queue_len);
    }

    // Priority order
    let priority_order = Order {
        id: "ord_priority".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 1.0,
        price: None,
        timestamp: 1700000000,
    };
    queue.push_priority(&priority_order).await?;
    println!("Priority order added to front of queue");

    // View queue
    let pending = queue.peek(10).await?;
    println!("\nOrders in queue:");
    for order in &pending {
        println!("  {} - {} {} {}",
            order.id, order.symbol,
            match order.side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" },
            order.quantity);
    }

    // Process queue
    println!("\nProcessing orders:");
    while let Some(order) = queue.pop().await? {
        println!("Processed: {} - {} @ {:?}", order.id, order.symbol, order.price);
    }

    Ok(())
}
```

## Pub/Sub for Market Data

### Quote Publisher

```rust
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::time::{interval, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    change_percent: f64,
    timestamp: u64,
}

struct MarketDataPublisher {
    client: redis::Client,
}

impl MarketDataPublisher {
    fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(MarketDataPublisher { client })
    }

    async fn publish(&self, channel: &str, update: &PriceUpdate) -> redis::RedisResult<i64> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(update).unwrap();

        con.publish(channel, json).await
    }

    async fn publish_to_symbol(&self, update: &PriceUpdate) -> redis::RedisResult<i64> {
        let channel = format!("prices:{}", update.symbol);
        self.publish(&channel, update).await
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let publisher = MarketDataPublisher::new("redis://127.0.0.1:6379/")?;

    let mut interval = interval(Duration::from_secs(1));
    let mut price = 42000.0_f64;

    println!("Starting BTC price publication...");

    for i in 0..10 {
        interval.tick().await;

        // Simulate price change
        let change = (rand::random::<f64>() - 0.5) * 100.0;
        price += change;
        let change_percent = (change / price) * 100.0;

        let update = PriceUpdate {
            symbol: "BTCUSDT".to_string(),
            price,
            change_percent,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let subscribers = publisher.publish_to_symbol(&update).await?;
        println!("[{}] BTC: ${:.2} ({:+.2}%) - {} subscribers",
            i + 1, update.price, update.change_percent, subscribers);
    }

    Ok(())
}
```

### Quote Subscriber

```rust
use redis::Client;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let client = Client::open("redis://127.0.0.1:6379/")?;
    let mut pubsub = client.get_async_pubsub().await?;

    // Subscribe to multiple channels
    pubsub.subscribe("prices:BTCUSDT").await?;
    pubsub.subscribe("prices:ETHUSDT").await?;

    // Or pattern subscription
    pubsub.psubscribe("prices:*").await?;

    println!("Subscribed to price updates...");

    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        let channel: String = msg.get_channel_name().to_string();
        let payload: String = msg.get_payload()?;

        println!("Channel: {} | Data: {}", channel, payload);

        // Parse and process
        if let Ok(update) = serde_json::from_str::<serde_json::Value>(&payload) {
            if let Some(price) = update.get("price").and_then(|p| p.as_f64()) {
                println!("  -> Price: ${:.2}", price);
            }
        }
    }

    Ok(())
}
```

## Rate Limiting for API

```rust
use redis::AsyncCommands;
use std::time::Duration;

struct RateLimiter {
    client: redis::Client,
    max_requests: u32,
    window_seconds: u64,
}

impl RateLimiter {
    fn new(redis_url: &str, max_requests: u32, window_seconds: u64) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(RateLimiter {
            client,
            max_requests,
            window_seconds,
        })
    }

    async fn check_rate_limit(&self, client_id: &str) -> redis::RedisResult<RateLimitResult> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = format!("ratelimit:{}", client_id);

        // Use INCR for atomicity
        let current: i64 = con.incr(&key, 1).await?;

        if current == 1 {
            // First request - set TTL
            con.expire(&key, self.window_seconds as i64).await?;
        }

        let remaining = self.max_requests as i64 - current;
        let ttl: i64 = con.ttl(&key).await?;

        if current > self.max_requests as i64 {
            Ok(RateLimitResult::Exceeded {
                retry_after: Duration::from_secs(ttl.max(0) as u64),
            })
        } else {
            Ok(RateLimitResult::Allowed {
                remaining: remaining.max(0) as u32,
                reset_in: Duration::from_secs(ttl.max(0) as u64),
            })
        }
    }
}

#[derive(Debug)]
enum RateLimitResult {
    Allowed { remaining: u32, reset_in: Duration },
    Exceeded { retry_after: Duration },
}

async fn handle_api_request(limiter: &RateLimiter, client_id: &str) -> Result<(), String> {
    match limiter.check_rate_limit(client_id).await {
        Ok(RateLimitResult::Allowed { remaining, reset_in }) => {
            println!("Request allowed. Remaining: {}, resets in: {:?}",
                remaining, reset_in);
            Ok(())
        }
        Ok(RateLimitResult::Exceeded { retry_after }) => {
            println!("Rate limit exceeded! Retry after: {:?}", retry_after);
            Err(format!("Rate limit exceeded. Retry after {:?}", retry_after))
        }
        Err(e) => {
            println!("Redis error: {}", e);
            // On Redis error — allow (fail open)
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let limiter = RateLimiter::new("redis://127.0.0.1:6379/", 5, 10)?; // 5 requests per 10 seconds

    let client_id = "trader_123";

    println!("Testing rate limiting:");
    for i in 1..=8 {
        println!("\nRequest #{}", i);
        let _ = handle_api_request(&limiter, client_id).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}
```

## Practical Example: Trading System with Redis

```rust
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trade {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    unrealized_pnl: f64,
}

struct TradingSystem {
    redis_client: redis::Client,
    positions: Arc<RwLock<HashMap<String, Position>>>,
}

impl TradingSystem {
    fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(TradingSystem {
            redis_client: client,
            positions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    // Cache positions
    async fn sync_positions_to_cache(&self) -> redis::RedisResult<()> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;
        let positions = self.positions.read().await;

        for (symbol, position) in positions.iter() {
            let key = format!("position:{}", symbol);
            let json = serde_json::to_string(position).unwrap();
            con.set_ex(&key, json, 300).await?; // 5 minute TTL
        }

        Ok(())
    }

    // Record trade to history
    async fn record_trade(&self, trade: &Trade) -> redis::RedisResult<()> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;

        // To recent trades list
        let trades_key = format!("trades:{}", trade.symbol);
        let json = serde_json::to_string(trade).unwrap();
        con.lpush(&trades_key, &json).await?;
        con.ltrim(&trades_key, 0, 999).await?; // Keep last 1000 trades

        // To sorted set for fast time-based lookup
        let history_key = format!("trade_history:{}", trade.symbol);
        con.zadd(&history_key, &json, trade.timestamp as f64).await?;

        // Publish event
        con.publish("trades:new", &json).await?;

        Ok(())
    }

    // Get recent trades
    async fn get_recent_trades(&self, symbol: &str, count: isize) -> redis::RedisResult<Vec<Trade>> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("trades:{}", symbol);

        let items: Vec<String> = con.lrange(&key, 0, count - 1).await?;

        Ok(items
            .into_iter()
            .filter_map(|j| serde_json::from_str(&j).ok())
            .collect())
    }

    // Priority order queue
    async fn submit_order(&self, order: &Order, priority: i64) -> redis::RedisResult<()> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(order).unwrap();

        // Use sorted set for priority queue
        // Lower score = higher priority
        con.zadd("orders:priority_queue", &json, priority as f64).await?;

        Ok(())
    }

    // Get next order by priority
    async fn get_next_order(&self) -> redis::RedisResult<Option<Order>> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;

        // ZPOPMIN returns element with minimum score
        let result: Vec<(String, f64)> = redis::cmd("ZPOPMIN")
            .arg("orders:priority_queue")
            .arg(1)
            .query_async(&mut con)
            .await?;

        Ok(result.first().and_then(|(json, _)| serde_json::from_str(json).ok()))
    }

    // Trading statistics for period
    async fn get_trading_stats(&self, symbol: &str) -> redis::RedisResult<TradingStats> {
        let mut con = self.redis_client.get_multiplexed_async_connection().await?;

        let trades_key = format!("trades:{}", symbol);
        let count: i64 = con.llen(&trades_key).await?;

        // Get recent trades for volume calculation
        let recent: Vec<String> = con.lrange(&trades_key, 0, 99).await?;
        let trades: Vec<Trade> = recent
            .into_iter()
            .filter_map(|j| serde_json::from_str(&j).ok())
            .collect();

        let total_volume: f64 = trades.iter().map(|t| t.quantity * t.price).sum();
        let avg_price = if !trades.is_empty() {
            trades.iter().map(|t| t.price).sum::<f64>() / trades.len() as f64
        } else {
            0.0
        };

        Ok(TradingStats {
            symbol: symbol.to_string(),
            trade_count: count,
            total_volume,
            avg_price,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct TradingStats {
    symbol: String,
    trade_count: i64,
    total_volume: f64,
    avg_price: f64,
}

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let system = TradingSystem::new("redis://127.0.0.1:6379/")?;

    // Record some trades
    let trades = vec![
        Trade {
            id: "t1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            price: 42000.0,
            quantity: 0.5,
            timestamp: 1700000001,
        },
        Trade {
            id: "t2".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            price: 42100.0,
            quantity: 0.3,
            timestamp: 1700000002,
        },
    ];

    for trade in &trades {
        system.record_trade(trade).await?;
        println!("Recorded trade: {} {} {} @ ${}",
            trade.id, trade.side, trade.quantity, trade.price);
    }

    // Submit orders with priorities
    let orders = vec![
        (Order {
            id: "o1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            price: 41000.0,
            quantity: 1.0,
        }, 10), // Low priority
        (Order {
            id: "o2".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            price: 43000.0,
            quantity: 0.5,
        }, 1),  // High priority
    ];

    for (order, priority) in &orders {
        system.submit_order(order, *priority).await?;
        println!("Order {} submitted with priority {}", order.id, priority);
    }

    // Get orders by priority
    println!("\nProcessing orders by priority:");
    while let Some(order) = system.get_next_order().await? {
        println!("  Processing: {} - {} {} @ ${}",
            order.id, order.side, order.quantity, order.price);
    }

    // Statistics
    let stats = system.get_trading_stats("BTCUSDT").await?;
    println!("\nBTCUSDT Statistics:");
    println!("  Trades: {}", stats.trade_count);
    println!("  Volume: ${:.2}", stats.total_volume);
    println!("  Avg Price: ${:.2}", stats.avg_price);

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Redis | In-memory storage for cache and queues |
| SET/GET | Basic key operations |
| TTL | Automatic data expiration |
| LPUSH/RPOP | Queues (FIFO) |
| Sorted Sets | Priority queues |
| Pub/Sub | Real-time event broadcasting |
| Rate Limiting | Request frequency control |

## Homework

1. **Quote Cache**: Implement a caching system that:
   - Stores quotes with different TTLs for different timeframes (1 sec for ticks, 1 min for candles)
   - Automatically updates cache when new data arrives
   - Supports batch retrieval of prices for multiple symbols

2. **Order Queue with DLQ**: Create an order processing system with:
   - Main order queue
   - Dead Letter Queue (DLQ) for failed orders
   - Automatic retry with exponential backoff
   - Processing success metrics

3. **Real-time Alerts**: Implement an alert system based on Pub/Sub:
   - Subscribe to price changes
   - Trigger conditions (price above/below threshold)
   - Notifications via separate channel
   - Alert history storage

4. **Distributed Rate Limiter**: Create a request limiting system for trading API:
   - Different limits for different endpoints
   - Burst mode support
   - Usage metrics
   - Graceful degradation when Redis is unavailable

## Navigation

[← Previous day](../233-postgresql-sqlx/en.md) | [Next day →](../235-mongodb-document-storage/en.md)
