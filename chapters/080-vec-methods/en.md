# Day 80: Vec Methods — push, pop, get, len

## Trading Analogy

In real trading, data constantly changes:
- **New trades** are added to the journal in real-time
- **Orders** get executed and removed from the order book
- **Price history** grows with each new candle
- **Portfolio** dynamically changes when buying/selling assets

Unlike fixed-size arrays, `Vec` (vector) is a **dynamic collection** that can grow and shrink during program execution.

## Creating a Vector

```rust
fn main() {
    // Empty vector with type annotation
    let mut prices: Vec<f64> = Vec::new();

    // Vector with initial values (vec! macro)
    let closes = vec![42000.0, 42100.0, 41900.0, 42200.0];

    // Vector with repeated value
    let zeros: Vec<f64> = vec![0.0; 10];  // 10 zeros

    // With specified capacity (optimization)
    let mut orders: Vec<f64> = Vec::with_capacity(100);

    println!("Closes: {:?}", closes);
    println!("Zeros: {:?}", zeros);
    println!("Orders capacity: {}", orders.capacity());
}
```

## The push() Method — Adding Elements

`push()` adds an element to the end of the vector:

```rust
fn main() {
    let mut trade_log: Vec<f64> = Vec::new();

    // Add trades as they execute
    trade_log.push(42150.50);  // First trade
    trade_log.push(42155.00);  // Second trade
    trade_log.push(42148.75);  // Third trade

    println!("Trade log: {:?}", trade_log);
    println!("Total trades: {}", trade_log.len());
}
```

### Practical Example: Real-time Price Log

```rust
fn main() {
    let mut price_feed: Vec<f64> = Vec::new();

    // Simulate receiving prices from exchange
    let incoming_prices = [42000.0, 42005.0, 41998.0, 42010.0, 42015.0];

    for price in incoming_prices {
        price_feed.push(price);
        println!("Received: ${:.2}, Total ticks: {}", price, price_feed.len());
    }

    println!("\nFull price feed: {:?}", price_feed);
}
```

## The pop() Method — Removing the Last Element

`pop()` removes and returns the last element. Returns `Option<T>`:

```rust
fn main() {
    let mut order_book: Vec<f64> = vec![100.0, 150.0, 200.0, 250.0];

    println!("Order book: {:?}", order_book);

    // Execute the last order
    match order_book.pop() {
        Some(order) => println!("Executed order: ${}", order),
        None => println!("Order book is empty!"),
    }

    println!("After execution: {:?}", order_book);

    // Works safely with empty vector
    let mut empty: Vec<f64> = Vec::new();
    let result = empty.pop();
    println!("Pop from empty: {:?}", result);  // None
}
```

### Practical Example: LIFO Order Stack

```rust
fn main() {
    let mut pending_orders: Vec<(String, f64, f64)> = Vec::new();

    // Add orders (symbol, price, quantity)
    pending_orders.push(("BTC".to_string(), 42000.0, 0.5));
    pending_orders.push(("ETH".to_string(), 2200.0, 2.0));
    pending_orders.push(("BTC".to_string(), 42100.0, 0.3));

    println!("Pending orders: {}", pending_orders.len());

    // Cancel the last order
    if let Some((symbol, price, qty)) = pending_orders.pop() {
        println!("Cancelled: {} {} @ ${}", qty, symbol, price);
    }

    println!("Remaining orders: {}", pending_orders.len());
}
```

## The len() Method — Vector Length

`len()` returns the number of elements:

```rust
fn main() {
    let portfolio = vec!["BTC", "ETH", "SOL", "ADA"];

    println!("Assets in portfolio: {}", portfolio.len());

    // Check if empty
    let empty_portfolio: Vec<&str> = Vec::new();
    println!("Is empty: {}", empty_portfolio.is_empty());
    println!("Has assets: {}", !portfolio.is_empty());

    // Capacity vs length
    let mut prices = Vec::with_capacity(100);
    prices.push(42000.0);
    prices.push(42100.0);

    println!("Length: {}", prices.len());       // 2
    println!("Capacity: {}", prices.capacity()); // 100
}
```

### Practical Example: Checking Data Sufficiency for Indicators

