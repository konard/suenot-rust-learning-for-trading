# Day 68: Field Privacy — Hiding Internal State

## Trading Analogy

Imagine you manage a hedge fund. You have a secret trading strategy — the fund's **internal state**. Clients can see the overall portfolio returns, but they don't have access to exact algorithms and positions. This is **encapsulation**: hiding implementation details while exposing only what's necessary.

In Rust, struct fields are **private by default**. This protects data from accidental modification from outside.

## Private by Default

```rust
mod trading {
    pub struct Order {
        id: u64,           // Private — only within module
        pub symbol: String, // Public — accessible everywhere
        pub quantity: f64,  // Public
        internal_fee: f64,  // Private — internal commission
    }

    impl Order {
        pub fn new(id: u64, symbol: String, quantity: f64) -> Self {
            Order {
                id,
                symbol,
                quantity,
                internal_fee: quantity * 0.001, // Internal calculation
            }
        }

        // Public method to access private field
        pub fn id(&self) -> u64 {
            self.id
        }
    }
}

fn main() {
    let order = trading::Order::new(1, String::from("BTC/USD"), 0.5);

    // Works — public fields
    println!("Symbol: {}", order.symbol);
    println!("Quantity: {}", order.quantity);

    // Works — through method
    println!("Order ID: {}", order.id());

    // WON'T COMPILE:
    // println!("Fee: {}", order.internal_fee); // private field!
    // println!("ID: {}", order.id);            // private field!
}
```

## Why Field Privacy Matters

### 1. Protecting Invariants

```rust
mod portfolio {
    pub struct Portfolio {
        balance: f64,      // Private — protect from invalid values
        positions: Vec<Position>,
        risk_limit: f64,
    }

    pub struct Position {
        pub symbol: String,
        pub quantity: f64,
        pub entry_price: f64,
    }

    impl Portfolio {
        pub fn new(initial_balance: f64, risk_limit: f64) -> Self {
            Portfolio {
                balance: initial_balance.max(0.0), // Guarantee non-negative
                positions: Vec::new(),
                risk_limit: risk_limit.clamp(0.0, 1.0), // 0% - 100%
            }
        }

        pub fn balance(&self) -> f64 {
            self.balance
        }

        // Controlled balance modification
        pub fn deposit(&mut self, amount: f64) -> Result<(), String> {
            if amount <= 0.0 {
                return Err(String::from("Deposit must be positive"));
            }
            self.balance += amount;
            Ok(())
        }

        pub fn withdraw(&mut self, amount: f64) -> Result<(), String> {
            if amount <= 0.0 {
                return Err(String::from("Withdrawal must be positive"));
            }
            if amount > self.balance {
                return Err(String::from("Insufficient funds"));
            }
            self.balance -= amount;
            Ok(())
        }

        pub fn open_position(&mut self, symbol: String, quantity: f64, price: f64) -> Result<(), String> {
            let cost = quantity * price;
            let risk = cost / self.balance;

            if risk > self.risk_limit {
                return Err(format!(
                    "Position exceeds risk limit: {:.1}% > {:.1}%",
                    risk * 100.0,
                    self.risk_limit * 100.0
                ));
            }

            if cost > self.balance {
                return Err(String::from("Insufficient balance"));
            }

            self.balance -= cost;
            self.positions.push(Position {
                symbol,
                quantity,
                entry_price: price,
            });

            Ok(())
        }
    }
}

fn main() {
    let mut portfolio = portfolio::Portfolio::new(10000.0, 0.1); // 10% risk limit

    println!("Initial balance: ${:.2}", portfolio.balance());

    // Attempt to open too large a position
    match portfolio.open_position(String::from("BTC/USD"), 1.0, 42000.0) {
        Ok(_) => println!("Position opened"),
        Err(e) => println!("Error: {}", e),
    }

    // Open position within limit
    match portfolio.open_position(String::from("ETH/USD"), 0.5, 1800.0) {
        Ok(_) => println!("ETH position opened"),
        Err(e) => println!("Error: {}", e),
    }

    println!("Final balance: ${:.2}", portfolio.balance());
}
```

### 2. Hiding Implementation Details

