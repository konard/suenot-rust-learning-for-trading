# День 147: Ротация логов: логи по дням

## Аналогия из трейдинга

Представь, что ты ведёшь торговый журнал. Каждый день ты записываешь все сделки, рыночные события, свои решения. Если бы ты писал всё в одну бесконечную тетрадь, она стала бы неподъёмной — сотни тысяч записей, найти что-то невозможно, а хранить неудобно.

Поэтому профессиональные трейдеры ведут **отдельный журнал на каждый день**: `2024-01-15.log`, `2024-01-16.log` и так далее. Старые журналы можно архивировать или удалять. Это и есть **ротация логов**.

## Зачем нужна ротация логов

В торговых системах генерируется огромное количество логов:
- Каждый тик цены
- Каждый сигнал стратегии
- Каждый ордер и его статус
- Ошибки подключения к бирже

Без ротации один лог-файл может вырасти до гигабайтов, что приводит к:
- Переполнению диска
- Медленному поиску по логам
- Проблемам с производительностью записи

## Установка зависимостей

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
chrono = "0.4"
```

## Базовая ротация по дням

```rust
use tracing::{info, warn, error, Level};
use tracing_subscriber::fmt;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

fn main() {
    // Создаём appender с ежедневной ротацией
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,           // Новый файл каждый день
        "logs",                    // Директория для логов
        "trading-bot.log",         // Префикс имени файла
    );

    // Настраиваем subscriber
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)  // Отключаем цвета в файле
        .init();

    // Теперь все логи пишутся в файлы вида:
    // logs/trading-bot.log.2024-01-15
    // logs/trading-bot.log.2024-01-16

    info!("Trading bot started");
    simulate_trading_day();
}

fn simulate_trading_day() {
    info!(symbol = "BTC/USDT", "Connecting to exchange");

    // Симуляция торговых событий
    for i in 1..=5 {
        info!(
            trade_id = i,
            symbol = "BTC/USDT",
            side = "BUY",
            price = 42000.0 + (i as f64 * 100.0),
            quantity = 0.1,
            "Order executed"
        );
    }

    warn!(latency_ms = 150, "High latency detected");
    info!("Trading day completed");
}
```

## Комбинирование с выводом в консоль

В реальных системах мы хотим видеть логи и в консоли, и в файлах:

```rust
use tracing::{info, warn, error, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, fmt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_appender::non_blocking;

fn main() {
    // Ротация по дням для файлов
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs",
        "trading-bot.log",
    );

    // Non-blocking writer для файла (не блокирует основной поток)
    let (non_blocking_file, _guard) = non_blocking(file_appender);

    // Создаём два слоя: консоль и файл
    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true);

    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_target(true);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    run_trading_bot();
}

fn run_trading_bot() {
    info!("Bot initialized");

    // Логи идут и в консоль, и в файл
    info!(
        exchange = "Binance",
        symbols = ?["BTC/USDT", "ETH/USDT"],
        "Subscribed to market data"
    );

    process_market_data();
}

fn process_market_data() {
    let prices = vec![
        ("BTC/USDT", 42150.50),
        ("ETH/USDT", 2280.75),
    ];

    for (symbol, price) in prices {
        info!(symbol = symbol, price = price, "Price update received");
    }
}
```

## Различные интервалы ротации

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

fn setup_different_rotations() {
    // Ежеминутная ротация (для тестирования или high-frequency логов)
    let minutely = RollingFileAppender::new(
        Rotation::MINUTELY,
        "logs/minutely",
        "hft-trades.log",
    );

    // Почасовая ротация
    let hourly = RollingFileAppender::new(
        Rotation::HOURLY,
        "logs/hourly",
        "market-data.log",
    );

    // Ежедневная ротация (рекомендуется для большинства случаев)
    let daily = RollingFileAppender::new(
        Rotation::DAILY,
        "logs/daily",
        "trading-bot.log",
    );

    // Без ротации (один файл, ручное управление)
    let never = RollingFileAppender::new(
        Rotation::NEVER,
        "logs",
        "audit.log",  // Важные события, которые нельзя потерять
    );
}
```

