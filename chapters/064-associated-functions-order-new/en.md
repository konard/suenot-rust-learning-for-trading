# Day 64: Associated Functions — Order::new()

## Trading Analogy

Imagine an exchange. The exchange has a special department that creates new orders. You don't create an order yourself — you go to the exchange department and say: "Create a BTC buy order for me." This department is part of the "Order" system, but it doesn't work with a specific order — it creates new ones.

In Rust, these are **associated functions** — functions linked to a struct but not working with a specific instance. The most well-known is `new()`, the constructor.

## Methods vs Associated Functions

```rust
struct Order {
    symbol: String,
    quantity: f64,
    price: f64,
}

impl Order {
    // Associated function — NO self parameter
    // Called: Order::new(...)
    fn new(symbol: &str, quantity: f64, price: f64) -> Order {
        Order {
            symbol: symbol.to_string(),
            quantity,
            price,
        }
    }

    // Method — HAS self parameter
    // Called: order.total_value()
    fn total_value(&self) -> f64 {
        self.quantity * self.price
    }
}

fn main() {
    // Associated function called with ::
    let order = Order::new("BTC", 0.5, 42000.0);

    // Method called with .
    println!("Total value: ${:.2}", order.total_value());
}
```

**Key difference:**
- `Order::new()` — double colon, no instance before the dot
- `order.total_value()` — dot, instance `order` exists

## The new() Constructor

The `new()` pattern is the standard way to create instances in Rust:

```rust
struct Trade {
    symbol: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    exit_price: Option<f64>,
}

impl Trade {
    fn new(symbol: &str, side: &str, quantity: f64, entry_price: f64) -> Trade {
        Trade {
            symbol: symbol.to_string(),
            side: side.to_string(),
            quantity,
            entry_price,
            exit_price: None, // Trade is still open
        }
    }
}

fn main() {
    let trade = Trade::new("ETH", "buy", 2.0, 2500.0);
    println!("Opened {} {} @ ${}", trade.side, trade.symbol, trade.entry_price);
}
```

## Multiple Constructors

You can create multiple associated functions for different scenarios:

```rust
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

impl Position {
    // Main constructor
    fn new(symbol: &str, quantity: f64, avg_price: f64) -> Position {
        Position {
            symbol: symbol.to_string(),
            quantity,
            avg_price,
        }
    }

    // Create empty position
    fn empty(symbol: &str) -> Position {
        Position {
            symbol: symbol.to_string(),
            quantity: 0.0,
            avg_price: 0.0,
        }
    }

    // Create from trade
    fn from_trade(symbol: &str, quantity: f64, price: f64) -> Position {
        Position::new(symbol, quantity, price)
    }

    fn value(&self) -> f64 {
        self.quantity * self.avg_price
    }
}

fn main() {
    let pos1 = Position::new("BTC", 0.5, 42000.0);
    let pos2 = Position::empty("ETH");
    let pos3 = Position::from_trade("SOL", 100.0, 25.0);

    println!("BTC position value: ${:.2}", pos1.value());
    println!("ETH position value: ${:.2}", pos2.value());
    println!("SOL position value: ${:.2}", pos3.value());
}
```

## Validation in Constructor

Associated functions can include validation logic:

```rust
struct Order {
    symbol: String,
    quantity: f64,
    price: f64,
}

impl Order {
    fn new(symbol: &str, quantity: f64, price: f64) -> Option<Order> {
        // Validation: quantity and price must be positive
        if quantity <= 0.0 || price <= 0.0 {
            return None;
        }

        // Validation: symbol must not be empty
        if symbol.is_empty() {
            return None;
        }

        Some(Order {
            symbol: symbol.to_string(),
            quantity,
            price,
        })
    }
}

fn main() {
    match Order::new("BTC", 0.5, 42000.0) {
        Some(order) => println!("Order created: {} @ ${}", order.symbol, order.price),
        None => println!("Invalid order parameters"),
    }

    match Order::new("", -1.0, 0.0) {
        Some(_) => println!("Order created"),
        None => println!("Invalid order parameters — rejected"),
    }
}
```

