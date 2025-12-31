# Day 111: Input Validation — Checking Trading Orders

## Trading Analogy

Imagine you're working at an exchange. Before accepting an order from a client, the system **must** verify its correctness:
- Is the price positive?
- Is the quantity within acceptable limits?
- Are there sufficient funds in the account?
- Does the ticker exist?

If even one check fails — the order is rejected **before** execution is attempted. This protects both the client and the exchange from errors. In programming, this is called **input validation**.

## Why Validation Matters

```rust
// ❌ Dangerous: no checks
fn execute_order_unsafe(price: f64, quantity: f64) -> f64 {
    price * quantity  // What if price = -1000 or quantity = 0?
}

// ✅ Safe: with validation
fn execute_order_safe(price: f64, quantity: f64) -> Result<f64, String> {
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }
    if quantity <= 0.0 {
        return Err(String::from("Quantity must be positive"));
    }
    Ok(price * quantity)
}
```

## Basic Validation Patterns

### 1. Numeric Value Validation

```rust
fn main() {
    // Price validation
    println!("{:?}", validate_price(42000.0));  // Ok
    println!("{:?}", validate_price(-100.0));   // Err
    println!("{:?}", validate_price(0.0));      // Err

    // Quantity validation
    println!("{:?}", validate_quantity(0.5, 0.001, 100.0));  // Ok
    println!("{:?}", validate_quantity(0.0001, 0.001, 100.0)); // Err: too small
    println!("{:?}", validate_quantity(150.0, 0.001, 100.0));  // Err: too large
}

fn validate_price(price: f64) -> Result<f64, String> {
    if price.is_nan() {
        return Err(String::from("Price cannot be NaN"));
    }
    if price.is_infinite() {
        return Err(String::from("Price cannot be infinite"));
    }
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }
    Ok(price)
}

fn validate_quantity(qty: f64, min: f64, max: f64) -> Result<f64, String> {
    if qty.is_nan() || qty.is_infinite() {
        return Err(String::from("Invalid quantity value"));
    }
    if qty < min {
        return Err(format!("Quantity {} is below minimum {}", qty, min));
    }
    if qty > max {
        return Err(format!("Quantity {} exceeds maximum {}", qty, max));
    }
    Ok(qty)
}
```

### 2. String Validation

```rust
fn main() {
    println!("{:?}", validate_ticker("BTCUSDT"));  // Ok
    println!("{:?}", validate_ticker(""));         // Err: empty
    println!("{:?}", validate_ticker("btc@usdt")); // Err: special chars
    println!("{:?}", validate_ticker("AB"));       // Err: too short
}

fn validate_ticker(ticker: &str) -> Result<&str, String> {
    if ticker.is_empty() {
        return Err(String::from("Ticker cannot be empty"));
    }
    if ticker.len() < 3 {
        return Err(String::from("Ticker too short (minimum 3 characters)"));
    }
    if ticker.len() > 20 {
        return Err(String::from("Ticker too long (maximum 20 characters)"));
    }
    if !ticker.chars().all(|c| c.is_alphanumeric()) {
        return Err(String::from("Ticker can only contain letters and digits"));
    }
    Ok(ticker)
}
```

### 3. Range Validation

```rust
fn main() {
    // Risk percentage
    println!("{:?}", validate_risk_percent(2.0));   // Ok
    println!("{:?}", validate_risk_percent(-1.0));  // Err
    println!("{:?}", validate_risk_percent(150.0)); // Err

    // Stop-loss must be below entry for long positions
    println!("{:?}", validate_stop_loss(42000.0, 41000.0, true));  // Ok
    println!("{:?}", validate_stop_loss(42000.0, 43000.0, true));  // Err
}

fn validate_risk_percent(risk: f64) -> Result<f64, String> {
    if risk <= 0.0 {
        return Err(String::from("Risk must be positive"));
    }
    if risk > 100.0 {
        return Err(String::from("Risk cannot exceed 100%"));
    }
    if risk > 10.0 {
        // Warning, but not an error
        println!("⚠️  Warning: high risk {}%", risk);
    }
    Ok(risk)
}

fn validate_stop_loss(entry: f64, stop_loss: f64, is_long: bool) -> Result<f64, String> {
    if is_long {
        if stop_loss >= entry {
            return Err(format!(
                "Stop-loss ({}) must be below entry price ({}) for long position",
                stop_loss, entry
            ));
        }
    } else {
        if stop_loss <= entry {
            return Err(format!(
                "Stop-loss ({}) must be above entry price ({}) for short position",
                stop_loss, entry
            ));
        }
    }
    Ok(stop_loss)
}
```