```rust
mod pricing {
    pub struct PriceEngine {
        // Internal data structure — hidden
        bid_levels: Vec<(f64, f64)>,  // (price, volume)
        ask_levels: Vec<(f64, f64)>,
        last_update: u64,
    }

    impl PriceEngine {
        pub fn new() -> Self {
            PriceEngine {
                bid_levels: Vec::new(),
                ask_levels: Vec::new(),
                last_update: 0,
            }
        }

        // Public interface — simple and clear
        pub fn best_bid(&self) -> Option<f64> {
            self.bid_levels.first().map(|(price, _)| *price)
        }

        pub fn best_ask(&self) -> Option<f64> {
            self.ask_levels.first().map(|(price, _)| *price)
        }

        pub fn spread(&self) -> Option<f64> {
            match (self.best_ask(), self.best_bid()) {
                (Some(ask), Some(bid)) => Some(ask - bid),
                _ => None,
            }
        }

        pub fn mid_price(&self) -> Option<f64> {
            match (self.best_ask(), self.best_bid()) {
                (Some(ask), Some(bid)) => Some((ask + bid) / 2.0),
                _ => None,
            }
        }

        // Internal update method
        pub fn update_book(&mut self, bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>, timestamp: u64) {
            self.bid_levels = bids;
            self.ask_levels = asks;
            self.last_update = timestamp;

            // Sort: bids descending, asks ascending
            self.bid_levels.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
            self.ask_levels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }
    }
}

fn main() {
    let mut engine = pricing::PriceEngine::new();

    engine.update_book(
        vec![(41950.0, 1.5), (41900.0, 2.0), (41850.0, 3.0)],
        vec![(42000.0, 1.0), (42050.0, 2.5), (42100.0, 1.8)],
        1234567890,
    );

    println!("Best Bid: {:?}", engine.best_bid());
    println!("Best Ask: {:?}", engine.best_ask());
    println!("Spread: {:?}", engine.spread());
    println!("Mid Price: {:?}", engine.mid_price());

    // Cannot directly modify internal data:
    // engine.bid_levels.push((42000.0, 100.0)); // Error!
}
```

### 3. Safe Implementation Changes

```rust
mod risk {
    // Version 1: simple risk calculation
    pub struct RiskCalculator {
        // Can change internal structure without changing API
        base_risk: f64,
        volatility_multiplier: f64,
        // Added new field — external code doesn't break
        market_regime: MarketRegime,
    }

    enum MarketRegime {
        Normal,
        HighVolatility,
        Crisis,
    }

    impl RiskCalculator {
        pub fn new(base_risk: f64) -> Self {
            RiskCalculator {
                base_risk,
                volatility_multiplier: 1.0,
                market_regime: MarketRegime::Normal,
            }
        }

        pub fn calculate_position_risk(&self, position_value: f64, volatility: f64) -> f64 {
            let regime_factor = match self.market_regime {
                MarketRegime::Normal => 1.0,
                MarketRegime::HighVolatility => 1.5,
                MarketRegime::Crisis => 2.0,
            };

            position_value * self.base_risk * volatility * self.volatility_multiplier * regime_factor
        }

        pub fn update_volatility_multiplier(&mut self, multiplier: f64) {
            self.volatility_multiplier = multiplier.max(0.1);
        }

        // Internal method — can add without changing public API
        fn detect_regime(&mut self, vix: f64) {
            self.market_regime = if vix > 30.0 {
                MarketRegime::Crisis
            } else if vix > 20.0 {
                MarketRegime::HighVolatility
            } else {
                MarketRegime::Normal
            };
        }
    }
}

fn main() {
    let mut calc = risk::RiskCalculator::new(0.02);

    let risk = calc.calculate_position_risk(10000.0, 0.15);
    println!("Position risk: ${:.2}", risk);

    calc.update_volatility_multiplier(1.5);
    let risk = calc.calculate_position_risk(10000.0, 0.15);
    println!("Updated risk: ${:.2}", risk);
}
```

## Visibility Levels

```rust
mod exchange {
    pub struct Exchange {
        pub name: String,              // Public — everywhere
        pub(crate) api_version: String, // Within crate
        pub(super) rate_limit: u32,    // In parent module
        api_key: String,               // Private — only here
    }

    pub mod orders {
        use super::Exchange;

        pub fn check_rate_limit(exchange: &Exchange) {
            // Can access pub(super) field
            println!("Rate limit: {}", exchange.rate_limit);
        }
    }

    impl Exchange {
        pub fn new(name: String, api_key: String) -> Self {
            Exchange {
                name,
                api_version: String::from("v3"),
                rate_limit: 100,
                api_key,
            }
        }

        // Safe way to work with API key
        pub fn sign_request(&self, payload: &str) -> String {
            // Use private api_key without exposing it
            format!("signed_{}_{}", payload, self.api_key.len())
        }
    }
}

fn main() {
    let binance = exchange::Exchange::new(
        String::from("Binance"),
        String::from("secret_key_12345"),
    );

    println!("Exchange: {}", binance.name);       // OK — public
    println!("API: {}", binance.api_version);     // OK — pub(crate)

    // binance.api_key — not accessible!

    let signed = binance.sign_request("order_data");
    println!("Signed: {}", signed);

    exchange::orders::check_rate_limit(&binance);
}
```

