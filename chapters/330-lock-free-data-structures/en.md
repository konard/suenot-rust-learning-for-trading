# Day 330: Lock-Free Data Structures

## Trading Analogy

Imagine an order book in a high-frequency trading system. Thousands of orders arrive every second, and each one must be processed as quickly as possible.

In the traditional approach with locks (mutexes), everything looks like this:
- Trader A wants to add an order — takes the order book lock
- Trader B wants to cancel an order — waits for A to release the lock
- Trader C wants to read the best price — also waits in queue

It's like having only one cashier at an exchange, with everyone lining up!

**Lock-free data structures** are like a modern electronic order book:
- Multiple traders can simultaneously view current prices
- New orders are added atomically, without stopping the entire system
- Cancellations are processed in parallel with new orders
- Nobody waits — everyone makes progress

In the lock-free approach, threads don't block each other. Even if one thread "hangs", the others continue working. This is critical for trading systems where every microsecond of delay costs money.

## Core Concepts

### Atomic Operations

Atomic operations are indivisible operations that either complete entirely or don't happen at all:

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// Atomic trade counter
struct TradeCounter {
    count: AtomicU64,
    total_volume: AtomicU64,
}

impl TradeCounter {
    fn new() -> Self {
        TradeCounter {
            count: AtomicU64::new(0),
            total_volume: AtomicU64::new(0),
        }
    }

    /// Add a trade (thread-safe, lock-free)
    fn add_trade(&self, volume: u64) {
        // fetch_add — atomic "read and increment" operation
        self.count.fetch_add(1, Ordering::SeqCst);
        self.total_volume.fetch_add(volume, Ordering::SeqCst);
    }

    fn get_stats(&self) -> (u64, u64) {
        (
            self.count.load(Ordering::SeqCst),
            self.total_volume.load(Ordering::SeqCst),
        )
    }
}

