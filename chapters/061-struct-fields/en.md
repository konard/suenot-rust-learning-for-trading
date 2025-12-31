# Day 61: Struct Fields — Price, Volume, Direction

## Trading Analogy

Every trade on an exchange contains several key parameters:
- **Price** — at what price the trade was executed
- **Volume** — how much of the asset was bought/sold
- **Direction** — whether it's a Buy or Sell

These data points always go together and describe a single entity — a trade. In Rust, we use **structs** to group related data with named fields.

## What is a Struct?

A struct is a custom data type that groups related values under named fields:

```rust
struct Trade {
    price: f64,
    volume: f64,
    is_buy: bool,
}

fn main() {
    let trade = Trade {
        price: 42000.0,
        volume: 0.5,
        is_buy: true,
    };

    println!("Price: {}", trade.price);
    println!("Volume: {}", trade.volume);
    println!("Direction: {}", if trade.is_buy { "Buy" } else { "Sell" });
}
```

## Advantages of Structs Over Tuples

Let's compare tuples and structs:

```rust
fn main() {
    // Tuple — unclear what each value represents
    let trade_tuple: (f64, f64, bool) = (42000.0, 0.5, true);
    println!("Price: {}", trade_tuple.0);  // What is .0?

    // Struct — clear from field names
    let trade_struct = Trade {
        price: 42000.0,
        volume: 0.5,
        is_buy: true,
    };
    println!("Price: {}", trade_struct.price);  // Obvious!
}

struct Trade {
    price: f64,
    volume: f64,
    is_buy: bool,
}
```

## Defining Structs

### Basic Struct

```rust
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    order_type: String,
}

fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 0.5,
        side: String::from("buy"),
        order_type: String::from("limit"),
    };

    println!("Order: {} {} {} @ {}",
        order.side, order.quantity, order.symbol, order.price);
}
```

### Struct with Various Field Types

```rust
struct Candle {
    symbol: String,
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() {
    let candle = Candle {
        symbol: String::from("ETH/USDT"),
        timestamp: 1703980800,
        open: 2200.0,
        high: 2250.0,
        low: 2180.0,
        close: 2230.0,
        volume: 1500.5,
    };

    let is_bullish = candle.close > candle.open;
    let body = (candle.close - candle.open).abs();
    let range = candle.high - candle.low;

    println!("Symbol: {}", candle.symbol);
    println!("Type: {}", if is_bullish { "Bullish" } else { "Bearish" });
    println!("Body: {:.2}, Range: {:.2}", body, range);
}
```

## Accessing Fields

### Reading Fields

```rust
struct Position {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    is_long: bool,
}

fn main() {
    let position = Position {
        symbol: String::from("BTC/USDT"),
        entry_price: 42000.0,
        quantity: 0.5,
        is_long: true,
    };

    // Access individual fields
    println!("Symbol: {}", position.symbol);
    println!("Entry: ${}", position.entry_price);
    println!("Size: {}", position.quantity);
    println!("Direction: {}", if position.is_long { "Long" } else { "Short" });

    // Use fields in calculations
    let notional_value = position.entry_price * position.quantity;
    println!("Notional Value: ${:.2}", notional_value);
}
```

### Modifying Fields (mut)

```rust
struct Portfolio {
    balance: f64,
    total_pnl: f64,
    trade_count: u32,
}

fn main() {
    let mut portfolio = Portfolio {
        balance: 10000.0,
        total_pnl: 0.0,
        trade_count: 0,
    };

    println!("Initial balance: ${}", portfolio.balance);

    // Simulate a profitable trade
    let trade_pnl = 150.0;
    portfolio.balance += trade_pnl;
    portfolio.total_pnl += trade_pnl;
    portfolio.trade_count += 1;

    println!("After trade: ${}", portfolio.balance);
    println!("Total PnL: ${}", portfolio.total_pnl);
    println!("Trades: {}", portfolio.trade_count);

    // Another trade (losing)
    let trade_pnl = -50.0;
    portfolio.balance += trade_pnl;
    portfolio.total_pnl += trade_pnl;
    portfolio.trade_count += 1;

    println!("\nFinal state:");
    println!("Balance: ${}", portfolio.balance);
    println!("Total PnL: ${}", portfolio.total_pnl);
    println!("Trades: {}", portfolio.trade_count);
}
```

