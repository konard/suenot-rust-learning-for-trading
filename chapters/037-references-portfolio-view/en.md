# Day 37: References — Looking at Someone's Portfolio

## Trading Analogy

Imagine a colleague shows you their trading portfolio on their screen. You **see** all their positions, you can analyze them, but you **cannot** change anything — it's **their** portfolio. You're just looking at it through a reference.

In Rust, a **reference** (`&T`) works the same way: it lets you "look at" data without owning it. The original stays with its owner.

## What Is a Reference?

A reference is an address of data in memory. Instead of copying or transferring ownership, we pass a "pointer" to the data.

```rust
fn main() {
    let portfolio_value = 100_000.0;

    // Create a reference to the value
    let reference = &portfolio_value;

    println!("Portfolio value: ${}", portfolio_value);
    println!("Through reference: ${}", reference);

    // Both point to the same data!
}
```

## Why Do We Need References?

### Without References — Transferring Ownership

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0];

    let avg = calculate_average(prices);  // prices is moved!
    println!("Average: {}", avg);

    // println!("{:?}", prices);  // ERROR! prices is no longer available
}

fn calculate_average(data: Vec<f64>) -> f64 {
    let sum: f64 = data.iter().sum();
    sum / data.len() as f64
}
```

### With References — Borrowing for Reading

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0];

    let avg = calculate_average(&prices);  // Pass a reference!
    println!("Average: {}", avg);

    println!("Prices: {:?}", prices);  // prices is still available!
}

fn calculate_average(data: &Vec<f64>) -> f64 {
    let sum: f64 = data.iter().sum();
    sum / data.len() as f64
}
```

## Reference Syntax

```rust
fn main() {
    let btc_price = 42000.0;

    // Creating a reference: &
    let price_ref = &btc_price;

    // Dereferencing: *
    let value = *price_ref;

    println!("Original: {}", btc_price);
    println!("Through reference: {}", price_ref);    // Auto-dereferencing
    println!("Dereferenced: {}", value);
}
```

## References to Structs

```rust
struct Portfolio {
    name: String,
    total_value: f64,
    positions_count: usize,
}

fn main() {
    let my_portfolio = Portfolio {
        name: String::from("Main Trading Account"),
        total_value: 150_000.0,
        positions_count: 12,
    };

    // Pass a reference to the portfolio
    display_portfolio(&my_portfolio);

    // The portfolio is still ours!
    println!("\nMy portfolio: {} - ${:.2}",
             my_portfolio.name,
             my_portfolio.total_value);
}

fn display_portfolio(portfolio: &Portfolio) {
    println!("╔═══════════════════════════════════╗");
    println!("║         PORTFOLIO VIEW            ║");
    println!("╠═══════════════════════════════════╣");
    println!("║ Name: {:>25} ║", portfolio.name);
    println!("║ Value: ${:>23.2} ║", portfolio.total_value);
    println!("║ Positions: {:>20} ║", portfolio.positions_count);
    println!("╚═══════════════════════════════════╝");
}
```

## Multiple Read References

You can create as many read references as you want simultaneously:

```rust
fn main() {
    let market_data = vec![42000.0, 42100.0, 41900.0, 42300.0, 42200.0];

    // Multiple references to the same data — that's fine!
    let ref1 = &market_data;
    let ref2 = &market_data;
    let ref3 = &market_data;

    // All can read simultaneously
    println!("Analyst 1 sees: {:?}", ref1);
    println!("Analyst 2 sees: {:?}", ref2);
    println!("Analyst 3 sees: {:?}", ref3);

    // It's like multiple traders looking at the same chart
}
```

## References in Analysis Functions

