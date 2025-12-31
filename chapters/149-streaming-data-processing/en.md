# Day 149: Streaming Data Processing

## Trading Analogy

Imagine a trading terminal receiving thousands of price updates per second. You can't load all the data into memory and only then start analysis — the system would choke. Instead, you process data **as it arrives**: receive quote → analyze → make decision → move to the next one. This is **streaming data processing**.

## Why Streaming Processing?

In trading we often work with:
- **Infinite streams** — real-time quotes never stop
- **Large files** — years of trading history can be gigabytes
- **Limited memory** — you can't load 100 GB of data into 16 GB RAM

Streaming processing allows working with data **in chunks**, using minimal memory.

## Iterators — The Foundation of Streaming

In Rust, iterators are "lazy" data streams. They compute the next element only when needed.

```rust
fn main() {
    // Simulating a price stream
    let price_stream = vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

    // Lazy processing — nothing is computed until .collect() or .for_each()
    let signals: Vec<&str> = price_stream
        .windows(2)  // Sliding window of 2 elements
        .map(|window| {
            if window[1] > window[0] {
                "BUY"
            } else {
                "SELL"
            }
        })
        .collect();

    println!("Trading signals: {:?}", signals);
}
```

## Reading Files Line by Line

Don't load the entire file — read line by line:

```rust
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    // Open trade history file
    let file = File::open("trades.csv")?;
    let reader = BufReader::new(file);

    let mut total_volume = 0.0;
    let mut trade_count = 0u64;

    // Read line by line — only one line in memory!
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
                trade_count += 1;
            }
        }
    }

    println!("Total trades: {}", trade_count);
    println!("Total volume: {:.2}", total_volume);
    println!("Average volume: {:.2}", total_volume / trade_count as f64);

    Ok(())
}
```

## Streaming Moving Average Calculation

Calculate SMA without storing all data in memory:

```rust
use std::collections::VecDeque;

/// Moving average with limited buffer
struct StreamingSMA {
    window: VecDeque<f64>,
    period: usize,
    sum: f64,
}

impl StreamingSMA {
    fn new(period: usize) -> Self {
        StreamingSMA {
            window: VecDeque::with_capacity(period),
            period,
            sum: 0.0,
        }
    }

    /// Adds new price and returns SMA (if enough data)
    fn update(&mut self, price: f64) -> Option<f64> {
        // Add new price
        self.window.push_back(price);
        self.sum += price;

        // If window overflows — remove old price
        if self.window.len() > self.period {
            if let Some(old) = self.window.pop_front() {
                self.sum -= old;
            }
        }

        // Return SMA only if enough data
        if self.window.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

fn main() {
    let mut sma = StreamingSMA::new(3);

    let prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0, 42300.0];

    for price in prices {
        match sma.update(price) {
            Some(avg) => println!("Price: {:.0} -> SMA-3: {:.2}", price, avg),
            None => println!("Price: {:.0} -> SMA-3: Accumulating data...", price),
        }
    }
}
```

## Streaming VWAP Calculation

VWAP (Volume Weighted Average Price) — key indicator for institutional traders:

```rust
/// Streaming VWAP calculation
struct StreamingVWAP {
    cumulative_volume: f64,
    cumulative_pv: f64,  // Price * Volume
}

impl StreamingVWAP {
    fn new() -> Self {
        StreamingVWAP {
            cumulative_volume: 0.0,
            cumulative_pv: 0.0,
        }
    }

    /// Adds trade and returns current VWAP
    fn update(&mut self, price: f64, volume: f64) -> f64 {
        self.cumulative_volume += volume;
        self.cumulative_pv += price * volume;

        if self.cumulative_volume > 0.0 {
            self.cumulative_pv / self.cumulative_volume
        } else {
            0.0
        }
    }

    /// Resets VWAP (usually at start of trading day)
    fn reset(&mut self) {
        self.cumulative_volume = 0.0;
        self.cumulative_pv = 0.0;
    }
}

fn main() {
    let mut vwap = StreamingVWAP::new();

    // Trade stream: (price, volume)
    let trades = [
        (42000.0, 1.5),
        (42050.0, 2.0),
        (42100.0, 0.8),
        (42080.0, 1.2),
        (42150.0, 3.0),
    ];

    println!("╔════════════════════════════════════════════╗");
    println!("║          STREAMING VWAP CALCULATION        ║");
    println!("╠════════════════════════════════════════════╣");

    for (price, volume) in trades {
        let current_vwap = vwap.update(price, volume);
        println!("║ Trade: {:.0} x {:.1} BTC -> VWAP: {:.2} ║", price, volume, current_vwap);
    }

    println!("╚════════════════════════════════════════════╝");
}
```

