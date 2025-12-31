# День 145: Уровни логирования для трейдинга

## Аналогия из трейдинга

Представь себе торговый терминал с разными уровнями уведомлений:

- **ERROR (ошибка)** — сработал стоп-лосс, биржа недоступна, ордер отклонён. Ты должен немедленно узнать!
- **WARN (предупреждение)** — маржа близка к лимиту, волатильность выше нормы. Нужно обратить внимание.
- **INFO (информация)** — ордер исполнен, позиция открыта/закрыта. Нормальный ход событий.
- **DEBUG (отладка)** — детали расчёта размера позиции, промежуточные значения индикаторов. Для разработчика.
- **TRACE (трассировка)** — каждый тик цены, каждый вызов функции. Максимальная детализация.

Логирование — это система записи событий в твоей торговой программе.

## Подключение крейта log

В Rust для логирования используется крейт `log` — это фасад (абстракция), а конкретную реализацию предоставляют крейты вроде `env_logger`, `tracing-subscriber` и другие.

```toml
# Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

## Базовое использование

```rust
use log::{error, warn, info, debug, trace};

fn main() {
    env_logger::init();

    let btc_price = 42000.0;
    let order_size = 0.5;

    trace!("Функция main() вызвана");
    debug!("Текущая цена BTC: {}", btc_price);
    info!("Открываем позицию: {} BTC по ${}", order_size, btc_price);
    warn!("Волатильность выше среднего: 15%");
    error!("Соединение с биржей потеряно!");
}
```

Запуск с разными уровнями:
```bash
# Только ошибки
RUST_LOG=error cargo run

# Ошибки и предупреждения
RUST_LOG=warn cargo run

# Всё включая debug
RUST_LOG=debug cargo run

# Максимальная детализация
RUST_LOG=trace cargo run
```

## Уровни логирования

```rust
use log::{error, warn, info, debug, trace, Level};

fn demonstrate_levels() {
    // ERROR — критические ошибки, требующие немедленного внимания
    error!("Критическая ошибка: баланс отрицательный!");

    // WARN — потенциальные проблемы
    warn!("Предупреждение: размер позиции превышает рекомендуемый");

    // INFO — важные события нормальной работы
    info!("Ордер #12345 исполнен успешно");

    // DEBUG — информация для отладки
    debug!("Рассчитанный RSI: 67.5");

    // TRACE — максимально детальная информация
    trace!("Вход в функцию calculate_position_size()");
}

fn main() {
    env_logger::init();
    demonstrate_levels();
}
```

## Логирование в торговых функциях

```rust
use log::{error, warn, info, debug, trace};

fn main() {
    env_logger::init();

    let result = execute_trade("BTCUSDT", 42000.0, 0.5, "BUY");
    match result {
        Ok(order_id) => info!("Сделка выполнена: {}", order_id),
        Err(e) => error!("Сделка не выполнена: {}", e),
    }
}

fn execute_trade(
    symbol: &str,
    price: f64,
    quantity: f64,
    side: &str,
) -> Result<String, String> {
    trace!("execute_trade() вызвана: {} {} {} @ {}", side, quantity, symbol, price);

    // Валидация
    if quantity <= 0.0 {
        error!("Некорректное количество: {}", quantity);
        return Err("Invalid quantity".to_string());
    }

    if price <= 0.0 {
        error!("Некорректная цена: {}", price);
        return Err("Invalid price".to_string());
    }

    let value = price * quantity;
    debug!("Общая стоимость ордера: ${:.2}", value);

    // Проверка рисков
    if value > 100_000.0 {
        warn!("Крупный ордер: ${:.2} — требуется дополнительная проверка", value);
    }

    // Имитация отправки ордера
    let order_id = format!("ORD-{}", chrono_like_id());
    info!("Ордер отправлен: {} {} {} @ {} = ${:.2}",
          side, quantity, symbol, price, value);

    Ok(order_id)
}

fn chrono_like_id() -> u64 {
    1234567890 // В реальности — временная метка
}
```

## Структурированное логирование

```rust
use log::{info, warn, error};