## Practical Example: Trading Account with Privacy

```rust
mod account {
    use std::collections::HashMap;

    pub struct TradingAccount {
        // Private fields — protected from external modification
        account_id: String,
        balance: f64,
        positions: HashMap<String, Position>,
        trade_history: Vec<Trade>,
        fee_rate: f64,
    }

    #[derive(Clone)]
    pub struct Position {
        pub symbol: String,
        pub quantity: f64,
        pub avg_price: f64,
        pub unrealized_pnl: f64,
    }

    #[derive(Clone)]
    pub struct Trade {
        pub id: u64,
        pub symbol: String,
        pub side: TradeSide,
        pub quantity: f64,
        pub price: f64,
        pub fee: f64,
        pub timestamp: u64,
    }

    #[derive(Clone, Copy)]
    pub enum TradeSide {
        Buy,
        Sell,
    }

    impl TradingAccount {
        pub fn new(account_id: String, initial_balance: f64) -> Self {
            TradingAccount {
                account_id,
                balance: initial_balance,
                positions: HashMap::new(),
                trade_history: Vec::new(),
                fee_rate: 0.001, // 0.1%
            }
        }

        // Getters for private fields
        pub fn account_id(&self) -> &str {
            &self.account_id
        }

        pub fn balance(&self) -> f64 {
            self.balance
        }

        pub fn equity(&self) -> f64 {
            self.balance + self.total_unrealized_pnl()
        }

        pub fn total_unrealized_pnl(&self) -> f64 {
            self.positions.values().map(|p| p.unrealized_pnl).sum()
        }

        // Get copy of positions (not reference to internal data)
        pub fn get_positions(&self) -> Vec<Position> {
            self.positions.values().cloned().collect()
        }

        pub fn get_position(&self, symbol: &str) -> Option<Position> {
            self.positions.get(symbol).cloned()
        }

        pub fn trade_count(&self) -> usize {
            self.trade_history.len()
        }

        // Controlled operations
        pub fn execute_trade(
            &mut self,
            symbol: String,
            side: TradeSide,
            quantity: f64,
            price: f64,
            timestamp: u64,
        ) -> Result<Trade, String> {
            let cost = quantity * price;
            let fee = cost * self.fee_rate;

            match side {
                TradeSide::Buy => {
                    if self.balance < cost + fee {
                        return Err(String::from("Insufficient balance"));
                    }
                    self.balance -= cost + fee;

                    let position = self.positions.entry(symbol.clone()).or_insert(Position {
                        symbol: symbol.clone(),
                        quantity: 0.0,
                        avg_price: 0.0,
                        unrealized_pnl: 0.0,
                    });

                    // Recalculate average price
                    let total_cost = position.quantity * position.avg_price + cost;
                    position.quantity += quantity;
                    position.avg_price = if position.quantity > 0.0 {
                        total_cost / position.quantity
                    } else {
                        0.0
                    };
                }
                TradeSide::Sell => {
                    let position = self.positions.get_mut(&symbol)
                        .ok_or("No position to sell")?;

                    if position.quantity < quantity {
                        return Err(String::from("Insufficient position"));
                    }

                    position.quantity -= quantity;
                    self.balance += cost - fee;

                    if position.quantity == 0.0 {
                        self.positions.remove(&symbol);
                    }
                }
            }

            let trade = Trade {
                id: self.trade_history.len() as u64 + 1,
                symbol,
                side,
                quantity,
                price,
                fee,
                timestamp,
            };

            self.trade_history.push(trade.clone());
            Ok(trade)
        }

        pub fn update_prices(&mut self, prices: &HashMap<String, f64>) {
            for (symbol, position) in self.positions.iter_mut() {
                if let Some(&current_price) = prices.get(symbol) {
                    position.unrealized_pnl =
                        (current_price - position.avg_price) * position.quantity;
                }
            }
        }
    }
}

fn main() {
    use std::collections::HashMap;
    use account::{TradingAccount, TradeSide};

    let mut acc = TradingAccount::new(String::from("ACC001"), 10000.0);

    println!("Account: {}", acc.account_id());
    println!("Initial balance: ${:.2}", acc.balance());

    // Buy BTC
    match acc.execute_trade(
        String::from("BTC/USD"),
        TradeSide::Buy,
        0.1,
        42000.0,
        1000000,
    ) {
        Ok(trade) => println!("Trade executed: {} {} @ ${:.2}",
            trade.quantity, trade.symbol, trade.price),
        Err(e) => println!("Trade failed: {}", e),
    }

    // Buy ETH
    match acc.execute_trade(
        String::from("ETH/USD"),
        TradeSide::Buy,
        1.0,
        1800.0,
        1000001,
    ) {
        Ok(trade) => println!("Trade executed: {} {} @ ${:.2}",
            trade.quantity, trade.symbol, trade.price),
        Err(e) => println!("Trade failed: {}", e),
    }

    println!("\nBalance after trades: ${:.2}", acc.balance());
    println!("Trade count: {}", acc.trade_count());

    // Update prices
    let mut prices = HashMap::new();
    prices.insert(String::from("BTC/USD"), 43000.0); // +$1000
    prices.insert(String::from("ETH/USD"), 1750.0);  // -$50

    acc.update_prices(&prices);

    println!("\n=== Portfolio ===");
    for pos in acc.get_positions() {
        println!("{}: {} @ ${:.2}, PnL: ${:.2}",
            pos.symbol, pos.quantity, pos.avg_price, pos.unrealized_pnl);
    }

    println!("\nTotal Equity: ${:.2}", acc.equity());
    println!("Unrealized PnL: ${:.2}", acc.total_unrealized_pnl());
}
```

