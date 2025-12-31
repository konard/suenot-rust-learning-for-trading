# Day 72: Option — Price Might Be Missing

## Trading Analogy

Imagine: you request an asset's price from an exchange. What will you get in response? Sometimes — a number (the price), and sometimes... nothing. Trading might have been halted, the asset delisted, or there's simply no data for that period.

In real trading, this happens constantly:
- **Closing price** — might not exist if there was no trading that day
- **Last trade** — might be absent for an illiquid asset
- **Stop-loss** — might not be set
- **Portfolio position** — the asset might not be in the portfolio

Rust solves this problem elegantly with the `Option<T>` type.

## What is Option?

`Option` is an enum that can either contain a value (`Some`) or be empty (`None`):

```rust
enum Option<T> {
    Some(T),  // Contains a value of type T
    None,     // No value
}
```

This forces the programmer to **explicitly** handle the case of missing values — no `null` and no unexpected program crashes!

## Basic Usage

```rust
fn main() {
    // Price might exist or might not
    let btc_price: Option<f64> = Some(42000.0);
    let delisted_price: Option<f64> = None;

    // Check if value exists
    if btc_price.is_some() {
        println!("BTC is trading");
    }

    if delisted_price.is_none() {
        println!("Asset is not trading");
    }
}
```

## Extracting Values

### match — the most reliable way

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);

    match price {
        Some(p) => println!("Price: ${:.2}", p),
        None => println!("Price unavailable"),
    }
}
```

### if let — when only one variant matters

```rust
fn main() {
    let stop_loss: Option<f64> = Some(41000.0);

    if let Some(sl) = stop_loss {
        println!("Stop-loss set at ${:.2}", sl);
    }

    let take_profit: Option<f64> = None;

    if let Some(tp) = take_profit {
        println!("Take-profit: ${:.2}", tp);
    } else {
        println!("Take-profit not set");
    }
}
```

### unwrap_or — default value

```rust
fn main() {
    let bid: Option<f64> = Some(42000.0);
    let ask: Option<f64> = None;

    // If None — use default value
    let bid_price = bid.unwrap_or(0.0);
    let ask_price = ask.unwrap_or(0.0);

    println!("Bid: {}, Ask: {}", bid_price, ask_price);
}
```

### unwrap_or_else — lazy default computation

```rust
fn main() {
    let cached_price: Option<f64> = None;

    // Function is called only if None
    let price = cached_price.unwrap_or_else(|| {
        println!("Fetching price from exchange...");
        fetch_price_from_exchange()
    });

    println!("Price: ${:.2}", price);
}

fn fetch_price_from_exchange() -> f64 {
    42500.0  // Simulating exchange request
}
```

## Option in Functions — Price Analysis

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    // Find maximum price
    match find_max_price(&prices) {
        Some(max) => println!("Maximum: ${:.2}", max),
        None => println!("No data for analysis"),
    }

    // Empty array
    let empty: Vec<f64> = vec![];
    match find_max_price(&empty) {
        Some(max) => println!("Maximum: ${:.2}", max),
        None => println!("Array is empty — no maximum"),
    }
}

fn find_max_price(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    let mut max = prices[0];
    for &price in &prices[1..] {
        if price > max {
            max = price;
        }
    }
    Some(max)
}
```

## Option for Order Management

```rust
fn main() {
    // Create order without stop-loss
    let mut order = Order {
        symbol: String::from("BTC/USDT"),
        side: OrderSide::Buy,
        price: 42000.0,
        quantity: 0.5,
        stop_loss: None,
        take_profit: None,
    };

    print_order(&order);

    // Set stop-loss
    order.stop_loss = Some(41000.0);
    order.take_profit = Some(45000.0);

    print_order(&order);

    // Check risk
    if let Some(risk) = calculate_risk(&order) {
        println!("Trade risk: ${:.2}", risk);
    }
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

struct Order {
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

fn print_order(order: &Order) {
    println!("\n=== Order ===");
    println!("Symbol: {}", order.symbol);
    println!("Side: {:?}", order.side);
    println!("Price: ${:.2}", order.price);
    println!("Quantity: {}", order.quantity);

    match order.stop_loss {
        Some(sl) => println!("Stop-Loss: ${:.2}", sl),
        None => println!("Stop-Loss: not set"),
    }

    match order.take_profit {
        Some(tp) => println!("Take-Profit: ${:.2}", tp),
        None => println!("Take-Profit: not set"),
    }
}

fn calculate_risk(order: &Order) -> Option<f64> {
    // Risk can only be calculated if stop-loss exists
    let stop_loss = order.stop_loss?;  // Early return if None

    let risk_per_unit = (order.price - stop_loss).abs();
    Some(risk_per_unit * order.quantity)
}
```

