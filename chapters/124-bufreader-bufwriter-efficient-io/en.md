# Day 124: BufReader and BufWriter — Efficient I/O

## Trading Analogy

Imagine you're a trader analyzing historical data. You have a file with millions of ticks (trades) from the past year. If you read each trade with a separate disk request — it's like calling your broker for each individual trade instead of getting a batch of orders at once.

**BufReader** is like receiving data in batches: instead of one tick at a time, you get 8KB of data into memory at once and then read from that buffer. This is much faster!

**BufWriter** works similarly for writing: instead of writing each line separately to disk, you accumulate data in a buffer and write it all at once.

## Why Does This Matter?

Every disk access is an expensive operation. Compare:

| Approach | Disk Operations | Speed |
|----------|-----------------|-------|
| Without buffer | 1 per byte | Very slow |
| With buffer | 1 per ~8KB of data | Fast |

In trading, where every millisecond can cost money, efficient I/O is critically important.

## Basic Example: Reading Trading Data

```rust
use std::fs::File;
use std::io::{BufReader, BufRead};

fn main() -> std::io::Result<()> {
    // Open file with historical prices
    let file = File::open("prices.csv")?;

    // Wrap in BufReader for efficient reading
    let reader = BufReader::new(file);

    let mut total_volume = 0.0;
    let mut count = 0;

    // Read line by line — each line is buffered
    for line in reader.lines() {
        let line = line?;

        // Skip header
        if line.starts_with("timestamp") {
            continue;
        }

        // Parse: timestamp,price,volume
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            if let Ok(volume) = parts[2].parse::<f64>() {
                total_volume += volume;
                count += 1;
            }
        }
    }

    println!("Processed {} records", count);
    println!("Total volume: {:.2}", total_volume);

    Ok(())
}
```

## BufWriter: Writing a Trade Journal

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

struct Trade {
    timestamp: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn main() -> std::io::Result<()> {
    let file = File::create("trade_log.csv")?;
    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(writer, "timestamp,symbol,side,price,quantity,value")?;

    // Simulate trades
    let trades = vec![
        Trade { timestamp: 1703980800, symbol: "BTC/USDT".to_string(),
                side: "BUY".to_string(), price: 42150.50, quantity: 0.5 },
        Trade { timestamp: 1703980860, symbol: "ETH/USDT".to_string(),
                side: "SELL".to_string(), price: 2250.75, quantity: 2.0 },
        Trade { timestamp: 1703980920, symbol: "BTC/USDT".to_string(),
                side: "SELL".to_string(), price: 42200.00, quantity: 0.5 },
    ];

    for trade in &trades {
        let value = trade.price * trade.quantity;
        writeln!(
            writer,
            "{},{},{},{:.2},{:.4},{:.2}",
            trade.timestamp,
            trade.symbol,
            trade.side,
            trade.price,
            trade.quantity,
            value
        )?;
    }

    // Important: data is guaranteed to be written to disk after flush
    writer.flush()?;

    println!("Wrote {} trades to journal", trades.len());

    Ok(())
}
```

## Configuring Buffer Size

The default buffer size is 8KB. For larger files, you can increase it:

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write};

fn main() -> std::io::Result<()> {
    // 64KB buffer for reading large price files
    let file = File::open("large_price_history.csv")?;
    let reader = BufReader::with_capacity(64 * 1024, file);

    // 32KB buffer for writing reports
    let output = File::create("analysis_report.csv")?;
    let mut writer = BufWriter::with_capacity(32 * 1024, output);

    writeln!(writer, "date,open,high,low,close,volume")?;

    for line in reader.lines() {
        let line = line?;
        // Process and write...
        writeln!(writer, "{}", line)?;
    }

    writer.flush()?;
    Ok(())
}
```

## Practical Example: OHLCV Data Analysis

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write};

#[derive(Debug)]
struct OHLCV {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug)]
struct DailyStats {
    date: String,
    avg_price: f64,
    volatility: f64,
    total_volume: f64,
    price_change_pct: f64,
}

fn parse_ohlcv(line: &str) -> Option<OHLCV> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 6 {
        return None;
    }

    Some(OHLCV {
        timestamp: parts[0].parse().ok()?,
        open: parts[1].parse().ok()?,
        high: parts[2].parse().ok()?,
        low: parts[3].parse().ok()?,
        close: parts[4].parse().ok()?,
        volume: parts[5].parse().ok()?,
    })
}

fn calculate_stats(candles: &[OHLCV]) -> Option<DailyStats> {
    if candles.is_empty() {
        return None;
    }

    let first = candles.first()?;
    let last = candles.last()?;

    let sum_price: f64 = candles.iter().map(|c| (c.high + c.low) / 2.0).sum();
    let avg_price = sum_price / candles.len() as f64;

    let total_volume: f64 = candles.iter().map(|c| c.volume).sum();

    // Simple volatility: (max high - min low) / avg price
    let max_high = candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
    let min_low = candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
    let volatility = (max_high - min_low) / avg_price * 100.0;

    let price_change_pct = (last.close - first.open) / first.open * 100.0;

    Some(DailyStats {
        date: format!("{}", first.timestamp),
        avg_price,
        volatility,
        total_volume,
        price_change_pct,
    })
}

