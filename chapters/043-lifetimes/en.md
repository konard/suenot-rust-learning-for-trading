# Day 43: Lifetimes — How Long Does an Order Live?

## Trading Analogy

Think of an exchange as a city with orders as residents. Each order has a **lifespan** — from creation to execution or cancellation. A limit order can live for seconds (scalping) or days (swing trading). GTC (Good Till Cancelled) lives until manually cancelled, while IOC (Immediate Or Cancel) dies instantly if not filled.

In Rust, **lifetimes** are how the compiler tracks how long references remain valid. Just as a trader must verify an order is still active before modifying it, Rust verifies that references point to living data.

## The Problem: Dangling References

```rust
fn main() {
    let order_ref;

    {
        let order = String::from("BUY BTC 42000");
        order_ref = &order;  // order_ref borrows order
    }  // order is destroyed here!

    // println!("{}", order_ref);  // ERROR! order no longer exists
}
```

This is like trying to cancel an order that's already been filled — it no longer exists in the system.

## Basic Lifetime Syntax

```rust
fn main() {
    let ticker = String::from("BTCUSDT");
    let result = get_first_char(&ticker);
    println!("First character: {}", result);
}

// 'a is a lifetime parameter
// It says: the returned reference lives as long as the input
fn get_first_char<'a>(s: &'a str) -> &'a str {
    &s[0..1]
}
```

**'a** (pronounced "lifetime a") is an annotation that links the lifetimes of input and output references.

## Lifetimes in Structs

```rust
// Structs containing references must have lifetime parameters
struct OrderBook<'a> {
    symbol: &'a str,
    best_bid: f64,
    best_ask: f64,
}

struct TradeContext<'a> {
    account_id: &'a str,
    active_orders: &'a [Order],
    market_data: &'a MarketData,
}

#[derive(Debug)]
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
}

struct MarketData {
    last_price: f64,
    volume: f64,
}

fn main() {
    let symbol = String::from("ETHUSDT");

    let book = OrderBook {
        symbol: &symbol,
        best_bid: 2350.00,
        best_ask: 2350.50,
    };

    println!("Order book {}: bid={}, ask={}",
             book.symbol, book.best_bid, book.best_ask);
}
```

## Multiple Lifetimes

```rust
fn main() {
    let ticker1 = String::from("BTCUSDT");
    let ticker2 = String::from("ETHUSDT");

    let longer = longest_ticker(&ticker1, &ticker2);
    println!("Longer ticker: {}", longer);
}

// Both references must live at least as long as 'a
fn longest_ticker<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

### Different Lifetimes for Different Parameters

```rust
fn main() {
    let market = String::from("BINANCE");

    {
        let symbol = String::from("BTCUSDT");
        let result = format_ticker(&market, &symbol);
        println!("{}", result);
    }  // symbol dies, but market keeps living

    println!("Market {} is still accessible", market);
}

// Different lifetimes for different parameters
fn format_ticker<'a, 'b>(market: &'a str, symbol: &'b str) -> String {
    format!("{}:{}", market, symbol)
}
```

## Elision — When Lifetimes Can Be Omitted

Rust automatically infers lifetimes in simple cases:

```rust
// These declarations are equivalent:

// With explicit lifetime
fn get_symbol_explicit<'a>(ticker: &'a str) -> &'a str {
    ticker
}

// Lifetime elided
fn get_symbol(ticker: &str) -> &str {
    ticker
}

// Elision rules:
// 1. Each reference parameter gets its own lifetime
// 2. If there's one input lifetime — it applies to output
// 3. If there's &self or &mut self — its lifetime applies to output
```

## 'static — The Eternal Lifetime

```rust
// 'static means: data lives for the entire program duration
const EXCHANGE: &'static str = "BINANCE";
static DEFAULT_SYMBOL: &'static str = "BTCUSDT";

