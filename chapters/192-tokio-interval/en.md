# Day 192: tokio::interval: Periodic Tasks

## Trading Analogy

Imagine a trading bot that needs to regularly perform certain actions:
- Check the current asset price every second
- Recalculate moving averages every 5 seconds
- Check portfolio balance every minute
- Save trading statistics every hour

It's like alarm clocks that go off at regular intervals. In tokio, `tokio::time::interval` is used for such tasks — a mechanism that "ticks" at a specified frequency.

## What is tokio::interval?

`tokio::interval` creates an asynchronous interval that triggers at specified time intervals. Unlike `sleep`, which just waits once, `interval` continues to "tick" indefinitely.

```rust
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() {
    // Create an interval with a 1-second period
    let mut ticker = interval(Duration::from_secs(1));

    // The first tick fires immediately!
    ticker.tick().await;
    println!("Tick 1");

    ticker.tick().await;
    println!("Tick 2");

    ticker.tick().await;
    println!("Tick 3");
}
```

**Important:** The first call to `tick()` fires immediately! This is convenient when you need to perform an action right away and then repeat it periodically.

## Asset Price Monitoring

Let's look at a practical example — monitoring Bitcoin price every second:

```rust
use tokio::time::{interval, Duration};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: u64,
}

struct PriceMonitor {
    current_price: Arc<RwLock<Option<PriceData>>>,
    price_history: Arc<RwLock<Vec<PriceData>>>,
}

impl PriceMonitor {
    fn new() -> Self {
        PriceMonitor {
            current_price: Arc::new(RwLock::new(None)),
            price_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn start_monitoring(&self) {
        let current_price = Arc::clone(&self.current_price);
        let price_history = Arc::clone(&self.price_history);

        let mut ticker = interval(Duration::from_secs(1));

        loop {
            ticker.tick().await;

            // Simulate fetching price (in reality — API request)
            let price = simulate_price_fetch().await;

            let data = PriceData {
                symbol: "BTC/USDT".to_string(),
                price,
                timestamp: get_timestamp(),
            };

            println!("BTC Price: ${:.2}", data.price);

            // Update current price
            {
                let mut current = current_price.write().await;
                *current = Some(data.clone());
            }

            // Save to history
            {
                let mut history = price_history.write().await;
                history.push(data);

                // Keep only the last 100 entries
                if history.len() > 100 {
                    history.remove(0);
                }
            }
        }
    }
}

async fn simulate_price_fetch() -> f64 {
    // Simulate random price around $42000
    42000.0 + (rand::random::<f64>() - 0.5) * 1000.0
}

fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn rand_random() -> f64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(get_timestamp());
    (hasher.finish() % 1000) as f64 / 1000.0
}

#[tokio::main]
async fn main() {
    let monitor = PriceMonitor::new();

    // Start monitoring (runs forever)
    // In a real application, you need a shutdown mechanism
    monitor.start_monitoring().await;
}
```

## Multiple Intervals Simultaneously

In trading, you often need to perform different tasks at different frequencies. We use `tokio::select!` for this:

```rust
use tokio::time::{interval, Duration};

struct TradingBot {
    symbol: String,
}

impl TradingBot {
    fn new(symbol: &str) -> Self {
        TradingBot {
            symbol: symbol.to_string(),
        }
    }

    async fn check_price(&self) {
        println!("[{}] Checking price...", self.symbol);
    }

    async fn calculate_indicators(&self) {
        println!("[{}] Calculating indicators (SMA, RSI, MACD)...", self.symbol);
    }

    async fn check_signals(&self) {
        println!("[{}] Checking trading signals...", self.symbol);
    }

    async fn save_statistics(&self) {
        println!("[{}] Saving statistics...", self.symbol);
    }

    async fn run(&self) {
        // Different intervals for different tasks
        let mut price_ticker = interval(Duration::from_secs(1));
        let mut indicator_ticker = interval(Duration::from_secs(5));
        let mut signal_ticker = interval(Duration::from_secs(10));
        let mut stats_ticker = interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = price_ticker.tick() => {
                    self.check_price().await;
                }
                _ = indicator_ticker.tick() => {
                    self.calculate_indicators().await;
                }
                _ = signal_ticker.tick() => {
                    self.check_signals().await;
                }
                _ = stats_ticker.tick() => {
                    self.save_statistics().await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let bot = TradingBot::new("BTC/USDT");
    bot.run().await;
}
```

## Missed Tick Behavior (MissedTickBehavior)

What happens if processing a tick takes longer than the interval? Tokio offers three strategies:

```rust
use tokio::time::{interval, Duration, MissedTickBehavior};

async fn demonstrate_missed_tick_behavior() {
    // Strategy 1: Burst (default)
    // Missed ticks are executed consecutively
    let mut burst_ticker = interval(Duration::from_millis(100));
    burst_ticker.set_missed_tick_behavior(MissedTickBehavior::Burst);

    // Strategy 2: Delay
    // Next tick is counted from when the previous one finished
    let mut delay_ticker = interval(Duration::from_millis(100));
    delay_ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    // Strategy 3: Skip
    // Missed ticks are ignored, next tick follows schedule
    let mut skip_ticker = interval(Duration::from_millis(100));
    skip_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
}
```