## Практический пример: торговый бот с полным логированием

```rust
use tracing::{info, warn, error, debug, instrument, Level};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    fmt,
    EnvFilter,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_appender::non_blocking;
use std::sync::Arc;

// Структура для хранения guard'ов (чтобы логи не потерялись)
struct LogGuards {
    _trade_guard: tracing_appender::non_blocking::WorkerGuard,
    _error_guard: tracing_appender::non_blocking::WorkerGuard,
}

fn setup_logging() -> LogGuards {
    // Отдельный файл для сделок
    let trade_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs/trades",
        "trades.log",
    );
    let (trade_writer, trade_guard) = non_blocking(trade_appender);

    // Отдельный файл для ошибок
    let error_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs/errors",
        "errors.log",
    );
    let (error_writer, error_guard) = non_blocking(error_appender);

    // Консоль для всех уровней
    let console_layer = fmt::layer()
        .with_target(true)
        .with_filter(EnvFilter::new("debug"));

    // Файл для сделок (только info и выше)
    let trade_layer = fmt::layer()
        .with_writer(trade_writer)
        .with_ansi(false)
        .with_filter(EnvFilter::new("info"));

    // Файл для ошибок (только warn и выше)
    let error_layer = fmt::layer()
        .with_writer(error_writer)
        .with_ansi(false)
        .with_filter(EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(console_layer)
        .with(trade_layer)
        .with(error_layer)
        .init();

    LogGuards {
        _trade_guard: trade_guard,
        _error_guard: error_guard,
    }
}

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

#[instrument(skip(trade), fields(trade_id = trade.id, symbol = %trade.symbol))]
fn execute_trade(trade: &Trade) -> Result<(), String> {
    info!(
        side = %trade.side,
        price = trade.price,
        quantity = trade.quantity,
        "Executing trade"
    );

    // Симуляция возможной ошибки
    if trade.price <= 0.0 {
        error!(price = trade.price, "Invalid price");
        return Err("Invalid price".to_string());
    }

    debug!(
        notional = trade.price * trade.quantity,
        "Trade notional calculated"
    );

    info!("Trade executed successfully");
    Ok(())
}

#[instrument]
fn process_order_book(symbol: &str, bids: &[(f64, f64)], asks: &[(f64, f64)]) {
    debug!(
        symbol = symbol,
        bid_levels = bids.len(),
        ask_levels = asks.len(),
        "Processing order book"
    );

    if let (Some(best_bid), Some(best_ask)) = (bids.first(), asks.first()) {
        let spread = best_ask.0 - best_bid.0;
        let spread_pct = (spread / best_bid.0) * 100.0;

        info!(
            symbol = symbol,
            best_bid = best_bid.0,
            best_ask = best_ask.0,
            spread = spread,
            spread_pct = format!("{:.4}%", spread_pct),
            "Order book update"
        );

        if spread_pct > 0.5 {
            warn!(
                symbol = symbol,
                spread_pct = format!("{:.4}%", spread_pct),
                "Wide spread detected"
            );
        }
    }
}

fn main() {
    // Храним guards на протяжении всей программы
    let _guards = setup_logging();

    info!("Trading bot starting");

    // Симуляция торгового дня
    let trades = vec![
        Trade {
            id: 1,
            symbol: "BTC/USDT".to_string(),
            side: "BUY".to_string(),
            price: 42000.0,
            quantity: 0.5,
        },
        Trade {
            id: 2,
            symbol: "ETH/USDT".to_string(),
            side: "SELL".to_string(),
            price: 2280.0,
            quantity: 2.0,
        },
    ];

    for trade in &trades {
        if let Err(e) = execute_trade(trade) {
            error!(error = %e, "Trade execution failed");
        }
    }

    // Симуляция обработки стакана
    let bids = vec![(42000.0, 1.5), (41990.0, 2.0), (41980.0, 3.0)];
    let asks = vec![(42010.0, 1.0), (42020.0, 1.5), (42030.0, 2.5)];
    process_order_book("BTC/USDT", &bids, &asks);

    info!("Trading bot shutting down");

    // При завершении программы guards автоматически
    // сбросят все буферы в файлы
}
```

