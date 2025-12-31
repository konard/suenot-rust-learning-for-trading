# Day 71: Enum with Data — OrderType with Parameters

## Trading Analogy

In the previous chapter, we created a simple `OrderSide` enum for trade direction — Buy or Sell. But real order types are more complex:

- **Market order** — just executes at market price, no parameters needed
- **Limit order** — needs a price at which we want to execute
- **Stop order** — needs a trigger price and possibly a limit price
- **Trailing Stop** — needs a percentage or amount offset

Each order type requires **different data**! In Rust, enums can store data inside variants — this is perfect for modeling order types.

## Enum with Data

In Rust, each enum variant can contain data:

```rust
enum OrderType {
    Market,                           // No data
    Limit(f64),                       // Price
    Stop { trigger: f64, limit: Option<f64> },  // Named fields
    TrailingStop { percent: f64 },    // Offset percentage
}

fn main() {
    let order1 = OrderType::Market;
    let order2 = OrderType::Limit(42000.0);
    let order3 = OrderType::Stop {
        trigger: 41000.0,
        limit: Some(40950.0)
    };
    let order4 = OrderType::TrailingStop { percent: 2.5 };

    println!("Created 4 different order types!");
}
```

## Three Forms of Data in Enum

### 1. Unit Variant (no data)

```rust
enum Signal {
    Hold,      // Do nothing
    Exit,      // Close position
}

fn main() {
    let signal = Signal::Hold;

    match signal {
        Signal::Hold => println!("Holding position"),
        Signal::Exit => println!("Closing position"),
    }
}
```

### 2. Tuple Variant (data as tuple)

```rust
enum OrderType {
    Market,
    Limit(f64),                    // One value
    StopLimit(f64, f64),           // Two values: stop, limit
    Iceberg(f64, f64, u32),        // Price, total size, visible size
}

fn main() {
    let limit_order = OrderType::Limit(42000.0);
    let stop_limit = OrderType::StopLimit(41000.0, 40950.0);
    let iceberg = OrderType::Iceberg(42100.0, 10.0, 1);

    match limit_order {
        OrderType::Market => println!("Market order"),
        OrderType::Limit(price) => println!("Limit @ {}", price),
        OrderType::StopLimit(stop, limit) => {
            println!("Stop-Limit: trigger={}, limit={}", stop, limit)
        }
        OrderType::Iceberg(price, total, visible) => {
            println!("Iceberg: price={}, total={}, show={}", price, total, visible)
        }
    }
}
```

### 3. Struct Variant (named fields)

```rust
enum Order {
    Market {
        symbol: String,
        quantity: f64,
    },
    Limit {
        symbol: String,
        quantity: f64,
        price: f64,
    },
    StopLoss {
        symbol: String,
        quantity: f64,
        trigger_price: f64,
        limit_price: Option<f64>,
    },
}

fn main() {
    let order = Order::Limit {
        symbol: String::from("BTC/USDT"),
        quantity: 0.5,
        price: 42000.0,
    };

    match order {
        Order::Market { symbol, quantity } => {
            println!("Market {} {} units", symbol, quantity);
        }
        Order::Limit { symbol, quantity, price } => {
            println!("Limit {} {} @ {}", symbol, quantity, price);
        }
        Order::StopLoss { symbol, quantity, trigger_price, limit_price } => {
            println!("Stop {} {} trigger={}", symbol, quantity, trigger_price);
            if let Some(limit) = limit_price {
                println!("  with limit: {}", limit);
            }
        }
    }
}
```

## Pattern Matching with Data

### Extracting Data with match

```rust
enum TradeResult {
    Profit(f64),
    Loss(f64),
    BreakEven,
}

fn main() {
    let results = vec![
        TradeResult::Profit(150.0),
        TradeResult::Loss(50.0),
        TradeResult::Profit(200.0),
        TradeResult::BreakEven,
        TradeResult::Loss(75.0),
    ];

    let mut total_pnl = 0.0;
    let mut wins = 0;
    let mut losses = 0;

    for result in &results {
        match result {
            TradeResult::Profit(amount) => {
                total_pnl += amount;
                wins += 1;
            }
            TradeResult::Loss(amount) => {
                total_pnl -= amount;
                losses += 1;
            }
            TradeResult::BreakEven => {
                // PnL unchanged
            }
        }
    }

    println!("Total PnL: {:+.2}", total_pnl);
    println!("Wins: {}, Losses: {}", wins, losses);
}
```

### if let for Single Variant

