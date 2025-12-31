# День 131: Опциональные поля в JSON

## Аналогия из трейдинга

Представь, что ты получаешь данные о сделках от разных бирж. Некоторые биржи отправляют **полную информацию** о сделке: цену, объём, время, комиссию, тип ордера. Другие биржи могут **не отправлять** некоторые поля — например, комиссия может отсутствовать в ответе API.

В реальном мире торговых API это **нормальная ситуация**:
- Поле `stop_loss` есть только у ордеров со стоп-лоссом
- Поле `take_profit` может отсутствовать
- Поле `leverage` есть только на фьючерсных рынках
- Поле `filled_at` появляется только после исполнения ордера

В Rust для таких случаев мы используем `Option<T>` вместе с serde.

## Основы Option в JSON

### Базовый пример: ордер с опциональным стоп-лоссом

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    stop_loss: Option<f64>,      // Может отсутствовать
    take_profit: Option<f64>,    // Может отсутствовать
}

fn main() {
    // JSON с опциональными полями
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

    // JSON без опциональных полей
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

    // Работа с опциональными полями
    match order1.stop_loss {
        Some(sl) => println!("Stop Loss установлен: ${:.2}", sl),
        None => println!("Stop Loss не установлен"),
    }
}
```

## #[serde(skip_serializing_if)] — пропуск None при сериализации

По умолчанию serde сериализует `Option<T>` как `null`. Чтобы **полностью пропустить** поле с `None`:

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
        take_profit: None,          // Будет пропущен
        trailing_stop: None,        // Будет пропущен
    };

    let json = serde_json::to_string_pretty(&signal).unwrap();
    println!("{}", json);
    // Вывод:
    // {
    //   "symbol": "BTC/USDT",
    //   "action": "BUY",
    //   "entry_price": 42000.0,
    //   "stop_loss": 41000.0
    // }
}
```

## #[serde(default)] — значения по умолчанию

Если поле отсутствует в JSON, можно использовать значение по умолчанию:

```rust
use serde::{Deserialize, Serialize};

fn default_leverage() -> f64 {
    1.0  // Без плеча
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

    #[serde(default)]  // Использует Default trait (0.0 для f64)
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

## Комбинирование default и skip_serializing_if

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
    // Минимальный JSON
    let json = r#"{"symbol": "ETH/USDT"}"#;
    let config: TradingConfig = serde_json::from_str(json).unwrap();

    println!("Risk: {}%", config.risk_percent);           // 2.0
    println!("Max Drawdown: {}%", config.max_drawdown);   // 0.0
    println!("Multiplier: {}x", config.position_multiplier); // 1.0

    // Сериализация обратно - дефолтные значения не включаются
    let json_out = serde_json::to_string(&config).unwrap();
    println!("JSON: {}", json_out);  // {"symbol":"ETH/USDT"}
}
```

## Вложенные опциональные структуры

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
    // Стратегия с полной конфигурацией риска
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

    // Простая стратегия без риск-менеджмента
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

## Обработка опциональных полей в коде

### Паттерн: безопасное извлечение данных

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

    // unwrap_or — значение по умолчанию
    println!("Volume: ${:.0}", data.volume_24h.unwrap_or(0.0));

    // if let — условная обработка
    if let Some(change) = data.change_percent {
        let direction = if change >= 0.0 { "📈" } else { "📉" };
        println!("Change: {}{:.2}%", direction, change);
    }

    // map — трансформация опционального значения
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

## Практический пример: Ордербук с опциональными полями

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct OrderBookLevel {
    price: f64,
    quantity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    orders_count: Option<u32>,  // Количество ордеров на уровне
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

## Что мы узнали

| Атрибут | Описание | Пример использования |
|---------|----------|---------------------|
| `Option<T>` | Поле может отсутствовать | `stop_loss: Option<f64>` |
| `#[serde(skip_serializing_if)]` | Пропуск при сериализации | Не включать None в JSON |
| `#[serde(default)]` | Значение по умолчанию | Leverage = 1.0 если не указан |
| `#[serde(default = "fn")]` | Кастомное значение | Risk = 2% по умолчанию |

## Практические упражнения

1. **Парсинг API ответа биржи**: Создай структуру `Ticker` с обязательными полями (symbol, price) и опциональными (volume, change_24h, high, low). Напиши функцию, которая безопасно извлекает все данные.

2. **Конфигурация торгового бота**: Создай структуру `BotConfig` где большинство полей имеют значения по умолчанию. Пользователь должен указать только symbol.

3. **Фильтрация сделок**: Напиши функцию, которая принимает JSON массив сделок с опциональным полем `fee` и возвращает только те сделки, где комиссия была указана.

4. **Генерация торгового отчёта**: Создай структуру `TradeReport` с опциональными секциями (summary, details, risk_metrics). Сериализуй только заполненные секции.

## Домашнее задание

1. Создай структуру `ExchangeResponse<T>` с опциональными полями `data: Option<T>`, `error: Option<String>`, `warning: Option<String>`. Реализуй метод `is_success()`.

2. Напиши парсер для разных форматов ордеров:
   - Spot ордер (без leverage, без liquidation_price)
   - Margin ордер (с leverage, без liquidation_price)
   - Futures ордер (с leverage, с liquidation_price)

3. Создай систему алертов `PriceAlert` где можно указать:
   - Обязательно: symbol, target_price
   - Опционально: expiry_time, repeat_count, notification_type

4. Реализуй функцию `merge_configs(base: Config, override: Config) -> Config` которая объединяет две конфигурации, где опциональные поля из override перезаписывают base.

## Навигация

[← Предыдущий день](../130-nested-json-structures/ru.md) | [Следующий день →](../132-serde-rename/ru.md)