## Field Init Shorthand

When a variable has the same name as a field:

```rust
struct Trade {
    symbol: String,
    price: f64,
    volume: f64,
}

fn main() {
    let symbol = String::from("BTC/USDT");
    let price = 42000.0;
    let volume = 0.5;

    // Full form
    let trade1 = Trade {
        symbol: symbol.clone(),
        price: price,
        volume: volume,
    };

    // Shorthand form (field init shorthand)
    let symbol = String::from("ETH/USDT");
    let price = 2200.0;
    let volume = 1.0;

    let trade2 = Trade {
        symbol,  // Shorthand for symbol: symbol
        price,   // Shorthand for price: price
        volume,  // Shorthand for volume: volume
    };

    println!("Trade 1: {} @ {}", trade1.symbol, trade1.price);
    println!("Trade 2: {} @ {}", trade2.symbol, trade2.price);
}
```

## Struct Update Syntax

Creating a new struct based on an existing one:

```rust
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

fn main() {
    let original_order = Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 0.5,
        side: String::from("buy"),
    };

    // Change only the price, copy the rest
    let modified_order = Order {
        price: 41500.0,
        ..original_order
    };

    // Note: original_order.symbol was moved!
    println!("Modified order: {} @ {}", modified_order.symbol, modified_order.price);
}
```

## Practical Example: Trading System

```rust
struct TradeSignal {
    symbol: String,
    direction: String,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
    position_size: f64,
}

fn main() {
    let signal = TradeSignal {
        symbol: String::from("BTC/USDT"),
        direction: String::from("long"),
        entry_price: 42000.0,
        stop_loss: 41000.0,
        take_profit: 44000.0,
        position_size: 0.5,
    };

    // Risk management calculations
    let risk_amount = (signal.entry_price - signal.stop_loss).abs() * signal.position_size;
    let reward_amount = (signal.take_profit - signal.entry_price).abs() * signal.position_size;
    let risk_reward_ratio = reward_amount / risk_amount;

    println!("╔══════════════════════════════════════╗");
    println!("║         TRADE SIGNAL                 ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ Symbol:      {}               ║", signal.symbol);
    println!("║ Direction:   {}                    ║", signal.direction);
    println!("║ Entry:       ${:.2}             ║", signal.entry_price);
    println!("║ Stop Loss:   ${:.2}             ║", signal.stop_loss);
    println!("║ Take Profit: ${:.2}             ║", signal.take_profit);
    println!("║ Size:        {} BTC                 ║", signal.position_size);
    println!("╠══════════════════════════════════════╣");
    println!("║ Risk:        ${:.2}               ║", risk_amount);
    println!("║ Reward:      ${:.2}              ║", reward_amount);
    println!("║ R:R Ratio:   {:.2}                   ║", risk_reward_ratio);
    println!("╚══════════════════════════════════════╝");
}
```

## Practical Example: Order Book Analysis

```rust
struct OrderBookLevel {
    price: f64,
    volume: f64,
    order_count: u32,
}

fn main() {
    // Top order book levels
    let best_bid = OrderBookLevel {
        price: 42000.0,
        volume: 5.5,
        order_count: 12,
    };

    let best_ask = OrderBookLevel {
        price: 42010.0,
        volume: 3.2,
        order_count: 8,
    };

    // Analysis
    let spread = best_ask.price - best_bid.price;
    let spread_percent = (spread / best_bid.price) * 100.0;
    let mid_price = (best_bid.price + best_ask.price) / 2.0;
    let imbalance = best_bid.volume / (best_bid.volume + best_ask.volume);

    println!("=== Order Book Analysis ===\n");

    println!("Best Bid: ${:.2} x {:.4} ({} orders)",
        best_bid.price, best_bid.volume, best_bid.order_count);
    println!("Best Ask: ${:.2} x {:.4} ({} orders)",
        best_ask.price, best_ask.volume, best_ask.order_count);

    println!("\nMetrics:");
    println!("  Spread:     ${:.2} ({:.4}%)", spread, spread_percent);
    println!("  Mid Price:  ${:.2}", mid_price);
    println!("  Imbalance:  {:.1}% bid-heavy", imbalance * 100.0);
}
```

