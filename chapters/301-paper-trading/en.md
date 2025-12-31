# Day 301: Paper Trading: Virtual Trading

## Trading Analogy

Imagine a beginner trader who wants to test a new strategy but doesn't want to risk real money. They open a demo account with virtual funds and start making trades. All operations look real: opening positions, stop-losses, profit-taking, but the money is virtual. This is **paper trading** — practice without financial risk.

Like a pilot training on a simulator before flying a real plane:
- All instruments work realistically
- Can practice emergency scenarios
- Mistakes don't cost lives (or money)
- Can repeat the same situation many times
- Builds confidence before real trading

In algorithmic trading, paper trading is a critical step between backtesting on historical data and live trading with real money.

## Why Paper Trading is Essential

| Stage | Purpose | Risk Level |
|-------|---------|------------|
| **Backtesting** | Test on historical data | No risk (past data) |
| **Paper Trading** | Test in real-time market | No financial risk |
| **Live Trading** | Real money, real trades | Full financial risk |

Paper trading allows you to:
1. Test strategy in real market conditions
2. Debug code in production-like environment
3. Practice order execution and management
4. Test API integration without risk
5. Build psychological readiness for real trading
6. Validate strategy performance beyond backtests

## Basic Paper Trading Account

```rust
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct PaperAccount {
    balance: f64,
    initial_balance: f64,
    positions: HashMap<String, Position>,
    trade_history: Vec<Trade>,
    commission_rate: f64, // Commission in percentage
}

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    entry_time: u64,
    side: Side,
}

#[derive(Debug, Clone, PartialEq)]
enum Side {
    Long,
    Short,
}

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    side: Side,
    quantity: f64,
    entry_price: f64,
    exit_price: f64,
    entry_time: u64,
    exit_time: u64,
    pnl: f64,
    commission: f64,
}

impl PaperAccount {
    fn new(initial_balance: f64, commission_rate: f64) -> Self {
        Self {
            balance: initial_balance,
            initial_balance,
            positions: HashMap::new(),
            trade_history: Vec::new(),
            commission_rate,
        }
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Open a long position
    fn open_long(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<String, String> {
        if self.positions.contains_key(symbol) {
            return Err(format!("Position for {} already exists", symbol));
        }

        let cost = quantity * price;
        let commission = cost * self.commission_rate;
        let total_cost = cost + commission;

        if total_cost > self.balance {
            return Err(format!(
                "Insufficient balance: need {:.2}, have {:.2}",
                total_cost, self.balance
            ));
        }

        self.balance -= total_cost;

        self.positions.insert(
            symbol.to_string(),
            Position {
                symbol: symbol.to_string(),
                quantity,
                entry_price: price,
                entry_time: Self::get_timestamp(),
                side: Side::Long,
            },
        );

        Ok(format!(
            "Opened LONG {} @ {:.2} (qty: {}, cost: {:.2}, commission: {:.2})",
            symbol, price, quantity, cost, commission
        ))
    }

    /// Open a short position
    fn open_short(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<String, String> {
        if self.positions.contains_key(symbol) {
            return Err(format!("Position for {} already exists", symbol));
        }

        let proceeds = quantity * price;
        let commission = proceeds * self.commission_rate;

        // For short, we receive proceeds but pay commission
        self.balance += proceeds - commission;

        self.positions.insert(
            symbol.to_string(),
            Position {
                symbol: symbol.to_string(),
                quantity,
                entry_price: price,
                entry_time: Self::get_timestamp(),
                side: Side::Short,
            },
        );

        Ok(format!(
            "Opened SHORT {} @ {:.2} (qty: {}, proceeds: {:.2}, commission: {:.2})",
            symbol, price, quantity, proceeds, commission
        ))
    }

    /// Close a position
    fn close_position(&mut self, symbol: &str, exit_price: f64) -> Result<String, String> {
        let position = self
            .positions
            .remove(symbol)
            .ok_or(format!("No position for {}", symbol))?;

        let exit_time = Self::get_timestamp();

        let (pnl, commission) = match position.side {
            Side::Long => {
                // Long: profit when price goes up
                let proceeds = position.quantity * exit_price;
                let commission = proceeds * self.commission_rate;
                let pnl = proceeds - (position.quantity * position.entry_price);
                self.balance += proceeds - commission;
                (pnl, commission)
            }
            Side::Short => {
                // Short: profit when price goes down
                let cost = position.quantity * exit_price;
                let commission = cost * self.commission_rate;
                let pnl = (position.quantity * position.entry_price) - cost;
                self.balance -= cost + commission;
                (pnl, commission)
            }
        };

        let trade = Trade {
            symbol: symbol.to_string(),
            side: position.side.clone(),
            quantity: position.quantity,
            entry_price: position.entry_price,
            exit_price,
            entry_time: position.entry_time,
            exit_time,
            pnl,
            commission,
        };

        self.trade_history.push(trade.clone());

        Ok(format!(
            "Closed {} {} @ {:.2} (entry: {:.2}, PnL: {:.2}, commission: {:.2})",
            if position.side == Side::Long {
                "LONG"
            } else {
                "SHORT"
            },
            symbol,
            exit_price,
            position.entry_price,
            pnl,
            commission
        ))
    }

    /// Get current unrealized PnL for all positions
    fn get_unrealized_pnl(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.positions
            .iter()
            .map(|(symbol, position)| {
                if let Some(&current_price) = current_prices.get(symbol) {
                    match position.side {
                        Side::Long => {
                            position.quantity * (current_price - position.entry_price)
                        }
                        Side::Short => {
                            position.quantity * (position.entry_price - current_price)
                        }
                    }
                } else {
                    0.0
                }
            })
            .sum()
    }

    /// Get total account value (balance + unrealized PnL)
    fn get_total_value(&self, current_prices: &HashMap<String, f64>) -> f64 {
        self.balance + self.get_unrealized_pnl(current_prices)
    }

    /// Get account statistics
    fn get_stats(&self) -> String {
        if self.trade_history.is_empty() {
            return "No trades yet".to_string();
        }

        let total_trades = self.trade_history.len();
        let winning_trades = self
            .trade_history
            .iter()
            .filter(|t| t.pnl > 0.0)
            .count();
        let losing_trades = total_trades - winning_trades;

        let total_pnl: f64 = self.trade_history.iter().map(|t| t.pnl).sum();
        let total_commission: f64 = self.trade_history.iter().map(|t| t.commission).sum();
        let net_pnl = total_pnl - total_commission;

        let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
        let roi = (net_pnl / self.initial_balance) * 100.0;

        format!(
            "=== Paper Trading Statistics ===\n\
             Initial Balance: ${:.2}\n\
             Current Balance: ${:.2}\n\
             Total Trades: {}\n\
             Winning Trades: {} ({:.1}%)\n\
             Losing Trades: {}\n\
             Total PnL: ${:.2}\n\
             Total Commission: ${:.2}\n\
             Net PnL: ${:.2}\n\
             ROI: {:.2}%",
            self.initial_balance,
            self.balance,
            total_trades,
            winning_trades,
            win_rate,
            losing_trades,
            total_pnl,
            total_commission,
            net_pnl,
            roi
        )
    }
}

fn main() {
    let mut account = PaperAccount::new(10000.0, 0.001); // $10,000 with 0.1% commission

    println!("=== Starting Paper Trading ===");
    println!("Initial Balance: ${:.2}\n", account.balance);

    // Simulate some trades
    println!("{}", account.open_long("BTC", 0.5, 42000.0).unwrap());
    println!("{}", account.open_short("ETH", 10.0, 2500.0).unwrap());
    println!();

    // Check positions
    let mut current_prices = HashMap::new();
    current_prices.insert("BTC".to_string(), 43000.0);
    current_prices.insert("ETH".to_string(), 2450.0);

    println!("Current Balance: ${:.2}", account.balance);
    println!(
        "Unrealized PnL: ${:.2}",
        account.get_unrealized_pnl(&current_prices)
    );
    println!(
        "Total Account Value: ${:.2}\n",
        account.get_total_value(&current_prices)
    );

    // Close positions
    println!("{}", account.close_position("BTC", 43000.0).unwrap());
    println!("{}", account.close_position("ETH", 2450.0).unwrap());
    println!();

    // Show statistics
    println!("{}", account.get_stats());
}
```

