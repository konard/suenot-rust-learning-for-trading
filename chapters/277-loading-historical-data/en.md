# Day 277: Loading Historical Data

## Trading Analogy

Before a pilot flies a new aircraft, they spend hours in a flight simulator. Similarly, before a trader deploys a strategy with real money, they need to test it on historical market data. This process is called **backtesting** — replaying past market conditions to see how a strategy would have performed.

Imagine you have a brilliant idea: "Buy when the price drops 5% and sell when it rises 3%." Before risking your savings, you'd want to know: how would this strategy have performed during the 2020 crash? During the 2021 bull run? Loading historical data is the first step in answering these questions.

In this chapter, we'll learn how to:
- Load market data from various file formats (CSV, JSON)
- Parse and validate OHLCV (Open, High, Low, Close, Volume) data
- Handle timestamps and time zones
- Build robust data loading pipelines for backtesting

## What is OHLCV Data?

OHLCV is the standard format for representing price data over a time period (candle):

| Field  | Description                                      |
|--------|--------------------------------------------------|
| Open   | First price at the start of the period           |
| High   | Highest price during the period                  |
| Low    | Lowest price during the period                   |
| Close  | Last price at the end of the period              |
| Volume | Total amount traded during the period            |

```rust
use chrono::{DateTime, Utc};

/// Represents a single OHLCV candle (price bar)
#[derive(Debug, Clone, PartialEq)]
pub struct Candle {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn new(
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Self, String> {
        // Validate OHLCV data
        if high < low {
            return Err("High price cannot be less than low price".to_string());
        }
        if high < open || high < close {
            return Err("High must be >= open and close".to_string());
        }
        if low > open || low > close {
            return Err("Low must be <= open and close".to_string());
        }
        if volume < 0.0 {
            return Err("Volume cannot be negative".to_string());
        }

        Ok(Candle {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        })
    }

    /// Calculate the candle body size (absolute difference between open and close)
    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Calculate the full range (high - low)
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Check if this is a bullish (green) candle
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Check if this is a bearish (red) candle
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }
}
```

## Loading Data from CSV

CSV (Comma-Separated Values) is the most common format for historical market data. Let's build a robust CSV loader:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use chrono::{DateTime, TimeZone, Utc};

/// Errors that can occur during data loading
#[derive(Debug)]
pub enum DataLoadError {
    FileNotFound(String),
    IoError(std::io::Error),
    ParseError { line: usize, message: String },
    ValidationError { line: usize, message: String },
    EmptyFile,
}

impl std::fmt::Display for DataLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataLoadError::FileNotFound(path) => write!(f, "File not found: {}", path),
            DataLoadError::IoError(e) => write!(f, "IO error: {}", e),
            DataLoadError::ParseError { line, message } => {
                write!(f, "Parse error at line {}: {}", line, message)
            }
            DataLoadError::ValidationError { line, message } => {
                write!(f, "Validation error at line {}: {}", line, message)
            }
            DataLoadError::EmptyFile => write!(f, "File contains no data"),
        }
    }
}

impl std::error::Error for DataLoadError {}

/// Load OHLCV data from a CSV file
/// Expected format: timestamp,open,high,low,close,volume
pub fn load_csv(path: &Path) -> Result<Vec<Candle>, DataLoadError> {
    if !path.exists() {
        return Err(DataLoadError::FileNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    let file = File::open(path).map_err(DataLoadError::IoError)?;
    let reader = BufReader::new(file);
    let mut candles = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(DataLoadError::IoError)?;

        // Skip header line
        if line_num == 0 && line.to_lowercase().contains("timestamp") {
            continue;
        }

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        let candle = parse_csv_line(&line, line_num + 1)?;
        candles.push(candle);
    }

    if candles.is_empty() {
        return Err(DataLoadError::EmptyFile);
    }

    // Sort by timestamp to ensure chronological order
    candles.sort_by_key(|c| c.timestamp);

    Ok(candles)
}

fn parse_csv_line(line: &str, line_num: usize) -> Result<Candle, DataLoadError> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if parts.len() < 6 {
        return Err(DataLoadError::ParseError {
            line: line_num,
            message: format!("Expected 6 fields, found {}", parts.len()),
        });
    }

    // Parse timestamp (supports Unix timestamp or ISO format)
    let timestamp = parse_timestamp(parts[0]).map_err(|e| DataLoadError::ParseError {
        line: line_num,
        message: format!("Invalid timestamp: {}", e),
    })?;

    // Parse numeric fields
    let open = parse_f64(parts[1], "open", line_num)?;
    let high = parse_f64(parts[2], "high", line_num)?;
    let low = parse_f64(parts[3], "low", line_num)?;
    let close = parse_f64(parts[4], "close", line_num)?;
    let volume = parse_f64(parts[5], "volume", line_num)?;

    Candle::new(timestamp, open, high, low, close, volume).map_err(|e| {
        DataLoadError::ValidationError {
            line: line_num,
            message: e,
        }
    })
}

