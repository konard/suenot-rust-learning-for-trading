# Day 156: Channels mpsc — Order Queue

## Trading Analogy

Imagine a trading floor: multiple traders (senders) shout their orders, while a single market maker (receiver) accepts and processes them in order. This is **mpsc** — multiple producers, single consumer.

In algorithmic trading:
- **Producers** — modules generating trading signals (RSI analysis, MACD, support levels)
- **Consumer** — order execution engine that processes orders sequentially
- **Channel** — order queue between them

## What is mpsc

`mpsc` (multi-producer, single-consumer) is a channel for passing data between threads:

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // Create channel: tx — transmitter, rx — receiver
    let (tx, rx) = mpsc::channel();

    // Producer thread sends an order
    thread::spawn(move || {
        let order = "BUY BTC 0.5";
        tx.send(order).unwrap();
        println!("Order sent: {}", order);
    });

    // Main thread receives the order
    let received = rx.recv().unwrap();
    println!("Order received: {}", received);
}
```

## Channel Types

### Unbounded Channel

Queue with no limit — like an order book without restrictions:

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel(); // Unbounded channel

    thread::spawn(move || {
        for i in 1..=5 {
            let order = format!("ORDER_{}", i);
            tx.send(order).unwrap();
        }
    });

    // Receive all orders
    for order in rx {
        println!("Processing: {}", order);
    }
}
```

### Bounded Channel

Channel with fixed capacity — like a queue with order limit:

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // sync_channel with capacity 2 — buffer for 2 orders
    let (tx, rx) = mpsc::sync_channel(2);

    thread::spawn(move || {
        for i in 1..=5 {
            println!("Sending order {}", i);
            tx.send(i).unwrap(); // Blocks if buffer is full
            println!("Order {} sent", i);
        }
    });

    thread::sleep(std::time::Duration::from_millis(100));

    for order in rx {
        println!("Received order: {}", order);
        thread::sleep(std::time::Duration::from_millis(50));
    }
}
```

## Multiple Producers

Multiple signal sources send orders to a single executor:

```rust
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    source: String,
}

fn main() {
    let (tx, rx) = mpsc::channel();

    // Clone transmitter for each signal source
    let tx_rsi = tx.clone();
    let tx_macd = tx.clone();
    let tx_support = tx;

    // RSI analyzer
    thread::spawn(move || {
        let order = Order {
            symbol: "BTC/USDT".to_string(),
            side: "BUY".to_string(),
            quantity: 0.5,
            source: "RSI".to_string(),
        };
        tx_rsi.send(order).unwrap();
    });

    // MACD analyzer
    thread::spawn(move || {
        let order = Order {
            symbol: "ETH/USDT".to_string(),
            side: "SELL".to_string(),
            quantity: 2.0,
            source: "MACD".to_string(),
        };
        tx_macd.send(order).unwrap();
    });

    // Support level analyzer
    thread::spawn(move || {
        let order = Order {
            symbol: "BTC/USDT".to_string(),
            side: "BUY".to_string(),
            quantity: 0.3,
            source: "Support Level".to_string(),
        };
        tx_support.send(order).unwrap();
    });

    // Executor processes all orders
    for _ in 0..3 {
        match rx.recv() {
            Ok(order) => {
                println!("Executing order from {}: {} {} {}",
                    order.source, order.side, order.quantity, order.symbol);
            }
            Err(_) => break,
        }
    }
}
```

## Error Handling

```rust
use std::sync::mpsc::{self, RecvTimeoutError, TryRecvError};
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        tx.send("Delayed order".to_string()).unwrap();
    });

    // try_recv — non-blocking receive
    match rx.try_recv() {
        Ok(order) => println!("Received: {}", order),
        Err(TryRecvError::Empty) => println!("Queue is empty"),
        Err(TryRecvError::Disconnected) => println!("Channel closed"),
    }

    // recv_timeout — receive with timeout
    match rx.recv_timeout(Duration::from_millis(100)) {
        Ok(order) => println!("Received: {}", order),
        Err(RecvTimeoutError::Timeout) => println!("Timeout waiting"),
        Err(RecvTimeoutError::Disconnected) => println!("Channel closed"),
    }

    // Wait long enough
    match rx.recv_timeout(Duration::from_millis(200)) {
        Ok(order) => println!("Received: {}", order),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Practical Example: Order Execution System

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
enum OrderType {
    Market,
    Limit(f64),
    StopLoss(f64),
}

#[derive(Debug, Clone)]
enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: Side,
    order_type: OrderType,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug)]
enum ExecutionResult {
    Filled { order_id: u64, price: f64, quantity: f64 },
    PartialFill { order_id: u64, filled: f64, remaining: f64 },
    Rejected { order_id: u64, reason: String },
}

