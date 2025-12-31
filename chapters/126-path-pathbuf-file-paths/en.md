# Day 126: Path and PathBuf — File Paths

## Trading Analogy

Imagine you have an office with multiple folders for different documents: historical data in one folder, strategy configurations in another, trade logs in a third. A **file path** is like an exact address for a document: "cabinet → shelf → folder → document".

In trading, we constantly work with files:
- Historical price data (CSV, JSON)
- Trading strategy configurations
- Trade and error logs
- Portfolio state files

`Path` is an immutable reference to a path (like `&str` for strings), while `PathBuf` is an owned path (like `String`).

## Basic Usage of Path and PathBuf

```rust
use std::path::{Path, PathBuf};

fn main() {
    // Path — immutable reference to a path
    let config_path = Path::new("config/strategy.json");
    println!("Config path: {:?}", config_path);

    // PathBuf — owned path, can be modified
    let mut data_path = PathBuf::new();
    data_path.push("data");
    data_path.push("historical");
    data_path.push("btc_usdt.csv");
    println!("Data path: {:?}", data_path);

    // Conversions
    let path_ref: &Path = data_path.as_path();  // PathBuf -> &Path
    let owned: PathBuf = config_path.to_path_buf();  // &Path -> PathBuf
}
```

## Creating Paths for Trading Data

```rust
use std::path::{Path, PathBuf};

fn main() {
    // Path to historical data file
    let historical_data = Path::new("data/historical/btc_usdt_1h.csv");

    // Dynamic path construction
    let ticker = "eth_usdt";
    let timeframe = "4h";

    let mut price_data_path = PathBuf::from("data");
    price_data_path.push("historical");
    price_data_path.push(format!("{}_{}.csv", ticker, timeframe));

    println!("Price data: {:?}", price_data_path);
    // Output: "data/historical/eth_usdt_4h.csv"
}
```

## Path Components

```rust
use std::path::Path;

fn main() {
    let log_path = Path::new("/var/log/trading/trades_2024.log");

    // File name
    if let Some(file_name) = log_path.file_name() {
        println!("File name: {:?}", file_name);  // "trades_2024.log"
    }

    // File extension
    if let Some(extension) = log_path.extension() {
        println!("Extension: {:?}", extension);  // "log"
    }

    // File name without extension
    if let Some(stem) = log_path.file_stem() {
        println!("File stem: {:?}", stem);  // "trades_2024"
    }

    // Parent directory
    if let Some(parent) = log_path.parent() {
        println!("Parent: {:?}", parent);  // "/var/log/trading"
    }
}
```

## Existence and Type Checks

```rust
use std::path::Path;

fn main() {
    let config_path = Path::new("config/strategy.json");
    let data_dir = Path::new("data/historical");

    // Check existence
    if config_path.exists() {
        println!("Config file exists");
    } else {
        println!("Config file not found - using defaults");
    }

    // Check type: file or directory
    if data_dir.is_dir() {
        println!("Data directory exists");
    }

    if config_path.is_file() {
        println!("Config is a file");
    }

    // Absolute or relative path
    println!("Is absolute: {}", config_path.is_absolute());
    println!("Is relative: {}", config_path.is_relative());
}
```

## Working with Paths in a Trading System

```rust
use std::path::{Path, PathBuf};

struct TradingDataManager {
    base_dir: PathBuf,
}

impl TradingDataManager {
    fn new(base_dir: &str) -> Self {
        TradingDataManager {
            base_dir: PathBuf::from(base_dir),
        }
    }

    fn get_historical_data_path(&self, ticker: &str, timeframe: &str) -> PathBuf {
        let mut path = self.base_dir.clone();
        path.push("historical");
        path.push(format!("{}_{}.csv", ticker, timeframe));
        path
    }

    fn get_strategy_config_path(&self, strategy_name: &str) -> PathBuf {
        let mut path = self.base_dir.clone();
        path.push("strategies");
        path.push(format!("{}.json", strategy_name));
        path
    }

    fn get_trade_log_path(&self, date: &str) -> PathBuf {
        let mut path = self.base_dir.clone();
        path.push("logs");
        path.push("trades");
        path.push(format!("trades_{}.log", date));
        path
    }

    fn get_portfolio_snapshot_path(&self, timestamp: u64) -> PathBuf {
        let mut path = self.base_dir.clone();
        path.push("snapshots");
        path.push(format!("portfolio_{}.json", timestamp));
        path
    }
}

fn main() {
    let manager = TradingDataManager::new("/home/trader/trading_system");

    println!("BTC 1h data: {:?}", manager.get_historical_data_path("btc_usdt", "1h"));
    println!("SMA strategy: {:?}", manager.get_strategy_config_path("sma_crossover"));
    println!("Today's trades: {:?}", manager.get_trade_log_path("2024-01-15"));
    println!("Portfolio: {:?}", manager.get_portfolio_snapshot_path(1705312800));
}
```

## Joining Paths with join

