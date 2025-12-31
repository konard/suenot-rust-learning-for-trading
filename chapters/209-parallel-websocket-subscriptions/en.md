# Day 209: Parallel WebSocket Subscriptions

## Trading Analogy

Imagine you're monitoring multiple cryptocurrency exchanges simultaneously: Binance, Kraken, and Coinbase. BTC prices can differ on each exchange, and arbitrage opportunities appear for just milliseconds. If you connect to exchanges sequentially — first Binance, then Kraken, then Coinbase — you'll lose precious time and miss profitable trades.

**Parallel WebSocket subscriptions** allow you to connect to all exchanges at once and receive price updates in real time. It's like having three traders, each watching their own exchange and instantly reporting changes.

In real algorithmic trading, this is critical for:
- **Arbitrage** — monitoring price differences between exchanges
- **Liquidity aggregation** — combining order books from different venues
- **Multi-asset strategies** — tracking correlations between assets
- **Risk management** — monitoring positions across different exchanges

## Theory: Running Parallel Tasks in Tokio

Tokio provides several ways to run tasks in parallel:

| Method | Description | Use Case |
|--------|-------------|----------|
| `tokio::spawn` | Launch an independent task | Background operations |
| `tokio::join!` | Wait for all tasks | Parallel launch with waiting |
| `tokio::select!` | Wait for first task | Racing between tasks |
| `FuturesUnordered` | Dynamic management | Changing set of tasks |

