# Day 164: Deadlock: When Threads Block Each Other

## Trading Analogy

Imagine a situation on an exchange: Trader A wants to swap BTC for ETH, while Trader B wants to swap ETH for BTC. Trader A first locks their BTC and waits for ETH to become available for the swap. At the same time, Trader B locks their ETH and waits for BTC. Both are waiting for each other — and neither can complete the trade. This is a **deadlock** — a situation where two or more threads are permanently blocked, each waiting for resources held by the other.

In real trading, this can happen when:
- One thread locks the order book and waits for access to balances
- Another thread locks balances and waits for access to the order book
- Both are stuck forever!

## What is a Deadlock?

A deadlock occurs when all four conditions are met simultaneously:

1. **Mutual Exclusion** — a resource can only be held by one thread
2. **Hold and Wait** — a thread holds one resource while waiting for another
3. **No Preemption** — resources cannot be forcibly taken away
4. **Circular Wait** — there exists a circular chain of threads waiting for each other

## Simple Deadlock Example

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Two resources: BTC balance and ETH balance
    let btc_balance = Arc::new(Mutex::new(10.0_f64));
    let eth_balance = Arc::new(Mutex::new(100.0_f64));

    let btc1 = Arc::clone(&btc_balance);
    let eth1 = Arc::clone(&eth_balance);

    let btc2 = Arc::clone(&btc_balance);
    let eth2 = Arc::clone(&eth_balance);

    // Thread 1: locks BTC first, then ETH
    let handle1 = thread::spawn(move || {
        println!("Thread 1: Trying to lock BTC...");
        let _btc = btc1.lock().unwrap();
        println!("Thread 1: BTC locked!");

        // Simulate work
        thread::sleep(Duration::from_millis(100));

        println!("Thread 1: Trying to lock ETH...");
        let _eth = eth1.lock().unwrap(); // DEADLOCK! Thread 2 already holds ETH
        println!("Thread 1: ETH locked!");

        println!("Thread 1: Executing BTC -> ETH swap");
    });

    // Thread 2: locks ETH first, then BTC
    let handle2 = thread::spawn(move || {
        println!("Thread 2: Trying to lock ETH...");
        let _eth = eth2.lock().unwrap();
        println!("Thread 2: ETH locked!");

        // Simulate work
        thread::sleep(Duration::from_millis(100));

        println!("Thread 2: Trying to lock BTC...");
        let _btc = btc2.lock().unwrap(); // DEADLOCK! Thread 1 already holds BTC
        println!("Thread 2: BTC locked!");

        println!("Thread 2: Executing ETH -> BTC swap");
    });

    // These joins will never complete!
    handle1.join().unwrap();
    handle2.join().unwrap();

    println!("Program finished"); // This line will never execute
}
```

**Important:** This code will hang forever! Don't run it in production.

## Visualizing Deadlock

```
Thread 1                    Thread 2
   |                           |
   v                           v
Locks BTC --------+    +------ Locks ETH
   |              |    |       |
   v              |    |       v
Waiting ETH <-----+----+--> Waiting BTC
   |                           |
   X DEADLOCK X           X DEADLOCK X
