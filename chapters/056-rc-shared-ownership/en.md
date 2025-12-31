# Day 56: Rc — Multiple Owners of One Asset

## Trading Analogy

Imagine this situation: you have **market data** (quotes, price history) that is simultaneously used by multiple trading strategies. Each strategy is an independent module, but they all work with the same data. Who "owns" this data? All strategies simultaneously!

By default in Rust, data can only have one owner. But `Rc<T>` (Reference Counting) allows having **multiple owners** of the same data. It's like shared access to a trading terminal — every trader can view quotes, but there's only one terminal.

## What is Rc?

`Rc` is a "smart pointer" with a reference counter:
- When cloning `Rc`, data is not copied — the counter increases
- When the counter reaches 0, data is deallocated
- `Rc` only works in single-threaded code (use `Arc` for multi-threading)

```rust
use std::rc::Rc;

fn main() {
    // Market data that will be used by multiple strategies
    let market_data = Rc::new(vec![42000.0, 42100.0, 41950.0, 42200.0]);

    println!("Reference count: {}", Rc::strong_count(&market_data));  // 1

    let strategy_a = Rc::clone(&market_data);
    println!("After cloning for strategy A: {}", Rc::strong_count(&market_data));  // 2

    let strategy_b = Rc::clone(&market_data);
    println!("After cloning for strategy B: {}", Rc::strong_count(&market_data));  // 3

    // All three variables point to the same data
    println!("Data from strategy_a: {:?}", strategy_a);
    println!("Data from strategy_b: {:?}", strategy_b);
}
```

**Important:** Use `Rc::clone(&x)` instead of `x.clone()`. This explicitly shows that we're incrementing the counter, not copying data.

## When to Use Rc in Trading

### 1. Shared Market Data for Multiple Strategies

```rust
use std::rc::Rc;

struct MarketData {
    symbol: String,
    prices: Vec<f64>,
    volumes: Vec<f64>,
}

struct MomentumStrategy {
    data: Rc<MarketData>,
    period: usize,
}

struct MeanReversionStrategy {
    data: Rc<MarketData>,
    threshold: f64,
}

impl MomentumStrategy {
    fn analyze(&self) -> &str {
        if self.data.prices.len() < self.period {
            return "INSUFFICIENT_DATA";
        }
        let recent = &self.data.prices[self.data.prices.len() - self.period..];
        let first = recent[0];
        let last = recent[recent.len() - 1];

        if last > first * 1.02 {
            "BUY"
        } else if last < first * 0.98 {
            "SELL"
        } else {
            "HOLD"
        }
    }
}

impl MeanReversionStrategy {
    fn analyze(&self) -> &str {
        if self.data.prices.is_empty() {
            return "INSUFFICIENT_DATA";
        }
        let avg: f64 = self.data.prices.iter().sum::<f64>() / self.data.prices.len() as f64;
        let current = self.data.prices.last().unwrap();
        let deviation = (current - avg) / avg;

        if deviation > self.threshold {
            "SELL"  // Price above average — expect reversion
        } else if deviation < -self.threshold {
            "BUY"   // Price below average — expect reversion
        } else {
            "HOLD"
        }
    }
}

fn main() {
    let data = Rc::new(MarketData {
        symbol: String::from("BTC/USDT"),
        prices: vec![42000.0, 42500.0, 43000.0, 43500.0, 44000.0],
        volumes: vec![100.0, 150.0, 200.0, 180.0, 220.0],
    });

    let momentum = MomentumStrategy {
        data: Rc::clone(&data),
        period: 3,
    };

    let mean_reversion = MeanReversionStrategy {
        data: Rc::clone(&data),
        threshold: 0.02,
    };

    println!("Symbol: {}", data.symbol);
    println!("Momentum signal: {}", momentum.analyze());
    println!("Mean Reversion signal: {}", mean_reversion.analyze());
    println!("Active references to data: {}", Rc::strong_count(&data));
}
```

### 2. Order Dependency Graph

In trading, orders can depend on each other (OCO, bracket orders). `Rc` allows modeling such relationships:

```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    linked_orders: Vec<Rc<RefCell<Order>>>,
}

impl Order {
    fn new(id: u64, symbol: &str, side: &str, price: f64, quantity: f64) -> Self {
        Order {
            id,
            symbol: String::from(symbol),
            side: String::from(side),
            price,
            quantity,
            linked_orders: Vec::new(),
        }
    }

    fn link_order(&mut self, order: Rc<RefCell<Order>>) {
        self.linked_orders.push(order);
    }

    fn cancel_linked(&self) {
        for order in &self.linked_orders {
            println!("Canceling linked order #{}", order.borrow().id);
        }
    }
}

fn main() {
    // Bracket order: main order + take profit + stop loss
    let main_order = Rc::new(RefCell::new(Order::new(
        1, "BTC/USDT", "BUY", 42000.0, 0.5
    )));

    let take_profit = Rc::new(RefCell::new(Order::new(
        2, "BTC/USDT", "SELL", 45000.0, 0.5
    )));

    let stop_loss = Rc::new(RefCell::new(Order::new(
        3, "BTC/USDT", "SELL", 40000.0, 0.5
    )));

    // Link take profit and stop loss to main order
    main_order.borrow_mut().link_order(Rc::clone(&take_profit));
    main_order.borrow_mut().link_order(Rc::clone(&stop_loss));

    // Take profit and stop loss are linked to each other (OCO)
    take_profit.borrow_mut().link_order(Rc::clone(&stop_loss));
    stop_loss.borrow_mut().link_order(Rc::clone(&take_profit));

    println!("Main order: {:?}", main_order.borrow().id);
    println!("When take profit executes, cancel linked:");
    take_profit.borrow().cancel_linked();
}
```

### 3. Shared Config for System Components

```rust
use std::rc::Rc;

struct TradingConfig {
    max_position_size: f64,
    risk_per_trade: f64,
    allowed_symbols: Vec<String>,
    api_endpoint: String,
}

struct RiskManager {
    config: Rc<TradingConfig>,
}

struct OrderExecutor {
    config: Rc<TradingConfig>,
}

struct PortfolioTracker {
    config: Rc<TradingConfig>,
}

impl RiskManager {
    fn check_position(&self, size: f64) -> bool {
        size <= self.config.max_position_size
    }

    fn calculate_position_size(&self, balance: f64) -> f64 {
        balance * (self.config.risk_per_trade / 100.0)
    }
}

impl OrderExecutor {
    fn can_trade(&self, symbol: &str) -> bool {
        self.config.allowed_symbols.contains(&symbol.to_string())
    }
}

impl PortfolioTracker {
    fn get_api(&self) -> &str {
        &self.config.api_endpoint
    }
}

fn main() {
    let config = Rc::new(TradingConfig {
        max_position_size: 10000.0,
        risk_per_trade: 2.0,
        allowed_symbols: vec![
            String::from("BTC/USDT"),
            String::from("ETH/USDT"),
        ],
        api_endpoint: String::from("https://api.exchange.com"),
    });

    let risk_mgr = RiskManager { config: Rc::clone(&config) };
    let executor = OrderExecutor { config: Rc::clone(&config) };
    let tracker = PortfolioTracker { config: Rc::clone(&config) };

    println!("Position 5000 allowed: {}", risk_mgr.check_position(5000.0));
    println!("Can trade BTC: {}", executor.can_trade("BTC/USDT"));
    println!("API endpoint: {}", tracker.get_api());
    println!("Components with config: {}", Rc::strong_count(&config));
}
```

## Rc + RefCell: Mutable Data with Multiple Owners

`Rc` gives immutable access. For mutable data, combine `Rc<RefCell<T>>`:

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Portfolio {
    balance: f64,
    positions: Vec<(String, f64)>,  // (symbol, quantity)
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            balance: initial_balance,
            positions: Vec::new(),
        }
    }

    fn buy(&mut self, symbol: &str, quantity: f64, price: f64) -> bool {
        let cost = quantity * price;
        if cost > self.balance {
            return false;
        }
        self.balance -= cost;
        self.positions.push((symbol.to_string(), quantity));
        true
    }

    fn get_balance(&self) -> f64 {
        self.balance
    }
}

