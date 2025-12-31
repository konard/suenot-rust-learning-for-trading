# Day 132: #[serde(rename)] — Different Field Names

## Trading Analogy

Imagine you're working with multiple exchanges simultaneously. Binance sends you a field called `"symbol"`, Kraken sends `"pair"`, and Coinbase sends `"product_id"`. But inside your code, you want to use a unified name `ticker` for all of them.

It's like a translator who converts different words from different languages into one language you understand. `#[serde(rename)]` is that translator between the exchange data format and your code.

## Basic Renaming

When the JSON field name differs from the struct field name:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Trade {
    #[serde(rename = "trade_id")]
    id: u64,

    #[serde(rename = "trading_pair")]
    symbol: String,

    #[serde(rename = "executed_price")]
    price: f64,

    #[serde(rename = "executed_qty")]
    quantity: f64,
}

fn main() {
    let json = r#"{
        "trade_id": 12345,
        "trading_pair": "BTC/USDT",
        "executed_price": 42500.50,
        "executed_qty": 0.5
    }"#;

    let trade: Trade = serde_json::from_str(json).unwrap();
    println!("Trade ID: {}", trade.id);
    println!("Pair: {}", trade.symbol);
    println!("Price: ${:.2}", trade.price);
    println!("Quantity: {}", trade.quantity);
}
```

**Important:** In code we use clear names (`id`, `symbol`), while JSON can have any naming convention.

## Why Do We Need This?

### 1. Exchange API Uses Inconvenient Names

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BinanceTicker {
    #[serde(rename = "s")]
    symbol: String,

    #[serde(rename = "p")]
    price_change: String,

    #[serde(rename = "P")]
    price_change_percent: String,

    #[serde(rename = "w")]
    weighted_avg_price: String,

    #[serde(rename = "c")]
    last_price: String,

    #[serde(rename = "Q")]
    last_quantity: String,

    #[serde(rename = "b")]
    best_bid: String,

    #[serde(rename = "a")]
    best_ask: String,
}

fn main() {
    // Real response from Binance WebSocket
    let json = r#"{
        "s": "BTCUSDT",
        "p": "-150.00",
        "P": "-0.35",
        "w": "42350.50",
        "c": "42500.00",
        "Q": "0.001",
        "b": "42499.99",
        "a": "42500.01"
    }"#;

    let ticker: BinanceTicker = serde_json::from_str(json).unwrap();
    println!("Symbol: {}", ticker.symbol);
    println!("Last price: ${}", ticker.last_price);
    println!("Best bid: ${}", ticker.best_bid);
    println!("Best ask: ${}", ticker.best_ask);
}
```

### 2. Rust Conventions vs JSON Conventions

Rust uses `snake_case`, while JSON often uses `camelCase`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct OrderBook {
    #[serde(rename = "lastUpdateId")]
    last_update_id: u64,

    #[serde(rename = "bidPrice")]
    bid_price: f64,

    #[serde(rename = "bidQty")]
    bid_quantity: f64,

    #[serde(rename = "askPrice")]
    ask_price: f64,

    #[serde(rename = "askQty")]
    ask_quantity: f64,
}

fn main() {
    let json = r#"{
        "lastUpdateId": 1234567890,
        "bidPrice": 42499.99,
        "bidQty": 1.5,
        "askPrice": 42500.01,
        "askQty": 2.0
    }"#;

    let order_book: OrderBook = serde_json::from_str(json).unwrap();
    println!("Last update: {}", order_book.last_update_id);
    println!("Spread: ${:.2}", order_book.ask_price - order_book.bid_price);
}
```

## Automatic Renaming: rename_all

Instead of renaming each field manually, you can set a rule for the entire struct:

```rust
use serde::{Deserialize, Serialize};

// All fields will be expected in camelCase
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarketData {
    symbol_name: String,
    last_price: f64,
    price_change_24h: f64,
    volume_24h: f64,
    high_price_24h: f64,
    low_price_24h: f64,
}

fn main() {
    let json = r#"{
        "symbolName": "BTC/USDT",
        "lastPrice": 42500.0,
        "priceChange24h": -150.0,
        "volume24h": 1234567.89,
        "highPrice24h": 43000.0,
        "lowPrice24h": 42000.0
    }"#;

    let data: MarketData = serde_json::from_str(json).unwrap();
    println!("{}: ${:.2}", data.symbol_name, data.last_price);
    println!("24h change: ${:.2}", data.price_change_24h);
}
```

### Available rename_all Options

```rust
use serde::{Deserialize, Serialize};

// snake_case -> camelCase
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CamelExample {
    order_id: u64,      // -> "orderId"
    trade_price: f64,   // -> "tradePrice"
}

// snake_case -> PascalCase
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PascalExample {
    order_id: u64,      // -> "OrderId"
    trade_price: f64,   // -> "TradePrice"
}

