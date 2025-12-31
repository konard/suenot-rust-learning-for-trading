# День 134: csv crate: читаем OHLCV

## Аналогия из трейдинга

Представь, что ты аналитик хедж-фонда. Каждый день ты получаешь файлы с историческими данными от брокера:

```
date,open,high,low,close,volume
2024-01-01,42000.00,42500.00,41800.00,42200.00,15000.50
2024-01-02,42200.00,43000.00,42100.00,42800.00,18500.75
...
```

Руками парсить такие файлы — утомительно и чревато ошибками. Крейт `csv` — это как опытный помощник, который:
- Автоматически разбирает строки на поля
- Преобразует данные в нужные типы (строки в числа)
- Обрабатывает краевые случаи (пустые поля, кавычки, экранирование)

## Установка csv crate

Добавь зависимость в `Cargo.toml`:

```toml
[dependencies]
csv = "1.3"
serde = { version = "1.0", features = ["derive"] }
```

`serde` нужен для автоматической десериализации CSV в структуры Rust.

## Базовое чтение CSV

```rust
use std::error::Error;
use std::fs::File;
use csv::Reader;

fn main() -> Result<(), Box<dyn Error>> {
    // Открываем CSV файл
    let file = File::open("prices.csv")?;
    let mut reader = Reader::from_reader(file);

    // Читаем заголовки
    let headers = reader.headers()?;
    println!("Columns: {:?}", headers);

    // Читаем строки как Vec<String>
    for result in reader.records() {
        let record = result?;
        println!("Row: {:?}", record);
    }

    Ok(())
}
```

## Чтение OHLCV в структуру

Самый удобный способ — использовать `serde` для автоматической десериализации:

```rust
use std::error::Error;
use std::fs::File;
use serde::Deserialize;
use csv::Reader;

// Структура свечи OHLCV
#[derive(Debug, Deserialize)]
struct Candle {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("btc_daily.csv")?;
    let mut reader = Reader::from_reader(file);

    println!("=== BTC Daily OHLCV ===\n");

    for result in reader.deserialize() {
        let candle: Candle = result?;

        // Рассчитываем размер свечи
        let body = (candle.close - candle.open).abs();
        let range = candle.high - candle.low;

        println!("{}: O={:.2} H={:.2} L={:.2} C={:.2} V={:.0}",
            candle.date, candle.open, candle.high,
            candle.low, candle.close, candle.volume);
        println!("  Body: ${:.2}, Range: ${:.2}", body, range);
    }

    Ok(())
}
```

## Чтение из строки (для тестов)

Часто нужно тестировать без файлов:

```rust
use std::error::Error;
use serde::Deserialize;
use csv::Reader;

#[derive(Debug, Deserialize)]
struct Candle {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    // CSV данные как строка
    let data = "\
date,open,high,low,close,volume
2024-01-15,42000.0,42500.0,41800.0,42200.0,15000.0
2024-01-16,42200.0,43100.0,42000.0,42900.0,18000.0
2024-01-17,42900.0,43500.0,42700.0,43200.0,22000.0
";

    let mut reader = Reader::from_reader(data.as_bytes());

    let candles: Vec<Candle> = reader
        .deserialize()
        .collect::<Result<Vec<_>, _>>()?;

    println!("Loaded {} candles", candles.len());

    // Находим свечу с максимальным объёмом
    if let Some(max_vol) = candles.iter().max_by(|a, b|
        a.volume.partial_cmp(&b.volume).unwrap()
    ) {
        println!("Max volume day: {} with {:.0} BTC",
            max_vol.date, max_vol.volume);
    }

    Ok(())
}
```

## Настройка парсера

CSV файлы бывают разных форматов:

```rust
use std::error::Error;
use csv::ReaderBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let data = "date;open;high;low;close\n2024-01-15;42000;42500;41800;42200";

    // Кастомный разделитель (точка с запятой вместо запятой)
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')           // Разделитель полей
        .has_headers(true)         // Первая строка — заголовки
        .flexible(false)           // Все строки должны иметь одинаковое кол-во полей
        .trim(csv::Trim::All)      // Убираем пробелы
        .from_reader(data.as_bytes());

    for result in reader.records() {
        let record = result?;
        println!("Date: {}, Close: {}", &record[0], &record[4]);
    }

    Ok(())
}
```

