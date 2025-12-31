# Day 54: Pattern — Borrow, Don't Own

## Trading Analogy

Imagine you're an analyst at an investment firm. When you need to analyze a client's portfolio, you don't take ownership of their assets — you simply **look** at them and draw conclusions. After the analysis, the portfolio remains intact with the client.

This is the **"Borrow, Don't Own"** pattern:
- The analyst **borrows** data for analysis
- The client **owns** their portfolio
- After work, data returns to the owner

In Rust, this pattern is the foundation of efficient and safe code.

## Theory: Why Borrow?

### The Problem with Ownership

```rust
fn main() {
    let portfolio = vec!["BTC", "ETH", "SOL"];

    // Transfer ownership — portfolio is no longer accessible!
    let total = count_assets(portfolio);

    // Error! portfolio has been moved
    // println!("{:?}", portfolio);
}

fn count_assets(assets: Vec<&str>) -> usize {
    assets.len()
}
```

### Solution: Borrowing

```rust
fn main() {
    let portfolio = vec!["BTC", "ETH", "SOL"];

    // Borrow — portfolio stays with us
    let total = count_assets(&portfolio);

    // Works! portfolio is still ours
    println!("Portfolio: {:?}, count: {}", portfolio, total);
}

fn count_assets(assets: &Vec<&str>) -> usize {
    assets.len()
}
```

## Pattern in Action: Market Analysis

### Example 1: Price Analysis Without Ownership

```rust
fn main() {
    let prices = vec![42000.0, 42500.0, 41800.0, 43200.0, 42900.0];

    // All functions borrow the data
    let avg = calculate_average(&prices);
    let max = find_max(&prices);
    let min = find_min(&prices);
    let volatility = calculate_volatility(&prices);

    // prices is still available for further use
    println!("Price analysis for {} data points:", prices.len());
    println!("  Average: ${:.2}", avg);
    println!("  Max: ${:.2}", max);
    println!("  Min: ${:.2}", min);
    println!("  Volatility: {:.2}%", volatility);
}

fn calculate_average(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}

fn find_max(prices: &[f64]) -> f64 {
    prices.iter().copied().fold(f64::MIN, f64::max)
}

fn find_min(prices: &[f64]) -> f64 {
    prices.iter().copied().fold(f64::MAX, f64::min)
}

fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    let avg = calculate_average(prices);
    let variance: f64 = prices.iter()
        .map(|p| (p - avg).powi(2))
        .sum::<f64>() / prices.len() as f64;

    (variance.sqrt() / avg) * 100.0
}
```

### Example 2: Order Analysis

```rust
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: String::from("buy"),
        price: 42000.0,
        quantity: 0.5,
    };

    // Each function borrows the order
    let value = calculate_order_value(&order);
    let fee = estimate_fee(&order, 0.1);
    let is_valid = validate_order(&order);

    // order is still available
    println!("Order: {} {} {} @ ${}",
        order.side, order.quantity, order.symbol, order.price);
    println!("Value: ${:.2}", value);
    println!("Fee: ${:.2}", fee);
    println!("Valid: {}", is_valid);
}

fn calculate_order_value(order: &Order) -> f64 {
    order.price * order.quantity
}

fn estimate_fee(order: &Order, fee_percent: f64) -> f64 {
    calculate_order_value(order) * (fee_percent / 100.0)
}

fn validate_order(order: &Order) -> bool {
    order.price > 0.0 && order.quantity > 0.0 && !order.symbol.is_empty()
}
```

### Example 3: Chain of Analyses

