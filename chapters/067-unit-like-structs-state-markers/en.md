# Day 67: Unit-like Structs — State Markers

## Trading Analogy

In trading, every order has a **status**:
- **Pending** — waiting for execution
- **Filled** — fully executed
- **PartiallyFilled** — partially executed
- **Cancelled** — cancelled
- **Rejected** — rejected

These statuses don't contain any data — they are simply **state markers**. In Rust, we use **unit-like structs** for such cases — structs without fields.

## What is a Unit-like Struct?

A unit-like struct is a struct without fields that occupies **zero bytes** in memory:

```rust
// Unit-like structs — contain no data
struct Pending;
struct Filled;
struct Cancelled;

fn main() {
    let status = Pending;

    // Size = 0 bytes!
    println!("Size of Pending: {} bytes", std::mem::size_of::<Pending>());
}
```

## Why Use Empty Structs?

### 1. Type Markers

```rust
// Market status markers
struct MarketOpen;
struct MarketClosed;
struct PreMarket;
struct AfterHours;

fn main() {
    let current_status = MarketOpen;

    println!("Market is now open!");
    println!("Type: {}", std::any::type_name::<MarketOpen>());
}
```

### 2. States in Generic Types

```rust
// Order state markers
struct New;
struct Submitted;
struct Executed;
struct Cancelled;

// Order with state as a type parameter
struct Order<State> {
    symbol: String,
    quantity: f64,
    price: f64,
    _state: std::marker::PhantomData<State>,
}

impl Order<New> {
    fn new(symbol: &str, quantity: f64, price: f64) -> Self {
        Order {
            symbol: symbol.to_string(),
            quantity,
            price,
            _state: std::marker::PhantomData,
        }
    }

    fn submit(self) -> Order<Submitted> {
        println!("Submitting order for {} {} @ {}",
                 self.quantity, self.symbol, self.price);
        Order {
            symbol: self.symbol,
            quantity: self.quantity,
            price: self.price,
            _state: std::marker::PhantomData,
        }
    }
}

impl Order<Submitted> {
    fn execute(self) -> Order<Executed> {
        println!("Order executed!");
        Order {
            symbol: self.symbol,
            quantity: self.quantity,
            price: self.price,
            _state: std::marker::PhantomData,
        }
    }

    fn cancel(self) -> Order<Cancelled> {
        println!("Order cancelled!");
        Order {
            symbol: self.symbol,
            quantity: self.quantity,
            price: self.price,
            _state: std::marker::PhantomData,
        }
    }
}

fn main() {
    // The compiler verifies correct state transitions!
    let order = Order::<New>::new("BTC/USDT", 0.5, 42000.0);
    let submitted = order.submit();
    let executed = submitted.execute();

    // Compile error! Cannot cancel an already executed order
    // executed.cancel(); // won't compile
}
```

## Practical Example: Trading Signals

```rust
// Signal markers
struct BuySignal;
struct SellSignal;
struct HoldSignal;

// Universal trait for signal handling
trait TradingSignal {
    fn action(&self) -> &'static str;
    fn emoji(&self) -> &'static str;
}

impl TradingSignal for BuySignal {
    fn action(&self) -> &'static str { "BUY" }
    fn emoji(&self) -> &'static str { "🟢" }
}

impl TradingSignal for SellSignal {
    fn action(&self) -> &'static str { "SELL" }
    fn emoji(&self) -> &'static str { "🔴" }
}

impl TradingSignal for HoldSignal {
    fn action(&self) -> &'static str { "HOLD" }
    fn emoji(&self) -> &'static str { "🟡" }
}

fn analyze_market(price: f64, sma: f64) -> Box<dyn TradingSignal> {
    if price > sma * 1.02 {
        Box::new(SellSignal)  // Price is 2% above SMA
    } else if price < sma * 0.98 {
        Box::new(BuySignal)   // Price is 2% below SMA
    } else {
        Box::new(HoldSignal)  // Price is around SMA
    }
}

fn main() {
    let current_price = 42000.0;
    let sma_20 = 41500.0;

    let signal = analyze_market(current_price, sma_20);

    println!("=== Market Analysis ===");
    println!("Price: ${:.2}", current_price);
    println!("SMA(20): ${:.2}", sma_20);
    println!("Signal: {} {}", signal.emoji(), signal.action());
}
```

## Trading Strategy States

