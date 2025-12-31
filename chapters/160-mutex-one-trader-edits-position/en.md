# Day 160: Mutex: One Trader Edits Position

## Trading Analogy

Imagine a trading floor where several traders work with a shared portfolio. If two traders try to modify the position simultaneously — chaos ensues: one is buying, another is selling, and nobody knows the actual balance.

**Mutex (Mutual Exclusion)** is like a key to the safe containing position documents. Only one trader can hold the key at any given moment. Others wait until the current holder finishes and returns the key.

## Why We Need Mutex

In multi-threaded programs, multiple threads may try to modify the same data simultaneously. Without synchronization, this leads to **data races** — unpredictable results.

```rust
use std::sync::Mutex;

fn main() {
    // Create a Mutex protecting account balance
    let balance = Mutex::new(10000.0_f64);

    // Access through lock()
    {
        let mut guard = balance.lock().unwrap();
        *guard -= 1500.0; // Deduct funds for purchase
        println!("Balance after purchase: ${:.2}", *guard);
    } // guard goes out of scope — lock is released

    // Now other code can access
    {
        let guard = balance.lock().unwrap();
        println!("Current balance: ${:.2}", *guard);
    }
}
```

**Key points:**
- `Mutex::new(value)` — creates a Mutex with initial value
- `lock()` — locks the Mutex and returns a `MutexGuard`
- `MutexGuard` — smart pointer with automatic lock release

## Why lock() Returns Result

```rust
use std::sync::Mutex;

fn main() {
    let position = Mutex::new(100);

    // lock() returns Result<MutexGuard, PoisonError>
    match position.lock() {
        Ok(guard) => println!("Position: {} shares", *guard),
        Err(poisoned) => {
            // Mutex is "poisoned" — previous thread panicked while holding lock
            println!("Warning: Mutex was poisoned!");
            let guard = poisoned.into_inner();
            println!("Recovered position: {}", *guard);
        }
    }
}
```

## Modifying Data Through Mutex

```rust
use std::sync::Mutex;

#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: i32,
    avg_price: f64,
}

fn main() {
    let position = Mutex::new(Position {
        symbol: String::from("AAPL"),
        quantity: 100,
        avg_price: 150.0,
    });

    // Add to position
    {
        let mut pos = position.lock().unwrap();
        let add_qty = 50;
        let add_price = 155.0;

        // Recalculate average price
        let total_value = pos.avg_price * pos.quantity as f64
                        + add_price * add_qty as f64;
        pos.quantity += add_qty;
        pos.avg_price = total_value / pos.quantity as f64;

        println!("Position updated: {:?}", *pos);
    }

    // Read final position
    let pos = position.lock().unwrap();
    println!("Total: {} shares of {} at ${:.2}",
             pos.quantity, pos.symbol, pos.avg_price);
}
```

## Practical Example: Trading Balance

```rust
use std::sync::Mutex;

struct TradingAccount {
    balance: Mutex<f64>,
    trades_count: Mutex<u32>,
}

impl TradingAccount {
    fn new(initial_balance: f64) -> Self {
        TradingAccount {
            balance: Mutex::new(initial_balance),
            trades_count: Mutex::new(0),
        }
    }

    fn buy(&self, symbol: &str, price: f64, quantity: u32) -> Result<(), String> {
        let cost = price * quantity as f64;

        let mut balance = self.balance.lock().unwrap();
        if *balance < cost {
            return Err(format!("Insufficient funds: need ${:.2}, have ${:.2}",
                              cost, *balance));
        }

        *balance -= cost;

        let mut trades = self.trades_count.lock().unwrap();
        *trades += 1;

        println!("Bought {} {} at ${:.2}. Remaining: ${:.2}",
                 quantity, symbol, price, *balance);
        Ok(())
    }

    fn sell(&self, symbol: &str, price: f64, quantity: u32) {
        let revenue = price * quantity as f64;

        let mut balance = self.balance.lock().unwrap();
        *balance += revenue;

        let mut trades = self.trades_count.lock().unwrap();
        *trades += 1;

        println!("Sold {} {} at ${:.2}. Balance: ${:.2}",
                 quantity, symbol, price, *balance);
    }

    fn get_stats(&self) -> (f64, u32) {
        let balance = *self.balance.lock().unwrap();
        let trades = *self.trades_count.lock().unwrap();
        (balance, trades)
    }
}

fn main() {
    let account = TradingAccount::new(10000.0);

    // Series of trades
    account.buy("AAPL", 150.0, 10).unwrap();
    account.buy("GOOGL", 140.0, 5).unwrap();
    account.sell("AAPL", 155.0, 5);

    let (balance, trades) = account.get_stats();
    println!("\n=== Statistics ===");
    println!("Balance: ${:.2}", balance);
    println!("Trades: {}", trades);
}
```

