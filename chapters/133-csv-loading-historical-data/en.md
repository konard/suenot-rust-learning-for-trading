# Day 133: CSV — Loading Historical Data

## Trading Analogy

Imagine you've downloaded historical Bitcoin data from Binance exchange. The file looks something like this:

```
timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.50,42150.00,41980.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
```

This is **CSV** (Comma-Separated Values) — the most popular format for storing tabular data. Each row is one candle (OHLCV), and values are separated by commas.

CSV is used everywhere:
- Exports from TradingView
- Data from CoinGecko, CoinMarketCap
- Historical data from brokers
- Trade reports

## Basic Manual CSV Parsing

Let's start with a simple approach — parsing CSV without external libraries:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    // Create test data
    let csv_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.50,42150.00,41980.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42400.00,42200.00,42350.00,1876.43";

    // Parse lines
    let mut lines = csv_data.lines();

    // Skip header
    let header = lines.next().unwrap();
    println!("Header: {}", header);

    println!("\n=== Candles ===");
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

## OHLCV Candle Structure

Let's create a proper data structure:

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
    /// Parse a CSV line into a candle
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

    /// Price change in percent
    fn change_percent(&self) -> f64 {
        ((self.close - self.open) / self.open) * 100.0
    }

    /// Candle body size
    fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Full candle range
    fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Is the candle bullish
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
        // Skip header
        if i == 0 {
            continue;
        }

        if let Some(candle) = Candle::from_csv_line(line) {
            candles.push(candle);
        }
    }

    println!("Loaded candles: {}\n", candles.len());

    for candle in &candles {
        let icon = if candle.is_bullish() { "🟢" } else { "🔴" };
        println!("{} {} | Change: {:+.2}% | Range: {:.2}",
            icon, candle.timestamp, candle.change_percent(), candle.range());
    }
}
```

## Reading CSV from a File

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

        // Skip header
        if i == 0 {
            continue;
        }

        // Skip empty lines
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
    // Create a test file
    let test_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.50,42150.00,41980.00,42100.00,1234.56
2024-01-01 01:00:00,42100.00,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42400.00,42200.00,42350.00,1876.43
2024-01-01 03:00:00,42350.00,42500.00,42300.00,42450.00,2156.78
2024-01-01 04:00:00,42450.00,42600.00,42400.00,42550.00,1987.65";

    let mut file = File::create("btc_history.csv")?;
    file.write_all(test_data.as_bytes())?;

    // Load data
    let candles = load_candles_from_file("btc_history.csv")?;

    println!("=== BTC History ===");
    println!("Loaded candles: {}\n", candles.len());

    for candle in &candles {
        println!("{}: O={:.2} H={:.2} L={:.2} C={:.2} V={:.2}",
            candle.timestamp, candle.open, candle.high,
            candle.low, candle.close, candle.volume);
    }

    // Clean up test file
    std::fs::remove_file("btc_history.csv")?;

    Ok(())
}
```

## Error Handling During Parsing

Real data often has problems: empty values, incorrect format, missing fields. Let's learn to handle them:

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
                write!(f, "Not enough fields: expected {}, got {}", expected, got)
            }
            ParseError::InvalidNumber { field, value } => {
                write!(f, "Invalid number in field '{}': '{}'", field, value)
            }
            ParseError::EmptyLine => {
                write!(f, "Empty line")
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
        // Skip header
        if line_num == 0 {
            continue;
        }

        match Candle::from_csv_line(line) {
            Ok(candle) => candles.push(candle),
            Err(e) => errors.push((line_num + 1, e)), // +1 for human-readable numbering
        }
    }

    (candles, errors)
}

fn main() {
    // CSV with some errors
    let csv_data = "timestamp,open,high,low,close,volume
2024-01-01 00:00:00,42000.50,42150.00,41980.00,42100.00,1234.56
2024-01-01 01:00:00,invalid,42300.00,42050.00,42250.00,2345.67
2024-01-01 02:00:00,42250.00,42400.00,42200.00,42350.00
2024-01-01 03:00:00,42350.00,42500.00,42300.00,42450.00,2156.78

2024-01-01 04:00:00,42450.00,42600.00,42400.00,42550.00,1987.65";

    let (candles, errors) = load_candles_with_errors(csv_data);

    println!("=== Loading Result ===");
    println!("Successfully loaded: {} candles", candles.len());
    println!("Errors: {}\n", errors.len());

    if !errors.is_empty() {
        println!("=== Errors ===");
        for (line_num, error) in &errors {
            println!("  Line {}: {}", line_num, error);
        }
        println!();
    }

    println!("=== Loaded Candles ===");
    for candle in &candles {
        println!("{}: C={:.2}", candle.timestamp, candle.close);
    }
}
```

## Analyzing Loaded Data

After loading data, you can perform analysis:

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

    // Load candles
    let candles: Vec<Candle> = csv_data
        .lines()
        .skip(1)
        .filter_map(|line| Candle::from_csv_line(line))
        .collect();

    // Market analysis
    let stats = analyze_market(&candles);

    println!("=== BTC Market Analysis ===\n");
    println!("Total candles: {}", stats.total_candles);
    println!("Bullish: {} ({:.1}%)", stats.bullish_candles,
        stats.bullish_candles as f64 / stats.total_candles as f64 * 100.0);
    println!("Bearish: {} ({:.1}%)", stats.bearish_candles,
        stats.bearish_candles as f64 / stats.total_candles as f64 * 100.0);
    println!("\nHighest price: ${:.2}", stats.highest_price);
    println!("Lowest price: ${:.2}", stats.lowest_price);
    println!("Range: ${:.2}", stats.highest_price - stats.lowest_price);
    println!("\nTotal volume: {:.2} BTC", stats.total_volume);
    println!("Average candle range: ${:.2}", stats.average_range);

    // SMA
    let sma3 = calculate_sma(&candles, 3);
    println!("\n=== SMA-3 ===");
    for (i, sma) in sma3.iter().enumerate() {
        println!("  Period {}: ${:.2}", i + 3, sma);
    }
}
```