```rust
enum Alert {
    PriceAbove(f64),
    PriceBelow(f64),
    VolumeSpike(f64),
    Custom(String),
}

fn main() {
    let alert = Alert::PriceAbove(45000.0);

    // When we're only interested in one variant
    if let Alert::PriceAbove(target) = alert {
        println!("Alert: price above {}", target);
    }

    // Or with else
    let alert2 = Alert::VolumeSpike(3.5);

    if let Alert::PriceAbove(target) = alert2 {
        println!("Price above {}", target);
    } else {
        println!("This is not a price alert");
    }
}
```

### while let for Iteration

```rust
enum OrderEvent {
    Filled { price: f64, quantity: f64 },
    PartialFill { price: f64, quantity: f64, remaining: f64 },
    Cancelled,
    Rejected(String),
}

fn main() {
    let mut events = vec![
        OrderEvent::PartialFill { price: 42000.0, quantity: 0.3, remaining: 0.2 },
        OrderEvent::Filled { price: 42010.0, quantity: 0.2 },
    ];

    // Process events while there are fills
    while let Some(event) = events.pop() {
        match event {
            OrderEvent::Filled { price, quantity } => {
                println!("Full fill: {} @ {}", quantity, price);
            }
            OrderEvent::PartialFill { price, quantity, remaining } => {
                println!("Partial: {} @ {}, remaining: {}", quantity, price, remaining);
            }
            _ => break, // Other events stop processing
        }
    }
}
```

## Practical Example: Trading Strategy

```rust
#[derive(Debug)]
enum Signal {
    Buy { price: f64, size: f64, reason: String },
    Sell { price: f64, size: f64, reason: String },
    Hold,
}

#[derive(Debug)]
enum OrderType {
    Market,
    Limit(f64),
    StopLimit { stop: f64, limit: f64 },
}

#[derive(Debug)]
struct TradeOrder {
    symbol: String,
    side: String,
    order_type: OrderType,
    quantity: f64,
}

fn generate_signal(price: f64, sma_20: f64, sma_50: f64) -> Signal {
    if sma_20 > sma_50 && price > sma_20 {
        Signal::Buy {
            price,
            size: 0.1,
            reason: String::from("Golden cross + price above SMA20"),
        }
    } else if sma_20 < sma_50 && price < sma_20 {
        Signal::Sell {
            price,
            size: 0.1,
            reason: String::from("Death cross + price below SMA20"),
        }
    } else {
        Signal::Hold
    }
}

fn signal_to_order(signal: Signal, symbol: &str) -> Option<TradeOrder> {
    match signal {
        Signal::Buy { price, size, reason } => {
            println!("Buy signal: {}", reason);
            Some(TradeOrder {
                symbol: symbol.to_string(),
                side: String::from("BUY"),
                order_type: OrderType::Limit(price * 0.999), // Slightly below current
                quantity: size,
            })
        }
        Signal::Sell { price, size, reason } => {
            println!("Sell signal: {}", reason);
            Some(TradeOrder {
                symbol: symbol.to_string(),
                side: String::from("SELL"),
                order_type: OrderType::Limit(price * 1.001), // Slightly above current
                quantity: size,
            })
        }
        Signal::Hold => {
            println!("No signal, holding position");
            None
        }
    }
}

fn main() {
    let current_price = 42500.0;
    let sma_20 = 42000.0;
    let sma_50 = 41500.0;

    let signal = generate_signal(current_price, sma_20, sma_50);
    println!("Signal: {:?}", signal);

    if let Some(order) = signal_to_order(signal, "BTC/USDT") {
        println!("\nOrder created: {:?}", order);

        match &order.order_type {
            OrderType::Market => println!("Type: market"),
            OrderType::Limit(price) => println!("Type: limit @ {:.2}", price),
            OrderType::StopLimit { stop, limit } => {
                println!("Type: stop-limit, trigger={}, limit={}", stop, limit)
            }
        }
    }
}
```

## Practical Example: Portfolio Analysis

