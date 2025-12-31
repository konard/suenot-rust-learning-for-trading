# День 135: Парсинг дат: chrono crate

## Аналогия из трейдинга

В трейдинге время — это всё. Каждая свеча на графике имеет временную метку. Когда ты загружаешь исторические данные, даты приходят в виде строк: `"2024-01-15 09:30:00"`, `"15.01.2024"`, `"Jan 15, 2024"`. Чтобы анализировать эти данные — сравнивать времена сделок, фильтровать по торговым сессиям, рассчитывать длительность позиций — нужно преобразовать строки в настоящие объекты даты и времени.

Крейт `chrono` — это стандартный инструмент Rust для работы с датами. Он как универсальный конвертер времени в торговом терминале: понимает любые форматы и позволяет делать с датами всё что угодно.

## Подключение chrono

Добавь в `Cargo.toml`:

```toml
[dependencies]
chrono = "0.4"
```

## Основные типы chrono

```rust
use chrono::{NaiveDate, NaiveTime, NaiveDateTime, DateTime, Utc, Local};

fn main() {
    // NaiveDate — дата без часового пояса
    let trade_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    println!("Дата сделки: {}", trade_date);

    // NaiveTime — время без часового пояса
    let trade_time = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
    println!("Время сделки: {}", trade_time);

    // NaiveDateTime — дата и время без часового пояса
    let trade_datetime = NaiveDateTime::new(trade_date, trade_time);
    println!("Дата и время: {}", trade_datetime);

    // DateTime<Utc> — дата и время с часовым поясом UTC
    let now_utc: DateTime<Utc> = Utc::now();
    println!("Сейчас UTC: {}", now_utc);

    // DateTime<Local> — локальное время
    let now_local: DateTime<Local> = Local::now();
    println!("Сейчас локально: {}", now_local);
}
```

## Парсинг дат из строк

### Стандартные форматы

```rust
use chrono::NaiveDate;

fn main() {
    // ISO формат (YYYY-MM-DD) — самый распространённый в торговых данных
    let date = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap();
    println!("Дата: {}", date);

    // Европейский формат
    let date_eu = NaiveDate::parse_from_str("15.01.2024", "%d.%m.%Y").unwrap();
    println!("Европейский формат: {}", date_eu);

    // Американский формат
    let date_us = NaiveDate::parse_from_str("01/15/2024", "%m/%d/%Y").unwrap();
    println!("Американский формат: {}", date_us);
}
```

### Парсинг даты и времени

```rust
use chrono::NaiveDateTime;

fn main() {
    // Формат биржевых данных
    let dt = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00",
        "%Y-%m-%d %H:%M:%S"
    ).unwrap();
    println!("Время сделки: {}", dt);

    // ISO 8601 с T-разделителем
    let dt_iso = NaiveDateTime::parse_from_str(
        "2024-01-15T09:30:00",
        "%Y-%m-%dT%H:%M:%S"
    ).unwrap();
    println!("ISO формат: {}", dt_iso);

    // С миллисекундами (часто в tick-данных)
    let dt_ms = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00.123",
        "%Y-%m-%d %H:%M:%S%.3f"
    ).unwrap();
    println!("С миллисекундами: {}", dt_ms);
}
```

### Спецификаторы формата

| Спецификатор | Значение | Пример |
|-------------|----------|--------|
| `%Y` | Год (4 цифры) | 2024 |
| `%m` | Месяц (01-12) | 01 |
| `%d` | День (01-31) | 15 |
| `%H` | Час (00-23) | 09 |
| `%M` | Минуты (00-59) | 30 |
| `%S` | Секунды (00-59) | 00 |
| `%f` | Микросекунды | 123456 |
| `%.3f` | Миллисекунды | .123 |
| `%Y-%m-%d` | ISO дата | 2024-01-15 |

## Безопасный парсинг для торговых данных

```rust
use chrono::NaiveDateTime;

fn parse_trade_time(time_str: &str) -> Option<NaiveDateTime> {
    // Пробуем разные форматы, которые могут прийти от биржи
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%d.%m.%Y %H:%M:%S",
    ];

    for format in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, format) {
            return Some(dt);
        }
    }
    None
}

fn main() {
    let test_times = [
        "2024-01-15 09:30:00",
        "2024-01-15T09:30:00",
        "2024-01-15 09:30:00.123",
        "15.01.2024 09:30:00",
        "invalid-date",
    ];

    for time_str in test_times {
        match parse_trade_time(time_str) {
            Some(dt) => println!("'{}' -> {}", time_str, dt),
            None => println!("'{}' -> Ошибка парсинга", time_str),
        }
    }
}
```

