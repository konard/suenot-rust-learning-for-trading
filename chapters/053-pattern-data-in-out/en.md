# Day 53: Pattern "Data In and Out" — Moving Data Through Functions

## Trading Analogy

Imagine a trading firm's workflow. When a client sends an order for execution, one of three things happens:

1. **Transfer of ownership** — the client fully transfers the order to the broker. The order now belongs to the broker; the client can no longer modify it.

2. **Show for analysis** — an analyst looks at the order but doesn't take it. The client remains the owner and can continue working with it.

3. **Pass for modification** — the risk manager receives the order, adjusts the position size, and returns it.

In Rust, these are the three ways to pass data to functions: **by value** (transfer ownership), **by reference** (borrow for reading), and **by mutable reference** (borrow for modification).

## Theory: How Data Flows in Rust

### Data Flow Rules

When data enters a function, there are three scenarios:

```
┌─────────────────────────────────────────────────────────────┐
│                      DATA IN                                 │
├─────────────────────────────────────────────────────────────┤
│  fn process(data: T)      → Take ownership (move)           │
│  fn analyze(data: &T)     → Look at it (borrow)            │
│  fn modify(data: &mut T)  → Change it (mutable borrow)     │
├─────────────────────────────────────────────────────────────┤
│                      DATA OUT                                │
├─────────────────────────────────────────────────────────────┤
│  fn create() -> T         → Create and give ownership      │
│  fn get(&self) -> &T      → Let them look                  │
│  fn get_mut(&mut self) -> &mut T → Let them modify         │
└─────────────────────────────────────────────────────────────┘
```

## Pattern 1: Take Ownership and Return Result

Function takes data, processes it, and returns a new result.

```rust
fn main() {
    let order = Order {
        symbol: String::from("BTC/USDT"),
        side: OrderSide::Buy,
        quantity: 0.5,
        price: 42000.0,
    };

    // Pass the order to the function — order is no longer accessible!
    let executed = execute_order(order);

    // println!("{}", order.symbol); // Error! order was moved
    println!("Executed: {} at ${}", executed.symbol, executed.fill_price);
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

struct Order {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: f64,
}

struct ExecutedOrder {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    fill_price: f64,
    commission: f64,
}

fn execute_order(order: Order) -> ExecutedOrder {
    // Function owns order, can use its fields
    let slippage = 0.001; // 0.1% slippage
    let commission_rate = 0.0004; // 0.04% commission

    let fill_price = match order.side {
        OrderSide::Buy => order.price * (1.0 + slippage),
        OrderSide::Sell => order.price * (1.0 - slippage),
    };

    let commission = fill_price * order.quantity * commission_rate;

    ExecutedOrder {
        symbol: order.symbol,  // Move the String
        side: order.side,
        quantity: order.quantity,
        fill_price,
        commission,
    }
}
```

**When to use:** When the function completely "consumes" data and creates something new.

## Pattern 2: Borrow for Analysis

Function only reads data without taking ownership.

