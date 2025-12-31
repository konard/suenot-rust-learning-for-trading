# Day 40: One Mutable Reference Rule

## Trading Analogy

Imagine a trading terminal where an order is open for editing. If **two traders simultaneously** start changing parameters of the same order — one sets the price to 42000, another to 43000 — the result will be unpredictable. Which price will end up in the order?

In trading systems, this is called a **race condition**, and it's one of the most dangerous errors. Rust **prevents these situations at compile time**: at any given moment, only **one** can edit the data.

## The Rust Rule

> **At any given time, you can have either one mutable reference `&mut T`, or any number of immutable references `&T`, but not both at the same time.**

This rule prevents:
- **Data races** — simultaneous reading and writing
- **Undefined behavior** — unpredictable program behavior
- **Hard-to-find bugs** — errors that appear "sometimes"

## Basic Example: Modifying Order Price

```rust
fn main() {
    let mut order_price = 42000.0;

    // Creating one mutable reference — ok
    let price_ref = &mut order_price;
    *price_ref = 42500.0;

    println!("New price: {}", order_price);
}
```

## Error: Two Mutable References

```rust
fn main() {
    let mut order_price = 42000.0;

    let ref1 = &mut order_price;
    let ref2 = &mut order_price;  // COMPILATION ERROR!

    *ref1 = 42500.0;
    *ref2 = 43000.0;
}
```

The compiler will produce an error:
```
error[E0499]: cannot borrow `order_price` as mutable more than once at a time
```

## Why This Matters for Trading

### Scenario 1: Portfolio Management

```rust
struct Portfolio {
    balance: f64,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

fn main() {
    let mut portfolio = Portfolio {
        balance: 100_000.0,
        positions: vec![],
    };

    // One portfolio "editor" — safe
    let editor = &mut portfolio;
    editor.balance -= 21_000.0;
    editor.positions.push(Position {
        symbol: String::from("BTC"),
        quantity: 0.5,
        entry_price: 42000.0,
    });

    println!("Balance: ${:.2}", portfolio.balance);
    println!("Positions: {}", portfolio.positions.len());
}
```

### Scenario 2: Order Modification

```rust
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    status: String,
}

fn update_order_price(order: &mut Order, new_price: f64) {
    println!("Order #{}: price {} -> {}", order.id, order.price, new_price);
    order.price = new_price;
}

fn update_order_quantity(order: &mut Order, new_quantity: f64) {
    println!("Order #{}: quantity {} -> {}", order.id, order.quantity, new_quantity);
    order.quantity = new_quantity;
}

fn main() {
    let mut order = Order {
        id: 12345,
        symbol: String::from("ETH/USDT"),
        price: 2500.0,
        quantity: 10.0,
        status: String::from("PENDING"),
    };

    // Sequential editing — safe
    update_order_price(&mut order, 2550.0);
    update_order_quantity(&mut order, 15.0);

    println!("Final order: {} @ ${}", order.quantity, order.price);
}
```

## Scope of Mutable References

A reference "lives" until its last use, not until the end of the block:

```rust
fn main() {
    let mut price = 42000.0;

    let ref1 = &mut price;
    *ref1 = 42500.0;
    println!("After ref1: {}", ref1);  // Last use of ref1

    // ref1 is no longer used — can create a new mutable reference
    let ref2 = &mut price;
    *ref2 = 43000.0;
    println!("After ref2: {}", ref2);
}
```

This is called **Non-Lexical Lifetimes (NLL)** — Rust understands when a reference is actually being used.

## Practical Example: Price Analysis

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42050.0];

    // Analyze prices (read-only)
    let avg = calculate_average(&prices);
    let max = find_max(&prices);
    println!("Average: {:.2}, Maximum: {:.2}", avg, max);

    // Now modify — add a new price
    add_price(&mut prices, 42300.0);

    // Can read again
    let new_avg = calculate_average(&prices);
    println!("New average: {:.2}", new_avg);
}

fn calculate_average(prices: &[f64]) -> f64 {
    prices.iter().sum::<f64>() / prices.len() as f64
}

fn find_max(prices: &[f64]) -> f64 {
    prices.iter().cloned().fold(f64::MIN, f64::max)
}

fn add_price(prices: &mut Vec<f64>, price: f64) {
    prices.push(price);
    println!("Added price: {}", price);
}
```

## Risk Management: Safe Limit Updates

```rust
struct RiskLimits {
    max_position_size: f64,
    max_daily_loss: f64,
    max_leverage: f64,
}

fn main() {
    let mut limits = RiskLimits {
        max_position_size: 100_000.0,
        max_daily_loss: 5_000.0,
        max_leverage: 10.0,
    };

    // One risk manager updates limits
    update_risk_limits(&mut limits, 2.0);

    println!("New position size: ${:.2}", limits.max_position_size);
    println!("New daily loss limit: ${:.2}", limits.max_daily_loss);
}

fn update_risk_limits(limits: &mut RiskLimits, multiplier: f64) {
    limits.max_position_size *= multiplier;
    limits.max_daily_loss *= multiplier;
    println!("Limits updated with multiplier {}", multiplier);
}
```

## Order Management: Execution Queue

```rust
struct OrderQueue {
    orders: Vec<Order>,
    total_volume: f64,
}

struct Order {
    id: u64,
    price: f64,
    quantity: f64,
}