fn main() -> std::io::Result<()> {
    let input_file = File::open("btc_1m_candles.csv")?;
    let reader = BufReader::new(input_file);

    let output_file = File::create("daily_stats.csv")?;
    let mut writer = BufWriter::new(output_file);

    writeln!(writer, "date,avg_price,volatility_pct,total_volume,change_pct")?;

    let mut candles: Vec<OHLCV> = Vec::new();
    let mut current_day: Option<u64> = None;

    for line in reader.lines().skip(1) {  // Skip header
        let line = line?;

        if let Some(candle) = parse_ohlcv(&line) {
            let day = candle.timestamp / 86400;  // Group by days

            match current_day {
                None => {
                    current_day = Some(day);
                    candles.push(candle);
                }
                Some(d) if d == day => {
                    candles.push(candle);
                }
                Some(_) => {
                    // New day — write stats for previous day
                    if let Some(stats) = calculate_stats(&candles) {
                        writeln!(
                            writer,
                            "{},{:.2},{:.2},{:.0},{:.2}",
                            stats.date,
                            stats.avg_price,
                            stats.volatility,
                            stats.total_volume,
                            stats.price_change_pct
                        )?;
                    }

                    candles.clear();
                    candles.push(candle);
                    current_day = Some(day);
                }
            }
        }
    }

    // Write last day
    if let Some(stats) = calculate_stats(&candles) {
        writeln!(
            writer,
            "{},{:.2},{:.2},{:.0},{:.2}",
            stats.date,
            stats.avg_price,
            stats.volatility,
            stats.total_volume,
            stats.price_change_pct
        )?;
    }

    writer.flush()?;
    println!("Analysis complete!");

    Ok(())
}
```

## Reading Binary Data

For maximum performance, trading data is often stored in binary format:

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

#[repr(C, packed)]
struct Tick {
    timestamp: u64,
    price: f64,
    volume: f64,
}

fn write_ticks(filename: &str, ticks: &[Tick]) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for tick in ticks {
        // Write structure as bytes
        let bytes: [u8; 24] = unsafe {
            std::mem::transmute_copy(tick)
        };
        writer.write_all(&bytes)?;
    }

    writer.flush()?;
    Ok(())
}

fn read_ticks(filename: &str) -> std::io::Result<Vec<Tick>> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    let mut ticks = Vec::new();
    let mut buffer = [0u8; 24];

    while reader.read_exact(&mut buffer).is_ok() {
        let tick: Tick = unsafe {
            std::mem::transmute_copy(&buffer)
        };
        ticks.push(tick);
    }

    Ok(ticks)
}

fn main() -> std::io::Result<()> {
    // Create test ticks
    let ticks = vec![
        Tick { timestamp: 1703980800000, price: 42150.50, volume: 1.5 },
        Tick { timestamp: 1703980800100, price: 42151.00, volume: 0.3 },
        Tick { timestamp: 1703980800200, price: 42149.75, volume: 2.1 },
    ];

    write_ticks("ticks.bin", &ticks)?;
    println!("Wrote {} ticks", ticks.len());

    let loaded = read_ticks("ticks.bin")?;
    println!("Loaded {} ticks", loaded.len());

    for tick in &loaded {
        println!(
            "Time: {}, Price: {:.2}, Volume: {:.2}",
            tick.timestamp, tick.price, tick.volume
        );
    }

    Ok(())
}
```

## Streaming Processing of Large Files

When a file doesn't fit in memory:

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write};

struct TradeSummary {
    total_trades: u64,
    total_volume: f64,
    total_value: f64,
    max_price: f64,
    min_price: f64,
}

impl TradeSummary {
    fn new() -> Self {
        TradeSummary {
            total_trades: 0,
            total_volume: 0.0,
            total_value: 0.0,
            max_price: f64::MIN,
            min_price: f64::MAX,
        }
    }

    fn update(&mut self, price: f64, volume: f64) {
        self.total_trades += 1;
        self.total_volume += volume;
        self.total_value += price * volume;
        self.max_price = self.max_price.max(price);
        self.min_price = self.min_price.min(price);
    }

    fn vwap(&self) -> f64 {
        if self.total_volume > 0.0 {
            self.total_value / self.total_volume
        } else {
            0.0
        }
    }
}

