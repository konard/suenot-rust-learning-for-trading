# День 133: CSV — загружаем исторические данные

## Аналогия из трейдинга

Представь, что ты скачал исторические данные по Bitcoin с биржи Binance. Файл выглядит примерно так:

```
timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.50,42150.00,41980.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
```

Это **CSV** (Comma-Separated Values) — самый популярный формат для хранения табличных данных. Каждая строка — это одна свеча (OHLCV), а значения разделены запятыми.

CSV используется повсеместно:
- Экспорт из TradingView
- Данные с CoinGecko, CoinMarketCap
- Исторические данные от брокеров
- Отчёты о сделках

## Базовый парсинг CSV вручную

Начнём с простого подхода — парсим CSV без внешних библиотек:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    // Создадим тестовые данные
    let csv_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.50,42150.00,41980.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42400.00,42200.00,42350.00,1876.43";

    // Парсим строки
    let mut lines = csv_data.lines();

    // Пропускаем заголовок
    let header = lines.next().unwrap();
    println!("Заголовок: {}", header);

    println!("\n=== Свечи ===");
    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();

        let timestamp = fields[0];
        let open: f64 = fields[1].parse().unwrap();
        let high: f64 = fields[2].parse().unwrap();
        let low: f64 = fields[3].parse().unwrap();
        let close: f64 = fields[4].parse().unwrap();
        let volume: f64 = fields[5].parse().unwrap();

        let change = ((close - open) / open) * 100.0;
        let direction = if close > open { "🟢" } else { "🔴" };

        println!("{} {} O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.2} ({:+.2}%)",
            direction, timestamp, open, high, low, close, volume, change);
    }
}
```

## Структура для свечи OHLCV

Создадим правильную структуру данных:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    /// Парсит строку CSV в свечу
    fn from_csv_line(line: &str) -> Option<Candle> {
        let fields: Vec<&str> = line.split(',').collect();

        if fields.len() < 6 {
            return None;
        }

        Some(Candle {
            timestamp: fields[0].to_string(),
            open: fields[1].parse().ok()?,
            high: fields[2].parse().ok()?,
            low: fields[3].parse().ok()?,
            close: fields[4].parse().ok()?,
            volume: fields[5].parse().ok()?,
        })
    }

    /// Изменение цены в процентах
    fn change_percent(&self) -> f64 {
        ((self.close - self.open) / self.open) * 100.0
    }

    /// Размер тела свечи
    fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Полный диапазон свечи
    fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Бычья ли свеча
    fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

fn main() {
    let csv_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.50,42150.00,41980.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42100.00,42000.00,42050.00,1876.43";

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

    println!("Загружено свечей: {}\n", candles.len());

    for candle in &candles {
        let icon = if candle.is_bullish() { "🟢" } else { "🔴" };
        println!("{} {} | Изменение: {:+.2}% | Диапазон: {:.2}",
            icon, candle.timestamp, candle.change_percent(), candle.range());
    }
}
```

## Чтение CSV из файла

```rust
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn from_csv_line(line: &str) -> Option<Candle> {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 6 {
            return None;
        }
        Some(Candle {
            timestamp: fields[0].to_string(),
            open: fields[1].parse().ok()?,
            high: fields[2].parse().ok()?,
            low: fields[3].parse().ok()?,
            close: fields[4].parse().ok()?,
            volume: fields[5].parse().ok()?,
        })
    }
}

fn load_candles_from_file(path: &str) -> Result<Vec<Candle>, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut candles = Vec::new();

    for (i, line_result) in reader.lines().enumerate() {
        let line = line_result?;

        // Пропускаем заголовок
        if i == 0 {
            continue;
        }

        // Пропускаем пустые строки
        if line.trim().is_empty() {
            continue;
        }

        if let Some(candle) = Candle::from_csv_line(&line) {
            candles.push(candle);
        }
    }

    Ok(candles)
}

fn main() -> Result<(), std::io::Error> {
    // Создаём тестовый файл
    let test_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.50,42150.00,41980.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42400.00,42200.00,42350.00,1876.43
2024-01-01 03:00:00,42350.00,42500.00,42300.00,42450.00,2156.78
2024-01-01 04:00:00,42450.00,42600.00,42400.00,42550.00,1987.65";

    let mut file = File::create("btc_history.csv")?;
    file.write_all(test_data.as_bytes())?;

    // Загружаем данные
    let candles = load_candles_from_file("btc_history.csv")?;

    println!("=== История BTC ===");
    println!("Загружено свечей: {}\n", candles.len());

    for candle in &candles {
        println!("{}: O={:.2} H={:.2} L={:.2} C={:.2} V={:.2}",
            candle.timestamp, candle.open, candle.high,
            candle.low, candle.close, candle.volume);
    }

    // Удаляем тестовый файл
    std::fs::remove_file("btc_history.csv")?;

    Ok(())
}
```

