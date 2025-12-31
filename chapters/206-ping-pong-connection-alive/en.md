# Day 206: Ping-Pong: Keeping Connection Alive

## Trading Analogy

Imagine you're connected to an exchange via WebSocket to receive real-time quotes. If the connection suddenly drops — you might miss an important price movement and lose money. But how do the exchange and your client know the connection is still alive?

It's like a security guard at their post: they must periodically press an "I'm here" button so the central control knows they haven't fallen asleep. If the signal doesn't come — an alarm goes off. In network protocols, this button is called **Ping**, and the response is called **Pong**.

In real trading, the ping-pong mechanism is critically important:
- **Exchanges disconnect inactive connections** after 30-60 seconds without a ping
- **You must monitor latency** — if pong takes too long, the network might be overloaded
- **When connection drops** you need to reconnect quickly to not miss trades

## What is Ping-Pong?

Ping-Pong is a **heartbeat** mechanism for TCP/WebSocket connections:

1. **Ping** — a message from client to server: "Are you still there?"
2. **Pong** — a response from server: "Yes, I'm here!"

```
Client                          Server
   |                               |
   |-------- PING --------------->|
   |                               |
   |<------- PONG ----------------|
   |                               |
   |-------- PING --------------->|
   |                               |
   |<------- PONG ----------------|
   |                               |
   ...
```

### Why Do We Need Ping-Pong?

| Problem | Ping-Pong Solution |
|---------|-------------------|
| Connection "frozen" | Pong timeout — reconnect |
| Server overloaded | Increased ping latency — warning |
| NAT/Firewall closes connection | Periodic pings keep connection open |
| Server-side failure | No pong = need to reconnect |

## Simple TCP Ping-Pong

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{interval, timeout, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
struct ConnectionStats {
    pings_sent: u64,
    pongs_received: u64,
    last_latency_ms: u64,
}

// Server: responds with PONG to each PING
async fn run_price_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Quote server started on 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection: {}", addr);

        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];

            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => {
                        println!("Client {} disconnected", addr);
                        break;
                    }
                    Ok(n) => {
                        let message = String::from_utf8_lossy(&buffer[..n]);

                        if message.trim() == "PING" {
                            println!("Received PING from {}", addr);
                            if let Err(e) = socket.write_all(b"PONG\n").await {
                                eprintln!("Error sending PONG: {}", e);
                                break;
                            }
                        } else if message.starts_with("SUBSCRIBE:") {
                            // Handle quote subscription
                            let symbol = message.trim().strip_prefix("SUBSCRIBE:").unwrap();
                            println!("Client {} subscribed to {}", addr, symbol);
                            socket.write_all(format!("SUBSCRIBED:{}\n", symbol).as_bytes()).await.ok();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from {}: {}", addr, e);
                        break;
                    }
                }
            }
        });
    }
}