```rust
use std::path::{Path, PathBuf};

fn main() {
    let base = Path::new("/data/trading");

    // join creates a new PathBuf
    let historical = base.join("historical");
    let btc_data = base.join("historical").join("btc_usdt_1h.csv");

    println!("Historical dir: {:?}", historical);
    println!("BTC data: {:?}", btc_data);

    // If the second path is absolute, it replaces the first
    let absolute = Path::new("/etc/config.json");
    let result = base.join(absolute);
    println!("Joined absolute: {:?}", result);  // "/etc/config.json"
}
```

## Modifying Paths

```rust
use std::path::PathBuf;

fn main() {
    let mut path = PathBuf::from("data/historical/btc_usdt_1h.csv");

    // Change extension
    path.set_extension("json");
    println!("Changed extension: {:?}", path);  // "data/historical/btc_usdt_1h.json"

    // Change file name
    path.set_file_name("eth_usdt_4h.json");
    println!("Changed file name: {:?}", path);  // "data/historical/eth_usdt_4h.json"

    // Remove last component
    path.pop();
    println!("After pop: {:?}", path);  // "data/historical"

    // Add new component
    path.push("sol_usdt_1d.csv");
    println!("After push: {:?}", path);  // "data/historical/sol_usdt_1d.csv"
}
```

## Iterating Over Path Components

```rust
use std::path::Path;

fn main() {
    let path = Path::new("/data/trading/historical/btc_usdt/2024/01/candles.csv");

    println!("Path components:");
    for component in path.components() {
        println!("  {:?}", component);
    }

    // Extract information from data path
    let parts: Vec<_> = path.components().collect();
    println!("\nTotal components: {}", parts.len());
}

fn parse_data_path(path: &Path) -> Option<DataPathInfo> {
    let components: Vec<_> = path.components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    // Expected format: .../historical/{ticker}/{year}/{month}/{filename}
    if components.len() >= 5 {
        let len = components.len();
        Some(DataPathInfo {
            ticker: components[len - 4].to_string(),
            year: components[len - 3].to_string(),
            month: components[len - 2].to_string(),
            filename: components[len - 1].to_string(),
        })
    } else {
        None
    }
}

struct DataPathInfo {
    ticker: String,
    year: String,
    month: String,
    filename: String,
}
```

## Cross-Platform Paths

```rust
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

fn main() {
    // Path separator depends on the OS
    println!("Path separator: '{}'", MAIN_SEPARATOR);
    // Windows: '\', Unix: '/'

    // Path automatically uses the correct separator
    let path = Path::new("data").join("historical").join("btc.csv");
    println!("Cross-platform path: {:?}", path);

    // Get string from path
    if let Some(path_str) = path.to_str() {
        println!("As string: {}", path_str);
    }

    // When you need a String
    let path_string: String = path.display().to_string();
    println!("Display: {}", path_string);
}
```

## Practical Example: Trading System File Manager

```rust
use std::path::{Path, PathBuf};

struct TradingFileManager {
    root: PathBuf,
}

#[derive(Debug)]
struct FileInfo {
    path: PathBuf,
    file_type: FileType,
    ticker: Option<String>,
    timeframe: Option<String>,
}

#[derive(Debug)]
enum FileType {
    HistoricalData,
    StrategyConfig,
    TradeLog,
    PortfolioSnapshot,
    Unknown,
}

impl TradingFileManager {
    fn new(root: &str) -> Self {
        TradingFileManager {
            root: PathBuf::from(root),
        }
    }

    fn classify_file(&self, path: &Path) -> FileInfo {
        let path_buf = path.to_path_buf();
        let extension = path.extension().and_then(|e| e.to_str());

        // Determine file type by extension and path
        let file_type = match extension {
            Some("csv") => {
                if path.to_string_lossy().contains("historical") {
                    FileType::HistoricalData
                } else {
                    FileType::Unknown
                }
            }
            Some("json") => {
                if path.to_string_lossy().contains("strateg") {
                    FileType::StrategyConfig
                } else if path.to_string_lossy().contains("portfolio") {
                    FileType::PortfolioSnapshot
                } else {
                    FileType::Unknown
                }
            }
            Some("log") => FileType::TradeLog,
            _ => FileType::Unknown,
        };

        // Extract ticker and timeframe from file name
        let (ticker, timeframe) = self.extract_ticker_info(path);

        FileInfo {
            path: path_buf,
            file_type,
            ticker,
            timeframe,
        }
    }

    fn extract_ticker_info(&self, path: &Path) -> (Option<String>, Option<String>) {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            // Format: ticker_timeframe (e.g.: btc_usdt_1h)
            let parts: Vec<&str> = stem.split('_').collect();
            if parts.len() >= 3 {
                let ticker = format!("{}_{}", parts[0], parts[1]);
                let timeframe = parts[2].to_string();
                return (Some(ticker), Some(timeframe));
            }
        }
        (None, None)
    }

    fn create_backup_path(&self, original: &Path) -> PathBuf {
        let mut backup = self.root.join("backups");

        if let Some(file_name) = original.file_name() {
            let timestamp = 1705312800u64; // In reality - current time
            let backup_name = format!("{}_{}", timestamp, file_name.to_string_lossy());
            backup.push(backup_name);
        }

        backup
    }

    fn get_all_data_files(&self, ticker: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let historical_dir = self.root.join("historical");

        // In reality, this would iterate over the directory
        // Demonstrating path construction for different timeframes
        for timeframe in &["1m", "5m", "15m", "1h", "4h", "1d"] {
            files.push(historical_dir.join(format!("{}_{}.csv", ticker, timeframe)));
        }

        files
    }
}

fn main() {
    let manager = TradingFileManager::new("/home/trader/trading");

    // File classification
    let files = vec![
        Path::new("data/historical/btc_usdt_1h.csv"),
        Path::new("config/strategies/sma_crossover.json"),
        Path::new("logs/trades_2024-01-15.log"),
        Path::new("snapshots/portfolio_1705312800.json"),
    ];

    println!("File Classification:");
    println!("════════════════════════════════════════");
    for file in &files {
        let info = manager.classify_file(file);
        println!("Path: {:?}", info.path);
        println!("Type: {:?}", info.file_type);
        if let Some(ticker) = &info.ticker {
            println!("Ticker: {}", ticker);
        }
        if let Some(tf) = &info.timeframe {
            println!("Timeframe: {}", tf);
        }
        println!("────────────────────────────────────────");
    }

    // Create backup paths
    let original = Path::new("config/strategies/sma_crossover.json");
    let backup = manager.create_backup_path(original);
    println!("Backup path: {:?}", backup);

    // Get all files for a ticker
    println!("\nAll BTC/USDT data files:");
    for file in manager.get_all_data_files("btc_usdt") {
        println!("  {:?}", file);
    }
}
```

