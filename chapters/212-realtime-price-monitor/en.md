# Day 212: Project: Real-time Price Monitor

## Trading Analogy

Imagine a trader watching multiple exchanges simultaneously. They have several monitors: one showing Binance prices, another showing Coinbase, and a third showing Kraken. They see prices in real-time, receive alerts about sudden movements, and can quickly react to arbitrage opportunities.

Our project is exactly such a price monitor. We'll combine all the knowledge from this month: async/await, tokio, WebSocket, HTTP requests, channels, and graceful shutdown — into a complete application for monitoring cryptocurrency prices in real-time.

## Project Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Real-time Price Monitor                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐        │
│  │   Binance    │   │   Coinbase   │   │   Mock API   │        │
│  │  WebSocket   │   │   HTTP API   │   │  (fallback)  │        │
│  └──────┬───────┘   └──────┬───────┘   └──────┬───────┘        │
│         │                  │                  │                 │
│         └────────┬─────────┴─────────┬────────┘                 │
│                  │                   │                          │
│         ┌────────▼───────────────────▼────────┐                 │
│         │         Price Aggregator             │                 │
│         │    (broadcast channel sender)        │                 │
│         └────────────────┬─────────────────────┘                 │
│                          │                                       │
│         ┌────────────────┼────────────────┐                     │
│         │                │                │                     │
│  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐             │
│  │   Console   │  │    Alert    │  │  Statistics │             │
│  │   Display   │  │   Service   │  │   Tracker   │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │              Graceful Shutdown Handler                      ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Project Dependencies

```toml
# Cargo.toml
[package]
name = "price-monitor"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.21", features = ["native-tls"] }
futures-util = "0.3"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
url = "2"
```

## Core Data Structures

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Price source
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PriceSource {
    Binance,
    Coinbase,
    Mock,
}

impl std::fmt::Display for PriceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PriceSource::Binance => write!(f, "Binance"),
            PriceSource::Coinbase => write!(f, "Coinbase"),
            PriceSource::Mock => write!(f, "Mock"),
        }
    }
}

/// Price update from a source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub symbol: String,
    pub price: f64,
    pub source: PriceSource,
    pub timestamp: DateTime<Utc>,
    pub volume_24h: Option<f64>,
}

impl PriceUpdate {
    pub fn new(symbol: String, price: f64, source: PriceSource) -> Self {
        Self {
            symbol,
            price,
            source,
            timestamp: Utc::now(),
            volume_24h: None,
        }
    }

    pub fn with_volume(mut self, volume: f64) -> Self {
        self.volume_24h = Some(volume);
        self
    }
}

/// Price event alert
#[derive(Debug, Clone)]
pub struct PriceAlert {
    pub symbol: String,
    pub alert_type: AlertType,
    pub current_price: f64,
    pub threshold: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    PriceAbove,
    PriceBelow,
    PercentageChange { change_percent: f64 },
    ArbitrageOpportunity { low_source: PriceSource, high_source: PriceSource },
}

/// Statistics per symbol
#[derive(Debug, Clone, Default)]
pub struct PriceStatistics {
    pub symbol: String,
    pub high_24h: f64,
    pub low_24h: f64,
    pub avg_price: f64,
    pub update_count: u64,
    pub last_update: Option<DateTime<Utc>>,
    pub prices_by_source: HashMap<PriceSource, f64>,
}

impl PriceStatistics {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            high_24h: f64::MIN,
            low_24h: f64::MAX,
            ..Default::default()
        }
    }

    pub fn update(&mut self, price_update: &PriceUpdate) {
        self.high_24h = self.high_24h.max(price_update.price);
        self.low_24h = self.low_24h.min(price_update.price);
        self.update_count += 1;

        // Simple moving average
        let old_sum = self.avg_price * (self.update_count - 1) as f64;
        self.avg_price = (old_sum + price_update.price) / self.update_count as f64;

        self.last_update = Some(price_update.timestamp);
        self.prices_by_source.insert(price_update.source.clone(), price_update.price);
    }

    /// Check for arbitrage opportunity between exchanges
    pub fn check_arbitrage(&self, min_spread_percent: f64) -> Option<(PriceSource, PriceSource, f64)> {
        if self.prices_by_source.len() < 2 {
            return None;
        }

        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;
        let mut min_source = None;
        let mut max_source = None;

        for (source, &price) in &self.prices_by_source {
            if price < min_price {
                min_price = price;
                min_source = Some(source.clone());
            }
            if price > max_price {
                max_price = price;
                max_source = Some(source.clone());
            }
        }

        let spread_percent = (max_price - min_price) / min_price * 100.0;

        if spread_percent >= min_spread_percent {
            Some((min_source.unwrap(), max_source.unwrap(), spread_percent))
        } else {
            None
        }
    }
}
```

## WebSocket Client for Binance

```rust
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