```rust
fn main() {
    let portfolio = Portfolio {
        positions: vec![
            Position { symbol: String::from("BTC"), quantity: 1.5, avg_price: 40000.0 },
            Position { symbol: String::from("ETH"), quantity: 10.0, avg_price: 2500.0 },
            Position { symbol: String::from("SOL"), quantity: 100.0, avg_price: 100.0 },
        ],
    };

    // Pass a reference — portfolio remains accessible
    let total = calculate_portfolio_value(&portfolio, &get_current_prices());
    println!("Portfolio value: ${:.2}", total);

    // portfolio is still available!
    let risk = analyze_portfolio_risk(&portfolio);
    println!("Portfolio risk score: {:.2}", risk);

    // Can call multiple functions with the same data
    let diversification = calculate_diversification(&portfolio);
    println!("Diversification index: {:.2}", diversification);
}

struct Position {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

struct Portfolio {
    positions: Vec<Position>,
}

use std::collections::HashMap;

fn get_current_prices() -> HashMap<String, f64> {
    let mut prices = HashMap::new();
    prices.insert(String::from("BTC"), 42000.0);
    prices.insert(String::from("ETH"), 2800.0);
    prices.insert(String::from("SOL"), 120.0);
    prices
}

fn calculate_portfolio_value(portfolio: &Portfolio, prices: &HashMap<String, f64>) -> f64 {
    portfolio.positions.iter()
        .map(|pos| {
            let current_price = prices.get(&pos.symbol).unwrap_or(&pos.avg_price);
            pos.quantity * current_price
        })
        .sum()
}

fn analyze_portfolio_risk(portfolio: &Portfolio) -> f64 {
    // Simple risk calculation based on concentration
    let total_value: f64 = portfolio.positions.iter()
        .map(|p| p.quantity * p.avg_price)
        .sum();

    if total_value == 0.0 {
        return 0.0;
    }

    let max_position_pct = portfolio.positions.iter()
        .map(|p| (p.quantity * p.avg_price) / total_value)
        .fold(0.0, f64::max);

    // Risk is higher if one position dominates
    max_position_pct * 100.0
}

fn calculate_diversification(portfolio: &Portfolio) -> f64 {
    // Diversification index: 1 / sum of squared weights
    let total_value: f64 = portfolio.positions.iter()
        .map(|p| p.quantity * p.avg_price)
        .sum();

    if total_value == 0.0 {
        return 0.0;
    }

    let sum_of_squares: f64 = portfolio.positions.iter()
        .map(|p| {
            let weight = (p.quantity * p.avg_price) / total_value;
            weight * weight
        })
        .sum();

    if sum_of_squares == 0.0 {
        0.0
    } else {
        1.0 / sum_of_squares
    }
}
```

**When to use:** When you only need to read data without modifying it.

## Pattern 3: Mutable Borrow

Function gets the ability to modify data.

```rust
fn main() {
    let mut trading_account = TradingAccount {
        balance: 10000.0,
        positions: Vec::new(),
        trade_history: Vec::new(),
    };

    println!("Initial balance: ${:.2}", trading_account.balance);

    // Pass mutable reference — function can modify
    open_position(&mut trading_account, "BTC/USDT", 0.1, 42000.0);
    println!("After opening BTC: ${:.2}", trading_account.balance);

    open_position(&mut trading_account, "ETH/USDT", 2.0, 2800.0);
    println!("After opening ETH: ${:.2}", trading_account.balance);

    // Close position
    close_position(&mut trading_account, "BTC/USDT", 43500.0);
    println!("After closing BTC: ${:.2}", trading_account.balance);

    // View history
    print_trade_history(&trading_account);
}

struct TradingAccount {
    balance: f64,
    positions: Vec<AccountPosition>,
    trade_history: Vec<Trade>,
}

struct AccountPosition {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

struct Trade {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    pnl: f64,
}

fn open_position(account: &mut TradingAccount, symbol: &str, quantity: f64, price: f64) {
    let cost = quantity * price;

    if account.balance >= cost {
        account.balance -= cost;
        account.positions.push(AccountPosition {
            symbol: String::from(symbol),
            quantity,
            entry_price: price,
        });
        account.trade_history.push(Trade {
            symbol: String::from(symbol),
            side: String::from("BUY"),
            quantity,
            price,
            pnl: 0.0,
        });
    }
}

fn close_position(account: &mut TradingAccount, symbol: &str, exit_price: f64) {
    if let Some(pos_index) = account.positions.iter().position(|p| p.symbol == symbol) {
        let position = account.positions.remove(pos_index);
        let revenue = position.quantity * exit_price;
        let pnl = (exit_price - position.entry_price) * position.quantity;

        account.balance += revenue;
        account.trade_history.push(Trade {
            symbol: String::from(symbol),
            side: String::from("SELL"),
            quantity: position.quantity,
            price: exit_price,
            pnl,
        });
    }
}

fn print_trade_history(account: &TradingAccount) {
    println!("\n=== Trade History ===");
    for trade in &account.trade_history {
        println!(
            "{} {} {} @ ${:.2} (PnL: ${:.2})",
            trade.side, trade.quantity, trade.symbol, trade.price, trade.pnl
        );
    }
}
```

