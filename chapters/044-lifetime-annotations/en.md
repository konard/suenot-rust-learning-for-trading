# Day 44: Lifetime Annotations — `'a` in Functions

## Trading Analogy

Imagine working with **trading sessions**. When you analyze orders within a session, those orders exist only while the session is open. You can't use an order from a closed session — it's already invalid.

In Rust, **lifetime** is a way to tell the compiler: "This reference is valid as long as the original object is alive." The `'a` annotation is a label that links lifetimes of references together.

## Why Do We Need Lifetimes?

Rust guarantees memory safety without a garbage collector. The compiler must know that all references are valid. When a function takes or returns references, you need to specify how they're connected.

```rust
// This function won't compile without lifetime annotations!
// fn get_best_price(bid: &f64, ask: &f64) -> &f64 {
//     if bid > ask { bid } else { ask }
// }

// Correct: linking lifetimes
fn get_best_price<'a>(bid: &'a f64, ask: &'a f64) -> &'a f64 {
    if bid > ask { bid } else { ask }
}

fn main() {
    let bid = 42000.0;
    let ask = 42100.0;
    let best = get_best_price(&bid, &ask);
    println!("Best price: ${}", best);
}
```

## Annotation Syntax

```rust
// 'a is the lifetime name (can be anything: 'b, 'price, 'session)
fn function_name<'a>(param: &'a Type) -> &'a Type {
    // ...
}
```

**Breakdown:**
- `<'a>` — declare a lifetime parameter (like a generic type)
- `&'a Type` — a reference with lifetime `'a`
- The returned reference lives as long as the input parameter

## Example: Choosing the Best Price

```rust
fn main() {
    let btc_price = 42000.0;
    let eth_price = 2800.0;

    let higher = get_higher_price(&btc_price, &eth_price);
    println!("Higher price: ${}", higher);

    // higher is still valid while btc_price and eth_price are in scope
}

fn get_higher_price<'a>(price1: &'a f64, price2: &'a f64) -> &'a f64 {
    if price1 > price2 {
        price1
    } else {
        price2
    }
}
```

## Different Lifetimes for Different Parameters

Sometimes you need to distinguish lifetimes of different parameters:

```rust
fn main() {
    let session_name = String::from("NYSE Morning");
    let prices = vec![42000.0, 42100.0, 42050.0];

    let report = create_price_report(&session_name, &prices);
    println!("{}", report);
}

// Here 'a and 'b are different lifetimes
// The returned string lives as long as session
fn create_price_report<'a, 'b>(session: &'a str, prices: &'b [f64]) -> &'a str {
    println!("Prices in session: {:?}", prices);
    session  // Return reference to session
}
```

## Lifetimes in Structs

If a struct contains references, you need to specify their lifetime:

```rust
// Struct with a reference to price data
struct PriceSnapshot<'a> {
    symbol: &'a str,
    price: &'a f64,
    timestamp: u64,
}

fn main() {
    let symbol = String::from("BTC/USDT");
    let price = 42000.0;

    let snapshot = PriceSnapshot {
        symbol: &symbol,
        price: &price,
        timestamp: 1704067200,
    };

    println!("{} @ ${}", snapshot.symbol, snapshot.price);
}
```

## Practical Example: Trading Data Analysis

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42100.0];
    let volumes = [100.0, 150.0, 80.0, 120.0, 90.0];

    // Get OHLC data
    let (open, high, low, close) = get_ohlc(&prices);
    println!("Open: {}, High: {}, Low: {}, Close: {}", open, high, low, close);

    // Find price with highest volume
    if let Some((price, volume)) = find_highest_volume_price(&prices, &volumes) {
        println!("Highest volume: {} at price ${}", volume, price);
    }
}

fn get_ohlc<'a>(prices: &'a [f64]) -> (&'a f64, &'a f64, &'a f64, &'a f64) {
    let open = &prices[0];
    let close = &prices[prices.len() - 1];

    let mut high = &prices[0];
    let mut low = &prices[0];

    for price in prices {
        if price > high { high = price; }
        if price < low { low = price; }
    }

    (open, high, low, close)
}