#[derive(Debug)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct Position {
    symbol: String,
    entry_price: f64,
    current_price: f64,
    quantity: f64,
    pnl: f64,
}

fn main() {
    env_logger::init();

    let order = Order {
        id: "ORD-001".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        price: 42000.0,
        quantity: 0.5,
    };

    // Логирование структур через Debug
    info!("Создан ордер: {:?}", order);

    let position = Position {
        symbol: "BTCUSDT".to_string(),
        entry_price: 42000.0,
        current_price: 43500.0,
        quantity: 0.5,
        pnl: 750.0,
    };

    if position.pnl < 0.0 {
        warn!("Позиция в убытке: {:?}", position);
    } else {
        info!("Позиция в прибыли: {:?}", position);
    }
}
```

## Фильтрация по модулям

```rust
// main.rs
mod trading_engine;
mod risk_manager;

use log::info;

fn main() {
    // Настройка: только warn для trading_engine, debug для risk_manager
    // RUST_LOG=warn,risk_manager=debug cargo run
    env_logger::init();

    info!("Запуск торговой системы");
    trading_engine::process_order();
    risk_manager::check_risk();
}

// trading_engine.rs
use log::{info, debug, trace};

pub fn process_order() {
    trace!("Вход в process_order");
    debug!("Обработка ордера...");
    info!("Ордер обработан");
}

// risk_manager.rs
use log::{info, debug, warn};

pub fn check_risk() {
    debug!("Проверка риск-лимитов");
    warn!("Риск близок к лимиту");
    info!("Проверка завершена");
}
```

## Настройка формата логов

```rust
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use chrono::Local;

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {} - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    log::info!("Торговая система запущена");
    log::warn!("Высокая волатильность на рынке");
    log::error!("Ошибка подключения к бирже");
}
```

Вывод:
```
2024-01-15 10:30:45.123 [INFO] trading_bot - Торговая система запущена
2024-01-15 10:30:45.124 [WARN] trading_bot - Высокая волатильность на рынке
2024-01-15 10:30:45.124 [ERROR] trading_bot - Ошибка подключения к бирже
```

## Логирование для мониторинга позиций

```rust
use log::{error, warn, info, debug};

struct PositionMonitor {
    symbol: String,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
    quantity: f64,
}

impl PositionMonitor {
    fn new(symbol: &str, entry: f64, stop: f64, take: f64, qty: f64) -> Self {
        info!("Создан монитор позиции: {} entry={} SL={} TP={}",
              symbol, entry, stop, take);

        Self {
            symbol: symbol.to_string(),
            entry_price: entry,
            stop_loss: stop,
            take_profit: take,
            quantity: qty,
        }
    }

    fn check_price(&self, current_price: f64) {
        let pnl = (current_price - self.entry_price) * self.quantity;
        let pnl_percent = ((current_price / self.entry_price) - 1.0) * 100.0;

        debug!("{}: цена={:.2} PnL={:.2} ({:.2}%)",
               self.symbol, current_price, pnl, pnl_percent);

        if current_price <= self.stop_loss {
            error!("СТОП-ЛОСС СРАБОТАЛ! {} цена={} <= SL={}",
                   self.symbol, current_price, self.stop_loss);
        } else if current_price >= self.take_profit {
            info!("ТЕЙК-ПРОФИТ ДОСТИГНУТ! {} цена={} >= TP={}",
                  self.symbol, current_price, self.take_profit);
        } else if pnl_percent < -5.0 {
            warn!("{}: убыток {:.2}% — приближаемся к стоп-лоссу",
                  self.symbol, pnl_percent);
        }
    }
}

fn main() {
    env_logger::init();

    let monitor = PositionMonitor::new("BTCUSDT", 42000.0, 40000.0, 45000.0, 0.5);

    // Симуляция изменения цены
    let prices = [42500.0, 41500.0, 40500.0, 39500.0, 45500.0];

    for price in prices {
        monitor.check_price(price);
    }
}
```

## Логирование производительности

```rust
use log::{info, debug, trace};
use std::time::Instant;

