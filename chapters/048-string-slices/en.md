# Day 48: String Slices — Part of Exchange Name

## Trading Analogy

Imagine working with exchange data:
- Full exchange name: **"Binance Futures"** → need to extract **"Binance"**
- Trading pair: **"BTC/USDT"** → need to get just **"BTC"** or **"USDT"**
- Order ID: **"ORD-2024-001234"** → extract year **"2024"** or number **"001234"**
- Log: **"[ERROR] Connection failed"** → get level **"ERROR"**

In trading, we constantly work with parts of strings — slices. Rust provides a safe and efficient way to work with such data through **string slices** (`&str`).

## What is a String Slice?

A string slice is a **reference to part of a string** without copying data:

```rust
fn main() {
    let exchange = String::from("Binance Futures");

    // Slice of first 7 characters (bytes)
    let name: &str = &exchange[0..7];
    println!("Exchange: {}", name);  // Binance

    // Slice from 8th to end
    let suffix: &str = &exchange[8..];
    println!("Type: {}", suffix);  // Futures

    // Entire string as slice
    let full: &str = &exchange[..];
    println!("Full: {}", full);  // Binance Futures
}
```

**Analogy:** Imagine a long scroll with data. A slice is a "window" into a specific part of the scroll. You see the data but don't copy it.

## Slice Syntax

```rust
fn main() {
    let pair = String::from("ETH/USDT");

    // [start..end] - from start to end (not including end)
    let base = &pair[0..3];      // "ETH"

    // [start..] - from start to end
    let from_slash = &pair[3..]; // "/USDT"

    // [..end] - from beginning to end
    let to_slash = &pair[..3];   // "ETH"

    // [..] - entire string
    let full = &pair[..];        // "ETH/USDT"

    println!("Base: {}", base);
    println!("From slash: {}", from_slash);
    println!("To slash: {}", to_slash);
    println!("Full: {}", full);
}
```

## Slices Work with Bytes, Not Characters!

**Important:** Indices in slices are **bytes**, not characters. For ASCII this isn't a problem, but for Unicode it can cause a panic:

```rust
fn main() {
    // ASCII - everything works (1 character = 1 byte)
    let ticker = "BTC";
    let first_two = &ticker[0..2];
    println!("First two: {}", first_two);  // "BT"

    // Unicode - need to be careful!
    let currency = "₿itcoin";  // ₿ takes 3 bytes

    // This will panic!
    // let wrong = &currency[0..1];  // PANIC!

    // Correct - at character boundaries
    let symbol = &currency[0..3];  // "₿" (3 bytes)
    println!("Symbol: {}", symbol);

    let rest = &currency[3..];  // "itcoin"
    println!("Rest: {}", rest);
}
```

## Safe Slice Extraction

For safe operation, use `get()` methods:

```rust
fn main() {
    let exchange = "Binance";

    // get() returns Option<&str>
    if let Some(slice) = exchange.get(0..3) {
        println!("First 3: {}", slice);  // "Bin"
    }

    // Safe - doesn't panic on invalid indices
    let result = exchange.get(0..100);
    println!("Out of bounds: {:?}", result);  // None

    // Safe - doesn't panic on invalid UTF-8 boundaries
    let unicode = "₿TC";
    let invalid = unicode.get(0..1);  // None (not at character boundary)
    println!("Invalid UTF-8 boundary: {:?}", invalid);
}
```

## Practical Example: Parsing Trading Pair

```rust
fn main() {
    let pairs = ["BTC/USDT", "ETH/BTC", "SOL/USDC", "INVALID"];

    for pair in pairs {
        match parse_trading_pair(pair) {
            Some((base, quote)) => {
                println!("{}: base={}, quote={}", pair, base, quote);
            }
            None => {
                println!("{}: invalid format", pair);
            }
        }
    }
}

fn parse_trading_pair(pair: &str) -> Option<(&str, &str)> {
    // Find position of delimiter
    let slash_pos = pair.find('/')?;

    // Extract slices before and after delimiter
    let base = pair.get(..slash_pos)?;
    let quote = pair.get(slash_pos + 1..)?;

    // Check that both parts are not empty
    if base.is_empty() || quote.is_empty() {
        return None;
    }

    Some((base, quote))
}
```