fn main() {
    let counter = Arc::new(TradeCounter::new());
    let mut handles = vec![];

    // 4 threads simulate trading
    for trader_id in 0..4 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for i in 0..1000 {
                let volume = (trader_id * 100 + i) as u64;
                counter.add_trade(volume);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let (count, volume) = counter.get_stats();
    println!("Total trades: {}", count);
    println!("Total volume: {}", volume);
}
```

### Compare-and-Swap (CAS)

CAS is a fundamental operation for lock-free algorithms:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

/// Lock-free best price update
struct BestPrice {
    // Store price as integer (in cents/satoshis)
    price: AtomicU64,
}

impl BestPrice {
    fn new(initial: u64) -> Self {
        BestPrice {
            price: AtomicU64::new(initial),
        }
    }

    /// Update price if new one is better (lower for ask, higher for bid)
    /// Returns true if update was successful
    fn update_if_better_ask(&self, new_price: u64) -> bool {
        loop {
            let current = self.price.load(Ordering::SeqCst);

            // New price is not better than current
            if new_price >= current {
                return false;
            }

            // Try to atomically replace
            // compare_exchange returns Ok if current hasn't changed
            match self.price.compare_exchange(
                current,
                new_price,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(_) => {
                    // Someone else changed the price, try again
                    continue;
                }
            }
        }
    }

    fn get(&self) -> u64 {
        self.price.load(Ordering::SeqCst)
    }
}

fn main() {
    let best_ask = Arc::new(BestPrice::new(u64::MAX));
    let mut handles = vec![];

    // Multiple threads try to update the best price
    for i in 0..10 {
        let best_ask = Arc::clone(&best_ask);
        let handle = thread::spawn(move || {
            // Simulate different prices from different market makers
            let prices = [42500, 42490, 42510, 42480, 42495];
            for &price in &prices {
                let adjusted_price = price + (i * 5); // Different sources
                if best_ask.update_if_better_ask(adjusted_price) {
                    println!("Thread {} updated price to {}", i, adjusted_price);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nBest ask: {}", best_ask.get());
}
```

## Lock-Free Order Queue

Let's implement a simple lock-free queue for trading orders:

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

struct Node {
    order: Option<Order>,
    next: AtomicPtr<Node>,
}

impl Node {
    fn new(order: Option<Order>) -> *mut Node {
        Box::into_raw(Box::new(Node {
            order,
            next: AtomicPtr::new(ptr::null_mut()),
        }))
    }
}

/// Lock-free queue (simplified Michael-Scott queue implementation)
pub struct LockFreeOrderQueue {
    head: AtomicPtr<Node>,
    tail: AtomicPtr<Node>,
    len: AtomicUsize,
}

impl LockFreeOrderQueue {
    pub fn new() -> Self {
        // Create empty dummy node
        let dummy = Node::new(None);
        LockFreeOrderQueue {
            head: AtomicPtr::new(dummy),
            tail: AtomicPtr::new(dummy),
            len: AtomicUsize::new(0),
        }
    }

    /// Add order to queue (lock-free)
    pub fn enqueue(&self, order: Order) {
        let new_node = Node::new(Some(order));

        loop {
            let tail = self.tail.load(Ordering::SeqCst);
            let tail_ref = unsafe { &*tail };
            let next = tail_ref.next.load(Ordering::SeqCst);

            if next.is_null() {
                // Try to add new node
                if tail_ref
                    .next
                    .compare_exchange(
                        ptr::null_mut(),
                        new_node,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    )
                    .is_ok()
                {
                    // Update tail
                    let _ = self.tail.compare_exchange(
                        tail,
                        new_node,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                    self.len.fetch_add(1, Ordering::SeqCst);
                    return;
                }
            } else {
                // Help update tail
                let _ = self.tail.compare_exchange(
                    tail,
                    next,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            }
        }
    }

    /// Remove order from queue (lock-free)
    pub fn dequeue(&self) -> Option<Order> {
        loop {
            let head = self.head.load(Ordering::SeqCst);
            let tail = self.tail.load(Ordering::SeqCst);
            let head_ref = unsafe { &*head };
            let next = head_ref.next.load(Ordering::SeqCst);

            if head == tail {
                if next.is_null() {
                    return None; // Queue is empty
                }
                // Tail is lagging, help update
                let _ = self.tail.compare_exchange(
                    tail,
                    next,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            } else {
                if next.is_null() {
                    continue;
                }
                let next_ref = unsafe { &*next };
                let order = next_ref.order.clone();

                if self
                    .head
                    .compare_exchange(head, next, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    self.len.fetch_sub(1, Ordering::SeqCst);
                    // Free old head
                    unsafe {
                        drop(Box::from_raw(head));
                    }
                    return order;
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// For simplified example, add Clone for Order
impl Clone for Order {
    fn clone(&self) -> Self {
        Order {
            id: self.id,
            symbol: self.symbol.clone(),
            price: self.price,
            quantity: self.quantity,
        }
    }
}

fn main() {
    let queue = Arc::new(LockFreeOrderQueue::new());
    let mut handles = vec![];

    // Producers — add orders
    for producer_id in 0..3 {
        let queue = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let order = Order {
                    id: producer_id * 1000 + i,
                    symbol: "BTCUSDT".to_string(),
                    price: 42500.0 + i as f64,
                    quantity: 0.1,
                };
                queue.enqueue(order);
            }
            println!("Producer {} added 100 orders", producer_id);
        });
        handles.push(handle);
    }

    // Consumers — process orders
    for consumer_id in 0..2 {
        let queue = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            let mut processed = 0;
            loop {
                match queue.dequeue() {
                    Some(_order) => {
                        processed += 1;
                    }
                    None => {
                        if processed > 0 {
                            break;
                        }
                        thread::yield_now();
                    }
                }
            }
            println!("Consumer {} processed {} orders", consumer_id, processed);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Remaining in queue: {}", queue.len());
}
```

## Lock-Free Price Stack

A stack is useful for storing price history or rollbacks:

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;
use std::sync::Arc;
use std::thread;

#[derive(Clone, Debug)]
struct PricePoint {
    timestamp: u64,
    price: f64,
}

struct StackNode {
    data: PricePoint,
    next: *mut StackNode,
}

/// Lock-free stack (Treiber stack)
pub struct LockFreePriceStack {
    head: AtomicPtr<StackNode>,
    len: AtomicUsize,
}

impl LockFreePriceStack {
    pub fn new() -> Self {
        LockFreePriceStack {
            head: AtomicPtr::new(ptr::null_mut()),
            len: AtomicUsize::new(0),
        }
    }

    /// Push price to stack (lock-free)
    pub fn push(&self, price_point: PricePoint) {
        let new_node = Box::into_raw(Box::new(StackNode {
            data: price_point,
            next: ptr::null_mut(),
        }));

        loop {
            let head = self.head.load(Ordering::SeqCst);
            unsafe {
                (*new_node).next = head;
            }

            if self
                .head
                .compare_exchange(head, new_node, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                self.len.fetch_add(1, Ordering::SeqCst);
                return;
            }
        }
    }

    /// Pop last price from stack (lock-free)
    pub fn pop(&self) -> Option<PricePoint> {
        loop {
            let head = self.head.load(Ordering::SeqCst);
            if head.is_null() {
                return None;
            }

            let next = unsafe { (*head).next };

            if self
                .head
                .compare_exchange(head, next, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                self.len.fetch_sub(1, Ordering::SeqCst);
                let data = unsafe {
                    let node = Box::from_raw(head);
                    node.data.clone()
                };
                return Some(data);
            }
        }
    }

    /// Peek at last price without removing
    pub fn peek(&self) -> Option<PricePoint> {
        let head = self.head.load(Ordering::SeqCst);
        if head.is_null() {
            None
        } else {
            unsafe { Some((*head).data.clone()) }
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }
}

fn main() {
    let stack = Arc::new(LockFreePriceStack::new());
    let mut handles = vec![];

    // Multiple threads write prices
    for thread_id in 0..4 {
        let stack = Arc::clone(&stack);
        let handle = thread::spawn(move || {
            for i in 0..50 {
                let price_point = PricePoint {
                    timestamp: thread_id * 1000 + i,
                    price: 42000.0 + (thread_id * 100 + i) as f64,
                };
                stack.push(price_point);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Entries in stack: {}", stack.len());

    // Read last 10 prices
    println!("\nLatest prices:");
    for _ in 0..10 {
        if let Some(point) = stack.pop() {
            println!("  ts={}: ${:.2}", point.timestamp, point.price);
        }
    }
}
```

## Atomic Flags for Market State

```rust
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum MarketState {
    PreOpen = 0,
    Open = 1,
    Halt = 2,
    Closed = 3,
}

impl From<u8> for MarketState {
    fn from(v: u8) -> Self {
        match v {
            0 => MarketState::PreOpen,
            1 => MarketState::Open,
            2 => MarketState::Halt,
            3 => MarketState::Closed,
            _ => MarketState::Closed,
        }
    }
}

/// Atomic market state
struct AtomicMarketState {
    state: AtomicU8,
    trading_enabled: AtomicBool,
}

impl AtomicMarketState {
    fn new() -> Self {
        AtomicMarketState {
            state: AtomicU8::new(MarketState::PreOpen as u8),
            trading_enabled: AtomicBool::new(false),
        }
    }

    fn get_state(&self) -> MarketState {
        MarketState::from(self.state.load(Ordering::SeqCst))
    }

    fn set_state(&self, new_state: MarketState) {
        self.state.store(new_state as u8, Ordering::SeqCst);

        // Automatically manage trading flag
        let trading = matches!(new_state, MarketState::Open);
        self.trading_enabled.store(trading, Ordering::SeqCst);
    }

    fn is_trading_enabled(&self) -> bool {
        self.trading_enabled.load(Ordering::SeqCst)
    }

    /// Attempt to halt market (atomically)
    fn try_halt(&self) -> bool {
        let current = MarketState::Open as u8;
        let new = MarketState::Halt as u8;

        match self.state.compare_exchange(
            current,
            new,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => {
                self.trading_enabled.store(false, Ordering::SeqCst);
                true
            }
            Err(_) => false,
        }
    }
}

fn main() {
    let market = Arc::new(AtomicMarketState::new());
    let mut handles = vec![];

    // Market controller
    {
        let market = Arc::clone(&market);
        let handle = thread::spawn(move || {
            println!("[Controller] Market in PreOpen state");
            thread::sleep(Duration::from_millis(100));

            market.set_state(MarketState::Open);
            println!("[Controller] Market is open!");
            thread::sleep(Duration::from_millis(300));

            market.set_state(MarketState::Closed);
            println!("[Controller] Market is closed");
        });
        handles.push(handle);
    }

    // Trading bots check state
    for bot_id in 0..3 {
        let market = Arc::clone(&market);
        let handle = thread::spawn(move || {
            let mut trades = 0;
            for _ in 0..50 {
                if market.is_trading_enabled() {
                    trades += 1;
                }
                thread::sleep(Duration::from_millis(10));
            }
            println!("[Bot {}] Trades executed: {}", bot_id, trades);
        });
        handles.push(handle);
    }

    // Risk monitor can halt trading
    {
        let market = Arc::clone(&market);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            if market.try_halt() {
                println!("[Risk Monitor] Trading halted!");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nFinal state: {:?}", market.get_state());
}
```

## Lock-Free Position Counter

```rust
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

/// Lock-free position tracker
struct PositionTracker {
    // Position in base units (can be negative for shorts)
    position: AtomicI64,
    // Total trade volume (always positive)
    total_volume: AtomicU64,
    // Number of trades
    trade_count: AtomicU64,
}

impl PositionTracker {
    fn new() -> Self {
        PositionTracker {
            position: AtomicI64::new(0),
            total_volume: AtomicU64::new(0),
            trade_count: AtomicU64::new(0),
        }
    }

    /// Buy (increase position)
    fn buy(&self, quantity: i64) {
        self.position.fetch_add(quantity, Ordering::SeqCst);
        self.total_volume.fetch_add(quantity.unsigned_abs(), Ordering::SeqCst);
        self.trade_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Sell (decrease position)
    fn sell(&self, quantity: i64) {
        self.position.fetch_sub(quantity, Ordering::SeqCst);
        self.total_volume.fetch_add(quantity.unsigned_abs(), Ordering::SeqCst);
        self.trade_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Get current position
    fn get_position(&self) -> i64 {
        self.position.load(Ordering::SeqCst)
    }

    /// Get statistics
    fn get_stats(&self) -> (i64, u64, u64) {
        (
            self.position.load(Ordering::SeqCst),
            self.total_volume.load(Ordering::SeqCst),
            self.trade_count.load(Ordering::SeqCst),
        )
    }

    /// Atomically check and update position
    /// Returns true if update was successful
    fn try_update_position(&self, expected: i64, new: i64) -> bool {
        self.position
            .compare_exchange(expected, new, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
}

fn main() {
    let tracker = Arc::new(PositionTracker::new());
    let mut handles = vec![];

    // Buyers
    for _ in 0..3 {
        let tracker = Arc::clone(&tracker);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                tracker.buy(10);
            }
        });
        handles.push(handle);
    }

    // Sellers
    for _ in 0..2 {
        let tracker = Arc::clone(&tracker);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                tracker.sell(10);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let (position, volume, trades) = tracker.get_stats();
    println!("=== Results ===");
    println!("Final position: {} units", position);
    println!("Total volume: {} units", volume);
    println!("Trade count: {}", trades);

    // Verification: 3 buyers * 100 * 10 - 2 sellers * 100 * 10 = 1000
    println!("\nExpected position: {}", (3 - 2) * 100 * 10);
}
```

## SeqLock for Ticker Data

SeqLock allows readers to work without locks:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug)]
struct TickerData {
    bid: f64,
    ask: f64,
    last: f64,
    volume: f64,
}

/// SeqLock for ticker data
/// One writer, many readers, lock-free for readers
struct SeqLockTicker {
    sequence: AtomicU64,
    data: std::cell::UnsafeCell<TickerData>,
}

unsafe impl Sync for SeqLockTicker {}

impl SeqLockTicker {
    fn new(data: TickerData) -> Self {
        SeqLockTicker {
            sequence: AtomicU64::new(0),
            data: std::cell::UnsafeCell::new(data),
        }
    }

    /// Write new data (single writer only!)
    fn write(&self, new_data: TickerData) {
        // Increment sequence by 1 (odd = writing)
        self.sequence.fetch_add(1, Ordering::SeqCst);

        // Write data
        unsafe {
            *self.data.get() = new_data;
        }

        // Increment sequence by 1 (even = data stable)
        self.sequence.fetch_add(1, Ordering::SeqCst);
    }

    /// Read data (lock-free for readers)
    fn read(&self) -> TickerData {
        loop {
            let seq1 = self.sequence.load(Ordering::SeqCst);

            // If sequence is odd, writer is active — wait
            if seq1 % 2 != 0 {
                std::hint::spin_loop();
                continue;
            }

            // Read data
            let data = unsafe { (*self.data.get()).clone() };

            // Check that sequence hasn't changed
            let seq2 = self.sequence.load(Ordering::SeqCst);

            if seq1 == seq2 {
                return data;
            }
            // If changed — retry read
        }
    }
}

fn main() {
    let ticker = Arc::new(SeqLockTicker::new(TickerData {
        bid: 42500.0,
        ask: 42501.0,
        last: 42500.5,
        volume: 0.0,
    }));

    let mut handles = vec![];

    // Writer — updates data
    {
        let ticker = Arc::clone(&ticker);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let data = TickerData {
                    bid: 42500.0 + i as f64,
                    ask: 42501.0 + i as f64,
                    last: 42500.5 + i as f64,
                    volume: i as f64 * 10.0,
                };
                ticker.write(data);
                thread::sleep(Duration::from_micros(100));
            }
        });
        handles.push(handle);
    }

    // Readers — read data without locks
    for reader_id in 0..4 {
        let ticker = Arc::clone(&ticker);
        let handle = thread::spawn(move || {
            let mut reads = 0;
            for _ in 0..500 {
                let data = ticker.read();
                reads += 1;
                if reads % 100 == 0 {
                    println!(
                        "[Reader {}] bid={:.2} ask={:.2} volume={:.1}",
                        reader_id, data.bid, data.ask, data.volume
                    );
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_data = ticker.read();
    println!("\nFinal data: {:?}", final_data);
}
```

## What We Learned

| Concept | Description | Trading Application |
|---------|-------------|---------------------|
| **Atomic Operations** | Indivisible operations with guarantees | Trade counters, volumes |
| **Compare-and-Swap** | Conditional atomic update | Best price updates |
| **Lock-free Queue** | Queue without locks | Order queue |
| **Lock-free Stack** | Stack without locks | Price history, rollbacks |
| **Atomic Flags** | Boolean flags for states | Market state |
| **SeqLock** | Optimization for frequent reads | Ticker data |

## Practical Exercises

1. **Lock-free Order Book**: Implement a simplified order book with lock-free order addition and cancellation.

2. **Atomic Price Aggregator**: Create a structure that atomically tracks min, max, avg price over a period.

3. **Lock-free Ring Buffer**: Implement a ring buffer for storing the last N ticks without locks.

4. **Atomic Rate Limiter**: Create an API rate limiter for exchange requests using atomic operations.

## Homework

1. **Lock-free Order Book Aggregator**: Implement a system that combines order books from multiple exchanges into a single view without using mutexes.

2. **Atomic PnL Tracker**: Create a profit/loss tracker that is updated from multiple threads using only atomic operations.

3. **Performance Comparison**: Write a benchmark comparing lock-free queue performance with `Mutex<VecDeque>` with varying numbers of producers and consumers.

4. **Lock-free Cache with TTL**: Implement a price cache with automatic invalidation using only atomic operations.

## Navigation

[← Previous day](../321-result-caching/en.md) | [Next day →](../331-*/en.md)
