# Day 35: Clone — Copying the Portfolio

## Trading Analogy

Imagine you have a trading portfolio with positions: BTC, ETH, SOL. You want to create an **exact copy** of this portfolio to test a new strategy without affecting the original.

In the real world:
- **Original portfolio** remains untouched
- **Copy** is an independent portfolio with the same assets
- Changes to the copy don't affect the original

In Rust, this is implemented through the **Clone** trait — explicit, deep copying of data.

## The Problem: Move Takes Ownership

Recall from the previous day — transferring ownership (move) makes the original inaccessible:

```rust
fn main() {
    let portfolio = String::from("BTC: 1.5, ETH: 10.0");

    // Move — portfolio is no longer accessible
    let backup = portfolio;

    // println!("{}", portfolio);  // ERROR! Value has been moved
    println!("{}", backup);
}
```

But what if we need **both** — the original and a copy?

## The Solution: Clone

`Clone` allows you to create a complete copy of data:

```rust
fn main() {
    let portfolio = String::from("BTC: 1.5, ETH: 10.0");

    // Clone — create a copy, original remains
    let backup = portfolio.clone();

    println!("Original: {}", portfolio);  // Works!
    println!("Copy: {}", backup);          // Also works!
}
```

## How Clone Works

```rust
fn main() {
    // String is stored on the heap
    let original = String::from("AAPL");

    // .clone() creates a new string on the heap
    // with a copy of all data
    let cloned = original.clone();

    // Now we have TWO independent Strings
    // Each owns its own memory

    println!("Original addr: {:p}", original.as_ptr());
    println!("Cloned addr: {:p}", cloned.as_ptr());
    // Addresses are different — these are different memory locations
}
```

## Clone for Trading Data

```rust
fn main() {
    // Portfolio position
    let position = String::from("LONG BTC @ 42000");

    // Save to history — need a copy
    let history_entry = position.clone();

    // Send to analytics — another copy
    let for_analytics = position.clone();

    // Original is still ours
    println!("Current position: {}", position);
    println!("In history: {}", history_entry);
    println!("For analytics: {}", for_analytics);
}
```

## Clone with Vec (List of Trades)

```rust
fn main() {
    // List of orders
    let orders = vec![
        String::from("BUY BTC 0.5"),
        String::from("SELL ETH 2.0"),
        String::from("BUY SOL 100"),
    ];

    // Clone the entire vector
    let orders_backup = orders.clone();

    // Both vectors are independent
    println!("Active orders: {:?}", orders);
    println!("Backup: {:?}", orders_backup);
}
```

**Important:** When cloning `Vec<String>`, both the vector itself and all strings inside it are copied — this is **deep copying**.

## When to Use Clone

### 1. Saving a State Snapshot

```rust
fn main() {
    let mut portfolio_value = String::from("$100,000");

    // Save state at the start of the day
    let morning_snapshot = portfolio_value.clone();

    // During the day, value changes
    portfolio_value = String::from("$105,000");

    println!("Morning: {}", morning_snapshot);
    println!("Now: {}", portfolio_value);
    println!("Profit: +$5,000");
}
```

### 2. Passing to a Function Without Losing Ownership

```rust
fn analyze_order(order: String) {
    println!("Analyzing: {}", order);
    // order will be destroyed after the function
}

fn main() {
    let order = String::from("LIMIT BUY BTC @ 40000");

    // Pass a clone — original remains
    analyze_order(order.clone());

    // We can continue using it
    println!("Order is active: {}", order);
}
```

### 3. Working with Multiple Data Streams

```rust
fn main() {
    let market_data = String::from("BTC=42000,ETH=2800,SOL=100");

    // For the chart
    let for_chart = market_data.clone();

    // For notifications
    let for_alerts = market_data.clone();

    // For logging
    let for_log = market_data.clone();

    process_chart(for_chart);
    check_alerts(for_alerts);
    write_log(for_log);

    // Original for current use
    println!("Data: {}", market_data);
}

fn process_chart(data: String) {
    println!("[CHART] {}", data);
}

fn check_alerts(data: String) {
    println!("[ALERT] {}", data);
}

fn write_log(data: String) {
    println!("[LOG] {}", data);
}
```

## Clone vs Move: Operation Cost

```rust
fn main() {
    let big_data = "X".repeat(1_000_000);  // 1 million characters

    // Move — instant (just passing a pointer)
    let moved = big_data;

    // Create again for demonstration
    let big_data2 = "Y".repeat(1_000_000);

    // Clone — copies 1 million characters!
    // This is an expensive operation
    let cloned = big_data2.clone();

    println!("Moved len: {}", moved.len());
    println!("Cloned len: {}", cloned.len());
}
```

