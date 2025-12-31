# Day 203: WebSocket: Streaming Data

## Trading Analogy

Imagine you're sitting in a stock exchange trading floor. Instead of constantly asking "What's the current price?" (like HTTP requests), you simply listen to the loudspeaker that announces every price change in real-time. This is **WebSocket** — a persistent bidirectional connection between client and server that allows receiving data instantly without constantly "polling" the server.

In real trading, WebSocket is used for:
- Receiving real-time price updates (tickers, OHLCV)
- Streaming order book changes
- Notifications about order executions
- Monitoring account balance

## What is WebSocket?

WebSocket is a protocol providing full-duplex communication over a single TCP connection. Unlike HTTP:

| HTTP | WebSocket |
|------|-----------|
| Client always initiates request | Both sides can send messages |
| Connection closes after response | Connection stays open |
| High overhead per request | Minimal overhead after setup |
| Suitable for infrequent requests | Ideal for streaming data |

## Basic WebSocket Message Structure

```rust
use serde::{Deserialize, Serialize};

// Typical exchange subscription message
#[derive(Serialize, Debug)]
struct SubscribeMessage {
    method: String,
    params: Vec<String>,
    id: u64,
}

// Typical ticker data message
#[derive(Deserialize, Debug)]
struct TickerData {
    symbol: String,
    price: String,
    timestamp: u64,
}

// Exchange response
#[derive(Deserialize, Debug)]
struct ExchangeMessage {
    stream: String,
    data: serde_json::Value,
}

fn main() {
    // Create subscription message
    let subscribe = SubscribeMessage {
        method: "SUBSCRIBE".to_string(),
        params: vec!["btcusdt@ticker".to_string()],
        id: 1,
    };

    let json = serde_json::to_string(&subscribe).unwrap();
    println!("Subscription message: {}", json);
    // Output: {"method":"SUBSCRIBE","params":["btcusdt@ticker"],"id":1}
}
```

## Simple WebSocket Client (Concept)

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// Mock WebSocket connection for demonstrating concepts
#[derive(Debug)]
struct MockWebSocket {
    connected: bool,
    messages: VecDeque<String>,
    last_ping: Instant,
}

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

impl MockWebSocket {
    fn new() -> Self {
        MockWebSocket {
            connected: false,
            messages: VecDeque::new(),
            last_ping: Instant::now(),
        }
    }

    fn connect(&mut self, url: &str) -> Result<(), String> {
        println!("Connecting to {}...", url);
        self.connected = true;
        self.last_ping = Instant::now();
        println!("Connection established!");
        Ok(())
    }

    fn subscribe(&mut self, channels: &[&str]) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }

        for channel in channels {
            println!("Subscribing to channel: {}", channel);
        }
        Ok(())
    }

    fn receive(&mut self) -> Option<PriceUpdate> {
        if !self.connected {
            return None;
        }

        // Simulate receiving data
        Some(PriceUpdate {
            symbol: "BTC/USDT".to_string(),
            price: 42000.0 + (rand_simple() * 1000.0),
            volume: 100.0 + (rand_simple() * 50.0),
            timestamp: current_timestamp(),
        })
    }

    fn send_ping(&mut self) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        self.last_ping = Instant::now();
        println!("Ping sent");
        Ok(())
    }

    fn disconnect(&mut self) {
        self.connected = false;
        println!("Connection closed");
    }
}

// Simple helper functions
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

