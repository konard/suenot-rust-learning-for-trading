# Day 29: String Parsing — Converting Input to Numbers

## Trading Analogy

In algorithmic trading, data often arrives in text format:
- **API responses:** `"price": "42150.50"` — price as a string
- **CSV files:** `BTC,42150.50,1.5` — all fields are text
- **User input:** `"Enter quantity: 0.5"`
- **WebSocket messages:** `{"bid": "42100", "ask": "42200"}`

Before performing calculations — computing profit, checking limits, comparing prices — you need to convert strings to numbers.

## The parse() Method

The main way to convert a string to a number in Rust:

```rust
fn main() {
    let price_str = "42150.50";

    // parse() returns Result<T, E>
    let price: f64 = price_str.parse().unwrap();

    println!("Price: {}", price);
    println!("Price +10%: {}", price * 1.1);
}
```

**Important:** You need to specify the result type — Rust must know what to parse into.

## Different Ways to Specify Type

```rust
fn main() {
    let amount_str = "100";

    // Method 1: Variable type annotation
    let amount1: i32 = amount_str.parse().unwrap();

    // Method 2: Turbofish syntax
    let amount2 = amount_str.parse::<i32>().unwrap();

    // Method 3: Type inferred from usage
    let amount3 = amount_str.parse().unwrap();
    let _doubled: i32 = amount3 * 2;

    println!("amount1: {}", amount1);
    println!("amount2: {}", amount2);
}
```

## Parsing Different Number Types

```rust
fn main() {
    // Integers
    let shares: i32 = "1000".parse().unwrap();
    let big_volume: i64 = "1000000000000".parse().unwrap();
    let order_id: u64 = "9876543210".parse().unwrap();

    // Floating point numbers
    let price: f64 = "42150.50".parse().unwrap();
    let ratio: f32 = "0.95".parse().unwrap();

    println!("Shares: {}", shares);
    println!("Volume: {}", big_volume);
    println!("Order ID: {}", order_id);
    println!("Price: {}", price);
    println!("Ratio: {}", ratio);
}
```

## Handling Parse Errors

`parse()` returns `Result` because parsing can fail:

```rust
fn main() {
    let valid = "42150.50";
    let invalid = "not_a_number";
    let empty = "";
    let with_spaces = " 100 ";

    // unwrap() will panic on error
    let price: f64 = valid.parse().unwrap();
    println!("Valid: {}", price);

    // Safe way with match
    match invalid.parse::<f64>() {
        Ok(num) => println!("Parsed: {}", num),
        Err(e) => println!("Error parsing '{}': {}", invalid, e),
    }

    // unwrap_or — default value
    let default_price: f64 = invalid.parse().unwrap_or(0.0);
    println!("Default: {}", default_price);

    // unwrap_or_else — lazy default value
    let fallback: f64 = empty.parse().unwrap_or_else(|_| {
        println!("Empty string, using market price");
        42000.0
    });
    println!("Fallback: {}", fallback);

    // trim() helps with whitespace
    let trimmed: i32 = with_spaces.trim().parse().unwrap();
    println!("Trimmed: {}", trimmed);
}
```

## Practical Example: Parsing Price from API

```rust
fn main() {
    // Simulating API response
    let api_response = r#"{"symbol": "BTCUSDT", "price": "42150.50"}"#;

    // Simple parsing (in practice use serde_json)
    if let Some(start) = api_response.find("\"price\": \"") {
        let price_start = start + 10;
        if let Some(end) = api_response[price_start..].find("\"") {
            let price_str = &api_response[price_start..price_start + end];

            match price_str.parse::<f64>() {
                Ok(price) => {
                    println!("BTC Price: ${:.2}", price);
                    println!("Buy 0.1 BTC: ${:.2}", price * 0.1);
                }
                Err(e) => println!("Failed to parse price: {}", e),
            }
        }
    }
}
```

## Practical Example: Parsing Trade Command

