# Day 11: Tuples — Bid and Ask Price Together

## Trading Analogy

In the order book, there are always two prices:
- **Bid** (buy) — the best price someone is willing to buy at
- **Ask** (sell) — the best price someone is willing to sell at

These two prices always come **together** — they're linked. In Rust, we use **tuples** to group related values.

## What is a Tuple?

A tuple is a fixed group of values, possibly of different types:

```rust
fn main() {
    // (bid, ask) - order book prices
    let spread: (f64, f64) = (42000.0, 42010.0);

    println!("Bid: {}, Ask: {}", spread.0, spread.1);
}
```

## Creating Tuples

```rust
fn main() {
    // Different data types in one tuple
    let trade: (&str, f64, f64, bool) = ("BTC/USDT", 42000.0, 0.5, true);

    println!("Symbol: {}", trade.0);
    println!("Price: {}", trade.1);
    println!("Quantity: {}", trade.2);
    println!("Is Long: {}", trade.3);
}
```

A tuple can contain up to 12 elements (more is not recommended).

## Accessing Elements

### By Index

```rust
fn main() {
    let candle = (42000.0, 42500.0, 41800.0, 42200.0); // O, H, L, C

    println!("Open: {}", candle.0);
    println!("High: {}", candle.1);
    println!("Low: {}", candle.2);
    println!("Close: {}", candle.3);
}
```

### Destructuring

```rust
fn main() {
    let candle: (f64, f64, f64, f64) = (42000.0, 42500.0, 41800.0, 42200.0);

    // Unpack tuple into variables
    let (open, high, low, close) = candle;

    println!("Open: {}", open);
    println!("High: {}", high);
    println!("Low: {}", low);
    println!("Close: {}", close);

    // Calculation
    let body = (close - open).abs();
    let range = high - low;

    println!("Body: {}", body);
    println!("Range: {}", range);
}
```

### Partial Destructuring

```rust
fn main() {
    let trade = ("BTC/USDT", 42000.0, 0.5, true, 123456);

    // We only need symbol and price
    let (symbol, price, ..) = trade;

    println!("Symbol: {}, Price: {}", symbol, price);

    // Or first and last
    let (sym, .., order_id) = trade;
    println!("Symbol: {}, Order ID: {}", sym, order_id);
}
```

## Mutable Tuples

```rust
fn main() {
    let mut position = ("BTC/USDT", 0.0, false); // (symbol, pnl, is_open)

    println!("Before: {:?}", position);

    // Open position
    position.2 = true;
    println!("Opened: {:?}", position);

    // Update PnL
    position.1 = 150.50;
    println!("Updated PnL: {:?}", position);
}
```

## Nested Tuples

```rust
fn main() {
    // ((bid, ask), (bid_size, ask_size))
    let order_book_top: ((f64, f64), (f64, f64)) = (
        (42000.0, 42010.0),   // Prices
        (1.5, 2.3)            // Volumes
    );

    let ((bid, ask), (bid_size, ask_size)) = order_book_top;

    println!("Bid: {} x {}", bid, bid_size);
    println!("Ask: {} x {}", ask, ask_size);

    let spread = ask - bid;
    println!("Spread: {}", spread);
}
```

## Tuples as Return Values

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42050.0];

    let (min, max) = find_min_max(&prices);
    println!("Min: {}, Max: {}", min, max);
    println!("Range: {}", max - min);
}