## Basic Example: Monitoring Multiple Pairs

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, interval};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct PriceUpdate {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct ArbitrageOpportunity {
    symbol: String,
    buy_exchange: String,
    sell_exchange: String,
    buy_price: f64,
    sell_price: f64,
    spread_percent: f64,
}

// Simulating WebSocket connection to an exchange
async fn subscribe_to_exchange(
    exchange: &str,
    symbols: Vec<&str>,
    tx: mpsc::Sender<PriceUpdate>,
) {
    println!("[{}] Connecting to WebSocket...", exchange);

    // Simulating connection delay
    sleep(Duration::from_millis(100)).await;
    println!("[{}] WebSocket connected!", exchange);

    let mut price_interval = interval(Duration::from_millis(500));
    let mut counter = 0u64;

    loop {
        price_interval.tick().await;
        counter += 1;

        for symbol in &symbols {
            // Simulating price reception (in reality this would be WebSocket message parsing)
            let base_price = match *symbol {
                "BTC/USDT" => 42000.0,
                "ETH/USDT" => 2200.0,
                "SOL/USDT" => 95.0,
                _ => 100.0,
            };

            // Adding random offset for each exchange
            let exchange_offset = match exchange {
                "Binance" => 0.0,
                "Kraken" => 15.0,
                "Coinbase" => -10.0,
                _ => 0.0,
            };

            let price = base_price + exchange_offset + (counter as f64 % 20.0) - 10.0;
            let spread = price * 0.001; // 0.1% spread

            let update = PriceUpdate {
                exchange: exchange.to_string(),
                symbol: symbol.to_string(),
                bid: price - spread / 2.0,
                ask: price + spread / 2.0,
                timestamp: counter,
            };

            if tx.send(update).await.is_err() {
                println!("[{}] Receiver disconnected, terminating", exchange);
                return;
            }
        }
    }
}

// Price aggregator from all exchanges
async fn price_aggregator(mut rx: mpsc::Receiver<PriceUpdate>) {
    // Storing latest prices: exchange -> symbol -> PriceUpdate
    let mut prices: HashMap<String, HashMap<String, PriceUpdate>> = HashMap::new();
    let mut update_count = 0;

    println!("\n=== Price Aggregator Started ===\n");

    while let Some(update) = rx.recv().await {
        update_count += 1;

        // Save the update
        prices
            .entry(update.exchange.clone())
            .or_insert_with(HashMap::new)
            .insert(update.symbol.clone(), update.clone());

        // Check for arbitrage every 10 updates
        if update_count % 10 == 0 {
            check_arbitrage(&prices);
        }

        // For demonstration, output every 5th update
        if update_count % 5 == 0 {
            println!(
                "[{}] {}: bid={:.2}, ask={:.2}",
                update.exchange, update.symbol, update.bid, update.ask
            );
        }

        // Stop after 50 updates for demonstration
        if update_count >= 50 {
            println!("\n=== Demo completed (50 updates) ===");
            break;
        }
    }
}

fn check_arbitrage(prices: &HashMap<String, HashMap<String, PriceUpdate>>) {
    let symbols = ["BTC/USDT", "ETH/USDT", "SOL/USDT"];

    for symbol in symbols {
        let mut best_bid: Option<(&str, f64)> = None;
        let mut best_ask: Option<(&str, f64)> = None;

        for (exchange, symbol_prices) in prices {
            if let Some(price) = symbol_prices.get(symbol) {
                // Best bid — highest (we can sell)
                if best_bid.is_none() || price.bid > best_bid.unwrap().1 {
                    best_bid = Some((exchange.as_str(), price.bid));
                }
                // Best ask — lowest (we can buy)
                if best_ask.is_none() || price.ask < best_ask.unwrap().1 {
                    best_ask = Some((exchange.as_str(), price.ask));
                }
            }
        }

        if let (Some((sell_ex, sell_price)), Some((buy_ex, buy_price))) = (best_bid, best_ask) {
            if sell_price > buy_price && sell_ex != buy_ex {
                let spread_percent = (sell_price - buy_price) / buy_price * 100.0;
                if spread_percent > 0.05 {
                    println!(
                        "\n!!! ARBITRAGE {} !!!\n    Buy on {} at {:.2}\n    Sell on {} at {:.2}\n    Spread: {:.3}%\n",
                        symbol, buy_ex, buy_price, sell_ex, sell_price, spread_percent
                    );
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Parallel WebSocket Subscriptions ===\n");

    // Channel for price updates
    let (tx, rx) = mpsc::channel::<PriceUpdate>(100);

    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT"];

    // Launch subscriptions to all exchanges in parallel
    let binance = {
        let tx = tx.clone();
        let symbols = symbols.clone();
        tokio::spawn(async move {
            subscribe_to_exchange("Binance", symbols, tx).await;
        })
    };

    let kraken = {
        let tx = tx.clone();
        let symbols = symbols.clone();
        tokio::spawn(async move {
            subscribe_to_exchange("Kraken", symbols, tx).await;
        })
    };

    let coinbase = {
        let tx = tx.clone();
        let symbols = symbols.clone();
        tokio::spawn(async move {
            subscribe_to_exchange("Coinbase", symbols, tx).await;
        })
    };

    // Important: close the original sender so the receiver can complete
    drop(tx);

    // Start the aggregator
    let aggregator = tokio::spawn(async move {
        price_aggregator(rx).await;
    });

    // Wait for the aggregator to finish (it will stop after 50 updates)
    let _ = aggregator.await;

    // Cancel subscriptions
    binance.abort();
    kraken.abort();
    coinbase.abort();

    println!("\nAll subscriptions closed");
}
```

## Using tokio::select! for Event Processing

`select!` allows you to wait for the first ready event from several:

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, timeout};

#[derive(Debug)]
enum TradingEvent {
    PriceUpdate { symbol: String, price: f64 },
    OrderFilled { order_id: u64, price: f64 },
    RiskAlert { message: String },
    Heartbeat,
}

async fn event_processor(
    mut price_rx: mpsc::Receiver<TradingEvent>,
    mut order_rx: mpsc::Receiver<TradingEvent>,
    mut risk_rx: mpsc::Receiver<TradingEvent>,
) {
    let heartbeat_interval = Duration::from_secs(5);
    let mut heartbeat = tokio::time::interval(heartbeat_interval);

    loop {
        tokio::select! {
            // Priority 1: Risk alerts (always process first)
            Some(event) = risk_rx.recv() => {
                match event {
                    TradingEvent::RiskAlert { message } => {
                        println!("!!! RISK ALERT: {} !!!", message);
                        // In reality, this could trigger emergency trading halt
                    }
                    _ => {}
                }
            }

            // Priority 2: Order executions
            Some(event) = order_rx.recv() => {
                match event {
                    TradingEvent::OrderFilled { order_id, price } => {
                        println!("Order #{} filled at price {:.2}", order_id, price);
                    }
                    _ => {}
                }
            }

            // Priority 3: Price updates
            Some(event) = price_rx.recv() => {
                match event {
                    TradingEvent::PriceUpdate { symbol, price } => {
                        println!("Price {}: {:.2}", symbol, price);
                    }
                    _ => {}
                }
            }

            // Heartbeat for connection health check
            _ = heartbeat.tick() => {
                println!("[Heartbeat] System operational");
            }

            // All channels closed
            else => {
                println!("All event sources closed");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (price_tx, price_rx) = mpsc::channel(100);
    let (order_tx, order_rx) = mpsc::channel(100);
    let (risk_tx, risk_rx) = mpsc::channel(100);

    // Start the event processor
    let processor = tokio::spawn(async move {
        event_processor(price_rx, order_rx, risk_rx).await;
    });

    // Simulate events
    let price_tx_clone = price_tx.clone();
    tokio::spawn(async move {
        for i in 0..5 {
            sleep(Duration::from_millis(200)).await;
            let _ = price_tx_clone.send(TradingEvent::PriceUpdate {
                symbol: "BTC/USDT".to_string(),
                price: 42000.0 + i as f64 * 10.0,
            }).await;
        }
    });

    tokio::spawn(async move {
        sleep(Duration::from_millis(500)).await;
        let _ = order_tx.send(TradingEvent::OrderFilled {
            order_id: 12345,
            price: 42050.0,
        }).await;
    });

    tokio::spawn(async move {
        sleep(Duration::from_millis(800)).await;
        let _ = risk_tx.send(TradingEvent::RiskAlert {
            message: "Daily loss limit exceeded".to_string(),
        }).await;
    });

    // Wait a bit and close
    sleep(Duration::from_secs(2)).await;
    drop(price_tx);

    let _ = processor.await;
}
```

## Advanced Example: Multi-Exchange Trading Bot

```rust
use tokio::sync::{mpsc, RwLock, broadcast};
use tokio::time::{sleep, Duration, interval};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct MarketData {
    exchange: String,
    symbol: String,
    bid: f64,
    ask: f64,
    bid_volume: f64,
    ask_volume: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    exchange: String,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    status: OrderStatus,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone)]
enum Command {
    PlaceOrder(Order),
    CancelOrder(u64),
    Shutdown,
}

struct TradingBot {
    // Current market data from all exchanges
    market_data: Arc<RwLock<HashMap<String, HashMap<String, MarketData>>>>,
    // Active orders
    orders: Arc<RwLock<HashMap<u64, Order>>>,
    // Order counter
    order_counter: Arc<RwLock<u64>>,
}

impl TradingBot {
    fn new() -> Self {
        TradingBot {
            market_data: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            order_counter: Arc::new(RwLock::new(0)),
        }
    }

    // Market data handler
    async fn market_data_handler(
        &self,
        mut rx: mpsc::Receiver<MarketData>,
        strategy_tx: broadcast::Sender<MarketData>,
    ) {
        while let Some(data) = rx.recv().await {
            // Save data
            {
                let mut market = self.market_data.write().await;
                market
                    .entry(data.exchange.clone())
                    .or_insert_with(HashMap::new)
                    .insert(data.symbol.clone(), data.clone());
            }

            // Notify strategy
            let _ = strategy_tx.send(data);
        }
    }

    // Simple arbitrage strategy
    async fn arbitrage_strategy(
        &self,
        mut data_rx: broadcast::Receiver<MarketData>,
        command_tx: mpsc::Sender<Command>,
    ) {
        let min_spread_percent = 0.1; // Minimum spread for arbitrage

        while let Ok(data) = data_rx.recv().await {
            let market = self.market_data.read().await;

            // Look for arbitrage opportunities
            for (other_exchange, symbols) in market.iter() {
                if *other_exchange == data.exchange {
                    continue;
                }

                if let Some(other_data) = symbols.get(&data.symbol) {
                    // Check: can we buy on one exchange and sell on another?

                    // Option 1: Buy on data.exchange, sell on other_exchange
                    let spread1 = (other_data.bid - data.ask) / data.ask * 100.0;
                    if spread1 > min_spread_percent {
                        println!(
                            "\n>>> Arbitrage found! {} <<<",
                            data.symbol
                        );
                        println!(
                            "    Buy on {} at {:.2}",
                            data.exchange, data.ask
                        );
                        println!(
                            "    Sell on {} at {:.2}",
                            other_exchange, other_data.bid
                        );
                        println!("    Profit: {:.3}%\n", spread1);

                        // Place orders (in reality, atomicity is needed)
                        let quantity = 0.01; // Minimum volume

                        let mut counter = self.order_counter.write().await;
                        *counter += 1;
                        let buy_order_id = *counter;
                        *counter += 1;
                        let sell_order_id = *counter;

                        // Buy order
                        let _ = command_tx.send(Command::PlaceOrder(Order {
                            id: buy_order_id,
                            exchange: data.exchange.clone(),
                            symbol: data.symbol.clone(),
                            side: OrderSide::Buy,
                            price: data.ask,
                            quantity,
                            status: OrderStatus::Pending,
                        })).await;

                        // Sell order
                        let _ = command_tx.send(Command::PlaceOrder(Order {
                            id: sell_order_id,
                            exchange: other_exchange.clone(),
                            symbol: data.symbol.clone(),
                            side: OrderSide::Sell,
                            price: other_data.bid,
                            quantity,
                            status: OrderStatus::Pending,
                        })).await;
                    }
                }
            }
        }
    }

    // Command handler (order execution)
    async fn order_handler(&self, mut rx: mpsc::Receiver<Command>) {
        while let Some(command) = rx.recv().await {
            match command {
                Command::PlaceOrder(order) => {
                    println!(
                        "Placing order #{}: {:?} {} {} at {:.2}",
                        order.id, order.side, order.quantity, order.symbol, order.price
                    );

                    // Simulate sending to exchange
                    let mut orders = self.orders.write().await;
                    orders.insert(order.id, order.clone());

                    // In reality, this would be a REST API call to the exchange
                    sleep(Duration::from_millis(10)).await;

                    // Simulate execution
                    if let Some(o) = orders.get_mut(&order.id) {
                        o.status = OrderStatus::Filled;
                        println!("Order #{} filled!", order.id);
                    }
                }
                Command::CancelOrder(order_id) => {
                    let mut orders = self.orders.write().await;
                    if let Some(order) = orders.get_mut(&order_id) {
                        order.status = OrderStatus::Cancelled;
                        println!("Order #{} cancelled", order_id);
                    }
                }
                Command::Shutdown => {
                    println!("Shutdown command received");
                    break;
                }
            }
        }
    }
}

// Exchange WebSocket simulator
async fn exchange_simulator(
    exchange: &str,
    symbols: Vec<&str>,
    tx: mpsc::Sender<MarketData>,
) {
    let mut tick_interval = interval(Duration::from_millis(100));
    let mut counter = 0u64;

    println!("[{}] Simulator started", exchange);

    loop {
        tick_interval.tick().await;
        counter += 1;

        for symbol in &symbols {
            let base_price = match *symbol {
                "BTC/USDT" => 42000.0,
                "ETH/USDT" => 2200.0,
                _ => 100.0,
            };

            // Different prices on different exchanges
            let exchange_modifier = match exchange {
                "Binance" => 1.0,
                "Kraken" => 1.002,  // 0.2% more expensive
                "Coinbase" => 0.998, // 0.2% cheaper
                _ => 1.0,
            };

            // Add noise
            let noise = ((counter as f64).sin() * 10.0) +
                       ((counter as f64 * 0.7).cos() * 5.0);

            let mid_price = base_price * exchange_modifier + noise;
            let spread = mid_price * 0.0005; // 0.05% spread

            let data = MarketData {
                exchange: exchange.to_string(),
                symbol: symbol.to_string(),
                bid: mid_price - spread,
                ask: mid_price + spread,
                bid_volume: 1.0 + (counter as f64 % 5.0),
                ask_volume: 1.0 + ((counter + 2) as f64 % 5.0),
                timestamp: counter,
            };

            if tx.send(data).await.is_err() {
                println!("[{}] Channel closed, terminating", exchange);
                return;
            }
        }

        // Limit updates for demonstration
        if counter >= 30 {
            println!("[{}] Simulation complete", exchange);
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Multi-Exchange Trading Bot ===\n");

    let bot = Arc::new(TradingBot::new());

    // Channels
    let (market_tx, market_rx) = mpsc::channel::<MarketData>(1000);
    let (strategy_tx, _) = broadcast::channel::<MarketData>(1000);
    let strategy_rx = strategy_tx.subscribe();
    let (command_tx, command_rx) = mpsc::channel::<Command>(100);

    let symbols = vec!["BTC/USDT", "ETH/USDT"];

    // Start market data handler
    let bot_clone = Arc::clone(&bot);
    let strategy_tx_clone = strategy_tx.clone();
    let market_handler = tokio::spawn(async move {
        bot_clone.market_data_handler(market_rx, strategy_tx_clone).await;
    });

    // Start strategy
    let bot_clone = Arc::clone(&bot);
    let strategy = tokio::spawn(async move {
        bot_clone.arbitrage_strategy(strategy_rx, command_tx).await;
    });

    // Start order handler
    let bot_clone = Arc::clone(&bot);
    let order_handler = tokio::spawn(async move {
        bot_clone.order_handler(command_rx).await;
    });

    // Start exchange simulators in parallel
    let exchanges = vec![
        ("Binance", symbols.clone()),
        ("Kraken", symbols.clone()),
        ("Coinbase", symbols.clone()),
    ];

    let mut exchange_handles = vec![];
    for (exchange, syms) in exchanges {
        let tx = market_tx.clone();
        let handle = tokio::spawn(async move {
            exchange_simulator(exchange, syms, tx).await;
        });
        exchange_handles.push(handle);
    }

    // Close original sender
    drop(market_tx);

    // Wait for simulators to finish
    for handle in exchange_handles {
        let _ = handle.await;
    }

    // Give time for remaining data processing
    sleep(Duration::from_millis(500)).await;

    // Print statistics
    let orders = bot.orders.read().await;
    println!("\n=== Statistics ===");
    println!("Total orders: {}", orders.len());
    println!(
        "Filled: {}",
        orders.values().filter(|o| o.status == OrderStatus::Filled).count()
    );

    println!("\nBot finished");
}
```

## Handling Reconnections and Errors

In real conditions, WebSocket connections can drop. It's important to be able to restore them:

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug)]
struct ConnectionState {
    is_connected: AtomicBool,
    reconnect_count: AtomicU32,
    max_reconnects: u32,
}

impl ConnectionState {
    fn new(max_reconnects: u32) -> Self {
        ConnectionState {
            is_connected: AtomicBool::new(false),
            reconnect_count: AtomicU32::new(0),
            max_reconnects,
        }
    }

    fn set_connected(&self, connected: bool) {
        self.is_connected.store(connected, Ordering::SeqCst);
    }

    fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    fn increment_reconnect(&self) -> bool {
        let count = self.reconnect_count.fetch_add(1, Ordering::SeqCst);
        count < self.max_reconnects
    }

    fn reset_reconnect_count(&self) {
        self.reconnect_count.store(0, Ordering::SeqCst);
    }
}

async fn resilient_websocket_connection(
    exchange: &str,
    state: Arc<ConnectionState>,
    tx: mpsc::Sender<String>,
) {
    loop {
        println!("[{}] Attempting connection...", exchange);

        // Simulate connection
        match simulate_connect(exchange).await {
            Ok(_) => {
                println!("[{}] Connected!", exchange);
                state.set_connected(true);
                state.reset_reconnect_count();

                // Simulate receiving data
                if let Err(e) = simulate_receive_data(exchange, &tx).await {
                    println!("[{}] Error receiving data: {}", exchange, e);
                    state.set_connected(false);
                }
            }
            Err(e) => {
                println!("[{}] Connection error: {}", exchange, e);
            }
        }

        // Try to reconnect
        if !state.increment_reconnect() {
            println!("[{}] Maximum reconnection attempts exceeded", exchange);
            break;
        }

        let reconnect_delay = Duration::from_secs(
            2u64.pow(state.reconnect_count.load(Ordering::SeqCst).min(5))
        );
        println!(
            "[{}] Reconnecting in {:?}...",
            exchange, reconnect_delay
        );
        sleep(reconnect_delay).await;
    }
}

async fn simulate_connect(exchange: &str) -> Result<(), String> {
    sleep(Duration::from_millis(100)).await;

    // Simulate random connection errors
    if exchange == "Kraken" && rand_bool(0.3) {
        return Err("Connection refused".to_string());
    }

    Ok(())
}

async fn simulate_receive_data(
    exchange: &str,
    tx: &mpsc::Sender<String>,
) -> Result<(), String> {
    for i in 0..10 {
        sleep(Duration::from_millis(200)).await;

        // Simulate random connection drop
        if rand_bool(0.1) {
            return Err("Connection reset by peer".to_string());
        }

        let msg = format!("[{}] Tick {}: BTC = {:.2}", exchange, i, 42000.0 + i as f64 * 10.0);
        if tx.send(msg).await.is_err() {
            return Err("Channel closed".to_string());
        }
    }

    Ok(())
}

// Simple random number generator for demonstration
fn rand_bool(probability: f64) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f64 / u32::MAX as f64) < probability
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(100);

    let state = Arc::new(ConnectionState::new(5));

    let handle = {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            resilient_websocket_connection("Binance", state, tx).await;
        })
    };

    // Receive messages
    while let Some(msg) = rx.recv().await {
        println!("{}", msg);
    }

    let _ = handle.await;
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `tokio::spawn` | Launch an async task in the background |
| `mpsc::channel` | Channel for passing data between tasks |
| `broadcast::channel` | Channel with multiple receivers |
| `tokio::select!` | Wait for first ready event |
| `RwLock` | Read/write lock for async |
| Exponential backoff | Increasing delay for reconnection |

## Homework

1. **Spread Monitor**: Create a program that connects to multiple "exchanges" and tracks the spread (difference between bid and ask) in real time for each pair. Output a warning if the spread exceeds 0.5%.

2. **Anomaly Detector**: Implement a system that:
   - Receives prices from multiple exchanges
   - Calculates the average price
   - Detects if a price on any exchange deviates more than 1% from the average
   - Logs all anomalies

3. **Order Book Synchronizer**: Write a program that:
   - Receives order book data (bid/ask with volumes) from three exchanges
   - Combines them into a single aggregated order book
   - Shows top-5 best bids and asks across all exchanges

4. **Alert System**: Create a system with priority alert levels:
   - Critical (price spike > 5%) — immediate trading halt
   - Important (spread > 1%) — warning
   - Informational (regular price updates) — logging

   Use `tokio::select!` with priority handling.

## Navigation

[← Previous day](../208-processing-websocket-messages/en.md) | [Next day →](../210-graceful-shutdown-async-tasks/en.md)