## Filtering Data

Often you need to filter data by various criteria:

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

    // Filter: only bullish candles
    let bullish: Vec<&Candle> = candles
        .iter()
        .filter(|c| c.is_bullish())
        .collect();

    println!("=== Bullish Candles ===");
    for c in &bullish {
        println!("{}: {:+.2}%", c.timestamp, c.change_percent());
    }

    // Filter: candles with volume above average
    let avg_volume: f64 = candles.iter().map(|c| c.volume).sum::<f64>() / candles.len() as f64;

    let high_volume: Vec<&Candle> = candles
        .iter()
        .filter(|c| c.volume > avg_volume)
        .collect();

    println!("\n=== Candles with Above-Average Volume ({:.2}) ===", avg_volume);
    for c in &high_volume {
        println!("{}: V={:.2}", c.timestamp, c.volume);
    }

    // Filter: candles with big moves (>0.2%)
    let big_moves: Vec<&Candle> = candles
        .iter()
        .filter(|c| c.change_percent().abs() > 0.2)
        .collect();

    println!("\n=== Big Moves (>0.2%) ===");
    for c in &big_moves {
        println!("{}: {:+.2}%", c.timestamp, c.change_percent());
    }
}
```

## Exporting Processed Data to CSV

After analysis, you often need to save the results:

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
        // Calculate SMA-3
        let sma_3 = if i >= 2 {
            let sum: f64 = candles[i-2..=i].iter().map(|c| c.close).sum();
            Some(sum / 3.0)
        } else {
            None
        };

        // Price change
        let change_percent = ((candle.close - candle.open) / candle.open) * 100.0;

        // Generate signal
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

    // Header
    writeln!(file, "timestamp,close,sma_3,change_percent,signal")?;

    // Data
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

    println!("=== Processed Data ===");
    println!("timestamp,close,sma_3,change_percent,signal");
    for p in &processed {
        println!("{}", p.to_csv_line());
    }

    // Save to file
    save_to_csv(&processed, "processed_btc.csv")?;
    println!("\nData saved to processed_btc.csv");

    // Clean up
    std::fs::remove_file("processed_btc.csv")?;

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `line.split(',')` | Split string by comma |
| `value.parse()` | Convert string to number |
| `BufReader` | Efficient line-by-line file reading |
| `filter_map()` | Filter and transform simultaneously |
| `Option`/`Result` | Parsing error handling |
| `writeln!` | Write line to file |

## Practical Exercises

1. **Loading and Validation**: Write a function that loads a CSV file with prices and validates that all candles are valid (high >= low, high >= open, high >= close, low <= open, low <= close).

2. **Pattern Detection**: Load historical data and find all "doji" candles — candles where |open - close| < 0.1% of the price.

3. **Resampling**: Write a function that takes an array of hourly candles and converts them to daily candles (OHLCV for 24 hours).

## Homework

1. Create a program that:
   - Loads a CSV file with historical data
   - Calculates SMA-5, SMA-10, SMA-20
   - Finds SMA crossover points (golden cross / death cross)
   - Saves results to a new CSV file

2. Implement an anomaly detection function:
   - Load data
   - Find candles with volume above 2 standard deviations from the mean
   - Find candles with price change more than 3%
   - Output an anomaly report

3. Create a data merging system:
   - Load data from multiple CSV files (BTC, ETH, SOL)
   - Merge them by timestamp
   - Calculate correlation between assets
   - Save a summary table

4. Write a format converter:
   - Input format: timestamp, price (tick data)
   - Output format: OHLCV candles of specified timeframe (1m, 5m, 1h)

## Navigation

[← Previous day](../132-serde-rename-field-names/en.md) | [Next day →](../134-csv-crate-reading-ohlcv/en.md)
