# Day 252: ATR: Average True Range

## Trading Analogy

Imagine you're a trader trying to decide where to place your stop-loss order. If you place it too close to the current price, normal market fluctuations will trigger it. If you place it too far away, you risk losing too much on a bad trade. How do you know what "normal" price movement looks like?

This is where **ATR (Average True Range)** comes in. Think of ATR as a market "volatility thermometer" — it measures how much an asset typically moves during a trading session. Just like you'd dress differently for 10°C vs 30°C weather, you'd set different stop-loss distances for low-volatility vs high-volatility markets.

In real trading, ATR is used for:
- **Stop-loss placement**: Setting stops 2-3 ATR away from entry
- **Position sizing**: Trading smaller positions when volatility is high
- **Breakout confirmation**: A breakout with high ATR is more significant
- **Comparing volatility**: Is BTC more volatile than ETH right now?

## What is ATR?

ATR was developed by J. Welles Wilder Jr. and introduced in his 1978 book "New Concepts in Technical Trading Systems." It measures market volatility by calculating the average of **True Range** values over a specified period.

### True Range (TR)

The True Range is the greatest of:
1. Current High minus Current Low
2. Absolute value of (Current High minus Previous Close)
3. Absolute value of (Current Low minus Previous Close)

```
TR = max(High - Low, |High - Prev Close|, |Low - Prev Close|)
```

The True Range accounts for gaps — if a stock closes at $100 and opens the next day at $105, the regular range (High - Low) wouldn't capture this $5 gap, but True Range does.

### ATR Calculation

ATR is typically a 14-period moving average of True Range values. The first ATR is a simple average, then subsequent values use:

```
ATR = ((Previous ATR × (n - 1)) + Current TR) / n
```

Where `n` is the period (usually 14).

## Basic ATR Implementation

```rust
/// Represents a single candlestick (OHLC data)
#[derive(Debug, Clone)]
struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// ATR indicator calculator
struct ATRCalculator {
    period: usize,
    tr_values: Vec<f64>,
    current_atr: Option<f64>,
    prev_close: Option<f64>,
}

impl ATRCalculator {
    fn new(period: usize) -> Self {
        ATRCalculator {
            period,
            tr_values: Vec::with_capacity(period),
            current_atr: None,
            prev_close: None,
        }
    }

    /// Calculate True Range for a candle
    fn calculate_true_range(&self, candle: &Candle) -> f64 {
        match self.prev_close {
            Some(prev_close) => {
                let high_low = candle.high - candle.low;
                let high_prev_close = (candle.high - prev_close).abs();
                let low_prev_close = (candle.low - prev_close).abs();

                high_low.max(high_prev_close).max(low_prev_close)
            }
            None => {
                // First candle: TR is simply High - Low
                candle.high - candle.low
            }
        }
    }

    /// Update ATR with a new candle
    fn update(&mut self, candle: &Candle) -> Option<f64> {
        let tr = self.calculate_true_range(candle);
        self.prev_close = Some(candle.close);

        match self.current_atr {
            None => {
                // Building initial ATR
                self.tr_values.push(tr);

                if self.tr_values.len() >= self.period {
                    // Calculate first ATR as simple average
                    let sum: f64 = self.tr_values.iter().sum();
                    self.current_atr = Some(sum / self.period as f64);
                    self.tr_values.clear(); // No longer needed
                }

                self.current_atr
            }
            Some(prev_atr) => {
                // Wilder's smoothing method
                let new_atr = ((prev_atr * (self.period - 1) as f64) + tr)
                    / self.period as f64;
                self.current_atr = Some(new_atr);
                self.current_atr
            }
        }
    }

    /// Get current ATR value
    fn value(&self) -> Option<f64> {
        self.current_atr
    }
}

fn main() {
    let mut atr = ATRCalculator::new(14);

    // Simulated BTC/USDT candles
    let candles = vec![
        Candle { timestamp: 1, open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0, volume: 100.0 },
        Candle { timestamp: 2, open: 42300.0, high: 42800.0, low: 42100.0, close: 42600.0, volume: 120.0 },
        Candle { timestamp: 3, open: 42600.0, high: 43100.0, low: 42400.0, close: 42900.0, volume: 150.0 },
        Candle { timestamp: 4, open: 42900.0, high: 43200.0, low: 42700.0, close: 43000.0, volume: 130.0 },
        Candle { timestamp: 5, open: 43000.0, high: 43500.0, low: 42800.0, close: 43200.0, volume: 140.0 },
        Candle { timestamp: 6, open: 43200.0, high: 43800.0, low: 43000.0, close: 43600.0, volume: 160.0 },
        Candle { timestamp: 7, open: 43600.0, high: 44000.0, low: 43400.0, close: 43800.0, volume: 145.0 },
        Candle { timestamp: 8, open: 43800.0, high: 44200.0, low: 43500.0, close: 44000.0, volume: 155.0 },
        Candle { timestamp: 9, open: 44000.0, high: 44500.0, low: 43800.0, close: 44300.0, volume: 170.0 },
        Candle { timestamp: 10, open: 44300.0, high: 44800.0, low: 44100.0, close: 44500.0, volume: 165.0 },
        Candle { timestamp: 11, open: 44500.0, high: 45000.0, low: 44300.0, close: 44700.0, volume: 175.0 },
        Candle { timestamp: 12, open: 44700.0, high: 45200.0, low: 44500.0, close: 45000.0, volume: 180.0 },
        Candle { timestamp: 13, open: 45000.0, high: 45500.0, low: 44800.0, close: 45300.0, volume: 190.0 },
        Candle { timestamp: 14, open: 45300.0, high: 45800.0, low: 45000.0, close: 45500.0, volume: 185.0 },
        Candle { timestamp: 15, open: 45500.0, high: 46000.0, low: 45200.0, close: 45800.0, volume: 195.0 },
    ];

    for candle in &candles {
        let atr_value = atr.update(candle);
        match atr_value {
            Some(val) => println!(
                "Candle {}: Close=${:.2}, ATR=${:.2}",
                candle.timestamp, candle.close, val
            ),
            None => println!(
                "Candle {}: Close=${:.2}, ATR=calculating...",
                candle.timestamp, candle.close
            ),
        }
    }
}
```

