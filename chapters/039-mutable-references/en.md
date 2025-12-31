# Day 39: Mutable References — Editing Someone's Order

## Trading Analogy

Imagine you work at a trading firm. Your colleague has an order that needs to be modified — for example, changing the price or volume. There are two approaches:

1. **Copy the order** — make a copy, modify it, and return it (inefficient)
2. **Get edit access** — your colleague gives you temporary permission to change their order directly

In Rust, the second approach is implemented through **mutable references** (`&mut`). It's like getting edit access to someone else's document — you can modify it, but only while they've granted permission.

## Regular References vs Mutable References

```rust
fn main() {
    let mut price = 42000.0;

    // Regular reference — read only
    let price_ref = &price;
    println!("Current price: ${}", price_ref);
    // price_ref = 43000.0;  // ERROR! Cannot modify through &

    // Mutable reference — can modify
    let price_mut = &mut price;
    *price_mut = 43000.0;  // OK! Modifying value through &mut
    println!("New price: ${}", price_mut);
}
```

## Mutable Reference Syntax

```rust
fn main() {
    let mut balance = 10000.0;

    // Create a mutable reference
    let balance_ref = &mut balance;

    // Use * to access the value (dereferencing)
    *balance_ref += 500.0;
    *balance_ref -= 100.0;

    println!("Balance: ${}", balance_ref);
}
```

- `&mut` — creates a mutable reference
- `*` — dereferencing (accessing the value)
- The original variable must be `mut`

## Mutable References in Functions

```rust
fn main() {
    let mut order_price = 42000.0;
    let mut order_quantity = 0.5;

    println!("Before: price=${}, quantity={}", order_price, order_quantity);

    // Pass mutable references to function
    modify_order(&mut order_price, &mut order_quantity);

    println!("After: price=${}, quantity={}", order_price, order_quantity);
}

fn modify_order(price: &mut f64, quantity: &mut f64) {
    *price = 42500.0;     // Change price
    *quantity = 0.75;     // Change quantity
}
```

## Practical Example: Updating a Position

```rust
fn main() {
    let mut position_size = 1.0;  // Current position size
    let mut average_price = 42000.0;  // Average price

    println!("Position: {} BTC @ ${:.2}", position_size, average_price);

    // Add to position
    add_to_position(&mut position_size, &mut average_price, 0.5, 43000.0);
    println!("After adding: {} BTC @ ${:.2}", position_size, average_price);

    // Partial sell
    reduce_position(&mut position_size, 0.3);
    println!("After selling: {} BTC @ ${:.2}", position_size, average_price);
}

fn add_to_position(
    size: &mut f64,
    avg_price: &mut f64,
    add_size: f64,
    add_price: f64
) {
    // Calculate new average price
    let total_value = (*size * *avg_price) + (add_size * add_price);
    *size += add_size;
    *avg_price = total_value / *size;
}

fn reduce_position(size: &mut f64, sell_size: f64) {
    *size -= sell_size;
}
```

## Mutable References to Structs

```rust
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    is_active: bool,
}

fn main() {
    let mut order = Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 0.5,
        is_active: true,
    };

    println!("Order: {} {} @ ${}", order.symbol, order.quantity, order.price);

    // Pass mutable reference to struct
    update_order_price(&mut order, 42500.0);
    println!("After update: {} @ ${}", order.symbol, order.price);

    cancel_order(&mut order);
    println!("Status: active = {}", order.is_active);
}

fn update_order_price(order: &mut Order, new_price: f64) {
    order.price = new_price;  // Automatic dereferencing for fields
}

fn cancel_order(order: &mut Order) {
    order.is_active = false;
}
```

## Modifying Vec Through Reference

```rust
fn main() {
    let mut portfolio: Vec<(&str, f64)> = vec![
        ("BTC", 0.5),
        ("ETH", 2.0),
        ("SOL", 10.0),
    ];

    println!("Portfolio before:");
    print_portfolio(&portfolio);

    // Add new asset
    add_asset(&mut portfolio, "AVAX", 5.0);

    // Update BTC quantity
    update_asset_quantity(&mut portfolio, "BTC", 0.75);

    println!("\nPortfolio after:");
    print_portfolio(&portfolio);
}

fn add_asset(portfolio: &mut Vec<(&str, f64)>, symbol: &str, quantity: f64) {
    portfolio.push((symbol, quantity));
}

fn update_asset_quantity(portfolio: &mut Vec<(&str, f64)>, symbol: &str, new_qty: f64) {
    for asset in portfolio.iter_mut() {
        if asset.0 == symbol {
            asset.1 = new_qty;
            return;
        }
    }
}

fn print_portfolio(portfolio: &Vec<(&str, f64)>) {
    for (symbol, qty) in portfolio {
        println!("  {}: {}", symbol, qty);
    }
}
```

## Practical Example: Risk Management

```rust
struct RiskManager {
    max_position_size: f64,
    current_exposure: f64,
    daily_loss_limit: f64,
    current_daily_loss: f64,
}

fn main() {
    let mut risk = RiskManager {
        max_position_size: 10.0,
        current_exposure: 0.0,
        daily_loss_limit: 500.0,
        current_daily_loss: 0.0,
    };

    println!("=== RISK MANAGEMENT ===\n");

    // Try to open a position
    if try_open_position(&mut risk, 5.0) {
        println!("Position 5.0 opened");
    }

    // Try to open another
    if try_open_position(&mut risk, 7.0) {
        println!("Position 7.0 opened");
    } else {
        println!("Position 7.0 rejected — limit exceeded");
    }

    // Record a loss
    record_loss(&mut risk, 200.0);
    println!("Loss recorded: $200");

    // Check if we can trade
    if can_trade(&risk) {
        println!("Trading allowed");
    }

    // Large loss
    record_loss(&mut risk, 350.0);
    println!("Loss recorded: $350");

    if !can_trade(&risk) {
        println!("STOP! Daily loss limit reached");
    }
}

fn try_open_position(risk: &mut RiskManager, size: f64) -> bool {
    if risk.current_exposure + size <= risk.max_position_size {
        risk.current_exposure += size;
        true
    } else {
        false
    }
}

fn record_loss(risk: &mut RiskManager, amount: f64) {
    risk.current_daily_loss += amount;
}

fn can_trade(risk: &RiskManager) -> bool {
    risk.current_daily_loss < risk.daily_loss_limit
}
```