```

## Trading Engine Example

Let's look at a more realistic example — a trading engine with orders:

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct OrderBook {
    bids: Vec<Order>,
    asks: Vec<Order>,
}

#[derive(Debug)]
struct Portfolio {
    cash: f64,
    positions: Vec<(String, f64)>, // (symbol, quantity)
}

fn main() {
    let order_book = Arc::new(Mutex::new(OrderBook {
        bids: vec![],
        asks: vec![],
    }));

    let portfolio = Arc::new(Mutex::new(Portfolio {
        cash: 100_000.0,
        positions: vec![("BTC".to_string(), 5.0)],
    }));

    let ob1 = Arc::clone(&order_book);
    let pf1 = Arc::clone(&portfolio);

    let ob2 = Arc::clone(&order_book);
    let pf2 = Arc::clone(&portfolio);

    // Thread 1: Adding an order
    // First checks portfolio, then updates order book
    let handle1 = thread::spawn(move || {
        println!("Thread 1 (Add Order): Locking portfolio...");
        let portfolio_guard = pf1.lock().unwrap();
        println!("Thread 1: Portfolio locked, checking balance...");

        if portfolio_guard.cash >= 50_000.0 {
            thread::sleep(Duration::from_millis(50));

            println!("Thread 1: Trying to lock order book...");
            let mut ob_guard = ob1.lock().unwrap();

            ob_guard.bids.push(Order {
                id: 1,
                symbol: "BTC".to_string(),
                price: 42000.0,
                quantity: 1.0,
            });
            println!("Thread 1: Order added!");
        }
    });

    // Thread 2: Executing an order
    // First looks at order book, then updates portfolio
    let handle2 = thread::spawn(move || {
        println!("Thread 2 (Execute Order): Locking order book...");
        let ob_guard = ob2.lock().unwrap();
        println!("Thread 2: Order book locked, searching for orders...");

        if !ob_guard.bids.is_empty() {
            thread::sleep(Duration::from_millis(50));

            println!("Thread 2: Trying to lock portfolio...");
            let mut portfolio_guard = pf2.lock().unwrap();

            portfolio_guard.cash -= 42000.0;
            println!("Thread 2: Order executed!");
        }
    });

    // Potential DEADLOCK!
    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

## Detecting Deadlock

Rust doesn't prevent deadlocks automatically. Here's how you can detect the problem:

### 1. try_lock — Non-blocking Lock Attempt

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let btc_balance = Arc::new(Mutex::new(10.0_f64));
    let eth_balance = Arc::new(Mutex::new(100.0_f64));

    let btc1 = Arc::clone(&btc_balance);
    let eth1 = Arc::clone(&eth_balance);

    let handle = thread::spawn(move || {
        // Lock BTC
        let btc_guard = btc1.lock().unwrap();
        println!("BTC locked: {}", *btc_guard);

        // Try to lock ETH without waiting
        match eth1.try_lock() {
            Ok(eth_guard) => {
                println!("ETH also locked: {}", *eth_guard);
                // Execute operation
            }
            Err(_) => {
                println!("Failed to lock ETH, resource is busy");
                // Can retry later or release BTC
            }
        }
    });

    handle.join().unwrap();
}
```

### 2. Timeout Using parking_lot

```rust
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let balance = Arc::new(Mutex::new(10.0_f64));
    let balance_clone = Arc::clone(&balance);

    // Thread 1 holds the lock for a long time
    let handle1 = thread::spawn(move || {
        let _guard = balance_clone.lock();
        println!("Thread 1: Holding the lock...");
        thread::sleep(Duration::from_secs(2));
        println!("Thread 1: Releasing the lock");
    });

    // Small delay so thread 1 can acquire the lock
    thread::sleep(Duration::from_millis(100));

    // Thread 2 tries with timeout
    let handle2 = thread::spawn(move || {
        println!("Thread 2: Trying to acquire with timeout...");

        if let Some(guard) = balance.try_lock_for(Duration::from_millis(500)) {
            println!("Thread 2: Success! Balance: {}", *guard);
        } else {
            println!("Thread 2: Timeout! Possible deadlock.");
        }
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

## Deadlock Prevention Patterns

### 1. Consistent Lock Ordering

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let btc_balance = Arc::new(Mutex::new(10.0_f64));
    let eth_balance = Arc::new(Mutex::new(100.0_f64));

    let btc1 = Arc::clone(&btc_balance);
    let eth1 = Arc::clone(&eth_balance);

    let btc2 = Arc::clone(&btc_balance);
    let eth2 = Arc::clone(&eth_balance);

    // Both threads lock in the SAME order: BTC first, then ETH
    let handle1 = thread::spawn(move || {
        let btc = btc1.lock().unwrap();
        let eth = eth1.lock().unwrap();
        println!("Thread 1: BTC={}, ETH={}", *btc, *eth);
    });

    let handle2 = thread::spawn(move || {
        let btc = btc2.lock().unwrap();
        let eth = eth2.lock().unwrap();
        println!("Thread 2: BTC={}, ETH={}", *btc, *eth);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    println!("Program completed successfully!");
}
```

