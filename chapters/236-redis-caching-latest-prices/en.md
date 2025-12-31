# Day 236: Redis: Caching Latest Prices

## Trading Analogy

Imagine yourself as a trader on an exchange. Every second you receive thousands of price updates across different instruments — stocks, cryptocurrencies, futures. If you queried the exchange directly for the latest price each time, you would get:
- Huge latency
- Exchange API overload
- Potential blocks for exceeding rate limits

Instead, experienced traders use a **local price cache** — a fast storage where current quotes are continuously updated. Redis is perfect for this task: it stores data in memory and provides microsecond access times.

It's like having a quote screen right in front of your eyes instead of calling your broker every time to ask for the current price.

## Why Redis for Price Caching?

| Feature | Trading Value |
|---------|---------------|
| In-memory storage | Microsecond access to prices |
| TTL (Time To Live) | Automatic expiration of stale prices |
| Atomic operations | Safe updates from multiple sources |
| Pub/Sub | Price change notifications (covered in next chapter) |
| Data structures | Hash for storing multiple price fields |

## Basic Price Cache Structure

```rust
use redis::{Client, Commands, RedisResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PriceData {
    symbol: String,
    bid: f64,           // Best buy price
    ask: f64,           // Best sell price
    last: f64,          // Last trade price
    volume_24h: f64,    // 24-hour volume
    timestamp: u64,     // Update time (Unix timestamp)
}

impl PriceData {
    fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    fn spread_percent(&self) -> f64 {
        (self.spread() / self.mid_price()) * 100.0
    }

    fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

fn main() -> RedisResult<()> {
    // Connect to Redis
    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    // Create price data
    let btc_price = PriceData {
        symbol: "BTC/USDT".to_string(),
        bid: 42150.50,
        ask: 42155.00,
        last: 42152.75,
        volume_24h: 15234.56,
        timestamp: 1704067200,
    };

    // Serialize to JSON and save
    let price_json = serde_json::to_string(&btc_price).unwrap();
    let _: () = con.set_ex("price:BTC/USDT", &price_json, 60)?; // TTL 60 seconds

    println!("BTC/USDT price saved to cache");
    println!("Spread: {:.2} ({:.4}%)", btc_price.spread(), btc_price.spread_percent());

    // Read from cache
    let cached: String = con.get("price:BTC/USDT")?;
    let restored: PriceData = serde_json::from_str(&cached).unwrap();

    println!("From cache: {:?}", restored);

    Ok(())
}
```

## Using Hash for Price Storage

Redis Hash allows storing and updating individual price fields without rewriting the entire object:

```rust
use redis::{Client, Commands, RedisResult};
use std::collections::HashMap;

struct PriceCache {
    con: redis::Connection,
}

impl PriceCache {
    fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(PriceCache { con })
    }

    // Update all price fields
    fn update_price(&mut self, symbol: &str, bid: f64, ask: f64, last: f64) -> RedisResult<()> {
        let key = format!("price:{}", symbol);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Use HSET to set multiple fields
        redis::cmd("HSET")
            .arg(&key)
            .arg("bid").arg(bid)
            .arg("ask").arg(ask)
            .arg("last").arg(last)
            .arg("timestamp").arg(timestamp)
            .query(&mut self.con)?;

        // Set TTL for the key
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(60) // 60 seconds
            .query(&mut self.con)?;

        Ok(())
    }

    // Update only bid/ask (for order book updates)
    fn update_quote(&mut self, symbol: &str, bid: f64, ask: f64) -> RedisResult<()> {
        let key = format!("price:{}", symbol);

        redis::cmd("HSET")
            .arg(&key)
            .arg("bid").arg(bid)
            .arg("ask").arg(ask)
            .query(&mut self.con)?;

        Ok(())
    }

    // Update only last price (for trade updates)
    fn update_last(&mut self, symbol: &str, last: f64) -> RedisResult<()> {
        let key = format!("price:{}", symbol);

        redis::cmd("HSET")
            .arg(&key)
            .arg("last").arg(last)
            .query(&mut self.con)?;

        Ok(())
    }

    // Get all price data
    fn get_price(&mut self, symbol: &str) -> RedisResult<Option<HashMap<String, String>>> {
        let key = format!("price:{}", symbol);
        let result: HashMap<String, String> = self.con.hgetall(&key)?;

        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    // Get specific field
    fn get_last_price(&mut self, symbol: &str) -> RedisResult<Option<f64>> {
        let key = format!("price:{}", symbol);
        let result: Option<String> = self.con.hget(&key, "last")?;
        Ok(result.map(|s| s.parse().unwrap_or(0.0)))
    }

    // Get bid/ask for spread calculation
    fn get_spread(&mut self, symbol: &str) -> RedisResult<Option<(f64, f64)>> {
        let key = format!("price:{}", symbol);
        let values: Vec<Option<String>> = redis::cmd("HMGET")
            .arg(&key)
            .arg("bid")
            .arg("ask")
            .query(&mut self.con)?;

        match (values.get(0), values.get(1)) {
            (Some(Some(bid)), Some(Some(ask))) => {
                let bid: f64 = bid.parse().unwrap_or(0.0);
                let ask: f64 = ask.parse().unwrap_or(0.0);
                Ok(Some((bid, ask)))
            }
            _ => Ok(None),
        }
    }
}

fn main() -> RedisResult<()> {
    let mut cache = PriceCache::new("redis://127.0.0.1/")?;

    // Update prices
    cache.update_price("BTC/USDT", 42150.50, 42155.00, 42152.75)?;
    cache.update_price("ETH/USDT", 2250.25, 2251.00, 2250.50)?;

    // Get data
    if let Some(btc_data) = cache.get_price("BTC/USDT")? {
        println!("BTC/USDT data: {:?}", btc_data);
    }

    if let Some(last) = cache.get_last_price("ETH/USDT")? {
        println!("ETH/USDT last price: {}", last);
    }

    if let Some((bid, ask)) = cache.get_spread("BTC/USDT")? {
        println!("BTC/USDT spread: {} - {} = {}", ask, bid, ask - bid);
    }

    Ok(())
}
```

## Caching Multiple Instrument Prices

```rust
use redis::{Client, Commands, RedisResult, Pipeline};
use std::collections::HashMap;

struct MultiPriceCache {
    con: redis::Connection,
}

impl MultiPriceCache {
    fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(MultiPriceCache { con })
    }

    // Batch price update (efficient for multiple instruments)
    fn batch_update(&mut self, prices: &[(String, f64, f64, f64)]) -> RedisResult<()> {
        let mut pipe = redis::pipe();

        for (symbol, bid, ask, last) in prices {
            let key = format!("price:{}", symbol);

            pipe.cmd("HSET")
                .arg(&key)
                .arg("bid").arg(*bid)
                .arg("ask").arg(*ask)
                .arg("last").arg(*last);

            pipe.cmd("EXPIRE").arg(&key).arg(60);
        }

        pipe.query(&mut self.con)?;
        Ok(())
    }

    // Get prices for a list of instruments
    fn get_multiple_prices(&mut self, symbols: &[&str]) -> RedisResult<HashMap<String, f64>> {
        let mut result = HashMap::new();

        // Use pipeline for efficient retrieval
        let mut pipe = redis::pipe();

        for symbol in symbols {
            let key = format!("price:{}", symbol);
            pipe.cmd("HGET").arg(&key).arg("last");
        }

        let prices: Vec<Option<String>> = pipe.query(&mut self.con)?;

        for (symbol, price) in symbols.iter().zip(prices.iter()) {
            if let Some(p) = price {
                if let Ok(val) = p.parse::<f64>() {
                    result.insert(symbol.to_string(), val);
                }
            }
        }

        Ok(result)
    }

    // Find instruments by pattern
    fn find_symbols(&mut self, pattern: &str) -> RedisResult<Vec<String>> {
        let search_pattern = format!("price:{}*", pattern);
        let keys: Vec<String> = self.con.keys(&search_pattern)?;

        Ok(keys.into_iter()
            .map(|k| k.replace("price:", ""))
            .collect())
    }

    // Get all cached symbols
    fn get_all_symbols(&mut self) -> RedisResult<Vec<String>> {
        self.find_symbols("")
    }
}

fn main() -> RedisResult<()> {
    let mut cache = MultiPriceCache::new("redis://127.0.0.1/")?;

    // Simulate price updates from exchange
    let market_data = vec![
        ("BTC/USDT".to_string(), 42150.50, 42155.00, 42152.75),
        ("ETH/USDT".to_string(), 2250.25, 2251.00, 2250.50),
        ("SOL/USDT".to_string(), 98.50, 98.60, 98.55),
        ("DOGE/USDT".to_string(), 0.0850, 0.0851, 0.0850),
    ];

    cache.batch_update(&market_data)?;
    println!("Updated {} instruments", market_data.len());

    // Get prices for portfolio
    let portfolio_symbols = ["BTC/USDT", "ETH/USDT"];
    let prices = cache.get_multiple_prices(&portfolio_symbols)?;

    println!("\nPortfolio prices:");
    for (symbol, price) in &prices {
        println!("  {}: ${:.2}", symbol, price);
    }

    // Find all USDT pairs
    let usdt_pairs = cache.find_symbols("*USDT")?;
    println!("\nFound USDT pairs: {:?}", usdt_pairs);

    Ok(())
}
```

