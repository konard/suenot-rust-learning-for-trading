# Day 10: Characters and Strings — Tickers AAPL, BTC, ETH

## Trading Analogy

On exchanges, everything has text names:
- Tickers: **BTC**, **ETH**, **AAPL**
- Trading pairs: **BTC/USDT**, **ETH/BTC**
- Exchange names: **Binance**, **Coinbase**
- Messages: **"Order filled"**, **"Insufficient balance"**

In Rust, there are two main types for working with text: `char` and strings.

## Characters (char)

`char` is a single character in single quotes:

```rust
fn main() {
    let currency_sign: char = '$';
    let btc_symbol: char = '₿';
    let up_arrow: char = '↑';
    let down_arrow: char = '↓';
    let check_mark: char = '✓';

    println!("Bitcoin {} {}", btc_symbol, up_arrow);
    println!("Trade executed {}", check_mark);
}
```

`char` in Rust is a Unicode character (4 bytes), so it supports emojis and any characters:

```rust
fn main() {
    let rocket: char = '🚀';
    let money: char = '💰';
    let chart: char = '📈';

    println!("To the moon! {} {} {}", rocket, money, chart);
}
```

## Two Types of Strings

### `&str` — String Slice (Borrowed)

Fixed text that cannot be modified:

```rust
fn main() {
    let ticker: &str = "BTC/USDT";
    let exchange: &str = "Binance";
    let status: &str = "Order filled";

    println!("Trading {} on {}", ticker, exchange);
    println!("Status: {}", status);
}
```

**Analogy:** This is like text on a sign — you can read it but can't change it.

### `String` — Owned String

A string that can be modified:

```rust
fn main() {
    // Creating String
    let mut message = String::from("Price: ");

    // Adding text
    message.push_str("42000");
    message.push_str(" USDT");

    println!("{}", message);  // Price: 42000 USDT

    // Adding a character
    message.push('!');
    println!("{}", message);  // Price: 42000 USDT!
}
```

**Analogy:** This is like a notebook — you can write, erase, and add text.

## Creating String

```rust
fn main() {
    // Different ways to create
    let s1 = String::from("BTC");
    let s2 = "ETH".to_string();
    let s3 = String::new();  // Empty string
    let s4: String = format!("{}/{}", "BTC", "USDT");

    println!("s1: {}", s1);
    println!("s2: {}", s2);
    println!("s3: '{}'", s3);
    println!("s4: {}", s4);
}
```

## String Concatenation

```rust
fn main() {
    // Method 1: format!
    let base = "BTC";
    let quote = "USDT";
    let pair = format!("{}/{}", base, quote);
    println!("Pair: {}", pair);

    // Method 2: + operator (takes ownership of first string)
    let mut s = String::from("Order #");
    s = s + "12345";
    println!("{}", s);

    // Method 3: push_str
    let mut log = String::from("[INFO] ");
    log.push_str("Trade executed at ");
    log.push_str("42000 USDT");
    println!("{}", log);
}
```

## Useful String Methods

```rust
fn main() {
    let ticker = String::from("  BTC/USDT  ");

    // Remove whitespace
    println!("Trimmed: '{}'", ticker.trim());  // 'BTC/USDT'

    // Length
    println!("Length: {}", ticker.len());  // 12

    // Check content
    println!("Contains BTC: {}", ticker.contains("BTC"));  // true
    println!("Starts with BTC: {}", ticker.trim().starts_with("BTC"));  // true
    println!("Ends with USDT: {}", ticker.trim().ends_with("USDT"));  // true

    // Replace
    let new_pair = ticker.trim().replace("USDT", "EUR");
    println!("New pair: {}", new_pair);  // BTC/EUR

    // Upper/lowercase
    println!("Upper: {}", ticker.to_uppercase());
    println!("Lower: {}", ticker.to_lowercase());

    // Is empty
    let empty = String::new();
    println!("Is empty: {}", empty.is_empty());  // true
}
```

## Splitting Strings

```rust
fn main() {
    let pair = "BTC/USDT";

    // Split by delimiter
    let parts: Vec<&str> = pair.split('/').collect();
    println!("Base: {}", parts[0]);   // BTC
    println!("Quote: {}", parts[1]);  // USDT

    // Split to iterator
    for part in pair.split('/') {
        println!("Part: {}", part);
    }

    // Split into lines
    let log = "Line 1\nLine 2\nLine 3";
    for line in log.lines() {
        println!("Line: {}", line);
    }
}
```

