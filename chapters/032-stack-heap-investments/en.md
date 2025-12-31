# Day 32: Stack and Heap — Short and Long-term Investments

## Trading Analogy

Imagine you have two types of investments:

- **Stack** — is like **day trading**. You quickly open a position, close it the same day, and free up capital. Everything happens fast, predictably, and you know exactly when the position will close.

- **Heap** — is like **long-term investments**. You buy stocks and hold them for an indefinite time. Position size can change (you can add or partially sell), and you don't know in advance when you'll close the position completely.

## What is the Stack?

The stack is a memory area that works on the **LIFO** principle (Last In, First Out):

```rust
fn main() {
    let btc_price = 42000.0;        // Push onto stack
    let eth_price = 2200.0;         // Push onto stack
    calculate_ratio(btc_price, eth_price);
}   // eth_price removed, then btc_price

fn calculate_ratio(btc: f64, eth: f64) -> f64 {
    let ratio = btc / eth;          // ratio on stack
    ratio                           // Return, ratio removed
}
```

**Stack characteristics:**
- Very fast memory allocation and deallocation
- Fixed data size (known at compile time)
- Automatic cleanup when exiting scope

## What is the Heap?

The heap is a memory area for **dynamically sized** data:

```rust
fn main() {
    // String stores data on the heap
    let ticker = String::from("BTC/USDT");

    // Vec stores elements on the heap
    let mut prices: Vec<f64> = Vec::new();
    prices.push(42000.0);
    prices.push(42100.0);
    prices.push(42050.0);

    println!("Ticker: {}, Prices: {:?}", ticker, prices);
}
```

**Heap characteristics:**
- Slower than stack (needs to find free space)
- Flexible data size (can grow and shrink)
- Requires explicit or automatic deallocation

## Visualization: Stack vs Heap

```
┌─────────────────────────────────────────────────────────────────┐
│                           MEMORY                                 │
├──────────────────────────────┬──────────────────────────────────┤
│           STACK              │              HEAP                │
│    (Stack - Day Trading)     │     (Heap - Long Positions)      │
├──────────────────────────────┼──────────────────────────────────┤
│                              │                                   │
│  ┌────────────────────┐      │    ┌─────────────────────────┐   │
│  │ ratio: f64         │      │    │ "BTC/USDT"              │   │
│  │ 19.09              │      │    └─────────────────────────┘   │
│  └────────────────────┘      │              ▲                   │
│  ┌────────────────────┐      │              │ ptr               │
│  │ eth_price: f64     │      │    ┌─────────────────────────┐   │
│  │ 2200.0             │      │    │ prices: [42000, 42100,  │   │
│  └────────────────────┘      │    │          42050]         │   │
│  ┌────────────────────┐      │    └─────────────────────────┘   │
│  │ btc_price: f64     │      │              ▲                   │
│  │ 42000.0            │      │              │ ptr               │
│  └────────────────────┘      │                                   │
│  ┌────────────────────┐      │                                   │
│  │ ticker: String     │──────┼──────────────┘                   │
│  │ (ptr, len, cap)    │      │                                   │
│  └────────────────────┘      │                                   │
│  ┌────────────────────┐      │                                   │
│  │ prices: Vec<f64>   │──────┼──────────────┘                   │
│  │ (ptr, len, cap)    │      │                                   │
│  └────────────────────┘      │                                   │
│                              │                                   │
│  Fast, fixed size            │  Slower, dynamic size             │
└──────────────────────────────┴──────────────────────────────────┘
```

## Data Types: Where Are They Stored?

### On the Stack (fixed size)

```rust
fn main() {
    // Primitive types — always on stack
    let price: f64 = 42000.0;           // 8 bytes
    let quantity: i32 = 100;             // 4 bytes
    let is_bullish: bool = true;         // 1 byte
    let symbol: char = '₿';              // 4 bytes

    // Fixed-size tuples — on stack
    let trade: (f64, f64, bool) = (42000.0, 0.5, true);

    // Fixed-size arrays — on stack
    let last_5_prices: [f64; 5] = [41900.0, 42000.0, 42100.0, 42050.0, 42080.0];

    println!("Price: {}, Qty: {}, Bullish: {}", price, quantity, is_bullish);
}
```