## ATR for Stop-Loss Placement

One of the most practical uses of ATR is determining stop-loss levels:

```rust
#[derive(Debug)]
struct Position {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    side: Side,
    stop_loss: f64,
    take_profit: f64,
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Long,
    Short,
}

struct ATRBasedRiskManager {
    atr_multiplier_stop: f64,  // Usually 2.0-3.0
    atr_multiplier_target: f64, // Usually 2.0-4.0
}

impl ATRBasedRiskManager {
    fn new(stop_multiplier: f64, target_multiplier: f64) -> Self {
        ATRBasedRiskManager {
            atr_multiplier_stop: stop_multiplier,
            atr_multiplier_target: target_multiplier,
        }
    }

    fn calculate_position(
        &self,
        symbol: &str,
        entry_price: f64,
        side: Side,
        atr: f64,
        risk_amount: f64,
    ) -> Position {
        let stop_distance = atr * self.atr_multiplier_stop;
        let target_distance = atr * self.atr_multiplier_target;

        let (stop_loss, take_profit) = match side {
            Side::Long => (
                entry_price - stop_distance,
                entry_price + target_distance,
            ),
            Side::Short => (
                entry_price + stop_distance,
                entry_price - target_distance,
            ),
        };

        // Position sizing based on risk
        let quantity = risk_amount / stop_distance;

        Position {
            symbol: symbol.to_string(),
            entry_price,
            quantity,
            side,
            stop_loss,
            take_profit,
        }
    }
}

fn main() {
    let risk_manager = ATRBasedRiskManager::new(2.0, 3.0);

    // Current ATR for BTC is $800
    let current_atr = 800.0;
    let entry_price = 45000.0;
    let risk_per_trade = 1000.0; // Risk $1000 per trade

    let long_position = risk_manager.calculate_position(
        "BTC/USDT",
        entry_price,
        Side::Long,
        current_atr,
        risk_per_trade,
    );

    println!("=== Long Position ===");
    println!("Entry: ${:.2}", long_position.entry_price);
    println!("Stop Loss: ${:.2} ({:.1}% away)",
        long_position.stop_loss,
        ((entry_price - long_position.stop_loss) / entry_price) * 100.0
    );
    println!("Take Profit: ${:.2} ({:.1}% away)",
        long_position.take_profit,
        ((long_position.take_profit - entry_price) / entry_price) * 100.0
    );
    println!("Quantity: {:.6} BTC", long_position.quantity);
    println!("Risk/Reward Ratio: 1:{:.1}",
        (long_position.take_profit - entry_price) / (entry_price - long_position.stop_loss)
    );
}
```

## Multi-Timeframe ATR Analysis