```rust
struct Portfolio {
    name: String,
    assets: Vec<Asset>,
    total_value: f64,
}

struct Asset {
    symbol: String,
    quantity: f64,
    price: f64,
}

fn main() {
    let portfolio = Portfolio {
        name: String::from("Main Portfolio"),
        assets: vec![
            Asset { symbol: String::from("BTC"), quantity: 1.5, price: 42000.0 },
            Asset { symbol: String::from("ETH"), quantity: 10.0, price: 2500.0 },
            Asset { symbol: String::from("SOL"), quantity: 100.0, price: 95.0 },
        ],
        total_value: 97500.0,
    };

    // Chain of analyses — each one borrows
    let report = generate_report(&portfolio);
    let allocation = calculate_allocation(&portfolio);
    let risk = assess_risk(&portfolio);

    // portfolio remains available
    println!("=== {} ===", portfolio.name);
    println!("{}", report);
    println!("\nAllocation:");
    for (symbol, percent) in allocation {
        println!("  {}: {:.1}%", symbol, percent);
    }
    println!("\nRisk level: {}", risk);
}

fn generate_report(portfolio: &Portfolio) -> String {
    format!(
        "Assets: {}, Total Value: ${:.2}",
        portfolio.assets.len(),
        portfolio.total_value
    )
}

fn calculate_allocation(portfolio: &Portfolio) -> Vec<(String, f64)> {
    portfolio.assets.iter()
        .map(|asset| {
            let value = asset.quantity * asset.price;
            let percent = (value / portfolio.total_value) * 100.0;
            (asset.symbol.clone(), percent)
        })
        .collect()
}

fn assess_risk(portfolio: &Portfolio) -> &'static str {
    let btc_allocation = portfolio.assets.iter()
        .find(|a| a.symbol == "BTC")
        .map(|a| (a.quantity * a.price) / portfolio.total_value)
        .unwrap_or(0.0);

    if btc_allocation > 0.5 {
        "High (>50% in BTC)"
    } else if btc_allocation > 0.3 {
        "Medium (30-50% in BTC)"
    } else {
        "Low (<30% in BTC)"
    }
}
```

## When to Use Borrowing

### Use `&T` (immutable reference) when:

```rust
// 1. You only need to read data
fn print_order(order: &Order) {
    println!("{}: {} @ {}", order.symbol, order.quantity, order.price);
}

// 2. You need to pass to multiple functions
fn analyze_order(order: &Order) {
    validate_order(order);
    calculate_order_value(order);
    estimate_fee(order, 0.1);
}

// 3. Working with large structures
fn process_large_history(history: &[Trade]) -> Summary {
    // Don't copy thousands of trades — just look at them
    Summary {
        count: history.len(),
        total_volume: history.iter().map(|t| t.volume).sum(),
    }
}
```

### Use `&mut T` (mutable reference) when:

```rust
// 1. You need to modify data but not take ownership
fn update_price(order: &mut Order, new_price: f64) {
    order.price = new_price;
}

// 2. You need to add elements to a collection
fn add_asset(portfolio: &mut Portfolio, asset: Asset) {
    portfolio.total_value += asset.quantity * asset.price;
    portfolio.assets.push(asset);
}

// 3. You need to modify state
fn execute_trade(position: &mut Position, trade: &Trade) {
    if trade.side == "buy" {
        position.quantity += trade.quantity;
    } else {
        position.quantity -= trade.quantity;
    }
}
```

## Practical Example: Trade Analyzer

