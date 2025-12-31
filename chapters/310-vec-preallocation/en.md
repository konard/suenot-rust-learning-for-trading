# Day 310: Vec with Pre-allocation

## Trading Analogy

Imagine you're processing thousands of stock price ticks in real-time. Every second, a stream of data arrives, and you're adding them to a list for subsequent analysis.

If you create an empty `Vec` and gradually add elements, Rust will be forced to **reallocate memory multiple times** — like trading on an exchange with a small money bag, and every time the bag fills up, you have to run for a new, bigger bag, transfer all the money, and throw away the old one.

**Pre-allocation** is like arriving at the exchange with a large briefcase that will definitely fit all your money for the day. You reserve the required amount of space in advance, and you don't need to spend time constantly replacing the container.

## What is Vec Pre-allocation?

When we know (or can estimate) the number of elements that will be added to a `Vec`, we can reserve memory in advance:

| Method | Description |
|--------|-------------|
| `Vec::new()` | Creates an empty Vec with zero capacity |
| `Vec::with_capacity(n)` | Creates a Vec with pre-allocated capacity for n elements |
| `vec.reserve(n)` | Reserves additional space for at least n elements |
| `vec.reserve_exact(n)` | Reserves space for exactly n additional elements |
| `vec.capacity()` | Returns the current capacity of the Vec |
| `vec.len()` | Returns the number of elements in the Vec |
| `vec.shrink_to_fit()` | Reduces capacity to match current length |

**Important**: `capacity` is how many elements can fit without reallocation, while `len` is how many elements are actually stored.

## Example Without Pre-allocation

```rust
fn collect_price_ticks_slow(count: usize) -> Vec<f64> {
    let mut prices = Vec::new(); // Capacity: 0

    for i in 0..count {
        let price = 100.0 + (i as f64 * 0.01);
        prices.push(price);

        // Vec will reallocate several times:
        // capacity: 0 → 4 → 8 → 16 → 32 → 64 → 128 → ...
    }

    prices
}

fn main() {
    let ticks = collect_price_ticks_slow(1000);
    println!("Collected {} ticks", ticks.len());
    println!("Vector capacity: {}", ticks.capacity());
    // Capacity will be greater than 1000 due to growth strategy
}
```

**Problem**: Each time the Vec overflows, it allocates a new buffer (usually 2x larger), copies all data, and frees the old buffer. This is slow!

## Example With Pre-allocation

```rust
fn collect_price_ticks_fast(count: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(count); // Reserve memory upfront

    for i in 0..count {
        let price = 100.0 + (i as f64 * 0.01);
        prices.push(price);
        // No reallocations! Memory is already reserved
    }

    prices
}

fn main() {
    let ticks = collect_price_ticks_fast(1000);
    println!("Collected {} ticks", ticks.len());
    println!("Vector capacity: {}", ticks.capacity());
    // Capacity will be exactly 1000 (or slightly more)
}
```

## Practical Example: Collecting Orders

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    quantity: u32,
    price: f64,
}

fn collect_orders_without_prealloc(order_count: usize) -> Vec<Order> {
    let mut orders = Vec::new(); // Bad practice for known size

    for i in 0..order_count {
        orders.push(Order {
            id: i as u64,
            symbol: "BTCUSDT".to_string(),
            quantity: 1,
            price: 50000.0 + (i as f64 * 10.0),
        });
    }

    orders
}

fn collect_orders_with_prealloc(order_count: usize) -> Vec<Order> {
    let mut orders = Vec::with_capacity(order_count); // Good practice

    for i in 0..order_count {
        orders.push(Order {
            id: i as u64,
            symbol: "BTCUSDT".to_string(),
            quantity: 1,
            price: 50000.0 + (i as f64 * 10.0),
        });
    }

    orders
}

