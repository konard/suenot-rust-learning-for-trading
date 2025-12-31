# День 136: Таймзоны: UTC и локальное время

## Аналогия из трейдинга

Представь, что ты торгуешь на нескольких биржах одновременно: NYSE открывается в 9:30 по Нью-Йорку, Московская биржа — в 10:00 по Москве, а крипто-биржи работают 24/7 по UTC. Чтобы не запутаться, когда какая биржа открывается, все профессиональные трейдеры используют **единое время — UTC** (Coordinated Universal Time). Это как "язык времени", который понимают все.

UTC — это "нулевой меридиан" времени. Москва = UTC+3, Нью-Йорк = UTC-5. Когда ты видишь timestamp сделки `1704067200`, это всегда UTC — универсальное время, не зависящее от часового пояса.

## Зачем это нужно в трейдинге?

```rust
// Проблема без единого времени:
// Сделка в Нью-Йорке: 2024-01-01 09:30:00 EST
// Сделка в Москве:    2024-01-01 17:30:00 MSK
// Какая была раньше? Нужно считать в голове!

// С UTC всё ясно:
// Сделка в Нью-Йорке: 2024-01-01 14:30:00 UTC
// Сделка в Москве:    2024-01-01 14:30:00 UTC
// Они одновременные!
```

## Подключаем chrono

```toml
# Cargo.toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
```

## UTC время — стандарт для бирж

```rust
use chrono::{Utc, DateTime};

fn main() {
    // Текущее время в UTC — так работают все биржи
    let now: DateTime<Utc> = Utc::now();
    println!("Текущее время UTC: {}", now);

    // Пример вывода: 2024-01-15 14:30:45.123456789 UTC

    // Для торговых систем важна точность
    println!("Дата: {}", now.format("%Y-%m-%d"));
    println!("Время: {}", now.format("%H:%M:%S"));
    println!("Миллисекунды: {}", now.format("%H:%M:%S%.3f"));
}
```

## Локальное время — для трейдера

```rust
use chrono::{Local, DateTime, Utc};

fn main() {
    // UTC — для хранения и API
    let utc_now: DateTime<Utc> = Utc::now();

    // Local — для отображения трейдеру
    let local_now: DateTime<Local> = Local::now();

    println!("Время биржи (UTC): {}", utc_now.format("%H:%M:%S"));
    println!("Твоё время:        {}", local_now.format("%H:%M:%S"));

    // Конвертация UTC -> Local для отображения
    let trade_time_utc = Utc::now();
    let trade_time_local: DateTime<Local> = trade_time_utc.with_timezone(&Local);

    println!("Сделка в UTC:      {}", trade_time_utc);
    println!("Сделка локально:   {}", trade_time_local);
}
```

## Работа с конкретными таймзонами

```rust
use chrono::{Utc, DateTime, FixedOffset, TimeZone};

fn main() {
    // Таймзоны бирж как смещения от UTC
    let moscow = FixedOffset::east_opt(3 * 3600).unwrap();      // UTC+3
    let new_york = FixedOffset::west_opt(5 * 3600).unwrap();    // UTC-5
    let tokyo = FixedOffset::east_opt(9 * 3600).unwrap();       // UTC+9

    let utc_now = Utc::now();

    println!("UTC:      {}", utc_now.format("%H:%M:%S"));
    println!("Москва:   {}", utc_now.with_timezone(&moscow).format("%H:%M:%S"));
    println!("Нью-Йорк: {}", utc_now.with_timezone(&new_york).format("%H:%M:%S"));
    println!("Токио:    {}", utc_now.with_timezone(&tokyo).format("%H:%M:%S"));
}
```

## Практический пример: Часы работы бирж

