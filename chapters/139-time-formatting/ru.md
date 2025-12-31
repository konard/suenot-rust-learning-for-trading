# День 139: Форматирование времени

## Аналогия из трейдинга

В трейдинге время — это деньги, и форматирование времени — это как разные способы записи времени сделки. Представь: у тебя есть момент открытия позиции. Ты можешь записать его как `2024-03-15 14:30:00`, как `15 марта 2024, 14:30`, или как `1710510600` (Unix timestamp). Разные ситуации требуют разных форматов: для API биржи нужен один формат, для отчёта клиенту — другой, для базы данных — третий.

## Базовое форматирование с chrono

```rust
use chrono::{DateTime, Utc, Local, NaiveDateTime};

fn main() {
    // Текущее время UTC
    let now_utc: DateTime<Utc> = Utc::now();

    // Стандартный формат RFC 3339 (ISO 8601)
    println!("RFC 3339: {}", now_utc.to_rfc3339());
    // Вывод: 2024-03-15T14:30:00+00:00

    // RFC 2822 (формат email)
    println!("RFC 2822: {}", now_utc.to_rfc2822());
    // Вывод: Fri, 15 Mar 2024 14:30:00 +0000

    // Простой вывод (Display trait)
    println!("Display: {}", now_utc);
    // Вывод: 2024-03-15 14:30:00 UTC
}
```

## Пользовательские форматы с format()

```rust
use chrono::{DateTime, Utc, Local};

fn main() {
    let trade_time: DateTime<Utc> = Utc::now();

    // Дата сделки для отчёта
    println!("Дата сделки: {}", trade_time.format("%Y-%m-%d"));
    // Вывод: 2024-03-15

    // Время сделки
    println!("Время сделки: {}", trade_time.format("%H:%M:%S"));
    // Вывод: 14:30:00

    // Полный формат для лога
    println!("Лог: {}", trade_time.format("%Y-%m-%d %H:%M:%S%.3f"));
    // Вывод: 2024-03-15 14:30:00.123

    // Формат для отчёта клиенту
    println!("Отчёт: {}", trade_time.format("%d.%m.%Y в %H:%M"));
    // Вывод: 15.03.2024 в 14:30
}
```

## Основные спецификаторы формата

| Спецификатор | Описание | Пример |
|-------------|----------|--------|
| `%Y` | Год (4 цифры) | 2024 |
| `%y` | Год (2 цифры) | 24 |
| `%m` | Месяц (01-12) | 03 |
| `%d` | День (01-31) | 15 |
| `%H` | Час 24-часовой (00-23) | 14 |
| `%I` | Час 12-часовой (01-12) | 02 |
| `%M` | Минуты (00-59) | 30 |
| `%S` | Секунды (00-59) | 45 |
| `%p` | AM/PM | PM |
| `%A` | День недели полный | Friday |
| `%a` | День недели короткий | Fri |
| `%B` | Месяц полный | March |
| `%b` | Месяц короткий | Mar |
| `%.3f` | Миллисекунды | .123 |
| `%.6f` | Микросекунды | .123456 |
| `%z` | Часовой пояс +HHMM | +0000 |
| `%Z` | Название часового пояса | UTC |

## Форматирование для разных контекстов трейдинга

```rust
use chrono::{DateTime, Utc, TimeZone};

fn main() {
    let trade_time = Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();

    // Формат для биржевого API (ISO 8601)
    let api_format = trade_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    println!("API: {}", api_format);
    // Вывод: 2024-03-15T14:30:00Z

    // Формат для имени файла с бэкапом
    let filename = format!("trades_{}.csv", trade_time.format("%Y%m%d_%H%M%S"));
    println!("Файл: {}", filename);
    // Вывод: trades_20240315_143000.csv

    // Формат для клиентского отчёта
    let report = trade_time.format("%d %B %Y, %H:%M:%S").to_string();
    println!("Отчёт: {}", report);
    // Вывод: 15 March 2024, 14:30:00

    // Короткий формат для интерфейса
    let ui = trade_time.format("%d.%m %H:%M").to_string();
    println!("UI: {}", ui);
    // Вывод: 15.03 14:30
}
```

