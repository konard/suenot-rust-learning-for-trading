# Day 63: Methods — order.execute()

## Trading Analogy

Imagine a trading order. It contains data (symbol, price, quantity), but it also needs to be able to **perform actions**:
- `order.execute()` — execute the order
- `order.cancel()` — cancel the order
- `order.total_value()` — get the total value

These are **methods** — functions that belong to a specific data type. If a function answers "what to do?", a method answers "what can this object do?".

## What Are Methods

Methods are functions defined inside an `impl` block for a struct:

```rust
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

impl Order {
    // This is a method — a function inside impl
    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }
}

fn main() {
    let order = Order {
        symbol: String::from("BTC"),
        price: 42000.0,
        quantity: 0.5,
        side: String::from("buy"),
    };

    // Call method using dot notation
    let value = order.total_value();
    println!("Order value: ${:.2}", value);
}
```

The `self` keyword is a reference to the struct instance on which the method is called.

## &self, &mut self, and self

Methods have three ways to receive `self`:

```rust
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

impl Position {
    // &self — borrowing (read only)
    fn market_value(&self, current_price: f64) -> f64 {
        self.quantity * current_price
    }

    // &mut self — mutable borrowing
    fn add_to_position(&mut self, additional_qty: f64, price: f64) {
        let total_value = self.quantity * self.entry_price + additional_qty * price;
        self.quantity += additional_qty;
        self.entry_price = total_value / self.quantity;  // Average price
    }

    // self — takes ownership (consumes the object)
    fn close(self) -> f64 {
        println!("Closing position: {} {}", self.quantity, self.symbol);
        self.quantity * self.entry_price
        // After this, position can no longer be used
    }
}

fn main() {
    let mut position = Position {
        symbol: String::from("ETH"),
        quantity: 2.0,
        entry_price: 2500.0,
    };

    // &self — position is not modified
    let value = position.market_value(2600.0);
    println!("Market value: ${:.2}", value);

    // &mut self — position is modified
    position.add_to_position(1.0, 2400.0);
    println!("New avg price: ${:.2}", position.entry_price);

    // self — position is consumed
    let final_value = position.close();
    println!("Final value: ${:.2}", final_value);
    // position can no longer be used!
}
```

## Associated Functions (Constructors)

Functions without `self` are called **associated functions**. The most common one is the `new` constructor:

```rust
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    status: String,
}

impl Order {
    // Associated function (no self) — called via ::
    fn new(symbol: &str, price: f64, quantity: f64, side: &str) -> Order {
        Order {
            symbol: String::from(symbol),
            price,
            quantity,
            side: String::from(side),
            status: String::from("pending"),
        }
    }

    // Alternative constructors
    fn market_buy(symbol: &str, quantity: f64, current_price: f64) -> Order {
        Order::new(symbol, current_price, quantity, "buy")
    }

    fn market_sell(symbol: &str, quantity: f64, current_price: f64) -> Order {
        Order::new(symbol, current_price, quantity, "sell")
    }

    // Method — has &self
    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }
}

fn main() {
    // Call associated function via ::
    let order1 = Order::new("BTC", 42000.0, 0.5, "buy");
    let order2 = Order::market_buy("ETH", 2.0, 2500.0);

    println!("Order 1: {} {} {} @ ${}",
        order1.side, order1.quantity, order1.symbol, order1.price);
    println!("Order 2 value: ${}", order2.total_value());
}
```

## The execute() Method for Orders

Now let's implement a complete order with the `execute()` method:

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    status: String,
}

impl Order {
    fn new(id: u64, symbol: &str, price: f64, quantity: f64, side: &str) -> Order {
        Order {
            id,
            symbol: String::from(symbol),
            price,
            quantity,
            side: String::from(side),
            status: String::from("pending"),
        }
    }

