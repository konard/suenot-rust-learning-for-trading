# Day 128: JSON: Exchange API Format

## Trading Analogy

Imagine an exchange is a foreign partner you need to communicate with. You need a **common language** for data exchange: prices, orders, balances. JSON (JavaScript Object Notation) is exactly that universal language. Every crypto exchange (Binance, Bybit, OKX) and broker speaks JSON.

When you request the current Bitcoin price, the exchange responds something like:

```json
{
  "symbol": "BTCUSDT",
  "price": "42150.50",
  "timestamp": 1704067200000
}
```

This is JSON — structured text that's easy for humans to read and easy for programs to process.

## What is JSON?

JSON is a text format for representing structured data. It consists of:

- **Objects** `{}` — key-value pairs
- **Arrays** `[]` — ordered lists
- **Strings** `"text"` — in double quotes
- **Numbers** `42`, `3.14` — integers and decimals
- **Booleans** `true`, `false`
- **null** — absence of value

## Real Examples from Exchange APIs

### Ticker Information

```json
{
  "symbol": "BTCUSDT",
  "lastPrice": "42150.50",
  "bidPrice": "42149.00",
  "askPrice": "42151.00",
  "volume": "12543.789",
  "high24h": "43200.00",
  "low24h": "41500.00"
}
```

### Order Book

```json
{
  "symbol": "ETHUSDT",
  "bids": [
    ["2250.00", "10.5"],
    ["2249.50", "25.3"],
    ["2249.00", "15.8"]
  ],
  "asks": [
    ["2250.50", "8.2"],
    ["2251.00", "12.1"],
    ["2251.50", "20.0"]
  ],
  "timestamp": 1704067200000
}
```

### Account Information

```json
{
  "accountType": "SPOT",
  "balances": [
    {
      "asset": "BTC",
      "free": "0.5",
      "locked": "0.1"
    },
    {
      "asset": "USDT",
      "free": "10000.00",
      "locked": "500.00"
    }
  ],
  "canTrade": true,
  "canWithdraw": true
}
```

### Order

```json
{
  "orderId": 123456789,
  "symbol": "BTCUSDT",
  "side": "BUY",
  "type": "LIMIT",
  "price": "42000.00",
  "origQty": "0.5",
  "executedQty": "0.3",
  "status": "PARTIALLY_FILLED",
  "timeInForce": "GTC",
  "createTime": 1704067200000
}
```

## Working with JSON in Rust: serde_json

For JSON handling in Rust, we use the `serde_json` library. Add to your `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Creating JSON Programmatically

```rust
use serde_json::{json, Value};

fn main() {
    // Create JSON using the json! macro
    let ticker: Value = json!({
        "symbol": "BTCUSDT",
        "price": 42150.50,
        "volume": 12543.789,
        "change24h": 2.35
    });

    println!("Ticker JSON:\n{}", serde_json::to_string_pretty(&ticker).unwrap());

    // Access fields
    println!("\nSymbol: {}", ticker["symbol"]);
    println!("Price: {}", ticker["price"]);
}
```

### Parsing JSON String

```rust
use serde_json::Value;

fn main() {
    let json_str = r#"
    {
        "symbol": "ETHUSDT",
        "lastPrice": "2250.50",
        "bidPrice": "2250.00",
        "askPrice": "2251.00",
        "volume": "45678.123"
    }
    "#;

    // Parse JSON
    let ticker: Value = serde_json::from_str(json_str).unwrap();

    // Extract data
    let symbol = ticker["symbol"].as_str().unwrap();
    let price: f64 = ticker["lastPrice"].as_str().unwrap().parse().unwrap();
    let bid: f64 = ticker["bidPrice"].as_str().unwrap().parse().unwrap();
    let ask: f64 = ticker["askPrice"].as_str().unwrap().parse().unwrap();

    let spread = ask - bid;
    let spread_percent = (spread / bid) * 100.0;

    println!("Symbol: {}", symbol);
    println!("Price: ${:.2}", price);
    println!("Bid: ${:.2} | Ask: ${:.2}", bid, ask);
    println!("Spread: ${:.2} ({:.4}%)", spread, spread_percent);
}
```

### Typed Deserialization

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Ticker {
    symbol: String,
    #[serde(rename = "lastPrice")]
    last_price: String,
    #[serde(rename = "bidPrice")]
    bid_price: String,
    #[serde(rename = "askPrice")]
    ask_price: String,
    volume: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderBook {
    symbol: String,
    bids: Vec<[String; 2]>,  // [price, quantity]
    asks: Vec<[String; 2]>,
    timestamp: u64,
}

fn main() {
    let json_str = r#"
    {
        "symbol": "BTCUSDT",
        "lastPrice": "42150.50",
        "bidPrice": "42149.00",
        "askPrice": "42151.00",
        "volume": "12543.789"
    }
    "#;

    let ticker: Ticker = serde_json::from_str(json_str).unwrap();

    println!("Parsed ticker: {:?}", ticker);
    println!("Symbol: {}", ticker.symbol);
    println!("Last price: {}", ticker.last_price);
}
```

