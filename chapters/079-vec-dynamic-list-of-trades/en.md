# Day 79: Vec — Dynamic List of Trades

## Trading Analogy

In real trading, the amount of data constantly changes:
- New trades are added to the journal
- Orders are cancelled and created
- Portfolio grows with new assets
- Price history expands with each candle

Fixed-size arrays don't work here — we need a **dynamic list** that can grow and shrink. In Rust, this is `Vec<T>` — the vector.

## What is Vec?

`Vec<T>` (vector) is:
- A dynamic array whose size can change
- Stores elements of the same type `T`
- Allocates memory on the heap
- The primary collection for lists in Rust

## Creating a Vector

```rust
fn main() {
    // Empty vector with explicit type
    let trades: Vec<f64> = Vec::new();
    println!("Empty trades: {:?}", trades);

    // vec! macro — convenient way to create
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0];
    println!("Prices: {:?}", prices);

    // Vector with initial capacity (optimization)
    let mut orders: Vec<String> = Vec::with_capacity(100);
    println!("Orders capacity: {}", orders.capacity());

    // Vector from repeated values
    let zeros = vec![0.0; 10];
    println!("Zeros: {:?}", zeros);
}
```

## Adding Elements

```rust
fn main() {
    let mut trades: Vec<f64> = Vec::new();

    // push — add to the end
    trades.push(42000.0);
    trades.push(42150.0);
    trades.push(41900.0);

    println!("Trades: {:?}", trades);
    println!("Count: {}", trades.len());

    // Adding buy orders
    let mut buy_orders = vec![100.0, 200.0];
    buy_orders.push(150.0);
    buy_orders.push(175.0);

    println!("Buy orders: {:?}", buy_orders);
}
```

## Accessing Elements

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // By index (can panic!)
    println!("First: {}", prices[0]);
    println!("Last: {}", prices[prices.len() - 1]);

    // Safe access with get()
    match prices.get(2) {
        Some(price) => println!("Price at index 2: {}", price),
        None => println!("Index out of bounds"),
    }

    // Safe for non-existent index
    if let Some(price) = prices.get(100) {
        println!("Price: {}", price);
    } else {
        println!("No price at index 100");
    }

    // first() and last()
    println!("First: {:?}", prices.first());
    println!("Last: {:?}", prices.last());
}
```

## Removing Elements

```rust
fn main() {
    let mut orders = vec!["BUY BTC", "SELL ETH", "BUY SOL", "SELL BTC"];
    println!("Orders: {:?}", orders);

    // pop — remove last element
    let last = orders.pop();
    println!("Removed: {:?}", last);
    println!("Orders: {:?}", orders);

    // remove — remove by index
    let removed = orders.remove(1);  // Remove "SELL ETH"
    println!("Removed: {}", removed);
    println!("Orders: {:?}", orders);

    // clear — empty the vector
    orders.clear();
    println!("After clear: {:?}", orders);
    println!("Is empty: {}", orders.is_empty());
}
```

## Modifying Elements

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 41900.0];
    println!("Before: {:?}", prices);

    // Modify by index
    prices[0] = 42050.0;
    prices[2] = 42000.0;
    println!("After: {:?}", prices);

    // Modify through iterator
    for price in &mut prices {
        *price *= 1.01;  // Increase by 1%
    }
    println!("After 1% increase: {:?}", prices);
}
```

## Iterating Over a Vector

```rust
fn main() {
    let trades = vec![
        ("BTC", 42000.0, 0.5),
        ("ETH", 2200.0, 2.0),
        ("SOL", 100.0, 10.0),
    ];

    // Simple for (moves ownership!)
    // for trade in trades { ... }

    // By reference — read only
    println!("=== Portfolio ===");
    for (symbol, price, amount) in &trades {
        let value = price * amount;
        println!("{}: {} @ ${} = ${}", symbol, amount, price, value);
    }

    // With index
    println!("\n=== With index ===");
    for (i, trade) in trades.iter().enumerate() {
        println!("[{}] {:?}", i, trade);
    }
}
```

## Practical Example: Trade Journal

