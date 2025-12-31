# Day 88: BinaryHeap — Priority Queue

## Trading Analogy

Imagine an order book: buy orders are sorted by price from high to low (best price at the top), while sell orders are sorted from low to high. This is a **priority queue** — a data structure where the element with the highest priority is always accessible first.

In algorithmic trading, priority queues are used for:
- **Selecting best orders** — orders with better prices are processed first
- **Prioritizing signals** — important signals are processed earlier
- **Event management** — events with earlier execution times run first
- **Strategy ranking** — selecting strategies with best returns

## What is BinaryHeap?

`BinaryHeap` in Rust is a priority queue implementation based on a binary heap. By default, it's a **max-heap**: the largest element is always at the top.

```rust
use std::collections::BinaryHeap;

fn main() {
    let mut prices = BinaryHeap::new();

    prices.push(42000);
    prices.push(42500);
    prices.push(41800);
    prices.push(42200);

    // Largest price is extracted first
    while let Some(price) = prices.pop() {
        println!("Price: {}", price);
    }
    // Output: 42500, 42200, 42000, 41800
}
```

## Basic Operations

### Creating and Adding Elements

```rust
use std::collections::BinaryHeap;

fn main() {
    // Create empty heap
    let mut heap: BinaryHeap<i32> = BinaryHeap::new();

    // Create with capacity
    let mut heap_with_capacity: BinaryHeap<i32> = BinaryHeap::with_capacity(100);

    // Create from vector
    let prices = vec![42000, 42500, 41800, 42200];
    let heap_from_vec: BinaryHeap<i32> = BinaryHeap::from(prices);

    // Add elements
    heap.push(100);
    heap.push(200);
    heap.push(150);

    println!("Heap: {:?}", heap);
    println!("Size: {}", heap.len());
}
```

### Extracting Elements

```rust
use std::collections::BinaryHeap;

fn main() {
    let mut order_priorities = BinaryHeap::from(vec![5, 10, 3, 8, 1]);

    // View maximum element (without removing)
    if let Some(&top) = order_priorities.peek() {
        println!("Highest priority: {}", top);
    }

    // Extract maximum element
    while let Some(priority) = order_priorities.pop() {
        println!("Processing priority: {}", priority);
    }
}
```

## Trading Signals with Priorities

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
struct TradingSignal {
    priority: u32,      // Signal priority (higher = more important)
    symbol: String,
    signal_type: String,
    price: u64,         // Price in cents for precision
}

// Implement Ord for comparison by priority
impl Ord for TradingSignal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for TradingSignal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut signal_queue = BinaryHeap::new();

    // Add signals with different priorities
    signal_queue.push(TradingSignal {
        priority: 5,
        symbol: "BTC/USDT".to_string(),
        signal_type: "BUY".to_string(),
        price: 4200000, // $42,000.00
    });

    signal_queue.push(TradingSignal {
        priority: 10,  // High priority — stop loss!
        symbol: "ETH/USDT".to_string(),
        signal_type: "STOP_LOSS".to_string(),
        price: 220000, // $2,200.00
    });

    signal_queue.push(TradingSignal {
        priority: 3,
        symbol: "SOL/USDT".to_string(),
        signal_type: "BUY".to_string(),
        price: 10000, // $100.00
    });

    // Process signals by priority
    println!("=== Processing Signals ===");
    while let Some(signal) = signal_queue.pop() {
        println!(
            "[Priority {}] {} {} @ ${:.2}",
            signal.priority,
            signal.signal_type,
            signal.symbol,
            signal.price as f64 / 100.0
        );
    }
}
```

## Min-Heap: Inverting Priority

By default, `BinaryHeap` is a max-heap. For min-heap, use `Reverse`:

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

fn main() {
    // Min-heap for Ask prices (we want minimum price)
    let mut ask_prices: BinaryHeap<Reverse<u64>> = BinaryHeap::new();

    ask_prices.push(Reverse(42010));
    ask_prices.push(Reverse(42005));
    ask_prices.push(Reverse(42020));
    ask_prices.push(Reverse(42008));

    println!("=== Best Ask Prices ===");
    while let Some(Reverse(price)) = ask_prices.pop() {
        println!("Ask: ${:.2}", price as f64 / 100.0);
    }
    // Output: 42005, 42008, 42010, 42020 (ascending order)
}
```

