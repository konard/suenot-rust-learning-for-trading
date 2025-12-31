# День 127: Сериализация: serde введение

## Аналогия из трейдинга

Представь, что ты хочешь отправить информацию о сделке своему брокеру или сохранить её в файл. Ты не можешь просто "телепортировать" объект из памяти — нужно **преобразовать его в текст** (или байты), который потом можно прочитать обратно.

Это как торговый отчёт:
- **Сериализация** — ты записываешь сделку на бумагу (Order → JSON/текст)
- **Десериализация** — ты читаешь отчёт и восстанавливаешь информацию (JSON/текст → Order)

**serde** — это библиотека Rust, которая делает это автоматически. Название — сокращение от **ser**ialize + **de**serialize.

## Что такое serde?

serde — это фреймворк для сериализации данных в Rust:
- **Быстрый** — нулевые накладные расходы во многих случаях
- **Гибкий** — поддерживает JSON, TOML, YAML, MessagePack и десятки других форматов
- **Безопасный** — проверки типов на этапе компиляции

## Добавление serde в проект

```toml
# Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"  # Для работы с JSON
```

## Базовый пример: структура Trade

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Trade {
    symbol: String,
    side: String,        // "BUY" или "SELL"
    price: f64,
    quantity: f64,
    timestamp: u64,
}

fn main() {
    // Создаём сделку
    let trade = Trade {
        symbol: String::from("BTC/USDT"),
        side: String::from("BUY"),
        price: 42000.50,
        quantity: 0.5,
        timestamp: 1703980800,
    };

    // Сериализация: Trade → JSON строка
    let json = serde_json::to_string(&trade).unwrap();
    println!("JSON: {}", json);

    // Десериализация: JSON строка → Trade
    let restored: Trade = serde_json::from_str(&json).unwrap();
    println!("Restored: {:?}", restored);
}
```

Вывод:
```
JSON: {"symbol":"BTC/USDT","side":"BUY","price":42000.5,"quantity":0.5,"timestamp":1703980800}
Restored: Trade { symbol: "BTC/USDT", side: "BUY", price: 42000.5, quantity: 0.5, timestamp: 1703980800 }
```

## Derive макросы: магия Rust

Ключевая особенность serde — макросы `Serialize` и `Deserialize`:

```rust
use serde::{Serialize, Deserialize};

// Добавляем #[derive(...)] и структура автоматически умеет
// преобразовываться в разные форматы и обратно
#[derive(Serialize, Deserialize)]
struct Order {
    id: u64,
    symbol: String,
    order_type: String,
    price: f64,
    quantity: f64,
}
```

Rust сам генерирует весь необходимый код!

## Красивый JSON

Для отладки удобно использовать форматированный вывод:

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

    // Красивый JSON с отступами
    let pretty_json = serde_json::to_string_pretty(&portfolio).unwrap();
    println!("{}", pretty_json);
}
```

Вывод:
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

## Сериализация enum: типы ордеров

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
    price: Option<f64>,  // None для Market ордеров
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

Вывод:
```
Limit: {"symbol":"ETH/USDT","order_type":"Limit","side":"Buy","price":2200.0,"quantity":2.0}
Market: {"symbol":"BTC/USDT","order_type":"Market","side":"Sell","price":null,"quantity":0.1}
```

## Работа с Vec и HashMap

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
    // Вектор свечей
    let candles = vec![
        Candle { open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0, volume: 1000.0 },
        Candle { open: 42300.0, high: 42800.0, low: 42100.0, close: 42600.0, volume: 1200.0 },
    ];

    let json = serde_json::to_string_pretty(&candles).unwrap();
    println!("Candles:\n{}", json);

    // HashMap с балансами
    let mut balances: HashMap<String, f64> = HashMap::new();
    balances.insert(String::from("BTC"), 0.5);
    balances.insert(String::from("ETH"), 5.0);
    balances.insert(String::from("USDT"), 10000.0);

    let balances_json = serde_json::to_string_pretty(&balances).unwrap();
    println!("\nBalances:\n{}", balances_json);
}
```

## Обработка ошибок при десериализации

В реальном трейдинге данные могут быть некорректными:

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
        .map_err(|e| format!("Ошибка парсинга: {}", e))
}

fn main() {
    // Корректный JSON
    let valid = r#"{"symbol":"BTC/USDT","price":42000.0,"timestamp":1703980800}"#;
    match parse_price_update(valid) {
        Ok(update) => println!("Получено обновление: {:?}", update),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Некорректный JSON (price как строка вместо числа)
    let invalid = r#"{"symbol":"BTC/USDT","price":"high","timestamp":1703980800}"#;
    match parse_price_update(invalid) {
        Ok(update) => println!("Получено обновление: {:?}", update),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Неполный JSON (отсутствует поле)
    let missing_field = r#"{"symbol":"BTC/USDT","price":42000.0}"#;
    match parse_price_update(missing_field) {
        Ok(update) => println!("Получено обновление: {:?}", update),
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Сохранение и загрузка из файла

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

    // Сохраняем
    if let Err(e) = save_config(&config, "trading_config.json") {
        println!("Ошибка сохранения: {}", e);
        return;
    }
    println!("Конфигурация сохранена!");

    // Загружаем
    match load_config("trading_config.json") {
        Ok(loaded) => println!("Загружено: {:?}", loaded),
        Err(e) => println!("Ошибка загрузки: {}", e),
    }
}
```

## Практический пример: история сделок

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

    // Статистика
    println!("\n═══════════════════════════════");
    println!("Итого PnL: ${:.2}", history.total_pnl);
    println!("Количество сделок: {}", history.trades.len());
    println!("═══════════════════════════════");
}
```

## Что мы узнали

| Концепт | Описание |
|---------|----------|
| `Serialize` | Трейт для преобразования в формат (JSON, TOML и т.д.) |
| `Deserialize` | Трейт для восстановления из формата |
| `#[derive(...)]` | Автоматическая генерация реализации |
| `serde_json` | Крейт для работы с JSON |
| `to_string()` | Сериализация в компактную строку |
| `to_string_pretty()` | Сериализация с форматированием |
| `from_str()` | Десериализация из строки |

## Практические задания

1. Создай структуру `OHLCV` (Open, High, Low, Close, Volume) и сериализуй массив свечей в JSON.

2. Напиши функцию, которая загружает конфигурацию торгового бота из JSON-файла и проверяет корректность данных.

3. Создай enum `OrderStatus` (Pending, Filled, Cancelled, Rejected) и сериализуй структуру ордера с этим статусом.

4. Напиши программу, которая читает JSON с обновлениями цен и выводит только те, где цена изменилась более чем на 1%.

## Домашнее задание

1. Создай систему логирования сделок:
   - Структура `TradeLog` с полями: id, timestamp, action, details
   - Функция сохранения лога в файл (добавление, не перезапись)
   - Функция загрузки и отображения всех логов

2. Реализуй сериализацию портфеля:
   - Структура `Portfolio` с позициями, балансом и метаданными
   - Сохранение состояния портфеля в файл каждые N секунд
   - Восстановление портфеля при перезапуске программы

3. Создай парсер для упрощённого биржевого API:
   - Разные типы сообщений (trade, orderbook, ticker)
   - Обработка ошибок парсинга
   - Статистика по полученным сообщениям

4. Напиши конвертер форматов:
   - Чтение данных из JSON
   - Сохранение в другом формате (например, в простой CSV-подобный текст)
   - Обратное преобразование

## Навигация

[← Предыдущий день](../126-path-pathbuf/ru.md) | [Следующий день →](../128-json-exchange-api/ru.md)
