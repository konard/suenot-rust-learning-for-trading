# Day 69: Getters and Setters — Controlling Price Access

## Trading Analogy

Imagine an exchange: you cannot directly modify an asset's price in the system. All changes go through **controlled channels** — orders, validation, limit checks. Getters and setters in programming work the same way: they provide **controlled access** to data, allowing you to add validation, logging, and protection.

## Why Use Getters and Setters?

In Rust, struct fields are private by default outside the module. This is good — we protect internal state. But how do we read and modify data? Through special methods:

- **Getter** — a method to read a field's value
- **Setter** — a method to modify a field's value with validation

```rust
fn main() {
    let mut price = Price::new(42000.0);

    // Getter — read the price
    println!("Current price: ${:.2}", price.value());

    // Setter — modify with validation
    if price.set_value(43500.0) {
        println!("Price updated to: ${:.2}", price.value());
    }

    // Attempt to set negative price
    if !price.set_value(-100.0) {
        println!("Invalid price rejected!");
    }
}

struct Price {
    value: f64,
}

impl Price {
    fn new(value: f64) -> Self {
        Price { value: value.max(0.0) }
    }

    // Getter
    fn value(&self) -> f64 {
        self.value
    }

    // Setter with validation
    fn set_value(&mut self, new_value: f64) -> bool {
        if new_value >= 0.0 {
            self.value = new_value;
            true
        } else {
            false
        }
    }
}
```

## Basic Getter Patterns

### Simple Getter — Return a Copy

```rust
struct TradingPair {
    base: String,
    quote: String,
    price: f64,
    volume: f64,
}

impl TradingPair {
    fn new(base: &str, quote: &str, price: f64, volume: f64) -> Self {
        TradingPair {
            base: base.to_string(),
            quote: quote.to_string(),
            price,
            volume,
        }
    }

    // Getters for primitives — return a copy
    fn price(&self) -> f64 {
        self.price
    }

    fn volume(&self) -> f64 {
        self.volume
    }

    // Getters for String — return a reference
    fn base(&self) -> &str {
        &self.base
    }

    fn quote(&self) -> &str {
        &self.quote
    }

    // Computed getter
    fn symbol(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }

    fn market_cap(&self) -> f64 {
        self.price * self.volume
    }
}

fn main() {
    let btc = TradingPair::new("BTC", "USD", 42000.0, 1500.0);

    println!("Pair: {}", btc.symbol());
    println!("Price: ${:.2}", btc.price());
    println!("Volume: {:.2}", btc.volume());
    println!("Market Cap: ${:.0}", btc.market_cap());
}
```

### Reference Getter — Avoid Copying

```rust
fn main() {
    let order = Order::new(
        "ORD-12345",
        "BTC/USD",
        42000.0,
        0.5,
        OrderSide::Buy,
    );

    println!("Order ID: {}", order.id());
    println!("Symbol: {}", order.symbol());
    println!("Side: {:?}", order.side());
    println!("Total: ${:.2}", order.total_value());
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

struct Order {
    id: String,
    symbol: String,
    price: f64,
    quantity: f64,
    side: OrderSide,
}

impl Order {
    fn new(id: &str, symbol: &str, price: f64, quantity: f64, side: OrderSide) -> Self {
        Order {
            id: id.to_string(),
            symbol: symbol.to_string(),
            price,
            quantity,
            side,
        }
    }

    // Return reference to string
    fn id(&self) -> &str {
        &self.id
    }

    fn symbol(&self) -> &str {
        &self.symbol
    }

    // Copy types can be returned by value
    fn price(&self) -> f64 {
        self.price
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn side(&self) -> OrderSide {
        self.side
    }

    // Computed value
    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }
}
```

## Setters with Validation

### Validating Price and Quantity

