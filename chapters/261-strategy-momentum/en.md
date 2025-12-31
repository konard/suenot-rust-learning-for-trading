# Day 261: Strategy: Momentum

## Trading Analogy

Imagine you're watching a busy marketplace. Some stalls have a growing line of customers — word has spread that they're selling something good. Other stalls are losing customers rapidly. As a savvy trader, you notice these **trends** and act accordingly: you invest in what's gaining popularity and avoid (or short) what's falling out of favor.

This is the essence of **momentum trading** — the strategy based on the principle that assets that have been rising tend to continue rising, and assets that have been falling tend to continue falling. It's like surfing: you catch a wave that's already moving and ride it in the same direction.

In financial markets, momentum works because:
- **Trend persistence**: Market trends often continue due to investor psychology
- **Herding behavior**: Traders follow other traders, amplifying price movements
- **Information diffusion**: News and information spread gradually, creating sustained moves
- **Institutional flows**: Large funds take time to build or exit positions

## What is Momentum?

Momentum is the rate of change in price over a specific period. It measures how fast and in which direction an asset's price is moving.

Key momentum concepts:
- **Absolute momentum**: Current price relative to its past price
- **Relative momentum**: Comparing performance across multiple assets
- **Rate of Change (ROC)**: Percentage change over N periods
- **Moving Average**: Smoothed price to identify trend direction

## Basic Momentum Calculation

```rust
/// Calculates the Rate of Change (ROC) momentum indicator
fn calculate_roc(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() <= period {
        return vec![];
    }

    prices
        .iter()
        .skip(period)
        .zip(prices.iter())
        .map(|(current, past)| ((current - past) / past) * 100.0)
        .collect()
}

/// Calculates Simple Moving Average
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    prices
        .windows(period)
        .map(|window| window.iter().sum::<f64>() / period as f64)
        .collect()
}

fn main() {
    // BTC daily closing prices (simulated)
    let btc_prices = vec![
        42000.0, 42500.0, 43200.0, 43800.0, 44500.0,
        45200.0, 44800.0, 45500.0, 46200.0, 47000.0,
        47800.0, 48500.0, 48200.0, 49000.0, 50000.0,
    ];

    println!("=== Momentum Analysis for BTC ===\n");

    // Calculate 5-period ROC
    let roc_5 = calculate_roc(&btc_prices, 5);
    println!("5-period ROC values:");
    for (i, roc) in roc_5.iter().enumerate() {
        let signal = if *roc > 5.0 {
            "STRONG BUY"
        } else if *roc > 0.0 {
            "BUY"
        } else if *roc > -5.0 {
            "SELL"
        } else {
            "STRONG SELL"
        };
        println!("  Day {}: ROC = {:.2}% -> {}", i + 6, roc, signal);
    }

    // Calculate 5-period SMA
    let sma_5 = calculate_sma(&btc_prices, 5);
    println!("\n5-period SMA values:");
    for (i, sma) in sma_5.iter().enumerate() {
        let current_price = btc_prices[i + 4];
        let trend = if current_price > *sma { "ABOVE (Bullish)" } else { "BELOW (Bearish)" };
        println!("  Day {}: Price ${:.0} vs SMA ${:.0} -> {}", i + 5, current_price, sma, trend);
    }
}
```