fn current_timestamp() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn main() {
    let mut ws = MockWebSocket::new();

    // Connect
    ws.connect("wss://stream.binance.com:9443/ws").unwrap();

    // Subscribe to channels
    ws.subscribe(&["btcusdt@ticker", "ethusdt@ticker"]).unwrap();

    // Receive a few updates
    for i in 0..5 {
        if let Some(update) = ws.receive() {
            println!(
                "Update #{}: {} = ${:.2} (volume: {:.2})",
                i + 1, update.symbol, update.price, update.volume
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    ws.disconnect();
}
```

## Processing Streaming Data

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    is_buyer_maker: bool,
    timestamp: u64,
}

#[derive(Debug)]
struct TradeAggregator {
    trades: Vec<Trade>,
    volume_by_symbol: HashMap<String, f64>,
    price_by_symbol: HashMap<String, f64>,
    start_time: Instant,
}

impl TradeAggregator {
    fn new() -> Self {
        TradeAggregator {
            trades: Vec::new(),
            volume_by_symbol: HashMap::new(),
            price_by_symbol: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    fn process_trade(&mut self, trade: Trade) {
        // Update last price
        self.price_by_symbol
            .insert(trade.symbol.clone(), trade.price);

        // Aggregate volume
        *self.volume_by_symbol
            .entry(trade.symbol.clone())
            .or_insert(0.0) += trade.quantity;

        // Store trade
        self.trades.push(trade);
    }

    fn get_vwap(&self, symbol: &str) -> Option<f64> {
        let symbol_trades: Vec<&Trade> = self.trades
            .iter()
            .filter(|t| t.symbol == symbol)
            .collect();

        if symbol_trades.is_empty() {
            return None;
        }

        let total_value: f64 = symbol_trades
            .iter()
            .map(|t| t.price * t.quantity)
            .sum();

        let total_quantity: f64 = symbol_trades
            .iter()
            .map(|t| t.quantity)
            .sum();

        Some(total_value / total_quantity)
    }

    fn get_buy_sell_ratio(&self, symbol: &str) -> (f64, f64) {
        let symbol_trades: Vec<&Trade> = self.trades
            .iter()
            .filter(|t| t.symbol == symbol)
            .collect();

        let buy_volume: f64 = symbol_trades
            .iter()
            .filter(|t| !t.is_buyer_maker)
            .map(|t| t.quantity)
            .sum();

        let sell_volume: f64 = symbol_trades
            .iter()
            .filter(|t| t.is_buyer_maker)
            .map(|t| t.quantity)
            .sum();

        (buy_volume, sell_volume)
    }

    fn summary(&self) {
        let elapsed = self.start_time.elapsed();
        println!("\n=== Summary for {:?} ===", elapsed);
        println!("Total trades: {}", self.trades.len());

        for (symbol, volume) in &self.volume_by_symbol {
            let price = self.price_by_symbol.get(symbol).unwrap_or(&0.0);
            let vwap = self.get_vwap(symbol).unwrap_or(0.0);
            let (buy_vol, sell_vol) = self.get_buy_sell_ratio(symbol);

            println!("\n{}:", symbol);
            println!("  Last price: ${:.2}", price);
            println!("  VWAP: ${:.2}", vwap);
            println!("  Volume: {:.4}", volume);
            println!("  Buy/Sell: {:.4}/{:.4}", buy_vol, sell_vol);
        }
    }
}

fn main() {
    let mut aggregator = TradeAggregator::new();

    // Simulate trade stream
    let trades = vec![
        Trade {
            id: 1,
            symbol: "BTC/USDT".to_string(),
            price: 42000.0,
            quantity: 0.5,
            is_buyer_maker: false,
            timestamp: 1700000001,
        },
        Trade {
            id: 2,
            symbol: "BTC/USDT".to_string(),
            price: 42010.0,
            quantity: 0.3,
            is_buyer_maker: true,
            timestamp: 1700000002,
        },
        Trade {
            id: 3,
            symbol: "ETH/USDT".to_string(),
            price: 2200.0,
            quantity: 2.0,
            is_buyer_maker: false,
            timestamp: 1700000003,
        },
        Trade {
            id: 4,
            symbol: "BTC/USDT".to_string(),
            price: 42050.0,
            quantity: 0.8,
            is_buyer_maker: false,
            timestamp: 1700000004,
        },
        Trade {
            id: 5,
            symbol: "ETH/USDT".to_string(),
            price: 2205.0,
            quantity: 1.5,
            is_buyer_maker: true,
            timestamp: 1700000005,
        },
    ];

    for trade in trades {
        println!("Trade received: {} {} @ ${:.2}",
            if trade.is_buyer_maker { "SELL" } else { "BUY" },
            trade.symbol,
            trade.price
        );
        aggregator.process_trade(trade);
    }

    aggregator.summary();
}
```

## Real-Time Order Book

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct OrderBookLevel {
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct OrderBook {
    symbol: String,
    bids: BTreeMap<i64, f64>, // Price * 100 -> Quantity (for sorting)
    asks: BTreeMap<i64, f64>,
    last_update_id: u64,
}

impl OrderBook {
    fn new(symbol: &str) -> Self {
        OrderBook {
            symbol: symbol.to_string(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_id: 0,
        }
    }

    fn price_to_key(price: f64) -> i64 {
        (price * 100.0) as i64
    }

    fn key_to_price(key: i64) -> f64 {
        key as f64 / 100.0
    }

    fn update_bid(&mut self, price: f64, quantity: f64) {
        let key = Self::price_to_key(price);
        if quantity == 0.0 {
            self.bids.remove(&key);
        } else {
            self.bids.insert(key, quantity);
        }
    }

    fn update_ask(&mut self, price: f64, quantity: f64) {
        let key = Self::price_to_key(price);
        if quantity == 0.0 {
            self.asks.remove(&key);
        } else {
            self.asks.insert(key, quantity);
        }
    }

    fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids.iter().next_back().map(|(&k, &v)| (Self::key_to_price(k), v))
    }

    fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks.iter().next().map(|(&k, &v)| (Self::key_to_price(k), v))
    }

    fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask - bid),
            _ => None,
        }
    }

    fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    fn total_bid_volume(&self, depth: usize) -> f64 {
        self.bids.iter().rev().take(depth).map(|(_, &v)| v).sum()
    }

    fn total_ask_volume(&self, depth: usize) -> f64 {
        self.asks.iter().take(depth).map(|(_, &v)| v).sum()
    }

    fn imbalance(&self, depth: usize) -> f64 {
        let bid_vol = self.total_bid_volume(depth);
        let ask_vol = self.total_ask_volume(depth);
        let total = bid_vol + ask_vol;

        if total == 0.0 {
            0.0
        } else {
            (bid_vol - ask_vol) / total
        }
    }

    fn display(&self, levels: usize) {
        println!("\n=== {} Order Book ===", self.symbol);

        // Show asks (top to bottom)
        let asks: Vec<_> = self.asks.iter().take(levels).collect();
        for (&key, &qty) in asks.iter().rev() {
            let price = Self::key_to_price(key);
            let bar = "█".repeat((qty * 10.0) as usize);
            println!("  ASK: ${:.2} | {:.4} {}", price, qty, bar);
        }

        // Spread
        if let Some(spread) = self.spread() {
            println!("  --- Spread: ${:.2} ---", spread);
        }

        // Show bids (top to bottom)
        let bids: Vec<_> = self.bids.iter().rev().take(levels).collect();
        for (&key, &qty) in bids.iter() {
            let price = Self::key_to_price(key);
            let bar = "█".repeat((qty * 10.0) as usize);
            println!("  BID: ${:.2} | {:.4} {}", price, qty, bar);
        }

        println!("\nImbalance (5 levels): {:.2}%", self.imbalance(5) * 100.0);
    }
}

