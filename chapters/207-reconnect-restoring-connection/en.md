# Day 207: Reconnect: Restoring Connection

## Trading Analogy

Imagine you're running an algorithmic trading bot that receives real-time price data from an exchange via WebSocket. Suddenly, the connection drops — the exchange server restarts, your network hiccups, or the connection times out. If your bot doesn't automatically reconnect, you'll miss critical price movements and trading opportunities. In real trading, **reconnection logic** is not optional — it's essential for survival.

A robust trading system must:
- Detect when a connection is lost
- Attempt to reconnect automatically
- Use exponential backoff to avoid overwhelming the server
- Restore subscriptions after reconnecting
- Handle partial failures gracefully

## What is Reconnection?

Reconnection is the process of automatically re-establishing a network connection after it has been lost. In async Rust with tokio, we implement reconnection using:

1. **Connection state tracking** — knowing when we're connected vs disconnected
2. **Retry logic** — attempting to reconnect with appropriate delays
3. **Exponential backoff** — increasing delays between retries to prevent server overload
4. **Maximum retry limits** — giving up after too many failures
5. **State restoration** — re-subscribing to channels after reconnecting

## Simple Reconnection Pattern

```rust
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for reconnection behavior
#[derive(Debug, Clone)]
struct ReconnectConfig {
    initial_delay: Duration,
    max_delay: Duration,
    max_retries: u32,
    backoff_multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        ReconnectConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            max_retries: 10,
            backoff_multiplier: 2.0,
        }
    }
}

/// Simulates connecting to an exchange (may fail)
async fn connect_to_exchange(url: &str) -> Result<String, String> {
    // Simulate network latency
    sleep(Duration::from_millis(50)).await;

    // Simulate random connection failures (30% failure rate)
    if rand::random::<f32>() < 0.3 {
        Err(format!("Failed to connect to {}", url))
    } else {
        Ok(format!("Connected to {}", url))
    }
}

/// Reconnection loop with exponential backoff
async fn connect_with_retry(url: &str, config: &ReconnectConfig) -> Result<String, String> {
    let mut attempts = 0;
    let mut delay = config.initial_delay;

    loop {
        attempts += 1;
        println!("Connection attempt {} to {}...", attempts, url);

        match connect_to_exchange(url).await {
            Ok(connection) => {
                println!("Successfully connected after {} attempts!", attempts);
                return Ok(connection);
            }
            Err(e) => {
                println!("Attempt {} failed: {}", attempts, e);

                if attempts >= config.max_retries {
                    return Err(format!(
                        "Failed to connect after {} attempts",
                        config.max_retries
                    ));
                }

                println!("Retrying in {:?}...", delay);
                sleep(delay).await;

                // Exponential backoff
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.backoff_multiplier)
                        .min(config.max_delay.as_secs_f64())
                );
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let config = ReconnectConfig::default();

    match connect_with_retry("wss://exchange.example.com/ws", &config).await {
        Ok(conn) => println!("Final result: {}", conn),
        Err(e) => println!("Connection failed: {}", e),
    }
}
```

## Trading Price Feed with Reconnection

Here's a more realistic example — a price feed client that automatically reconnects:

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Instant};

/// Represents a price update from the exchange
#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: Instant,
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting { attempt: u32 },
}

/// Price feed client with automatic reconnection
struct PriceFeedClient {
    url: String,
    state: Arc<RwLock<ConnectionState>>,
    subscriptions: Arc<RwLock<Vec<String>>>,
    config: ReconnectConfig,
}

#[derive(Debug, Clone)]
struct ReconnectConfig {
    initial_delay: Duration,
    max_delay: Duration,
    max_retries: u32,
    backoff_multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        ReconnectConfig {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            max_retries: 5,
            backoff_multiplier: 2.0,
        }
    }
}

