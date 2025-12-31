# День 123: Построчное чтение — парсим CSV вручную

## Аналогия из трейдинга

Представь, что ты получаешь выписку сделок от брокера в формате CSV. Каждая строка — это одна сделка: дата, тикер, цена, объём. Чтобы проанализировать эти данные, нужно прочитать файл строка за строкой и разобрать каждую на части.

CSV (Comma-Separated Values) — это как таблица Excel, сохранённая в текстовом формате. Каждая строка — это строка таблицы, а значения разделены запятыми (или точкой с запятой).

## Формат CSV для торговых данных

Типичный CSV с OHLCV данными:

```
date,open,high,low,close,volume
2024-01-15,42000.50,42500.00,41800.25,42200.75,1500000
2024-01-16,42200.75,42800.00,42100.00,42650.50,1750000
2024-01-17,42650.50,43100.00,42400.00,42900.25,2100000
```

## Чтение файла построчно

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    // Открываем файл
    let file = File::open("trades.csv").expect("Cannot open file");
    let reader = BufReader::new(file);

    // Читаем построчно
    for line in reader.lines() {
        match line {
            Ok(content) => println!("{}", content),
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
}
```

## Парсинг строки CSV

```rust
fn main() {
    let csv_line = "2024-01-15,42000.50,42500.00,41800.25,42200.75,1500000";

    // Разделяем строку по запятой
    let parts: Vec<&str> = csv_line.split(',').collect();

    println!("Date: {}", parts[0]);
    println!("Open: {}", parts[1]);
    println!("High: {}", parts[2]);
    println!("Low: {}", parts[3]);
    println!("Close: {}", parts[4]);
    println!("Volume: {}", parts[5]);
}
```

## Преобразование в числа

```rust
fn main() {
    let csv_line = "2024-01-15,42000.50,42500.00,41800.25,42200.75,1500000";
    let parts: Vec<&str> = csv_line.split(',').collect();

    // Парсим числа
    let open: f64 = parts[1].parse().expect("Invalid open price");
    let high: f64 = parts[2].parse().expect("Invalid high price");
    let low: f64 = parts[3].parse().expect("Invalid low price");
    let close: f64 = parts[4].parse().expect("Invalid close price");
    let volume: u64 = parts[5].parse().expect("Invalid volume");

    println!("OHLCV: O={:.2}, H={:.2}, L={:.2}, C={:.2}, V={}",
        open, high, low, close, volume);

    // Расчёт размера свечи
    let candle_size = high - low;
    println!("Candle size: {:.2}", candle_size);
}
```

## Структура для свечи OHLCV

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Candle {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: u64,
}

impl Candle {
    fn from_csv_line(line: &str) -> Option<Candle> {
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() != 6 {
            return None;
        }

        Some(Candle {
            date: parts[0].to_string(),
            open: parts[1].parse().ok()?,
            high: parts[2].parse().ok()?,
            low: parts[3].parse().ok()?,
            close: parts[4].parse().ok()?,
            volume: parts[5].parse().ok()?,
        })
    }

    fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

fn main() {
    let csv_data = "date,open,high,low,close,volume
2024-01-15,42000.50,42500.00,41800.25,42200.75,1500000
2024-01-16,42200.75,42800.00,42100.00,42650.50,1750000
2024-01-17,42650.50,43100.00,42400.00,42900.25,2100000";

    let mut candles: Vec<Candle> = Vec::new();

    for (i, line) in csv_data.lines().enumerate() {
        // Пропускаем заголовок
        if i == 0 {
            continue;
        }

        if let Some(candle) = Candle::from_csv_line(line) {
            candles.push(candle);
        }
    }

    println!("Loaded {} candles:", candles.len());
    for candle in &candles {
        let trend = if candle.is_bullish() { "BULL" } else { "BEAR" };
        println!("  {} [{}]: O={:.2} C={:.2} Body={:.2}",
            candle.date, trend, candle.open, candle.close, candle.body_size());
    }
}
```

## Обработка ошибок при парсинге

```rust
#[derive(Debug)]
enum ParseError {
    InvalidFormat,
    InvalidNumber(String),
}

struct Trade {
    date: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

impl Trade {
    fn from_csv_line(line: &str) -> Result<Trade, ParseError> {
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() != 5 {
            return Err(ParseError::InvalidFormat);
        }

        let price = parts[3].parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber(parts[3].to_string()))?;

        let quantity = parts[4].parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber(parts[4].to_string()))?;

        Ok(Trade {
            date: parts[0].to_string(),
            symbol: parts[1].to_string(),
            side: parts[2].to_string(),
            price,
            quantity,
        })
    }

    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }
}

fn main() {
    let trades_csv = "date,symbol,side,price,quantity
2024-01-15,BTCUSD,BUY,42000.50,0.5
2024-01-15,ETHUSD,BUY,2200.25,5.0
2024-01-15,BTCUSD,SELL,42500.00,0.3
2024-01-15,INVALID,BUY,not_a_number,1.0";

    println!("=== Parsing trades ===\n");

    for (i, line) in trades_csv.lines().enumerate() {
        if i == 0 {
            continue; // Пропускаем заголовок
        }

        match Trade::from_csv_line(line) {
            Ok(trade) => {
                println!("Trade: {} {} {} @ ${:.2} x {:.4} = ${:.2}",
                    trade.date, trade.side, trade.symbol,
                    trade.price, trade.quantity, trade.total_value());
            }
            Err(ParseError::InvalidFormat) => {
                println!("Error: Invalid format in line {}", i + 1);
            }
            Err(ParseError::InvalidNumber(s)) => {
                println!("Error: Cannot parse '{}' as number in line {}", s, i + 1);
            }
        }
    }
}
```

## Работа с разделителями

```rust
fn main() {
    // Разные разделители
    let comma_csv = "BTCUSD,42000.50,1.5";
    let semicolon_csv = "BTCUSD;42000.50;1.5";
    let tab_csv = "BTCUSD\t42000.50\t1.5";

    // Парсим с запятой
    let parts: Vec<&str> = comma_csv.split(',').collect();
    println!("Comma: {:?}", parts);

    // Парсим с точкой с запятой
    let parts: Vec<&str> = semicolon_csv.split(';').collect();
    println!("Semicolon: {:?}", parts);

    // Парсим с табуляцией
    let parts: Vec<&str> = tab_csv.split('\t').collect();
    println!("Tab: {:?}", parts);
}
```

## Пропуск заголовков и пустых строк

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let csv_content = "
# Comment line
date,open,high,low,close,volume

2024-01-15,42000.50,42500.00,41800.25,42200.75,1500000
2024-01-16,42200.75,42800.00,42100.00,42650.50,1750000

2024-01-17,42650.50,43100.00,42400.00,42900.25,2100000
";

    let mut data_lines = 0;

    for line in csv_content.lines() {
        // Пропускаем пустые строки
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Пропускаем комментарии
        if trimmed.starts_with('#') {
            continue;
        }

        // Пропускаем заголовок
        if trimmed.starts_with("date,") {
            continue;
        }

        println!("Data: {}", trimmed);
        data_lines += 1;
    }

    println!("\nTotal data lines: {}", data_lines);
}
```

## Практический пример: загрузка исторических данных

```rust
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone)]
struct OhlcvCandle {
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn parse_ohlcv_csv(content: &str) -> Vec<OhlcvCandle> {
    let mut candles = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Пропускаем пустые строки и заголовок
        if trimmed.is_empty() || i == 0 {
            continue;
        }

        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() >= 6 {
            if let (Ok(o), Ok(h), Ok(l), Ok(c), Ok(v)) = (
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>(),
                parts[3].parse::<f64>(),
                parts[4].parse::<f64>(),
                parts[5].parse::<f64>(),
            ) {
                candles.push(OhlcvCandle {
                    timestamp: parts[0].to_string(),
                    open: o,
                    high: h,
                    low: l,
                    close: c,
                    volume: v,
                });
            }
        }
    }