### 2. Short Lock Duration

```rust
use std::sync::{Arc, Mutex};
use std::thread;

struct TradingEngine {
    btc_balance: Arc<Mutex<f64>>,
    eth_balance: Arc<Mutex<f64>>,
}

impl TradingEngine {
    fn new(btc: f64, eth: f64) -> Self {
        TradingEngine {
            btc_balance: Arc::new(Mutex::new(btc)),
            eth_balance: Arc::new(Mutex::new(eth)),
        }
    }

    // Short, atomic operations
    fn get_btc_balance(&self) -> f64 {
        *self.btc_balance.lock().unwrap()
    }

    fn get_eth_balance(&self) -> f64 {
        *self.eth_balance.lock().unwrap()
    }

    fn swap(&self, btc_amount: f64, eth_amount: f64) -> Result<(), String> {
        // Check balances (short locks)
        let current_btc = self.get_btc_balance();
        let current_eth = self.get_eth_balance();

        if current_btc < btc_amount {
            return Err("Insufficient BTC".to_string());
        }

        // Execute swap (separate short locks)
        {
            let mut btc = self.btc_balance.lock().unwrap();
            *btc -= btc_amount;
        }

        {
            let mut eth = self.eth_balance.lock().unwrap();
            *eth += eth_amount;
        }

        Ok(())
    }
}

fn main() {
    let engine = Arc::new(TradingEngine::new(10.0, 100.0));

    let e1 = Arc::clone(&engine);
    let e2 = Arc::clone(&engine);

    let h1 = thread::spawn(move || {
        for i in 0..5 {
            match e1.swap(1.0, 10.0) {
                Ok(_) => println!("Swap {} successful", i + 1),
                Err(e) => println!("Swap {} error: {}", i + 1, e),
            }
        }
    });

    let h2 = thread::spawn(move || {
        for _ in 0..5 {
            println!("BTC: {}, ETH: {}",
                e2.get_btc_balance(),
                e2.get_eth_balance());
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
}
```

### 3. Single Lock for Related Data