## Форматирование журнала сделок

```rust
use chrono::{DateTime, Utc, TimeZone};

struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    executed_at: DateTime<Utc>,
}

impl Trade {
    fn format_for_log(&self) -> String {
        format!(
            "[{}] {} {} {} @ {:.2} x {}",
            self.executed_at.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.symbol,
            self.side,
            self.price,
            self.price,
            self.quantity
        )
    }

    fn format_for_report(&self) -> String {
        format!(
            "Сделка: {} {} {:.4} {} по цене ${:.2}\nВремя: {}",
            self.side,
            self.quantity,
            self.quantity,
            self.symbol,
            self.price,
            self.executed_at.format("%d.%m.%Y %H:%M:%S")
        )
    }

    fn format_short(&self) -> String {
        format!(
            "{} {} {} @ ${:.2}",
            self.executed_at.format("%H:%M:%S"),
            self.side,
            self.symbol,
            self.price
        )
    }
}

fn main() {
    let trade = Trade {
        symbol: "BTC/USDT".to_string(),
        side: "BUY".to_string(),
        price: 42500.50,
        quantity: 0.5,
        executed_at: Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap(),
    };

    println!("Лог:\n{}\n", trade.format_for_log());
    println!("Отчёт:\n{}\n", trade.format_for_report());
    println!("Короткий:\n{}", trade.format_short());
}
```

## Парсинг времени из строки

```rust
use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};

fn main() {
    // Парсинг RFC 3339
    let time_str = "2024-03-15T14:30:00Z";
    let parsed: DateTime<Utc> = DateTime::parse_from_rfc3339(time_str)
        .expect("Неверный формат RFC 3339")
        .with_timezone(&Utc);
    println!("Parsed RFC 3339: {}", parsed);

    // Парсинг пользовательского формата
    let custom_str = "15.03.2024 14:30:00";
    let naive = NaiveDateTime::parse_from_str(custom_str, "%d.%m.%Y %H:%M:%S")
        .expect("Неверный формат даты");
    let datetime = Utc.from_utc_datetime(&naive);
    println!("Parsed custom: {}", datetime);

    // Парсинг формата биржи Binance
    let binance_time = "2024-03-15 14:30:00";
    let binance_parsed = NaiveDateTime::parse_from_str(binance_time, "%Y-%m-%d %H:%M:%S")
        .expect("Неверный формат Binance");
    println!("Binance: {}", Utc.from_utc_datetime(&binance_parsed));
}
```

## Форматирование относительного времени

```rust
use chrono::{DateTime, Utc, Duration, TimeZone};

fn format_relative_time(time: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(time);

    if diff < Duration::seconds(60) {
        format!("{} сек. назад", diff.num_seconds())
    } else if diff < Duration::minutes(60) {
        format!("{} мин. назад", diff.num_minutes())
    } else if diff < Duration::hours(24) {
        format!("{} ч. назад", diff.num_hours())
    } else if diff < Duration::days(7) {
        format!("{} дн. назад", diff.num_days())
    } else {
        time.format("%d.%m.%Y").to_string()
    }
}

fn main() {
    let now = Utc::now();

    // Сделка 30 секунд назад
    let recent = now - Duration::seconds(30);
    println!("Недавняя сделка: {}", format_relative_time(recent));

    // Сделка 2 часа назад
    let earlier = now - Duration::hours(2);
    println!("Ранее: {}", format_relative_time(earlier));

    // Сделка неделю назад
    let old = now - Duration::days(10);
    println!("Старая: {}", format_relative_time(old));
}
```

## Форматирование для свечей (OHLCV)

