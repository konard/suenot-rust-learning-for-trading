# Day 97: while let — Process While Successful

## Trading Analogy

Imagine you're working with an order queue. While there are orders in the queue — you process them. Once the queue is empty — you stop. This is exactly like `while let`: **keep executing an action while the pattern successfully matches**.

Another example: you're receiving quotes from an exchange. While the connection is active and data arrives — you process it. Once you get `None` or an error — you exit the loop.

## Basic while let Syntax

```rust
fn main() {
    let mut prices = vec![42000.0, 42100.0, 42050.0];

    // Extract prices while the vector is not empty
    while let Some(price) = prices.pop() {
        println!("Processing price: ${:.2}", price);
    }

    println!("All prices processed");
}
```

**Important:** `while let` is syntactic sugar for a `loop` with `match`. The loop continues while the pattern `Some(price)` matches.

## Comparison with loop + match

```rust
fn main() {
    let mut orders = vec![100.0, 200.0, 150.0];

    // Equivalent code with loop + match
    loop {
        match orders.pop() {
            Some(order) => println!("Order: ${:.2}", order),
            None => break,
        }
    }

    // Same thing with while let — much cleaner!
    let mut orders2 = vec![100.0, 200.0, 150.0];
    while let Some(order) = orders2.pop() {
        println!("Order: ${:.2}", order);
    }
}
```

## Processing Order Queue

```rust
fn main() {
    let mut order_queue: Vec<Order> = vec![
        Order { id: 1, symbol: "BTC", quantity: 0.5, price: 42000.0 },
        Order { id: 2, symbol: "ETH", quantity: 10.0, price: 2200.0 },
        Order { id: 3, symbol: "BTC", quantity: 0.25, price: 42100.0 },
    ];

    println!("Starting order queue processing...\n");

    while let Some(order) = order_queue.pop() {
        process_order(&order);
    }

    println!("\nQueue empty. All orders processed.");
}

struct Order {
    id: u32,
    symbol: &'static str,
    quantity: f64,
    price: f64,
}

fn process_order(order: &Order) {
    let value = order.quantity * order.price;
    println!(
        "Order #{}: {} {} @ ${:.2} = ${:.2}",
        order.id, order.quantity, order.symbol, order.price, value
    );
}
```

## Reading Data from Iterator

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];
    let mut iter = prices.iter();

    let mut sum = 0.0;
    let mut count = 0;

    while let Some(&price) = iter.next() {
        sum += price;
        count += 1;
        println!("Price {}: ${:.2}, sum: ${:.2}", count, price, sum);
    }

    let average = sum / count as f64;
    println!("\nAverage price: ${:.2}", average);
}
```

## Parsing Quote Stream

```rust
fn main() {
    // Simulating data stream from exchange
    let raw_data = vec!["42000.50", "42100.75", "invalid", "42050.25"];
    let mut data_iter = raw_data.iter();

    let mut valid_prices = Vec::new();

    while let Some(raw) = data_iter.next() {
        // Try to parse, skip invalid entries
        if let Ok(price) = raw.parse::<f64>() {
            valid_prices.push(price);
            println!("Received price: ${:.2}", price);
        } else {
            println!("Skipping invalid data: {}", raw);
        }
    }

    println!("\nValid prices: {}", valid_prices.len());
}
```

## Nested while let

```rust
fn main() {
    let portfolios = vec![
        vec!["BTC", "ETH", "SOL"],
        vec!["AAPL", "GOOGL"],
        vec!["EUR/USD", "GBP/USD", "USD/JPY"],
    ];

    let mut portfolio_iter = portfolios.iter();
    let mut portfolio_num = 0;

    while let Some(portfolio) = portfolio_iter.next() {
        portfolio_num += 1;
        println!("Portfolio {}:", portfolio_num);

        let mut asset_iter = portfolio.iter();
        while let Some(asset) = asset_iter.next() {
            println!("  - {}", asset);
        }
    }
}
```

## Processing Result in Loop

```rust
fn main() {
    let trade_strings = vec!["BTC:0.5:42000", "ETH:10:2200", "invalid", "SOL:100:25"];
    let mut iter = trade_strings.iter();

    let mut successful_trades = 0;
    let mut total_value = 0.0;

    while let Some(trade_str) = iter.next() {
        match parse_trade(trade_str) {
            Ok((symbol, qty, price)) => {
                let value = qty * price;
                println!("{}: {} @ ${:.2} = ${:.2}", symbol, qty, price, value);
                successful_trades += 1;
                total_value += value;
            }
            Err(e) => {
                println!("Parse error '{}': {}", trade_str, e);
            }
        }
    }

    println!("\nSuccessful trades: {}", successful_trades);
    println!("Total value: ${:.2}", total_value);
}

