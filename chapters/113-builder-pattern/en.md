# Day 113: Builder Pattern — Creating Complex Structures

## Trading Analogy

Imagine you're creating a trading order on an exchange. An order can have many parameters:
- Asset symbol (required)
- Direction: buy/sell (required)
- Quantity (required)
- Order type: market, limit, stop
- Price (for limit orders)
- Stop-loss
- Take-profit
- Time in force (GTC, IOC, FOK)
- Leverage
- Comment

If you create a structure through a constructor with 10+ parameters, the code becomes unreadable:

```rust
// Bad: what does each parameter mean?
let order = Order::new("BTC/USDT", Buy, 0.5, Limit, Some(42000.0), Some(41000.0), Some(44000.0), GTC, None, "my order");
```

**Builder Pattern** solves this problem — it allows you to create objects step by step, with clear and readable code.

## What is Builder Pattern

Builder is a creational design pattern that allows you to construct complex objects step by step. It's especially useful when:

1. An object has many parameters
2. Some parameters are optional
3. Validation is needed during creation
4. You want an immutable result

## Basic Builder

```rust
fn main() {
    // Clean and readable code!
    let order = OrderBuilder::new("BTC/USDT", Side::Buy, 0.5)
        .order_type(OrderType::Limit)
        .price(42000.0)
        .stop_loss(41000.0)
        .take_profit(44000.0)
        .build();

    println!("{:#?}", order);
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
    StopLimit,
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: Side,
    quantity: f64,
    order_type: OrderType,
    price: Option<f64>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

struct OrderBuilder {
    symbol: String,
    side: Side,
    quantity: f64,
    order_type: OrderType,
    price: Option<f64>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl OrderBuilder {
    fn new(symbol: &str, side: Side, quantity: f64) -> Self {
        OrderBuilder {
            symbol: symbol.to_string(),
            side,
            quantity,
            order_type: OrderType::Market, // Default value
            price: None,
            stop_loss: None,
            take_profit: None,
        }
    }

    fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = order_type;
        self
    }

    fn price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }

    fn stop_loss(mut self, stop_loss: f64) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    fn take_profit(mut self, take_profit: f64) -> Self {
        self.take_profit = Some(take_profit);
        self
    }

    fn build(self) -> Order {
        Order {
            symbol: self.symbol,
            side: self.side,
            quantity: self.quantity,
            order_type: self.order_type,
            price: self.price,
            stop_loss: self.stop_loss,
            take_profit: self.take_profit,
        }
    }
}
```

## Builder with Validation

In real trading, validation is critically important. Builder can return `Result`:

```rust
fn main() {
    // Valid order
    let order = OrderBuilder::new("BTC/USDT", Side::Buy, 0.5)
        .order_type(OrderType::Limit)
        .price(42000.0)
        .stop_loss(41000.0)
        .take_profit(44000.0)
        .build();

    match order {
        Ok(o) => println!("Order created: {:?}", o),
        Err(e) => println!("Error: {}", e),
    }

    // Invalid order: limit without price
    let invalid_order = OrderBuilder::new("BTC/USDT", Side::Buy, 0.5)
        .order_type(OrderType::Limit)
        .build();

    match invalid_order {
        Ok(o) => println!("Order created: {:?}", o),
        Err(e) => println!("Validation error: {}", e),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderType {
    Market,
    Limit,
    StopLimit,
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: Side,
    quantity: f64,
    order_type: OrderType,
    price: Option<f64>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

struct OrderBuilder {
    symbol: String,
    side: Side,
    quantity: f64,
    order_type: OrderType,
    price: Option<f64>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl OrderBuilder {
    fn new(symbol: &str, side: Side, quantity: f64) -> Self {
        OrderBuilder {
            symbol: symbol.to_string(),
            side,
            quantity,
            order_type: OrderType::Market,
            price: None,
            stop_loss: None,
            take_profit: None,
        }
    }

    fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = order_type;
        self
    }

    fn price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }

    fn stop_loss(mut self, stop_loss: f64) -> Self {
        self.stop_loss = Some(stop_loss);
        self
    }

    fn take_profit(mut self, take_profit: f64) -> Self {
        self.take_profit = Some(take_profit);
        self
    }

    fn build(self) -> Result<Order, String> {
        // Quantity validation
        if self.quantity <= 0.0 {
            return Err("Quantity must be positive".to_string());
        }

        // Limit order validation
        if self.order_type == OrderType::Limit && self.price.is_none() {
            return Err("Limit order requires price".to_string());
        }

        // Stop-loss validation for buy orders
        if let (Some(price), Some(sl)) = (self.price, self.stop_loss) {
            if self.side == Side::Buy && sl >= price {
                return Err("Stop-loss must be below entry price for buy orders".to_string());
            }
            if self.side == Side::Sell && sl <= price {
                return Err("Stop-loss must be above entry price for sell orders".to_string());
            }
        }

        // Take-profit validation for buy orders
        if let (Some(price), Some(tp)) = (self.price, self.take_profit) {
            if self.side == Side::Buy && tp <= price {
                return Err("Take-profit must be above entry price for buy orders".to_string());
            }
            if self.side == Side::Sell && tp >= price {
                return Err("Take-profit must be below entry price for sell orders".to_string());
            }
        }

        Ok(Order {
            symbol: self.symbol,
            side: self.side,
            quantity: self.quantity,
            order_type: self.order_type,
            price: self.price,
            stop_loss: self.stop_loss,
            take_profit: self.take_profit,
        })
    }
}
```