```rust
#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,      // "BUY" or "SELL"
    price: f64,
    quantity: f64,
    timestamp: u64,
}

impl Trade {
    fn new(symbol: &str, side: &str, price: f64, quantity: f64, timestamp: u64) -> Self {
        Trade {
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            quantity,
            timestamp,
        }
    }

    fn value(&self) -> f64 {
        self.price * self.quantity
    }
}

fn main() {
    let mut trade_log: Vec<Trade> = Vec::new();

    // Add trades
    trade_log.push(Trade::new("BTC", "BUY", 42000.0, 0.5, 1000));
    trade_log.push(Trade::new("ETH", "BUY", 2200.0, 2.0, 1001));
    trade_log.push(Trade::new("BTC", "SELL", 43000.0, 0.3, 1002));
    trade_log.push(Trade::new("SOL", "BUY", 100.0, 10.0, 1003));

    println!("=== Trade Log ({} trades) ===", trade_log.len());
    for trade in &trade_log {
        println!("{} {} {} @ ${:.2} = ${:.2}",
            trade.side, trade.quantity, trade.symbol,
            trade.price, trade.value());
    }

    // Total volume
    let total_volume: f64 = trade_log.iter()
        .map(|t| t.value())
        .sum();
    println!("\nTotal volume: ${:.2}", total_volume);
}
```

## Practical Example: Order Management

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    status: String,
}

fn main() {
    let mut order_book: Vec<Order> = Vec::new();

    // Create orders
    order_book.push(Order {
        id: 1, symbol: "BTC".to_string(), side: "BUY".to_string(),
        price: 41000.0, quantity: 0.5, status: "OPEN".to_string()
    });
    order_book.push(Order {
        id: 2, symbol: "ETH".to_string(), side: "SELL".to_string(),
        price: 2300.0, quantity: 2.0, status: "OPEN".to_string()
    });
    order_book.push(Order {
        id: 3, symbol: "BTC".to_string(), side: "BUY".to_string(),
        price: 40500.0, quantity: 1.0, status: "OPEN".to_string()
    });

    println!("=== Open Orders ===");
    for order in &order_book {
        println!("#{}: {} {} {} @ ${}",
            order.id, order.side, order.quantity, order.symbol, order.price);
    }

    // Cancel order #2
    if let Some(pos) = order_book.iter().position(|o| o.id == 2) {
        let cancelled = order_book.remove(pos);
        println!("\nCancelled order #{}", cancelled.id);
    }

    // Find all BTC orders
    let btc_orders: Vec<&Order> = order_book.iter()
        .filter(|o| o.symbol == "BTC")
        .collect();

    println!("\n=== BTC Orders ===");
    for order in btc_orders {
        println!("#{}: {} @ ${}", order.id, order.side, order.price);
    }
}
```

## Practical Example: Price Analysis

```rust
fn main() {
    // BTC price history
    let mut prices: Vec<f64> = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    // Add new prices
    prices.push(42600.0);
    prices.push(42550.0);
    prices.push(42700.0);

    println!("Price history ({} candles):", prices.len());

    // Calculate SMA
    let sma5 = calculate_sma(&prices, 5);
    let sma10 = calculate_sma(&prices, 10);

    println!("SMA-5: ${:.2}", sma5);
    println!("SMA-10: ${:.2}", sma10);

    // Current price vs SMA
    let current = *prices.last().unwrap();
    if current > sma5 {
        println!("Price above SMA-5 - bullish signal");
    } else {
        println!("Price below SMA-5 - bearish signal");
    }

    // Volatility
    let volatility = calculate_volatility(&prices);
    println!("Volatility: {:.2}%", volatility);
}

fn calculate_sma(prices: &[f64], period: usize) -> f64 {
    if prices.len() < period {
        return 0.0;
    }
    let sum: f64 = prices.iter().rev().take(period).sum();
    sum / period as f64
}

fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    let mean: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
    let variance: f64 = prices.iter()
        .map(|p| (p - mean).powi(2))
        .sum::<f64>() / prices.len() as f64;

    (variance.sqrt() / mean) * 100.0
}
```

## Practical Example: Portfolio Management

```rust
#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

impl Position {
    fn value(&self, current_price: f64) -> f64 {
        self.quantity * current_price
    }

    fn pnl(&self, current_price: f64) -> f64 {
        self.quantity * (current_price - self.avg_price)
    }

    fn pnl_percent(&self, current_price: f64) -> f64 {
        (current_price - self.avg_price) / self.avg_price * 100.0
    }
}