### Serialization (Rust -> JSON)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    symbol: String,
    side: String,
    #[serde(rename = "type")]
    order_type: String,
    quantity: f64,
    price: Option<f64>,  // None for market orders
}

fn main() {
    // Limit buy order
    let limit_order = Order {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        order_type: "LIMIT".to_string(),
        quantity: 0.5,
        price: Some(42000.0),
    };

    // Market sell order
    let market_order = Order {
        symbol: "ETHUSDT".to_string(),
        side: "SELL".to_string(),
        order_type: "MARKET".to_string(),
        quantity: 1.0,
        price: None,
    };

    println!("Limit order JSON:");
    println!("{}", serde_json::to_string_pretty(&limit_order).unwrap());

    println!("\nMarket order JSON:");
    println!("{}", serde_json::to_string_pretty(&market_order).unwrap());
}
```

## Working with Nested Structures

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Balance {
    asset: String,
    free: String,
    locked: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountInfo {
    #[serde(rename = "accountType")]
    account_type: String,
    balances: Vec<Balance>,
    #[serde(rename = "canTrade")]
    can_trade: bool,
    #[serde(rename = "canWithdraw")]
    can_withdraw: bool,
}

fn main() {
    let json_str = r#"
    {
        "accountType": "SPOT",
        "balances": [
            {"asset": "BTC", "free": "0.5", "locked": "0.1"},
            {"asset": "ETH", "free": "5.0", "locked": "0.0"},
            {"asset": "USDT", "free": "10000.00", "locked": "500.00"}
        ],
        "canTrade": true,
        "canWithdraw": true
    }
    "#;

    let account: AccountInfo = serde_json::from_str(json_str).unwrap();

    println!("Account Type: {}", account.account_type);
    println!("Can Trade: {}", account.can_trade);
    println!("\nBalances:");

    for balance in &account.balances {
        let free: f64 = balance.free.parse().unwrap_or(0.0);
        let locked: f64 = balance.locked.parse().unwrap_or(0.0);
        let total = free + locked;

        if total > 0.0 {
            println!(
                "  {} - Free: {}, Locked: {}, Total: {}",
                balance.asset, balance.free, balance.locked, total
            );
        }
    }
}
```

## Handling Parsing Errors

```rust
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, Serialize, Deserialize)]
struct Price {
    symbol: String,
    price: String,
}

fn parse_price(json_str: &str) -> Result<Price> {
    serde_json::from_str(json_str)
}

fn main() {
    // Valid JSON
    let valid_json = r#"{"symbol": "BTCUSDT", "price": "42000.00"}"#;

    match parse_price(valid_json) {
        Ok(price) => println!("Parsed: {} = {}", price.symbol, price.price),
        Err(e) => println!("Error: {}", e),
    }

    // Invalid JSON (missing field)
    let invalid_json = r#"{"symbol": "BTCUSDT"}"#;

    match parse_price(invalid_json) {
        Ok(price) => println!("Parsed: {:?}", price),
        Err(e) => println!("Error parsing: {}", e),
    }

    // Broken syntax
    let broken_json = r#"{"symbol": BTCUSDT}"#;

    match parse_price(broken_json) {
        Ok(price) => println!("Parsed: {:?}", price),
        Err(e) => println!("Syntax error: {}", e),
    }
}
```

## Practical Example: Order Book Analysis

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct OrderBook {
    symbol: String,
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
    timestamp: u64,
}

struct OrderBookAnalysis {
    best_bid: f64,
    best_ask: f64,
    spread: f64,
    spread_percent: f64,
    bid_depth: f64,
    ask_depth: f64,
    imbalance: f64,
}

fn analyze_order_book(order_book: &OrderBook) -> OrderBookAnalysis {
    let best_bid: f64 = order_book.bids[0][0].parse().unwrap();
    let best_ask: f64 = order_book.asks[0][0].parse().unwrap();
    let spread = best_ask - best_bid;
    let spread_percent = (spread / best_bid) * 100.0;

    // Total volume on bid and ask sides
    let bid_depth: f64 = order_book.bids
        .iter()
        .map(|level| level[1].parse::<f64>().unwrap())
        .sum();

    let ask_depth: f64 = order_book.asks
        .iter()
        .map(|level| level[1].parse::<f64>().unwrap())
        .sum();

    // Imbalance: positive = more buyers
    let imbalance = (bid_depth - ask_depth) / (bid_depth + ask_depth);

    OrderBookAnalysis {
        best_bid,
        best_ask,
        spread,
        spread_percent,
        bid_depth,
        ask_depth,
        imbalance,
    }
}