    fn execute(&mut self) -> Result<f64, String> {
        if self.status != "pending" {
            return Err(format!("Order {} is already {}", self.id, self.status));
        }

        if self.quantity <= 0.0 {
            return Err(String::from("Invalid quantity"));
        }

        if self.price <= 0.0 {
            return Err(String::from("Invalid price"));
        }

        // Simulate execution
        self.status = String::from("filled");
        let value = self.total_value();

        println!("Order {} executed: {} {} {} @ ${:.2}",
            self.id, self.side, self.quantity, self.symbol, self.price);

        Ok(value)
    }

    fn cancel(&mut self) -> Result<(), String> {
        if self.status != "pending" {
            return Err(format!("Cannot cancel: order is {}", self.status));
        }

        self.status = String::from("cancelled");
        println!("Order {} cancelled", self.id);
        Ok(())
    }

    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }

    fn is_filled(&self) -> bool {
        self.status == "filled"
    }

    fn is_pending(&self) -> bool {
        self.status == "pending"
    }
}

fn main() {
    let mut order = Order::new(1, "BTC", 42000.0, 0.5, "buy");

    println!("Order status: {}", order.status);
    println!("Total value: ${:.2}", order.total_value());

    // Execute the order
    match order.execute() {
        Ok(value) => println!("Executed for ${:.2}", value),
        Err(e) => println!("Error: {}", e),
    }

    // Attempt to execute again
    match order.execute() {
        Ok(value) => println!("Executed for ${:.2}", value),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Multiple impl Blocks

Methods can be split across multiple `impl` blocks:

```rust
struct Trade {
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    fees: f64,
}

// Core methods
impl Trade {
    fn new(entry: f64, exit: f64, quantity: f64) -> Trade {
        Trade {
            entry_price: entry,
            exit_price: exit,
            quantity,
            fees: 0.0,
        }
    }

    fn gross_pnl(&self) -> f64 {
        (self.exit_price - self.entry_price) * self.quantity
    }
}

// Fee-related methods
impl Trade {
    fn with_fees(mut self, fee_percent: f64) -> Trade {
        let total_value = (self.entry_price + self.exit_price) * self.quantity;
        self.fees = total_value * (fee_percent / 100.0);
        self
    }

    fn net_pnl(&self) -> f64 {
        self.gross_pnl() - self.fees
    }
}

// Analysis methods
impl Trade {
    fn profit_percent(&self) -> f64 {
        if self.entry_price == 0.0 {
            return 0.0;
        }
        ((self.exit_price - self.entry_price) / self.entry_price) * 100.0
    }

    fn is_profitable(&self) -> bool {
        self.net_pnl() > 0.0
    }
}

fn main() {
    let trade = Trade::new(42000.0, 43500.0, 0.5)
        .with_fees(0.1);

    println!("Gross PnL: ${:.2}", trade.gross_pnl());
    println!("Fees: ${:.2}", trade.fees);
    println!("Net PnL: ${:.2}", trade.net_pnl());
    println!("Profit: {:.2}%", trade.profit_percent());
    println!("Profitable: {}", trade.is_profitable());
}
```

## Method Chaining

By returning `self`, you can build chains of method calls:

```rust
struct OrderBuilder {
    symbol: Option<String>,
    price: Option<f64>,
    quantity: Option<f64>,
    side: Option<String>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl OrderBuilder {
    fn new() -> OrderBuilder {
        OrderBuilder {
            symbol: None,
            price: None,
            quantity: None,
            side: None,
            stop_loss: None,
            take_profit: None,
        }
    }

    fn symbol(mut self, symbol: &str) -> OrderBuilder {
        self.symbol = Some(String::from(symbol));
        self
    }

    fn price(mut self, price: f64) -> OrderBuilder {
        self.price = Some(price);
        self
    }

    fn quantity(mut self, quantity: f64) -> OrderBuilder {
        self.quantity = Some(quantity);
        self
    }

    fn buy(mut self) -> OrderBuilder {
        self.side = Some(String::from("buy"));
        self
    }

    fn sell(mut self) -> OrderBuilder {
        self.side = Some(String::from("sell"));
        self
    }

    fn stop_loss(mut self, price: f64) -> OrderBuilder {
        self.stop_loss = Some(price);
        self
    }

    fn take_profit(mut self, price: f64) -> OrderBuilder {
        self.take_profit = Some(price);
        self
    }

    fn build(self) -> Result<String, String> {
        let symbol = self.symbol.ok_or("Symbol is required")?;
        let price = self.price.ok_or("Price is required")?;
        let quantity = self.quantity.ok_or("Quantity is required")?;
        let side = self.side.ok_or("Side is required")?;

        let mut order_str = format!(
            "{} {} {} @ ${:.2}", side, quantity, symbol, price
        );

        if let Some(sl) = self.stop_loss {
            order_str.push_str(&format!(" SL: ${:.2}", sl));
        }

        if let Some(tp) = self.take_profit {
            order_str.push_str(&format!(" TP: ${:.2}", tp));
        }

        Ok(order_str)
    }
}

fn main() {
    // Method chaining
    let order = OrderBuilder::new()
        .symbol("BTC")
        .price(42000.0)
        .quantity(0.5)
        .buy()
        .stop_loss(41000.0)
        .take_profit(45000.0)
        .build();

    match order {
        Ok(o) => println!("Order: {}", o),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Practical Example: Trading Portfolio

```rust
struct Portfolio {
    name: String,
    balance: f64,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

impl Position {
    fn new(symbol: &str, quantity: f64, entry_price: f64) -> Position {
        Position {
            symbol: String::from(symbol),
            quantity,
            entry_price,
        }
    }

    fn market_value(&self, current_price: f64) -> f64 {
        self.quantity * current_price
    }

    fn unrealized_pnl(&self, current_price: f64) -> f64 {
        (current_price - self.entry_price) * self.quantity
    }

    fn pnl_percent(&self, current_price: f64) -> f64 {
        if self.entry_price == 0.0 {
            return 0.0;
        }
        ((current_price - self.entry_price) / self.entry_price) * 100.0
    }
}

impl Portfolio {
    fn new(name: &str, initial_balance: f64) -> Portfolio {
        Portfolio {
            name: String::from(name),
            balance: initial_balance,
            positions: Vec::new(),
        }
    }

    fn buy(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<(), String> {
        let cost = quantity * price;

        if cost > self.balance {
            return Err(format!(
                "Insufficient balance: need ${:.2}, have ${:.2}",
                cost, self.balance
            ));
        }

        self.balance -= cost;

        // Check if position already exists
        for pos in &mut self.positions {
            if pos.symbol == symbol {
                // Average into position
                let total_value = pos.quantity * pos.entry_price + cost;
                pos.quantity += quantity;
                pos.entry_price = total_value / pos.quantity;

                println!("Added to {}: {} @ ${:.2}", symbol, quantity, price);
                return Ok(());
            }
        }

        // New position
        self.positions.push(Position::new(symbol, quantity, price));
        println!("Bought {} {} @ ${:.2}", quantity, symbol, price);
        Ok(())
    }

    fn sell(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<f64, String> {
        for pos in &mut self.positions {
            if pos.symbol == symbol {
                if quantity > pos.quantity {
                    return Err(format!(
                        "Cannot sell {}: have only {}",
                        quantity, pos.quantity
                    ));
                }

                let pnl = (price - pos.entry_price) * quantity;
                let proceeds = quantity * price;

                pos.quantity -= quantity;
                self.balance += proceeds;

                println!("Sold {} {} @ ${:.2}, PnL: ${:.2}",
                    quantity, symbol, price, pnl);

                return Ok(pnl);
            }
        }

        Err(format!("No position in {}", symbol))
    }

    fn total_value(&self, prices: &[(String, f64)]) -> f64 {
        let mut total = self.balance;

        for pos in &self.positions {
            for (symbol, price) in prices {
                if &pos.symbol == symbol {
                    total += pos.market_value(*price);
                    break;
                }
            }
        }

        total
    }

    fn print_summary(&self, prices: &[(String, f64)]) {
        println!("\n╔══════════════════════════════════════╗");
        println!("║  Portfolio: {:25} ║", self.name);
        println!("╚══════════════════════════════════════╝");
        println!("\nCash: ${:.2}", self.balance);
        println!("\nPositions:");
        println!("{:-<50}", "");

        let mut total_unrealized = 0.0;

        for pos in &self.positions {
            if pos.quantity == 0.0 {
                continue;
            }

            for (symbol, price) in prices {
                if &pos.symbol == symbol {
                    let pnl = pos.unrealized_pnl(*price);
                    let pnl_pct = pos.pnl_percent(*price);
                    total_unrealized += pnl;

                    let pnl_sign = if pnl >= 0.0 { "+" } else { "" };

                    println!("{}: {} @ ${:.2} (now ${:.2}) | {}{:.2} ({}{:.2}%)",
                        pos.symbol, pos.quantity, pos.entry_price, price,
                        pnl_sign, pnl, pnl_sign, pnl_pct);
                    break;
                }
            }
        }

        println!("{:-<50}", "");
        println!("Total Value: ${:.2}", self.total_value(prices));
        println!("Unrealized PnL: ${:.2}", total_unrealized);
    }
}

fn main() {
    let mut portfolio = Portfolio::new("My Trading Portfolio", 10000.0);

    // Buy positions
    let _ = portfolio.buy("BTC", 0.1, 42000.0);
    let _ = portfolio.buy("ETH", 2.0, 2500.0);
    let _ = portfolio.buy("BTC", 0.05, 41000.0);  // Add to position

    // Current prices
    let prices = vec![
        (String::from("BTC"), 43000.0),
        (String::from("ETH"), 2600.0),
    ];

    portfolio.print_summary(&prices);

    // Sell some
    println!("\n--- Selling 0.05 BTC ---");
    let _ = portfolio.sell("BTC", 0.05, 43000.0);

    portfolio.print_summary(&prices);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `impl Type` | Block for defining methods |
| `&self` | Borrowing (read only) |
| `&mut self` | Mutable borrowing |
| `self` | Takes ownership |
| `Type::func()` | Associated function |
| `obj.method()` | Method call |
| Method chaining | Chaining methods via `self` |

## Homework

1. **Trading Strategy**: Create a `Strategy` struct with methods:
   - `new(name)` — constructor
   - `should_buy(&self, price, indicators) -> bool`
   - `should_sell(&self, price, indicators) -> bool`
   - `backtest(&self, prices: &[f64]) -> BacktestResult`

2. **Order Book**: Implement an `OrderBook` struct with methods:
   - `add_bid(&mut self, price, quantity)`
   - `add_ask(&mut self, price, quantity)`
   - `best_bid(&self) -> Option<f64>`
   - `best_ask(&self) -> Option<f64>`
   - `spread(&self) -> f64`

3. **Risk Manager**: Create a `RiskManager` struct with methods:
   - `new(max_position_size, max_daily_loss)`
   - `can_open_position(&self, size, portfolio) -> bool`
   - `calculate_position_size(&self, balance, risk_percent, stop_distance) -> f64`
   - `update_daily_pnl(&mut self, pnl)`

4. **Trade Journal**: Implement a `TradeJournal` with methods:
   - `log_trade(&mut self, trade)`
   - `win_rate(&self) -> f64`
   - `average_win(&self) -> f64`
   - `average_loss(&self) -> f64`
   - `profit_factor(&self) -> f64`

## Navigation

[← Previous day](../062-impl-blocks/en.md) | [Next day →](../064-associated-functions/en.md)