Professional traders often analyze ATR across multiple timeframes:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Timeframe {
    M15,   // 15 minutes
    H1,    // 1 hour
    H4,    // 4 hours
    D1,    // Daily
}

impl Timeframe {
    fn name(&self) -> &str {
        match self {
            Timeframe::M15 => "15min",
            Timeframe::H1 => "1hour",
            Timeframe::H4 => "4hour",
            Timeframe::D1 => "Daily",
        }
    }
}

struct MultiTimeframeATR {
    atrs: HashMap<Timeframe, ATRCalculator>,
}

impl MultiTimeframeATR {
    fn new(period: usize) -> Self {
        let mut atrs = HashMap::new();
        atrs.insert(Timeframe::M15, ATRCalculator::new(period));
        atrs.insert(Timeframe::H1, ATRCalculator::new(period));
        atrs.insert(Timeframe::H4, ATRCalculator::new(period));
        atrs.insert(Timeframe::D1, ATRCalculator::new(period));

        MultiTimeframeATR { atrs }
    }

    fn update(&mut self, timeframe: Timeframe, candle: &Candle) -> Option<f64> {
        self.atrs.get_mut(&timeframe)?.update(candle)
    }

    fn get_atr(&self, timeframe: Timeframe) -> Option<f64> {
        self.atrs.get(&timeframe)?.value()
    }

    fn analyze_volatility(&self) -> VolatilityAnalysis {
        let atrs: Vec<(Timeframe, f64)> = [
            Timeframe::M15,
            Timeframe::H1,
            Timeframe::H4,
            Timeframe::D1,
        ]
        .iter()
        .filter_map(|&tf| self.get_atr(tf).map(|atr| (tf, atr)))
        .collect();

        if atrs.is_empty() {
            return VolatilityAnalysis {
                trend: VolatilityTrend::Unknown,
                recommendation: "Insufficient data".to_string(),
            };
        }

        // Compare short-term vs long-term ATR
        let short_term = self.get_atr(Timeframe::M15).or(self.get_atr(Timeframe::H1));
        let long_term = self.get_atr(Timeframe::D1).or(self.get_atr(Timeframe::H4));

        match (short_term, long_term) {
            (Some(short), Some(long)) => {
                // Normalize by comparing ratios
                let ratio = short / long;

                let (trend, recommendation) = if ratio > 1.5 {
                    (
                        VolatilityTrend::Increasing,
                        "High short-term volatility. Consider tighter stops or reduced position size.".to_string()
                    )
                } else if ratio < 0.5 {
                    (
                        VolatilityTrend::Decreasing,
                        "Low short-term volatility. Potential breakout setup forming.".to_string()
                    )
                } else {
                    (
                        VolatilityTrend::Stable,
                        "Stable volatility. Normal trading conditions.".to_string()
                    )
                };

                VolatilityAnalysis { trend, recommendation }
            }
            _ => VolatilityAnalysis {
                trend: VolatilityTrend::Unknown,
                recommendation: "Waiting for more data...".to_string(),
            },
        }
    }
}

#[derive(Debug)]
enum VolatilityTrend {
    Increasing,
    Decreasing,
    Stable,
    Unknown,
}

#[derive(Debug)]
struct VolatilityAnalysis {
    trend: VolatilityTrend,
    recommendation: String,
}

fn main() {
    let mut mtf_atr = MultiTimeframeATR::new(14);

    println!("=== Multi-Timeframe ATR Analysis ===\n");

    // Simulate some candle data for different timeframes
    // In real trading, these would come from your data feed

    let daily_candle = Candle {
        timestamp: 1,
        open: 44000.0,
        high: 46000.0,
        low: 43500.0,
        close: 45500.0,
        volume: 10000.0,
    };

    let h4_candle = Candle {
        timestamp: 1,
        open: 45000.0,
        high: 45800.0,
        low: 44800.0,
        close: 45500.0,
        volume: 2500.0,
    };

    mtf_atr.update(Timeframe::D1, &daily_candle);
    mtf_atr.update(Timeframe::H4, &h4_candle);

    for tf in [Timeframe::M15, Timeframe::H1, Timeframe::H4, Timeframe::D1] {
        match mtf_atr.get_atr(tf) {
            Some(atr) => println!("{}: ATR = ${:.2}", tf.name(), atr),
            None => println!("{}: ATR = calculating...", tf.name()),
        }
    }

    let analysis = mtf_atr.analyze_volatility();
    println!("\nVolatility Trend: {:?}", analysis.trend);
    println!("Recommendation: {}", analysis.recommendation);
}
```

## ATR-Based Position Sizing

Position sizing is crucial for risk management:

```rust
struct PortfolioManager {
    total_capital: f64,
    risk_per_trade_percent: f64,  // e.g., 1% or 2%
    max_positions: usize,
}

