// Test compilation of code examples from Chapter 132

use serde::{Deserialize, Serialize};

// Basic renaming test
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

// Binance ticker format test
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

// rename_all test
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

// Different rename_all options
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CamelExample {
    order_id: u64,
    trade_price: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PascalExample {
    order_id: u64,
    trade_price: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct ScreamingExample {
    order_id: u64,
    trade_price: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct KebabExample {
    order_id: u64,
    trade_price: f64,
}

// Different names for serialize/deserialize
#[derive(Debug, Serialize, Deserialize)]
struct Order {
    #[serde(rename(serialize = "sym", deserialize = "symbol"))]
    ticker: String,

    #[serde(rename(serialize = "qty", deserialize = "quantity"))]
    amount: f64,

    #[serde(rename(serialize = "px", deserialize = "price"))]
    price: f64,
}

// Enum renaming
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
    #[serde(rename = "type")]
    order_type: OrderType,
    quantity: f64,
}

// Alias test
#[derive(Debug, Deserialize)]
struct TradeWithAlias {
    #[serde(alias = "trade_id", alias = "id", alias = "tid")]
    transaction_id: u64,

    #[serde(alias = "sym", alias = "pair", alias = "symbol")]
    ticker: String,

    #[serde(alias = "px", alias = "price", alias = "executed_price")]
    execution_price: f64,
}

// Combined attributes test
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Position {
    #[serde(rename = "sym")]
    symbol: String,

    entry_price: f64,

    current_price: f64,

    #[serde(rename = "qty")]
    quantity: f64,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stopLoss")]
    stop_loss: Option<f64>,

    #[serde(default)]
    unrealized_pnl: f64,
}

fn main() {
    println!("=== Basic Renaming Test ===");
    let json = r#"{
        "trade_id": 12345,
        "trading_pair": "BTC/USDT",
        "executed_price": 42500.50,
        "executed_qty": 0.5
    }"#;
    let trade: Trade = serde_json::from_str(json).unwrap();
    println!("Trade ID: {}", trade.id);
    println!("Pair: {}", trade.symbol);

    println!("\n=== Binance Ticker Test ===");
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

    println!("\n=== rename_all camelCase Test ===");
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

    println!("\n=== Different rename_all Options ===");
    let camel = CamelExample { order_id: 1, trade_price: 100.0 };
    println!("camelCase: {}", serde_json::to_string(&camel).unwrap());

    let pascal = PascalExample { order_id: 1, trade_price: 100.0 };
    println!("PascalCase: {}", serde_json::to_string(&pascal).unwrap());

    let screaming = ScreamingExample { order_id: 1, trade_price: 100.0 };
    println!("SCREAMING: {}", serde_json::to_string(&screaming).unwrap());

    let kebab = KebabExample { order_id: 1, trade_price: 100.0 };
    println!("kebab-case: {}", serde_json::to_string(&kebab).unwrap());

    println!("\n=== Serialize/Deserialize Different Names Test ===");
    let response = r#"{"symbol": "ETHUSDT", "quantity": 2.5, "price": 2200.0}"#;
    let order: Order = serde_json::from_str(response).unwrap();
    println!("Received order: {:?}", order);
    let request = serde_json::to_string(&order).unwrap();
    println!("Sending: {}", request);

    println!("\n=== Enum Renaming Test ===");
    let order = OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: 0.5,
    };
    let json = serde_json::to_string_pretty(&order).unwrap();
    println!("Order:\n{}", json);

    println!("\n=== Alias Test ===");
    let formats = [
        r#"{"trade_id": 1, "sym": "BTCUSDT", "px": 42500.0}"#,
        r#"{"id": 2, "pair": "BTCUSDT", "price": 42501.0}"#,
        r#"{"tid": 3, "symbol": "BTCUSDT", "executed_price": 42502.0}"#,
        r#"{"transaction_id": 4, "ticker": "BTCUSDT", "execution_price": 42503.0}"#,
    ];
    for json in formats {
        let trade: TradeWithAlias = serde_json::from_str(json).unwrap();
        println!("Trade #{}: {} @ ${:.2}",
            trade.transaction_id,
            trade.ticker,
            trade.execution_price
        );
    }

    println!("\n=== Combined Attributes Test ===");
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

    println!("\n=== All tests passed! ===");
}
