# Day 148: Data Compression — Working with Large Files

## Trading Analogy

Imagine storing a year's worth of tick data — millions of records. Without compression, the file takes 10 GB, but with compression — only 1 GB. It's like packing a huge warehouse into a compact container for shipping. Data compression is a critical technique for traders working with historical data, backtesting, and storing trading operation logs.

## Why Compress Data in Trading

1. **Disk space savings** — years of tick history takes terabytes
2. **Faster network transfer** — compressed data downloads faster from exchanges
3. **Archiving** — old logs and backups in compressed form
4. **Data streaming** — real-time compression for high-frequency trading

## Adding Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
flate2 = "1.0"
```

The `flate2` crate provides gzip, deflate, and zlib compression — the most common algorithms.

## Basic String Compression

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::{Write, Read};

fn main() {
    let original = "BTC,42000.50,1.5,buy\nETH,2800.00,10.0,sell\n".repeat(1000);
    println!("Original: {} bytes", original.len());

    // Compress
    let compressed = compress_data(original.as_bytes()).unwrap();
    println!("Compressed: {} bytes", compressed.len());
    println!("Compression ratio: {:.2}x", original.len() as f64 / compressed.len() as f64);

    // Decompress
    let decompressed = decompress_data(&compressed).unwrap();
    let restored = String::from_utf8(decompressed).unwrap();

    assert_eq!(original, restored);
    println!("Data restored correctly!");
}

fn compress_data(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

fn decompress_data(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}
```

## Compression Levels

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;
use std::time::Instant;

fn main() {
    // Generate test data: trade history
    let trades: String = (0..100_000)
        .map(|i| format!("BTCUSD,{:.2},{:.4},buy,{}\n",
            42000.0 + (i as f64 * 0.01).sin() * 100.0,
            0.001 + (i as f64 * 0.1).cos().abs(),
            1700000000 + i
        ))
        .collect();

    let data = trades.as_bytes();
    println!("Original size: {} bytes\n", data.len());

    let levels = [
        (Compression::none(), "No compression"),
        (Compression::fast(), "Fast (level 1)"),
        (Compression::default(), "Default (level 6)"),
        (Compression::best(), "Maximum (level 9)"),
    ];

    for (level, name) in levels {
        let start = Instant::now();

        let mut encoder = GzEncoder::new(Vec::new(), level);
        encoder.write_all(data).unwrap();
        let compressed = encoder.finish().unwrap();

        let elapsed = start.elapsed();
        let ratio = data.len() as f64 / compressed.len() as f64;

        println!("{}: {} bytes ({:.2}x) in {:?}",
            name, compressed.len(), ratio, elapsed);
    }
}
```

## Working with Files: Compressing Price History

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Read, BufRead};

fn main() -> std::io::Result<()> {
    // Create test file with OHLCV history
    create_sample_ohlcv("price_history.csv")?;

    // Compress the file
    compress_file("price_history.csv", "price_history.csv.gz")?;

    // Compare sizes
    let original_size = std::fs::metadata("price_history.csv")?.len();
    let compressed_size = std::fs::metadata("price_history.csv.gz")?.len();

    println!("Original: {} bytes", original_size);
    println!("Compressed: {} bytes", compressed_size);
    println!("Savings: {:.1}%",
        (1.0 - compressed_size as f64 / original_size as f64) * 100.0);

    // Read compressed file directly
    println!("\nFirst 5 lines from compressed file:");
    read_compressed_lines("price_history.csv.gz", 5)?;

    Ok(())
}

fn create_sample_ohlcv(path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "timestamp,open,high,low,close,volume")?;

    let mut price = 42000.0_f64;
    for i in 0..50_000 {
        let volatility = (i as f64 * 0.1).sin() * 100.0;
        let open = price;
        let high = price + volatility.abs() + 50.0;
        let low = price - volatility.abs() - 50.0;
        let close = price + volatility;
        let volume = 100.0 + (i as f64 * 0.05).cos().abs() * 1000.0;

        writeln!(writer, "{},{:.2},{:.2},{:.2},{:.2},{:.4}",
            1700000000 + i * 60, open, high, low, close, volume)?;

        price = close;
    }

    Ok(())
}

fn compress_file(input: &str, output: &str) -> std::io::Result<()> {
    let input_file = File::open(input)?;
    let mut reader = BufReader::new(input_file);

    let output_file = File::create(output)?;
    let mut encoder = GzEncoder::new(BufWriter::new(output_file), Compression::default());

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        encoder.write_all(&buffer[..bytes_read])?;
    }

    encoder.finish()?;
    Ok(())
}

fn read_compressed_lines(path: &str, count: usize) -> std::io::Result<()> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let reader = BufReader::new(decoder);

    for line in reader.lines().take(count) {
        println!("{}", line?);
    }

    Ok(())
}
```