```rust
use chrono::{Utc, DateTime, Weekday, Datelike, Timelike, FixedOffset, TimeZone};

struct Exchange {
    name: String,
    timezone: FixedOffset,
    open_hour: u32,
    open_minute: u32,
    close_hour: u32,
    close_minute: u32,
}

impl Exchange {
    fn new(name: &str, utc_offset_hours: i32, open: (u32, u32), close: (u32, u32)) -> Self {
        let offset = if utc_offset_hours >= 0 {
            FixedOffset::east_opt(utc_offset_hours * 3600).unwrap()
        } else {
            FixedOffset::west_opt(-utc_offset_hours * 3600).unwrap()
        };

        Exchange {
            name: name.to_string(),
            timezone: offset,
            open_hour: open.0,
            open_minute: open.1,
            close_hour: close.0,
            close_minute: close.1,
        }
    }

    fn is_open(&self, utc_time: DateTime<Utc>) -> bool {
        let local_time = utc_time.with_timezone(&self.timezone);

        // Проверяем выходные
        match local_time.weekday() {
            Weekday::Sat | Weekday::Sun => return false,
            _ => {}
        }

        let hour = local_time.hour();
        let minute = local_time.minute();
        let current_minutes = hour * 60 + minute;
        let open_minutes = self.open_hour * 60 + self.open_minute;
        let close_minutes = self.close_hour * 60 + self.close_minute;

        current_minutes >= open_minutes && current_minutes < close_minutes
    }

    fn time_until_open(&self, utc_time: DateTime<Utc>) -> Option<String> {
        if self.is_open(utc_time) {
            return None;
        }

        let local_time = utc_time.with_timezone(&self.timezone);
        let current_minutes = local_time.hour() * 60 + local_time.minute();
        let open_minutes = self.open_hour * 60 + self.open_minute;

        if current_minutes < open_minutes {
            let diff = open_minutes - current_minutes;
            Some(format!("{}ч {}мин", diff / 60, diff % 60))
        } else {
            Some("Завтра".to_string())
        }
    }
}

fn main() {
    let exchanges = vec![
        Exchange::new("NYSE", -5, (9, 30), (16, 0)),
        Exchange::new("MOEX", 3, (10, 0), (18, 50)),
        Exchange::new("TSE", 9, (9, 0), (15, 0)),
    ];

    let now = Utc::now();
    println!("Текущее время UTC: {}\n", now.format("%Y-%m-%d %H:%M:%S"));

    for exchange in &exchanges {
        let status = if exchange.is_open(now) {
            "ОТКРЫТА".to_string()
        } else {
            match exchange.time_until_open(now) {
                Some(time) => format!("Закрыта (откроется через {})", time),
                None => "Закрыта".to_string(),
            }
        };
        println!("{}: {}", exchange.name, status);
    }
}
```

## Парсинг времени с таймзоной из API

```rust
use chrono::{DateTime, Utc, FixedOffset, NaiveDateTime, TimeZone};

fn main() {
    // Разные форматы времени от бирж

    // 1. ISO 8601 с таймзоной (частый формат)
    let iso_time = "2024-01-15T14:30:00+03:00";
    let parsed: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(iso_time).unwrap();
    let utc_time: DateTime<Utc> = parsed.with_timezone(&Utc);
    println!("ISO -> UTC: {}", utc_time);

    // 2. Время без таймзоны (нужно знать в какой оно зоне)
    let naive_time = "2024-01-15 14:30:00";
    let naive: NaiveDateTime = NaiveDateTime::parse_from_str(naive_time, "%Y-%m-%d %H:%M:%S").unwrap();

    // Предположим, это время Московской биржи (UTC+3)
    let moscow = FixedOffset::east_opt(3 * 3600).unwrap();
    let moscow_time = moscow.from_local_datetime(&naive).unwrap();
    let utc_from_moscow: DateTime<Utc> = moscow_time.with_timezone(&Utc);
    println!("Москва -> UTC: {}", utc_from_moscow);

    // 3. Время с суффиксом Z (уже UTC)
    let z_time = "2024-01-15T14:30:00Z";
    let parsed_z: DateTime<Utc> = z_time.parse().unwrap();
    println!("Z-формат: {}", parsed_z);
}
```

## Хранение сделок в UTC

```rust
use chrono::{DateTime, Utc, Local};

#[derive(Debug)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: DateTime<Utc>,  // Всегда храним в UTC!
}

impl Trade {
    fn new(id: u64, symbol: &str, price: f64, quantity: f64) -> Self {
        Trade {
            id,
            symbol: symbol.to_string(),
            price,
            quantity,
            timestamp: Utc::now(),
        }
    }

    fn display_local_time(&self) -> String {
        let local: DateTime<Local> = self.timestamp.with_timezone(&Local);
        local.format("%Y-%m-%d %H:%M:%S %Z").to_string()
    }

    fn display_utc_time(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }
}

fn main() {
    let trade = Trade::new(1, "BTC/USDT", 42500.0, 0.5);

    println!("Сделка #{}", trade.id);
    println!("  Символ: {}", trade.symbol);
    println!("  Цена: ${:.2}", trade.price);
    println!("  Объём: {}", trade.quantity);
    println!("  Время (UTC):    {}", trade.display_utc_time());
    println!("  Время (local):  {}", trade.display_local_time());
}
```