fn parse_trade(s: &str) -> Result<(&str, f64, f64), &'static str> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return Err("invalid format");
    }

    let symbol = parts[0];
    let qty = parts[1].parse::<f64>().map_err(|_| "invalid quantity")?;
    let price = parts[2].parse::<f64>().map_err(|_| "invalid price")?;

    Ok((symbol, qty, price))
}
```

## Practical Example: Trading Stream Simulation

```rust
fn main() {
    let mut trading_session = TradingSession::new(10000.0);

    // Simulating signals
    let signals = vec![
        Some(Signal::Buy { symbol: "BTC", qty: 0.1, price: 42000.0 }),
        Some(Signal::Buy { symbol: "ETH", qty: 5.0, price: 2200.0 }),
        Some(Signal::Sell { symbol: "BTC", qty: 0.1, price: 42500.0 }),
        None, // End of session
    ];

    let mut signal_iter = signals.into_iter();

    println!("Starting trading session");
    println!("Balance: ${:.2}\n", trading_session.balance);

    while let Some(Some(signal)) = signal_iter.next() {
        trading_session.execute(signal);
    }

    println!("\nSession ended");
    println!("Final balance: ${:.2}", trading_session.balance);
    println!("PnL: ${:.2}", trading_session.pnl);
}

struct TradingSession {
    balance: f64,
    pnl: f64,
}

enum Signal {
    Buy { symbol: &'static str, qty: f64, price: f64 },
    Sell { symbol: &'static str, qty: f64, price: f64 },
}

impl TradingSession {
    fn new(balance: f64) -> Self {
        TradingSession { balance, pnl: 0.0 }
    }

    fn execute(&mut self, signal: Signal) {
        match signal {
            Signal::Buy { symbol, qty, price } => {
                let cost = qty * price;
                self.balance -= cost;
                println!("BUY {} {} @ ${:.2} (cost: ${:.2})", qty, symbol, price, cost);
            }
            Signal::Sell { symbol, qty, price } => {
                let revenue = qty * price;
                self.balance += revenue;
                self.pnl += revenue - (qty * 42000.0); // simplified PnL
                println!("SELL {} {} @ ${:.2} (revenue: ${:.2})", qty, symbol, price, revenue);
            }
        }
    }
}
```

## Combining with break and continue

```rust
fn main() {
    let orders = vec![
        Order { id: 1, amount: 500.0, is_valid: true },
        Order { id: 2, amount: 150.0, is_valid: false },
        Order { id: 3, amount: 10000.0, is_valid: true },  // too large
        Order { id: 4, amount: 800.0, is_valid: true },
    ];

    let mut iter = orders.iter();
    let max_order_size = 5000.0;
    let mut processed = 0;

    while let Some(order) = iter.next() {
        // Skip invalid orders
        if !order.is_valid {
            println!("Order #{} is invalid, skipping", order.id);
            continue;
        }

        // Break on oversized order
        if order.amount > max_order_size {
            println!("Order #{} exceeds limit ${:.2}, stopping", order.id, max_order_size);
            break;
        }

        println!("Processed order #{}: ${:.2}", order.id, order.amount);
        processed += 1;
    }

    println!("\nOrders processed: {}", processed);
}

struct Order {
    id: u32,
    amount: f64,
    is_valid: bool,
}
```

## while let Usage Patterns

```rust
fn main() {
    // 1. Draining a queue
    let mut queue = vec![1, 2, 3];
    while let Some(item) = queue.pop() {
        println!("Item: {}", item);
    }

    // 2. Iterating over an iterator
    let data = [10, 20, 30];
    let mut iter = data.iter();
    while let Some(value) = iter.next() {
        println!("Value: {}", value);
    }

    // 3. Destructuring tuples
    let pairs = vec![(1, "a"), (2, "b")];
    let mut pair_iter = pairs.iter();
    while let Some((num, letter)) = pair_iter.next() {
        println!("{}: {}", num, letter);
    }

    // 4. Nested Option
    let nested: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
    let mut nested_iter = nested.iter();
    while let Some(opt) = nested_iter.next() {
        if let Some(val) = opt {
            println!("Found: {}", val);
        }
    }
}
```

## What We Learned

| Construct | Description | When to Use |
|-----------|-------------|-------------|
| `while let Some(x) = expr` | Loop while Option contains value | Processing queues, iterators |
| `while let Ok(x) = expr` | Loop while Result is successful | Reading data, parsing |
| `while let Pattern = expr` | General pattern | Destructuring in loop |

## Practical Exercises

### Exercise 1: Processing Order Book
Create a function that processes an order book using `while let`, stopping when it encounters an order with zero quantity.

### Exercise 2: Parsing Trade History
Write a parser that reads trade history lines and stops at the first parsing error.

### Exercise 3: Candlestick Aggregation
Implement a function that aggregates tick data into candlesticks, using `while let` to read the tick stream.

### Exercise 4: Position Filtering
Create a function that iterates through positions and closes losing ones, using `while let` with `continue` and `break` conditions.

## Homework

1. Write a function `process_market_data(stream: &mut Iterator<Item = MarketTick>) -> Summary` that uses `while let` to process a stream of market data and returns summary statistics.

2. Create an order queue simulator with priorities. Use `while let` to process orders in priority order.

3. Implement an arbitrage opportunity finder that scans quote streams from different exchanges using `while let`.

4. Write a CSV file parser for trade history that uses `while let` for line-by-line reading and stops at the first critical error.

## Navigation

[← Previous day](../096-if-let-order-matching/en.md) | [Next day →](../098-advanced-matching/en.md)