/// Binance WebSocket client
pub struct BinanceWebSocket {
    symbols: Vec<String>,
    price_sender: broadcast::Sender<PriceUpdate>,
}

impl BinanceWebSocket {
    pub fn new(symbols: Vec<String>, price_sender: broadcast::Sender<PriceUpdate>) -> Self {
        Self { symbols, price_sender }
    }

    /// Start WebSocket connection with automatic reconnection
    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) {
        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Binance WebSocket: shutdown signal received");
                    break;
                }
                result = self.connect_and_listen() => {
                    match result {
                        Ok(_) => {
                            info!("Binance WebSocket: connection closed normally");
                        }
                        Err(e) => {
                            error!("Binance WebSocket: connection error: {}", e);
                        }
                    }
                    // Reconnect after 5 seconds
                    info!("Binance WebSocket: reconnecting in 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect_and_listen(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Build URL for subscribing to multiple symbols
        let streams: Vec<String> = self.symbols
            .iter()
            .map(|s| format!("{}@trade", s.to_lowercase()))
            .collect();

        let url = format!(
            "wss://stream.binance.com:9443/stream?streams={}",
            streams.join("/")
        );

        info!("Binance WebSocket: connecting to {}", url);

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();

        info!("Binance WebSocket: connected!");

        // Set up ping interval
        let ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        tokio::pin!(ping_interval);

        loop {
            tokio::select! {
                // Send ping every 30 seconds
                _ = ping_interval.tick() => {
                    write.send(Message::Ping(vec![])).await?;
                }

                // Receive messages
                message = read.next() => {
                    match message {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = self.process_message(&text) {
                                warn!("Binance: error processing message: {}", e);
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            // Pong received, connection is alive
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("Binance WebSocket: server closed connection");
                            break;
                        }
                        Some(Err(e)) => {
                            error!("Binance WebSocket: error: {}", e);
                            break;
                        }
                        None => {
                            info!("Binance WebSocket: stream ended");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn process_message(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[derive(Deserialize)]
        struct BinanceStreamMessage {
            stream: String,
            data: BinanceTradeData,
        }

        #[derive(Deserialize)]
        struct BinanceTradeData {
            #[serde(rename = "s")]
            symbol: String,
            #[serde(rename = "p")]
            price: String,
            #[serde(rename = "q")]
            quantity: String,
        }

        let msg: BinanceStreamMessage = serde_json::from_str(text)?;
        let price: f64 = msg.data.price.parse()?;
        let volume: f64 = msg.data.quantity.parse()?;

        let update = PriceUpdate::new(
            msg.data.symbol,
            price,
            PriceSource::Binance,
        ).with_volume(volume);

        // Send update to all subscribers
        let _ = self.price_sender.send(update);

        Ok(())
    }
}
```

## HTTP Client for Coinbase

```rust
use reqwest::Client;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

/// HTTP client for fetching prices from Coinbase
pub struct CoinbaseHttpClient {
    client: Client,
    symbols: Vec<String>,
    price_sender: broadcast::Sender<PriceUpdate>,
    poll_interval: Duration,
}

impl CoinbaseHttpClient {
    pub fn new(
        symbols: Vec<String>,
        price_sender: broadcast::Sender<PriceUpdate>,
        poll_interval: Duration,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            symbols,
            price_sender,
            poll_interval,
        }
    }

    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) {
        let mut interval = tokio::time::interval(self.poll_interval);

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Coinbase HTTP: shutdown signal received");
                    break;
                }
                _ = interval.tick() => {
                    self.fetch_all_prices().await;
                }
            }
        }
    }

    async fn fetch_all_prices(&self) {
        for symbol in &self.symbols {
            match self.fetch_price(symbol).await {
                Ok(update) => {
                    let _ = self.price_sender.send(update);
                }
                Err(e) => {
                    warn!("Coinbase: error fetching price for {}: {}", symbol, e);
                }
            }
        }
    }

    async fn fetch_price(&self, symbol: &str) -> Result<PriceUpdate, Box<dyn std::error::Error + Send + Sync>> {
        #[derive(Deserialize)]
        struct CoinbaseTickerResponse {
            price: String,
            volume: String,
        }

        // Convert symbol (BTCUSDT -> BTC-USD)
        let product_id = self.convert_symbol(symbol);
        let url = format!("https://api.coinbase.com/v2/prices/{}/spot", product_id);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        #[derive(Deserialize)]
        struct CoinbaseResponse {
            data: CoinbaseData,
        }

        #[derive(Deserialize)]
        struct CoinbaseData {
            amount: String,
        }

        let data: CoinbaseResponse = response.json().await?;
        let price: f64 = data.data.amount.parse()?;

        Ok(PriceUpdate::new(
            symbol.to_string(),
            price,
            PriceSource::Coinbase,
        ))
    }

    fn convert_symbol(&self, symbol: &str) -> String {
        // Simple conversion BTCUSDT -> BTC-USD
        if symbol.ends_with("USDT") {
            let base = symbol.trim_end_matches("USDT");
            format!("{}-USD", base)
        } else {
            symbol.to_string()
        }
    }
}
```

## Mock API for Testing

```rust
use rand::Rng;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::info;

/// Mock price source for testing without real API
pub struct MockPriceSource {
    symbols: Vec<(String, f64)>, // (symbol, base price)
    price_sender: broadcast::Sender<PriceUpdate>,
    update_interval: Duration,
}

impl MockPriceSource {
    pub fn new(
        symbols: Vec<(String, f64)>,
        price_sender: broadcast::Sender<PriceUpdate>,
        update_interval: Duration,
    ) -> Self {
        Self {
            symbols,
            price_sender,
            update_interval,
        }
    }

    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) {
        let mut interval = tokio::time::interval(self.update_interval);
        let mut rng = rand::thread_rng();

        info!("Mock price source started");

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Mock: shutdown signal received");
                    break;
                }
                _ = interval.tick() => {
                    for (symbol, base_price) in &self.symbols {
                        // Generate random price change ±2%
                        let change: f64 = rng.gen_range(-0.02..0.02);
                        let price = base_price * (1.0 + change);
                        let volume: f64 = rng.gen_range(0.1..10.0);

                        let update = PriceUpdate::new(
                            symbol.clone(),
                            price,
                            PriceSource::Mock,
                        ).with_volume(volume);

                        let _ = self.price_sender.send(update);
                    }
                }
            }
        }
    }
}
```

## Alert Service

```rust
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