## Complete Order Validation

```rust
fn main() {
    let order1 = OrderInput {
        ticker: String::from("BTCUSDT"),
        side: String::from("BUY"),
        price: 42000.0,
        quantity: 0.5,
        stop_loss: Some(41000.0),
        take_profit: Some(45000.0),
    };

    let order2 = OrderInput {
        ticker: String::from(""),
        side: String::from("INVALID"),
        price: -100.0,
        quantity: 0.0,
        stop_loss: None,
        take_profit: None,
    };

    match validate_order(&order1) {
        Ok(valid) => println!("✅ Order valid: {:?}", valid),
        Err(errors) => println!("❌ Errors: {:?}", errors),
    }

    match validate_order(&order2) {
        Ok(valid) => println!("✅ Order valid: {:?}", valid),
        Err(errors) => println!("❌ Errors: {:?}", errors),
    }
}

#[derive(Debug)]
struct OrderInput {
    ticker: String,
    side: String,      // "BUY" or "SELL"
    price: f64,
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

#[derive(Debug)]
struct ValidatedOrder {
    ticker: String,
    is_buy: bool,
    price: f64,
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    total_value: f64,
}

fn validate_order(input: &OrderInput) -> Result<ValidatedOrder, Vec<String>> {
    let mut errors = Vec::new();

    // Ticker validation
    if input.ticker.is_empty() {
        errors.push(String::from("Ticker cannot be empty"));
    } else if input.ticker.len() < 3 {
        errors.push(String::from("Ticker too short"));
    }

    // Side validation
    let is_buy = match input.side.to_uppercase().as_str() {
        "BUY" | "LONG" => true,
        "SELL" | "SHORT" => false,
        _ => {
            errors.push(format!("Unknown side: {}", input.side));
            true // default value to continue validation
        }
    };

    // Price validation
    if input.price <= 0.0 {
        errors.push(String::from("Price must be positive"));
    }

    // Quantity validation
    if input.quantity <= 0.0 {
        errors.push(String::from("Quantity must be positive"));
    }

    // Stop-loss validation (if provided)
    if let Some(sl) = input.stop_loss {
        if sl <= 0.0 {
            errors.push(String::from("Stop-loss must be positive"));
        } else if is_buy && sl >= input.price {
            errors.push(String::from("Stop-loss must be below price for buy orders"));
        } else if !is_buy && sl <= input.price {
            errors.push(String::from("Stop-loss must be above price for sell orders"));
        }
    }

    // Take-profit validation (if provided)
    if let Some(tp) = input.take_profit {
        if tp <= 0.0 {
            errors.push(String::from("Take-profit must be positive"));
        } else if is_buy && tp <= input.price {
            errors.push(String::from("Take-profit must be above price for buy orders"));
        } else if !is_buy && tp >= input.price {
            errors.push(String::from("Take-profit must be below price for sell orders"));
        }
    }

    // Return errors if any
    if !errors.is_empty() {
        return Err(errors);
    }

    // Create validated order
    Ok(ValidatedOrder {
        ticker: input.ticker.clone(),
        is_buy,
        price: input.price,
        quantity: input.quantity,
        stop_loss: input.stop_loss,
        take_profit: input.take_profit,
        total_value: input.price * input.quantity,
    })
}
```

## Validation with Error Accumulation