## Парсинг OHLCV данных с датами

```rust
use chrono::NaiveDateTime;

struct Candle {
    timestamp: NaiveDateTime,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn parse_candle(line: &str) -> Option<Candle> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 6 {
        return None;
    }

    let timestamp = NaiveDateTime::parse_from_str(parts[0], "%Y-%m-%d %H:%M:%S").ok()?;
    let open = parts[1].parse().ok()?;
    let high = parts[2].parse().ok()?;
    let low = parts[3].parse().ok()?;
    let close = parts[4].parse().ok()?;
    let volume = parts[5].parse().ok()?;

    Some(Candle {
        timestamp,
        open,
        high,
        low,
        close,
        volume,
    })
}

fn main() {
    let csv_lines = [
        "2024-01-15 09:30:00,42000.0,42150.0,41950.0,42100.0,1500.5",
        "2024-01-15 09:35:00,42100.0,42200.0,42050.0,42180.0,1200.3",
        "2024-01-15 09:40:00,42180.0,42250.0,42100.0,42150.0,980.7",
    ];

    println!("Загруженные свечи:");
    println!("{:<20} {:>10} {:>10} {:>10} {:>10}",
             "Время", "Open", "High", "Low", "Close");
    println!("{}", "-".repeat(65));

    for line in csv_lines {
        if let Some(candle) = parse_candle(line) {
            println!("{:<20} {:>10.2} {:>10.2} {:>10.2} {:>10.2}",
                     candle.timestamp.format("%Y-%m-%d %H:%M"),
                     candle.open, candle.high, candle.low, candle.close);
        }
    }
}
```

## Операции с датами

### Сравнение дат

```rust
use chrono::NaiveDateTime;

fn main() {
    let entry_time = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    let exit_time = NaiveDateTime::parse_from_str(
        "2024-01-15 14:45:00", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    if exit_time > entry_time {
        println!("Выход после входа — корректная сделка");
    }

    // Длительность позиции
    let duration = exit_time - entry_time;
    println!("Позиция удерживалась: {} часов {} минут",
             duration.num_hours(),
             duration.num_minutes() % 60);
}
```

### Арифметика с датами

```rust
use chrono::{NaiveDateTime, Duration};

fn main() {
    let trade_time = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    // Добавляем время
    let plus_5min = trade_time + Duration::minutes(5);
    let plus_1hour = trade_time + Duration::hours(1);
    let plus_1day = trade_time + Duration::days(1);

    println!("Время сделки: {}", trade_time);
    println!("+ 5 минут:    {}", plus_5min);
    println!("+ 1 час:      {}", plus_1hour);
    println!("+ 1 день:     {}", plus_1day);

    // Вычитаем время
    let yesterday = trade_time - Duration::days(1);
    println!("Вчера:        {}", yesterday);
}
```

### Извлечение компонентов

```rust
use chrono::{NaiveDateTime, Datelike, Timelike, Weekday};

fn main() {
    let trade_time = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:00", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    // Компоненты даты
    println!("Год: {}", trade_time.year());
    println!("Месяц: {}", trade_time.month());
    println!("День: {}", trade_time.day());
    println!("День недели: {:?}", trade_time.weekday());

    // Компоненты времени
    println!("Час: {}", trade_time.hour());
    println!("Минуты: {}", trade_time.minute());
    println!("Секунды: {}", trade_time.second());

    // Проверка на торговый день (не выходные)
    let weekday = trade_time.weekday();
    let is_trading_day = weekday != Weekday::Sat && weekday != Weekday::Sun;
    println!("Торговый день: {}", is_trading_day);
}
```

## Фильтрация сделок по времени