struct Strategy {
    name: String,
    portfolio: Rc<RefCell<Portfolio>>,
}

impl Strategy {
    fn execute_buy(&self, symbol: &str, quantity: f64, price: f64) {
        let success = self.portfolio.borrow_mut().buy(symbol, quantity, price);
        if success {
            println!("{}: Bought {} {} at ${}", self.name, quantity, symbol, price);
        } else {
            println!("{}: Insufficient funds", self.name);
        }
    }

    fn check_balance(&self) {
        println!("{}: Current balance ${:.2}",
            self.name,
            self.portfolio.borrow().get_balance()
        );
    }
}

fn main() {
    let portfolio = Rc::new(RefCell::new(Portfolio::new(10000.0)));

    let strategy_a = Strategy {
        name: String::from("Momentum"),
        portfolio: Rc::clone(&portfolio),
    };

    let strategy_b = Strategy {
        name: String::from("Scalping"),
        portfolio: Rc::clone(&portfolio),
    };

    strategy_a.check_balance();
    strategy_a.execute_buy("BTC/USDT", 0.1, 42000.0);
    strategy_b.check_balance();  // Balance already changed!
    strategy_b.execute_buy("ETH/USDT", 1.0, 2500.0);
    strategy_a.check_balance();

    println!("\nFinal balance: ${:.2}", portfolio.borrow().get_balance());
}
```

## Reference Counting

```rust
use std::rc::Rc;

fn main() {
    let data = Rc::new(vec![42000.0, 42100.0]);
    println!("After creation: {}", Rc::strong_count(&data));  // 1

    {
        let ref1 = Rc::clone(&data);
        println!("Inside block: {}", Rc::strong_count(&data));  // 2

        let ref2 = Rc::clone(&data);
        println!("Another reference: {}", Rc::strong_count(&data));  // 3
    }  // ref1 and ref2 go out of scope

    println!("After block: {}", Rc::strong_count(&data));  // 1
}
```

## Weak References — Preventing Cycles

`Rc::downgrade` creates a weak reference that doesn't increment the counter:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct TradingBot {
    name: String,
    parent: Option<Weak<RefCell<TradingBot>>>,  // Weak reference to parent
    children: Vec<Rc<RefCell<TradingBot>>>,     // Strong references to children
}

impl TradingBot {
    fn new(name: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(TradingBot {
            name: name.to_string(),
            parent: None,
            children: Vec::new(),
        }))
    }

    fn add_child(parent: &Rc<RefCell<Self>>, child: &Rc<RefCell<Self>>) {
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        parent.borrow_mut().children.push(Rc::clone(child));
    }
}

fn main() {
    let master_bot = TradingBot::new("MasterBot");
    let btc_bot = TradingBot::new("BTCBot");
    let eth_bot = TradingBot::new("ETHBot");

    TradingBot::add_child(&master_bot, &btc_bot);
    TradingBot::add_child(&master_bot, &eth_bot);

    // Child bot can access parent
    if let Some(parent_weak) = &btc_bot.borrow().parent {
        if let Some(parent) = parent_weak.upgrade() {
            println!("BTCBot's parent: {}", parent.borrow().name);
        }
    }

    println!("References to MasterBot: {}", Rc::strong_count(&master_bot));  // 1
    println!("Child bots: {}", master_bot.borrow().children.len());  // 2
}
```

## Practical Example: Portfolio Monitoring System

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct PriceData {
    symbol: String,
    current_price: f64,
    history: Vec<f64>,
}

impl PriceData {
    fn new(symbol: &str, initial_price: f64) -> Self {
        PriceData {
            symbol: symbol.to_string(),
            current_price: initial_price,
            history: vec![initial_price],
        }
    }

    fn update(&mut self, new_price: f64) {
        self.current_price = new_price;
        self.history.push(new_price);
    }

