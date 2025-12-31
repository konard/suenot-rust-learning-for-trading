# Day 281: Commissions: Accounting for Costs

## Trading Analogy

Imagine you decide to sell your old phone through a marketplace app. The buyer is ready to pay $100. You're excited, but then you discover: the platform takes a 5% fee, and the bank charges another 1% for the transfer. In the end, you receive not $100, but $94. **Fees ate 6% of your profit!**

Trading on exchanges works exactly the same way. Every trade costs money:
- **Exchange trading fee** — typically 0.1-0.5% of trade volume
- **Maker/Taker fee** — different rates for adding/removing liquidity
- **Spread** — hidden cost in the bid/ask difference
- **Withdrawal fee** — fixed amount for transferring funds

In backtesting, we **must** account for these costs, otherwise we'll get unrealistic results. A strategy showing +20% profit without commissions might actually be losing money in reality!

## Types of Commissions

```rust
/// Exchange commission model
#[derive(Debug, Clone)]
pub struct CommissionModel {
    /// Commission percentage for taker (removes liquidity)
    pub taker_fee_percent: f64,
    /// Commission percentage for maker (adds liquidity)
    pub maker_fee_percent: f64,
    /// Minimum fee per trade
    pub min_fee: f64,
    /// Maximum fee per trade (if capped)
    pub max_fee: Option<f64>,
}

impl CommissionModel {
    /// Creates a model with the same commission for all order types
    pub fn flat(fee_percent: f64) -> Self {
        CommissionModel {
            taker_fee_percent: fee_percent,
            maker_fee_percent: fee_percent,
            min_fee: 0.0,
            max_fee: None,
        }
    }

    /// Binance Spot commissions (standard tier)
    pub fn binance_spot() -> Self {
        CommissionModel {
            taker_fee_percent: 0.1,  // 0.1%
            maker_fee_percent: 0.1,  // 0.1%
            min_fee: 0.0,
            max_fee: None,
        }
    }

    /// Binance Futures commissions
    pub fn binance_futures() -> Self {
        CommissionModel {
            taker_fee_percent: 0.04,  // 0.04%
            maker_fee_percent: 0.02,  // 0.02%
            min_fee: 0.0,
            max_fee: None,
        }
    }

    /// Bybit commissions
    pub fn bybit() -> Self {
        CommissionModel {
            taker_fee_percent: 0.075,
            maker_fee_percent: 0.025,
            min_fee: 0.0,
            max_fee: None,
        }
    }

    /// Calculates commission for a trade
    pub fn calculate(&self, trade_value: f64, is_taker: bool) -> f64 {
        let fee_percent = if is_taker {
            self.taker_fee_percent
        } else {
            self.maker_fee_percent
        };

        let fee = trade_value * (fee_percent / 100.0);

        // Apply minimum fee
        let fee = fee.max(self.min_fee);

        // Apply maximum fee if set
        match self.max_fee {
            Some(max) => fee.min(max),
            None => fee,
        }
    }
}

fn main() {
    let binance = CommissionModel::binance_spot();

    // Buying BTC worth $10,000
    let trade_value = 10_000.0;
    let commission = binance.calculate(trade_value, true);

    println!("Trade value: ${:.2}", trade_value);
    println!("Commission (taker): ${:.2}", commission);
    println!("Percentage: {:.3}%", (commission / trade_value) * 100.0);
}
```

## Impact of Commissions on Results