fn main() {
    let s: &'static str = "This is a string literal — always 'static";

    println!("Exchange: {}", EXCHANGE);
    println!("Default symbol: {}", DEFAULT_SYMBOL);
}

// Function returning a 'static reference
fn get_default_market() -> &'static str {
    "SPOT"
}
```

## Practical Example: Order Management System

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    status: OrderStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancelled,
}

// OrderManager holds a reference to the order list
struct OrderManager<'a> {
    orders: &'a mut Vec<Order>,
    max_orders: usize,
}

impl<'a> OrderManager<'a> {
    fn new(orders: &'a mut Vec<Order>, max_orders: usize) -> Self {
        OrderManager { orders, max_orders }
    }

    // Returned reference lives as long as self
    fn find_order(&self, id: u64) -> Option<&Order> {
        self.orders.iter().find(|o| o.id == id)
    }

    fn find_order_mut(&mut self, id: u64) -> Option<&mut Order> {
        self.orders.iter_mut().find(|o| o.id == id)
    }

    fn get_active_orders(&self) -> Vec<&Order> {
        self.orders
            .iter()
            .filter(|o| o.status == OrderStatus::New || o.status == OrderStatus::PartiallyFilled)
            .collect()
    }

    fn cancel_order(&mut self, id: u64) -> Result<(), &'static str> {
        match self.find_order_mut(id) {
            Some(order) => {
                if order.status == OrderStatus::Filled {
                    Err("Cannot cancel filled order")
                } else {
                    order.status = OrderStatus::Cancelled;
                    Ok(())
                }
            }
            None => Err("Order not found"),
        }
    }

    fn add_order(&mut self, order: Order) -> Result<(), &'static str> {
        if self.orders.len() >= self.max_orders {
            return Err("Maximum orders reached");
        }
        self.orders.push(order);
        Ok(())
    }
}

fn main() {
    let mut orders = vec![
        Order {
            id: 1,
            symbol: String::from("BTCUSDT"),
            side: OrderSide::Buy,
            price: 42000.0,
            quantity: 0.5,
            status: OrderStatus::New,
        },
        Order {
            id: 2,
            symbol: String::from("ETHUSDT"),
            side: OrderSide::Sell,
            price: 2400.0,
            quantity: 10.0,
            status: OrderStatus::PartiallyFilled,
        },
    ];

    let mut manager = OrderManager::new(&mut orders, 100);

    // Find order
    if let Some(order) = manager.find_order(1) {
        println!("Found order: {:?}", order);
    }

    // Get active orders
    let active = manager.get_active_orders();
    println!("Active orders: {}", active.len());

    // Cancel order
    match manager.cancel_order(1) {
        Ok(()) => println!("Order cancelled"),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Lifetime Bounds in Generics

```rust
use std::fmt::Display;

// T must live at least as long as 'a
fn print_with_context<'a, T: Display + 'a>(value: &'a T, context: &str) {
    println!("[{}] {}", context, value);
}

// Struct with generic type and lifetime
struct PriceCache<'a, T> {
    prices: &'a [T],
    last_update: u64,
}

impl<'a, T: Display + Copy> PriceCache<'a, T> {
    fn new(prices: &'a [T]) -> Self {
        PriceCache {
            prices,
            last_update: 0,
        }
    }

    fn get_latest(&self) -> Option<&T> {
        self.prices.last()
    }

    fn print_all(&self) {
        for price in self.prices {
            println!("Price: {}", price);
        }
    }
}

fn main() {
    let prices = [42000.0, 42100.0, 42050.0, 42200.0];
    let cache = PriceCache::new(&prices);

    if let Some(latest) = cache.get_latest() {
        println!("Latest price: {}", latest);
    }

    cache.print_all();
}
```

## Market Data Analyzer with Lifetimes

```rust
#[derive(Debug)]
struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

struct MarketAnalyzer<'a> {
    candles: &'a [Candle],
    symbol: &'a str,
}