fn find_highest_volume_price<'a>(
    prices: &'a [f64],
    volumes: &'a [f64]
) -> Option<(&'a f64, &'a f64)> {
    if prices.is_empty() || volumes.is_empty() {
        return None;
    }

    let mut max_idx = 0;
    let mut max_vol = volumes[0];

    for (idx, &vol) in volumes.iter().enumerate() {
        if vol > max_vol {
            max_vol = vol;
            max_idx = idx;
        }
    }

    Some((&prices[max_idx], &volumes[max_idx]))
}
```

## Elision Rules — When Lifetimes Can Be Omitted

Rust has "elision" rules — when the compiler infers lifetimes automatically:

```rust
// Rule 1: One input parameter — lifetime is copied to output
fn get_first_price(prices: &[f64]) -> &f64 {
    &prices[0]
}
// Equivalent: fn get_first_price<'a>(prices: &'a [f64]) -> &'a f64

// Rule 2: Method with &self — self's lifetime is copied to output
struct Portfolio {
    name: String,
    balance: f64,
}

impl Portfolio {
    fn get_name(&self) -> &str {
        &self.name
    }
    // Equivalent: fn get_name<'a>(&'a self) -> &'a str
}

fn main() {
    let prices = [42000.0, 42100.0];
    println!("First: {}", get_first_price(&prices));

    let portfolio = Portfolio {
        name: String::from("Main"),
        balance: 10000.0,
    };
    println!("Portfolio: {}", portfolio.get_name());
}
```

## Static Lifetime `'static`

`'static` means the data lives for the entire program runtime:

```rust
fn main() {
    // String literals have 'static lifetime
    let default_symbol: &'static str = "BTC/USDT";

    let symbol = get_default_symbol();
    println!("Default symbol: {}", symbol);
}

fn get_default_symbol() -> &'static str {
    "BTC/USDT"  // String literal — always 'static
}

// Example with constants
const EXCHANGE_NAME: &str = "Binance";  // Implicitly 'static
static MAX_LEVERAGE: f64 = 100.0;       // Static variable
```

## Complex Example: Order Filtering

```rust
#[derive(Debug)]
struct Order<'a> {
    id: u64,
    symbol: &'a str,
    price: f64,
    quantity: f64,
    side: &'a str,
}

fn main() {
    let btc = "BTC/USDT";
    let eth = "ETH/USDT";
    let buy = "BUY";
    let sell = "SELL";

    let orders = vec![
        Order { id: 1, symbol: btc, price: 42000.0, quantity: 0.5, side: buy },
        Order { id: 2, symbol: eth, price: 2800.0, quantity: 5.0, side: sell },
        Order { id: 3, symbol: btc, price: 42100.0, quantity: 0.3, side: buy },
        Order { id: 4, symbol: btc, price: 41900.0, quantity: 0.2, side: sell },
    ];

    let btc_orders = filter_by_symbol(&orders, btc);
    println!("BTC orders: {:?}", btc_orders);

    let buy_orders = filter_by_side(&orders, buy);
    println!("Buy orders: {:?}", buy_orders);

    if let Some(best) = find_best_bid(&orders, btc) {
        println!("Best BTC bid: ${} for {} units", best.price, best.quantity);
    }
}

fn filter_by_symbol<'a, 'b>(
    orders: &'a [Order<'b>],
    symbol: &str
) -> Vec<&'a Order<'b>> {
    orders.iter()
        .filter(|o| o.symbol == symbol)
        .collect()
}

fn filter_by_side<'a, 'b>(
    orders: &'a [Order<'b>],
    side: &str
) -> Vec<&'a Order<'b>> {
    orders.iter()
        .filter(|o| o.side == side)
        .collect()
}

fn find_best_bid<'a, 'b>(
    orders: &'a [Order<'b>],
    symbol: &str
) -> Option<&'a Order<'b>> {
    orders.iter()
        .filter(|o| o.symbol == symbol && o.side == "BUY")
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
}
```

## Common Errors and Solutions

### Error: Returning Reference to Local Variable

