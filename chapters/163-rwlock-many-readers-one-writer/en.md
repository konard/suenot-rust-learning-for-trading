# Day 163: RwLock — Many Readers, One Writer

## Trading Analogy

Imagine a trading terminal displaying real-time market data:
- **Hundreds of traders** are watching the order book simultaneously — they only **read** prices
- **One market maker** occasionally updates bid/ask quotes — they need to **write** new prices
- Reading doesn't block other readers — everyone can see prices at the same time
- Writing requires exclusive access — no one should read stale data while prices update

This is exactly what `RwLock` (Read-Write Lock) provides:
- **Multiple concurrent readers** — great for analytics, charting, display
- **Exclusive writer access** — ensures data consistency when updating
- **Higher throughput** than Mutex when reads vastly outnumber writes

In trading systems, read-heavy workloads are common:
- Price feeds are read by many components but updated by one
- Portfolio balances are checked constantly but modified rarely
- Order books are displayed to many users but updated by matching engine

## What is RwLock?

`RwLock<T>` is a synchronization primitive that allows:
- **Many simultaneous readers** (`read()` locks)
- **One exclusive writer** (`write()` lock)

```rust
use std::sync::RwLock;

fn main() {
    let price = RwLock::new(42000.0);

    // Reading (shared access)
    {
        let read_guard = price.read().unwrap();
        println!("Current price: ${:.2}", *read_guard);
    } // read lock released here

    // Writing (exclusive access)
    {
        let mut write_guard = price.write().unwrap();
        *write_guard = 42500.0;
        println!("Price updated to: ${:.2}", *write_guard);
    } // write lock released here
}
```

## RwLock vs Mutex

| Feature | `Mutex<T>` | `RwLock<T>` |
|---------|------------|-------------|
| Readers | One at a time | Many concurrent |
| Writers | One at a time | One at a time |
| Use case | Frequent writes | Rare writes, many reads |
| Overhead | Lower | Higher |
| Trading example | Order execution | Price display |

```rust
use std::sync::{Arc, RwLock, Mutex};
use std::thread;

fn main() {
    // RwLock - better when many threads read, few write
    let market_data = Arc::new(RwLock::new(42000.0));

    // Mutex - simpler when read/write ratio is similar
    let order_count = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    // Many readers accessing price
    for i in 0..5 {
        let data = Arc::clone(&market_data);
        handles.push(thread::spawn(move || {
            let price = data.read().unwrap();
            println!("Reader {}: price = ${:.2}", i, *price);
        }));
    }

    // One writer updating price
    let data = Arc::clone(&market_data);
    handles.push(thread::spawn(move || {
        let mut price = data.write().unwrap();
        *price = 42100.0;
        println!("Writer: updated price to ${:.2}", *price);
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Practical Example: Real-Time Price Monitor

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

struct MarketPrice {
    symbol: String,
    bid: f64,
    ask: f64,
    last_update: u64,
}

impl MarketPrice {
    fn new(symbol: &str, bid: f64, ask: f64) -> Self {
        MarketPrice {
            symbol: symbol.to_string(),
            bid,
            ask,
            last_update: 0,
        }
    }

    fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║     REAL-TIME PRICE MONITOR           ║");
    println!("╚═══════════════════════════════════════╝\n");

    let btc_price = Arc::new(RwLock::new(MarketPrice::new("BTC/USD", 41950.0, 42050.0)));

    let mut handles = vec![];

    // Price feed simulator (writer)
    let price_feed = Arc::clone(&btc_price);
    handles.push(thread::spawn(move || {
        for tick in 1..=3 {
            thread::sleep(Duration::from_millis(100));
            let mut price = price_feed.write().unwrap();
            price.bid += 25.0;
            price.ask += 25.0;
            price.last_update = tick;
            println!("[FEED] Tick {}: Updated to bid=${:.2}, ask=${:.2}",
                     tick, price.bid, price.ask);
        }
    }));

    // Chart display (reader 1)
    let chart_data = Arc::clone(&btc_price);
    handles.push(thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(50));
            let price = chart_data.read().unwrap();
            println!("[CHART] Mid price: ${:.2}", price.mid_price());
        }
    }));

    // Spread analyzer (reader 2)
    let spread_data = Arc::clone(&btc_price);
    handles.push(thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(60));
            let price = spread_data.read().unwrap();
            println!("[SPREAD] Current spread: ${:.2}", price.spread());
        }
    }));

    // Alert system (reader 3)
    let alert_data = Arc::clone(&btc_price);
    handles.push(thread::spawn(move || {
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(70));
            let price = alert_data.read().unwrap();
            if price.mid_price() > 42000.0 {
                println!("[ALERT] Price above $42,000!");
            }
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\n--- Final State ---");
    let final_price = btc_price.read().unwrap();
    println!("Symbol: {}", final_price.symbol);
    println!("Bid: ${:.2}", final_price.bid);
    println!("Ask: ${:.2}", final_price.ask);
    println!("Spread: ${:.2}", final_price.spread());
}
```