```rust
use chrono::{DateTime, Utc, TimeZone, Duration};

struct Candle {
    open_time: DateTime<Utc>,
    close_time: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn timeframe_label(&self) -> String {
        let duration = self.close_time.signed_duration_since(self.open_time);

        if duration <= Duration::minutes(1) {
            "1m".to_string()
        } else if duration <= Duration::minutes(5) {
            "5m".to_string()
        } else if duration <= Duration::minutes(15) {
            "15m".to_string()
        } else if duration <= Duration::hours(1) {
            "1h".to_string()
        } else if duration <= Duration::hours(4) {
            "4h".to_string()
        } else if duration <= Duration::days(1) {
            "1d".to_string()
        } else {
            "1w".to_string()
        }
    }

    fn format_for_chart(&self) -> String {
        format!(
            "[{}] {} O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.2}",
            self.timeframe_label(),
            self.open_time.format("%Y-%m-%d %H:%M"),
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume
        )
    }
}

fn main() {
    let candle = Candle {
        open_time: Utc.with_ymd_and_hms(2024, 3, 15, 14, 0, 0).unwrap(),
        close_time: Utc.with_ymd_and_hms(2024, 3, 15, 15, 0, 0).unwrap(),
        open: 42000.0,
        high: 42500.0,
        low: 41800.0,
        close: 42300.0,
        volume: 1500.5,
    };

    println!("{}", candle.format_for_chart());
}
```

## Форматирование торговых сессий

```rust
use chrono::{DateTime, Utc, Weekday, Timelike, Datelike, TimeZone};

struct TradingSession {
    name: String,
    open_hour: u32,
    close_hour: u32,
}

fn format_session_time(time: DateTime<Utc>, session: &TradingSession) -> String {
    let hour = time.hour();
    let is_open = hour >= session.open_hour && hour < session.close_hour;

    let status = if is_open { "ОТКРЫТА" } else { "ЗАКРЫТА" };

    format!(
        "{}: {} ({}:00 - {}:00 UTC) | Текущее время: {}",
        session.name,
        status,
        session.open_hour,
        session.close_hour,
        time.format("%H:%M:%S")
    )
}

fn main() {
    let now = Utc::now();

    let sessions = vec![
        TradingSession {
            name: "Токио".to_string(),
            open_hour: 0,  // 00:00 UTC = 09:00 JST
            close_hour: 9,
        },
        TradingSession {
            name: "Лондон".to_string(),
            open_hour: 8,
            close_hour: 17,
        },
        TradingSession {
            name: "Нью-Йорк".to_string(),
            open_hour: 13,
            close_hour: 22,
        },
    ];

    println!("Статус торговых сессий:\n");
    for session in &sessions {
        println!("{}", format_session_time(now, session));
    }
}
```

## Локализация формата времени

```rust
use chrono::{DateTime, Utc, TimeZone};

fn format_localized(time: DateTime<Utc>, locale: &str) -> String {
    match locale {
        "ru" => {
            let months = [
                "января", "февраля", "марта", "апреля", "мая", "июня",
                "июля", "августа", "сентября", "октября", "ноября", "декабря"
            ];
            let month_idx = time.month0() as usize;
            format!(
                "{} {} {} г., {}",
                time.day(),
                months[month_idx],
                time.year(),
                time.format("%H:%M:%S")
            )
        }
        "en" => time.format("%B %d, %Y at %I:%M:%S %p").to_string(),
        _ => time.to_rfc3339(),
    }
}

fn main() {
    let trade_time = Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();

    println!("Русский: {}", format_localized(trade_time, "ru"));
    println!("English: {}", format_localized(trade_time, "en"));
    println!("Default: {}", format_localized(trade_time, "other"));
}
```

## Практический пример: генератор отчёта сделок

