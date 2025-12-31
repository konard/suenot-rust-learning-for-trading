# Day 61: Struct Fields — Price, Volume, Direction

## Trading Analogy

Think of a trading order as a form that a trader fills out:
- **Ticker** — which asset to trade
- **Price** — at what price to buy/sell
- **Volume** — how many units
- **Direction** — buy or sell

Each item on the form is a **struct field**. Fields have names and types, making code clear and reliable.

## What Are Struct Fields?

Fields are named components of a struct. Each field has:
- **Name** — what it's called
- **Type** — what data it holds

```rust
struct Order {
    symbol: String,      // Field symbol of type String
    price: f64,          // Field price of type f64
    quantity: f64,       // Field quantity of type f64
    is_buy: bool,        // Field is_buy of type bool
}

fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 0.5,
        is_buy: true,
    };

    println!("Order: {} {} {} @ {}",
        if order.is_buy { "BUY" } else { "SELL" },
        order.quantity,
        order.symbol,
        order.price
    );
}
```

## Accessing Fields

Use dot notation to access fields:

```rust
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    is_long: bool,
}

fn main() {
    let trade = Trade {
        symbol: String::from("ETH/USDT"),
        entry_price: 2500.0,
        exit_price: 2650.0,
        quantity: 2.0,
        is_long: true,
    };

    // Access individual fields
    println!("Symbol: {}", trade.symbol);
    println!("Entry: ${}", trade.entry_price);
    println!("Exit: ${}", trade.exit_price);
    println!("Size: {}", trade.quantity);

    // Calculate PnL based on fields
    let price_diff = if trade.is_long {
        trade.exit_price - trade.entry_price
    } else {
        trade.entry_price - trade.exit_price
    };

    let pnl = price_diff * trade.quantity;
    println!("PnL: ${:.2}", pnl);
}
```

## Mutable Fields

To modify fields, the struct must be declared as `mut`:

```rust
struct Position {
    symbol: String,
    size: f64,
    entry_price: f64,
    unrealized_pnl: f64,
}

fn main() {
    let mut position = Position {
        symbol: String::from("BTC/USDT"),
        size: 1.0,
        entry_price: 42000.0,
        unrealized_pnl: 0.0,
    };

    println!("Initial position: {} BTC @ ${}", position.size, position.entry_price);

    // Price changed — update PnL
    let current_price = 43500.0;
    position.unrealized_pnl = (current_price - position.entry_price) * position.size;

    println!("Current price: ${}", current_price);
    println!("Unrealized PnL: ${:.2}", position.unrealized_pnl);

    // Add to position (averaging)
    let add_size = 0.5;
    let add_price = 43000.0;

    let total_cost = position.entry_price * position.size + add_price * add_size;
    position.size += add_size;
    position.entry_price = total_cost / position.size;

    println!("\nAfter averaging:");
    println!("New size: {} BTC", position.size);
    println!("Average entry: ${:.2}", position.entry_price);
}
```

## Field Types

Fields can be any type, including other structs:

```rust
struct Price {
    bid: f64,
    ask: f64,
}

struct Volume {
    bid_size: f64,
    ask_size: f64,
}

struct OrderBookLevel {
    price: Price,       // Nested struct
    volume: Volume,     // Nested struct
    timestamp: u64,
}

fn main() {
    let top_of_book = OrderBookLevel {
        price: Price {
            bid: 42000.0,
            ask: 42010.0,
        },
        volume: Volume {
            bid_size: 2.5,
            ask_size: 1.8,
        },
        timestamp: 1703980800,
    };

    // Access nested fields
    let spread = top_of_book.price.ask - top_of_book.price.bid;
    let total_liquidity = top_of_book.volume.bid_size + top_of_book.volume.ask_size;

    println!("Best Bid: ${} x {:.2}",
        top_of_book.price.bid,
        top_of_book.volume.bid_size
    );
    println!("Best Ask: ${} x {:.2}",
        top_of_book.price.ask,
        top_of_book.volume.ask_size
    );
    println!("Spread: ${:.2}", spread);
    println!("Total liquidity: {:.2} BTC", total_liquidity);
}
```

## Practical Example: Candle Analysis

```rust
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    timestamp: u64,
}

fn main() {
    let candle = Candle {
        open: 42000.0,
        high: 42800.0,
        low: 41500.0,
        close: 42600.0,
        volume: 1250.5,
        timestamp: 1703980800,
    };

    // Analyze candle via fields
    let is_bullish = candle.close > candle.open;
    let body = (candle.close - candle.open).abs();
    let range = candle.high - candle.low;
    let upper_shadow = candle.high - candle.close.max(candle.open);
    let lower_shadow = candle.close.min(candle.open) - candle.low;

    println!("╔═══════════════════════════════════╗");
    println!("║       CANDLE ANALYSIS             ║");
    println!("╠═══════════════════════════════════╣");
    println!("║ Open:   ${:.2}               ║", candle.open);
    println!("║ High:   ${:.2}               ║", candle.high);
    println!("║ Low:    ${:.2}               ║", candle.low);
    println!("║ Close:  ${:.2}               ║", candle.close);
    println!("║ Volume: {:.2} BTC             ║", candle.volume);
    println!("╠═══════════════════════════════════╣");
    println!("║ Type: {}                   ║",
        if is_bullish { "BULLISH" } else { "BEARISH" });
    println!("║ Body:  ${:.2}                 ║", body);
    println!("║ Range: ${:.2}                ║", range);
    println!("║ Upper Shadow: ${:.2}          ║", upper_shadow);
    println!("║ Lower Shadow: ${:.2}          ║", lower_shadow);
    println!("╚═══════════════════════════════════╝");
}
```

## Practical Example: Order Management

