# День 106: Логирование ошибок при торговле

## Аналогия из трейдинга

Представь торговый зал биржи. Каждая транзакция, каждая ошибка, каждое предупреждение записывается в журнал. Если что-то пойдёт не так — аудиторы могут восстановить всю цепочку событий. Логирование в программировании — это тот же журнал транзакций: он помогает понять, что произошло, когда и почему.

Без логов отладка торгового бота похожа на поиск иголки в стоге сена с завязанными глазами. С логами — ты видишь каждый шаг системы.

## Зачем логировать ошибки?

В трейдинге критически важно знать:
- **Когда** произошла ошибка (время критично для торговли)
- **Что** пошло не так (какой запрос, какие данные)
- **Где** в коде возникла проблема
- **Контекст**: какой ордер, какой инструмент, какая цена

## Крейт log: стандартный интерфейс логирования

```rust
// Cargo.toml:
// [dependencies]
// log = "0.4"
// env_logger = "0.10"

use log::{debug, info, warn, error, trace};

fn main() {
    // Инициализируем логгер
    env_logger::init();

    info!("Торговый бот запущен");

    let ticker = "BTC/USDT";
    let price = 42000.0;

    debug!("Получена цена {} для {}", price, ticker);

    if price > 50000.0 {
        warn!("Цена {} выше порога для {}", price, ticker);
    }

    match execute_order(ticker, price) {
        Ok(order_id) => info!("Ордер {} исполнен", order_id),
        Err(e) => error!("Ошибка исполнения ордера: {}", e),
    }
}

fn execute_order(ticker: &str, price: f64) -> Result<u64, String> {
    if price <= 0.0 {
        return Err("Некорректная цена".to_string());
    }
    Ok(12345)
}
```

**Запуск с разными уровнями:**
```bash
RUST_LOG=debug cargo run    # Покажет debug, info, warn, error
RUST_LOG=info cargo run     # Покажет info, warn, error
RUST_LOG=warn cargo run     # Покажет только warn и error
```

## Уровни логирования

| Уровень | Когда использовать | Пример из трейдинга |
|---------|-------------------|---------------------|
| `trace!` | Детали алгоритмов | Каждый тик цены |
| `debug!` | Отладочная информация | Расчёт индикаторов |
| `info!` | Важные события | Исполнение ордера |
| `warn!` | Потенциальные проблемы | API rate limit близок |
| `error!` | Ошибки операций | Ордер отклонён |

```rust
use log::{trace, debug, info, warn, error};

fn process_market_data(prices: &[f64]) {
    trace!("Входные данные: {:?}", prices);

    let avg = prices.iter().sum::<f64>() / prices.len() as f64;
    debug!("Рассчитана средняя цена: {:.2}", avg);

    if prices.len() < 10 {
        warn!("Мало данных для надёжного анализа: {} точек", prices.len());
    }

    info!("Обработано {} ценовых точек", prices.len());
}
```

## Логирование с контекстом ошибки

```rust
use log::{error, warn, info};

#[derive(Debug)]
struct TradeError {
    code: String,
    message: String,
    ticker: String,
    attempted_price: f64,
}

fn place_order(ticker: &str, side: &str, quantity: f64, price: f64) -> Result<u64, TradeError> {
    info!("Размещение ордера: {} {} {} по {}", side, quantity, ticker, price);

    // Симуляция проверок
    if quantity <= 0.0 {
        let err = TradeError {
            code: "INVALID_QTY".to_string(),
            message: "Количество должно быть положительным".to_string(),
            ticker: ticker.to_string(),
            attempted_price: price,
        };
        error!(
            "Ошибка ордера [{}]: {} | ticker={}, price={}, qty={}",
            err.code, err.message, ticker, price, quantity
        );
        return Err(err);
    }

    if price <= 0.0 {
        let err = TradeError {
            code: "INVALID_PRICE".to_string(),
            message: "Цена должна быть положительной".to_string(),
            ticker: ticker.to_string(),
            attempted_price: price,
        };
        error!(
            "Ошибка ордера [{}]: {} | ticker={}, price={}",
            err.code, err.message, ticker, price
        );
        return Err(err);
    }

    // Успешное размещение
    let order_id = 98765;
    info!("Ордер размещён: id={}, ticker={}, side={}, qty={}, price={}",
          order_id, ticker, side, quantity, price);
    Ok(order_id)
}

fn main() {
    env_logger::init();

    let _ = place_order("BTC/USDT", "BUY", 0.5, 42000.0);  // OK
    let _ = place_order("ETH/USDT", "BUY", -1.0, 3000.0);  // Ошибка
    let _ = place_order("SOL/USDT", "SELL", 10.0, 0.0);    // Ошибка
}
```