## Builder for Trading Strategy

```rust
fn main() {
    let strategy = StrategyBuilder::new("MA Crossover")
        .description("Moving average crossover strategy for BTC")
        .add_symbol("BTC/USDT")
        .add_symbol("ETH/USDT")
        .timeframe(Timeframe::H1)
        .risk_per_trade(2.0)
        .max_positions(3)
        .max_drawdown(10.0)
        .enable_trailing_stop(true)
        .build()
        .expect("Invalid strategy configuration");

    println!("{:#?}", strategy);
}

#[derive(Debug, Clone, Copy)]
enum Timeframe {
    M1,
    M5,
    M15,
    H1,
    H4,
    D1,
}

#[derive(Debug)]
struct Strategy {
    name: String,
    description: String,
    symbols: Vec<String>,
    timeframe: Timeframe,
    risk_per_trade: f64,
    max_positions: usize,
    max_drawdown: f64,
    trailing_stop: bool,
}

struct StrategyBuilder {
    name: String,
    description: String,
    symbols: Vec<String>,
    timeframe: Timeframe,
    risk_per_trade: f64,
    max_positions: usize,
    max_drawdown: f64,
    trailing_stop: bool,
}

impl StrategyBuilder {
    fn new(name: &str) -> Self {
        StrategyBuilder {
            name: name.to_string(),
            description: String::new(),
            symbols: Vec::new(),
            timeframe: Timeframe::H1,
            risk_per_trade: 1.0,
            max_positions: 1,
            max_drawdown: 20.0,
            trailing_stop: false,
        }
    }

    fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    fn add_symbol(mut self, symbol: &str) -> Self {
        self.symbols.push(symbol.to_string());
        self
    }

    fn timeframe(mut self, tf: Timeframe) -> Self {
        self.timeframe = tf;
        self
    }

    fn risk_per_trade(mut self, risk: f64) -> Self {
        self.risk_per_trade = risk;
        self
    }

    fn max_positions(mut self, max: usize) -> Self {
        self.max_positions = max;
        self
    }

    fn max_drawdown(mut self, dd: f64) -> Self {
        self.max_drawdown = dd;
        self
    }

    fn enable_trailing_stop(mut self, enable: bool) -> Self {
        self.trailing_stop = enable;
        self
    }

    fn build(self) -> Result<Strategy, String> {
        if self.name.is_empty() {
            return Err("Strategy name is required".to_string());
        }

        if self.symbols.is_empty() {
            return Err("At least one symbol is required".to_string());
        }

        if self.risk_per_trade <= 0.0 || self.risk_per_trade > 100.0 {
            return Err("Risk per trade must be between 0 and 100%".to_string());
        }

        if self.max_drawdown <= 0.0 || self.max_drawdown > 100.0 {
            return Err("Max drawdown must be between 0 and 100%".to_string());
        }

        Ok(Strategy {
            name: self.name,
            description: self.description,
            symbols: self.symbols,
            timeframe: self.timeframe,
            risk_per_trade: self.risk_per_trade,
            max_positions: self.max_positions,
            max_drawdown: self.max_drawdown,
            trailing_stop: self.trailing_stop,
        })
    }
}
```