impl OrderQueue {
    fn new() -> Self {
        OrderQueue {
            orders: vec![],
            total_volume: 0.0,
        }
    }

    fn add_order(&mut self, order: Order) {
        self.total_volume += order.price * order.quantity;
        self.orders.push(order);
    }

    fn remove_order(&mut self, id: u64) -> Option<Order> {
        if let Some(pos) = self.orders.iter().position(|o| o.id == id) {
            let order = self.orders.remove(pos);
            self.total_volume -= order.price * order.quantity;
            Some(order)
        } else {
            None
        }
    }

    fn get_stats(&self) -> (usize, f64) {
        (self.orders.len(), self.total_volume)
    }
}

fn main() {
    let mut queue = OrderQueue::new();

    // Add orders (mutable access)
    queue.add_order(Order { id: 1, price: 42000.0, quantity: 0.5 });
    queue.add_order(Order { id: 2, price: 42100.0, quantity: 1.0 });
    queue.add_order(Order { id: 3, price: 41900.0, quantity: 0.25 });

    // Read statistics (immutable access)
    let (count, volume) = queue.get_stats();
    println!("Orders in queue: {}, Volume: ${:.2}", count, volume);

    // Remove order (mutable access)
    if let Some(removed) = queue.remove_order(2) {
        println!("Removed order #{} at ${:.2}", removed.id, removed.price);
    }

    let (count, volume) = queue.get_stats();
    println!("After removal: {} orders, ${:.2}", count, volume);
}
```

## Pattern: Separating Read and Write

```rust
struct TradingAccount {
    balance: f64,
    equity: f64,
    margin_used: f64,
}

impl TradingAccount {
    // Read-only methods — &self
    fn available_margin(&self) -> f64 {
        self.equity - self.margin_used
    }

    fn margin_level(&self) -> f64 {
        if self.margin_used > 0.0 {
            (self.equity / self.margin_used) * 100.0
        } else {
            f64::INFINITY
        }
    }

    // Modifying methods — &mut self
    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
        self.equity += amount;
        println!("Deposit: +${:.2}", amount);
    }

    fn use_margin(&mut self, amount: f64) -> bool {
        if amount <= self.available_margin() {
            self.margin_used += amount;
            true
        } else {
            false
        }
    }
}

fn main() {
    let mut account = TradingAccount {
        balance: 10_000.0,
        equity: 10_500.0,
        margin_used: 2_000.0,
    };

    // Read data
    println!("Available margin: ${:.2}", account.available_margin());
    println!("Margin level: {:.1}%", account.margin_level());

    // Modify
    account.deposit(5_000.0);

    // Read again
    println!("New available margin: ${:.2}", account.available_margin());
}
```

## Trading Strategy: Safe State Updates

```rust
struct TradingStrategy {
    name: String,
    is_active: bool,
    current_position: f64,
    total_pnl: f64,
    trade_count: u32,
}

impl TradingStrategy {
    fn new(name: &str) -> Self {
        TradingStrategy {
            name: String::from(name),
            is_active: false,
            current_position: 0.0,
            total_pnl: 0.0,
            trade_count: 0,
        }
    }

    fn activate(&mut self) {
        self.is_active = true;
        println!("Strategy '{}' activated", self.name);
    }

    fn execute_trade(&mut self, quantity: f64, pnl: f64) {
        if !self.is_active {
            println!("Strategy is not active!");
            return;
        }
        self.current_position += quantity;
        self.total_pnl += pnl;
        self.trade_count += 1;
        println!("Trade #{}: qty {}, PnL ${:.2}",
                 self.trade_count, quantity, pnl);
    }

    fn get_summary(&self) -> String {
        format!("{}: {} trades, PnL ${:.2}",
                self.name, self.trade_count, self.total_pnl)
    }
}

fn main() {
    let mut strategy = TradingStrategy::new("SMA Crossover");

    strategy.activate();
    strategy.execute_trade(0.5, 150.0);
    strategy.execute_trade(-0.5, 0.0);
    strategy.execute_trade(1.0, -50.0);

    println!("\n{}", strategy.get_summary());
}
```

## What We Learned

| Rule | Description | Analogy |
|------|-------------|---------|
| One `&mut` | Only one mutable reference at a time | One order editor |
| Many `&` | Any number of immutable references | Many terminal viewers |
| No mixing | `&mut` and `&` cannot coexist | Can't view while editing |
| NLL | Reference ends at last usage | Editor releases access |

## Homework

1. **Portfolio with validation**: Create a `Portfolio` struct with methods `add_position(&mut self, ...)` and `get_total_value(&self) -> f64`. Ensure the compiler correctly tracks references.

2. **Trade journal**: Write a function `record_trade(journal: &mut TradeJournal, trade: Trade)` and a function `analyze_journal(journal: &TradeJournal) -> Stats`. Demonstrate sequential use of `&mut` and `&`.

3. **Risk calculator**: Create a `RiskCalculator` struct that stores risk settings. Write a method `update_settings(&mut self, ...)` and a method `calculate_position_size(&self, ...) -> f64`. Show how the one mutable reference rule protects against simultaneous settings changes.

4. **Multi-user access** (advanced): Try to create a situation where you need two mutable references. What error does the compiler produce? How can you rewrite the code so it compiles?

## Navigation

[← Previous day](../039-mutable-references/en.md) | [Next day →](../041-no-mixing-references/en.md)