## Practical Example: Portfolio Tracker

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::collections::HashMap;

struct Portfolio {
    holdings: HashMap<String, f64>,
    total_value: f64,
}

impl Portfolio {
    fn new() -> Self {
        Portfolio {
            holdings: HashMap::new(),
            total_value: 0.0,
        }
    }

    fn add_position(&mut self, symbol: &str, value: f64) {
        *self.holdings.entry(symbol.to_string()).or_insert(0.0) += value;
        self.recalculate_total();
    }

    fn recalculate_total(&mut self) {
        self.total_value = self.holdings.values().sum();
    }

    fn get_allocation(&self, symbol: &str) -> f64 {
        if self.total_value == 0.0 {
            return 0.0;
        }
        let holding = self.holdings.get(symbol).unwrap_or(&0.0);
        (holding / self.total_value) * 100.0
    }
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║       PORTFOLIO TRACKER               ║");
    println!("╚═══════════════════════════════════════╝\n");

    let portfolio = Arc::new(RwLock::new(Portfolio::new()));

    // Initialize portfolio
    {
        let mut p = portfolio.write().unwrap();
        p.add_position("BTC", 50000.0);
        p.add_position("ETH", 30000.0);
        p.add_position("SOL", 20000.0);
    }

    let mut handles = vec![];

    // Multiple dashboard readers
    for i in 1..=3 {
        let p = Arc::clone(&portfolio);
        handles.push(thread::spawn(move || {
            let portfolio = p.read().unwrap();
            println!("[Dashboard {}] Total Value: ${:.2}", i, portfolio.total_value);
            println!("[Dashboard {}] BTC allocation: {:.1}%", i, portfolio.get_allocation("BTC"));
        }));
    }