fn parse_f64(s: &str, field: &str, line_num: usize) -> Result<f64, DataLoadError> {
    s.parse::<f64>().map_err(|_| DataLoadError::ParseError {
        line: line_num,
        message: format!("Invalid {} value: '{}'", field, s),
    })
}

fn parse_timestamp(s: &str) -> Result<DateTime<Utc>, String> {
    // Try Unix timestamp (seconds)
    if let Ok(ts) = s.parse::<i64>() {
        return Utc
            .timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| "Invalid Unix timestamp".to_string());
    }

    // Try Unix timestamp (milliseconds)
    if let Ok(ts_ms) = s.parse::<i64>() {
        if ts_ms > 1_000_000_000_000 {
            return Utc
                .timestamp_millis_opt(ts_ms)
                .single()
                .ok_or_else(|| "Invalid Unix timestamp (ms)".to_string());
        }
    }

    // Try ISO 8601 format
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            // Try common format: YYYY-MM-DD HH:MM:SS
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| Utc.from_utc_datetime(&ndt))
        })
        .map_err(|e| format!("Cannot parse timestamp '{}': {}", s, e))
}
```

## Loading Data from JSON

Many APIs provide data in JSON format. Here's how to handle it:

```rust
use serde::{Deserialize, Serialize};
use std::fs;

/// JSON representation of OHLCV data (common API format)
#[derive(Debug, Deserialize, Serialize)]
pub struct CandleJson {
    #[serde(alias = "t", alias = "time", alias = "timestamp")]
    pub timestamp: i64,

    #[serde(alias = "o")]
    pub open: f64,

    #[serde(alias = "h")]
    pub high: f64,

    #[serde(alias = "l")]
    pub low: f64,

    #[serde(alias = "c")]
    pub close: f64,

    #[serde(alias = "v", alias = "vol")]
    pub volume: f64,
}

/// Load OHLCV data from a JSON file
pub fn load_json(path: &Path) -> Result<Vec<Candle>, DataLoadError> {
    if !path.exists() {
        return Err(DataLoadError::FileNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    let content = fs::read_to_string(path).map_err(DataLoadError::IoError)?;

    let json_candles: Vec<CandleJson> =
        serde_json::from_str(&content).map_err(|e| DataLoadError::ParseError {
            line: 0,
            message: format!("JSON parse error: {}", e),
        })?;

    let mut candles = Vec::with_capacity(json_candles.len());

    for (idx, jc) in json_candles.iter().enumerate() {
        // Detect if timestamp is in seconds or milliseconds
        let timestamp = if jc.timestamp > 1_000_000_000_000 {
            Utc.timestamp_millis_opt(jc.timestamp)
                .single()
                .ok_or_else(|| DataLoadError::ValidationError {
                    line: idx + 1,
                    message: "Invalid timestamp".to_string(),
                })?
        } else {
            Utc.timestamp_opt(jc.timestamp, 0)
                .single()
                .ok_or_else(|| DataLoadError::ValidationError {
                    line: idx + 1,
                    message: "Invalid timestamp".to_string(),
                })?
        };

        let candle = Candle::new(timestamp, jc.open, jc.high, jc.low, jc.close, jc.volume)
            .map_err(|e| DataLoadError::ValidationError {
                line: idx + 1,
                message: e,
            })?;

        candles.push(candle);
    }

    if candles.is_empty() {
        return Err(DataLoadError::EmptyFile);
    }

    candles.sort_by_key(|c| c.timestamp);
    Ok(candles)
}
```

## Building a Universal Data Loader

Let's create a unified interface that handles multiple formats:

```rust
use std::path::Path;

/// Supported data formats
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    Csv,
    Json,
    Auto, // Detect from file extension
}

/// Universal historical data loader
pub struct HistoricalDataLoader {
    format: DataFormat,
}

impl HistoricalDataLoader {
    pub fn new(format: DataFormat) -> Self {
        HistoricalDataLoader { format }
    }

    /// Auto-detect format based on file extension
    pub fn auto() -> Self {
        HistoricalDataLoader {
            format: DataFormat::Auto,
        }
    }

    /// Load data from a file
    pub fn load(&self, path: &Path) -> Result<Vec<Candle>, DataLoadError> {
        let format = match self.format {
            DataFormat::Auto => Self::detect_format(path)?,
            other => other,
        };

        match format {
            DataFormat::Csv => load_csv(path),
            DataFormat::Json => load_json(path),
            DataFormat::Auto => unreachable!(),
        }
    }

    /// Load data from multiple files and merge
    pub fn load_multiple(&self, paths: &[&Path]) -> Result<Vec<Candle>, DataLoadError> {
        let mut all_candles = Vec::new();

        for path in paths {
            let candles = self.load(path)?;
            all_candles.extend(candles);
        }

        // Sort and remove duplicates
        all_candles.sort_by_key(|c| c.timestamp);
        all_candles.dedup_by_key(|c| c.timestamp);

        Ok(all_candles)
    }

    fn detect_format(path: &Path) -> Result<DataFormat, DataLoadError> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match extension.as_deref() {
            Some("csv") => Ok(DataFormat::Csv),
            Some("json") => Ok(DataFormat::Json),
            _ => Err(DataLoadError::ParseError {
                line: 0,
                message: "Cannot detect file format. Use .csv or .json extension".to_string(),
            }),
        }
    }
}
```

## Data Validation and Cleaning

Real-world data often contains errors. Let's add validation:

```rust
/// Statistics about loaded data quality
#[derive(Debug, Default)]
pub struct DataQualityReport {
    pub total_records: usize,
    pub valid_records: usize,
    pub gaps_detected: usize,
    pub duplicates_removed: usize,
    pub outliers_detected: usize,
}

