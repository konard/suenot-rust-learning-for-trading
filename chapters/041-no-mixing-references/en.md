# Day 41: Rule — No Mixing References

## Trading Analogy

Imagine a trading terminal displaying an order book. **Multiple traders** can simultaneously **view** the order book — this is safe, the data doesn't change.

But what if one trader starts **editing** the order book while others are reading it? Chaos! Someone sees old data, someone sees new data, someone sees partially updated data. This is a data race.

Rust **forbids** such situations at compile time. The rule is simple:

> **Either many readers, OR one writer — but not at the same time.**

## The Borrowing Rule

Rust enforces a strict rule:

```
At any given time, you can have:
- Either ONE mutable reference (&mut T)
- OR ANY number of immutable references (&T)
- But NOT both at the same time!
```

### Why Does This Matter?

```rust
fn main() {
    let mut portfolio_value = 100000.0;

    // Create an immutable reference
    let reader = &portfolio_value;

    // Try to create a mutable reference
    let writer = &mut portfolio_value;  // ERROR!

    println!("Value: {}", reader);
}
```

The compiler will say:
```
error[E0502]: cannot borrow `portfolio_value` as mutable
              because it is also borrowed as immutable
```

**Analogy:** You can't edit a document while others are reading it. Either close the document for all readers, or wait.

## The Problem: Data Races

Imagine code without Rust's protection:

```rust
fn main() {
    let mut balance = 10000.0;

    // Thread 1: reads balance for verification
    let check = &balance;  // Sees 10000

    // Thread 2: deducts money
    let modify = &mut balance;
    *modify -= 5000.0;  // Now 5000

    // Thread 1: makes decision based on STALE data!
    println!("Check sees: {}", check);  // Still thinks it's 10000!
}
```

In a real bot, this could lead to:
- Opening a position you can no longer afford
- Incorrect risk calculations
- Losing money!

**Rust prevents this at compile time.**

## Reference Scope (NLL)

A reference "lives" until its **last use**, not until the end of the block. This is called Non-Lexical Lifetimes (NLL):

```rust
fn main() {
    let mut order_price = 42000.0;

    // Immutable reference
    let snapshot = &order_price;
    println!("Price snapshot: {}", snapshot);
    // snapshot is no longer used — reference "dies"

    // Now we can create a mutable reference!
    let update = &mut order_price;
    *update = 42500.0;
    println!("Updated price: {}", update);
}
```

This code **compiles** because `snapshot` isn't used after `println!`.

### When This Doesn't Work:

```rust
fn main() {
    let mut order_price = 42000.0;

    let snapshot = &order_price;
    let update = &mut order_price;  // ERROR!

    // snapshot is used AFTER creating update
    println!("Snapshot: {}", snapshot);
}
```

## Practical Examples

### Example 1: Reading and Updating Position

```rust
fn main() {
    let mut position_size = 1.5;  // 1.5 BTC

    // First, read
    let current = &position_size;
    println!("Current position: {} BTC", current);
    // current is no longer needed

    // Now update
    let updater = &mut position_size;
    *updater += 0.5;
    println!("After adding: {} BTC", updater);
}
```

### Example 2: Multiple Readers

```rust
fn main() {
    let portfolio_value = 150000.0;

    // Many immutable references — this is OK!
    let reader1 = &portfolio_value;
    let reader2 = &portfolio_value;
    let reader3 = &portfolio_value;

    println!("Risk manager sees: {}", reader1);
    println!("Report shows: {}", reader2);
    println!("Dashboard displays: {}", reader3);
}
```

**Analogy:** Everyone can view quotes on screen simultaneously. This is safe — nobody is changing anything.

### Example 3: Sequential Modifications

```rust
fn main() {
    let mut balance = 10000.0;

    println!("Initial balance: {}", balance);

    // First modification
    {
        let trade1 = &mut balance;
        *trade1 += 500.0;
        println!("After trade 1: {}", trade1);
    }  // trade1 goes out of scope

    // Second modification
    {
        let trade2 = &mut balance;
        *trade2 -= 200.0;
        println!("After trade 2: {}", trade2);
    }

    // Can read again
    println!("Final balance: {}", balance);
}
```

### Example 4: Error on Simultaneous Access

```rust
fn main() {
    let mut orders = vec!["BTC-USDT", "ETH-USDT", "SOL-USDT"];

    // Get a reference to the first element
    let first = &orders[0];

    // Try to add a new order
    orders.push("DOGE-USDT");  // ERROR!

    println!("First order: {}", first);
}
```

Why the error? `push` might reallocate the Vec in memory, and then `first` would point to invalid memory!

**Correct Solution:**