## Order Management System

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: Side,
    order_type: OrderType,
    quantity: f64,
    price: Option<f64>, // For limit/stop orders
    status: OrderStatus,
    created_at: u64,
    filled_at: Option<u64>,
}

struct OrderBook {
    next_order_id: u64,
    pending_orders: VecDeque<Order>,
    filled_orders: Vec<Order>,
}

impl OrderBook {
    fn new() -> Self {
        Self {
            next_order_id: 1,
            pending_orders: VecDeque::new(),
            filled_orders: Vec::new(),
        }
    }

    fn place_order(
        &mut self,
        symbol: &str,
        side: Side,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
    ) -> u64 {
        let order = Order {
            id: self.next_order_id,
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity,
            price,
            status: OrderStatus::Pending,
            created_at: PaperAccount::get_timestamp(),
            filled_at: None,
        };

        self.next_order_id += 1;
        self.pending_orders.push_back(order);
        self.next_order_id - 1
    }

    fn process_orders(
        &mut self,
        account: &mut PaperAccount,
        current_prices: &HashMap<String, f64>,
    ) {
        let mut to_fill = Vec::new();

        for order in &self.pending_orders {
            if let Some(&current_price) = current_prices.get(&order.symbol) {
                let should_fill = match order.order_type {
                    OrderType::Market => true,
                    OrderType::Limit => {
                        if let Some(limit_price) = order.price {
                            match order.side {
                                Side::Long => current_price <= limit_price,
                                Side::Short => current_price >= limit_price,
                            }
                        } else {
                            false
                        }
                    }
                    OrderType::StopLoss => {
                        if let Some(stop_price) = order.price {
                            match order.side {
                                Side::Long => current_price >= stop_price,
                                Side::Short => current_price <= stop_price,
                            }
                        } else {
                            false
                        }
                    }
                    OrderType::TakeProfit => {
                        if let Some(take_profit_price) = order.price {
                            match order.side {
                                Side::Long => current_price >= take_profit_price,
                                Side::Short => current_price <= take_profit_price,
                            }
                        } else {
                            false
                        }
                    }
                };

                if should_fill {
                    to_fill.push(order.id);
                }
            }
        }

        // Fill matched orders
        for order_id in to_fill {
            if let Some(pos) = self
                .pending_orders
                .iter()
                .position(|o| o.id == order_id)
            {
                let mut order = self.pending_orders.remove(pos).unwrap();
                let current_price = current_prices[&order.symbol];

                let result = match order.side {
                    Side::Long => account.open_long(&order.symbol, order.quantity, current_price),
                    Side::Short => {
                        account.open_short(&order.symbol, order.quantity, current_price)
                    }
                };

                match result {
                    Ok(msg) => {
                        order.status = OrderStatus::Filled;
                        order.filled_at = Some(PaperAccount::get_timestamp());
                        println!("Order #{} filled: {}", order.id, msg);
                    }
                    Err(err) => {
                        order.status = OrderStatus::Rejected;
                        println!("Order #{} rejected: {}", order.id, err);
                    }
                }

                self.filled_orders.push(order);
            }
        }
    }

