# Day 193: Async Mutex: tokio::sync::Mutex

## Trading Analogy

Imagine a trading floor where hundreds of traders are simultaneously trying to update a shared portfolio. In the synchronous world, each trader would be forced to wait in line, blocking their entire workflow. It's as if a trader waiting for terminal access couldn't even check news or analyze charts.

In the async world with `tokio::sync::Mutex`, the trader **yields the waiting to the system**: while waiting for access to a shared resource, the system can execute other tasks — fetching market data, processing signals, or sending notifications. As soon as the resource becomes available, the trader instantly continues their work.

**Key difference from `std::sync::Mutex`:**
- `std::sync::Mutex` — blocks the entire thread while waiting
- `tokio::sync::Mutex` — frees the thread for other tasks while waiting for the lock

## Why Do We Need Async Mutex?

In async applications, using `std::sync::Mutex` can lead to serious problems:

```rust
// BAD: std::sync::Mutex in async code
use std::sync::Mutex;

async fn bad_example(mutex: &Mutex<i32>) {
    let guard = mutex.lock().unwrap(); // Blocks the entire thread!

    // If we call .await here, other tasks on this thread will freeze
    some_async_operation().await; // DANGEROUS!

    // guard is still held
}
```

**Problem:** when `std::sync::Mutex` is locked, the entire thread is blocked. In an async runtime like tokio, where many tasks execute on a limited number of threads, this can lead to:
- Hanging of other tasks on the same thread
- Potential deadlocks
- Performance degradation

## Basics of tokio::sync::Mutex

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug)]
struct Portfolio {
    cash: f64,
    btc_amount: f64,
    last_update: String,
}

#[tokio::main]
async fn main() {
    // Create async mutex with portfolio
    let portfolio = Arc::new(Mutex::new(Portfolio {
        cash: 100_000.0,
        btc_amount: 0.0,
        last_update: "init".to_string(),
    }));

    // lock() returns a Future — needs .await
    let mut guard = portfolio.lock().await;
    guard.cash -= 42_000.0;
    guard.btc_amount += 1.0;
    guard.last_update = "bought BTC".to_string();

    println!("Portfolio: {:?}", *guard);
    // guard is released here when going out of scope
}
```

## Comparing std::sync::Mutex and tokio::sync::Mutex

```rust
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // std::sync::Mutex — synchronous blocking
    let std_mutex = Arc::new(StdMutex::new(100.0_f64));
    {
        let guard = std_mutex.lock().unwrap(); // Blocks the thread!
        println!("std::sync::Mutex: {}", *guard);
    }

    // tokio::sync::Mutex — asynchronous blocking
    let tokio_mutex = Arc::new(TokioMutex::new(100.0_f64));
    {
        let guard = tokio_mutex.lock().await; // Frees thread while waiting
        println!("tokio::sync::Mutex: {}", *guard);
    }
}
```

## Practical Example: Async Trading Engine

```rust
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,      // "buy" or "sell"
    price: f64,
    quantity: f64,
    status: String,    // "pending", "filled", "cancelled"
}

#[derive(Debug)]
struct TradingEngine {
    orders: Mutex<HashMap<u64, Order>>,
    next_order_id: Mutex<u64>,
    balances: Mutex<HashMap<String, f64>>,
}

impl TradingEngine {
    fn new() -> Self {
        let mut balances = HashMap::new();
        balances.insert("USD".to_string(), 100_000.0);
        balances.insert("BTC".to_string(), 0.0);
        balances.insert("ETH".to_string(), 0.0);

        TradingEngine {
            orders: Mutex::new(HashMap::new()),
            next_order_id: Mutex::new(1),
            balances: Mutex::new(balances),
        }
    }

