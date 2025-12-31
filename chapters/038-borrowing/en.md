# Day 38: Borrowing — Temporary Data Access

## Trading Analogy

Imagine you're managing a trading fund. You have a **portfolio** (data), and different people want to work with it:

- **Analysts** want to **view** the portfolio to assess risks
- **Auditor** wants to **check** the position composition
- **Risk manager** wants to **modify** the limits

The key point: you **don't transfer ownership** of the portfolio — you only **grant temporary access**. Moreover:
- **Many can view simultaneously** (read-only access)
- Only **one person can modify** (exclusive access)

In Rust, this is called **borrowing**.

## What is Borrowing?

Borrowing is creating a **reference** to data without transferring ownership. The owner remains the same, but other code gets temporary access.

```rust
fn main() {
    let portfolio = String::from("BTC: 2.5, ETH: 10.0, SOL: 100.0");

    // Create a reference — borrow the data
    let portfolio_ref = &portfolio;

    println!("Viewing: {}", portfolio_ref);
    println!("Original: {}", portfolio);  // portfolio is still accessible!
}
```

The `&` symbol creates a **reference** — it's like letting someone view a document without giving it away.

## Two Types of References

### 1. Immutable Reference `&T`

Allows only **reading** data:

```rust
fn main() {
    let btc_price = 42000.0;

    // Multiple immutable references at once — this is OK
    let ref1 = &btc_price;
    let ref2 = &btc_price;
    let ref3 = &btc_price;

    println!("Terminal 1 sees: {}", ref1);
    println!("Terminal 2 sees: {}", ref2);
    println!("Terminal 3 sees: {}", ref3);
}
```

**Analogy:** Multiple traders looking at the same order book — everyone sees the same data.

### 2. Mutable Reference `&mut T`

Allows **modifying** data:

```rust
fn main() {
    let mut balance = 10000.0;

    // Create a mutable reference
    let balance_ref = &mut balance;

    // Through the reference we can modify the value
    *balance_ref += 500.0;  // Add profit

    println!("New balance: {}", balance);
}
```

The `*` operator (dereferencing) allows accessing the value through the reference.

## Borrowing Rules

Rust strictly enforces safety. Here are the main rules:

### Rule 1: Many Readers OR One Writer

```rust
fn main() {
    let mut order_book = String::from("Bid: 42000, Ask: 42010");

    // OK: many immutable references
    let view1 = &order_book;
    let view2 = &order_book;
    println!("{} | {}", view1, view2);

    // OK: one mutable reference
    let editor = &mut order_book;
    editor.push_str(", Spread: 10");
    println!("{}", editor);
}
```

### Rule 2: Cannot Mix Mutable and Immutable References

```rust
fn main() {
    let mut price = 42000.0;

    let reader = &price;        // immutable reference
    // let writer = &mut price; // ERROR! Cannot create &mut while & exists

    println!("Price: {}", reader);

    // After the last use of reader, we can create &mut
    let writer = &mut price;
    *writer = 42500.0;
}
```

**Analogy:** While auditors are reviewing a report (reading), no one can edit it.

## Borrowing in Functions

The most common use — passing data to functions without losing ownership:

```rust
// Function takes a reference — only reads
fn calculate_position_value(price: &f64, quantity: &f64) -> f64 {
    price * quantity
}

// Function takes a mutable reference — can modify
fn apply_fee(balance: &mut f64, fee_percent: f64) {
    let fee = *balance * fee_percent / 100.0;
    *balance -= fee;
}

fn main() {
    let btc_price = 42000.0;
    let btc_quantity = 0.5;
    let mut balance = 10000.0;

    // Pass references — ownership stays with us
    let value = calculate_position_value(&btc_price, &btc_quantity);
    println!("Position value: {} USDT", value);

    // Pass a mutable reference
    apply_fee(&mut balance, 0.1);
    println!("Balance after fee: {} USDT", balance);

    // Variables are still ours!
    println!("BTC price: {}", btc_price);
}
```

## Practical Example: Portfolio Analyzer

```rust
struct Portfolio {
    btc: f64,
    eth: f64,
    usdt: f64,
}

// Read only — immutable reference
fn total_value(portfolio: &Portfolio, btc_price: f64, eth_price: f64) -> f64 {
    portfolio.btc * btc_price + portfolio.eth * eth_price + portfolio.usdt
}

// Read only — risk check
fn check_concentration(portfolio: &Portfolio, btc_price: f64, eth_price: f64) -> bool {
    let total = total_value(portfolio, btc_price, eth_price);
    let btc_share = (portfolio.btc * btc_price) / total * 100.0;

    if btc_share > 50.0 {
        println!("Warning! BTC is {:.1}% of portfolio", btc_share);
        return false;
    }
    true
}

// Modification — rebalancing
fn rebalance(portfolio: &mut Portfolio, sell_btc: f64, btc_price: f64) {
    portfolio.btc -= sell_btc;
    portfolio.usdt += sell_btc * btc_price;
    println!("Sold {} BTC at price {}", sell_btc, btc_price);
}

fn main() {
    let mut my_portfolio = Portfolio {
        btc: 2.0,
        eth: 10.0,
        usdt: 5000.0,
    };

    let btc_price = 42000.0;
    let eth_price = 2200.0;

    // Read multiple times — OK
    let value = total_value(&my_portfolio, btc_price, eth_price);
    println!("Portfolio value: {} USDT", value);

    let is_balanced = check_concentration(&my_portfolio, btc_price, eth_price);

    if !is_balanced {
        // Modify the portfolio
        rebalance(&mut my_portfolio, 0.5, btc_price);

        // Check the result
        let new_value = total_value(&my_portfolio, btc_price, eth_price);
        println!("New value: {} USDT", new_value);
    }
}
```

