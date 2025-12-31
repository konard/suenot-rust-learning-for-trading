# Day 334: Project — High-Performance Matcher

## Introduction

This is a **project chapter** that combines the month's optimization knowledge into a practical mini-project. We'll create a **high-performance order matcher** — the heart of any trading exchange, applying optimization techniques learned this month.

## Trading Analogy

Imagine a central stock exchange floor during peak hours. Thousands of traders are shouting their orders simultaneously:

```
"Buying 100 BTC at $50,000!"
"Selling 50 BTC at $50,000!"
"Buying 200 ETH at $3,000!"
...
```

The **Matcher** is the exchange's "super-listener" that must:
1. Instantly hear every order
2. Find compatible orders (buyer willing to pay >= seller's price)
3. Execute the trade in microseconds
4. Process millions of such operations per second

| Characteristic | Requirement |
|----------------|-------------|
| Latency | < 1 microsecond |
| Throughput | > 1M orders/sec |
| Determinism | Predictable timing |
| Allocations | Minimal or zero |
| Order | FIFO preservation |

## Matcher Architecture

```
+---------------------------------------------------------+
|                    MATCHING ENGINE                       |
+---------------------------------------------------------+
|  +-------------+    +-------------+    +-------------+   |
|  |  PARSER     |--->|  VALIDATOR  |--->|  SEQUENCER  |   |
|  | Zero-copy   |    | Pre-alloc   |    | Lock-free   |   |
|  +-------------+    +-------------+    +-------------+   |
|          |                                    |          |
|          v                                    v          |
|  +--------------------------------------------------+   |
|  |              ORDER BOOK (per symbol)              |   |
|  |  +------------+         +------------+            |   |
|  |  |   BIDS     |         |   ASKS     |            |   |
|  |  | BTreeMap   |<------->| BTreeMap   |            |   |
|  |  | max-heap   | MATCH!  | min-heap   |            |   |
|  |  +------------+         +------------+            |   |
|  +--------------------------------------------------+   |
|          |                                    |          |
|          v                                    v          |
|  +-------------+    +-------------+    +-------------+   |
|  |  EXECUTOR   |--->|  PUBLISHER  |--->|  PERSISTER  |   |
|  | Arena alloc |    | Ring buffer |    | Async batch |   |
|  +-------------+    +-------------+    +-------------+   |
+---------------------------------------------------------+
```

## Part 1: Base Structures with Minimal Allocations

```rust
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    Bid = 0,  // Buy
    Ask = 1,  // Sell
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OrderType {
    Limit = 0,
    Market = 1,
    IoC = 2,    // Immediate or Cancel
    FoK = 3,    // Fill or Kill
}

/// Order — optimized for cache line placement (64 bytes)
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Order {
    pub id: u64,            // 8 bytes
    pub price: u64,         // 8 bytes (in "ticks" for precision)
    pub quantity: u64,      // 8 bytes
    pub filled: u64,        // 8 bytes
    pub timestamp: u64,     // 8 bytes
    pub side: Side,         // 1 byte
    pub order_type: OrderType, // 1 byte
    pub symbol_id: u16,     // 2 bytes
    _padding: [u8; 4],      // 4 bytes alignment
}                           // Total: 48 bytes

impl Order {
    #[inline]
    pub fn new(
        id: u64,
        price: u64,
        quantity: u64,
        side: Side,
        order_type: OrderType,
        symbol_id: u16,
        timestamp: u64,
    ) -> Self {
        Order {
            id,
            price,
            quantity,
            filled: 0,
            timestamp,
            side,
            order_type,
            symbol_id,
            _padding: [0; 4],
        }
    }

    #[inline]
    pub fn remaining(&self) -> u64 {
        self.quantity - self.filled
    }

    #[inline]
    pub fn is_filled(&self) -> bool {
        self.filled >= self.quantity
    }
}

/// Execution result — no allocations
#[derive(Debug, Clone, Copy)]
pub struct Fill {
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: u64,
}

fn main() {
    println!("Size of Order: {} bytes", std::mem::size_of::<Order>());
    println!("Size of Fill: {} bytes", std::mem::size_of::<Fill>());

    let order = Order::new(
        1,
        50000_00, // $50,000.00 in cents
        100,
        Side::Bid,
        OrderType::Limit,
        1, // BTC/USDT
        1234567890,
    );

    println!("Order: {:?}", order);
    println!("Remaining: {}", order.remaining());
}
```

## Part 2: Price Level with Memory Pool

```rust
use std::collections::VecDeque;

/// Pre-allocated order pool to avoid allocations
pub struct OrderPool {
    orders: Vec<Order>,
    free_indices: Vec<usize>,
}

impl OrderPool {
    pub fn with_capacity(capacity: usize) -> Self {
        OrderPool {
            orders: Vec::with_capacity(capacity),
            free_indices: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn allocate(&mut self, order: Order) -> usize {
        if let Some(idx) = self.free_indices.pop() {
            self.orders[idx] = order;
            idx
        } else {
            let idx = self.orders.len();
            self.orders.push(order);
            idx
        }
    }

    #[inline]
    pub fn deallocate(&mut self, idx: usize) {
        self.free_indices.push(idx);
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<&Order> {
        self.orders.get(idx)
    }

    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Order> {
        self.orders.get_mut(idx)
    }
}

/// Price level — stores indices in the pool
#[derive(Debug)]
pub struct PriceLevel {
    pub price: u64,
    pub total_quantity: u64,
    order_indices: VecDeque<usize>,
}

impl PriceLevel {
    #[inline]
    pub fn new(price: u64) -> Self {
        PriceLevel {
            price,
            total_quantity: 0,
            order_indices: VecDeque::with_capacity(64),
        }
    }

    #[inline]
    pub fn add_order(&mut self, idx: usize, quantity: u64) {
        self.order_indices.push_back(idx);
        self.total_quantity += quantity;
    }

    #[inline]
    pub fn front(&self) -> Option<usize> {
        self.order_indices.front().copied()
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<usize> {
        self.order_indices.pop_front()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.order_indices.is_empty()
    }

    #[inline]
    pub fn order_count(&self) -> usize {
        self.order_indices.len()
    }
}

fn main() {
    let mut pool = OrderPool::with_capacity(1000);

    // Add orders
    let order1 = Order::new(1, 50000_00, 100, Side::Bid, OrderType::Limit, 1, 1000);
    let order2 = Order::new(2, 50000_00, 200, Side::Bid, OrderType::Limit, 1, 1001);

    let idx1 = pool.allocate(order1);
    let idx2 = pool.allocate(order2);

    let mut level = PriceLevel::new(50000_00);
    level.add_order(idx1, 100);
    level.add_order(idx2, 200);

    println!("Price level: {} cents", level.price);
    println!("Total quantity: {}", level.total_quantity);
    println!("Order count: {}", level.order_count());
}
```

## Part 3: High-Performance Order Book

```rust
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::cmp::Reverse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OrderType {
    Limit = 0,
    Market = 1,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Order {
    pub id: u64,
    pub price: u64,
    pub quantity: u64,
    pub filled: u64,
    pub timestamp: u64,
    pub side: Side,
    pub order_type: OrderType,
    pub symbol_id: u16,
    _padding: [u8; 4],
}

impl Order {
    pub fn new(id: u64, price: u64, quantity: u64, side: Side,
               order_type: OrderType, symbol_id: u16, timestamp: u64) -> Self {
        Order { id, price, quantity, filled: 0, timestamp, side, order_type, symbol_id, _padding: [0; 4] }
    }

    #[inline]
    pub fn remaining(&self) -> u64 { self.quantity - self.filled }
}

#[derive(Debug, Clone, Copy)]
pub struct Fill {
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: u64,
}

pub struct PriceLevel {
    pub price: u64,
    pub total_quantity: u64,
    orders: VecDeque<Order>,
}

impl PriceLevel {
    pub fn new(price: u64) -> Self {
        PriceLevel { price, total_quantity: 0, orders: VecDeque::with_capacity(32) }
    }

    #[inline]
    pub fn add_order(&mut self, order: Order) {
        self.total_quantity += order.remaining();
        self.orders.push_back(order);
    }

    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut Order> { self.orders.front_mut() }

    #[inline]
    pub fn pop_front(&mut self) -> Option<Order> { self.orders.pop_front() }

    #[inline]
    pub fn is_empty(&self) -> bool { self.orders.is_empty() }
}

/// High-performance order book
pub struct OrderBook {
    pub symbol_id: u16,
    /// Bids: use Reverse for descending order
    bids: BTreeMap<Reverse<u64>, PriceLevel>,
    /// Asks: ascending order (default)
    asks: BTreeMap<u64, PriceLevel>,
    /// Order counter
    order_count: u64,
    /// Buffer for fills (avoid allocations)
    fills_buffer: Vec<Fill>,
}

impl OrderBook {
    pub fn new(symbol_id: u16) -> Self {
        OrderBook {
            symbol_id,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_count: 0,
            fills_buffer: Vec::with_capacity(1024),
        }
    }

    /// Best bid
    #[inline]
    pub fn best_bid(&self) -> Option<u64> {
        self.bids.keys().next().map(|r| r.0)
    }

    /// Best ask
    #[inline]
    pub fn best_ask(&self) -> Option<u64> {
        self.asks.keys().next().copied()
    }

    /// Spread
    #[inline]
    pub fn spread(&self) -> Option<u64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) if ask > bid => Some(ask - bid),
            _ => None,
        }
    }

    /// Add limit order with automatic matching
    pub fn add_limit_order(&mut self, mut order: Order) -> Vec<Fill> {
        self.fills_buffer.clear();
        let timestamp = order.timestamp;

        match order.side {
            Side::Bid => {
                // Try to match with asks
                while order.remaining() > 0 {
                    let best_ask = match self.asks.first_entry() {
                        Some(entry) if *entry.key() <= order.price => entry,
                        _ => break,
                    };

                    let ask_price = *best_ask.key();
                    let level = best_ask.into_mut();

                    while order.remaining() > 0 && !level.is_empty() {
                        let maker = level.front_mut().unwrap();
                        let fill_qty = order.remaining().min(maker.remaining());
                        let maker_id = maker.id;
                        let maker_price = maker.price;

                        maker.filled += fill_qty;
                        order.filled += fill_qty;

                        let maker_done = maker.remaining() == 0;

                        let fill = Fill {
                            maker_order_id: maker_id,
                            taker_order_id: order.id,
                            price: maker_price,
                            quantity: fill_qty,
                            timestamp,
                        };
                        self.fills_buffer.push(fill);

                        level.total_quantity -= fill_qty;

                        if maker_done {
                            level.pop_front();
                        }
                    }

                    if level.is_empty() {
                        self.asks.remove(&ask_price);
                    }
                }

                // Add remainder to book
                if order.remaining() > 0 {
                    self.bids
                        .entry(Reverse(order.price))
                        .or_insert_with(|| PriceLevel::new(order.price))
                        .add_order(order);
                    self.order_count += 1;
                }
            }
            Side::Ask => {
                // Try to match with bids
                while order.remaining() > 0 {
                    let best_bid = match self.bids.first_entry() {
                        Some(entry) if entry.key().0 >= order.price => entry,
                        _ => break,
                    };

                    let bid_price = best_bid.key().0;
                    let level = best_bid.into_mut();

                    while order.remaining() > 0 && !level.is_empty() {
                        let maker = level.front_mut().unwrap();
                        let fill_qty = order.remaining().min(maker.remaining());
                        let maker_id = maker.id;
                        let maker_price = maker.price;

                        maker.filled += fill_qty;
                        order.filled += fill_qty;

                        let maker_done = maker.remaining() == 0;

                        let fill = Fill {
                            maker_order_id: maker_id,
                            taker_order_id: order.id,
                            price: maker_price,
                            quantity: fill_qty,
                            timestamp,
                        };
                        self.fills_buffer.push(fill);

                        level.total_quantity -= fill_qty;

                        if maker_done {
                            level.pop_front();
                        }
                    }

                    if level.is_empty() {
                        self.bids.remove(&Reverse(bid_price));
                    }
                }

                // Add remainder to book
                if order.remaining() > 0 {
                    self.asks
                        .entry(order.price)
                        .or_insert_with(|| PriceLevel::new(order.price))
                        .add_order(order);
                    self.order_count += 1;
                }
            }
        }

        self.fills_buffer.clone()
    }

    /// Book statistics
    pub fn stats(&self) -> BookStats {
        BookStats {
            bid_levels: self.bids.len(),
            ask_levels: self.asks.len(),
            best_bid: self.best_bid(),
            best_ask: self.best_ask(),
            spread: self.spread(),
            total_orders: self.order_count,
        }
    }
}

#[derive(Debug)]
pub struct BookStats {
    pub bid_levels: usize,
    pub ask_levels: usize,
    pub best_bid: Option<u64>,
    pub best_ask: Option<u64>,
    pub spread: Option<u64>,
    pub total_orders: u64,
}

fn main() {
    let mut book = OrderBook::new(1);
    let mut order_id = 1u64;

    // Add asks (sellers)
    for i in 0..5 {
        let price = 50100 + i * 10; // $501.00, $501.10, ...
        let order = Order::new(order_id, price, 100, Side::Ask, OrderType::Limit, 1, order_id);
        order_id += 1;
        book.add_limit_order(order);
    }

    // Add bids (buyers)
    for i in 0..5 {
        let price = 50000 - i * 10; // $500.00, $499.90, ...
        let order = Order::new(order_id, price, 100, Side::Bid, OrderType::Limit, 1, order_id);
        order_id += 1;
        book.add_limit_order(order);
    }

    println!("=== Order Book Stats ===");
    let stats = book.stats();
    println!("{:?}", stats);

    // Aggressive buy order — should match
    println!("\n=== Matching Buy Order ===");
    let aggressive_buy = Order::new(order_id, 50120, 250, Side::Bid, OrderType::Limit, 1, order_id);
    let fills = book.add_limit_order(aggressive_buy);

    for fill in &fills {
        println!("Fill: {} units @ {} (maker: {}, taker: {})",
            fill.quantity, fill.price, fill.maker_order_id, fill.taker_order_id);
    }

    println!("\n=== Updated Stats ===");
    println!("{:?}", book.stats());
}
```

## Part 4: Lock-free Order Queue

For multi-threaded processing, we use lock-free structures:

```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::cell::UnsafeCell;

/// Lock-free Single Producer Single Consumer ring buffer
pub struct SpscQueue<T> {
    buffer: Vec<UnsafeCell<Option<T>>>,
    capacity: usize,
    head: AtomicUsize,  // Read index
    tail: AtomicUsize,  // Write index
}

// Safe for transfer between threads
unsafe impl<T: Send> Send for SpscQueue<T> {}
unsafe impl<T: Send> Sync for SpscQueue<T> {}

impl<T> SpscQueue<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(UnsafeCell::new(None));
        }

        SpscQueue {
            buffer,
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Try to write (producer)
    pub fn try_push(&self, value: T) -> Result<(), T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) & (self.capacity - 1);

        if next_tail == self.head.load(Ordering::Acquire) {
            return Err(value); // Buffer full
        }

        unsafe {
            *self.buffer[tail].get() = Some(value);
        }

        self.tail.store(next_tail, Ordering::Release);
        Ok(())
    }

    /// Try to read (consumer)
    pub fn try_pop(&self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);

        if head == self.tail.load(Ordering::Acquire) {
            return None; // Buffer empty
        }

        let value = unsafe {
            (*self.buffer[head].get()).take()
        };

        let next_head = (head + 1) & (self.capacity - 1);
        self.head.store(next_head, Ordering::Release);

        value
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }

    pub fn len(&self) -> usize {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Relaxed);
        if tail >= head {
            tail - head
        } else {
            self.capacity - head + tail
        }
    }
}

/// Matcher message
#[derive(Debug, Clone)]
pub enum MatcherMessage {
    NewOrder {
        id: u64,
        price: u64,
        quantity: u64,
        side: u8, // 0 = Bid, 1 = Ask
        symbol_id: u16,
    },
    CancelOrder {
        id: u64,
        symbol_id: u16,
    },
    Shutdown,
}

fn main() {
    let queue: SpscQueue<MatcherMessage> = SpscQueue::with_capacity(1024);

    // Simulate producer
    for i in 0..10 {
        let msg = MatcherMessage::NewOrder {
            id: i,
            price: 50000 + i * 10,
            quantity: 100,
            side: (i % 2) as u8,
            symbol_id: 1,
        };

        match queue.try_push(msg) {
            Ok(()) => println!("Pushed order {}", i),
            Err(_) => println!("Queue full!"),
        }
    }

    println!("\nQueue length: {}", queue.len());

    // Simulate consumer
    while let Some(msg) = queue.try_pop() {
        println!("Received: {:?}", msg);
    }

    println!("Queue empty: {}", queue.is_empty());
}
```

## Part 5: Complete Matcher with Benchmarking

```rust
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::cmp::Reverse;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side { Bid, Ask }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType { Limit, Market }

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub price: u64,
    pub quantity: u64,
    pub filled: u64,
    pub timestamp: u64,
    pub side: Side,
}

impl Order {
    pub fn new(id: u64, price: u64, quantity: u64, side: Side, timestamp: u64) -> Self {
        Order { id, price, quantity, filled: 0, timestamp, side }
    }

    #[inline]
    pub fn remaining(&self) -> u64 { self.quantity - self.filled }
}

#[derive(Debug, Clone, Copy)]
pub struct Fill {
    pub maker_id: u64,
    pub taker_id: u64,
    pub price: u64,
    pub quantity: u64,
}

pub struct PriceLevel {
    pub price: u64,
    pub total_qty: u64,
    orders: VecDeque<Order>,
}

impl PriceLevel {
    pub fn new(price: u64) -> Self {
        PriceLevel { price, total_qty: 0, orders: VecDeque::with_capacity(64) }
    }

    pub fn add(&mut self, order: Order) {
        self.total_qty += order.remaining();
        self.orders.push_back(order);
    }

    pub fn front_mut(&mut self) -> Option<&mut Order> { self.orders.front_mut() }
    pub fn pop_front(&mut self) -> Option<Order> { self.orders.pop_front() }
    pub fn is_empty(&self) -> bool { self.orders.is_empty() }
}

/// High-performance matcher
pub struct Matcher {
    bids: BTreeMap<Reverse<u64>, PriceLevel>,
    asks: BTreeMap<u64, PriceLevel>,
    fills: Vec<Fill>,

    // Statistics
    orders_processed: u64,
    total_fills: u64,
    total_volume: u64,
}

impl Matcher {
    pub fn new() -> Self {
        Matcher {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            fills: Vec::with_capacity(1024),
            orders_processed: 0,
            total_fills: 0,
            total_volume: 0,
        }
    }

    /// Process new order
    #[inline]
    pub fn process_order(&mut self, mut order: Order) -> &[Fill] {
        self.fills.clear();
        self.orders_processed += 1;

        match order.side {
            Side::Bid => self.match_bid(&mut order),
            Side::Ask => self.match_ask(&mut order),
        }

        &self.fills
    }

    #[inline]
    fn match_bid(&mut self, order: &mut Order) {
        // Match with asks
        while order.remaining() > 0 {
            let Some(mut entry) = self.asks.first_entry() else { break };
            if *entry.key() > order.price { break; }

            let ask_price = *entry.key();
            let level = entry.get_mut();

            while order.remaining() > 0 && !level.is_empty() {
                let maker = level.front_mut().unwrap();
                let fill_qty = order.remaining().min(maker.remaining());
                let maker_id = maker.id;
                let maker_price = maker.price;

                maker.filled += fill_qty;
                order.filled += fill_qty;

                let maker_done = maker.remaining() == 0;

                self.fills.push(Fill {
                    maker_id,
                    taker_id: order.id,
                    price: maker_price,
                    quantity: fill_qty,
                });

                level.total_qty -= fill_qty;
                self.total_fills += 1;
                self.total_volume += fill_qty;

                if maker_done {
                    level.pop_front();
                }
            }

            if level.is_empty() {
                self.asks.remove(&ask_price);
            }
        }

        // Add remainder
        if order.remaining() > 0 {
            self.bids
                .entry(Reverse(order.price))
                .or_insert_with(|| PriceLevel::new(order.price))
                .add(order.clone());
        }
    }

    #[inline]
    fn match_ask(&mut self, order: &mut Order) {
        // Match with bids
        while order.remaining() > 0 {
            let Some(mut entry) = self.bids.first_entry() else { break };
            if entry.key().0 < order.price { break; }

            let bid_price = entry.key().0;
            let level = entry.get_mut();

            while order.remaining() > 0 && !level.is_empty() {
                let maker = level.front_mut().unwrap();
                let fill_qty = order.remaining().min(maker.remaining());
                let maker_id = maker.id;
                let maker_price = maker.price;

                maker.filled += fill_qty;
                order.filled += fill_qty;

                let maker_done = maker.remaining() == 0;

                self.fills.push(Fill {
                    maker_id,
                    taker_id: order.id,
                    price: maker_price,
                    quantity: fill_qty,
                });

                level.total_qty -= fill_qty;
                self.total_fills += 1;
                self.total_volume += fill_qty;

                if maker_done {
                    level.pop_front();
                }
            }

            if level.is_empty() {
                self.bids.remove(&Reverse(bid_price));
            }
        }

        // Add remainder
        if order.remaining() > 0 {
            self.asks
                .entry(order.price)
                .or_insert_with(|| PriceLevel::new(order.price))
                .add(order.clone());
        }
    }

    pub fn best_bid(&self) -> Option<u64> {
        self.bids.keys().next().map(|r| r.0)
    }

    pub fn best_ask(&self) -> Option<u64> {
        self.asks.keys().next().copied()
    }

    pub fn stats(&self) -> MatcherStats {
        MatcherStats {
            orders_processed: self.orders_processed,
            total_fills: self.total_fills,
            total_volume: self.total_volume,
            bid_levels: self.bids.len(),
            ask_levels: self.asks.len(),
        }
    }
}

#[derive(Debug)]
pub struct MatcherStats {
    pub orders_processed: u64,
    pub total_fills: u64,
    pub total_volume: u64,
    pub bid_levels: usize,
    pub ask_levels: usize,
}

fn main() {
    let mut matcher = Matcher::new();

    println!("=== High-Performance Matcher Benchmark ===\n");

    // Warmup
    for i in 0..1000 {
        let side = if i % 2 == 0 { Side::Bid } else { Side::Ask };
        let price = 50000 + (i % 100) * 10;
        let order = Order::new(i, price, 100, side, i);
        matcher.process_order(order);
    }

    // Benchmark
    let num_orders = 100_000u64;
    let start = Instant::now();

    for i in 0..num_orders {
        let side = if i % 2 == 0 { Side::Bid } else { Side::Ask };
        // Create orders near mid-price for maximum matching
        let price = 50000 + ((i as i64 % 20) - 10) as u64 * 5;
        let order = Order::new(i + 1000, price, 10 + (i % 50), side, i);
        matcher.process_order(order);
    }

    let elapsed = start.elapsed();
    let orders_per_sec = num_orders as f64 / elapsed.as_secs_f64();
    let latency_ns = elapsed.as_nanos() as f64 / num_orders as f64;

    println!("Processed {} orders in {:?}", num_orders, elapsed);
    println!("Throughput: {:.0} orders/sec", orders_per_sec);
    println!("Average latency: {:.0} ns/order", latency_ns);
    println!("\nMatcher stats: {:?}", matcher.stats());
    println!("Best bid: {:?}, Best ask: {:?}", matcher.best_bid(), matcher.best_ask());
}
```

## Part 6: Real-World Optimizations

### Best Price Caching

```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic best price cache
pub struct BestPriceCache {
    best_bid: AtomicU64,
    best_ask: AtomicU64,
}

impl BestPriceCache {
    pub const NO_PRICE: u64 = u64::MAX;

    pub fn new() -> Self {
        BestPriceCache {
            best_bid: AtomicU64::new(Self::NO_PRICE),
            best_ask: AtomicU64::new(Self::NO_PRICE),
        }
    }

    #[inline]
    pub fn update_bid(&self, price: u64) {
        self.best_bid.store(price, Ordering::Release);
    }

    #[inline]
    pub fn update_ask(&self, price: u64) {
        self.best_ask.store(price, Ordering::Release);
    }

    #[inline]
    pub fn get_bid(&self) -> Option<u64> {
        let bid = self.best_bid.load(Ordering::Acquire);
        if bid == Self::NO_PRICE { None } else { Some(bid) }
    }

    #[inline]
    pub fn get_ask(&self) -> Option<u64> {
        let ask = self.best_ask.load(Ordering::Acquire);
        if ask == Self::NO_PRICE { None } else { Some(ask) }
    }

    #[inline]
    pub fn spread(&self) -> Option<u64> {
        match (self.get_bid(), self.get_ask()) {
            (Some(bid), Some(ask)) if ask > bid => Some(ask - bid),
            _ => None,
        }
    }
}

fn main() {
    let cache = BestPriceCache::new();

    cache.update_bid(50000);
    cache.update_ask(50010);

    println!("Best bid: {:?}", cache.get_bid());
    println!("Best ask: {:?}", cache.get_ask());
    println!("Spread: {:?}", cache.spread());
}
```

### Pre-allocation and Arena Allocator

```rust
/// Simple arena allocator for fills
pub struct FillArena {
    buffer: Vec<Fill>,
    cursor: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Fill {
    pub maker_id: u64,
    pub taker_id: u64,
    pub price: u64,
    pub quantity: u64,
}

impl FillArena {
    pub fn with_capacity(capacity: usize) -> Self {
        FillArena {
            buffer: vec![Fill::default(); capacity],
            cursor: 0,
        }
    }

    /// Reset arena for reuse
    #[inline]
    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    /// Allocate new fill without heap allocation
    #[inline]
    pub fn alloc(&mut self, fill: Fill) -> Option<&Fill> {
        if self.cursor >= self.buffer.len() {
            return None;
        }
        self.buffer[self.cursor] = fill;
        let result = &self.buffer[self.cursor];
        self.cursor += 1;
        Some(result)
    }

    /// Get all fills
    pub fn fills(&self) -> &[Fill] {
        &self.buffer[..self.cursor]
    }

    pub fn len(&self) -> usize {
        self.cursor
    }
}

fn main() {
    let mut arena = FillArena::with_capacity(1000);

    // Allocation without heap
    for i in 0..10 {
        arena.alloc(Fill {
            maker_id: i,
            taker_id: i + 100,
            price: 50000,
            quantity: 100,
        });
    }

    println!("Fills count: {}", arena.len());
    for fill in arena.fills() {
        println!("  {:?}", fill);
    }

    // Reset for reuse
    arena.reset();
    println!("After reset: {}", arena.len());
}
```

## Practical Exercises

### Exercise 1: Order Cancellation

Add order cancellation functionality to the matcher:

```rust
impl Matcher {
    /// Cancel order by ID
    pub fn cancel_order(&mut self, order_id: u64) -> Option<Order> {
        // Your code here
        // Hint: search both sides of the book
        // and remove order from the corresponding price level
        todo!()
    }
}
```

### Exercise 2: Fill-or-Kill Orders

Implement FoK order processing (complete fill or cancel):

```rust
impl Matcher {
    /// FoK order — fills completely or cancels
    pub fn process_fok_order(&mut self, order: Order) -> Result<Vec<Fill>, &'static str> {
        // Your code here
        // 1. Check if there's enough liquidity
        // 2. If yes — execute completely
        // 3. If no — return error
        todo!()
    }
}
```

### Exercise 3: Immediate-or-Cancel Orders

Implement IoC orders (immediate partial execution):

```rust
impl Matcher {
    /// IoC order — fills what's possible, cancels remainder
    pub fn process_ioc_order(&mut self, order: Order) -> Vec<Fill> {
        // Your code here
        // 1. Match like a regular order
        // 2. DON'T add remainder to book
        todo!()
    }
}
```

### Exercise 4: Latency Metrics

Add latency metrics collection:

```rust
pub struct LatencyStats {
    pub min_ns: u64,
    pub max_ns: u64,
    pub avg_ns: f64,
    pub p50_ns: u64,
    pub p99_ns: u64,
    pub p999_ns: u64,
}

impl Matcher {
    /// Get latency statistics
    pub fn latency_stats(&self) -> LatencyStats {
        // Your code here
        todo!()
    }
}
```

## Homework

### 1. Multi-threaded Matcher

Create a multi-threaded version of the matcher:

```rust
/// Multi-threaded matcher with symbol sharding
pub struct ShardedMatcher {
    shards: Vec<std::sync::Mutex<Matcher>>,
    shard_count: usize,
}

impl ShardedMatcher {
    pub fn new(shard_count: usize) -> Self { todo!() }

    /// Determine shard by symbol_id
    fn get_shard(&self, symbol_id: u16) -> usize { todo!() }

    /// Process order in the corresponding shard
    pub fn process_order(&self, order: Order, symbol_id: u16) -> Vec<Fill> { todo!() }
}
```

Requirements:
- Symbol-based sharding for parallel processing
- Lock-free queue for incoming orders
- Atomic counters for statistics
- Multi-threaded performance benchmark

### 2. Persistence and Recovery

Implement state save and restore system:

```rust
/// Matcher state snapshot
pub struct MatcherSnapshot {
    pub timestamp: u64,
    pub orders: Vec<Order>,
    pub sequence_number: u64,
}

impl Matcher {
    /// Create snapshot of current state
    pub fn snapshot(&self) -> MatcherSnapshot { todo!() }

    /// Restore state from snapshot
    pub fn restore(snapshot: MatcherSnapshot) -> Self { todo!() }

    /// Operations log for replay
    pub fn operation_log(&self) -> Vec<MatcherOperation> { todo!() }
}
```

### 3. Market Simulation

Create a market simulator for testing:

```rust
/// Market data generator
pub struct MarketSimulator {
    pub symbol_id: u16,
    pub base_price: u64,
    pub volatility: f64,
    pub order_rate: f64,  // orders per second
}

impl MarketSimulator {
    /// Generate random order
    pub fn generate_order(&mut self) -> Order { todo!() }

    /// Simulate trading session
    pub fn simulate(&mut self, matcher: &mut Matcher, duration_secs: u64) -> SimulationResult {
        todo!()
    }
}

pub struct SimulationResult {
    pub total_orders: u64,
    pub total_fills: u64,
    pub total_volume: u64,
    pub final_price: u64,
    pub price_volatility: f64,
    pub avg_latency_ns: f64,
}
```

### 4. Advanced Order Types

Implement additional order types:

```rust
#[derive(Debug, Clone)]
pub enum AdvancedOrder {
    /// Stop-loss order
    StopLoss {
        order: Order,
        trigger_price: u64,
    },
    /// Take-profit order
    TakeProfit {
        order: Order,
        trigger_price: u64,
    },
    /// Trailing stop
    TrailingStop {
        order: Order,
        trail_amount: u64,
        highest_price: u64,
    },
    /// Iceberg order (hidden volume)
    Iceberg {
        order: Order,
        visible_quantity: u64,
        total_quantity: u64,
    },
}

impl Matcher {
    /// Process advanced order types
    pub fn process_advanced_order(&mut self, order: AdvancedOrder) -> Vec<Fill> {
        todo!()
    }

    /// Check triggers at current price
    pub fn check_triggers(&mut self, current_price: u64) -> Vec<Fill> {
        todo!()
    }
}
```

## What We Learned

| Concept | Application |
|---------|-------------|
| **Zero-copy** | Passing orders without copying |
| **Pre-allocation** | Memory pools for orders and fills |
| **Lock-free structures** | SPSC queues for multi-threading |
| **Cache optimization** | Struct alignment for cache lines |
| **BTreeMap** | Sorted storage of price levels |
| **Atomic operations** | Best price caching |
| **Arena allocator** | Avoiding heap allocations |
| **Benchmarking** | Throughput measurement |

## Key Performance Metrics

| Metric | Typical Value | Target |
|--------|--------------|--------|
| Latency | 100-500 ns | < 1 us |
| Throughput | 1-5M orders/sec | > 1M |
| Allocations | 0 on hot path | 0 |
| Cache misses | Minimal | < 1% |

## Navigation

[← Previous day](../326-async-vs-threading/en.md) | [Next day →](../335-*/en.md)