## Builder for Portfolio

```rust
fn main() {
    let portfolio = PortfolioBuilder::new("Crypto Portfolio")
        .initial_balance(10000.0)
        .add_allocation("BTC", 40.0)
        .add_allocation("ETH", 30.0)
        .add_allocation("SOL", 20.0)
        .add_allocation("USDT", 10.0)
        .rebalance_threshold(5.0)
        .build()
        .expect("Invalid portfolio configuration");

    println!("{:#?}", portfolio);
    println!("\nTotal allocation: {:.1}%", portfolio.total_allocation());
}

#[derive(Debug)]
struct Allocation {
    asset: String,
    target_percent: f64,
}

#[derive(Debug)]
struct Portfolio {
    name: String,
    initial_balance: f64,
    allocations: Vec<Allocation>,
    rebalance_threshold: f64,
}

impl Portfolio {
    fn total_allocation(&self) -> f64 {
        self.allocations.iter().map(|a| a.target_percent).sum()
    }
}

struct PortfolioBuilder {
    name: String,
    initial_balance: f64,
    allocations: Vec<Allocation>,
    rebalance_threshold: f64,
}

impl PortfolioBuilder {
    fn new(name: &str) -> Self {
        PortfolioBuilder {
            name: name.to_string(),
            initial_balance: 0.0,
            allocations: Vec::new(),
            rebalance_threshold: 5.0,
        }
    }

    fn initial_balance(mut self, balance: f64) -> Self {
        self.initial_balance = balance;
        self
    }

    fn add_allocation(mut self, asset: &str, percent: f64) -> Self {
        self.allocations.push(Allocation {
            asset: asset.to_string(),
            target_percent: percent,
        });
        self
    }

    fn rebalance_threshold(mut self, threshold: f64) -> Self {
        self.rebalance_threshold = threshold;
        self
    }

    fn build(self) -> Result<Portfolio, String> {
        if self.name.is_empty() {
            return Err("Portfolio name is required".to_string());
        }

        if self.initial_balance <= 0.0 {
            return Err("Initial balance must be positive".to_string());
        }

        if self.allocations.is_empty() {
            return Err("At least one allocation is required".to_string());
        }

        let total: f64 = self.allocations.iter().map(|a| a.target_percent).sum();
        if (total - 100.0).abs() > 0.01 {
            return Err(format!("Allocations must sum to 100%, got {:.2}%", total));
        }

        Ok(Portfolio {
            name: self.name,
            initial_balance: self.initial_balance,
            allocations: self.allocations,
            rebalance_threshold: self.rebalance_threshold,
        })
    }
}
```

## Builder for Risk Management