```rust
fn main() {
    let params = TradingParams {
        balance: -1000.0,      // Error
        risk_percent: 150.0,   // Error
        max_positions: 0,      // Error
        min_trade_size: -1.0,  // Error
    };

    match validate_trading_params(&params) {
        Ok(valid) => println!("Parameters valid: {:?}", valid),
        Err(errors) => {
            println!("Found {} errors:", errors.len());
            for (i, err) in errors.iter().enumerate() {
                println!("  {}. {}", i + 1, err);
            }
        }
    }
}

#[derive(Debug)]
struct TradingParams {
    balance: f64,
    risk_percent: f64,
    max_positions: usize,
    min_trade_size: f64,
}

#[derive(Debug)]
struct ValidatedParams {
    balance: f64,
    risk_percent: f64,
    max_positions: usize,
    min_trade_size: f64,
    max_risk_per_trade: f64,
}

fn validate_trading_params(params: &TradingParams) -> Result<ValidatedParams, Vec<String>> {
    let mut errors = Vec::new();

    if params.balance <= 0.0 {
        errors.push(format!(
            "Balance must be positive, got: {}",
            params.balance
        ));
    }

    if params.risk_percent <= 0.0 || params.risk_percent > 100.0 {
        errors.push(format!(
            "Risk percent must be between 0 and 100, got: {}",
            params.risk_percent
        ));
    }

    if params.max_positions == 0 {
        errors.push(String::from("Max positions must be greater than 0"));
    }

    if params.min_trade_size <= 0.0 {
        errors.push(format!(
            "Min trade size must be positive, got: {}",
            params.min_trade_size
        ));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(ValidatedParams {
        balance: params.balance,
        risk_percent: params.risk_percent,
        max_positions: params.max_positions,
        min_trade_size: params.min_trade_size,
        max_risk_per_trade: params.balance * (params.risk_percent / 100.0),
    })
}
```

## Type-Based Validation

```rust
fn main() {
    // Creating safe types
    match Price::new(42000.0) {
        Ok(price) => println!("Price: {}", price.value()),
        Err(e) => println!("Error: {}", e),
    }

    match Quantity::new(0.5, 0.001, 100.0) {
        Ok(qty) => println!("Quantity: {}", qty.value()),
        Err(e) => println!("Error: {}", e),
    }

    // Using validated types
    let price = Price::new(42000.0).unwrap();
    let qty = Quantity::new(0.5, 0.001, 100.0).unwrap();

    println!("Total value: {}", calculate_total(&price, &qty));
}

#[derive(Debug, Clone, Copy)]
struct Price(f64);

impl Price {
    fn new(value: f64) -> Result<Self, String> {
        if value.is_nan() || value.is_infinite() {
            return Err(String::from("Invalid price value"));
        }
        if value <= 0.0 {
            return Err(String::from("Price must be positive"));
        }
        Ok(Price(value))
    }

    fn value(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

impl Quantity {
    fn new(value: f64, min: f64, max: f64) -> Result<Self, String> {
        if value.is_nan() || value.is_infinite() {
            return Err(String::from("Invalid quantity value"));
        }
        if value < min {
            return Err(format!("Quantity below minimum: {} < {}", value, min));
        }
        if value > max {
            return Err(format!("Quantity above maximum: {} > {}", value, max));
        }
        Ok(Quantity(value))
    }

    fn value(&self) -> f64 {
        self.0
    }
}

// Function accepts only validated types
fn calculate_total(price: &Price, qty: &Quantity) -> f64 {
    price.value() * qty.value()
}
```

## Practical Example: Order Validation System

