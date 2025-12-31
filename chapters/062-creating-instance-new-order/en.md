# Day 62: Creating Instance — New Order

## Trading Analogy

When you want to buy or sell an asset on an exchange, you fill out an **order form**: specifying the ticker, direction (buy/sell), price, and quantity. This is exactly what creating an instance means — you take a template (struct) and fill it with specific data.

## Basic Instance Creation

```rust
// Define order structure
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    // Create instance — fill in all fields
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("Buy"),
        price: 42000.0,
        quantity: 0.5,
    };

    println!("Order: {} {} {} at price {}",
        order.side, order.quantity, order.symbol, order.price);
}
```

**Important:** You must specify values for **all** struct fields!

## Field Order Doesn't Matter

```rust
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
}

fn main() {
    // Fields can be specified in any order
    let trade = Trade {
        quantity: 1.0,           // Quantity first
        symbol: String::from("ETH/USDT"),  // Then symbol
        exit_price: 2200.0,      // Exit price
        entry_price: 2000.0,     // Entry price
    };

    let pnl = (trade.exit_price - trade.entry_price) * trade.quantity;
    println!("Trade on {}: PnL = ${:.2}", trade.symbol, pnl);
}
```

## Field Init Shorthand

When a variable name matches the field name, you can use shorthand syntax:

```rust
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

fn main() {
    let symbol = String::from("SOL/USDT");
    let quantity = 10.0;
    let entry_price = 100.0;

    // Long form
    let position1 = Position {
        symbol: symbol.clone(),
        quantity: quantity,
        entry_price: entry_price,
    };

    // Shorthand form — when names match
    let symbol = String::from("AVAX/USDT");
    let quantity = 5.0;
    let entry_price = 35.0;

    let position2 = Position {
        symbol,       // instead of symbol: symbol
        quantity,     // instead of quantity: quantity
        entry_price,  // instead of entry_price: entry_price
    };

    println!("Position 1: {} x {}", position1.symbol, position1.quantity);
    println!("Position 2: {} x {}", position2.symbol, position2.quantity);
}
```

## Struct Update Syntax

Allows creating a new instance based on an existing one, changing only some fields:

```rust
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    time_in_force: String,
}

fn main() {
    // Base order
    let base_order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("Buy"),
        price: 42000.0,
        quantity: 1.0,
        time_in_force: String::from("GTC"), // Good Till Cancelled
    };

    // New order with different price, rest from base_order
    let limit_order = Order {
        price: 41500.0,  // Different price
        ..base_order     // Rest of fields from base_order
    };

    // Careful! base_order.symbol was moved to limit_order
    // base_order can no longer be used as a whole

    println!("Limit order at price: {}", limit_order.price);
    println!("Symbol: {}", limit_order.symbol);
}
```

**Important:** `..base_order` must be at the end!

## Creating with Clone for Reuse

```rust
#[derive(Clone)]
struct OrderTemplate {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    let template = OrderTemplate {
        symbol: String::from("ETH/USDT"),
        side: String::from("Buy"),
        price: 2000.0,
        quantity: 1.0,
    };

    // Create multiple orders based on template
    let order1 = OrderTemplate {
        price: 1950.0,
        ..template.clone()  // Clone so template remains available
    };

    let order2 = OrderTemplate {
        price: 1900.0,
        ..template.clone()
    };

    let order3 = OrderTemplate {
        price: 1850.0,
        ..template  // Last use — can skip clone
    };

    println!("Ladder orders: {}, {}, {}",
        order1.price, order2.price, order3.price);
}
```

## Practical Example: Creating Trading Orders

```rust
struct MarketOrder {
    symbol: String,
    side: String,
    quantity: f64,
    timestamp: u64,
}

struct LimitOrder {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    time_in_force: String,
}

struct StopOrder {
    symbol: String,
    side: String,
    stop_price: f64,
    quantity: f64,
}

fn main() {
    // Market order — executes immediately at current price
    let market_buy = MarketOrder {
        symbol: String::from("BTC/USDT"),
        side: String::from("Buy"),
        quantity: 0.1,
        timestamp: 1704067200,
    };

    // Limit order — waits for desired price
    let limit_buy = LimitOrder {
        symbol: String::from("BTC/USDT"),
        side: String::from("Buy"),
        price: 40000.0,
        quantity: 0.5,
        time_in_force: String::from("GTC"),
    };

    // Stop order — activates when price is reached
    let stop_loss = StopOrder {
        symbol: String::from("BTC/USDT"),
        side: String::from("Sell"),
        stop_price: 38000.0,
        quantity: 0.5,
    };

    println!("Market: {} {} x {}",
        market_buy.side, market_buy.symbol, market_buy.quantity);
    println!("Limit: {} {} x {} @ {}",
        limit_buy.side, limit_buy.symbol, limit_buy.quantity, limit_buy.price);
    println!("Stop: {} {} x {} @ {}",
        stop_loss.side, stop_loss.symbol, stop_loss.quantity, stop_loss.stop_price);
}
```

## Creating Instances in Functions