```rust
fn main() {
    let mut orders = vec!["BTC-USDT", "ETH-USDT", "SOL-USDT"];

    // First read and save the value
    let first = orders[0].to_string();

    // Now safe to add
    orders.push("DOGE-USDT");

    println!("First order was: {}", first);
    println!("All orders: {:?}", orders);
}
```

## Functions and Mixing References

### Error: Reading and Writing Simultaneously

```rust
fn update_and_log(value: &mut f64, log: &f64) {
    println!("Was: {}", log);
    *value += 100.0;
    println!("Now: {}", value);
}

fn main() {
    let mut price = 42000.0;

    // Can't pass one variable as both &mut and & at the same time!
    update_and_log(&mut price, &price);  // ERROR!
}
```

### Correct Solution:

```rust
fn update_and_log(value: &mut f64) {
    println!("Was: {}", *value);
    *value += 100.0;
    println!("Now: {}", *value);
}

fn main() {
    let mut price = 42000.0;
    update_and_log(&mut price);  // OK!
}
```

## Pattern: Read First, Then Write

```rust
fn analyze_and_update_position(
    current_price: f64,
    position: &mut f64,
    entry_price: f64,
) {
    // First calculate based on current value
    let current_pnl = (*position) * (current_price - entry_price);
    println!("Current PnL: {:.2} USDT", current_pnl);

    // Then update
    if current_pnl > 100.0 {
        *position *= 0.5;  // Take partial profits
        println!("Position reduced by half");
    }
}

fn main() {
    let mut position = 2.0;  // 2 BTC
    let entry_price = 40000.0;
    let current_price = 42000.0;

    analyze_and_update_position(current_price, &mut position, entry_price);
    println!("Final position: {} BTC", position);
}
```

## Real-World Example: Trading System

```rust
struct TradingAccount {
    balance: f64,
    open_positions: i32,
    total_pnl: f64,
}

fn display_account(account: &TradingAccount) {
    println!("=== Account Status ===");
    println!("Balance: {:.2} USDT", account.balance);
    println!("Open positions: {}", account.open_positions);
    println!("Total PnL: {:.2} USDT", account.total_pnl);
}

fn execute_trade(account: &mut TradingAccount, profit: f64) {
    account.balance += profit;
    account.total_pnl += profit;
    if profit > 0.0 {
        println!("Profitable trade: +{:.2}", profit);
    } else {
        println!("Losing trade: {:.2}", profit);
    }
}

fn main() {
    let mut account = TradingAccount {
        balance: 10000.0,
        open_positions: 0,
        total_pnl: 0.0,
    };

    // Read state
    display_account(&account);
    println!();

    // Execute trades
    execute_trade(&mut account, 150.0);
    execute_trade(&mut account, -50.0);
    execute_trade(&mut account, 300.0);
    println!();

    // Read again
    display_account(&account);
}
```

Output:
```
=== Account Status ===
Balance: 10000.00 USDT
Open positions: 0
Total PnL: 0.00 USDT

Profitable trade: +150.00
Losing trade: -50.00
Profitable trade: +300.00

=== Account Status ===
Balance: 10400.00 USDT
Open positions: 0
Total PnL: 400.00 USDT
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Mixing rule | Cannot have `&mut` and `&` simultaneously |
| Data race | Problem that Rust prevents |
| NLL | References live until last use |
| Multiple `&` | Many immutable references — OK |
| One `&mut` | Only one mutable reference at a time |

## Homework

1. **Fix the error:**
   ```rust
   fn main() {
       let mut prices = vec![100.0, 200.0, 300.0];
       let first = &prices[0];
       prices.push(400.0);
       println!("First price: {}", first);
   }
   ```

2. **Rewrite this code correctly:**
   ```rust
   fn main() {
       let mut balance = 5000.0;
       let reader = &balance;
       let writer = &mut balance;
       *writer += 100.0;
       println!("Reader sees: {}", reader);
   }
   ```

3. **Create a `Portfolio` struct** with fields `cash` and `positions`. Write functions:
   - `display(portfolio: &Portfolio)` — prints the state
   - `deposit(portfolio: &mut Portfolio, amount: f64)` — adds to cash
   - Demonstrate sequential calls to both functions

4. **Bonus challenge:** Explain why this code compiles:
   ```rust
   fn main() {
       let mut value = 42;
       let r1 = &value;
       let r2 = &value;
       println!("{} and {}", r1, r2);
       let r3 = &mut value;
       *r3 += 1;
       println!("{}", r3);
   }
   ```

## Navigation

[← Day 40: One Mutable Reference Rule](../040-one-mutable-reference/en.md) | [Day 42: Dangling References →](../042-dangling-references/en.md)
