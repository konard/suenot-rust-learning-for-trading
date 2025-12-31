# Day 77: ? Operator — Propagating Errors Up

## Trading Analogy

Imagine a chain of operations on an exchange: fetch data from API → parse JSON → validate price → create order. If any step fails, you need to immediately abort the entire chain and report the error. The `?` operator is like an **automatic stop-loss** for operations: at the first error, function execution stops, and the error "bubbles up" to the caller.

## The Problem Without ?

Without `?`, code becomes a staircase of match statements:

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_portfolio_verbose() -> Result<String, io::Error> {
    let file_result = File::open("portfolio.json");

    let mut file = match file_result {
        Ok(f) => f,
        Err(e) => return Err(e),  // If error — return it
    };

    let mut contents = String::new();

    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => Err(e),
    }
}
```

Too much repetitive code! Each `Result` requires explicit handling.

## The ? Operator — An Elegant Solution

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_portfolio() -> Result<String, io::Error> {
    let mut file = File::open("portfolio.json")?;  // ? unwraps Ok or returns Err
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    match read_portfolio() {
        Ok(data) => println!("Portfolio: {}", data),
        Err(e) => println!("Error: {}", e),
    }
}
```

**The `?` operator** does the following:
- If `Result` is `Ok(value)`, it extracts `value` and continues execution
- If `Result` is `Err(e)`, it immediately returns `Err(e)` from the function

## How ? Works Under the Hood

```rust
// This:
let file = File::open("data.json")?;

// Is equivalent to:
let file = match File::open("data.json") {
    Ok(f) => f,
    Err(e) => return Err(e.into()),  // .into() for error type conversion
};
```

## Chaining Operations with ?

```rust
use std::io::{self, Read};
use std::fs::File;

fn load_trading_config() -> Result<String, io::Error> {
    let mut contents = String::new();
    File::open("config.json")?
        .read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    match load_trading_config() {
        Ok(config) => println!("Config loaded: {} bytes", config.len()),
        Err(e) => eprintln!("Failed to load config: {}", e),
    }
}
```

## The ? Operator with Option

The `?` operator also works with `Option`:

```rust
fn get_best_bid(order_book: &[(f64, f64)]) -> Option<f64> {
    let (price, _quantity) = order_book.first()?;  // Returns None if empty
    Some(*price)
}

fn calculate_spread(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> Option<f64> {
    let best_bid = get_best_bid(bids)?;
    let best_ask = get_best_bid(asks)?;  // Reusing the function
    Some(best_ask - best_bid)
}

fn main() {
    let bids = [(42000.0, 1.5), (41990.0, 2.0)];
    let asks = [(42010.0, 1.0), (42020.0, 3.0)];

    match calculate_spread(&bids, &asks) {
        Some(spread) => println!("Spread: ${:.2}", spread),
        None => println!("Cannot calculate spread"),
    }

    // Empty order book
    let empty: [(f64, f64); 0] = [];
    println!("Empty spread: {:?}", calculate_spread(&empty, &asks));
}
```

## Practical Example: Loading and Parsing Data

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
}

fn parse_trade_line(line: &str) -> Result<Trade, String> {
    let parts: Vec<&str> = line.split(',').collect();

    if parts.len() != 3 {
        return Err(format!("Invalid format: expected 3 fields, got {}", parts.len()));
    }

    let symbol = parts[0].to_string();
    let price = parts[1].parse::<f64>()
        .map_err(|_| format!("Invalid price: {}", parts[1]))?;
    let quantity = parts[2].parse::<f64>()
        .map_err(|_| format!("Invalid quantity: {}", parts[2]))?;

    Ok(Trade { symbol, price, quantity })
}

fn load_trades(filename: &str) -> Result<Vec<Trade>, String> {
    let file = File::open(filename)
        .map_err(|e| format!("Cannot open file: {}", e))?;

    let reader = BufReader::new(file);
    let mut trades = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result
            .map_err(|e| format!("Error reading line {}: {}", line_num + 1, e))?;

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let trade = parse_trade_line(&line)
            .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;

        trades.push(trade);
    }

    Ok(trades)
}

