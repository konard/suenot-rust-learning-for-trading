# Day 112: Newtype Pattern for Validation

## Trading Analogy

Imagine you have a trading terminal where price and quantity are just `f64` numbers. What happens if you accidentally pass quantity instead of price? The compiler won't catch the error, and you might place an order to buy 42000 bitcoins at price 0.5!

**Newtype pattern** is like special envelopes for different types of data. Price goes in a "Price envelope", quantity goes in a "Quantity envelope". Now it's impossible to mix them up.

## What is the Newtype Pattern?

Newtype is a wrapper struct with a single field that creates a new type based on an existing one:

```rust
struct Price(f64);      // Price is f64, but "wrapped"
struct Quantity(f64);   // Quantity is also f64, but a different type!
```

## Basic Example: Preventing Mix-ups

```rust
fn main() {
    let price = Price::new(42000.0).unwrap();
    let quantity = Quantity::new(0.5).unwrap();

    // This compiles:
    let order = create_order(price, quantity);

    // But this does NOT! Compiler protects us:
    // let bad_order = create_order(quantity, price); // Compilation error!
}

#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

#[derive(Debug)]
struct Order {
    price: Price,
    quantity: Quantity,
}

fn create_order(price: Price, quantity: Quantity) -> Order {
    Order { price, quantity }
}

impl Price {
    fn new(value: f64) -> Option<Price> {
        if value > 0.0 {
            Some(Price(value))
        } else {
            None
        }
    }

    fn value(&self) -> f64 {
        self.0
    }
}

impl Quantity {
    fn new(value: f64) -> Option<Quantity> {
        if value > 0.0 {
            Some(Quantity(value))
        } else {
            None
        }
    }

    fn value(&self) -> f64 {
        self.0
    }
}
```

## Validation at Creation

The main power of newtype is validation right in the constructor:

```rust
fn main() {
    // Valid values
    let valid_price = Price::new(42000.0);
    println!("Valid price: {:?}", valid_price);

    // Invalid values are rejected
    let invalid_price = Price::new(-100.0);
    println!("Invalid price: {:?}", invalid_price);

    let zero_price = Price::new(0.0);
    println!("Zero price: {:?}", zero_price);
}

#[derive(Debug, Clone, Copy)]
struct Price(f64);

impl Price {
    fn new(value: f64) -> Result<Price, PriceError> {
        if value.is_nan() {
            return Err(PriceError::NotANumber);
        }
        if value.is_infinite() {
            return Err(PriceError::Infinite);
        }
        if value <= 0.0 {
            return Err(PriceError::NonPositive(value));
        }
        if value > 1_000_000_000.0 {
            return Err(PriceError::TooLarge(value));
        }
        Ok(Price(value))
    }

    fn value(&self) -> f64 {
        self.0
    }
}

#[derive(Debug)]
enum PriceError {
    NonPositive(f64),
    TooLarge(f64),
    NotANumber,
    Infinite,
}
```

## Newtype for Risk Percentage

```rust
fn main() {
    let risk = RiskPercent::new(2.0).unwrap();
    println!("Risk: {}%", risk.value());

    let balance = 10000.0;
    let risk_amount = risk.calculate_amount(balance);
    println!("Risk amount for ${}: ${:.2}", balance, risk_amount);

    // Attempting to create invalid risk
    match RiskPercent::new(150.0) {
        Ok(_) => println!("Created"),
        Err(e) => println!("Error: {:?}", e),
    }
}

#[derive(Debug, Clone, Copy)]
struct RiskPercent(f64);

#[derive(Debug)]
enum RiskError {
    Negative(f64),
    TooHigh(f64),
    Zero,
}

impl RiskPercent {
    /// Creates a risk percentage (allowed from 0.01% to 100%)
    fn new(value: f64) -> Result<RiskPercent, RiskError> {
        if value <= 0.0 {
            return Err(RiskError::Zero);
        }
        if value < 0.0 {
            return Err(RiskError::Negative(value));
        }
        if value > 100.0 {
            return Err(RiskError::TooHigh(value));
        }
        Ok(RiskPercent(value))
    }

    fn value(&self) -> f64 {
        self.0
    }

    fn calculate_amount(&self, balance: f64) -> f64 {
        balance * (self.0 / 100.0)
    }
}
```

## Newtype for Ticker (Asset Symbol)

