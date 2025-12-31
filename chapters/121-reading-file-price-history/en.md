# Day 121: Reading File — Loading Price History

## Trading Analogy

Imagine you come to the office and need to analyze historical data for the past year. This data is stored in a file on disk — a CSV with prices, JSON with strategy settings, or a text log of trades. Reading a file in programming is like opening a document for analysis: you get access to data that was saved earlier.

## Why Read Files in Trading?

1. **Loading historical data** — backtesting strategies
2. **Configurations** — API key settings, limits
3. **Trade logs** — analyzing past operations
4. **Caching** — local storage of exchange data

## Basics of File Reading in Rust

### Reading Entire File to String

```rust
use std::fs;

fn main() {
    // The simplest way — read the entire file
    match fs::read_to_string("prices.txt") {
        Ok(content) => {
            println!("File contains {} bytes", content.len());
            println!("Content:\n{}", content);
        }
        Err(e) => println!("Error reading file: {}", e),
    }
}
```

### Error Handling When Reading

```rust
use std::fs;
use std::io;

fn main() {
    match load_price_file("btc_prices.txt") {
        Ok(prices) => {
            println!("Loaded {} prices", prices.len());
            if let Some(last) = prices.last() {
                println!("Last price: ${:.2}", last);
            }
        }
        Err(e) => eprintln!("Failed to load prices: {}", e),
    }
}

fn load_price_file(path: &str) -> io::Result<Vec<f64>> {
    let content = fs::read_to_string(path)?;  // ? propagates the error

    let prices: Vec<f64> = content
        .lines()                              // Split into lines
        .filter(|line| !line.is_empty())      // Skip empty lines
        .filter_map(|line| line.parse().ok()) // Parse to f64
        .collect();

    Ok(prices)
}
```

## Reading File Line by Line

For large files, it's better to read line by line to avoid loading the entire file into memory:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    match load_ohlcv_data("btc_1h.csv") {
        Ok(candles) => {
            println!("Loaded {} candles", candles.len());

            // Find the maximum price
            if let Some(max_high) = candles.iter().map(|c| c.high).reduce(f64::max) {
                println!("Historical high: ${:.2}", max_high);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn load_ohlcv_data(path: &str) -> std::io::Result<Vec<Candle>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut candles = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;

        // Skip header
        if line_num == 0 && line.contains("timestamp") {
            continue;
        }

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse CSV: timestamp,open,high,low,close,volume
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 6 {
            if let (Ok(ts), Ok(o), Ok(h), Ok(l), Ok(c), Ok(v)) = (
                parts[0].parse::<u64>(),
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>(),
                parts[3].parse::<f64>(),
                parts[4].parse::<f64>(),
                parts[5].parse::<f64>(),
            ) {
                candles.push(Candle {
                    timestamp: ts,
                    open: o,
                    high: h,
                    low: l,
                    close: c,
                    volume: v,
                });
            }
        }
    }

    Ok(candles)
}
```

## Checking if File Exists

```rust
use std::path::Path;
use std::fs;

fn main() {
    let config_path = "trading_config.json";

    if Path::new(config_path).exists() {
        println!("Configuration found, loading...");
        match fs::read_to_string(config_path) {
            Ok(config) => println!("Config: {}", config),
            Err(e) => eprintln!("Read error: {}", e),
        }
    } else {
        println!("Configuration not found, using defaults");
    }
}
```

## Reading with Path Building

```rust
use std::fs;
use std::path::PathBuf;