fn main() {
    let mut order_book = OrderBook::new("BTC/USDT");

    // Initialize order book
    order_book.update_bid(42000.0, 1.5);
    order_book.update_bid(41990.0, 2.3);
    order_book.update_bid(41980.0, 0.8);
    order_book.update_bid(41970.0, 3.1);
    order_book.update_bid(41960.0, 1.2);

    order_book.update_ask(42010.0, 1.2);
    order_book.update_ask(42020.0, 2.1);
    order_book.update_ask(42030.0, 0.5);
    order_book.update_ask(42040.0, 1.8);
    order_book.update_ask(42050.0, 2.5);

    order_book.display(5);

    // Simulate WebSocket updates
    println!("\n>>> Update received: BID $42005.00 x 0.7");
    order_book.update_bid(42005.0, 0.7);

    println!(">>> Update received: ASK $42010.00 removed");
    order_book.update_ask(42010.0, 0.0);

    order_book.display(5);

    println!("\nBest BID: {:?}", order_book.best_bid());
    println!("Best ASK: {:?}", order_book.best_ask());
    println!("Mid-price: ${:.2}", order_book.mid_price().unwrap_or(0.0));
}
```

## Reconnection and Error Handling

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

#[derive(Debug)]
struct ReconnectingWebSocket {
    url: String,
    state: ConnectionState,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    base_delay: Duration,
    last_message_time: Option<Instant>,
    heartbeat_interval: Duration,
}

impl ReconnectingWebSocket {
    fn new(url: &str) -> Self {
        ReconnectingWebSocket {
            url: url.to_string(),
            state: ConnectionState::Disconnected,
            reconnect_attempts: 0,
            max_reconnect_attempts: 5,
            base_delay: Duration::from_secs(1),
            last_message_time: None,
            heartbeat_interval: Duration::from_secs(30),
        }
    }

    fn connect(&mut self) -> Result<(), String> {
        self.state = ConnectionState::Connecting;
        println!("Connecting to {}...", self.url);

        // Simulate connection
        self.state = ConnectionState::Connected;
        self.reconnect_attempts = 0;
        self.last_message_time = Some(Instant::now());

        println!("Connected successfully!");
        Ok(())
    }

    fn reconnect(&mut self) -> Result<(), String> {
        if self.reconnect_attempts >= self.max_reconnect_attempts {
            return Err("Max reconnection attempts exceeded".to_string());
        }

        self.state = ConnectionState::Reconnecting;
        self.reconnect_attempts += 1;

        // Exponential backoff
        let delay = self.base_delay * 2u32.pow(self.reconnect_attempts - 1);
        println!(
            "Reconnection attempt {} of {} in {:?}...",
            self.reconnect_attempts,
            self.max_reconnect_attempts,
            delay
        );

        std::thread::sleep(delay);
        self.connect()
    }

    fn handle_disconnect(&mut self) {
        println!("Connection lost!");
        self.state = ConnectionState::Disconnected;

        // Try to reconnect
        loop {
            match self.reconnect() {
                Ok(_) => {
                    println!("Reconnection successful!");
                    break;
                }
                Err(e) => {
                    println!("Reconnection error: {}", e);
                    if self.reconnect_attempts >= self.max_reconnect_attempts {
                        println!("Giving up reconnection.");
                        break;
                    }
                }
            }
        }
    }

    fn check_heartbeat(&mut self) -> bool {
        if let Some(last_time) = self.last_message_time {
            if last_time.elapsed() > self.heartbeat_interval {
                println!("Heartbeat timeout!");
                return false;
            }
        }
        true
    }

    fn on_message(&mut self, _message: &str) {
        self.last_message_time = Some(Instant::now());
    }
}

fn main() {
    let mut ws = ReconnectingWebSocket::new("wss://stream.binance.com:9443/ws");

    // Connect
    ws.connect().unwrap();

    // Simulate receiving messages
    for i in 0..3 {
        ws.on_message(&format!("message_{}", i));
        println!("Received message {}", i);
        std::thread::sleep(Duration::from_millis(100));
    }

    // Simulate connection loss
    ws.handle_disconnect();

    println!("\nState: {:?}", ws.state);
}
```