## Constructor with Result

For more detailed error information, use Result:

```rust
#[derive(Debug)]
enum OrderError {
    InvalidQuantity,
    InvalidPrice,
    EmptySymbol,
}

struct LimitOrder {
    symbol: String,
    quantity: f64,
    price: f64,
}

impl LimitOrder {
    fn new(symbol: &str, quantity: f64, price: f64) -> Result<LimitOrder, OrderError> {
        if symbol.is_empty() {
            return Err(OrderError::EmptySymbol);
        }
        if quantity <= 0.0 {
            return Err(OrderError::InvalidQuantity);
        }
        if price <= 0.0 {
            return Err(OrderError::InvalidPrice);
        }

        Ok(LimitOrder {
            symbol: symbol.to_string(),
            quantity,
            price,
        })
    }
}

fn main() {
    match LimitOrder::new("BTC", 0.5, 42000.0) {
        Ok(order) => println!("Order: {} {} @ ${}", order.symbol, order.quantity, order.price),
        Err(e) => println!("Error: {:?}", e),
    }

    match LimitOrder::new("BTC", -1.0, 42000.0) {
        Ok(_) => println!("Order created"),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Factory Functions

Associated functions can create instances with preset parameters:

```rust
struct RiskSettings {
    max_position_size: f64,
    max_loss_percent: f64,
    max_daily_trades: u32,
}

impl RiskSettings {
    fn new(max_position: f64, max_loss: f64, max_trades: u32) -> RiskSettings {
        RiskSettings {
            max_position_size: max_position,
            max_loss_percent: max_loss,
            max_daily_trades: max_trades,
        }
    }

    // Conservative settings
    fn conservative() -> RiskSettings {
        RiskSettings {
            max_position_size: 1000.0,
            max_loss_percent: 1.0,
            max_daily_trades: 3,
        }
    }

    // Moderate settings
    fn moderate() -> RiskSettings {
        RiskSettings {
            max_position_size: 5000.0,
            max_loss_percent: 2.0,
            max_daily_trades: 10,
        }
    }

    // Aggressive settings
    fn aggressive() -> RiskSettings {
        RiskSettings {
            max_position_size: 20000.0,
            max_loss_percent: 5.0,
            max_daily_trades: 50,
        }
    }
}

fn main() {
    let settings = RiskSettings::conservative();
    println!("Max position: ${}", settings.max_position_size);
    println!("Max loss: {}%", settings.max_loss_percent);
    println!("Max trades: {}", settings.max_daily_trades);

    let aggressive = RiskSettings::aggressive();
    println!("\nAggressive - Max position: ${}", aggressive.max_position_size);
}
```

## Practical Example: Order Builder

```rust
#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    order_type: String,
    quantity: f64,
    price: Option<f64>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl Order {
    // Counter for ID generation
    fn next_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    // Market buy order
    fn market_buy(symbol: &str, quantity: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: "buy".to_string(),
            order_type: "market".to_string(),
            quantity,
            price: None,
            stop_loss: None,
            take_profit: None,
        }
    }

    // Market sell order
    fn market_sell(symbol: &str, quantity: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: "sell".to_string(),
            order_type: "market".to_string(),
            quantity,
            price: None,
            stop_loss: None,
            take_profit: None,
        }
    }

    // Limit buy order
    fn limit_buy(symbol: &str, quantity: f64, price: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            quantity,
            price: Some(price),
            stop_loss: None,
            take_profit: None,
        }
    }

    // Limit sell order
    fn limit_sell(symbol: &str, quantity: f64, price: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: "sell".to_string(),
            order_type: "limit".to_string(),
            quantity,
            price: Some(price),
            stop_loss: None,
            take_profit: None,
        }
    }

    // Order with protective levels (bracket order)
    fn with_bracket(symbol: &str, quantity: f64, entry: f64, sl: f64, tp: f64) -> Order {
        Order {
            id: Order::next_id(),
            symbol: symbol.to_string(),
            side: if entry > sl { "buy" } else { "sell" }.to_string(),
            order_type: "limit".to_string(),
            quantity,
            price: Some(entry),
            stop_loss: Some(sl),
            take_profit: Some(tp),
        }
    }

    fn describe(&self) {
        println!("Order #{}: {} {} {} @ {:?}",
            self.id, self.side, self.quantity, self.symbol,
            self.price.map(|p| format!("${:.2}", p)).unwrap_or("MARKET".to_string())
        );
        if let Some(sl) = self.stop_loss {
            println!("  Stop Loss: ${:.2}", sl);
        }
        if let Some(tp) = self.take_profit {
            println!("  Take Profit: ${:.2}", tp);
        }
    }
}