fn find_min_max(prices: &[f64]) -> (f64, f64) {
    let mut min = f64::MAX;
    let mut max = f64::MIN;

    for &price in prices {
        if price < min {
            min = price;
        }
        if price > max {
            max = price;
        }
    }

    (min, max)  // Return tuple
}
```

## Practical Example: Spread Analysis

```rust
fn main() {
    // Order book data
    let bid = 42000.0;
    let ask = 42015.0;
    let bid_size = 2.5;
    let ask_size = 1.8;

    // Group related data
    let best_bid: (f64, f64) = (bid, bid_size);
    let best_ask: (f64, f64) = (ask, ask_size);

    // Calculations
    let spread = best_ask.0 - best_bid.0;
    let spread_percent = (spread / best_bid.0) * 100.0;
    let mid_price = (best_bid.0 + best_ask.0) / 2.0;

    println!("╔════════════════════════════════╗");
    println!("║      ORDER BOOK TOP            ║");
    println!("╠════════════════════════════════╣");
    println!("║ Best Bid: ${:.2} x {:.4}    ║", best_bid.0, best_bid.1);
    println!("║ Best Ask: ${:.2} x {:.4}    ║", best_ask.0, best_ask.1);
    println!("╠════════════════════════════════╣");
    println!("║ Spread:    ${:.2}            ║", spread);
    println!("║ Spread %:   {:.4}%           ║", spread_percent);
    println!("║ Mid Price: ${:.2}           ║", mid_price);
    println!("╚════════════════════════════════╝");
}
```

## Practical Example: OHLCV Candle

```rust
fn main() {
    // OHLCV: Open, High, Low, Close, Volume
    let candle: (f64, f64, f64, f64, f64) = (42000.0, 42500.0, 41800.0, 42200.0, 150.5);

    let (open, high, low, close, volume) = candle;

    // Candle analysis
    let is_bullish = close > open;
    let body_size = (close - open).abs();
    let upper_shadow = high - close.max(open);
    let lower_shadow = close.min(open) - low;
    let total_range = high - low;

    println!("=== Candle Analysis ===");
    println!("Open: {}, Close: {}", open, close);
    println!("High: {}, Low: {}", high, low);
    println!("Volume: {}", volume);
    println!();
    println!("Type: {}", if is_bullish { "Bullish" } else { "Bearish" });
    println!("Body: {:.2}", body_size);
    println!("Upper Shadow: {:.2}", upper_shadow);
    println!("Lower Shadow: {:.2}", lower_shadow);
    println!("Range: {:.2}", total_range);
    println!("Body/Range: {:.1}%", (body_size / total_range) * 100.0);
}
```

## Practical Example: Trade Result

```rust
fn main() {
    // Simulate several trades
    let trades: [(f64, f64, f64); 5] = [
        (42000.0, 42500.0, 0.5),   // entry, exit, size
        (42500.0, 42200.0, 0.3),
        (42200.0, 42800.0, 0.4),
        (42800.0, 42600.0, 0.2),
        (42600.0, 43000.0, 0.6),
    ];

    let mut total_pnl = 0.0;
    let mut wins = 0;
    let mut losses = 0;

    println!("=== Trade Results ===");
    println!("{:<5} {:>10} {:>10} {:>8} {:>10}", "#", "Entry", "Exit", "Size", "PnL");
    println!("{}", "-".repeat(48));

    for (i, trade) in trades.iter().enumerate() {
        let (entry, exit, size) = *trade;
        let pnl = (exit - entry) * size;

        if pnl > 0.0 {
            wins += 1;
        } else {
            losses += 1;
        }

        total_pnl += pnl;

        println!("{:<5} {:>10.2} {:>10.2} {:>8.2} {:>+10.2}",
            i + 1, entry, exit, size, pnl);
    }

    println!("{}", "-".repeat(48));
    println!("Total PnL: {:+.2}", total_pnl);
    println!("Wins: {}, Losses: {}", wins, losses);
    println!("Win Rate: {:.1}%", (wins as f64 / trades.len() as f64) * 100.0);
}
```

## Unit Tuple

The empty tuple `()` is called "unit" and means "nothing":

```rust
fn main() {
    let nothing: () = ();

    println!("This prints nothing: {:?}", nothing);
}

// A function with no return value returns ()
fn do_something() {
    println!("Doing something...");
    // Implicitly returns ()
}
```

## Comparing Tuples

```rust
fn main() {
    let candle1 = (42000.0, 42500.0, 41800.0, 42200.0);
    let candle2 = (42000.0, 42500.0, 41800.0, 42200.0);
    let candle3 = (42100.0, 42500.0, 41800.0, 42200.0);

    println!("candle1 == candle2: {}", candle1 == candle2);  // true
    println!("candle1 == candle3: {}", candle1 == candle3);  // false

    // Comparison by order
    let a = (1, 2);
    let b = (1, 3);
    println!("(1,2) < (1,3): {}", a < b);  // true
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `(T1, T2, ...)` | Creating a tuple |
| `tuple.0` | Access by index |
| `let (a, b) = tuple` | Destructuring |
| `(..)` | Ignoring some elements |
| `()` | Empty tuple (unit) |

## Homework

1. Create a tuple to store order data: (symbol, side, price, quantity, filled)

2. Write a function that takes an OHLC tuple and returns (is_bullish, body_size, range)

3. Create an array of 5 (bid, ask) tuples and find the minimum/maximum spread

4. Implement a PnL calculation function that takes (entry, exit, size, fee_percent) and returns (gross_pnl, fee, net_pnl)

## Navigation

[← Previous day](../010-strings-tickers/en.md) | [Next day →](../012-arrays-closing-prices/en.md)
