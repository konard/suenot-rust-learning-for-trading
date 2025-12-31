# Day 87: VecDeque — Order Queue

## Trading Analogy

On an exchange, orders are processed in a specific order:
- **Order queue**: first in — first out (FIFO)
- **Order book**: need fast access to best prices on both sides
- **Trade history**: new trades added to the end, old ones removed from the beginning

`VecDeque` (double-ended queue) is the ideal structure for such scenarios:
- O(1) insertion/removal from both ends
- More efficient than `Vec` when you need operations at the beginning of the collection

## Creating VecDeque

```rust
use std::collections::VecDeque;

fn main() {
    // Empty order queue
    let mut order_queue: VecDeque<String> = VecDeque::new();

    // With initial capacity (for high-frequency trading)
    let mut hft_queue: VecDeque<f64> = VecDeque::with_capacity(10000);

    // From vector
    let prices = vec![42000.0, 42100.0, 42050.0];
    let price_buffer: VecDeque<f64> = VecDeque::from(prices);

    // From iterator
    let quick_queue: VecDeque<i32> = [1, 2, 3, 4, 5].into_iter().collect();

    println!("Order queue len: {}", order_queue.len());
    println!("HFT queue capacity: {}", hft_queue.capacity());
    println!("Price buffer: {:?}", price_buffer);
}
```

## Adding Elements

```rust
use std::collections::VecDeque;

fn main() {
    let mut orders: VecDeque<&str> = VecDeque::new();

    // push_back — add to end (new order to queue)
    orders.push_back("BUY BTC 0.1");
    orders.push_back("SELL ETH 2.0");
    orders.push_back("BUY BTC 0.5");

    println!("Order queue: {:?}", orders);
    // ["BUY BTC 0.1", "SELL ETH 2.0", "BUY BTC 0.5"]

    // push_front — add to beginning (priority order)
    orders.push_front("URGENT: SELL BTC 1.0");

    println!("With priority order: {:?}", orders);
    // ["URGENT: SELL BTC 1.0", "BUY BTC 0.1", "SELL ETH 2.0", "BUY BTC 0.5"]
}
```

## Removing Elements

```rust
use std::collections::VecDeque;

fn main() {
    let mut orders: VecDeque<&str> = VecDeque::new();
    orders.push_back("Order 1");
    orders.push_back("Order 2");
    orders.push_back("Order 3");
    orders.push_back("Order 4");

    // pop_front — extract first (FIFO processing)
    if let Some(order) = orders.pop_front() {
        println!("Processing: {}", order);  // Order 1
    }

    // pop_back — extract last (cancel last order)
    if let Some(order) = orders.pop_back() {
        println!("Cancelled: {}", order);  // Order 4
    }

    println!("Remaining: {:?}", orders);  // ["Order 2", "Order 3"]
}
```

## Sliding Price Window

One of the main usage patterns for VecDeque is the sliding window:

```rust
use std::collections::VecDeque;

fn main() {
    // Sliding window for SMA-5 calculation
    let mut price_window: VecDeque<f64> = VecDeque::with_capacity(5);
    let window_size = 5;

    // Stream of new prices
    let incoming_prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
                           42300.0, 42250.0, 42400.0];

    for price in incoming_prices {
        // Add new price to the end
        price_window.push_back(price);

        // If window is full — remove old price
        if price_window.len() > window_size {
            price_window.pop_front();
        }

        // Calculate SMA when window is full
        if price_window.len() == window_size {
            let sma: f64 = price_window.iter().sum::<f64>() / window_size as f64;
            println!("Price: {:.0} | SMA-5: {:.2}", price, sma);
        } else {
            println!("Price: {:.0} | Collecting data ({}/{})",
                     price, price_window.len(), window_size);
        }
    }
}
```

## Accessing Elements

```rust
use std::collections::VecDeque;

fn main() {
    let mut prices: VecDeque<f64> = VecDeque::new();
    prices.push_back(42000.0);
    prices.push_back(42100.0);
    prices.push_back(42200.0);
    prices.push_back(42300.0);

    // Index access
    println!("First price: {}", prices[0]);
    println!("Last price: {}", prices[prices.len() - 1]);

    // Safe access
    if let Some(price) = prices.get(2) {
        println!("Price at index 2: {}", price);
    }

    // front() and back() — access ends
    println!("Front (oldest): {:?}", prices.front());
    println!("Back (newest): {:?}", prices.back());

    // Mutable access
    if let Some(price) = prices.front_mut() {
        *price = 41900.0;  // Correct first price
    }

    println!("Updated prices: {:?}", prices);
}
```

## Order Book Implementation

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

