# Day 131: Optional JSON Fields

## Trading Analogy

Imagine you're receiving trade data from different exchanges. Some exchanges send **complete information** about a trade: price, volume, timestamp, commission, order type. Other exchanges might **not send** certain fields — for example, commission might be absent from the API response.

In the real world of trading APIs, this is a **normal situation**:
- The `stop_loss` field exists only for orders with stop-loss
- The `take_profit` field might be absent
- The `leverage` field exists only on futures markets
- The `filled_at` field appears only after order execution

In Rust, we use `Option<T>` together with serde for such cases.

## Option Basics in JSON

### Basic Example: Order with Optional Stop-Loss

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    stop_loss: Option<f64>,      // May be absent
    take_profit: Option<f64>,    // May be absent
}

fn main() {
    // JSON with optional fields
    let json_with_stops = r#"
    {
        "symbol": "BTC/USDT",
        "side": "buy",
        "price": 42000.0,
        "quantity": 0.5,
        "stop_loss": 41000.0,
        "take_profit": 45000.0
    }
    "#;

    // JSON without optional fields
    let json_without_stops = r#"
    {
        "symbol": "ETH/USDT",
        "side": "sell",
        "price": 2500.0,
        "quantity": 2.0
    }
    "#;

    let order1: Order = serde_json::from_str(json_with_stops).unwrap();
    let order2: Order = serde_json::from_str(json_without_stops).unwrap();

    println!("Order 1: {:?}", order1);
    println!("Order 2: {:?}", order2);

    // Working with optional fields
    match order1.stop_loss {
        Some(sl) => println!("Stop Loss set: ${:.2}", sl),
        None => println!("Stop Loss not set"),
    }
}
```

## #[serde(skip_serializing_if)] — Skip None During Serialization

By default, serde serializes `Option<T>` as `null`. To **completely skip** a field with `None`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TradeSignal {
    symbol: String,
    action: String,
    entry_price: f64,

    #[serde(skip_serializing_if = "Option::is_none")]
    stop_loss: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    take_profit: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    trailing_stop: Option<f64>,
}

fn main() {
    let signal = TradeSignal {
        symbol: "BTC/USDT".to_string(),
        action: "BUY".to_string(),
        entry_price: 42000.0,
        stop_loss: Some(41000.0),
        take_profit: None,          // Will be skipped
        trailing_stop: None,        // Will be skipped
    };

    let json = serde_json::to_string_pretty(&signal).unwrap();
    println!("{}", json);
    // Output:
    // {
    //   "symbol": "BTC/USDT",
    //   "action": "BUY",
    //   "entry_price": 42000.0,
    //   "stop_loss": 41000.0
    // }
}
```

## #[serde(default)] — Default Values

If a field is absent in JSON, you can use a default value:

```rust
use serde::{Deserialize, Serialize};

fn default_leverage() -> f64 {
    1.0  // No leverage
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Serialize, Deserialize)]
struct FuturesPosition {
    symbol: String,
    size: f64,
    entry_price: f64,

    #[serde(default = "default_leverage")]
    leverage: f64,

    #[serde(default = "default_false")]
    is_isolated: bool,

    #[serde(default)]  // Uses Default trait (0.0 for f64)
    unrealized_pnl: f64,
}

fn main() {
    let json = r#"
    {
        "symbol": "BTC/USDT",
        "size": 0.5,
        "entry_price": 42000.0
    }
    "#;

    let position: FuturesPosition = serde_json::from_str(json).unwrap();

    println!("Symbol: {}", position.symbol);
    println!("Leverage: {}x", position.leverage);     // 1.0 (default)
    println!("Isolated: {}", position.is_isolated);   // false (default)
    println!("PnL: ${:.2}", position.unrealized_pnl); // 0.0 (default)
}
```

## Combining default and skip_serializing_if