    candles
}

fn calculate_sma(candles: &[OhlcvCandle], period: usize) -> Vec<f64> {
    if candles.len() < period {
        return Vec::new();
    }

    let mut sma_values = Vec::new();

    for i in period..=candles.len() {
        let sum: f64 = candles[i - period..i]
            .iter()
            .map(|c| c.close)
            .sum();
        sma_values.push(sum / period as f64);
    }

    sma_values
}

fn find_highest_volume(candles: &[OhlcvCandle]) -> Option<&OhlcvCandle> {
    candles.iter().max_by(|a, b| {
        a.volume.partial_cmp(&b.volume).unwrap()
    })
}

fn main() {
    let csv_data = "timestamp,open,high,low,close,volume
2024-01-10,41500.00,42000.00,41200.00,41800.00,1200000
2024-01-11,41800.00,42200.00,41600.00,42000.00,1350000
2024-01-12,42000.00,42100.00,41400.00,41500.00,980000
2024-01-13,41500.00,42300.00,41400.00,42100.00,1500000
2024-01-14,42100.00,42800.00,42000.00,42600.00,1800000
2024-01-15,42600.00,43000.00,42400.00,42900.00,2100000
2024-01-16,42900.00,43200.00,42700.00,43100.00,1750000";

    let candles = parse_ohlcv_csv(csv_data);

    println!("=== Loaded {} candles ===\n", candles.len());

    // Выводим все свечи
    for candle in &candles {
        let change = candle.close - candle.open;
        let change_pct = (change / candle.open) * 100.0;
        let trend = if change >= 0.0 { "+" } else { "" };

        println!("{}: O={:.2} H={:.2} L={:.2} C={:.2} V={:.0} ({}{:.2}%)",
            candle.timestamp, candle.open, candle.high,
            candle.low, candle.close, candle.volume,
            trend, change_pct);
    }

    // Расчёт SMA-3
    println!("\n=== SMA-3 ===");
    let sma3 = calculate_sma(&candles, 3);
    for (i, value) in sma3.iter().enumerate() {
        println!("SMA-3[{}]: {:.2}", i + 3, value);
    }

    // Свеча с максимальным объёмом
    if let Some(max_vol) = find_highest_volume(&candles) {
        println!("\n=== Highest Volume ===");
        println!("{}: Volume = {:.0}", max_vol.timestamp, max_vol.volume);
    }

    // Общая статистика
    let total_volume: f64 = candles.iter().map(|c| c.volume).sum();
    let avg_volume = total_volume / candles.len() as f64;
    let first_close = candles.first().map(|c| c.close).unwrap_or(0.0);
    let last_close = candles.last().map(|c| c.close).unwrap_or(0.0);
    let total_return = ((last_close - first_close) / first_close) * 100.0;

    println!("\n=== Statistics ===");
    println!("Total volume: {:.0}", total_volume);
    println!("Average volume: {:.0}", avg_volume);
    println!("Total return: {:.2}%", total_return);
}
```

## Практический пример: загрузка портфеля

```rust
#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
    current_price: f64,
}