```rust
fn main() {
    let mut position = Position::new("BTC/USD");

    // Set entry price
    match position.set_entry_price(42000.0) {
        Ok(_) => println!("Entry price set"),
        Err(e) => println!("Error: {}", e),
    }

    // Set quantity
    match position.set_quantity(0.5) {
        Ok(_) => println!("Quantity set to {}", position.quantity()),
        Err(e) => println!("Error: {}", e),
    }

    // Attempt to set negative quantity
    match position.set_quantity(-1.0) {
        Ok(_) => println!("Quantity set"),
        Err(e) => println!("Validation failed: {}", e),
    }

    // Set stop loss
    match position.set_stop_loss(41000.0) {
        Ok(_) => println!("Stop loss set to ${:.2}", position.stop_loss().unwrap()),
        Err(e) => println!("Error: {}", e),
    }

    println!("\nPosition summary:");
    println!("  Symbol: {}", position.symbol());
    println!("  Entry: ${:.2}", position.entry_price().unwrap_or(0.0));
    println!("  Quantity: {:.4}", position.quantity());
    println!("  Stop Loss: ${:.2}", position.stop_loss().unwrap_or(0.0));
}

struct Position {
    symbol: String,
    entry_price: Option<f64>,
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl Position {
    fn new(symbol: &str) -> Self {
        Position {
            symbol: symbol.to_string(),
            entry_price: None,
            quantity: 0.0,
            stop_loss: None,
            take_profit: None,
        }
    }

    // Getters
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn entry_price(&self) -> Option<f64> {
        self.entry_price
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn stop_loss(&self) -> Option<f64> {
        self.stop_loss
    }

    fn take_profit(&self) -> Option<f64> {
        self.take_profit
    }

    // Setters with validation
    fn set_entry_price(&mut self, price: f64) -> Result<(), String> {
        if price <= 0.0 {
            return Err("Entry price must be positive".to_string());
        }
        self.entry_price = Some(price);
        Ok(())
    }

    fn set_quantity(&mut self, qty: f64) -> Result<(), String> {
        if qty < 0.0 {
            return Err("Quantity cannot be negative".to_string());
        }
        self.quantity = qty;
        Ok(())
    }

    fn set_stop_loss(&mut self, price: f64) -> Result<(), String> {
        if price <= 0.0 {
            return Err("Stop loss must be positive".to_string());
        }
        if let Some(entry) = self.entry_price {
            if price >= entry {
                return Err("Stop loss must be below entry price for long position".to_string());
            }
        }
        self.stop_loss = Some(price);
        Ok(())
    }

    fn set_take_profit(&mut self, price: f64) -> Result<(), String> {
        if price <= 0.0 {
            return Err("Take profit must be positive".to_string());
        }
        if let Some(entry) = self.entry_price {
            if price <= entry {
                return Err("Take profit must be above entry price for long position".to_string());
            }
        }
        self.take_profit = Some(price);
        Ok(())
    }
}
```

## Builder Pattern with Setters

```rust
fn main() {
    // Fluent interface — chain of setters
    let order = OrderBuilder::new("BTC/USD")
        .price(42000.0)
        .quantity(0.5)
        .side(OrderSide::Buy)
        .stop_loss(41000.0)
        .take_profit(45000.0)
        .build();

    match order {
        Ok(o) => {
            println!("Order created:");
            println!("  Symbol: {}", o.symbol());
            println!("  Price: ${:.2}", o.price());
            println!("  Quantity: {:.4}", o.quantity());
            println!("  Total: ${:.2}", o.total_value());
        }
        Err(e) => println!("Failed to create order: {}", e),
    }
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
    side: OrderSide,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl Order {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn price(&self) -> f64 {
        self.price
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn side(&self) -> OrderSide {
        self.side
    }

    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }
}

struct OrderBuilder {
    symbol: String,
    price: Option<f64>,
    quantity: Option<f64>,
    side: Option<OrderSide>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

impl OrderBuilder {
    fn new(symbol: &str) -> Self {
        OrderBuilder {
            symbol: symbol.to_string(),
            price: None,
            quantity: None,
            side: None,
            stop_loss: None,
            take_profit: None,
        }
    }

    // Each setter returns self for chaining
    fn price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }

    fn quantity(mut self, qty: f64) -> Self {
        self.quantity = Some(qty);
        self
    }

    fn side(mut self, side: OrderSide) -> Self {
        self.side = Some(side);
        self
    }

    fn stop_loss(mut self, price: f64) -> Self {
        self.stop_loss = Some(price);
        self
    }

    fn take_profit(mut self, price: f64) -> Self {
        self.take_profit = Some(price);
        self
    }

    fn build(self) -> Result<Order, String> {
        let price = self.price.ok_or("Price is required")?;
        let quantity = self.quantity.ok_or("Quantity is required")?;
        let side = self.side.ok_or("Side is required")?;

        if price <= 0.0 {
            return Err("Price must be positive".to_string());
        }
        if quantity <= 0.0 {
            return Err("Quantity must be positive".to_string());
        }

        Ok(Order {
            symbol: self.symbol,
            price,
            quantity,
            side,
            stop_loss: self.stop_loss,
            take_profit: self.take_profit,
        })
    }
}
```