```rust
struct LimitOrder {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    filled: f64,
    status: String,
}

fn main() {
    let mut order = LimitOrder {
        id: 12345,
        symbol: String::from("BTC/USDT"),
        side: String::from("BUY"),
        price: 42000.0,
        quantity: 1.0,
        filled: 0.0,
        status: String::from("NEW"),
    };

    println!("=== Order #{} ===", order.id);
    println!("{} {} {} @ ${}",
        order.side, order.quantity, order.symbol, order.price);
    println!("Status: {}", order.status);

    // Partial fill
    let fill_qty = 0.3;
    order.filled += fill_qty;
    order.status = String::from("PARTIALLY_FILLED");

    println!("\n--- Partial fill: {} ---", fill_qty);
    println!("Filled: {}/{}", order.filled, order.quantity);
    println!("Remaining: {}", order.quantity - order.filled);
    println!("Status: {}", order.status);

    // Complete fill
    order.filled = order.quantity;
    order.status = String::from("FILLED");

    println!("\n--- Order complete ---");
    println!("Status: {}", order.status);
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
    let assets = [
        Asset {
            symbol: String::from("BTC"),
            quantity: 1.5,
            avg_price: 40000.0,
            current_price: 43000.0,
        },
        Asset {
            symbol: String::from("ETH"),
            quantity: 10.0,
            avg_price: 2200.0,
            current_price: 2500.0,
        },
        Asset {
            symbol: String::from("SOL"),
            quantity: 50.0,
            avg_price: 80.0,
            current_price: 95.0,
        },
    ];

    println!("{:<8} {:>10} {:>12} {:>12} {:>12}",
        "Asset", "Quantity", "Avg Price", "Current", "PnL");
    println!("{}", "-".repeat(58));

    let mut total_value = 0.0;
    let mut total_cost = 0.0;

    for asset in &assets {
        let cost = asset.quantity * asset.avg_price;
        let value = asset.quantity * asset.current_price;
        let pnl = value - cost;

        total_cost += cost;
        total_value += value;

        println!("{:<8} {:>10.2} {:>12.2} {:>12.2} {:>+12.2}",
            asset.symbol,
            asset.quantity,
            asset.avg_price,
            asset.current_price,
            pnl
        );
    }

    println!("{}", "-".repeat(58));
    println!("{:<8} {:>10} {:>12.2} {:>12.2} {:>+12.2}",
        "TOTAL", "", total_cost, total_value, total_value - total_cost);
}
```

## Practical Example: Risk Management

```rust
struct RiskParams {
    max_position_size: f64,
    max_loss_percent: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
}

struct TradeSetup {
    symbol: String,
    entry_price: f64,
    position_size: f64,
    direction: String,
}

fn main() {
    let risk = RiskParams {
        max_position_size: 10000.0,   // Max position size in USD
        max_loss_percent: 2.0,         // Max loss 2% of capital
        stop_loss_percent: 1.5,        // Stop loss 1.5% from entry
        take_profit_percent: 3.0,      // Take profit 3% from entry
    };

    let setup = TradeSetup {
        symbol: String::from("BTC/USDT"),
        entry_price: 42000.0,
        position_size: 5000.0,
        direction: String::from("LONG"),
    };

    // Calculate levels based on fields
    let stop_loss = if setup.direction == "LONG" {
        setup.entry_price * (1.0 - risk.stop_loss_percent / 100.0)
    } else {
        setup.entry_price * (1.0 + risk.stop_loss_percent / 100.0)
    };

    let take_profit = if setup.direction == "LONG" {
        setup.entry_price * (1.0 + risk.take_profit_percent / 100.0)
    } else {
        setup.entry_price * (1.0 - risk.take_profit_percent / 100.0)
    };

    let potential_loss = (setup.entry_price - stop_loss).abs()
        * (setup.position_size / setup.entry_price);
    let potential_profit = (take_profit - setup.entry_price).abs()
        * (setup.position_size / setup.entry_price);
    let risk_reward = potential_profit / potential_loss;

    println!("=== Trade Setup: {} ===", setup.symbol);
    println!("Direction: {}", setup.direction);
    println!("Entry: ${:.2}", setup.entry_price);
    println!("Position: ${:.2}", setup.position_size);
    println!();
    println!("Stop Loss: ${:.2} ({:.1}%)", stop_loss, risk.stop_loss_percent);
    println!("Take Profit: ${:.2} ({:.1}%)", take_profit, risk.take_profit_percent);
    println!();
    println!("Potential Loss: ${:.2}", potential_loss);
    println!("Potential Profit: ${:.2}", potential_profit);
    println!("Risk/Reward: 1:{:.2}", risk_reward);

    // Risk check
    let size_ok = setup.position_size <= risk.max_position_size;
    println!("\nRisk Check:");
    println!("  Position size OK: {} (${:.0} <= ${:.0})",
        size_ok, setup.position_size, risk.max_position_size);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `struct.field` | Access a struct field |
| `mut struct` | Mutable struct |
| Nested fields | Structs inside structs |
| Named fields | Clear and readable code |

## Homework

1. Create a `Ticker` struct with fields: symbol, last_price, change_24h, volume_24h. Write code to display ticker information.

2. Create a `Wallet` struct with fields: currency, balance, available, locked. Implement logic to lock part of the balance for an order.

3. Create a `TradeStats` struct with fields: total_trades, wins, losses, gross_profit, gross_loss. Calculate win rate and profit factor.

4. Create nested structs to represent a trading pair: `TradingPair` contains `BaseAsset` and `QuoteAsset`, each having symbol and precision.

## Navigation

[← Previous day](../060-structs-creating-order-type/en.md) | [Next day →](../062-creating-instance-new-order/en.md)