```rust
use chrono::{NaiveDateTime, NaiveTime, Timelike};

struct Trade {
    time: NaiveDateTime,
    symbol: String,
    price: f64,
    quantity: f64,
}

fn is_during_trading_hours(time: &NaiveDateTime) -> bool {
    let market_open = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
    let market_close = NaiveTime::from_hms_opt(16, 0, 0).unwrap();

    let trade_time = time.time();
    trade_time >= market_open && trade_time < market_close
}

fn is_first_hour(time: &NaiveDateTime) -> bool {
    let hour = time.hour();
    hour == 9 || (hour == 10 && time.minute() < 30)
}

fn is_last_hour(time: &NaiveDateTime) -> bool {
    let hour = time.hour();
    hour == 15 || (hour == 14 && time.minute() >= 30)
}

fn main() {
    let trades = vec![
        Trade {
            time: NaiveDateTime::parse_from_str("2024-01-15 09:35:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            price: 42000.0,
            quantity: 0.5,
        },
        Trade {
            time: NaiveDateTime::parse_from_str("2024-01-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            price: 42150.0,
            quantity: 0.3,
        },
        Trade {
            time: NaiveDateTime::parse_from_str("2024-01-15 15:45:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            price: 42200.0,
            quantity: 0.4,
        },
        Trade {
            time: NaiveDateTime::parse_from_str("2024-01-15 17:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            price: 42100.0,
            quantity: 0.2,
        },
    ];

    println!("Анализ сделок по времени:");
    println!("{}", "-".repeat(60));

    for trade in &trades {
        let during_hours = is_during_trading_hours(&trade.time);
        let first_hour = is_first_hour(&trade.time);
        let last_hour = is_last_hour(&trade.time);

        println!("{} - {} @ ${:.2}",
                 trade.time.format("%H:%M:%S"),
                 trade.symbol,
                 trade.price);
        println!("  В торговые часы: {}, Первый час: {}, Последний час: {}",
                 during_hours, first_hour, last_hour);
    }

    // Фильтруем только сделки в торговые часы
    let valid_trades: Vec<_> = trades
        .iter()
        .filter(|t| is_during_trading_hours(&t.time))
        .collect();

    println!("\nСделки в торговые часы: {}", valid_trades.len());
}
```

## Группировка свечей по периодам

```rust
use chrono::{NaiveDateTime, Duration, Timelike};

struct Candle {
    timestamp: NaiveDateTime,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn aggregate_to_hourly(candles: &[Candle]) -> Vec<Candle> {
    if candles.is_empty() {
        return vec![];
    }

    let mut hourly: Vec<Candle> = vec![];
    let mut current_hour = candles[0].timestamp.date().and_hms_opt(
        candles[0].timestamp.hour(), 0, 0
    ).unwrap();

    let mut open = candles[0].open;
    let mut high = candles[0].high;
    let mut low = candles[0].low;
    let mut close = candles[0].close;
    let mut volume = 0.0;

    for candle in candles {
        let candle_hour = candle.timestamp.date().and_hms_opt(
            candle.timestamp.hour(), 0, 0
        ).unwrap();

        if candle_hour != current_hour {
            // Сохраняем часовую свечу
            hourly.push(Candle {
                timestamp: current_hour,
                open,
                high,
                low,
                close,
                volume,
            });

            // Начинаем новую
            current_hour = candle_hour;
            open = candle.open;
            high = candle.high;
            low = candle.low;
            volume = 0.0;
        }

        high = high.max(candle.high);
        low = low.min(candle.low);
        close = candle.close;
        volume += candle.volume;
    }

    // Последняя свеча
    hourly.push(Candle {
        timestamp: current_hour,
        open,
        high,
        low,
        close,
        volume,
    });

    hourly
}

fn main() {
    let candles_5m = vec![
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42000.0, high: 42050.0, low: 41980.0, close: 42030.0, volume: 100.0,
        },
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 09:05:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42030.0, high: 42100.0, low: 42020.0, close: 42080.0, volume: 150.0,
        },
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 09:10:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42080.0, high: 42120.0, low: 42050.0, close: 42100.0, volume: 120.0,
        },
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42100.0, high: 42200.0, low: 42090.0, close: 42180.0, volume: 200.0,
        },
        Candle {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 10:05:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            open: 42180.0, high: 42250.0, low: 42150.0, close: 42220.0, volume: 180.0,
        },
    ];

    let hourly = aggregate_to_hourly(&candles_5m);

    println!("Часовые свечи:");
    for candle in hourly {
        println!("{}: O={:.0} H={:.0} L={:.0} C={:.0} V={:.0}",
                 candle.timestamp.format("%Y-%m-%d %H:00"),
                 candle.open, candle.high, candle.low, candle.close, candle.volume);
    }
}
```

## Форматирование дат для вывода

```rust
use chrono::NaiveDateTime;

fn main() {
    let dt = NaiveDateTime::parse_from_str(
        "2024-01-15 09:30:45", "%Y-%m-%d %H:%M:%S"
    ).unwrap();

    // Разные форматы вывода
    println!("ISO:           {}", dt.format("%Y-%m-%d %H:%M:%S"));
    println!("Компактный:    {}", dt.format("%Y%m%d_%H%M%S"));
    println!("Читаемый:      {}", dt.format("%d %B %Y, %H:%M"));
    println!("Только дата:   {}", dt.format("%Y-%m-%d"));
    println!("Только время:  {}", dt.format("%H:%M:%S"));
    println!("Для файла:     {}", dt.format("%Y-%m-%d_%H-%M-%S"));
}
```

