# День 47: 'static — данные живут вечно, как история рынка

## Аналогия из трейдинга

Представь себе **историю цен Bitcoin с момента его создания** — эти данные существуют "вечно" в контексте твоей программы. Они были до запуска, будут доступны всё время работы, и не зависят от конкретных функций или областей видимости. Это и есть `'static` — время жизни данных, которые существуют на протяжении всей работы программы.

Другой пример: **название биржи "Binance"** или **тикер "BTC/USDT"** — это константы, которые не меняются и доступны всегда. Они встроены прямо в код программы.

## Что такое 'static?

`'static` — это специальное время жизни (lifetime) в Rust, означающее, что данные:
- Живут на протяжении всей работы программы
- Хранятся в бинарном файле программы (для строковых литералов)
- Или созданы и никогда не освобождаются (для `Box::leak` и подобных)

```rust
fn main() {
    // Строковый литерал имеет время жизни 'static
    let exchange: &'static str = "Binance";
    let ticker: &'static str = "BTC/USDT";

    println!("Trading {} on {}", ticker, exchange);
}
```

## Строковые литералы — самый частый случай

```rust
fn main() {
    // Все строковые литералы имеют тип &'static str
    let buy_signal: &'static str = "BUY";
    let sell_signal: &'static str = "SELL";
    let hold_signal: &'static str = "HOLD";

    let signal = get_trading_signal(42000.0, 41000.0);
    println!("Signal: {}", signal);
}

fn get_trading_signal(current_price: f64, sma: f64) -> &'static str {
    if current_price > sma * 1.02 {
        "BUY"  // Это &'static str
    } else if current_price < sma * 0.98 {
        "SELL"
    } else {
        "HOLD"
    }
}
```

## Статические константы для конфигурации

```rust
// Статические константы — классический пример 'static
static EXCHANGE_NAME: &str = "Binance";
static DEFAULT_FEE_PERCENT: f64 = 0.1;
static MAX_POSITION_SIZE: f64 = 100000.0;
static SUPPORTED_PAIRS: [&str; 4] = ["BTC/USDT", "ETH/USDT", "SOL/USDT", "XRP/USDT"];

fn main() {
    println!("Trading on: {}", EXCHANGE_NAME);
    println!("Default fee: {}%", DEFAULT_FEE_PERCENT);
    println!("Max position: ${}", MAX_POSITION_SIZE);

    println!("Supported pairs:");
    for pair in SUPPORTED_PAIRS.iter() {
        println!("  - {}", pair);
    }
}
```

## const vs static

```rust
// const — значение подставляется в каждое место использования
const RISK_PERCENT: f64 = 2.0;
const TRADING_HOURS_START: u32 = 9;
const TRADING_HOURS_END: u32 = 17;

// static — одна ячейка памяти на всю программу
static BROKER_NAME: &str = "Interactive Brokers";
static mut TOTAL_TRADES: u32 = 0;  // Изменяемая статика (опасно!)

fn main() {
    // const просто подставляется
    let risk = RISK_PERCENT;

    // static имеет фиксированный адрес
    println!("Broker: {}", BROKER_NAME);

    // Изменяемая статика требует unsafe
    unsafe {
        TOTAL_TRADES += 1;
        println!("Total trades: {}", TOTAL_TRADES);
    }
}
```

## 'static в типах — требование к данным

```rust
use std::thread;

fn main() {
    let ticker = String::from("BTC/USDT");

    // Ошибка: ticker не 'static, он принадлежит main
    // thread::spawn(|| {
    //     println!("Trading {}", ticker);
    // });

    // Решение 1: move — передать владение в поток
    let ticker_for_thread = ticker.clone();
    thread::spawn(move || {
        println!("Trading {}", ticker_for_thread);
    });

    // Решение 2: использовать 'static данные
    thread::spawn(|| {
        let static_ticker: &'static str = "ETH/USDT";
        println!("Also trading {}", static_ticker);
    });

    // Даём потокам время завершиться
    thread::sleep(std::time::Duration::from_millis(100));
}
```