impl Position {
    fn cost_basis(&self) -> f64 {
        self.quantity * self.avg_price
    }

    fn market_value(&self) -> f64 {
        self.quantity * self.current_price
    }

    fn pnl(&self) -> f64 {
        self.market_value() - self.cost_basis()
    }

    fn pnl_percent(&self) -> f64 {
        if self.cost_basis() == 0.0 {
            return 0.0;
        }
        (self.pnl() / self.cost_basis()) * 100.0
    }
}

fn parse_portfolio(csv: &str) -> Vec<Position> {
    let mut positions = Vec::new();

    for (i, line) in csv.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            if let (Ok(qty), Ok(avg), Ok(curr)) = (
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>(),
                parts[3].parse::<f64>(),
            ) {
                positions.push(Position {
                    symbol: parts[0].to_string(),
                    quantity: qty,
                    avg_price: avg,
                    current_price: curr,
                });
            }
        }
    }

    positions
}

fn main() {
    let portfolio_csv = "symbol,quantity,avg_price,current_price
BTC,0.5,35000.00,42000.00
ETH,5.0,1800.00,2200.00
SOL,100.0,25.00,95.00
DOGE,10000.0,0.08,0.12";

    let positions = parse_portfolio(portfolio_csv);

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    PORTFOLIO SUMMARY                          ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Symbol │   Qty   │  Avg Price │ Curr Price │     PnL    │  %  ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");

    let mut total_cost = 0.0;
    let mut total_value = 0.0;

    for pos in &positions {
        let pnl_sign = if pos.pnl() >= 0.0 { "+" } else { "" };
        println!("║ {:6} │ {:7.2} │ {:10.2} │ {:10.2} │ {:>+10.2} │{:>+5.1}%║",
            pos.symbol, pos.quantity, pos.avg_price,
            pos.current_price, pos.pnl(), pos.pnl_percent());

        total_cost += pos.cost_basis();
        total_value += pos.market_value();
    }

    let total_pnl = total_value - total_cost;
    let total_pnl_pct = if total_cost > 0.0 {
        (total_pnl / total_cost) * 100.0
    } else {
        0.0
    };

    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ TOTAL           │ Cost: {:10.2} │ Value: {:10.2}        ║",
        total_cost, total_value);
    println!("║                 │ PnL: {:>+10.2} ({:>+.2}%)                 ║",
        total_pnl, total_pnl_pct);
    println!("╚═══════════════════════════════════════════════════════════════╝");
}
```

## Экранированные поля и кавычки

В CSV поля со спецсимволами обычно заключены в кавычки:

```rust
fn parse_quoted_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;

    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '"' {
            if in_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                // Экранированная кавычка ""
                current_field.push('"');
                i += 2;
                continue;
            }
            in_quotes = !in_quotes;
        } else if c == ',' && !in_quotes {
            fields.push(current_field.clone());
            current_field.clear();
        } else {
            current_field.push(c);
        }

        i += 1;
    }

    fields.push(current_field);
    fields
}