    async fn place_order(&self, symbol: &str, side: &str, price: f64, quantity: f64) -> Result<u64, String> {
        // Generate order ID
        let order_id = {
            let mut id_guard = self.next_order_id.lock().await;
            let id = *id_guard;
            *id_guard += 1;
            id
        };

        // Check balance
        {
            let balances = self.balances.lock().await;
            if side == "buy" {
                let usd_balance = balances.get("USD").unwrap_or(&0.0);
                let required = price * quantity;
                if *usd_balance < required {
                    return Err(format!("Insufficient USD: need {}, have {}", required, usd_balance));
                }
            } else {
                let asset_balance = balances.get(symbol).unwrap_or(&0.0);
                if *asset_balance < quantity {
                    return Err(format!("Insufficient {}: need {}, have {}", symbol, quantity, asset_balance));
                }
            }
        }

        // Create and save order
        let order = Order {
            id: order_id,
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            quantity,
            status: "pending".to_string(),
        };

        {
            let mut orders = self.orders.lock().await;
            orders.insert(order_id, order);
        }

        println!("Order #{} created: {} {} {} @ ${}", order_id, side, quantity, symbol, price);
        Ok(order_id)
    }

    async fn execute_order(&self, order_id: u64) -> Result<(), String> {
        // Get order
        let order = {
            let orders = self.orders.lock().await;
            orders.get(&order_id).cloned().ok_or("Order not found")?
        };

        // Update balances
        {
            let mut balances = self.balances.lock().await;
            let total = order.price * order.quantity;

            if order.side == "buy" {
                *balances.get_mut("USD").unwrap() -= total;
                *balances.entry(order.symbol.clone()).or_insert(0.0) += order.quantity;
            } else {
                *balances.get_mut("USD").unwrap() += total;
                *balances.get_mut(&order.symbol).unwrap() -= order.quantity;
            }
        }

        // Update order status
        {
            let mut orders = self.orders.lock().await;
            if let Some(o) = orders.get_mut(&order_id) {
                o.status = "filled".to_string();
            }
        }

        println!("Order #{} executed", order_id);
        Ok(())
    }

    async fn get_balance(&self, asset: &str) -> f64 {
        let balances = self.balances.lock().await;
        *balances.get(asset).unwrap_or(&0.0)
    }

    async fn get_all_balances(&self) -> HashMap<String, f64> {
        let balances = self.balances.lock().await;
        balances.clone()
    }
}

#[tokio::main]
async fn main() {
    let engine = Arc::new(TradingEngine::new());

    // Launch several parallel trading operations
    let e1 = Arc::clone(&engine);
    let e2 = Arc::clone(&engine);
    let e3 = Arc::clone(&engine);

    let trader1 = tokio::spawn(async move {
        for i in 0..3 {
            let price = 42000.0 + i as f64 * 100.0;
            match e1.place_order("BTC", "buy", price, 0.1).await {
                Ok(id) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    let _ = e1.execute_order(id).await;
                }
                Err(e) => println!("Trader 1 error: {}", e),
            }
        }
    });

    let trader2 = tokio::spawn(async move {
        for i in 0..3 {
            let price = 2500.0 + i as f64 * 50.0;
            match e2.place_order("ETH", "buy", price, 1.0).await {
                Ok(id) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
                    let _ = e2.execute_order(id).await;
                }
                Err(e) => println!("Trader 2 error: {}", e),
            }
        }
    });

    let monitor = tokio::spawn(async move {
        for _ in 0..5 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let balances = e3.get_all_balances().await;
            println!("Balances: {:?}", balances);
        }
    });

    // Wait for all tasks to complete
    let _ = tokio::join!(trader1, trader2, monitor);

    println!("\nFinal balances: {:?}", engine.get_all_balances().await);
}
```

## try_lock() — Non-blocking Attempt

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug)]
struct MarketData {
    last_price: f64,
    volume_24h: f64,
}

#[tokio::main]
async fn main() {
    let market_data = Arc::new(Mutex::new(MarketData {
        last_price: 42000.0,
        volume_24h: 1_000_000.0,
    }));

    let md1 = Arc::clone(&market_data);
    let md2 = Arc::clone(&market_data);

    // Data update task
    let updater = tokio::spawn(async move {
        loop {
            {
                let mut data = md1.lock().await;
                data.last_price += 10.0;
                data.volume_24h += 100.0;
                println!("Updated: price = {}", data.last_price);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // Data reading task (non-blocking)
    let reader = tokio::spawn(async move {
        for i in 0..10 {
            // try_lock() doesn't block — returns result immediately
            match md2.try_lock() {
                Ok(data) => {
                    println!("Read (attempt {}): price = {}, volume = {}",
                             i + 1, data.last_price, data.volume_24h);
                }
                Err(_) => {
                    println!("Attempt {}: data locked, skipping", i + 1);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });

    // Wait only for reader, updater runs infinitely
    let _ = reader.await;
    updater.abort();
}
```