## Кастомный формат имени файла

```rust
use tracing_appender::rolling::RollingFileAppender;
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

struct CustomDailyLogger {
    log_dir: String,
    prefix: String,
}

impl CustomDailyLogger {
    fn new(log_dir: &str, prefix: &str) -> Self {
        fs::create_dir_all(log_dir).expect("Failed to create log directory");

        CustomDailyLogger {
            log_dir: log_dir.to_string(),
            prefix: prefix.to_string(),
        }
    }

    fn get_today_filename(&self) -> String {
        let today = Local::now().format("%Y-%m-%d");
        format!("{}/{}_{}.log", self.log_dir, self.prefix, today)
    }

    fn log(&self, level: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("[{}] {} - {}\n", timestamp, level, message);

        let filename = self.get_today_filename();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
            .expect("Failed to open log file");

        file.write_all(log_line.as_bytes())
            .expect("Failed to write to log file");
    }

    fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    fn warn(&self, message: &str) {
        self.log("WARN", message);
    }

    fn error(&self, message: &str) {
        self.log("ERROR", message);
    }
}

fn main() {
    let logger = CustomDailyLogger::new("custom_logs", "trading");

    logger.info("Trading session started");
    logger.info("Connected to Binance API");
    logger.warn("Rate limit approaching: 1150/1200 requests");
    logger.info("Order placed: BUY 0.5 BTC @ 42000");
    logger.error("Order rejected: Insufficient balance");
    logger.info("Trading session ended");

    // Файлы будут созданы как:
    // custom_logs/trading_2024-01-15.log
    // custom_logs/trading_2024-01-16.log
}
```

## Очистка старых логов

```rust
use std::fs;
use std::path::Path;
use chrono::{Local, Duration};
use tracing::{info, warn};

fn cleanup_old_logs(log_dir: &str, days_to_keep: i64) -> std::io::Result<usize> {
    let cutoff = Local::now() - Duration::days(days_to_keep);
    let cutoff_date = cutoff.format("%Y-%m-%d").to_string();

    let mut deleted_count = 0;

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Извлекаем дату из имени файла (формат: prefix.log.2024-01-15)
            if let Some(date_part) = filename.rsplit('.').next() {
                if date_part < cutoff_date.as_str() {
                    info!(file = filename, "Deleting old log file");
                    fs::remove_file(&path)?;
                    deleted_count += 1;
                }
            }
        }
    }

    Ok(deleted_count)
}

fn main() {
    // Удаляем логи старше 30 дней
    match cleanup_old_logs("logs", 30) {
        Ok(count) => info!(deleted = count, "Old logs cleaned up"),
        Err(e) => warn!(error = %e, "Failed to cleanup logs"),
    }
}
```

## Ротация по размеру файла

Иногда нужна ротация не по времени, а по размеру:

```rust
use std::fs::{self, File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::Path;

struct SizeBasedLogger {
    log_dir: String,
    prefix: String,
    max_size_bytes: u64,
    current_file_num: u32,
}

impl SizeBasedLogger {
    fn new(log_dir: &str, prefix: &str, max_size_mb: u64) -> Self {
        fs::create_dir_all(log_dir).expect("Failed to create log directory");

        SizeBasedLogger {
            log_dir: log_dir.to_string(),
            prefix: prefix.to_string(),
            max_size_bytes: max_size_mb * 1024 * 1024,
            current_file_num: 0,
        }
    }

    fn get_current_filename(&self) -> String {
        format!("{}/{}.{}.log", self.log_dir, self.prefix, self.current_file_num)
    }

    fn should_rotate(&self) -> bool {
        let path = self.get_current_filename();
        if let Ok(metadata) = fs::metadata(&path) {
            metadata.len() >= self.max_size_bytes
        } else {
            false
        }
    }

    fn log(&mut self, message: &str) {
        if self.should_rotate() {
            self.current_file_num += 1;
        }

        let filename = self.get_current_filename();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
            .expect("Failed to open log file");

        writeln!(file, "{}", message).expect("Failed to write to log file");
    }
}

fn main() {
    // Максимум 10 MB на файл
    let mut logger = SizeBasedLogger::new("logs/sized", "trades", 10);

    for i in 0..1000 {
        logger.log(&format!("Trade #{}: BUY 0.1 BTC @ {}", i, 42000 + i));
    }
}
```

