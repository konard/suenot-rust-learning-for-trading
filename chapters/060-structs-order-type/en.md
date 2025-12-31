# Day 60: Structs — Creating the Order Type

## Trading Analogy

When you place an order on an exchange, you specify many parameters: symbol, direction (buy/sell), quantity, price, order type. All this data is interconnected — it's **one order**. Structs in Rust allow you to combine related data into a single type.

Think of a trading terminal: each order is not just a collection of scattered numbers and strings, but a **complete object** with all its characteristics.

## What is a Struct?

A struct (`struct`) is a custom data type that groups related values under one name.

```rust
fn main() {
    // Without struct — scattered variables
    let symbol = "BTC/USDT";
    let side = "buy";
    let quantity = 0.5;
    let price = 42000.0;

    // With struct — everything in one place
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.5,
        price: 42000.0,
    };

    println!("Order: {} {} {} @ {}", order.side, order.quantity, order.symbol, order.price);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}
```

## Defining a Struct

```rust
struct Order {
    symbol: String,      // Trading pair
    side: String,        // "buy" or "sell"
    quantity: f64,       // Amount
    price: f64,          // Price
    order_type: String,  // "limit", "market", "stop"
    timestamp: u64,      // Creation time (Unix timestamp)
}
```

**Important:**
- Struct names use `PascalCase` (each word capitalized)
- Fields are separated by commas
- Trailing comma after the last field is optional but recommended

## Creating a Struct Instance

```rust
fn main() {
    let order = Order {
        symbol: String::from("ETH/USDT"),
        side: String::from("buy"),
        quantity: 2.5,
        price: 2500.0,
        order_type: String::from("limit"),
        timestamp: 1703980800,
    };

    println!("Created order for {}", order.symbol);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    order_type: String,
    timestamp: u64,
}
```

## Accessing Fields

```rust
fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.1,
        price: 42000.0,
        order_type: String::from("limit"),
        timestamp: 1703980800,
    };

    // Access via dot notation
    println!("Symbol: {}", order.symbol);
    println!("Side: {}", order.side);
    println!("Quantity: {}", order.quantity);
    println!("Price: ${}", order.price);

    // Calculate order value
    let order_value = order.quantity * order.price;
    println!("Order value: ${:.2}", order_value);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    order_type: String,
    timestamp: u64,
}
```

## Mutable Structs

```rust
fn main() {
    // To modify fields, the struct must be mut
    let mut order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.1,
        price: 42000.0,
        order_type: String::from("limit"),
        timestamp: 1703980800,
    };

    println!("Original price: ${}", order.price);

    // Modify price
    order.price = 41500.0;
    println!("Updated price: ${}", order.price);

    // Modify quantity
    order.quantity = 0.2;
    println!("Updated quantity: {}", order.quantity);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    order_type: String,
    timestamp: u64,
}
```

**Important:** In Rust, you can't make only individual fields mutable — the entire struct is either mutable or immutable.

## Field Init Shorthand

When a variable has the same name as a struct field:

```rust
fn main() {
    let symbol = String::from("SOL/USDT");
    let side = String::from("sell");
    let quantity = 10.0;
    let price = 100.0;

    // Full form
    let order1 = Order {
        symbol: symbol.clone(),
        side: side.clone(),
        quantity: quantity,
        price: price,
    };

    // Shorthand form (field init shorthand)
    let order2 = Order {
        symbol,  // Equivalent to symbol: symbol
        side,    // Equivalent to side: side
        quantity,
        price,
    };

    println!("Order 1: {} {}", order1.symbol, order1.quantity);
    println!("Order 2: {} {}", order2.symbol, order2.quantity);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}
```

## Struct Update Syntax

Creating a new struct based on an existing one:

```rust
fn main() {
    let order1 = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.5,
        price: 42000.0,
        order_type: String::from("limit"),
    };

    // Create a similar order, changing only quantity
    let order2 = Order {
        quantity: 1.0,          // New value
        ..order1                // Rest from order1
    };

    // Note: order1 is no longer accessible because String was moved!
    println!("Order 2: {} @ {}", order2.quantity, order2.price);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    order_type: String,
}
```

## Tuple Structs

Structs without named fields:

```rust
fn main() {
    // Tuple struct for bid/ask
    let spread = BidAsk(42000.0, 42050.0);

    println!("Bid: {}, Ask: {}", spread.0, spread.1);
    println!("Spread: ${:.2}", spread.1 - spread.0);

    // Tuple struct for OHLCV
    let candle = OHLCV(42000.0, 42500.0, 41800.0, 42300.0, 1500.0);

    println!("Open: {}, High: {}, Low: {}, Close: {}, Volume: {}",
             candle.0, candle.1, candle.2, candle.3, candle.4);
}

struct BidAsk(f64, f64);
struct OHLCV(f64, f64, f64, f64, f64);
```

## Unit-like Structs

Structs without fields (marker types):

```rust
fn main() {
    let _connected = Connected;
    let _disconnected = Disconnected;

    println!("Connection states defined");
}

struct Connected;
struct Disconnected;
```

## Practical Example: Trading System