```rust
use chrono::{DateTime, Utc, TimeZone, Duration};

struct Trade {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    pnl: f64,
    executed_at: DateTime<Utc>,
}

struct TradeReport {
    trades: Vec<Trade>,
    generated_at: DateTime<Utc>,
}

impl TradeReport {
    fn new(trades: Vec<Trade>) -> Self {
        TradeReport {
            trades,
            generated_at: Utc::now(),
        }
    }

    fn generate_header(&self) -> String {
        format!(
            "╔══════════════════════════════════════════════════════════════╗\n\
             ║                    ОТЧЁТ О СДЕЛКАХ                           ║\n\
             ║  Сгенерировано: {:^43} ║\n\
             ╠══════════════════════════════════════════════════════════════╣",
            self.generated_at.format("%d.%m.%Y %H:%M:%S UTC")
        )
    }

    fn format_trade(&self, trade: &Trade) -> String {
        let pnl_sign = if trade.pnl >= 0.0 { "+" } else { "" };
        format!(
            "║ #{:04} | {} | {:>8} {:>10} | {:.4} @ ${:>10.2} | {}${:>8.2} ║",
            trade.id,
            trade.executed_at.format("%d.%m %H:%M"),
            trade.side,
            trade.symbol,
            trade.quantity,
            trade.price,
            pnl_sign,
            trade.pnl
        )
    }

    fn generate_summary(&self) -> String {
        let total_pnl: f64 = self.trades.iter().map(|t| t.pnl).sum();
        let winning_trades = self.trades.iter().filter(|t| t.pnl > 0.0).count();
        let win_rate = (winning_trades as f64 / self.trades.len() as f64) * 100.0;

        format!(
            "╠══════════════════════════════════════════════════════════════╣\n\
             ║ Всего сделок: {:>3}    Win Rate: {:>5.1}%    PnL: ${:>12.2} ║\n\
             ╚══════════════════════════════════════════════════════════════╝",
            self.trades.len(),
            win_rate,
            total_pnl
        )
    }

    fn generate(&self) -> String {
        let mut report = self.generate_header();
        report.push('\n');

        for trade in &self.trades {
            report.push_str(&self.format_trade(trade));
            report.push('\n');
        }

        report.push_str(&self.generate_summary());
        report
    }
}

fn main() {
    let base_time = Utc.with_ymd_and_hms(2024, 3, 15, 10, 0, 0).unwrap();

    let trades = vec![
        Trade {
            id: 1,
            symbol: "BTC/USDT".to_string(),
            side: "BUY".to_string(),
            price: 42000.0,
            quantity: 0.5,
            pnl: 250.0,
            executed_at: base_time,
        },
        Trade {
            id: 2,
            symbol: "ETH/USDT".to_string(),
            side: "SELL".to_string(),
            price: 2800.0,
            quantity: 2.0,
            pnl: -50.0,
            executed_at: base_time + Duration::hours(2),
        },
        Trade {
            id: 3,
            symbol: "BTC/USDT".to_string(),
            side: "SELL".to_string(),
            price: 42500.0,
            quantity: 0.5,
            pnl: 250.0,
            executed_at: base_time + Duration::hours(4),
        },
    ];

    let report = TradeReport::new(trades);
    println!("{}", report.generate());
}
```

## Что мы узнали

| Метод | Описание | Пример вывода |
|-------|----------|---------------|
| `to_rfc3339()` | Стандарт ISO 8601 | 2024-03-15T14:30:00+00:00 |
| `to_rfc2822()` | Формат email | Fri, 15 Mar 2024 14:30:00 +0000 |
| `format("%...")` | Пользовательский формат | Любой |
| `parse_from_rfc3339()` | Парсинг ISO 8601 | DateTime |
| `parse_from_str()` | Парсинг пользовательского формата | NaiveDateTime |

## Домашнее задание

1. Напиши функцию `format_trade_time(time: DateTime<Utc>, format_type: &str) -> String`, которая поддерживает форматы: "api", "log", "report", "short"

2. Создай структуру `OrderExecution` с временными метками создания, отправки и исполнения ордера. Добавь метод для форматирования всех этапов

3. Реализуй функцию `parse_exchange_time(time_str: &str, exchange: &str) -> Result<DateTime<Utc>, String>`, которая парсит время в формате разных бирж (Binance, Coinbase, Kraken)

4. Напиши генератор ежедневного отчёта с группировкой сделок по часам и форматированием для вывода в терминал

## Навигация

[← Предыдущий день](../138-duration-time-between-trades/ru.md) | [Следующий день →](../140-toml-trading-bot-config/ru.md)
