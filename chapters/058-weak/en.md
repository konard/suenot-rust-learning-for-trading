# Day 58: Weak — Weak References to Related Orders

## Trading Analogy

In trading, orders are often interconnected:
- **Main order** to buy BTC
- **Take Profit order** — automatically sells when target price is reached
- **Stop Loss order** — automatically sells when loss level is reached

These relationships create a potential problem: if the main order is executed and deleted, related orders should "know" about this and not try to access non-existent data.

**Weak (weak references)** solve this problem: they allow referencing data without preventing its deletion.

## The Problem of Circular References

```rust
use std::rc::Rc;
use std::cell::RefCell;

// ❌ PROBLEM: circular references with Rc
struct Order {
    id: String,
    price: f64,
    related_orders: Vec<Rc<RefCell<Order>>>,  // Related orders
}

fn main() {
    let order1 = Rc::new(RefCell::new(Order {
        id: "ORD-001".to_string(),
        price: 42000.0,
        related_orders: vec![],
    }));

    let order2 = Rc::new(RefCell::new(Order {
        id: "ORD-002".to_string(),
        price: 43000.0,
        related_orders: vec![],
    }));

    // Creating circular reference
    order1.borrow_mut().related_orders.push(Rc::clone(&order2));
    order2.borrow_mut().related_orders.push(Rc::clone(&order1));

    // ⚠️ Memory will never be freed!
    // Reference count of both orders will never reach 0
}
```

## Solution with Weak

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

// ✅ SOLUTION: weak references don't increase the count
struct Order {
    id: String,
    price: f64,
    quantity: f64,
    // Weak references to related orders
    take_profit: Option<Weak<RefCell<Order>>>,
    stop_loss: Option<Weak<RefCell<Order>>>,
    // Weak reference to parent order
    parent_order: Option<Weak<RefCell<Order>>>,
}

impl Order {
    fn new(id: &str, price: f64, quantity: f64) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Order {
            id: id.to_string(),
            price,
            quantity,
            take_profit: None,
            stop_loss: None,
            parent_order: None,
        }))
    }

    fn set_take_profit(&mut self, tp: &Rc<RefCell<Order>>) {
        self.take_profit = Some(Rc::downgrade(tp));
    }

    fn set_stop_loss(&mut self, sl: &Rc<RefCell<Order>>) {
        self.stop_loss = Some(Rc::downgrade(sl));
    }

    fn set_parent(&mut self, parent: &Rc<RefCell<Order>>) {
        self.parent_order = Some(Rc::downgrade(parent));
    }
}

fn main() {
    // Main buy order
    let main_order = Order::new("BUY-001", 42000.0, 0.5);

    // Take Profit order
    let tp_order = Order::new("TP-001", 45000.0, 0.5);

    // Stop Loss order
    let sl_order = Order::new("SL-001", 40000.0, 0.5);

    // Link orders
    {
        let mut main = main_order.borrow_mut();
        main.set_take_profit(&tp_order);
        main.set_stop_loss(&sl_order);
    }

    {
        tp_order.borrow_mut().set_parent(&main_order);
        sl_order.borrow_mut().set_parent(&main_order);
    }

    println!("Orders created and linked");
    println!("Main order refs: {}", Rc::strong_count(&main_order));
    println!("TP order refs: {}", Rc::strong_count(&tp_order));
    // Weak references don't increase the count!
}
```

## Rc::downgrade and Weak::upgrade

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Position {
    symbol: String,
    size: f64,
    entry_price: f64,
}

#[derive(Debug)]
struct RiskMonitor {
    // Weak reference to position
    position: Weak<RefCell<Position>>,
    max_loss_percent: f64,
}

impl RiskMonitor {
    fn new(position: &Rc<RefCell<Position>>, max_loss: f64) -> Self {
        RiskMonitor {
            // downgrade: Rc -> Weak
            position: Rc::downgrade(position),
            max_loss_percent: max_loss,
        }
    }

    fn check_risk(&self, current_price: f64) {
        // upgrade: Weak -> Option<Rc>
        match self.position.upgrade() {
            Some(pos) => {
                let position = pos.borrow();
                let pnl_percent = (current_price - position.entry_price)
                    / position.entry_price * 100.0;

                if pnl_percent < -self.max_loss_percent {
                    println!(
                        "⚠️ RISK ALERT: {} loss {:.2}% exceeds limit {:.1}%",
                        position.symbol, pnl_percent.abs(), self.max_loss_percent
                    );
                } else {
                    println!(
                        "✅ {} P&L: {:.2}% (limit: {:.1}%)",
                        position.symbol, pnl_percent, self.max_loss_percent
                    );
                }
            }
            None => {
                // Position already closed/deleted
                println!("Position no longer exists");
            }
        }
    }
}

fn main() {
    let position = Rc::new(RefCell::new(Position {
        symbol: "BTC/USDT".to_string(),
        size: 0.5,
        entry_price: 42000.0,
    }));

    let monitor = RiskMonitor::new(&position, 5.0);

    // Check risk at different prices
    monitor.check_risk(41000.0);  // -2.38%
    monitor.check_risk(39000.0);  // -7.14% - exceeds limit!

    // Delete position
    drop(position);

    // Monitor safely handles deleted position
    monitor.check_risk(40000.0);  // Position no longer exists
}
```