## Обработка ошибок при парсинге

В реальных данных часто встречаются проблемы: пустые значения, неправильный формат, пропущенные поля. Научимся обрабатывать их:

```rust
use std::fmt;

#[derive(Debug)]
enum ParseError {
    NotEnoughFields { expected: usize, got: usize },
    InvalidNumber { field: String, value: String },
    EmptyLine,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NotEnoughFields { expected, got } => {
                write!(f, "Недостаточно полей: ожидалось {}, получено {}", expected, got)
            }
            ParseError::InvalidNumber { field, value } => {
                write!(f, "Неверное число в поле '{}': '{}'", field, value)
            }
            ParseError::EmptyLine => {
                write!(f, "Пустая строка")
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn from_csv_line(line: &str) -> Result<Candle, ParseError> {
        let line = line.trim();

        if line.is_empty() {
            return Err(ParseError::EmptyLine);
        }

        let fields: Vec<&str> = line.split(',').collect();

        if fields.len() < 6 {
            return Err(ParseError::NotEnoughFields {
                expected: 6,
                got: fields.len(),
            });
        }

        let parse_f64 = |field_name: &str, value: &str| -> Result<f64, ParseError> {
            value.trim().parse().map_err(|_| ParseError::InvalidNumber {
                field: field_name.to_string(),
                value: value.to_string(),
            })
        };

        Ok(Candle {
            timestamp: fields[0].to_string(),
            open: parse_f64("open", fields[1])?,
            high: parse_f64("high", fields[2])?,
            low: parse_f64("low", fields[3])?,
            close: parse_f64("close", fields[4])?,
            volume: parse_f64("volume", fields[5])?,
        })
    }
}

fn load_candles_with_errors(csv_data: &str) -> (Vec<Candle>, Vec<(usize, ParseError)>) {
    let mut candles = Vec::new();
    let mut errors = Vec::new();

    for (line_num, line) in csv_data.lines().enumerate() {
        // Пропускаем заголовок
        if line_num == 0 {
            continue;
        }

        match Candle::from_csv_line(line) {
            Ok(candle) => candles.push(candle),
            Err(e) => errors.push((line_num + 1, e)), // +1 для человеческой нумерации
        }
    }

    (candles, errors)
}

fn main() {
    // CSV с некоторыми ошибками
    let csv_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.50,42150.00,41980.00,42100.00,1234.56
2024-01-01 01:00:00,invalid,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42400.00,42200.00,42350.00
2024-01-01 03:00:00,42350.00,42500.00,42300.00,42450.00,2156.78

2024-01-01 04:00:00,42450.00,42600.00,42400.00,42550.00,1987.65";

    let (candles, errors) = load_candles_with_errors(csv_data);

    println!("=== Результат загрузки ===");
    println!("Успешно загружено: {} свечей", candles.len());
    println!("Ошибок: {}\n", errors.len());

    if !errors.is_empty() {
        println!("=== Ошибки ===");
        for (line_num, error) in &errors {
            println!("  Строка {}: {}", line_num, error);
        }
        println!();
    }

    println!("=== Загруженные свечи ===");
    for candle in &candles {
        println!("{}: C={:.2}", candle.timestamp, candle.close);
    }
}
```