## try_lock: Non-Blocking Lock Attempt

```rust
use std::sync::Mutex;

fn main() {
    let order_book = Mutex::new(vec!["BUY 100 AAPL @ 150"]);

    // Acquire lock
    let _guard = order_book.lock().unwrap();

    // Attempt to lock without waiting
    match order_book.try_lock() {
        Ok(guard) => println!("Orders: {:?}", *guard),
        Err(_) => println!("Order book is busy with another trader"),
    }

    // _guard still holds the lock, so try_lock failed
}
```

## Mutex in Portfolio Structure

```rust
use std::sync::Mutex;
use std::collections::HashMap;

struct Portfolio {
    positions: Mutex<HashMap<String, i32>>,
    cash: Mutex<f64>,
}

impl Portfolio {
    fn new(initial_cash: f64) -> Self {
        Portfolio {
            positions: Mutex::new(HashMap::new()),
            cash: Mutex::new(initial_cash),
        }
    }

    fn update_position(&self, symbol: &str, delta: i32, price: f64) {
        let cost = price * delta.abs() as f64;

        // Lock cash
        {
            let mut cash = self.cash.lock().unwrap();
            if delta > 0 {
                *cash -= cost; // Buy
            } else {
                *cash += cost; // Sell
            }
        }

        // Lock positions
        {
            let mut positions = self.positions.lock().unwrap();
            let current = positions.entry(symbol.to_string()).or_insert(0);
            *current += delta;

            // Remove zero positions
            if *current == 0 {
                positions.remove(symbol);
            }
        }
    }

    fn print_portfolio(&self) {
        let positions = self.positions.lock().unwrap();
        let cash = self.cash.lock().unwrap();

        println!("\n╔════════════════════════════╗");
        println!("║        PORTFOLIO           ║");
        println!("╠════════════════════════════╣");
        for (symbol, qty) in positions.iter() {
            println!("║ {:6} {:>16} pcs ║", symbol, qty);
        }
        println!("╠════════════════════════════╣");
        println!("║ Cash:     ${:>14.2} ║", *cash);
        println!("╚════════════════════════════╝");
    }
}

fn main() {
    let portfolio = Portfolio::new(50000.0);

    portfolio.update_position("AAPL", 100, 150.0);
    portfolio.update_position("GOOGL", 50, 140.0);
    portfolio.update_position("MSFT", 75, 380.0);
    portfolio.update_position("AAPL", -30, 155.0);

    portfolio.print_portfolio();
}
```

## Important Mutex Rules

### 1. Minimize Lock Duration

```rust
use std::sync::Mutex;

fn main() {
    let data = Mutex::new(vec![1, 2, 3, 4, 5]);

    // Bad: long lock duration
    // let guard = data.lock().unwrap();
    // expensive_calculation(&guard);  // Long operation
    // another_calculation(&guard);

    // Good: quick copy and release
    let local_copy = {
        let guard = data.lock().unwrap();
        guard.clone()
    }; // Lock released

    // Now work with copy without holding lock
    let sum: i32 = local_copy.iter().sum();
    println!("Sum: {}", sum);
}
```

