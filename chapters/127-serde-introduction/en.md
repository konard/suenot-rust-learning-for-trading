# Day 127: Serialization: serde Introduction

## Trading Analogy

Imagine you want to send trade information to your broker or save it to a file. You can't just "teleport" an object from memory — you need to **convert it to text** (or bytes) that can later be read back.

It's like a trading report:
- **Serialization** — you write down the trade on paper (Order → JSON/text)
- **Deserialization** — you read the report and restore the information (JSON/text → Order)

**serde** is a Rust library that does this automatically. The name is a shorthand for **ser**ialize + **de**serialize.

## What is serde?

serde is a serialization framework for Rust:
- **Fast** — zero-cost abstractions in many cases
- **Flexible** — supports JSON, TOML, YAML, MessagePack, and dozens of other formats
- **Safe** — compile-time type checking

## Adding serde to Your Project

```toml
# Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"  # For working with JSON
```

## Basic Example: Trade Structure

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Trade {
    symbol: String,
    side: String,        // "BUY" or "SELL"
    price: f64,
    quantity: f64,
    timestamp: u64,
}

fn main() {
    // Create a trade
    let trade = Trade {
        symbol: String::from("BTC/USDT"),
        side: String::from("BUY"),
        price: 42000.50,
        quantity: 0.5,
        timestamp: 1703980800,
    };

    // Serialization: Trade → JSON string
    let json = serde_json::to_string(&trade).unwrap();
    println!("JSON: {}", json);

    // Deserialization: JSON string → Trade
    let restored: Trade = serde_json::from_str(&json).unwrap();
    println!("Restored: {:?}", restored);
}
```

Output:
```
JSON: {"symbol":"BTC/USDT","side":"BUY","price":42000.5,"quantity":0.5,"timestamp":1703980800}
Restored: Trade { symbol: "BTC/USDT", side: "BUY", price: 42000.5, quantity: 0.5, timestamp: 1703980800 }
```

## Derive Macros: Rust Magic

The key feature of serde is the `Serialize` and `Deserialize` macros:

```rust
use serde::{Serialize, Deserialize};

// Add #[derive(...)] and the struct automatically knows how to
// convert to/from different formats
#[derive(Serialize, Deserialize)]
struct Order {
    id: u64,
    symbol: String,
    order_type: String,
    price: f64,
    quantity: f64,
}
```

Rust generates all the necessary code for you!

## Pretty JSON

For debugging, formatted output is convenient:

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Portfolio {
    name: String,
    balance: f64,
    positions: Vec<Position>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

fn main() {
    let portfolio = Portfolio {
        name: String::from("Main Trading Account"),
        balance: 10000.0,
        positions: vec![
            Position {
                symbol: String::from("BTC"),
                quantity: 0.5,
                avg_price: 42000.0,
            },
            Position {
                symbol: String::from("ETH"),
                quantity: 5.0,
                avg_price: 2200.0,
            },
        ],
    };

    // Pretty JSON with indentation
    let pretty_json = serde_json::to_string_pretty(&portfolio).unwrap();
    println!("{}", pretty_json);
}
```

Output:
```json
{
  "name": "Main Trading Account",
  "balance": 10000.0,
  "positions": [
    {
      "symbol": "BTC",
      "quantity": 0.5,
      "avg_price": 42000.0
    },
    {
      "symbol": "ETH",
      "quantity": 5.0,
      "avg_price": 2200.0
    }
  ]
}
```

## Serializing Enums: Order Types

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

#[derive(Debug, Serialize, Deserialize)]
enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    symbol: String,
    order_type: OrderType,
    side: Side,
    price: Option<f64>,  // None for Market orders
    quantity: f64,
}

fn main() {
    let limit_order = Order {
        symbol: String::from("ETH/USDT"),
        order_type: OrderType::Limit,
        side: Side::Buy,
        price: Some(2200.0),
        quantity: 2.0,
    };

    let market_order = Order {
        symbol: String::from("BTC/USDT"),
        order_type: OrderType::Market,
        side: Side::Sell,
        price: None,
        quantity: 0.1,
    };

    println!("Limit: {}", serde_json::to_string(&limit_order).unwrap());
    println!("Market: {}", serde_json::to_string(&market_order).unwrap());
}
```

Output:
```
Limit: {"symbol":"ETH/USDT","order_type":"Limit","side":"Buy","price":2200.0,"quantity":2.0}
Market: {"symbol":"BTC/USDT","order_type":"Market","side":"Sell","price":null,"quantity":0.1}
```

## Working with Vec and HashMap

```rust
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() {
    // Vector of candles
    let candles = vec![
        Candle { open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0, volume: 1000.0 },
        Candle { open: 42300.0, high: 42800.0, low: 42100.0, close: 42600.0, volume: 1200.0 },
    ];

    let json = serde_json::to_string_pretty(&candles).unwrap();
    println!("Candles:\n{}", json);

    // HashMap with balances
    let mut balances: HashMap<String, f64> = HashMap::new();
    balances.insert(String::from("BTC"), 0.5);
    balances.insert(String::from("ETH"), 5.0);
    balances.insert(String::from("USDT"), 10000.0);

    let balances_json = serde_json::to_string_pretty(&balances).unwrap();
    println!("\nBalances:\n{}", balances_json);
}
```

## Error Handling During Deserialization

In real trading, data can be incorrect:

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

fn parse_price_update(json_str: &str) -> Result<PriceUpdate, String> {
    serde_json::from_str(json_str)
        .map_err(|e| format!("Parse error: {}", e))
}

fn main() {
    // Valid JSON
    let valid = r#"{"symbol":"BTC/USDT","price":42000.0,"timestamp":1703980800}"#;
    match parse_price_update(valid) {
        Ok(update) => println!("Received update: {:?}", update),
        Err(e) => println!("Error: {}", e),
    }

    // Invalid JSON (price as string instead of number)
    let invalid = r#"{"symbol":"BTC/USDT","price":"high","timestamp":1703980800}"#;
    match parse_price_update(invalid) {
        Ok(update) => println!("Received update: {:?}", update),
        Err(e) => println!("Error: {}", e),
    }

    // Incomplete JSON (missing field)
    let missing_field = r#"{"symbol":"BTC/USDT","price":42000.0}"#;
    match parse_price_update(missing_field) {
        Ok(update) => println!("Received update: {:?}", update),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Saving and Loading from File

```rust
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct TradingConfig {
    api_endpoint: String,
    max_position_size: f64,
    risk_per_trade: f64,
    symbols: Vec<String>,
}