fn main() {
    let json_str = r#"
    {
        "symbol": "BTCUSDT",
        "bids": [
            ["42149.00", "2.5"],
            ["42148.00", "5.0"],
            ["42147.00", "10.0"],
            ["42146.00", "8.5"],
            ["42145.00", "15.0"]
        ],
        "asks": [
            ["42151.00", "1.8"],
            ["42152.00", "3.5"],
            ["42153.00", "7.0"],
            ["42154.00", "4.2"],
            ["42155.00", "9.5"]
        ],
        "timestamp": 1704067200000
    }
    "#;

    let order_book: OrderBook = serde_json::from_str(json_str).unwrap();
    let analysis = analyze_order_book(&order_book);

    println!("╔══════════════════════════════════════════╗");
    println!("║        ORDER BOOK ANALYSIS               ║");
    println!("║        {}                        ║", order_book.symbol);
    println!("╠══════════════════════════════════════════╣");
    println!("║ Best Bid:      ${:>18.2}    ║", analysis.best_bid);
    println!("║ Best Ask:      ${:>18.2}    ║", analysis.best_ask);
    println!("║ Spread:        ${:>18.2}    ║", analysis.spread);
    println!("║ Spread %:       {:>18.4}%   ║", analysis.spread_percent);
    println!("╠══════════════════════════════════════════╣");
    println!("║ Bid Depth:      {:>18.2} BTC ║", analysis.bid_depth);
    println!("║ Ask Depth:      {:>18.2} BTC ║", analysis.ask_depth);
    println!("║ Imbalance:      {:>18.2}%    ║", analysis.imbalance * 100.0);
    println!("╚══════════════════════════════════════════╝");

    if analysis.imbalance > 0.2 {
        println!("\n>> Strong buying pressure detected!");
    } else if analysis.imbalance < -0.2 {
        println!("\n>> Strong selling pressure detected!");
    } else {
        println!("\n>> Market is balanced");
    }
}
```

## Optional Fields with Default Values

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Trade {
    id: u64,
    symbol: String,
    price: String,
    quantity: String,
    #[serde(default)]
    is_buyer_maker: bool,
    #[serde(default = "default_commission")]
    commission: String,
}

fn default_commission() -> String {
    "0.0".to_string()
}

fn main() {
    // JSON without optional fields
    let json_str = r#"
    {
        "id": 123456,
        "symbol": "BTCUSDT",
        "price": "42000.00",
        "quantity": "0.5"
    }
    "#;

    let trade: Trade = serde_json::from_str(json_str).unwrap();
    println!("Trade: {:?}", trade);
    println!("Commission (default): {}", trade.commission);

    // JSON with all fields
    let json_str_full = r#"
    {
        "id": 123457,
        "symbol": "BTCUSDT",
        "price": "42100.00",
        "quantity": "1.0",
        "is_buyer_maker": true,
        "commission": "0.001"
    }
    "#;

    let trade_full: Trade = serde_json::from_str(json_str_full).unwrap();
    println!("\nFull trade: {:?}", trade_full);
}
```

## What We Learned

| Concept | Description | Example |
|---------|-------------|---------|
| JSON object | Key-value pairs | `{"symbol": "BTC"}` |
| JSON array | Ordered list | `["BTC", "ETH"]` |
| Parsing | JSON -> Rust | `serde_json::from_str()` |
| Serialization | Rust -> JSON | `serde_json::to_string()` |
| `#[derive]` | Automatic implementation | `Serialize, Deserialize` |
| `#[serde(rename)]` | Field renaming | `rename = "lastPrice"` |
| `#[serde(default)]` | Default value | For optional fields |

## Practical Exercises

1. **Ticker Parsing**: Write a program that parses ticker JSON and displays:
   - Symbol and current price
   - 24-hour change percentage
   - Bid/ask ratio

2. **Balance Analysis**: Create a structure for account info and a function that:
   - Parses JSON with balances
   - Filters out assets with zero balance
   - Calculates total portfolio value in USDT (use provided prices)

3. **Order Converter**: Write a function that:
   - Takes order parameters (symbol, side, type, quantity, price)
   - Creates JSON for sending to exchange
   - Validates input data

4. **API Error Handling**: Create a structure for exchange errors:
   ```json
   {"code": -1121, "msg": "Invalid symbol."}
   ```
   And a function that determines if the response is successful or an error.

## Homework

1. Write a parser for WebSocket trade stream:
   ```json
   {
     "e": "trade",
     "s": "BTCUSDT",
     "p": "42150.50",
     "q": "0.5",
     "T": 1704067200000,
     "m": true
   }
   ```
   Create a `TradeEvent` struct and `parse_trade_event` function.

2. Implement a candlestick history analyzer:
   - Parse JSON array of OHLCV data
   - Calculate SMA, high, and low for the period
   - Determine trend (uptrend/downtrend/sideways)

3. Create an exchange response simulator:
   - Structures for different response types (order, balance, error)
   - A function that generates random responses
   - A handler that correctly reacts to each type

4. Write a converter between different exchange formats:
   - Binance and Bybit use different field names
   - Create a universal `UnifiedTicker` structure
   - Implement `From<BinanceTicker>` and `From<BybitTicker>` traits

## Navigation

[← Previous day](../127-serde-introduction/en.md) | [Next day →](../129-serde-json-parsing/en.md)