## The ? Operator for Option

The `?` operator allows elegant handling of Option chains:

```rust
fn main() {
    let portfolio = Portfolio {
        positions: vec![
            Position { symbol: String::from("BTC"), quantity: Some(0.5), avg_price: Some(42000.0) },
            Position { symbol: String::from("ETH"), quantity: Some(10.0), avg_price: Some(2200.0) },
            Position { symbol: String::from("DOGE"), quantity: None, avg_price: None },
        ],
    };

    for pos in &portfolio.positions {
        match calculate_position_value(pos) {
            Some(value) => println!("{}: ${:.2}", pos.symbol, value),
            None => println!("{}: no position", pos.symbol),
        }
    }
}

struct Position {
    symbol: String,
    quantity: Option<f64>,
    avg_price: Option<f64>,
}

struct Portfolio {
    positions: Vec<Position>,
}

fn calculate_position_value(position: &Position) -> Option<f64> {
    let qty = position.quantity?;      // Returns None if quantity = None
    let price = position.avg_price?;   // Returns None if avg_price = None
    Some(qty * price)
}
```

## Option Transformation Methods

### map — transform the value

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);

    // Transform price to string
    let formatted: Option<String> = price.map(|p| format!("${:.2}", p));
    println!("{:?}", formatted);  // Some("$42000.00")

    // For None, map returns None
    let no_price: Option<f64> = None;
    let formatted_none: Option<String> = no_price.map(|p| format!("${:.2}", p));
    println!("{:?}", formatted_none);  // None
}
```

### and_then — chain Options

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0];

    // Chain: find maximum -> calculate fee
    let max_fee = find_max_price(&prices)
        .and_then(|max| calculate_fee(max, 0.1));

    match max_fee {
        Some(fee) => println!("Fee on maximum: ${:.2}", fee),
        None => println!("Cannot calculate"),
    }
}

fn find_max_price(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }
    prices.iter().cloned().reduce(f64::max)
}

fn calculate_fee(amount: f64, fee_percent: f64) -> Option<f64> {
    if amount <= 0.0 {
        return None;
    }
    Some(amount * fee_percent / 100.0)
}
```

### filter — conditional filtering

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);

    // Keep only if price is above threshold
    let valid_price = price.filter(|&p| p > 40000.0);
    println!("Valid price: {:?}", valid_price);  // Some(42000.0)

    let low_price: Option<f64> = Some(35000.0);
    let filtered = low_price.filter(|&p| p > 40000.0);
    println!("Filtered: {:?}", filtered);  // None
}
```

## Practical Example: Risk Management System

```rust
fn main() {
    let trade_params = TradeParams {
        entry_price: Some(42000.0),
        stop_loss: Some(41000.0),
        take_profit: Some(45000.0),
        position_size: Some(0.5),
        account_balance: Some(10000.0),
    };

    match analyze_trade_risk(&trade_params) {
        Some(analysis) => {
            println!("╔════════════════════════════════════╗");
            println!("║       TRADE RISK ANALYSIS          ║");
            println!("╠════════════════════════════════════╣");
            println!("║ Risk Amount:       ${:>13.2} ║", analysis.risk_amount);
            println!("║ Risk of Balance:   {:>13.2}% ║", analysis.risk_percent);
            println!("║ Potential Profit:  ${:>13.2} ║", analysis.potential_profit);
            println!("║ Risk/Reward:       {:>13.2}  ║", analysis.risk_reward_ratio);
            println!("║ Recommendation:    {:>13}  ║", analysis.recommendation);
            println!("╚════════════════════════════════════╝");
        }
        None => println!("Insufficient data for risk analysis"),
    }
}

struct TradeParams {
    entry_price: Option<f64>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    position_size: Option<f64>,
    account_balance: Option<f64>,
}

struct RiskAnalysis {
    risk_amount: f64,
    risk_percent: f64,
    potential_profit: f64,
    risk_reward_ratio: f64,
    recommendation: String,
}