## Практический пример: конфигурация торговой системы

```rust
// Константы конфигурации с 'static временем жизни
static CONFIG: TradingConfig = TradingConfig {
    exchange: "Binance",
    base_currency: "USDT",
    risk_per_trade: 2.0,
    max_daily_trades: 10,
    allowed_pairs: &["BTC/USDT", "ETH/USDT", "SOL/USDT"],
};

struct TradingConfig {
    exchange: &'static str,
    base_currency: &'static str,
    risk_per_trade: f64,
    max_daily_trades: u32,
    allowed_pairs: &'static [&'static str],
}

impl TradingConfig {
    fn is_pair_allowed(&self, pair: &str) -> bool {
        self.allowed_pairs.contains(&pair)
    }

    fn calculate_position_size(&self, balance: f64) -> f64 {
        balance * (self.risk_per_trade / 100.0)
    }
}

fn main() {
    println!("Exchange: {}", CONFIG.exchange);
    println!("Risk per trade: {}%", CONFIG.risk_per_trade);

    let pair = "BTC/USDT";
    if CONFIG.is_pair_allowed(pair) {
        let position = CONFIG.calculate_position_size(10000.0);
        println!("Position size for {}: ${:.2}", pair, position);
    }

    // Проверка недопустимой пары
    if !CONFIG.is_pair_allowed("DOGE/USDT") {
        println!("DOGE/USDT is not in allowed pairs");
    }
}
```

## Создание 'static данных динамически (Box::leak)

```rust
fn main() {
    // Создаём данные, которые будут жить вечно
    let market_data: &'static MarketSnapshot = create_static_snapshot();

    println!("Static snapshot:");
    println!("  Price: ${}", market_data.price);
    println!("  Volume: {}", market_data.volume);

    // Эти данные теперь доступны везде и всегда
    process_in_another_function(market_data);
}

struct MarketSnapshot {
    price: f64,
    volume: f64,
    timestamp: u64,
}

fn create_static_snapshot() -> &'static MarketSnapshot {
    let snapshot = Box::new(MarketSnapshot {
        price: 42000.0,
        volume: 1500.0,
        timestamp: 1234567890,
    });

    // Box::leak превращает данные в 'static
    // ВНИМАНИЕ: память никогда не освободится!
    Box::leak(snapshot)
}

fn process_in_another_function(data: &'static MarketSnapshot) {
    println!("Processing snapshot from timestamp: {}", data.timestamp);
}
```

## 'static bound vs 'static lifetime

```rust
use std::fmt::Display;

// T: 'static означает, что T не содержит НЕ-'static ссылок
// Это НЕ означает, что T сам должен быть &'static
fn log_trade_info<T: Display + 'static>(info: T) {
    println!("[TRADE LOG] {}", info);
}

fn main() {
    // String владеет данными, не содержит ссылок — OK
    let trade = String::from("BUY 0.5 BTC @ $42,000");
    log_trade_info(trade);

    // i64 — примитив, без ссылок — OK
    log_trade_info(42000_i64);

    // &'static str — статическая ссылка — OK
    log_trade_info("Executed");

    // НЕ сработает с временной ссылкой:
    // let local = String::from("local");
    // log_trade_info(&local);  // Ошибка: &String не 'static
}
```

## Справочная таблица рыночных данных

```rust
// Статическая таблица типов ордеров
static ORDER_TYPES: &[OrderTypeInfo] = &[
    OrderTypeInfo { name: "MARKET", requires_price: false, description: "Execute immediately at market price" },
    OrderTypeInfo { name: "LIMIT", requires_price: true, description: "Execute at specified price or better" },
    OrderTypeInfo { name: "STOP", requires_price: true, description: "Trigger market order at stop price" },
    OrderTypeInfo { name: "STOP_LIMIT", requires_price: true, description: "Trigger limit order at stop price" },
];

struct OrderTypeInfo {
    name: &'static str,
    requires_price: bool,
    description: &'static str,
}

fn get_order_type_info(name: &str) -> Option<&'static OrderTypeInfo> {
    ORDER_TYPES.iter().find(|info| info.name == name)
}

fn main() {
    if let Some(info) = get_order_type_info("LIMIT") {
        println!("Order type: {}", info.name);
        println!("Requires price: {}", info.requires_price);
        println!("Description: {}", info.description);
    }

    println!("\nAll order types:");
    for order_type in ORDER_TYPES.iter() {
        println!("  {} - {}", order_type.name, order_type.description);
    }
}
```