## Getters for Nested Structures

```rust
fn main() {
    let portfolio = Portfolio::new("Main Trading Account");

    // Add positions through method
    let mut portfolio = portfolio;
    portfolio.add_position("BTC/USD", 42000.0, 0.5);
    portfolio.add_position("ETH/USD", 2200.0, 5.0);
    portfolio.add_position("SOL/USD", 100.0, 50.0);

    println!("Portfolio: {}", portfolio.name());
    println!("Positions: {}", portfolio.position_count());
    println!("Total Value: ${:.2}", portfolio.total_value());

    // Access positions through getter
    for pos in portfolio.positions() {
        println!("  {} - ${:.2}", pos.symbol(), pos.value());
    }
}

struct PortfolioPosition {
    symbol: String,
    entry_price: f64,
    quantity: f64,
}

impl PortfolioPosition {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn entry_price(&self) -> f64 {
        self.entry_price
    }

    fn quantity(&self) -> f64 {
        self.quantity
    }

    fn value(&self) -> f64 {
        self.entry_price * self.quantity
    }
}

struct Portfolio {
    name: String,
    positions: Vec<PortfolioPosition>,
}

impl Portfolio {
    fn new(name: &str) -> Self {
        Portfolio {
            name: name.to_string(),
            positions: Vec::new(),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    // Getter returns a slice — immutable access to collection
    fn positions(&self) -> &[PortfolioPosition] {
        &self.positions
    }

    fn position_count(&self) -> usize {
        self.positions.len()
    }

    fn total_value(&self) -> f64 {
        self.positions.iter().map(|p| p.value()).sum()
    }

    // Method for adding (not a classic setter)
    fn add_position(&mut self, symbol: &str, price: f64, qty: f64) {
        self.positions.push(PortfolioPosition {
            symbol: symbol.to_string(),
            entry_price: price,
            quantity: qty,
        });
    }
}
```

## Type-Based Protection — Alternative to Setters

```rust
fn main() {
    // Price cannot be negative — guaranteed by the type!
    let price = match ValidatedPrice::new(42000.0) {
        Some(p) => p,
        None => {
            println!("Invalid price!");
            return;
        }
    };

    println!("Price: ${:.2}", price.value());

    // Quantity with positive value guarantee
    let qty = match PositiveQuantity::new(0.5) {
        Some(q) => q,
        None => {
            println!("Invalid quantity!");
            return;
        }
    };

    // Now we can safely use them
    let order = SafeOrder::new("BTC/USD", price, qty);
    println!("Order value: ${:.2}", order.total_value());
}

// Newtype with validity guarantee
struct ValidatedPrice(f64);

impl ValidatedPrice {
    fn new(value: f64) -> Option<Self> {
        if value > 0.0 {
            Some(ValidatedPrice(value))
        } else {
            None
        }
    }

    fn value(&self) -> f64 {
        self.0
    }
}

struct PositiveQuantity(f64);

impl PositiveQuantity {
    fn new(value: f64) -> Option<Self> {
        if value > 0.0 {
            Some(PositiveQuantity(value))
        } else {
            None
        }
    }

    fn value(&self) -> f64 {
        self.0
    }
}

struct SafeOrder {
    symbol: String,
    price: ValidatedPrice,
    quantity: PositiveQuantity,
}

impl SafeOrder {
    // Accepts already validated types — no setter needed!
    fn new(symbol: &str, price: ValidatedPrice, quantity: PositiveQuantity) -> Self {
        SafeOrder {
            symbol: symbol.to_string(),
            price,
            quantity,
        }
    }

    fn total_value(&self) -> f64 {
        self.price.value() * self.quantity.value()
    }
}
```