## Modeling an Order Book

```rust
use std::collections::BinaryHeap;
use std::cmp::{Ordering, Reverse};

#[derive(Debug, Clone, Eq, PartialEq)]
struct Order {
    price: u64,      // Price in cents
    quantity: u64,   // Quantity
    timestamp: u64,  // Placement time
    order_id: u64,
}

// For Bid: higher price first, then earlier time if equal
impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Equal => other.timestamp.cmp(&self.timestamp), // Earlier time ranks higher
            other => other, // Higher price ranks higher
        }
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Wrapper for Ask orders (min-heap by price)
#[derive(Debug, Clone, Eq, PartialEq)]
struct AskOrder(Order);

impl Ord for AskOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.0.price.cmp(&self.0.price) { // Invert for min-heap
            Ordering::Equal => other.0.timestamp.cmp(&self.0.timestamp),
            other_ord => other_ord,
        }
    }
}

impl PartialOrd for AskOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut bids: BinaryHeap<Order> = BinaryHeap::new();
    let mut asks: BinaryHeap<AskOrder> = BinaryHeap::new();

    // Add buy orders (Bid)
    bids.push(Order { price: 4200000, quantity: 100, timestamp: 1, order_id: 1 });
    bids.push(Order { price: 4199500, quantity: 200, timestamp: 2, order_id: 2 });
    bids.push(Order { price: 4200000, quantity: 150, timestamp: 3, order_id: 3 });
    bids.push(Order { price: 4198000, quantity: 300, timestamp: 4, order_id: 4 });

    // Add sell orders (Ask)
    asks.push(AskOrder(Order { price: 4201000, quantity: 80, timestamp: 1, order_id: 5 }));
    asks.push(AskOrder(Order { price: 4202500, quantity: 120, timestamp: 2, order_id: 6 }));
    asks.push(AskOrder(Order { price: 4201000, quantity: 90, timestamp: 3, order_id: 7 }));

    println!("╔═══════════════════════════════════════════╗");
    println!("║           ORDER BOOK (BTC/USDT)           ║");
    println!("╠═══════════════════════════════════════════╣");

    // Show best Asks (top to bottom)
    println!("║ {:^10} {:>12} {:>10} {:>6} ║", "Side", "Price", "Qty", "ID");
    println!("╠═══════════════════════════════════════════╣");

    // Clone for display (to preserve the queue)
    let asks_display: Vec<_> = asks.clone().into_sorted_vec();
    for ask in asks_display.iter().rev().take(3) {
        println!(
            "║ {:^10} {:>12.2} {:>10} {:>6} ║",
            "ASK",
            ask.0.price as f64 / 100.0,
            ask.0.quantity,
            ask.0.order_id
        );
    }

    println!("╠═══════════════════════════════════════════╣");

    let bids_display: Vec<_> = bids.clone().into_sorted_vec();
    for bid in bids_display.iter().rev().take(3) {
        println!(
            "║ {:^10} {:>12.2} {:>10} {:>6} ║",
            "BID",
            bid.price as f64 / 100.0,
            bid.quantity,
            bid.order_id
        );
    }

    println!("╚═══════════════════════════════════════════╝");

    // Spread
    if let (Some(best_bid), Some(best_ask)) = (bids.peek(), asks.peek()) {
        let spread = best_ask.0.price - best_bid.price;
        println!("\nSpread: ${:.2}", spread as f64 / 100.0);
    }
}
```

## Event Queue by Time

```rust
use std::collections::BinaryHeap;
use std::cmp::{Ordering, Reverse};

#[derive(Debug, Eq, PartialEq)]
struct ScheduledEvent {
    execute_at: u64,    // Execution time (timestamp)
    event_type: String,
    data: String,
}

// Min-heap by time (earlier time = higher priority)
impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        other.execute_at.cmp(&self.execute_at) // Invert for min-heap
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut event_queue = BinaryHeap::new();

    // Schedule events
    event_queue.push(ScheduledEvent {
        execute_at: 1000,
        event_type: "CHECK_STOP_LOSS".to_string(),
        data: "BTC/USDT".to_string(),
    });

    event_queue.push(ScheduledEvent {
        execute_at: 500,
        event_type: "PLACE_ORDER".to_string(),
        data: "ETH/USDT BUY 1.5".to_string(),
    });

    event_queue.push(ScheduledEvent {
        execute_at: 750,
        event_type: "CANCEL_ORDER".to_string(),
        data: "Order #12345".to_string(),
    });

    event_queue.push(ScheduledEvent {
        execute_at: 500,
        event_type: "UPDATE_PRICE".to_string(),
        data: "SOL/USDT".to_string(),
    });

    // Process events in time order
    println!("=== Event Timeline ===");
    while let Some(event) = event_queue.pop() {
        println!(
            "[T={}] {}: {}",
            event.execute_at,
            event.event_type,
            event.data
        );
    }
}
```