## Example: Trade Monitoring

```rust
struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

// Immutable reference — trade analysis
fn analyze_trade(trade: &Trade) {
    let value = trade.price * trade.quantity;
    println!(
        "Analysis: {} {} {} at {} = {} USDT",
        trade.side, trade.quantity, trade.symbol, trade.price, value
    );
}

// Immutable reference — limit check
fn check_limits(trade: &Trade, max_value: f64) -> bool {
    let value = trade.price * trade.quantity;
    if value > max_value {
        println!("Limit exceeded! {} > {}", value, max_value);
        return false;
    }
    true
}

// Mutable reference — quantity adjustment
fn adjust_quantity(trade: &mut Trade, max_value: f64) {
    let current_value = trade.price * trade.quantity;
    if current_value > max_value {
        trade.quantity = max_value / trade.price;
        println!("Quantity adjusted to {}", trade.quantity);
    }
}

fn main() {
    let mut trade = Trade {
        symbol: String::from("BTC/USDT"),
        side: String::from("BUY"),
        price: 42000.0,
        quantity: 1.0,
    };

    let max_trade_value = 30000.0;

    // Analyze (read)
    analyze_trade(&trade);

    // Check limits (read)
    if !check_limits(&trade, max_trade_value) {
        // Adjust (modify)
        adjust_quantity(&mut trade, max_trade_value);

        // Re-analyze
        analyze_trade(&trade);
    }
}
```

## Reference Lifetimes

A reference cannot outlive the data it points to:

```rust
fn main() {
    let reference;

    {
        let price = 42000.0;
        reference = &price;
        println!("Inside block: {}", reference);
    } // price is destroyed here

    // println!("{}", reference);  // ERROR! price no longer exists
}
```

Rust checks this at compile time — dangling references are impossible!

## Slices — A Special Kind of Borrowing

Slices allow borrowing a part of a collection:

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 42050.0, 41900.0, 42200.0];

    // Slice — a reference to part of the vector
    let last_three = &prices[2..5];

    println!("Last 3 prices: {:?}", last_three);
    println!("All prices: {:?}", prices);  // Original is accessible
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `&T` | Immutable reference (many simultaneously) |
| `&mut T` | Mutable reference (only one) |
| `*` | Dereference operator |
| Rules | Many & OR one &mut, but not together |
| Functions | Borrowing to pass without losing ownership |

## Practical Exercises

### Exercise 1: Risk Calculator
Write a function `calculate_risk` that takes an immutable reference to a `Position` struct and returns the risk amount:

```rust
struct Position {
    entry_price: f64,
    stop_loss: f64,
    quantity: f64,
}

// Implement the function
fn calculate_risk(position: &Position) -> f64 {
    // Risk = (entry_price - stop_loss) * quantity
    todo!()
}
```

### Exercise 2: Stop-Loss Update
Write a function `update_stop_loss` that takes a mutable reference and updates the stop-loss:

```rust
fn update_stop_loss(position: &mut Position, new_stop: f64) {
    // Update the position's stop_loss
    todo!()
}
```

### Exercise 3: Multiple Analysis
Create several functions that analyze a portfolio through immutable references:

```rust
fn get_largest_position(portfolio: &Portfolio) -> &str {
    // Return the name of the largest position
    todo!()
}

fn count_positions(portfolio: &Portfolio) -> usize {
    // Count positions > 0
    todo!()
}
```

### Exercise 4: Safe Modification
Write a function that increases a position only if there are enough free funds:

```rust
fn add_to_position(portfolio: &mut Portfolio, amount: f64, price: f64) -> bool {
    // Check if there's enough usdt
    // If yes — add to btc and subtract from usdt
    // Return true on success, false on failure
    todo!()
}
```

## Homework

1. **Trade Journal**: Create a `TradeLog` struct with a vector of trades. Write functions:
   - `add_trade(&mut TradeLog, Trade)` — add a trade
   - `total_pnl(&TradeLog) -> f64` — calculate total PnL
   - `best_trade(&TradeLog) -> &Trade` — return a reference to the best trade

2. **Limit System**: Create a `RiskLimits` struct with limits. Write:
   - `check_trade(&RiskLimits, &Trade) -> bool` — trade validation
   - `update_limit(&mut RiskLimits, limit_name: &str, new_value: f64)` — limit update

3. **Experiment**: Try creating both `&` and `&mut` references to the same variable simultaneously. Read the error message and explain why Rust prohibits this.

## Navigation

[← Previous day](../037-ownership-basics/en.md) | [Next day →](../039-lifetimes-intro/en.md)