## Practical Example: Extracting Data from Order ID

```rust
fn main() {
    let order_ids = [
        "ORD-2024-001234",
        "ORD-2023-999999",
        "INVALID",
    ];

    for order_id in order_ids {
        if let Some(info) = parse_order_id(order_id) {
            println!("Order {}: year={}, number={}",
                     order_id, info.0, info.1);
        } else {
            println!("Invalid order ID: {}", order_id);
        }
    }
}

fn parse_order_id(order_id: &str) -> Option<(&str, &str)> {
    // Format: ORD-YYYY-NNNNNN
    // Check prefix
    if !order_id.starts_with("ORD-") {
        return None;
    }

    // Extract year (positions 4-8)
    let year = order_id.get(4..8)?;

    // Check that year is digits
    if !year.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Extract number (after second hyphen)
    let number = order_id.get(9..)?;

    if number.is_empty() {
        return None;
    }

    Some((year, number))
}
```

## Practical Example: Parsing Log Messages

```rust
fn main() {
    let logs = [
        "[INFO] Trade executed: BTC/USDT @ 42000",
        "[ERROR] Connection timeout",
        "[WARN] Low balance: 0.001 BTC",
        "Invalid log format",
    ];

    for log in logs {
        if let Some((level, message)) = parse_log(log) {
            println!("Level: {:6} | Message: {}", level, message);
        } else {
            println!("Invalid log: {}", log);
        }
    }
}

fn parse_log(log: &str) -> Option<(&str, &str)> {
    // Format: [LEVEL] message

    // Check if starts with [
    if !log.starts_with('[') {
        return None;
    }

    // Find closing bracket
    let end_bracket = log.find(']')?;

    // Extract level (between [ and ])
    let level = log.get(1..end_bracket)?;

    // Extract message (after ] and space)
    let message_start = end_bracket + 2;  // ] + space
    let message = log.get(message_start..)?;

    Some((level, message.trim()))
}
```

## Slices and Functions

Functions accepting `&str` work with both string literals and `String` slices:

```rust
fn main() {
    // String literal
    let literal: &str = "BINANCE";

    // String
    let owned = String::from("COINBASE");

    // Both work with the same function
    let binance_short = get_exchange_short_name(literal);
    let coinbase_short = get_exchange_short_name(&owned);

    println!("Binance: {} -> {}", literal, binance_short);
    println!("Coinbase: {} -> {}", owned, coinbase_short);

    // String slice also works
    let full_name = String::from("Kraken Exchange");
    let kraken_short = get_exchange_short_name(&full_name[0..6]);
    println!("Kraken: {}", kraken_short);
}

fn get_exchange_short_name(name: &str) -> &str {
    // Return first 3 characters or entire name if shorter
    if name.len() >= 3 {
        &name[0..3]
    } else {
        name
    }
}
```

## Practical Example: Extracting Prices from Text

```rust
fn main() {
    let messages = [
        "BTC price: $42,500.00",
        "ETH at $2,800.50 now",
        "Current rate: 1.0850",
    ];

    for msg in messages {
        if let Some(price_str) = extract_price_string(msg) {
            println!("Found price string: '{}' in '{}'", price_str, msg);
        } else {
            println!("No price found in: {}", msg);
        }
    }
}

fn extract_price_string(text: &str) -> Option<&str> {
    // Find start of number (digit or $)
    let start = text.find(|c: char| c.is_ascii_digit() || c == '$')?;

    // Determine actual number start (skip $)
    let num_start = if text.get(start..start+1) == Some("$") {
        start + 1
    } else {
        start
    };

    // Find end of number
    let slice = text.get(num_start..)?;
    let end_offset = slice
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != ',')
        .unwrap_or(slice.len());

    text.get(num_start..num_start + end_offset)
}
```

## Practical Example: Normalizing Tickers

```rust
fn main() {
    let tickers = [
        "  btc  ",
        "ETH/USDT",
        "sol",
        " DOGE ",
    ];

    for ticker in tickers {
        let normalized = normalize_ticker(ticker);
        println!("'{}' -> '{}'", ticker, normalized);
    }
}

fn normalize_ticker(ticker: &str) -> String {
    // Remove whitespace and convert to uppercase
    let trimmed = ticker.trim();

    // If contains /, take only base currency
    if let Some(slash_pos) = trimmed.find('/') {
        trimmed[..slash_pos].to_uppercase()
    } else {
        trimmed.to_uppercase()
    }
}
```

