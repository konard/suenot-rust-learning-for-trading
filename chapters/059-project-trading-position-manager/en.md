# Day 59: Project — Trading Position Manager

## Project Overview

Today we'll build a complete **Trading Position Manager** — a mini-application that combines all ownership concepts learned throughout the month. This project demonstrates how Rust's ownership system helps create safe and reliable trading systems.

## Trading Analogy

Imagine a stock trading terminal:
- **Position** — an asset you own. Like in Rust, each position has one owner
- **Transferring a position** — when you close a position, ownership is transferred (move)
- **Viewing a position** — you can show a position to an analyst without transferring ownership (borrow)
- **Modifying a position** — to change position size, you need exclusive access (mutable borrow)

## Project Architecture

```
trading_position_manager/
├── Cargo.toml
└── src/
    └── main.rs
```

## Step 1: Defining Data Structures

```rust
/// Trading position — the basic unit of our system
#[derive(Debug, Clone)]
struct Position {
    symbol: String,      // Asset ticker (owns the string)
    quantity: f64,       // Quantity
    entry_price: f64,    // Entry price
    side: Side,          // Direction
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Side {
    Long,   // Buy
    Short,  // Sell
}

/// Portfolio — owns a collection of positions
struct Portfolio {
    name: String,
    positions: Vec<Position>,  // Vector owns the positions
    balance: f64,
}
```

**Ownership Concept:** `Portfolio` owns the `positions` vector, which in turn owns each `Position`. When `Portfolio` goes out of scope, all positions are automatically deallocated.

## Step 2: Implementing Position

```rust
impl Position {
    /// Creates a new position
    fn new(symbol: String, quantity: f64, entry_price: f64, side: Side) -> Self {
        Position {
            symbol,       // Move: String is moved into the struct
            quantity,
            entry_price,
            side,
        }
    }

    /// Calculates current market value of the position
    fn market_value(&self, current_price: f64) -> f64 {
        self.quantity * current_price
    }

    /// Calculates unrealized P&L
    fn unrealized_pnl(&self, current_price: f64) -> f64 {
        let price_diff = current_price - self.entry_price;
        match self.side {
            Side::Long => price_diff * self.quantity,
            Side::Short => -price_diff * self.quantity,
        }
    }

    /// Calculates P&L percentage
    fn pnl_percent(&self, current_price: f64) -> f64 {
        let pnl = self.unrealized_pnl(current_price);
        let cost = self.entry_price * self.quantity;
        if cost == 0.0 { 0.0 } else { (pnl / cost) * 100.0 }
    }

    /// Returns a reference to the symbol (borrowing)
    fn symbol(&self) -> &str {
        &self.symbol  // Return a reference without transferring ownership
    }

    /// Modifies quantity (mutable borrow)
    fn adjust_quantity(&mut self, delta: f64) {
        self.quantity += delta;
    }
}
```

**Ownership Concepts:**
- `&self` — immutable borrow, allows reading data
- `&mut self` — mutable borrow, allows modifying data
- `&str` return — we borrow the string instead of cloning

## Step 3: Implementing Portfolio