impl<'a> MarketAnalyzer<'a> {
    fn new(candles: &'a [Candle], symbol: &'a str) -> Self {
        MarketAnalyzer { candles, symbol }
    }

    // Find candle with maximum volume
    fn find_highest_volume(&self) -> Option<&Candle> {
        self.candles
            .iter()
            .max_by(|a, b| a.volume.partial_cmp(&b.volume).unwrap())
    }

    // Find candles above given price
    fn find_above_price(&self, price: f64) -> Vec<&Candle> {
        self.candles
            .iter()
            .filter(|c| c.close > price)
            .collect()
    }

    // Calculate SMA for last N candles
    fn calculate_sma(&self, period: usize) -> Option<f64> {
        if self.candles.len() < period {
            return None;
        }

        let sum: f64 = self.candles
            .iter()
            .rev()
            .take(period)
            .map(|c| c.close)
            .sum();

        Some(sum / period as f64)
    }

    // Determine trend
    fn get_trend(&self) -> &'static str {
        if self.candles.len() < 2 {
            return "UNDEFINED";
        }

        let first = self.candles.first().unwrap().close;
        let last = self.candles.last().unwrap().close;

        if last > first * 1.01 {
            "UPTREND"
        } else if last < first * 0.99 {
            "DOWNTREND"
        } else {
            "SIDEWAYS"
        }
    }
}

fn main() {
    let candles = vec![
        Candle { timestamp: 1, open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0, volume: 1000.0 },
        Candle { timestamp: 2, open: 42300.0, high: 42800.0, low: 42200.0, close: 42700.0, volume: 1500.0 },
        Candle { timestamp: 3, open: 42700.0, high: 43000.0, low: 42500.0, close: 42900.0, volume: 1200.0 },
        Candle { timestamp: 4, open: 42900.0, high: 43200.0, low: 42800.0, close: 43100.0, volume: 2000.0 },
    ];

    let symbol = "BTCUSDT";
    let analyzer = MarketAnalyzer::new(&candles, symbol);

    println!("=== {} Analysis ===", symbol);

    if let Some(candle) = analyzer.find_highest_volume() {
        println!("Highest volume candle: close={}, volume={}",
                 candle.close, candle.volume);
    }

    if let Some(sma) = analyzer.calculate_sma(3) {
        println!("SMA(3): {:.2}", sma);
    }

    println!("Trend: {}", analyzer.get_trend());

    let above_42500 = analyzer.find_above_price(42500.0);
    println!("Candles above 42500: {}", above_42500.len());
}
```

## Common Errors and Solutions

### Error 1: Returning Reference to Local Data

```rust
// WRONG — does not compile
// fn create_order_ref() -> &Order {
//     let order = Order { ... };
//     &order  // order will die when function exits!
// }

// CORRECT — return ownership
fn create_order() -> Order {
    Order {
        id: 1,
        symbol: String::from("BTCUSDT"),
        side: OrderSide::Buy,
        price: 42000.0,
        quantity: 1.0,
        status: OrderStatus::New,
    }
}
```

### Error 2: Conflicting Lifetimes

```rust
fn main() {
    let ticker1 = String::from("BTCUSDT");
    let result;

    {
        let ticker2 = String::from("ETHUSDT");
        // result = longest_ticker(&ticker1, &ticker2);
        // ERROR! result cannot outlive ticker2
    }

    // Solution: ensure both references live long enough
    let ticker2 = String::from("ETHUSDT");
    result = longest_ticker(&ticker1, &ticker2);
    println!("{}", result);
}

fn longest_ticker<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

## Exercises

### Exercise 1: Trade Signal Parser