## Stream Multiplexing

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum StreamMessage {
    Ticker { symbol: String, price: f64, volume: f64 },
    Trade { symbol: String, price: f64, quantity: f64, side: String },
    OrderBook { symbol: String, bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)> },
    Kline { symbol: String, open: f64, high: f64, low: f64, close: f64 },
}

#[derive(Debug)]
struct StreamManager {
    subscriptions: HashMap<String, Vec<String>>, // stream_type -> symbols
    message_count: HashMap<String, u64>,
}

impl StreamManager {
    fn new() -> Self {
        StreamManager {
            subscriptions: HashMap::new(),
            message_count: HashMap::new(),
        }
    }

    fn subscribe(&mut self, stream_type: &str, symbols: Vec<&str>) {
        let symbols: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        self.subscriptions.insert(stream_type.to_string(), symbols.clone());

        println!("Subscribed to {} for {:?}", stream_type, symbols);
    }

    fn unsubscribe(&mut self, stream_type: &str) {
        if self.subscriptions.remove(stream_type).is_some() {
            println!("Unsubscribed from {}", stream_type);
        }
    }

    fn process_message(&mut self, message: StreamMessage) {
        match &message {
            StreamMessage::Ticker { symbol, price, volume } => {
                *self.message_count.entry("ticker".to_string()).or_insert(0) += 1;
                println!("TICKER {}: ${:.2} (volume: {:.2})", symbol, price, volume);
            }
            StreamMessage::Trade { symbol, price, quantity, side } => {
                *self.message_count.entry("trade".to_string()).or_insert(0) += 1;
                println!("TRADE {}: {} {:.4} @ ${:.2}", symbol, side, quantity, price);
            }
            StreamMessage::OrderBook { symbol, bids, asks } => {
                *self.message_count.entry("orderbook".to_string()).or_insert(0) += 1;
                println!(
                    "ORDERBOOK {}: {} bids, {} asks",
                    symbol, bids.len(), asks.len()
                );
            }
            StreamMessage::Kline { symbol, open, high, low, close } => {
                *self.message_count.entry("kline".to_string()).or_insert(0) += 1;
                println!(
                    "KLINE {}: O={:.2} H={:.2} L={:.2} C={:.2}",
                    symbol, open, high, low, close
                );
            }
        }
    }

