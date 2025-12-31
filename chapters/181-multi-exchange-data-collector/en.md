# Day 181: Project: Multi-Exchange Data Collector

## Project Overview

Welcome to the project chapter of the concurrency month! We will build a **multi-exchange data collector** — a system that simultaneously collects market data from multiple cryptocurrency exchanges, aggregates it, and provides a unified data stream for analysis.

### Trading Analogy

Imagine you're a professional trader working on multiple exchanges simultaneously: Binance, Kraken, Coinbase. Each exchange is a separate data source with different prices, different update speeds, and different formats. To find the best price or arbitrage opportunity, you need to:

1. **Fetch data in parallel** from all exchanges (threads)
2. **Safely aggregate** it into a unified storage (synchronization)
3. **Handle errors** without stopping the entire system (fault tolerance)
4. **Analyze spreads** between exchanges (data processing)

This is exactly what we will implement!

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Multi-Exchange Data Collector                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Binance  │  │  Kraken  │  │ Coinbase │  │  Bybit   │        │
│  │ Fetcher  │  │ Fetcher  │  │ Fetcher  │  │ Fetcher  │        │
│  │ (Thread) │  │ (Thread) │  │ (Thread) │  │ (Thread) │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
│       │             │             │             │               │
│       └─────────────┴──────┬──────┴─────────────┘               │
│                            │                                    │
│                            ▼                                    │
│                 ┌──────────────────┐                           │
│                 │  Price Channel   │                           │
│                 │  (mpsc::channel) │                           │
│                 └────────┬─────────┘                           │
│                          │                                      │
│                          ▼                                      │
│                 ┌──────────────────┐                           │
│                 │   Aggregator     │                           │
│                 │    (Thread)      │                           │
│                 └────────┬─────────┘                           │
│                          │                                      │
│                          ▼                                      │
│                 ┌──────────────────┐                           │
│                 │  Price Storage   │                           │
│                 │ (Arc<RwLock<>>)  │                           │
│                 └────────┬─────────┘                           │
│                          │                                      │
│           ┌──────────────┼──────────────┐                      │
│           ▼              ▼              ▼                      │
│    ┌───────────┐  ┌───────────┐  ┌───────────┐                │
│    │ Analyzer  │  │ Arbitrage │  │   Logger  │                │
│    │ (Thread)  │  │ Detector  │  │  (Thread) │                │
│    └───────────┘  └───────────┘  └───────────┘                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Part 1: Basic Data Structures

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Exchange identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Exchange {
    Binance,
    Kraken,
    Coinbase,
    Bybit,
}

impl Exchange {
    /// Returns all supported exchanges
    pub fn all() -> Vec<Exchange> {
        vec![
            Exchange::Binance,
            Exchange::Kraken,
            Exchange::Coinbase,
            Exchange::Bybit,
        ]
    }

    /// Display name of the exchange
    pub fn name(&self) -> &'static str {
        match self {
            Exchange::Binance => "Binance",
            Exchange::Kraken => "Kraken",
            Exchange::Coinbase => "Coinbase",
            Exchange::Bybit => "Bybit",
        }
    }

    /// Simulated API latency (in milliseconds)
    pub fn api_latency(&self) -> u64 {
        match self {
            Exchange::Binance => 50,
            Exchange::Kraken => 80,
            Exchange::Coinbase => 60,
            Exchange::Bybit => 70,
        }
    }
}

/// Trading pair
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradingPair {
    pub base: String,   // BTC
    pub quote: String,  // USDT
}

impl TradingPair {
    pub fn new(base: &str, quote: &str) -> Self {
        TradingPair {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }

    pub fn symbol(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }
}

/// Tick data from an exchange
#[derive(Debug, Clone)]
pub struct TickData {
    pub exchange: Exchange,
    pub pair: TradingPair,
    pub bid: f64,           // Best bid price
    pub ask: f64,           // Best ask price
    pub bid_volume: f64,    // Volume at best bid
    pub ask_volume: f64,    // Volume at best ask
    pub last_price: f64,    // Last trade price
    pub timestamp: u64,     // Unix timestamp in milliseconds
    pub sequence: u64,      // Update sequence number
}

impl TickData {
    /// Spread between bid and ask
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Spread as percentage
    pub fn spread_percent(&self) -> f64 {
        (self.spread() / self.mid_price()) * 100.0
    }

