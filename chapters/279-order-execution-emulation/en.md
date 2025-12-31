# Day 279: Order Execution Emulation

## Trading Analogy

Imagine you're developing a trading strategy and want to test how it would have performed on historical data. But there's a problem: in reality, when you send an order to an exchange, many things happen — network delays, price slippage, partial fills, order queue position. If in backtesting you simply "execute" orders instantly at the desired price, the results will be unrealistically good.

**Order execution emulation** is the simulation of how orders would be executed under real market conditions. This is a key component of quality backtesting that makes the difference between "paper profits" and real results.

Key factors to emulate:
- **Slippage** — the difference between expected and actual execution price
- **Partial Fill** — when order size exceeds available liquidity
- **Latency** — time between order submission and execution
- **Queue Position** — for limit orders, it matters who was first in line

## Order Execution Models

### 1. Naive Model (Instant Execution)

The simplest but unrealistic model:

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>, // None for market orders
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
struct Fill {
    order_id: u64,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

// Naive model — just execute at current price
fn naive_execute(order: &Order, current_price: f64) -> Fill {
    Fill {
        order_id: order.id,
        price: current_price, // Unrealistic!
        quantity: order.quantity,
        timestamp: 0,
    }
}

fn main() {
    let order = Order {
        id: 1,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 10.0,
        price: None,
    };

    let fill = naive_execute(&order, 42000.0);
    println!("Naive execution: {:?}", fill);
    // Problem: in reality, buying 10 BTC would move the price!
}
```

### 2. Slippage Model

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
struct Fill {
    order_id: u64,
    price: f64,
    quantity: f64,
    slippage: f64,
}

#[derive(Debug, Clone)]
struct SlippageModel {
    // Base slippage in percentage
    base_slippage_pct: f64,
    // Additional slippage per N volume units
    volume_impact_pct: f64,
    volume_unit: f64,
}

impl SlippageModel {
    fn new(base_slippage: f64, volume_impact: f64, volume_unit: f64) -> Self {
        SlippageModel {
            base_slippage_pct: base_slippage,
            volume_impact_pct: volume_impact,
            volume_unit: volume_unit,
        }
    }

    fn calculate_slippage(&self, order: &Order) -> f64 {
        let volume_multiplier = order.quantity / self.volume_unit;
        let total_slippage = self.base_slippage_pct +
            (volume_multiplier * self.volume_impact_pct);

        // Slippage is always against us
        match order.side {
            OrderSide::Buy => total_slippage,   // Buy higher
            OrderSide::Sell => -total_slippage, // Sell lower
        }
    }

    fn execute(&self, order: &Order, mid_price: f64) -> Fill {
        let slippage_pct = self.calculate_slippage(order);
        let execution_price = mid_price * (1.0 + slippage_pct / 100.0);

        Fill {
            order_id: order.id,
            price: execution_price,
            quantity: order.quantity,
            slippage: slippage_pct,
        }
    }
}

fn main() {
    // Model: 0.05% base slippage + 0.01% per 1 BTC
    let slippage_model = SlippageModel::new(0.05, 0.01, 1.0);

    let small_order = Order {
        id: 1,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 0.1, // Small order
        price: None,
    };

    let large_order = Order {
        id: 2,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 50.0, // Large order
        price: None,
    };

    let mid_price = 42000.0;

    let fill_small = slippage_model.execute(&small_order, mid_price);
    let fill_large = slippage_model.execute(&large_order, mid_price);

    println!("Small order (0.1 BTC):");
    println!("  Execution price: ${:.2}", fill_small.price);
    println!("  Slippage: {:.4}%", fill_small.slippage);

    println!("\nLarge order (50 BTC):");
    println!("  Execution price: ${:.2}", fill_large.price);
    println!("  Slippage: {:.4}%", fill_large.slippage);
}
```

## Order Book Emulation

For more realistic simulation, we need to model the order book:

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone)]
struct PriceLevel {
    price: f64,
    quantity: f64,
    order_count: u32,
}

#[derive(Debug, Clone)]
struct OrderBook {
    // Bids sorted by descending price (best price = highest)
    bids: BTreeMap<i64, PriceLevel>, // Using i64 as key (price * 100)
    // Asks sorted by ascending price (best price = lowest)
    asks: BTreeMap<i64, PriceLevel>,
    tick_size: f64,
}

#[derive(Debug, Clone)]
struct ExecutionResult {
    fills: Vec<(f64, f64)>, // (price, quantity)
    average_price: f64,
    total_quantity: f64,
    unfilled_quantity: f64,
}