```rust
#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: String,
    pub side: TradeSide,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub is_taker_entry: bool,
    pub is_taker_exit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSide {
    Long,
    Short,
}

impl Trade {
    /// Calculate profit WITHOUT commissions
    pub fn gross_profit(&self) -> f64 {
        let price_diff = self.exit_price - self.entry_price;
        match self.side {
            TradeSide::Long => price_diff * self.quantity,
            TradeSide::Short => -price_diff * self.quantity,
        }
    }

    /// Calculate profit WITH commissions
    pub fn net_profit(&self, commission: &CommissionModel) -> f64 {
        let gross = self.gross_profit();

        // Entry commission
        let entry_value = self.entry_price * self.quantity;
        let entry_fee = commission.calculate(entry_value, self.is_taker_entry);

        // Exit commission
        let exit_value = self.exit_price * self.quantity;
        let exit_fee = commission.calculate(exit_value, self.is_taker_exit);

        // Net profit = Gross - commissions
        gross - entry_fee - exit_fee
    }

    /// Total commission for the trade
    pub fn total_commission(&self, commission: &CommissionModel) -> f64 {
        let entry_value = self.entry_price * self.quantity;
        let entry_fee = commission.calculate(entry_value, self.is_taker_entry);

        let exit_value = self.exit_price * self.quantity;
        let exit_fee = commission.calculate(exit_value, self.is_taker_exit);

        entry_fee + exit_fee
    }
}

fn main() {
    let commission = CommissionModel::binance_spot();

    let trade = Trade {
        symbol: "BTC/USDT".to_string(),
        side: TradeSide::Long,
        entry_price: 40_000.0,
        exit_price: 40_400.0,  // +1% move
        quantity: 0.25,         // 0.25 BTC
        is_taker_entry: true,
        is_taker_exit: true,
    };

    let gross = trade.gross_profit();
    let net = trade.net_profit(&commission);
    let fees = trade.total_commission(&commission);

    println!("═══════════════════════════════════════");
    println!("           TRADE ANALYSIS");
    println!("═══════════════════════════════════════");
    println!("Symbol: {}", trade.symbol);
    println!("Direction: {:?}", trade.side);
    println!("Entry: ${:.2} x {:.4}", trade.entry_price, trade.quantity);
    println!("Exit: ${:.2} x {:.4}", trade.exit_price, trade.quantity);
    println!("───────────────────────────────────────");
    println!("Gross profit: ${:.2}", gross);
    println!("Commissions:  ${:.2}", fees);
    println!("Net profit:   ${:.2}", net);
    println!("───────────────────────────────────────");
    println!("Fees consumed: {:.1}% of gross profit", (fees / gross) * 100.0);
    println!("═══════════════════════════════════════");
}
```

## Commission Tracker in Backtester

