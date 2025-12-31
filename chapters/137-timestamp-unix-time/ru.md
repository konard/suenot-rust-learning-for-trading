# День 137: Timestamp — Unix время для бирж

## Аналогия из трейдинга

Каждая сделка на бирже имеет **точную метку времени**. Когда ты смотришь историю ордеров, ты видишь: "2024-01-15 14:32:05.123". Но для компьютеров и бирж удобнее хранить время как **одно число** — количество секунд (или миллисекунд) с 1 января 1970 года.

Представь, что вместо даты "15 января 2024, 14:32:05" биржа записывает **1705329125**. Это и есть **Unix timestamp** — универсальный способ записи времени, который:
- Одинаково работает во всех часовых поясах
- Легко сравнивать (больше = позже)
- Удобно хранить в базе данных
- Используется всеми крупными биржами (Binance, Bybit, Kraken)

**Timestamp — это как уникальный номер чека на бирже**, только вместо порядкового номера используется время.

## Что такое Unix Timestamp?

```rust
fn main() {
    // Unix timestamp — секунды с 1 января 1970 года (UTC)
    let timestamp: i64 = 1705329125;

    println!("Timestamp: {}", timestamp);
    println!("Это примерно: 15 января 2024, 14:32:05 UTC");

    // Биржи часто используют миллисекунды
    let timestamp_ms: i64 = 1705329125123;
    println!("В миллисекундах: {}", timestamp_ms);
}
```

## Получение текущего времени

```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Текущий Unix timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let timestamp_secs = now.as_secs();
    let timestamp_millis = now.as_millis();

    println!("Секунды с 1970: {}", timestamp_secs);
    println!("Миллисекунды с 1970: {}", timestamp_millis);
}
```

## Timestamp в торговых операциях

### Запись сделки

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct Trade {
    symbol: String,
    side: String,       // "BUY" или "SELL"
    price: f64,
    quantity: f64,
    timestamp: i64,     // Unix timestamp в миллисекундах
}

fn create_trade(symbol: &str, side: &str, price: f64, quantity: f64) -> Trade {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    Trade {
        symbol: symbol.to_string(),
        side: side.to_string(),
        price,
        quantity,
        timestamp,
    }
}

fn main() {
    let trade = create_trade("BTC/USDT", "BUY", 42500.0, 0.5);

    println!("Сделка:");
    println!("  Пара: {}", trade.symbol);
    println!("  Сторона: {}", trade.side);
    println!("  Цена: ${:.2}", trade.price);
    println!("  Количество: {}", trade.quantity);
    println!("  Timestamp: {}", trade.timestamp);
}
```

### Свеча (OHLCV)

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct Candle {
    open_time: i64,     // Время открытия свечи
    close_time: i64,    // Время закрытия свечи
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn create_1h_candle(open_time: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Candle {
    // Часовая свеча длится 3600 секунд (1 час)
    let close_time = open_time + 3600 * 1000 - 1; // в миллисекундах, -1 чтобы не пересекаться

    Candle {
        open_time,
        close_time,
        open,
        high,
        low,
        close,
        volume,
    }
}

fn main() {
    // Начало часа: 15 января 2024, 14:00:00 UTC
    let open_time: i64 = 1705327200000; // миллисекунды

    let candle = create_1h_candle(
        open_time,
        42000.0,  // open
        42500.0,  // high
        41800.0,  // low
        42300.0,  // close
        1250.5    // volume
    );

    println!("1H Свеча:");
    println!("  Открытие: {} ({})", candle.open_time, format_timestamp(candle.open_time));
    println!("  Закрытие: {} ({})", candle.close_time, format_timestamp(candle.close_time));
    println!("  OHLC: {}/{}/{}/{}", candle.open, candle.high, candle.low, candle.close);
    println!("  Объём: {:.2}", candle.volume);
}

fn format_timestamp(ts: i64) -> String {
    // Простое форматирование для демонстрации
    format!("{}ms от UNIX epoch", ts)
}
```

## Сравнение и сортировка по времени

```rust
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
    timestamp: i64,
}

fn main() {
    let mut orders = vec![
        Order { id: 1, price: 42000.0, quantity: 0.5, timestamp: 1705329125000 },
        Order { id: 2, price: 42100.0, quantity: 0.3, timestamp: 1705329120000 },
        Order { id: 3, price: 41900.0, quantity: 0.7, timestamp: 1705329130000 },
    ];

    // Сортировка по времени (от старых к новым)
    orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    println!("Ордера по времени создания:");
    for order in &orders {
        println!("  ID: {}, Цена: {}, Время: {}", order.id, order.price, order.timestamp);
    }

    // Найти самый новый ордер
    if let Some(newest) = orders.iter().max_by_key(|o| o.timestamp) {
        println!("\nСамый новый ордер: ID {}", newest.id);
    }

    // Найти ордера за последние 5 секунд
    let five_seconds_ago = 1705329130000 - 5000;
    let recent: Vec<&Order> = orders
        .iter()
        .filter(|o| o.timestamp >= five_seconds_ago)
        .collect();

    println!("\nОрдера за последние 5 секунд: {}", recent.len());
}
```