## Логирование цепочки ошибок с anyhow

```rust
use anyhow::{Context, Result};
use log::{error, info};

fn fetch_price(ticker: &str) -> Result<f64> {
    // Симуляция ошибки API
    if ticker == "INVALID" {
        anyhow::bail!("Тикер не найден");
    }
    Ok(42000.0)
}

fn calculate_position_value(ticker: &str, quantity: f64) -> Result<f64> {
    let price = fetch_price(ticker)
        .with_context(|| format!("Не удалось получить цену для {}", ticker))?;

    Ok(price * quantity)
}

fn process_portfolio(positions: &[(&str, f64)]) -> Result<f64> {
    let mut total = 0.0;

    for (ticker, qty) in positions {
        match calculate_position_value(ticker, *qty) {
            Ok(value) => {
                info!("Позиция {}: ${:.2}", ticker, value);
                total += value;
            }
            Err(e) => {
                // Логируем полную цепочку ошибок
                error!("Ошибка обработки позиции {}: {:?}", ticker, e);

                // Можно продолжить с остальными позициями
                // или вернуть ошибку — зависит от логики
            }
        }
    }

    Ok(total)
}

fn main() -> Result<()> {
    env_logger::init();

    let positions = vec![
        ("BTC/USDT", 0.5),
        ("ETH/USDT", 2.0),
        ("INVALID", 1.0),
    ];

    let total = process_portfolio(&positions)?;
    info!("Общая стоимость портфеля: ${:.2}", total);

    Ok(())
}
```

## Структурированное логирование

Для анализа логов в продакшене полезны структурированные форматы:

```rust
use log::info;
use serde::Serialize;

#[derive(Serialize)]
struct OrderLog {
    event: &'static str,
    order_id: u64,
    ticker: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
}

fn log_order_event(order: &OrderLog) {
    // Логируем как JSON для парсинга инструментами мониторинга
    info!(
        target: "orders",
        "{}",
        serde_json::to_string(order).unwrap_or_else(|_| format!("{:?}", order.order_id))
    );
}

fn main() {
    env_logger::init();

    let order_log = OrderLog {
        event: "ORDER_PLACED",
        order_id: 12345,
        ticker: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        quantity: 0.5,
        price: 42000.0,
        status: "PENDING".to_string(),
    };

    log_order_event(&order_log);
}
```

## Логирование в файл

```rust
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

fn setup_file_logger() -> Result<(), Box<dyn std::error::Error>> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}\n"
        )))
        .build("trading_bot.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}
```

## Практический пример: логирование торговой сессии

```rust
use log::{debug, info, warn, error};
use std::time::Instant;

struct TradingSession {
    start_time: Instant,
    trades_count: u32,
    errors_count: u32,
}

impl TradingSession {
    fn new() -> Self {
        info!("═══════════════════════════════════════");
        info!("Торговая сессия начата");
        info!("═══════════════════════════════════════");

        TradingSession {
            start_time: Instant::now(),
            trades_count: 0,
            errors_count: 0,
        }
    }

    fn execute_trade(&mut self, ticker: &str, side: &str, qty: f64, price: f64) -> Result<(), String> {
        debug!("Попытка исполнения: {} {} {} @ {}", side, qty, ticker, price);

        // Симуляция проверок
        if qty <= 0.0 {
            self.errors_count += 1;
            error!("Отклонено: некорректное количество {} для {}", qty, ticker);
            return Err("Invalid quantity".to_string());
        }

        if price <= 0.0 {
            self.errors_count += 1;
            error!("Отклонено: некорректная цена {} для {}", price, ticker);
            return Err("Invalid price".to_string());
        }

        // Симуляция случайного сбоя (10% шанс)
        if (self.trades_count % 10) == 7 {
            self.errors_count += 1;
            warn!("Временный сбой API при исполнении ордера {}", ticker);
            return Err("API timeout".to_string());
        }

        self.trades_count += 1;
        info!("✓ Исполнено: {} {} {} @ {} | Сделка #{}",
              side, qty, ticker, price, self.trades_count);

        Ok(())
    }

    fn end_session(&self) {
        let duration = self.start_time.elapsed();

        info!("═══════════════════════════════════════");
        info!("Торговая сессия завершена");
        info!("Длительность: {:.2} сек", duration.as_secs_f64());
        info!("Сделок: {}", self.trades_count);
        info!("Ошибок: {}", self.errors_count);

        if self.errors_count > 0 {
            warn!("Сессия завершена с {} ошибками", self.errors_count);
        }

        let success_rate = if self.trades_count + self.errors_count > 0 {
            (self.trades_count as f64) / ((self.trades_count + self.errors_count) as f64) * 100.0
        } else {
            0.0
        };

        info!("Успешность: {:.1}%", success_rate);
        info!("═══════════════════════════════════════");
    }
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("debug")
    ).init();

    let mut session = TradingSession::new();

    // Серия торговых операций
    let orders = vec![
        ("BTC/USDT", "BUY", 0.1, 42000.0),
        ("ETH/USDT", "BUY", 1.0, 3000.0),
        ("SOL/USDT", "SELL", -5.0, 100.0),  // Ошибка: отрицательное количество
        ("DOGE/USDT", "BUY", 1000.0, 0.0),  // Ошибка: нулевая цена
        ("ADA/USDT", "BUY", 500.0, 0.5),
        ("DOT/USDT", "SELL", 20.0, 7.0),
        ("LINK/USDT", "BUY", 10.0, 15.0),
        ("BTC/USDT", "SELL", 0.05, 42500.0),  // Может быть API timeout
    ];

    for (ticker, side, qty, price) in orders {
        let _ = session.execute_trade(ticker, side, qty, price);
    }

    session.end_session();
}
```