```rust
// Implement a SignalParser struct that holds a reference to a signal string
// and provides methods to extract signal parts

struct SignalParser<'a> {
    signal: &'a str,
}

impl<'a> SignalParser<'a> {
    fn new(signal: &'a str) -> Self {
        // Implement
        todo!()
    }

    // Extract action (BUY/SELL) from the beginning of string
    fn get_action(&self) -> &str {
        // Implement
        todo!()
    }

    // Extract symbol (e.g., BTCUSDT)
    fn get_symbol(&self) -> &str {
        // Implement
        todo!()
    }
}

fn main() {
    let signal = "BUY BTCUSDT 42000";
    let parser = SignalParser::new(signal);

    println!("Action: {}", parser.get_action());
    println!("Symbol: {}", parser.get_symbol());
}
```

### Exercise 2: Quote Cache

```rust
// Create a QuoteCache struct that holds references to bid and ask prices
// Implement methods for calculating spread and mid-price

struct QuoteCache<'a> {
    bids: &'a [f64],
    asks: &'a [f64],
}

impl<'a> QuoteCache<'a> {
    fn new(bids: &'a [f64], asks: &'a [f64]) -> Self {
        // Implement
        todo!()
    }

    fn best_bid(&self) -> Option<f64> {
        // Implement: return maximum bid
        todo!()
    }

    fn best_ask(&self) -> Option<f64> {
        // Implement: return minimum ask
        todo!()
    }

    fn spread(&self) -> Option<f64> {
        // Implement: best_ask - best_bid
        todo!()
    }

    fn mid_price(&self) -> Option<f64> {
        // Implement: (best_bid + best_ask) / 2
        todo!()
    }
}
```

### Exercise 3: Trade Log

```rust
// Create a TradeLog struct with a method that returns
// a reference to the last profitable trade

#[derive(Debug)]
struct Trade {
    id: u64,
    pnl: f64,
}

struct TradeLog<'a> {
    trades: &'a [Trade],
}

impl<'a> TradeLog<'a> {
    fn new(trades: &'a [Trade]) -> Self {
        // Implement
        todo!()
    }

    // Return reference to last profitable trade
    fn last_profitable(&self) -> Option<&Trade> {
        // Implement
        todo!()
    }

    // Return all profitable trades
    fn all_profitable(&self) -> Vec<&Trade> {
        // Implement
        todo!()
    }
}
```

### Exercise 4: Position Filter

```rust
// Create a function that takes a slice of positions and returns
// only open positions

#[derive(Debug)]
struct Position {
    symbol: String,
    size: f64,
    is_open: bool,
}

// Implement the function with correct lifetime annotations
fn filter_open_positions(/* parameters */) -> Vec<&Position> {
    // Implement
    todo!()
}
```

## What We Learned

| Concept | Syntax | Purpose |
|---------|--------|---------|
| Lifetime parameter | `'a` | Links reference lifetimes |
| Lifetime in function | `fn foo<'a>(x: &'a T) -> &'a T` | Input-output connection |
| Lifetime in struct | `struct Foo<'a> { x: &'a T }` | Struct with references |
| Static lifetime | `'static` | Lives for entire program |
| Lifetime bounds | `T: 'a` | T must live at least as long as 'a |
| Elision | Omitting `'a` | Automatic inference |

## Homework

1. **Order Monitoring System**: Create an `OrderMonitor<'a>` struct that holds a reference to an order list and provides methods for:
   - Finding orders by symbol
   - Counting total volume by side (buy/sell)
   - Finding the order with maximum price

2. **Order Book Analyzer**: Implement `OrderBookAnalyzer<'a>` with methods:
   - `imbalance()` — bid volume to ask volume ratio
   - `spread_percent()` — spread as percentage
   - `depth_at_level(n)` — total volume at first n levels

3. **Portfolio Tracker**: Create `PortfolioView<'a>` that:
   - Takes a reference to a position list
   - Calculates total PnL
   - Finds most profitable/losing position
   - Returns positions by given symbol

4. **Signal Validator**: Write a function `validate_signals<'a>(signals: &'a [Signal], rules: &Rules) -> Vec<&'a Signal>` that returns only valid signals.

## Navigation

[← Previous day](../042-borrowing-data-access/en.md) | [Next day →](../044-lifetime-elision/en.md)