fn main() {
    println!("=== Order Factory ===\n");

    let order1 = Order::market_buy("BTC", 0.1);
    order1.describe();

    let order2 = Order::limit_sell("ETH", 5.0, 2800.0);
    order2.describe();

    let order3 = Order::with_bracket("BTC", 0.5, 42000.0, 40000.0, 46000.0);
    order3.describe();
}
```

## Self as Return Type

Instead of the struct name, you can use `Self`:

```rust
struct Wallet {
    balance: f64,
    currency: String,
}

impl Wallet {
    fn new(balance: f64, currency: &str) -> Self {
        Self {
            balance,
            currency: currency.to_string(),
        }
    }

    fn usd(balance: f64) -> Self {
        Self::new(balance, "USD")
    }

    fn btc(balance: f64) -> Self {
        Self::new(balance, "BTC")
    }

    fn empty(currency: &str) -> Self {
        Self::new(0.0, currency)
    }
}

fn main() {
    let wallet1 = Wallet::usd(10000.0);
    let wallet2 = Wallet::btc(0.5);
    let wallet3 = Wallet::empty("ETH");

    println!("{}: {:.2}", wallet1.currency, wallet1.balance);
    println!("{}: {:.8}", wallet2.currency, wallet2.balance);
    println!("{}: {:.4}", wallet3.currency, wallet3.balance);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Struct::func()` | Calling an associated function |
| No `self` | No instance required |
| `new()` | Standard constructor |
| `Self` | Alias for current type |
| Factory functions | Creation with presets |
| Validation | Option/Result in constructor |

## Exercises

1. Create a `Candle` struct (OHLCV) with associated functions:
   - `new(open, high, low, close, volume)` — standard constructor
   - `from_price(price)` — candle where O=H=L=C=price, volume=0
   - `bullish(open, close, volume)` — green candle (close > open)
   - `bearish(open, close, volume)` — red candle (close < open)

2. Create a `StopOrder` struct with validation:
   - `new()` returns `Result<StopOrder, String>`
   - Check that stop_price > 0
   - Check that quantity > 0

3. Create a `TradingSession` struct with factory functions:
   - `new_york()` — NYSE hours
   - `london()` — LSE hours
   - `tokyo()` — TSE hours
   - `crypto()` — 24/7

## Homework

1. Create a `Portfolio` struct with:
   - `new()` — empty portfolio
   - `with_cash(amount)` — portfolio with initial capital
   - `demo()` — demo portfolio with virtual $100,000

2. Create a `Strategy` struct with factories:
   - `sma_crossover(fast, slow)` — SMA crossover strategy
   - `rsi_oversold(period, level)` — RSI oversold strategy
   - `breakout(period)` — breakout strategy

3. Create an `ExchangeAPI` struct with:
   - `binance()` — Binance API
   - `bybit()` — Bybit API
   - `demo()` — Demo mode without real requests

## Navigation

[← Day 63: Methods](../063-methods-order-execute/en.md) | [Day 65: Multiple impl Blocks →](../065-multiple-impl-blocks/en.md)