    /// Mid price
    pub fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

/// Aggregated price data for a pair across all exchanges
#[derive(Debug, Clone)]
pub struct AggregatedPrice {
    pub pair: TradingPair,
    pub best_bid: (Exchange, f64),      // Best bid price and exchange
    pub best_ask: (Exchange, f64),      // Best ask price and exchange
    pub prices: HashMap<Exchange, TickData>,
    pub last_update: u64,
}

impl AggregatedPrice {
    pub fn new(pair: TradingPair) -> Self {
        AggregatedPrice {
            pair,
            best_bid: (Exchange::Binance, 0.0),
            best_ask: (Exchange::Binance, f64::MAX),
            prices: HashMap::new(),
            last_update: 0,
        }
    }

    /// Update with new tick data from an exchange
    pub fn update(&mut self, tick: TickData) {
        // Update best bid (find maximum)
        if tick.bid > self.best_bid.1 {
            self.best_bid = (tick.exchange, tick.bid);
        }

        // Update best ask (find minimum)
        if tick.ask < self.best_ask.1 {
            self.best_ask = (tick.exchange, tick.ask);
        }

        self.last_update = tick.timestamp;
        self.prices.insert(tick.exchange, tick);
    }

    /// Check for arbitrage opportunity
    /// Arbitrage is possible when best_bid > best_ask on different exchanges
    pub fn arbitrage_opportunity(&self) -> Option<ArbitrageOpportunity> {
        if self.best_bid.0 != self.best_ask.0 && self.best_bid.1 > self.best_ask.1 {
            let profit_percent = ((self.best_bid.1 - self.best_ask.1) / self.best_ask.1) * 100.0;
            Some(ArbitrageOpportunity {
                pair: self.pair.clone(),
                buy_exchange: self.best_ask.0,
                buy_price: self.best_ask.1,
                sell_exchange: self.best_bid.0,
                sell_price: self.best_bid.1,
                profit_percent,
            })
        } else {
            None
        }
    }
}

/// Arbitrage opportunity
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub pair: TradingPair,
    pub buy_exchange: Exchange,
    pub buy_price: f64,
    pub sell_exchange: Exchange,
    pub sell_price: f64,
    pub profit_percent: f64,
}

impl ArbitrageOpportunity {
    pub fn display(&self) -> String {
        format!(
            "ARBITRAGE: {} - Buy on {} @ {:.2}, Sell on {} @ {:.2}, Profit: {:.4}%",
            self.pair.symbol(),
            self.buy_exchange.name(),
            self.buy_price,
            self.sell_exchange.name(),
            self.sell_price,
            self.profit_percent
        )
    }
}
```

## Part 2: Exchange Simulator

In a real project, this would be HTTP/WebSocket clients. For learning purposes, we'll create a simulator:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// Sequence generator for simulation
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Exchange API simulator
pub struct ExchangeSimulator {
    exchange: Exchange,
    base_prices: HashMap<String, f64>,
    volatility: f64,
}

impl ExchangeSimulator {
    pub fn new(exchange: Exchange) -> Self {
        let mut base_prices = HashMap::new();

        // Base prices differ slightly for each exchange
        // (simulating real-world discrepancies)
        let offset = match exchange {
            Exchange::Binance => 0.0,
            Exchange::Kraken => 0.15,
            Exchange::Coinbase => -0.10,
            Exchange::Bybit => 0.05,
        };

        base_prices.insert("BTC".to_string(), 42000.0 + offset * 100.0);
        base_prices.insert("ETH".to_string(), 2200.0 + offset * 10.0);
        base_prices.insert("SOL".to_string(), 95.0 + offset);

        ExchangeSimulator {
            exchange,
            base_prices,
            volatility: 0.001, // 0.1% volatility
        }
    }

    /// Simulate fetching tick data
    pub fn fetch_tick(&self, pair: &TradingPair) -> Result<TickData, String> {
        // Simulate API latency
        thread::sleep(Duration::from_millis(self.exchange.api_latency()));

        // Simulate random errors (5% probability)
        if rand_simple() < 0.05 {
            return Err(format!("{}: Connection timeout", self.exchange.name()));
        }

        let base_price = self.base_prices
            .get(&pair.base)
            .copied()
            .unwrap_or(100.0);

        // Add random deviation
        let random_factor = 1.0 + (rand_simple() - 0.5) * 2.0 * self.volatility;
        let mid_price = base_price * random_factor;

        // Spread depends on exchange
        let spread_percent = match self.exchange {
            Exchange::Binance => 0.0005,  // 0.05%
            Exchange::Kraken => 0.0008,   // 0.08%
            Exchange::Coinbase => 0.0010, // 0.10%
            Exchange::Bybit => 0.0006,    // 0.06%
        };

        let half_spread = mid_price * spread_percent / 2.0;
        let bid = mid_price - half_spread;
        let ask = mid_price + half_spread;

        Ok(TickData {
            exchange: self.exchange,
            pair: pair.clone(),
            bid,
            ask,
            bid_volume: rand_simple() * 10.0,
            ask_volume: rand_simple() * 10.0,
            last_price: mid_price,
            timestamp: current_timestamp_ms(),
            sequence: SEQUENCE.fetch_add(1, Ordering::SeqCst),
        })
    }
}

/// Simple random number generator (no external dependencies)
fn rand_simple() -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    thread::current().id().hash(&mut hasher);

    let hash = hasher.finish();
    (hash as f64) / (u64::MAX as f64)
}

/// Current timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
```

## Part 3: Data Collector with Channels

```rust
/// Message to the data channel
#[derive(Debug)]
pub enum DataMessage {
    Tick(TickData),
    Error { exchange: Exchange, error: String },
    Shutdown,
}

/// Collector statistics
#[derive(Debug, Default)]
pub struct CollectorStats {
    pub ticks_received: u64,
    pub errors: u64,
    pub last_tick_time: Option<Instant>,
}

/// Collector configuration
pub struct CollectorConfig {
    pub pairs: Vec<TradingPair>,
    pub update_interval: Duration,
    pub max_retries: u32,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        CollectorConfig {
            pairs: vec![
                TradingPair::new("BTC", "USDT"),
                TradingPair::new("ETH", "USDT"),
            ],
            update_interval: Duration::from_millis(500),
            max_retries: 3,
        }
    }
}

/// Data fetcher for a single exchange
pub struct ExchangeFetcher {
    exchange: Exchange,
    simulator: ExchangeSimulator,
    config: Arc<CollectorConfig>,
    sender: mpsc::Sender<DataMessage>,
    running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<CollectorStats>>,
}

impl ExchangeFetcher {
    pub fn new(
        exchange: Exchange,
        config: Arc<CollectorConfig>,
        sender: mpsc::Sender<DataMessage>,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        ExchangeFetcher {
            exchange,
            simulator: ExchangeSimulator::new(exchange),
            config,
            sender,
            running,
            stats: Arc::new(RwLock::new(CollectorStats::default())),
        }
    }

    /// Main data collection loop
    pub fn run(&self) {
        println!("[{}] Fetcher started", self.exchange.name());

        while *self.running.read().unwrap() {
            for pair in &self.config.pairs {
                if !*self.running.read().unwrap() {
                    break;
                }

                self.fetch_with_retry(pair);
            }

            thread::sleep(self.config.update_interval);
        }

        println!("[{}] Fetcher stopped", self.exchange.name());
    }

    /// Fetch data with retry logic
    fn fetch_with_retry(&self, pair: &TradingPair) {
        let mut retries = 0;

        loop {
            match self.simulator.fetch_tick(pair) {
                Ok(tick) => {
                    // Update statistics
                    {
                        let mut stats = self.stats.write().unwrap();
                        stats.ticks_received += 1;
                        stats.last_tick_time = Some(Instant::now());
                    }

                    // Send to channel
                    if self.sender.send(DataMessage::Tick(tick)).is_err() {
                        println!("[{}] Channel closed", self.exchange.name());
                        return;
                    }
                    break;
                }
                Err(error) => {
                    retries += 1;

                    {
                        let mut stats = self.stats.write().unwrap();
                        stats.errors += 1;
                    }

                    if retries >= self.config.max_retries {
                        let _ = self.sender.send(DataMessage::Error {
                            exchange: self.exchange,
                            error: error.clone(),
                        });
                        println!(
                            "[{}] Failed after {} retries: {}",
                            self.exchange.name(),
                            retries,
                            error
                        );
                        break;
                    }

                    // Exponential backoff
                    thread::sleep(Duration::from_millis(100 * retries as u64));
                }
            }
        }
    }

    pub fn get_stats(&self) -> CollectorStats {
        self.stats.read().unwrap().clone()
    }
}
```

## Part 4: Data Aggregator

```rust
/// Storage for aggregated prices
pub type PriceStorage = Arc<RwLock<HashMap<String, AggregatedPrice>>>;

/// Data aggregator from all exchanges
pub struct DataAggregator {
    receiver: mpsc::Receiver<DataMessage>,
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
    arbitrage_callback: Option<Box<dyn Fn(ArbitrageOpportunity) + Send>>,
}

impl DataAggregator {
    pub fn new(
        receiver: mpsc::Receiver<DataMessage>,
        storage: PriceStorage,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        DataAggregator {
            receiver,
            storage,
            running,
            arbitrage_callback: None,
        }
    }

    /// Set callback for arbitrage opportunities
    pub fn set_arbitrage_callback<F>(&mut self, callback: F)
    where
        F: Fn(ArbitrageOpportunity) + Send + 'static,
    {
        self.arbitrage_callback = Some(Box::new(callback));
    }

    /// Main aggregation loop
    pub fn run(&self) {
        println!("[Aggregator] Started");

        loop {
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(DataMessage::Tick(tick)) => {
                    self.process_tick(tick);
                }
                Ok(DataMessage::Error { exchange, error }) => {
                    println!("[Aggregator] Error from {}: {}", exchange.name(), error);
                }
                Ok(DataMessage::Shutdown) => {
                    println!("[Aggregator] Received shutdown signal");
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !*self.running.read().unwrap() {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    println!("[Aggregator] Channel disconnected");
                    break;
                }
            }
        }

        println!("[Aggregator] Stopped");
    }

    fn process_tick(&self, tick: TickData) {
        let symbol = tick.pair.symbol();

        let mut storage = self.storage.write().unwrap();
        let aggregated = storage
            .entry(symbol.clone())
            .or_insert_with(|| AggregatedPrice::new(tick.pair.clone()));

        aggregated.update(tick);

        // Check for arbitrage opportunities
        if let Some(opportunity) = aggregated.arbitrage_opportunity() {
            if let Some(ref callback) = self.arbitrage_callback {
                callback(opportunity);
            }
        }
    }
}
```

## Part 5: Analyzer and Monitoring

```rust
/// System statistics
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub total_ticks: u64,
    pub ticks_per_second: f64,
    pub active_pairs: usize,
    pub active_exchanges: usize,
    pub arbitrage_opportunities: u64,
    pub uptime_seconds: u64,
}

/// Market data analyzer
pub struct MarketAnalyzer {
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
    analysis_interval: Duration,
}

impl MarketAnalyzer {
    pub fn new(storage: PriceStorage, running: Arc<RwLock<bool>>) -> Self {
        MarketAnalyzer {
            storage,
            running,
            analysis_interval: Duration::from_secs(5),
        }
    }

    pub fn run(&self) {
        println!("[Analyzer] Started");

        while *self.running.read().unwrap() {
            self.analyze();
            thread::sleep(self.analysis_interval);
        }

        println!("[Analyzer] Stopped");
    }

    fn analyze(&self) {
        let storage = self.storage.read().unwrap();

        println!("\n========== Market Analysis ==========");

        for (symbol, agg) in storage.iter() {
            println!("\n{}", symbol);
            println!("  Best Bid: {} @ {:.2}", agg.best_bid.0.name(), agg.best_bid.1);
            println!("  Best Ask: {} @ {:.2}", agg.best_ask.0.name(), agg.best_ask.1);

            // Show prices from all exchanges
            for (exchange, tick) in &agg.prices {
                println!(
                    "  {} - Bid: {:.2}, Ask: {:.2}, Spread: {:.4}%",
                    exchange.name(),
                    tick.bid,
                    tick.ask,
                    tick.spread_percent()
                );
            }

            // Check for arbitrage
            if let Some(opp) = agg.arbitrage_opportunity() {
                println!("  *** {} ***", opp.display());
            }
        }

        println!("\n======================================\n");
    }
}
```

## Part 6: Main Program

```rust
use std::sync::atomic::AtomicU64;

/// Arbitrage opportunity counter
static ARBITRAGE_COUNT: AtomicU64 = AtomicU64::new(0);

/// Main data collector structure
pub struct MultiExchangeCollector {
    config: Arc<CollectorConfig>,
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
}

impl MultiExchangeCollector {
    pub fn new(config: CollectorConfig) -> Self {
        MultiExchangeCollector {
            config: Arc::new(config),
            storage: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(true)),
        }
    }

    pub fn run(&self, duration: Duration) {
        println!("Starting Multi-Exchange Data Collector...");
        println!("Collecting data for {:?}", duration);
        println!("Pairs: {:?}", self.config.pairs.iter().map(|p| p.symbol()).collect::<Vec<_>>());
        println!("Exchanges: {:?}", Exchange::all().iter().map(|e| e.name()).collect::<Vec<_>>());
        println!();

        // Create channel for data transfer
        let (sender, receiver) = mpsc::channel::<DataMessage>();

        let mut handles = vec![];

        // Start fetcher for each exchange
        for exchange in Exchange::all() {
            let fetcher = ExchangeFetcher::new(
                exchange,
                Arc::clone(&self.config),
                sender.clone(),
                Arc::clone(&self.running),
            );

            let handle = thread::spawn(move || {
                fetcher.run();
            });

            handles.push(handle);
        }

        // Start aggregator
        let storage_clone = Arc::clone(&self.storage);
        let running_clone = Arc::clone(&self.running);

        let aggregator_handle = thread::spawn(move || {
            let mut aggregator = DataAggregator::new(receiver, storage_clone, running_clone);

            // Set arbitrage callback
            aggregator.set_arbitrage_callback(|opp| {
                ARBITRAGE_COUNT.fetch_add(1, Ordering::SeqCst);
                println!("!!! ARBITRAGE DETECTED: {}", opp.display());
            });

            aggregator.run();
        });

        // Start analyzer
        let storage_clone = Arc::clone(&self.storage);
        let running_clone = Arc::clone(&self.running);

        let analyzer_handle = thread::spawn(move || {
            let analyzer = MarketAnalyzer::new(storage_clone, running_clone);
            analyzer.run();
        });

        // Wait for specified duration
        thread::sleep(duration);

        // Stop all threads
        println!("\nStopping collector...");
        *self.running.write().unwrap() = false;

        // Send shutdown signal
        let _ = sender.send(DataMessage::Shutdown);

        // Wait for all threads to complete
        for handle in handles {
            let _ = handle.join();
        }
        let _ = aggregator_handle.join();
        let _ = analyzer_handle.join();

        self.print_final_stats();
    }

    fn print_final_stats(&self) {
        let storage = self.storage.read().unwrap();

        println!("\n========== Final Statistics ==========");
        println!("Total pairs tracked: {}", storage.len());
        println!(
            "Arbitrage opportunities found: {}",
            ARBITRAGE_COUNT.load(Ordering::SeqCst)
        );

        for (symbol, agg) in storage.iter() {
            println!("\n{}", symbol);
            println!("  Exchanges with data: {}", agg.prices.len());

            let total_volume: f64 = agg.prices.values().map(|t| t.bid_volume + t.ask_volume).sum();
            println!("  Total volume: {:.2}", total_volume);

            if let Some(opp) = agg.arbitrage_opportunity() {
                println!("  Current arbitrage: {:.4}%", opp.profit_percent);
            }
        }

        println!("\n======================================");
    }
}

fn main() {
    let config = CollectorConfig {
        pairs: vec![
            TradingPair::new("BTC", "USDT"),
            TradingPair::new("ETH", "USDT"),
            TradingPair::new("SOL", "USDT"),
        ],
        update_interval: Duration::from_millis(200),
        max_retries: 3,
    };

    let collector = MultiExchangeCollector::new(config);

    // Run for 30 seconds
    collector.run(Duration::from_secs(30));
}
```

## Part 7: Complete Working Example

Here is the complete `main.rs` file that can be compiled and run:

```rust
//! Multi-Exchange Data Collector
//!
//! Example of a multi-threaded data collector from multiple exchanges.
//! Demonstrates: threads, channels, synchronization, RwLock, Arc.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ==================== Basic Structures ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Exchange {
    Binance,
    Kraken,
    Coinbase,
    Bybit,
}

impl Exchange {
    pub fn all() -> Vec<Exchange> {
        vec![
            Exchange::Binance,
            Exchange::Kraken,
            Exchange::Coinbase,
            Exchange::Bybit,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Exchange::Binance => "Binance",
            Exchange::Kraken => "Kraken",
            Exchange::Coinbase => "Coinbase",
            Exchange::Bybit => "Bybit",
        }
    }

    pub fn api_latency(&self) -> u64 {
        match self {
            Exchange::Binance => 50,
            Exchange::Kraken => 80,
            Exchange::Coinbase => 60,
            Exchange::Bybit => 70,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradingPair {
    pub base: String,
    pub quote: String,
}

impl TradingPair {
    pub fn new(base: &str, quote: &str) -> Self {
        TradingPair {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }

    pub fn symbol(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }
}

#[derive(Debug, Clone)]
pub struct TickData {
    pub exchange: Exchange,
    pub pair: TradingPair,
    pub bid: f64,
    pub ask: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
    pub last_price: f64,
    pub timestamp: u64,
    pub sequence: u64,
}

impl TickData {
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    pub fn spread_percent(&self) -> f64 {
        (self.spread() / self.mid_price()) * 100.0
    }

    pub fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }
}

#[derive(Debug, Clone)]
pub struct AggregatedPrice {
    pub pair: TradingPair,
    pub best_bid: (Exchange, f64),
    pub best_ask: (Exchange, f64),
    pub prices: HashMap<Exchange, TickData>,
    pub last_update: u64,
}

impl AggregatedPrice {
    pub fn new(pair: TradingPair) -> Self {
        AggregatedPrice {
            pair,
            best_bid: (Exchange::Binance, 0.0),
            best_ask: (Exchange::Binance, f64::MAX),
            prices: HashMap::new(),
            last_update: 0,
        }
    }

    pub fn update(&mut self, tick: TickData) {
        if tick.bid > self.best_bid.1 {
            self.best_bid = (tick.exchange, tick.bid);
        }
        if tick.ask < self.best_ask.1 {
            self.best_ask = (tick.exchange, tick.ask);
        }
        self.last_update = tick.timestamp;
        self.prices.insert(tick.exchange, tick);
    }

    pub fn arbitrage_opportunity(&self) -> Option<ArbitrageOpportunity> {
        if self.best_bid.0 != self.best_ask.0 && self.best_bid.1 > self.best_ask.1 {
            let profit_percent =
                ((self.best_bid.1 - self.best_ask.1) / self.best_ask.1) * 100.0;
            Some(ArbitrageOpportunity {
                pair: self.pair.clone(),
                buy_exchange: self.best_ask.0,
                buy_price: self.best_ask.1,
                sell_exchange: self.best_bid.0,
                sell_price: self.best_bid.1,
                profit_percent,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub pair: TradingPair,
    pub buy_exchange: Exchange,
    pub buy_price: f64,
    pub sell_exchange: Exchange,
    pub sell_price: f64,
    pub profit_percent: f64,
}

impl ArbitrageOpportunity {
    pub fn display(&self) -> String {
        format!(
            "{} - Buy {} @ {:.2}, Sell {} @ {:.2}, Profit: {:.4}%",
            self.pair.symbol(),
            self.buy_exchange.name(),
            self.buy_price,
            self.sell_exchange.name(),
            self.sell_price,
            self.profit_percent
        )
    }
}

// ==================== Simulator ====================

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
static RAND_STATE: AtomicU64 = AtomicU64::new(12345);

fn rand_simple() -> f64 {
    let mut state = RAND_STATE.load(Ordering::Relaxed);
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    RAND_STATE.store(state, Ordering::Relaxed);
    (state >> 33) as f64 / (u32::MAX as f64)
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub struct ExchangeSimulator {
    exchange: Exchange,
    base_prices: HashMap<String, f64>,
    volatility: f64,
}

impl ExchangeSimulator {
    pub fn new(exchange: Exchange) -> Self {
        let mut base_prices = HashMap::new();
        let offset = match exchange {
            Exchange::Binance => 0.0,
            Exchange::Kraken => 0.15,
            Exchange::Coinbase => -0.10,
            Exchange::Bybit => 0.05,
        };
        base_prices.insert("BTC".to_string(), 42000.0 + offset * 100.0);
        base_prices.insert("ETH".to_string(), 2200.0 + offset * 10.0);
        base_prices.insert("SOL".to_string(), 95.0 + offset);

        ExchangeSimulator {
            exchange,
            base_prices,
            volatility: 0.002,
        }
    }

    pub fn fetch_tick(&self, pair: &TradingPair) -> Result<TickData, String> {
        thread::sleep(Duration::from_millis(self.exchange.api_latency()));

        if rand_simple() < 0.02 {
            return Err(format!("{}: Connection timeout", self.exchange.name()));
        }

        let base_price = self.base_prices.get(&pair.base).copied().unwrap_or(100.0);
        let random_factor = 1.0 + (rand_simple() - 0.5) * 2.0 * self.volatility;
        let mid_price = base_price * random_factor;

        let spread_percent = match self.exchange {
            Exchange::Binance => 0.0005,
            Exchange::Kraken => 0.0008,
            Exchange::Coinbase => 0.0010,
            Exchange::Bybit => 0.0006,
        };

        let half_spread = mid_price * spread_percent / 2.0;

        Ok(TickData {
            exchange: self.exchange,
            pair: pair.clone(),
            bid: mid_price - half_spread,
            ask: mid_price + half_spread,
            bid_volume: rand_simple() * 10.0,
            ask_volume: rand_simple() * 10.0,
            last_price: mid_price,
            timestamp: current_timestamp_ms(),
            sequence: SEQUENCE.fetch_add(1, Ordering::SeqCst),
        })
    }
}

// ==================== Messages and Collector ====================

#[derive(Debug)]
pub enum DataMessage {
    Tick(TickData),
    Error { exchange: Exchange, error: String },
    Shutdown,
}

pub struct CollectorConfig {
    pub pairs: Vec<TradingPair>,
    pub update_interval: Duration,
    pub max_retries: u32,
}

pub struct ExchangeFetcher {
    exchange: Exchange,
    simulator: ExchangeSimulator,
    config: Arc<CollectorConfig>,
    sender: mpsc::Sender<DataMessage>,
    running: Arc<RwLock<bool>>,
}

impl ExchangeFetcher {
    pub fn new(
        exchange: Exchange,
        config: Arc<CollectorConfig>,
        sender: mpsc::Sender<DataMessage>,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        ExchangeFetcher {
            exchange,
            simulator: ExchangeSimulator::new(exchange),
            config,
            sender,
            running,
        }
    }

    pub fn run(&self) {
        println!("[{}] Fetcher started", self.exchange.name());

        while *self.running.read().unwrap() {
            for pair in &self.config.pairs {
                if !*self.running.read().unwrap() {
                    break;
                }
                self.fetch_with_retry(pair);
            }
            thread::sleep(self.config.update_interval);
        }

        println!("[{}] Fetcher stopped", self.exchange.name());
    }

    fn fetch_with_retry(&self, pair: &TradingPair) {
        let mut retries = 0;
        loop {
            match self.simulator.fetch_tick(pair) {
                Ok(tick) => {
                    if self.sender.send(DataMessage::Tick(tick)).is_err() {
                        return;
                    }
                    break;
                }
                Err(error) => {
                    retries += 1;
                    if retries >= self.config.max_retries {
                        let _ = self.sender.send(DataMessage::Error {
                            exchange: self.exchange,
                            error,
                        });
                        break;
                    }
                    thread::sleep(Duration::from_millis(50 * retries as u64));
                }
            }
        }
    }
}

// ==================== Aggregator ====================

pub type PriceStorage = Arc<RwLock<HashMap<String, AggregatedPrice>>>;

static ARBITRAGE_COUNT: AtomicU64 = AtomicU64::new(0);

pub struct DataAggregator {
    receiver: mpsc::Receiver<DataMessage>,
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
}

impl DataAggregator {
    pub fn new(
        receiver: mpsc::Receiver<DataMessage>,
        storage: PriceStorage,
        running: Arc<RwLock<bool>>,
    ) -> Self {
        DataAggregator {
            receiver,
            storage,
            running,
        }
    }

    pub fn run(&self) {
        println!("[Aggregator] Started");

        loop {
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(DataMessage::Tick(tick)) => {
                    self.process_tick(tick);
                }
                Ok(DataMessage::Error { exchange, error }) => {
                    println!("[Aggregator] Error from {}: {}", exchange.name(), error);
                }
                Ok(DataMessage::Shutdown) => {
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !*self.running.read().unwrap() {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        println!("[Aggregator] Stopped");
    }

    fn process_tick(&self, tick: TickData) {
        let symbol = tick.pair.symbol();
        let mut storage = self.storage.write().unwrap();
        let aggregated = storage
            .entry(symbol)
            .or_insert_with(|| AggregatedPrice::new(tick.pair.clone()));

        aggregated.update(tick);

        if let Some(opp) = aggregated.arbitrage_opportunity() {
            ARBITRAGE_COUNT.fetch_add(1, Ordering::SeqCst);
            println!(">>> ARBITRAGE: {}", opp.display());
        }
    }
}

// ==================== Analyzer ====================

pub struct MarketAnalyzer {
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
}

impl MarketAnalyzer {
    pub fn new(storage: PriceStorage, running: Arc<RwLock<bool>>) -> Self {
        MarketAnalyzer { storage, running }
    }

    pub fn run(&self) {
        println!("[Analyzer] Started");

        while *self.running.read().unwrap() {
            thread::sleep(Duration::from_secs(5));
            self.analyze();
        }

        println!("[Analyzer] Stopped");
    }

    fn analyze(&self) {
        let storage = self.storage.read().unwrap();
        println!("\n========== Market Analysis ==========");

        for (symbol, agg) in storage.iter() {
            println!("\n{}", symbol);
            println!(
                "  Best Bid: {} @ {:.2}",
                agg.best_bid.0.name(),
                agg.best_bid.1
            );
            println!(
                "  Best Ask: {} @ {:.2}",
                agg.best_ask.0.name(),
                agg.best_ask.1
            );

            for (exchange, tick) in &agg.prices {
                println!(
                    "    {} - Bid: {:.2}, Ask: {:.2}, Spread: {:.4}%",
                    exchange.name(),
                    tick.bid,
                    tick.ask,
                    tick.spread_percent()
                );
            }
        }

        println!("\n======================================\n");
    }
}

// ==================== Main Collector ====================

pub struct MultiExchangeCollector {
    config: Arc<CollectorConfig>,
    storage: PriceStorage,
    running: Arc<RwLock<bool>>,
}

impl MultiExchangeCollector {
    pub fn new(config: CollectorConfig) -> Self {
        MultiExchangeCollector {
            config: Arc::new(config),
            storage: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(true)),
        }
    }

    pub fn run(&self, duration: Duration) {
        println!("=== Multi-Exchange Data Collector ===");
        println!("Duration: {:?}", duration);
        println!(
            "Pairs: {:?}",
            self.config.pairs.iter().map(|p| p.symbol()).collect::<Vec<_>>()
        );
        println!(
            "Exchanges: {:?}\n",
            Exchange::all().iter().map(|e| e.name()).collect::<Vec<_>>()
        );

        let (sender, receiver) = mpsc::channel::<DataMessage>();
        let mut handles = vec![];

        // Start fetchers
        for exchange in Exchange::all() {
            let fetcher = ExchangeFetcher::new(
                exchange,
                Arc::clone(&self.config),
                sender.clone(),
                Arc::clone(&self.running),
            );
            handles.push(thread::spawn(move || fetcher.run()));
        }

        // Start aggregator
        let storage_clone = Arc::clone(&self.storage);
        let running_clone = Arc::clone(&self.running);
        let aggregator_handle = thread::spawn(move || {
            let aggregator = DataAggregator::new(receiver, storage_clone, running_clone);
            aggregator.run();
        });

        // Start analyzer
        let storage_clone = Arc::clone(&self.storage);
        let running_clone = Arc::clone(&self.running);
        let analyzer_handle = thread::spawn(move || {
            let analyzer = MarketAnalyzer::new(storage_clone, running_clone);
            analyzer.run();
        });

        // Wait
        thread::sleep(duration);

        // Stop
        println!("\nStopping...");
        *self.running.write().unwrap() = false;
        let _ = sender.send(DataMessage::Shutdown);

        for handle in handles {
            let _ = handle.join();
        }
        let _ = aggregator_handle.join();
        let _ = analyzer_handle.join();

        self.print_stats();
    }

    fn print_stats(&self) {
        let storage = self.storage.read().unwrap();
        println!("\n========== Final Statistics ==========");
        println!("Pairs tracked: {}", storage.len());
        println!(
            "Arbitrage opportunities: {}",
            ARBITRAGE_COUNT.load(Ordering::SeqCst)
        );

        for (symbol, agg) in storage.iter() {
            println!("\n{}: {} exchanges", symbol, agg.prices.len());
        }
        println!("\n======================================");
    }
}

fn main() {
    let config = CollectorConfig {
        pairs: vec![
            TradingPair::new("BTC", "USDT"),
            TradingPair::new("ETH", "USDT"),
            TradingPair::new("SOL", "USDT"),
        ],
        update_interval: Duration::from_millis(200),
        max_retries: 3,
    };

    let collector = MultiExchangeCollector::new(config);
    collector.run(Duration::from_secs(15));
}
```

## What We Learned

| Concept | Application in Project |
|---------|----------------------|
| `thread::spawn` | Creating separate threads for each exchange |
| `mpsc::channel` | Passing tick data from fetchers to aggregator |
| `Arc<RwLock<T>>` | Shared price storage with multiple readers |
| `Arc<Mutex<T>>` | Thread-safe counters and statistics |
| `AtomicU64` | Lightweight atomic counters without locks |
| Graceful shutdown | Properly stopping all threads |
| Error handling | Retries and API error handling |
| Callback pattern | Notifications for arbitrage opportunities |

## Homework

### Exercise 1: Adding a New Exchange

Add support for another exchange (e.g., OKX or Huobi):
- Add a new variant to the `Exchange` enum
- Configure simulation parameters (latency, spread, base prices)
- Make sure the new exchange participates in arbitrage calculations

### Exercise 2: Improved Arbitrage Detector

Modify `ArbitrageOpportunity` to account for:
- Exchange fees (maker/taker fees)
- Minimum volume for a trade
- Time validity of the opportunity (staleness check)

```rust
pub struct ImprovedArbitrage {
    // ... base fields ...
    pub maker_fee: f64,      // 0.1%
    pub taker_fee: f64,      // 0.1%
    pub min_volume: f64,     // minimum volume
    pub net_profit: f64,     // profit after fees
    pub is_profitable: bool, // profitable after fees?
}
```

### Exercise 3: Persistent Storage

Add price history saving to a file:
- Every N seconds, write current prices to CSV
- Format: `timestamp,exchange,pair,bid,ask,volume`
- Use a separate thread for writing

### Exercise 4: Rate Limiting

Implement request rate limiting for exchanges:
- No more than N requests per second per exchange
- Use `std::sync::Condvar` for waiting
- Add metrics: how many requests were delayed

```rust
pub struct RateLimiter {
    max_requests_per_second: u32,
    requests: Arc<Mutex<VecDeque<Instant>>>,
}

impl RateLimiter {
    pub fn acquire(&self) {
        // Wait if rate limit exceeded
    }
}
```

### Exercise 5: System Health Monitoring

Create a monitoring component:
- Track how long since the last tick from each exchange
- If an exchange doesn't respond for > 5 seconds — display a warning
- Keep statistics: uptime for each exchange, average latency

## Navigation

[← Day 180: Thread Profiling](../180-thread-profiling/en.md) | [Day 182: Sync vs Async →](../182-sync-vs-async/en.md)