```rust
use std::collections::HashMap;

/// Commission statistics for a period
#[derive(Debug, Default, Clone)]
pub struct CommissionTracker {
    /// Total fees
    total_fees: f64,
    /// Fees by symbol
    fees_by_symbol: HashMap<String, f64>,
    /// Trade count
    trade_count: u64,
    /// Fee history for analysis
    fee_history: Vec<FeeRecord>,
}

#[derive(Debug, Clone)]
pub struct FeeRecord {
    pub timestamp: u64,
    pub symbol: String,
    pub trade_value: f64,
    pub fee_amount: f64,
    pub fee_type: FeeType,
}

#[derive(Debug, Clone, Copy)]
pub enum FeeType {
    Entry,
    Exit,
}

impl CommissionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a fee
    pub fn record_fee(
        &mut self,
        timestamp: u64,
        symbol: &str,
        trade_value: f64,
        fee_amount: f64,
        fee_type: FeeType,
    ) {
        self.total_fees += fee_amount;
        *self.fees_by_symbol.entry(symbol.to_string()).or_default() += fee_amount;
        self.trade_count += 1;

        self.fee_history.push(FeeRecord {
            timestamp,
            symbol: symbol.to_string(),
            trade_value,
            fee_amount,
            fee_type,
        });
    }

    /// Total fees
    pub fn total_fees(&self) -> f64 {
        self.total_fees
    }

    /// Average fee per trade
    pub fn average_fee(&self) -> f64 {
        if self.trade_count == 0 {
            0.0
        } else {
            self.total_fees / self.trade_count as f64
        }
    }

    /// Fees for a symbol
    pub fn fees_for_symbol(&self, symbol: &str) -> f64 {
        self.fees_by_symbol.get(symbol).copied().unwrap_or(0.0)
    }

    /// Top-N symbols by fees
    pub fn top_symbols_by_fees(&self, n: usize) -> Vec<(String, f64)> {
        let mut sorted: Vec<_> = self.fees_by_symbol.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted.truncate(n);
        sorted
    }

    /// Fee report
    pub fn print_report(&self) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║          COMMISSION REPORT            ║");
        println!("╠═══════════════════════════════════════╣");
        println!("║ Total fees:        ${:>16.2} ║", self.total_fees);
        println!("║ Trade count:       {:>17} ║", self.trade_count);
        println!("║ Average fee:       ${:>16.2} ║", self.average_fee());
        println!("╠═══════════════════════════════════════╣");
        println!("║             BY SYMBOL                 ║");
        println!("╠═══════════════════════════════════════╣");

        for (symbol, fee) in self.top_symbols_by_fees(5) {
            let percent = (fee / self.total_fees) * 100.0;
            println!("║ {:12} ${:>10.2} ({:>5.1}%) ║", symbol, fee, percent);
        }

        println!("╚═══════════════════════════════════════╝");
    }
}

fn main() {
    let mut tracker = CommissionTracker::new();

    // Simulate several trades
    tracker.record_fee(1, "BTC/USDT", 10_000.0, 10.0, FeeType::Entry);
    tracker.record_fee(2, "BTC/USDT", 10_100.0, 10.1, FeeType::Exit);
    tracker.record_fee(3, "ETH/USDT", 5_000.0, 5.0, FeeType::Entry);
    tracker.record_fee(4, "ETH/USDT", 5_050.0, 5.05, FeeType::Exit);
    tracker.record_fee(5, "SOL/USDT", 1_000.0, 1.0, FeeType::Entry);
    tracker.record_fee(6, "SOL/USDT", 950.0, 0.95, FeeType::Exit);

    tracker.print_report();
}
```

## Advanced Commission Models

### Tiered Commissions (Based on Volume)

```rust
/// Commission tier based on trading volume
#[derive(Debug, Clone)]
pub struct TieredCommission {
    tiers: Vec<CommissionTier>,
}

#[derive(Debug, Clone)]
pub struct CommissionTier {
    /// Minimum volume for this tier (30-day)
    pub min_volume: f64,
    /// Taker fee, %
    pub taker_fee: f64,
    /// Maker fee, %
    pub maker_fee: f64,
}

impl TieredCommission {
    /// Binance VIP tiers
    pub fn binance_vip() -> Self {
        TieredCommission {
            tiers: vec![
                CommissionTier { min_volume: 0.0, taker_fee: 0.1, maker_fee: 0.1 },
                CommissionTier { min_volume: 1_000_000.0, taker_fee: 0.09, maker_fee: 0.09 },
                CommissionTier { min_volume: 5_000_000.0, taker_fee: 0.08, maker_fee: 0.07 },
                CommissionTier { min_volume: 20_000_000.0, taker_fee: 0.07, maker_fee: 0.05 },
                CommissionTier { min_volume: 100_000_000.0, taker_fee: 0.06, maker_fee: 0.04 },
                CommissionTier { min_volume: 500_000_000.0, taker_fee: 0.05, maker_fee: 0.03 },
            ],
        }
    }

    /// Gets commission for given volume
    pub fn get_commission(&self, monthly_volume: f64) -> CommissionModel {
        let tier = self.tiers.iter()
            .filter(|t| t.min_volume <= monthly_volume)
            .last()
            .unwrap();

        CommissionModel {
            taker_fee_percent: tier.taker_fee,
            maker_fee_percent: tier.maker_fee,
            min_fee: 0.0,
            max_fee: None,
        }
    }
}

fn main() {
    let tiered = TieredCommission::binance_vip();

    let volumes = [10_000.0, 500_000.0, 2_000_000.0, 50_000_000.0, 200_000_000.0];

    println!("Binance VIP Tiers:");
    println!("────────────────────────────────────────────");
    println!("{:>15} | {:>10} | {:>10}", "30d Volume", "Taker", "Maker");
    println!("────────────────────────────────────────────");

    for volume in volumes {
        let commission = tiered.get_commission(volume);
        println!(
            "${:>14.0} | {:>9.3}% | {:>9.3}%",
            volume, commission.taker_fee_percent, commission.maker_fee_percent
        );
    }
}
```