**When to use:** When the function must modify data.

## Pattern 4: Take and Return Ownership

Sometimes you need to take data, process it, and return it (transformation).

```rust
fn main() {
    let mut candles = vec![
        Candle { open: 42000.0, high: 42500.0, low: 41800.0, close: 42200.0, volume: 100.0 },
        Candle { open: 42200.0, high: 42800.0, low: 42100.0, close: 42600.0, volume: 150.0 },
        Candle { open: 42600.0, high: 43000.0, low: 42400.0, close: 42900.0, volume: 120.0 },
    ];

    // Transfer ownership and get it back with calculated indicators
    candles = add_sma_indicator(candles, 2);

    // Now candles contains SMA
    for (i, candle) in candles.iter().enumerate() {
        println!("Candle {}: close={}, SMA={:?}", i, candle.close, candle.sma);
    }
}

struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    #[allow(dead_code)]
    sma: Option<f64>,
}

// Implicitly define struct with default sma
impl Default for Candle {
    fn default() -> Self {
        Candle {
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
            sma: None,
        }
    }
}

fn add_sma_indicator(mut candles: Vec<Candle>, period: usize) -> Vec<Candle> {
    for i in 0..candles.len() {
        if i + 1 >= period {
            let sum: f64 = candles[i + 1 - period..=i]
                .iter()
                .map(|c| c.close)
                .sum();
            candles[i].sma = Some(sum / period as f64);
        }
    }
    candles  // Return ownership
}
```

**When to use:** When you need to transform data entirely.

## Pattern 5: Builder Pattern

Creating complex objects through a chain of calls.

```rust
fn main() {
    let strategy = StrategyBuilder::new("SMA Crossover")
        .with_symbol("BTC/USDT")
        .with_timeframe("1h")
        .with_fast_period(10)
        .with_slow_period(20)
        .with_risk_percent(2.0)
        .build();

    println!("Strategy: {}", strategy.name);
    println!("Symbol: {}", strategy.symbol);
    println!("Timeframe: {}", strategy.timeframe);
    println!("Fast SMA: {}", strategy.fast_period);
    println!("Slow SMA: {}", strategy.slow_period);
    println!("Risk: {}%", strategy.risk_percent);
}

struct Strategy {
    name: String,
    symbol: String,
    timeframe: String,
    fast_period: usize,
    slow_period: usize,
    risk_percent: f64,
}

struct StrategyBuilder {
    name: String,
    symbol: String,
    timeframe: String,
    fast_period: usize,
    slow_period: usize,
    risk_percent: f64,
}

impl StrategyBuilder {
    fn new(name: &str) -> Self {
        StrategyBuilder {
            name: String::from(name),
            symbol: String::from("BTC/USDT"),
            timeframe: String::from("1h"),
            fast_period: 10,
            slow_period: 20,
            risk_percent: 1.0,
        }
    }

    fn with_symbol(mut self, symbol: &str) -> Self {
        self.symbol = String::from(symbol);
        self  // Return self for chaining
    }

    fn with_timeframe(mut self, timeframe: &str) -> Self {
        self.timeframe = String::from(timeframe);
        self
    }

    fn with_fast_period(mut self, period: usize) -> Self {
        self.fast_period = period;
        self
    }

    fn with_slow_period(mut self, period: usize) -> Self {
        self.slow_period = period;
        self
    }

    fn with_risk_percent(mut self, percent: f64) -> Self {
        self.risk_percent = percent;
        self
    }

    fn build(self) -> Strategy {
        Strategy {
            name: self.name,
            symbol: self.symbol,
            timeframe: self.timeframe,
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            risk_percent: self.risk_percent,
        }
    }
}
```

**When to use:** For creating complex objects with many optional parameters.

## Pattern 6: Processing with Callback