fn main() {
    // Build path programmatically
    let mut data_path = PathBuf::from("data");
    data_path.push("historical");
    data_path.push("btc_usdt_2024.csv");

    println!("Reading file: {:?}", data_path);

    match fs::read_to_string(&data_path) {
        Ok(content) => {
            let line_count = content.lines().count();
            println!("File contains {} lines of data", line_count);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Practical Example: Loading Trade History

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    match load_trade_history("trades.csv") {
        Ok(trades) => {
            println!("═══════════════════════════════════════");
            println!("         TRADE HISTORY ANALYSIS        ");
            println!("═══════════════════════════════════════");
            println!("Total trades: {}", trades.len());

            let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
            let winning: Vec<_> = trades.iter().filter(|t| t.pnl > 0.0).collect();
            let losing: Vec<_> = trades.iter().filter(|t| t.pnl < 0.0).collect();

            println!("Winning: {} ({:.1}%)",
                winning.len(),
                winning.len() as f64 / trades.len() as f64 * 100.0
            );
            println!("Losing: {} ({:.1}%)",
                losing.len(),
                losing.len() as f64 / trades.len() as f64 * 100.0
            );
            println!("Total PnL: ${:.2}", total_pnl);

            if !winning.is_empty() {
                let avg_win: f64 = winning.iter().map(|t| t.pnl).sum::<f64>()
                    / winning.len() as f64;
                println!("Average win: ${:.2}", avg_win);
            }

            if !losing.is_empty() {
                let avg_loss: f64 = losing.iter().map(|t| t.pnl).sum::<f64>()
                    / losing.len() as f64;
                println!("Average loss: ${:.2}", avg_loss);
            }
            println!("═══════════════════════════════════════");
        }
        Err(e) => eprintln!("Load error: {}", e),
    }
}

struct Trade {
    symbol: String,
    side: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

fn load_trade_history(path: &str) -> std::io::Result<Vec<Trade>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut trades = Vec::new();

    for (i, line_result) in reader.lines().enumerate() {
        let line = line_result?;

        // Skip header
        if i == 0 {
            continue;
        }

        // symbol,side,entry,exit,qty,pnl
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 6 {
            if let (Ok(entry), Ok(exit), Ok(qty), Ok(pnl)) = (
                parts[2].parse::<f64>(),
                parts[3].parse::<f64>(),
                parts[4].parse::<f64>(),
                parts[5].parse::<f64>(),
            ) {
                trades.push(Trade {
                    symbol: parts[0].to_string(),
                    side: parts[1].to_string(),
                    entry_price: entry,
                    exit_price: exit,
                    quantity: qty,
                    pnl,
                });
            }
        }
    }

    Ok(trades)
}
```

## Reading Binary Data

For efficient price data storage, binary format is sometimes used:

```rust
use std::fs::File;
use std::io::{Read, BufReader};

fn main() {
    match load_binary_prices("prices.bin") {
        Ok(prices) => {
            println!("Loaded {} prices from binary file", prices.len());
            if let Some(last) = prices.last() {
                println!("Last price: ${:.2}", last);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn load_binary_prices(path: &str) -> std::io::Result<Vec<f64>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut prices = Vec::new();

    // Read record count (u32)
    let mut count_bytes = [0u8; 4];
    reader.read_exact(&mut count_bytes)?;
    let count = u32::from_le_bytes(count_bytes) as usize;

    // Read prices (f64 = 8 bytes each)
    for _ in 0..count {
        let mut price_bytes = [0u8; 8];
        reader.read_exact(&mut price_bytes)?;
        let price = f64::from_le_bytes(price_bytes);
        prices.push(price);
    }

    Ok(prices)
}
```

## Working with Different Data Formats

### Loading Configuration from Text File

```rust
use std::fs;
use std::collections::HashMap;

fn main() {
    match load_config("strategy.conf") {
        Ok(config) => {
            println!("Strategy configuration:");
            for (key, value) in &config {
                println!("  {} = {}", key, value);
            }

            // Use values
            if let Some(risk) = config.get("risk_percent") {
                if let Ok(risk_pct) = risk.parse::<f64>() {
                    println!("\nRisk per trade: {}%", risk_pct);
                }
            }
        }
        Err(e) => eprintln!("Config load error: {}", e),
    }
}

fn load_config(path: &str) -> std::io::Result<HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let mut config = HashMap::new();

    for line in content.lines() {
        // Skip comments and empty lines
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key=value
        if let Some((key, value)) = line.split_once('=') {
            config.insert(
                key.trim().to_string(),
                value.trim().to_string()
            );
        }
    }

    Ok(config)
}
```

## Handling Read Errors

```rust
use std::fs;
use std::io::{self, ErrorKind};

fn main() {
    let files = ["prices.csv", "backup_prices.csv", "default_prices.csv"];

    match load_first_available(&files) {
        Ok((filename, prices)) => {
            println!("Loaded from '{}': {} records", filename, prices.len());
        }
        Err(e) => {
            eprintln!("Failed to load data from any file: {}", e);
        }
    }
}

fn load_first_available(paths: &[&str]) -> io::Result<(String, Vec<f64>)> {
    for path in paths {
        match fs::read_to_string(path) {
            Ok(content) => {
                let prices: Vec<f64> = content
                    .lines()
                    .filter_map(|l| l.parse().ok())
                    .collect();
                return Ok((path.to_string(), prices));
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                println!("File '{}' not found, trying next...", path);
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Err(io::Error::new(
        ErrorKind::NotFound,
        "No files found"
    ))
}
```

## What We Learned

| Method | Usage | When to Use |
|--------|-------|-------------|
| `fs::read_to_string` | Entire file to string | Small files |
| `BufReader::lines()` | Line-by-line reading | Large files |
| `Path::exists()` | Check existence | Before reading |
| `PathBuf` | Build paths | Dynamic paths |
| `read_exact` | Binary data | Compact storage |

## Practical Exercises

1. **Price Loader**: Write a function that reads a file with prices (one price per line) and returns `Vec<f64>`. Handle the case of an empty file.

2. **OHLCV Parser**: Create a function to read CSV with columns `date,open,high,low,close,volume`. Skip invalid lines but log their line numbers.

3. **Smart Loader**: Implement a function that tries to load data from multiple sources in order: first from the main file, then from backup, then from archive.

4. **Data Validator**: Write a function that reads a price file and validates the data (prices are positive, no gaps).

## Homework

1. Create a function `load_portfolio(path: &str) -> Result<Portfolio, String>` that loads a portfolio from a file in the format:
   ```
   BTC,0.5,42000.0
   ETH,10.0,2500.0
   SOL,100.0,95.0
   ```

2. Write a function `find_data_files(dir: &str) -> Vec<PathBuf>` that finds all CSV files in a directory.

3. Implement a function `merge_price_files(files: &[&str]) -> Vec<(u64, f64)>` that merges multiple price files (timestamp, price) into one sorted vector.

4. Create a struct `PriceDataLoader` with methods:
   - `new(path: &str)` — create loader
   - `load()` — load data
   - `get_range(start: u64, end: u64)` — get data for a period

## Navigation

[← Previous day](../120-project-robust-api-client/en.md) | [Next day →](../122-writing-file-saving-trades/en.md)