#[derive(Debug)]
struct TradeSetup {
    symbol: String,
    entry_price: f64,
    atr: f64,
    atr_multiplier: f64,
}

#[derive(Debug)]
struct PositionSize {
    quantity: f64,
    dollar_risk: f64,
    stop_loss: f64,
    position_value: f64,
    percent_of_portfolio: f64,
}

impl PortfolioManager {
    fn new(capital: f64, risk_percent: f64, max_positions: usize) -> Self {
        PortfolioManager {
            total_capital: capital,
            risk_per_trade_percent: risk_percent,
            max_positions,
        }
    }

    fn calculate_position_size(&self, setup: &TradeSetup) -> PositionSize {
        // Maximum dollar risk per trade
        let max_risk = self.total_capital * (self.risk_per_trade_percent / 100.0);

        // Stop distance based on ATR
        let stop_distance = setup.atr * setup.atr_multiplier;

        // Calculate quantity based on risk
        let quantity = max_risk / stop_distance;

        // Position value
        let position_value = quantity * setup.entry_price;

        // Percentage of portfolio
        let percent_of_portfolio = (position_value / self.total_capital) * 100.0;

        PositionSize {
            quantity,
            dollar_risk: max_risk,
            stop_loss: setup.entry_price - stop_distance,
            position_value,
            percent_of_portfolio,
        }
    }

    fn validate_position(&self, position: &PositionSize) -> Result<(), String> {
        // Check if position is too large
        if position.percent_of_portfolio > 25.0 {
            return Err(format!(
                "Position too large: {:.1}% of portfolio (max 25%)",
                position.percent_of_portfolio
            ));
        }

        Ok(())
    }
}

fn main() {
    let portfolio = PortfolioManager::new(100_000.0, 2.0, 5);

    let btc_setup = TradeSetup {
        symbol: "BTC/USDT".to_string(),
        entry_price: 45000.0,
        atr: 800.0,
        atr_multiplier: 2.0,
    };

    let eth_setup = TradeSetup {
        symbol: "ETH/USDT".to_string(),
        entry_price: 2500.0,
        atr: 60.0,
        atr_multiplier: 2.0,
    };

    println!("=== ATR-Based Position Sizing ===\n");
    println!("Portfolio: ${:.0}", portfolio.total_capital);
    println!("Risk per trade: {:.1}%\n", portfolio.risk_per_trade_percent);

    for setup in [&btc_setup, &eth_setup] {
        let position = portfolio.calculate_position_size(setup);

        println!("--- {} ---", setup.symbol);
        println!("Entry: ${:.2}", setup.entry_price);
        println!("ATR: ${:.2}", setup.atr);
        println!("Stop Loss: ${:.2}", position.stop_loss);
        println!("Quantity: {:.6}", position.quantity);
        println!("Position Value: ${:.2}", position.position_value);
        println!("Portfolio %: {:.1}%", position.percent_of_portfolio);
        println!("Dollar Risk: ${:.2}", position.dollar_risk);

        match portfolio.validate_position(&position) {
            Ok(()) => println!("Status: VALID"),
            Err(e) => println!("Status: INVALID - {}", e),
        }
        println!();
    }
}
```

## ATR Trailing Stop Strategy

A dynamic trailing stop based on ATR:

```rust
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    side: Side,
    highest_price: f64,  // For long positions
    lowest_price: f64,   // For short positions
    trailing_stop: f64,
}

struct ATRTrailingStop {
    atr_multiplier: f64,
}

impl ATRTrailingStop {
    fn new(multiplier: f64) -> Self {
        ATRTrailingStop {
            atr_multiplier: multiplier,
        }
    }