```rust
use serde::{Deserialize, Serialize};

fn is_zero(value: &f64) -> bool {
    *value == 0.0
}

fn is_one(value: &f64) -> bool {
    *value == 1.0
}

#[derive(Debug, Serialize, Deserialize)]
struct TradingConfig {
    symbol: String,

    #[serde(default = "default_risk_percent", skip_serializing_if = "is_default_risk")]
    risk_percent: f64,

    #[serde(default, skip_serializing_if = "is_zero")]
    max_drawdown: f64,

    #[serde(default = "default_one", skip_serializing_if = "is_one")]
    position_multiplier: f64,
}

fn default_risk_percent() -> f64 { 2.0 }
fn default_one() -> f64 { 1.0 }
fn is_default_risk(value: &f64) -> bool { *value == 2.0 }

fn main() {
    // Minimal JSON
    let json = r#"{"symbol": "ETH/USDT"}"#;
    let config: TradingConfig = serde_json::from_str(json).unwrap();

    println!("Risk: {}%", config.risk_percent);           // 2.0
    println!("Max Drawdown: {}%", config.max_drawdown);   // 0.0
    println!("Multiplier: {}x", config.position_multiplier); // 1.0

    // Serialization back - default values not included
    let json_out = serde_json::to_string(&config).unwrap();
    println!("JSON: {}", json_out);  // {"symbol":"ETH/USDT"}
}
```