```rust
#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct TradeStats {
    total_trades: usize,
    buy_trades: usize,
    sell_trades: usize,
    total_volume: f64,
    average_price: f64,
    pnl: f64,
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC".into(), side: "buy".into(), price: 42000.0, quantity: 0.5, timestamp: 1000 },
        Trade { symbol: "BTC".into(), side: "buy".into(), price: 41500.0, quantity: 0.3, timestamp: 2000 },
        Trade { symbol: "BTC".into(), side: "sell".into(), price: 43000.0, quantity: 0.4, timestamp: 3000 },
        Trade { symbol: "BTC".into(), side: "sell".into(), price: 42500.0, quantity: 0.2, timestamp: 4000 },
        Trade { symbol: "BTC".into(), side: "buy".into(), price: 42200.0, quantity: 0.1, timestamp: 5000 },
    ];

    println!("=== Trade Analysis ===\n");

    // All functions borrow trades
    let stats = calculate_stats(&trades);
    let best_trade = find_best_trade(&trades);
    let worst_trade = find_worst_trade(&trades);

    // trades is still available
    println!("Statistics:");
    println!("  Total trades: {}", stats.total_trades);
    println!("  Buy trades: {}", stats.buy_trades);
    println!("  Sell trades: {}", stats.sell_trades);
    println!("  Total volume: {:.4} BTC", stats.total_volume);
    println!("  Average price: ${:.2}", stats.average_price);
    println!("  Estimated PnL: ${:.2}", stats.pnl);

    if let Some(trade) = best_trade {
        println!("\nBest trade (highest sell):");
        println!("  {} {} @ ${}", trade.side, trade.quantity, trade.price);
    }

    if let Some(trade) = worst_trade {
        println!("\nWorst trade (highest buy):");
        println!("  {} {} @ ${}", trade.side, trade.quantity, trade.price);
    }

    // We can continue using trades
    println!("\nAll {} trades are still accessible!", trades.len());
}

fn calculate_stats(trades: &[Trade]) -> TradeStats {
    let total_trades = trades.len();
    let buy_trades = trades.iter().filter(|t| t.side == "buy").count();
    let sell_trades = trades.iter().filter(|t| t.side == "sell").count();

    let total_volume: f64 = trades.iter().map(|t| t.quantity).sum();

    let total_value: f64 = trades.iter().map(|t| t.price * t.quantity).sum();
    let average_price = if total_volume > 0.0 { total_value / total_volume } else { 0.0 };

    // Simple PnL calculation
    let buy_cost: f64 = trades.iter()
        .filter(|t| t.side == "buy")
        .map(|t| t.price * t.quantity)
        .sum();

    let sell_revenue: f64 = trades.iter()
        .filter(|t| t.side == "sell")
        .map(|t| t.price * t.quantity)
        .sum();

    TradeStats {
        total_trades,
        buy_trades,
        sell_trades,
        total_volume,
        average_price,
        pnl: sell_revenue - buy_cost,
    }
}

fn find_best_trade<'a>(trades: &'a [Trade]) -> Option<&'a Trade> {
    trades.iter()
        .filter(|t| t.side == "sell")
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
}

fn find_worst_trade<'a>(trades: &'a [Trade]) -> Option<&'a Trade> {
    trades.iter()
        .filter(|t| t.side == "buy")
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
}
```

## Comparing Approaches

```rust
// BAD: taking ownership unnecessarily
fn bad_calculate_total(prices: Vec<f64>) -> f64 {
    prices.iter().sum()
    // prices is destroyed, caller can't use it anymore
}

// GOOD: borrowing data
fn good_calculate_total(prices: &[f64]) -> f64 {
    prices.iter().sum()
    // prices remains with the caller
}

// BAD: cloning unnecessarily
fn bad_find_symbol(orders: Vec<Order>, symbol: &str) -> Option<Order> {
    orders.into_iter().find(|o| o.symbol == symbol)
}

// GOOD: returning a reference
fn good_find_symbol<'a>(orders: &'a [Order], symbol: &str) -> Option<&'a Order> {
    orders.iter().find(|o| o.symbol == symbol)
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `&T` | Immutable borrow — read-only access |
| `&mut T` | Mutable borrow — read and write access |
| `&[T]` | Slice — borrowing part of a collection |
| Pattern | Pass `&T` if ownership isn't needed |
| Benefit | Data stays with the owner |

## Homework

1. **Portfolio Analyzer**:
   Create functions that take `&Portfolio` and return:
   - Total value
   - Number of assets
   - Most expensive asset
   - Cryptocurrency percentage

2. **Order Validator**:
   Write a function `validate_orders(&[Order]) -> Vec<&Order>` that returns references only to valid orders.

3. **Price Comparison**:
   Create functions to compare two price time series:
   - `compare_averages(&[f64], &[f64]) -> f64`
   - `find_correlation(&[f64], &[f64]) -> f64`
   - `find_divergence(&[f64], &[f64]) -> Vec<(usize, f64)>`

4. **Risk Manager**:
   Write a `RiskManager` struct with methods that take `&Portfolio`:
   - `check_exposure(&self, portfolio: &Portfolio) -> RiskLevel`
   - `suggest_rebalance(&self, portfolio: &Portfolio) -> Vec<Suggestion>`
   - `generate_report(&self, portfolio: &Portfolio) -> String`

## Navigation

[← Previous day](../053-data-in-out/en.md) | [Next day →](../055-debugging-ownership/en.md)