## Linked Orders System

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderType {
    Market,
    Limit,
    TakeProfit,
    StopLoss,
}

struct LinkedOrder {
    id: String,
    order_type: OrderType,
    symbol: String,
    price: f64,
    quantity: f64,
    status: OrderStatus,
    // Weak references to avoid cycles
    parent: Option<Weak<RefCell<LinkedOrder>>>,
    children: Vec<Weak<RefCell<LinkedOrder>>>,
}

impl LinkedOrder {
    fn new(id: &str, order_type: OrderType, symbol: &str, price: f64, quantity: f64) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(LinkedOrder {
            id: id.to_string(),
            order_type,
            symbol: symbol.to_string(),
            price,
            quantity,
            status: OrderStatus::Pending,
            parent: None,
            children: vec![],
        }))
    }

    fn link_child(&mut self, child: &Rc<RefCell<LinkedOrder>>) {
        self.children.push(Rc::downgrade(child));
    }

    fn set_parent(&mut self, parent: &Rc<RefCell<LinkedOrder>>) {
        self.parent = Some(Rc::downgrade(parent));
    }

    fn fill(&mut self) {
        self.status = OrderStatus::Filled;
        println!("✅ Order {} filled at ${:.2}", self.id, self.price);
    }

    fn cancel(&mut self) {
        self.status = OrderStatus::Cancelled;
        println!("❌ Order {} cancelled", self.id);
    }

    fn cancel_children(&self) {
        for weak_child in &self.children {
            if let Some(child) = weak_child.upgrade() {
                let mut child_order = child.borrow_mut();
                if child_order.status == OrderStatus::Pending {
                    child_order.cancel();
                }
            }
        }
    }

    fn check_parent_status(&self) -> Option<OrderStatus> {
        self.parent.as_ref().and_then(|weak_parent| {
            weak_parent.upgrade().map(|parent| {
                parent.borrow().status.clone()
            })
        })
    }
}