## Обработка ошибок при парсинге

В реальных данных бывают ошибки. Важно их корректно обрабатывать:

```rust
use std::error::Error;
use serde::Deserialize;
use csv::Reader;

#[derive(Debug, Deserialize)]
struct Candle {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let data = "\
date,open,high,low,close,volume
2024-01-15,42000.0,42500.0,41800.0,42200.0,15000.0
2024-01-16,INVALID,43100.0,42000.0,42900.0,18000.0
2024-01-17,42900.0,43500.0,42700.0,43200.0,22000.0
";

    let mut reader = Reader::from_reader(data.as_bytes());
    let mut valid_candles = Vec::new();
    let mut errors = Vec::new();

    for (line_num, result) in reader.deserialize().enumerate() {
        match result {
            Ok(candle) => valid_candles.push(candle),
            Err(e) => {
                errors.push(format!("Line {}: {}", line_num + 2, e));
            }
        }
    }

    println!("Loaded {} valid candles", valid_candles.len());

    if !errors.is_empty() {
        println!("\nErrors encountered:");
        for err in &errors {
            println!("  - {}", err);
        }
    }

    Ok(())
}
```

## Практический пример: загрузка и анализ

```rust
use std::error::Error;
use serde::Deserialize;
use csv::Reader;

#[derive(Debug, Deserialize, Clone)]
struct Candle {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

// Статистика по загруженным данным
struct OhlcvStats {
    count: usize,
    highest_price: f64,
    lowest_price: f64,
    total_volume: f64,
    avg_range: f64,
}

fn calculate_stats(candles: &[Candle]) -> OhlcvStats {
    let count = candles.len();

    let highest_price = candles.iter()
        .map(|c| c.high)
        .fold(f64::MIN, f64::max);

    let lowest_price = candles.iter()
        .map(|c| c.low)
        .fold(f64::MAX, f64::min);

    let total_volume: f64 = candles.iter()
        .map(|c| c.volume)
        .sum();

    let avg_range: f64 = candles.iter()
        .map(|c| c.high - c.low)
        .sum::<f64>() / count as f64;

    OhlcvStats {
        count,
        highest_price,
        lowest_price,
        total_volume,
        avg_range,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let data = "\
date,open,high,low,close,volume
2024-01-15,42000.0,42500.0,41800.0,42200.0,15000.0
2024-01-16,42200.0,43100.0,42000.0,42900.0,18000.0
2024-01-17,42900.0,43500.0,42700.0,43200.0,22000.0
2024-01-18,43200.0,44000.0,43000.0,43800.0,25000.0
2024-01-19,43800.0,44200.0,43500.0,43600.0,19000.0
";

    let mut reader = Reader::from_reader(data.as_bytes());

    let candles: Vec<Candle> = reader
        .deserialize()
        .collect::<Result<Vec<_>, _>>()?;

    let stats = calculate_stats(&candles);

    println!("=== OHLCV Analysis Report ===\n");
    println!("Period: {} - {}",
        candles.first().map(|c| &c.date[..]).unwrap_or("N/A"),
        candles.last().map(|c| &c.date[..]).unwrap_or("N/A"));
    println!("Candles loaded: {}", stats.count);
    println!("Highest price: ${:.2}", stats.highest_price);
    println!("Lowest price: ${:.2}", stats.lowest_price);
    println!("Price range: ${:.2}", stats.highest_price - stats.lowest_price);
    println!("Total volume: {:.2} BTC", stats.total_volume);
    println!("Avg daily range: ${:.2}", stats.avg_range);

    Ok(())
}
```

## Чтение больших файлов

Для больших файлов используй итератор — не загружай всё в память:

```rust
use std::error::Error;
use std::fs::File;
use serde::Deserialize;
use csv::Reader;

#[derive(Debug, Deserialize)]
struct Candle {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("large_dataset.csv")?;
    let mut reader = Reader::from_reader(file);

    let mut count = 0;
    let mut total_volume = 0.0;
    let mut max_price = f64::MIN;

    // Обрабатываем по одной свече — не загружаем всё в память
    for result in reader.deserialize() {
        let candle: Candle = result?;

        count += 1;
        total_volume += candle.volume;

        if candle.high > max_price {
            max_price = candle.high;
        }

        // Прогресс каждые 10000 записей
        if count % 10000 == 0 {
            println!("Processed {} candles...", count);
        }
    }

    println!("\n=== Summary ===");
    println!("Total candles: {}", count);
    println!("Total volume: {:.2}", total_volume);
    println!("All-time high: ${:.2}", max_price);

    Ok(())
}
```