```rust
#[derive(Debug)]
enum PositionStatus {
    Open {
        entry_price: f64,
        quantity: f64,
        unrealized_pnl: f64,
    },
    Closed {
        entry_price: f64,
        exit_price: f64,
        quantity: f64,
        realized_pnl: f64,
    },
    Pending {
        target_price: f64,
        quantity: f64,
    },
}

struct Position {
    symbol: String,
    status: PositionStatus,
}

fn analyze_portfolio(positions: &[Position]) {
    let mut total_unrealized = 0.0;
    let mut total_realized = 0.0;
    let mut open_count = 0;
    let mut closed_count = 0;

    println!("=== Portfolio Analysis ===\n");

    for pos in positions {
        print!("{}: ", pos.symbol);

        match &pos.status {
            PositionStatus::Open { entry_price, quantity, unrealized_pnl } => {
                println!("OPEN {} @ {} (PnL: {:+.2})",
                    quantity, entry_price, unrealized_pnl);
                total_unrealized += unrealized_pnl;
                open_count += 1;
            }
            PositionStatus::Closed { entry_price, exit_price, quantity, realized_pnl } => {
                println!("CLOSED {} @ {} -> {} (PnL: {:+.2})",
                    quantity, entry_price, exit_price, realized_pnl);
                total_realized += realized_pnl;
                closed_count += 1;
            }
            PositionStatus::Pending { target_price, quantity } => {
                println!("PENDING {} @ {}", quantity, target_price);
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Open positions: {}", open_count);
    println!("Closed positions: {}", closed_count);
    println!("Unrealized PnL: {:+.2}", total_unrealized);
    println!("Realized PnL: {:+.2}", total_realized);
    println!("Total PnL: {:+.2}", total_unrealized + total_realized);
}

fn main() {
    let portfolio = vec![
        Position {
            symbol: String::from("BTC/USDT"),
            status: PositionStatus::Open {
                entry_price: 42000.0,
                quantity: 0.5,
                unrealized_pnl: 250.0,
            },
        },
        Position {
            symbol: String::from("ETH/USDT"),
            status: PositionStatus::Closed {
                entry_price: 2200.0,
                exit_price: 2350.0,
                quantity: 2.0,
                realized_pnl: 300.0,
            },
        },
        Position {
            symbol: String::from("SOL/USDT"),
            status: PositionStatus::Open {
                entry_price: 100.0,
                quantity: 10.0,
                unrealized_pnl: -50.0,
            },
        },
        Position {
            symbol: String::from("DOGE/USDT"),
            status: PositionStatus::Pending {
                target_price: 0.15,
                quantity: 1000.0,
            },
        },
    ];

    analyze_portfolio(&portfolio);
}
```

## Practical Example: Exchange Event Processing

```rust
#[derive(Debug)]
enum ExchangeEvent {
    // Market data
    Trade {
        symbol: String,
        price: f64,
        quantity: f64,
        is_buyer_maker: bool,
    },
    OrderBookUpdate {
        symbol: String,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    },
    Ticker {
        symbol: String,
        last_price: f64,
        volume_24h: f64,
        change_24h: f64,
    },

    // Account events
    OrderUpdate {
        order_id: u64,
        status: String,
        filled_qty: f64,
    },
    BalanceUpdate {
        asset: String,
        free: f64,
        locked: f64,
    },

    // System
    Heartbeat,
    Error(String),
}

fn process_event(event: ExchangeEvent) {
    match event {
        ExchangeEvent::Trade { symbol, price, quantity, is_buyer_maker } => {
            let side = if is_buyer_maker { "SELL" } else { "BUY" };
            println!("[TRADE] {} {} {} @ {}", symbol, side, quantity, price);
        }

        ExchangeEvent::OrderBookUpdate { symbol, bids, asks } => {
            println!("[BOOK] {} | Best Bid: {:?} | Best Ask: {:?}",
                symbol,
                bids.first(),
                asks.first()
            );
        }

        ExchangeEvent::Ticker { symbol, last_price, volume_24h, change_24h } => {
            let emoji = if change_24h >= 0.0 { "📈" } else { "📉" };
            println!("[TICK] {} {} {:.2} ({:+.2}%) Vol: {:.0}",
                emoji, symbol, last_price, change_24h, volume_24h);
        }

        ExchangeEvent::OrderUpdate { order_id, status, filled_qty } => {
            println!("[ORDER] #{} -> {} (filled: {})", order_id, status, filled_qty);
        }

        ExchangeEvent::BalanceUpdate { asset, free, locked } => {
            println!("[BAL] {} Free: {:.4}, Locked: {:.4}", asset, free, locked);
        }

        ExchangeEvent::Heartbeat => {
            // Silently ignore heartbeat
        }

        ExchangeEvent::Error(msg) => {
            println!("[ERROR] {}", msg);
        }
    }
}

fn main() {
    let events = vec![
        ExchangeEvent::Ticker {
            symbol: String::from("BTC/USDT"),
            last_price: 42500.0,
            volume_24h: 15000.0,
            change_24h: 2.5,
        },
        ExchangeEvent::Trade {
            symbol: String::from("BTC/USDT"),
            price: 42510.0,
            quantity: 0.5,
            is_buyer_maker: false,
        },
        ExchangeEvent::OrderUpdate {
            order_id: 12345,
            status: String::from("FILLED"),
            filled_qty: 0.5,
        },
        ExchangeEvent::BalanceUpdate {
            asset: String::from("BTC"),
            free: 1.5,
            locked: 0.0,
        },
        ExchangeEvent::Heartbeat,
        ExchangeEvent::Error(String::from("Rate limit exceeded")),
    ];

    println!("=== Processing Exchange Events ===\n");
    for event in events {
        process_event(event);
    }
}
```