```rust
use std::sync::{Arc, Mutex};
use std::thread;

// Group related data into one structure
#[derive(Debug, Clone)]
struct TradingState {
    btc_balance: f64,
    eth_balance: f64,
    pending_orders: u32,
}

fn main() {
    // One lock for entire state
    let state = Arc::new(Mutex::new(TradingState {
        btc_balance: 10.0,
        eth_balance: 100.0,
        pending_orders: 0,
    }));

    let state1 = Arc::clone(&state);
    let state2 = Arc::clone(&state);

    let handle1 = thread::spawn(move || {
        let mut s = state1.lock().unwrap();
        s.btc_balance -= 1.0;
        s.eth_balance += 15.0;
        s.pending_orders += 1;
        println!("Thread 1: {:?}", *s);
    });

    let handle2 = thread::spawn(move || {
        let mut s = state2.lock().unwrap();
        s.eth_balance -= 15.0;
        s.btc_balance += 1.0;
        s.pending_orders -= 1;
        println!("Thread 2: {:?}", *s);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

## Practical Example: Safe Trading Engine

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

#[derive(Debug)]
struct SafeTradingEngine {
    // All data under one lock — no deadlock
    state: Mutex<EngineState>,
}

#[derive(Debug)]
struct EngineState {
    cash: f64,
    positions: HashMap<String, Position>,
    order_count: u64,
}

impl SafeTradingEngine {
    fn new(initial_cash: f64) -> Self {
        SafeTradingEngine {
            state: Mutex::new(EngineState {
                cash: initial_cash,
                positions: HashMap::new(),
                order_count: 0,
            }),
        }
    }

    fn buy(&self, symbol: &str, quantity: f64, price: f64) -> Result<u64, String> {
        let mut state = self.state.lock().unwrap();

        let cost = quantity * price;
        if state.cash < cost {
            return Err(format!(
                "Insufficient funds: need {}, have {}",
                cost, state.cash
            ));
        }

        state.cash -= cost;
        state.order_count += 1;
        let order_id = state.order_count;

        state.positions
            .entry(symbol.to_string())
            .and_modify(|pos| {
                let total_qty = pos.quantity + quantity;
                let total_cost = pos.avg_price * pos.quantity + price * quantity;
                pos.avg_price = total_cost / total_qty;
                pos.quantity = total_qty;
            })
            .or_insert(Position {
                symbol: symbol.to_string(),
                quantity,
                avg_price: price,
            });

        Ok(order_id)
    }

    fn sell(&self, symbol: &str, quantity: f64, price: f64) -> Result<u64, String> {
        let mut state = self.state.lock().unwrap();

        let position = state.positions.get(symbol)
            .ok_or_else(|| format!("No position in {}", symbol))?;

        if position.quantity < quantity {
            return Err(format!(
                "Insufficient {}: have {}, need {}",
                symbol, position.quantity, quantity
            ));
        }

        let revenue = quantity * price;
        state.cash += revenue;
        state.order_count += 1;
        let order_id = state.order_count;

        if let Some(pos) = state.positions.get_mut(symbol) {
            pos.quantity -= quantity;
            if pos.quantity <= 0.0 {
                state.positions.remove(symbol);
            }
        }

        Ok(order_id)
    }

    fn get_status(&self) -> String {
        let state = self.state.lock().unwrap();
        format!(
            "Balance: ${:.2}, Positions: {}, Orders: {}",
            state.cash,
            state.positions.len(),
            state.order_count
        )
    }
}

fn main() {
    let engine = Arc::new(SafeTradingEngine::new(100_000.0));

    let e1 = Arc::clone(&engine);
    let e2 = Arc::clone(&engine);
    let e3 = Arc::clone(&engine);

    let buyer = thread::spawn(move || {
        for i in 0..10 {
            match e1.buy("BTC", 0.1, 42000.0 + i as f64 * 100.0) {
                Ok(id) => println!("Buy #{}: order {}", i + 1, id),
                Err(e) => println!("Buy error: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    let seller = thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(200)); // Wait for position buildup
        for i in 0..5 {
            match e2.sell("BTC", 0.1, 43000.0 + i as f64 * 100.0) {
                Ok(id) => println!("Sell #{}: order {}", i + 1, id),
                Err(e) => println!("Sell error: {}", e),
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    let monitor = thread::spawn(move || {
        for _ in 0..10 {
            println!("Status: {}", e3.get_status());
            thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    buyer.join().unwrap();
    seller.join().unwrap();
    monitor.join().unwrap();

    println!("\nFinal status: {}", engine.get_status());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Deadlock | Mutual blocking of threads waiting for each other |
| 4 Deadlock Conditions | Mutual exclusion, Hold and wait, No preemption, Circular wait |
| `try_lock()` | Non-blocking attempt to acquire a mutex |
| Consistent Ordering | Always lock resources in the same order |
| Short Locks | Minimize lock holding duration |
| Data Grouping | Related data under one lock |

## Homework

1. **Creating Deadlock**: Write a program with three mutexes and three threads that guarantees a deadlock. Add logging to see at which point the blocking occurs.

2. **Fixing Deadlock**: Take the program from exercise 1 and fix it using:
   - Consistent lock ordering
   - `try_lock()` with retry logic

3. **Safe Exchange**: Implement an `Exchange` struct with methods:
   - `place_order(order: Order)` — place an order
   - `cancel_order(order_id: u64)` — cancel an order
   - `get_order_book()` — get the order book

   Make sure multiple threads can safely call these methods without risk of deadlock.

4. **Deadlock Detector**: Using `try_lock()` and an attempt counter, create a wrapper function that:
   - Tries to acquire multiple mutexes
   - If it fails after N attempts — logs a warning about possible deadlock
   - Returns a `Result` with success or error information

## Navigation

[← Previous day](../163-rwlock-readers-writer/en.md) | [Next day →](../165-avoiding-deadlock-lock-ordering/en.md)