### Trading Example

```rust
use tokio::time::{interval, Duration, MissedTickBehavior, sleep};

struct OrderBookMonitor {
    symbol: String,
}

impl OrderBookMonitor {
    async fn heavy_analysis(&self) {
        // Simulate heavy processing (200ms with 100ms interval)
        println!("[{}] Starting order book analysis...", self.symbol);
        sleep(Duration::from_millis(200)).await;
        println!("[{}] Analysis complete", self.symbol);
    }

    async fn run_with_skip(&self) {
        // Skip — best choice for real-time monitoring
        // We care about fresh data, not accumulated old data
        let mut ticker = interval(Duration::from_millis(100));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        for i in 0..5 {
            ticker.tick().await;
            println!("Tick {}", i + 1);
            self.heavy_analysis().await;
        }
    }

    async fn run_with_delay(&self) {
        // Delay — guarantees minimum interval between runs
        // Good for tasks where processing completeness matters
        let mut ticker = interval(Duration::from_millis(100));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        for i in 0..5 {
            ticker.tick().await;
            println!("Tick {}", i + 1);
            self.heavy_analysis().await;
        }
    }
}

#[tokio::main]
async fn main() {
    let monitor = OrderBookMonitor {
        symbol: "ETH/USDT".to_string(),
    };

    println!("=== Skip Mode ===");
    monitor.run_with_skip().await;

    println!("\n=== Delay Mode ===");
    monitor.run_with_delay().await;
}
```

## Stopping an Interval

Real applications need a shutdown mechanism:

```rust
use tokio::time::{interval, Duration};
use tokio::sync::watch;

struct PriceAlert {
    symbol: String,
    target_price: f64,
}

impl PriceAlert {
    async fn monitor_until_target(
        &self,
        mut shutdown: watch::Receiver<bool>,
    ) -> Option<f64> {
        let mut ticker = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let current_price = self.fetch_price().await;
                    println!("[{}] Price: ${:.2}, Target: ${:.2}",
                        self.symbol, current_price, self.target_price);

                    if current_price >= self.target_price {
                        println!("Target price reached!");
                        return Some(current_price);
                    }
                }
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        println!("Monitoring stopped");
                        return None;
                    }
                }
            }
        }
    }

    async fn fetch_price(&self) -> f64 {
        // Simulate price with growth
        static COUNTER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        42000.0 + count as f64 * 100.0
    }
}

#[tokio::main]
async fn main() {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let alert = PriceAlert {
        symbol: "BTC/USDT".to_string(),
        target_price: 42500.0,
    };

    // Start monitoring in a separate task
    let handle = tokio::spawn(async move {
        alert.monitor_until_target(shutdown_rx).await
    });

    // Can stop manually after 10 seconds
    // shutdown_tx.send(true).unwrap();

    match handle.await.unwrap() {
        Some(price) => println!("Final price: ${:.2}", price),
        None => println!("Monitoring was interrupted"),
    }
}
```

## interval_at: Start at a Specific Moment

Sometimes you need to start an interval not immediately, but at a specific moment:

```rust
use tokio::time::{interval_at, Duration, Instant};

async fn schedule_market_tasks() {
    let now = Instant::now();

    // Start after 5 seconds, then every minute
    let start = now + Duration::from_secs(5);
    let mut ticker = interval_at(start, Duration::from_secs(60));

    println!("Scheduler started, first tick in 5 seconds...");

    loop {
        ticker.tick().await;
        println!("Executing scheduled task: {:?}", Instant::now());
    }
}

#[tokio::main]
async fn main() {
    schedule_market_tasks().await;
}
```

## Practical Example: Trading Scheduler