```rust
fn main() {
    let mut candles: Vec<f64> = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    let sma_period = 5;
    let ema_period = 12;
    let rsi_period = 14;

    println!("Candles available: {}", candles.len());
    println!("Can calculate SMA-{}: {}", sma_period, candles.len() >= sma_period);
    println!("Can calculate EMA-{}: {}", ema_period, candles.len() >= ema_period);
    println!("Can calculate RSI-{}: {}", rsi_period, candles.len() >= rsi_period);

    // Add more data
    for i in 0..10 {
        candles.push(42000.0 + (i as f64 * 50.0));
    }

    println!("\nAfter adding data:");
    println!("Candles available: {}", candles.len());
    println!("Can calculate RSI-{}: {}", rsi_period, candles.len() >= rsi_period);
}
```

## The get() Method — Safe Element Access

`get()` returns `Option<&T>`, preventing panic on out-of-bounds access:

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Safe access
    match prices.get(2) {
        Some(price) => println!("Price at index 2: ${}", price),
        None => println!("Index out of bounds"),
    }

    // Access non-existent index
    match prices.get(100) {
        Some(price) => println!("Price: ${}", price),
        None => println!("No price at index 100"),
    }

    // Shorthand with if let
    if let Some(last) = prices.get(prices.len() - 1) {
        println!("Latest price: ${}", last);
    }

    // DANGEROUS: direct index access can panic!
    // println!("{}", prices[100]);  // panic!
}
```

### Practical Example: Safe Moving Average Calculation

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Get prices for SMA-3 calculation
    let sma_window = 3;

    // Safely get the last N prices
    if prices.len() >= sma_window {
        let start_index = prices.len() - sma_window;
        let mut sum = 0.0;

        for i in start_index..prices.len() {
            if let Some(price) = prices.get(i) {
                sum += price;
            }
        }

        let sma = sum / sma_window as f64;
        println!("SMA-{}: ${:.2}", sma_window, sma);
    } else {
        println!("Not enough data for SMA-{}", sma_window);
    }
}
```

## Combining Methods

```rust
fn main() {
    let mut price_history: Vec<f64> = Vec::new();

    // Populate history
    let ticks = [42000.0, 42050.0, 42025.0, 42100.0, 42080.0, 42150.0];

    for tick in ticks {
        price_history.push(tick);

        // Print stats after each tick
        let len = price_history.len();
        let last = price_history.get(len - 1).unwrap();

        print!("Tick #{}: ${:.2}", len, last);

        // If we have enough data, calculate change
        if len >= 2 {
            if let Some(prev) = price_history.get(len - 2) {
                let change = (last - prev) / prev * 100.0;
                let sign = if change >= 0.0 { "+" } else { "" };
                print!(" ({}{}%)", sign, format!("{:.2}", change));
            }
        }

        println!();
    }
}
```

## Practical Example: Trade Journal

```rust
#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() {
    let mut trades: Vec<Trade> = Vec::new();

    // Add trades
    trades.push(Trade {
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        price: 42000.0,
        quantity: 0.5,
    });

    trades.push(Trade {
        symbol: "ETH/USDT".to_string(),
        side: "BUY".to_string(),
        price: 2200.0,
        quantity: 2.0,
    });

    trades.push(Trade {
        symbol: "BTC/USDT".to_string(),
        side: "SELL".to_string(),
        price: 42500.0,
        quantity: 0.5,
    });

    // Statistics
    println!("Total trades: {}", trades.len());

    // Last trade
    if let Some(last) = trades.get(trades.len() - 1) {
        println!("Last trade: {} {} {} @ ${}",
            last.side, last.quantity, last.symbol, last.price);
    }

    // Calculate P&L for BTC
    let mut btc_pnl = 0.0;
    for trade in &trades {
        if trade.symbol == "BTC/USDT" {
            if trade.side == "BUY" {
                btc_pnl -= trade.price * trade.quantity;
            } else {
                btc_pnl += trade.price * trade.quantity;
            }
        }
    }

    println!("BTC P&L: ${:.2}", btc_pnl);
}
```

## Practical Example: Order Book Management