// snake_case -> SCREAMING_SNAKE_CASE
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct ScreamingExample {
    order_id: u64,      // -> "ORDER_ID"
    trade_price: f64,   // -> "TRADE_PRICE"
}

// snake_case -> kebab-case
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct KebabExample {
    order_id: u64,      // -> "order-id"
    trade_price: f64,   // -> "trade-price"
}

fn main() {
    let camel = CamelExample { order_id: 1, trade_price: 100.0 };
    println!("camelCase: {}", serde_json::to_string(&camel).unwrap());

    let pascal = PascalExample { order_id: 1, trade_price: 100.0 };
    println!("PascalCase: {}", serde_json::to_string(&pascal).unwrap());

    let screaming = ScreamingExample { order_id: 1, trade_price: 100.0 };
    println!("SCREAMING: {}", serde_json::to_string(&screaming).unwrap());

    let kebab = KebabExample { order_id: 1, trade_price: 100.0 };
    println!("kebab-case: {}", serde_json::to_string(&kebab).unwrap());
}
```

## Different Names for Serialization and Deserialization

Sometimes APIs accept data in one format but return it in another:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    #[serde(rename(serialize = "sym", deserialize = "symbol"))]
    ticker: String,

    #[serde(rename(serialize = "qty", deserialize = "quantity"))]
    amount: f64,

    #[serde(rename(serialize = "px", deserialize = "price"))]
    price: f64,
}

fn main() {
    // Deserialization: reading long names
    let response = r#"{"symbol": "ETHUSDT", "quantity": 2.5, "price": 2200.0}"#;
    let order: Order = serde_json::from_str(response).unwrap();
    println!("Received order: {:?}", order);

    // Serialization: sending short names
    let request = serde_json::to_string(&order).unwrap();
    println!("Sending: {}", request);
    // Outputs: {"sym":"ETHUSDT","qty":2.5,"px":2200.0}
}
```

## Renaming Enum Variants

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize)]
enum OrderType {
    #[serde(rename = "LIMIT")]
    Limit,

    #[serde(rename = "MARKET")]
    Market,

    #[serde(rename = "STOP_LOSS")]
    StopLoss,

    #[serde(rename = "TAKE_PROFIT")]
    TakeProfit,

    #[serde(rename = "STOP_LOSS_LIMIT")]
    StopLossLimit,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderRequest {
    symbol: String,
    side: OrderSide,
    #[serde(rename = "type")]  // "type" is a Rust keyword!
    order_type: OrderType,
    quantity: f64,
}

fn main() {
    let order = OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: 0.5,
    };

    let json = serde_json::to_string_pretty(&order).unwrap();
    println!("Order:\n{}", json);

    // Deserialization
    let response = r#"{
        "symbol": "BTCUSDT",
        "side": "SELL",
        "type": "STOP_LOSS",
        "quantity": 1.0
    }"#;

    let parsed: OrderRequest = serde_json::from_str(response).unwrap();
    println!("\nReceived: {:?}", parsed);
}
```

## Practical Example: Universal Exchange Parser

```rust
use serde::Deserialize;

// Common interface for tickers from different exchanges
#[derive(Debug)]
struct UnifiedTicker {
    symbol: String,
    price: f64,
    volume_24h: f64,
}

// Binance format
#[derive(Deserialize)]
struct BinanceTicker {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c")]
    last_price: String,
    #[serde(rename = "v")]
    volume: String,
}

// Kraken format
#[derive(Deserialize)]
struct KrakenTicker {
    #[serde(rename = "a")]
    ask: Vec<String>,
    #[serde(rename = "b")]
    bid: Vec<String>,
    #[serde(rename = "c")]
    last_trade: Vec<String>,
    #[serde(rename = "v")]
    volume: Vec<String>,
}

// Coinbase format
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct CoinbaseTicker {
    product_id: String,
    price: String,
    volume_24h: String,
}

impl From<BinanceTicker> for UnifiedTicker {
    fn from(t: BinanceTicker) -> Self {
        UnifiedTicker {
            symbol: t.symbol,
            price: t.last_price.parse().unwrap_or(0.0),
            volume_24h: t.volume.parse().unwrap_or(0.0),
        }
    }
}

impl From<CoinbaseTicker> for UnifiedTicker {
    fn from(t: CoinbaseTicker) -> Self {
        UnifiedTicker {
            symbol: t.product_id,
            price: t.price.parse().unwrap_or(0.0),
            volume_24h: t.volume_24h.parse().unwrap_or(0.0),
        }
    }
}

fn parse_binance(json: &str) -> UnifiedTicker {
    let ticker: BinanceTicker = serde_json::from_str(json).unwrap();
    ticker.into()
}

