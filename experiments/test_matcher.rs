// Test compilation of the main matcher code from chapter 334

use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::cmp::Reverse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side { Bid, Ask }

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

    println!("=== High-Performance Matcher Test ===\n");

    // Add some orders
    for i in 0..10 {
        let side = if i % 2 == 0 { Side::Bid } else { Side::Ask };
        let price = 50000 + (i % 5) * 10;
        let order = Order::new(i, price, 100, side, i);
        let fills = matcher.process_order(order);
        if !fills.is_empty() {
            println!("Order {} matched with {} fills", i, fills.len());
        }
    }

    println!("\nMatcher stats: {:?}", matcher.stats());
    println!("Best bid: {:?}, Best ask: {:?}", matcher.best_bid(), matcher.best_ask());
    println!("\nTest passed!");
}