```rust
impl Portfolio {
    /// Creates a new portfolio
    fn new(name: String, initial_balance: f64) -> Self {
        Portfolio {
            name,
            positions: Vec::new(),  // Create an empty vector
            balance: initial_balance,
        }
    }

    /// Opens a new position (moves Position into portfolio)
    fn open_position(&mut self, position: Position) {
        let cost = position.entry_price * position.quantity;
        if cost <= self.balance {
            self.balance -= cost;
            self.positions.push(position);  // Move: position is moved into vector
            println!("Position opened!");
        } else {
            println!("Insufficient funds!");
            // position will be dropped here since it wasn't moved
        }
    }

    /// Closes a position by index and returns it (transfer ownership)
    fn close_position(&mut self, index: usize, current_price: f64) -> Option<Position> {
        if index < self.positions.len() {
            let position = self.positions.remove(index);  // Extract and transfer ownership
            let value = position.market_value(current_price);
            self.balance += value;
            Some(position)  // Transfer ownership to caller
        } else {
            None
        }
    }

    /// Returns a reference to a position (borrowing)
    fn get_position(&self, index: usize) -> Option<&Position> {
        self.positions.get(index)  // Return a reference
    }

    /// Returns a mutable reference to a position
    fn get_position_mut(&mut self, index: usize) -> Option<&mut Position> {
        self.positions.get_mut(index)
    }

    /// Iterates over all positions (borrowing)
    fn iter_positions(&self) -> impl Iterator<Item = &Position> {
        self.positions.iter()
    }

    /// Calculates total portfolio P&L
    fn total_unrealized_pnl(&self, prices: &[(String, f64)]) -> f64 {
        self.positions.iter().map(|pos| {
            let current_price = prices
                .iter()
                .find(|(s, _)| s == &pos.symbol)
                .map(|(_, p)| *p)
                .unwrap_or(pos.entry_price);
            pos.unrealized_pnl(current_price)
        }).sum()
    }

    /// Total portfolio value
    fn total_value(&self, prices: &[(String, f64)]) -> f64 {
        let positions_value: f64 = self.positions.iter().map(|pos| {
            let current_price = prices
                .iter()
                .find(|(s, _)| s == &pos.symbol)
                .map(|(_, p)| *p)
                .unwrap_or(pos.entry_price);
            pos.market_value(current_price)
        }).sum();

        self.balance + positions_value
    }
}
```

## Step 4: Display Information

```rust
/// Formats a position report (takes a reference)
fn format_position_report(position: &Position, current_price: f64) -> String {
    let pnl = position.unrealized_pnl(current_price);
    let pnl_pct = position.pnl_percent(current_price);
    let side_str = match position.side {
        Side::Long => "LONG",
        Side::Short => "SHORT",
    };

    format!(
        "{} {} | Qty: {:.4} | Entry: ${:.2} | Current: ${:.2} | P&L: ${:.2} ({:.2}%)",
        position.symbol(),
        side_str,
        position.quantity,
        position.entry_price,
        current_price,
        pnl,
        pnl_pct
    )
}

/// Prints portfolio report
fn print_portfolio_report(portfolio: &Portfolio, prices: &[(String, f64)]) {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  PORTFOLIO: {:<53} ║", portfolio.name);
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Available Balance: ${:<44.2} ║", portfolio.balance);
    println!("╠══════════════════════════════════════════════════════════════════╣");

    if portfolio.positions.is_empty() {
        println!("║  No open positions                                               ║");
    } else {
        for position in portfolio.iter_positions() {
            let current_price = prices
                .iter()
                .find(|(s, _)| s == position.symbol())
                .map(|(_, p)| *p)
                .unwrap_or(position.entry_price);

            let report = format_position_report(position, current_price);
            println!("║  {}  ║", report);
        }
    }

    println!("╠══════════════════════════════════════════════════════════════════╣");
    let total_pnl = portfolio.total_unrealized_pnl(prices);
    let total_value = portfolio.total_value(prices);
    println!("║  Total P&L: ${:<53.2} ║", total_pnl);
    println!("║  Total Value: ${:<51.2} ║", total_value);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}
```

## Step 5: Main Function

```rust
fn main() {
    println!("=== Trading Position Manager ===\n");

    // Create portfolio
    let mut portfolio = Portfolio::new(
        String::from("Main Portfolio"),
        100_000.0
    );

    // Current prices (in a real system — from API)
    let mut prices: Vec<(String, f64)> = vec![
        (String::from("BTC"), 43500.0),
        (String::from("ETH"), 2650.0),
        (String::from("SOL"), 98.0),
    ];

    // Open positions
    println!("--- Opening Positions ---");

    let btc_position = Position::new(
        String::from("BTC"),
        0.5,
        42000.0,
        Side::Long
    );
    portfolio.open_position(btc_position);  // Move: btc_position is moved
    // btc_position is no longer accessible!

    let eth_position = Position::new(
        String::from("ETH"),
        5.0,
        2500.0,
        Side::Long
    );
    portfolio.open_position(eth_position);

    let sol_position = Position::new(
        String::from("SOL"),
        100.0,
        95.0,
        Side::Short
    );
    portfolio.open_position(sol_position);

    // Portfolio report
    print_portfolio_report(&portfolio, &prices);

    // Modify position (mutable borrow)
    println!("\n--- Increasing ETH Position ---");
    if let Some(eth_pos) = portfolio.get_position_mut(1) {
        eth_pos.adjust_quantity(2.5);  // Add to position
        println!("ETH position increased to {} units", eth_pos.quantity);
    }

    // Update prices
    println!("\n--- Market Price Change ---");
    prices = vec![
        (String::from("BTC"), 45000.0),
        (String::from("ETH"), 2800.0),
        (String::from("SOL"), 92.0),
    ];

    print_portfolio_report(&portfolio, &prices);

    // Close position
    println!("\n--- Closing BTC Position ---");
    if let Some(closed_position) = portfolio.close_position(0, 45000.0) {
        println!(
            "Position {} closed. Realized profit: ${:.2}",
            closed_position.symbol(),
            closed_position.unrealized_pnl(45000.0)
        );
        // closed_position will be dropped at the end of this block
    }

    // Final report
    println!("\n--- Final Report ---");
    print_portfolio_report(&portfolio, &prices);

    println!("\n=== Program Complete ===");
}
```

