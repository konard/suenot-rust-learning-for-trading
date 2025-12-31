# Day 57: RefCell — Mutate Through Immutable Reference

## Trading Analogy

Imagine a trading terminal at a brokerage firm. The terminal is installed on a trader's workstation and **appears as a regular application** (immutable reference — everyone sees the same interface). However, **inside** the terminal, constant updates are happening: quotes change, positions open and close, balances update.

From the outside, the terminal looks "immutable" — it's just a window on the screen. But **internal mutation** happens constantly. This is the essence of `RefCell` — the ability to mutate data through an immutable reference, checking borrowing rules **at runtime** rather than at compile time.

## The Problem: Is Rust Too Strict?

Regular Rust borrowing rules are checked **at compile time**:
- Either one mutable reference (`&mut`)
- Or many immutable references (`&`)
- But not both at the same time

Sometimes these rules are too strict. For example:

```rust
// This will NOT compile!
struct Portfolio {
    balance: f64,
}

impl Portfolio {
    fn get_balance(&self) -> f64 {
        self.balance
    }

    fn update_balance(&self, new_balance: f64) {
        // Error! self is an immutable reference
        // self.balance = new_balance;
    }
}
```

## What is RefCell?

`RefCell<T>` is a smart pointer that:
1. Stores data of type `T`
2. Allows **dynamic** borrowing of data
3. Checks borrowing rules **at runtime**
4. Panics when rules are violated

```rust
use std::cell::RefCell;

fn main() {
    let price = RefCell::new(42000.0);

    // Immutable borrow
    println!("Price: {}", *price.borrow());

    // Mutable borrow
    *price.borrow_mut() += 100.0;

    println!("New price: {}", *price.borrow());
}
```

## Main RefCell Methods

```rust
use std::cell::RefCell;

fn main() {
    let balance = RefCell::new(10000.0);

    // borrow() — immutable borrow (Ref<T>)
    {
        let b = balance.borrow();
        println!("Balance: {}", *b);
    } // Ref goes out of scope

    // borrow_mut() — mutable borrow (RefMut<T>)
    {
        let mut b = balance.borrow_mut();
        *b += 500.0;
        println!("New balance: {}", *b);
    } // RefMut goes out of scope

    // into_inner() — extract the value
    let final_balance = balance.into_inner();
    println!("Final balance: {}", final_balance);
}
```

## Panic on Rule Violation

```rust
use std::cell::RefCell;

fn main() {
    let price = RefCell::new(42000.0);

    let borrow1 = price.borrow();

    // PANIC! Already have an immutable borrow
    // let borrow2 = price.borrow_mut();

    println!("Price: {}", *borrow1);

    // After borrow1 goes out of scope,
    // we can borrow again
    drop(borrow1);

    let mut borrow2 = price.borrow_mut();
    *borrow2 = 43000.0;
    println!("New price: {}", *borrow2);
}
```

## Safe Check: try_borrow

```rust
use std::cell::RefCell;

fn main() {
    let balance = RefCell::new(10000.0);

    let borrow1 = balance.borrow();

    // try_borrow_mut returns Err instead of panicking
    match balance.try_borrow_mut() {
        Ok(mut b) => {
            *b += 100.0;
        }
        Err(_) => {
            println!("Could not get mutable reference — data already borrowed");
        }
    }

    println!("Balance: {}", *borrow1);
}
```

## Practical Example: Trading Account