Passing a function to process each element.

```rust
fn main() {
    let trades = vec![
        TradeRecord { symbol: String::from("BTC"), pnl: 500.0 },
        TradeRecord { symbol: String::from("ETH"), pnl: -200.0 },
        TradeRecord { symbol: String::from("BTC"), pnl: 300.0 },
        TradeRecord { symbol: String::from("SOL"), pnl: -50.0 },
        TradeRecord { symbol: String::from("BTC"), pnl: 150.0 },
    ];

    // Process each profitable trade
    process_trades(&trades, |trade| {
        if trade.pnl > 0.0 {
            println!("Profit on {}: ${:.2}", trade.symbol, trade.pnl);
        }
    });

    // Calculate total BTC PnL
    let btc_pnl = aggregate_trades(&trades, |trade| {
        if trade.symbol == "BTC" {
            trade.pnl
        } else {
            0.0
        }
    });
    println!("\nTotal BTC PnL: ${:.2}", btc_pnl);
}

struct TradeRecord {
    symbol: String,
    pnl: f64,
}

fn process_trades<F>(trades: &[TradeRecord], mut processor: F)
where
    F: FnMut(&TradeRecord),
{
    for trade in trades {
        processor(trade);
    }
}

fn aggregate_trades<F>(trades: &[TradeRecord], selector: F) -> f64
where
    F: Fn(&TradeRecord) -> f64,
{
    trades.iter().map(|t| selector(t)).sum()
}
```

**When to use:** When you need flexible data processing with different logic.

## Practical Example: Trading Engine

```rust
fn main() {
    let mut engine = TradingEngine::new(10000.0);

    // Get market data
    let market_data = MarketData {
        symbol: String::from("BTC/USDT"),
        bid: 41990.0,
        ask: 42010.0,
        last: 42000.0,
    };

    // Generate signal (analyze data without taking ownership)
    if let Some(signal) = engine.generate_signal(&market_data) {
        println!("Signal: {:?} {} at ${}", signal.side, signal.symbol, signal.price);

        // Create order from signal (signal is moved)
        let order = engine.create_order(signal);
        println!("Created order: {} {} @ ${}", order.side, order.quantity, order.price);

        // Execute order (modify engine state)
        if let Ok(execution) = engine.execute(&mut order.clone()) {
            println!("Executed: {} {} @ ${}", execution.side, execution.quantity, execution.fill_price);
        }
    }

    // View current state
    engine.print_status();
}

#[derive(Debug, Clone)]
enum Side {
    Buy,
    Sell,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

struct MarketData {
    symbol: String,
    bid: f64,
    ask: f64,
    last: f64,
}

#[derive(Debug)]
struct Signal {
    symbol: String,
    side: Side,
    price: f64,
    strength: f64,
}

#[derive(Clone)]
struct TradeOrder {
    symbol: String,
    side: Side,
    quantity: f64,
    price: f64,
}

struct Execution {
    symbol: String,
    side: Side,
    quantity: f64,
    fill_price: f64,
}

struct EnginePosition {
    symbol: String,
    quantity: f64,
    avg_price: f64,
}

struct TradingEngine {
    balance: f64,
    positions: Vec<EnginePosition>,
    risk_per_trade: f64,
}

impl TradingEngine {
    fn new(initial_balance: f64) -> Self {
        TradingEngine {
            balance: initial_balance,
            positions: Vec::new(),
            risk_per_trade: 0.02, // 2% risk per trade
        }
    }

    // Analyze market without taking ownership of data
    fn generate_signal(&self, data: &MarketData) -> Option<Signal> {
        // Simple logic: buy if spread is tight
        let spread_pct = (data.ask - data.bid) / data.last * 100.0;

        if spread_pct < 0.1 {
            Some(Signal {
                symbol: data.symbol.clone(),
                side: Side::Buy,
                price: data.ask,
                strength: 1.0 - spread_pct,
            })
        } else {
            None
        }
    }

    // Create order from signal (takes ownership of signal)
    fn create_order(&self, signal: Signal) -> TradeOrder {
        let risk_amount = self.balance * self.risk_per_trade;
        let quantity = risk_amount / signal.price;

        TradeOrder {
            symbol: signal.symbol,
            side: signal.side,
            quantity,
            price: signal.price,
        }
    }

    // Execute order, modifying engine state
    fn execute(&mut self, order: &mut TradeOrder) -> Result<Execution, String> {
        let cost = order.quantity * order.price;

        match order.side {
            Side::Buy => {
                if self.balance < cost {
                    return Err(String::from("Insufficient balance"));
                }

                self.balance -= cost;
                self.positions.push(EnginePosition {
                    symbol: order.symbol.clone(),
                    quantity: order.quantity,
                    avg_price: order.price,
                });
            }
            Side::Sell => {
                // Sell logic...
            }
        }

        Ok(Execution {
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            quantity: order.quantity,
            fill_price: order.price,
        })
    }

    fn print_status(&self) {
        println!("\n=== Engine Status ===");
        println!("Balance: ${:.2}", self.balance);
        println!("Positions: {}", self.positions.len());
        for pos in &self.positions {
            println!("  {} {} @ ${:.2}", pos.symbol, pos.quantity, pos.avg_price);
        }
    }
}
```