```rust
fn main() {
    let risk_config = RiskConfigBuilder::new()
        .max_position_size(5.0)        // 5% of portfolio per position
        .max_daily_loss(3.0)           // 3% max daily loss
        .max_total_exposure(30.0)      // 30% total exposure
        .max_correlation(0.7)          // Max correlation between positions
        .require_stop_loss(true)
        .min_risk_reward(2.0)          // Minimum 1:2 risk/reward
        .build()
        .expect("Invalid risk configuration");

    println!("{:#?}", risk_config);

    // Check order against risk config
    let order_size_percent = 3.0;
    let risk_reward = 2.5;
    let has_stop_loss = true;

    if risk_config.validate_order(order_size_percent, risk_reward, has_stop_loss) {
        println!("\n✓ Order passes risk checks");
    } else {
        println!("\n✗ Order fails risk checks");
    }
}

#[derive(Debug)]
struct RiskConfig {
    max_position_size: f64,
    max_daily_loss: f64,
    max_total_exposure: f64,
    max_correlation: f64,
    require_stop_loss: bool,
    min_risk_reward: f64,
}

impl RiskConfig {
    fn validate_order(&self, size_percent: f64, risk_reward: f64, has_stop_loss: bool) -> bool {
        if size_percent > self.max_position_size {
            return false;
        }

        if self.require_stop_loss && !has_stop_loss {
            return false;
        }

        if risk_reward < self.min_risk_reward {
            return false;
        }

        true
    }
}

struct RiskConfigBuilder {
    max_position_size: f64,
    max_daily_loss: f64,
    max_total_exposure: f64,
    max_correlation: f64,
    require_stop_loss: bool,
    min_risk_reward: f64,
}

impl RiskConfigBuilder {
    fn new() -> Self {
        RiskConfigBuilder {
            max_position_size: 10.0,      // Default values
            max_daily_loss: 5.0,
            max_total_exposure: 50.0,
            max_correlation: 0.8,
            require_stop_loss: false,
            min_risk_reward: 1.0,
        }
    }

    fn max_position_size(mut self, size: f64) -> Self {
        self.max_position_size = size;
        self
    }

    fn max_daily_loss(mut self, loss: f64) -> Self {
        self.max_daily_loss = loss;
        self
    }

    fn max_total_exposure(mut self, exposure: f64) -> Self {
        self.max_total_exposure = exposure;
        self
    }

    fn max_correlation(mut self, corr: f64) -> Self {
        self.max_correlation = corr;
        self
    }

    fn require_stop_loss(mut self, require: bool) -> Self {
        self.require_stop_loss = require;
        self
    }

    fn min_risk_reward(mut self, rr: f64) -> Self {
        self.min_risk_reward = rr;
        self
    }

    fn build(self) -> Result<RiskConfig, String> {
        if self.max_position_size <= 0.0 || self.max_position_size > 100.0 {
            return Err("Max position size must be between 0 and 100%".to_string());
        }

        if self.max_daily_loss <= 0.0 || self.max_daily_loss > 100.0 {
            return Err("Max daily loss must be between 0 and 100%".to_string());
        }

        if self.min_risk_reward < 0.0 {
            return Err("Min risk/reward must be non-negative".to_string());
        }

        if self.max_correlation < 0.0 || self.max_correlation > 1.0 {
            return Err("Max correlation must be between 0 and 1".to_string());
        }

        Ok(RiskConfig {
            max_position_size: self.max_position_size,
            max_daily_loss: self.max_daily_loss,
            max_total_exposure: self.max_total_exposure,
            max_correlation: self.max_correlation,
            require_stop_loss: self.require_stop_loss,
            min_risk_reward: self.min_risk_reward,
        })
    }
}
```

## Type-State Builder Pattern

Advanced technique — the compiler guarantees correct method call order:

```rust
fn main() {
    // Compiler forces calling methods in correct order
    let order = OrderBuilder::new()
        .symbol("BTC/USDT")   // Now we can set side
        .side(Side::Buy)       // Now we can set quantity
        .quantity(0.5)         // Now we can build()
        .build();

    println!("{:?}", order);

    // This will NOT compile:
    // let order = OrderBuilder::new().build();  // Error!
    // let order = OrderBuilder::new().quantity(0.5).build();  // Error!
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: Side,
    quantity: f64,
}

// Marker types for states
struct NoSymbol;
struct HasSymbol;
struct HasSide;
struct HasQuantity;

struct OrderBuilder<State> {
    symbol: Option<String>,
    side: Option<Side>,
    quantity: Option<f64>,
    state: std::marker::PhantomData<State>,
}

impl OrderBuilder<NoSymbol> {
    fn new() -> Self {
        OrderBuilder {
            symbol: None,
            side: None,
            quantity: None,
            state: std::marker::PhantomData,
        }
    }

    fn symbol(self, symbol: &str) -> OrderBuilder<HasSymbol> {
        OrderBuilder {
            symbol: Some(symbol.to_string()),
            side: None,
            quantity: None,
            state: std::marker::PhantomData,
        }
    }
}

impl OrderBuilder<HasSymbol> {
    fn side(self, side: Side) -> OrderBuilder<HasSide> {
        OrderBuilder {
            symbol: self.symbol,
            side: Some(side),
            quantity: None,
            state: std::marker::PhantomData,
        }
    }
}

impl OrderBuilder<HasSide> {
    fn quantity(self, quantity: f64) -> OrderBuilder<HasQuantity> {
        OrderBuilder {
            symbol: self.symbol,
            side: self.side,
            quantity: Some(quantity),
            state: std::marker::PhantomData,
        }
    }
}

impl OrderBuilder<HasQuantity> {
    fn build(self) -> Order {
        Order {
            symbol: self.symbol.unwrap(),
            side: self.side.unwrap(),
            quantity: self.quantity.unwrap(),
        }
    }
}
```

## Practical Example: Backtest Constructor

