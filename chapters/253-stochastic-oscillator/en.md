# Day 253: Stochastic Oscillator

## Trading Analogy

Imagine you're watching a ball bouncing between the floor and ceiling of a room. When the ball approaches the ceiling, it loses momentum and will soon start falling. When it approaches the floor, it's about to bounce and start rising.

The **Stochastic Oscillator** works exactly the same way for market prices! It measures where the current price is relative to the price range over a specific period:
- If the price is close to the high of the range (near the ceiling) — the asset is **overbought** and may soon go down
- If the price is close to the low of the range (near the floor) — the asset is **oversold** and may soon go up

This indicator was developed by George Lane in the 1950s and remains one of the most popular technical analysis tools to this day.

## What is the Stochastic Oscillator?

The Stochastic Oscillator is a momentum indicator that compares the closing price to the price range over a specific period.

### Formulas

The oscillator consists of two lines:

**%K (fast line)**:
```
%K = ((Current Close - Lowest Low over N periods) / (Highest High over N periods - Lowest Low over N periods)) × 100
```

**%D (slow line)** — smoothed version of %K:
```
%D = SMA(%K, M periods)
```

Where:
- N — typically 14 periods
- M — typically 3 periods for smoothing

### Interpreting Values

| Zone | Value | Interpretation |
|------|-------|----------------|
| Overbought | > 80 | Asset may be overvalued, potential reversal down |
| Neutral | 20-80 | Normal market condition |
| Oversold | < 20 | Asset may be undervalued, potential reversal up |

## Basic Implementation

