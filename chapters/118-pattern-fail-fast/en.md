# Day 118: Pattern: Fail Fast

## Trading Analogy

Imagine you're trading on an exchange and receiving asset price data. What's better:
- **Keep working** with invalid data and lose money on a wrong trade?
- **Stop immediately** at the first error and report the problem?

The answer is obvious: **fail fast** is a pattern where we immediately stop execution upon detecting an error, instead of trying to "drag" broken data further.

In trading, this is critical:
- Got an empty price? Better not trade than trade with price 0
- Negative balance? Immediately stop the bot
- API returned an error? Don't try to guess the data

## What is Fail Fast?

**Fail Fast** is a development philosophy where:
1. Errors are detected as early as possible
2. On error, the program immediately reports the problem
3. There are no "silent" errors that accumulate

```rust
// Bad: trying to continue with invalid data
fn bad_calculate_position_size(balance: f64, risk_percent: f64) -> f64 {
    if balance <= 0.0 {
        return 0.0; // Silently return 0, hiding the problem
    }
    balance * risk_percent / 100.0
}

// Good: fail fast — immediately report the error
fn good_calculate_position_size(balance: f64, risk_percent: f64) -> Result<f64, String> {
    if balance <= 0.0 {
        return Err(format!("Invalid balance: {}. Balance must be positive", balance));
    }
    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return Err(format!("Invalid risk: {}%. Must be between 0 and 100", risk_percent));
    }
    Ok(balance * risk_percent / 100.0)
}

fn main() {
    // Bad approach — error is hidden
    let size1 = bad_calculate_position_size(-1000.0, 2.0);
    println!("Position size (bad): {}", size1); // 0.0 — but why?

    // Good approach — error is visible immediately
    match good_calculate_position_size(-1000.0, 2.0) {
        Ok(size) => println!("Position size: {}", size),
        Err(e) => println!("Error: {}", e), // Immediately clear what's wrong
    }
}
```

## Fail Fast in Order Validation

```rust
#[derive(Debug)]
struct Order {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
enum OrderError {
    EmptySymbol,
    InvalidQuantity(f64),
    InvalidPrice(f64),
    InsufficientBalance { required: f64, available: f64 },
}

impl std::fmt::Display for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderError::EmptySymbol => write!(f, "Symbol cannot be empty"),
            OrderError::InvalidQuantity(q) => write!(f, "Invalid quantity: {}", q),
            OrderError::InvalidPrice(p) => write!(f, "Invalid price: {}", p),
            OrderError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient funds: required {}, available {}", required, available)
            }
        }
    }
}

/// Validates an order using the fail fast principle.
/// Returns Err at the first error encountered.
fn validate_order(order: &Order, balance: f64) -> Result<(), OrderError> {
    // Check 1: symbol is not empty
    if order.symbol.is_empty() {
        return Err(OrderError::EmptySymbol);
    }

    // Check 2: quantity is positive
    if order.quantity <= 0.0 {
        return Err(OrderError::InvalidQuantity(order.quantity));
    }

    // Check 3: price is positive
    if order.price <= 0.0 {
        return Err(OrderError::InvalidPrice(order.price));
    }

    // Check 4: sufficient funds
    let required = order.quantity * order.price;
    if required > balance {
        return Err(OrderError::InsufficientBalance {
            required,
            available: balance,
        });
    }

    Ok(())
}

fn main() {
    let balance = 10000.0;

    // Invalid order — empty symbol
    let order1 = Order {
        symbol: String::new(),
        side: OrderSide::Buy,
        quantity: 0.5,
        price: 42000.0,
    };

    match validate_order(&order1, balance) {
        Ok(()) => println!("Order is valid"),
        Err(e) => println!("Fail fast: {}", e),
    }

    // Invalid order — negative quantity
    let order2 = Order {
        symbol: "BTC".to_string(),
        side: OrderSide::Buy,
        quantity: -1.0,
        price: 42000.0,
    };

    match validate_order(&order2, balance) {
        Ok(()) => println!("Order is valid"),
        Err(e) => println!("Fail fast: {}", e),
    }

    // Valid order
    let order3 = Order {
        symbol: "BTC".to_string(),
        side: OrderSide::Buy,
        quantity: 0.1,
        price: 42000.0,
    };

    match validate_order(&order3, balance) {
        Ok(()) => println!("Order is valid, ready to send"),
        Err(e) => println!("Fail fast: {}", e),
    }
}
```