## Nested Optional Structures

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct RiskManagement {
    max_position_size: f64,
    max_daily_loss: f64,
    trailing_stop_percent: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Strategy {
    name: String,
    timeframe: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    risk_management: Option<RiskManagement>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

fn main() {
    // Strategy with full risk configuration
    let strategy_with_risk = Strategy {
        name: "Scalping".to_string(),
        timeframe: "1m".to_string(),
        risk_management: Some(RiskManagement {
            max_position_size: 1000.0,
            max_daily_loss: 100.0,
            trailing_stop_percent: Some(0.5),
        }),
        description: Some("Quick trades with tight stops".to_string()),
    };

    // Simple strategy without risk management
    let simple_strategy = Strategy {
        name: "HODL".to_string(),
        timeframe: "1d".to_string(),
        risk_management: None,
        description: None,
    };

    println!("Full strategy:\n{}", serde_json::to_string_pretty(&strategy_with_risk).unwrap());
    println!("\nSimple strategy:\n{}", serde_json::to_string_pretty(&simple_strategy).unwrap());
}
```

## Processing Optional Fields in Code

### Pattern: Safe Data Extraction

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct MarketData {
    symbol: String,
    price: f64,
    volume_24h: Option<f64>,
    high_24h: Option<f64>,
    low_24h: Option<f64>,
    change_percent: Option<f64>,
}

fn analyze_market(data: &MarketData) {
    println!("=== {} ===", data.symbol);
    println!("Price: ${:.2}", data.price);

    // unwrap_or — default value
    println!("Volume: ${:.0}", data.volume_24h.unwrap_or(0.0));

    // if let — conditional processing
    if let Some(change) = data.change_percent {
        let direction = if change >= 0.0 { "📈" } else { "📉" };
        println!("Change: {}{:.2}%", direction, change);
    }

    // map — transform optional value
    let volatility = data.high_24h
        .zip(data.low_24h)
        .map(|(high, low)| ((high - low) / low) * 100.0);

    match volatility {
        Some(v) => println!("Volatility: {:.2}%", v),
        None => println!("Volatility: N/A"),
    }
}

fn main() {
    let full_data = r#"
    {
        "symbol": "BTC/USDT",
        "price": 42000.0,
        "volume_24h": 1500000000.0,
        "high_24h": 43000.0,
        "low_24h": 41000.0,
        "change_percent": 2.5
    }
    "#;

    let partial_data = r#"
    {
        "symbol": "NEW/USDT",
        "price": 0.001
    }
    "#;

    let data1: MarketData = serde_json::from_str(full_data).unwrap();
    let data2: MarketData = serde_json::from_str(partial_data).unwrap();

    analyze_market(&data1);
    println!();
    analyze_market(&data2);
}
```

## Practical Example: Order Book with Optional Fields

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct OrderBookLevel {
    price: f64,
    quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    orders_count: Option<u32>,  // Number of orders at the level
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderBook {
    symbol: String,
    timestamp: u64,
    bids: Vec<OrderBookLevel>,
    asks: Vec<OrderBookLevel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    last_trade_price: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    spread: Option<f64>,
}

impl OrderBook {
    fn calculate_spread(&self) -> Option<f64> {
        let best_bid = self.bids.first().map(|l| l.price);
        let best_ask = self.asks.first().map(|l| l.price);

        match (best_bid, best_ask) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    fn total_bid_volume(&self) -> f64 {
        self.bids.iter().map(|l| l.quantity).sum()
    }

    fn total_ask_volume(&self) -> f64 {
        self.asks.iter().map(|l| l.quantity).sum()
    }

    fn order_imbalance(&self) -> f64 {
        let bid_vol = self.total_bid_volume();
        let ask_vol = self.total_ask_volume();
        let total = bid_vol + ask_vol;

        if total > 0.0 {
            (bid_vol - ask_vol) / total
        } else {
            0.0
        }
    }
}

fn main() {
    let json = r#"
    {
        "symbol": "BTC/USDT",
        "timestamp": 1704067200000,
        "bids": [
            {"price": 42000.0, "quantity": 1.5, "orders_count": 3},
            {"price": 41990.0, "quantity": 2.0},
            {"price": 41980.0, "quantity": 0.8}
        ],
        "asks": [
            {"price": 42010.0, "quantity": 1.2},
            {"price": 42020.0, "quantity": 3.0, "orders_count": 5},
            {"price": 42030.0, "quantity": 1.0}
        ],
        "last_trade_price": 42005.0
    }
    "#;

    let orderbook: OrderBook = serde_json::from_str(json).unwrap();

    println!("Symbol: {}", orderbook.symbol);
    println!("Spread: ${:.2}", orderbook.calculate_spread().unwrap_or(0.0));
    println!("Bid Volume: {:.2} BTC", orderbook.total_bid_volume());
    println!("Ask Volume: {:.2} BTC", orderbook.total_ask_volume());
    println!("Order Imbalance: {:.2}", orderbook.order_imbalance());

    if let Some(last_price) = orderbook.last_trade_price {
        println!("Last Trade: ${:.2}", last_price);
    }
}
```

## What We Learned

| Attribute | Description | Use Case |
|-----------|-------------|----------|
| `Option<T>` | Field may be absent | `stop_loss: Option<f64>` |
| `#[serde(skip_serializing_if)]` | Skip during serialization | Don't include None in JSON |
| `#[serde(default)]` | Default value | Leverage = 1.0 if not specified |
| `#[serde(default = "fn")]` | Custom default value | Risk = 2% by default |

## Practical Exercises

1. **Parsing Exchange API Response**: Create a `Ticker` struct with required fields (symbol, price) and optional fields (volume, change_24h, high, low). Write a function that safely extracts all data.

2. **Trading Bot Configuration**: Create a `BotConfig` struct where most fields have default values. User should only need to specify symbol.

3. **Trade Filtering**: Write a function that takes a JSON array of trades with optional `fee` field and returns only trades where commission was specified.

4. **Generating Trading Report**: Create a `TradeReport` struct with optional sections (summary, details, risk_metrics). Serialize only filled sections.

## Homework

1. Create an `ExchangeResponse<T>` struct with optional fields `data: Option<T>`, `error: Option<String>`, `warning: Option<String>`. Implement an `is_success()` method.

2. Write a parser for different order formats:
   - Spot order (no leverage, no liquidation_price)
   - Margin order (with leverage, no liquidation_price)
   - Futures order (with leverage, with liquidation_price)

3. Create a `PriceAlert` system where you can specify:
   - Required: symbol, target_price
   - Optional: expiry_time, repeat_count, notification_type

4. Implement a function `merge_configs(base: Config, override: Config) -> Config` that merges two configurations, where optional fields from override overwrite base.

## Navigation

[← Previous day](../130-nested-json-structures/en.md) | [Next day →](../132-serde-rename/en.md)