## Сравнение времени сделок

```rust
use chrono::{DateTime, Utc, Duration};

fn main() {
    let trade1_time: DateTime<Utc> = "2024-01-15T14:30:00Z".parse().unwrap();
    let trade2_time: DateTime<Utc> = "2024-01-15T14:30:05Z".parse().unwrap();

    // Сравнение
    if trade1_time < trade2_time {
        println!("Сделка 1 была раньше сделки 2");
    }

    // Разница во времени
    let diff: Duration = trade2_time - trade1_time;
    println!("Между сделками прошло: {} секунд", diff.num_seconds());

    // Проверка: была ли сделка недавно?
    let now = Utc::now();
    let one_hour_ago = now - Duration::hours(1);

    if trade1_time > one_hour_ago {
        println!("Сделка была в течение последнего часа");
    } else {
        println!("Сделка была более часа назад");
    }
}
```

## Фильтрация сделок по времени

```rust
use chrono::{DateTime, Utc, Duration, Timelike};

struct Trade {
    symbol: String,
    price: f64,
    timestamp: DateTime<Utc>,
}

fn filter_by_hour(trades: &[Trade], start_hour: u32, end_hour: u32) -> Vec<&Trade> {
    trades
        .iter()
        .filter(|t| {
            let hour = t.timestamp.hour();
            hour >= start_hour && hour < end_hour
        })
        .collect()
}

fn filter_last_n_minutes(trades: &[Trade], minutes: i64) -> Vec<&Trade> {
    let cutoff = Utc::now() - Duration::minutes(minutes);
    trades
        .iter()
        .filter(|t| t.timestamp > cutoff)
        .collect()
}

fn main() {
    let now = Utc::now();

    let trades = vec![
        Trade {
            symbol: "BTC".to_string(),
            price: 42000.0,
            timestamp: now - Duration::minutes(5)
        },
        Trade {
            symbol: "ETH".to_string(),
            price: 2500.0,
            timestamp: now - Duration::minutes(30)
        },
        Trade {
            symbol: "BTC".to_string(),
            price: 42100.0,
            timestamp: now - Duration::minutes(120)
        },
    ];

    // Сделки за последние 15 минут
    let recent = filter_last_n_minutes(&trades, 15);
    println!("Сделок за последние 15 минут: {}", recent.len());

    for trade in recent {
        println!("  {} @ ${:.2}", trade.symbol, trade.price);
    }
}
```

## Работа с торговыми сессиями

```rust
use chrono::{DateTime, Utc, FixedOffset, TimeZone, Timelike, Datelike, Weekday};

#[derive(Debug, Clone, Copy, PartialEq)]
enum TradingSession {
    Asian,      // Токио: 00:00-09:00 UTC
    European,   // Лондон: 08:00-17:00 UTC
    American,   // Нью-Йорк: 13:00-22:00 UTC
    Overlap,    // Пересечение сессий
    Weekend,    // Выходные
}

fn get_current_session(utc_time: DateTime<Utc>) -> TradingSession {
    match utc_time.weekday() {
        Weekday::Sat | Weekday::Sun => return TradingSession::Weekend,
        _ => {}
    }

    let hour = utc_time.hour();

    match hour {
        // Пересечение Азии и Европы
        8 => TradingSession::Overlap,
        // Пересечение Европы и Америки
        13..=16 => TradingSession::Overlap,
        // Только Азия
        0..=7 => TradingSession::Asian,
        // Только Европа
        9..=12 => TradingSession::European,
        // Только Америка
        17..=21 => TradingSession::American,
        // Ночь / переход
        _ => TradingSession::Asian,
    }
}

fn get_session_volatility_multiplier(session: TradingSession) -> f64 {
    match session {
        TradingSession::Overlap => 1.5,   // Высокая волатильность
        TradingSession::American => 1.2,  // Повышенная
        TradingSession::European => 1.1,  // Умеренная
        TradingSession::Asian => 0.8,     // Низкая
        TradingSession::Weekend => 0.5,   // Минимальная (крипто)
    }
}

fn main() {
    let now = Utc::now();
    let session = get_current_session(now);
    let volatility = get_session_volatility_multiplier(session);

    println!("Текущее время UTC: {}", now.format("%H:%M"));
    println!("Торговая сессия: {:?}", session);
    println!("Множитель волатильности: {:.1}x", volatility);

    // Демо для разных часов
    println!("\nСессии в течение дня:");
    for hour in [3, 8, 10, 14, 18, 23] {
        let test_time = Utc::now()
            .with_hour(hour).unwrap()
            .with_minute(0).unwrap();
        let s = get_current_session(test_time);
        println!("  {:02}:00 UTC -> {:?}", hour, s);
    }
}
```