## Practical Example: Trading Monitor with Caching

```rust
use redis::{Client, Commands, RedisResult};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;

#[derive(Debug, Clone)]
struct MarketTick {
    symbol: String,
    bid: f64,
    ask: f64,
    last: f64,
    timestamp: u64,
}

struct TradingMonitor {
    con: redis::Connection,
    price_ttl: usize,
}

impl TradingMonitor {
    fn new(redis_url: &str, price_ttl_seconds: usize) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(TradingMonitor {
            con,
            price_ttl: price_ttl_seconds,
        })
    }

    // Process tick from exchange
    fn process_tick(&mut self, tick: &MarketTick) -> RedisResult<()> {
        let key = format!("price:{}", tick.symbol);

        // Save current price
        redis::cmd("HSET")
            .arg(&key)
            .arg("bid").arg(tick.bid)
            .arg("ask").arg(tick.ask)
            .arg("last").arg(tick.last)
            .arg("timestamp").arg(tick.timestamp)
            .query(&mut self.con)?;

        // Set TTL
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(self.price_ttl)
            .query(&mut self.con)?;

        // Update price history (keep last 100 values)
        let history_key = format!("price_history:{}", tick.symbol);
        let price_entry = format!("{}:{}", tick.timestamp, tick.last);

        let _: () = self.con.lpush(&history_key, &price_entry)?;
        let _: () = self.con.ltrim(&history_key, 0, 99)?; // Keep only 100 entries

        Ok(())
    }

    // Get current price
    fn get_current_price(&mut self, symbol: &str) -> RedisResult<Option<MarketTick>> {
        let key = format!("price:{}", symbol);
        let data: HashMap<String, String> = self.con.hgetall(&key)?;

        if data.is_empty() {
            return Ok(None);
        }

        Ok(Some(MarketTick {
            symbol: symbol.to_string(),
            bid: data.get("bid").and_then(|s| s.parse().ok()).unwrap_or(0.0),
            ask: data.get("ask").and_then(|s| s.parse().ok()).unwrap_or(0.0),
            last: data.get("last").and_then(|s| s.parse().ok()).unwrap_or(0.0),
            timestamp: data.get("timestamp").and_then(|s| s.parse().ok()).unwrap_or(0),
        }))
    }

    // Check price freshness
    fn is_price_fresh(&mut self, symbol: &str, max_age_seconds: u64) -> RedisResult<bool> {
        let key = format!("price:{}", symbol);
        let timestamp: Option<u64> = self.con.hget(&key, "timestamp")?;

        match timestamp {
            Some(ts) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                Ok(now - ts <= max_age_seconds)
            }
            None => Ok(false),
        }
    }

    // Get price history
    fn get_price_history(&mut self, symbol: &str, count: isize) -> RedisResult<Vec<(u64, f64)>> {
        let key = format!("price_history:{}", symbol);
        let entries: Vec<String> = self.con.lrange(&key, 0, count - 1)?;

        let history: Vec<(u64, f64)> = entries
            .iter()
            .filter_map(|e| {
                let parts: Vec<&str> = e.split(':').collect();
                if parts.len() == 2 {
                    let ts = parts[0].parse().ok()?;
                    let price = parts[1].parse().ok()?;
                    Some((ts, price))
                } else {
                    None
                }
            })
            .collect();

        Ok(history)
    }

    // Calculate average price over period
    fn get_average_price(&mut self, symbol: &str, count: isize) -> RedisResult<Option<f64>> {
        let history = self.get_price_history(symbol, count)?;

        if history.is_empty() {
            return Ok(None);
        }

        let sum: f64 = history.iter().map(|(_, p)| p).sum();
        Ok(Some(sum / history.len() as f64))
    }
}

fn main() -> RedisResult<()> {
    let mut monitor = TradingMonitor::new("redis://127.0.0.1/", 60)?;

    // Simulate market data stream
    let ticks = vec![
        MarketTick {
            symbol: "BTC/USDT".to_string(),
            bid: 42150.0,
            ask: 42155.0,
            last: 42152.0,
            timestamp: 1704067200,
        },
        MarketTick {
            symbol: "BTC/USDT".to_string(),
            bid: 42155.0,
            ask: 42160.0,
            last: 42158.0,
            timestamp: 1704067201,
        },
        MarketTick {
            symbol: "BTC/USDT".to_string(),
            bid: 42148.0,
            ask: 42153.0,
            last: 42150.0,
            timestamp: 1704067202,
        },
        MarketTick {
            symbol: "ETH/USDT".to_string(),
            bid: 2250.0,
            ask: 2251.0,
            last: 2250.5,
            timestamp: 1704067200,
        },
    ];

    // Process ticks
    for tick in &ticks {
        monitor.process_tick(tick)?;
        println!("Processed tick: {} @ {}", tick.symbol, tick.last);
    }

    // Get current prices
    println!("\n--- Current Prices ---");
    if let Some(btc) = monitor.get_current_price("BTC/USDT")? {
        println!("BTC/USDT: bid={}, ask={}, last={}", btc.bid, btc.ask, btc.last);
    }

    // Get history
    println!("\n--- BTC/USDT History ---");
    let history = monitor.get_price_history("BTC/USDT", 10)?;
    for (ts, price) in &history {
        println!("  {}: ${:.2}", ts, price);
    }

    // Average price
    if let Some(avg) = monitor.get_average_price("BTC/USDT", 10)? {
        println!("\nBTC/USDT average price: ${:.2}", avg);
    }

    Ok(())
}
```