fn main() {
    match load_trades("trades.csv") {
        Ok(trades) => {
            println!("Loaded {} trades:", trades.len());
            for trade in &trades {
                println!("  {:?}", trade);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Converting Error Types with ?

The `?` operator automatically converts error types via the `From` trait:

```rust
use std::fs::File;
use std::io::{self, Read};
use std::num::ParseFloatError;

#[derive(Debug)]
enum TradingError {
    IoError(io::Error),
    ParseError(ParseFloatError),
    ValidationError(String),
}

// Implement From for automatic conversion
impl From<io::Error> for TradingError {
    fn from(err: io::Error) -> Self {
        TradingError::IoError(err)
    }
}

impl From<ParseFloatError> for TradingError {
    fn from(err: ParseFloatError) -> Self {
        TradingError::ParseError(err)
    }
}

fn read_and_parse_price(filename: &str) -> Result<f64, TradingError> {
    let mut contents = String::new();
    File::open(filename)?.read_to_string(&mut contents)?;  // io::Error -> TradingError

    let price: f64 = contents.trim().parse()?;  // ParseFloatError -> TradingError

    if price <= 0.0 {
        return Err(TradingError::ValidationError(
            "Price must be positive".to_string()
        ));
    }

    Ok(price)
}

fn main() {
    match read_and_parse_price("btc_price.txt") {
        Ok(price) => println!("BTC Price: ${:.2}", price),
        Err(TradingError::IoError(e)) => eprintln!("IO Error: {}", e),
        Err(TradingError::ParseError(e)) => eprintln!("Parse Error: {}", e),
        Err(TradingError::ValidationError(msg)) => eprintln!("Validation: {}", msg),
    }
}
```

## Using ? in the main Function

You can use `?` directly in `main` by specifying a return type:

```rust
use std::fs::File;
use std::io::{self, Read};

fn main() -> Result<(), io::Error> {
    let mut file = File::open("config.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("Config: {}", contents);
    Ok(())
}
```

On error, the program will print a message and exit with a non-zero code.

## Combining ? with Result Methods

```rust
fn fetch_price(symbol: &str) -> Result<f64, String> {
    // Simulating API request
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2800.0),
        _ => Err(format!("Unknown symbol: {}", symbol)),
    }
}

fn calculate_portfolio_value(holdings: &[(&str, f64)]) -> Result<f64, String> {
    let mut total = 0.0;

    for (symbol, quantity) in holdings {
        let price = fetch_price(symbol)?;
        total += price * quantity;
    }

    Ok(total)
}

fn get_portfolio_with_margin(holdings: &[(&str, f64)], margin: f64) -> Result<f64, String> {
    let value = calculate_portfolio_value(holdings)?;

    if margin < 0.0 || margin > 1.0 {
        return Err("Margin must be between 0 and 1".to_string());
    }

    Ok(value * (1.0 + margin))
}

fn main() {
    let holdings = [("BTC", 0.5), ("ETH", 2.0)];

    match get_portfolio_with_margin(&holdings, 0.1) {
        Ok(value) => println!("Portfolio value with 10% margin: ${:.2}", value),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Pattern: Early Return with Validation

```rust
fn execute_trade(
    symbol: &str,
    side: &str,
    quantity: f64,
    price: f64,
    balance: f64,
) -> Result<String, String> {
    // Validation with early return
    if quantity <= 0.0 {
        return Err("Quantity must be positive".to_string());
    }

    if price <= 0.0 {
        return Err("Price must be positive".to_string());
    }

    let cost = quantity * price;

    if side == "BUY" && cost > balance {
        return Err(format!(
            "Insufficient balance: need ${:.2}, have ${:.2}",
            cost, balance
        ));
    }

    // Main logic
    Ok(format!(
        "Executed {} {} {} @ ${:.2} (total: ${:.2})",
        side, quantity, symbol, price, cost
    ))
}

fn process_orders(orders: &[(&str, &str, f64, f64)], balance: f64) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    let mut remaining_balance = balance;

    for (symbol, side, qty, price) in orders {
        let result = execute_trade(symbol, side, *qty, *price, remaining_balance)?;

        if *side == "BUY" {
            remaining_balance -= qty * price;
        } else {
            remaining_balance += qty * price;
        }

        results.push(result);
    }

    Ok(results)
}

fn main() {
    let orders = [
        ("BTC", "BUY", 0.1, 42000.0),
        ("ETH", "BUY", 1.0, 2800.0),
        ("BTC", "SELL", 0.05, 42500.0),
    ];

    match process_orders(&orders, 10000.0) {
        Ok(results) => {
            println!("All orders executed:");
            for r in results {
                println!("  {}", r);
            }
        }
        Err(e) => eprintln!("Order processing failed: {}", e),
    }
}
```

## Comparing Approaches

```rust
// ❌ Without ? — lots of boilerplate
fn without_question_mark() -> Result<i32, String> {
    let a = match step_one() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let b = match step_two(a) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    match step_three(b) {
        Ok(v) => Ok(v),
        Err(e) => Err(e),
    }
}

// ✅ With ? — clean and readable code
fn with_question_mark() -> Result<i32, String> {
    let a = step_one()?;
    let b = step_two(a)?;
    step_three(b)
}

fn step_one() -> Result<i32, String> { Ok(1) }
fn step_two(x: i32) -> Result<i32, String> { Ok(x + 1) }
fn step_three(x: i32) -> Result<i32, String> { Ok(x * 2) }

fn main() {
    println!("Result: {:?}", with_question_mark());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `?` with Result | Extracts Ok or returns Err |
| `?` with Option | Extracts Some or returns None |
| Chaining `?` | Multiple operations in sequence |
| From trait | Automatic error type conversion |
| `main() -> Result` | Using ? in main |

## Homework

1. Write a function `load_and_validate_orders(filename: &str) -> Result<Vec<Order>, String>` that reads a file with orders, parses each line, and validates the data. Use the `?` operator at each step.

2. Create a chain of three functions for processing trading data: `fetch_data() -> Result<String, ApiError>`, `parse_data(data: &str) -> Result<Vec<Trade>, ParseError>`, `analyze_trades(trades: &[Trade]) -> Result<Report, AnalysisError>`. Combine them into a single function with a unified error type.

3. Implement a function `calculate_portfolio_stats(filename: &str) -> Result<PortfolioStats, Box<dyn std::error::Error>>` that reads a file with positions, parses the data, and calculates statistics (total value, PnL, percentage allocations).

4. Write a program that uses `?` in `main()` to: read configuration, load market data, perform analysis, and output results.

## Navigation

[← Previous day](../076-result-methods/en.md) | [Next day →](../078-custom-error-types/en.md)