impl PriceFeedClient {
    fn new(url: &str, config: ReconnectConfig) -> Self {
        PriceFeedClient {
            url: url.to_string(),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Subscribe to a trading pair
    async fn subscribe(&self, symbol: &str) {
        let mut subs = self.subscriptions.write().await;
        if !subs.contains(&symbol.to_string()) {
            subs.push(symbol.to_string());
            println!("Subscribed to {}", symbol);
        }
    }

    /// Simulate establishing a connection
    async fn establish_connection(&self) -> Result<(), String> {
        // Simulate connection attempt
        sleep(Duration::from_millis(100)).await;

        // 20% chance of failure
        if rand::random::<f32>() < 0.2 {
            Err("Connection refused".to_string())
        } else {
            Ok(())
        }
    }

    /// Restore subscriptions after reconnecting
    async fn restore_subscriptions(&self) -> Result<(), String> {
        let subs = self.subscriptions.read().await;
        for symbol in subs.iter() {
            println!("Restoring subscription: {}", symbol);
            // Simulate subscription request
            sleep(Duration::from_millis(50)).await;
        }
        println!("All {} subscriptions restored", subs.len());
        Ok(())
    }

    /// Main connection loop with automatic reconnection
    async fn run(&self, price_tx: mpsc::Sender<PriceUpdate>) -> Result<(), String> {
        loop {
            // Update state to Connecting
            {
                let mut state = self.state.write().await;
                *state = ConnectionState::Connecting;
            }

            // Try to connect with retries
            let mut attempts = 0;
            let mut delay = self.config.initial_delay;

            let connected = loop {
                attempts += 1;

                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Reconnecting { attempt: attempts };
                }

                println!("Connection attempt {}...", attempts);

                match self.establish_connection().await {
                    Ok(()) => {
                        println!("Connected to {}", self.url);
                        break true;
                    }
                    Err(e) => {
                        println!("Connection failed: {}", e);

                        if attempts >= self.config.max_retries {
                            println!("Max retries reached, giving up");
                            break false;
                        }

                        println!("Waiting {:?} before retry...", delay);
                        sleep(delay).await;

                        delay = Duration::from_secs_f64(
                            (delay.as_secs_f64() * self.config.backoff_multiplier)
                                .min(self.config.max_delay.as_secs_f64())
                        );
                    }
                }
            };

            if !connected {
                return Err("Failed to establish connection".to_string());
            }

            // Update state to Connected
            {
                let mut state = self.state.write().await;
                *state = ConnectionState::Connected;
            }

            // Restore subscriptions
            self.restore_subscriptions().await?;

            // Simulate receiving price updates
            let result = self.receive_prices(&price_tx).await;

            match result {
                Ok(()) => {
                    // Clean shutdown
                    println!("Connection closed gracefully");
                    return Ok(());
                }
                Err(e) => {
                    // Connection lost, will reconnect
                    println!("Connection lost: {}. Reconnecting...", e);

                    {
                        let mut state = self.state.write().await;
                        *state = ConnectionState::Disconnected;
                    }

                    // Small delay before reconnection attempt
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Simulate receiving price updates (may fail to simulate disconnection)
    async fn receive_prices(&self, price_tx: &mpsc::Sender<PriceUpdate>) -> Result<(), String> {
        let subs = self.subscriptions.read().await;
        let symbols: Vec<String> = subs.clone();
        drop(subs);

        for i in 0..10 {
            // Simulate random disconnection (10% per iteration)
            if rand::random::<f32>() < 0.1 {
                return Err("Connection reset by peer".to_string());
            }

            // Generate price updates for all subscribed symbols
            for symbol in &symbols {
                let base_price = match symbol.as_str() {
                    "BTC/USDT" => 42000.0,
                    "ETH/USDT" => 2500.0,
                    _ => 100.0,
                };

                let price_update = PriceUpdate {
                    symbol: symbol.clone(),
                    bid: base_price + (i as f64 * 10.0) - 5.0,
                    ask: base_price + (i as f64 * 10.0) + 5.0,
                    timestamp: Instant::now(),
                };

                if price_tx.send(price_update).await.is_err() {
                    return Ok(()); // Receiver dropped, clean shutdown
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let config = ReconnectConfig::default();
    let client = Arc::new(PriceFeedClient::new("wss://exchange.example.com/ws", config));

    // Subscribe to trading pairs
    client.subscribe("BTC/USDT").await;
    client.subscribe("ETH/USDT").await;

    // Channel for price updates
    let (price_tx, mut price_rx) = mpsc::channel(100);

    // Spawn connection task
    let client_clone = Arc::clone(&client);
    let connection_task = tokio::spawn(async move {
        client_clone.run(price_tx).await
    });

    // Process price updates
    let processing_task = tokio::spawn(async move {
        while let Some(update) = price_rx.recv().await {
            println!(
                "Price: {} bid={:.2} ask={:.2}",
                update.symbol, update.bid, update.ask
            );
        }
    });

    // Wait for connection task to complete
    match connection_task.await {
        Ok(Ok(())) => println!("Connection task completed successfully"),
        Ok(Err(e)) => println!("Connection task failed: {}", e),
        Err(e) => println!("Connection task panicked: {}", e),
    }

    processing_task.abort();
}
```

## Reconnection with Circuit Breaker Pattern

For production systems, we often combine reconnection with a circuit breaker to prevent cascading failures:

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{sleep, Instant};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, reject requests
    HalfOpen,    // Testing if service recovered
}

/// Circuit breaker for connection management
struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure: Arc<RwLock<Option<Instant>>>,
    failure_threshold: u32,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        CircuitBreaker {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure: Arc::new(RwLock::new(None)),
            failure_threshold,
            reset_timeout,
        }
    }

    /// Check if we should attempt a connection
    async fn should_attempt(&self) -> bool {
        let state = self.state.read().await;

        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if reset timeout has passed
                let last = self.last_failure.read().await;
                if let Some(last_time) = *last {
                    if last_time.elapsed() >= self.reset_timeout {
                        drop(state);
                        drop(last);
                        let mut state = self.state.write().await;
                        *state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful connection
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        let mut count = self.failure_count.write().await;

        *state = CircuitState::Closed;
        *count = 0;

        println!("Circuit breaker: Connection successful, circuit CLOSED");
    }

    /// Record a failed connection
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let mut count = self.failure_count.write().await;
        let mut last = self.last_failure.write().await;

        *count += 1;
        *last = Some(Instant::now());

        if *count >= self.failure_threshold {
            *state = CircuitState::Open;
            println!(
                "Circuit breaker: {} failures, circuit OPEN for {:?}",
                *count, self.reset_timeout
            );
        }
    }
}

/// Exchange connection with circuit breaker
struct ExchangeConnection {
    url: String,
    circuit_breaker: CircuitBreaker,
}

impl ExchangeConnection {
    fn new(url: &str) -> Self {
        ExchangeConnection {
            url: url.to_string(),
            circuit_breaker: CircuitBreaker::new(3, Duration::from_secs(10)),
        }
    }

    /// Attempt connection with circuit breaker protection
    async fn connect(&self) -> Result<(), String> {
        if !self.circuit_breaker.should_attempt().await {
            return Err("Circuit breaker is OPEN, refusing connection".to_string());
        }

        // Simulate connection attempt
        sleep(Duration::from_millis(100)).await;

        // 40% failure rate for demo
        if rand::random::<f32>() < 0.4 {
            self.circuit_breaker.record_failure().await;
            Err("Connection failed".to_string())
        } else {
            self.circuit_breaker.record_success().await;
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() {
    let connection = ExchangeConnection::new("wss://exchange.example.com/ws");

    // Attempt multiple connections
    for i in 1..=10 {
        println!("\n--- Attempt {} ---", i);

        match connection.connect().await {
            Ok(()) => println!("Connected successfully!"),
            Err(e) => println!("Failed: {}", e),
        }

        sleep(Duration::from_secs(2)).await;
    }
}
```

## Multi-Exchange Reconnection Manager

For trading systems connecting to multiple exchanges:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Status of a single exchange connection
#[derive(Debug, Clone)]
struct ExchangeStatus {
    name: String,
    connected: bool,
    last_message: Option<std::time::Instant>,
    reconnect_attempts: u32,
}

/// Manager for multiple exchange connections
struct MultiExchangeManager {
    exchanges: Arc<RwLock<HashMap<String, ExchangeStatus>>>,
}

impl MultiExchangeManager {
    fn new() -> Self {
        MultiExchangeManager {
            exchanges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an exchange to manage
    async fn add_exchange(&self, name: &str) {
        let mut exchanges = self.exchanges.write().await;
        exchanges.insert(name.to_string(), ExchangeStatus {
            name: name.to_string(),
            connected: false,
            last_message: None,
            reconnect_attempts: 0,
        });
    }

    /// Simulate connecting to an exchange
    async fn connect_exchange(&self, name: &str) -> Result<(), String> {
        sleep(Duration::from_millis(100)).await;

        // Different failure rates per exchange
        let failure_rate = match name {
            "Binance" => 0.1,
            "Kraken" => 0.2,
            "Coinbase" => 0.15,
            _ => 0.3,
        };

        if rand::random::<f32>() < failure_rate {
            Err(format!("{} connection refused", name))
        } else {
            Ok(())
        }
    }

    /// Connect to all exchanges with individual reconnection
    async fn connect_all(&self) {
        let exchanges = self.exchanges.read().await;
        let exchange_names: Vec<String> = exchanges.keys().cloned().collect();
        drop(exchanges);

        let mut handles = Vec::new();

        for name in exchange_names {
            let manager = Arc::new(self.exchanges.clone());
            let exchange_name = name.clone();

            let handle = tokio::spawn(async move {
                let mut attempts = 0;
                let max_attempts = 5;
                let mut delay = Duration::from_millis(500);

                loop {
                    attempts += 1;
                    println!("[{}] Connection attempt {}...", exchange_name, attempts);

                    // Simulate connection
                    sleep(Duration::from_millis(100)).await;
                    let success = rand::random::<f32>() > 0.3;

                    if success {
                        let mut exchanges = manager.write().await;
                        if let Some(status) = exchanges.get_mut(&exchange_name) {
                            status.connected = true;
                            status.last_message = Some(std::time::Instant::now());
                            status.reconnect_attempts = attempts;
                        }
                        println!("[{}] Connected after {} attempts!", exchange_name, attempts);
                        break;
                    } else {
                        println!("[{}] Failed, retrying in {:?}...", exchange_name, delay);

                        if attempts >= max_attempts {
                            println!("[{}] Max attempts reached!", exchange_name);
                            break;
                        }

                        sleep(delay).await;
                        delay = std::cmp::min(delay * 2, Duration::from_secs(10));
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all connections
        for handle in handles {
            let _ = handle.await;
        }
    }

    /// Get status summary
    async fn get_status_summary(&self) -> String {
        let exchanges = self.exchanges.read().await;
        let connected: Vec<_> = exchanges.values()
            .filter(|e| e.connected)
            .map(|e| e.name.as_str())
            .collect();
        let disconnected: Vec<_> = exchanges.values()
            .filter(|e| !e.connected)
            .map(|e| e.name.as_str())
            .collect();

        format!(
            "Connected: {:?}, Disconnected: {:?}",
            connected, disconnected
        )
    }
}

#[tokio::main]
async fn main() {
    let manager = MultiExchangeManager::new();

    // Add exchanges to manage
    manager.add_exchange("Binance").await;
    manager.add_exchange("Kraken").await;
    manager.add_exchange("Coinbase").await;
    manager.add_exchange("FTX").await;

    println!("Starting multi-exchange connection...\n");

    // Connect to all exchanges in parallel
    manager.connect_all().await;

    println!("\n{}", manager.get_status_summary().await);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Reconnection | Automatically re-establishing a lost connection |
| Exponential Backoff | Increasing delays between retry attempts |
| Circuit Breaker | Pattern to prevent cascading failures |
| State Restoration | Re-subscribing to channels after reconnecting |
| Connection State | Tracking whether we're connected, disconnected, or reconnecting |
| Multi-connection | Managing connections to multiple exchanges simultaneously |

## Homework

1. **Basic Reconnection**: Implement a simple price feed client that:
   - Connects to a simulated exchange
   - Automatically reconnects when the connection drops
   - Uses exponential backoff with a maximum delay of 30 seconds
   - Logs all connection state changes

2. **Subscription Restoration**: Extend the client from exercise 1 to:
   - Maintain a list of subscribed trading pairs
   - Automatically re-subscribe to all pairs after reconnecting
   - Handle subscription failures gracefully

3. **Circuit Breaker Trading System**: Create a trading system that:
   - Connects to 3 exchanges (simulated)
   - Uses a circuit breaker for each connection
   - Stops trading on an exchange when its circuit opens
   - Resumes trading when the circuit closes
   - Logs all circuit state changes

4. **Health Monitor**: Build a connection health monitor that:
   - Tracks connection uptime for multiple exchanges
   - Calculates connection reliability percentage
   - Alerts when an exchange's reliability drops below 95%
   - Provides a dashboard summary of all connection health metrics

## Navigation

[← Previous day](../206-ping-pong-keeping-connection/en.md) | [Next day →](../208-processing-websocket-messages/en.md)