## Type Conversions

```rust
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

fn main() {
    // String/&str -> Path/PathBuf
    let s = "data/prices.csv";
    let path = Path::new(s);
    let path_buf = PathBuf::from(s);

    // Path/PathBuf -> &str (may fail for non-UTF8 paths)
    if let Some(str_ref) = path.to_str() {
        println!("As &str: {}", str_ref);
    }

    // Path/PathBuf -> String
    let string = path.display().to_string();
    println!("As String: {}", string);

    // PathBuf <-> Path
    let path_ref: &Path = path_buf.as_path();
    let owned: PathBuf = path.to_path_buf();

    // Working with OsStr for file names
    let file_name: &OsStr = path.file_name().unwrap();
    println!("File name (OsStr): {:?}", file_name);
}
```

## Path Patterns

```rust
use std::path::{Path, PathBuf};

// 1. Function accepts &Path — works with both Path and PathBuf
fn process_data_file(path: &Path) -> bool {
    path.exists() && path.extension().map(|e| e == "csv").unwrap_or(false)
}

// 2. Function returns PathBuf — creates a new path
fn create_output_path(input: &Path) -> PathBuf {
    let mut output = input.to_path_buf();
    output.set_extension("processed.csv");
    output
}

// 3. Function modifies PathBuf in place
fn add_timestamp(path: &mut PathBuf, timestamp: u64) {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        path.set_file_name(format!("{}_{}.{}", stem, timestamp, ext));
    }
}

fn main() {
    let csv_path = Path::new("data/btc.csv");
    let txt_path = Path::new("readme.txt");

    println!("Is CSV data file: {}", process_data_file(csv_path));
    println!("Is CSV data file: {}", process_data_file(txt_path));

    let output = create_output_path(csv_path);
    println!("Output: {:?}", output);

    let mut path = PathBuf::from("data/prices.csv");
    add_timestamp(&mut path, 1705312800);
    println!("With timestamp: {:?}", path);
}
```

## What We Learned

| Type | Ownership | Analog | Use Case |
|------|-----------|--------|----------|
| `Path` | Borrowed | `&str` | Immutable operations |
| `PathBuf` | Owned | `String` | Creating and modifying paths |

| Method | Description |
|--------|-------------|
| `new()` | Create Path from string |
| `push()` | Add component to PathBuf |
| `join()` | Join paths (new PathBuf) |
| `parent()` | Parent directory |
| `file_name()` | File name |
| `extension()` | File extension |
| `exists()` | Check existence |
| `is_file()` / `is_dir()` | Check type |

## Homework

1. Write a function `organize_data_files(files: &[PathBuf]) -> HashMap<String, Vec<PathBuf>>` that groups files by ticker

2. Create a `DataPathBuilder` struct with builder methods for creating paths to various types of trading data

3. Implement a function `validate_data_directory(path: &Path) -> Result<DataDirInfo, String>` that validates the structure of a trading data directory

4. Write a function `find_latest_snapshot(snapshots_dir: &Path) -> Option<PathBuf>` that finds the most recent portfolio snapshot file by file name (timestamp in name)

## Navigation

[← Previous day](../125-file-system-basics/en.md) | [Next day →](../127-reading-files/en.md)
