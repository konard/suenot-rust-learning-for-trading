# Day 321: Result Caching

## Trading Analogy

Imagine you're managing a trading terminal that displays data for hundreds of assets. Every time a trader opens an asset page, the system requests:
- Current price from the exchange
- Trade history for the last hour
- Technical indicator calculations
- News about the asset

Without caching, each of 1000 traders opening the BTC page creates 4 requests. That's 4000 requests per second for the same data!

**Result caching** is like an information board in a trading floor:
- Data is updated centrally once per second
- All traders look at the same board
- No need for everyone to call the exchange separately

Unlike memoization (caching at the function level), **result caching** works at the system level — caching API responses, complex query results, and aggregated data.

## When to Use Result Caching?

| Scenario | Trading Example | Benefit |
|----------|----------------|---------|
| **Frequent identical requests** | BTC price requested 1000 times/sec | Reduced exchange load |
| **Expensive computations** | Portfolio risk calculation | CPU savings |
| **Slow external services** | Broker API requests | Faster response |
| **Rarely changing data** | List of available assets | Minimized network requests |

## Simple Cache with Invalidation

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cached value with timestamp
struct CachedValue<T> {
    value: T,
    cached_at: Instant,
    ttl: Duration,
}

impl<T: Clone> CachedValue<T> {
    fn new(value: T, ttl: Duration) -> Self {
        CachedValue {
            value,
            cached_at: Instant::now(),
            ttl,
        }
    }

    fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }

    fn get(&self) -> Option<T> {
        if self.is_valid() {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

/// Asset price cache with automatic invalidation
struct PriceCache {
    cache: HashMap<String, CachedValue<f64>>,
    default_ttl: Duration,
    hits: u64,
    misses: u64,
}

impl PriceCache {
    fn new(ttl_seconds: u64) -> Self {
        PriceCache {
            cache: HashMap::new(),
            default_ttl: Duration::from_secs(ttl_seconds),
            hits: 0,
            misses: 0,
        }
    }

    /// Get price from cache or compute
    fn get_or_fetch<F>(&mut self, symbol: &str, fetch_fn: F) -> f64
    where
        F: FnOnce() -> f64,
    {
        // Try to get from cache
        if let Some(cached) = self.cache.get(symbol) {
            if let Some(price) = cached.get() {
                self.hits += 1;
                return price;
            }
        }

        // Cache empty or stale — fetch fresh data
        self.misses += 1;
        let price = fetch_fn();

        // Store in cache
        self.cache.insert(
            symbol.to_string(),
            CachedValue::new(price, self.default_ttl),
        );

        price
    }

    /// Force invalidation
    fn invalidate(&mut self, symbol: &str) {
        self.cache.remove(symbol);
    }

    /// Cleanup stale entries
    fn cleanup(&mut self) {
        self.cache.retain(|_, v| v.is_valid());
    }

    /// Cache statistics
    fn stats(&self) -> (u64, u64, f64) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 {
            (self.hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        (self.hits, self.misses, hit_rate)
    }
}

fn main() {
    let mut cache = PriceCache::new(5); // TTL = 5 seconds

    // Simulate fetching price from exchange
    let fetch_btc_price = || {
        println!("  [API] Fetching BTC price from exchange...");
        42500.0
    };

    println!("=== Price Caching Test ===\n");

    // First request — cache miss
    let price1 = cache.get_or_fetch("BTC", fetch_btc_price);
    println!("BTC price: ${:.2}\n", price1);

    // Repeated requests — cache hits
    for i in 2..=5 {
        let price = cache.get_or_fetch("BTC", fetch_btc_price);
        println!("Request #{}: ${:.2} (from cache)", i, price);
    }

    let (hits, misses, rate) = cache.stats();
    println!("\n=== Statistics ===");
    println!("Hits: {}", hits);
    println!("Misses: {}", misses);
    println!("Efficiency: {:.1}%", rate);
}
```

## Multi-Level Cache

Trading systems often use multi-level caching:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cache level
#[derive(Debug, Clone, Copy)]
enum CacheLevel {
    L1Memory,    // Fast, small (local memory)
    L2Shared,    // Medium (Redis, shared memory)
    L3Database,  // Slow, large (database)
}

struct CacheEntry {
    value: f64,
    created_at: Instant,
    ttl: Duration,
    level: CacheLevel,
}

impl CacheEntry {
    fn is_valid(&self) -> bool {
        self.created_at.elapsed() < self.ttl
    }
}

struct MultiLevelCache {
    l1: HashMap<String, CacheEntry>,
    l2: HashMap<String, CacheEntry>,
    l1_ttl: Duration,
    l2_ttl: Duration,
}

impl MultiLevelCache {
    fn new() -> Self {
        MultiLevelCache {
            l1: HashMap::new(),
            l2: HashMap::new(),
            l1_ttl: Duration::from_millis(100),  // 100ms for hot data
            l2_ttl: Duration::from_secs(5),      // 5 seconds for warm data
        }
    }

    fn get(&mut self, key: &str) -> Option<(f64, CacheLevel)> {
        // Check L1 (fastest)
        if let Some(entry) = self.l1.get(key) {
            if entry.is_valid() {
                return Some((entry.value, CacheLevel::L1Memory));
            }
        }

        // Check L2
        if let Some(entry) = self.l2.get(key) {
            if entry.is_valid() {
                // Promote to L1
                self.l1.insert(key.to_string(), CacheEntry {
                    value: entry.value,
                    created_at: Instant::now(),
                    ttl: self.l1_ttl,
                    level: CacheLevel::L1Memory,
                });
                return Some((entry.value, CacheLevel::L2Shared));
            }
        }

        None
    }

    fn set(&mut self, key: &str, value: f64) {
        // Store in both levels
        let now = Instant::now();

        self.l1.insert(key.to_string(), CacheEntry {
            value,
            created_at: now,
            ttl: self.l1_ttl,
            level: CacheLevel::L1Memory,
        });

        self.l2.insert(key.to_string(), CacheEntry {
            value,
            created_at: now,
            ttl: self.l2_ttl,
            level: CacheLevel::L2Shared,
        });
    }

    fn get_or_fetch<F>(&mut self, key: &str, fetch: F) -> (f64, CacheLevel)
    where
        F: FnOnce() -> f64,
    {
        if let Some((value, level)) = self.get(key) {
            return (value, level);
        }

        let value = fetch();
        self.set(key, value);
        (value, CacheLevel::L3Database)
    }
}

fn main() {
    let mut cache = MultiLevelCache::new();

    println!("=== Multi-Level Cache ===\n");

    // First request — L3 (database)
    let (price, level) = cache.get_or_fetch("BTCUSDT", || {
        println!("[L3] Loading from database...");
        42500.0
    });
    println!("BTC: ${:.2} (level: {:?})\n", price, level);

    // Second request — L1 (memory)
    let (price, level) = cache.get_or_fetch("BTCUSDT", || 0.0);
    println!("BTC: ${:.2} (level: {:?})", price, level);

    // Wait for L1 expiration
    println!("\nWaiting 150ms...");
    std::thread::sleep(Duration::from_millis(150));

    // Third request — L2 (after L1 expiration)
    let (price, level) = cache.get_or_fetch("BTCUSDT", || 0.0);
    println!("BTC: ${:.2} (level: {:?})", price, level);
}
```

## Caching with Versioning

For trading data, tracking versions is important:

```rust
use std::collections::HashMap;
use std::time::Instant;

/// Versioned cache entry
#[derive(Clone)]
struct VersionedEntry<T> {
    value: T,
    version: u64,
    updated_at: Instant,
}

/// Order book cache with versioning
struct OrderBookCache {
    cache: HashMap<String, VersionedEntry<OrderBook>>,
    current_versions: HashMap<String, u64>,
}

#[derive(Clone, Debug)]
struct OrderBook {
    symbol: String,
    bids: Vec<(f64, f64)>,  // (price, volume)
    asks: Vec<(f64, f64)>,
}

impl OrderBookCache {
    fn new() -> Self {
        OrderBookCache {
            cache: HashMap::new(),
            current_versions: HashMap::new(),
        }
    }

    /// Update order book (only if version is newer)
    fn update(&mut self, symbol: &str, book: OrderBook, version: u64) -> bool {
        let current = self.current_versions.get(symbol).copied().unwrap_or(0);

        if version > current {
            self.cache.insert(symbol.to_string(), VersionedEntry {
                value: book,
                version,
                updated_at: Instant::now(),
            });
            self.current_versions.insert(symbol.to_string(), version);
            true
        } else {
            false  // Skip stale update
        }
    }

    /// Get order book
    fn get(&self, symbol: &str) -> Option<&OrderBook> {
        self.cache.get(symbol).map(|e| &e.value)
    }

    /// Get if newer than specified version
    fn get_if_newer(&self, symbol: &str, min_version: u64) -> Option<&OrderBook> {
        self.cache.get(symbol).and_then(|entry| {
            if entry.version > min_version {
                Some(&entry.value)
            } else {
                None
            }
        })
    }

    /// Current version
    fn version(&self, symbol: &str) -> u64 {
        self.current_versions.get(symbol).copied().unwrap_or(0)
    }
}

fn main() {
    let mut cache = OrderBookCache::new();

    println!("=== Versioned Order Book Cache ===\n");

    // First update
    let book_v1 = OrderBook {
        symbol: "BTCUSDT".to_string(),
        bids: vec![(42500.0, 1.5), (42490.0, 2.0)],
        asks: vec![(42510.0, 1.0), (42520.0, 3.0)],
    };

    let updated = cache.update("BTCUSDT", book_v1, 1);
    println!("Update v1: {}", if updated { "accepted" } else { "rejected" });

    // Attempt to update with older version (should be rejected)
    let book_old = OrderBook {
        symbol: "BTCUSDT".to_string(),
        bids: vec![(42000.0, 1.0)],
        asks: vec![(43000.0, 1.0)],
    };

    let updated = cache.update("BTCUSDT", book_old, 0);
    println!("Update v0: {}", if updated { "accepted" } else { "rejected" });

    // New update with higher version
    let book_v2 = OrderBook {
        symbol: "BTCUSDT".to_string(),
        bids: vec![(42505.0, 2.0), (42495.0, 1.5)],
        asks: vec![(42515.0, 1.5), (42525.0, 2.5)],
    };

    let updated = cache.update("BTCUSDT", book_v2, 2);
    println!("Update v2: {}", if updated { "accepted" } else { "rejected" });

    // Get current data
    if let Some(book) = cache.get("BTCUSDT") {
        println!("\nCurrent order book:");
        println!("  Best bid: ${:.2}", book.bids[0].0);
        println!("  Best ask: ${:.2}", book.asks[0].0);
        println!("  Version: {}", cache.version("BTCUSDT"));
    }
}
```

## Cache Invalidation Strategies

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Invalidation strategy
#[derive(Debug, Clone)]
enum InvalidationStrategy {
    /// Time-based invalidation (TTL)
    TimeToLive(Duration),
    /// Invalidation by access count
    MaxHits(u32),
    /// Event-based invalidation (e.g., new trade)
    EventBased,
    /// Never invalidate (for static data)
    Never,
}

struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    hits: u32,
    strategy: InvalidationStrategy,
}

impl<T: Clone> CacheEntry<T> {
    fn is_valid(&self) -> bool {
        match &self.strategy {
            InvalidationStrategy::TimeToLive(ttl) => {
                self.created_at.elapsed() < *ttl
            }
            InvalidationStrategy::MaxHits(max) => {
                self.hits < *max
            }
            InvalidationStrategy::EventBased => {
                true  // Managed externally
            }
            InvalidationStrategy::Never => {
                true
            }
        }
    }

    fn access(&mut self) -> Option<T> {
        if self.is_valid() {
            self.hits += 1;
            Some(self.value.clone())
        } else {
            None
        }
    }
}

struct TradingCache<T> {
    entries: HashMap<String, CacheEntry<T>>,
}

impl<T: Clone> TradingCache<T> {
    fn new() -> Self {
        TradingCache {
            entries: HashMap::new(),
        }
    }

    fn set(&mut self, key: String, value: T, strategy: InvalidationStrategy) {
        self.entries.insert(key, CacheEntry {
            value,
            created_at: Instant::now(),
            hits: 0,
            strategy,
        });
    }

    fn get(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.entries.get_mut(key) {
            return entry.access();
        }
        None
    }

    /// Event-based invalidation (for EventBased strategy)
    fn invalidate_by_event(&mut self, pattern: &str) {
        self.entries.retain(|key, _| !key.contains(pattern));
    }

    fn cleanup(&mut self) {
        self.entries.retain(|_, entry| entry.is_valid());
    }
}

fn main() {
    let mut cache: TradingCache<f64> = TradingCache::new();

    println!("=== Invalidation Strategies ===\n");

    // 1. TTL strategy for prices (updated every 100ms)
    cache.set(
        "price:BTCUSDT".to_string(),
        42500.0,
        InvalidationStrategy::TimeToLive(Duration::from_millis(100)),
    );

    // 2. MaxHits for expensive computations (cache for 10 accesses)
    cache.set(
        "risk:portfolio".to_string(),
        0.15,
        InvalidationStrategy::MaxHits(10),
    );

    // 3. EventBased for trade-dependent data
    cache.set(
        "volume:BTCUSDT:1h".to_string(),
        1500.5,
        InvalidationStrategy::EventBased,
    );

    // 4. Never for static data
    cache.set(
        "config:min_order_size".to_string(),
        0.001,
        InvalidationStrategy::Never,
    );

    // Test TTL
    println!("1. TTL Test:");
    if let Some(price) = cache.get("price:BTCUSDT") {
        println!("   Price (fresh): ${:.2}", price);
    }
    std::thread::sleep(Duration::from_millis(150));
    match cache.get("price:BTCUSDT") {
        Some(p) => println!("   Price (after TTL): ${:.2}", p),
        None => println!("   Price (after TTL): cache expired"),
    }

    // Test MaxHits
    println!("\n2. MaxHits Test:");
    for i in 1..=12 {
        match cache.get("risk:portfolio") {
            Some(risk) => println!("   Request #{}: risk = {:.2}%", i, risk * 100.0),
            None => println!("   Request #{}: cache exhausted", i),
        }
    }

    // Test EventBased
    println!("\n3. EventBased Test:");
    if let Some(volume) = cache.get("volume:BTCUSDT:1h") {
        println!("   Volume before trade: {:.2}", volume);
    }

    // Simulate new trade — invalidate related data
    println!("   [EVENT] New trade on BTCUSDT");
    cache.invalidate_by_event("BTCUSDT");

    match cache.get("volume:BTCUSDT:1h") {
        Some(v) => println!("   Volume after trade: {:.2}", v),
        None => println!("   Volume after trade: cache invalidated"),
    }

    // Test Never
    println!("\n4. Never Test:");
    for _ in 0..100 {
        let _ = cache.get("config:min_order_size");
    }
    if let Some(size) = cache.get("config:min_order_size") {
        println!("   Min order size (after 100 requests): {}", size);
    }
}
```

## Caching with Write-Through and Write-Back

```rust
use std::collections::HashMap;
use std::time::Instant;

/// Write mode
#[derive(Debug, Clone, Copy)]
enum WriteMode {
    /// Synchronous write to storage
    WriteThrough,
    /// Deferred write (buffering)
    WriteBack,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

struct PositionCache {
    cache: HashMap<String, Position>,
    dirty: HashMap<String, Instant>,  // Modified entries for WriteBack
    mode: WriteMode,
}

impl PositionCache {
    fn new(mode: WriteMode) -> Self {
        PositionCache {
            cache: HashMap::new(),
            dirty: HashMap::new(),
            mode,
        }
    }

    fn update_position(&mut self, position: Position) {
        let symbol = position.symbol.clone();

        match self.mode {
            WriteMode::WriteThrough => {
                // First write to "database"
                println!("[WRITE-THROUGH] Saving {} to DB", symbol);
                self.simulate_db_write(&position);
                // Then update cache
                self.cache.insert(symbol, position);
            }
            WriteMode::WriteBack => {
                // Only update cache
                println!("[WRITE-BACK] Updating {} in cache", symbol);
                self.cache.insert(symbol.clone(), position);
                // Mark as "dirty" entry
                self.dirty.insert(symbol, Instant::now());
            }
        }
    }

    fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.cache.get(symbol)
    }

    /// Flush "dirty" entries to storage (for WriteBack)
    fn flush(&mut self) {
        if matches!(self.mode, WriteMode::WriteBack) {
            println!("\n[FLUSH] Saving {} modified entries", self.dirty.len());
            for (symbol, _) in self.dirty.drain() {
                if let Some(position) = self.cache.get(&symbol) {
                    self.simulate_db_write(position);
                }
            }
        }
    }

    fn simulate_db_write(&self, position: &Position) {
        println!(
            "  -> DB: {} qty={:.4} @ ${:.2}",
            position.symbol, position.quantity, position.entry_price
        );
    }

    fn dirty_count(&self) -> usize {
        self.dirty.len()
    }
}

fn main() {
    println!("=== Write-Through vs Write-Back ===\n");

    // Write-Through: each change goes to DB immediately
    println!("--- Write-Through mode ---");
    let mut wt_cache = PositionCache::new(WriteMode::WriteThrough);

    wt_cache.update_position(Position {
        symbol: "BTCUSDT".to_string(),
        quantity: 0.5,
        entry_price: 42500.0,
    });

    wt_cache.update_position(Position {
        symbol: "ETHUSDT".to_string(),
        quantity: 5.0,
        entry_price: 2500.0,
    });

    // Write-Back: changes accumulate in cache
    println!("\n--- Write-Back mode ---");
    let mut wb_cache = PositionCache::new(WriteMode::WriteBack);

    wb_cache.update_position(Position {
        symbol: "BTCUSDT".to_string(),
        quantity: 0.5,
        entry_price: 42500.0,
    });

    wb_cache.update_position(Position {
        symbol: "ETHUSDT".to_string(),
        quantity: 5.0,
        entry_price: 2500.0,
    });

    wb_cache.update_position(Position {
        symbol: "SOLUSDT".to_string(),
        quantity: 100.0,
        entry_price: 100.0,
    });

    println!("\nAccumulated changes: {}", wb_cache.dirty_count());

    // Periodic flush to DB
    wb_cache.flush();
}
```

## Practical Example: Market Data Cache

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct MarketTick {
    symbol: String,
    bid: f64,
    ask: f64,
    last_trade: f64,
    volume_24h: f64,
    timestamp: u64,
}

#[derive(Clone, Debug)]
struct OHLCV {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

struct MarketDataCache {
    ticks: HashMap<String, (MarketTick, Instant)>,
    candles: HashMap<String, (Vec<OHLCV>, Instant)>,
    tick_ttl: Duration,
    candle_ttl: Duration,

    // Statistics
    tick_hits: u64,
    tick_misses: u64,
    candle_hits: u64,
    candle_misses: u64,
}

impl MarketDataCache {
    fn new() -> Self {
        MarketDataCache {
            ticks: HashMap::new(),
            candles: HashMap::new(),
            tick_ttl: Duration::from_millis(50),   // Ticks expire quickly
            candle_ttl: Duration::from_secs(60),   // Candles live longer
            tick_hits: 0,
            tick_misses: 0,
            candle_hits: 0,
            candle_misses: 0,
        }
    }

    /// Get latest tick
    fn get_tick(&mut self, symbol: &str) -> Option<MarketTick> {
        if let Some((tick, cached_at)) = self.ticks.get(symbol) {
            if cached_at.elapsed() < self.tick_ttl {
                self.tick_hits += 1;
                return Some(tick.clone());
            }
        }
        self.tick_misses += 1;
        None
    }

    /// Update tick
    fn update_tick(&mut self, tick: MarketTick) {
        let symbol = tick.symbol.clone();
        self.ticks.insert(symbol, (tick, Instant::now()));
    }

    /// Get candles
    fn get_candles(&mut self, symbol: &str, timeframe: &str) -> Option<Vec<OHLCV>> {
        let key = format!("{}:{}", symbol, timeframe);
        if let Some((candles, cached_at)) = self.candles.get(&key) {
            if cached_at.elapsed() < self.candle_ttl {
                self.candle_hits += 1;
                return Some(candles.clone());
            }
        }
        self.candle_misses += 1;
        None
    }

    /// Update candles
    fn update_candles(&mut self, symbol: &str, timeframe: &str, candles: Vec<OHLCV>) {
        let key = format!("{}:{}", symbol, timeframe);
        self.candles.insert(key, (candles, Instant::now()));
    }

    /// Invalidation on new trade
    fn on_trade(&mut self, symbol: &str) {
        // Invalidate all related candles
        let prefix = format!("{}:", symbol);
        self.candles.retain(|k, _| !k.starts_with(&prefix));
    }

    fn print_stats(&self) {
        let tick_total = self.tick_hits + self.tick_misses;
        let candle_total = self.candle_hits + self.candle_misses;

        println!("\n=== Cache Statistics ===");
        println!("Ticks:");
        println!("  Hits: {}", self.tick_hits);
        println!("  Misses: {}", self.tick_misses);
        if tick_total > 0 {
            println!("  Efficiency: {:.1}%",
                     self.tick_hits as f64 / tick_total as f64 * 100.0);
        }

        println!("Candles:");
        println!("  Hits: {}", self.candle_hits);
        println!("  Misses: {}", self.candle_misses);
        if candle_total > 0 {
            println!("  Efficiency: {:.1}%",
                     self.candle_hits as f64 / candle_total as f64 * 100.0);
        }
    }
}

fn main() {
    let mut cache = MarketDataCache::new();

    println!("=== Market Data Cache ===\n");

    // Simulate tick stream
    for i in 0..5 {
        let tick = MarketTick {
            symbol: "BTCUSDT".to_string(),
            bid: 42500.0 + i as f64,
            ask: 42501.0 + i as f64,
            last_trade: 42500.5 + i as f64,
            volume_24h: 15000.0,
            timestamp: 1000 + i,
        };
        cache.update_tick(tick);
    }

    // Tick cache requests
    println!("Tick requests:");
    for _ in 0..10 {
        match cache.get_tick("BTCUSDT") {
            Some(tick) => println!("  BTC bid: ${:.2}", tick.bid),
            None => println!("  Cache stale, need exchange request"),
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    // Load candles
    let candles = vec![
        OHLCV { open: 42000.0, high: 42500.0, low: 41800.0, close: 42400.0, volume: 100.0 },
        OHLCV { open: 42400.0, high: 42600.0, low: 42200.0, close: 42500.0, volume: 150.0 },
    ];
    cache.update_candles("BTCUSDT", "1h", candles);

    println!("\nCandle requests:");
    for i in 0..5 {
        match cache.get_candles("BTCUSDT", "1h") {
            Some(c) => println!("  Request #{}: {} candles", i + 1, c.len()),
            None => println!("  Request #{}: miss", i + 1),
        }
    }

    // Simulate new trade
    println!("\n[EVENT] New trade on BTCUSDT");
    cache.on_trade("BTCUSDT");

    match cache.get_candles("BTCUSDT", "1h") {
        Some(_) => println!("Candles: in cache"),
        None => println!("Candles: invalidated"),
    }

    cache.print_stats();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Result Caching** | Storing computed values for reuse |
| **TTL (Time To Live)** | Automatic time-based invalidation |
| **Multi-Level Cache** | L1 (memory) → L2 (shared) → L3 (DB) |
| **Versioning** | Tracking data freshness |
| **Write-Through** | Synchronous write to cache and storage |
| **Write-Back** | Deferred write with buffering |
| **Event-based Invalidation** | Cache reset on events |

## Practical Exercises

1. **Order Cache with Priorities**: Implement a cache where orders with larger volume have longer TTL.

2. **Cache Warmup**: Create a system for preloading frequently used data at startup.

3. **Distributed Cache**: Implement a simple cache synchronization protocol between multiple nodes.

## Homework

1. **Adaptive TTL**: Implement a cache that automatically adjusts TTL based on market volatility — TTL decreases during high volatility.

2. **Predictive Cache**: Create a system that predicts what data a trader will need and loads it in advance.

3. **Cache Metrics**: Implement a dashboard with statistics: hit rate, latency, memory usage, eviction rate.

4. **Hybrid Cache**: Combine multiple invalidation strategies (TTL + Event + Version) into a single smart cache for a trading system.

## Navigation

[← Previous day](../320-*/en.md) | [Next day →](../322-*/en.md)