## Fail Fast with the ? Operator

The `?` operator in Rust is the perfect tool for fail fast. It immediately interrupts function execution at the first error:

```rust
#[derive(Debug)]
struct TradeSignal {
    symbol: String,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
    position_size: f64,
}

#[derive(Debug)]
enum SignalError {
    InvalidPrice(String),
    StopLossAboveEntry,
    TakeProfitBelowEntry,
    InvalidRiskReward(f64),
}

impl std::fmt::Display for SignalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalError::InvalidPrice(msg) => write!(f, "Invalid price: {}", msg),
            SignalError::StopLossAboveEntry => write!(f, "Stop-loss is above entry price"),
            SignalError::TakeProfitBelowEntry => write!(f, "Take-profit is below entry price"),
            SignalError::InvalidRiskReward(rr) => write!(f, "Bad R:R = {:.2}, minimum is 2.0", rr),
        }
    }
}

/// Creates a trade signal with fail fast checks.
fn create_long_signal(
    symbol: &str,
    entry: f64,
    stop_loss: f64,
    take_profit: f64,
    position_size: f64,
) -> Result<TradeSignal, SignalError> {
    // Each check uses ? to immediately return an error
    validate_price(entry, "entry")?;
    validate_price(stop_loss, "stop_loss")?;
    validate_price(take_profit, "take_profit")?;
    validate_stop_loss_for_long(entry, stop_loss)?;
    validate_take_profit_for_long(entry, take_profit)?;
    validate_risk_reward(entry, stop_loss, take_profit)?;

    Ok(TradeSignal {
        symbol: symbol.to_string(),
        entry_price: entry,
        stop_loss,
        take_profit,
        position_size,
    })
}

fn validate_price(price: f64, name: &str) -> Result<(), SignalError> {
    if price <= 0.0 || price.is_nan() || price.is_infinite() {
        return Err(SignalError::InvalidPrice(format!("{} = {}", name, price)));
    }
    Ok(())
}

fn validate_stop_loss_for_long(entry: f64, stop_loss: f64) -> Result<(), SignalError> {
    if stop_loss >= entry {
        return Err(SignalError::StopLossAboveEntry);
    }
    Ok(())
}

fn validate_take_profit_for_long(entry: f64, take_profit: f64) -> Result<(), SignalError> {
    if take_profit <= entry {
        return Err(SignalError::TakeProfitBelowEntry);
    }
    Ok(())
}

fn validate_risk_reward(entry: f64, stop_loss: f64, take_profit: f64) -> Result<(), SignalError> {
    let risk = entry - stop_loss;
    let reward = take_profit - entry;
    let rr = reward / risk;
    if rr < 2.0 {
        return Err(SignalError::InvalidRiskReward(rr));
    }
    Ok(())
}

fn main() {
    // Attempt to create signal with invalid price — fail fast on first check
    match create_long_signal("BTC", -100.0, 40000.0, 45000.0, 0.1) {
        Ok(signal) => println!("Signal created: {:?}", signal),
        Err(e) => println!("Error creating signal: {}", e),
    }

    // Stop-loss above entry — fail fast
    match create_long_signal("BTC", 42000.0, 43000.0, 45000.0, 0.1) {
        Ok(signal) => println!("Signal created: {:?}", signal),
        Err(e) => println!("Error creating signal: {}", e),
    }

    // Bad Risk:Reward — fail fast
    match create_long_signal("BTC", 42000.0, 41000.0, 42500.0, 0.1) {
        Ok(signal) => println!("Signal created: {:?}", signal),
        Err(e) => println!("Error creating signal: {}", e),
    }

    // Valid signal
    match create_long_signal("BTC", 42000.0, 41000.0, 45000.0, 0.1) {
        Ok(signal) => println!("Signal created: {:?}", signal),
        Err(e) => println!("Error creating signal: {}", e),
    }
}
```

## Fail Fast in Constructors

The best place for fail fast is the constructor. If an object cannot be created in a valid state, it's better not to create it at all:

```rust
#[derive(Debug)]
struct Portfolio {
    name: String,
    balance: f64,
    max_positions: usize,
    risk_per_trade: f64,
}

#[derive(Debug)]
enum PortfolioError {
    EmptyName,
    NegativeBalance(f64),
    ZeroPositions,
    InvalidRisk(f64),
}

impl std::fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortfolioError::EmptyName => write!(f, "Portfolio name cannot be empty"),
            PortfolioError::NegativeBalance(b) => write!(f, "Balance cannot be negative: {}", b),
            PortfolioError::ZeroPositions => write!(f, "Number of positions must be greater than 0"),
            PortfolioError::InvalidRisk(r) => write!(f, "Risk must be between 0.1% and 10%: {}%", r),
        }
    }
}

impl Portfolio {
    /// Constructor with fail fast validation.
    /// Guarantees that Portfolio is always in a valid state.
    pub fn new(
        name: String,
        balance: f64,
        max_positions: usize,
        risk_per_trade: f64,
    ) -> Result<Self, PortfolioError> {
        // Fail fast: check all invariants upfront
        if name.trim().is_empty() {
            return Err(PortfolioError::EmptyName);
        }

        if balance < 0.0 {
            return Err(PortfolioError::NegativeBalance(balance));
        }

        if max_positions == 0 {
            return Err(PortfolioError::ZeroPositions);
        }

        if risk_per_trade < 0.1 || risk_per_trade > 10.0 {
            return Err(PortfolioError::InvalidRisk(risk_per_trade));
        }

        // All checks passed — create the object
        Ok(Portfolio {
            name,
            balance,
            max_positions,
            risk_per_trade,
        })
    }

    pub fn calculate_position_size(&self, entry: f64, stop_loss: f64) -> f64 {
        let risk_amount = self.balance * (self.risk_per_trade / 100.0);
        let price_risk = (entry - stop_loss).abs();
        if price_risk == 0.0 {
            0.0
        } else {
            risk_amount / price_risk
        }
    }
}

fn main() {
    // Attempts to create invalid portfolio — fail fast
    println!("Attempt 1: empty name");
    match Portfolio::new("".to_string(), 10000.0, 5, 2.0) {
        Ok(p) => println!("Created: {:?}", p),
        Err(e) => println!("Error: {}", e),
    }

    println!("\nAttempt 2: negative balance");
    match Portfolio::new("Main".to_string(), -5000.0, 5, 2.0) {
        Ok(p) => println!("Created: {:?}", p),
        Err(e) => println!("Error: {}", e),
    }

    println!("\nAttempt 3: risk too high");
    match Portfolio::new("Main".to_string(), 10000.0, 5, 50.0) {
        Ok(p) => println!("Created: {:?}", p),
        Err(e) => println!("Error: {}", e),
    }

    println!("\nAttempt 4: valid portfolio");
    match Portfolio::new("Main Portfolio".to_string(), 10000.0, 5, 2.0) {
        Ok(p) => {
            println!("Created: {:?}", p);
            let size = p.calculate_position_size(42000.0, 41000.0);
            println!("Position size: {:.6} BTC", size);
        }
        Err(e) => println!("Error: {}", e),
    }
}
```

## Fail Fast vs. Collect All Errors

Sometimes you need to collect all errors at once instead of stopping at the first one. This is useful for UI where the user needs to see all problems at once:

```rust
#[derive(Debug)]
struct OrderValidation {
    symbol: String,
    quantity: f64,
    price: f64,
    stop_loss: Option<f64>,
}

#[derive(Debug)]
struct ValidationErrors {
    errors: Vec<String>,
}

impl ValidationErrors {
    fn new() -> Self {
        ValidationErrors { errors: Vec::new() }
    }

    fn add(&mut self, error: String) {
        self.errors.push(error);
    }

    fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    fn into_result(self) -> Result<(), Self> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "  - {}", error)?;
        }
        Ok(())
    }
}

/// Collects all validation errors (not fail fast).
/// Useful for showing all problems to the user at once.
fn validate_order_collect_all(order: &OrderValidation) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();

    if order.symbol.is_empty() {
        errors.add("Symbol cannot be empty".to_string());
    }

    if order.quantity <= 0.0 {
        errors.add(format!("Quantity must be positive: {}", order.quantity));
    }

    if order.price <= 0.0 {
        errors.add(format!("Price must be positive: {}", order.price));
    }

    if let Some(sl) = order.stop_loss {
        if sl <= 0.0 {
            errors.add(format!("Stop-loss must be positive: {}", sl));
        }
        if sl >= order.price {
            errors.add(format!("Stop-loss ({}) must be below entry price ({})", sl, order.price));
        }
    }

    errors.into_result()
}

fn main() {
    // Order with multiple errors
    let bad_order = OrderValidation {
        symbol: String::new(),
        quantity: -1.0,
        price: 0.0,
        stop_loss: Some(-100.0),
    };

    println!("Validating bad order:");
    match validate_order_collect_all(&bad_order) {
        Ok(()) => println!("Order is valid"),
        Err(errors) => {
            println!("Errors found:\n{}", errors);
        }
    }

    println!("\nValidating good order:");
    let good_order = OrderValidation {
        symbol: "BTC".to_string(),
        quantity: 0.5,
        price: 42000.0,
        stop_loss: Some(41000.0),
    };

    match validate_order_collect_all(&good_order) {
        Ok(()) => println!("Order is valid!"),
        Err(errors) => println!("Errors found:\n{}", errors),
    }
}
```

## When to Use Fail Fast

| Situation | Approach | Reason |
|-----------|----------|--------|
| API request | Fail fast | Invalid data cannot be sent |
| Object constructor | Fail fast | Object must always be valid |
| Critical checks | Fail fast | Safety over UX |
| User form | Collect all | UX: show all errors at once |
| Batch processing | Collect all | Process what's possible, log errors |
| Config parsing | Fail fast | Can't work without config |

## Practical Example: Trading Bot with Fail Fast

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct TradingBot {
    name: String,
    api_key: String,
    balance: f64,
    positions: HashMap<String, f64>,
    max_positions: usize,
    risk_per_trade: f64,
}

#[derive(Debug)]
enum BotError {
    EmptyName,
    EmptyApiKey,
    NegativeBalance(f64),
    InvalidRisk(f64),
    TooManyPositions { current: usize, max: usize },
    InsufficientBalance { required: f64, available: f64 },
    PositionNotFound(String),
}

impl std::fmt::Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotError::EmptyName => write!(f, "Bot name cannot be empty"),
            BotError::EmptyApiKey => write!(f, "API key is required"),
            BotError::NegativeBalance(b) => write!(f, "Balance cannot be negative: {}", b),
            BotError::InvalidRisk(r) => write!(f, "Risk must be 0.1-5%: {}%", r),
            BotError::TooManyPositions { current, max } => {
                write!(f, "Position limit exceeded: {} of {}", current, max)
            }
            BotError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient funds: need ${:.2}, have ${:.2}", required, available)
            }
            BotError::PositionNotFound(s) => write!(f, "Position not found: {}", s),
        }
    }
}

impl TradingBot {
    /// Constructor with fail fast — all checks before object creation.
    pub fn new(
        name: String,
        api_key: String,
        balance: f64,
        max_positions: usize,
        risk_per_trade: f64,
    ) -> Result<Self, BotError> {
        if name.trim().is_empty() {
            return Err(BotError::EmptyName);
        }
        if api_key.trim().is_empty() {
            return Err(BotError::EmptyApiKey);
        }
        if balance < 0.0 {
            return Err(BotError::NegativeBalance(balance));
        }
        if risk_per_trade < 0.1 || risk_per_trade > 5.0 {
            return Err(BotError::InvalidRisk(risk_per_trade));
        }

        Ok(TradingBot {
            name,
            api_key,
            balance,
            positions: HashMap::new(),
            max_positions,
            risk_per_trade,
        })
    }