### 2. Avoid Nested Locks (Deadlock Risk)

```rust
use std::sync::Mutex;

struct Account {
    balance: Mutex<f64>,
}

// Potential deadlock when transferring between accounts!
// fn transfer_bad(from: &Account, to: &Account, amount: f64) {
//     let mut from_balance = from.balance.lock().unwrap();
//     let mut to_balance = to.balance.lock().unwrap(); // May deadlock
//     ...
// }

// Solution: always lock in consistent order
fn transfer_safe(from: &Account, to: &Account, amount: f64) {
    // Use addresses to determine order
    let (first, second, is_from_first) = {
        let from_ptr = from as *const _ as usize;
        let to_ptr = to as *const _ as usize;
        if from_ptr < to_ptr {
            (&from.balance, &to.balance, true)
        } else {
            (&to.balance, &from.balance, false)
        }
    };

    let mut first_guard = first.lock().unwrap();
    let mut second_guard = second.lock().unwrap();

    if is_from_first {
        *first_guard -= amount;
        *second_guard += amount;
    } else {
        *second_guard -= amount;
        *first_guard += amount;
    }

    println!("Transfer of ${:.2} completed", amount);
}

fn main() {
    let account1 = Account { balance: Mutex::new(1000.0) };
    let account2 = Account { balance: Mutex::new(500.0) };

    transfer_safe(&account1, &account2, 200.0);

    println!("Account 1: ${:.2}", *account1.balance.lock().unwrap());
    println!("Account 2: ${:.2}", *account2.balance.lock().unwrap());
}
```

## Pattern: Protected Resource

```rust
use std::sync::Mutex;

// Wrapper for thread-safe resource
struct Protected<T> {
    data: Mutex<T>,
}

impl<T> Protected<T> {
    fn new(value: T) -> Self {
        Protected {
            data: Mutex::new(value),
        }
    }

    fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.data.lock().unwrap();
        f(&mut *guard)
    }
}

fn main() {
    let balance = Protected::new(10000.0_f64);

    // Convenient API for working with protected data
    balance.with(|b| {
        *b -= 1500.0;
        println!("New balance: ${:.2}", b);
    });

    let current = balance.with(|b| *b);
    println!("Current balance: ${:.2}", current);
}
```

## What We Learned

| Concept | Description | Trading Application |
|---------|-------------|---------------------|
| `Mutex::new()` | Create mutex | Protect shared data |
| `lock()` | Blocking lock | Safe position updates |
| `try_lock()` | Non-blocking attempt | Check resource availability |
| `MutexGuard` | RAII lock guard | Automatic release |
| Poison | Poisoning on panic | Thread error handling |

## Practical Exercises

1. **Trade Counter**: Create a `TradeCounter` struct with Mutex that counts BUY and SELL trades separately

2. **Position Limit**: Implement a `PositionManager` struct that prevents a position from exceeding a specified limit

3. **Trade Logger**: Create a thread-safe `TradeLogger` that records all trades to a Vec

4. **Order Book**: Implement a simple `OrderBook` with Mutex where orders can be added and removed

## Homework

1. Create a `RiskManager` struct that:
   - Stores maximum daily loss in a Mutex
   - Tracks current PnL
   - Returns an error when attempting a trade if daily limit is exhausted

2. Implement `PriceCache` — a cache of latest prices for multiple tickers:
   - Use `Mutex<HashMap<String, f64>>`
   - Add methods `update_price`, `get_price`, `get_all_prices`

3. Write a function that safely transfers funds between two `TradingAccount`s, avoiding deadlock

4. Create an `ExecutionQueue` — a queue of orders for execution:
   - Methods `push_order`, `pop_order`, `peek_order`
   - Calculate total volume in queue

## Navigation

[← Previous day](../159-sync-channel-bounded-queue/en.md) | [Next day →](../161-arc-shared-access/en.md)