impl OrderBook {
    fn new(tick_size: f64) -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            tick_size,
        }
    }

    fn price_to_key(&self, price: f64) -> i64 {
        (price / self.tick_size).round() as i64
    }

    fn add_level(&mut self, side: OrderSide, price: f64, quantity: f64, order_count: u32) {
        let key = self.price_to_key(price);
        let level = PriceLevel {
            price,
            quantity,
            order_count,
        };

        match side {
            OrderSide::Buy => {
                self.bids.insert(-key, level); // Negative key for descending sort
            }
            OrderSide::Sell => {
                self.asks.insert(key, level);
            }
        }
    }

    fn best_bid(&self) -> Option<f64> {
        self.bids.values().next().map(|l| l.price)
    }

    fn best_ask(&self) -> Option<f64> {
        self.asks.values().next().map(|l| l.price)
    }

    fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    // Simulate market order execution
    fn execute_market_order(&self, side: OrderSide, quantity: f64) -> ExecutionResult {
        let mut fills = Vec::new();
        let mut remaining = quantity;
        let mut total_cost = 0.0;

        // Select the appropriate side of the book
        let levels: Vec<&PriceLevel> = match side {
            OrderSide::Buy => self.asks.values().collect(),
            OrderSide::Sell => self.bids.values().collect(),
        };

        for level in levels {
            if remaining <= 0.0 {
                break;
            }

            let fill_qty = remaining.min(level.quantity);
            fills.push((level.price, fill_qty));
            total_cost += level.price * fill_qty;
            remaining -= fill_qty;
        }

        let total_filled = quantity - remaining;
        let average_price = if total_filled > 0.0 {
            total_cost / total_filled
        } else {
            0.0
        };

        ExecutionResult {
            fills,
            average_price,
            total_quantity: total_filled,
            unfilled_quantity: remaining,
        }
    }
}

fn main() {
    let mut order_book = OrderBook::new(0.01);

    // Add levels to the order book (typical BTC/USDT book)
    order_book.add_level(OrderSide::Sell, 42010.0, 2.5, 15);
    order_book.add_level(OrderSide::Sell, 42005.0, 1.2, 8);
    order_book.add_level(OrderSide::Sell, 42001.0, 0.5, 3);

    order_book.add_level(OrderSide::Buy, 41999.0, 0.8, 5);
    order_book.add_level(OrderSide::Buy, 41995.0, 1.5, 10);
    order_book.add_level(OrderSide::Buy, 41990.0, 3.0, 20);

    println!("=== Order Book ===");
    println!("Best bid: ${:.2}", order_book.best_bid().unwrap());
    println!("Best ask: ${:.2}", order_book.best_ask().unwrap());
    println!("Spread: ${:.2}", order_book.spread().unwrap());

    // Simulate buying 2 BTC
    println!("\n=== Buying 2 BTC with market order ===");
    let result = order_book.execute_market_order(OrderSide::Buy, 2.0);

    println!("Fills:");
    for (price, qty) in &result.fills {
        println!("  {:.4} BTC @ ${:.2}", qty, price);
    }
    println!("Average price: ${:.2}", result.average_price);
    println!("Filled: {:.4} BTC", result.total_quantity);

    // Simulate buying 10 BTC (more than liquidity)
    println!("\n=== Buying 10 BTC with market order ===");
    let result_large = order_book.execute_market_order(OrderSide::Buy, 10.0);

    println!("Fills:");
    for (price, qty) in &result_large.fills {
        println!("  {:.4} BTC @ ${:.2}", qty, price);
    }
    println!("Average price: ${:.2}", result_large.average_price);
    println!("Filled: {:.4} BTC", result_large.total_quantity);
    println!("Unfilled: {:.4} BTC", result_large.unfilled_quantity);
}
```

## Limit Order Emulation

Limit orders require special logic — they execute only when the price reaches the specified level:

```rust
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
struct LimitOrder {
    id: u64,
    side: OrderSide,
    price: f64,
    quantity: f64,
    filled_quantity: f64,
    timestamp: u64,
    status: OrderStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone)]
struct MarketTick {
    timestamp: u64,
    bid: f64,
    ask: f64,
    last_price: f64,
    volume: f64,
}

#[derive(Debug, Clone)]
struct Fill {
    order_id: u64,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct LimitOrderEmulator {
    pending_orders: HashMap<u64, LimitOrder>,
    fills: Vec<Fill>,
    fill_probability: f64, // Probability of fill when price is touched
    next_order_id: u64,
}

impl LimitOrderEmulator {
    fn new(fill_probability: f64) -> Self {
        LimitOrderEmulator {
            pending_orders: HashMap::new(),
            fills: Vec::new(),
            fill_probability,
            next_order_id: 1,
        }
    }