## Methods for Enum with Data

```rust
#[derive(Debug, Clone)]
enum OrderType {
    Market,
    Limit(f64),
    StopLimit { stop: f64, limit: f64 },
    TrailingStop { percent: f64 },
}

impl OrderType {
    fn description(&self) -> String {
        match self {
            OrderType::Market => String::from("Market: execution at market price"),
            OrderType::Limit(price) => format!("Limit @ {:.2}: execution at specified price", price),
            OrderType::StopLimit { stop, limit } => {
                format!("Stop-Limit: trigger {:.2}, limit {:.2}", stop, limit)
            }
            OrderType::TrailingStop { percent } => {
                format!("Trailing Stop: offset {:.1}%", percent)
            }
        }
    }

    fn is_conditional(&self) -> bool {
        matches!(self, OrderType::StopLimit { .. } | OrderType::TrailingStop { .. })
    }

    fn get_limit_price(&self) -> Option<f64> {
        match self {
            OrderType::Limit(price) => Some(*price),
            OrderType::StopLimit { limit, .. } => Some(*limit),
            _ => None,
        }
    }

    fn with_price(self, new_price: f64) -> Self {
        match self {
            OrderType::Limit(_) => OrderType::Limit(new_price),
            OrderType::StopLimit { stop, .. } => OrderType::StopLimit { stop, limit: new_price },
            other => other,
        }
    }
}

fn main() {
    let orders = vec![
        OrderType::Market,
        OrderType::Limit(42000.0),
        OrderType::StopLimit { stop: 41000.0, limit: 40950.0 },
        OrderType::TrailingStop { percent: 2.0 },
    ];

    for order in &orders {
        println!("{}", order.description());
        println!("  Conditional: {}", order.is_conditional());
        if let Some(price) = order.get_limit_price() {
            println!("  Limit price: {:.2}", price);
        }
        println!();
    }

    // Modify price
    let original = OrderType::Limit(42000.0);
    let updated = original.with_price(42500.0);
    println!("Updated price: {:?} -> {:?}", OrderType::Limit(42000.0), updated);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Variant(T)` | Tuple variant with data |
| `Variant { field: T }` | Struct variant with named fields |
| `match` with extraction | Get data from variant |
| `if let` | Check single variant |
| `matches!` | Check variant without extracting data |

## Homework

1. **Order Types**: Create an `AdvancedOrderType` enum with variants:
   - `Market` (no data)
   - `Limit(price: f64)`
   - `StopLoss { trigger: f64, size: f64 }`
   - `TakeProfit { target: f64, size: f64 }`
   - `OCO { stop_loss: f64, take_profit: f64 }` (One Cancels Other)

   Implement `describe()` and `requires_trigger()` methods.

2. **Wallet Events**: Create a `WalletEvent` enum:
   - `Deposit { asset: String, amount: f64, from: String }`
   - `Withdrawal { asset: String, amount: f64, to: String, fee: f64 }`
   - `Transfer { asset: String, amount: f64, from_wallet: String, to_wallet: String }`
   - `Swap { from_asset: String, from_amount: f64, to_asset: String, to_amount: f64 }`

   Write a function that processes a vector of events and calculates the final balance change for each asset.

3. **Risk Management**: Create a `RiskCheckResult` enum:
   - `Approved`
   - `ApprovedWithWarning(String)`
   - `Rejected { reason: String, max_allowed: f64 }`
   - `RequiresManualApproval { reason: String }`

   Write a function `check_order_risk(order_size: f64, account_balance: f64, max_risk_percent: f64)` that returns the appropriate result.

4. **Strategy with Data**: Create a `StrategyState` enum:
   - `Idle`
   - `WaitingForEntry { target_price: f64 }`
   - `InPosition { entry_price: f64, quantity: f64, stop_loss: f64, take_profit: f64 }`
   - `Exiting { reason: String }`

   Implement state transition functions and current state display.

## Navigation

[← Previous day](../070-enum-order-side/en.md) | [Next day →](../072-option-price-missing/en.md)
