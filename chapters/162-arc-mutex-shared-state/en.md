# Day 162: Arc<Mutex<T>>: Shared Mutable Structure

## Trading Analogy

Imagine a trading terminal where multiple strategies work simultaneously with a **shared portfolio**. Each strategy is a separate thread, and all of them need to:
- Read the current balance
- Modify positions
- Update shared statistics

`Arc<Mutex<T>>` is like a **safe in a traders' office**:
- `Arc` — multiple keys to the office (each trader can enter)
- `Mutex` — lock on the safe (only one can open the safe at a time)
- `T` — contents of the safe (portfolio, balance, positions)

## Why Do We Need Arc<Mutex<T>>?

| Primitive | Purpose |
|-----------|---------|
| `Arc<T>` | Shared ownership between threads (read-only) |
| `Mutex<T>` | Exclusive access for modification (single thread) |
| `Arc<Mutex<T>>` | Shared access + ability to modify |

## Basic Example: Shared Balance

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Shared balance for all threads
    let balance = Arc::new(Mutex::new(10000.0_f64));
    let mut handles = vec![];

    // Three strategies trade in parallel
    for strategy_id in 1..=3 {
        let balance_clone = Arc::clone(&balance);

        let handle = thread::spawn(move || {
            // Lock the mutex for modification
            let mut bal = balance_clone.lock().unwrap();
            let profit = strategy_id as f64 * 100.0;
            *bal += profit;
            println!("Strategy {}: added ${:.2}, balance: ${:.2}",
                     strategy_id, profit, *bal);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final balance: ${:.2}", *balance.lock().unwrap());
}
```

## Portfolio Structure

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Portfolio {
    balance: f64,
    positions: HashMap<String, f64>,  // ticker -> quantity
    total_pnl: f64,
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            balance: initial_balance,
            positions: HashMap::new(),
            total_pnl: 0.0,
        }
    }

    fn buy(&mut self, ticker: &str, quantity: f64, price: f64) -> Result<(), String> {
        let cost = quantity * price;
        if cost > self.balance {
            return Err(format!("Insufficient funds: need ${:.2}, have ${:.2}",
                              cost, self.balance));
        }

        self.balance -= cost;
        *self.positions.entry(ticker.to_string()).or_insert(0.0) += quantity;
        Ok(())
    }

    fn sell(&mut self, ticker: &str, quantity: f64, price: f64) -> Result<f64, String> {
        let position = self.positions.get(ticker).copied().unwrap_or(0.0);
        if quantity > position {
            return Err(format!("Insufficient {}: need {}, have {}",
                              ticker, quantity, position));
        }

        let revenue = quantity * price;
        self.balance += revenue;
        *self.positions.get_mut(ticker).unwrap() -= quantity;

        // Remove position if it became zero
        if self.positions[ticker] == 0.0 {
            self.positions.remove(ticker);
        }

        Ok(revenue)
    }
}

fn main() {
    let portfolio = Arc::new(Mutex::new(Portfolio::new(100000.0)));
    let mut handles = vec![];

    // Strategy 1: buys BTC
    let p1 = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let mut port = p1.lock().unwrap();
        match port.buy("BTC", 0.5, 42000.0) {
            Ok(_) => println!("Strategy 1: bought 0.5 BTC"),
            Err(e) => println!("Strategy 1: error - {}", e),
        }
    }));

    // Strategy 2: buys ETH
    let p2 = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let mut port = p2.lock().unwrap();
        match port.buy("ETH", 5.0, 2200.0) {
            Ok(_) => println!("Strategy 2: bought 5 ETH"),
            Err(e) => println!("Strategy 2: error - {}", e),
        }
    }));

    // Strategy 3: buys SOL
    let p3 = Arc::clone(&portfolio);
    handles.push(thread::spawn(move || {
        let mut port = p3.lock().unwrap();
        match port.buy("SOL", 100.0, 95.0) {
            Ok(_) => println!("Strategy 3: bought 100 SOL"),
            Err(e) => println!("Strategy 3: error - {}", e),
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let final_portfolio = portfolio.lock().unwrap();
    println!("\nFinal portfolio:");
    println!("Balance: ${:.2}", final_portfolio.balance);
    println!("Positions: {:?}", final_portfolio.positions);
}
```

## Pattern: Minimizing Lock Duration

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct OrderBook {
    bids: Vec<(f64, f64)>,  // (price, volume)
    asks: Vec<(f64, f64)>,
}

fn main() {
    let order_book = Arc::new(Mutex::new(OrderBook {
        bids: vec![(42000.0, 1.5), (41990.0, 2.0)],
        asks: vec![(42010.0, 1.0), (42020.0, 3.0)],
    }));

    let ob = Arc::clone(&order_book);
    let analyzer = thread::spawn(move || {
        // BAD: long lock duration
        // let book = ob.lock().unwrap();
        // thread::sleep(Duration::from_secs(1)); // Analysis
        // println!("Spread: {}", book.asks[0].0 - book.bids[0].0);

        // GOOD: copy data and release lock
        let (best_bid, best_ask) = {
            let book = ob.lock().unwrap();
            (book.bids[0].0, book.asks[0].0)
        }; // Lock released!

        // Now we can analyze for a long time
        thread::sleep(Duration::from_millis(100));
        let spread = best_ask - best_bid;
        println!("Spread: ${:.2}", spread);
    });

    analyzer.join().unwrap();
}
```

## Error Handling When Locking

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data = Arc::new(Mutex::new(vec![42000.0, 42100.0, 41900.0]));

    let d1 = Arc::clone(&data);
    let handle = thread::spawn(move || {
        // Using lock() with handling potential panic
        match d1.lock() {
            Ok(mut prices) => {
                prices.push(42050.0);
                println!("New price added");
            }
            Err(poisoned) => {
                // Mutex is poisoned due to panic in another thread
                println!("Mutex poisoned, recovering data");
                let mut prices = poisoned.into_inner();
                prices.clear();
                prices.push(42000.0);
            }
        }
    });

    handle.join().unwrap();

    // try_lock() - non-blocking lock attempt
    match data.try_lock() {
        Ok(prices) => println!("Prices: {:?}", *prices),
        Err(_) => println!("Mutex busy, will try later"),
    }
}
```

## Practical Example: Price Aggregator

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct PriceData {
    ticker: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

struct PriceAggregator {
    prices: HashMap<String, PriceData>,
    update_count: u64,
}

use std::collections::HashMap;

impl PriceAggregator {
    fn new() -> Self {
        PriceAggregator {
            prices: HashMap::new(),
            update_count: 0,
        }
    }

    fn update_price(&mut self, data: PriceData) {
        self.prices.insert(data.ticker.clone(), data);
        self.update_count += 1;
    }

    fn get_price(&self, ticker: &str) -> Option<f64> {
        self.prices.get(ticker).map(|d| d.price)
    }

    fn get_all_prices(&self) -> Vec<(String, f64)> {
        self.prices
            .iter()
            .map(|(k, v)| (k.clone(), v.price))
            .collect()
    }
}

fn main() {
    let aggregator = Arc::new(Mutex::new(PriceAggregator::new()));
    let mut handles = vec![];

    // BTC update thread
    let agg1 = Arc::clone(&aggregator);
    handles.push(thread::spawn(move || {
        for i in 0..5 {
            let price = 42000.0 + (i as f64 * 10.0);
            {
                let mut agg = agg1.lock().unwrap();
                agg.update_price(PriceData {
                    ticker: "BTC".to_string(),
                    price,
                    volume: 1.5,
                    timestamp: i as u64,
                });
            }
            thread::sleep(Duration::from_millis(50));
        }
    }));

    // ETH update thread
    let agg2 = Arc::clone(&aggregator);
    handles.push(thread::spawn(move || {
        for i in 0..5 {
            let price = 2200.0 + (i as f64 * 5.0);
            {
                let mut agg = agg2.lock().unwrap();
                agg.update_price(PriceData {
                    ticker: "ETH".to_string(),
                    price,
                    volume: 10.0,
                    timestamp: i as u64,
                });
            }
            thread::sleep(Duration::from_millis(50));
        }
    }));

    // Price reading thread
    let agg3 = Arc::clone(&aggregator);
    handles.push(thread::spawn(move || {
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(30));
            let agg = agg3.lock().unwrap();
            println!("Current prices: {:?}", agg.get_all_prices());
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    let final_agg = aggregator.lock().unwrap();
    println!("\nTotal updates: {}", final_agg.update_count);
}
```

## Pattern: Trade Statistics

```rust
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Default)]
struct TradeStats {
    total_trades: u64,
    winning_trades: u64,
    losing_trades: u64,
    total_pnl: f64,
    max_profit: f64,
    max_loss: f64,
}

impl TradeStats {
    fn record_trade(&mut self, pnl: f64) {
        self.total_trades += 1;
        self.total_pnl += pnl;

        if pnl > 0.0 {
            self.winning_trades += 1;
            if pnl > self.max_profit {
                self.max_profit = pnl;
            }
        } else if pnl < 0.0 {
            self.losing_trades += 1;
            if pnl < self.max_loss {
                self.max_loss = pnl;
            }
        }
    }

    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            return 0.0;
        }
        (self.winning_trades as f64 / self.total_trades as f64) * 100.0
    }
}

fn main() {
    let stats = Arc::new(Mutex::new(TradeStats::default()));
    let mut handles = vec![];

    // Simulate multiple trading threads
    for thread_id in 0..3 {
        let stats_clone = Arc::clone(&stats);

        handles.push(thread::spawn(move || {
            let trades = vec![150.0, -50.0, 200.0, -30.0, 100.0];

            for pnl in trades {
                let adjusted_pnl = pnl * (thread_id as f64 + 1.0);
                let mut s = stats_clone.lock().unwrap();
                s.record_trade(adjusted_pnl);
                println!("Thread {}: trade ${:.2}", thread_id, adjusted_pnl);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_stats = stats.lock().unwrap();
    println!("\n=== Final Statistics ===");
    println!("Total trades: {}", final_stats.total_trades);
    println!("Winning: {}", final_stats.winning_trades);
    println!("Losing: {}", final_stats.losing_trades);
    println!("Win Rate: {:.1}%", final_stats.win_rate());
    println!("Total PnL: ${:.2}", final_stats.total_pnl);
    println!("Max profit: ${:.2}", final_stats.max_profit);
    println!("Max loss: ${:.2}", final_stats.max_loss);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Arc::new(Mutex::new(T))` | Create shared mutable data |
| `Arc::clone(&arc)` | Get a new reference for a thread |
| `.lock().unwrap()` | Lock and get access |
| `try_lock()` | Non-blocking lock attempt |
| Minimizing lock time | Copy data before releasing |
| Poisoned Mutex | Handling panic in another thread |

## Important Rules

1. **Minimize lock duration** — copy data and release the mutex
2. **Avoid nested locks** — this leads to deadlock
3. **Use `try_lock()`** when immediate access is not critical
4. **Handle poisoned mutex** in production code

## Homework

1. Create a `SharedOrderManager` structure with methods `place_order()`, `cancel_order()`, `get_orders()` for multi-threaded order management

2. Implement a `RiskManager` with `Arc<Mutex<T>>` that checks limits before each trade and updates the used risk

3. Write a market maker simulator where one thread updates quotes and other threads "trade" on those quotes

4. Create a trade logging system where multiple strategies write to a shared log via `Arc<Mutex<Vec<TradeLog>>>`

## Navigation

[← Previous day](../161-arc-shared-access/en.md) | [Next day →](../163-rwlock-readers-writer/en.md)