/// Validate and clean historical data
pub fn validate_and_clean(
    candles: Vec<Candle>,
    expected_interval_secs: i64,
) -> (Vec<Candle>, DataQualityReport) {
    let mut report = DataQualityReport {
        total_records: candles.len(),
        ..Default::default()
    };

    let mut cleaned: Vec<Candle> = Vec::with_capacity(candles.len());
    let mut prev_timestamp: Option<DateTime<Utc>> = None;

    for candle in candles {
        // Check for gaps
        if let Some(prev) = prev_timestamp {
            let gap = candle.timestamp.signed_duration_since(prev).num_seconds();
            if gap > expected_interval_secs * 2 {
                report.gaps_detected += 1;
                println!(
                    "Gap detected: {} seconds between {} and {}",
                    gap, prev, candle.timestamp
                );
            }
        }

        // Check for duplicates
        if Some(candle.timestamp) == prev_timestamp {
            report.duplicates_removed += 1;
            continue;
        }

        // Check for outliers (price change > 50% in one candle)
        if let Some(last) = cleaned.last() {
            let price_change = ((candle.close - last.close) / last.close).abs();
            if price_change > 0.5 {
                report.outliers_detected += 1;
                println!(
                    "Outlier detected at {}: {:.2}% price change",
                    candle.timestamp,
                    price_change * 100.0
                );
            }
        }

        prev_timestamp = Some(candle.timestamp);
        cleaned.push(candle);
        report.valid_records += 1;
    }

    (cleaned, report)
}
```

## Practical Example: Complete Backtesting Data Pipeline

Let's put it all together with a complete example:

```rust
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::path::Path;

/// Represents a trading symbol with its historical data
pub struct SymbolData {
    pub symbol: String,
    pub timeframe: String,
    pub candles: Vec<Candle>,
}

impl SymbolData {
    /// Get candles within a date range
    pub fn get_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&Candle> {
        self.candles
            .iter()
            .filter(|c| c.timestamp >= start && c.timestamp <= end)
            .collect()
    }

    /// Calculate simple statistics
    pub fn statistics(&self) -> DataStatistics {
        if self.candles.is_empty() {
            return DataStatistics::default();
        }

        let prices: Vec<f64> = self.candles.iter().map(|c| c.close).collect();
        let volumes: Vec<f64> = self.candles.iter().map(|c| c.volume).collect();

        let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;
        let avg_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;

        let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        DataStatistics {
            count: self.candles.len(),
            start_date: self.candles.first().map(|c| c.timestamp),
            end_date: self.candles.last().map(|c| c.timestamp),
            avg_price,
            min_price,
            max_price,
            avg_volume,
        }
    }
}

#[derive(Debug, Default)]
pub struct DataStatistics {
    pub count: usize,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub avg_price: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub avg_volume: f64,
}

/// Example: Load and analyze BTC/USDT data
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample data for demonstration
    let sample_csv = r#"timestamp,open,high,low,close,volume
