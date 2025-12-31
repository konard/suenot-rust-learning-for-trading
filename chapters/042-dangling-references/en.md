# Day 42: Dangling References

## Trading Analogy

Imagine this situation: you saved an order ID to check its status later. But while you were busy with other tasks, the system cancelled and **deleted** that order. Now your ID points to a non-existent order — this is a **dangling reference**.

In C/C++, this leads to unpredictable behavior: reading garbage from memory or crashing the program. Rust **will not allow** you to create a dangling reference — the compiler stops you before the program even runs.

## What is a Dangling Reference?

A dangling reference is a reference that points to memory that has already been freed. In Rust, this is impossible thanks to the ownership system.

```rust
// THIS WON'T COMPILE!
fn main() {
    let reference_to_nothing = dangle();
}

fn dangle() -> &String {  // Error! Returning reference to local variable
    let order_id = String::from("ORD-12345");
    &order_id  // order_id will be dropped after function exits!
}
```

The compiler will produce an error:
```
error[E0106]: missing lifetime specifier
  --> src/main.rs:6:16
   |
6  | fn dangle() -> &String {
   |                ^ expected named lifetime parameter
```

## Why This Matters in Trading

In trading systems, data constantly appears and disappears:
- Orders get executed and deleted
- Tickers get delisted from exchanges
- Historical data gets cleaned up
- Exchange connections get closed

If your code holds a reference to deleted data — disaster is inevitable.

## Examples of Dangerous Situations

### Example 1: Returning Reference to Local Variable

```rust
// THIS WON'T COMPILE!
fn get_best_ticker() -> &str {
    let ticker = String::from("BTC/USDT");
    &ticker  // ticker dies here, reference becomes dangling
}
```

### Solution: Return Ownership

```rust
fn get_best_ticker() -> String {
    let ticker = String::from("BTC/USDT");
    ticker  // Transfer ownership to the caller
}

fn main() {
    let ticker = get_best_ticker();
    println!("Best ticker: {}", ticker);
}
```

### Example 2: Reference to Deleted Order

```rust
// THIS WON'T COMPILE!
fn main() {
    let order_ref;

    {
        let order = String::from("ORD-BTC-001");
        order_ref = &order;  // order_ref references order
    }  // order is dropped here!

    println!("Order: {}", order_ref);  // ERROR: order no longer exists
}
```

The compiler will report:
```
error[E0597]: `order` does not live long enough
```

### Solution: Proper Lifetimes

```rust
fn main() {
    let order = String::from("ORD-BTC-001");
    let order_ref = &order;  // Reference lives as long as order lives

    println!("Order: {}", order_ref);  // OK!
}
```

## Safe Patterns for Trading

### Pattern 1: Return Ownership Instead of Reference

```rust
struct Order {
    id: String,
    symbol: String,
    price: f64,
    quantity: f64,
}

fn create_market_order(symbol: &str, quantity: f64, current_price: f64) -> Order {
    Order {
        id: format!("ORD-{}", chrono_placeholder()),
        symbol: symbol.to_string(),
        price: current_price,
        quantity,
    }
}

fn chrono_placeholder() -> u64 {
    1234567890  // Placeholder for example
}

fn main() {
    let order = create_market_order("BTC/USDT", 0.5, 42000.0);
    println!("Created order: {} for {} {}", order.id, order.quantity, order.symbol);
}
```

### Pattern 2: Clone When Necessary

```rust
fn main() {
    let tickers = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT"];

    // Get a copy, not a reference
    let best_ticker = find_best_ticker(&tickers);

    // tickers can be modified, best_ticker remains valid
    println!("Best: {}", best_ticker);
}

fn find_best_ticker(tickers: &[&str]) -> String {
    // Return String, not &str
    tickers.first().unwrap_or(&"UNKNOWN").to_string()
}
```

### Pattern 3: Option for Missing Data

```rust
struct OrderBook {
    orders: Vec<Order>,
}

struct Order {
    id: String,
    price: f64,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook { orders: Vec::new() }
    }

    fn add_order(&mut self, id: String, price: f64) {
        self.orders.push(Order { id, price });
    }

    // Return Option, not bare reference
    fn find_order(&self, id: &str) -> Option<&Order> {
        self.orders.iter().find(|o| o.id == id)
    }

    // Safe deletion with check
    fn cancel_order(&mut self, id: &str) -> Option<Order> {
        if let Some(pos) = self.orders.iter().position(|o| o.id == id) {
            Some(self.orders.remove(pos))
        } else {
            None
        }
    }
}

fn main() {
    let mut book = OrderBook::new();
    book.add_order("ORD-001".to_string(), 42000.0);
    book.add_order("ORD-002".to_string(), 42100.0);

    // Safe lookup
    match book.find_order("ORD-001") {
        Some(order) => println!("Found: {} at {}", order.id, order.price),
        None => println!("Order not found"),
    }

    // Safe deletion
    if let Some(cancelled) = book.cancel_order("ORD-001") {
        println!("Cancelled: {}", cancelled.id);
    }

    // Search for deleted order again
    match book.find_order("ORD-001") {
        Some(_) => println!("Still exists"),
        None => println!("Order no longer exists"),  // This will print
    }
}
```

## Common Mistakes and How to Avoid Them

### Mistake 1: Trying to Return Reference to Computed Value

```rust
// WON'T COMPILE!
fn calculate_spread(bid: f64, ask: f64) -> &f64 {
    let spread = ask - bid;
    &spread  // spread will be dropped!
}
```

**Solution:** Return the value, not a reference:

```rust
fn calculate_spread(bid: f64, ask: f64) -> f64 {
    ask - bid
}

fn main() {
    let spread = calculate_spread(42000.0, 42010.0);
    println!("Spread: ${:.2}", spread);
}
```

### Mistake 2: Holding Reference While Modifying Collection

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 42050.0];

    // This is safe:
    let first = prices[0];  // Copy of value
    prices.push(42200.0);   // Modify vector
    println!("First: {}", first);  // OK!

    // This would be dangerous (Rust won't allow):
    // let first_ref = &prices[0];
    // prices.push(42200.0);  // ERROR: can't modify with active reference
    // println!("First: {}", first_ref);
}
```

### Mistake 3: Returning Reference to Temporary Value

```rust
// WON'T COMPILE!
fn get_formatted_price(price: f64) -> &str {
    let formatted = format!("${:.2}", price);
    &formatted  // Temporary string will be dropped!
}
```

**Solution:**

```rust
fn get_formatted_price(price: f64) -> String {
    format!("${:.2}", price)
}

fn main() {
    let price_str = get_formatted_price(42000.0);
    println!("Price: {}", price_str);
}
```

## Practical Example: Safe Portfolio Manager

```rust
#[derive(Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

struct Portfolio {
    positions: Vec<Position>,
    balance: f64,
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            positions: Vec::new(),
            balance: initial_balance,
        }
    }

    // Returns copy of position (safe)
    fn get_position(&self, symbol: &str) -> Option<Position> {
        self.positions
            .iter()
            .find(|p| p.symbol == symbol)
            .cloned()  // Clone to avoid lifetime dependency
    }

    // Open position
    fn open_position(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<(), String> {
        let cost = quantity * price;
        if cost > self.balance {
            return Err("Insufficient balance".to_string());
        }

        self.balance -= cost;
        self.positions.push(Position {
            symbol: symbol.to_string(),
            quantity,
            entry_price: price,
        });

        Ok(())
    }

    // Close position — returns PnL
    fn close_position(&mut self, symbol: &str, exit_price: f64) -> Result<f64, String> {
        let pos_index = self.positions
            .iter()
            .position(|p| p.symbol == symbol)
            .ok_or_else(|| format!("Position {} not found", symbol))?;

        let position = self.positions.remove(pos_index);
        let pnl = (exit_price - position.entry_price) * position.quantity;
        let proceeds = position.quantity * exit_price;
        self.balance += proceeds;

        Ok(pnl)
    }

    // Total PnL (returns value, not reference)
    fn calculate_total_pnl(&self, current_prices: &[(String, f64)]) -> f64 {
        self.positions.iter().map(|pos| {
            let current = current_prices
                .iter()
                .find(|(s, _)| s == &pos.symbol)
                .map(|(_, p)| *p)
                .unwrap_or(pos.entry_price);
            (current - pos.entry_price) * pos.quantity
        }).sum()
    }
}

fn main() {
    let mut portfolio = Portfolio::new(100000.0);

    // Open positions
    portfolio.open_position("BTC", 0.5, 42000.0).unwrap();
    portfolio.open_position("ETH", 5.0, 2500.0).unwrap();

    // Get copy of position (safe)
    if let Some(btc_pos) = portfolio.get_position("BTC") {
        println!("BTC position: {} @ ${}", btc_pos.quantity, btc_pos.entry_price);
    }

    // Calculate PnL with current prices
    let current_prices = vec![
        ("BTC".to_string(), 43000.0),
        ("ETH".to_string(), 2600.0),
    ];

    let total_pnl = portfolio.calculate_total_pnl(&current_prices);
    println!("Total unrealized PnL: ${:.2}", total_pnl);

    // Close position
    match portfolio.close_position("BTC", 43000.0) {
        Ok(pnl) => println!("BTC closed with PnL: ${:.2}", pnl),
        Err(e) => println!("Error: {}", e),
    }

    // Try to get closed position
    match portfolio.get_position("BTC") {
        Some(_) => println!("BTC still open"),
        None => println!("BTC position closed"),  // This will print
    }

    println!("Final balance: ${:.2}", portfolio.balance);
}
```

## Comparison of Approaches

| Approach | Safety | Performance | When to Use |
|----------|--------|-------------|-------------|
| Return ownership | High | Medium | Data needed long-term |
| Cloning | High | Lower | Small data |
| References with lifetimes | High | High | Data lives long enough |
| Option/Result | High | High | Data may not exist |

## What We Learned

1. **Dangling reference** — reference to freed memory
2. Rust **prevents** creation of dangling references at compile time
3. Return **ownership** instead of reference when data is created inside function
4. Use **Option** for handling missing data
5. **Clone** data when you need an independent copy

## Homework

1. Write a function `create_trade_report(trades: &[Trade]) -> String` that creates a report string and safely returns it

2. Implement an `OrderManager` struct with methods:
   - `submit_order(&mut self, order: Order) -> String` — returns order ID
   - `get_order(&self, id: &str) -> Option<Order>` — returns copy of order
   - `cancel_order(&mut self, id: &str) -> Result<Order, String>` — cancels and returns order

3. Create a function `find_best_price` that takes `&[f64]` and returns the best price safely (think about what to return: `f64`, `Option<f64>`, or `Result<f64, String>`)

4. Fix the following code (it won't compile):
```rust
fn get_ticker_info(symbol: &str) -> &str {
    let info = format!("{} - Active", symbol);
    &info
}
```

## Navigation

[← Previous day](../041-no-mixing-references/en.md) | [Next day →](../043-lifetimes-order-duration/en.md)