```rust
fn main() {
    match Ticker::new("BTCUSDT") {
        Ok(ticker) => println!("Valid ticker: {}", ticker.as_str()),
        Err(e) => println!("Error: {:?}", e),
    }

    match Ticker::new("") {
        Ok(ticker) => println!("Valid ticker: {}", ticker.as_str()),
        Err(e) => println!("Error: {:?}", e),
    }

    match Ticker::new("BTC-USDT-PERP-FUTURE") {
        Ok(ticker) => println!("Valid ticker: {}", ticker.as_str()),
        Err(e) => println!("Error: {:?}", e),
    }
}

#[derive(Debug, Clone)]
struct Ticker(String);

#[derive(Debug)]
enum TickerError {
    Empty,
    TooLong(usize),
    InvalidCharacter(char),
}

impl Ticker {
    fn new(value: &str) -> Result<Ticker, TickerError> {
        if value.is_empty() {
            return Err(TickerError::Empty);
        }

        if value.len() > 20 {
            return Err(TickerError::TooLong(value.len()));
        }

        // Only letters, digits, and hyphens
        for ch in value.chars() {
            if !ch.is_alphanumeric() && ch != '-' {
                return Err(TickerError::InvalidCharacter(ch));
            }
        }

        Ok(Ticker(value.to_uppercase()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}
```

## Implementing Arithmetic Operations

Newtypes can be given arithmetic capabilities:

```rust
use std::ops::{Add, Sub, Mul};

fn main() {
    let price1 = Price::new(42000.0).unwrap();
    let price2 = Price::new(1500.0).unwrap();

    let sum = price1 + price2;
    println!("Sum: ${:.2}", sum.value());

    let diff = price1 - price2;
    println!("Diff: ${:.2}", diff);

    let qty = Quantity::new(0.5).unwrap();
    let value = price1 * qty;
    println!("Position value: ${:.2}", value);
}

#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

impl Price {
    fn new(value: f64) -> Option<Price> {
        if value > 0.0 { Some(Price(value)) } else { None }
    }
    fn value(&self) -> f64 { self.0 }
}

impl Quantity {
    fn new(value: f64) -> Option<Quantity> {
        if value > 0.0 { Some(Quantity(value)) } else { None }
    }
    fn value(&self) -> f64 { self.0 }
}

// Price + Price = Price
impl Add for Price {
    type Output = Price;
    fn add(self, other: Price) -> Price {
        Price(self.0 + other.0)
    }
}

// Price - Price = f64 (difference can be negative!)
impl Sub for Price {
    type Output = f64;  // Return f64 since difference can be negative
    fn sub(self, other: Price) -> f64 {
        self.0 - other.0
    }
}

// Price * Quantity = f64 (position value)
impl Mul<Quantity> for Price {
    type Output = f64;
    fn mul(self, qty: Quantity) -> f64 {
        self.0 * qty.0
    }
}
```

## Complete Example: Order Management System

```rust
fn main() {
    // Create validated values
    let ticker = Ticker::new("BTCUSDT").expect("Invalid ticker");
    let price = Price::new(42000.0).expect("Invalid price");
    let quantity = Quantity::new(0.5).expect("Invalid quantity");
    let stop_loss = Price::new(40000.0).expect("Invalid stop loss");

    // Create order with full validation
    match Order::new(ticker, price, quantity, Some(stop_loss)) {
        Ok(order) => {
            println!("{}", order.summary());
        }
        Err(e) => {
            println!("Order creation failed: {:?}", e);
        }
    }
}

#[derive(Debug, Clone)]
struct Ticker(String);

#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

#[derive(Debug)]
struct Order {
    ticker: Ticker,
    price: Price,
    quantity: Quantity,
    stop_loss: Option<Price>,
}

#[derive(Debug)]
enum OrderError {
    StopLossTooHigh,
    PositionTooSmall,
}

impl Ticker {
    fn new(value: &str) -> Option<Ticker> {
        if !value.is_empty() && value.len() <= 20 {
            Some(Ticker(value.to_uppercase()))
        } else {
            None
        }
    }
    fn as_str(&self) -> &str { &self.0 }
}

impl Price {
    fn new(value: f64) -> Option<Price> {
        if value > 0.0 && value.is_finite() { Some(Price(value)) } else { None }
    }
    fn value(&self) -> f64 { self.0 }
}

impl Quantity {
    fn new(value: f64) -> Option<Quantity> {
        if value > 0.0 && value.is_finite() { Some(Quantity(value)) } else { None }
    }
    fn value(&self) -> f64 { self.0 }
}

impl Order {
    fn new(
        ticker: Ticker,
        price: Price,
        quantity: Quantity,
        stop_loss: Option<Price>,
    ) -> Result<Order, OrderError> {
        // Additional validation at order level
        if let Some(sl) = stop_loss {
            if sl.value() >= price.value() {
                return Err(OrderError::StopLossTooHigh);
            }
        }

        let position_value = price.value() * quantity.value();
        if position_value < 10.0 {
            return Err(OrderError::PositionTooSmall);
        }

        Ok(Order {
            ticker,
            price,
            quantity,
            stop_loss,
        })
    }

    fn position_value(&self) -> f64 {
        self.price.value() * self.quantity.value()
    }

    fn summary(&self) -> String {
        let sl_info = match &self.stop_loss {
            Some(sl) => format!(" | SL: ${:.2}", sl.value()),
            None => String::new(),
        };

        format!(
            "Order: {} | {:.4} @ ${:.2} | Value: ${:.2}{}",
            self.ticker.as_str(),
            self.quantity.value(),
            self.price.value(),
            self.position_value(),
            sl_info
        )
    }
}
```