```rust
fn main() {
    let commands = [
        "buy BTC 0.5 42000",
        "sell ETH 10 2500.50",
        "buy DOGE 10000 0.08",
        "invalid command",
    ];

    for cmd in commands {
        println!("\nCommand: '{}'", cmd);
        parse_trade_command(cmd);
    }
}

fn parse_trade_command(command: &str) {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.len() != 4 {
        println!("  Error: invalid command format");
        return;
    }

    let action = parts[0];
    let symbol = parts[1];

    // Parse quantity
    let quantity: f64 = match parts[2].parse() {
        Ok(q) => q,
        Err(_) => {
            println!("  Error: invalid quantity '{}'", parts[2]);
            return;
        }
    };

    // Parse price
    let price: f64 = match parts[3].parse() {
        Ok(p) => p,
        Err(_) => {
            println!("  Error: invalid price '{}'", parts[3]);
            return;
        }
    };

    let total = quantity * price;

    println!("  Action: {}", action.to_uppercase());
    println!("  Symbol: {}", symbol);
    println!("  Quantity: {}", quantity);
    println!("  Price: ${:.2}", price);
    println!("  Total: ${:.2}", total);
}
```

## Practical Example: Parsing CSV Line

```rust
fn main() {
    // Trade data in CSV format
    let csv_data = "BTCUSDT,42150.50,0.5,BUY
ETHUSDT,2500.00,10.0,SELL
DOGEUSDT,0.08,50000,BUY";

    println!("=== Parsing Trades from CSV ===\n");

    let mut total_volume = 0.0;

    for (i, line) in csv_data.lines().enumerate() {
        println!("Trade #{}:", i + 1);

        let fields: Vec<&str> = line.split(',').collect();

        if fields.len() != 4 {
            println!("  Error: invalid line format\n");
            continue;
        }

        let symbol = fields[0];

        let price: f64 = match fields[1].parse() {
            Ok(p) => p,
            Err(_) => {
                println!("  Error parsing price\n");
                continue;
            }
        };

        let quantity: f64 = match fields[2].parse() {
            Ok(q) => q,
            Err(_) => {
                println!("  Error parsing quantity\n");
                continue;
            }
        };

        let side = fields[3];
        let volume = price * quantity;
        total_volume += volume;

        println!("  {} {} {} @ ${:.2}", side, quantity, symbol, price);
        println!("  Volume: ${:.2}\n", volume);
    }

    println!("Total Volume: ${:.2}", total_volume);
}
```

## Practical Example: Order Validation

```rust
fn main() {
    // User input (simulation)
    let inputs = [
        ("1000", "42000.50"),    // Valid
        ("abc", "42000"),        // Invalid quantity
        ("100", ""),             // Empty price
        ("-50", "42000"),        // Negative quantity
        ("100", "-100"),         // Negative price
    ];

    for (qty_str, price_str) in inputs {
        println!("\nInput: quantity='{}', price='{}'", qty_str, price_str);

        match validate_order(qty_str, price_str) {
            Ok((qty, price)) => {
                println!("  Order valid!");
                println!("  Quantity: {}", qty);
                println!("  Price: ${:.2}", price);
                println!("  Total: ${:.2}", qty * price);
            }
            Err(e) => println!("  Error: {}", e),
        }
    }
}

fn validate_order(qty_str: &str, price_str: &str) -> Result<(f64, f64), String> {
    // Check for empty values
    if qty_str.is_empty() {
        return Err("Quantity cannot be empty".to_string());
    }
    if price_str.is_empty() {
        return Err("Price cannot be empty".to_string());
    }

    // Parse quantity
    let quantity: f64 = qty_str
        .trim()
        .parse()
        .map_err(|_| format!("Cannot parse quantity: '{}'", qty_str))?;

    // Parse price
    let price: f64 = price_str
        .trim()
        .parse()
        .map_err(|_| format!("Cannot parse price: '{}'", price_str))?;

    // Validate values
    if quantity <= 0.0 {
        return Err("Quantity must be positive".to_string());
    }
    if price <= 0.0 {
        return Err("Price must be positive".to_string());
    }

    Ok((quantity, price))
}
```

## Parsing with Data Cleaning