## Практический пример: анализ торговой сессии

```rust
use chrono::{NaiveDateTime, Duration, Timelike, Weekday, Datelike};

struct Trade {
    timestamp: NaiveDateTime,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    pnl: f64,
}

struct SessionStats {
    date: String,
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    total_pnl: f64,
    first_trade: String,
    last_trade: String,
    session_duration_mins: i64,
}

fn analyze_session(trades: &[Trade]) -> Option<SessionStats> {
    if trades.is_empty() {
        return None;
    }

    let first = trades.first().unwrap();
    let last = trades.last().unwrap();

    let winning = trades.iter().filter(|t| t.pnl > 0.0).count();
    let losing = trades.iter().filter(|t| t.pnl < 0.0).count();
    let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();

    let duration = last.timestamp - first.timestamp;

    Some(SessionStats {
        date: first.timestamp.format("%Y-%m-%d").to_string(),
        total_trades: trades.len(),
        winning_trades: winning,
        losing_trades: losing,
        total_pnl,
        first_trade: first.timestamp.format("%H:%M:%S").to_string(),
        last_trade: last.timestamp.format("%H:%M:%S").to_string(),
        session_duration_mins: duration.num_minutes(),
    })
}

fn main() {
    let trades = vec![
        Trade {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 09:32:15", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            side: "BUY".to_string(),
            price: 42000.0,
            quantity: 0.5,
            pnl: 150.0,
        },
        Trade {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 10:45:30", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "ETHUSD".to_string(),
            side: "SELL".to_string(),
            price: 2500.0,
            quantity: 2.0,
            pnl: -50.0,
        },
        Trade {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 14:20:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            side: "BUY".to_string(),
            price: 42150.0,
            quantity: 0.3,
            pnl: 200.0,
        },
        Trade {
            timestamp: NaiveDateTime::parse_from_str("2024-01-15 15:55:45", "%Y-%m-%d %H:%M:%S").unwrap(),
            symbol: "BTCUSD".to_string(),
            side: "SELL".to_string(),
            price: 42300.0,
            quantity: 0.4,
            pnl: 75.0,
        },
    ];

    if let Some(stats) = analyze_session(&trades) {
        println!("╔═══════════════════════════════════════════╗");
        println!("║         СТАТИСТИКА СЕССИИ                 ║");
        println!("╠═══════════════════════════════════════════╣");
        println!("║ Дата:              {:>20} ║", stats.date);
        println!("║ Всего сделок:      {:>20} ║", stats.total_trades);
        println!("║ Прибыльных:        {:>20} ║", stats.winning_trades);
        println!("║ Убыточных:         {:>20} ║", stats.losing_trades);
        println!("║ Общий PnL:         ${:>19.2} ║", stats.total_pnl);
        println!("║ Первая сделка:     {:>20} ║", stats.first_trade);
        println!("║ Последняя сделка:  {:>20} ║", stats.last_trade);
        println!("║ Длительность:      {:>17} мин ║", stats.session_duration_mins);
        println!("╚═══════════════════════════════════════════╝");
    }
}
```

## Что мы узнали

| Тип | Описание | Пример |
|-----|----------|--------|
| `NaiveDate` | Дата без часового пояса | `2024-01-15` |
| `NaiveTime` | Время без часового пояса | `09:30:00` |
| `NaiveDateTime` | Дата и время | `2024-01-15 09:30:00` |
| `DateTime<Utc>` | Дата/время в UTC | С часовым поясом |
| `Duration` | Промежуток времени | Часы, минуты, секунды |

## Домашнее задание

1. Напиши функцию `parse_exchange_timestamp(s: &str) -> Result<NaiveDateTime, String>`, которая поддерживает минимум 5 разных форматов дат от разных бирж

2. Создай функцию `get_trading_session(time: &NaiveDateTime) -> &str`, которая возвращает "pre-market", "regular", "after-hours" или "closed" в зависимости от времени

3. Реализуй функцию `calculate_holding_period(entry: &str, exit: &str) -> Result<Duration, String>`, которая вычисляет время удержания позиции

4. Напиши программу, которая группирует список сделок по дням и выводит статистику за каждый день

## Навигация

[← Предыдущий день](../134-csv-reading-ohlcv/ru.md) | [Следующий день →](../136-timezones-utc-local/ru.md)