## Slices and Ownership

Slices **borrow** data, they don't own it:

```rust
fn main() {
    let exchange = String::from("Binance Futures");

    // Create slice - borrowing
    let name = &exchange[0..7];

    // exchange is still accessible!
    println!("Full: {}", exchange);
    println!("Name: {}", name);

    // But can't modify exchange while slice exists
    // exchange.push_str("!");  // ERROR!

    // After last use of slice, can modify
    drop(name);  // Explicitly "destroy" slice (usually not needed)

    let mut exchange = exchange;  // Make mutable
    exchange.push_str(" Pro");
    println!("Modified: {}", exchange);
}
```

## Comparison: Slices vs Copying

```rust
fn main() {
    let data = String::from("BINANCE:BTC/USDT:SPOT:1000.50");

    // Slices - efficient, no copying
    let parts: Vec<&str> = data.split(':').collect();
    println!("Exchange (slice): {}", parts[0]);

    // Copying - creates new String
    let exchange_owned: String = parts[0].to_string();
    println!("Exchange (owned): {}", exchange_owned);

    // When to use what:
    // - Slices: for reading, temporary processing
    // - String: when need to store/modify data
}
```

## Exercises

### Exercise 1: Exchange Message Parser
```rust
// Implement a function that extracts data from exchange message
// Format: "EXCHANGE:PAIR:SIDE:PRICE:QUANTITY"
// Example: "BINANCE:BTC/USDT:BUY:42000.50:0.5"

fn parse_exchange_message(msg: &str) -> Option<TradeInfo> {
    // TODO: implement
    todo!()
}

struct TradeInfo<'a> {
    exchange: &'a str,
    pair: &'a str,
    side: &'a str,
    price: &'a str,
    quantity: &'a str,
}
```

### Exercise 2: Extract Domain from Exchange URL
```rust
// Extract domain from URL
// "https://api.binance.com/v3/ticker" -> "binance.com"
// "https://ftx.com/api/markets" -> "ftx.com"

fn extract_domain(url: &str) -> Option<&str> {
    // TODO: implement
    todo!()
}
```

### Exercise 3: API Key Masking
```rust
// Mask API key, showing only first and last 4 characters
// "abcd1234efgh5678" -> "abcd********5678"

fn mask_api_key(key: &str) -> String {
    // TODO: implement
    todo!()
}
```

### Exercise 4: Trade Command Parser
```rust
// Parse command like "buy 0.5 BTC at 42000"
// Return struct with fields: action, amount, asset, price

fn parse_trade_command(cmd: &str) -> Option<TradeCommand> {
    // TODO: implement
    todo!()
}

struct TradeCommand<'a> {
    action: &'a str,
    amount: &'a str,
    asset: &'a str,
    price: &'a str,
}
```

## Homework

1. **WebSocket Message Parser**: Write a function that parses JSON-like messages:
   ```
   {"event":"trade","symbol":"BTC/USDT","price":"42000.50","side":"buy"}
   ```
   Extract field values using only slices (without serde).

2. **Historical Data Analyzer**: Create a function for parsing CSV line:
   ```
   2024-01-15,42000.00,42500.00,41800.00,42300.00,1500.5
   ```
   (date,open,high,low,close,volume)

3. **Ticker Filter**: Write a function that extracts only valid tickers from a list of strings (2-5 characters, letters only).

4. **API Router**: Create a function that extracts endpoint and parameters from URL:
   ```
   "/api/v1/ticker?symbol=BTCUSDT" -> (endpoint: "ticker", params: "symbol=BTCUSDT")
   ```

## What We Learned

| Concept | Description |
|---------|-------------|
| `&str` | String slice — reference to part of a string |
| `[start..end]` | Slice syntax |
| `get(range)` | Safe slice extraction |
| Bytes vs characters | Indices are bytes, not characters |
| Borrowing | Slices borrow data, don't copy |

## Navigation

[← Day 47](../047-ownership-rules/en.md) | [Day 49 →](../049-string-methods/en.md)