```rust
fn main() {
    let validator = OrderValidator::new(
        10000.0,    // balance
        100.0,      // max position size
        0.001,      // min quantity
        100.0,      // max quantity
    );

    // Test orders
    let orders = vec![
        ("BTCUSDT", 42000.0, 0.1),
        ("BTCUSDT", 42000.0, 1.0),      // Exceeds balance
        ("ETHUSDT", -1500.0, 0.5),      // Negative price
        ("", 100.0, 1.0),               // Empty ticker
    ];

    for (ticker, price, qty) in orders {
        println!("\n📋 Checking: {} @ {} x {}", ticker, price, qty);
        match validator.validate(ticker, price, qty) {
            Ok(order) => {
                println!("  ✅ Accepted");
                println!("  💰 Value: ${:.2}", order.total_value);
            }
            Err(errors) => {
                println!("  ❌ Rejected:");
                for err in errors {
                    println!("     - {}", err);
                }
            }
        }
    }
}

struct OrderValidator {
    balance: f64,
    max_position_value: f64,
    min_quantity: f64,
    max_quantity: f64,
}

struct ValidOrder {
    ticker: String,
    price: f64,
    quantity: f64,
    total_value: f64,
}

impl OrderValidator {
    fn new(balance: f64, max_position_value: f64, min_qty: f64, max_qty: f64) -> Self {
        OrderValidator {
            balance,
            max_position_value,
            min_quantity: min_qty,
            max_quantity: max_qty,
        }
    }

    fn validate(&self, ticker: &str, price: f64, quantity: f64) -> Result<ValidOrder, Vec<String>> {
        let mut errors = Vec::new();

        // Ticker validation
        if ticker.is_empty() {
            errors.push(String::from("Ticker is required"));
        } else if !ticker.chars().all(|c| c.is_alphanumeric()) {
            errors.push(String::from("Ticker contains invalid characters"));
        }

        // Price validation
        if price <= 0.0 {
            errors.push(String::from("Price must be positive"));
        } else if price.is_nan() || price.is_infinite() {
            errors.push(String::from("Invalid price value"));
        }

        // Quantity validation
        if quantity < self.min_quantity {
            errors.push(format!(
                "Quantity {} is below minimum {}",
                quantity, self.min_quantity
            ));
        }
        if quantity > self.max_quantity {
            errors.push(format!(
                "Quantity {} exceeds maximum {}",
                quantity, self.max_quantity
            ));
        }

        // Value check (only if price and quantity are valid)
        if price > 0.0 && quantity > 0.0 {
            let total = price * quantity;

            if total > self.balance {
                errors.push(format!(
                    "Insufficient funds: need ${:.2}, available ${:.2}",
                    total, self.balance
                ));
            }

            if total > self.max_position_value {
                errors.push(format!(
                    "Position limit exceeded: ${:.2} > ${:.2}",
                    total, self.max_position_value
                ));
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidOrder {
            ticker: ticker.to_string(),
            price,
            quantity,
            total_value: price * quantity,
        })
    }
}
```

## What We Learned

| Pattern | Use Case | Trading Example |
|---------|----------|-----------------|
| Early return | Quick check for critical conditions | Price > 0 |
| Error accumulation | Show all problems at once | All order fields |
| Wrapper types | Type-level validity guarantee | Price, Quantity |
| Complex validation | Check related fields | Stop-loss vs Entry |

## Key Validation Rules in Trading

1. **Always check for NaN and Infinity** — floating point numbers are tricky
2. **Validate boundaries** — minimums and maximums for all values
3. **Validate relationships** — stop-loss must match position direction
4. **Accumulate errors** — users prefer seeing all problems at once
5. **Use types** — validated types don't need re-validation

## Homework

1. Write a function `validate_portfolio_allocation(allocations: &[f64]) -> Result<(), String>` that checks if allocations sum to 100%

2. Create a validator for trading strategy parameters:
   - SMA period (integer, 1 to 200)
   - RSI period (integer, 2 to 100)
   - Risk per trade (0.1% to 5%)
   - Take-profit ratio (1.0 to 10.0)

3. Implement a `RiskPercentage` type that:
   - Accepts values from 0.1 to 100.0
   - Has a method `of_balance(balance: f64) -> f64`
   - Automatically rounds to 2 decimal places

4. Write a function to validate an array of historical prices:
   - Non-empty array
   - All values positive
   - No sudden jumps (more than 50% in one candle)
   - Returns `Vec<String>` with all anomalies found

## Navigation

[← Previous day](../110-error-matching-patterns/en.md) | [Next day →](../112-error-custom-types/en.md)