fn main() {
    println!("=== Linked Orders System ===\n");

    // Create main buy order
    let buy_order = LinkedOrder::new(
        "BUY-001",
        OrderType::Market,
        "BTC/USDT",
        42000.0,
        0.5
    );

    // Create Take Profit
    let tp_order = LinkedOrder::new(
        "TP-001",
        OrderType::TakeProfit,
        "BTC/USDT",
        45000.0,
        0.5
    );

    // Create Stop Loss
    let sl_order = LinkedOrder::new(
        "SL-001",
        OrderType::StopLoss,
        "BTC/USDT",
        40000.0,
        0.5
    );

    // Link orders
    {
        let mut buy = buy_order.borrow_mut();
        buy.link_child(&tp_order);
        buy.link_child(&sl_order);
    }
    tp_order.borrow_mut().set_parent(&buy_order);
    sl_order.borrow_mut().set_parent(&buy_order);

    // Main order filled
    buy_order.borrow_mut().fill();

    // Simulation: price reached Take Profit
    println!("\n--- Price reached Take Profit ---");
    tp_order.borrow_mut().fill();

    // When TP is filled, cancel SL
    buy_order.borrow().cancel_children();

    // Check parent order status from child
    if let Some(status) = sl_order.borrow().check_parent_status() {
        println!("\nParent order status: {:?}", status);
    }
}
```

## OCO (One-Cancels-Other) Orders

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
enum OcoStatus {
    Active,
    OneFilled,
    AllCancelled,
}

struct OcoOrder {
    id: String,
    take_profit: Option<Weak<RefCell<LimitOrder>>>,
    stop_loss: Option<Weak<RefCell<LimitOrder>>>,
    status: OcoStatus,
}

struct LimitOrder {
    id: String,
    price: f64,
    quantity: f64,
    is_filled: bool,
    oco_group: Option<Weak<RefCell<OcoOrder>>>,
}

impl LimitOrder {
    fn new(id: &str, price: f64, quantity: f64) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(LimitOrder {
            id: id.to_string(),
            price,
            quantity,
            is_filled: false,
            oco_group: None,
        }))
    }

    fn fill(&mut self) {
        self.is_filled = true;
        println!("Order {} filled at ${:.2}", self.id, self.price);

        // Notify OCO group
        if let Some(ref oco_weak) = self.oco_group {
            if let Some(oco) = oco_weak.upgrade() {
                oco.borrow_mut().on_order_filled(&self.id);
            }
        }
    }
}

impl OcoOrder {
    fn new(
        id: &str,
        tp: &Rc<RefCell<LimitOrder>>,
        sl: &Rc<RefCell<LimitOrder>>
    ) -> Rc<RefCell<Self>> {
        let oco = Rc::new(RefCell::new(OcoOrder {
            id: id.to_string(),
            take_profit: Some(Rc::downgrade(tp)),
            stop_loss: Some(Rc::downgrade(sl)),
            status: OcoStatus::Active,
        }));

        // Link orders to OCO group
        tp.borrow_mut().oco_group = Some(Rc::downgrade(&oco));
        sl.borrow_mut().oco_group = Some(Rc::downgrade(&oco));

        oco
    }

    fn on_order_filled(&mut self, filled_id: &str) {
        self.status = OcoStatus::OneFilled;

        // Cancel the other order
        let other = if self.take_profit.as_ref()
            .and_then(|w| w.upgrade())
            .map(|o| o.borrow().id.clone())
            .as_deref() == Some(filled_id)
        {
            &self.stop_loss
        } else {
            &self.take_profit
        };

        if let Some(ref weak_order) = other {
            if let Some(order) = weak_order.upgrade() {
                println!("OCO: Cancelling order {}", order.borrow().id);
            }
        }
    }
}

fn main() {
    println!("=== OCO (One-Cancels-Other) ===\n");

    let tp = LimitOrder::new("TP-001", 45000.0, 0.5);
    let sl = LimitOrder::new("SL-001", 40000.0, 0.5);

    let _oco = OcoOrder::new("OCO-001", &tp, &sl);

    println!("OCO group created:");
    println!("- Take Profit: ${:.2}", tp.borrow().price);
    println!("- Stop Loss: ${:.2}\n", sl.borrow().price);

    // Price reached Take Profit
    tp.borrow_mut().fill();
}
```

## Portfolio with Position Tracking

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

struct Portfolio {
    name: String,
    positions: HashMap<String, Rc<RefCell<TrackedPosition>>>,
}

struct TrackedPosition {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    portfolio: Weak<RefCell<Portfolio>>,
}

impl Portfolio {
    fn new(name: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Portfolio {
            name: name.to_string(),
            positions: HashMap::new(),
        }))
    }

    fn add_position(
        portfolio: &Rc<RefCell<Self>>,
        symbol: &str,
        quantity: f64,
        price: f64
    ) {
        let position = Rc::new(RefCell::new(TrackedPosition {
            symbol: symbol.to_string(),
            quantity,
            entry_price: price,
            portfolio: Rc::downgrade(portfolio),
        }));

        portfolio.borrow_mut().positions.insert(
            symbol.to_string(),
            position
        );
    }

    fn total_value(&self, prices: &HashMap<String, f64>) -> f64 {
        self.positions.iter().map(|(symbol, pos)| {
            let position = pos.borrow();
            let price = prices.get(symbol).unwrap_or(&position.entry_price);
            position.quantity * price
        }).sum()
    }

    fn print_summary(&self, prices: &HashMap<String, f64>) {
        println!("\n=== Portfolio: {} ===", self.name);
        for (symbol, pos) in &self.positions {
            let position = pos.borrow();
            let current_price = prices.get(symbol).unwrap_or(&position.entry_price);
            let pnl = (current_price - position.entry_price) * position.quantity;
            let pnl_percent = (current_price - position.entry_price)
                / position.entry_price * 100.0;

            println!(
                "{}: {:.4} @ ${:.2} | P&L: ${:.2} ({:.2}%)",
                symbol, position.quantity, current_price, pnl, pnl_percent
            );
        }
        println!("Total Value: ${:.2}", self.total_value(prices));
    }
}