## Pattern: Holding Lock Across .await

**Important:** `tokio::sync::Mutex` is safe to hold across `.await`:

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

async fn fetch_price(symbol: &str) -> f64 {
    // Simulate HTTP request
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        _ => 0.0,
    }
}

#[derive(Debug)]
struct PriceCache {
    prices: std::collections::HashMap<String, f64>,
}

#[tokio::main]
async fn main() {
    let cache = Arc::new(Mutex::new(PriceCache {
        prices: std::collections::HashMap::new(),
    }));

    // Safe to hold tokio::sync::Mutex across .await
    let mut guard = cache.lock().await;

    // Can do async operations while holding the lock
    let btc_price = fetch_price("BTC").await;
    guard.prices.insert("BTC".to_string(), btc_price);

    let eth_price = fetch_price("ETH").await;
    guard.prices.insert("ETH".to_string(), eth_price);

    println!("Price cache: {:?}", guard.prices);
    // guard is released here
}
```

## When to Use std::sync::Mutex vs tokio::sync::Mutex

| Scenario | Recommendation |
|----------|----------------|
| Short operations without `.await` inside critical section | `std::sync::Mutex` (faster) |
| Need `.await` inside critical section | `tokio::sync::Mutex` (required) |
| Many concurrent async tasks | `tokio::sync::Mutex` |
| Synchronous code or minimal async | `std::sync::Mutex` |

```rust
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;
use std::sync::Arc;

// Good: std::sync::Mutex for quick operations
async fn quick_update(counter: &Arc<StdMutex<u64>>) {
    let mut guard = counter.lock().unwrap();
    *guard += 1;
    // No .await inside — this is fine
}

// Good: tokio::sync::Mutex when .await is needed
async fn slow_update(cache: &Arc<TokioMutex<String>>) {
    let mut guard = cache.lock().await;
    // Async operation inside critical section
    let new_data = fetch_data().await;
    *guard = new_data;
}

async fn fetch_data() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    "updated".to_string()
}
```

## Practical Example: Async Risk Manager

```rust
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }

    fn pnl_percent(&self) -> f64 {
        ((self.current_price - self.entry_price) / self.entry_price) * 100.0
    }
}

#[derive(Debug)]
struct RiskManager {
    positions: Mutex<HashMap<String, Position>>,
    max_loss_percent: f64,
    max_position_size: f64,
}

impl RiskManager {
    fn new(max_loss_percent: f64, max_position_size: f64) -> Self {
        RiskManager {
            positions: Mutex::new(HashMap::new()),
            max_loss_percent,
            max_position_size,
        }
    }

    async fn add_position(&self, symbol: &str, quantity: f64, price: f64) -> Result<(), String> {
        if quantity > self.max_position_size {
            return Err(format!(
                "Position size exceeded: {} > {}",
                quantity, self.max_position_size
            ));
        }

        let mut positions = self.positions.lock().await;
        positions.insert(symbol.to_string(), Position {
            symbol: symbol.to_string(),
            quantity,
            entry_price: price,
            current_price: price,
        });

        println!("Position opened: {} {} @ ${}", quantity, symbol, price);
        Ok(())
    }

    async fn update_price(&self, symbol: &str, new_price: f64) -> Option<String> {
        let mut positions = self.positions.lock().await;

        if let Some(pos) = positions.get_mut(symbol) {
            pos.current_price = new_price;
            let pnl_pct = pos.pnl_percent();

            // Check risk limit
            if pnl_pct < -self.max_loss_percent {
                let warning = format!(
                    "RISK ALERT: {} loss {:.2}% exceeds limit -{:.2}%",
                    symbol, pnl_pct.abs(), self.max_loss_percent
                );
                return Some(warning);
            }
        }
        None
    }