// Client: sends PING and waits for PONG
async fn run_trading_client() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to quote server");

    let stats = Arc::new(Mutex::new(ConnectionStats {
        pings_sent: 0,
        pongs_received: 0,
        last_latency_ms: 0,
    }));

    let stats_ping = Arc::clone(&stats);

    // Task for sending pings
    let (mut read_half, mut write_half) = stream.into_split();

    let ping_task = tokio::spawn(async move {
        let mut ping_interval = interval(Duration::from_secs(5));

        loop {
            ping_interval.tick().await;

            let start = std::time::Instant::now();

            if let Err(e) = write_half.write_all(b"PING\n").await {
                eprintln!("Error sending PING: {}", e);
                break;
            }

            let mut stats = stats_ping.lock().await;
            stats.pings_sent += 1;
            println!("Sent PING #{}", stats.pings_sent);
        }
    });

    // Task for receiving responses
    let stats_pong = Arc::clone(&stats);
    let pong_task = tokio::spawn(async move {
        let mut buffer = [0u8; 1024];

        loop {
            match read_half.read(&mut buffer).await {
                Ok(0) => {
                    println!("Server closed connection");
                    break;
                }
                Ok(n) => {
                    let message = String::from_utf8_lossy(&buffer[..n]);

                    if message.trim() == "PONG" {
                        let mut stats = stats_pong.lock().await;
                        stats.pongs_received += 1;
                        println!("Received PONG #{}", stats.pongs_received);
                    }
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for completion
    tokio::select! {
        _ = ping_task => {},
        _ = pong_task => {},
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Start server in a separate task
    tokio::spawn(run_price_server());

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Start client
    if let Err(e) = run_trading_client().await {
        eprintln!("Client error: {}", e);
    }
}
```

## Ping-Pong with Timeouts for Trading

In real trading, it's critically important to react quickly to connection loss:

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{interval, timeout, Duration, Instant};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

const PING_INTERVAL: Duration = Duration::from_secs(5);
const PONG_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_MISSED_PONGS: u32 = 3;

#[derive(Debug, Clone)]
enum ConnectionState {
    Connected,
    Degraded { missed_pongs: u32 },
    Disconnected,
}

#[derive(Debug)]
struct ExchangeConnection {
    state: ConnectionState,
    last_pong: Instant,
    latency_ms: Vec<u64>,
}

impl ExchangeConnection {
    fn new() -> Self {
        ExchangeConnection {
            state: ConnectionState::Connected,
            last_pong: Instant::now(),
            latency_ms: Vec::new(),
        }
    }

    fn record_pong(&mut self, latency: u64) {
        self.last_pong = Instant::now();
        self.latency_ms.push(latency);

        // Keep only last 100 measurements
        if self.latency_ms.len() > 100 {
            self.latency_ms.remove(0);
        }

        self.state = ConnectionState::Connected;
    }

    fn miss_pong(&mut self) {
        match &mut self.state {
            ConnectionState::Connected => {
                self.state = ConnectionState::Degraded { missed_pongs: 1 };
            }
            ConnectionState::Degraded { missed_pongs } => {
                *missed_pongs += 1;
                if *missed_pongs >= MAX_MISSED_PONGS {
                    self.state = ConnectionState::Disconnected;
                }
            }
            ConnectionState::Disconnected => {}
        }
    }

    fn average_latency(&self) -> Option<f64> {
        if self.latency_ms.is_empty() {
            return None;
        }
        let sum: u64 = self.latency_ms.iter().sum();
        Some(sum as f64 / self.latency_ms.len() as f64)
    }

    fn is_healthy(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }
}

async fn exchange_client_with_heartbeat() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let connection = Arc::new(Mutex::new(ExchangeConnection::new()));

    let (mut read_half, mut write_half) = stream.into_split();
    let (pong_tx, mut pong_rx) = mpsc::channel::<Instant>(10);

    // Read task
    let read_task = tokio::spawn(async move {
        let mut buffer = [0u8; 1024];

        loop {
            match read_half.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let message = String::from_utf8_lossy(&buffer[..n]);
                    if message.trim() == "PONG" {
                        pong_tx.send(Instant::now()).await.ok();
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Ping-pong task with timeout
    let conn_clone = Arc::clone(&connection);
    let heartbeat_task = tokio::spawn(async move {
        let mut ping_interval = interval(PING_INTERVAL);

        loop {
            ping_interval.tick().await;

            let ping_time = Instant::now();

            // Send PING
            if write_half.write_all(b"PING\n").await.is_err() {
                break;
            }

            // Wait for PONG with timeout
            match timeout(PONG_TIMEOUT, pong_rx.recv()).await {
                Ok(Some(pong_time)) => {
                    let latency = pong_time.duration_since(ping_time).as_millis() as u64;
                    let mut conn = conn_clone.lock().await;
                    conn.record_pong(latency);

                    println!(
                        "PONG received: latency={}ms, avg={:.2}ms",
                        latency,
                        conn.average_latency().unwrap_or(0.0)
                    );

                    // Warning for high latency
                    if latency > 100 {
                        println!("⚠️  High latency! Possible issues with order execution");
                    }
                }
                Ok(None) => {
                    println!("Channel closed");
                    break;
                }
                Err(_) => {
                    let mut conn = conn_clone.lock().await;
                    conn.miss_pong();

                    match &conn.state {
                        ConnectionState::Degraded { missed_pongs } => {
                            println!(
                                "⚠️  PONG not received! Missed: {}/{}",
                                missed_pongs, MAX_MISSED_PONGS
                            );
                        }
                        ConnectionState::Disconnected => {
                            println!("❌ Connection lost! Initiating reconnection...");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    // Connection state monitoring
    let conn_monitor = Arc::clone(&connection);
    let monitor_task = tokio::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(1));

        loop {
            check_interval.tick().await;

            let conn = conn_monitor.lock().await;
            if !conn.is_healthy() {
                println!("Connection unhealthy: {:?}", conn.state);
            }
        }
    });

    tokio::select! {
        _ = read_task => println!("Read task completed"),
        _ = heartbeat_task => println!("Heartbeat task completed"),
        _ = monitor_task => println!("Monitor task completed"),
    }

    Ok(())
}
```

## WebSocket Ping-Pong for Crypto Exchanges

Most crypto exchanges use WebSocket with built-in ping-pong:

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval, Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct PingMessage {
    op: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct PongMessage {
    op: String,
    #[serde(default)]
    id: Option<u64>,
}

#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

async fn binance_ws_client() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Binance WebSocket API
    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";

    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to Binance WebSocket");

    let (mut write, mut read) = ws_stream.split();

    let mut last_message_time = Instant::now();
    let mut ping_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // Send ping every 30 seconds
            _ = ping_interval.tick() => {
                // Check how long since we received data
                let silence_duration = last_message_time.elapsed();

                if silence_duration > Duration::from_secs(60) {
                    println!("⚠️  No data for over 60 seconds!");
                }

                // WebSocket ping (protocol level)
                write.send(Message::Ping(vec![])).await?;
                println!("Sent WebSocket PING");
            }

            // Read incoming messages
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        last_message_time = Instant::now();

                        // Parse trade data
                        if let Ok(trade) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(price) = trade.get("p").and_then(|p| p.as_str()) {
                                println!("BTC/USDT: ${}", price);
                            }
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_message_time = Instant::now();
                        println!("Received PONG from Binance");
                    }
                    Some(Ok(Message::Ping(data))) => {
                        // Respond to server ping
                        write.send(Message::Pong(data)).await?;
                        println!("Responded PONG to server PING");
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("Server closed connection");
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        println!("Stream closed");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
```

## Automatic Reconnection

In trading, reconnection must be automatic and fast:

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, timeout, Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting { attempt: u32 },
}

struct TradingClient {
    status: Arc<RwLock<ConnectionStatus>>,
    server_addr: String,
    max_reconnect_attempts: u32,
    base_reconnect_delay: Duration,
}

impl TradingClient {
    fn new(server_addr: &str) -> Self {
        TradingClient {
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            server_addr: server_addr.to_string(),
            max_reconnect_attempts: 10,
            base_reconnect_delay: Duration::from_secs(1),
        }
    }

    async fn connect_with_retry(&self) -> Result<TcpStream, String> {
        let mut attempt = 0;

        loop {
            attempt += 1;

            {
                let mut status = self.status.write().await;
                *status = ConnectionStatus::Reconnecting { attempt };
            }

            println!("Connection attempt #{}", attempt);

            match timeout(
                Duration::from_secs(5),
                TcpStream::connect(&self.server_addr)
            ).await {
                Ok(Ok(stream)) => {
                    let mut status = self.status.write().await;
                    *status = ConnectionStatus::Connected;
                    println!("✅ Connection successful!");
                    return Ok(stream);
                }
                Ok(Err(e)) => {
                    eprintln!("Connection error: {}", e);
                }
                Err(_) => {
                    eprintln!("Connection timeout");
                }
            }

            if attempt >= self.max_reconnect_attempts {
                let mut status = self.status.write().await;
                *status = ConnectionStatus::Disconnected;
                return Err("Maximum reconnection attempts exceeded".to_string());
            }

            // Exponential backoff with jitter
            let delay = self.calculate_backoff(attempt);
            println!("Next attempt in {:?}", delay);
            sleep(delay).await;
        }
    }

    fn calculate_backoff(&self, attempt: u32) -> Duration {
        // Exponential backoff: 1s, 2s, 4s, 8s, ... up to 60s
        let base = self.base_reconnect_delay.as_secs_f64();
        let exp_delay = base * 2.0_f64.powi((attempt - 1) as i32);
        let capped_delay = exp_delay.min(60.0);

        // Add random jitter (±20%)
        let jitter = capped_delay * 0.2 * (rand::random::<f64>() - 0.5);
        Duration::from_secs_f64(capped_delay + jitter)
    }

    async fn run_with_heartbeat(self: Arc<Self>) -> Result<(), String> {
        loop {
            let stream = self.connect_with_retry().await?;
            let (mut read_half, mut write_half) = stream.into_split();

            let status = Arc::clone(&self.status);

            // Ping-pong loop
            let ping_handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(5));
                let mut missed_pongs = 0;

                loop {
                    interval.tick().await;

                    if write_half.write_all(b"PING\n").await.is_err() {
                        return false; // Need to reconnect
                    }

                    // Simple state check
                    let current_status = status.read().await;
                    if !matches!(*current_status, ConnectionStatus::Connected) {
                        return false;
                    }
                }
            });

            let read_handle = tokio::spawn(async move {
                let mut buffer = [0u8; 1024];

                loop {
                    match read_half.read(&mut buffer).await {
                        Ok(0) => return false,
                        Ok(_) => {
                            // Process messages
                        }
                        Err(_) => return false,
                    }
                }
            });

            // Wait for any task to complete
            tokio::select! {
                result = ping_handle => {
                    if let Ok(false) = result {
                        println!("Connection lost, reconnecting...");
                    }
                }
                result = read_handle => {
                    if let Ok(false) = result {
                        println!("Connection closed by server, reconnecting...");
                    }
                }
            }

            // Small pause before reconnecting
            sleep(Duration::from_millis(500)).await;
        }
    }
}

// Simple random for example (use rand crate in real code)
mod rand {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random<T: From<f64>>() -> T {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        T::from((nanos % 1000) as f64 / 1000.0)
    }
}
```

## Connection Quality Monitoring

For algorithmic trading, it's important not only to maintain the connection but also to monitor its quality:

```rust
use std::collections::VecDeque;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct LatencyStats {
    samples: VecDeque<u64>,
    max_samples: usize,
}

impl LatencyStats {
    fn new(max_samples: usize) -> Self {
        LatencyStats {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    fn add_sample(&mut self, latency_ms: u64) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(latency_ms);
    }

    fn average(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        let sum: u64 = self.samples.iter().sum();
        Some(sum as f64 / self.samples.len() as f64)
    }

    fn percentile(&self, p: f64) -> Option<u64> {
        if self.samples.is_empty() {
            return None;
        }

        let mut sorted: Vec<u64> = self.samples.iter().copied().collect();
        sorted.sort();

        let index = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        Some(sorted[index])
    }

    fn min(&self) -> Option<u64> {
        self.samples.iter().copied().min()
    }

    fn max(&self) -> Option<u64> {
        self.samples.iter().copied().max()
    }
}

#[derive(Debug)]
struct ConnectionQualityMonitor {
    latency_stats: LatencyStats,
    missed_pongs: u32,
    total_pings: u64,
    total_pongs: u64,
    last_pong_time: Option<Instant>,
}

impl ConnectionQualityMonitor {
    fn new() -> Self {
        ConnectionQualityMonitor {
            latency_stats: LatencyStats::new(100),
            missed_pongs: 0,
            total_pings: 0,
            total_pongs: 0,
            last_pong_time: None,
        }
    }

    fn record_ping(&mut self) {
        self.total_pings += 1;
    }

    fn record_pong(&mut self, latency_ms: u64) {
        self.total_pongs += 1;
        self.missed_pongs = 0;
        self.last_pong_time = Some(Instant::now());
        self.latency_stats.add_sample(latency_ms);
    }

    fn record_missed_pong(&mut self) {
        self.missed_pongs += 1;
    }

    fn success_rate(&self) -> f64 {
        if self.total_pings == 0 {
            return 100.0;
        }
        (self.total_pongs as f64 / self.total_pings as f64) * 100.0
    }

    fn report(&self) -> String {
        format!(
            "📊 Connection Statistics:\n\
             • Success rate: {:.1}%\n\
             • Missed pongs: {}\n\
             • Latency avg: {:.2}ms\n\
             • Latency p50: {}ms\n\
             • Latency p99: {}ms\n\
             • Latency min/max: {}/{}ms",
            self.success_rate(),
            self.missed_pongs,
            self.latency_stats.average().unwrap_or(0.0),
            self.latency_stats.percentile(50.0).unwrap_or(0),
            self.latency_stats.percentile(99.0).unwrap_or(0),
            self.latency_stats.min().unwrap_or(0),
            self.latency_stats.max().unwrap_or(0),
        )
    }

    fn should_reconnect(&self) -> bool {
        // Reconnect if:
        // 1. Missed 3+ pongs in a row
        // 2. Success rate below 90%
        // 3. Latency p99 > 1000ms
        self.missed_pongs >= 3
            || self.success_rate() < 90.0
            || self.latency_stats.percentile(99.0).unwrap_or(0) > 1000
    }

    fn is_suitable_for_trading(&self) -> bool {
        // For active trading we need:
        // 1. Latency p99 < 100ms
        // 2. Success rate > 99%
        // 3. No missed pongs
        self.latency_stats.percentile(99.0).unwrap_or(u64::MAX) < 100
            && self.success_rate() > 99.0
            && self.missed_pongs == 0
    }
}

fn main() {
    let mut monitor = ConnectionQualityMonitor::new();

    // Simulate operation
    for latency in [15, 18, 22, 19, 45, 21, 17, 120, 23, 19] {
        monitor.record_ping();
        monitor.record_pong(latency);
    }

    println!("{}", monitor.report());
    println!("\nSuitable for trading: {}", monitor.is_suitable_for_trading());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Ping-Pong | Heartbeat mechanism for checking connection liveness |
| Pong timeout | Maximum time to wait for ping response |
| Exponential backoff | Increasing delay between reconnection attempts |
| Jitter | Random offset to prevent thundering herd |
| Latency monitoring | Tracking delay to assess connection quality |
| Auto-reconnection | Automatic restoration of lost connection |

## Practical Exercises

1. **Basic heartbeat**: Implement a simple TCP server and client with ping-pong mechanism. The server should disconnect clients that haven't sent a ping for more than 30 seconds.

2. **Latency monitoring**: Add latency statistics collection to the client and output a warning if p95 latency exceeds 100ms.

3. **Smart reconnection**: Implement a client with exponential backoff and jitter. Add maximum attempt count and notification when unable to connect.

4. **Multiplexing**: Create a client that maintains connections to multiple exchanges simultaneously and tracks the quality of each connection separately.

## Homework

1. **Exchange Connection Simulator**: Create a program that:
   - Connects to a "quote server" (you can use your own TCP server)
   - Sends ping every 5 seconds
   - Tracks latency and outputs statistics every minute
   - Automatically reconnects when connection is lost

2. **Network Problem Detector**: Extend the program from exercise 1:
   - Add detection of "silent" disconnects (when data just stops coming)
   - Implement alerts for connection quality degradation
   - Add logging of all issues for later analysis

3. **Connection Balancer**: Create a client that:
   - Maintains connections to multiple servers
   - Selects the server with lowest latency for sending orders
   - Switches to backup server when primary has issues

4. **Stability Analyzer**: Write a program that:
   - Collects ping-pong statistics over extended period
   - Identifies degradation patterns (e.g., issues at certain times of day)
   - Outputs recommendations for connection optimization

## Navigation

[← Previous day](../205-websocket-streaming-data/en.md) | [Next day →](../207-graceful-shutdown/en.md)