```rust
/// Structure to hold candlestick data (OHLC)
#[derive(Debug, Clone)]
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    timestamp: u64,
}

/// Result of stochastic oscillator calculation
#[derive(Debug, Clone)]
struct StochasticResult {
    k_value: f64,  // Fast line %K
    d_value: f64,  // Slow line %D
    signal: Signal,
}

#[derive(Debug, Clone, PartialEq)]
enum Signal {
    Buy,
    Sell,
    Neutral,
}

/// Stochastic oscillator calculator
struct StochasticOscillator {
    k_period: usize,  // Period for %K calculation (typically 14)
    d_period: usize,  // Period for %D smoothing (typically 3)
    overbought: f64,  // Overbought level (typically 80)
    oversold: f64,    // Oversold level (typically 20)
}

impl StochasticOscillator {
    fn new(k_period: usize, d_period: usize) -> Self {
        StochasticOscillator {
            k_period,
            d_period,
            overbought: 80.0,
            oversold: 20.0,
        }
    }

    /// Finds the highest High over the period
    fn highest_high(&self, candles: &[Candle]) -> f64 {
        candles
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Finds the lowest Low over the period
    fn lowest_low(&self, candles: &[Candle]) -> f64 {
        candles
            .iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, f64::min)
    }

    /// Calculates %K value for a single period
    fn calculate_k(&self, candles: &[Candle]) -> Option<f64> {
        if candles.len() < self.k_period {
            return None;
        }

        let period_candles = &candles[candles.len() - self.k_period..];
        let highest = self.highest_high(period_candles);
        let lowest = self.lowest_low(period_candles);
        let current_close = candles.last()?.close;

        // Avoid division by zero
        if (highest - lowest).abs() < f64::EPSILON {
            return Some(50.0); // If range is zero, return middle
        }

        let k = ((current_close - lowest) / (highest - lowest)) * 100.0;
        Some(k.clamp(0.0, 100.0))
    }

    /// Calculates SMA for %D smoothing
    fn calculate_sma(&self, values: &[f64], period: usize) -> Option<f64> {
        if values.len() < period {
            return None;
        }
        let sum: f64 = values[values.len() - period..].iter().sum();
        Some(sum / period as f64)
    }

    /// Full stochastic calculation for a series of candles
    fn calculate(&self, candles: &[Candle]) -> Vec<StochasticResult> {
        let mut results = Vec::new();
        let mut k_values = Vec::new();

        // Calculate %K for each position
        for i in self.k_period..=candles.len() {
            if let Some(k) = self.calculate_k(&candles[..i]) {
                k_values.push(k);
            }
        }

        // Calculate %D and generate results
        for i in self.d_period..=k_values.len() {
            let k_value = k_values[i - 1];
            let d_value = self.calculate_sma(&k_values[..i], self.d_period)
                .unwrap_or(k_value);

            // Determine signal
            let signal = self.generate_signal(k_value, d_value, &k_values, i);

            results.push(StochasticResult {
                k_value,
                d_value,
                signal,
            });
        }

        results
    }

    /// Generates trading signal
    fn generate_signal(
        &self,
        k: f64,
        d: f64,
        k_history: &[f64],
        current_idx: usize,
    ) -> Signal {
        if current_idx < 2 {
            return Signal::Neutral;
        }

        let prev_k = k_history[current_idx - 2];
        let prev_d = if current_idx >= self.d_period + 1 {
            k_history[current_idx - self.d_period - 1..current_idx - 1]
                .iter()
                .sum::<f64>() / self.d_period as f64
        } else {
            prev_k
        };

        // Bullish signal: %K crosses above %D in oversold zone
        if k > d && prev_k <= prev_d && k < self.oversold + 10.0 {
            return Signal::Buy;
        }

        // Bearish signal: %K crosses below %D in overbought zone
        if k < d && prev_k >= prev_d && k > self.overbought - 10.0 {
            return Signal::Sell;
        }

        Signal::Neutral
    }
}

fn main() {
    // Create test data — historical BTC/USDT candles
    let candles = vec![
        Candle { open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0, timestamp: 1 },
        Candle { open: 42300.0, high: 42800.0, low: 42100.0, close: 42600.0, timestamp: 2 },
        Candle { open: 42600.0, high: 43200.0, low: 42400.0, close: 43000.0, timestamp: 3 },
        Candle { open: 43000.0, high: 43500.0, low: 42800.0, close: 43400.0, timestamp: 4 },
        Candle { open: 43400.0, high: 44000.0, low: 43200.0, close: 43800.0, timestamp: 5 },
        Candle { open: 43800.0, high: 44200.0, low: 43500.0, close: 44100.0, timestamp: 6 },
        Candle { open: 44100.0, high: 44500.0, low: 43900.0, close: 44400.0, timestamp: 7 },
        Candle { open: 44400.0, high: 44800.0, low: 44200.0, close: 44600.0, timestamp: 8 },
        Candle { open: 44600.0, high: 44700.0, low: 44000.0, close: 44200.0, timestamp: 9 },
        Candle { open: 44200.0, high: 44400.0, low: 43800.0, close: 43900.0, timestamp: 10 },
        Candle { open: 43900.0, high: 44100.0, low: 43500.0, close: 43600.0, timestamp: 11 },
        Candle { open: 43600.0, high: 43800.0, low: 43200.0, close: 43300.0, timestamp: 12 },
        Candle { open: 43300.0, high: 43500.0, low: 42800.0, close: 42900.0, timestamp: 13 },
        Candle { open: 42900.0, high: 43100.0, low: 42500.0, close: 42600.0, timestamp: 14 },
        Candle { open: 42600.0, high: 42800.0, low: 42200.0, close: 42300.0, timestamp: 15 },
        Candle { open: 42300.0, high: 42600.0, low: 42100.0, close: 42500.0, timestamp: 16 },
        Candle { open: 42500.0, high: 43000.0, low: 42400.0, close: 42900.0, timestamp: 17 },
        Candle { open: 42900.0, high: 43400.0, low: 42800.0, close: 43300.0, timestamp: 18 },
    ];

    // Create oscillator with periods 14 and 3
    let stochastic = StochasticOscillator::new(14, 3);
    let results = stochastic.calculate(&candles);

    println!("=== Stochastic Oscillator BTC/USDT ===\n");
    println!("{:>8} {:>8} {:>10}", "%K", "%D", "Signal");
    println!("{:-<30}", "");

    for result in &results {
        let signal_str = match result.signal {
            Signal::Buy => "BUY",
            Signal::Sell => "SELL",
            Signal::Neutral => "-",
        };
        println!(
            "{:>8.2} {:>8.2} {:>10}",
            result.k_value, result.d_value, signal_str
        );
    }
}
```

## Advanced Implementation with Trading Strategy