    fn cancel_order(&mut self, order_id: u64) -> Result<(), String> {
        if let Some(pos) = self
            .pending_orders
            .iter()
            .position(|o| o.id == order_id)
        {
            let mut order = self.pending_orders.remove(pos).unwrap();
            order.status = OrderStatus::Cancelled;
            self.filled_orders.push(order);
            Ok(())
        } else {
            Err(format!("Order {} not found in pending orders", order_id))
        }
    }

    fn get_pending_count(&self) -> usize {
        self.pending_orders.len()
    }
}

fn main() {
    let mut account = PaperAccount::new(10000.0, 0.001);
    let mut order_book = OrderBook::new();

    println!("=== Paper Trading with Order Book ===\n");

    // Place some orders
    let order1 = order_book.place_order("BTC", Side::Long, OrderType::Limit, 0.5, Some(42000.0));
    let order2 = order_book.place_order("ETH", Side::Short, OrderType::Market, 10.0, None);
    let order3 = order_book.place_order("BTC", Side::Long, OrderType::Limit, 0.3, Some(41500.0));

    println!("Placed {} orders\n", order_book.get_pending_count());

    // Simulate price updates
    let mut prices = HashMap::new();
    prices.insert("BTC".to_string(), 42500.0);
    prices.insert("ETH".to_string(), 2500.0);

    println!("Processing orders at BTC: $42,500, ETH: $2,500");
    order_book.process_orders(&mut account, &prices);
    println!("Pending orders: {}\n", order_book.get_pending_count());

    // Price drops
    prices.insert("BTC".to_string(), 41800.0);
    println!("Processing orders at BTC: $41,800");
    order_book.process_orders(&mut account, &prices);
    println!("Pending orders: {}\n", order_book.get_pending_count());

    // Cancel remaining order
    if let Err(e) = order_book.cancel_order(order3) {
        println!("Failed to cancel: {}", e);
    } else {
        println!("Order #{} cancelled", order3);
    }

    println!("\n{}", account.get_stats());
}
```

## Risk Management in Paper Trading

```rust
#[derive(Debug)]
struct RiskManager {
    max_position_size: f64,     // Maximum % of account per position
    max_total_exposure: f64,    // Maximum % of account in all positions
    max_drawdown: f64,          // Maximum % loss from peak
    daily_loss_limit: f64,      // Maximum daily loss %
    peak_balance: f64,
    daily_start_balance: f64,
}