fn main() {
    env_logger::init();

    let prices: Vec<f64> = (0..10000)
        .map(|i| 42000.0 + (i as f64 * 0.1))
        .collect();

    let sma = timed_sma(&prices, 20);
    info!("SMA-20 рассчитана: {:.2}", sma.unwrap_or(0.0));
}

fn timed_sma(prices: &[f64], period: usize) -> Option<f64> {
    let start = Instant::now();
    trace!("Начало расчёта SMA-{}", period);

    if prices.len() < period {
        debug!("Недостаточно данных для SMA-{}: {} < {}",
               period, prices.len(), period);
        return None;
    }

    let sum: f64 = prices.iter().rev().take(period).sum();
    let result = sum / period as f64;

    let elapsed = start.elapsed();
    debug!("SMA-{} рассчитана за {:?}: {:.4}", period, elapsed, result);

    if elapsed.as_millis() > 100 {
        log::warn!("Медленный расчёт SMA: {:?}", elapsed);
    }

    Some(result)
}
```

## Условное логирование

```rust
use log::{info, debug, warn, log_enabled, Level};

fn calculate_signals(prices: &[f64]) -> Vec<String> {
    let mut signals = Vec::new();

    // Проверяем, включён ли debug-уровень
    if log_enabled!(Level::Debug) {
        debug!("Анализируем {} ценовых точек", prices.len());
        for (i, price) in prices.iter().enumerate() {
            debug!("  [{}] {:.2}", i, price);
        }
    }

    // Дорогостоящие вычисления только если включён trace
    if log_enabled!(Level::Trace) {
        let avg: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance: f64 = prices.iter()
            .map(|p| (p - avg).powi(2))
            .sum::<f64>() / prices.len() as f64;
        log::trace!("Статистика: avg={:.2}, var={:.2}", avg, variance);
    }

    // Основная логика
    if prices.len() >= 2 {
        let last = prices[prices.len() - 1];
        let prev = prices[prices.len() - 2];

        if last > prev * 1.01 {
            let signal = "BUY".to_string();
            info!("Сигнал: {} (рост {:.2}%)", signal, (last/prev - 1.0) * 100.0);
            signals.push(signal);
        } else if last < prev * 0.99 {
            let signal = "SELL".to_string();
            warn!("Сигнал: {} (падение {:.2}%)", signal, (1.0 - last/prev) * 100.0);
            signals.push(signal);
        }
    }

    signals
}

fn main() {
    env_logger::init();

    let prices = vec![42000.0, 42100.0, 42500.0, 42300.0, 43000.0];
    let signals = calculate_signals(&prices);

    info!("Всего сигналов: {}", signals.len());
}
```

## Практический пример: торговый бот с логированием

```rust
use log::{error, warn, info, debug, trace, LevelFilter};
use env_logger::Builder;
use std::io::Write;

