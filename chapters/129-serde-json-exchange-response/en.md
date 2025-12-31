# Day 129: serde_json: Parsing Exchange Response

## Trading Analogy

When you make a request to an exchange API, it responds in JSON — a text data format. It's like receiving a telegram in a foreign language: you need a translator. `serde_json` is your translator that converts JSON text into Rust structures and back.

## Adding Dependencies

In `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Basic Parsing: From JSON to Struct

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Ticker {
    symbol: String,
    price: f64,
}

fn main() {
    // Exchange response in JSON format
    let json_response = r#"{"symbol": "BTCUSDT", "price": 42150.50}"#;

    // Parse JSON into struct
    let ticker: Ticker = serde_json::from_str(json_response).unwrap();

    println!("Ticker: {}", ticker.symbol);
    println!("Price: ${:.2}", ticker.price);
}
```

**Important:** The `#[derive(Deserialize)]` attribute automatically generates parsing code!

## Handling Parsing Errors

```rust
use serde::Deserialize;
use serde_json::Error;

#[derive(Debug, Deserialize)]
struct Trade {
    id: u64,
    price: f64,
    qty: f64,
    side: String,
}

fn parse_trade(json: &str) -> Result<Trade, Error> {
    serde_json::from_str(json)
}

fn main() {
    // Valid JSON
    let valid_json = r#"{"id": 12345, "price": 42000.0, "qty": 0.5, "side": "BUY"}"#;
    match parse_trade(valid_json) {
        Ok(trade) => println!("Trade #{}: {} {} @ ${}",
            trade.id, trade.side, trade.qty, trade.price),
        Err(e) => println!("Parsing error: {}", e),
    }

    // Invalid JSON
    let invalid_json = r#"{"id": "not_a_number"}"#;
    match parse_trade(invalid_json) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Parsing Exchange Response: Real Example

```rust
use serde::Deserialize;

// Structure for exchange balance response
#[derive(Debug, Deserialize)]
struct AccountBalance {
    asset: String,
    free: String,      // Exchanges often return numbers as strings!
    locked: String,
}

#[derive(Debug, Deserialize)]
struct AccountInfo {
    #[serde(rename = "makerCommission")]
    maker_commission: u32,
    #[serde(rename = "takerCommission")]
    taker_commission: u32,
    balances: Vec<AccountBalance>,
}

fn main() {
    let exchange_response = r#"{
        "makerCommission": 10,
        "takerCommission": 10,
        "balances": [
            {"asset": "BTC", "free": "0.5", "locked": "0.1"},
            {"asset": "USDT", "free": "10000.0", "locked": "0.0"},
            {"asset": "ETH", "free": "5.0", "locked": "0.5"}
        ]
    }"#;

    let account: AccountInfo = serde_json::from_str(exchange_response).unwrap();

    println!("Maker commission: {}%", account.maker_commission as f64 / 100.0);
    println!("Taker commission: {}%", account.taker_commission as f64 / 100.0);
    println!("\nBalances:");

    for balance in &account.balances {
        let free: f64 = balance.free.parse().unwrap_or(0.0);
        let locked: f64 = balance.locked.parse().unwrap_or(0.0);
        if free > 0.0 || locked > 0.0 {
            println!("  {}: free {}, locked {}",
                balance.asset, free, locked);
        }
    }
}
```

## Parsing Array of Trades

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RecentTrade {
    id: u64,
    price: String,
    qty: String,
    time: u64,
    #[serde(rename = "isBuyerMaker")]
    is_buyer_maker: bool,
}

fn main() {
    let trades_json = r#"[
        {"id": 1001, "price": "42150.00", "qty": "0.1", "time": 1703001200000, "isBuyerMaker": false},
        {"id": 1002, "price": "42155.50", "qty": "0.25", "time": 1703001201000, "isBuyerMaker": true},
        {"id": 1003, "price": "42160.00", "qty": "0.5", "time": 1703001202000, "isBuyerMaker": false}
    ]"#;

    let trades: Vec<RecentTrade> = serde_json::from_str(trades_json).unwrap();

    println!("Recent trades:");
    for trade in &trades {
        let price: f64 = trade.price.parse().unwrap();
        let qty: f64 = trade.qty.parse().unwrap();
        let side = if trade.is_buyer_maker { "SELL" } else { "BUY" };

        println!("  #{}: {} {:.4} BTC @ ${:.2}", trade.id, side, qty, price);
    }

    // Calculate volume-weighted average price
    let total_volume: f64 = trades.iter()
        .map(|t| t.qty.parse::<f64>().unwrap())
        .sum();

    let vwap: f64 = trades.iter()
        .map(|t| {
            let p: f64 = t.price.parse().unwrap();
            let q: f64 = t.qty.parse().unwrap();
            p * q
        })
        .sum::<f64>() / total_volume;

    println!("\nVWAP: ${:.2}", vwap);
}
```