```rust
use std::marker::PhantomData;

// Strategy states
struct Backtesting;
struct PaperTrading;
struct LiveTrading;

struct Strategy<Mode> {
    name: String,
    capital: f64,
    _mode: PhantomData<Mode>,
}

impl Strategy<Backtesting> {
    fn new(name: &str, capital: f64) -> Self {
        println!("[BACKTEST] Strategy '{}' created", name);
        Strategy {
            name: name.to_string(),
            capital,
            _mode: PhantomData,
        }
    }

    fn run_backtest(&self, data: &[f64]) {
        println!("[BACKTEST] Running on {} candles", data.len());
        // Backtest logic
    }

    fn to_paper(self) -> Strategy<PaperTrading> {
        println!("[PAPER] Switching to paper trading");
        Strategy {
            name: self.name,
            capital: self.capital,
            _mode: PhantomData,
        }
    }
}

impl Strategy<PaperTrading> {
    fn simulate_trade(&self, symbol: &str, side: &str, amount: f64) {
        println!("[PAPER] {} {} {} (simulated)", side, amount, symbol);
    }

    fn to_live(self) -> Strategy<LiveTrading> {
        println!("[LIVE] ⚠️ Going LIVE with ${:.2}!", self.capital);
        Strategy {
            name: self.name,
            capital: self.capital,
            _mode: PhantomData,
        }
    }
}

impl Strategy<LiveTrading> {
    fn execute_trade(&self, symbol: &str, side: &str, amount: f64) {
        println!("[LIVE] 🚨 EXECUTING: {} {} {}", side, amount, symbol);
    }
}

fn main() {
    // Clear strategy lifecycle
    let historical_data = vec![42000.0, 42100.0, 41900.0, 42200.0];

    // 1. First, backtest
    let strategy = Strategy::<Backtesting>::new("SMA Crossover", 10000.0);
    strategy.run_backtest(&historical_data);

    // 2. Then paper trading
    let paper = strategy.to_paper();
    paper.simulate_trade("BTC/USDT", "BUY", 0.1);

    // 3. Only then go live
    let live = paper.to_live();
    live.execute_trade("BTC/USDT", "BUY", 0.1);

    // Cannot go directly from backtest to live!
    // let strategy2 = Strategy::<Backtesting>::new("Test", 1000.0);
    // strategy2.to_live(); // Compile error!
}
```

## Position Direction Markers

```rust
struct Long;
struct Short;

struct Position<Direction> {
    symbol: String,
    entry_price: f64,
    size: f64,
    _direction: std::marker::PhantomData<Direction>,
}

impl<D> Position<D> {
    fn value(&self) -> f64 {
        self.entry_price * self.size
    }
}

impl Position<Long> {
    fn open_long(symbol: &str, price: f64, size: f64) -> Self {
        println!("Opening LONG {} @ {} x {}", symbol, price, size);
        Position {
            symbol: symbol.to_string(),
            entry_price: price,
            size,
            _direction: std::marker::PhantomData,
        }
    }

    fn calculate_pnl(&self, current_price: f64) -> f64 {
        // Long: profit when price rises
        (current_price - self.entry_price) * self.size
    }
}

impl Position<Short> {
    fn open_short(symbol: &str, price: f64, size: f64) -> Self {
        println!("Opening SHORT {} @ {} x {}", symbol, price, size);
        Position {
            symbol: symbol.to_string(),
            entry_price: price,
            size,
            _direction: std::marker::PhantomData,
        }
    }

    fn calculate_pnl(&self, current_price: f64) -> f64 {
        // Short: profit when price falls
        (self.entry_price - current_price) * self.size
    }
}

fn main() {
    let long_pos = Position::<Long>::open_long("BTC/USDT", 42000.0, 0.5);
    let short_pos = Position::<Short>::open_short("ETH/USDT", 2500.0, 2.0);

    let btc_current = 43000.0;
    let eth_current = 2400.0;

    println!("\n=== PnL Report ===");
    println!("BTC Long:  ${:+.2}", long_pos.calculate_pnl(btc_current));
    println!("ETH Short: ${:+.2}", short_pos.calculate_pnl(eth_current));
}
```

## Exchange Connection States