    fn place_order(&mut self, side: OrderSide, price: f64, quantity: f64, timestamp: u64) -> u64 {
        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let order = LimitOrder {
            id: order_id,
            side,
            price,
            quantity,
            filled_quantity: 0.0,
            timestamp,
            status: OrderStatus::Pending,
        };

        self.pending_orders.insert(order_id, order);
        order_id
    }

    fn cancel_order(&mut self, order_id: u64) -> bool {
        if let Some(order) = self.pending_orders.get_mut(&order_id) {
            order.status = OrderStatus::Cancelled;
            self.pending_orders.remove(&order_id);
            true
        } else {
            false
        }
    }

    fn process_tick(&mut self, tick: &MarketTick) -> Vec<Fill> {
        let mut new_fills = Vec::new();
        let mut orders_to_remove = Vec::new();

        for (order_id, order) in self.pending_orders.iter_mut() {
            let should_fill = match order.side {
                // Buy executes when ask <= order price
                OrderSide::Buy => tick.ask <= order.price,
                // Sell executes when bid >= order price
                OrderSide::Sell => tick.bid >= order.price,
            };

            if should_fill {
                // Simulate fill probability
                // In reality this depends on queue position
                let random_value = (tick.timestamp % 100) as f64 / 100.0;

                if random_value < self.fill_probability {
                    // Determine fill volume based on tick volume
                    let available_volume = tick.volume * 0.1; // Assume 10% volume available
                    let fill_quantity = (order.quantity - order.filled_quantity)
                        .min(available_volume);

                    if fill_quantity > 0.0 {
                        let fill = Fill {
                            order_id: *order_id,
                            price: order.price,
                            quantity: fill_quantity,
                            timestamp: tick.timestamp,
                        };

                        order.filled_quantity += fill_quantity;
                        new_fills.push(fill);

                        if order.filled_quantity >= order.quantity {
                            order.status = OrderStatus::Filled;
                            orders_to_remove.push(*order_id);
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
                        }
                    }
                }
            }
        }

        // Remove fully filled orders
        for order_id in orders_to_remove {
            self.pending_orders.remove(&order_id);
        }

        self.fills.extend(new_fills.clone());
        new_fills
    }