```rust
use tokio::time::{interval, Duration, MissedTickBehavior};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct MarketData {
    price: f64,
    volume: f64,
    sma_20: Option<f64>,
    rsi: Option<f64>,
}

#[derive(Debug)]
struct TradingScheduler {
    symbol: String,
    market_data: Arc<RwLock<MarketData>>,
    price_history: Arc<RwLock<Vec<f64>>>,
}

impl TradingScheduler {
    fn new(symbol: &str) -> Self {
        TradingScheduler {
            symbol: symbol.to_string(),
            market_data: Arc::new(RwLock::new(MarketData {
                price: 0.0,
                volume: 0.0,
                sma_20: None,
                rsi: None,
            })),
            price_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn update_price(&self) {
        let price = simulate_price();

        let mut data = self.market_data.write().await;
        data.price = price;

        let mut history = self.price_history.write().await;
        history.push(price);

        // Keep only the last 100 prices
        if history.len() > 100 {
            history.remove(0);
        }

        println!("[Price] {}: ${:.2}", self.symbol, price);
    }

    async fn calculate_sma(&self) {
        let history = self.price_history.read().await;

        if history.len() >= 20 {
            let sum: f64 = history.iter().rev().take(20).sum();
            let sma = sum / 20.0;

            drop(history); // Release read lock

            let mut data = self.market_data.write().await;
            data.sma_20 = Some(sma);

            println!("[SMA-20] {}: ${:.2}", self.symbol, sma);
        }
    }

    async fn calculate_rsi(&self) {
        let history = self.price_history.read().await;

        if history.len() >= 14 {
            // Simplified RSI calculation
            let changes: Vec<f64> = history
                .windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            let gains: f64 = changes.iter()
                .filter(|&&x| x > 0.0)
                .sum();
            let losses: f64 = changes.iter()
                .filter(|&&x| x < 0.0)
                .map(|x| x.abs())
                .sum();

            let rsi = if losses == 0.0 {
                100.0
            } else {
                let rs = gains / losses;
                100.0 - (100.0 / (1.0 + rs))
            };

            drop(history);

            let mut data = self.market_data.write().await;
            data.rsi = Some(rsi);

            println!("[RSI] {}: {:.1}", self.symbol, rsi);
        }
    }

    async fn check_trading_signals(&self) {
        let data = self.market_data.read().await;

        if let (Some(sma), Some(rsi)) = (data.sma_20, data.rsi) {
            let price = data.price;

            // Simple strategy:
            // Buy: price above SMA and RSI < 30
            // Sell: price below SMA and RSI > 70
            if price > sma && rsi < 30.0 {
                println!("[SIGNAL] {} - BUY! Price ${:.2} > SMA ${:.2}, RSI {:.1}",
                    self.symbol, price, sma, rsi);
            } else if price < sma && rsi > 70.0 {
                println!("[SIGNAL] {} - SELL! Price ${:.2} < SMA ${:.2}, RSI {:.1}",
                    self.symbol, price, sma, rsi);
            }
        }
    }

    async fn run(&self, max_iterations: usize) {
        let mut price_ticker = interval(Duration::from_millis(500));
        let mut sma_ticker = interval(Duration::from_secs(2));
        let mut rsi_ticker = interval(Duration::from_secs(3));
        let mut signal_ticker = interval(Duration::from_secs(5));

        // For indicator tasks, use Skip
        sma_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        rsi_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut iterations = 0;

        loop {
            if iterations >= max_iterations {
                println!("\nScheduler shutting down");
                break;
            }

            tokio::select! {
                _ = price_ticker.tick() => {
                    self.update_price().await;
                    iterations += 1;
                }
                _ = sma_ticker.tick() => {
                    self.calculate_sma().await;
                }
                _ = rsi_ticker.tick() => {
                    self.calculate_rsi().await;
                }
                _ = signal_ticker.tick() => {
                    self.check_trading_signals().await;
                }
            }
        }
    }
}

fn simulate_price() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    42000.0 + (nanos as f64 / 1_000_000.0) - 500.0
}

#[tokio::main]
async fn main() {
    let scheduler = TradingScheduler::new("BTC/USDT");

    println!("Starting trading scheduler...\n");
    scheduler.run(30).await;

    println!("\nFinal state:");
    let data = scheduler.market_data.read().await;
    println!("{:?}", *data);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `interval(duration)` | Creates a periodic timer |
| `tick().await` | Waits for the next tick |
| First tick | Fires immediately |
| `MissedTickBehavior::Burst` | Missed ticks execute consecutively |
| `MissedTickBehavior::Delay` | Counts from previous completion |
| `MissedTickBehavior::Skip` | Missed ticks are ignored |
| `interval_at(start, period)` | Start at a specific moment |
| `tokio::select!` | Handle multiple intervals |

## Practical Exercises

1. **Multi-Asset Monitor**: Create a program that simultaneously tracks prices of BTC, ETH, and SOL with different intervals (1s, 2s, 3s respectively).

2. **Volatility Detector**: Write a program that every 5 seconds calculates volatility (standard deviation) over the last 20 price measurements and outputs a warning if volatility exceeds a threshold.

3. **Portfolio Rebalancer**: Implement a program that every minute checks the asset allocation in a portfolio and outputs rebalancing recommendations if the deviation from target allocation exceeds 5%.

## Homework

1. **Alert System**: Implement a price alert system with different types:
   - Alert when price is reached
   - Alert when price changes by N%
   - Alert when price crosses moving average

   Use different intervals for checking different alert types.

2. **Heartbeat Monitor**: Create a system that:
   - Sends a "heartbeat" every 10 seconds
   - Checks that all system components are "alive"
   - Outputs a warning if a component doesn't respond for 30 seconds

3. **Smart Scheduler**: Implement a scheduler that:
   - Increases price check frequency during high volatility
   - Decreases frequency during calm periods
   - Uses `interval` with dynamic period changes

4. **Trading Session Simulator**: Create a program that:
   - Simulates a trading session with market open and close
   - Uses `interval_at` to start at a specific time
   - Performs different actions before, during, and after the session

## Navigation

[← Previous day](../191-tokio-sleep-delay/en.md) | [Next day →](../193-tokio-timeout/en.md)