1609459200,29000.0,29500.0,28800.0,29400.0,1500.5
1609462800,29400.0,30000.0,29300.0,29800.0,2000.0
1609466400,29800.0,30200.0,29600.0,30100.0,1800.0
1609470000,30100.0,30500.0,29900.0,30300.0,2200.0
1609473600,30300.0,30800.0,30100.0,30600.0,1900.0
"#;

    // Write sample data to temp file
    std::fs::write("/tmp/btc_sample.csv", sample_csv)?;

    // Load the data
    let loader = HistoricalDataLoader::auto();
    let candles = loader.load(Path::new("/tmp/btc_sample.csv"))?;

    println!("Loaded {} candles", candles.len());

    // Validate and clean
    let (cleaned, report) = validate_and_clean(candles, 3600); // 1-hour candles
    println!("\nData Quality Report:");
    println!("  Total records: {}", report.total_records);
    println!("  Valid records: {}", report.valid_records);
    println!("  Gaps detected: {}", report.gaps_detected);
    println!("  Duplicates removed: {}", report.duplicates_removed);

    // Create symbol data
    let btc_data = SymbolData {
        symbol: "BTC/USDT".to_string(),
        timeframe: "1h".to_string(),
        candles: cleaned,
    };

    // Print statistics
    let stats = btc_data.statistics();
    println!("\nBTC/USDT Statistics:");
    println!("  Data points: {}", stats.count);
    if let (Some(start), Some(end)) = (stats.start_date, stats.end_date) {
        println!("  Period: {} to {}", start, end);
    }
    println!("  Avg price: ${:.2}", stats.avg_price);
    println!("  Min price: ${:.2}", stats.min_price);
    println!("  Max price: ${:.2}", stats.max_price);
    println!("  Avg volume: {:.2} BTC", stats.avg_volume);

    // Analyze individual candles
    println!("\nCandle Analysis:");
    for (i, candle) in btc_data.candles.iter().enumerate() {
        let trend = if candle.is_bullish() {
            "BULLISH"
        } else if candle.is_bearish() {
            "BEARISH"
        } else {
            "DOJI"
        };

        println!(
            "  Candle {}: {} - Open: ${:.2}, Close: ${:.2}, Range: ${:.2} ({})",
            i + 1,
            candle.timestamp.format("%Y-%m-%d %H:%M"),
            candle.open,
            candle.close,
            candle.range(),
            trend
        );
    }

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| OHLCV | Standard format: Open, High, Low, Close, Volume |
| Candle | A single price bar representing a time period |
| CSV Loading | Parse comma-separated values with error handling |
| JSON Loading | Parse structured data with serde |
| Timestamp Parsing | Handle Unix timestamps and ISO 8601 formats |
| Data Validation | Check for gaps, duplicates, and outliers |
| Data Cleaning | Remove invalid entries, sort chronologically |
| Universal Loader | Support multiple formats with auto-detection |

## Exercises

1. **Add Parquet Support**: Extend the `HistoricalDataLoader` to support Apache Parquet format, which is commonly used for large datasets.

2. **Implement Gap Filling**: Create a function that fills missing candles by interpolating or carrying forward the last known value.

3. **Time Zone Handling**: Modify the loader to accept a source timezone and convert all timestamps to UTC.

4. **Streaming Loader**: Implement an iterator-based loader that processes data line-by-line without loading the entire file into memory.

## Homework

1. **Multi-Symbol Loader**: Create a `MarketDataManager` struct that can:
   - Load data for multiple symbols
   - Align timestamps across symbols
   - Handle different timeframes (1m, 5m, 1h, 1d)
   - Return synchronized data for backtesting

2. **Data Source Abstraction**: Design a trait `DataSource` with implementations for:
   - Local file system
   - HTTP API (mock implementation)
   - In-memory cache
   This allows the backtester to switch between data sources seamlessly.

3. **Candlestick Pattern Detection**: Using the `Candle` struct, implement functions to detect:
   - Doji (open == close, within tolerance)
   - Hammer (long lower wick, small body at top)
   - Engulfing pattern (current candle engulfs previous)
   - Morning/Evening star (three-candle pattern)

4. **Data Quality Dashboard**: Create a comprehensive `analyze_data_quality` function that returns:
   - Missing data percentage
   - Average gap duration
   - Price volatility metrics
   - Volume anomalies
   - Timestamp consistency check

## Navigation

[← Previous day](../276-backtesting-framework-design/en.md) | [Next day →](../278-simulating-order-execution/en.md)