## Архивирование старых логов

```rust
use std::fs::{self, File};
use std::io::{Read, Write};
use flate2::write::GzEncoder;
use flate2::Compression;
use chrono::{Local, Duration};
use tracing::info;

fn archive_old_logs(log_dir: &str, days_old: i64) -> std::io::Result<usize> {
    let cutoff = Local::now() - Duration::days(days_old);
    let cutoff_date = cutoff.format("%Y-%m-%d").to_string();

    let archive_dir = format!("{}/archive", log_dir);
    fs::create_dir_all(&archive_dir)?;

    let mut archived_count = 0;

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |e| e != "gz") {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Проверяем дату в имени файла
            if let Some(date_part) = filename.rsplit('.').next() {
                if date_part < cutoff_date.as_str() {
                    // Сжимаем и перемещаем в архив
                    let archive_path = format!("{}/{}.gz", archive_dir, filename);

                    let mut input = File::open(&path)?;
                    let mut content = Vec::new();
                    input.read_to_end(&mut content)?;

                    let output = File::create(&archive_path)?;
                    let mut encoder = GzEncoder::new(output, Compression::default());
                    encoder.write_all(&content)?;
                    encoder.finish()?;

                    fs::remove_file(&path)?;

                    info!(
                        original = filename,
                        archived = format!("{}.gz", filename),
                        "Log file archived"
                    );

                    archived_count += 1;
                }
            }
        }
    }

    Ok(archived_count)
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `Rotation::DAILY` | Новый файл каждый день |
| `Rotation::HOURLY` | Новый файл каждый час |
| `non_blocking` | Асинхронная запись логов |
| `WorkerGuard` | Гарантирует сброс буферов |
| Несколько слоёв | Логи в консоль и файлы одновременно |
| Очистка логов | Удаление старых файлов |
| Архивирование | Сжатие старых логов |

## Практические задания

1. **Разделение по типам событий**: Создай систему логирования, где сделки пишутся в `trades/`, ошибки в `errors/`, а рыночные данные в `market-data/`. Каждая директория должна иметь ежедневную ротацию.

2. **Анализатор логов**: Напиши программу, которая читает лог-файлы за указанный день и выводит статистику: количество сделок, количество ошибок, среднее время между сделками.

3. **Автоматическое архивирование**: Реализуй фоновую задачу, которая каждый час проверяет директорию логов и архивирует файлы старше 7 дней, удаляя архивы старше 30 дней.

## Домашнее задание

1. Создай полноценную систему логирования для торгового бота с:
   - Ежедневной ротацией для обычных логов
   - Отдельным файлом для критических ошибок (без ротации)
   - Почасовой ротацией для high-frequency данных
   - Автоматической очисткой логов старше 90 дней

2. Реализуй парсер лог-файлов, который может:
   - Найти все ошибки за указанный период
   - Подсчитать количество сделок по каждому символу
   - Вычислить общий PnL на основе записей о сделках

3. Добавь в систему логирования возможность:
   - Отправлять критические ошибки по email/Telegram
   - Генерировать ежедневный отчёт на основе логов
   - Визуализировать активность торгового бота по часам

## Навигация

[← Предыдущий день](../146-structured-logs-tracing/ru.md) | [Следующий день →](../148-data-compression-large-files/ru.md)