    fn get_change_percent(&self) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }
        let first = self.history[0];
        ((self.current_price - first) / first) * 100.0
    }
}

struct Position {
    price_data: Rc<RefCell<PriceData>>,
    quantity: f64,
    entry_price: f64,
}

impl Position {
    fn get_pnl(&self) -> f64 {
        let current = self.price_data.borrow().current_price;
        (current - self.entry_price) * self.quantity
    }

    fn get_pnl_percent(&self) -> f64 {
        let current = self.price_data.borrow().current_price;
        ((current - self.entry_price) / self.entry_price) * 100.0
    }
}

struct RiskMonitor {
    price_data: Rc<RefCell<PriceData>>,
    alert_threshold: f64,
}

impl RiskMonitor {
    fn check_alerts(&self) -> Option<String> {
        let data = self.price_data.borrow();
        let change = data.get_change_percent();

        if change.abs() > self.alert_threshold {
            Some(format!(
                "ALERT: {} changed by {:.2}%!",
                data.symbol, change
            ))
        } else {
            None
        }
    }
}

fn main() {
    // Shared price data
    let btc_data = Rc::new(RefCell::new(PriceData::new("BTC/USDT", 42000.0)));

    // Position uses the same data
    let position = Position {
        price_data: Rc::clone(&btc_data),
        quantity: 0.5,
        entry_price: 42000.0,
    };

    // Risk monitor too
    let risk_monitor = RiskMonitor {
        price_data: Rc::clone(&btc_data),
        alert_threshold: 5.0,
    };

    println!("=== Initial State ===");
    println!("PnL: ${:.2} ({:.2}%)", position.get_pnl(), position.get_pnl_percent());

    // Update price — all components see changes
    btc_data.borrow_mut().update(44000.0);

    println!("\n=== After Price Increase ===");
    println!("PnL: ${:.2} ({:.2}%)", position.get_pnl(), position.get_pnl_percent());

    if let Some(alert) = risk_monitor.check_alerts() {
        println!("{}", alert);
    }

    btc_data.borrow_mut().update(40000.0);

    println!("\n=== After Price Drop ===");
    println!("PnL: ${:.2} ({:.2}%)", position.get_pnl(), position.get_pnl_percent());

    if let Some(alert) = risk_monitor.check_alerts() {
        println!("{}", alert);
    }
}
```

## Rc vs Other Approaches

| Approach | When to Use |
|----------|-------------|
| Ownership | Single owner, data is moved |
| References | Temporary borrowing, known lifetime |
| `Rc<T>` | Multiple owners, immutable data |
| `Rc<RefCell<T>>` | Multiple owners, need to mutate data |
| `Arc<T>` | Multiple owners in multi-threaded code |

## Rc Limitations

1. **Single-threaded only** — use `Arc` for multi-threading
2. **Cycles possible** — use `Weak` to prevent memory leaks
3. **Overhead** — reference counting requires memory and time

## What We Learned

| Concept | Description |
|---------|-------------|
| `Rc<T>` | Smart pointer with reference counting |
| `Rc::clone()` | Increments counter, doesn't copy data |
| `Rc::strong_count()` | Returns number of strong references |
| `Rc<RefCell<T>>` | Combination for mutable data |
| `Rc::downgrade()` | Creates a weak reference (Weak) |
| `Weak::upgrade()` | Attempts to get Rc from Weak |

## Homework

1. Create a system with multiple trading strategies using shared `Rc<MarketData>`. Implement data updates and verify that all strategies see current prices.

2. Implement an OCO (One-Cancels-Other) order graph where executing one order automatically cancels linked ones. Use `Rc<RefCell<Order>>`.

3. Create a trading bot hierarchy (master bot and child bots for different pairs). Use `Weak` for references from children to parent.

4. Implement a price update subscription system: multiple subscribers (`Rc`) to one data source, with unsubscribe capability (removing reference).

## Navigation

[← Previous day](../055-box-heap-allocation/en.md) | [Next day →](../057-refcell-interior-mutability/en.md)