    async fn get_total_pnl(&self) -> f64 {
        let positions = self.positions.lock().await;
        positions.values().map(|p| p.pnl()).sum()
    }

    async fn get_positions_report(&self) -> String {
        let positions = self.positions.lock().await;
        let mut report = String::from("=== Positions Report ===\n");

        for pos in positions.values() {
            report.push_str(&format!(
                "{}: {} @ ${:.2} -> ${:.2} | PnL: ${:.2} ({:+.2}%)\n",
                pos.symbol, pos.quantity, pos.entry_price,
                pos.current_price, pos.pnl(), pos.pnl_percent()
            ));
        }

        report.push_str(&format!("Total PnL: ${:.2}",
            positions.values().map(|p| p.pnl()).sum::<f64>()));
        report
    }
}

async fn simulate_price_feed(symbol: &str, base_price: f64) -> f64 {
    // Simulate fetching price from exchange
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let change = ((seed % 1000) as f64 - 500.0) / 100.0;
    base_price * (1.0 + change / 100.0)
}

#[tokio::main]
async fn main() {
    let risk_manager = Arc::new(RiskManager::new(5.0, 10.0)); // 5% max loss, 10 max size

    // Open positions
    let rm1 = Arc::clone(&risk_manager);
    rm1.add_position("BTC", 1.0, 42000.0).await.unwrap();
    rm1.add_position("ETH", 5.0, 2500.0).await.unwrap();

    // Start price feed simulation
    let rm2 = Arc::clone(&risk_manager);
    let price_feed = tokio::spawn(async move {
        for _ in 0..10 {
            let btc_price = simulate_price_feed("BTC", 42000.0).await;
            let eth_price = simulate_price_feed("ETH", 2500.0).await;

            if let Some(alert) = rm2.update_price("BTC", btc_price).await {
                println!("{}", alert);
            }
            if let Some(alert) = rm2.update_price("ETH", eth_price).await {
                println!("{}", alert);
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    });

    // Position monitoring
    let rm3 = Arc::clone(&risk_manager);
    let monitor = tokio::spawn(async move {
        for _ in 0..5 {
            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
            println!("\n{}\n", rm3.get_positions_report().await);
        }
    });

    let _ = tokio::join!(price_feed, monitor);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `tokio::sync::Mutex` | Asynchronous mutex for async code |
| `.lock().await` | Asynchronously acquire the lock |
| `try_lock()` | Non-blocking attempt to acquire lock |
| vs `std::sync::Mutex` | Safe to hold across `.await` |
| Thread release | Thread is free for other tasks while waiting |

## Practical Exercises

1. **Async Quote Cache**: Implement a `QuoteCache` struct that:
   - Stores latest prices for a list of symbols
   - Updates prices asynchronously at different intervals
   - Provides a `get_quote(symbol)` method without blocking the main thread

2. **Parallel Order Loader**: Create a system that:
   - Loads orders from multiple "sources" in parallel
   - Saves them to a shared structure using `tokio::sync::Mutex`
   - Shows loading progress in real-time

3. **API Rate Limiter**: Implement a request limiter:
   - Tracks the number of requests in the last minute
   - Blocks new requests when the limit is exceeded
   - Works asynchronously and doesn't block other tasks

## Homework

1. **Performance Comparison**: Write a benchmark comparing `std::sync::Mutex` and `tokio::sync::Mutex` in different scenarios:
   - Many short operations
   - Few long operations with `.await` inside
   - High contention (many tasks, one resource)

2. **Async Exchange**: Extend the trading engine example:
   - Add an order book with bid/ask
   - Implement order matching
   - Add WebSocket-like trade notifications

3. **Deadlock Detector**: Create a wrapper around `tokio::sync::Mutex` that:
   - Logs lock wait time
   - Warns if a lock is held for too long
   - Outputs call stack on suspected deadlock

## Navigation

[← Previous day](../192-async-channels-tokio-mpsc/en.md) | [Next day →](../194-async-rwlock-tokio/en.md)