## Momentum Strategy Structure

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Signal {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Position {
    Long,
    Short,
    Flat,
}

#[derive(Debug)]
struct MomentumStrategy {
    lookback_period: usize,
    entry_threshold: f64,
    exit_threshold: f64,
    price_history: VecDeque<f64>,
    position: Position,
    entry_price: Option<f64>,
}

impl MomentumStrategy {
    fn new(lookback_period: usize, entry_threshold: f64, exit_threshold: f64) -> Self {
        MomentumStrategy {
            lookback_period,
            entry_threshold,
            exit_threshold,
            price_history: VecDeque::with_capacity(lookback_period + 1),
            position: Position::Flat,
            entry_price: None,
        }
    }

    fn update(&mut self, price: f64) -> Signal {
        self.price_history.push_back(price);

        // Maintain only needed history
        while self.price_history.len() > self.lookback_period + 1 {
            self.price_history.pop_front();
        }

        // Need enough data for calculation
        if self.price_history.len() <= self.lookback_period {
            return Signal::Hold;
        }

        let momentum = self.calculate_momentum();
        self.generate_signal(momentum, price)
    }

    fn calculate_momentum(&self) -> f64 {
        let current = *self.price_history.back().unwrap();
        let past = *self.price_history.front().unwrap();
        ((current - past) / past) * 100.0
    }

    fn generate_signal(&mut self, momentum: f64, current_price: f64) -> Signal {
        match self.position {
            Position::Flat => {
                if momentum > self.entry_threshold {
                    self.position = Position::Long;
                    self.entry_price = Some(current_price);
                    if momentum > self.entry_threshold * 2.0 {
                        Signal::StrongBuy
                    } else {
                        Signal::Buy
                    }
                } else if momentum < -self.entry_threshold {
                    self.position = Position::Short;
                    self.entry_price = Some(current_price);
                    if momentum < -self.entry_threshold * 2.0 {
                        Signal::StrongSell
                    } else {
                        Signal::Sell
                    }
                } else {
                    Signal::Hold
                }
            }
            Position::Long => {
                if momentum < self.exit_threshold {
                    self.position = Position::Flat;
                    self.entry_price = None;
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            Position::Short => {
                if momentum > -self.exit_threshold {
                    self.position = Position::Flat;
                    self.entry_price = None;
                    Signal::Buy
                } else {
                    Signal::Hold
                }
            }
        }
    }

    fn get_position(&self) -> Position {
        self.position
    }

    fn get_unrealized_pnl(&self, current_price: f64) -> Option<f64> {
        self.entry_price.map(|entry| {
            match self.position {
                Position::Long => ((current_price - entry) / entry) * 100.0,
                Position::Short => ((entry - current_price) / entry) * 100.0,
                Position::Flat => 0.0,
            }
        })
    }
}

fn main() {
    let mut strategy = MomentumStrategy::new(10, 5.0, 0.0);

    // Simulated price data with trending periods
    let prices = vec![
        100.0, 101.0, 102.5, 104.0, 106.0, 108.5, 111.0, 114.0, 117.0, 120.0,  // Uptrend
        121.0, 122.0, 121.5, 120.0, 118.0, 115.0, 112.0, 110.0, 108.0, 105.0,  // Downtrend
        104.0, 105.0, 107.0, 110.0, 113.0, 116.0, 120.0, 124.0, 128.0, 132.0,  // New uptrend
    ];

    println!("=== Momentum Strategy Backtest ===\n");
    println!("{:>4} {:>10} {:>12} {:>10} {:>12}", "Day", "Price", "Signal", "Position", "PnL %");
    println!("{}", "-".repeat(52));

    for (day, &price) in prices.iter().enumerate() {
        let signal = strategy.update(price);
        let position = strategy.get_position();
        let pnl = strategy.get_unrealized_pnl(price)
            .map(|p| format!("{:+.2}%", p))
            .unwrap_or_else(|| "N/A".to_string());

        println!("{:>4} {:>10.2} {:>12?} {:>10?} {:>12}",
            day + 1, price, signal, position, pnl);
    }
}
```

## Relative Momentum: Comparing Assets

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Asset {
    symbol: String,
    prices: Vec<f64>,
}

impl Asset {
    fn new(symbol: &str, prices: Vec<f64>) -> Self {
        Asset {
            symbol: symbol.to_string(),
            prices,
        }
    }

    fn momentum(&self, period: usize) -> Option<f64> {
        if self.prices.len() <= period {
            return None;
        }

        let current = *self.prices.last()?;
        let past = self.prices[self.prices.len() - 1 - period];
        Some(((current - past) / past) * 100.0)
    }

    fn latest_price(&self) -> Option<f64> {
        self.prices.last().copied()
    }
}

#[derive(Debug)]
struct RelativeMomentumRanker {
    assets: Vec<Asset>,
    lookback_period: usize,
    top_n: usize,
}

impl RelativeMomentumRanker {
    fn new(lookback_period: usize, top_n: usize) -> Self {
        RelativeMomentumRanker {
            assets: Vec::new(),
            lookback_period,
            top_n,
        }
    }

    fn add_asset(&mut self, asset: Asset) {
        self.assets.push(asset);
    }

    fn rank_assets(&self) -> Vec<(&Asset, f64)> {
        let mut rankings: Vec<_> = self.assets
            .iter()
            .filter_map(|asset| {
                asset.momentum(self.lookback_period)
                    .map(|mom| (asset, mom))
            })
            .collect();

        // Sort by momentum descending
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings
    }

    fn get_top_momentum(&self) -> Vec<(&Asset, f64)> {
        self.rank_assets()
            .into_iter()
            .take(self.top_n)
            .collect()
    }

    fn get_bottom_momentum(&self) -> Vec<(&Asset, f64)> {
        let mut rankings = self.rank_assets();
        rankings.reverse();
        rankings.into_iter().take(self.top_n).collect()
    }
}

fn main() {
    let mut ranker = RelativeMomentumRanker::new(5, 3);

    // Add various crypto assets
    ranker.add_asset(Asset::new("BTC", vec![
        40000.0, 41000.0, 42500.0, 44000.0, 46000.0, 48000.0
    ]));

    ranker.add_asset(Asset::new("ETH", vec![
        2200.0, 2300.0, 2250.0, 2400.0, 2550.0, 2700.0
    ]));

    ranker.add_asset(Asset::new("SOL", vec![
        80.0, 85.0, 95.0, 110.0, 130.0, 150.0
    ]));

    ranker.add_asset(Asset::new("ADA", vec![
        0.50, 0.48, 0.45, 0.42, 0.40, 0.38
    ]));

    ranker.add_asset(Asset::new("DOGE", vec![
        0.08, 0.085, 0.082, 0.079, 0.075, 0.072
    ]));

    println!("=== Relative Momentum Rankings ===\n");

    println!("All assets ranked by 5-day momentum:");
    for (rank, (asset, momentum)) in ranker.rank_assets().iter().enumerate() {
        let direction = if *momentum > 0.0 { "+" } else { "" };
        println!("  {}. {} -> {}{:.2}%", rank + 1, asset.symbol, direction, momentum);
    }

    println!("\n--- Portfolio Selection ---");

    println!("\nLONG candidates (top momentum):");
    for (asset, momentum) in ranker.get_top_momentum() {
        println!("  BUY {} (momentum: +{:.2}%)", asset.symbol, momentum);
    }

    println!("\nSHORT candidates (bottom momentum):");
    for (asset, momentum) in ranker.get_bottom_momentum() {
        println!("  SELL {} (momentum: {:.2}%)", asset.symbol, momentum);
    }
}
```

## Dual Momentum Strategy

Dual momentum combines absolute and relative momentum for better risk-adjusted returns:

```rust
#[derive(Debug, Clone)]
struct DualMomentumStrategy {
    assets: Vec<String>,
    risk_free_rate: f64,  // Annual rate
    lookback_months: usize,
}

#[derive(Debug)]
struct MonthlyReturns {
    symbol: String,
    returns: Vec<f64>,
}

impl MonthlyReturns {
    fn new(symbol: &str, returns: Vec<f64>) -> Self {
        MonthlyReturns {
            symbol: symbol.to_string(),
            returns,
        }
    }

    fn cumulative_return(&self, months: usize) -> f64 {
        if self.returns.len() < months {
            return 0.0;
        }

        let start_idx = self.returns.len() - months;
        self.returns[start_idx..]
            .iter()
            .fold(1.0, |acc, r| acc * (1.0 + r / 100.0)) - 1.0
    }
}

impl DualMomentumStrategy {
    fn new(assets: Vec<String>, risk_free_rate: f64, lookback_months: usize) -> Self {
        DualMomentumStrategy {
            assets,
            risk_free_rate,
            lookback_months,
        }
    }

    fn select_asset(&self, asset_returns: &[MonthlyReturns]) -> Option<String> {
        // Step 1: Find asset with best relative momentum
        let mut best_asset: Option<(&MonthlyReturns, f64)> = None;

        for returns in asset_returns {
            if self.assets.contains(&returns.symbol) {
                let cum_return = returns.cumulative_return(self.lookback_months);

                match &best_asset {
                    None => best_asset = Some((returns, cum_return)),
                    Some((_, best_return)) if cum_return > *best_return => {
                        best_asset = Some((returns, cum_return));
                    }
                    _ => {}
                }
            }
        }

        // Step 2: Check absolute momentum (vs risk-free rate)
        if let Some((asset, momentum)) = best_asset {
            // Convert annual risk-free rate to period rate
            let period_rf_rate = (1.0 + self.risk_free_rate / 100.0)
                .powf(self.lookback_months as f64 / 12.0) - 1.0;

            if momentum > period_rf_rate {
                return Some(asset.symbol.clone());
            }
        }

        // If no asset beats risk-free, go to cash
        None
    }
}

fn main() {
    // Create dual momentum strategy
    let strategy = DualMomentumStrategy::new(
        vec!["SPY".to_string(), "EFA".to_string(), "BTC".to_string()],
        4.0,  // 4% annual risk-free rate
        12,   // 12-month lookback
    );

    // Simulated 12-month returns for each asset
    let asset_returns = vec![
        MonthlyReturns::new("SPY", vec![
            1.5, 2.0, -0.5, 1.0, 2.5, 1.8, -1.0, 0.5, 2.0, 1.5, 0.8, 1.2
        ]),
        MonthlyReturns::new("EFA", vec![
            0.8, 1.5, -1.0, 0.5, 1.8, 2.0, -0.5, 0.2, 1.5, 0.8, 0.5, 0.8
        ]),
        MonthlyReturns::new("BTC", vec![
            5.0, 8.0, -3.0, 10.0, 15.0, -5.0, 2.0, 7.0, 12.0, -2.0, 5.0, 8.0
        ]),
    ];

    println!("=== Dual Momentum Strategy ===\n");

    // Display cumulative returns
    println!("12-Month Cumulative Returns:");
    for returns in &asset_returns {
        let cum_return = returns.cumulative_return(12) * 100.0;
        println!("  {}: {:.2}%", returns.symbol, cum_return);
    }

    // Calculate risk-free threshold
    let rf_threshold = ((1.0 + 4.0 / 100.0_f64).powf(1.0) - 1.0) * 100.0;
    println!("\nRisk-Free Threshold: {:.2}%", rf_threshold);

    // Get recommendation
    match strategy.select_asset(&asset_returns) {
        Some(asset) => {
            println!("\nRECOMMENDATION: Invest in {}", asset);
            println!("Rationale: Best relative momentum AND beats risk-free rate");
        }
        None => {
            println!("\nRECOMMENDATION: Stay in CASH");
            println!("Rationale: No asset beats the risk-free rate");
        }
    }
}
```

## Momentum with Risk Management

```rust
#[derive(Debug, Clone)]
struct RiskManagedMomentum {
    symbol: String,
    position_size: f64,
    entry_price: Option<f64>,
    stop_loss_pct: f64,
    take_profit_pct: f64,
    trailing_stop_pct: f64,
    highest_price: f64,
}

#[derive(Debug)]
enum TradeAction {
    Enter { price: f64, size: f64 },
    Exit { price: f64, reason: String, pnl: f64 },
    Hold,
    UpdateTrailingStop { new_stop: f64 },
}

impl RiskManagedMomentum {
    fn new(symbol: &str, stop_loss_pct: f64, take_profit_pct: f64, trailing_stop_pct: f64) -> Self {
        RiskManagedMomentum {
            symbol: symbol.to_string(),
            position_size: 0.0,
            entry_price: None,
            stop_loss_pct,
            take_profit_pct,
            trailing_stop_pct,
            highest_price: 0.0,
        }
    }

    fn enter_position(&mut self, price: f64, size: f64) -> TradeAction {
        self.entry_price = Some(price);
        self.position_size = size;
        self.highest_price = price;
        TradeAction::Enter { price, size }
    }

    fn update(&mut self, current_price: f64) -> TradeAction {
        let entry = match self.entry_price {
            Some(p) => p,
            None => return TradeAction::Hold,
        };

        let pnl_pct = ((current_price - entry) / entry) * 100.0;

        // Update highest price for trailing stop
        if current_price > self.highest_price {
            self.highest_price = current_price;
        }

        // Calculate stop levels
        let fixed_stop = entry * (1.0 - self.stop_loss_pct / 100.0);
        let trailing_stop = self.highest_price * (1.0 - self.trailing_stop_pct / 100.0);
        let effective_stop = fixed_stop.max(trailing_stop);

        let take_profit = entry * (1.0 + self.take_profit_pct / 100.0);

        // Check exit conditions
        if current_price <= effective_stop {
            let reason = if current_price <= fixed_stop {
                "Stop Loss Hit".to_string()
            } else {
                "Trailing Stop Hit".to_string()
            };
            self.close_position();
            return TradeAction::Exit {
                price: current_price,
                reason,
                pnl: pnl_pct,
            };
        }

        if current_price >= take_profit {
            self.close_position();
            return TradeAction::Exit {
                price: current_price,
                reason: "Take Profit Hit".to_string(),
                pnl: pnl_pct,
            };
        }

        // Check if trailing stop was updated
        if trailing_stop > fixed_stop {
            TradeAction::UpdateTrailingStop { new_stop: trailing_stop }
        } else {
            TradeAction::Hold
        }
    }

    fn close_position(&mut self) {
        self.entry_price = None;
        self.position_size = 0.0;
        self.highest_price = 0.0;
    }
}

fn main() {
    let mut trader = RiskManagedMomentum::new(
        "BTC",
        5.0,   // 5% stop loss
        15.0,  // 15% take profit
        3.0,   // 3% trailing stop
    );

    println!("=== Risk-Managed Momentum Trading ===\n");

    // Enter position
    let entry_action = trader.enter_position(50000.0, 1.0);
    println!("Entry: {:?}\n", entry_action);

    // Simulate price movements
    let prices = vec![
        50500.0, 51000.0, 52000.0, 53500.0, 55000.0,  // Uptrend
        54000.0, 53000.0, 52500.0,  // Pullback
        54000.0, 56000.0, 57000.0, 57500.0,  // Resume uptrend
        56500.0, 55000.0, 54000.0,  // Drop - trailing stop may trigger
    ];

    println!("{:>6} {:>12} {:>12} {:>10}", "Step", "Price", "P&L %", "Action");
    println!("{}", "-".repeat(45));

    for (i, &price) in prices.iter().enumerate() {
        let entry_price = 50000.0;
        let pnl = ((price - entry_price) / entry_price) * 100.0;
        let action = trader.update(price);

        let action_str = match &action {
            TradeAction::Hold => "Hold".to_string(),
            TradeAction::Exit { reason, .. } => format!("EXIT: {}", reason),
            TradeAction::UpdateTrailingStop { new_stop } => {
                format!("Trail: ${:.0}", new_stop)
            }
            _ => "".to_string(),
        };

        println!("{:>6} {:>12.0} {:>+11.2}% {:>10}",
            i + 1, price, pnl, action_str);

        if matches!(action, TradeAction::Exit { .. }) {
            break;
        }
    }
}
```

## Complete Momentum Trading System

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct PriceBar {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug)]
struct MomentumTradingSystem {
    symbol: String,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    bars: Vec<PriceBar>,
    position: f64,
    cash: f64,
    trades: Vec<Trade>,
}

#[derive(Debug, Clone)]
struct Trade {
    entry_time: u64,
    exit_time: Option<u64>,
    entry_price: f64,
    exit_price: Option<f64>,
    size: f64,
    pnl: Option<f64>,
}

impl MomentumTradingSystem {
    fn new(symbol: &str, initial_cash: f64) -> Self {
        MomentumTradingSystem {
            symbol: symbol.to_string(),
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            bars: Vec::new(),
            position: 0.0,
            cash: initial_cash,
            trades: Vec::new(),
        }
    }

    fn add_bar(&mut self, bar: PriceBar) {
        self.bars.push(bar);
        self.evaluate_signals();
    }

    fn calculate_ema(&self, period: usize) -> Vec<f64> {
        let closes: Vec<f64> = self.bars.iter().map(|b| b.close).collect();

        if closes.len() < period {
            return vec![];
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = Vec::with_capacity(closes.len());

        // First EMA is SMA
        let first_sma: f64 = closes[..period].iter().sum::<f64>() / period as f64;
        ema.push(first_sma);

        for i in period..closes.len() {
            let new_ema = (closes[i] - ema.last().unwrap()) * multiplier + ema.last().unwrap();
            ema.push(new_ema);
        }

        ema
    }

    fn calculate_macd(&self) -> Option<(f64, f64, f64)> {
        let fast_ema = self.calculate_ema(self.fast_period);
        let slow_ema = self.calculate_ema(self.slow_period);

        if fast_ema.is_empty() || slow_ema.is_empty() {
            return None;
        }

        // Align EMAs
        let offset = self.slow_period - self.fast_period;
        if fast_ema.len() <= offset {
            return None;
        }

        let macd_line: Vec<f64> = fast_ema[offset..]
            .iter()
            .zip(slow_ema.iter())
            .map(|(f, s)| f - s)
            .collect();

        if macd_line.len() < self.signal_period {
            return None;
        }

        // Calculate signal line (EMA of MACD)
        let multiplier = 2.0 / (self.signal_period as f64 + 1.0);
        let first_signal: f64 = macd_line[..self.signal_period].iter().sum::<f64>()
            / self.signal_period as f64;

        let mut signal_line = first_signal;
        for &macd in &macd_line[self.signal_period..] {
            signal_line = (macd - signal_line) * multiplier + signal_line;
        }

        let current_macd = *macd_line.last().unwrap();
        let histogram = current_macd - signal_line;

        Some((current_macd, signal_line, histogram))
    }

    fn calculate_rsi(&self, period: usize) -> Option<f64> {
        if self.bars.len() < period + 1 {
            return None;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        let start = self.bars.len() - period - 1;
        for i in start..self.bars.len() - 1 {
            let change = self.bars[i + 1].close - self.bars[i].close;
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    fn evaluate_signals(&mut self) {
        let current_bar = match self.bars.last() {
            Some(b) => b.clone(),
            None => return,
        };

        let macd = match self.calculate_macd() {
            Some(m) => m,
            None => return,
        };

        let rsi = match self.calculate_rsi(14) {
            Some(r) => r,
            None => return,
        };

        let (macd_line, signal_line, histogram) = macd;

        // Trading logic
        if self.position == 0.0 {
            // Entry conditions
            let macd_bullish = histogram > 0.0 && macd_line > signal_line;
            let rsi_not_overbought = rsi < 70.0;
            let rsi_recovering = rsi > 30.0;

            if macd_bullish && rsi_not_overbought && rsi_recovering {
                // Calculate position size (use 95% of cash)
                let size = (self.cash * 0.95) / current_bar.close;
                self.position = size;
                self.cash -= size * current_bar.close;

                self.trades.push(Trade {
                    entry_time: current_bar.timestamp,
                    exit_time: None,
                    entry_price: current_bar.close,
                    exit_price: None,
                    size,
                    pnl: None,
                });

                println!("BUY: {} units @ ${:.2} | RSI: {:.1} | MACD Hist: {:.4}",
                    size, current_bar.close, rsi, histogram);
            }
        } else {
            // Exit conditions
            let macd_bearish = histogram < 0.0;
            let rsi_overbought = rsi > 70.0;

            if macd_bearish || rsi_overbought {
                let exit_price = current_bar.close;
                let exit_value = self.position * exit_price;

                if let Some(trade) = self.trades.last_mut() {
                    let pnl = exit_value - (trade.size * trade.entry_price);
                    trade.exit_time = Some(current_bar.timestamp);
                    trade.exit_price = Some(exit_price);
                    trade.pnl = Some(pnl);

                    println!("SELL: {} units @ ${:.2} | PnL: ${:.2} | RSI: {:.1}",
                        self.position, exit_price, pnl, rsi);
                }

                self.cash += exit_value;
                self.position = 0.0;
            }
        }
    }

    fn get_performance(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        let total_pnl: f64 = self.trades
            .iter()
            .filter_map(|t| t.pnl)
            .sum();

        let winning_trades: Vec<_> = self.trades
            .iter()
            .filter(|t| t.pnl.map(|p| p > 0.0).unwrap_or(false))
            .collect();

        let total_trades = self.trades.iter().filter(|t| t.pnl.is_some()).count();
        let win_rate = if total_trades > 0 {
            winning_trades.len() as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        stats.insert("total_pnl".to_string(), total_pnl);
        stats.insert("total_trades".to_string(), total_trades as f64);
        stats.insert("win_rate".to_string(), win_rate);
        stats.insert("current_cash".to_string(), self.cash);
        stats.insert("position_value".to_string(),
            self.position * self.bars.last().map(|b| b.close).unwrap_or(0.0));

        stats
    }
}

fn main() {
    let mut system = MomentumTradingSystem::new("BTC", 100000.0);

    println!("=== Complete Momentum Trading System ===\n");

    // Generate simulated price data with trends
    let mut price = 40000.0;
    for day in 0..100 {
        // Simulate price movement
        let trend = if day < 30 { 1.005 }
            else if day < 50 { 0.998 }
            else if day < 80 { 1.008 }
            else { 0.995 };

        let volatility = (rand_simple(day) - 0.5) * 0.02;
        price *= trend + volatility;

        let bar = PriceBar {
            timestamp: 1700000000 + day as u64 * 86400,
            open: price * 0.999,
            high: price * 1.01,
            low: price * 0.99,
            close: price,
            volume: 1000.0 + rand_simple(day) * 500.0,
        };

        system.add_bar(bar);
    }

    println!("\n=== Performance Summary ===");
    for (key, value) in system.get_performance() {
        println!("{}: {:.2}", key, value);
    }
}

// Simple deterministic "random" for reproducibility
fn rand_simple(seed: u32) -> f64 {
    let x = seed.wrapping_mul(1103515245).wrapping_add(12345);
    (x as f64 / u32::MAX as f64).abs()
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Momentum | Rate of price change over a time period |
| ROC (Rate of Change) | Percentage change between current and past price |
| SMA/EMA | Moving averages to smooth price and identify trends |
| Relative Momentum | Comparing performance across multiple assets |
| Dual Momentum | Combining absolute and relative momentum |
| MACD | Momentum oscillator using EMA crossovers |
| RSI | Relative Strength Index for overbought/oversold |
| Trailing Stop | Dynamic stop-loss that follows price higher |

## Practice Exercises

1. **Basic Momentum Calculator**: Write a function that calculates both 10-day and 20-day momentum for a price series and returns a signal based on their crossover.

2. **Multi-Asset Ranker**: Create a program that takes price data for 5 cryptocurrencies and ranks them by 7-day momentum, displaying the top 2 for potential longs.

3. **Momentum Reversal Detector**: Implement a function that detects when momentum is weakening (slowing down even if still positive) as an early warning signal.

4. **Volume-Weighted Momentum**: Modify the basic momentum calculation to weight price changes by volume, giving more importance to high-volume moves.

## Homework

1. **Momentum Divergence Detector**: Implement a system that detects divergence between price (making new highs) and momentum (making lower highs). This often signals trend reversals.

2. **Sector Rotation Strategy**: Create a program that:
   - Tracks momentum for 4 different "sectors" (e.g., DeFi, L1s, Meme coins, Stablecoins)
   - Rotates capital to the sector with the highest momentum
   - Includes a "risk-off" mode when all sectors have negative momentum

3. **Adaptive Momentum**: Build a momentum strategy that automatically adjusts its lookback period based on market volatility (shorter periods in high volatility, longer in low volatility).

4. **Momentum + Mean Reversion Hybrid**: Design a trading system that:
   - Uses momentum for the primary trend direction
   - Uses mean reversion for entry timing (buying dips in uptrends)
   - Includes proper position sizing based on volatility

## Navigation

[← Previous day](../260-strategy-market-making/en.md) | [Next day →](../262-strategy-pairs-trading/en.md)