fn process_large_file(input: &str, output: &str) -> std::io::Result<()> {
    let file = File::open(input)?;
    let reader = BufReader::with_capacity(128 * 1024, file);  // 128KB buffer

    let out_file = File::create(output)?;
    let mut writer = BufWriter::new(out_file);

    let mut summary = TradeSummary::new();
    let mut processed_lines = 0u64;

    writeln!(writer, "Processing started...")?;

    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() >= 3 {
            if let (Ok(price), Ok(volume)) = (
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>()
            ) {
                summary.update(price, volume);
            }
        }

        processed_lines += 1;

        // Progress report every million lines
        if processed_lines % 1_000_000 == 0 {
            writeln!(
                writer,
                "Processed {} M lines, current VWAP: {:.2}",
                processed_lines / 1_000_000,
                summary.vwap()
            )?;
            writer.flush()?;  // Flush buffer for immediate output
        }
    }

    // Final report
    writeln!(writer, "\n=== FINAL SUMMARY ===")?;
    writeln!(writer, "Total trades: {}", summary.total_trades)?;
    writeln!(writer, "Total volume: {:.4}", summary.total_volume)?;
    writeln!(writer, "VWAP: {:.2}", summary.vwap())?;
    writeln!(writer, "Price range: {:.2} - {:.2}", summary.min_price, summary.max_price)?;

    writer.flush()?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    process_large_file("all_trades_2023.csv", "summary_report.txt")
}
```

## Error Handling in I/O

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead, Write};

fn safe_process_trades(input_path: &str, output_path: &str) -> Result<usize, String> {
    // Open file with error handling
    let input_file = File::open(input_path)
        .map_err(|e| format!("Could not open {}: {}", input_path, e))?;

    let output_file = File::create(output_path)
        .map_err(|e| format!("Could not create {}: {}", output_path, e))?;

    let reader = BufReader::new(input_file);
    let mut writer = BufWriter::new(output_file);

    let mut processed = 0;
    let mut errors = 0;

    for (line_num, line_result) in reader.lines().enumerate() {
        match line_result {
            Ok(line) => {
                // Process line
                if let Err(e) = writeln!(writer, "Processed: {}", line) {
                    return Err(format!("Write error on line {}: {}", line_num, e));
                }
                processed += 1;
            }
            Err(e) => {
                eprintln!("Read error on line {}: {}", line_num, e);
                errors += 1;
                // Continue processing
            }
        }
    }

    writer.flush()
        .map_err(|e| format!("Error flushing buffer: {}", e))?;

    if errors > 0 {
        eprintln!("Warning: {} lines with errors", errors);
    }

    Ok(processed)
}

fn main() {
    match safe_process_trades("trades.csv", "processed.csv") {
        Ok(count) => println!("Successfully processed {} records", count),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Performance Comparison

```rust
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter, BufRead};
use std::time::Instant;

fn benchmark_unbuffered_read(filename: &str) -> std::io::Result<(usize, u128)> {
    let start = Instant::now();
    let mut file = File::open(filename)?;
    let mut buffer = [0u8; 1];
    let mut count = 0;

    while file.read(&mut buffer)? > 0 {
        count += 1;
    }

    Ok((count, start.elapsed().as_millis()))
}

fn benchmark_buffered_read(filename: &str) -> std::io::Result<(usize, u128)> {
    let start = Instant::now();
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        line?;
        count += 1;
    }

    Ok((count, start.elapsed().as_millis()))
}

fn main() -> std::io::Result<()> {
    // Create test file
    {
        let file = File::create("benchmark_data.csv")?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "timestamp,price,volume")?;
        for i in 0..100_000 {
            writeln!(writer, "{},{:.2},{:.4}", 1700000000 + i, 42000.0 + (i as f64 * 0.01), 0.1)?;
        }
        writer.flush()?;
    }

    println!("File reading benchmark:");
    println!("========================");

    let (lines, time) = benchmark_buffered_read("benchmark_data.csv")?;
    println!("With buffer: {} lines in {} ms", lines, time);

    // Note: unbuffered will be very slow for large files
    // Uncomment only for small tests
    // let (bytes, time) = benchmark_unbuffered_read("benchmark_data.csv")?;
    // println!("Without buffer: {} bytes in {} ms", bytes, time);

    Ok(())
}
```

## What We Learned

| Type | Purpose | When to Use |
|------|---------|-------------|
| `BufReader` | Buffered reading | Reading files line by line or in chunks |
| `BufWriter` | Buffered writing | Multiple writes to a file |
| `with_capacity` | Configure buffer size | Large files, optimization |
| `flush()` | Force write | Guarantee data is written to disk |
| `lines()` | Line iterator | Reading text files |

## Practical Exercises

1. **Format Converter**: Write a program that reads a CSV file with prices and converts it to JSON format. Use `BufReader` and `BufWriter`.

2. **Tick Aggregator**: Create a program that reads a file with tick data (timestamp, price, volume) and aggregates it into minute candles (OHLCV).

3. **Pattern Search**: Write a program that finds all trades with volume greater than a given threshold and writes them to a separate file.

4. **File Merger**: Implement a program that reads multiple CSV files with prices from different assets and merges them into one file sorted by time.

## Homework

1. Write a function `calculate_rolling_vwap(input: &str, output: &str, window: usize)` — calculates rolling VWAP and writes the result to a file.

2. Create a `LogRotator` — a structure that automatically creates a new file for writing when the current one exceeds a specified size (important for trade logs).

3. Implement a `TradeFilter` — reads trades from a file and writes only those that match criteria (symbol, min_volume, time_range).

4. Write a benchmark comparing the reading speed of a single file: without buffer, with 8KB buffer, 64KB, 512KB. Find the optimal size for your system.

## Navigation

[← Previous day](../123-file-io-price-history/en.md) | [Next day →](../125-serde-json-api-data/en.md)