/// Alert configuration
#[derive(Debug, Clone)]
pub struct AlertConfig {
    pub symbol: String,
    pub price_above: Option<f64>,
    pub price_below: Option<f64>,
    pub percent_change_threshold: Option<f64>,
}

/// Alert service
pub struct AlertService {
    configs: Vec<AlertConfig>,
    last_prices: HashMap<String, f64>,
    alert_sender: broadcast::Sender<PriceAlert>,
}

impl AlertService {
    pub fn new(configs: Vec<AlertConfig>, alert_sender: broadcast::Sender<PriceAlert>) -> Self {
        Self {
            configs,
            last_prices: HashMap::new(),
            alert_sender,
        }
    }

    pub async fn run(&mut self, mut price_receiver: broadcast::Receiver<PriceUpdate>, mut shutdown: broadcast::Receiver<()>) {
        info!("Alert service started");

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("AlertService: shutdown signal received");
                    break;
                }
                result = price_receiver.recv() => {
                    match result {
                        Ok(update) => self.check_alerts(&update),
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            info!("AlertService: {} messages skipped", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("AlertService: channel closed");
                            break;
                        }
                    }
                }
            }
        }
    }

    fn check_alerts(&mut self, update: &PriceUpdate) {
        for config in &self.configs {
            if config.symbol != update.symbol {
                continue;
            }

            // Check price above threshold
            if let Some(threshold) = config.price_above {
                if update.price > threshold {
                    self.send_alert(PriceAlert {
                        symbol: update.symbol.clone(),
                        alert_type: AlertType::PriceAbove,
                        current_price: update.price,
                        threshold,
                        timestamp: update.timestamp,
                    });
                }
            }

            // Check price below threshold
            if let Some(threshold) = config.price_below {
                if update.price < threshold {
                    self.send_alert(PriceAlert {
                        symbol: update.symbol.clone(),
                        alert_type: AlertType::PriceBelow,
                        current_price: update.price,
                        threshold,
                        timestamp: update.timestamp,
                    });
                }
            }

            // Check percentage change
            if let Some(percent_threshold) = config.percent_change_threshold {
                if let Some(&last_price) = self.last_prices.get(&update.symbol) {
                    let change_percent = (update.price - last_price) / last_price * 100.0;

                    if change_percent.abs() >= percent_threshold {
                        self.send_alert(PriceAlert {
                            symbol: update.symbol.clone(),
                            alert_type: AlertType::PercentageChange { change_percent },
                            current_price: update.price,
                            threshold: percent_threshold,
                            timestamp: update.timestamp,
                        });
                    }
                }
            }
        }

        // Update last price
        self.last_prices.insert(update.symbol.clone(), update.price);
    }

    fn send_alert(&self, alert: PriceAlert) {
        info!("ALERT: {:?}", alert);
        let _ = self.alert_sender.send(alert);
    }
}
```

## Statistics Tracker

```rust
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::info;

