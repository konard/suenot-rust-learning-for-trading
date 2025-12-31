// Test file for Chapter 159: sync_channel examples

use std::sync::mpsc::{sync_channel, TrySendError};
use std::thread;
use std::time::Duration;

fn test_basic_sync_channel() {
    println!("=== Test 1: Basic sync_channel ===");
    let (sender, receiver) = sync_channel::<f64>(3);

    let producer = thread::spawn(move || {
        let prices = [42000.0, 42100.0, 42050.0];

        for price in prices {
            println!("[Producer] Sending price: {}", price);
            sender.send(price).unwrap();
            println!("[Producer] Price {} sent", price);
        }
    });

    let consumer = thread::spawn(move || {
        while let Ok(price) = receiver.recv() {
            println!("[Consumer] Received price: {}", price);
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
    println!();
}

fn test_rendezvous_channel() {
    println!("=== Test 2: Rendezvous channel (sync_channel(0)) ===");
    let (sender, receiver) = sync_channel::<(String, f64)>(0);

    let order_executor = thread::spawn(move || {
        while let Ok((symbol, price)) = receiver.recv() {
            println!("[Executor] Executing order: {} @ {:.2}", symbol, price);
        }
    });

    let order_sender = thread::spawn(move || {
        let orders = [
            ("BTC".to_string(), 42000.0),
            ("ETH".to_string(), 2200.0),
        ];

        for (symbol, price) in orders {
            println!("[Sender] Sending order: {} @ {:.2}", symbol, price);
            sender.send((symbol.clone(), price)).unwrap();
            println!("[Sender] Order {} accepted", symbol);
        }
    });

    order_sender.join().unwrap();
    order_executor.join().unwrap();
    println!();
}

fn test_try_send() {
    println!("=== Test 3: try_send non-blocking ===");

    #[derive(Debug)]
    struct MarketTick {
        symbol: String,
        price: f64,
        timestamp: u64,
    }

    let (tx, rx) = sync_channel::<MarketTick>(3);

    let producer = thread::spawn(move || {
        let mut timestamp = 0u64;

        for i in 0..10 {
            let tick = MarketTick {
                symbol: "BTCUSDT".to_string(),
                price: 42000.0 + (i as f64 * 5.0),
                timestamp,
            };
            timestamp += 1;

            match tx.try_send(tick) {
                Ok(()) => println!("[Feed] Tick {} sent", i),
                Err(TrySendError::Full(tick)) => {
                    println!("[Feed] Buffer full, tick {} dropped (price: {})", i, tick.price);
                }
                Err(TrySendError::Disconnected(_)) => {
                    println!("[Feed] Channel closed");
                    break;
                }
            }

            thread::sleep(Duration::from_millis(5));
        }
    });

    let consumer = thread::spawn(move || {
        let mut count = 0;
        while let Ok(tick) = rx.recv() {
            println!(
                "[Strategy] Processing tick: {} @ {:.2} (ts: {})",
                tick.symbol, tick.price, tick.timestamp
            );
            count += 1;
            thread::sleep(Duration::from_millis(20));
        }
        println!("[Strategy] Processed {} ticks", count);
    });

    producer.join().unwrap();
    // tx is already moved into producer, no need to drop
    consumer.join().unwrap();
    println!();
}

fn test_order_struct() {
    println!("=== Test 4: Order struct with sync_channel ===");

    #[derive(Debug, Clone)]
    struct Order {
        id: u64,
        symbol: String,
        side: String,
        price: f64,
        quantity: f64,
    }

    let (order_tx, order_rx) = sync_channel::<Order>(2);

    let processor = thread::spawn(move || {
        while let Ok(order) = order_rx.recv() {
            println!("[Processor] Processing order #{}: {} {} @ {:.2}",
                order.id, order.side, order.symbol, order.price);
        }
    });

    let generator = thread::spawn(move || {
        for i in 0..5 {
            let order = Order {
                id: i,
                symbol: "BTCUSDT".to_string(),
                side: if i % 2 == 0 { "BUY".to_string() } else { "SELL".to_string() },
                price: 42000.0 + (i as f64 * 10.0),
                quantity: 0.1,
            };
            println!("[Generator] Sending order #{}...", i);
            order_tx.send(order).unwrap();
        }
    });

    generator.join().unwrap();
    // order_tx is already moved into generator, no need to drop
    processor.join().unwrap();
    println!();
}

fn test_buffer_size_choice() {
    println!("=== Test 5: Buffer size selection ===");

    // 0 — Rendezvous
    let (_tx, _rx) = sync_channel::<i32>(0);
    println!("sync_channel(0) - Rendezvous channel created");

    // 1-10 — Small buffer
    let (_tx, _rx) = sync_channel::<i32>(5);
    println!("sync_channel(5) - Small buffer created");

    // 10-100 — Medium buffer
    let (_tx, _rx) = sync_channel::<i32>(50);
    println!("sync_channel(50) - Medium buffer created");

    // 100+ — Large buffer
    let (_tx, _rx) = sync_channel::<i32>(1000);
    println!("sync_channel(1000) - Large buffer created");

    println!("Buffer size is chosen based on latency and throughput requirements\n");
}

fn main() {
    println!("Chapter 159: sync_channel - Bounded Queue\n");
    println!("Running code examples...\n");

    test_basic_sync_channel();
    test_rendezvous_channel();
    test_try_send();
    test_order_struct();
    test_buffer_size_choice();

    println!("All tests completed successfully!");
}