```rust
use std::cell::RefCell;

struct TradingAccount {
    name: String,
    balance: RefCell<f64>,
    open_positions: RefCell<Vec<Position>>,
}

#[derive(Clone, Debug)]
struct Position {
    symbol: String,
    size: f64,
    entry_price: f64,
}

impl TradingAccount {
    fn new(name: &str, initial_balance: f64) -> Self {
        TradingAccount {
            name: name.to_string(),
            balance: RefCell::new(initial_balance),
            open_positions: RefCell::new(Vec::new()),
        }
    }

    // &self — immutable reference, but we can modify balance
    fn deposit(&self, amount: f64) {
        *self.balance.borrow_mut() += amount;
        println!("Deposit: +${:.2}", amount);
    }

    fn withdraw(&self, amount: f64) -> bool {
        let mut balance = self.balance.borrow_mut();
        if *balance >= amount {
            *balance -= amount;
            println!("Withdrawal: -${:.2}", amount);
            true
        } else {
            println!("Insufficient funds!");
            false
        }
    }

    fn open_position(&self, symbol: &str, size: f64, price: f64) {
        let cost = size * price;

        // First check balance
        {
            let balance = self.balance.borrow();
            if *balance < cost {
                println!("Insufficient funds to open position");
                return;
            }
        }

        // Now modify balance and add position
        *self.balance.borrow_mut() -= cost;

        self.open_positions.borrow_mut().push(Position {
            symbol: symbol.to_string(),
            size,
            entry_price: price,
        });

        println!("Opened position: {} x {} @ {}", symbol, size, price);
    }

    fn get_balance(&self) -> f64 {
        *self.balance.borrow()
    }

    fn print_status(&self) {
        println!("\n=== {} ===", self.name);
        println!("Balance: ${:.2}", self.balance.borrow());
        println!("Open positions:");
        for pos in self.open_positions.borrow().iter() {
            println!("  {} x {} @ ${:.2}", pos.symbol, pos.size, pos.entry_price);
        }
    }
}

fn main() {
    let account = TradingAccount::new("My Account", 100000.0);

    account.print_status();

    account.deposit(5000.0);
    account.open_position("BTC/USDT", 0.5, 42000.0);
    account.open_position("ETH/USDT", 2.0, 2500.0);

    account.print_status();

    println!("\nCurrent balance: ${:.2}", account.get_balance());
}
```

## RefCell + Rc: Shared Mutable Data

The combination `Rc<RefCell<T>>` allows having **multiple owners** of mutable data:

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct OrderBook {
    bids: Vec<(f64, f64)>,  // (price, size)
    asks: Vec<(f64, f64)>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    fn add_bid(&mut self, price: f64, size: f64) {
        self.bids.push((price, size));
        self.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    }

    fn add_ask(&mut self, price: f64, size: f64) {
        self.asks.push((price, size));
        self.asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids.first().copied()
    }

    fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks.first().copied()
    }

    fn spread(&self) -> Option<f64> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.0 - bid.0),
            _ => None,
        }
    }
}

struct MarketDataFeed {
    order_book: Rc<RefCell<OrderBook>>,
}

struct TradingStrategy {
    order_book: Rc<RefCell<OrderBook>>,
    name: String,
}

impl MarketDataFeed {
    fn new(order_book: Rc<RefCell<OrderBook>>) -> Self {
        MarketDataFeed { order_book }
    }

    fn update_bid(&self, price: f64, size: f64) {
        self.order_book.borrow_mut().add_bid(price, size);
    }

    fn update_ask(&self, price: f64, size: f64) {
        self.order_book.borrow_mut().add_ask(price, size);
    }
}

impl TradingStrategy {
    fn new(name: &str, order_book: Rc<RefCell<OrderBook>>) -> Self {
        TradingStrategy {
            order_book,
            name: name.to_string(),
        }
    }

    fn analyze(&self) {
        let book = self.order_book.borrow();
        println!("\n[{}] Order book analysis:", self.name);

        if let Some((price, size)) = book.best_bid() {
            println!("  Best bid: {} x {}", price, size);
        }
        if let Some((price, size)) = book.best_ask() {
            println!("  Best ask: {} x {}", price, size);
        }
        if let Some(spread) = book.spread() {
            println!("  Spread: {:.2}", spread);
        }
    }
}