```rust
fn main() {
    let mut bid_orders: Vec<f64> = Vec::new();
    let mut ask_orders: Vec<f64> = Vec::new();

    // Add buy orders (bid)
    bid_orders.push(41900.0);
    bid_orders.push(41850.0);
    bid_orders.push(41800.0);

    // Add sell orders (ask)
    ask_orders.push(42000.0);
    ask_orders.push(42050.0);
    ask_orders.push(42100.0);

    // Best prices
    let best_bid = bid_orders.get(0);
    let best_ask = ask_orders.get(0);

    match (best_bid, best_ask) {
        (Some(bid), Some(ask)) => {
            println!("Best Bid: ${}", bid);
            println!("Best Ask: ${}", ask);
            println!("Spread: ${:.2} ({:.4}%)",
                ask - bid,
                (ask - bid) / bid * 100.0);
        }
        _ => println!("Order book is incomplete"),
    }

    // Execute order (remove best ask)
    if let Some(executed) = ask_orders.pop() {
        println!("\nMarket buy executed at ${}", executed);
    }

    // Market depth
    println!("\nBid depth: {} orders", bid_orders.len());
    println!("Ask depth: {} orders", ask_orders.len());
}
```

## Practical Example: Sliding Price Window

```rust
fn main() {
    let mut price_window: Vec<f64> = Vec::new();
    let window_size = 5;

    let price_stream = [
        42000.0, 42050.0, 42025.0, 42100.0, 42080.0,
        42150.0, 42120.0, 42200.0, 42180.0, 42250.0
    ];

    for price in price_stream {
        // Add new price
        price_window.push(price);

        // Remove old data if window is full
        while price_window.len() > window_size {
            price_window.remove(0);  // Remove first element
        }

        // Calculate SMA if window is full
        if price_window.len() == window_size {
            let sum: f64 = price_window.iter().sum();
            let sma = sum / window_size as f64;
            println!("Price: ${:.2} | SMA-{}: ${:.2} | Window: {:?}",
                price, window_size, sma, price_window);
        } else {
            println!("Price: ${:.2} | Collecting data... ({}/{})",
                price, price_window.len(), window_size);
        }
    }
}
```

## Additional Useful Methods

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 41900.0, 42200.0];

    // first() and last()
    println!("First: {:?}", prices.first());
    println!("Last: {:?}", prices.last());

    // insert() - insert at index
    prices.insert(0, 41800.0);  // At beginning
    println!("After insert: {:?}", prices);

    // remove() - remove at index
    let removed = prices.remove(2);
    println!("Removed: {}, Now: {:?}", removed, prices);

    // clear() - remove all elements
    prices.clear();
    println!("After clear: {:?}, len: {}", prices, prices.len());

    // extend() - add multiple elements
    prices.extend([42000.0, 42100.0, 42200.0]);
    println!("After extend: {:?}", prices);

    // contains()
    println!("Contains 42100: {}", prices.contains(&42100.0));
}
```

## What We Learned

| Method | Description | Returns |
|--------|-------------|---------|
| `push(val)` | Adds element to the end | `()` |
| `pop()` | Removes and returns last element | `Option<T>` |
| `len()` | Returns number of elements | `usize` |
| `get(i)` | Safe access by index | `Option<&T>` |
| `is_empty()` | Check if empty | `bool` |
| `capacity()` | Current capacity | `usize` |
| `first()` | First element | `Option<&T>` |
| `last()` | Last element | `Option<&T>` |

## Homework

1. **Trading Ticker Simulation**: Create a vector that accumulates prices from a "data stream". After each tick, output the current price, average price, minimum and maximum.

2. **Order Stack**: Implement an order system using `push` and `pop`. Add functions to add an order, cancel the last order, and peek at the last order without removing it.

3. **Moving Average**: Implement a function that maintains a fixed-size vector (e.g., 20 elements) and removes the oldest element when adding a new one.

4. **Trade Journal with Filtering**: Create a vector of trades and implement functions for:
   - Counting total number of trades
   - Getting the last N trades
   - Filtering trades by symbol

## Navigation

[← Previous day](../079-vec-introduction/en.md) | [Next day →](../081-vec-iteration/en.md)