```rust
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
}

fn main() {
    let trades = vec![
        Trade { symbol: String::from("BTC"), entry_price: 42000.0, exit_price: 43500.0, quantity: 0.5 },
        Trade { symbol: String::from("ETH"), entry_price: 2200.0, exit_price: 2350.0, quantity: 5.0 },
        Trade { symbol: String::from("BTC"), entry_price: 43000.0, exit_price: 42500.0, quantity: 0.3 },
    ];

    // Pass references — don't take ownership
    let total_pnl = calculate_total_pnl(&trades);
    let profitable = count_profitable_trades(&trades);
    let win_rate = calculate_win_rate(&trades);

    println!("Total PnL: ${:.2}", total_pnl);
    println!("Profitable trades: {}", profitable);
    println!("Win rate: {:.1}%", win_rate * 100.0);

    // trades is still available for further analysis!
    for trade in &trades {
        let pnl = (trade.exit_price - trade.entry_price) * trade.quantity;
        println!("{}: ${:.2}", trade.symbol, pnl);
    }
}

fn calculate_total_pnl(trades: &Vec<Trade>) -> f64 {
    trades.iter()
        .map(|t| (t.exit_price - t.entry_price) * t.quantity)
        .sum()
}

fn count_profitable_trades(trades: &Vec<Trade>) -> usize {
    trades.iter()
        .filter(|t| t.exit_price > t.entry_price)
        .count()
}

fn calculate_win_rate(trades: &Vec<Trade>) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }
    let profitable = count_profitable_trades(trades);
    profitable as f64 / trades.len() as f64
}
```

## References to Slices

Instead of `&Vec<T>`, it's often better to use `&[T]` — a slice:

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    // Full slice
    let sma5 = calculate_sma(&prices);
    println!("SMA-5: {:.2}", sma5);

    // Last 3 prices
    let sma3 = calculate_sma(&prices[2..]);
    println!("SMA-3 (last): {:.2}", sma3);

    // First 3 prices
    let sma3_first = calculate_sma(&prices[..3]);
    println!("SMA-3 (first): {:.2}", sma3_first);
}

// Accepts a slice — works with Vec, arrays, and parts of them
fn calculate_sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## String References: &str vs &String

```rust
fn main() {
    let ticker_owned = String::from("BTCUSDT");
    let ticker_literal = "ETHUSDT";  // This is already &str

    // Both work with a function accepting &str
    display_ticker(&ticker_owned);   // &String auto-converts to &str
    display_ticker(ticker_literal);  // &str stays &str
}

fn display_ticker(ticker: &str) {
    println!("Trading: {}", ticker);
}
```

## Practical Example: Portfolio Analysis

```rust
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

struct Portfolio {
    name: String,
    positions: Vec<Position>,
}

fn main() {
    let portfolio = Portfolio {
        name: String::from("Crypto Portfolio"),
        positions: vec![
            Position { symbol: String::from("BTC"), quantity: 1.5, entry_price: 40000.0, current_price: 42000.0 },
            Position { symbol: String::from("ETH"), quantity: 10.0, entry_price: 2000.0, current_price: 2200.0 },
            Position { symbol: String::from("SOL"), quantity: 50.0, entry_price: 100.0, current_price: 95.0 },
        ],
    };

    // All functions take references — portfolio is not moved
    println!("Portfolio: {}", portfolio.name);
    println!("Total Value: ${:.2}", calculate_portfolio_value(&portfolio));
    println!("Total PnL: ${:.2}", calculate_portfolio_pnl(&portfolio));
    println!("Unrealized PnL%: {:.2}%", calculate_portfolio_pnl_percent(&portfolio));

    print_portfolio_summary(&portfolio);
}

fn calculate_portfolio_value(portfolio: &Portfolio) -> f64 {
    portfolio.positions.iter()
        .map(|p| p.quantity * p.current_price)
        .sum()
}

fn calculate_portfolio_pnl(portfolio: &Portfolio) -> f64 {
    portfolio.positions.iter()
        .map(|p| (p.current_price - p.entry_price) * p.quantity)
        .sum()
}

fn calculate_portfolio_pnl_percent(portfolio: &Portfolio) -> f64 {
    let cost: f64 = portfolio.positions.iter()
        .map(|p| p.entry_price * p.quantity)
        .sum();

    if cost == 0.0 {
        return 0.0;
    }

    let pnl = calculate_portfolio_pnl(portfolio);
    (pnl / cost) * 100.0
}

fn print_portfolio_summary(portfolio: &Portfolio) {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║                PORTFOLIO SUMMARY                 ║");
    println!("╠══════════════════════════════════════════════════╣");

    for position in &portfolio.positions {
        let pnl = (position.current_price - position.entry_price) * position.quantity;
        let pnl_symbol = if pnl >= 0.0 { "+" } else { "" };
        println!("║ {:6} | {:>6.2} units | PnL: {}{:>10.2} ║",
                 position.symbol, position.quantity, pnl_symbol, pnl);
    }

    println!("╚══════════════════════════════════════════════════╝");
}
```