fn main() {
    // Shared order book
    let order_book = Rc::new(RefCell::new(OrderBook::new()));

    // Create data feed and strategies — all use the same order book
    let feed = MarketDataFeed::new(Rc::clone(&order_book));
    let strategy1 = TradingStrategy::new("Arbitrage", Rc::clone(&order_book));
    let strategy2 = TradingStrategy::new("Market-making", Rc::clone(&order_book));

    // Feed updates the order book
    feed.update_bid(42000.0, 1.5);
    feed.update_bid(41990.0, 2.0);
    feed.update_bid(41980.0, 3.5);

    feed.update_ask(42010.0, 1.0);
    feed.update_ask(42020.0, 2.5);
    feed.update_ask(42030.0, 1.8);

    // Strategies analyze the same data
    strategy1.analyze();
    strategy2.analyze();

    // Add more data
    feed.update_bid(42005.0, 0.5);

    println!("\n--- After update ---");
    strategy1.analyze();
}
```

## Practical Example: Market Data Cache

```rust
use std::cell::RefCell;
use std::collections::HashMap;

struct MarketDataCache {
    prices: RefCell<HashMap<String, f64>>,
    access_count: RefCell<u32>,
}

impl MarketDataCache {
    fn new() -> Self {
        MarketDataCache {
            prices: RefCell::new(HashMap::new()),
            access_count: RefCell::new(0),
        }
    }

    // &self — can be called through an immutable reference
    fn update_price(&self, symbol: &str, price: f64) {
        self.prices.borrow_mut().insert(symbol.to_string(), price);
    }

    fn get_price(&self, symbol: &str) -> Option<f64> {
        *self.access_count.borrow_mut() += 1;
        self.prices.borrow().get(symbol).copied()
    }

    fn get_access_count(&self) -> u32 {
        *self.access_count.borrow()
    }

    fn print_all_prices(&self) {
        println!("\n=== Market Data Cache ===");
        println!("Cache accesses: {}", self.access_count.borrow());
        for (symbol, price) in self.prices.borrow().iter() {
            println!("  {}: ${:.2}", symbol, price);
        }
    }
}

fn main() {
    let cache = MarketDataCache::new();

    // Update prices
    cache.update_price("BTC/USDT", 42000.0);
    cache.update_price("ETH/USDT", 2500.0);
    cache.update_price("SOL/USDT", 95.0);

    // Get prices (counter increments)
    if let Some(btc) = cache.get_price("BTC/USDT") {
        println!("BTC: ${:.2}", btc);
    }

    if let Some(eth) = cache.get_price("ETH/USDT") {
        println!("ETH: ${:.2}", eth);
    }

    // Update BTC price
    cache.update_price("BTC/USDT", 42500.0);

    if let Some(btc) = cache.get_price("BTC/USDT") {
        println!("BTC (updated): ${:.2}", btc);
    }

    cache.print_all_prices();
}
```

## Practical Example: Trade Journal

```rust
use std::cell::RefCell;

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    size: f64,
    pnl: Option<f64>,
}

struct TradeJournal {
    trades: RefCell<Vec<Trade>>,
    next_id: RefCell<u64>,
}

impl TradeJournal {
    fn new() -> Self {
        TradeJournal {
            trades: RefCell::new(Vec::new()),
            next_id: RefCell::new(1),
        }
    }

    fn record_trade(&self, symbol: &str, side: &str, price: f64, size: f64) -> u64 {
        let mut next_id = self.next_id.borrow_mut();
        let id = *next_id;
        *next_id += 1;

        let trade = Trade {
            id,
            symbol: symbol.to_string(),
            side: side.to_string(),
            price,
            size,
            pnl: None,
        };

        self.trades.borrow_mut().push(trade);

        println!("Recorded trade #{}: {} {} @ {:.2}", id, side, symbol, price);
        id
    }

    fn set_pnl(&self, trade_id: u64, pnl: f64) {
        let mut trades = self.trades.borrow_mut();
        if let Some(trade) = trades.iter_mut().find(|t| t.id == trade_id) {
            trade.pnl = Some(pnl);
            println!("Set PnL for trade #{}: {:+.2}", trade_id, pnl);
        }
    }