## Конвертация времени для отчётов

```rust
use chrono::{DateTime, Utc, Local, FixedOffset, TimeZone};

struct TradeReport {
    trades: Vec<(DateTime<Utc>, f64)>, // (время, pnl)
}

impl TradeReport {
    fn new() -> Self {
        TradeReport { trades: Vec::new() }
    }

    fn add_trade(&mut self, time: DateTime<Utc>, pnl: f64) {
        self.trades.push((time, pnl));
    }

    fn print_for_timezone(&self, name: &str, offset_hours: i32) {
        let tz = if offset_hours >= 0 {
            FixedOffset::east_opt(offset_hours * 3600).unwrap()
        } else {
            FixedOffset::west_opt(-offset_hours * 3600).unwrap()
        };

        println!("\n=== Отчёт для {} (UTC{:+}) ===", name, offset_hours);

        let mut total = 0.0;
        for (utc_time, pnl) in &self.trades {
            let local_time = utc_time.with_timezone(&tz);
            total += pnl;
            let sign = if *pnl >= 0.0 { "+" } else { "" };
            println!(
                "{} | PnL: {}${:.2}",
                local_time.format("%Y-%m-%d %H:%M:%S"),
                sign,
                pnl
            );
        }
        println!("Итого: ${:.2}", total);
    }
}

fn main() {
    let mut report = TradeReport::new();

    // Добавляем сделки (время в UTC)
    let base_time: DateTime<Utc> = "2024-01-15T14:30:00Z".parse().unwrap();
    report.add_trade(base_time, 150.0);
    report.add_trade(base_time + chrono::Duration::hours(1), -50.0);
    report.add_trade(base_time + chrono::Duration::hours(3), 200.0);

    // Выводим для разных часовых поясов
    report.print_for_timezone("Москва", 3);
    report.print_for_timezone("Нью-Йорк", -5);
    report.print_for_timezone("Лондон", 0);
}
```

## Что мы узнали

| Концепция | Описание | Применение в трейдинге |
|-----------|----------|----------------------|
| `Utc` | Универсальное время | Хранение, API, сравнение |
| `Local` | Локальное время | Отображение пользователю |
| `FixedOffset` | Конкретная таймзона | Биржи разных стран |
| `with_timezone()` | Конвертация | Отчёты, UI |
| Сравнение времён | `<`, `>`, `-` | Сортировка сделок |

## Правила для торговых систем

1. **Храни в UTC** — все timestamps в базе данных только в UTC
2. **Получай в UTC** — от API запрашивай время в UTC
3. **Конвертируй для отображения** — показывай пользователю в его timezone
4. **Сравнивай в UTC** — все вычисления с временем только в UTC
5. **Логируй в UTC** — логи всегда с UTC timestamp

## Домашнее задание

1. Напиши функцию `is_market_open(exchange: &str, utc_time: DateTime<Utc>) -> bool`, которая проверяет, открыта ли конкретная биржа (поддержи NYSE, NASDAQ, MOEX, LSE)

2. Создай структуру `OrderWithTimezone`, которая хранит время создания в UTC, но может отображать его в любой таймзоне

3. Реализуй функцию `group_trades_by_session(trades: &[Trade]) -> HashMap<TradingSession, Vec<&Trade>>`, которая группирует сделки по торговым сессиям

4. Напиши функцию для расчёта статистики: сколько сделок было в каждую торговую сессию за последний месяц

## Навигация

[← Предыдущий день](../135-date-parsing-chrono/ru.md) | [Следующий день →](../137-timestamp-unix-time/ru.md)