    // Risk analyzer (reader)
    let risk_portfolio = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let p = risk_portfolio.read().unwrap();
        let btc_alloc = p.get_allocation("BTC");
        if btc_alloc > 40.0 {
            println!("[RISK] Warning: BTC concentration at {:.1}%", btc_alloc);
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Practical Example: Order Book Viewer

```rust
use std::sync::{Arc, RwLock};
use std::thread;

struct OrderBook {
    bids: Vec<(f64, f64)>, // (price, quantity)
    asks: Vec<(f64, f64)>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: vec![
                (41990.0, 1.5),
                (41980.0, 2.3),
                (41970.0, 0.8),
            ],
            asks: vec![
                (42010.0, 1.2),
                (42020.0, 1.8),
                (42030.0, 2.1),
            ],
        }
    }

    fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids.first().copied()
    }

    fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks.first().copied()
    }

    fn spread(&self) -> f64 {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => ask - bid,
            _ => 0.0,
        }
    }

    fn update_bid(&mut self, price: f64, quantity: f64) {
        if let Some(pos) = self.bids.iter().position(|(p, _)| *p == price) {
            self.bids[pos].1 = quantity;
        } else {
            self.bids.push((price, quantity));
            self.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        }
    }
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║       ORDER BOOK VIEWER               ║");
    println!("╚═══════════════════════════════════════╝\n");

    let order_book = Arc::new(RwLock::new(OrderBook::new()));

    let mut handles = vec![];

    // Multiple UI readers
    for i in 1..=4 {
        let book = Arc::clone(&order_book);
        handles.push(thread::spawn(move || {
            let ob = book.read().unwrap();
            if let Some((price, qty)) = ob.best_bid() {
                println!("[UI {}] Best Bid: ${:.2} x {:.2}", i, price, qty);
            }
            if let Some((price, qty)) = ob.best_ask() {
                println!("[UI {}] Best Ask: ${:.2} x {:.2}", i, price, qty);
            }
            println!("[UI {}] Spread: ${:.2}", i, ob.spread());
        }));
    }

    // Matching engine (writer) - updates order book
    let book = Arc::clone(&order_book);
    handles.push(thread::spawn(move || {
        let mut ob = book.write().unwrap();
        ob.update_bid(41995.0, 3.0);
        println!("[ENGINE] Updated bid: $41995.00 x 3.00");
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## try_read and try_write

Non-blocking alternatives that return immediately:

```rust
use std::sync::RwLock;

fn main() {
    let price = RwLock::new(42000.0);

    // Acquire write lock
    let _write_guard = price.write().unwrap();

    // try_read returns Err if write lock is held
    match price.try_read() {
        Ok(guard) => println!("Price: ${:.2}", *guard),
        Err(_) => println!("Cannot read - write in progress"),
    }

    // try_write returns Err if any lock is held
    match price.try_write() {
        Ok(mut guard) => *guard = 42500.0,
        Err(_) => println!("Cannot write - lock held"),
    }
}
```

## Practical Example: Trading Signal Cache

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Debug)]
struct Signal {
    symbol: String,
    direction: String,
    strength: f64,
    timestamp: u64,
}

struct SignalCache {
    signals: HashMap<String, Signal>,
    last_update: u64,
}

impl SignalCache {
    fn new() -> Self {
        SignalCache {
            signals: HashMap::new(),
            last_update: 0,
        }
    }

    fn get_signal(&self, symbol: &str) -> Option<&Signal> {
        self.signals.get(symbol)
    }

    fn update_signal(&mut self, signal: Signal) {
        self.last_update += 1;
        self.signals.insert(signal.symbol.clone(), signal);
    }

    fn all_signals(&self) -> Vec<&Signal> {
        self.signals.values().collect()
    }
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║     TRADING SIGNAL CACHE              ║");
    println!("╚═══════════════════════════════════════╝\n");

    let cache = Arc::new(RwLock::new(SignalCache::new()));

    // Initialize with some signals
    {
        let mut c = cache.write().unwrap();
        c.update_signal(Signal {
            symbol: "BTC".to_string(),
            direction: "LONG".to_string(),
            strength: 0.8,
            timestamp: 1,
        });
        c.update_signal(Signal {
            symbol: "ETH".to_string(),
            direction: "SHORT".to_string(),
            strength: 0.6,
            timestamp: 1,
        });
    }

    let mut handles = vec![];

    // Signal generator (writer)
    let writer_cache = Arc::clone(&cache);
    handles.push(thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        let mut c = writer_cache.write().unwrap();
        c.update_signal(Signal {
            symbol: "SOL".to_string(),
            direction: "LONG".to_string(),
            strength: 0.9,
            timestamp: 2,
        });
        println!("[GENERATOR] New signal: SOL LONG (0.9)");
    }));

    // Multiple strategy readers
    for i in 1..=3 {
        let reader_cache = Arc::clone(&cache);
        handles.push(thread::spawn(move || {
            let c = reader_cache.read().unwrap();
            for signal in c.all_signals() {
                println!("[Strategy {}] {} {} (strength: {:.1})",
                         i, signal.symbol, signal.direction, signal.strength);
            }
        }));
    }

    // Risk monitor (reader)
    let risk_cache = Arc::clone(&cache);
    handles.push(thread::spawn(move || {
        let c = risk_cache.read().unwrap();
        let strong_signals: Vec<_> = c.all_signals()
            .into_iter()
            .filter(|s| s.strength > 0.7)
            .collect();
        println!("[RISK] Strong signals count: {}", strong_signals.len());
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Common Patterns

### Pattern 1: Read-Heavy Configuration

```rust
use std::sync::{Arc, RwLock};

struct TradingConfig {
    max_position_size: f64,
    risk_per_trade: f64,
    allowed_symbols: Vec<String>,
}

fn main() {
    let config = Arc::new(RwLock::new(TradingConfig {
        max_position_size: 10000.0,
        risk_per_trade: 0.02,
        allowed_symbols: vec!["BTC".to_string(), "ETH".to_string()],
    }));

    // Many readers checking config
    let cfg = config.read().unwrap();
    println!("Max position: ${:.2}", cfg.max_position_size);
    println!("Risk per trade: {:.1}%", cfg.risk_per_trade * 100.0);

    // Rare config update
    drop(cfg); // Release read lock first
    {
        let mut cfg = config.write().unwrap();
        cfg.allowed_symbols.push("SOL".to_string());
        println!("Added SOL to allowed symbols");
    }
}
```

### Pattern 2: Snapshot for Long Calculations

```rust
use std::sync::{Arc, RwLock};

struct MarketData {
    prices: Vec<f64>,
}

fn main() {
    let data = Arc::new(RwLock::new(MarketData {
        prices: vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0],
    }));

    // Take a snapshot for calculations
    let prices_snapshot: Vec<f64>;
    {
        let d = data.read().unwrap();
        prices_snapshot = d.prices.clone(); // Clone and release lock
    }

    // Long calculation without holding lock
    let avg: f64 = prices_snapshot.iter().sum::<f64>() / prices_snapshot.len() as f64;
    let variance: f64 = prices_snapshot.iter()
        .map(|p| (p - avg).powi(2))
        .sum::<f64>() / prices_snapshot.len() as f64;
    let std_dev = variance.sqrt();

    println!("Average: ${:.2}", avg);
    println!("Std Dev: ${:.2}", std_dev);
}
```

## Potential Deadlocks with RwLock

Be careful with lock ordering:

```rust
use std::sync::RwLock;

fn main() {
    let price = RwLock::new(42000.0);

    // DANGER: Don't upgrade read to write!
    // This will deadlock:
    // let read_guard = price.read().unwrap();
    // let write_guard = price.write().unwrap(); // DEADLOCK!

    // Correct approach: release read, then acquire write
    {
        let read_guard = price.read().unwrap();
        let current = *read_guard;
        drop(read_guard); // Release read lock

        let mut write_guard = price.write().unwrap();
        *write_guard = current + 100.0;
    }

    println!("Price: ${:.2}", *price.read().unwrap());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `RwLock<T>` | Lock allowing many readers OR one writer |
| `read()` | Acquire shared read lock |
| `write()` | Acquire exclusive write lock |
| `try_read()` | Non-blocking read attempt |
| `try_write()` | Non-blocking write attempt |
| `Arc<RwLock<T>>` | Thread-safe shared RwLock |

## When to Use RwLock vs Mutex

- Use `RwLock` when reads vastly outnumber writes (10:1 or more)
- Use `Mutex` when read/write ratio is balanced
- `RwLock` has higher overhead per operation
- `Mutex` is simpler and avoids writer starvation

## Homework

1. **Price Aggregator**: Create a system where one thread updates prices for multiple assets while multiple analysis threads read and calculate correlations. Use `RwLock<HashMap<String, f64>>` for the price cache.

2. **Strategy Configuration Manager**: Build a trading strategy that reads its configuration (risk limits, position sizes) frequently but allows for runtime configuration updates. Implement safe configuration hot-reloading.

3. **Order Book Depth Cache**: Implement an order book depth cache that is read by multiple display components (web UI, CLI, alerts) and updated by a single market data feed. Include methods for getting top N levels and calculating total liquidity.

4. **Session State Tracker**: Create a trading session tracker that multiple components read (P&L calculator, risk monitor, reporting) while a single order manager updates. Track open positions, realized P&L, and trade count.

## Navigation

[← Previous day](../162-arc-mutex-shared-mutable-structure/en.md) | [Next day →](../164-deadlock-when-threads-block/en.md)