impl RiskManager {
    fn new(
        max_position_size: f64,
        max_total_exposure: f64,
        max_drawdown: f64,
        daily_loss_limit: f64,
        initial_balance: f64,
    ) -> Self {
        Self {
            max_position_size,
            max_total_exposure,
            max_drawdown,
            daily_loss_limit,
            peak_balance: initial_balance,
            daily_start_balance: initial_balance,
        }
    }

    fn update_peak(&mut self, current_balance: f64) {
        if current_balance > self.peak_balance {
            self.peak_balance = current_balance;
        }
    }

    fn reset_daily(&mut self, current_balance: f64) {
        self.daily_start_balance = current_balance;
    }

    fn check_position_size(
        &self,
        quantity: f64,
        price: f64,
        account_balance: f64,
    ) -> Result<(), String> {
        let position_value = quantity * price;
        let position_pct = (position_value / account_balance) * 100.0;

        if position_pct > self.max_position_size {
            return Err(format!(
                "Position size {:.1}% exceeds limit {:.1}%",
                position_pct, self.max_position_size
            ));
        }

        Ok(())
    }

    fn check_total_exposure(
        &self,
        account: &PaperAccount,
        current_prices: &HashMap<String, f64>,
    ) -> Result<(), String> {
        let total_exposure: f64 = account
            .positions
            .iter()
            .map(|(symbol, position)| {
                if let Some(&price) = current_prices.get(symbol) {
                    position.quantity * price
                } else {
                    0.0
                }
            })
            .sum();

        let exposure_pct = (total_exposure / account.balance) * 100.0;

        if exposure_pct > self.max_total_exposure {
            return Err(format!(
                "Total exposure {:.1}% exceeds limit {:.1}%",
                exposure_pct, self.max_total_exposure
            ));
        }

        Ok(())
    }

    fn check_drawdown(&self, current_balance: f64) -> Result<(), String> {
        let drawdown = ((self.peak_balance - current_balance) / self.peak_balance) * 100.0;

        if drawdown > self.max_drawdown {
            return Err(format!(
                "Drawdown {:.1}% exceeds limit {:.1}%",
                drawdown, self.max_drawdown
            ));
        }

        Ok(())
    }