```rust
fn main() {
    // Create several orders
    let orders = vec![
        Order {
            id: 1,
            symbol: String::from("BTC/USDT"),
            side: String::from("buy"),
            quantity: 0.5,
            price: 42000.0,
            status: String::from("filled"),
        },
        Order {
            id: 2,
            symbol: String::from("ETH/USDT"),
            side: String::from("sell"),
            quantity: 5.0,
            price: 2500.0,
            status: String::from("pending"),
        },
        Order {
            id: 3,
            symbol: String::from("BTC/USDT"),
            side: String::from("sell"),
            quantity: 0.3,
            price: 43000.0,
            status: String::from("filled"),
        },
    ];

    print_order_book(&orders);

    let stats = calculate_portfolio_stats(&orders);
    print_portfolio_stats(&stats);
}

struct Order {
    id: u64,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
}

struct PortfolioStats {
    total_orders: usize,
    filled_orders: usize,
    pending_orders: usize,
    total_buy_value: f64,
    total_sell_value: f64,
}

fn print_order_book(orders: &[Order]) {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                      ORDER BOOK                           ║");
    println!("╠═════╦════════════╦══════╦══════════╦════════════╦═════════╣");
    println!("║ ID  ║   Symbol   ║ Side ║ Quantity ║    Price   ║ Status  ║");
    println!("╠═════╬════════════╬══════╬══════════╬════════════╬═════════╣");

    for order in orders {
        println!("║ {:>3} ║ {:^10} ║ {:^4} ║ {:>8.4} ║ ${:>9.2} ║ {:^7} ║",
                 order.id,
                 order.symbol,
                 order.side,
                 order.quantity,
                 order.price,
                 order.status);
    }

    println!("╚═════╩════════════╩══════╩══════════╩════════════╩═════════╝");
}

fn calculate_portfolio_stats(orders: &[Order]) -> PortfolioStats {
    let total_orders = orders.len();
    let filled_orders = orders.iter().filter(|o| o.status == "filled").count();
    let pending_orders = orders.iter().filter(|o| o.status == "pending").count();

    let total_buy_value: f64 = orders
        .iter()
        .filter(|o| o.side == "buy" && o.status == "filled")
        .map(|o| o.quantity * o.price)
        .sum();

    let total_sell_value: f64 = orders
        .iter()
        .filter(|o| o.side == "sell" && o.status == "filled")
        .map(|o| o.quantity * o.price)
        .sum();

    PortfolioStats {
        total_orders,
        filled_orders,
        pending_orders,
        total_buy_value,
        total_sell_value,
    }
}

fn print_portfolio_stats(stats: &PortfolioStats) {
    println!("\n╔═══════════════════════════════════════╗");
    println!("║          PORTFOLIO STATISTICS         ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Total Orders:      {:>18} ║", stats.total_orders);
    println!("║ Filled Orders:     {:>18} ║", stats.filled_orders);
    println!("║ Pending Orders:    {:>18} ║", stats.pending_orders);
    println!("║ Total Buy Value:   ${:>16.2} ║", stats.total_buy_value);
    println!("║ Total Sell Value:  ${:>16.2} ║", stats.total_sell_value);
    println!("║ Net Position:      ${:>16.2} ║", stats.total_sell_value - stats.total_buy_value);
    println!("╚═══════════════════════════════════════╝");
}
```

## Nested Structs

```rust
fn main() {
    let trade = Trade {
        id: 12345,
        order: Order {
            symbol: String::from("BTC/USDT"),
            side: String::from("buy"),
            quantity: 0.5,
            price: 42000.0,
        },
        execution: Execution {
            filled_quantity: 0.5,
            average_price: 41950.0,
            fee: 10.49,
            timestamp: 1703980800,
        },
    };

    println!("Trade #{}", trade.id);
    println!("Symbol: {}", trade.order.symbol);
    println!("Requested: {} @ ${}", trade.order.quantity, trade.order.price);
    println!("Executed: {} @ ${:.2}", trade.execution.filled_quantity, trade.execution.average_price);
    println!("Fee: ${:.2}", trade.execution.fee);

    // Calculate PnL
    let cost = trade.order.quantity * trade.order.price;
    let value = trade.execution.filled_quantity * trade.execution.average_price;
    let pnl = value - cost - trade.execution.fee;
    println!("Slippage PnL: ${:.2}", pnl);
}

struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

struct Execution {
    filled_quantity: f64,
    average_price: f64,
    fee: f64,
    timestamp: u64,
}

struct Trade {
    id: u64,
    order: Order,
    execution: Execution,
}
```

## Debugging Structs with #[derive(Debug)]

```rust
fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        quantity: 0.5,
        price: 42000.0,
    };

    // Compact output
    println!("Order: {:?}", order);

    // Pretty output
    println!("Order: {:#?}", order);
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}
```

Output:
```
Order: Order { symbol: "BTC/USDT", side: "buy", quantity: 0.5, price: 42000.0 }
Order: Order {
    symbol: "BTC/USDT",
    side: "buy",
    quantity: 0.5,
    price: 42000.0,
}
```

## What We Learned

| Concept | Description | Example |
|---------|-------------|---------|
| Definition | Grouping related data | `struct Order { ... }` |
| Creation | Initializing an instance | `Order { symbol: ... }` |
| Access | Reading fields via dot | `order.price` |
| Mutation | Requires `mut` | `order.price = 42000.0` |
| Shorthand | Short initialization | `Order { symbol, price }` |
| Update syntax | Copying fields | `Order { price, ..order1 }` |
| Tuple | Without field names | `struct Point(f64, f64)` |
| Debug | Output for debugging | `#[derive(Debug)]` |

## Homework

1. Create a `Candle` struct with fields `open`, `high`, `low`, `close`, `volume`, `timestamp`. Write a function that determines the candle type (bullish/bearish/doji).

2. Create a `Position` struct with fields `symbol`, `side`, `entry_price`, `quantity`, `current_price`. Write a function to calculate unrealized PnL.

3. Create a `TradingBot` struct with nested `Config` and `State` structs. Write a function that updates the bot's state.

4. Using the `Order` struct, create a vector of orders and write functions to:
   - Filter by symbol
   - Calculate total buy and sell volume
   - Find the order with maximum value

## Navigation

[← Previous day](../059-shadowing-strategy-params/en.md) | [Next day →](../061-struct-methods-order-actions/en.md)