## Работа с разными форматами дат

CSV файлы от разных источников используют разные форматы дат:

```rust
use std::error::Error;
use serde::Deserialize;
use csv::Reader;

#[derive(Debug, Deserialize)]
struct Candle {
    // Пока храним как строку, в следующей главе научимся парсить
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Данные с Unix timestamp
    let data = "\
timestamp,open,high,low,close,volume
1705276800,42000.0,42500.0,41800.0,42200.0,15000.0
1705363200,42200.0,43100.0,42000.0,42900.0,18000.0
";

    let mut reader = Reader::from_reader(data.as_bytes());

    for result in reader.deserialize() {
        let candle: Candle = result?;
        println!("Timestamp: {}, Close: ${:.2}",
            candle.timestamp, candle.close);
    }

    Ok(())
}
```

## Переименование полей с serde

Если имена колонок в CSV не соответствуют именам полей в структуре:

```rust
use std::error::Error;
use serde::Deserialize;
use csv::Reader;

#[derive(Debug, Deserialize)]
struct Candle {
    #[serde(rename = "Date")]
    date: String,

    #[serde(rename = "Open")]
    open: f64,

    #[serde(rename = "High")]
    high: f64,

    #[serde(rename = "Low")]
    low: f64,

    #[serde(rename = "Close")]
    close: f64,

    #[serde(rename = "Volume")]
    volume: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    // CSV с заглавными буквами в заголовках
    let data = "\
Date,Open,High,Low,Close,Volume
2024-01-15,42000.0,42500.0,41800.0,42200.0,15000.0
";

    let mut reader = Reader::from_reader(data.as_bytes());

    for result in reader.deserialize() {
        let candle: Candle = result?;
        println!("{:?}", candle);
    }

    Ok(())
}
```

## Опциональные поля

Некоторые поля могут отсутствовать:

```rust
use std::error::Error;
use serde::Deserialize;
use csv::Reader;

#[derive(Debug, Deserialize)]
struct Candle {
    date: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    #[serde(default)]  // Если отсутствует, используем default (0.0 для f64)
    volume: f64,
    #[serde(default)]
    trades: Option<u64>,  // Опциональное поле
}

fn main() -> Result<(), Box<dyn Error>> {
    let data = "\
date,open,high,low,close,volume,trades
2024-01-15,42000.0,42500.0,41800.0,42200.0,15000.0,1250
2024-01-16,42200.0,43100.0,42000.0,42900.0,18000.0,
";

    let mut reader = Reader::from_reader(data.as_bytes());

    for result in reader.deserialize() {
        let candle: Candle = result?;
        match candle.trades {
            Some(t) => println!("{}: {} trades", candle.date, t),
            None => println!("{}: trade count unknown", candle.date),
        }
    }

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `csv::Reader` | Основной тип для чтения CSV |
| `reader.deserialize()` | Автоматическая десериализация в структуру |
| `#[derive(Deserialize)]` | Макрос serde для десериализации |
| `ReaderBuilder` | Настройка парсера (разделитель, заголовки) |
| `#[serde(rename)]` | Переименование полей |
| `#[serde(default)]` | Значение по умолчанию |

## Домашнее задание

1. Создай CSV файл с данными OHLCV за неделю и загрузи его в программу. Рассчитай:
   - Среднюю цену закрытия
   - День с максимальным объёмом
   - Общий диапазон цен (max high - min low)

2. Напиши функцию, которая фильтрует свечи по условию (например, только "зелёные" свечи где close > open)

3. Реализуй загрузку OHLCV из CSV и расчёт простой скользящей средней (SMA) для последних N свечей

4. Обработай CSV файл с ошибками: пропусти невалидные строки, выведи отчёт об ошибках и продолжи работу с валидными данными

## Навигация

[← День 133: CSV: загружаем исторические данные](../133-csv-loading-historical-data/ru.md) | [День 135: Парсинг дат: chrono crate →](../135-date-parsing-chrono-crate/ru.md)