## Mutable Access Through Getters

```rust
fn main() {
    let mut account = TradingAccount::new("ACC-001", 10000.0);

    println!("Initial balance: ${:.2}", account.balance());

    // Get mutable reference to settings
    account.settings_mut().max_position_size = 5000.0;
    account.settings_mut().max_leverage = 5.0;

    println!("Max position: ${:.2}", account.settings().max_position_size);
    println!("Max leverage: {}x", account.settings().max_leverage);
}

struct AccountSettings {
    max_position_size: f64,
    max_leverage: f64,
    risk_per_trade: f64,
}

impl Default for AccountSettings {
    fn default() -> Self {
        AccountSettings {
            max_position_size: 1000.0,
            max_leverage: 1.0,
            risk_per_trade: 2.0,
        }
    }
}

struct TradingAccount {
    id: String,
    balance: f64,
    settings: AccountSettings,
}

impl TradingAccount {
    fn new(id: &str, balance: f64) -> Self {
        TradingAccount {
            id: id.to_string(),
            balance,
            settings: AccountSettings::default(),
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn balance(&self) -> f64 {
        self.balance
    }

    // Immutable getter for settings
    fn settings(&self) -> &AccountSettings {
        &self.settings
    }

    // Mutable getter — allows modifying settings
    fn settings_mut(&mut self) -> &mut AccountSettings {
        &mut self.settings
    }
}
```

## Practical Example: Risk Manager

