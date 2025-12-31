# Day 33: Ownership — Who Holds the Asset?

## Trading Analogy

Imagine you own Tesla (TSLA) shares. These shares belong to **you** — you're the real owner. You can:
- **Sell** them to someone else (transfer ownership)
- **Show** your portfolio to a friend (borrowing)
- **Close** the position (destroy)

But what you **cannot** do:
- Sell the same shares to two different people at the same time
- Use the shares after you've sold them

This is exactly how **Ownership** works in Rust! It's a system that guarantees memory safety without a garbage collector.

## Three Rules of Ownership

```rust
fn main() {
    // Rule 1: Each value has an owner
    let portfolio = String::from("BTC: 1.5, ETH: 10.0");

    // Rule 2: There can only be ONE owner at a time
    // Rule 3: When the owner goes out of scope, the value is dropped

    println!("Portfolio: {}", portfolio);
} // <- portfolio is dropped here
```

## Move: Transferring Ownership

When you assign a value to another variable, a **move** occurs:

```rust
fn main() {
    // Create an order
    let order = String::from("BUY BTC 0.5 @ 42000");

    // Transfer ownership to a new variable
    let executed_order = order;  // MOVE happens here!

    // println!("{}", order);  // ERROR! order no longer owns the data
    println!("Executed: {}", executed_order);  // OK
}
```

**Analogy:** You sold your Tesla shares. After the sale, they belong to the new owner — you can no longer use them.

## Move vs Copy

Simple types (numbers, bool) are copied, not moved:

```rust
fn main() {
    // Numbers are COPIED (Copy trait)
    let btc_price = 42000.0;
    let saved_price = btc_price;  // Copy!

    println!("BTC now: ${}", btc_price);        // OK!
    println!("Saved price: ${}", saved_price);  // OK!

    // String is MOVED (no Copy trait)
    let ticker = String::from("BTC/USDT");
    let saved_ticker = ticker;  // MOVE!

    // println!("{}", ticker);  // ERROR!
    println!("Ticker: {}", saved_ticker);  // OK
}
```

**Why is that?**
- Numbers are small and copy quickly
- Strings can be huge — copying would be expensive

## Passing Ownership to Functions

When you pass a value to a function, a move occurs:

```rust
fn main() {
    let order = String::from("SELL ETH 5.0 @ 2800");

    execute_order(order);  // Ownership transferred to function

    // println!("{}", order);  // ERROR! order is no longer valid
}

fn execute_order(order: String) {
    println!("Executing order: {}", order);
    // order is dropped at the end of this function
}
```

**Analogy:** You handed the order to a broker. After handing it over, the broker has the order — you no longer control it.

## Returning Ownership from Functions

A function can return ownership back:

```rust
fn main() {
    let order = String::from("BUY DOGE 1000 @ 0.15");

    // Pass ownership and get it back
    let confirmed_order = process_order(order);

    println!("Confirmed order: {}", confirmed_order);
}

fn process_order(order: String) -> String {
    println!("Processing: {}", order);
    // Return ownership with modification
    format!("[CONFIRMED] {}", order)
}
```

## Practical Example: Portfolio Management

```rust
fn main() {
    // Create portfolio
    let portfolio = create_portfolio();

    // Analyze (transfer ownership)
    let analyzed = analyze_portfolio(portfolio);

    // Display results
    display_results(analyzed);

    // println!("{:?}", portfolio);  // ERROR! portfolio was already moved
}

fn create_portfolio() -> String {
    String::from("BTC: 2.0, ETH: 15.0, SOL: 100.0")
}

fn analyze_portfolio(portfolio: String) -> String {
    let total_positions = portfolio.matches(',').count() + 1;
    format!("{} | Positions: {}", portfolio, total_positions)
}

fn display_results(report: String) {
    println!("═══════════════════════════════════");
    println!("PORTFOLIO REPORT");
    println!("═══════════════════════════════════");
    println!("{}", report);
    println!("═══════════════════════════════════");
}
```

## Practical Example: Order Processing

```rust
fn main() {
    // Order processing chain
    let order = create_order("BTC/USDT", "BUY", 0.5, 42000.0);
    let validated = validate_order(order);
    let executed = execute_order(validated);
    log_trade(executed);
}

fn create_order(symbol: &str, side: &str, qty: f64, price: f64) -> String {
    format!("{}|{}|{}|{}", symbol, side, qty, price)
}

fn validate_order(order: String) -> String {
    // Validate order
    let parts: Vec<&str> = order.split('|').collect();
    if parts.len() == 4 {
        format!("[VALID] {}", order)
    } else {
        format!("[INVALID] {}", order)
    }
}

fn execute_order(order: String) -> String {
    if order.starts_with("[VALID]") {
        let order_id = 12345;  // In reality — ID generation
        format!("[EXECUTED #{}] {}", order_id, order)
    } else {
        format!("[REJECTED] {}", order)
    }
}

fn log_trade(trade: String) {
    println!("Trade log: {}", trade);
}
```

## Practical Example: Risk Calculation