fn analyze_trade_risk(params: &TradeParams) -> Option<RiskAnalysis> {
    // Extract all required parameters
    let entry = params.entry_price?;
    let stop = params.stop_loss?;
    let target = params.take_profit?;
    let size = params.position_size?;
    let balance = params.account_balance?;

    // Calculations
    let risk_per_unit = (entry - stop).abs();
    let profit_per_unit = (target - entry).abs();

    let risk_amount = risk_per_unit * size;
    let risk_percent = (risk_amount / balance) * 100.0;
    let potential_profit = profit_per_unit * size;

    let risk_reward_ratio = if risk_amount > 0.0 {
        potential_profit / risk_amount
    } else {
        0.0
    };

    let recommendation = if risk_percent > 5.0 {
        String::from("HIGH RISK")
    } else if risk_reward_ratio < 1.5 {
        String::from("LOW R/R")
    } else {
        String::from("APPROVED")
    };

    Some(RiskAnalysis {
        risk_amount,
        risk_percent,
        potential_profit,
        risk_reward_ratio,
        recommendation,
    })
}
```

## Working with Collections and Option

```rust
fn main() {
    let trades: Vec<Option<f64>> = vec![
        Some(150.0),   // Profitable trade
        Some(-50.0),   // Losing trade
        None,          // No data
        Some(200.0),   // Profitable
        None,          // No data
        Some(-30.0),   // Losing
    ];

    // Filter out None and sum
    let total_pnl: f64 = trades
        .iter()
        .filter_map(|&t| t)  // Removes None, extracts values
        .sum();

    println!("Total PnL: ${:.2}", total_pnl);

    // Count only profitable trades
    let profitable_count = trades
        .iter()
        .filter_map(|&t| t)
        .filter(|&pnl| pnl > 0.0)
        .count();

    println!("Profitable trades: {}", profitable_count);

    // Average PnL (only known trades)
    let known_trades: Vec<f64> = trades
        .iter()
        .filter_map(|&t| t)
        .collect();

    if !known_trades.is_empty() {
        let avg_pnl = known_trades.iter().sum::<f64>() / known_trades.len() as f64;
        println!("Average PnL: ${:.2}", avg_pnl);
    }
}
```

## What We Learned

| Method/Construct | Description | Example |
|------------------|-------------|---------|
| `Some(value)` | Creates Option with value | `Some(42000.0)` |
| `None` | Empty Option | `let price: Option<f64> = None` |
| `is_some()` | Check if value exists | `price.is_some()` |
| `is_none()` | Check if empty | `price.is_none()` |
| `unwrap_or(default)` | Value or default | `price.unwrap_or(0.0)` |
| `unwrap_or_else(f)` | Value or function result | `price.unwrap_or_else(\|\| calc())` |
| `map(f)` | Transform value | `price.map(\|p\| p * 2.0)` |
| `and_then(f)` | Chain Options | `price.and_then(calc_fee)` |
| `filter(pred)` | Conditional filter | `price.filter(\|&p\| p > 0.0)` |
| `?` operator | Early return on None | `let p = price?;` |

## Exercises

1. **Get Price**: Write a function `get_price(symbol: &str, prices: &HashMap<String, f64>) -> Option<f64>` that returns an asset's price if it exists in the map.

2. **Calculate Spread**: Create a function `calculate_spread(bid: Option<f64>, ask: Option<f64>) -> Option<f64>` that returns the spread only if both values are available.

3. **Find in Portfolio**: Write a function `find_position(portfolio: &[Position], symbol: &str) -> Option<&Position>` that finds a position by symbol.

4. **Validation Chain**: Create a function `validate_order(order: &Order) -> Option<ValidatedOrder>` that checks all fields and returns a valid order only if everything is correct.

## Homework

1. Implement a price caching system where `get_cached_price` returns `Option<f64>` — the cached price or None if the cache is stale.

2. Create a function `find_best_entry(candles: &[Candle]) -> Option<EntrySignal>` that analyzes candles and returns an entry signal only under certain conditions.

3. Write a function `calculate_portfolio_stats(positions: &[Position]) -> Option<PortfolioStats>` that returns portfolio statistics or None if the portfolio is empty.

4. Implement a function `get_trading_recommendation(price: f64, indicators: &Indicators) -> Option<Recommendation>` where `Indicators` contains `Option<f64>` for various indicators (SMA, RSI, MACD). A recommendation is given only when all required indicators are present.

## Navigation

[← Previous day](../071-result-order-execution/en.md) | [Next day →](../073-result-error-handling/en.md)