## Exercises

### Exercise 1: Quote Processor

Create a function that takes a vector of quotes and returns a processed vector with added indicators (SMA, RSI).

```rust
fn process_quotes(quotes: Vec<Quote>) -> Vec<ProcessedQuote> {
    // Your code here
}
```

### Exercise 2: Risk Manager

Create a `RiskManager` struct with methods:
- `check_position(&self, position: &Position) -> RiskReport` — position analysis
- `adjust_position(&mut self, position: &mut Position)` — adjustment
- `close_if_needed(self, position: Position) -> Option<Position>` — close if condition met

### Exercise 3: Data Processing Pipeline

Implement a chain of market data processing:

```rust
let result = market_data
    .validate()     // Check validity
    .normalize()    // Normalize
    .calculate()    // Calculate indicators
    .filter()       // Filter
    .collect();     // Collect results
```

### Exercise 4: Report Generator

Create a builder function for a trading report:

```rust
let report = ReportBuilder::new()
    .with_trades(&trades)
    .with_period("2024-01")
    .with_metrics(&["pnl", "win_rate", "sharpe"])
    .build()?;
```

## What We Learned

| Pattern | Signature | When to Use |
|---------|-----------|-------------|
| Take ownership | `fn(T) -> U` | Consumption/transformation |
| Read | `fn(&T) -> U` | Analysis without modification |
| Modify | `fn(&mut T)` | In-place modification |
| Transform | `fn(T) -> T` | Modify and return |
| Builder | `fn(self) -> Self` | Method chaining |
| Callback | `fn(&T, F) where F: Fn(&T)` | Flexible processing |

## Homework

1. **Trading Journal**: Create a `TradeJournal` struct with methods:
   - `record(&mut self, trade: Trade)` — record a trade
   - `analyze(&self) -> JournalStats` — analyze statistics
   - `export(self) -> String` — export to JSON (consumes journal)

2. **Position Optimizer**: Write a function `optimize_positions(portfolio: &mut Portfolio, target_allocation: &Allocation)` that rebalances the portfolio.

3. **Signal Generator**: Implement a `SignalGenerator` trait with method `fn generate(&self, data: &MarketData) -> Option<Signal>` and create several implementations (SMA crossover, RSI, Bollinger Bands).

4. **Backtest Engine**: Create a `Backtest` struct with Builder pattern:
   ```rust
   let result = Backtest::new()
       .with_strategy(strategy)      // Takes strategy
       .with_data(&historical_data)  // Borrows data
       .with_initial_capital(10000.0)
       .run()?;                       // Run and get result
   ```

## Navigation

[← Previous day](../052-ownership-move-semantics/en.md) | [Next day →](../054-pattern-borrowed-vs-owned/en.md)