## Анализ загруженных данных

После загрузки данных можно проводить анализ:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn from_csv_line(line: &str) -> Option<Candle> {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 6 { return None; }
        Some(Candle {
            timestamp: fields[0].to_string(),
            open: fields[1].parse().ok()?,
            high: fields[2].parse().ok()?,
            low: fields[3].parse().ok()?,
            close: fields[4].parse().ok()?,
            volume: fields[5].parse().ok()?,
        })
    }

    fn is_bullish(&self) -> bool { self.close > self.open }
}

struct MarketStats {
    total_candles: usize,
    bullish_candles: usize,
    bearish_candles: usize,
    highest_price: f64,
    lowest_price: f64,
    total_volume: f64,
    average_range: f64,
}

fn analyze_market(candles: &[Candle]) -> MarketStats {
    let mut bullish = 0;
    let mut bearish = 0;
    let mut highest = f64::MIN;
    let mut lowest = f64::MAX;
    let mut total_volume = 0.0;
    let mut total_range = 0.0;

    for candle in candles {
        if candle.is_bullish() {
            bullish += 1;
        } else {
            bearish += 1;
        }

        if candle.high > highest {
            highest = candle.high;
        }
        if candle.low < lowest {
            lowest = candle.low;
        }

        total_volume += candle.volume;
        total_range += candle.high - candle.low;
    }

    MarketStats {
        total_candles: candles.len(),
        bullish_candles: bullish,
        bearish_candles: bearish,
        highest_price: highest,
        lowest_price: lowest,
        total_volume,
        average_range: if candles.is_empty() { 0.0 } else { total_range / candles.len() as f64 },
    }
}

fn calculate_sma(candles: &[Candle], period: usize) -> Vec<f64> {
    if candles.len() < period {
        return Vec::new();
    }

    let mut sma_values = Vec::new();

    for i in (period - 1)..candles.len() {
        let sum: f64 = candles[(i + 1 - period)..=i]
            .iter()
            .map(|c| c.close)
            .sum();
        sma_values.push(sum / period as f64);
    }

    sma_values
}

fn main() {
    let csv_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.00,42150.00,41900.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42400.00,42100.00,42150.00,1876.43
2024-01-01 03:00:00,42150.00,42500.00,42100.00,42450.00,2156.78
2024-01-01 04:00:00,42450.00,42600.00,42300.00,42350.00,1987.65
2024-01-01 05:00:00,42350.00,42550.00,42200.00,42500.00,2234.89
2024-01-01 06:00:00,42500.00,42700.00,42400.00,42650.00,2567.12
2024-01-01 07:00:00,42650.00,42800.00,42500.00,42550.00,2123.45";

    // Загружаем свечи
    let candles: Vec<Candle> = csv_data
        .lines()
        .skip(1)
        .filter_map(|line| Candle::from_csv_line(line))
        .collect();

    // Анализ рынка
    let stats = analyze_market(&candles);

    println!("=== Анализ рынка BTC ===\n");
    println!("Всего свечей: {}", stats.total_candles);
    println!("Бычьих: {} ({:.1}%)", stats.bullish_candles,
        stats.bullish_candles as f64 / stats.total_candles as f64 * 100.0);
    println!("Медвежьих: {} ({:.1}%)", stats.bearish_candles,
        stats.bearish_candles as f64 / stats.total_candles as f64 * 100.0);
    println!("\nМаксимальная цена: ${:.2}", stats.highest_price);
    println!("Минимальная цена: ${:.2}", stats.lowest_price);
    println!("Диапазон: ${:.2}", stats.highest_price - stats.lowest_price);
    println!("\nОбщий объём: {:.2} BTC", stats.total_volume);
    println!("Средний диапазон свечи: ${:.2}", stats.average_range);

    // SMA
    let sma3 = calculate_sma(&candles, 3);
    println!("\n=== SMA-3 ===");
    for (i, sma) in sma3.iter().enumerate() {
        println!("  Период {}: ${:.2}", i + 3, sma);
    }
}
```

## Фильтрация данных

Часто нужно отфильтровать данные по различным критериям:

```rust
#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn from_csv_line(line: &str) -> Option<Candle> {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 6 { return None; }
        Some(Candle {
            timestamp: fields[0].to_string(),
            open: fields[1].parse().ok()?,
            high: fields[2].parse().ok()?,
            low: fields[3].parse().ok()?,
            close: fields[4].parse().ok()?,
            volume: fields[5].parse().ok()?,
        })
    }

    fn change_percent(&self) -> f64 {
        ((self.close - self.open) / self.open) * 100.0
    }

    fn is_bullish(&self) -> bool { self.close > self.open }
}