```rust
fn main() {
    let mut risk_manager = RiskManager::new(10000.0);

    // Configure risk parameters
    risk_manager.set_max_risk_per_trade(2.0).unwrap();
    risk_manager.set_max_daily_loss(5.0).unwrap();
    risk_manager.set_max_open_positions(5).unwrap();

    println!("Risk Manager Settings:");
    println!("  Max risk per trade: {}%", risk_manager.max_risk_per_trade());
    println!("  Max daily loss: {}%", risk_manager.max_daily_loss());
    println!("  Max positions: {}", risk_manager.max_open_positions());

    // Check trade
    let trade_risk = 150.0; // $150 risk
    if risk_manager.can_take_trade(trade_risk) {
        println!("\nTrade approved!");
        risk_manager.record_trade(trade_risk);
    }

    // Calculate position size
    let entry = 42000.0;
    let stop_loss = 41000.0;
    let position_size = risk_manager.calculate_position_size(entry, stop_loss);
    println!("Recommended position size: {:.4} units", position_size);

    // Daily status
    println!("\nDaily Stats:");
    println!("  Trades taken: {}", risk_manager.trades_today());
    println!("  Risk used: ${:.2}", risk_manager.daily_risk_used());
    println!("  Remaining risk: ${:.2}", risk_manager.remaining_daily_risk());
}

struct RiskManager {
    account_balance: f64,
    max_risk_per_trade: f64,     // percentage
    max_daily_loss: f64,         // percentage
    max_open_positions: usize,
    daily_risk_used: f64,
    trades_today: usize,
}

impl RiskManager {
    fn new(balance: f64) -> Self {
        RiskManager {
            account_balance: balance,
            max_risk_per_trade: 1.0,
            max_daily_loss: 3.0,
            max_open_positions: 3,
            daily_risk_used: 0.0,
            trades_today: 0,
        }
    }

    // Getters
    fn account_balance(&self) -> f64 {
        self.account_balance
    }

    fn max_risk_per_trade(&self) -> f64 {
        self.max_risk_per_trade
    }

    fn max_daily_loss(&self) -> f64 {
        self.max_daily_loss
    }

    fn max_open_positions(&self) -> usize {
        self.max_open_positions
    }

    fn daily_risk_used(&self) -> f64 {
        self.daily_risk_used
    }

    fn trades_today(&self) -> usize {
        self.trades_today
    }

    // Computed getters
    fn max_risk_amount(&self) -> f64 {
        self.account_balance * (self.max_risk_per_trade / 100.0)
    }

    fn max_daily_loss_amount(&self) -> f64 {
        self.account_balance * (self.max_daily_loss / 100.0)
    }

    fn remaining_daily_risk(&self) -> f64 {
        self.max_daily_loss_amount() - self.daily_risk_used
    }

    // Setters with validation
    fn set_max_risk_per_trade(&mut self, percent: f64) -> Result<(), String> {
        if percent <= 0.0 || percent > 10.0 {
            return Err("Risk per trade must be between 0 and 10%".to_string());
        }
        self.max_risk_per_trade = percent;
        Ok(())
    }

    fn set_max_daily_loss(&mut self, percent: f64) -> Result<(), String> {
        if percent <= 0.0 || percent > 20.0 {
            return Err("Daily loss limit must be between 0 and 20%".to_string());
        }
        if percent < self.max_risk_per_trade {
            return Err("Daily loss must be >= risk per trade".to_string());
        }
        self.max_daily_loss = percent;
        Ok(())
    }

    fn set_max_open_positions(&mut self, count: usize) -> Result<(), String> {
        if count == 0 || count > 20 {
            return Err("Position count must be between 1 and 20".to_string());
        }
        self.max_open_positions = count;
        Ok(())
    }

    // Business logic
    fn can_take_trade(&self, risk_amount: f64) -> bool {
        risk_amount <= self.max_risk_amount() &&
        self.daily_risk_used + risk_amount <= self.max_daily_loss_amount() &&
        self.trades_today < self.max_open_positions
    }

    fn record_trade(&mut self, risk_amount: f64) {
        self.daily_risk_used += risk_amount;
        self.trades_today += 1;
    }

    fn calculate_position_size(&self, entry: f64, stop_loss: f64) -> f64 {
        let risk_per_unit = (entry - stop_loss).abs();
        if risk_per_unit == 0.0 {
            return 0.0;
        }
        self.max_risk_amount() / risk_per_unit
    }

    fn reset_daily_stats(&mut self) {
        self.daily_risk_used = 0.0;
        self.trades_today = 0;
    }
}
```

## What We Learned

| Pattern | Example | When to Use |
|---------|---------|-------------|
| Simple getter | `fn value(&self) -> f64` | Reading primitives |
| Reference getter | `fn name(&self) -> &str` | Strings, large objects |
| Option getter | `fn price(&self) -> Option<f64>` | Optional fields |
| Computed getter | `fn total(&self) -> f64` | Derived values |
| Bool setter | `fn set(&mut self, v: T) -> bool` | Simple validation |
| Result setter | `fn set(&mut self, v: T) -> Result<(), E>` | Detailed errors |
| Builder setter | `fn price(mut self, v: f64) -> Self` | Fluent interface |
| Mutable getter | `fn data_mut(&mut self) -> &mut T` | Access to nested data |

## Homework

1. Create a `TradeJournal` struct with getters for statistics (win rate, average profit, total trades) and a setter for adding trades with validation

2. Implement `PriceAlert` with getters for current price, target price, and direction (above/below), and a setter for changing target price with validation

3. Write a `PositionSizer` with configurable parameters (risk per trade, max position size) through validated setters and a position size calculation method

4. Create an `OrderBook` with getters for best bid/ask, spread, and depth, with methods for adding and removing orders

## Navigation

[← Previous day](../068-privacy-visibility/en.md) | [Next day →](../070-mutable-references-in-structs/en.md)