## Practical Example: Portfolio Tracking

```rust
struct Asset {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    current_price: f64,
}

fn main() {
    let mut assets = [
        Asset {
            symbol: String::from("BTC"),
            quantity: 0.5,
            avg_price: 40000.0,
            current_price: 42000.0,
        },
        Asset {
            symbol: String::from("ETH"),
            quantity: 5.0,
            avg_price: 2000.0,
            current_price: 2200.0,
        },
        Asset {
            symbol: String::from("SOL"),
            quantity: 50.0,
            avg_price: 80.0,
            current_price: 95.0,
        },
    ];

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║                    PORTFOLIO TRACKER                    ║");
    println!("╠════════╦══════════╦══════════╦══════════╦══════════════╣");
    println!("║ Asset  ║ Quantity ║ Avg Cost ║ Current  ║ PnL          ║");
    println!("╠════════╬══════════╬══════════╬══════════╬══════════════╣");

    let mut total_cost = 0.0;
    let mut total_value = 0.0;

    for asset in &assets {
        let cost = asset.quantity * asset.avg_price;
        let value = asset.quantity * asset.current_price;
        let pnl = value - cost;
        let pnl_percent = (pnl / cost) * 100.0;

        total_cost += cost;
        total_value += value;

        println!("║ {:6} ║ {:8.4} ║ {:>8.2} ║ {:>8.2} ║ {:+8.2} ({:+.1}%) ║",
            asset.symbol, asset.quantity, asset.avg_price,
            asset.current_price, pnl, pnl_percent);
    }

    let total_pnl = total_value - total_cost;
    let total_pnl_percent = (total_pnl / total_cost) * 100.0;

    println!("╠════════╩══════════╩══════════╩══════════╩══════════════╣");
    println!("║ Total Cost:  ${:>10.2}                              ║", total_cost);
    println!("║ Total Value: ${:>10.2}                              ║", total_value);
    println!("║ Total PnL:   ${:>+10.2} ({:+.2}%)                    ║", total_pnl, total_pnl_percent);
    println!("╚════════════════════════════════════════════════════════╝");
}
```

## Debug Output for Structs

For debugging, use the `#[derive(Debug)]` attribute:

```rust
#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    volume: f64,
    is_buy: bool,
}

fn main() {
    let trade = Trade {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        volume: 0.5,
        is_buy: true,
    };

    // Regular debug output
    println!("{:?}", trade);

    // Pretty debug output
    println!("{:#?}", trade);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `struct Name { field: Type }` | Defining a struct |
| `instance.field` | Accessing a field |
| `mut` | Mutable struct |
| `field,` | Field init shorthand |
| `..other` | Struct update syntax |
| `#[derive(Debug)]` | Debug output for struct |

## Homework

1. Create a `MarketData` struct with fields: symbol, bid, ask, last_price, volume_24h. Write code to calculate spread and mid-price.

2. Create a `RiskMetrics` struct with fields: position_size, entry_price, stop_loss, take_profit, account_balance. Calculate risk as a percentage of account balance.

3. Create an array of several `Trade` structs and calculate total volume and volume-weighted average price.

4. Implement a `TradingStrategy` struct with strategy settings fields (take_profit_percent, stop_loss_percent, max_position_size). Use it to generate a trading signal.

## Navigation

[← Previous day](../060-structs-intro/en.md) | [Next day →](../062-struct-methods/en.md)