## Cache Invalidation Strategies

```rust
use redis::{Client, Commands, RedisResult};

struct PriceCacheWithInvalidation {
    con: redis::Connection,
}

impl PriceCacheWithInvalidation {
    fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(PriceCacheWithInvalidation { con })
    }

    // Set price with TTL
    fn set_price_with_ttl(&mut self, symbol: &str, price: f64, ttl_seconds: usize) -> RedisResult<()> {
        let key = format!("price:{}", symbol);
        let _: () = self.con.set_ex(&key, price, ttl_seconds)?;
        Ok(())
    }

    // Conditional update — only if price changed significantly
    fn update_if_significant_change(
        &mut self,
        symbol: &str,
        new_price: f64,
        threshold_percent: f64
    ) -> RedisResult<bool> {
        let key = format!("price:{}", symbol);

        // Get current price
        let current: Option<f64> = self.con.get(&key)?;

        match current {
            Some(current_price) => {
                let change_percent = ((new_price - current_price) / current_price).abs() * 100.0;

                if change_percent >= threshold_percent {
                    let _: () = self.con.set_ex(&key, new_price, 60)?;
                    println!(
                        "{}: Price updated {} -> {} (change {:.2}%)",
                        symbol, current_price, new_price, change_percent
                    );
                    Ok(true)
                } else {
                    // Only extend TTL
                    let _: () = self.con.expire(&key, 60)?;
                    Ok(false)
                }
            }
            None => {
                // No price exists — set new one
                let _: () = self.con.set_ex(&key, new_price, 60)?;
                Ok(true)
            }
        }
    }

    // Invalidate by pattern (be careful — can be slow!)
    fn invalidate_by_pattern(&mut self, pattern: &str) -> RedisResult<usize> {
        let search_pattern = format!("price:{}*", pattern);
        let keys: Vec<String> = self.con.keys(&search_pattern)?;

        let count = keys.len();
        for key in keys {
            let _: () = self.con.del(&key)?;
        }

        Ok(count)
    }

    // Invalidate all prices from an exchange
    fn invalidate_exchange(&mut self, exchange: &str) -> RedisResult<usize> {
        self.invalidate_by_pattern(&format!("{}:", exchange))
    }

    // Check and refresh stale prices
    fn refresh_stale_prices<F>(
        &mut self,
        symbols: &[&str],
        max_age_seconds: i64,
        fetch_price: F
    ) -> RedisResult<Vec<String>>
    where
        F: Fn(&str) -> Option<f64>
    {
        let mut refreshed = Vec::new();

        for symbol in symbols {
            let key = format!("price:{}", symbol);
            let ttl: i64 = self.con.ttl(&key)?;

            // If TTL is below threshold — refresh
            if ttl < max_age_seconds || ttl < 0 {
                if let Some(new_price) = fetch_price(symbol) {
                    let _: () = self.con.set_ex(&key, new_price, 60)?;
                    refreshed.push(symbol.to_string());
                }
            }
        }

        Ok(refreshed)
    }
}

fn main() -> RedisResult<()> {
    let mut cache = PriceCacheWithInvalidation::new("redis://127.0.0.1/")?;

    // Set initial prices
    cache.set_price_with_ttl("BTC/USDT", 42000.0, 60)?;
    cache.set_price_with_ttl("ETH/USDT", 2200.0, 60)?;

    // Conditional update
    println!("--- Conditional Update ---");
    cache.update_if_significant_change("BTC/USDT", 42010.0, 0.1)?; // Won't update (< 0.1%)
    cache.update_if_significant_change("BTC/USDT", 42100.0, 0.1)?; // Will update (> 0.1%)

    // Invalidation
    println!("\n--- Invalidation ---");
    let count = cache.invalidate_by_pattern("ETH")?;
    println!("Invalidated {} keys containing ETH", count);

    Ok(())
}
```