**Rule:** Use `clone()` only when you **really need** an independent copy. Don't overuse it — it's an expensive operation.

## Types with Clone

Many standard types implement Clone:

```rust
fn main() {
    // String
    let s1 = String::from("BTC");
    let s2 = s1.clone();

    // Vec
    let v1 = vec![1, 2, 3];
    let v2 = v1.clone();

    // Box
    let b1 = Box::new(42000);
    let b2 = b1.clone();

    println!("String: {} -> {}", s1, s2);
    println!("Vec: {:?} -> {:?}", v1, v2);
    println!("Box: {} -> {}", b1, b2);
}
```

## Practical Example: Order System

```rust
fn main() {
    // Create an order
    let order = String::from("LIMIT BUY BTC 0.5 @ 42000");

    // Send to exchange (need a copy for history)
    let for_exchange = order.clone();
    send_to_exchange(for_exchange);

    // Save to history
    let for_history = order.clone();
    save_to_history(for_history);

    // Display to user
    display_order(&order);  // Here we can use a reference
}

fn send_to_exchange(order: String) {
    println!("[EXCHANGE] Sent: {}", order);
}

fn save_to_history(order: String) {
    println!("[HISTORY] Saved: {}", order);
}

fn display_order(order: &String) {
    println!("[UI] Displaying: {}", order);
}
```

## Practical Example: Trade Journal

```rust
fn main() {
    let mut trade_log: Vec<String> = Vec::new();

    // Execute trades
    let trade1 = String::from("10:00 BUY BTC 0.1 @ 42000");
    trade_log.push(trade1.clone());  // Copy to log
    println!("Executed: {}", trade1);

    let trade2 = String::from("10:15 SELL ETH 1.0 @ 2800");
    trade_log.push(trade2.clone());  // Copy to log
    println!("Executed: {}", trade2);

    let trade3 = String::from("10:30 BUY SOL 10 @ 100");
    trade_log.push(trade3.clone());  // Copy to log
    println!("Executed: {}", trade3);

    // Print the journal
    println!("\n=== Trade Journal ===");
    for (i, trade) in trade_log.iter().enumerate() {
        println!("#{}: {}", i + 1, trade);
    }
}
```

## Practical Example: Portfolio Backup

```rust
fn main() {
    // Current portfolio
    let mut portfolio = vec![
        String::from("BTC: 1.5"),
        String::from("ETH: 10.0"),
        String::from("SOL: 100.0"),
    ];

    // Create backup before risky operation
    let backup = portfolio.clone();

    println!("Portfolio before operation: {:?}", portfolio);

    // Risky operation — sell everything
    portfolio.clear();
    portfolio.push(String::from("USDT: 150000"));

    println!("Portfolio after operation: {:?}", portfolio);

    // Rollback to backup
    portfolio = backup.clone();
    println!("Rollback to backup: {:?}", portfolio);
}
```

## Clone and Functions

```rust
// Function takes ownership
fn process_data(data: String) -> String {
    format!("Processed: {}", data)
}

// Function works with reference (preferred)
fn analyze_data(data: &String) -> usize {
    data.len()
}

fn main() {
    let price_data = String::from("42000.50");

    // Method 1: clone for process_data
    let result = process_data(price_data.clone());
    println!("Result: {}", result);
    println!("Original: {}", price_data);  // Still available

    // Method 2: use reference
    let length = analyze_data(&price_data);
    println!("Length: {}", length);
    println!("Original: {}", price_data);  // Untouched
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Clone` | Trait for explicit deep copying |
| `.clone()` | Method to create a copy |
| Deep copying | All nested data is copied |
| Cost | Clone can be an expensive operation |
| When to use | When you need two independent copies |

## Homework

1. **Portfolio Snapshot**
   Create a function that takes a portfolio (Vec<String>) and creates its snapshot. Then modify the original portfolio and verify that the snapshot hasn't changed.

2. **Backup System**
   Implement a simple system with three backups:
   ```
   backup1 = portfolio at 10:00
   backup2 = portfolio at 12:00
   backup3 = portfolio at 14:00
   ```
   You should be able to rollback to any backup.

3. **Trading Simulator**
   Create a list of 5 orders. Clone it for "simulation" — execute all orders in the copy (clear the list), but the original list should remain untouched.

4. **Performance Analysis**
   Write a program that creates a string of 10,000 characters and clones it 100 times. Think about: in which cases is it better to use references instead of cloning?

## Navigation

[← Previous day](../034-move-ownership/en.md) | [Next day →](../036-copy-trait/en.md)