```rust
use std::collections::VecDeque;

/// Trading position
#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    side: PositionSide,
    entry_price: f64,
    quantity: f64,
    stop_loss: f64,
    take_profit: f64,
}

#[derive(Debug, Clone, PartialEq)]
enum PositionSide {
    Long,
    Short,
}

/// Stochastic-based trading engine
struct StochasticTradingEngine {
    oscillator: StochasticOscillator,
    candle_buffer: VecDeque<Candle>,
    current_position: Option<Position>,
    balance: f64,
    trade_history: Vec<TradeResult>,
}

#[derive(Debug, Clone)]
struct TradeResult {
    symbol: String,
    side: PositionSide,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
    pnl_percent: f64,
}

impl StochasticTradingEngine {
    fn new(k_period: usize, d_period: usize, initial_balance: f64) -> Self {
        StochasticTradingEngine {
            oscillator: StochasticOscillator::new(k_period, d_period),
            candle_buffer: VecDeque::with_capacity(k_period + d_period),
            current_position: None,
            balance: initial_balance,
            trade_history: Vec::new(),
        }
    }

    /// Adds a new candle and returns trading signal
    fn on_candle(&mut self, candle: Candle) -> Option<Signal> {
        self.candle_buffer.push_back(candle.clone());

        // Limit buffer size
        let max_size = self.oscillator.k_period + self.oscillator.d_period + 10;
        while self.candle_buffer.len() > max_size {
            self.candle_buffer.pop_front();
        }

        // Check stop-loss and take-profit
        self.check_exit_conditions(&candle);

        // Calculate stochastic
        let candles: Vec<Candle> = self.candle_buffer.iter().cloned().collect();
        let results = self.oscillator.calculate(&candles);

        results.last().map(|r| r.signal.clone())
    }

    /// Checks exit conditions for position
    fn check_exit_conditions(&mut self, candle: &Candle) {
        if let Some(position) = &self.current_position {
            let should_close = match position.side {
                PositionSide::Long => {
                    candle.low <= position.stop_loss || candle.high >= position.take_profit
                }
                PositionSide::Short => {
                    candle.high >= position.stop_loss || candle.low <= position.take_profit
                }
            };

            if should_close {
                let exit_price = match position.side {
                    PositionSide::Long => {
                        if candle.low <= position.stop_loss {
                            position.stop_loss
                        } else {
                            position.take_profit
                        }
                    }
                    PositionSide::Short => {
                        if candle.high >= position.stop_loss {
                            position.stop_loss
                        } else {
                            position.take_profit
                        }
                    }
                };

                self.close_position(exit_price);
            }
        }
    }

    /// Opens a new position
    fn open_position(&mut self, symbol: &str, side: PositionSide, price: f64, risk_percent: f64) {
        if self.current_position.is_some() {
            println!("Position already open!");
            return;
        }

        // Calculate position size based on risk
        let risk_amount = self.balance * risk_percent;
        let stop_distance = price * 0.02; // 2% stop-loss

        let quantity = risk_amount / stop_distance;
        let position_value = quantity * price;

        if position_value > self.balance {
            println!("Insufficient funds!");
            return;
        }

        let (stop_loss, take_profit) = match side {
            PositionSide::Long => (price - stop_distance, price + stop_distance * 2.0),
            PositionSide::Short => (price + stop_distance, price - stop_distance * 2.0),
        };

        let position = Position {
            symbol: symbol.to_string(),
            side: side.clone(),
            entry_price: price,
            quantity,
            stop_loss,
            take_profit,
        };

        println!(
            "Opened {:?} position: {} @ {:.2}, SL: {:.2}, TP: {:.2}",
            side, symbol, price, stop_loss, take_profit
        );

        self.current_position = Some(position);
    }

    /// Closes current position
    fn close_position(&mut self, exit_price: f64) {
        if let Some(position) = self.current_position.take() {
            let pnl = match position.side {
                PositionSide::Long => (exit_price - position.entry_price) * position.quantity,
                PositionSide::Short => (position.entry_price - exit_price) * position.quantity,
            };

            let pnl_percent = (pnl / (position.entry_price * position.quantity)) * 100.0;

            self.balance += pnl;

            let trade = TradeResult {
                symbol: position.symbol.clone(),
                side: position.side.clone(),
                entry_price: position.entry_price,
                exit_price,
                quantity: position.quantity,
                pnl,
                pnl_percent,
            };

            println!(
                "Closed {:?} position: {} @ {:.2} -> {:.2}, PnL: {:.2} ({:.2}%)",
                position.side, position.symbol, position.entry_price, exit_price, pnl, pnl_percent
            );

            self.trade_history.push(trade);
        }
    }

    /// Returns trading statistics
    fn get_statistics(&self) -> TradingStatistics {
        let total_trades = self.trade_history.len();
        let winning_trades = self.trade_history.iter().filter(|t| t.pnl > 0.0).count();
        let losing_trades = self.trade_history.iter().filter(|t| t.pnl < 0.0).count();

        let total_pnl: f64 = self.trade_history.iter().map(|t| t.pnl).sum();
        let gross_profit: f64 = self.trade_history.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();
        let gross_loss: f64 = self.trade_history.iter().filter(|t| t.pnl < 0.0).map(|t| t.pnl.abs()).sum();

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        TradingStatistics {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_pnl,
            profit_factor,
            final_balance: self.balance,
        }
    }
}

#[derive(Debug)]
struct TradingStatistics {
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    win_rate: f64,
    total_pnl: f64,
    profit_factor: f64,
    final_balance: f64,
}

fn main() {
    let mut engine = StochasticTradingEngine::new(14, 3, 10000.0);

    // Simulate trading
    let candles = generate_sample_candles();

    println!("=== Stochastic-Based Trading Strategy ===\n");

    for candle in candles {
        if let Some(signal) = engine.on_candle(candle.clone()) {
            match signal {
                Signal::Buy if engine.current_position.is_none() => {
                    engine.open_position("BTC/USDT", PositionSide::Long, candle.close, 0.02);
                }
                Signal::Sell if engine.current_position.is_none() => {
                    engine.open_position("BTC/USDT", PositionSide::Short, candle.close, 0.02);
                }
                _ => {}
            }
        }
    }

    // Close position at the end if open
    if engine.current_position.is_some() {
        engine.close_position(42500.0);
    }

    let stats = engine.get_statistics();
    println!("\n=== Trading Statistics ===");
    println!("Total trades: {}", stats.total_trades);
    println!("Winning: {}", stats.winning_trades);
    println!("Losing: {}", stats.losing_trades);
    println!("Win Rate: {:.2}%", stats.win_rate);
    println!("Total PnL: ${:.2}", stats.total_pnl);
    println!("Profit Factor: {:.2}", stats.profit_factor);
    println!("Final Balance: ${:.2}", stats.final_balance);
}

fn generate_sample_candles() -> Vec<Candle> {
    // Generate realistic data for testing
    let mut candles = Vec::new();
    let mut price = 42000.0;

    for i in 0..50 {
        let volatility = 200.0;
        let change = if i % 7 < 3 {
            volatility * 0.5
        } else if i % 7 < 5 {
            -volatility * 0.3
        } else {
            volatility * 0.1
        };

        let open = price;
        let close = price + change;
        let high = f64::max(open, close) + volatility * 0.3;
        let low = f64::min(open, close) - volatility * 0.3;

        candles.push(Candle {
            open,
            high,
            low,
            close,
            timestamp: i as u64,
        });

        price = close;
    }

    candles
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Stochastic Oscillator | Momentum indicator comparing closing price to range |
| %K line | Fast line showing current price position |
| %D line | Slow line (SMA of %K) for smoothing |
| Overbought | Zone above 80 — potential reversal down |
| Oversold | Zone below 20 — potential reversal up |
| Line crossover | Signal for entering/exiting positions |

## Practical Exercises

1. **Manual calculation**: Given an array of closing prices `[100, 102, 98, 105, 103, 107, 104, 108]`. Calculate the %K value for the last candle with period 5.

2. **Zone identification**: For stochastic values `[78, 82, 85, 83, 79, 75]`, determine when the asset entered the overbought zone and when it exited.

3. **Finding signals**: Given %K values `[15, 18, 25, 30]` and %D values `[12, 16, 22, 28]`. Determine if there is a buy signal.

## Homework

1. **Slow Stochastic**: Implement the Slow Stochastic version where:
   - Slow %K = Fast %D (SMA of Fast %K)
   - Slow %D = SMA of Slow %K

   This provides smoother signals.

2. **Divergence**: Add a function to detect divergence:
   - Bullish divergence: price makes a new low, but stochastic doesn't
   - Bearish divergence: price makes a new high, but stochastic doesn't

3. **Multi-timeframe analysis**: Create a structure that analyzes stochastic on multiple timeframes (1h, 4h, 1d) and generates a signal only when all timeframes align.

4. **Backtesting**: Write a complete backtester for the stochastic strategy:
   - Load historical data from CSV
   - Calculate metrics: Sharpe Ratio, Maximum Drawdown, Win Rate
   - Output equity curve graph (can be text-based in console)

## Navigation

[← Previous day](../252-rsi-indicator/en.md) | [Next day →](../254-macd-indicator/en.md)