```rust
struct Disconnected;
struct Connecting;
struct Connected;
struct Authenticated;

struct Exchange<State> {
    name: String,
    _state: std::marker::PhantomData<State>,
}

impl Exchange<Disconnected> {
    fn new(name: &str) -> Self {
        Exchange {
            name: name.to_string(),
            _state: std::marker::PhantomData,
        }
    }

    fn connect(self) -> Exchange<Connecting> {
        println!("[{}] Connecting...", self.name);
        Exchange {
            name: self.name,
            _state: std::marker::PhantomData,
        }
    }
}

impl Exchange<Connecting> {
    fn on_connected(self) -> Exchange<Connected> {
        println!("[{}] Connected!", self.name);
        Exchange {
            name: self.name,
            _state: std::marker::PhantomData,
        }
    }
}

impl Exchange<Connected> {
    fn authenticate(self, api_key: &str) -> Exchange<Authenticated> {
        println!("[{}] Authenticating with key {}...", self.name, &api_key[..8]);
        Exchange {
            name: self.name,
            _state: std::marker::PhantomData,
        }
    }
}

impl Exchange<Authenticated> {
    fn get_balance(&self) -> f64 {
        println!("[{}] Fetching balance...", self.name);
        10000.0  // Simulation
    }

    fn place_order(&self, symbol: &str, side: &str, amount: f64) {
        println!("[{}] Placing {} {} {}", self.name, side, amount, symbol);
    }
}

fn main() {
    let exchange = Exchange::<Disconnected>::new("Binance");

    // Correct sequence
    let connecting = exchange.connect();
    let connected = connecting.on_connected();
    let authenticated = connected.authenticate("sk-1234567890abcdef");

    // Now we can trade!
    let balance = authenticated.get_balance();
    println!("Balance: ${:.2}", balance);

    authenticated.place_order("BTC/USDT", "BUY", 0.1);

    // Cannot trade without authentication!
    // let ex = Exchange::<Connected>::new("Test");
    // ex.place_order(...); // Error: method doesn't exist for Connected
}
```

## Data Validation Markers

```rust
struct Unvalidated;
struct Validated;

struct MarketData<State> {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
    _state: std::marker::PhantomData<State>,
}

impl MarketData<Unvalidated> {
    fn new(symbol: &str, price: f64, volume: f64, timestamp: u64) -> Self {
        MarketData {
            symbol: symbol.to_string(),
            price,
            volume,
            timestamp,
            _state: std::marker::PhantomData,
        }
    }

    fn validate(self) -> Result<MarketData<Validated>, String> {
        // Validations
        if self.price <= 0.0 {
            return Err("Price must be positive".to_string());
        }
        if self.volume < 0.0 {
            return Err("Volume cannot be negative".to_string());
        }
        if self.symbol.is_empty() {
            return Err("Symbol cannot be empty".to_string());
        }

        Ok(MarketData {
            symbol: self.symbol,
            price: self.price,
            volume: self.volume,
            timestamp: self.timestamp,
            _state: std::marker::PhantomData,
        })
    }
}

impl MarketData<Validated> {
    // Only validated data can be used for trading
    fn calculate_notional(&self) -> f64 {
        self.price * self.volume
    }

    fn to_csv(&self) -> String {
        format!("{},{},{},{}", self.symbol, self.price, self.volume, self.timestamp)
    }
}

fn process_data(data: MarketData<Validated>) {
    println!("Processing: {} @ ${:.2}", data.symbol, data.price);
    println!("Notional: ${:.2}", data.calculate_notional());
}

fn main() {
    // Create unvalidated data
    let raw_data = MarketData::<Unvalidated>::new("BTC/USDT", 42000.0, 1.5, 1704067200);

    // Validate
    match raw_data.validate() {
        Ok(valid_data) => {
            process_data(valid_data);
        }
        Err(e) => {
            println!("Validation error: {}", e);
        }
    }

    // Invalid data
    let bad_data = MarketData::<Unvalidated>::new("", -100.0, 1.0, 0);
    if let Err(e) = bad_data.validate() {
        println!("Expected error: {}", e);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `struct Name;` | Unit-like struct without fields |
| Zero-sized type | Occupies no memory |
| Type marker | Compile-time type marking |
| PhantomData | Phantom parameter for generic types |
| State machine | State machine with compiler verification |

## Exercises

### Exercise 1: Order Statuses

Create unit-like structs for all possible exchange order statuses and implement transitions between them.

```rust
struct OrderNew;
struct OrderPending;
struct OrderPartiallyFilled;
struct OrderFilled;
struct OrderCancelled;
struct OrderRejected;

// Implement BrokerOrder<Status> struct with transition methods
```

### Exercise 2: Trading Session States

Implement a state machine for trading sessions:
- PreMarket → MarketOpen → MarketClose → AfterHours → PreMarket

### Exercise 3: Risk Management

Create a system with states:
- RiskNormal — normal mode
- RiskWarning — warning level
- RiskCritical — critical level
- TradingHalted — trading stopped

### Exercise 4: Order Validation

Implement an order validation chain:
1. Unvalidated → SizeValidated → PriceValidated → RiskChecked → ReadyToSubmit

## Homework

1. Implement a complete state machine for a trading strategy: Development → Testing → Staging → Production with appropriate restrictions for each state

2. Create a portfolio management system with position states: Opening → Open → Closing → Closed, where each transition requires specific actions

3. Implement a WebSocket client for an exchange with states: Idle → Connecting → Connected → Subscribing → Streaming → Disconnecting

4. Create a market data processing system with validation: RawTick → ParsedTick → ValidatedTick → EnrichedTick → StoredTick

## Navigation

[← Previous day](../066-tuple-structs/en.md) | [Next day →](../068-generic-structs/en.md)