fn save_config(config: &TradingConfig, path: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| e.to_string())?;
    fs::write(path, json)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn load_config(path: &str) -> Result<TradingConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&content)
        .map_err(|e| e.to_string())
}

fn main() {
    let config = TradingConfig {
        api_endpoint: String::from("https://api.exchange.com"),
        max_position_size: 1000.0,
        risk_per_trade: 0.02,
        symbols: vec![
            String::from("BTC/USDT"),
            String::from("ETH/USDT"),
            String::from("SOL/USDT"),
        ],
    };

    // Save
    if let Err(e) = save_config(&config, "trading_config.json") {
        println!("Save error: {}", e);
        return;
    }
    println!("Configuration saved!");

    // Load
    match load_config("trading_config.json") {
        Ok(loaded) => println!("Loaded: {:?}", loaded),
        Err(e) => println!("Load error: {}", e),
    }
}
```

## Practical Example: Trade History

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct TradeHistory {
    account_id: String,
    trades: Vec<TradeRecord>,
    total_pnl: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradeRecord {
    id: u64,
    symbol: String,
    side: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
    timestamp: u64,
}

fn calculate_pnl(side: &str, entry: f64, exit: f64, qty: f64) -> f64 {
    match side {
        "BUY" => (exit - entry) * qty,
        "SELL" => (entry - exit) * qty,
        _ => 0.0,
    }
}

fn main() {
    let trades = vec![
        TradeRecord {
            id: 1,
            symbol: String::from("BTC/USDT"),
            side: String::from("BUY"),
            entry_price: 42000.0,
            exit_price: 43500.0,
            quantity: 0.5,
            pnl: calculate_pnl("BUY", 42000.0, 43500.0, 0.5),
            timestamp: 1703980800,
        },
        TradeRecord {
            id: 2,
            symbol: String::from("ETH/USDT"),
            side: String::from("SELL"),
            entry_price: 2300.0,
            exit_price: 2200.0,
            quantity: 2.0,
            pnl: calculate_pnl("SELL", 2300.0, 2200.0, 2.0),
            timestamp: 1703984400,
        },
    ];

    let total: f64 = trades.iter().map(|t| t.pnl).sum();

    let history = TradeHistory {
        account_id: String::from("ACC-001"),
        trades,
        total_pnl: total,
    };

    let json = serde_json::to_string_pretty(&history).unwrap();
    println!("{}", json);

    // Statistics
    println!("\n═══════════════════════════════");
    println!("Total PnL: ${:.2}", history.total_pnl);
    println!("Number of trades: {}", history.trades.len());
    println!("═══════════════════════════════");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Serialize` | Trait for converting to a format (JSON, TOML, etc.) |
| `Deserialize` | Trait for restoring from a format |
| `#[derive(...)]` | Automatic implementation generation |
| `serde_json` | Crate for working with JSON |
| `to_string()` | Serialize to compact string |
| `to_string_pretty()` | Serialize with formatting |
| `from_str()` | Deserialize from string |

## Practical Exercises

1. Create an `OHLCV` structure (Open, High, Low, Close, Volume) and serialize an array of candles to JSON.

2. Write a function that loads trading bot configuration from a JSON file and validates the data.

3. Create an `OrderStatus` enum (Pending, Filled, Cancelled, Rejected) and serialize an order structure with this status.

4. Write a program that reads JSON with price updates and outputs only those where the price changed by more than 1%.

## Homework

1. Create a trade logging system:
   - `TradeLog` structure with fields: id, timestamp, action, details
   - Function to save log to file (append, not overwrite)
   - Function to load and display all logs

2. Implement portfolio serialization:
   - `Portfolio` structure with positions, balance, and metadata
   - Save portfolio state to file every N seconds
   - Restore portfolio on program restart

3. Create a parser for a simplified exchange API:
   - Different message types (trade, orderbook, ticker)
   - Handle parsing errors
   - Statistics on received messages

4. Write a format converter:
   - Read data from JSON
   - Save in another format (e.g., simple CSV-like text)
   - Reverse conversion

## Navigation

[← Previous day](../126-path-pathbuf/en.md) | [Next day →](../128-json-exchange-api/en.md)