struct OrderBook {
    buy_orders: VecDeque<Order>,   // Buy orders
    sell_orders: VecDeque<Order>,  // Sell orders
    next_id: u64,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            buy_orders: VecDeque::new(),
            sell_orders: VecDeque::new(),
            next_id: 1,
        }
    }

    fn add_order(&mut self, side: &str, price: f64, quantity: f64, timestamp: u64) {
        let order = Order {
            id: self.next_id,
            side: side.to_string(),
            price,
            quantity,
            timestamp,
        };
        self.next_id += 1;

        match side {
            "BUY" => self.buy_orders.push_back(order),
            "SELL" => self.sell_orders.push_back(order),
            _ => println!("Unknown order side"),
        }
    }

    fn process_next_buy(&mut self) -> Option<Order> {
        self.buy_orders.pop_front()
    }

    fn process_next_sell(&mut self) -> Option<Order> {
        self.sell_orders.pop_front()
    }

    fn cancel_last_order(&mut self, side: &str) -> Option<Order> {
        match side {
            "BUY" => self.buy_orders.pop_back(),
            "SELL" => self.sell_orders.pop_back(),
            _ => None,
        }
    }

    fn display(&self) {
        println!("=== Order Book ===");
        println!("Buy orders ({}):", self.buy_orders.len());
        for order in &self.buy_orders {
            println!("  #{}: {} {} @ {:.2}",
                     order.id, order.quantity, order.side, order.price);
        }
        println!("Sell orders ({}):", self.sell_orders.len());
        for order in &self.sell_orders {
            println!("  #{}: {} {} @ {:.2}",
                     order.id, order.quantity, order.side, order.price);
        }
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Add orders
    book.add_order("BUY", 41950.0, 0.5, 1000);
    book.add_order("BUY", 41900.0, 1.0, 1001);
    book.add_order("SELL", 42000.0, 0.3, 1002);
    book.add_order("SELL", 42050.0, 0.7, 1003);

    book.display();

    // Execute first buy order
    if let Some(order) = book.process_next_buy() {
        println!("\nExecuted: {:?}", order);
    }

    // Cancel last sell order
    if let Some(order) = book.cancel_last_order("SELL") {
        println!("Cancelled: {:?}", order);
    }

    println!();
    book.display();
}
```

## Trade History with Limit

```rust
use std::collections::VecDeque;

#[derive(Debug)]
struct Trade {
    price: f64,
    quantity: f64,
    side: String,
    timestamp: u64,
}

struct TradeHistory {
    trades: VecDeque<Trade>,
    max_size: usize,
}