fn main() {
    let csv_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.00,42150.00,41900.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42400.00,42100.00,42150.00,876.43
2024-01-01 03:00:00,42150.00,42500.00,42100.00,42450.00,3156.78
2024-01-01 04:00:00,42450.00,42600.00,42300.00,42350.00,987.65
2024-01-01 05:00:00,42350.00,42550.00,42200.00,42500.00,2234.89
2024-01-01 06:00:00,42500.00,42700.00,42400.00,42650.00,4567.12
2024-01-01 07:00:00,42650.00,42800.00,42500.00,42550.00,1123.45";

    let candles: Vec<Candle> = csv_data
        .lines()
        .skip(1)
        .filter_map(|line| Candle::from_csv_line(line))
        .collect();

    // Фильтр: только бычьи свечи
    let bullish: Vec<&Candle> = candles
        .iter()
        .filter(|c| c.is_bullish())
        .collect();

    println!("=== Бычьи свечи ===");
    for c in &bullish {
        println!("{}: {:+.2}%", c.timestamp, c.change_percent());
    }

    // Фильтр: свечи с объёмом выше среднего
    let avg_volume: f64 = candles.iter().map(|c| c.volume).sum::<f64>() / candles.len() as f64;

    let high_volume: Vec<&Candle> = candles
        .iter()
        .filter(|c| c.volume > avg_volume)
        .collect();

    println!("\n=== Свечи с объёмом выше среднего ({:.2}) ===", avg_volume);
    for c in &high_volume {
        println!("{}: V={:.2}", c.timestamp, c.volume);
    }

    // Фильтр: свечи с большим движением (>0.5%)
    let big_moves: Vec<&Candle> = candles
        .iter()
        .filter(|c| c.change_percent().abs() > 0.2)
        .collect();

    println!("\n=== Большие движения (>0.2%) ===");
    for c in &big_moves {
        println!("{}: {:+.2}%", c.timestamp, c.change_percent());
    }
}
```

## Экспорт обработанных данных в CSV

После анализа часто нужно сохранить результаты:

```rust
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone)]
struct Candle {
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn from_csv_line(line: &str) -> Option<Candle> {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 6 { return None; }
        Some(Candle {
            timestamp: fields[0].to_string(),
            open: fields[1].parse().ok()?,
            high: fields[2].parse().ok()?,
            low: fields[3].parse().ok()?,
            close: fields[4].parse().ok()?,
            volume: fields[5].parse().ok()?,
        })
    }
}

#[derive(Debug)]
struct ProcessedCandle {
    timestamp: String,
    close: f64,
    sma_3: Option<f64>,
    change_percent: f64,
    signal: String,
}

impl ProcessedCandle {
    fn to_csv_line(&self) -> String {
        format!("{},{:.2},{},{:.4},{}",
            self.timestamp,
            self.close,
            self.sma_3.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            self.change_percent,
            self.signal
        )
    }
}