```rust
fn main() {
    let position = String::from("LONG BTC 1.5 @ 40000");

    // Calculate stop-loss
    let (position, stop_loss) = calculate_stop_loss(position, 0.02);

    // Calculate take-profit
    let (position, take_profit) = calculate_take_profit(position, 0.05);

    println!("Position: {}", position);
    println!("Stop-Loss: ${:.2}", stop_loss);
    println!("Take-Profit: ${:.2}", take_profit);
}

fn calculate_stop_loss(position: String, risk_percent: f64) -> (String, f64) {
    // Parse price from position
    let price: f64 = position
        .split('@')
        .last()
        .unwrap_or("0")
        .trim()
        .parse()
        .unwrap_or(0.0);

    let stop_loss = price * (1.0 - risk_percent);
    (position, stop_loss)  // Return ownership back
}

fn calculate_take_profit(position: String, profit_percent: f64) -> (String, f64) {
    let price: f64 = position
        .split('@')
        .last()
        .unwrap_or("0")
        .trim()
        .parse()
        .unwrap_or(0.0);

    let take_profit = price * (1.0 + profit_percent);
    (position, take_profit)  // Return ownership back
}
```

## Clone: Creating a Copy

Sometimes you need to keep the original. Use `.clone()`:

```rust
fn main() {
    let original_order = String::from("BUY ETH 10.0 @ 2500");

    // Create a copy for backup
    let backup = original_order.clone();

    // Send original for execution
    execute_order(original_order);

    // Backup is still available!
    println!("Backup: {}", backup);
}

fn execute_order(order: String) {
    println!("Executing: {}", order);
}
```

**Warning:** `.clone()` is explicit data copying. It can be expensive for large structures!

## Types with Copy Trait

These types are copied automatically (no move):

```rust
fn main() {
    // All these types have the Copy trait
    let shares: i32 = 100;           // Number of shares
    let price: f64 = 42000.50;       // Price
    let is_profitable: bool = true;  // Profitability
    let side: char = 'B';            // Buy/Sell

    // Pass to functions — they are copied
    print_shares(shares);
    print_price(price);

    // Originals are still available!
    println!("Shares: {}, Price: ${}", shares, price);
}

fn print_shares(s: i32) {
    println!("Shares: {}", s);
}

fn print_price(p: f64) {
    println!("Price: ${:.2}", p);
}
```

**Copy types:**
- All integers (`i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `isize`, `usize`)
- Floating-point numbers (`f32`, `f64`)
- Boolean type (`bool`)
- Character (`char`)
- Tuples of Copy types: `(i32, f64)`

## What We Learned

| Concept | Description | Trading Analogy |
|---------|-------------|-----------------|
| Ownership | Each value has an owner | Stock ownership |
| Move | Transfer of ownership | Selling an asset |
| Drop | Deletion when leaving scope | Closing a position |
| Clone | Explicit copying | Creating a duplicate order |
| Copy | Automatic copying | Numbers in reports |

## Why Is This Important for Algo Trading?

1. **Memory Safety** — no memory leaks, no dangling pointers
2. **Predictability** — clear when data is freed
3. **Performance** — no garbage collector with pauses
4. **Reliability** — compiler catches errors at compile time

```rust
fn main() {
    // Rust won't let you make this mistake:
    let order = String::from("BUY BTC");
    let order2 = order;

    // send_to_exchange(order);   // Compilation ERROR!
    // send_to_backup(order);     // You won't send one order twice!

    println!("Safe trading with {}", order2);
}
```

## Exercises

### Exercise 1: Passing a Portfolio
Fix the code so it compiles:

```rust
fn main() {
    let portfolio = String::from("BTC: 1.0, ETH: 5.0");

    show_portfolio(portfolio);
    calculate_value(portfolio);  // Error!
}

fn show_portfolio(p: String) {
    println!("Portfolio: {}", p);
}

fn calculate_value(p: String) {
    println!("Calculating value for: {}", p);
}
```

### Exercise 2: Processing Chain
Implement an order processing chain with correct ownership transfer:
1. `create_order()` → creates the order
2. `add_timestamp()` → adds timestamp
3. `sign_order()` → signs
4. `send_order()` → sends and prints

### Exercise 3: Clone vs Move
Determine where `.clone()` is needed and where you can do without it:

```rust
fn main() {
    let ticker = String::from("AAPL");
    let price: f64 = 175.50;

    log_ticker(ticker);      // Need clone?
    log_price(price);        // Need clone?

    println!("{} @ ${}", ticker, price);  // Error?
}
```

## Homework

1. **Order System:** Create functions for the complete order lifecycle (creation → validation → execution → logging) with correct ownership transfer

2. **Portfolio Manager:** Write a program that:
   - Creates a portfolio
   - Adds positions (using ownership return)
   - Calculates total value
   - Outputs a report

3. **Trade Logger:** Implement a trade logger where each trade goes through multiple processing stages, and metadata is added at each stage

4. **Explore:** What happens if you try to use a variable after a move? What messages does the compiler give?

## Navigation

[← Previous day](../032-statements-expressions/en.md) | [Next day →](../034-references-borrowing/en.md)