## Top-N Best Trades

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    pnl: f64,
    entry_price: f64,
    exit_price: f64,
}

fn find_top_n_trades(trades: &[Trade], n: usize) -> Vec<Trade> {
    // Use min-heap of size n for efficient top-N search
    let mut min_heap: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();

    for (idx, trade) in trades.iter().enumerate() {
        // Convert f64 to i64 for comparison (multiply by 100 for cents)
        let pnl_cents = (trade.pnl * 100.0) as i64;

        if min_heap.len() < n {
            min_heap.push(Reverse((pnl_cents, idx)));
        } else if let Some(&Reverse((min_pnl, _))) = min_heap.peek() {
            if pnl_cents > min_pnl {
                min_heap.pop();
                min_heap.push(Reverse((pnl_cents, idx)));
            }
        }
    }

    // Extract indices and sort by descending PnL
    let mut result: Vec<_> = min_heap
        .into_iter()
        .map(|Reverse((_, idx))| trades[idx].clone())
        .collect();

    result.sort_by(|a, b| b.pnl.partial_cmp(&a.pnl).unwrap());
    result
}

fn main() {
    let trades = vec![
        Trade { symbol: "BTC/USDT".to_string(), pnl: 150.50, entry_price: 42000.0, exit_price: 42150.50 },
        Trade { symbol: "ETH/USDT".to_string(), pnl: -30.25, entry_price: 2200.0, exit_price: 2169.75 },
        Trade { symbol: "SOL/USDT".to_string(), pnl: 85.00, entry_price: 100.0, exit_price: 185.00 },
        Trade { symbol: "BTC/USDT".to_string(), pnl: 200.00, entry_price: 41800.0, exit_price: 42000.0 },
        Trade { symbol: "ADA/USDT".to_string(), pnl: 45.75, entry_price: 0.50, exit_price: 0.55 },
        Trade { symbol: "DOT/USDT".to_string(), pnl: -15.00, entry_price: 7.50, exit_price: 7.35 },
        Trade { symbol: "ETH/USDT".to_string(), pnl: 120.00, entry_price: 2100.0, exit_price: 2220.0 },
    ];

    let top_3 = find_top_n_trades(&trades, 3);

    println!("=== Top 3 Profitable Trades ===");
    println!("{:<12} {:>12} {:>12} {:>12}", "Symbol", "Entry", "Exit", "PnL");
    println!("{}", "-".repeat(50));

    for trade in top_3 {
        println!(
            "{:<12} {:>12.2} {:>12.2} {:>+12.2}",
            trade.symbol, trade.entry_price, trade.exit_price, trade.pnl
        );
    }
}
```

## Practical Example: Rebalancing Scheduler

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
struct RebalanceTask {
    urgency: u32,           // Urgency (1-10)
    deviation_percent: u32, // Deviation from target % (in hundredths)
    symbol: String,
    current_weight: u32,    // Current portfolio weight (in hundredths of %)
    target_weight: u32,     // Target weight (in hundredths of %)
}

impl Ord for RebalanceTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // First by urgency, then by deviation
        match self.urgency.cmp(&other.urgency) {
            Ordering::Equal => self.deviation_percent.cmp(&other.deviation_percent),
            other => other,
        }
    }
}

impl PartialOrd for RebalanceTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut rebalance_queue = BinaryHeap::new();

    // Analyze portfolio and create rebalancing tasks
    let portfolio = vec![
        ("BTC", 4500, 4000),   // (symbol, current_weight, target_weight) in hundredths %
        ("ETH", 2800, 3000),
        ("SOL", 1500, 1500),
        ("ADA", 800, 1000),
        ("DOT", 400, 500),
    ];

    for (symbol, current, target) in portfolio {
        let deviation = if current > target {
            current - target
        } else {
            target - current
        };

        // Urgency depends on deviation magnitude
        let urgency = match deviation {
            0..=100 => 1,
            101..=300 => 3,
            301..=500 => 5,
            501..=1000 => 7,
            _ => 10,
        };

        if deviation > 0 {
            rebalance_queue.push(RebalanceTask {
                urgency,
                deviation_percent: deviation,
                symbol: symbol.to_string(),
                current_weight: current,
                target_weight: target,
            });
        }
    }

    println!("=== Portfolio Rebalancing Queue ===");
    println!("{:<8} {:>10} {:>10} {:>10} {:>10}",
             "Symbol", "Current%", "Target%", "Dev%", "Urgency");
    println!("{}", "-".repeat(55));

    while let Some(task) = rebalance_queue.pop() {
        let action = if task.current_weight > task.target_weight {
            "SELL"
        } else {
            "BUY"
        };

        println!(
            "{:<8} {:>9.2}% {:>9.2}% {:>9.2}% {:>10} → {}",
            task.symbol,
            task.current_weight as f64 / 100.0,
            task.target_weight as f64 / 100.0,
            task.deviation_percent as f64 / 100.0,
            task.urgency,
            action
        );
    }
}
```

