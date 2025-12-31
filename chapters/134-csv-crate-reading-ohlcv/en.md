# Day 134: csv Crate: Reading OHLCV

## Trading Analogy

Imagine you're an analyst at a hedge fund. Every day you receive files with historical data from your broker:

```
date,open,high,low,close,volume
2024-01-01,42000.00,42500.00,41800.00,42200.00,15000.50
2024-01-02,42200.00,43000.00,42100.00,42800.00,18500.75
...
```

Parsing such files manually is tedious and error-prone. The `csv` crate is like an experienced assistant that:
- Automatically splits lines into fields
- Converts data into required types (strings to numbers)
- Handles edge cases (empty fields, quotes, escaping)

## Installing csv Crate

Add the dependency to `Cargo.toml`:

```toml
[dependencies]
csv = "1.3"
serde = { version = "1.0", features = ["derive"] }
```

`serde` is needed for automatic CSV deserialization into Rust structures.

## Basic CSV Reading

```rust
use std::error::Error;
use std::fs::File;
use csv::Reader;

fn main() -> Result<(), Box<dyn Error>> {
    // Open CSV file
    let file = File::open("prices.csv")?;
    let mut reader = Reader::from_reader(file);

    // Read headers
    let headers = reader.headers()?;
    println!("Columns: {:?}", headers);

    // Read rows as Vec<String>
    for result in reader.records() {
        let record = result?;
        println!("Row: {:?}", record);
    }

    Ok(())
}
```

## Reading OHLCV into a Struct

The most convenient way is to use `serde` for automatic deserialization:

```rust
use std::error::Error;
use std::fs::File;
use serde::Deserialize;
use csv::Reader;

// OHLCV candle structure
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

        // Calculate candle size
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

## Reading from String (for Tests)

Often you need to test without files:

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
    // CSV data as string
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

    // Find candle with maximum volume
    if let Some(max_vol) = candles.iter().max_by(|a, b|
        a.volume.partial_cmp(&b.volume).unwrap()
    ) {
        println!("Max volume day: {} with {:.0} BTC",
            max_vol.date, max_vol.volume);
    }

    Ok(())
}
```

## Configuring the Parser

CSV files come in different formats:

```rust
use std::error::Error;
use csv::ReaderBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let data = "date;open;high;low;close\n2024-01-15;42000;42500;41800;42200";

    // Custom delimiter (semicolon instead of comma)
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')           // Field delimiter
        .has_headers(true)         // First row is headers
        .flexible(false)           // All rows must have same number of fields
        .trim(csv::Trim::All)      // Trim whitespace
        .from_reader(data.as_bytes());

    for result in reader.records() {
        let record = result?;
        println!("Date: {}, Close: {}", &record[0], &record[4]);
    }

    Ok(())
}
```

## Handling Parse Errors

Real data often contains errors. It's important to handle them correctly:

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

## Practical Example: Loading and Analysis

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

// Statistics for loaded data
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

## Reading Large Files

For large files, use an iterator — don't load everything into memory:

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

    // Process one candle at a time — don't load all into memory
    for result in reader.deserialize() {
        let candle: Candle = result?;

        count += 1;
        total_volume += candle.volume;

        if candle.high > max_price {
            max_price = candle.high;
        }

        // Progress every 10000 records
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

## Working with Different Date Formats

CSV files from different sources use different date formats:

```rust
use std::error::Error;
use serde::Deserialize;
use csv::Reader;

#[derive(Debug, Deserialize)]
struct Candle {
    // Store as string for now, we'll learn to parse dates in the next chapter
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Data with Unix timestamp
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

## Renaming Fields with serde

When CSV column names don't match struct field names:

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
    // CSV with capitalized headers
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

## Optional Fields

Some fields may be missing:

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
    #[serde(default)]  // If missing, use default (0.0 for f64)
    volume: f64,
    #[serde(default)]
    trades: Option<u64>,  // Optional field
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

## What We Learned

| Concept | Description |
|---------|-------------|
| `csv::Reader` | Main type for reading CSV |
| `reader.deserialize()` | Automatic deserialization into struct |
| `#[derive(Deserialize)]` | serde macro for deserialization |
| `ReaderBuilder` | Parser configuration (delimiter, headers) |
| `#[serde(rename)]` | Field renaming |
| `#[serde(default)]` | Default value |

## Homework

1. Create a CSV file with OHLCV data for a week and load it into your program. Calculate:
   - Average closing price
   - Day with maximum volume
   - Total price range (max high - min low)

2. Write a function that filters candles by condition (e.g., only "green" candles where close > open)

3. Implement OHLCV loading from CSV and calculate simple moving average (SMA) for the last N candles

4. Process a CSV file with errors: skip invalid rows, output an error report, and continue working with valid data

## Navigation

[← Day 133: CSV: Loading Historical Data](../133-csv-loading-historical-data/en.md) | [Day 135: Date Parsing: chrono Crate →](../135-date-parsing-chrono-crate/en.md)