### Discounts and Cashbacks

```rust
/// Extended commission model with discounts
#[derive(Debug, Clone)]
pub struct AdvancedCommissionModel {
    base: CommissionModel,
    /// Discount when paying with native token (e.g., BNB)
    native_token_discount: f64,
    /// Referral discount
    referral_discount: f64,
    /// Volume-based cashback
    volume_cashback_percent: f64,
}

impl AdvancedCommissionModel {
    pub fn new(base: CommissionModel) -> Self {
        AdvancedCommissionModel {
            base,
            native_token_discount: 0.0,
            referral_discount: 0.0,
            volume_cashback_percent: 0.0,
        }
    }

    /// Binance with 25% BNB discount
    pub fn binance_with_bnb() -> Self {
        AdvancedCommissionModel {
            base: CommissionModel::binance_spot(),
            native_token_discount: 25.0,  // 25% discount
            referral_discount: 0.0,
            volume_cashback_percent: 0.0,
        }
    }

    /// Calculates final commission
    pub fn calculate(&self, trade_value: f64, is_taker: bool, use_native_token: bool) -> f64 {
        let base_fee = self.base.calculate(trade_value, is_taker);

        // Apply discounts
        let mut discount_multiplier = 1.0;

        if use_native_token {
            discount_multiplier -= self.native_token_discount / 100.0;
        }

        discount_multiplier -= self.referral_discount / 100.0;

        let discounted_fee = base_fee * discount_multiplier;

        // Account for cashback (returned later but reduces effective fee)
        let cashback = discounted_fee * (self.volume_cashback_percent / 100.0);

        discounted_fee - cashback
    }
}

fn main() {
    let model = AdvancedCommissionModel::binance_with_bnb();
    let trade_value = 10_000.0;

    let fee_without_bnb = model.calculate(trade_value, true, false);
    let fee_with_bnb = model.calculate(trade_value, true, true);

    println!("Trade value: ${:.2}", trade_value);
    println!("Fee without BNB: ${:.2}", fee_without_bnb);
    println!("Fee with BNB:    ${:.2}", fee_with_bnb);
    println!("Savings:         ${:.2} ({:.1}%)",
        fee_without_bnb - fee_with_bnb,
        ((fee_without_bnb - fee_with_bnb) / fee_without_bnb) * 100.0
    );
}
```

## Integration with Backtester