    /// Open position with fail fast checks.
    pub fn open_position(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<(), BotError> {
        // Fail fast: check position limit
        if self.positions.len() >= self.max_positions {
            return Err(BotError::TooManyPositions {
                current: self.positions.len(),
                max: self.max_positions,
            });
        }

        // Fail fast: check balance
        let cost = quantity * price;
        if cost > self.balance {
            return Err(BotError::InsufficientBalance {
                required: cost,
                available: self.balance,
            });
        }

        // All checks passed — open position
        self.balance -= cost;
        *self.positions.entry(symbol.to_string()).or_insert(0.0) += quantity;

        println!("[{}] Opened position: {} {} @ ${:.2}", self.name, quantity, symbol, price);
        Ok(())
    }

    /// Close position with fail fast.
    pub fn close_position(&mut self, symbol: &str, price: f64) -> Result<f64, BotError> {
        // Fail fast: position must exist
        let quantity = self.positions.remove(symbol)
            .ok_or_else(|| BotError::PositionNotFound(symbol.to_string()))?;

        let proceeds = quantity * price;
        self.balance += proceeds;

        println!("[{}] Closed position: {} {} @ ${:.2}", self.name, quantity, symbol, price);
        Ok(proceeds)
    }

    pub fn status(&self) {
        println!("\n=== {} ===", self.name);
        println!("Balance: ${:.2}", self.balance);
        println!("Positions: {:?}", self.positions);
        println!("Risk per trade: {}%", self.risk_per_trade);
    }
}

fn main() {
    println!("=== Creating bot ===\n");

    // Fail fast when creating bot
    let bot_result = TradingBot::new(
        "AlphaBot".to_string(),
        "secret-api-key".to_string(),
        10000.0,
        3,
        2.0,
    );

    let mut bot = match bot_result {
        Ok(b) => b,
        Err(e) => {
            println!("Failed to create bot: {}", e);
            return;
        }
    };

    bot.status();

    println!("\n=== Opening positions ===\n");

    // Successful opening
    if let Err(e) = bot.open_position("BTC", 0.1, 42000.0) {
        println!("Error: {}", e);
    }

    if let Err(e) = bot.open_position("ETH", 2.0, 2500.0) {
        println!("Error: {}", e);
    }

    if let Err(e) = bot.open_position("SOL", 50.0, 95.0) {
        println!("Error: {}", e);
    }

    // Fail fast: position limit exceeded
    println!("\nAttempting to open 4th position:");
    if let Err(e) = bot.open_position("DOGE", 1000.0, 0.08) {
        println!("Fail fast: {}", e);
    }

    bot.status();

    // Fail fast: insufficient funds
    println!("\nAttempting to open large position:");
    if let Err(e) = bot.open_position("BTC", 1.0, 42000.0) {
        println!("Fail fast: {}", e);
    }

    println!("\n=== Closing positions ===\n");

    // Successful close
    match bot.close_position("ETH", 2600.0) {
        Ok(proceeds) => println!("Received: ${:.2}", proceeds),
        Err(e) => println!("Error: {}", e),
    }

    // Fail fast: position not found
    println!("\nAttempting to close non-existent position:");
    match bot.close_position("MATIC", 1.0) {
        Ok(proceeds) => println!("Received: ${:.2}", proceeds),
        Err(e) => println!("Fail fast: {}", e),
    }

    bot.status();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Fail Fast | Immediately stop execution on error |
| Constructor validation | Guarantees object validity |
| `?` operator | Tool for fail fast |
| Early return | `return Err(...)` at first problem |
| Collect All | Alternative for UI — gather all errors |

## Homework

1. **Trading Strategy Validator**
   Write a function `validate_strategy()` that checks:
   - Strategy name is not empty
   - Timeframe is valid (1m, 5m, 15m, 1h, 4h, 1d)
   - Risk:Reward is at least 2.0
   - Maximum risk per trade is no more than 3%
   Use fail fast approach with the `?` operator

2. **Exchange Connection Constructor**
   Create a struct `ExchangeConnection` with fields:
   - `exchange_name: String`
   - `api_key: String`
   - `api_secret: String`
   - `rate_limit: u32` (requests per second)
   Implement `new()` with fail fast validation of all fields.

3. **Order Parser from JSON**
   Write a function `parse_order(json: &str) -> Result<Order, OrderParseError>` that:
   - Parses JSON
   - Checks for required fields
   - Validates values
   Use fail fast — return Err at the first parsing error.

4. **Risk Check System**
   Create a `RiskManager` with a `check_trade()` method that:
   - Checks maximum position size
   - Checks daily loss limit
   - Checks correlation with open positions
   If any check fails — fail fast.

## Navigation

[← Previous day](../117-errors-in-async-preview/en.md) | [Next day →](../119-pattern-error-as-value/en.md)
