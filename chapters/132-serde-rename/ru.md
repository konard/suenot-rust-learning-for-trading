# День 132: #[serde(rename)] — разные имена полей

## Аналогия из трейдинга

Представь, что ты работаешь с несколькими биржами одновременно. Binance отправляет тебе поле `"symbol"`, Kraken — `"pair"`, а Coinbase — `"product_id"`. Но внутри твоего кода ты хочешь использовать единое имя `ticker` для всех.

Это как переводчик, который переводит разные слова с разных языков на один понятный тебе язык. `#[serde(rename)]` — это такой переводчик между форматом данных биржи и твоим кодом.

## Базовое переименование

Когда имя поля в JSON отличается от имени поля в структуре:

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
    println!("ID сделки: {}", trade.id);
    println!("Пара: {}", trade.symbol);
    println!("Цена: ${:.2}", trade.price);
    println!("Объём: {}", trade.quantity);
}
```

**Важно:** В коде используем понятные имена (`id`, `symbol`), а в JSON они могут быть какими угодно.

## Зачем это нужно?

### 1. API биржи использует неудобные имена

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
    // Реальный ответ от Binance WebSocket
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
    println!("Символ: {}", ticker.symbol);
    println!("Текущая цена: ${}", ticker.last_price);
    println!("Лучший бид: ${}", ticker.best_bid);
    println!("Лучший аск: ${}", ticker.best_ask);
}
```

### 2. Rust-конвенции против JSON-конвенций

Rust использует `snake_case`, а JSON часто `camelCase`:

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
    println!("Последнее обновление: {}", order_book.last_update_id);
    println!("Спред: ${:.2}", order_book.ask_price - order_book.bid_price);
}
```

## Автоматическое переименование: rename_all

Вместо переименования каждого поля вручную, можно задать правило для всей структуры:

```rust
use serde::{Deserialize, Serialize};

// Все поля будут ожидаться в camelCase
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
    println!("24ч изменение: ${:.2}", data.price_change_24h);
}
```

### Доступные варианты rename_all

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

## Разные имена для сериализации и десериализации

Иногда API принимает данные в одном формате, а отдаёт в другом:

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
    // Десериализация: читаем длинные имена
    let response = r#"{"symbol": "ETHUSDT", "quantity": 2.5, "price": 2200.0}"#;
    let order: Order = serde_json::from_str(response).unwrap();
    println!("Получен ордер: {:?}", order);

    // Сериализация: отправляем короткие имена
    let request = serde_json::to_string(&order).unwrap();
    println!("Отправляем: {}", request);
    // Выведет: {"sym":"ETHUSDT","qty":2.5,"px":2200.0}
}
```

## Переименование вариантов enum

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
    #[serde(rename = "type")]  // "type" - ключевое слово Rust!
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
    println!("Ордер:\n{}", json);

    // Десериализация
    let response = r#"{
        "symbol": "BTCUSDT",
        "side": "SELL",
        "type": "STOP_LOSS",
        "quantity": 1.0
    }"#;

    let parsed: OrderRequest = serde_json::from_str(response).unwrap();
    println!("\nПолучен: {:?}", parsed);
}
```

## Практический пример: универсальный парсер бирж

```rust
use serde::Deserialize;

// Общий интерфейс для тикеров разных бирж
#[derive(Debug)]
struct UnifiedTicker {
    symbol: String,
    price: f64,
    volume_24h: f64,
}

// Формат Binance
#[derive(Deserialize)]
struct BinanceTicker {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c")]
    last_price: String,
    #[serde(rename = "v")]
    volume: String,
}

// Формат Kraken
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

// Формат Coinbase
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

    // Теперь можно сравнивать цены!
    let spread = coinbase_ticker.price - binance_ticker.price;
    println!("Разница цен: ${:.2}", spread);
}
```

## Алиасы: несколько имён для одного поля

Когда разные версии API используют разные имена:

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
    // Все эти JSON корректно парсятся в одну структуру
    let formats = [
        r#"{"trade_id": 1, "sym": "BTCUSDT", "px": 42500.0}"#,
        r#"{"id": 2, "pair": "BTCUSDT", "price": 42501.0}"#,
        r#"{"tid": 3, "symbol": "BTCUSDT", "executed_price": 42502.0}"#,
        r#"{"transaction_id": 4, "ticker": "BTCUSDT", "execution_price": 42503.0}"#,
    ];

    for json in formats {
        let trade: Trade = serde_json::from_str(json).unwrap();
        println!("Сделка #{}: {} @ ${:.2}",
            trade.transaction_id,
            trade.ticker,
            trade.execution_price
        );
    }
}
```

## Комбинирование с другими атрибутами

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Position {
    #[serde(rename = "sym")]  // Переопределяет rename_all для этого поля
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

    // Парсим обратно
    let parsed: Position = serde_json::from_str(&json).unwrap();
    println!("\nPnL: ${:.2}", parsed.unrealized_pnl);
}
```

## Что мы узнали

| Атрибут | Описание | Пример |
|---------|----------|--------|
| `#[serde(rename = "x")]` | Переименовать поле | `"trade_id"` -> `id` |
| `#[serde(rename_all = "...")]` | Переименовать все поля | `camelCase`, `SCREAMING_SNAKE_CASE` |
| `#[serde(rename(serialize = "...", deserialize = "..."))]` | Разные имена для чтения/записи | Разные API форматы |
| `#[serde(alias = "x")]` | Альтернативное имя при чтении | Совместимость версий |

## Практические задания

1. Создай структуру для парсинга ответа биржи, где все поля имеют однобуквенные имена (`p`, `q`, `s`, `t`), но в коде используй понятные имена

2. Напиши структуру `TradeHistory` с `rename_all = "camelCase"`, содержащую поля: `trade_id`, `executed_at`, `fill_price`, `fill_quantity`

3. Создай enum `TimeInForce` с вариантами `GoodTillCancel`, `ImmediateOrCancel`, `FillOrKill`, которые сериализуются как `"GTC"`, `"IOC"`, `"FOK"`

4. Реализуй структуру, которая принимает данные от "старого" API (с полями `symbol`, `amount`) и "нового" API (с полями `pair`, `quantity`), используя `alias`

## Домашнее задание

1. Создай универсальный парсер для трёх бирж (Binance, Kraken, Coinbase) с разными форматами данных, используя `rename` для маппинга полей

2. Напиши структуру `OrderRequest`, которая сериализуется в короткий формат (`sym`, `qty`, `px`) для отправки, но десериализуется из полного формата (`symbol`, `quantity`, `price`) при получении ответа

3. Создай enum `OrderStatus` с 10 различными статусами, которые сериализуются в формат биржи (например, `"PARTIALLY_FILLED"`, `"NEW"`, `"CANCELED"`)

4. Реализуй структуру `MultiExchangePosition`, которая может парсить позиции от разных бирж с совершенно разными форматами JSON, приводя их к единому виду

## Навигация

[← Предыдущий день](../131-optional-json-fields/ru.md) | [Следующий день →](../133-csv-historical-data/ru.md)