```rust
/// Backtest result with commission accounting
#[derive(Debug)]
pub struct BacktestResult {
    pub gross_profit: f64,
    pub total_commissions: f64,
    pub net_profit: f64,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub commission_tracker: CommissionTracker,
}

impl BacktestResult {
    pub fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        }
    }

    /// Percentage of profit consumed by commissions
    pub fn commission_impact(&self) -> f64 {
        if self.gross_profit <= 0.0 {
            0.0
        } else {
            (self.total_commissions / self.gross_profit) * 100.0
        }
    }

    pub fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════════╗");
        println!("║              BACKTEST RESULTS                 ║");
        println!("╠═══════════════════════════════════════════════╣");
        println!("║                   PROFIT                      ║");
        println!("╟───────────────────────────────────────────────╢");
        println!("║ Gross profit:       {:>24.2} ║", self.gross_profit);
        println!("║ Commissions:        {:>24.2} ║", self.total_commissions);
        println!("║ Net profit:         {:>24.2} ║", self.net_profit);
        println!("╟───────────────────────────────────────────────╢");
        println!("║                   TRADES                      ║");
        println!("╟───────────────────────────────────────────────╢");
        println!("║ Total trades:       {:>24} ║", self.total_trades);
        println!("║ Winning:            {:>24} ║", self.winning_trades);
        println!("║ Losing:             {:>24} ║", self.losing_trades);
        println!("║ Win Rate:           {:>23.1}% ║", self.win_rate());
        println!("╟───────────────────────────────────────────────╢");
        println!("║                   ANALYSIS                    ║");
        println!("╟───────────────────────────────────────────────╢");
        println!("║ Fees consumed:      {:>23.1}% ║", self.commission_impact());
        println!("║ Avg fee/trade:      {:>24.2} ║",
            self.commission_tracker.average_fee());
        println!("╚═══════════════════════════════════════════════╝");
    }
}

/// Simple backtester with commission support
pub struct SimpleBacktester {
    commission_model: CommissionModel,
    commission_tracker: CommissionTracker,
    trades: Vec<Trade>,
}

impl SimpleBacktester {
    pub fn new(commission_model: CommissionModel) -> Self {
        SimpleBacktester {
            commission_model,
            commission_tracker: CommissionTracker::new(),
            trades: Vec::new(),
        }
    }

    pub fn add_trade(&mut self, trade: Trade, timestamp: u64) {
        // Record commissions
        let entry_value = trade.entry_price * trade.quantity;
        let entry_fee = self.commission_model.calculate(entry_value, trade.is_taker_entry);
        self.commission_tracker.record_fee(
            timestamp, &trade.symbol, entry_value, entry_fee, FeeType::Entry
        );

        let exit_value = trade.exit_price * trade.quantity;
        let exit_fee = self.commission_model.calculate(exit_value, trade.is_taker_exit);
        self.commission_tracker.record_fee(
            timestamp + 1, &trade.symbol, exit_value, exit_fee, FeeType::Exit
        );

        self.trades.push(trade);
    }

    pub fn run(&self) -> BacktestResult {
        let mut gross_profit = 0.0;
        let mut net_profit = 0.0;
        let mut winning_trades = 0u64;
        let mut losing_trades = 0u64;

        for trade in &self.trades {
            let gross = trade.gross_profit();
            let net = trade.net_profit(&self.commission_model);

            gross_profit += gross;
            net_profit += net;

            if net > 0.0 {
                winning_trades += 1;
            } else {
                losing_trades += 1;
            }
        }

        BacktestResult {
            gross_profit,
            total_commissions: self.commission_tracker.total_fees(),
            net_profit,
            total_trades: self.trades.len() as u64,
            winning_trades,
            losing_trades,
            commission_tracker: self.commission_tracker.clone(),
        }
    }
}

fn main() {
    let commission = CommissionModel::binance_spot();
    let mut backtester = SimpleBacktester::new(commission);

    // Add test trades
    let trades = vec![
        Trade {
            symbol: "BTC/USDT".to_string(),
            side: TradeSide::Long,
            entry_price: 40_000.0,
            exit_price: 40_800.0,  // +2%
            quantity: 0.5,
            is_taker_entry: true,
            is_taker_exit: true,
        },
        Trade {
            symbol: "ETH/USDT".to_string(),
            side: TradeSide::Long,
            entry_price: 2_500.0,
            exit_price: 2_450.0,  // -2%
            quantity: 2.0,
            is_taker_entry: true,
            is_taker_exit: true,
        },
        Trade {
            symbol: "BTC/USDT".to_string(),
            side: TradeSide::Short,
            entry_price: 41_000.0,
            exit_price: 40_500.0,  // +1.2% for short
            quantity: 0.3,
            is_taker_entry: false,  // limit order = maker
            is_taker_exit: true,
        },
        Trade {
            symbol: "SOL/USDT".to_string(),
            side: TradeSide::Long,
            entry_price: 100.0,
            exit_price: 105.0,  // +5%
            quantity: 50.0,
            is_taker_entry: true,
            is_taker_exit: false,  // limit order = maker
        },
    ];

    for (i, trade) in trades.into_iter().enumerate() {
        backtester.add_trade(trade, i as u64 * 100);
    }

    let result = backtester.run();
    result.print_summary();
    result.commission_tracker.print_report();
}
```