```rust
fn main() {
    // Dirty data from various sources
    let dirty_prices = [
        "  42150.50  ",     // Whitespace
        "$42,150.50",       // Currency symbol and commas
        "42150.50 USD",     // Currency suffix
        "42_150.50",        // Underscores
        "+42150.50",        // Plus sign
    ];

    for dirty in dirty_prices {
        let clean = clean_price_string(dirty);
        match clean.parse::<f64>() {
            Ok(price) => println!("'{}' -> {:.2}", dirty, price),
            Err(_) => println!("'{}' -> failed to parse", dirty),
        }
    }
}

fn clean_price_string(s: &str) -> String {
    s.trim()
        .replace("$", "")
        .replace(",", "")
        .replace("_", "")
        .replace(" USD", "")
        .replace(" USDT", "")
        .replace("+", "")
}
```

## Parsing Different Number Bases

```rust
fn main() {
    // Sometimes IDs or codes come in different formats

    // Decimal (default)
    let decimal: i32 = "255".parse().unwrap();

    // Hexadecimal (0x prefix or from_str_radix)
    let hex = i32::from_str_radix("FF", 16).unwrap();
    let hex2 = i32::from_str_radix("ff", 16).unwrap();  // Case insensitive

    // Binary
    let binary = i32::from_str_radix("11111111", 2).unwrap();

    // Octal
    let octal = i32::from_str_radix("377", 8).unwrap();

    println!("Decimal '255': {}", decimal);
    println!("Hex 'FF': {}", hex);
    println!("Hex 'ff': {}", hex2);
    println!("Binary '11111111': {}", binary);
    println!("Octal '377': {}", octal);

    // All equal 255
    assert_eq!(decimal, hex);
    assert_eq!(hex, binary);
    assert_eq!(binary, octal);
    println!("\nAll values equal 255!");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `parse()` | Main method for parsing string to number |
| Turbofish `::<T>` | Syntax for specifying type for parse |
| `Result<T, E>` | Parsing can fail |
| `unwrap()` | Panics on error |
| `unwrap_or()` | Default value on error |
| `trim()` | Remove whitespace before parsing |
| `from_str_radix` | Parse in different number bases |

## Exercises

### Exercise 1: Profit Calculator

Write a function that takes strings for buy price, sell price, and quantity, and returns profit/loss:

```rust
fn calculate_profit(buy_price: &str, sell_price: &str, quantity: &str) -> Result<f64, String> {
    // Your code here
}

// Usage:
// calculate_profit("42000", "43000", "0.5") -> Ok(500.0)
// calculate_profit("abc", "43000", "0.5") -> Err("...")
```

### Exercise 2: Order Book Level Parser

Write a function to parse an order book level string in format `"price:quantity"`:

```rust
fn parse_order_book_level(level: &str) -> Result<(f64, f64), String> {
    // Your code here
}

// Usage:
// parse_order_book_level("42000.50:1.5") -> Ok((42000.50, 1.5))
```

### Exercise 3: Trade History Parser

Write a function to parse a trade string in format `"timestamp,symbol,side,price,quantity"`:

```rust
struct Trade {
    timestamp: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn parse_trade(line: &str) -> Result<Trade, String> {
    // Your code here
}
```

### Exercise 4: Limits Validator

Write a function to validate trading limits:

```rust
fn validate_trade_limits(
    quantity_str: &str,
    price_str: &str,
    min_qty: f64,
    max_qty: f64,
    min_price: f64,
    max_price: f64,
) -> Result<(f64, f64), String> {
    // Your code here
}
```

## Homework

1. **Currency Converter:** Write a program that parses a string like `"100 USD to EUR"` and performs conversion (use fixed exchange rates).

2. **Portfolio Parser:** Create a function to parse a portfolio string `"BTC:0.5,ETH:10,DOGE:50000"` into a vector of tuples (symbol, quantity).

3. **Position Calculator:** Write a function that parses a string with multiple trades and calculates the average entry price:
   ```
   "BUY 0.5 @ 42000, BUY 0.3 @ 41000, BUY 0.2 @ 43000"
   ```

4. **Report Generator:** Create a program that parses CSV trade data and outputs statistics: total volume, average price, number of buy/sell trades.

## Navigation

[← Previous day](../028-user-input/en.md) | [Next day →](../030-result-type/en.md)