```rust
// WON'T COMPILE!
// fn calculate_and_return() -> &f64 {
//     let result = 42.0;
//     &result  // result will be destroyed when function returns!
// }

// Solution: return value, not reference
fn calculate_and_return() -> f64 {
    let result = 42.0;
    result
}

fn main() {
    let value = calculate_and_return();
    println!("Value: {}", value);
}
```

### Error: Incompatible Lifetimes

```rust
// WON'T COMPILE!
// fn wrong_lifetime<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
//     if x.len() > y.len() { x } else { y }  // y has lifetime 'b, not 'a!
// }

// Solution: use the same lifetime
fn correct_lifetime<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let x = "Bitcoin";
    let y = "Ethereum";
    println!("Longer: {}", correct_lifetime(x, y));
}
```

## Practical Exercises

### Exercise 1: Find Minimum and Maximum Price

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0];

    // Implement the function
    let (min, max) = find_min_max(&prices);
    println!("Min: {}, Max: {}", min, max);
}

// TODO: Add lifetime annotations
fn find_min_max(prices: &[f64]) -> (&f64, &f64) {
    let mut min = &prices[0];
    let mut max = &prices[0];

    for price in prices {
        if price < min { min = price; }
        if price > max { max = price; }
    }

    (min, max)
}
```

### Exercise 2: Struct with References

```rust
// TODO: Add lifetime parameter
struct Trade {
    symbol: &str,
    entry_price: &f64,
    exit_price: &f64,
}

impl Trade {
    // TODO: Add lifetime annotations
    fn pnl(&self) -> f64 {
        self.exit_price - self.entry_price
    }

    fn get_symbol(&self) -> &str {
        self.symbol
    }
}

fn main() {
    let sym = String::from("BTC/USDT");
    let entry = 42000.0;
    let exit = 43500.0;

    let trade = Trade {
        symbol: &sym,
        entry_price: &entry,
        exit_price: &exit,
    };

    println!("{}: PnL = ${}", trade.get_symbol(), trade.pnl());
}
```

### Exercise 3: Filtering with Lifetimes

```rust
// TODO: Implement filter_profitable_trades function
// Should return a vector of references to profitable trades

struct TradeResult<'a> {
    id: u64,
    symbol: &'a str,
    pnl: f64,
}

fn main() {
    let btc = "BTC";
    let eth = "ETH";

    let trades = vec![
        TradeResult { id: 1, symbol: btc, pnl: 500.0 },
        TradeResult { id: 2, symbol: eth, pnl: -200.0 },
        TradeResult { id: 3, symbol: btc, pnl: 300.0 },
        TradeResult { id: 4, symbol: eth, pnl: -50.0 },
    ];

    // TODO: Implement this function
    // let profitable = filter_profitable_trades(&trades);
    // println!("Profitable trades: {}", profitable.len());
}
```

## What We Learned

| Concept | Syntax | Description |
|---------|--------|-------------|
| Lifetime parameter | `<'a>` | Declare lifetime in signature |
| Reference annotation | `&'a T` | Reference with specific lifetime |
| Multiple lifetimes | `<'a, 'b>` | Different lifetimes for different data |
| Struct lifetime | `struct Foo<'a>` | Struct contains references |
| Static lifetime | `'static` | Data lives forever |
| Elision | (automatic) | Compiler infers lifetime |

## Homework

1. **Write a function** `get_best_order<'a>(orders: &'a [Order], side: &str) -> Option<&'a Order>` — returns the best order (maximum price for BUY, minimum for SELL)

2. **Create a struct** `TradingSession<'a>` with fields `name: &'a str`, `orders: Vec<&'a Order>`. Add methods for working with the session

3. **Implement a function** `merge_price_data<'a>(prices1: &'a [f64], prices2: &'a [f64]) -> Vec<&'a f64>` — merges two collections of price references

4. **Advanced task:** Create a trading log parser that returns references to found symbols and prices without copying data

## Navigation

[← Previous day](../043-lifetime-basics/en.md) | [Next day →](../045-lifetime-structs/en.md)