impl TradeHistory {
    fn new(max_size: usize) -> Self {
        TradeHistory {
            trades: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn add_trade(&mut self, price: f64, quantity: f64, side: &str, timestamp: u64) {
        let trade = Trade {
            price,
            quantity,
            side: side.to_string(),
            timestamp,
        };

        self.trades.push_back(trade);

        // Remove old trades if limit exceeded
        while self.trades.len() > self.max_size {
            self.trades.pop_front();
        }
    }

    fn get_recent(&self, count: usize) -> Vec<&Trade> {
        self.trades.iter().rev().take(count).collect()
    }

    fn average_price(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.trades.iter().map(|t| t.price).sum();
        sum / self.trades.len() as f64
    }

    fn total_volume(&self) -> f64 {
        self.trades.iter().map(|t| t.quantity).sum()
    }
}

fn main() {
    let mut history = TradeHistory::new(5);  // Keep only 5 recent trades

    // Add trades
    history.add_trade(42000.0, 0.5, "BUY", 1000);
    history.add_trade(42010.0, 0.3, "BUY", 1001);
    history.add_trade(42005.0, 0.8, "SELL", 1002);
    history.add_trade(42020.0, 0.2, "BUY", 1003);
    history.add_trade(42015.0, 0.6, "SELL", 1004);
    history.add_trade(42025.0, 0.4, "BUY", 1005);  // First trade will be removed
    history.add_trade(42030.0, 0.1, "BUY", 1006);  // Second trade will be removed

    println!("Trade history (max 5):");
    for trade in &history.trades {
        println!("  {:?}", trade);
    }

    println!("\nRecent 3 trades:");
    for trade in history.get_recent(3) {
        println!("  {} {:.1} @ {:.2}", trade.side, trade.quantity, trade.price);
    }

    println!("\nAverage price: {:.2}", history.average_price());
    println!("Total volume: {:.2}", history.total_volume());
}
```

## Iteration and Transformation

```rust
use std::collections::VecDeque;

fn main() {
    let mut prices: VecDeque<f64> = VecDeque::new();
    prices.push_back(42000.0);
    prices.push_back(42100.0);
    prices.push_back(42050.0);
    prices.push_back(42200.0);

    // Iteration
    println!("All prices:");
    for price in &prices {
        println!("  {}", price);
    }

    // With index
    println!("\nWith index:");
    for (i, price) in prices.iter().enumerate() {
        println!("  [{}] {}", i, price);
    }

    // Mutable iteration — apply fee
    for price in prices.iter_mut() {
        *price *= 0.999;  // -0.1% fee
    }
    println!("\nAfter fees: {:?}", prices);

    // Convert to Vec
    let price_vec: Vec<f64> = prices.clone().into_iter().collect();
    println!("\nAs Vec: {:?}", price_vec);

    // make_contiguous — ensures contiguous memory layout
    let slice = prices.make_contiguous();
    println!("As slice: {:?}", slice);
}
```

## Rotate and Other Operations

```rust
use std::collections::VecDeque;

fn main() {
    let mut queue: VecDeque<i32> = (1..=5).collect();
    println!("Original: {:?}", queue);

    // rotate_left — shift elements left
    queue.rotate_left(2);
    println!("Rotate left 2: {:?}", queue);  // [3, 4, 5, 1, 2]

    // rotate_right — shift elements right
    queue.rotate_right(2);
    println!("Rotate right 2: {:?}", queue);  // [1, 2, 3, 4, 5]

    // swap — swap elements
    queue.swap(0, 4);
    println!("After swap(0,4): {:?}", queue);  // [5, 2, 3, 4, 1]

    // retain — keep only elements matching condition
    queue.retain(|&x| x > 2);
    println!("After retain(>2): {:?}", queue);  // [5, 3, 4]

    // clear — empty the queue
    queue.clear();
    println!("After clear: {:?}", queue);  // []
}
```

## VecDeque vs Vec: When to Use Which

```rust
use std::collections::VecDeque;

fn main() {
    // VecDeque is better for:
    // 1. Queues (FIFO) — add to end, remove from beginning
    let mut fifo: VecDeque<i32> = VecDeque::new();
    fifo.push_back(1);   // O(1)
    fifo.pop_front();    // O(1) — Vec would be O(n)!

    // 2. Sliding windows
    // 3. Double-ended operations (deque)

    // Vec is better for:
    // 1. Random access (both O(1), but Vec is faster due to cache)
    // 2. Only add/remove from end
    // 3. Working with slices

    println!("VecDeque: optimal for queues and sliding windows");
    println!("Vec: optimal for stacks and random access");
}
```

## Practical Example: Rate Limiter

```rust
use std::collections::VecDeque;

struct RateLimiter {
    requests: VecDeque<u64>,  // request timestamps
    max_requests: usize,
    window_ms: u64,
}

impl RateLimiter {
    fn new(max_requests: usize, window_ms: u64) -> Self {
        RateLimiter {
            requests: VecDeque::with_capacity(max_requests),
            max_requests,
            window_ms,
        }
    }

    fn allow_request(&mut self, current_time: u64) -> bool {
        // Remove outdated requests
        while let Some(&oldest) = self.requests.front() {
            if current_time - oldest > self.window_ms {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Check limit
        if self.requests.len() < self.max_requests {
            self.requests.push_back(current_time);
            true
        } else {
            false
        }
    }
}

fn main() {
    // Maximum 5 requests per 1000ms
    let mut limiter = RateLimiter::new(5, 1000);

    let timestamps = [100, 200, 300, 400, 500, 600, 700, 1200, 1300];

    for ts in timestamps {
        let allowed = limiter.allow_request(ts);
        println!("Time {}: {}", ts, if allowed { "ALLOWED" } else { "BLOCKED" });
    }
}
```

## What We Learned

| Method | Description | Complexity |
|--------|-------------|------------|
| `push_back(x)` | Add to end | O(1) |
| `push_front(x)` | Add to beginning | O(1) |
| `pop_back()` | Remove from end | O(1) |
| `pop_front()` | Remove from beginning | O(1) |
| `front()` / `back()` | Access ends | O(1) |
| `get(i)` | Index access | O(1) |
| `len()` | Size | O(1) |
| `rotate_left(n)` | Shift left | O(n) |

## Homework

1. **Priority order queue**: create a system where VIP clients can add orders to the front of the queue via `push_front`, while regular clients use `push_back`

2. **Sliding window volatility**: implement standard deviation calculation for prices in a sliding window of size N

3. **Tick buffer**: create a structure that stores the last 1000 ticks and allows getting:
   - Last N ticks
   - Average price over the window
   - Maximum spread for the period

4. **Matching Engine**: implement a simple order matching engine using two VecDeques for bid and ask orders. When adding a new order, check if execution is possible.

## Navigation

[← Previous day](../086-btreemap-sorted-prices/en.md) | [Next day →](../088-binaryheap-priority-queue/en.md)