## Расчёт времени исполнения ордера

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct OrderExecution {
    order_id: u64,
    created_at: i64,
    executed_at: i64,
}

impl OrderExecution {
    fn execution_time_ms(&self) -> i64 {
        self.executed_at - self.created_at
    }

    fn execution_time_secs(&self) -> f64 {
        self.execution_time_ms() as f64 / 1000.0
    }
}

fn main() {
    let execution = OrderExecution {
        order_id: 12345,
        created_at: 1705329125000,
        executed_at: 1705329125150, // исполнен через 150мс
    };

    println!("Ордер #{}", execution.order_id);
    println!("  Время исполнения: {}мс", execution.execution_time_ms());
    println!("  Время исполнения: {:.3}с", execution.execution_time_secs());

    // Проверка на медленное исполнение
    if execution.execution_time_ms() > 100 {
        println!("  ВНИМАНИЕ: Медленное исполнение!");
    } else {
        println!("  Быстрое исполнение");
    }
}
```

## Интервалы свечей

```rust
const MINUTE: i64 = 60 * 1000;        // 1 минута в миллисекундах
const HOUR: i64 = 60 * MINUTE;        // 1 час
const DAY: i64 = 24 * HOUR;           // 1 день
const WEEK: i64 = 7 * DAY;            // 1 неделя

fn get_candle_start(timestamp: i64, interval: i64) -> i64 {
    // Округление вниз до начала интервала
    (timestamp / interval) * interval
}

fn get_candle_end(timestamp: i64, interval: i64) -> i64 {
    get_candle_start(timestamp, interval) + interval - 1
}

fn main() {
    let now: i64 = 1705329125500; // Пример текущего времени

    println!("Текущий timestamp: {}", now);
    println!();

    // 1-минутная свеча
    let m1_start = get_candle_start(now, MINUTE);
    let m1_end = get_candle_end(now, MINUTE);
    println!("1m свеча: {} - {}", m1_start, m1_end);

    // 1-часовая свеча
    let h1_start = get_candle_start(now, HOUR);
    let h1_end = get_candle_end(now, HOUR);
    println!("1h свеча: {} - {}", h1_start, h1_end);

    // Дневная свеча
    let d1_start = get_candle_start(now, DAY);
    let d1_end = get_candle_end(now, DAY);
    println!("1d свеча: {} - {}", d1_start, d1_end);
}
```

## Работа с API бирж

```rust
// Binance использует миллисекунды
fn binance_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// Некоторые биржи используют секунды
fn kraken_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

// Конвертация между форматами
fn millis_to_secs(millis: i64) -> i64 {
    millis / 1000
}

fn secs_to_millis(secs: i64) -> i64 {
    secs * 1000
}

fn main() {
    let binance_ts = 1705329125123_i64; // миллисекунды
    let kraken_ts = 1705329125_i64;     // секунды

    // Приводим к единому формату (миллисекунды)
    let normalized_binance = binance_ts;
    let normalized_kraken = secs_to_millis(kraken_ts);

    println!("Binance: {} ms", normalized_binance);
    println!("Kraken:  {} ms", normalized_kraken);

    // Разница во времени между биржами
    let diff = (normalized_binance - normalized_kraken).abs();
    println!("Разница: {} ms", diff);
}
```

## Проверка устаревших данных

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct MarketData {
    symbol: String,
    price: f64,
    timestamp: i64,
}

fn is_stale(data: &MarketData, max_age_ms: i64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    now - data.timestamp > max_age_ms
}

fn main() {
    let data = MarketData {
        symbol: "BTC/USDT".to_string(),
        price: 42500.0,
        timestamp: 1705329125000,
    };

    // Проверка: данные устарели более чем на 1 секунду?
    let max_age = 1000; // 1 секунда

    // Для демонстрации сравним с фиксированным временем
    let now = 1705329127000_i64; // 2 секунды спустя
    let age = now - data.timestamp;

    println!("Возраст данных: {}ms", age);

    if age > max_age {
        println!("ВНИМАНИЕ: Данные устарели! Не использовать для торговли.");
    } else {
        println!("Данные актуальны.");
    }
}
```

## Timestamp в логах и отладке