### On the Heap (dynamic size)

```rust
fn main() {
    // String — dynamic string, data on heap
    let ticker = String::from("ETH/USDT");

    // Vec — dynamic array, data on heap
    let mut order_book: Vec<(f64, f64)> = Vec::new();
    order_book.push((42000.0, 1.5));
    order_book.push((41999.0, 2.3));
    order_book.push((41998.0, 0.8));

    // Box — explicit heap allocation
    let big_order = Box::new(Order {
        symbol: String::from("BTC/USDT"),
        price: 42000.0,
        quantity: 10.0,
        side: OrderSide::Buy,
    });

    println!("Ticker: {}", ticker);
    println!("Order book depth: {}", order_book.len());
    println!("Big order price: {}", big_order.price);
}

struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: OrderSide,
}

enum OrderSide {
    Buy,
    Sell,
}
```

## Practical Example: Order Book

```rust
fn main() {
    let mut order_book = OrderBook::new("BTC/USDT");

    // Add orders (data grows on heap)
    order_book.add_bid(41990.0, 1.5);
    order_book.add_bid(41980.0, 2.3);
    order_book.add_bid(41970.0, 0.8);

    order_book.add_ask(42000.0, 1.0);
    order_book.add_ask(42010.0, 2.0);
    order_book.add_ask(42020.0, 1.5);

    order_book.print_book();

    // Calculate spread (uses stack for calculations)
    if let Some(spread) = order_book.calculate_spread() {
        println!("\nSpread: ${:.2} ({:.4}%)", spread.0, spread.1);
    }
}

struct OrderBook {
    symbol: String,           // Stores data on heap
    bids: Vec<(f64, f64)>,   // Stores data on heap
    asks: Vec<(f64, f64)>,   // Stores data on heap
}

impl OrderBook {
    fn new(symbol: &str) -> Self {
        OrderBook {
            symbol: String::from(symbol),
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    fn add_bid(&mut self, price: f64, quantity: f64) {
        self.bids.push((price, quantity));
        // Sort by descending price
        self.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    }

    fn add_ask(&mut self, price: f64, quantity: f64) {
        self.asks.push((price, quantity));
        // Sort by ascending price
        self.asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    fn calculate_spread(&self) -> Option<(f64, f64)> {
        // These variables are on stack
        let best_bid = self.bids.first()?.0;
        let best_ask = self.asks.first()?.0;
        let spread = best_ask - best_bid;
        let spread_pct = (spread / best_bid) * 100.0;
        Some((spread, spread_pct))  // Return tuple (copied)
    }

    fn print_book(&self) {
        println!("╔═══════════════════════════════════════╗");
        println!("║     ORDER BOOK: {:^20} ║", self.symbol);
        println!("╠═══════════════════════════════════════╣");
        println!("║  ASKS (Sell orders)                   ║");
        for (price, qty) in self.asks.iter().rev().take(3) {
            println!("║  ${:>10.2} | {:>8.4} BTC         ║", price, qty);
        }
        println!("╠═══════════════════════════════════════╣");
        println!("║  BIDS (Buy orders)                    ║");
        for (price, qty) in self.bids.iter().take(3) {
            println!("║  ${:>10.2} | {:>8.4} BTC         ║", price, qty);
        }
        println!("╚═══════════════════════════════════════╝");
    }
}
```

## Why Does This Matter for Trading?

### 1. Performance is Critical

```rust
fn main() {
    // Fast: data on stack
    let prices: [f64; 1000] = [42000.0; 1000];
    let sum_stack: f64 = prices.iter().sum();

    // Slower: data on heap (but flexible)
    let prices_vec: Vec<f64> = vec![42000.0; 1000];
    let sum_heap: f64 = prices_vec.iter().sum();

    println!("Stack sum: {}", sum_stack);
    println!("Heap sum: {}", sum_heap);
}
```

### 2. Choosing Data Structures