    fn check_daily_loss(&self, current_balance: f64) -> Result<(), String> {
        let daily_loss =
            ((self.daily_start_balance - current_balance) / self.daily_start_balance) * 100.0;

        if daily_loss > self.daily_loss_limit {
            return Err(format!(
                "Daily loss {:.1}% exceeds limit {:.1}%",
                daily_loss, self.daily_loss_limit
            ));
        }

        Ok(())
    }

    fn validate_trade(
        &mut self,
        account: &PaperAccount,
        quantity: f64,
        price: f64,
        current_prices: &HashMap<String, f64>,
    ) -> Result<(), String> {
        self.update_peak(account.balance);

        self.check_position_size(quantity, price, account.balance)?;
        self.check_total_exposure(account, current_prices)?;
        self.check_drawdown(account.balance)?;
        self.check_daily_loss(account.balance)?;

        Ok(())
    }
}

fn main() {
    let mut account = PaperAccount::new(10000.0, 0.001);
    let mut risk_manager = RiskManager::new(
        10.0, // Max 10% per position
        50.0, // Max 50% total exposure
        20.0, // Max 20% drawdown
        5.0,  // Max 5% daily loss
        10000.0,
    );

    let mut prices = HashMap::new();
    prices.insert("BTC".to_string(), 42000.0);

    println!("=== Paper Trading with Risk Management ===\n");

    // Try to open a position
    let quantity = 0.5;
    let price = 42000.0;

    match risk_manager.validate_trade(&account, quantity, price, &prices) {
        Ok(_) => {
            println!("Risk check passed!");
            if let Ok(msg) = account.open_long("BTC", quantity, price) {
                println!("{}", msg);
            }
        }
        Err(e) => println!("Risk check failed: {}", e),
    }

    // Try to open too large position
    let large_quantity = 5.0; // Would be ~200% of account
    match risk_manager.validate_trade(&account, large_quantity, price, &prices) {
        Ok(_) => println!("Large position approved (shouldn't happen)"),
        Err(e) => println!("\nLarge position rejected: {}", e),
    }

    println!("\n{}", account.get_stats());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Paper Trading** | Risk-free trading with virtual money |
| **Position Management** | Opening and closing long/short positions |
| **Order Types** | Market, limit, stop-loss, take-profit orders |
| **Order Book** | Managing pending and filled orders |
| **PnL Calculation** | Realized and unrealized profit/loss |
| **Risk Management** | Position sizing, exposure limits, drawdown control |
| **Commission** | Transaction costs in virtual trading |
| **Trade Statistics** | Win rate, ROI, total trades tracking |

## Homework

1. **Enhanced Paper Trading Account**: Extend the basic paper trading account with:
   - Support for multiple currencies (USD, EUR, BTC)
   - Margin trading with leverage (2x, 5x, 10x)
   - Liquidation when margin falls below maintenance level
   - Interest calculation on borrowed funds
   - Detailed trade journal with notes

2. **Advanced Order Types**: Implement additional order types:
   - OCO (One-Cancels-Other): Two orders where filling one cancels the other
   - Trailing stop-loss: Stop that moves with profitable price
   - Iceberg orders: Large order split into smaller visible chunks
   - Time-in-force: GTC (Good-Till-Cancel), IOC (Immediate-Or-Cancel), FOK (Fill-Or-Kill)

3. **Portfolio Rebalancing**: Create a system that:
   - Maintains target allocation percentages (e.g., 60% BTC, 30% ETH, 10% stablecoins)
   - Automatically rebalances when allocations drift by >5%
   - Minimizes transaction costs during rebalancing
   - Supports rebalancing on schedule (daily, weekly) or on threshold

4. **Performance Analytics**: Build a comprehensive analytics module:
   - Sharpe ratio calculation (risk-adjusted returns)
   - Maximum drawdown and recovery time
   - Win/loss streaks tracking
   - Average profit per trade vs. average loss
   - Profit factor (gross profit / gross loss)
   - Export trading history to CSV for external analysis
   - Generate equity curve chart data

## Navigation

[← Previous day](../294-overfitting-strategy-optimization/en.md)