fn main() {
    use std::time::Instant;

    // Test without pre-allocation
    let start = Instant::now();
    let orders1 = collect_orders_without_prealloc(10000);
    let duration1 = start.elapsed();

    // Test with pre-allocation
    let start = Instant::now();
    let orders2 = collect_orders_with_prealloc(10000);
    let duration2 = start.elapsed();

    println!("Without pre-allocation: {:?}", duration1);
    println!("With pre-allocation: {:?}", duration2);
    println!("Speedup: {:.2}x", duration1.as_secs_f64() / duration2.as_secs_f64());
}
```

## Using reserve() for Dynamic Growth

Sometimes we don't know the exact size in advance, but can estimate:

```rust
fn process_market_data_stream() {
    let mut prices = Vec::new();

    // Get first batch of data
    let initial_batch_size = 1000;
    prices.reserve(initial_batch_size);

    for i in 0..initial_batch_size {
        prices.push(100.0 + i as f64);
    }

    println!("After first batch: len={}, capacity={}",
             prices.len(), prices.capacity());

    // Get second batch
    let second_batch_size = 5000;
    prices.reserve(second_batch_size);

    for i in 0..second_batch_size {
        prices.push(100.0 + i as f64);
    }

    println!("After second batch: len={}, capacity={}",
             prices.len(), prices.capacity());
}

fn main() {
    process_market_data_stream();
}
```

## Example: Aggregating Trades to Order Book Levels

```rust
#[derive(Debug)]
struct Trade {
    price: f64,
    volume: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct OrderBookLevel {
    price: f64,
    total_volume: f64,
    trade_count: usize,
}

fn aggregate_trades_to_levels(trades: &[Trade], level_size: f64) -> Vec<OrderBookLevel> {
    if trades.is_empty() {
        return Vec::new();
    }

    // Estimate number of price levels
    let min_price = trades.iter().map(|t| t.price).fold(f64::INFINITY, f64::min);
    let max_price = trades.iter().map(|t| t.price).fold(f64::NEG_INFINITY, f64::max);
    let estimated_levels = ((max_price - min_price) / level_size).ceil() as usize + 1;

    // Pre-allocate for levels
    let mut levels = Vec::with_capacity(estimated_levels);

    let mut current_level_price = (min_price / level_size).floor() * level_size;
    let mut current_volume = 0.0;
    let mut current_count = 0;

    for trade in trades {
        let trade_level_price = (trade.price / level_size).floor() * level_size;

        if trade_level_price != current_level_price {
            // Save previous level
            if current_count > 0 {
                levels.push(OrderBookLevel {
                    price: current_level_price,
                    total_volume: current_volume,
                    trade_count: current_count,
                });
            }

            // Start new level
            current_level_price = trade_level_price;
            current_volume = trade.volume;
            current_count = 1;
        } else {
            current_volume += trade.volume;
            current_count += 1;
        }
    }

    // Add last level
    if current_count > 0 {
        levels.push(OrderBookLevel {
            price: current_level_price,
            total_volume: current_volume,
            trade_count: current_count,
        });
    }

    levels
}

fn main() {
    // Generate test trades
    let mut trades = Vec::with_capacity(10000);
    for i in 0..10000 {
        trades.push(Trade {
            price: 50000.0 + (i as f64 % 100.0) * 0.1,
            volume: 0.1 + (i as f64 % 10.0) * 0.05,
            timestamp: i as u64,
        });
    }

    let levels = aggregate_trades_to_levels(&trades, 1.0);

    println!("Processed {} trades", trades.len());
    println!("Created {} price levels", levels.len());
    println!("First level: {:?}", levels.first());
    println!("Last level: {:?}", levels.last());
}
```

## Performance Measurement

```rust
use std::time::Instant;

fn benchmark_vec_allocations() {
    const SIZE: usize = 100_000;
    const ITERATIONS: usize = 100;

    // Test 1: without pre-allocation
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut vec = Vec::new();
        for i in 0..SIZE {
            vec.push(i as f64);
        }
    }
    let without_prealloc = start.elapsed();