/// Price statistics tracker
pub struct StatisticsTracker {
    stats: HashMap<String, PriceStatistics>,
    min_arbitrage_spread: f64,
}

impl StatisticsTracker {
    pub fn new(symbols: Vec<String>, min_arbitrage_spread: f64) -> Self {
        let mut stats = HashMap::new();
        for symbol in symbols {
            stats.insert(symbol.clone(), PriceStatistics::new(symbol));
        }
        Self { stats, min_arbitrage_spread }
    }

    pub async fn run(
        &mut self,
        mut price_receiver: broadcast::Receiver<PriceUpdate>,
        alert_sender: broadcast::Sender<PriceAlert>,
        mut shutdown: broadcast::Receiver<()>,
    ) {
        info!("Statistics tracker started");

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("StatisticsTracker: shutdown signal received");
                    self.print_final_stats();
                    break;
                }
                result = price_receiver.recv() => {
                    match result {
                        Ok(update) => {
                            self.process_update(&update, &alert_sender);
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            info!("StatisticsTracker: {} messages skipped", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn process_update(&mut self, update: &PriceUpdate, alert_sender: &broadcast::Sender<PriceAlert>) {
        if let Some(stats) = self.stats.get_mut(&update.symbol) {
            stats.update(update);

            // Check for arbitrage opportunity
            if let Some((low_source, high_source, spread)) =
                stats.check_arbitrage(self.min_arbitrage_spread)
            {
                let alert = PriceAlert {
                    symbol: update.symbol.clone(),
                    alert_type: AlertType::ArbitrageOpportunity { low_source, high_source },
                    current_price: update.price,
                    threshold: spread,
                    timestamp: update.timestamp,
                };
                let _ = alert_sender.send(alert);
            }
        }
    }

    fn print_final_stats(&self) {
        println!("\nFinal Statistics:");
        println!("{:-<60}", "");

        for (symbol, stats) in &self.stats {
            println!("Symbol: {}", symbol);
            println!("  High 24h: {:.2}", stats.high_24h);
            println!("  Low 24h:  {:.2}", stats.low_24h);
            println!("  Average:  {:.2}", stats.avg_price);
            println!("  Updates:  {}", stats.update_count);
            println!("  Prices by source:");
            for (source, price) in &stats.prices_by_source {
                println!("    {}: {:.2}", source, price);
            }
            println!();
        }
    }

    pub fn get_stats(&self) -> &HashMap<String, PriceStatistics> {
        &self.stats
    }
}
```

## Console Display

```rust
use std::io::{self, Write};
use tokio::sync::broadcast;
use tracing::info;

/// Console price display
pub struct ConsoleDisplay {
    update_rate: std::time::Duration,
}

impl ConsoleDisplay {
    pub fn new(update_rate: std::time::Duration) -> Self {
        Self { update_rate }
    }

    pub async fn run(
        &self,
        mut price_receiver: broadcast::Receiver<PriceUpdate>,
        mut alert_receiver: broadcast::Receiver<PriceAlert>,
        mut shutdown: broadcast::Receiver<()>,
    ) {
        info!("Console display started");

        let mut last_prices: std::collections::HashMap<(String, PriceSource), PriceUpdate> =
            std::collections::HashMap::new();
        let mut interval = tokio::time::interval(self.update_rate);

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("ConsoleDisplay: shutdown signal received");
                    break;
                }

                // Receive price updates
                result = price_receiver.recv() => {
                    match result {
                        Ok(update) => {
                            let key = (update.symbol.clone(), update.source.clone());
                            last_prices.insert(key, update);
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {}
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }

                // Receive alerts
                result = alert_receiver.recv() => {
                    if let Ok(alert) = result {
                        self.display_alert(&alert);
                    }
                }

                // Update display
                _ = interval.tick() => {
                    self.render(&last_prices);
                }
            }
        }
    }

    fn render(&self, prices: &std::collections::HashMap<(String, PriceSource), PriceUpdate>) {
        // Clear screen (ANSI escape code)
        print!("\x1B[2J\x1B[1;1H");

        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║             Real-time Price Monitor                        ║");
        println!("╠════════════════════════════════════════════════════════════╣");
        println!("║ {:^10} │ {:^12} │ {:^12} │ {:^18} ║",
                 "Symbol", "Source", "Price", "Time");
        println!("╠════════════════════════════════════════════════════════════╣");

        let mut sorted_prices: Vec<_> = prices.values().collect();
        sorted_prices.sort_by(|a, b| a.symbol.cmp(&b.symbol));

        for update in sorted_prices {
            let time_str = update.timestamp.format("%H:%M:%S").to_string();
            println!("║ {:^10} │ {:^12} │ {:>12.2} │ {:^18} ║",
                     update.symbol,
                     update.source.to_string(),
                     update.price,
                     time_str);
        }

        println!("╚════════════════════════════════════════════════════════════╝");
        println!("\nPress Ctrl+C to exit...");

        io::stdout().flush().unwrap();
    }

    fn display_alert(&self, alert: &PriceAlert) {
        let alert_msg = match &alert.alert_type {
            AlertType::PriceAbove => {
                format!("Price above: {} exceeded {:.2}! Current: {:.2}",
                        alert.symbol, alert.threshold, alert.current_price)
            }
            AlertType::PriceBelow => {
                format!("Price below: {} dropped below {:.2}! Current: {:.2}",
                        alert.symbol, alert.threshold, alert.current_price)
            }
            AlertType::PercentageChange { change_percent } => {
                let direction = if *change_percent > 0.0 { "UP" } else { "DOWN" };
                format!("{} {} changed by {:.2}%! Current: {:.2}",
                        direction, alert.symbol, change_percent, alert.current_price)
            }
            AlertType::ArbitrageOpportunity { low_source, high_source } => {
                format!("ARBITRAGE {}! Buy on {}, sell on {}. Spread: {:.2}%",
                        alert.symbol, low_source, high_source, alert.threshold)
            }
        };

        println!("\nALERT: {}", alert_msg);
    }
}
```

## Main Application File

```rust
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Module imports (in a real project these would be in separate files)
mod types;
mod binance_ws;
mod coinbase_http;
mod mock_source;
mod alert_service;
mod statistics;
mod display;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("price_monitor=info".parse()?))
        .init();

    info!("Starting Real-time Price Monitor");

    // Create communication channels
    let (price_tx, _) = broadcast::channel::<PriceUpdate>(1000);
    let (alert_tx, _) = broadcast::channel::<PriceAlert>(100);
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    // List of symbols to track
    let symbols = vec![
        "BTCUSDT".to_string(),
        "ETHUSDT".to_string(),
        "BNBUSDT".to_string(),
    ];

    // Alert configuration
    let alert_configs = vec![
        AlertConfig {
            symbol: "BTCUSDT".to_string(),
            price_above: Some(100000.0),
            price_below: Some(90000.0),
            percent_change_threshold: Some(1.0),
        },
        AlertConfig {
            symbol: "ETHUSDT".to_string(),
            price_above: Some(4000.0),
            price_below: Some(3000.0),
            percent_change_threshold: Some(2.0),
        },
    ];

    // Create and start Mock source (for demonstration)
    let mock_symbols = vec![
        ("BTCUSDT".to_string(), 95000.0),
        ("ETHUSDT".to_string(), 3500.0),
        ("BNBUSDT".to_string(), 600.0),
    ];
    let mock_source = MockPriceSource::new(
        mock_symbols,
        price_tx.clone(),
        Duration::from_millis(500),
    );

    // Create alert service
    let mut alert_service = AlertService::new(
        alert_configs,
        alert_tx.clone(),
    );

    // Create statistics tracker
    let mut stats_tracker = StatisticsTracker::new(
        symbols.clone(),
        0.5, // Minimum spread for arbitrage 0.5%
    );

    // Create console display
    let console_display = ConsoleDisplay::new(Duration::from_millis(200));

    // Start all tasks
    let mock_handle = tokio::spawn({
        let shutdown_rx = shutdown_tx.subscribe();
        async move {
            mock_source.run(shutdown_rx).await;
        }
    });

    let alert_handle = tokio::spawn({
        let price_rx = price_tx.subscribe();
        let shutdown_rx = shutdown_tx.subscribe();
        async move {
            alert_service.run(price_rx, shutdown_rx).await;
        }
    });

    let stats_handle = tokio::spawn({
        let price_rx = price_tx.subscribe();
        let alert_tx_clone = alert_tx.clone();
        let shutdown_rx = shutdown_tx.subscribe();
        async move {
            stats_tracker.run(price_rx, alert_tx_clone, shutdown_rx).await;
        }
    });

    let display_handle = tokio::spawn({
        let price_rx = price_tx.subscribe();
        let alert_rx = alert_tx.subscribe();
        let shutdown_rx = shutdown_tx.subscribe();
        async move {
            console_display.run(price_rx, alert_rx, shutdown_rx).await;
        }
    });

    // Wait for shutdown signal (Ctrl+C)
    tokio::signal::ctrl_c().await?;

    info!("Shutdown signal received, stopping services...");

    // Send shutdown signal to all tasks
    let _ = shutdown_tx.send(());

    // Give time for graceful shutdown
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Wait for all tasks to complete
    let _ = tokio::join!(mock_handle, alert_handle, stats_handle, display_handle);

    info!("Price Monitor stopped");

    Ok(())
}
```

## What We Learned in This Project

| Concept | Application in Project |
|---------|----------------------|
| async/await | All data source functions are asynchronous |
| tokio::spawn | Parallel execution of multiple tasks |
| tokio::select! | Handling multiple async events |
| broadcast channel | Distributing price updates to all subscribers |
| WebSocket | Real-time data from Binance |
| HTTP client | Polling Coinbase API with reqwest |
| Graceful shutdown | Proper termination of all tasks on Ctrl+C |
| Reconnection | Automatic WebSocket connection recovery |
| Timeouts | Limiting HTTP request wait time |
| Structured logging | tracing for debugging |

## Practical Exercises

1. **Adding a New Exchange**: Implement a client for another exchange (e.g., Kraken). Use their public API to fetch prices.

2. **History Storage**: Add writing all price updates to a CSV file using `tokio::fs` for async writing.

3. **Web Interface**: Create a simple HTTP server with axum or actix-web that shows current prices via REST API.

4. **Telegram Bot**: Integrate alert sending to Telegram using teloxide or another library.

## Homework

1. **Full Arbitrage Scanner**:
   - Connect to 3+ real exchanges
   - Track price differences in real-time
   - Account for fees when calculating profitability
   - Log all discovered opportunities

2. **Monitoring with Persistence**:
   - Save statistics to SQLite (using sqlx)
   - Restore state on restart
   - Add historical charts via API

3. **Performance Optimization**:
   - Measure latency from data receipt to display
   - Optimize channel usage
   - Add performance metrics

## Month Summary

Congratulations! You've completed Month 7: Async and Networking. This month you learned:

- The difference between synchronous and asynchronous code
- Working with async/await and Future
- Tokio runtime and its capabilities
- Async channels and synchronization
- HTTP requests with reqwest
- WebSocket connections with tokio-tungstenite
- Graceful shutdown and signal handling
- Building a complete real-time application

You're now ready to create high-performance network applications for trading!

## Navigation

[← Previous day](../211-debugging-async-code/en.md) | [Next day →](../213-why-databases-persistence/en.md)