## Streaming Compression for Large Files

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Read, BufRead};

const CHUNK_SIZE: usize = 64 * 1024; // 64 KB

fn main() -> std::io::Result<()> {
    // Create large tick file
    println!("Creating large tick file...");
    create_large_tick_file("ticks.csv", 1_000_000)?;

    let original_size = std::fs::metadata("ticks.csv")?.len();
    println!("Tick file size: {} MB", original_size / 1_000_000);

    // Stream compression
    println!("Compressing with streaming...");
    stream_compress("ticks.csv", "ticks.csv.gz")?;

    let compressed_size = std::fs::metadata("ticks.csv.gz")?.len();
    println!("Compressed size: {} MB", compressed_size / 1_000_000);
    println!("Compression ratio: {:.2}x", original_size as f64 / compressed_size as f64);

    // Stream reading and analysis
    println!("\nAnalyzing compressed file...");
    let stats = analyze_compressed_ticks("ticks.csv.gz")?;
    println!("Total ticks: {}", stats.count);
    println!("Min price: {:.2}", stats.min_price);
    println!("Max price: {:.2}", stats.max_price);
    println!("Total volume: {:.4}", stats.total_volume);

    Ok(())
}

fn create_large_tick_file(path: &str, count: usize) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "timestamp,price,volume,side")?;

    let mut price = 42000.0_f64;
    for i in 0..count {
        price += ((i as f64 * 0.001).sin() * 10.0) as f64;
        let volume = 0.001 + (i as f64 * 0.0001).cos().abs();
        let side = if i % 2 == 0 { "buy" } else { "sell" };

        writeln!(writer, "{},{:.2},{:.6},{}",
            1700000000000_u64 + i as u64, price, volume, side)?;
    }

    Ok(())
}

fn stream_compress(input: &str, output: &str) -> std::io::Result<()> {
    let input_file = File::open(input)?;
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, input_file);

    let output_file = File::create(output)?;
    let mut encoder = GzEncoder::new(
        BufWriter::with_capacity(CHUNK_SIZE, output_file),
        Compression::default()
    );

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut total_read = 0u64;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        encoder.write_all(&buffer[..bytes_read])?;
        total_read += bytes_read as u64;

        // Progress every 10 MB
        if total_read % (10 * 1024 * 1024) == 0 {
            print!("\rProcessed: {} MB", total_read / 1_000_000);
        }
    }
    println!();

    encoder.finish()?;
    Ok(())
}

struct TickStats {
    count: usize,
    min_price: f64,
    max_price: f64,
    total_volume: f64,
}

fn analyze_compressed_ticks(path: &str) -> std::io::Result<TickStats> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let reader = BufReader::new(decoder);

    let mut stats = TickStats {
        count: 0,
        min_price: f64::MAX,
        max_price: f64::MIN,
        total_volume: 0.0,
    };

    for line in reader.lines().skip(1) { // Skip header
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() >= 3 {
            if let (Ok(price), Ok(volume)) = (
                parts[1].parse::<f64>(),
                parts[2].parse::<f64>()
            ) {
                stats.count += 1;
                stats.min_price = stats.min_price.min(price);
                stats.max_price = stats.max_price.max(price);
                stats.total_volume += volume;
            }
        }
    }

    Ok(stats)
}
```

## In-Memory Compression for API Responses

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::{Write, Read};

fn main() {
    // Simulate exchange response — order list
    let orders = generate_order_book_json();
    println!("Order book JSON size: {} bytes", orders.len());

    // Compress for transfer
    let compressed = compress_for_transfer(&orders);
    println!("Compressed size: {} bytes", compressed.len());
    println!("Bandwidth savings: {:.1}%",
        (1.0 - compressed.len() as f64 / orders.len() as f64) * 100.0);

    // Decompress on client
    let restored = decompress_from_transfer(&compressed);
    assert_eq!(orders, restored);
    println!("Data restored correctly!");
}

fn generate_order_book_json() -> String {
    let mut json = String::from(r#"{"bids":["#);

    for i in 0..1000 {
        if i > 0 { json.push(','); }
        json.push_str(&format!(
            r#"{{"price":{:.2},"qty":{:.4}}}"#,
            42000.0 - i as f64 * 0.5,
            1.0 + (i as f64 * 0.1).sin().abs()
        ));
    }

    json.push_str(r#"],"asks":["#);

    for i in 0..1000 {
        if i > 0 { json.push(','); }
        json.push_str(&format!(
            r#"{{"price":{:.2},"qty":{:.4}}}"#,
            42001.0 + i as f64 * 0.5,
            1.0 + (i as f64 * 0.1).cos().abs()
        ));
    }

    json.push_str("]}");
    json
}

fn compress_for_transfer(data: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data.as_bytes()).unwrap();
    encoder.finish().unwrap()
}

fn decompress_from_transfer(data: &[u8]) -> String {
    let mut decoder = GzDecoder::new(data);
    let mut result = String::new();
    decoder.read_to_string(&mut result).unwrap();
    result
}
```