    fn stats(&self) {
        println!("\n=== Stream Statistics ===");
        for (stream, count) in &self.message_count {
            println!("{}: {} messages", stream, count);
        }
    }
}

fn main() {
    let mut manager = StreamManager::new();

    // Subscribe to different streams
    manager.subscribe("ticker", vec!["BTC/USDT", "ETH/USDT"]);
    manager.subscribe("trade", vec!["BTC/USDT"]);
    manager.subscribe("orderbook", vec!["BTC/USDT"]);

    // Simulate receiving messages
    let messages = vec![
        StreamMessage::Ticker {
            symbol: "BTC/USDT".to_string(),
            price: 42000.0,
            volume: 1500.0,
        },
        StreamMessage::Trade {
            symbol: "BTC/USDT".to_string(),
            price: 42001.0,
            quantity: 0.5,
            side: "BUY".to_string(),
        },
        StreamMessage::OrderBook {
            symbol: "BTC/USDT".to_string(),
            bids: vec![(42000.0, 1.5), (41999.0, 2.0)],
            asks: vec![(42001.0, 1.2), (42002.0, 1.8)],
        },
        StreamMessage::Kline {
            symbol: "BTC/USDT".to_string(),
            open: 41950.0,
            high: 42100.0,
            low: 41900.0,
            close: 42000.0,
        },
        StreamMessage::Ticker {
            symbol: "ETH/USDT".to_string(),
            price: 2200.0,
            volume: 5000.0,
        },
    ];

    println!("\n=== Processing Messages ===\n");
    for msg in messages {
        manager.process_message(msg);
    }

    manager.stats();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| WebSocket | Full-duplex protocol for streaming data |
| Subscription | Mechanism for selecting data channels |
| Order Book | Structure for storing the order book |
| Reconnection | Reconnection strategy with exponential backoff |
| Heartbeat | Connection activity verification |
| Multiplexing | Handling multiple streams in one connection |

## Practice Exercises

1. **Ticker Simulator**: Create a `TickerSimulator` struct that:
   - Generates random price changes
   - Sends updates at a specified interval
   - Supports multiple symbols

2. **Order Book Analyzer**: Extend `OrderBook`:
   - Add method `get_depth(price_range: f64)` — number of levels within range
   - Add method `get_liquidity(volume: f64)` — execution price for given volume
   - Add text-based "bar" visualization

3. **Rate Limiter**: Implement a rate limiter:
   - No more than N messages per second
   - Buffer excess messages
   - Log limit violations

4. **Error Handler**: Create a handler for different WebSocket error types:
   - Connection error
   - Timeout
   - Invalid data
   - Server-side connection close

## Homework

1. **Data Aggregator**: Write a program that:
   - Receives data from multiple "streams" (can be simulated)
   - Aggregates OHLCV candles for different periods (1m, 5m, 15m)
   - Calculates moving averages in real-time

2. **Anomaly Detector**: Implement an unusual activity detector:
   - Track sudden volume spikes
   - Detect large spreads
   - Identify large trades
   - Log warnings with timestamps

3. **Snapshot + Diff**: Implement the order book initialization pattern:
   - Load initial snapshot
   - Apply incremental updates (diff)
   - Periodically verify against full snapshot
   - Handle discrepancies

## Navigation

[← Previous day](../202-retry-with-backoff-repeating-requests/en.md) | [Next day →](../204-tokio-tungstenite-websocket-client/en.md)