```rust
fn main() {
    let backtest = BacktestBuilder::new()
        .strategy("MA Crossover")
        .symbol("BTC/USDT")
        .start_date("2024-01-01")
        .end_date("2024-12-31")
        .initial_capital(10000.0)
        .commission(0.1)
        .slippage(0.05)
        .build()
        .expect("Invalid backtest configuration");

    println!("{:#?}", backtest);
    println!("\nStarting backtest simulation...");
    backtest.run();
}

#[derive(Debug)]
struct Backtest {
    strategy: String,
    symbol: String,
    start_date: String,
    end_date: String,
    initial_capital: f64,
    commission: f64,
    slippage: f64,
}

impl Backtest {
    fn run(&self) {
        println!("Running {} on {} from {} to {}",
            self.strategy, self.symbol, self.start_date, self.end_date);
        println!("Initial capital: ${:.2}", self.initial_capital);
        println!("Commission: {:.2}%, Slippage: {:.2}%",
            self.commission, self.slippage);
    }
}

struct BacktestBuilder {
    strategy: Option<String>,
    symbol: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    initial_capital: f64,
    commission: f64,
    slippage: f64,
}

impl BacktestBuilder {
    fn new() -> Self {
        BacktestBuilder {
            strategy: None,
            symbol: None,
            start_date: None,
            end_date: None,
            initial_capital: 10000.0,
            commission: 0.1,
            slippage: 0.0,
        }
    }

    fn strategy(mut self, name: &str) -> Self {
        self.strategy = Some(name.to_string());
        self
    }

    fn symbol(mut self, symbol: &str) -> Self {
        self.symbol = Some(symbol.to_string());
        self
    }

    fn start_date(mut self, date: &str) -> Self {
        self.start_date = Some(date.to_string());
        self
    }

    fn end_date(mut self, date: &str) -> Self {
        self.end_date = Some(date.to_string());
        self
    }

    fn initial_capital(mut self, capital: f64) -> Self {
        self.initial_capital = capital;
        self
    }

    fn commission(mut self, comm: f64) -> Self {
        self.commission = comm;
        self
    }

    fn slippage(mut self, slip: f64) -> Self {
        self.slippage = slip;
        self
    }

    fn build(self) -> Result<Backtest, String> {
        let strategy = self.strategy
            .ok_or("Strategy name is required")?;
        let symbol = self.symbol
            .ok_or("Symbol is required")?;
        let start_date = self.start_date
            .ok_or("Start date is required")?;
        let end_date = self.end_date
            .ok_or("End date is required")?;

        if self.initial_capital <= 0.0 {
            return Err("Initial capital must be positive".to_string());
        }

        if self.commission < 0.0 {
            return Err("Commission cannot be negative".to_string());
        }

        if self.slippage < 0.0 {
            return Err("Slippage cannot be negative".to_string());
        }

        Ok(Backtest {
            strategy,
            symbol,
            start_date,
            end_date,
            initial_capital: self.initial_capital,
            commission: self.commission,
            slippage: self.slippage,
        })
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Builder Pattern | Pattern for step-by-step object creation |
| Method Chaining | Returning `self` for call chaining |
| `mut self` | Moving and modifying the builder |
| Default Values | Reasonable defaults |
| Validation | Checking in `build()` method |
| `Result<T, E>` | Returning error for invalid data |
| Type-State | Compile-time call order guarantees |

## Homework

1. **TradeBuilder**: Create a builder for a `Trade` struct with fields:
   - `entry_price`, `exit_price`, `quantity`, `side`
   - `entry_time`, `exit_time` (optional)
   - `commission`, `slippage`
   - Add a `profit()` method to calculate profit

2. **IndicatorBuilder**: Implement a builder for configuring technical indicators:
   - Indicator type (SMA, EMA, RSI, MACD)
   - Period(s)
   - Data source (close, open, high, low)
   - Validate parameter compatibility

3. **AlertBuilder**: Create a builder for trading alerts:
   - Condition (price above/below, MA crossover, etc.)
   - Symbol and timeframe
   - Notification method (email, telegram, webhook)
   - One-time or recurring

4. **Type-State Challenge**: Refactor `TradeBuilder` using the Type-State pattern so that:
   - Cannot create a trade without entry_price
   - exit_price can only be set after entry_price
   - quantity is required for build()

## Navigation

[← Previous day](../112-newtype-pattern/en.md) | [Next day →](../114-factory-pattern/en.md)