fn main() {
    let line1 = "BTCUSD,42000.50,\"Bitcoin, USD\"";
    let line2 = "ETHUSD,2200.00,\"Ethereum \"\"Classic\"\"\"";

    println!("Line 1: {:?}", parse_quoted_csv_line(line1));
    println!("Line 2: {:?}", parse_quoted_csv_line(line2));
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `lines()` | Итератор по строкам |
| `split(',')` | Разделение строки |
| `parse::<T>()` | Преобразование в тип |
| `trim()` | Удаление пробелов |
| `starts_with()` | Проверка начала строки |
| `Option` | Для необязательных полей |
| `Result` | Для обработки ошибок |

## Домашнее задание

1. Напиши функцию `load_trades_from_csv(content: &str) -> Result<Vec<Trade>, ParseError>`, которая загружает сделки и обрабатывает все возможные ошибки

2. Создай парсер для CSV с нестандартным разделителем (точка с запятой) и датой в формате DD.MM.YYYY

3. Реализуй функцию, которая читает CSV с OHLCV данными и находит:
   - Самую волатильную свечу (high - low)
   - Дни с объёмом выше среднего
   - Серии последовательных бычьих/медвежьих свечей

4. Напиши парсер для CSV с котировками, где могут быть пропущенные значения (пустые поля), и используй `Option<f64>` для таких случаев

## Навигация

[← Предыдущий день](../122-writing-file-trades/ru.md) | [Следующий день →](../124-bufreader-bufwriter/ru.md)