```rust
use std::time::{SystemTime, UNIX_EPOCH};

enum LogLevel {
    Info,
    Warning,
    Error,
}

struct LogEntry {
    timestamp: i64,
    level: LogLevel,
    message: String,
}

fn log(level: LogLevel, message: &str) -> LogEntry {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    LogEntry {
        timestamp,
        level,
        message: message.to_string(),
    }
}

fn print_log(entry: &LogEntry) {
    let level_str = match entry.level {
        LogLevel::Info => "INFO",
        LogLevel::Warning => "WARN",
        LogLevel::Error => "ERROR",
    };

    println!("[{}] [{}] {}", entry.timestamp, level_str, entry.message);
}

fn main() {
    let logs = vec![
        LogEntry {
            timestamp: 1705329125000,
            level: LogLevel::Info,
            message: "Соединение с биржей установлено".to_string()
        },
        LogEntry {
            timestamp: 1705329125050,
            level: LogLevel::Info,
            message: "Получены данные по BTC/USDT".to_string()
        },
        LogEntry {
            timestamp: 1705329125100,
            level: LogLevel::Warning,
            message: "Высокая задержка: 85ms".to_string()
        },
        LogEntry {
            timestamp: 1705329125500,
            level: LogLevel::Error,
            message: "Ордер отклонён: недостаточно средств".to_string()
        },
    ];

    println!("=== Лог торговой системы ===");
    for entry in &logs {
        print_log(entry);
    }

    // Анализ: сколько времени между первым и последним событием?
    if logs.len() >= 2 {
        let duration = logs.last().unwrap().timestamp - logs.first().unwrap().timestamp;
        println!("\nОбщее время работы: {}ms", duration);
    }
}
```

## Практический пример: анализатор latency

```rust
use std::time::{SystemTime, UNIX_EPOCH};

struct LatencyStats {
    min: i64,
    max: i64,
    total: i64,
    count: usize,
}

impl LatencyStats {
    fn new() -> Self {
        LatencyStats {
            min: i64::MAX,
            max: i64::MIN,
            total: 0,
            count: 0,
        }
    }

    fn add(&mut self, latency: i64) {
        if latency < self.min { self.min = latency; }
        if latency > self.max { self.max = latency; }
        self.total += latency;
        self.count += 1;
    }

    fn average(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.total as f64 / self.count as f64 }
    }

    fn print(&self) {
        println!("Статистика задержек:");
        println!("  Запросов: {}", self.count);
        println!("  Мин: {}ms", self.min);
        println!("  Макс: {}ms", self.max);
        println!("  Среднее: {:.2}ms", self.average());
    }
}

fn main() {
    // Симуляция данных о задержках запросов к бирже
    let request_times = [
        (1705329125000_i64, 1705329125015_i64), // 15ms
        (1705329125100_i64, 1705329125108_i64), // 8ms
        (1705329125200_i64, 1705329125225_i64), // 25ms
        (1705329125300_i64, 1705329125312_i64), // 12ms
        (1705329125400_i64, 1705329125450_i64), // 50ms - медленный
    ];

    let mut stats = LatencyStats::new();

    for (sent, received) in request_times.iter() {
        let latency = received - sent;
        stats.add(latency);

        if latency > 20 {
            println!("SLOW REQUEST: {}ms at timestamp {}", latency, sent);
        }
    }

    println!();
    stats.print();
}
```

## Что мы узнали

| Концепция | Описание | Пример использования |
|-----------|----------|---------------------|
| Unix timestamp | Секунды/миллисекунды с 1.1.1970 | Время сделки |
| SystemTime | Системное время в Rust | Получение текущего времени |
| Миллисекунды | 1/1000 секунды | Binance API |
| Сравнение timestamp | Простое сравнение чисел | Сортировка ордеров |
| Интервалы | Расчёт начала/конца свечи | OHLCV данные |

## Практические задания

1. **Конвертер времени**: Напиши функцию `timestamp_to_date_parts(ts: i64) -> (i32, u32, u32, u32, u32, u32)`, которая извлекает год, месяц, день, часы, минуты, секунды из timestamp (без внешних библиотек).

2. **Детектор гэпов**: Напиши функцию `find_data_gaps(timestamps: &[i64], expected_interval: i64) -> Vec<(i64, i64)>`, которая находит пропуски в данных свечей.

3. **Rate limiter**: Реализуй структуру `RateLimiter`, которая отслеживает количество запросов за последнюю секунду и блокирует превышение лимита.

4. **Синхронизация часов**: Напиши функцию, которая рассчитывает разницу между локальным временем и временем сервера биржи на основе timestamp в ответе API.

## Домашнее задание

1. Создай структуру `TradeHistory`, которая хранит сделки и позволяет:
   - Добавлять сделки с автоматическим timestamp
   - Получать сделки за последние N минут
   - Рассчитывать среднюю цену за период

2. Напиши функцию агрегации тиков в свечи: `aggregate_to_candles(ticks: &[Tick], interval: i64) -> Vec<Candle>`

3. Реализуй "heartbeat" систему, которая проверяет, что данные от биржи приходят регулярно, и выдаёт предупреждение при задержке более 5 секунд

4. Создай утилиту для синхронизации данных между биржами с разными форматами timestamp (секунды vs миллисекунды)

## Навигация

[← Предыдущий день](../136-timezones-utc-local/ru.md) | [Следующий день →](../138-duration-time-between-trades/ru.md)