## Accessing Characters

```rust
fn main() {
    let ticker = "BTC";

    // Get characters (iterator)
    for c in ticker.chars() {
        println!("Char: {}", c);
    }

    // First character
    let first = ticker.chars().next();
    println!("First: {:?}", first);  // Some('B')

    // Nth character
    let second = ticker.chars().nth(1);
    println!("Second: {:?}", second);  // Some('T')

    // CANNOT index directly!
    // let c = ticker[0];  // ERROR!
}
```

**Important:** In Rust, strings are UTF-8, and one "character" can take multiple bytes. That's why indexing is forbidden.

## Practical Example: Parsing Trading Pair

```rust
fn main() {
    let pair = "ETH/USDT";

    // Split the pair
    let parts: Vec<&str> = pair.split('/').collect();

    if parts.len() == 2 {
        let base = parts[0];
        let quote = parts[1];

        println!("Trading pair: {}", pair);
        println!("Base currency: {}", base);
        println!("Quote currency: {}", quote);

        // Check if it's a stablecoin pair
        let stablecoins = ["USDT", "USDC", "BUSD", "DAI"];
        let is_stable_pair = stablecoins.contains(&quote);

        println!("Stable pair: {}", is_stable_pair);
    } else {
        println!("Invalid pair format!");
    }
}
```

## Practical Example: Message Formatting

```rust
fn main() {
    // Trade data
    let symbol = "BTC/USDT";
    let side = "BUY";
    let price = 42000.50;
    let quantity = 0.5;
    let order_id = 123456789;

    // Format message for log
    let log_message = format!(
        "[TRADE] {} {} {:.8} @ {:.2} | Order #{}",
        side, symbol, quantity, price, order_id
    );
    println!("{}", log_message);

    // Format for user
    let user_message = format!(
        "Bought {} {} at ${:.2} each. Total: ${:.2}",
        quantity, symbol.split('/').next().unwrap_or(""),
        price, price * quantity
    );
    println!("{}", user_message);

    // Format for API
    let api_payload = format!(
        r#"{{"symbol":"{}","side":"{}","price":{},"quantity":{}}}"#,
        symbol, side, price, quantity
    );
    println!("API: {}", api_payload);
}
```

## Practical Example: Ticker Validation

```rust
fn main() {
    let tickers = ["BTC", "eth", "XRP123", "DOGE", ""];

    for ticker in tickers {
        let is_valid = validate_ticker(ticker);
        println!("{:10} -> valid: {}", format!("'{}'", ticker), is_valid);
    }
}

fn validate_ticker(ticker: &str) -> bool {
    // Rules:
    // 1. Not empty
    // 2. Only letters
    // 3. 2 to 10 characters
    // 4. Uppercase

    if ticker.is_empty() {
        return false;
    }

    if ticker.len() < 2 || ticker.len() > 10 {
        return false;
    }

    // Check all characters are uppercase letters
    ticker.chars().all(|c| c.is_ascii_uppercase())
}
```

## Converting &str and String

```rust
fn main() {
    // &str -> String
    let s: &str = "BTC";
    let string1: String = s.to_string();
    let string2: String = String::from(s);
    let string3: String = s.to_owned();

    // String -> &str
    let string = String::from("ETH");
    let slice: &str = &string;
    let slice2: &str = string.as_str();

    println!("String: {}", string);
    println!("Slice: {}", slice);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `char` | Single Unicode character |
| `&str` | Immutable string slice |
| `String` | Mutable string |
| `format!` | String formatting |
| `split`, `trim` | String processing |

## Homework

1. Write a function that takes a trading pair (e.g., "BTC/USDT") and returns a tuple (base, quote)

2. Create a ticker validation function with rules:
   - Length 2-6 characters
   - Letters only
   - Uppercase

3. Write a formatter for displaying a trade:
   ```
   ═══════════════════════
   TRADE EXECUTED
   Symbol: BTC/USDT
   Side: BUY
   Price: $42,000.50
   Quantity: 0.50000000
   Total: $21,000.25
   ═══════════════════════
   ```

4. Implement a simple command parser: "buy BTC 0.5" -> (action, symbol, amount)

## Navigation

[← Previous day](../009-booleans-market-status/en.md) | [Next day →](../011-tuples-bid-ask/en.md)
