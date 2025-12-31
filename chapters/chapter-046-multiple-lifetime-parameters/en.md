# Day 46: Multiple Lifetime Parameters

## Trading Analogy

Imagine you're analyzing a trade using data from two different sources:
- **Exchange data** (quotes, volumes) — updates in real-time
- **Historical data** (trade archive) — stored long-term

This data has **different lifetimes**: exchange data lives while the connection is open, while historical data lives for the program's duration. When a function works with both sources simultaneously, Rust requires you to explicitly specify which data lives longer.

## Why Do We Need Multiple Lifetime Parameters?

When a function takes multiple references and returns a reference, the compiler needs to know: which input parameter is the returned reference tied to?

```rust
// One lifetime parameter is enough when all references are tied equally
fn get_best_price<'a>(bid: &'a f64, ask: &'a f64) -> &'a f64 {
    if bid > ask { bid } else { ask }
}

fn main() {
    let bid = 42000.0;
    let ask = 42050.0;
    let best = get_best_price(&bid, &ask);
    println!("Best price: {}", best);
}
```

But what if references have **different lifetimes**?

## Two Lifetime Parameters

```rust
// Function compares current price with historical
// Returns reference ONLY to current price
fn compare_with_history<'current, 'history>(
    current_price: &'current f64,
    historical_price: &'history f64,
) -> &'current f64 {
    println!("Historical: ${:.2}, Current: ${:.2}", historical_price, current_price);
    current_price  // Return only current price
}

fn main() {
    let current = 42500.0;

    let result = {
        let historical = 41000.0;  // Lives only in this block
        compare_with_history(&current, &historical)
    };  // historical is destroyed here

    // result is still valid because it's tied to current, not historical
    println!("Result: ${:.2}", result);
}
```

## Practical Example: Order Analyzer

```rust
#[derive(Debug)]
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct MarketData {
    bid: f64,
    ask: f64,
    last_price: f64,
}

// Function takes order and market data with different lifetimes
// Returns reference to price from market data
fn get_execution_price<'order, 'market>(
    order: &'order Order,
    market: &'market MarketData,
) -> &'market f64 {
    // Execution price logic
    if order.quantity > 0.0 {
        // Buy executes at ask
        &market.ask
    } else {
        // Sell executes at bid
        &market.bid
    }
}

fn main() {
    let market_data = MarketData {
        bid: 41950.0,
        ask: 42050.0,
        last_price: 42000.0,
    };

    let execution_price = {
        let buy_order = Order {
            symbol: String::from("BTC/USDT"),
            price: 42000.0,
            quantity: 0.5,
        };

        get_execution_price(&buy_order, &market_data)
    };  // buy_order is destroyed, but execution_price is valid

    println!("Execution price: ${:.2}", execution_price);
}
```

## Choosing the Return Lifetime

```rust
struct PriceLevel {
    price: f64,
    volume: f64,
}

struct OrderBook {
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
}

// Return reference from first OR second argument
// Both must live long enough
fn get_best_level<'a>(
    order_book: &'a OrderBook,
    side: &str,
) -> Option<&'a PriceLevel> {
    match side {
        "buy" => order_book.asks.first(),
        "sell" => order_book.bids.first(),
        _ => None,
    }
}

// Compare two order books, return best price from each
// Here we need TWO lifetime parameters
fn compare_order_books<'a, 'b>(
    book_a: &'a OrderBook,
    book_b: &'b OrderBook,
) -> (Option<&'a PriceLevel>, Option<&'b PriceLevel>) {
    let best_a = book_a.bids.first();
    let best_b = book_b.bids.first();
    (best_a, best_b)
}

fn main() {
    let book1 = OrderBook {
        bids: vec![PriceLevel { price: 42000.0, volume: 1.5 }],
        asks: vec![PriceLevel { price: 42050.0, volume: 2.0 }],
    };

    let book2 = OrderBook {
        bids: vec![PriceLevel { price: 41990.0, volume: 3.0 }],
        asks: vec![PriceLevel { price: 42060.0, volume: 1.0 }],
    };

    let (best1, best2) = compare_order_books(&book1, &book2);

    if let (Some(b1), Some(b2)) = (best1, best2) {
        println!("Book1 best bid: ${:.2}", b1.price);
        println!("Book2 best bid: ${:.2}", b2.price);
    }
}
```

## Structs with Multiple Lifetimes