## Channels for Inter-Thread Data Processing

Using `std::sync::mpsc` to pass data between threads:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Create channel: tx — sender, rx — receiver
    let (tx, rx) = mpsc::channel();

    // Producer thread: generates quotes
    let producer = thread::spawn(move || {
        let prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

        for price in prices {
            println!("[Producer] Sending price: {}", price);
            tx.send(price).unwrap();
            thread::sleep(Duration::from_millis(100));
        }

        println!("[Producer] Done sending");
    });

    // Consumer thread: processes quotes
    let consumer = thread::spawn(move || {
        let mut sum = 0.0;
        let mut count = 0;

        // Receive data while channel is open
        while let Ok(price) = rx.recv() {
            sum += price;
            count += 1;
            println!("[Consumer] Received: {}, Running avg: {:.2}",
                     price, sum / count as f64);
        }

        println!("[Consumer] Final average: {:.2}", sum / count as f64);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

## Streaming Trade Statistics

```rust
/// Streaming statistics using Welford's algorithm
struct StreamingStats {
    count: u64,
    mean: f64,
    m2: f64,  // Sum of squared deviations
    min: f64,
    max: f64,
}

impl StreamingStats {
    fn new() -> Self {
        StreamingStats {
            count: 0,
            mean: 0.0,
            m2: 0.0,
            min: f64::MAX,
            max: f64::MIN,
        }
    }

    /// Adds new value (Welford's algorithm for numerical stability)
    fn update(&mut self, value: f64) {
        self.count += 1;

        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;

        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
    }

    fn variance(&self) -> f64 {
        if self.count < 2 {
            0.0
        } else {
            self.m2 / (self.count - 1) as f64
        }
    }

    fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    fn print_summary(&self) {
        println!("╔═══════════════════════════════════════╗");
        println!("║       STREAMING TRADE STATISTICS      ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Count:    {:>25} ║", self.count);
        println!("║ Mean:     {:>25.2} ║", self.mean);
        println!("║ Std Dev:  {:>25.2} ║", self.std_dev());
        println!("║ Min:      {:>25.2} ║", self.min);
        println!("║ Max:      {:>25.2} ║", self.max);
        println!("╚═══════════════════════════════════════╝");
    }
}

fn main() {
    let mut stats = StreamingStats::new();

    // PnL stream from trades
    let pnl_stream = [150.0, -50.0, 200.0, -30.0, 180.0, -80.0, 220.0, 100.0];

    for pnl in pnl_stream {
        stats.update(pnl);
        println!("PnL: {:>7.2} | Running mean: {:>7.2} | Std: {:>7.2}",
                 pnl, stats.mean, stats.std_dev());
    }

    println!();
    stats.print_summary();
}
```

## Streaming Anomaly Detection

Detect unusual price movements in real-time:

```rust
/// Streaming anomaly detector based on Z-score
struct AnomalyDetector {
    stats: StreamingStats,
    threshold: f64,  // Threshold in standard deviations
}

/// Statistics for detector (simplified version)
struct StreamingStats {
    count: u64,
    mean: f64,
    m2: f64,
}

impl StreamingStats {
    fn new() -> Self {
        StreamingStats { count: 0, mean: 0.0, m2: 0.0 }
    }

    fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    fn std_dev(&self) -> f64 {
        if self.count < 2 { 0.0 }
        else { (self.m2 / (self.count - 1) as f64).sqrt() }
    }
}

impl AnomalyDetector {
    fn new(threshold: f64) -> Self {
        AnomalyDetector {
            stats: StreamingStats::new(),
            threshold,
        }
    }

    /// Checks if value is an anomaly
    fn check(&mut self, value: f64) -> (bool, f64) {
        let std_dev = self.stats.std_dev();
        let mean = self.stats.mean;

        // Calculate Z-score
        let z_score = if std_dev > 0.0 && self.stats.count > 10 {
            (value - mean) / std_dev
        } else {
            0.0
        };

        // Update statistics
        self.stats.update(value);

        // Anomaly if Z-score exceeds threshold
        let is_anomaly = z_score.abs() > self.threshold;

        (is_anomaly, z_score)
    }
}

fn main() {
    let mut detector = AnomalyDetector::new(2.0);  // 2 standard deviations

    // Price change stream (%)
    let price_changes = [
        0.1, 0.2, -0.1, 0.15, -0.2, 0.1, -0.15, 0.2,
        0.1, -0.1, 0.15, -0.2,
        5.0,   // <- Anomaly! Sharp spike
        0.1, -0.1, 0.2, -0.15,
        -4.5,  // <- Anomaly! Sharp drop
        0.1, 0.15
    ];

    println!("Streaming Anomaly Detection (threshold: 2σ)");
    println!("═══════════════════════════════════════════");

    for (i, &change) in price_changes.iter().enumerate() {
        let (is_anomaly, z_score) = detector.check(change);

        if is_anomaly {
            println!("⚠️  [{}] Change: {:>6.2}% | Z-score: {:>6.2} | ANOMALY!",
                     i, change, z_score);
        } else {
            println!("   [{:>2}] Change: {:>6.2}% | Z-score: {:>6.2}",
                     i, change, z_score);
        }
    }
}
```

## Real-Time Candle Aggregation

Build OHLCV candles from trade stream:

```rust
/// OHLCV Candle
#[derive(Debug, Clone)]
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    trade_count: u32,
}

/// Candle builder from trade stream
struct CandleBuilder {
    current: Option<Candle>,
}

impl CandleBuilder {
    fn new() -> Self {
        CandleBuilder { current: None }
    }

    /// Adds trade to current candle
    fn add_trade(&mut self, price: f64, volume: f64) {
        match &mut self.current {
            Some(candle) => {
                if price > candle.high {
                    candle.high = price;
                }
                if price < candle.low {
                    candle.low = price;
                }
                candle.close = price;
                candle.volume += volume;
                candle.trade_count += 1;
            }
            None => {
                self.current = Some(Candle {
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume,
                    trade_count: 1,
                });
            }
        }
    }

    /// Closes current candle and starts a new one
    fn close_candle(&mut self) -> Option<Candle> {
        self.current.take()
    }
}

fn main() {
    let mut builder = CandleBuilder::new();

    // Trade stream: (price, volume)
    let trades = [
        (42000.0, 1.0),
        (42050.0, 0.5),
        (42100.0, 2.0),  // High
        (41950.0, 1.5),  // Low
        (42020.0, 0.8),  // Close
    ];

    println!("Building candle from trade stream...\n");

    for (i, &(price, volume)) in trades.iter().enumerate() {
        builder.add_trade(price, volume);
        println!("Trade {}: price={}, volume={}", i + 1, price, volume);
    }

    if let Some(candle) = builder.close_candle() {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║          COMPLETED CANDLE             ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Open:        {:>20.2} ║", candle.open);
        println!("║ High:        {:>20.2} ║", candle.high);
        println!("║ Low:         {:>20.2} ║", candle.low);
        println!("║ Close:       {:>20.2} ║", candle.close);
        println!("║ Volume:      {:>20.2} ║", candle.volume);
        println!("║ Trades:      {:>20} ║", candle.trade_count);
        println!("╚═══════════════════════════════════════╝");
    }
}
```

## Streaming Processing Patterns

```rust
// 1. Map — transform each element
let prices = [42000.0, 42100.0, 42050.0];
let returns: Vec<f64> = prices
    .windows(2)
    .map(|w| (w[1] - w[0]) / w[0] * 100.0)
    .collect();

// 2. Filter — select by condition
let large_trades: Vec<f64> = volumes
    .iter()
    .filter(|&&v| v > 10.0)
    .cloned()
    .collect();

// 3. Fold/Reduce — aggregation
let total: f64 = prices.iter().fold(0.0, |acc, &x| acc + x);

// 4. Take/Skip — limit the stream
let first_10: Vec<_> = prices.iter().take(10).collect();
let after_warmup: Vec<_> = prices.iter().skip(100).collect();

// 5. Scan — cumulative transformation
let cumulative: Vec<f64> = prices
    .iter()
    .scan(0.0, |state, &x| {
        *state += x;
        Some(*state)
    })
    .collect();
```

## What We Learned

| Concept | Description | Application |
|---------|-------------|-------------|
| Iterators | Lazy data streams | Processing large files |
| BufReader | Buffered reading | Line-by-line log reading |
| VecDeque | Double-ended queue | Sliding windows |
| Channels (mpsc) | Inter-thread messaging | Producer-consumer separation |
| Welford's Algorithm | Streaming mean/variance | Real-time statistics |

## Homework

1. Implement `StreamingEMA` — streaming exponential moving average with smoothing parameter

2. Create `StreamingBollingerBands` — streaming Bollinger Bands calculation (mean ± 2 standard deviations)

3. Write a program that reads a large CSV file with trade history and builds 1-minute candles using only streaming processing

4. Implement a streaming detector for "double top" pattern based on sliding window of last N candles

## Navigation

[← Previous day](../148-data-compression/en.md) | [Next day →](../150-memoization/en.md)