impl TrackedPosition {
    fn get_portfolio_name(&self) -> Option<String> {
        self.portfolio.upgrade().map(|p| p.borrow().name.clone())
    }
}

fn main() {
    let portfolio = Portfolio::new("Main Trading Account");

    Portfolio::add_position(&portfolio, "BTC", 0.5, 42000.0);
    Portfolio::add_position(&portfolio, "ETH", 5.0, 2500.0);
    Portfolio::add_position(&portfolio, "SOL", 100.0, 95.0);

    let mut prices = HashMap::new();
    prices.insert("BTC".to_string(), 43500.0);
    prices.insert("ETH".to_string(), 2650.0);
    prices.insert("SOL".to_string(), 102.0);

    portfolio.borrow().print_summary(&prices);

    // Position knows its portfolio
    if let Some(btc_pos) = portfolio.borrow().positions.get("BTC") {
        if let Some(name) = btc_pos.borrow().get_portfolio_name() {
            println!("\nBTC position belongs to: {}", name);
        }
    }
}
```

## Weak with Arc for Multithreading

```rust
use std::sync::{Arc, Weak, Mutex};
use std::thread;

struct SharedMarketData {
    symbol: String,
    price: Mutex<f64>,
}

struct PriceObserver {
    name: String,
    data_source: Weak<SharedMarketData>,
}

impl PriceObserver {
    fn new(name: &str, source: &Arc<SharedMarketData>) -> Self {
        PriceObserver {
            name: name.to_string(),
            data_source: Arc::downgrade(source),
        }
    }

    fn check_price(&self) {
        match self.data_source.upgrade() {
            Some(data) => {
                let price = data.price.lock().unwrap();
                println!("{} sees {}: ${:.2}", self.name, data.symbol, *price);
            }
            None => {
                println!("{}: Data source no longer available", self.name);
            }
        }
    }
}

fn main() {
    let market_data = Arc::new(SharedMarketData {
        symbol: "BTC/USDT".to_string(),
        price: Mutex::new(42000.0),
    });

    let observer1 = PriceObserver::new("Strategy A", &market_data);
    let observer2 = PriceObserver::new("Strategy B", &market_data);

    // Update price
    *market_data.price.lock().unwrap() = 42500.0;

    observer1.check_price();
    observer2.check_price();

    // Release data source
    drop(market_data);

    // Observers safely handle missing data
    observer1.check_price();
    observer2.check_price();
}
```

## Weak Methods

```rust
use std::rc::{Rc, Weak};

fn main() {
    let strong = Rc::new(42);
    let weak: Weak<i32> = Rc::downgrade(&strong);

    // strong_count - number of Rc references
    println!("Strong count: {}", Rc::strong_count(&strong));

    // weak_count - number of Weak references
    println!("Weak count: {}", Rc::weak_count(&strong));

    // upgrade - attempt to get Rc
    if let Some(value) = weak.upgrade() {
        println!("Value: {}", value);
    }

    // After drop
    drop(strong);

    // upgrade returns None
    assert!(weak.upgrade().is_none());
    println!("Weak reference is now invalid");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `Weak<T>` | Weak reference, doesn't own data |
| `Rc::downgrade(&rc)` | Creates Weak from Rc |
| `weak.upgrade()` | Attempts to get Rc (Option) |
| `Rc::strong_count()` | Number of strong references |
| `Rc::weak_count()` | Number of weak references |
| `Arc`/`Weak` | Thread-safe versions |

## Practical Exercises

### Exercise 1: Strategy Graph
Create a trading strategy system where strategies can depend on each other without circular references.

### Exercise 2: Order Cache
Implement an executed orders cache using Weak references for automatic cleanup.

### Exercise 3: Event System
Create an event system where subscribers are stored as Weak references.

### Exercise 4: Portfolio Tree
Implement a portfolio hierarchy (main -> sub-portfolios) with back-references.

## Homework

1. **OCO Order System**: Extend the OCO example to support multiple linked orders (OSO - One-Sends-Other).

2. **Position Monitoring**: Create a monitoring system where multiple monitors track one position through Weak references.

3. **Bracket Order**: Implement a Bracket Order (entry + TP + SL) with correct links via Weak.

4. **Cleanup System**: Write a function that checks all Weak references and removes invalid ones from collections.

## Navigation

[← Previous day](../057-rc-refcell/en.md) | [Next day →](../059-interior-mutability/en.md)