fn parse_coinbase(json: &str) -> UnifiedTicker {
    let ticker: CoinbaseTicker = serde_json::from_str(json).unwrap();
    ticker.into()
}

fn main() {
    let binance_json = r#"{"s":"BTCUSDT","c":"42500.00","v":"12345.67"}"#;
    let coinbase_json = r#"{"product_id":"BTC-USD","price":"42510.00","volume_24h":"9876.54"}"#;

    let binance_ticker = parse_binance(binance_json);
    let coinbase_ticker = parse_coinbase(coinbase_json);

    println!("Binance:  {:?}", binance_ticker);
    println!("Coinbase: {:?}", coinbase_ticker);

    // Now we can compare prices!
    let spread = coinbase_ticker.price - binance_ticker.price;
    println!("Price difference: ${:.2}", spread);
}
```

## Aliases: Multiple Names for One Field

When different API versions use different names:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Trade {
    #[serde(alias = "trade_id", alias = "id", alias = "tid")]
    transaction_id: u64,

    #[serde(alias = "sym", alias = "pair", alias = "symbol")]
    ticker: String,

    #[serde(alias = "px", alias = "price", alias = "executed_price")]
    execution_price: f64,
}

fn main() {
    // All these JSON formats correctly parse into the same struct
    let formats = [
        r#"{"trade_id": 1, "sym": "BTCUSDT", "px": 42500.0}"#,
        r#"{"id": 2, "pair": "BTCUSDT", "price": 42501.0}"#,
        r#"{"tid": 3, "symbol": "BTCUSDT", "executed_price": 42502.0}"#,
        r#"{"transaction_id": 4, "ticker": "BTCUSDT", "execution_price": 42503.0}"#,
    ];

    for json in formats {
        let trade: Trade = serde_json::from_str(json).unwrap();
        println!("Trade #{}: {} @ ${:.2}",
            trade.transaction_id,
            trade.ticker,
            trade.execution_price
        );
    }
}
```

## Combining with Other Attributes

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Position {
    #[serde(rename = "sym")]  // Overrides rename_all for this field
    symbol: String,

    entry_price: f64,  // -> "entryPrice"

    current_price: f64,  // -> "currentPrice"

    #[serde(rename = "qty")]
    quantity: f64,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stopLoss")]
    stop_loss: Option<f64>,

    #[serde(default)]
    unrealized_pnl: f64,  // -> "unrealizedPnl"
}

fn main() {
    let position = Position {
        symbol: "ETHUSDT".to_string(),
        entry_price: 2200.0,
        current_price: 2250.0,
        quantity: 5.0,
        stop_loss: Some(2100.0),
        unrealized_pnl: 250.0,
    };

    let json = serde_json::to_string_pretty(&position).unwrap();
    println!("JSON:\n{}", json);

    // Parse back
    let parsed: Position = serde_json::from_str(&json).unwrap();
    println!("\nPnL: ${:.2}", parsed.unrealized_pnl);
}
```

## What We Learned

| Attribute | Description | Example |
|-----------|-------------|---------|
| `#[serde(rename = "x")]` | Rename a field | `"trade_id"` -> `id` |
| `#[serde(rename_all = "...")]` | Rename all fields | `camelCase`, `SCREAMING_SNAKE_CASE` |
| `#[serde(rename(serialize = "...", deserialize = "..."))]` | Different names for read/write | Different API formats |
| `#[serde(alias = "x")]` | Alternative name when reading | Version compatibility |

## Practice Exercises

1. Create a struct for parsing exchange responses where all fields have single-letter names (`p`, `q`, `s`, `t`), but use descriptive names in your code

2. Write a `TradeHistory` struct with `rename_all = "camelCase"`, containing fields: `trade_id`, `executed_at`, `fill_price`, `fill_quantity`

3. Create an enum `TimeInForce` with variants `GoodTillCancel`, `ImmediateOrCancel`, `FillOrKill`, which serialize as `"GTC"`, `"IOC"`, `"FOK"`

4. Implement a struct that accepts data from an "old" API (with fields `symbol`, `amount`) and a "new" API (with fields `pair`, `quantity`), using `alias`

## Homework

1. Create a universal parser for three exchanges (Binance, Kraken, Coinbase) with different data formats, using `rename` to map fields

2. Write an `OrderRequest` struct that serializes to a short format (`sym`, `qty`, `px`) for sending, but deserializes from a full format (`symbol`, `quantity`, `price`) when receiving responses

3. Create an enum `OrderStatus` with 10 different statuses that serialize to exchange format (e.g., `"PARTIALLY_FILLED"`, `"NEW"`, `"CANCELED"`)

4. Implement a `MultiExchangePosition` struct that can parse positions from different exchanges with completely different JSON formats, normalizing them to a unified representation

## Navigation

[← Previous day](../131-optional-json-fields/en.md) | [Next day →](../133-csv-historical-data/en.md)