## Useful BinaryHeap Methods

```rust
use std::collections::BinaryHeap;

fn main() {
    let mut heap = BinaryHeap::from(vec![3, 1, 4, 1, 5, 9, 2, 6]);

    // Size and capacity
    println!("Length: {}", heap.len());
    println!("Capacity: {}", heap.capacity());
    println!("Is empty: {}", heap.is_empty());

    // View maximum element
    if let Some(&max) = heap.peek() {
        println!("Max element: {}", max);
    }

    // Modify maximum element
    if let Some(max) = heap.peek_mut() {
        *max = 100;
    }
    println!("After peek_mut: {:?}", heap.peek());

    // Clear
    heap.clear();
    println!("After clear, is empty: {}", heap.is_empty());

    // Create from iterator
    let prices = vec![42000, 42100, 41900];
    let heap2: BinaryHeap<_> = prices.into_iter().collect();
    println!("From iterator: {:?}", heap2);

    // Convert to sorted vector
    let sorted: Vec<_> = heap2.into_sorted_vec();
    println!("Sorted (ascending): {:?}", sorted);
}
```

## Comparing BinaryHeap with Other Collections

| Operation | BinaryHeap | Vec (sorted) | VecDeque |
|-----------|------------|--------------|----------|
| push | O(log n) | O(n) | O(1) |
| pop (max/min) | O(log n) | O(1) | O(1) |
| peek | O(1) | O(1) | O(1) |
| Search | O(n) | O(log n) | O(n) |
| Use case | Priority queues | Sorted data | FIFO/LIFO |

## What We Learned

| Concept | Description |
|---------|-------------|
| `BinaryHeap::new()` | Create empty heap (max-heap) |
| `heap.push(x)` | Add element |
| `heap.pop()` | Extract maximum |
| `heap.peek()` | View maximum |
| `Reverse(x)` | Inversion for min-heap |
| `impl Ord` | Custom sort order |

## Homework

1. **Order Queue**: Create a `LimitOrder` struct with `price`, `quantity`, `timestamp`, and `is_buy` fields. Implement two queues: Bid (max-heap by price) and Ask (min-heap by price). When prices are equal, earlier orders have priority.

2. **Top-5 Volatile Assets**: Given an array of `Asset { symbol, volatility }` structs, use BinaryHeap to find the 5 assets with highest volatility in O(n log 5) time.

3. **Signal Scheduler**: Create a system where trading signals have `priority` (1-10) and `expire_at` (timestamp). Expired signals should be ignored when extracting. Implement `add_signal()` and `get_next_valid_signal()` methods.

4. **Median Tracker**: Using two BinaryHeaps (max-heap and min-heap), implement a `MedianTracker` structure with methods:
   - `add_price(price: f64)` — add a price
   - `get_median() -> f64` — get current median

   Hint: max-heap stores the smaller half, min-heap stores the larger half.

## Navigation

[← Previous day](../087-vecdeque-order-queue/en.md) | [Next day →](../089-combining-structs/en.md)