    fn get_order_status(&self, order_id: u64) -> Option<&LimitOrder> {
        self.pending_orders.get(&order_id)
    }
}

fn main() {
    let mut emulator = LimitOrderEmulator::new(0.7); // 70% fill probability

    // Place limit orders
    let buy_order = emulator.place_order(OrderSide::Buy, 41500.0, 1.0, 0);
    let sell_order = emulator.place_order(OrderSide::Sell, 42500.0, 0.5, 0);

    println!("Orders placed:");
    println!("  Buy: id={}, 1 BTC @ $41,500", buy_order);
    println!("  Sell: id={}, 0.5 BTC @ $42,500", sell_order);

    // Simulate market ticks
    let ticks = vec![
        MarketTick { timestamp: 1, bid: 42000.0, ask: 42010.0, last_price: 42005.0, volume: 5.0 },
        MarketTick { timestamp: 2, bid: 41600.0, ask: 41610.0, last_price: 41605.0, volume: 3.0 },
        MarketTick { timestamp: 3, bid: 41480.0, ask: 41490.0, last_price: 41485.0, volume: 8.0 },
        MarketTick { timestamp: 4, bid: 41700.0, ask: 41710.0, last_price: 41705.0, volume: 2.0 },
        MarketTick { timestamp: 5, bid: 42400.0, ask: 42410.0, last_price: 42405.0, volume: 4.0 },
        MarketTick { timestamp: 6, bid: 42510.0, ask: 42520.0, last_price: 42515.0, volume: 6.0 },
    ];

    println!("\nProcessing market ticks:");
    for tick in &ticks {
        let fills = emulator.process_tick(tick);

        println!("\nTick {}: bid=${:.0}, ask=${:.0}, volume={:.1}",
                 tick.timestamp, tick.bid, tick.ask, tick.volume);

        for fill in fills {
            println!("  >>> FILL: order {}, {:.4} @ ${:.2}",
                     fill.order_id, fill.quantity, fill.price);
        }
    }

    println!("\n=== Final Report ===");
    println!("Total fills: {}", emulator.fills.len());
    for fill in &emulator.fills {
        println!("  Order {}: {:.4} @ ${:.2} (t={})",
                 fill.order_id, fill.quantity, fill.price, fill.timestamp);
    }
}
```

## Complete Execution Emulation System

Let's combine all components into a unified system:

```rust
use std::collections::HashMap;

// ============ Data Types ============

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Rejected,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: f64,
    limit_price: Option<f64>,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct Fill {
    order_id: u64,
    price: f64,
    quantity: f64,
    commission: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct OHLCV {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

// ============ Emulation Configuration ============

#[derive(Debug, Clone)]
struct ExecutionConfig {
    // Slippage
    base_slippage_bps: f64,      // In basis points (1 bp = 0.01%)
    volume_impact_bps: f64,      // Additional slippage per volume

    // Commissions
    maker_fee_pct: f64,          // Maker commission
    taker_fee_pct: f64,          // Taker commission

    // Limit order execution
    limit_fill_probability: f64, // Probability of limit order fill
    partial_fill_enabled: bool,  // Allow partial fills

    // Latency
    execution_delay_ms: u64,     // Execution delay in ms
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig {
            base_slippage_bps: 5.0,        // 0.05%
            volume_impact_bps: 1.0,        // 0.01% per unit
            maker_fee_pct: 0.02,           // 0.02%
            taker_fee_pct: 0.04,           // 0.04%
            limit_fill_probability: 0.8,
            partial_fill_enabled: true,
            execution_delay_ms: 50,
        }
    }
}

// ============ Emulation Engine ============

#[derive(Debug)]
struct ExecutionEmulator {
    config: ExecutionConfig,
    pending_orders: HashMap<u64, Order>,
    fills: Vec<Fill>,
    next_order_id: u64,
}

impl ExecutionEmulator {
    fn new(config: ExecutionConfig) -> Self {
        ExecutionEmulator {
            config,
            pending_orders: HashMap::new(),
            fills: Vec::new(),
            next_order_id: 1,
        }
    }

    fn submit_order(&mut self, order: Order) -> u64 {
        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let mut order = order;
        order.id = order_id;

        self.pending_orders.insert(order_id, order);
        order_id
    }

    fn calculate_slippage(&self, order: &Order, base_price: f64) -> f64 {
        let slippage_bps = self.config.base_slippage_bps +
            (order.quantity * self.config.volume_impact_bps);

        let slippage_multiplier = slippage_bps / 10000.0;

        match order.side {
            OrderSide::Buy => base_price * (1.0 + slippage_multiplier),
            OrderSide::Sell => base_price * (1.0 - slippage_multiplier),
        }
    }

    fn calculate_commission(&self, order: &Order, fill_price: f64, quantity: f64) -> f64 {
        let fee_pct = match order.order_type {
            OrderType::Market => self.config.taker_fee_pct,
            OrderType::Limit => self.config.maker_fee_pct,
        };

        fill_price * quantity * (fee_pct / 100.0)
    }

    fn process_candle(&mut self, candle: &OHLCV) -> Vec<Fill> {
        let mut new_fills = Vec::new();
        let mut orders_to_remove = Vec::new();

        for (order_id, order) in self.pending_orders.iter() {
            match order.order_type {
                OrderType::Market => {
                    // Market orders execute immediately
                    let execution_price = self.calculate_slippage(order, candle.open);
                    let commission = self.calculate_commission(order, execution_price, order.quantity);

                    new_fills.push(Fill {
                        order_id: *order_id,
                        price: execution_price,
                        quantity: order.quantity,
                        commission,
                        timestamp: candle.timestamp + self.config.execution_delay_ms,
                    });

                    orders_to_remove.push(*order_id);
                }

                OrderType::Limit => {
                    if let Some(limit_price) = order.limit_price {
                        let price_touched = match order.side {
                            OrderSide::Buy => candle.low <= limit_price,
                            OrderSide::Sell => candle.high >= limit_price,
                        };

                        if price_touched {
                            // Check fill probability
                            let random = ((candle.timestamp * 7 + *order_id * 13) % 100) as f64 / 100.0;

                            if random < self.config.limit_fill_probability {
                                let fill_quantity = if self.config.partial_fill_enabled {
                                    // Partial fill based on candle volume
                                    order.quantity.min(candle.volume * 0.1)
                                } else {
                                    order.quantity
                                };

                                let commission = self.calculate_commission(order, limit_price, fill_quantity);

                                new_fills.push(Fill {
                                    order_id: *order_id,
                                    price: limit_price,
                                    quantity: fill_quantity,
                                    commission,
                                    timestamp: candle.timestamp + self.config.execution_delay_ms,
                                });

                                if fill_quantity >= order.quantity {
                                    orders_to_remove.push(*order_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        for order_id in orders_to_remove {
            self.pending_orders.remove(&order_id);
        }

        self.fills.extend(new_fills.clone());
        new_fills
    }

    fn get_statistics(&self) -> ExecutionStatistics {
        let total_volume: f64 = self.fills.iter().map(|f| f.quantity * f.price).sum();
        let total_commission: f64 = self.fills.iter().map(|f| f.commission).sum();

        ExecutionStatistics {
            total_fills: self.fills.len(),
            total_volume,
            total_commission,
            average_fill_size: if self.fills.is_empty() {
                0.0
            } else {
                total_volume / self.fills.len() as f64
            },
        }
    }
}

#[derive(Debug)]
struct ExecutionStatistics {
    total_fills: usize,
    total_volume: f64,
    total_commission: f64,
    average_fill_size: f64,
}

fn main() {
    let config = ExecutionConfig {
        base_slippage_bps: 5.0,
        volume_impact_bps: 2.0,
        maker_fee_pct: 0.02,
        taker_fee_pct: 0.04,
        limit_fill_probability: 0.75,
        partial_fill_enabled: true,
        execution_delay_ms: 100,
    };

    let mut emulator = ExecutionEmulator::new(config);

    // Submit orders
    let market_buy = Order {
        id: 0,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: 0.5,
        limit_price: None,
        timestamp: 0,
    };

    let limit_sell = Order {
        id: 0,
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Sell,
        order_type: OrderType::Limit,
        quantity: 0.5,
        limit_price: Some(43000.0),
        timestamp: 0,
    };

    let id1 = emulator.submit_order(market_buy);
    let id2 = emulator.submit_order(limit_sell);

    println!("Orders submitted: Market Buy (id={}), Limit Sell (id={})", id1, id2);

    // Simulate candles
    let candles = vec![
        OHLCV { timestamp: 1000, open: 42000.0, high: 42500.0, low: 41800.0, close: 42200.0, volume: 100.0 },
        OHLCV { timestamp: 2000, open: 42200.0, high: 42800.0, low: 42100.0, close: 42700.0, volume: 80.0 },
        OHLCV { timestamp: 3000, open: 42700.0, high: 43200.0, low: 42600.0, close: 43100.0, volume: 120.0 },
    ];

    println!("\n=== Processing Candles ===");
    for candle in &candles {
        println!("\nCandle: O={:.0} H={:.0} L={:.0} C={:.0} V={:.0}",
                 candle.open, candle.high, candle.low, candle.close, candle.volume);

        let fills = emulator.process_candle(candle);
        for fill in fills {
            println!("  FILL: order {} | {:.4} @ ${:.2} | commission: ${:.4}",
                     fill.order_id, fill.quantity, fill.price, fill.commission);
        }
    }

    let stats = emulator.get_statistics();
    println!("\n=== Execution Statistics ===");
    println!("Total fills: {}", stats.total_fills);
    println!("Total volume: ${:.2}", stats.total_volume);
    println!("Total commission: ${:.4}", stats.total_commission);
    println!("Average fill size: ${:.2}", stats.average_fill_size);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Slippage | Difference between expected and actual execution price |
| Partial Fill | When an order doesn't fully execute due to insufficient liquidity |
| Order Book | Model for simulating execution across multiple price levels |
| Limit Orders | Require checking price touch and queue position |
| Commissions | Maker/Taker fee model of exchanges |
| Execution Latency | Time between order submission and execution |

## Practical Exercises

### Exercise 1: Market Impact Model
Implement a model where large orders impact the price:
```rust
// Hint: use the square root formula
// impact = sqrt(order_size / average_volume) * impact_coefficient
```

### Exercise 2: Queue Emulation
Add queue simulation for limit orders — orders placed earlier get filled first.

### Exercise 3: Stop Orders
Implement support for stop orders (Stop-Loss and Take-Profit) that convert to market orders when the trigger price is reached.

## Homework

1. **Realistic Order Book**: Create a synthetic order book generator with realistic liquidity distribution (more liquidity near current price, less further away).

2. **Model Comparison**: Run the same strategy through different execution models (naive, with slippage, with full order book) and compare PnL results.

3. **Latency Arbitrage**: Simulate a situation where execution latency affects strategy profitability. Find the latency threshold at which the strategy becomes unprofitable.

4. **Fill Ratio Analysis**: Implement tracking and analysis of fill ratio (what percentage of limit orders get filled) for different distances from current price.

## Navigation

[← Previous day](../278-backtesting-framework-core/en.md) | [Next day →](../280-slippage-models-realistic-fills/en.md)