## Lazy static — инициализация при первом использовании

```rust
use std::sync::OnceLock;
use std::collections::HashMap;

// OnceLock позволяет один раз инициализировать статические данные
static EXCHANGE_FEES: OnceLock<HashMap<&'static str, f64>> = OnceLock::new();

fn get_exchange_fees() -> &'static HashMap<&'static str, f64> {
    EXCHANGE_FEES.get_or_init(|| {
        let mut fees = HashMap::new();
        fees.insert("Binance", 0.1);
        fees.insert("Coinbase", 0.5);
        fees.insert("Kraken", 0.26);
        fees.insert("FTX", 0.07);
        fees
    })
}

fn main() {
    let fees = get_exchange_fees();

    println!("Exchange fees:");
    for (exchange, fee) in fees.iter() {
        println!("  {}: {}%", exchange, fee);
    }

    // Второй вызов вернёт те же данные (уже инициализированы)
    let fee = get_exchange_fees().get("Binance").unwrap();
    println!("\nBinance fee: {}%", fee);
}
```

## Когда использовать 'static

| Сценарий | Пример | Решение |
|----------|--------|---------|
| Константные строки | Названия бирж, тикеры | `&'static str` литералы |
| Глобальная конфигурация | Настройки системы | `static CONFIG: ...` |
| Lookup таблицы | Типы ордеров, комиссии | `static TABLE: &[...]` |
| Данные для потоков | Thread-safe доступ | `Arc<T>` или `'static` |
| Кэш на весь срок работы | Исторические данные | `Box::leak` или `lazy_static` |

## Предостережения

```rust
// ⚠️ Box::leak создаёт утечку памяти — используй осторожно!
fn bad_example() {
    for _ in 0..1000000 {
        let leaked: &'static str = Box::leak(String::from("data").into_boxed_str());
        // Каждая итерация утекает память!
    }
}

// ✅ Лучше использовать Arc для shared данных
use std::sync::Arc;

fn good_example() {
    let shared_data = Arc::new(String::from("shared market data"));

    let data_clone = Arc::clone(&shared_data);
    std::thread::spawn(move || {
        println!("Thread 1: {}", data_clone);
    });

    let data_clone2 = Arc::clone(&shared_data);
    std::thread::spawn(move || {
        println!("Thread 2: {}", data_clone2);
    });
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `'static` lifetime | Данные живут всю программу |
| `&'static str` | Строковые литералы в коде |
| `static` переменные | Глобальные данные с фиксированным адресом |
| `const` | Значение подставляется при компиляции |
| `T: 'static` bound | Тип не содержит не-static ссылок |
| `Box::leak` | Создаёт 'static данные динамически |
| `OnceLock` | Ленивая инициализация статических данных |

## Домашнее задание

1. **Конфигурация бирж**: Создай статическую таблицу с информацией о биржах (название, комиссии, минимальный лот) и функции для запроса этих данных.

2. **Система сигналов**: Напиши функцию `get_signal_description(signal: &str) -> Option<&'static SignalInfo>`, которая возвращает описание торгового сигнала из статической таблицы.

3. **Кэш исторических данных**: Используя `OnceLock`, создай кэш с историческими ценами, который инициализируется при первом обращении.

4. **Thread-safe логгер**: Создай простой логгер торговых операций, который можно безопасно использовать из нескольких потоков, используя `'static` данные.

## Навигация

[← Предыдущий день](../046-lifetime-annotations/ru.md) | [Следующий день →](../048-lifetime-elision/ru.md)