    fn update_trailing_stop(&self, trade: &mut Trade, current_price: f64, atr: f64) -> TradeAction {
        let trail_distance = atr * self.atr_multiplier;

        match trade.side {
            Side::Long => {
                // Update highest price if we have a new high
                if current_price > trade.highest_price {
                    trade.highest_price = current_price;
                    trade.trailing_stop = current_price - trail_distance;
                    return TradeAction::UpdateStop(trade.trailing_stop);
                }

                // Check if stop is hit
                if current_price <= trade.trailing_stop {
                    let pnl = (trade.trailing_stop - trade.entry_price) * trade.quantity;
                    return TradeAction::Exit {
                        exit_price: trade.trailing_stop,
                        pnl,
                        reason: "Trailing stop hit".to_string(),
                    };
                }

                TradeAction::Hold
            }
            Side::Short => {
                // Update lowest price if we have a new low
                if current_price < trade.lowest_price {
                    trade.lowest_price = current_price;
                    trade.trailing_stop = current_price + trail_distance;
                    return TradeAction::UpdateStop(trade.trailing_stop);
                }

                // Check if stop is hit
                if current_price >= trade.trailing_stop {
                    let pnl = (trade.entry_price - trade.trailing_stop) * trade.quantity;
                    return TradeAction::Exit {
                        exit_price: trade.trailing_stop,
                        pnl,
                        reason: "Trailing stop hit".to_string(),
                    };
                }

                TradeAction::Hold
            }
        }
    }
}

#[derive(Debug)]
enum TradeAction {
    Hold,
    UpdateStop(f64),
    Exit {
        exit_price: f64,
        pnl: f64,
        reason: String,
    },
}

fn main() {
    let trailing_stop = ATRTrailingStop::new(2.0);
    let atr = 800.0;

    let mut trade = Trade {
        symbol: "BTC/USDT".to_string(),
        entry_price: 45000.0,
        quantity: 0.1,
        side: Side::Long,
        highest_price: 45000.0,
        lowest_price: 45000.0,
        trailing_stop: 45000.0 - (atr * 2.0), // Initial stop
    };

    println!("=== ATR Trailing Stop Simulation ===\n");
    println!("Entry: ${:.2}", trade.entry_price);
    println!("Initial Stop: ${:.2}", trade.trailing_stop);
    println!("ATR: ${:.2}\n", atr);

    // Simulate price movements
    let prices = vec![
        45500.0, 46000.0, 46500.0, 47000.0, 47500.0,
        47200.0, 46800.0, 46000.0, 45500.0, 45000.0,
    ];

    for (i, price) in prices.iter().enumerate() {
        let action = trailing_stop.update_trailing_stop(&mut trade, *price, atr);

        print!("Tick {}: Price=${:.2} | ", i + 1, price);

        match action {
            TradeAction::Hold => {
                println!("Hold (Stop: ${:.2})", trade.trailing_stop);
            }
            TradeAction::UpdateStop(new_stop) => {
                println!("Stop updated to ${:.2}", new_stop);
            }
            TradeAction::Exit { exit_price, pnl, reason } => {
                println!("{} at ${:.2}", reason, exit_price);
                println!("\n=== Trade Closed ===");
                println!("Entry: ${:.2}", trade.entry_price);
                println!("Exit: ${:.2}", exit_price);
                println!("PnL: ${:.2}", pnl);
                break;
            }
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| True Range | Maximum of: (High-Low), \|High-PrevClose\|, \|Low-PrevClose\| |
| ATR | Average of True Range values over N periods |
| Wilder's Smoothing | ATR = ((Prev ATR × (n-1)) + TR) / n |
| Stop Placement | Typically 2-3 ATR from entry price |
| Position Sizing | Risk Amount / (ATR × Multiplier) = Quantity |
| Trailing Stop | Stop follows price at ATR distance |

## Homework

1. **ATR with Different Periods**: Implement an ATR calculator that can switch between different periods (7, 14, 21). Compare how they react to the same price data and explain which period is better for:
   - Day trading
   - Swing trading
   - Long-term investing

2. **ATR Breakout Strategy**: Create a simple trading strategy that:
   - Calculates a 14-period ATR
   - Generates a buy signal when price breaks above (Previous Close + 1.5 × ATR)
   - Generates a sell signal when price breaks below (Previous Close - 1.5 × ATR)
   - Tracks hypothetical trades and calculates win rate

3. **Volatility Comparison Tool**: Build a tool that:
   - Calculates ATR for multiple trading pairs (BTC, ETH, SOL)
   - Normalizes ATR as percentage of price (ATR / Price × 100)
   - Ranks assets by volatility
   - Suggests position size adjustments for each asset

4. **ATR-Based Risk Manager**: Create a complete risk management system that:
   - Uses ATR for stop-loss placement (2× ATR)
   - Uses ATR for take-profit levels (3× ATR)
   - Calculates position size based on account risk (1-2%)
   - Validates that total portfolio risk doesn't exceed 10%
   - Logs all calculations for review

## Navigation

[← Previous day](../251-stochastic-oscillator/en.md) | [Next day →](../253-bollinger-bands/en.md)
