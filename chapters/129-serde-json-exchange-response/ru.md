# День 129: serde_json: парсим ответ биржи

## Аналогия из трейдинга

Когда ты делаешь запрос к API биржи, она отвечает на языке JSON — текстовом формате данных. Это как получить телеграмму на иностранном языке: тебе нужен переводчик. `serde_json` — это твой переводчик, который превращает текст JSON в структуры Rust и обратно.

## Подключаем зависимости

В `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Базовый парсинг: от JSON к структуре

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Ticker {
    symbol: String,
    price: f64,
}

fn main() {
    // Ответ биржи в формате JSON
    let json_response = r#"{"symbol": "BTCUSDT", "price": 42150.50}"#;

    // Парсим JSON в структуру
    let ticker: Ticker = serde_json::from_str(json_response).unwrap();

    println!("Тикер: {}", ticker.symbol);
    println!("Цена: ${:.2}", ticker.price);
}
```

**Важно:** Атрибут `#[derive(Deserialize)]` автоматически генерирует код парсинга!

## Обработка ошибок парсинга

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
    // Корректный JSON
    let valid_json = r#"{"id": 12345, "price": 42000.0, "qty": 0.5, "side": "BUY"}"#;
    match parse_trade(valid_json) {
        Ok(trade) => println!("Сделка #{}: {} {} @ ${}",
            trade.id, trade.side, trade.qty, trade.price),
        Err(e) => println!("Ошибка парсинга: {}", e),
    }

    // Некорректный JSON
    let invalid_json = r#"{"id": "not_a_number"}"#;
    match parse_trade(invalid_json) {
        Ok(_) => println!("Успех"),
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Парсинг ответа биржи: реальный пример

```rust
use serde::Deserialize;

// Структура для ответа биржи с балансами
#[derive(Debug, Deserialize)]
struct AccountBalance {
    asset: String,
    free: String,      // Биржи часто возвращают числа как строки!
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

    println!("Комиссия мейкера: {}%", account.maker_commission as f64 / 100.0);
    println!("Комиссия тейкера: {}%", account.taker_commission as f64 / 100.0);
    println!("\nБалансы:");

    for balance in &account.balances {
        let free: f64 = balance.free.parse().unwrap_or(0.0);
        let locked: f64 = balance.locked.parse().unwrap_or(0.0);
        if free > 0.0 || locked > 0.0 {
            println!("  {}: свободно {}, заблокировано {}",
                balance.asset, free, locked);
        }
    }
}
```

## Парсинг массива сделок

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

    println!("Последние сделки:");
    for trade in &trades {
        let price: f64 = trade.price.parse().unwrap();
        let qty: f64 = trade.qty.parse().unwrap();
        let side = if trade.is_buyer_maker { "SELL" } else { "BUY" };

        println!("  #{}: {} {:.4} BTC @ ${:.2}", trade.id, side, qty, price);
    }

    // Рассчитываем средневзвешенную цену
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

## Парсинг стакана заявок (Order Book)

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OrderBook {
    #[serde(rename = "lastUpdateId")]
    last_update_id: u64,
    bids: Vec<(String, String)>,  // (цена, количество)
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

    // Рассчитываем спред
    if let (Some(best_bid), Some(best_ask)) =
        (orderbook.bids.first(), orderbook.asks.first())
    {
        let bid_price: f64 = best_bid.0.parse().unwrap();
        let ask_price: f64 = best_ask.0.parse().unwrap();
        let spread = ask_price - bid_price;
        let spread_pct = (spread / bid_price) * 100.0;

        println!("\nСпред: ${:.2} ({:.4}%)", spread, spread_pct);
    }
}
```

## Сериализация: от структуры к JSON

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
    // Создаём ордер для отправки на биржу
    let order = OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        order_type: "LIMIT".to_string(),
        quantity: 0.01,
        price: Some(42000.0),
    };

    // Сериализуем в JSON
    let json = serde_json::to_string(&order).unwrap();
    println!("Запрос (компактный):\n{}\n", json);

    // Красивый вывод с отступами
    let json_pretty = serde_json::to_string_pretty(&order).unwrap();
    println!("Запрос (форматированный):\n{}", json_pretty);
}
```

## Работа с serde_json::Value (динамический JSON)

Когда структура ответа неизвестна заранее:

```rust
use serde_json::{Value, json};

fn main() {
    // Неизвестная структура ответа
    let response = r#"{
        "status": "ok",
        "data": {
            "price": 42150.50,
            "change24h": -2.5,
            "volume": 1234567.89
        },
        "timestamp": 1703001200
    }"#;

    // Парсим в динамический Value
    let v: Value = serde_json::from_str(response).unwrap();

    // Доступ к полям через индексацию
    println!("Статус: {}", v["status"]);
    println!("Цена: {}", v["data"]["price"]);
    println!("Изменение за 24ч: {}%", v["data"]["change24h"]);

    // Безопасный доступ с проверкой типа
    if let Some(price) = v["data"]["price"].as_f64() {
        println!("Цена как f64: {:.2}", price);
    }

    // Создаём JSON программно
    let request = json!({
        "method": "subscribe",
        "params": ["btcusdt@ticker", "ethusdt@ticker"],
        "id": 1
    });

    println!("\nЗапрос подписки:\n{}",
        serde_json::to_string_pretty(&request).unwrap());
}
```

## Практический пример: парсинг OHLCV свечей

```rust
use serde::Deserialize;

// Кортежная структура для свечей (Binance формат)
// [открытие_время, открытие, макс, мин, закрытие, объём, ...]
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

    println!("OHLCV свечи:");
    println!("{:^13} | {:^10} | {:^10} | {:^10} | {:^10} | {:^10}",
        "Time", "Open", "High", "Low", "Close", "Volume");
    println!("{:-^13}-+-{:-^10}-+-{:-^10}-+-{:-^10}-+-{:-^10}-+-{:-^10}",
        "", "", "", "", "", "");

    for candle in &candles {
        println!("{:>13} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2}",
            candle.open_time / 1000, // Переводим в секунды
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume);
    }

    // Рассчитываем изменение цены
    if let (Some(first), Some(last)) = (candles.first(), candles.last()) {
        let change = last.close - first.open;
        let change_pct = (change / first.open) * 100.0;
        println!("\nИзменение: ${:.2} ({:.2}%)", change, change_pct);
    }
}
```

## Обработка ошибок при парсинге

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
    // Тест 1: Валидный JSON
    let valid = r#"[{"symbol": "BTCUSDT", "price": 42000.0}]"#;
    match parse_prices(valid) {
        Ok(prices) => {
            for p in prices {
                println!("{}: ${}", p.symbol, p.price);
            }
        }
        Err(e) => println!("Ошибка: {}", e),
    }

    // Тест 2: Невалидный JSON синтаксис
    let invalid_syntax = r#"{"symbol": "BTCUSDT", price: 42000}"#;
    match parse_prices(invalid_syntax) {
        Ok(_) => println!("Успех"),
        Err(e) => println!("Ошибка синтаксиса: {}", e),
    }

    // Тест 3: Неправильный тип данных
    let wrong_type = r#"[{"symbol": "BTCUSDT", "price": "not_a_number"}]"#;
    match parse_prices(wrong_type) {
        Ok(_) => println!("Успех"),
        Err(e) => println!("Ошибка типа: {}", e),
    }

    // Тест 4: Отсутствует обязательное поле
    let missing_field = r#"[{"symbol": "BTCUSDT"}]"#;
    match parse_prices(missing_field) {
        Ok(_) => println!("Успех"),
        Err(e) => println!("Отсутствует поле: {}", e),
    }
}
```

## Что мы узнали

| Операция | Метод | Описание |
|----------|-------|----------|
| JSON → Структура | `serde_json::from_str()` | Парсинг строки JSON |
| JSON → Value | `serde_json::from_str()` | Динамический парсинг |
| Структура → JSON | `serde_json::to_string()` | Компактная сериализация |
| Структура → JSON | `serde_json::to_string_pretty()` | Форматированный вывод |
| Атрибут | `#[derive(Deserialize)]` | Автогенерация парсера |
| Атрибут | `#[derive(Serialize)]` | Автогенерация сериализатора |
| Переименование | `#[serde(rename = "...")]` | Другое имя в JSON |

## Домашнее задание

1. Напиши парсер для WebSocket сообщения биржи с ценой тикера. Структура:
   ```json
   {"e": "24hrTicker", "s": "BTCUSDT", "c": "42150.00", "P": "-2.5"}
   ```

2. Создай структуру для ответа API с балансами и напиши функцию, которая находит все активы с ненулевым балансом

3. Реализуй парсер для ответа с историей сделок пользователя:
   ```json
   [{"id": 1, "symbol": "BTCUSDT", "price": "42000", "qty": "0.1", "side": "BUY", "time": 1703001200000}]
   ```
   Рассчитай общий объём покупок и продаж

4. Напиши функцию, которая принимает `serde_json::Value` и безопасно извлекает цену, возвращая `Option<f64>`

## Навигация

[← Предыдущий день](../128-json-exchange-api/ru.md) | [Следующий день →](../130-nested-json/ru.md)