## Archiving Trading Logs

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> std::io::Result<()> {
    // Create log archiver
    let mut archiver = LogArchiver::new("trading_logs")?;

    // Simulate writing logs during a "day"
    for hour in 0..24 {
        for minute in 0..60 {
            let log_entry = format!(
                "[2024-01-15 {:02}:{:02}:00] TRADE executed: BTC/USD {} @ {:.2}\n",
                hour, minute,
                if (hour + minute) % 2 == 0 { "BUY" } else { "SELL" },
                42000.0 + (hour * 60 + minute) as f64 * 0.5
            );
            archiver.write_log(&log_entry)?;
        }
    }

    // Finish and close archive
    let stats = archiver.finish()?;

    println!("Archiving complete:");
    println!("  Entries: {}", stats.entries);
    println!("  Bytes written: {}", stats.bytes_written);
    println!("  Archive size: {}", stats.archive_size);
    println!("  Compression: {:.1}x", stats.bytes_written as f64 / stats.archive_size as f64);

    Ok(())
}

struct LogArchiver {
    encoder: GzEncoder<BufWriter<File>>,
    entries: usize,
    bytes_written: usize,
}

struct ArchiveStats {
    entries: usize,
    bytes_written: usize,
    archive_size: usize,
}

impl LogArchiver {
    fn new(base_name: &str) -> std::io::Result<Self> {
        let filename = format!("{}.log.gz", base_name);
        let file = File::create(&filename)?;
        let encoder = GzEncoder::new(
            BufWriter::new(file),
            Compression::default()
        );

        Ok(LogArchiver {
            encoder,
            entries: 0,
            bytes_written: 0,
        })
    }

    fn write_log(&mut self, entry: &str) -> std::io::Result<()> {
        self.encoder.write_all(entry.as_bytes())?;
        self.entries += 1;
        self.bytes_written += entry.len();
        Ok(())
    }

    fn finish(self) -> std::io::Result<ArchiveStats> {
        let entries = self.entries;
        let bytes_written = self.bytes_written;

        let inner = self.encoder.finish()?;
        inner.into_inner()?.sync_all()?;

        let archive_size = std::fs::metadata("trading_logs.log.gz")?.len() as usize;

        Ok(ArchiveStats {
            entries,
            bytes_written,
            archive_size,
        })
    }
}
```

## Comparing Compression Algorithms

```rust
use flate2::Compression;
use flate2::write::{GzEncoder, ZlibEncoder, DeflateEncoder};
use std::io::Write;
use std::time::Instant;

fn main() {
    // Generate typical trading data
    let data = generate_trading_data();
    println!("Original size: {} bytes\n", data.len());

    // Test different algorithms
    println!("{:<15} {:>12} {:>10} {:>12}", "Algorithm", "Size", "Ratio", "Time");
    println!("{}", "-".repeat(52));

    // Gzip
    let (size, time) = test_gzip(&data);
    println!("{:<15} {:>12} {:>10.2}x {:>10.2}ms",
        "Gzip", size, data.len() as f64 / size as f64, time);

    // Zlib
    let (size, time) = test_zlib(&data);
    println!("{:<15} {:>12} {:>10.2}x {:>10.2}ms",
        "Zlib", size, data.len() as f64 / size as f64, time);

    // Deflate
    let (size, time) = test_deflate(&data);
    println!("{:<15} {:>12} {:>10.2}x {:>10.2}ms",
        "Deflate", size, data.len() as f64 / size as f64, time);
}

fn generate_trading_data() -> Vec<u8> {
    let mut data = String::new();

    // OHLCV data has high repetition — compresses well
    for i in 0..10_000 {
        data.push_str(&format!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.4}\n",
            1700000000 + i * 60,
            42000.0 + (i as f64 * 0.1).sin() * 100.0,
            42100.0 + (i as f64 * 0.1).sin() * 100.0,
            41900.0 + (i as f64 * 0.1).sin() * 100.0,
            42050.0 + (i as f64 * 0.1).sin() * 100.0,
            100.0 + (i as f64 * 0.05).cos().abs() * 50.0
        ));
    }

    data.into_bytes()
}