```rust
// Analyzer stores references to data with DIFFERENT lifetimes
struct TradeAnalyzer<'current, 'historical> {
    current_data: &'current [f64],
    historical_data: &'historical [f64],
}

impl<'current, 'historical> TradeAnalyzer<'current, 'historical> {
    fn new(
        current: &'current [f64],
        historical: &'historical [f64],
    ) -> Self {
        TradeAnalyzer {
            current_data: current,
            historical_data: historical,
        }
    }

    // Returns reference only to current data
    fn get_current_high(&self) -> Option<&'current f64> {
        self.current_data.iter().max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    // Returns reference only to historical data
    fn get_historical_average(&self) -> f64 {
        if self.historical_data.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.historical_data.iter().sum();
        sum / self.historical_data.len() as f64
    }

    // Compares current data with historical
    fn analyze(&self) {
        let current_high = self.get_current_high().unwrap_or(&0.0);
        let historical_avg = self.get_historical_average();

        println!("Current High: ${:.2}", current_high);
        println!("Historical Avg: ${:.2}", historical_avg);

        let diff_percent = ((current_high - historical_avg) / historical_avg) * 100.0;
        println!("Difference: {:.2}%", diff_percent);
    }
}

fn main() {
    // Historical data lives for program's lifetime
    let historical_prices = vec![40000.0, 41000.0, 39500.0, 42000.0, 41500.0];

    // Current data can be updated
    let current_prices = vec![42500.0, 42600.0, 42400.0, 42550.0];

    let analyzer = TradeAnalyzer::new(&current_prices, &historical_prices);
    analyzer.analyze();
}
```

## Practical Example: Portfolio Comparison

```rust
#[derive(Debug)]
struct Portfolio {
    name: String,
    assets: Vec<(String, f64)>,  // (ticker, amount)
}

#[derive(Debug)]
struct PortfolioComparison<'a, 'b> {
    portfolio_a: &'a Portfolio,
    portfolio_b: &'b Portfolio,
}

impl<'a, 'b> PortfolioComparison<'a, 'b> {
    fn new(a: &'a Portfolio, b: &'b Portfolio) -> Self {
        PortfolioComparison {
            portfolio_a: a,
            portfolio_b: b,
        }
    }

    fn total_value(portfolio: &Portfolio) -> f64 {
        portfolio.assets.iter().map(|(_, value)| value).sum()
    }

    fn get_larger(&self) -> &str {
        let value_a = Self::total_value(self.portfolio_a);
        let value_b = Self::total_value(self.portfolio_b);

        if value_a >= value_b {
            &self.portfolio_a.name
        } else {
            &self.portfolio_b.name
        }
    }

    fn compare(&self) {
        let value_a = Self::total_value(self.portfolio_a);
        let value_b = Self::total_value(self.portfolio_b);

        println!("=== Portfolio Comparison ===");
        println!("{}: ${:.2}", self.portfolio_a.name, value_a);
        println!("{}: ${:.2}", self.portfolio_b.name, value_b);
        println!("Larger: {}", self.get_larger());

        let diff = (value_a - value_b).abs();
        let diff_percent = (diff / value_a.max(value_b)) * 100.0;
        println!("Difference: ${:.2} ({:.2}%)", diff, diff_percent);
    }
}

fn main() {
    let conservative = Portfolio {
        name: String::from("Conservative"),
        assets: vec![
            (String::from("BTC"), 50000.0),
            (String::from("ETH"), 30000.0),
            (String::from("USDT"), 20000.0),
        ],
    };

    let aggressive = Portfolio {
        name: String::from("Aggressive"),
        assets: vec![
            (String::from("BTC"), 80000.0),
            (String::from("SOL"), 15000.0),
            (String::from("DOGE"), 5000.0),
        ],
    };

    let comparison = PortfolioComparison::new(&conservative, &aggressive);
    comparison.compare();
}
```

## When to Use Different Lifetimes

| Situation | Solution |
|-----------|----------|
| All references tied equally | Single parameter `'a` |
| Return tied to only one argument | Different parameters |
| Struct stores independent references | Different parameters |
| Comparing two independent sources | Different parameters |

## Common Mistakes

```rust
// ERROR: Returning reference to local variable
fn bad_example<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    let local = String::from("local");
    // &local  // Can't! local will be destroyed
    x  // Correct: return input parameter
}

// ERROR: Wrong return lifetime
// fn wrong_lifetime<'a, 'b>(x: &'a str, y: &'b str) -> &'b str {
//     x  // Can't! x has lifetime 'a, not 'b
// }

// CORRECT: Lifetime matches returned reference
fn correct<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x  // x lives for 'a, we return 'a — everything matches
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `<'a, 'b>` | Two lifetime parameters |
| Independent references | Different data sources |
| Return connection | Return is tied to specific parameter |
| Structs | Can store references with different lifetimes |

## Exercises

1. **Quote Comparison**: Write a function that takes quotes from two exchanges (with different lifetimes) and returns the best price.

2. **Spread Analyzer**: Create a `SpreadAnalyzer<'bid, 'ask>` struct that stores references to bid and ask price arrays with different lifetimes.

3. **Risk Manager**: Implement a struct that compares current position with risk limits using different lifetimes.

4. **PnL Calculator**: Write a function with two lifetime parameters for calculating PnL using current and historical prices.

## Homework

1. Create a `TradeMatcher<'orders, 'market>` struct for matching orders with market data:
   ```rust
   struct TradeMatcher<'orders, 'market> {
       pending_orders: &'orders [Order],
       market_data: &'market MarketData,
   }
   ```

2. Implement a `find_arbitrage<'a, 'b>` function for finding arbitrage opportunities between two exchanges.

3. Create an analyzer that compares performance of two strategies with different data lifetimes.

## Navigation

[← Previous day](../chapter-045-lifetimes-basics/en.md) | [Next day →](../chapter-047-lifetime-elision/en.md)