fn main() {
    // Настройка логгера с кастомным форматом
    Builder::new()
        .format(|buf, record| {
            let level_style = match record.level() {
                log::Level::Error => "\x1b[31m", // красный
                log::Level::Warn => "\x1b[33m",  // жёлтый
                log::Level::Info => "\x1b[32m",  // зелёный
                log::Level::Debug => "\x1b[36m", // голубой
                log::Level::Trace => "\x1b[90m", // серый
            };
            writeln!(
                buf,
                "{}[{}]\x1b[0m {} - {}",
                level_style,
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)
        .init();

    info!("=== Торговый бот запущен ===");

    let mut bot = TradingBot::new(10000.0);

    // Симуляция торговой сессии
    let market_data = vec![
        ("BTCUSDT", 42000.0),
        ("BTCUSDT", 42500.0),
        ("BTCUSDT", 41800.0),
        ("BTCUSDT", 40000.0), // резкое падение
        ("BTCUSDT", 43000.0),
    ];

    for (symbol, price) in market_data {
        bot.on_price_update(symbol, price);
    }

    info!("=== Торговый бот остановлен ===");
    info!("Итоговый баланс: ${:.2}", bot.balance);
}

struct TradingBot {
    balance: f64,
    position: Option<Position>,
    trade_count: u32,
}

struct Position {
    symbol: String,
    entry_price: f64,
    quantity: f64,
}

impl TradingBot {
    fn new(initial_balance: f64) -> Self {
        debug!("Инициализация бота с балансом ${}", initial_balance);
        Self {
            balance: initial_balance,
            position: None,
            trade_count: 0,
        }
    }

    fn on_price_update(&mut self, symbol: &str, price: f64) {
        trace!("Получено обновление: {} = {}", symbol, price);

        match &self.position {
            None => self.check_entry(symbol, price),
            Some(pos) => self.check_exit(price),
        }
    }

    fn check_entry(&mut self, symbol: &str, price: f64) {
        debug!("Проверка условий входа для {} @ {}", symbol, price);

        // Простая стратегия: покупаем если цена "низкая"
        if price < 42000.0 {
            let quantity = (self.balance * 0.1) / price; // 10% от баланса

            info!("ОТКРЫТИЕ ПОЗИЦИИ: {} {} @ ${:.2}",
                  symbol, quantity, price);

            self.position = Some(Position {
                symbol: symbol.to_string(),
                entry_price: price,
                quantity,
            });

            self.balance -= quantity * price;
            self.trade_count += 1;
        }
    }

    fn check_exit(&mut self, price: f64) {
        if let Some(ref pos) = self.position {
            let pnl = (price - pos.entry_price) * pos.quantity;
            let pnl_percent = ((price / pos.entry_price) - 1.0) * 100.0;

            debug!("Позиция {}: PnL = ${:.2} ({:.2}%)",
                   pos.symbol, pnl, pnl_percent);

            // Стоп-лосс: -5%
            if pnl_percent <= -5.0 {
                error!("СТОП-ЛОСС! Закрытие {} с убытком {:.2}%",
                       pos.symbol, pnl_percent);
                self.close_position(price);
            }
            // Тейк-профит: +3%
            else if pnl_percent >= 3.0 {
                info!("ТЕЙК-ПРОФИТ! Закрытие {} с прибылью {:.2}%",
                      pos.symbol, pnl_percent);
                self.close_position(price);
            }
            // Предупреждение при убытке > 3%
            else if pnl_percent < -3.0 {
                warn!("Позиция {} в убытке {:.2}% — близко к стоп-лоссу",
                      pos.symbol, pnl_percent);
            }
        }
    }

    fn close_position(&mut self, price: f64) {
        if let Some(pos) = self.position.take() {
            let proceeds = pos.quantity * price;
            self.balance += proceeds;

            let pnl = (price - pos.entry_price) * pos.quantity;
            info!("Позиция закрыта: {} @ ${:.2}, PnL: ${:.2}",
                  pos.symbol, price, pnl);
            debug!("Новый баланс: ${:.2}", self.balance);
        }
    }
}
```

## Что мы узнали

| Уровень | Назначение | Пример в трейдинге |
|---------|------------|-------------------|
| `error!` | Критические ошибки | Стоп-лосс, ошибка API |
| `warn!` | Предупреждения | Высокая волатильность, близость к лимитам |
| `info!` | Важные события | Исполнение ордера, открытие позиции |
| `debug!` | Отладка | Расчёты индикаторов, состояние |
| `trace!` | Трассировка | Каждый тик, вызовы функций |

## Домашнее задание

1. Создай торгового бота с полным логированием всех операций. Используй все 5 уровней логирования уместно.

2. Реализуй функцию `log_trade_summary(trades: &[Trade])`, которая выводит:
   - INFO: общее количество сделок
   - DEBUG: детали каждой сделки
   - WARN: убыточные сделки
   - ERROR: сделки с убытком > 10%

3. Напиши систему мониторинга портфеля с логированием:
   - TRACE: каждое обновление цены
   - DEBUG: пересчёт PnL
   - INFO: изменение состава портфеля
   - WARN: просадка > 5%
   - ERROR: просадка > 10%

4. Реализуй настраиваемый логгер, который записывает:
   - Все уровни в файл `trading.log`
   - Только WARN и ERROR в консоль
   - Добавь временные метки и название модуля

## Навигация

[← Предыдущий день](../144-file-serialization/ru.md) | [Следующий день →](../146-tracing-spans/ru.md)