fn main() {
    let mut portfolio: Vec<Position> = Vec::new();

    // Add positions
    portfolio.push(Position {
        symbol: "BTC".to_string(),
        quantity: 0.5,
        avg_price: 40000.0,
    });
    portfolio.push(Position {
        symbol: "ETH".to_string(),
        quantity: 5.0,
        avg_price: 2000.0,
    });
    portfolio.push(Position {
        symbol: "SOL".to_string(),
        quantity: 50.0,
        avg_price: 80.0,
    });

    // Current prices
    let prices = [("BTC", 42000.0), ("ETH", 2200.0), ("SOL", 100.0)];

    println!("=== Portfolio ===");
    let mut total_value = 0.0;
    let mut total_pnl = 0.0;

    for pos in &portfolio {
        // Find current price
        let current = prices.iter()
            .find(|(s, _)| *s == pos.symbol)
            .map(|(_, p)| *p)
            .unwrap_or(0.0);

        let value = pos.value(current);
        let pnl = pos.pnl(current);
        let pnl_pct = pos.pnl_percent(current);

        println!("{}: {} @ ${:.2} -> ${:.2} | P&L: ${:.2} ({:+.2}%)",
            pos.symbol, pos.quantity, pos.avg_price,
            value, pnl, pnl_pct);

        total_value += value;
        total_pnl += pnl;
    }

    println!("\nTotal Value: ${:.2}", total_value);
    println!("Total P&L: ${:.2}", total_pnl);
}
```

## Useful Vec Methods

```rust
fn main() {
    let mut prices = vec![42000.0, 41500.0, 42500.0, 41000.0, 43000.0];

    // Vector info
    println!("Length: {}", prices.len());
    println!("Capacity: {}", prices.capacity());
    println!("Is empty: {}", prices.is_empty());

    // Search
    println!("Contains 42000: {}", prices.contains(&42000.0));

    // Sorting
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("Sorted: {:?}", prices);

    // Reverse
    prices.reverse();
    println!("Reversed: {:?}", prices);

    // Deduplication (after sorting)
    let mut data = vec![1, 2, 2, 3, 3, 3, 4];
    data.dedup();
    println!("Deduped: {:?}", data);

    // Retain — keep elements by condition
    let mut trades = vec![100.0, -50.0, 200.0, -100.0, 150.0];
    trades.retain(|&x| x > 0.0);  // Only profitable
    println!("Profitable: {:?}", trades);
}
```

## Combining Vectors

```rust
fn main() {
    let mut btc_trades = vec![42000.0, 42100.0, 42200.0];
    let eth_trades = vec![2200.0, 2250.0, 2300.0];

    // extend — add all elements
    // btc_trades.extend(eth_trades);

    // append — move all elements (eth_trades becomes empty)
    let mut all_trades = btc_trades.clone();
    let mut eth_copy = eth_trades.clone();
    all_trades.append(&mut eth_copy);

    println!("All trades: {:?}", all_trades);
    println!("ETH copy after append: {:?}", eth_copy);

    // concat via iterators
    let combined: Vec<f64> = btc_trades.iter()
        .chain(eth_trades.iter())
        .cloned()
        .collect();
    println!("Combined: {:?}", combined);
}
```

## Converting to Slice

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Vec automatically coerces to slice
    print_prices(&prices);

    // Explicit conversion
    let slice: &[f64] = &prices[..];
    println!("Slice: {:?}", slice);

    // Part of vector
    let last_three = &prices[2..];
    println!("Last 3: {:?}", last_three);
}

fn print_prices(prices: &[f64]) {
    println!("Prices ({}):", prices.len());
    for price in prices {
        println!("  ${:.2}", price);
    }
}
```

## Vec vs Array — When to Use What

| Characteristic | Array `[T; N]` | Vec `Vec<T>` |
|---------------|----------------|--------------|
| Size | Fixed | Dynamic |
| Memory | Stack | Heap |
| Performance | Slightly faster | More flexible |
| Use case | Known size | Unknown size |

**Use Array when:**
- Size is known at compile time
- Maximum performance needed
- Example: OHLC candle `[f64; 4]`

**Use Vec when:**
- Size is unknown or changes
- Data arrives dynamically
- Example: trade history, orders

## What We Learned

| Concept | Description |
|---------|-------------|
| `Vec::new()` | Create empty vector |
| `vec![...]` | Macro to create vector |
| `push()` | Add element to end |
| `pop()` | Remove and return last |
| `get(i)` | Safe access by index |
| `len()` | Number of elements |
| `iter()` | Iterator over elements |

## Homework

1. **Trade Journal**: Create a `TradeLog` struct with methods:
   - `add_trade()` — add a trade
   - `total_volume()` — total volume
   - `profitable_trades()` — list of profitable trades
   - `by_symbol()` — trades by ticker

2. **Order Book**: Implement a simple order book:
   - Add limit orders
   - Cancel by ID
   - Find best bid/ask price
   - Matching (execution) on crossover

3. **Indicator Calculation**: Write functions for Vec<f64>:
   - `sma(period)` — simple moving average
   - `ema(period)` — exponential moving average
   - `rsi(period)` — relative strength index

4. **Risk Management**: Create a position tracker with:
   - Total portfolio risk calculation
   - Finding positions with loss > 5%
   - Rebalancing by weights

## Navigation

[← Day 78](../078-hashmap-asset-lookup/en.md) | [Day 80 →](../080-hashset-unique-tickers/en.md)
