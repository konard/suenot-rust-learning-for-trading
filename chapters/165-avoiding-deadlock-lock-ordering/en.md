# Day 165: Avoiding Deadlock: Lock Ordering

## Trading Analogy

In the previous chapter, we learned about deadlock — mutual thread blocking. Imagine a large exchange where thousands of assets are traded: BTC, ETH, SOL, USDT, and others. Each asset has its own balance, and for operational safety, each balance is protected by a mutex.

Now imagine two traders:
- Trader A wants to swap BTC for ETH: first locks BTC, then ETH
- Trader B wants to swap ETH for BTC: first locks ETH, then BTC

The result — deadlock! But there's a simple solution: **everyone must lock assets in the same order**. For example, always BTC first, then ETH — regardless of the swap direction.

This is similar to traffic rules: if everyone drives on the right side of the road, there won't be collisions. Lock ordering is the "traffic rules" for threads.

## What is Lock Ordering?

**Lock Ordering** is a deadlock prevention technique where all threads acquire mutexes in a strictly defined, pre-agreed order.

### Why Does This Work?

Let's recall the four conditions for deadlock:
1. **Mutual Exclusion** — a resource can only be held by one thread
2. **Hold and Wait** — a thread holds one resource while waiting for another
3. **No Preemption** — resources cannot be forcibly taken away
4. **Circular Wait** — there exists a circular chain of threads waiting for each other

Lock ordering **eliminates Circular Wait**. If all threads lock resources A, B, C in the order A → B → C, then a situation where one thread holds C and waits for A is impossible (this would violate the ordering).

## The Problem: Unordered Locks

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Cryptocurrency trading portfolios
struct CryptoPortfolio {
    btc: f64,
    eth: f64,
    sol: f64,
}