## Reference Rules

1. **A reference cannot outlive the data**
```rust
fn main() {
    let reference;
    {
        let price = 42000.0;
        reference = &price;
    }  // price is dropped
    // println!("{}", reference);  // ERROR! Data no longer exists
}
```

2. **Cannot modify data through a regular reference**
```rust
fn main() {
    let price = 42000.0;
    let reference = &price;

    // *reference = 43000.0;  // ERROR! Reference is read-only
}
```

3. **Many readers is fine**
```rust
fn main() {
    let data = vec![1, 2, 3];
    let r1 = &data;
    let r2 = &data;
    let r3 = &data;
    println!("{:?} {:?} {:?}", r1, r2, r3);  // OK
}
```

## Exercises

### Exercise 1: Price Array Analysis
```rust
// Implement functions using references

fn find_min_price(prices: &[f64]) -> f64 {
    // Find the minimum price
    todo!()
}

fn find_max_price(prices: &[f64]) -> f64 {
    // Find the maximum price
    todo!()
}

fn calculate_volatility(prices: &[f64]) -> f64 {
    // Calculate volatility: (max - min) / min * 100
    todo!()
}

fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    println!("Min: ${:.2}", find_min_price(&prices));
    println!("Max: ${:.2}", find_max_price(&prices));
    println!("Volatility: {:.2}%", calculate_volatility(&prices));
}
```

### Exercise 2: Trade Filtering
```rust
struct Trade {
    symbol: String,
    pnl: f64,
}

fn filter_by_symbol<'a>(trades: &'a [Trade], symbol: &str) -> Vec<&'a Trade> {
    // Return references to trades with the specified symbol
    todo!()
}

fn calculate_symbol_pnl(trades: &[Trade], symbol: &str) -> f64 {
    // Calculate PnL for a specific symbol
    todo!()
}
```

### Exercise 3: Find Best Trade
```rust
struct Trade {
    id: u32,
    pnl: f64,
}

fn find_best_trade(trades: &[Trade]) -> Option<&Trade> {
    // Return a reference to the trade with maximum PnL
    todo!()
}

fn find_worst_trade(trades: &[Trade]) -> Option<&Trade> {
    // Return a reference to the trade with minimum PnL
    todo!()
}
```

## What We Learned

| Concept | Syntax | Description |
|---------|--------|-------------|
| Create reference | `&value` | Get address of value |
| Reference parameter | `fn foo(x: &T)` | Function borrows data |
| Slice | `&[T]` | Reference to part of collection |
| Dereference | `*reference` | Get value by reference |
| String slice | `&str` | Reference to string data |

## Homework

1. Write a function `analyze_order_book(bids: &[f64], asks: &[f64]) -> (f64, f64, f64)` that returns the best bid, best ask, and spread

2. Create a `MarketData` struct with price history and write technical analysis functions using references:
   - `calculate_ema(data: &MarketData, period: usize) -> f64`
   - `find_support_resistance(data: &MarketData) -> (f64, f64)`

3. Implement a function `compare_portfolios(p1: &Portfolio, p2: &Portfolio) -> PortfolioComparison` that compares two portfolios without owning them

4. Write a function `get_top_performers(positions: &[Position], n: usize) -> Vec<&Position>` that returns references to the N best positions by PnL

## Navigation

[← Previous day](../036-copy-lightweight-types/en.md) | [Next day →](../038-borrowing-temporary-access/en.md)