## Key Ownership Concepts in the Project

### 1. Ownership

```rust
let btc_position = Position::new(...);
portfolio.open_position(btc_position);  // Move
// btc_position is no longer accessible — ownership transferred to portfolio
```

### 2. Borrowing

```rust
// Immutable borrow — can read
fn print_portfolio_report(portfolio: &Portfolio, prices: &[(String, f64)])

// Mutable borrow — can modify
fn adjust_quantity(&mut self, delta: f64)
```

### 3. Lifetimes

```rust
// Reference to string inside Position
fn symbol(&self) -> &str {
    &self.symbol  // Reference lives as long as Position lives
}
```

### 4. Move Semantics

```rust
fn close_position(&mut self, index: usize, price: f64) -> Option<Position> {
    let position = self.positions.remove(index);  // Move from vector
    Some(position)  // Move to caller
}
```

## Exercises

### Exercise 1: Stop-Loss

Add a `stop_loss: Option<f64>` field to `Position` and a method to check if stop is triggered.

```rust
impl Position {
    fn is_stopped_out(&self, current_price: f64) -> bool {
        // Implement the logic
        todo!()
    }
}
```

### Exercise 2: Trade History

Create a `TradeHistory` structure that stores closed positions:

```rust
struct TradeHistory {
    trades: Vec<ClosedTrade>,
}

struct ClosedTrade {
    position: Position,  // Owns the position
    close_price: f64,
    close_time: String,
}
```

### Exercise 3: Find Position

Implement a method to find a position by symbol:

```rust
impl Portfolio {
    fn find_by_symbol(&self, symbol: &str) -> Option<&Position> {
        // Return a reference to the position without transferring ownership
        todo!()
    }
}
```

### Exercise 4: Risk Management

Add a maximum risk check when opening a position:

```rust
impl Portfolio {
    fn open_position_with_risk_check(
        &mut self,
        position: Position,
        max_risk_percent: f64
    ) -> Result<(), String> {
        // Check if the position exceeds allowed risk
        todo!()
    }
}
```

## Homework

1. **Extend Portfolio:**
   - Add a `close_all_positions()` method that closes all positions
   - Implement `clone_positions()` that returns a clone of the positions vector

2. **Create an Order System:**
   - `Order` struct with type (Market, Limit)
   - `place_order()` method in Portfolio
   - Order execution logic

3. **Implement Statistics:**
   - Total number of trades
   - Average P&L
   - Win rate (percentage of profitable trades)

4. **Add Serialization:**
   - Save portfolio to a string (format of your choice)
   - Load portfolio from a string

## Complete Project Code

Combine all parts in `src/main.rs` and run:

```bash
cargo new trading_position_manager
cd trading_position_manager
# Copy the code to src/main.rs
cargo run
```

## What We Learned

| Concept | Application in Project |
|---------|----------------------|
| Ownership | Position belongs to Portfolio |
| Move | Transfer position when opening/closing |
| Borrow | Get references to positions for reading |
| Mutable Borrow | Modify quantity in a position |
| Lifetimes | Reference to symbol lives as long as Position |
| Vec ownership | Vector owns all its elements |

## Navigation

[← Previous day](../058-ownership-review/en.md) | [Next day →](../060-month2-summary/en.md)