## Integration with Trading Strategy

```rust
use redis::{Client, Commands, RedisResult};
use std::collections::HashMap;

struct PriceFeed {
    con: redis::Connection,
}

impl PriceFeed {
    fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let con = client.get_connection()?;
        Ok(PriceFeed { con })
    }

    fn get_price(&mut self, symbol: &str) -> RedisResult<Option<f64>> {
        let key = format!("price:{}", symbol);
        self.con.get(&key)
    }

    fn set_price(&mut self, symbol: &str, price: f64) -> RedisResult<()> {
        let key = format!("price:{}", symbol);
        self.con.set_ex(&key, price, 60)
    }
}

struct Portfolio {
    positions: HashMap<String, f64>, // symbol -> quantity
}

impl Portfolio {
    fn new() -> Self {
        Portfolio {
            positions: HashMap::new(),
        }
    }

    fn add_position(&mut self, symbol: &str, quantity: f64) {
        *self.positions.entry(symbol.to_string()).or_insert(0.0) += quantity;
    }

    // Calculate portfolio value using cached prices
    fn calculate_value(&self, price_feed: &mut PriceFeed) -> RedisResult<f64> {
        let mut total = 0.0;

        for (symbol, quantity) in &self.positions {
            if let Some(price) = price_feed.get_price(symbol)? {
                total += price * quantity;
            } else {
                println!("Warning: no price for {}", symbol);
            }
        }

        Ok(total)
    }

    // Calculate exposure for each asset
    fn get_exposures(&self, price_feed: &mut PriceFeed) -> RedisResult<HashMap<String, f64>> {
        let total_value = self.calculate_value(price_feed)?;
        let mut exposures = HashMap::new();

        for (symbol, quantity) in &self.positions {
            if let Some(price) = price_feed.get_price(symbol)? {
                let position_value = price * quantity;
                let exposure = if total_value > 0.0 {
                    (position_value / total_value) * 100.0
                } else {
                    0.0
                };
                exposures.insert(symbol.clone(), exposure);
            }
        }

        Ok(exposures)
    }
}

struct RiskManager {
    max_position_percent: f64,
    max_drawdown_percent: f64,
}

impl RiskManager {
    fn new(max_position: f64, max_drawdown: f64) -> Self {
        RiskManager {
            max_position_percent: max_position,
            max_drawdown_percent: max_drawdown,
        }
    }

    // Check risks based on current prices
    fn check_risks(
        &self,
        portfolio: &Portfolio,
        price_feed: &mut PriceFeed
    ) -> RedisResult<Vec<String>> {
        let mut warnings = Vec::new();
        let exposures = portfolio.get_exposures(price_feed)?;

        for (symbol, exposure) in &exposures {
            if *exposure > self.max_position_percent {
                warnings.push(format!(
                    "Position {} exceeds limit: {:.1}% > {:.1}%",
                    symbol, exposure, self.max_position_percent
                ));
            }
        }

        Ok(warnings)
    }
}

fn main() -> RedisResult<()> {
    let mut price_feed = PriceFeed::new("redis://127.0.0.1/")?;

    // Set current prices
    price_feed.set_price("BTC/USDT", 42000.0)?;
    price_feed.set_price("ETH/USDT", 2200.0)?;
    price_feed.set_price("SOL/USDT", 95.0)?;

    // Create portfolio
    let mut portfolio = Portfolio::new();
    portfolio.add_position("BTC/USDT", 2.5);   // 2.5 BTC
    portfolio.add_position("ETH/USDT", 15.0);  // 15 ETH
    portfolio.add_position("SOL/USDT", 100.0); // 100 SOL

    // Calculate value
    let total_value = portfolio.calculate_value(&mut price_feed)?;
    println!("Total portfolio value: ${:.2}", total_value);

    // Get exposures
    let exposures = portfolio.get_exposures(&mut price_feed)?;
    println!("\nExposures:");
    for (symbol, exposure) in &exposures {
        println!("  {}: {:.1}%", symbol, exposure);
    }

    // Check risks
    let risk_manager = RiskManager::new(50.0, 10.0);
    let warnings = risk_manager.check_risks(&portfolio, &mut price_feed)?;

    if !warnings.is_empty() {
        println!("\nRisk Warnings:");
        for warning in &warnings {
            println!("  Warning: {}", warning);
        }
    }

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| SET/GET with TTL | Basic price caching with automatic expiration |
| HSET/HGET | Storing structured price data in Hash |
| Pipeline | Batch operations for efficiency |
| TTL | Automatic invalidation of stale prices |
| Price history | Storing in Redis lists (LPUSH/LRANGE) |
| Conditional update | Update only on significant changes |

## Exercises

1. **OHLCV Cache**: Implement caching for candlestick data (Open, High, Low, Close, Volume) using Hash. Add methods for retrieving data over a time period.

2. **Price Aggregator**: Create a system that accepts prices from multiple exchanges and calculates a volume-weighted average price. Store each exchange's prices separately.

3. **Anomaly Detector**: Implement a function that compares a new price with history and logs a warning when there's a sudden change (more than 5% from the average).

## Homework

1. **Advanced Price Cache**: Create an `AdvancedPriceCache` struct with methods:
   - `update_price(symbol, bid, ask, last, volume)` — update price
   - `get_vwap(symbol, period)` — calculate VWAP over period
   - `get_volatility(symbol, period)` — calculate volatility
   - `subscribe_to_updates(callback)` — subscribe to updates (preparation for Pub/Sub)

2. **Multi-Exchange Arbitrage**: Implement a system that:
   - Stores prices for the same instrument from different exchanges
   - Finds arbitrage opportunities (price difference > 0.1%)
   - Logs potential trades

3. **Portfolio Monitor**: Create a web service (using `actix-web` or `axum`) that:
   - Accepts price updates via REST API
   - Stores them in Redis
   - Returns current portfolio value via GET request

4. **Stress Test**: Write a benchmark that:
   - Writes 10,000 price updates per second
   - Simultaneously reads random prices
   - Measures operation latency

## Navigation

[← Previous day](../235-redis-rs-connection/en.md) | [Next day →](../237-redis-pubsub-notifications/en.md)