fn main() {
    let btc_wallet = Arc::new(Mutex::new(10.0_f64));
    let eth_wallet = Arc::new(Mutex::new(100.0_f64));
    let sol_wallet = Arc::new(Mutex::new(500.0_f64));

    let btc1 = Arc::clone(&btc_wallet);
    let eth1 = Arc::clone(&eth_wallet);
    let sol1 = Arc::clone(&sol_wallet);

    let btc2 = Arc::clone(&btc_wallet);
    let eth2 = Arc::clone(&eth_wallet);
    let sol2 = Arc::clone(&sol_wallet);

    // Thread 1: Rebalancing BTC → ETH → SOL
    let handle1 = thread::spawn(move || {
        println!("Thread 1: Starting portfolio rebalancing...");

        let _btc = btc1.lock().unwrap();
        println!("Thread 1: BTC locked");
        thread::sleep(Duration::from_millis(50));

        let _eth = eth1.lock().unwrap();
        println!("Thread 1: ETH locked");
        thread::sleep(Duration::from_millis(50));

        let _sol = sol1.lock().unwrap();
        println!("Thread 1: SOL locked");

        println!("Thread 1: Rebalancing complete!");
    });

    // Thread 2: Rebalancing SOL → ETH → BTC (REVERSE order!)
    let handle2 = thread::spawn(move || {
        println!("Thread 2: Starting portfolio rebalancing...");

        let _sol = sol2.lock().unwrap();
        println!("Thread 2: SOL locked");
        thread::sleep(Duration::from_millis(50));

        let _eth = eth2.lock().unwrap();
        println!("Thread 2: ETH locked");
        thread::sleep(Duration::from_millis(50));

        let _btc = btc2.lock().unwrap();
        println!("Thread 2: BTC locked");

        println!("Thread 2: Rebalancing complete!");
    });

    // DEADLOCK is inevitable!
    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

**Problem:** Thread 1 locks BTC and waits for SOL, while Thread 2 locks SOL and waits for BTC.

## Solution 1: Fixed Alphabetical Order

The simplest approach — order resources alphabetically or by another static criterion:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Order: BTC < ETH < SOL (alphabetically)
    let btc_wallet = Arc::new(Mutex::new(10.0_f64));
    let eth_wallet = Arc::new(Mutex::new(100.0_f64));
    let sol_wallet = Arc::new(Mutex::new(500.0_f64));

    let btc1 = Arc::clone(&btc_wallet);
    let eth1 = Arc::clone(&eth_wallet);
    let sol1 = Arc::clone(&sol_wallet);

    let btc2 = Arc::clone(&btc_wallet);
    let eth2 = Arc::clone(&eth_wallet);
    let sol2 = Arc::clone(&sol_wallet);

    // Thread 1: BTC → ETH → SOL (correct order)
    let handle1 = thread::spawn(move || {
        println!("Thread 1: Locking in order BTC → ETH → SOL");

        let btc = btc1.lock().unwrap();
        let eth = eth1.lock().unwrap();
        let sol = sol1.lock().unwrap();

        println!("Thread 1: BTC={}, ETH={}, SOL={}", *btc, *eth, *sol);
    });

    // Thread 2: BTC → ETH → SOL (SAME order!)
    let handle2 = thread::spawn(move || {
        println!("Thread 2: Locking in order BTC → ETH → SOL");

        let btc = btc2.lock().unwrap();
        let eth = eth2.lock().unwrap();
        let sol = sol2.lock().unwrap();

        println!("Thread 2: BTC={}, ETH={}, SOL={}", *btc, *eth, *sol);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    println!("Program completed successfully — no deadlock!");
}
```

## Solution 2: Order by Identifier (for Dynamic Resources)

In real trading systems, assets are added dynamically. We use unique IDs:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::cmp::Ordering;

/// Trading asset with unique ID for ordering
struct TradingAsset {
    id: u64,            // Unique ID for lock ordering
    symbol: String,
    balance: Mutex<f64>,
}

impl TradingAsset {
    fn new(id: u64, symbol: &str, balance: f64) -> Self {
        TradingAsset {
            id,
            symbol: symbol.to_string(),
            balance: Mutex::new(balance),
        }
    }
}

/// Safely locks two assets in the correct order
fn lock_two_assets<'a>(
    asset1: &'a TradingAsset,
    asset2: &'a TradingAsset,
) -> (
    std::sync::MutexGuard<'a, f64>,
    std::sync::MutexGuard<'a, f64>,
) {
    // Always lock by ascending ID
    match asset1.id.cmp(&asset2.id) {
        Ordering::Less => {
            let guard1 = asset1.balance.lock().unwrap();
            let guard2 = asset2.balance.lock().unwrap();
            (guard1, guard2)
        }
        Ordering::Greater => {
            let guard2 = asset2.balance.lock().unwrap();
            let guard1 = asset1.balance.lock().unwrap();
            (guard1, guard2)
        }
        Ordering::Equal => {
            panic!("Cannot lock the same asset twice!");
        }
    }
}

/// Safe swap between two assets
fn safe_swap(
    from_asset: &TradingAsset,
    to_asset: &TradingAsset,
    amount: f64,
    rate: f64,
) -> Result<(), String> {
    let (mut from_guard, mut to_guard) = lock_two_assets(from_asset, to_asset);

    // Check balance
    if *from_guard < amount {
        return Err(format!(
            "Insufficient {}: have {}, need {}",
            from_asset.symbol, *from_guard, amount
        ));
    }

    // Execute swap
    *from_guard -= amount;
    *to_guard += amount * rate;

    println!(
        "Swap: {} {} → {} {} (rate: {})",
        amount, from_asset.symbol,
        amount * rate, to_asset.symbol,
        rate
    );

    Ok(())
}

fn main() {
    // Create assets with unique IDs
    let btc = Arc::new(TradingAsset::new(1, "BTC", 10.0));
    let eth = Arc::new(TradingAsset::new(2, "ETH", 100.0));
    let sol = Arc::new(TradingAsset::new(3, "SOL", 500.0));

    let btc1 = Arc::clone(&btc);
    let eth1 = Arc::clone(&eth);

    let eth2 = Arc::clone(&eth);
    let sol2 = Arc::clone(&sol);

    let btc3 = Arc::clone(&btc);
    let sol3 = Arc::clone(&sol);

    // Three threads perform swaps simultaneously
    let h1 = thread::spawn(move || {
        for _ in 0..5 {
            let _ = safe_swap(&btc1, &eth1, 0.1, 15.0);
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let h2 = thread::spawn(move || {
        for _ in 0..5 {
            let _ = safe_swap(&eth2, &sol2, 1.0, 5.0);
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let h3 = thread::spawn(move || {
        for _ in 0..5 {
            let _ = safe_swap(&sol3, &btc3, 10.0, 0.002);
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();

    println!("\nFinal balances:");
    println!("BTC: {}", *btc.balance.lock().unwrap());
    println!("ETH: {}", *eth.balance.lock().unwrap());
    println!("SOL: {}", *sol.balance.lock().unwrap());
}
```

## Solution 3: Order by Memory Address

When resources don't have a natural ID, you can use their memory address:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

struct OrderBook {
    symbol: String,
    bids: Vec<(f64, f64)>, // (price, quantity)
    asks: Vec<(f64, f64)>,
}

struct Portfolio {
    cash: f64,
    positions: Vec<(String, f64)>,
}

/// Locks two mutexes by pointer address
fn lock_by_address<'a, T, U>(
    m1: &'a Mutex<T>,
    m2: &'a Mutex<U>,
) -> (std::sync::MutexGuard<'a, T>, std::sync::MutexGuard<'a, U>) {
    let ptr1 = m1 as *const Mutex<T> as usize;
    let ptr2 = m2 as *const Mutex<U> as usize;

    if ptr1 < ptr2 {
        let g1 = m1.lock().unwrap();
        let g2 = m2.lock().unwrap();
        (g1, g2)
    } else {
        let g2 = m2.lock().unwrap();
        let g1 = m1.lock().unwrap();
        (g1, g2)
    }
}

fn execute_market_order(
    order_book: &Mutex<OrderBook>,
    portfolio: &Mutex<Portfolio>,
    symbol: &str,
    quantity: f64,
    is_buy: bool,
) -> Result<f64, String> {
    // Lock in address order — deadlock impossible!
    let (mut ob, mut pf) = lock_by_address(order_book, portfolio);

    // Find best price
    let price = if is_buy {
        ob.asks.first().map(|(p, _)| *p).unwrap_or(0.0)
    } else {
        ob.bids.first().map(|(p, _)| *p).unwrap_or(0.0)
    };

    if price == 0.0 {
        return Err("No orders in the book".to_string());
    }

    let cost = price * quantity;

    if is_buy {
        if pf.cash < cost {
            return Err(format!("Insufficient funds: {} < {}", pf.cash, cost));
        }
        pf.cash -= cost;
        pf.positions.push((symbol.to_string(), quantity));
    } else {
        pf.cash += cost;
        // Remove position (simplified)
    }

    println!(
        "{} {} {} at price {} = {}",
        if is_buy { "Buy" } else { "Sell" },
        quantity,
        symbol,
        price,
        cost
    );

    Ok(price)
}

fn main() {
    let order_book = Arc::new(Mutex::new(OrderBook {
        symbol: "BTC/USDT".to_string(),
        bids: vec![(41000.0, 5.0), (40900.0, 10.0)],
        asks: vec![(41100.0, 3.0), (41200.0, 7.0)],
    }));

    let portfolio = Arc::new(Mutex::new(Portfolio {
        cash: 100_000.0,
        positions: vec![],
    }));

    let ob1 = Arc::clone(&order_book);
    let pf1 = Arc::clone(&portfolio);

    let ob2 = Arc::clone(&order_book);
    let pf2 = Arc::clone(&portfolio);

    // Parallel trading operations
    let buyer = thread::spawn(move || {
        for _ in 0..3 {
            let _ = execute_market_order(&ob1, &pf1, "BTC", 0.5, true);
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let seller = thread::spawn(move || {
        for _ in 0..3 {
            let _ = execute_market_order(&ob2, &pf2, "BTC", 0.3, false);
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    buyer.join().unwrap();
    seller.join().unwrap();

    println!("\nFinal portfolio:");
    let pf = portfolio.lock().unwrap();
    println!("Cash: ${:.2}", pf.cash);
    for (sym, qty) in &pf.positions {
        println!("{}: {}", sym, qty);
    }
}
```

## Solution 4: Hierarchical Locking

For complex systems, it's convenient to use a hierarchy of levels:

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// Lock hierarchy levels (lock from lowest to highest)
/// Level 1: Individual balances
/// Level 2: Order books
/// Level 3: Risk management
/// Level 4: Global settings

struct TradingSystem {
    // Level 1: Balances (lock first)
    btc_balance: Mutex<f64>,
    usdt_balance: Mutex<f64>,

    // Level 2: Order book
    order_book: Mutex<Vec<(f64, f64, bool)>>, // (price, qty, is_bid)

    // Level 3: Risk management
    risk_limits: RwLock<RiskLimits>,

    // Level 4: Global settings (lock last)
    global_config: RwLock<GlobalConfig>,
}

struct RiskLimits {
    max_position_size: f64,
    max_daily_loss: f64,
    current_daily_pnl: f64,
}

struct GlobalConfig {
    trading_enabled: bool,
    maintenance_mode: bool,
}

impl TradingSystem {
    fn new() -> Self {
        TradingSystem {
            btc_balance: Mutex::new(10.0),
            usdt_balance: Mutex::new(100_000.0),
            order_book: Mutex::new(vec![]),
            risk_limits: RwLock::new(RiskLimits {
                max_position_size: 5.0,
                max_daily_loss: 10_000.0,
                current_daily_pnl: 0.0,
            }),
            global_config: RwLock::new(GlobalConfig {
                trading_enabled: true,
                maintenance_mode: false,
            }),
        }
    }

    /// Place order following the hierarchy
    fn place_order(&self, price: f64, quantity: f64, is_buy: bool) -> Result<(), String> {
        // Level 4: Check global settings
        {
            let config = self.global_config.read().unwrap();
            if !config.trading_enabled || config.maintenance_mode {
                return Err("Trading temporarily unavailable".to_string());
            }
        }

        // Level 3: Check risk limits
        {
            let risk = self.risk_limits.read().unwrap();
            if quantity > risk.max_position_size {
                return Err(format!(
                    "Position limit exceeded: {} > {}",
                    quantity, risk.max_position_size
                ));
            }
        }

        // Level 1: Lock balances
        let cost = price * quantity;
        if is_buy {
            let mut usdt = self.usdt_balance.lock().unwrap();
            if *usdt < cost {
                return Err(format!("Insufficient USDT: {} < {}", *usdt, cost));
            }
            *usdt -= cost;
        } else {
            let mut btc = self.btc_balance.lock().unwrap();
            if *btc < quantity {
                return Err(format!("Insufficient BTC: {} < {}", *btc, quantity));
            }
            *btc -= quantity;
        }

        // Level 2: Add to order book
        {
            let mut book = self.order_book.lock().unwrap();
            book.push((price, quantity, is_buy));
            println!(
                "Order placed: {} {} BTC @ {}",
                if is_buy { "BUY" } else { "SELL" },
                quantity,
                price
            );
        }

        Ok(())
    }

    /// Update risk limits (requires write lock)
    fn update_risk_limits(&self, new_max_position: f64) {
        // Level 4 → Level 3 (first config, then risk)
        let config = self.global_config.read().unwrap();
        if config.maintenance_mode {
            println!("In maintenance mode — limits unchanged");
            return;
        }
        drop(config); // Release before next lock

        let mut risk = self.risk_limits.write().unwrap();
        risk.max_position_size = new_max_position;
        println!("Risk limits updated: max_position = {}", new_max_position);
    }

    fn get_status(&self) -> String {
        let btc = self.btc_balance.lock().unwrap();
        let usdt = self.usdt_balance.lock().unwrap();
        let orders = self.order_book.lock().unwrap();

        format!(
            "BTC: {:.4}, USDT: {:.2}, Active orders: {}",
            *btc, *usdt, orders.len()
        )
    }
}

fn main() {
    let system = Arc::new(TradingSystem::new());

    let s1 = Arc::clone(&system);
    let s2 = Arc::clone(&system);
    let s3 = Arc::clone(&system);

    let trader1 = thread::spawn(move || {
        for i in 0..5 {
            let price = 42000.0 + i as f64 * 100.0;
            match s1.place_order(price, 0.5, true) {
                Ok(_) => {}
                Err(e) => println!("Error: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(20));
        }
    });

    let trader2 = thread::spawn(move || {
        for i in 0..5 {
            let price = 43000.0 - i as f64 * 100.0;
            match s2.place_order(price, 0.3, false) {
                Ok(_) => {}
                Err(e) => println!("Error: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(20));
        }
    });

    let admin = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(50));
        s3.update_risk_limits(3.0);
        println!("Status: {}", s3.get_status());
    });

    trader1.join().unwrap();
    trader2.join().unwrap();
    admin.join().unwrap();

    println!("\nFinal status: {}", system.get_status());
}
```

## Practical Example: Multi-Currency Exchange

Let's create a complete example of an exchange with multiple currency pairs:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::BTreeMap;

/// Trading pair on the exchange
#[derive(Debug)]
struct TradingPair {
    id: u64,
    base: String,    // E.g.: BTC
    quote: String,   // E.g.: USDT
    price: f64,
    volume_24h: f64,
}

/// Multi-currency exchange with ordered locks
struct MultiCurrencyExchange {
    // BTreeMap automatically sorts keys — we use it for ordering
    balances: BTreeMap<String, Arc<Mutex<f64>>>,
    pairs: Vec<Arc<Mutex<TradingPair>>>,
}

impl MultiCurrencyExchange {
    fn new() -> Self {
        let mut balances = BTreeMap::new();

        // Balances are automatically sorted alphabetically
        balances.insert("BTC".to_string(), Arc::new(Mutex::new(10.0)));
        balances.insert("ETH".to_string(), Arc::new(Mutex::new(100.0)));
        balances.insert("SOL".to_string(), Arc::new(Mutex::new(500.0)));
        balances.insert("USDT".to_string(), Arc::new(Mutex::new(100_000.0)));

        let pairs = vec![
            Arc::new(Mutex::new(TradingPair {
                id: 1,
                base: "BTC".to_string(),
                quote: "USDT".to_string(),
                price: 42000.0,
                volume_24h: 0.0,
            })),
            Arc::new(Mutex::new(TradingPair {
                id: 2,
                base: "ETH".to_string(),
                quote: "USDT".to_string(),
                price: 2200.0,
                volume_24h: 0.0,
            })),
            Arc::new(Mutex::new(TradingPair {
                id: 3,
                base: "SOL".to_string(),
                quote: "USDT".to_string(),
                price: 100.0,
                volume_24h: 0.0,
            })),
        ];

        MultiCurrencyExchange { balances, pairs }
    }

    /// Safe swap between any two currencies
    fn swap(&self, from: &str, to: &str, amount: f64, rate: f64) -> Result<f64, String> {
        // BTreeMap guarantees order — iterate and lock in order
        let from_lock = self.balances.get(from)
            .ok_or_else(|| format!("Unknown currency: {}", from))?;
        let to_lock = self.balances.get(to)
            .ok_or_else(|| format!("Unknown currency: {}", to))?;

        // Lock in alphabetical order (BTreeMap guarantees this)
        let (mut from_guard, mut to_guard) = if from < to {
            let f = from_lock.lock().unwrap();
            let t = to_lock.lock().unwrap();
            (f, t)
        } else {
            let t = to_lock.lock().unwrap();
            let f = from_lock.lock().unwrap();
            (f, t)
        };

        // Check balance
        if *from_guard < amount {
            return Err(format!(
                "Insufficient {}: {} < {}",
                from, *from_guard, amount
            ));
        }

        // Execute swap
        let received = amount * rate;
        *from_guard -= amount;
        *to_guard += received;

        Ok(received)
    }

    /// Get all balances (locks in correct order)
    fn get_all_balances(&self) -> Vec<(String, f64)> {
        // BTreeMap iterates in sorted order
        self.balances
            .iter()
            .map(|(name, lock)| {
                let balance = lock.lock().unwrap();
                (name.clone(), *balance)
            })
            .collect()
    }

    /// Update pair price
    fn update_price(&self, pair_id: u64, new_price: f64) {
        for pair in &self.pairs {
            let mut p = pair.lock().unwrap();
            if p.id == pair_id {
                p.price = new_price;
                break;
            }
        }
    }
}

fn main() {
    let exchange = Arc::new(MultiCurrencyExchange::new());

    let ex1 = Arc::clone(&exchange);
    let ex2 = Arc::clone(&exchange);
    let ex3 = Arc::clone(&exchange);

    // Trader 1: Buys BTC with USDT
    let trader1 = thread::spawn(move || {
        for i in 0..5 {
            match ex1.swap("USDT", "BTC", 4200.0, 1.0 / 42000.0) {
                Ok(received) => println!(
                    "Trader 1: Bought {} BTC (operation {})",
                    received, i + 1
                ),
                Err(e) => println!("Trader 1: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(30));
        }
    });

    // Trader 2: Buys ETH with USDT
    let trader2 = thread::spawn(move || {
        for i in 0..5 {
            match ex2.swap("USDT", "ETH", 2200.0, 1.0 / 2200.0) {
                Ok(received) => println!(
                    "Trader 2: Bought {} ETH (operation {})",
                    received, i + 1
                ),
                Err(e) => println!("Trader 2: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(30));
        }
    });

    // Trader 3: Arbitrage BTC ↔ ETH
    let trader3 = thread::spawn(move || {
        for i in 0..5 {
            // BTC → ETH
            match ex3.swap("BTC", "ETH", 0.1, 15.0) {
                Ok(received) => println!(
                    "Trader 3: Swapped 0.1 BTC → {} ETH (operation {})",
                    received, i + 1
                ),
                Err(e) => println!("Trader 3: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(30));
        }
    });

    trader1.join().unwrap();
    trader2.join().unwrap();
    trader3.join().unwrap();

    println!("\n=== Final Balances ===");
    for (currency, balance) in exchange.get_all_balances() {
        println!("{}: {:.4}", currency, balance);
    }
}
```

## Lock Ordering Best Practices

| Practice | Description | Example |
|----------|-------------|---------|
| Alphabetical order | Lock resources alphabetically | BTC → ETH → SOL → USDT |
| By ID | Use unique numeric ID | id=1 → id=2 → id=3 |
| By address | Compare pointer addresses | ptr1 < ptr2 |
| Hierarchy | Define priority levels | Balances → Orders → Risks |
| BTreeMap | Use sorted collection | Automatic ordering |

## What We Learned

| Concept | Description |
|---------|-------------|
| Lock Ordering | Fixed order for acquiring locks |
| Circular Wait | Waiting cycle — one of the deadlock conditions |
| Order by ID | Using unique identifiers |
| Order by address | Comparing pointers to determine order |
| Hierarchical locking | Resource priority levels |
| BTreeMap | Automatic key sorting |

## Homework

1. **Triangular Swap**: Implement a `triangular_swap` function that safely swaps three assets simultaneously (A → B → C → A). Use lock ordering by ID.

2. **Trading Engine with Queue**: Create a system where:
   - There are 5 currency pairs
   - Multiple threads place orders
   - One thread executes orders from a queue
   - Use hierarchical locking: Orders → Balances → Statistics

3. **Order Violation Detector**: Write a Mutex wrapper that:
   - Remembers thread-local lock order
   - In debug mode, checks that new locks don't violate the order
   - Logs a warning on potential deadlock

4. **Exchange Arbitrage**: Implement a system for finding arbitrage opportunities between three exchanges:
   - Each exchange has its own balances
   - An arbitrage bot works on all exchanges simultaneously
   - Use lock ordering by exchange name (alphabetical)

## Navigation

[← Previous day](../164-deadlock-threads-block/en.md) | [Next day →](../166-condition-variables/en.md)