```rust
struct Trade {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    pnl: f64,
}

fn create_trade(id: u64, symbol: &str, side: &str, price: f64, quantity: f64) -> Trade {
    Trade {
        id,
        symbol: String::from(symbol),
        side: String::from(side),
        price,
        quantity,
        pnl: 0.0,  // PnL will be calculated later
    }
}

fn close_trade(open_trade: &Trade, exit_price: f64) -> Trade {
    let pnl = if open_trade.side == "Buy" {
        (exit_price - open_trade.price) * open_trade.quantity
    } else {
        (open_trade.price - exit_price) * open_trade.quantity
    };

    Trade {
        id: open_trade.id,
        symbol: open_trade.symbol.clone(),
        side: if open_trade.side == "Buy" {
            String::from("Sell")
        } else {
            String::from("Buy")
        },
        price: exit_price,
        quantity: open_trade.quantity,
        pnl,
    }
}

fn main() {
    let entry = create_trade(1, "BTC/USDT", "Buy", 42000.0, 0.5);
    println!("Opened: {} {} @ {}", entry.side, entry.symbol, entry.price);

    let exit = close_trade(&entry, 43500.0);
    println!("Closed: {} {} @ {}", exit.side, exit.symbol, exit.price);
    println!("PnL: ${:.2}", exit.pnl);
}
```

## Nested Structs

```rust
struct Price {
    value: f64,
    currency: String,
}

struct OrderDetails {
    symbol: String,
    side: String,
}

struct CompleteOrder {
    details: OrderDetails,
    price: Price,
    quantity: f64,
}

fn main() {
    // Create nested structures
    let order = CompleteOrder {
        details: OrderDetails {
            symbol: String::from("ETH/USDT"),
            side: String::from("Buy"),
        },
        price: Price {
            value: 2000.0,
            currency: String::from("USDT"),
        },
        quantity: 2.5,
    };

    println!("Order: {} {} x {} @ {} {}",
        order.details.side,
        order.details.symbol,
        order.quantity,
        order.price.value,
        order.price.currency
    );
}
```

## Array of Struct Instances

```rust
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

fn main() {
    // Array of candles
    let candles = [
        Candle { open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0 },
        Candle { open: 42300.0, high: 42800.0, low: 42100.0, close: 42600.0 },
        Candle { open: 42600.0, high: 43000.0, low: 42400.0, close: 42900.0 },
    ];

    println!("Price history:");
    for (i, candle) in candles.iter().enumerate() {
        let change = candle.close - candle.open;
        let emoji = if change >= 0.0 { "📈" } else { "📉" };
        println!("  Candle {}: O={} H={} L={} C={} {}",
            i + 1, candle.open, candle.high, candle.low, candle.close, emoji);
    }
}
```

## Instance Creation Patterns

```rust
struct RiskParams {
    max_position_size: f64,
    max_loss_per_trade: f64,
    daily_loss_limit: f64,
}

// Pattern 1: Create with default values via function
fn default_risk_params() -> RiskParams {
    RiskParams {
        max_position_size: 1000.0,
        max_loss_per_trade: 50.0,
        daily_loss_limit: 200.0,
    }
}

// Pattern 2: Create aggressive parameters
fn aggressive_risk_params() -> RiskParams {
    RiskParams {
        max_position_size: 5000.0,
        max_loss_per_trade: 200.0,
        daily_loss_limit: 1000.0,
    }
}

// Pattern 3: Create conservative parameters
fn conservative_risk_params() -> RiskParams {
    RiskParams {
        max_position_size: 500.0,
        max_loss_per_trade: 25.0,
        daily_loss_limit: 100.0,
    }
}

fn main() {
    let default_risk = default_risk_params();
    let aggressive_risk = aggressive_risk_params();
    let conservative_risk = conservative_risk_params();

    println!("Standard risk: max position = ${}", default_risk.max_position_size);
    println!("Aggressive risk: max position = ${}", aggressive_risk.max_position_size);
    println!("Conservative risk: max position = ${}", conservative_risk.max_position_size);
}
```

## What We Learned

| Concept | Syntax | Description |
|---------|--------|-------------|
| Basic creation | `Struct { field: value }` | Specify all fields |
| Shorthand form | `Struct { field }` | When variable name = field name |
| Update syntax | `Struct { field: val, ..other }` | Take remaining fields from another instance |
| Nested structs | `Struct { inner: Inner { } }` | Struct inside struct |
| Struct array | `[Struct { }, Struct { }]` | Collection of instances |

## Homework

1. Create a `Portfolio` struct with fields: `name`, `balance`, `positions_count`. Create three different portfolios

2. Implement a `TradeSignal` struct with fields: `symbol`, `action` (Buy/Sell), `confidence` (0.0-1.0), `timestamp`. Create an array of 5 signals

3. Create an `ExchangeConfig` struct and use struct update syntax to create configurations for different exchanges with shared base settings

4. Write a function `create_bracket_orders(symbol, entry_price, stop_loss, take_profit, quantity)` that returns a tuple of three orders: entry, stop-loss, and take-profit

## Navigation

[← Previous day](../061-struct-fields-price-volume-direction/en.md) | [Next day →](../063-methods-order-execute/en.md)