fn test_gzip(data: &[u8]) -> (usize, f64) {
    let start = Instant::now();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();
    (compressed.len(), start.elapsed().as_secs_f64() * 1000.0)
}

fn test_zlib(data: &[u8]) -> (usize, f64) {
    let start = Instant::now();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();
    (compressed.len(), start.elapsed().as_secs_f64() * 1000.0)
}

fn test_deflate(data: &[u8]) -> (usize, f64) {
    let start = Instant::now();
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();
    (compressed.len(), start.elapsed().as_secs_f64() * 1000.0)
}
```

## Practical Example: Historical Data Archive

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::collections::HashMap;

fn main() -> std::io::Result<()> {
    // Create archive with multiple trading pairs
    let pairs = vec!["BTCUSD", "ETHUSD", "SOLUSD"];

    for pair in &pairs {
        create_pair_archive(pair, 50_000)?;
    }

    // Read and analyze all archives
    println!("\n{:<10} {:>15} {:>15} {:>15}", "Pair", "Candles", "Min Price", "Max Price");
    println!("{}", "-".repeat(60));

    for pair in &pairs {
        let stats = read_pair_archive(pair)?;
        println!("{:<10} {:>15} {:>15.2} {:>15.2}",
            pair, stats.count, stats.min_price, stats.max_price);
    }

    Ok(())
}

fn create_pair_archive(pair: &str, candles: usize) -> std::io::Result<()> {
    let filename = format!("{}_1m.csv.gz", pair.to_lowercase());
    let file = File::create(&filename)?;
    let mut encoder = GzEncoder::new(BufWriter::new(file), Compression::default());

    writeln!(encoder, "timestamp,open,high,low,close,volume")?;

    let base_price = match pair {
        "BTCUSD" => 42000.0,
        "ETHUSD" => 2800.0,
        "SOLUSD" => 100.0,
        _ => 1000.0,
    };

    let mut price = base_price;
    for i in 0..candles {
        let volatility = (i as f64 * 0.1).sin() * base_price * 0.001;
        let open = price;
        let high = price + volatility.abs() + base_price * 0.0005;
        let low = price - volatility.abs() - base_price * 0.0005;
        let close = price + volatility;
        let volume = 100.0 + (i as f64 * 0.05).cos().abs() * 1000.0;

        writeln!(encoder, "{},{:.2},{:.2},{:.2},{:.2},{:.4}",
            1700000000 + i * 60, open, high, low, close, volume)?;

        price = close;
    }

    let inner = encoder.finish()?;
    inner.into_inner()?.sync_all()?;

    let size = std::fs::metadata(&filename)?.len();
    println!("Created {}: {} KB", filename, size / 1024);

    Ok(())
}

struct PairStats {
    count: usize,
    min_price: f64,
    max_price: f64,
}

fn read_pair_archive(pair: &str) -> std::io::Result<PairStats> {
    let filename = format!("{}_1m.csv.gz", pair.to_lowercase());
    let file = File::open(&filename)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let reader = BufReader::new(decoder);

    let mut stats = PairStats {
        count: 0,
        min_price: f64::MAX,
        max_price: f64::MIN,
    };

    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() >= 5 {
            if let (Ok(low), Ok(high)) = (
                parts[3].parse::<f64>(),
                parts[2].parse::<f64>()
            ) {
                stats.count += 1;
                stats.min_price = stats.min_price.min(low);
                stats.max_price = stats.max_price.max(high);
            }
        }
    }

    Ok(stats)
}
```

## What We Learned

| Technique | Application | Advantage |
|-----------|-------------|-----------|
| `GzEncoder` | File/data compression | Universal .gz format |
| `GzDecoder` | Gzip decompression | Streaming read |
| `Compression::fast()` | Fast compression | Minimal latency |
| `Compression::best()` | Maximum compression | Minimal size |
| Streaming processing | Large files | Low memory usage |

## Tips for Choosing Compression in Trading

1. **For real-time data**: `Compression::fast()` — minimal latency
2. **For archives**: `Compression::best()` — maximum space savings
3. **For backtesting**: `Compression::default()` — balance of speed and size
4. **For APIs**: gzip — supported by all browsers and libraries

## Homework

1. Create a function `compress_trade_history(trades: &[Trade]) -> Vec<u8>` that serializes a list of trades to JSON and compresses it

2. Implement `CompressedDataReader` that reads compressed CSV files line by line and parses them into `Candle` structures

3. Write a utility for archiving trading bot logs: daily rotation, automatic compression of files older than 24 hours

4. Create a benchmark comparing different compression levels on real trading data. Determine the optimal level for your use case

## Navigation

[← Previous day](../147-log-rotation/en.md) | [Next day →](../149-streaming-data-processing/en.md)