## Modifying Through Methods

```rust
struct TradingAccount {
    balance: f64,
    positions: Vec<String>,
}

impl TradingAccount {
    fn new(initial_balance: f64) -> Self {
        TradingAccount {
            balance: initial_balance,
            positions: Vec::new(),
        }
    }

    // &self — immutable access
    fn get_balance(&self) -> f64 {
        self.balance
    }

    // &mut self — mutable access
    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
    }

    fn withdraw(&mut self, amount: f64) -> bool {
        if amount <= self.balance {
            self.balance -= amount;
            true
        } else {
            false
        }
    }

    fn open_position(&mut self, symbol: &str) {
        self.positions.push(String::from(symbol));
    }

    fn close_position(&mut self, symbol: &str) {
        self.positions.retain(|s| s != symbol);
    }
}

fn main() {
    let mut account = TradingAccount::new(10000.0);

    println!("Initial balance: ${}", account.get_balance());

    account.deposit(5000.0);
    println!("After deposit: ${}", account.get_balance());

    account.open_position("BTC/USDT");
    account.open_position("ETH/USDT");
    println!("Positions: {:?}", account.positions);

    account.close_position("BTC/USDT");
    println!("After closing BTC: {:?}", account.positions);

    if account.withdraw(20000.0) {
        println!("Withdrawal successful");
    } else {
        println!("Insufficient funds");
    }
}
```

## Mutable Reference to Part of Data

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42050.0];

    println!("Prices before: {:?}", prices);

    // Get mutable reference to last price
    if let Some(last) = prices.last_mut() {
        *last = 42300.0;
    }

    // Get mutable reference to first price
    if let Some(first) = prices.first_mut() {
        *first = 41800.0;
    }

    println!("Prices after: {:?}", prices);

    // Modify all prices
    for price in prices.iter_mut() {
        *price *= 1.01;  // Increase by 1%
    }

    println!("After 1% increase: {:?}", prices);
}
```

## Swap — Exchanging Values

```rust
fn main() {
    let mut bid = 41950.0;
    let mut ask = 42050.0;

    println!("Before swap: bid={}, ask={}", bid, ask);

    // Swap using mutable references
    swap_prices(&mut bid, &mut ask);

    println!("After swap: bid={}, ask={}", bid, ask);

    // There's a built-in function std::mem::swap
    std::mem::swap(&mut bid, &mut ask);
    println!("After std::mem::swap: bid={}, ask={}", bid, ask);
}

fn swap_prices(a: &mut f64, b: &mut f64) {
    let temp = *a;
    *a = *b;
    *b = temp;
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `&mut T` | Mutable reference |
| `*ref` | Dereferencing to access value |
| `fn f(x: &mut T)` | Function takes mutable reference |
| `&mut self` | Method with mutable access to self |
| `.iter_mut()` | Iterator with mutable references |
| `last_mut()`, `first_mut()` | Mutable access to elements |

## Exercises

1. **Balance Management**
   Write functions for working with balance:
   ```rust
   fn deposit(balance: &mut f64, amount: f64) { ... }
   fn withdraw(balance: &mut f64, amount: f64) -> bool { ... }
   fn apply_fee(balance: &mut f64, fee_percent: f64) { ... }
   ```

2. **Order Update**
   Create a `LimitOrder` struct and functions to modify it:
   ```rust
   fn update_price(order: &mut LimitOrder, new_price: f64) { ... }
   fn update_quantity(order: &mut LimitOrder, new_qty: f64) { ... }
   fn cancel(order: &mut LimitOrder) { ... }
   ```

3. **Portfolio Normalization**
   Write a function that normalizes asset weights in a portfolio to sum to 100%:
   ```rust
   fn normalize_weights(weights: &mut Vec<f64>) { ... }
   ```

4. **Stop-Loss Manager**
   Implement a trailing stop-loss system:
   ```rust
   fn update_trailing_stop(
       stop_price: &mut f64,
       current_price: f64,
       trail_percent: f64
   ) { ... }
   ```

## Homework

1. Create a `Portfolio` struct with methods using `&mut self`:
   - `add_asset()` — add an asset
   - `remove_asset()` — remove an asset
   - `update_quantity()` — change quantity
   - `rebalance()` — rebalance to target weights

2. Implement a function to recalculate average position price when adding:
   ```rust
   fn add_to_position(
       current_qty: &mut f64,
       avg_price: &mut f64,
       add_qty: f64,
       add_price: f64
   )
   ```

3. Write a function that applies commission to a list of trades:
   ```rust
   fn apply_commission(trades: &mut Vec<Trade>, commission_rate: f64)
   ```

4. Create a risk management system with methods:
   - `check_and_update_exposure()` — check and update exposure
   - `record_pnl()` — record trade result
   - `reset_daily_stats()` — reset daily statistics

## Navigation

[← Previous day](../038-borrowing/en.md) | [Next day →](../040-one-mutable-reference-rule/en.md)