## Parsing Order Book

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OrderBook {
    #[serde(rename = "lastUpdateId")]
    last_update_id: u64,
    bids: Vec<(String, String)>,  // (price, quantity)
    asks: Vec<(String, String)>,
}

fn main() {
    let orderbook_json = r#"{
        "lastUpdateId": 28912345,
        "bids": [
            ["42150.00", "1.5"],
            ["42149.00", "2.0"],
            ["42148.00", "0.8"]
        ],
        "asks": [
            ["42151.00", "1.2"],
            ["42152.00", "3.0"],
            ["42155.00", "0.5"]
        ]
    }"#;

    let orderbook: OrderBook = serde_json::from_str(orderbook_json).unwrap();

    println!("Order Book (ID: {})", orderbook.last_update_id);
    println!("\n{:^15} | {:^15}", "BIDS", "ASKS");
    println!("{:-^15}-+-{:-^15}", "", "");

    let max_levels = orderbook.bids.len().max(orderbook.asks.len());

    for i in 0..max_levels {
        let bid = orderbook.bids.get(i)
            .map(|(p, q)| format!("{} x {}", p, q))
            .unwrap_or_default();
        let ask = orderbook.asks.get(i)
            .map(|(p, q)| format!("{} x {}", p, q))
            .unwrap_or_default();

        println!("{:>15} | {:<15}", bid, ask);
    }

    // Calculate spread
    if let (Some(best_bid), Some(best_ask)) =
        (orderbook.bids.first(), orderbook.asks.first())
    {
        let bid_price: f64 = best_bid.0.parse().unwrap();
        let ask_price: f64 = best_ask.0.parse().unwrap();
        let spread = ask_price - bid_price;
        let spread_pct = (spread / bid_price) * 100.0;

        println!("\nSpread: ${:.2} ({:.4}%)", spread, spread_pct);
    }
}
```

## Serialization: From Struct to JSON

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct OrderRequest {
    symbol: String,
    side: String,
    #[serde(rename = "type")]
    order_type: String,
    quantity: f64,
    price: Option<f64>,
}

fn main() {
    // Create order to send to exchange
    let order = OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        order_type: "LIMIT".to_string(),
        quantity: 0.01,
        price: Some(42000.0),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&order).unwrap();
    println!("Request (compact):\n{}\n", json);

    // Pretty print with indentation
    let json_pretty = serde_json::to_string_pretty(&order).unwrap();
    println!("Request (formatted):\n{}", json_pretty);
}
```

## Working with serde_json::Value (Dynamic JSON)

When the response structure is unknown in advance:

```rust
use serde_json::{Value, json};

fn main() {
    // Unknown response structure
    let response = r#"{
        "status": "ok",
        "data": {
            "price": 42150.50,
            "change24h": -2.5,
            "volume": 1234567.89
        },
        "timestamp": 1703001200
    }"#;

    // Parse into dynamic Value
    let v: Value = serde_json::from_str(response).unwrap();

    // Access fields via indexing
    println!("Status: {}", v["status"]);
    println!("Price: {}", v["data"]["price"]);
    println!("24h change: {}%", v["data"]["change24h"]);

    // Safe access with type checking
    if let Some(price) = v["data"]["price"].as_f64() {
        println!("Price as f64: {:.2}", price);
    }

    // Create JSON programmatically
    let request = json!({
        "method": "subscribe",
        "params": ["btcusdt@ticker", "ethusdt@ticker"],
        "id": 1
    });

    println!("\nSubscription request:\n{}",
        serde_json::to_string_pretty(&request).unwrap());
}
```

## Practical Example: Parsing OHLCV Candles