## Newtype for Identifiers

```rust
fn main() {
    let order_id = OrderId::generate();
    let trade_id = TradeId::generate();

    println!("Order ID: {}", order_id.as_str());
    println!("Trade ID: {}", trade_id.as_str());

    // This won't compile — different types!
    // process_order(trade_id); // Error!
    process_order(order_id);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OrderId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TradeId(String);

impl OrderId {
    fn generate() -> OrderId {
        OrderId(format!("ORD-{}", random_suffix()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl TradeId {
    fn generate() -> TradeId {
        TradeId(format!("TRD-{}", random_suffix()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

fn random_suffix() -> u32 {
    // Simplified generation for example
    12345
}

fn process_order(order_id: OrderId) {
    println!("Processing order: {}", order_id.as_str());
}
```

## Parse, Don't Validate Pattern

Instead of validating data in every function, validate once at creation:

```rust
fn main() {
    // ❌ Bad: validation in every function
    // fn calculate_risk(balance: f64, risk_pct: f64) -> f64 {
    //     assert!(balance > 0.0);
    //     assert!(risk_pct > 0.0 && risk_pct <= 100.0);
    //     balance * risk_pct / 100.0
    // }

    // ✅ Good: validation at type creation
    let balance = Balance::new(10000.0).expect("Invalid balance");
    let risk = RiskPercent::new(2.0).expect("Invalid risk");

    // Function receives already validated types
    let risk_amount = calculate_risk(balance, risk);
    println!("Risk amount: ${:.2}", risk_amount);
}

#[derive(Debug, Clone, Copy)]
struct Balance(f64);

#[derive(Debug, Clone, Copy)]
struct RiskPercent(f64);

impl Balance {
    fn new(value: f64) -> Option<Balance> {
        if value >= 0.0 && value.is_finite() {
            Some(Balance(value))
        } else {
            None
        }
    }
    fn value(&self) -> f64 { self.0 }
}

impl RiskPercent {
    fn new(value: f64) -> Option<RiskPercent> {
        if value > 0.0 && value <= 100.0 {
            Some(RiskPercent(value))
        } else {
            None
        }
    }
    fn value(&self) -> f64 { self.0 }
}

// Function cannot receive invalid data!
fn calculate_risk(balance: Balance, risk: RiskPercent) -> f64 {
    balance.value() * risk.value() / 100.0
}
```

## What We Learned

| Concept | Description | Example |
|---------|-------------|---------|
| Newtype | Wrapper struct with single field | `struct Price(f64)` |
| Type Safety | Compiler won't let you mix types | `Price` ≠ `Quantity` |
| Validation | Check at creation | `Price::new()` |
| Encapsulation | Access only through methods | `price.value()` |
| Parse, don't validate | Validate once | Types guarantee correctness |

## Exercises

1. **Create newtype `Leverage`** for leverage (from 1x to 125x):
   ```rust
   // Leverage::new(10) -> Ok(Leverage)
   // Leverage::new(200) -> Err(LeverageError::TooHigh)
   ```

2. **Implement newtype `Email`** for email addresses with basic validation (must contain @)

3. **Add arithmetic** to type `Balance`:
   - `Balance + Balance = Balance`
   - `Balance - Balance = Result<Balance, BalanceError>` (can't go negative)

4. **Create a type system** for trading:
   - `BaseAsset` and `QuoteAsset` (can't mix up BTC and USDT)
   - `TradingPair` that combines them

## Homework

1. Create a complete type system for an order with newtype for all fields:
   - `OrderId`, `Price`, `Quantity`, `Leverage`, `Ticker`
   - All types must be validated at creation

2. Implement a position calculator with type-safe computations:
   - `PositionSize = Balance / Price`
   - `PositionValue = Price * Quantity`
   - `PnL = (ExitPrice - EntryPrice) * Quantity`

3. Create newtype `Timestamp` for timestamps with validation (not in the future, not before 2009 — the year Bitcoin was created)

4. Implement a currency converter with newtype for each currency:
   ```rust
   let btc = Btc::new(1.0)?;
   let rate = ExchangeRate::new(42000.0)?;
   let usdt: Usdt = btc.convert(rate);
   ```

## Navigation

[← Previous day](../111-input-validation/en.md) | [Next day →](../113-builder-pattern/en.md)