fn main() {
    let (order_tx, order_rx) = mpsc::channel::<Order>();
    let (result_tx, result_rx) = mpsc::channel::<ExecutionResult>();

    // Order execution thread
    let exec_result_tx = result_tx.clone();
    thread::spawn(move || {
        let mut current_price = 42000.0;

        for order in order_rx {
            println!("Processing order #{}: {:?} {} {}",
                order.id, order.side, order.quantity, order.symbol);

            // Simulate price change
            current_price += (order.id as f64 - 2.0) * 50.0;

            let result = match order.order_type {
                OrderType::Market => {
                    ExecutionResult::Filled {
                        order_id: order.id,
                        price: current_price,
                        quantity: order.quantity,
                    }
                }
                OrderType::Limit(limit_price) => {
                    let can_fill = match order.side {
                        Side::Buy => current_price <= limit_price,
                        Side::Sell => current_price >= limit_price,
                    };

                    if can_fill {
                        ExecutionResult::Filled {
                            order_id: order.id,
                            price: limit_price,
                            quantity: order.quantity,
                        }
                    } else {
                        ExecutionResult::Rejected {
                            order_id: order.id,
                            reason: format!(
                                "Price {} doesn't match limit {}",
                                current_price, limit_price
                            ),
                        }
                    }
                }
                OrderType::StopLoss(stop_price) => {
                    let triggered = match order.side {
                        Side::Buy => current_price >= stop_price,
                        Side::Sell => current_price <= stop_price,
                    };

                    if triggered {
                        ExecutionResult::Filled {
                            order_id: order.id,
                            price: current_price,
                            quantity: order.quantity,
                        }
                    } else {
                        ExecutionResult::Rejected {
                            order_id: order.id,
                            reason: format!(
                                "Stop-loss not triggered: price {} stop {}",
                                current_price, stop_price
                            ),
                        }
                    }
                }
            };

            exec_result_tx.send(result).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
    });

    // Result processing thread
    let result_handler = thread::spawn(move || {
        let mut filled_count = 0;
        let mut rejected_count = 0;
        let mut total_volume = 0.0;

        for result in result_rx {
            match result {
                ExecutionResult::Filled { order_id, price, quantity } => {
                    println!("  ✓ Order #{} filled: {} @ ${:.2}",
                        order_id, quantity, price);
                    filled_count += 1;
                    total_volume += price * quantity;
                }
                ExecutionResult::PartialFill { order_id, filled, remaining } => {
                    println!("  ◐ Order #{} partial: {} filled, {} remaining",
                        order_id, filled, remaining);
                }
                ExecutionResult::Rejected { order_id, reason } => {
                    println!("  ✗ Order #{} rejected: {}", order_id, reason);
                    rejected_count += 1;
                }
            }
        }

        println!("\n=== Statistics ===");
        println!("Filled: {}", filled_count);
        println!("Rejected: {}", rejected_count);
        println!("Total volume: ${:.2}", total_volume);
    });

    // Generate orders
    let orders = vec![
        Order {
            id: 1,
            symbol: "BTC/USDT".to_string(),
            side: Side::Buy,
            order_type: OrderType::Market,
            quantity: 0.5,
            timestamp: 1000,
        },
        Order {
            id: 2,
            symbol: "BTC/USDT".to_string(),
            side: Side::Sell,
            order_type: OrderType::Limit(42100.0),
            quantity: 0.3,
            timestamp: 1001,
        },
        Order {
            id: 3,
            symbol: "ETH/USDT".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit(3000.0),
            quantity: 2.0,
            timestamp: 1002,
        },
        Order {
            id: 4,
            symbol: "BTC/USDT".to_string(),
            side: Side::Sell,
            order_type: OrderType::StopLoss(41000.0),
            quantity: 0.2,
            timestamp: 1003,
        },
    ];

    for order in orders {
        order_tx.send(order).unwrap();
    }

    drop(order_tx); // Close order channel
    drop(result_tx); // Close result channel

    result_handler.join().unwrap();
}
```

## Usage Patterns

### 1. Worker Pool

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let rx = std::sync::Arc::new(std::sync::Mutex::new(rx));

    // Create worker pool
    let mut handles = vec![];

    for id in 0..3 {
        let rx = rx.clone();
        let handle = thread::spawn(move || {
            loop {
                let task = {
                    let rx = rx.lock().unwrap();
                    rx.recv()
                };

                match task {
                    Ok(order) => {
                        println!("Worker {} processing: {}", id, order);
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => break,
                }
            }
        });
        handles.push(handle);
    }

    // Send tasks
    for i in 1..=10 {
        tx.send(format!("Order_{}", i)).unwrap();
    }

    drop(tx);

    for handle in handles {
        handle.join().unwrap();
    }
}
```

### 2. Request-Response

```rust
use std::sync::mpsc;
use std::thread;

struct PriceRequest {
    symbol: String,
    response_tx: mpsc::Sender<f64>,
}

fn main() {
    let (request_tx, request_rx) = mpsc::channel::<PriceRequest>();

    // Price fetching thread
    thread::spawn(move || {
        for req in request_rx {
            let price = match req.symbol.as_str() {
                "BTC/USDT" => 42000.0,
                "ETH/USDT" => 2500.0,
                _ => 0.0,
            };
            req.response_tx.send(price).unwrap();
        }
    });

    // Request prices
    for symbol in ["BTC/USDT", "ETH/USDT", "XRP/USDT"] {
        let (resp_tx, resp_rx) = mpsc::channel();

        request_tx.send(PriceRequest {
            symbol: symbol.to_string(),
            response_tx: resp_tx,
        }).unwrap();

        let price = resp_rx.recv().unwrap();
        println!("{}: ${:.2}", symbol, price);
    }
}
```

### 3. Event Stream

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum MarketEvent {
    PriceUpdate { symbol: String, price: f64 },
    TradeExecuted { symbol: String, quantity: f64, price: f64 },
    OrderBookUpdate { symbol: String, bids: Vec<f64>, asks: Vec<f64> },
}

fn main() {
    let (tx, rx) = mpsc::channel();

    // Market event simulator
    let tx_clone = tx.clone();
    thread::spawn(move || {
        for i in 0..5 {
            tx_clone.send(MarketEvent::PriceUpdate {
                symbol: "BTC/USDT".to_string(),
                price: 42000.0 + (i as f64) * 10.0,
            }).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        tx.send(MarketEvent::TradeExecuted {
            symbol: "BTC/USDT".to_string(),
            quantity: 0.5,
            price: 42005.0,
        }).unwrap();
    });

    // Event handler
    for _ in 0..6 {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => {
                match event {
                    MarketEvent::PriceUpdate { symbol, price } => {
                        println!("[PRICE] {}: ${:.2}", symbol, price);
                    }
                    MarketEvent::TradeExecuted { symbol, quantity, price } => {
                        println!("[TRADE] {}: {} @ ${:.2}", symbol, quantity, price);
                    }
                    MarketEvent::OrderBookUpdate { symbol, .. } => {
                        println!("[BOOK] {} updated", symbol);
                    }
                }
            }
            Err(_) => break,
        }
    }
}
```

## What We Learned

| Concept | Description | Trading Application |
|---------|-------------|-------------------|
| `mpsc::channel()` | Unbounded channel | Order queue without limit |
| `mpsc::sync_channel(n)` | Bounded channel | Load control |
| `tx.clone()` | Multiple senders | Multiple signal sources |
| `rx.recv()` | Blocking receive | Wait for order |
| `rx.try_recv()` | Non-blocking receive | Check without waiting |
| `rx.recv_timeout()` | Receive with timeout | Execution timeout |

## Practical Exercises

### Exercise 1: Signal Aggregator

Create a system where 3 strategies (RSI, MACD, Bollinger) send signals to one aggregator:

```rust
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
struct Signal {
    strategy: String,
    symbol: String,
    action: String, // "BUY", "SELL", "HOLD"
    confidence: f64,
}

fn main() {
    // Your code here
    // 1. Create a channel for signals
    // 2. Spawn 3 strategy threads
    // 3. Aggregate signals and make a decision
}
```

### Exercise 2: Rate Limiter

Implement an order sending rate limiter (maximum 10 orders per second):

```rust
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // Your code here
    // Use sync_channel for limiting
}
```

### Exercise 3: Priority Queue

Modify the system so that stop-loss orders are processed first:

```rust
// Hint: use two channels — one for urgent orders,
// another for regular orders
```

## Homework

1. **Position Monitoring System**: Create a system where multiple threads monitor different instruments and send notifications to a central handler when stop-loss or take-profit levels are reached.

2. **Distributed Indicator Calculator**: Implement a system where one thread loads historical data, several threads calculate different indicators in parallel (RSI, MACD, SMA), and results are collected in one thread for analysis.

3. **Exchange Simulator**: Create a simplified exchange simulator with:
   - Channel for incoming orders
   - Matching engine that matches orders
   - Channel for execution notifications
   - Multiple "traders" sending orders

4. **Graceful Shutdown**: Modify the order execution system to gracefully terminate when receiving a shutdown signal, processing all orders in the queue first.

## Navigation

[← Previous day](../155-threads-price-monitor/en.md) | [Next day →](../157-channels-crossbeam/en.md)