    fn total_pnl(&self) -> f64 {
        self.trades
            .borrow()
            .iter()
            .filter_map(|t| t.pnl)
            .sum()
    }

    fn print_summary(&self) {
        let trades = self.trades.borrow();

        println!("\n=== Trade Journal ===");
        println!("Total trades: {}", trades.len());

        for trade in trades.iter() {
            let pnl_str = match trade.pnl {
                Some(pnl) => format!("{:+.2}", pnl),
                None => "N/A".to_string(),
            };
            println!(
                "  #{}: {} {} x {} @ {:.2} | PnL: {}",
                trade.id, trade.side, trade.symbol, trade.size, trade.price, pnl_str
            );
        }

        println!("Total PnL: {:+.2}", self.total_pnl());
    }
}

fn main() {
    let journal = TradeJournal::new();

    // Record trades
    let trade1 = journal.record_trade("BTC/USDT", "BUY", 42000.0, 0.5);
    let trade2 = journal.record_trade("ETH/USDT", "BUY", 2500.0, 2.0);
    let trade3 = journal.record_trade("BTC/USDT", "SELL", 42500.0, 0.5);

    // Set PnL
    journal.set_pnl(trade1, 250.0);
    journal.set_pnl(trade2, -100.0);
    journal.set_pnl(trade3, 0.0);  // This was the closing trade

    journal.print_summary();
}
```

## Cell vs RefCell

| Characteristic | Cell<T> | RefCell<T> |
|---------------|---------|------------|
| Copy requirement | Requires `Copy` | Any type |
| Data access | Copies the value | Returns a reference |
| Checking | No checks | Runtime check |
| Panic | Never panics | Can panic |
| Use case | Simple types (i32, f64) | Complex types (Vec, String) |

```rust
use std::cell::{Cell, RefCell};

fn main() {
    // Cell — for Copy types
    let price = Cell::new(42000.0);
    price.set(42100.0);
    println!("Price: {}", price.get());

    // RefCell — for non-Copy types
    let prices = RefCell::new(vec![42000.0, 42100.0]);
    prices.borrow_mut().push(42200.0);
    println!("Prices: {:?}", prices.borrow());
}
```

## When to Use RefCell

**Use RefCell when:**
- You need interior mutability
- Borrowing rules cannot be checked at compile time
- Data is modified through immutable references
- You need to modify struct fields in methods with `&self`

**Don't use RefCell when:**
- Regular `&mut` is sufficient
- Working in a multithreaded context (use `Mutex`)
- Simple Copy types (use `Cell`)

## What We Learned

| Concept | Description |
|---------|-------------|
| `RefCell<T>` | Container with runtime borrow checking |
| `borrow()` | Immutable borrow → `Ref<T>` |
| `borrow_mut()` | Mutable borrow → `RefMut<T>` |
| `try_borrow()` | Safe borrow attempt |
| `Rc<RefCell<T>>` | Shared mutable data |

## Homework

1. Create a `Portfolio` struct with `RefCell<HashMap<String, Position>>` to store positions. Implement `add_position`, `update_position`, and `get_total_value` methods using `&self`.

2. Implement a price update subscription system: multiple `TradingStrategy` instances use one `Rc<RefCell<MarketData>>`. When data is updated, all strategies should have access to current prices.

3. Create a `TransactionLog` with `RefCell<Vec<Transaction>>`. Implement methods for adding transactions, calculating balance, and rolling back the last transaction.

4. Write a trading bot with `RefCell` fields for balance, open orders, and history. The bot should be able to open/close positions, update stop-losses and take-profits through methods with `&self`.

## Navigation

[← Previous day](../056-cell-interior-mutability/en.md) | [Next day →](../058-mutex-thread-safe-interior-mutability/en.md)