    // Test 2: with pre-allocation
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut vec = Vec::with_capacity(SIZE);
        for i in 0..SIZE {
            vec.push(i as f64);
        }
    }
    let with_prealloc = start.elapsed();

    // Test 3: collect() from iterator (optimal variant)
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let vec: Vec<f64> = (0..SIZE).map(|i| i as f64).collect();
    }
    let with_collect = start.elapsed();

    println!("Results for {} elements, {} iterations:", SIZE, ITERATIONS);
    println!("  Without pre-allocation: {:?}", without_prealloc);
    println!("  With pre-allocation:    {:?}", with_prealloc);
    println!("  Using collect():        {:?}", with_collect);
    println!();
    println!("Speedup:");
    println!("  with_capacity: {:.2}x faster",
             without_prealloc.as_secs_f64() / with_prealloc.as_secs_f64());
    println!("  collect():     {:.2}x faster",
             without_prealloc.as_secs_f64() / with_collect.as_secs_f64());
}

fn main() {
    benchmark_vec_allocations();
}
```

## When to Use Pre-allocation?

| Situation | Recommendation |
|-----------|----------------|
| Know exact size | `Vec::with_capacity(n)` |
| Can estimate size | `Vec::with_capacity(estimated)` |
| Unknown size but many additions expected | `vec.reserve(large_number)` at first opportunity |
| Size unpredictable and small | `Vec::new()` — pre-allocation not critical |
| Collecting from iterator | Use `.collect()` — it optimizes automatically |

## What We Learned

| Concept | Description |
|---------|-------------|
| Pre-allocation | Reserving memory for Vec in advance |
| `Vec::with_capacity(n)` | Creating Vec with reserved capacity |
| `reserve(n)` | Reserving additional space |
| `capacity()` vs `len()` | Capacity vs actual number of elements |
| Performance | Pre-allocation avoids multiple memory reallocations |
| Usage pattern | Reserve memory when size is known or predictable |

## Practical Exercises

1. **Basic Pre-allocation**: Write a function `collect_prices(count: usize) -> Vec<f64>` that collects `count` random prices. Measure the difference between versions with `Vec::new()` and `Vec::with_capacity(count)`.

2. **Dynamic Reservation**: Implement a function that reads prices from a data stream in batches of 1000 elements. Use `reserve()` before each batch.

3. **Aggregation Optimization**: Create a function to calculate OHLC (Open, High, Low, Close) candles from ticks. Use pre-allocation for the candle vector if you know the number of time intervals.

4. **Method Comparison**: Compare performance of three approaches for creating a Vec of 1 million elements:
   - `Vec::new()` + `push()`
   - `Vec::with_capacity()` + `push()`
   - Iterator + `collect()`

## Homework

1. **Orderbook Analysis**: Write a function that accepts a stream of orderbook updates (each update is a structure with price and volume) and aggregates them into price levels with 0.01 step size. Use pre-allocation for optimization.

2. **Historical Data Caching**: Implement a `PriceCache` structure that stores the last N prices. On creation, use `Vec::with_capacity(N)` and add a method for data rotation (removing old entries when adding new ones without reallocation).

3. **Batch Trade Processing**: Create a system for processing trades in batches. Implement `TradeProcessor` that collects trades into fixed-size batches (e.g., 1000) with pre-allocation and processes them when full.

4. **Benchmarks**: Write a benchmark suite to compare performance of different Vec creation strategies:
   - Without pre-allocation
   - With exact pre-allocation
   - With excessive pre-allocation (reserve 2x needed)
   - With insufficient pre-allocation (reserve 0.5x, then catch up)

   Test on sizes: 100, 1_000, 10_000, 100_000, 1_000_000 elements.

## Navigation

[← Previous Day](../294-overfitting-strategy-optimization/en.md) | [Next Day →](../311-hashmap-preallocation/en.md)