## Exercises

### Exercise 1: Protected Wallet

```rust
mod wallet {
    pub struct Wallet {
        // Make fields private and add safe methods
        pub balance: f64,
        pub transactions: Vec<Transaction>,
    }

    pub struct Transaction {
        pub amount: f64,
        pub tx_type: TxType,
    }

    pub enum TxType {
        Deposit,
        Withdrawal,
    }

    // Implement:
    // - new(initial_balance) -> Wallet
    // - balance(&self) -> f64
    // - deposit(&mut self, amount: f64) -> Result<(), String>
    // - withdraw(&mut self, amount: f64) -> Result<(), String>
    // - transaction_count(&self) -> usize
}
```

### Exercise 2: Order Book

```rust
mod orderbook {
    pub struct OrderBook {
        // Private fields
        bids: Vec<Level>,
        asks: Vec<Level>,
    }

    struct Level {
        price: f64,
        quantity: f64,
    }

    // Implement public API:
    // - new() -> OrderBook
    // - add_bid(&mut self, price: f64, quantity: f64)
    // - add_ask(&mut self, price: f64, quantity: f64)
    // - best_bid(&self) -> Option<(f64, f64)>
    // - best_ask(&self) -> Option<(f64, f64)>
    // - spread(&self) -> Option<f64>
    // - depth(&self, levels: usize) -> (Vec<(f64, f64)>, Vec<(f64, f64)>)
}
```

### Exercise 3: Risk Manager

```rust
mod risk_manager {
    pub struct RiskManager {
        // Private configuration
        max_position_size: f64,
        max_daily_loss: f64,
        daily_pnl: f64,
        is_trading_enabled: bool,
    }

    // Implement:
    // - new(max_position: f64, max_loss: f64) -> RiskManager
    // - can_trade(&self) -> bool
    // - check_order(&self, size: f64) -> Result<(), String>
    // - record_pnl(&mut self, pnl: f64)
    // - reset_daily(&mut self)
    // - daily_pnl(&self) -> f64
}
```

## What We Learned

| Concept | Description | Trading Application |
|---------|-------------|-------------------|
| Private fields | Hidden by default | Protecting balances, API keys |
| `pub` | Public access | Tickers, public data |
| `pub(crate)` | Crate-level access | Internal exchange services |
| `pub(super)` | Parent module access | Modular architecture |
| Getters | Controlled access | Safe balance reading |
| Invariants | Correctness guarantees | Non-negative balance |

## Homework

1. **Secret Trading Bot**: Create a `TradingBot` struct with private fields `api_key`, `secret_key`, `strategy_config`. Implement a safe constructor and `sign_request()` method that uses keys without exposing them.

2. **Portfolio with Limits**: Implement `Portfolio` with private `max_positions`, `max_allocation_per_asset`. Methods should check limits before adding positions.

3. **Audit Log**: Create an `AuditLog` struct with a private vector of records. Implement `log_trade()`, `log_error()`, `get_recent(n)` — returning last n records as copies.

4. **Fee Calculator**: Implement `FeeCalculator` with private fee tiers (VIP1, VIP2, VIP3). Method `calculate_fee(volume, vip_level)` should return the fee while hiding exact percentages.

## Navigation

[← Previous day: Unit-like Structs](../067-unit-structs-markers/en.md) | [Next day: Getters and Setters →](../069-getters-setters/en.md)
