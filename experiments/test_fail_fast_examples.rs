// Test file for Chapter 118: Fail Fast Pattern
// This file contains all code examples from the chapter to verify they compile

// Example 1: Basic fail fast vs bad approach
fn bad_calculate_position_size(balance: f64, risk_percent: f64) -> f64 {
    if balance <= 0.0 {
        return 0.0;
    }
    balance * risk_percent / 100.0
}

fn good_calculate_position_size(balance: f64, risk_percent: f64) -> Result<f64, String> {
    if balance <= 0.0 {
        return Err(format!("Invalid balance: {}. Balance must be positive", balance));
    }
    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return Err(format!("Invalid risk: {}%. Must be between 0 and 100", risk_percent));
    }
    Ok(balance * risk_percent / 100.0)
}

// Example 2: Order validation
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

fn validate_order(order: &Order, balance: f64) -> Result<(), OrderError> {
    if order.symbol.is_empty() {
        return Err(OrderError::EmptySymbol);
    }
    if order.quantity <= 0.0 {
        return Err(OrderError::InvalidQuantity(order.quantity));
    }
    if order.price <= 0.0 {
        return Err(OrderError::InvalidPrice(order.price));
    }
    let required = order.quantity * order.price;
    if required > balance {
        return Err(OrderError::InsufficientBalance {
            required,
            available: balance,
        });
    }
    Ok(())
}

// Example 3: Trade Signal with ? operator
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

fn create_long_signal(
    symbol: &str,
    entry: f64,
    stop_loss: f64,
    take_profit: f64,
    position_size: f64,
) -> Result<TradeSignal, SignalError> {
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

// Example 4: Portfolio constructor
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
    pub fn new(
        name: String,
        balance: f64,
        max_positions: usize,
        risk_per_trade: f64,
    ) -> Result<Self, PortfolioError> {
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

// Example 5: Collect all errors
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

// Example 6: Trading Bot with HashMap
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

    pub fn open_position(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<(), BotError> {
        if self.positions.len() >= self.max_positions {
            return Err(BotError::TooManyPositions {
                current: self.positions.len(),
                max: self.max_positions,
            });
        }

        let cost = quantity * price;
        if cost > self.balance {
            return Err(BotError::InsufficientBalance {
                required: cost,
                available: self.balance,
            });
        }

        self.balance -= cost;
        *self.positions.entry(symbol.to_string()).or_insert(0.0) += quantity;

        println!("[{}] Opened position: {} {} @ ${:.2}", self.name, quantity, symbol, price);
        Ok(())
    }

    pub fn close_position(&mut self, symbol: &str, price: f64) -> Result<f64, BotError> {
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
    println!("=== Testing Chapter 118 Examples ===\n");

    // Test Example 1
    println!("--- Example 1: Basic fail fast ---");
    let size1 = bad_calculate_position_size(-1000.0, 2.0);
    println!("Position size (bad): {}", size1);

    match good_calculate_position_size(-1000.0, 2.0) {
        Ok(size) => println!("Position size: {}", size),
        Err(e) => println!("Error: {}", e),
    }

    // Test Example 2
    println!("\n--- Example 2: Order validation ---");
    let order = Order {
        symbol: String::new(),
        side: OrderSide::Buy,
        quantity: 0.5,
        price: 42000.0,
    };
    match validate_order(&order, 10000.0) {
        Ok(()) => println!("Order is valid"),
        Err(e) => println!("Fail fast: {}", e),
    }

    // Test Example 3
    println!("\n--- Example 3: Trade Signal with ? ---");
    match create_long_signal("BTC", 42000.0, 41000.0, 45000.0, 0.1) {
        Ok(signal) => println!("Signal created: {:?}", signal),
        Err(e) => println!("Error creating signal: {}", e),
    }

    // Test Example 4
    println!("\n--- Example 4: Portfolio constructor ---");
    match Portfolio::new("Main Portfolio".to_string(), 10000.0, 5, 2.0) {
        Ok(p) => {
            println!("Created: {:?}", p);
            let size = p.calculate_position_size(42000.0, 41000.0);
            println!("Position size: {:.6} BTC", size);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test Example 5
    println!("\n--- Example 5: Collect all errors ---");
    let bad_order = OrderValidation {
        symbol: String::new(),
        quantity: -1.0,
        price: 0.0,
        stop_loss: Some(-100.0),
    };
    match validate_order_collect_all(&bad_order) {
        Ok(()) => println!("Order is valid"),
        Err(errors) => println!("Errors found:\n{}", errors),
    }

    // Test Example 6
    println!("\n--- Example 6: Trading Bot ---");
    let bot_result = TradingBot::new(
        "AlphaBot".to_string(),
        "secret-api-key".to_string(),
        10000.0,
        3,
        2.0,
    );

    if let Ok(mut bot) = bot_result {
        bot.status();
        let _ = bot.open_position("BTC", 0.1, 42000.0);
        bot.status();
    }

    println!("\n=== All examples compiled and ran successfully! ===");
}