```rust
use serde::Deserialize;

// Tuple struct for candles (Binance format)
// [open_time, open, high, low, close, volume, ...]
#[derive(Debug, Deserialize)]
struct Kline(
    u64,    // Open time
    String, // Open
    String, // High
    String, // Low
    String, // Close
    String, // Volume
    u64,    // Close time
    String, // Quote asset volume
    u64,    // Number of trades
    String, // Taker buy base asset volume
    String, // Taker buy quote asset volume
    String, // Ignore
);

struct Candle {
    open_time: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl From<Kline> for Candle {
    fn from(k: Kline) -> Self {
        Candle {
            open_time: k.0,
            open: k.1.parse().unwrap_or(0.0),
            high: k.2.parse().unwrap_or(0.0),
            low: k.3.parse().unwrap_or(0.0),
            close: k.4.parse().unwrap_or(0.0),
            volume: k.5.parse().unwrap_or(0.0),
        }
    }
}

fn main() {
    let klines_json = r#"[
        [1703001200000, "42000.00", "42100.00", "41900.00", "42050.00", "100.5", 1703001259999, "4225275.00", 1500, "50.25", "2112637.50", "0"],
        [1703001260000, "42050.00", "42150.00", "42000.00", "42100.00", "85.2", 1703001319999, "3587160.00", 1200, "42.60", "1793580.00", "0"],
        [1703001320000, "42100.00", "42200.00", "42050.00", "42180.00", "120.8", 1703001379999, "5091984.00", 1800, "60.40", "2545992.00", "0"]
    ]"#;

    let klines: Vec<Kline> = serde_json::from_str(klines_json).unwrap();
    let candles: Vec<Candle> = klines.into_iter().map(Candle::from).collect();

    println!("OHLCV Candles:");
    println!("{:^13} | {:^10} | {:^10} | {:^10} | {:^10} | {:^10}",
        "Time", "Open", "High", "Low", "Close", "Volume");
    println!("{:-^13}-+-{:-^10}-+-{:-^10}-+-{:-^10}-+-{:-^10}-+-{:-^10}",
        "", "", "", "", "", "");

    for candle in &candles {
        println!("{:>13} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2}",
            candle.open_time / 1000, // Convert to seconds
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume);
    }

    // Calculate price change
    if let (Some(first), Some(last)) = (candles.first(), candles.last()) {
        let change = last.close - first.open;
        let change_pct = (change / first.open) * 100.0;
        println!("\nChange: ${:.2} ({:.2}%)", change, change_pct);
    }
}
```

## Error Handling During Parsing

```rust
use serde::Deserialize;
use serde_json::Error;

#[derive(Debug, Deserialize)]
struct Price {
    symbol: String,
    price: f64,
}

fn parse_prices(json: &str) -> Result<Vec<Price>, Error> {
    serde_json::from_str(json)
}

fn main() {
    // Test 1: Valid JSON
    let valid = r#"[{"symbol": "BTCUSDT", "price": 42000.0}]"#;
    match parse_prices(valid) {
        Ok(prices) => {
            for p in prices {
                println!("{}: ${}", p.symbol, p.price);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 2: Invalid JSON syntax
    let invalid_syntax = r#"{"symbol": "BTCUSDT", price: 42000}"#;
    match parse_prices(invalid_syntax) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Syntax error: {}", e),
    }

    // Test 3: Wrong data type
    let wrong_type = r#"[{"symbol": "BTCUSDT", "price": "not_a_number"}]"#;
    match parse_prices(wrong_type) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Type error: {}", e),
    }

    // Test 4: Missing required field
    let missing_field = r#"[{"symbol": "BTCUSDT"}]"#;
    match parse_prices(missing_field) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Missing field: {}", e),
    }
}
```

## What We Learned

| Operation | Method | Description |
|-----------|--------|-------------|
| JSON → Struct | `serde_json::from_str()` | Parse JSON string |
| JSON → Value | `serde_json::from_str()` | Dynamic parsing |
| Struct → JSON | `serde_json::to_string()` | Compact serialization |
| Struct → JSON | `serde_json::to_string_pretty()` | Formatted output |
| Attribute | `#[derive(Deserialize)]` | Auto-generate parser |
| Attribute | `#[derive(Serialize)]` | Auto-generate serializer |
| Rename | `#[serde(rename = "...")]` | Different name in JSON |

## Homework

1. Write a parser for a WebSocket ticker message from exchange. Structure:
   ```json
   {"e": "24hrTicker", "s": "BTCUSDT", "c": "42150.00", "P": "-2.5"}
   ```

2. Create a structure for API response with balances and write a function that finds all assets with non-zero balance

3. Implement a parser for user trade history response:
   ```json
   [{"id": 1, "symbol": "BTCUSDT", "price": "42000", "qty": "0.1", "side": "BUY", "time": 1703001200000}]
   ```
   Calculate total buy and sell volumes

4. Write a function that takes `serde_json::Value` and safely extracts a price, returning `Option<f64>`

## Navigation

[← Previous day](../128-json-exchange-api/en.md) | [Next day →](../130-nested-json/en.md)