```rust
fn main() {
    // If size is known in advance — use array (stack)
    let ohlc: [f64; 4] = [42000.0, 42500.0, 41800.0, 42200.0];

    // If size changes — use Vec (heap)
    let mut trade_history: Vec<Trade> = Vec::new();
    trade_history.push(Trade::new(42000.0, 0.5, true));
    trade_history.push(Trade::new(42100.0, 0.3, false));

    // If you need a fixed string — &str (stack/static memory)
    let symbol: &str = "BTC";

    // If string is created dynamically — String (heap)
    let full_symbol = format!("{}/USDT", symbol);

    println!("OHLC: {:?}", ohlc);
    println!("Symbol: {}", full_symbol);
}

struct Trade {
    price: f64,
    quantity: f64,
    is_buy: bool,
}

impl Trade {
    fn new(price: f64, quantity: f64, is_buy: bool) -> Self {
        Trade { price, quantity, is_buy }
    }
}
```

### 3. Working with Large Data

```rust
fn main() {
    // For large structures use Box (heap allocation)
    let market_data = Box::new(MarketData::new("BTC/USDT"));

    println!("Symbol: {}", market_data.symbol);
    println!("Candle count: {}", market_data.candles.len());
}

struct MarketData {
    symbol: String,
    candles: Vec<Candle>,
    order_book: OrderBookSnapshot,
    trades: Vec<TradeRecord>,
}

struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

struct OrderBookSnapshot {
    bids: Vec<(f64, f64)>,
    asks: Vec<(f64, f64)>,
}

struct TradeRecord {
    price: f64,
    quantity: f64,
    is_buyer_maker: bool,
    timestamp: u64,
}

impl MarketData {
    fn new(symbol: &str) -> Self {
        MarketData {
            symbol: String::from(symbol),
            candles: Vec::new(),
            order_book: OrderBookSnapshot {
                bids: Vec::new(),
                asks: Vec::new(),
            },
            trades: Vec::new(),
        }
    }
}
```

## Comparison: When to Use What

```rust
fn main() {
    // ✅ Stack: simple calculations
    let entry_price: f64 = 42000.0;
    let exit_price: f64 = 43000.0;
    let pnl = exit_price - entry_price;

    // ✅ Stack: fixed data
    let ohlc: [f64; 4] = [42000.0, 42500.0, 41800.0, 42200.0];

    // ✅ Heap: dynamic list
    let mut positions: Vec<Position> = Vec::new();
    positions.push(Position { symbol: String::from("BTC"), size: 0.5 });

    // ✅ Heap: strings created at runtime
    let report = generate_report(&positions);

    println!("PnL: {}", pnl);
    println!("Report: {}", report);
}

struct Position {
    symbol: String,
    size: f64,
}

fn generate_report(positions: &[Position]) -> String {
    let mut report = String::from("Portfolio:\n");
    for pos in positions {
        report.push_str(&format!("  {} : {}\n", pos.symbol, pos.size));
    }
    report
}
```

## What We Learned

| Characteristic | Stack | Heap |
|----------------|-------|------|
| Analogy | Day Trading | Long-term positions |
| Speed | Very fast | Slower |
| Data size | Known at compile time | Dynamic |
| Lifetime | Until end of scope | While owner exists |
| Types | i32, f64, bool, [T; N] | String, Vec<T>, Box<T> |
| Management | Automatic | Through ownership |

## Homework

1. Create a `Portfolio` struct that stores a list of positions on the heap (Vec) and calculates total value using local variables on the stack

2. Write a function that takes a fixed-size price array `[f64; 10]` (stack) and returns `Vec<f64>` with moving averages (heap)

3. Implement a `TradeJournal` that:
   - Stores trade history in `Vec<Trade>` (heap)
   - Uses local variables for calculations (stack)
   - Returns a `String` with a report (heap)

4. Compare performance: create an array `[f64; 10000]` on stack and `Vec<f64>` with 10000 elements on heap, measure summation time

## Navigation

[← Previous day](../031-ownership-asset-ownership/en.md) | [Next day →](../033-references-borrowing/en.md)