## Паттерны логирования для трейдинга

### 1. Логирование с идентификатором корреляции

```rust
use uuid::Uuid;
use log::info;

fn process_order_with_correlation(ticker: &str, qty: f64) {
    let correlation_id = Uuid::new_v4();

    info!("[{}] Начало обработки ордера {} qty={}", correlation_id, ticker, qty);
    // ... операции ...
    info!("[{}] Ордер обработан", correlation_id);
}
```

### 2. Логирование времени выполнения

```rust
use std::time::Instant;
use log::{debug, warn};

fn timed_operation<F, T>(name: &str, f: F) -> T
where
    F: FnOnce() -> T
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();

    if duration.as_millis() > 100 {
        warn!("{} заняло {:.2}ms (медленно!)", name, duration.as_secs_f64() * 1000.0);
    } else {
        debug!("{} заняло {:.2}ms", name, duration.as_secs_f64() * 1000.0);
    }

    result
}
```

### 3. Счётчик ошибок с логированием

```rust
use std::collections::HashMap;
use log::{error, warn};

struct ErrorTracker {
    counts: HashMap<String, u32>,
    threshold: u32,
}

impl ErrorTracker {
    fn new(threshold: u32) -> Self {
        ErrorTracker {
            counts: HashMap::new(),
            threshold,
        }
    }

    fn record_error(&mut self, error_type: &str, message: &str) {
        let count = self.counts.entry(error_type.to_string()).or_insert(0);
        *count += 1;

        if *count == self.threshold {
            warn!(
                "Достигнут порог ошибок для '{}': {} за сессию",
                error_type, self.threshold
            );
        }

        error!("[{}] (#{}) {}", error_type, count, message);
    }
}
```

## Что мы узнали

| Концепт | Описание |
|---------|----------|
| `log` crate | Стандартный интерфейс логирования Rust |
| Уровни логов | trace, debug, info, warn, error |
| `env_logger` | Простой логгер с настройкой через переменные среды |
| Контекст | Добавляй тикер, цену, order_id в каждое сообщение |
| Структурированные логи | JSON формат для анализа инструментами |
| Время выполнения | Логируй медленные операции |

## Домашнее задание

1. Создай логгер для торгового бота, который:
   - Записывает все ордера на уровне `info`
   - Логирует ошибки API на уровне `error`
   - Выводит детали расчёта индикаторов на уровне `debug`

2. Реализуй функцию `log_trade_chain()`, которая логирует всю цепочку: от получения сигнала до исполнения ордера, с временными метками каждого этапа

3. Напиши `ErrorAggregator`, который собирает ошибки за сессию и в конце выводит статистику: какие ошибки, сколько раз, какой процент от общего числа операций

4. Реализуй структурированный логгер, который выводит события в JSON формате для последующего анализа в системах мониторинга

## Навигация

[← Предыдущий день](../105-error-context/ru.md) | [Следующий день →](../107-recovering-from-errors/ru.md)