## Comparing Strategies with Commission Impact

```rust
/// Strategy sensitivity analysis to commissions
pub fn analyze_commission_sensitivity(
    base_gross_profit: f64,
    trade_count: u64,
    avg_trade_value: f64,
) {
    println!("\n═══════════════════════════════════════════════════════════");
    println!("         COMMISSION SENSITIVITY ANALYSIS");
    println!("═══════════════════════════════════════════════════════════");
    println!("Gross profit: ${:.2}", base_gross_profit);
    println!("Trade count: {}", trade_count);
    println!("Average trade value: ${:.2}", avg_trade_value);
    println!("───────────────────────────────────────────────────────────");
    println!("{:>10} | {:>15} | {:>15} | {:>12}",
        "Fee Rate", "Total Fees", "Net Profit", "% of Gross");
    println!("───────────────────────────────────────────────────────────");

    let commission_rates = [0.01, 0.05, 0.1, 0.15, 0.2, 0.3, 0.5];

    for rate in commission_rates {
        // Fee for entry and exit
        let total_commission = avg_trade_value * (rate / 100.0) * 2.0 * trade_count as f64;
        let net_profit = base_gross_profit - total_commission;
        let impact = (total_commission / base_gross_profit) * 100.0;

        let status = if net_profit > 0.0 { "+" } else { "" };

        println!(
            "{:>9.2}% | ${:>14.2} | {:>1}${:>13.2} | {:>11.1}%",
            rate, total_commission, status, net_profit.abs(), impact
        );
    }
    println!("═══════════════════════════════════════════════════════════");
}

fn main() {
    // Strategy A: many small trades
    println!("\nStrategy A: Scalping (many small trades)");
    analyze_commission_sensitivity(
        5000.0,      // Gross profit
        500,         // 500 trades
        1000.0,      // Average value $1000
    );

    // Strategy B: few large trades
    println!("\nStrategy B: Swing Trading (few large trades)");
    analyze_commission_sensitivity(
        5000.0,      // Same gross profit
        20,          // Only 20 trades
        10000.0,     // Average value $10,000
    );
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Trading Commissions | Exchanges take a percentage of trade volume |
| Taker/Maker | Different rates for removing/adding liquidity |
| Gross vs Net | Profit before and after commission deduction |
| Tiered Commissions | Volume-based fee discounts |
| Frequency Impact | More trades = more commissions |
| Backtesting | Results are unrealistic without commission accounting |

## Homework

1. **Commission Calculator**: Create a function `compare_exchanges(trade_value: f64, monthly_volume: f64)` that compares fees across different exchanges (Binance, Bybit, OKX) with tiered discounts and outputs the most cost-effective option.

2. **Optimal Trade Frequency**: Write a program that, for a given strategy with fixed expected returns, calculates the optimal number of trades per month where Net profit is maximized.

3. **Cost Tracker**: Extend `CommissionTracker` by adding:
   - Fee tracking by day/week/month
   - Average daily commission calculation
   - Warning when fees exceed a threshold percentage of profit

4. **Strategy Comparison**: Given two strategies with the same Gross profit but different trade counts, calculate at what exchange fee rate they become equally profitable.

## Navigation

[← Previous day](../280-slippage-model/en.md) | [Next day →](../282-equity-curve/en.md)