fn process_candles(candles: &[Candle]) -> Vec<ProcessedCandle> {
    let mut processed = Vec::new();

    for (i, candle) in candles.iter().enumerate() {
        // Рассчитываем SMA-3
        let sma_3 = if i >= 2 {
            let sum: f64 = candles[i-2..=i].iter().map(|c| c.close).sum();
            Some(sum / 3.0)
        } else {
            None
        };

        // Изменение цены
        let change_percent = ((candle.close - candle.open) / candle.open) * 100.0;

        // Генерируем сигнал
        let signal = match sma_3 {
            Some(sma) if candle.close > sma => "BUY".to_string(),
            Some(sma) if candle.close < sma => "SELL".to_string(),
            _ => "HOLD".to_string(),
        };

        processed.push(ProcessedCandle {
            timestamp: candle.timestamp.clone(),
            close: candle.close,
            sma_3,
            change_percent,
            signal,
        });
    }

    processed
}

fn save_to_csv(data: &[ProcessedCandle], path: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    // Заголовок
    writeln!(file, "timestamp,close,sma_3,change_percent,signal")?;

    // Данные
    for item in data {
        writeln!(file, "{}", item.to_csv_line())?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let csv_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.00,42150.00,41900.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42400.00,42100.00,42150.00,1876.43
2024-01-01 03:00:00,42150.00,42500.00,42100.00,42450.00,2156.78
2024-01-01 04:00:00,42450.00,42600.00,42300.00,42350.00,1987.65";

    let candles: Vec<Candle> = csv_data
        .lines()
        .skip(1)
        .filter_map(|line| Candle::from_csv_line(line))
        .collect();

    let processed = process_candles(&candles);

    println!("=== Обработанные данные ===");
    println!("timestamp,close,sma_3,change_percent,signal");
    for p in &processed {
        println!("{}", p.to_csv_line());
    }

    // Сохраняем в файл
    save_to_csv(&processed, "processed_btc.csv")?;
    println!("\nДанные сохранены в processed_btc.csv");

    // Чистим за собой
    std::fs::remove_file("processed_btc.csv")?;

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `line.split(',')` | Разделение строки по запятой |
| `value.parse()` | Преобразование строки в число |
| `BufReader` | Эффективное чтение файла построчно |
| `filter_map()` | Фильтрация и преобразование одновременно |
| `Option`/`Result` | Обработка ошибок парсинга |
| `writeln!` | Запись строки в файл |

## Практические задания

1. **Загрузка и валидация**: Напиши функцию, которая загружает CSV файл с ценами и проверяет, что все свечи валидны (high >= low, high >= open, high >= close, low <= open, low <= close).

2. **Поиск паттернов**: Загрузи исторические данные и найди все "доджи" — свечи, где |open - close| < 0.1% от цены.

3. **Ресемплинг**: Напиши функцию, которая принимает массив часовых свечей и преобразует их в дневные свечи (OHLCV за 24 часа).

## Домашнее задание

1. Создай программу, которая:
   - Загружает CSV файл с историческими данными
   - Рассчитывает SMA-5, SMA-10, SMA-20
   - Находит точки пересечения SMA (golden cross / death cross)
   - Сохраняет результат в новый CSV файл

2. Реализуй функцию обнаружения аномалий:
   - Загрузи данные
   - Найди свечи с объёмом выше 2-х стандартных отклонений от среднего
   - Найди свечи с изменением цены более 3%
   - Выведи отчёт с аномалиями

3. Создай систему объединения данных:
   - Загрузи данные из нескольких CSV файлов (BTC, ETH, SOL)
   - Объедини их по timestamp
   - Рассчитай корреляцию между активами
   - Сохрани сводную таблицу

4. Напиши конвертер форматов:
   - Входной формат: timestamp, price (тиковые данные)
   - Выходной формат: OHLCV свечи заданного таймфрейма (1m, 5m, 1h)

## Навигация

[← Предыдущий день](../132-serde-rename-field-names/ru.md) | [Следующий день →](../134-csv-crate-reading-ohlcv/ru.md)
