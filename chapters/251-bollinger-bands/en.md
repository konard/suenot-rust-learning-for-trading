# Day 251: Bollinger Bands — Volatility Channels for Trading

## Trading Analogy

Imagine you're watching BTC price fluctuations. Sometimes the price moves in a narrow range (low volatility), sometimes it swings wildly (high volatility). Bollinger Bands are like **dynamic boundaries** that expand and contract based on market volatility — they help traders identify when price is relatively high or low compared to recent history.

Think of it as a river: the banks (upper and lower bands) widen during floods (high volatility) and narrow during calm periods. When the price touches the upper bank, it might be overextended; when it touches the lower bank, it might be oversold.

## What Are Bollinger Bands?

Bollinger Bands consist of three lines:
1. **Middle Band**: Simple Moving Average (typically 20 periods)
2. **Upper Band**: Middle Band + (Standard Deviation × multiplier)
3. **Lower Band**: Middle Band - (Standard Deviation × multiplier)

The standard multiplier is 2.0, meaning the bands are 2 standard deviations away from the mean.

## Basic Implementation

```rust
fn main() {
    let prices = [
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
        42800.0, 42750.0, 42900.0, 42850.0, 43000.0,
    ];

    if let Some(bands) = calculate_bollinger_bands(&prices, 20, 2.0) {
        println!("BTC Bollinger Bands:");
        println!("  Upper Band:  ${:.2}", bands.upper);
        println!("  Middle Band: ${:.2}", bands.middle);
        println!("  Lower Band:  ${:.2}", bands.lower);
        println!("  Bandwidth:   {:.4}", bands.bandwidth);
    }
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

fn calculate_bollinger_bands(prices: &[f64], period: usize, multiplier: f64) -> Option<BollingerBands> {
    if prices.len() < period {
        return None;
    }

    let slice = &prices[prices.len() - period..];

    // Calculate SMA (middle band)
    let sma: f64 = slice.iter().sum::<f64>() / period as f64;

    // Calculate standard deviation
    let variance: f64 = slice
        .iter()
        .map(|price| (price - sma).powi(2))
        .sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();

    // Calculate bands
    let upper = sma + (std_dev * multiplier);
    let lower = sma - (std_dev * multiplier);
    let bandwidth = (upper - lower) / sma;  // Normalized bandwidth

    Some(BollingerBands {
        upper,
        middle: sma,
        lower,
        bandwidth,
    })
}
```

## Calculating Rolling Bollinger Bands

For real trading, you need bands calculated at each point in time:

```rust
fn main() {
    let btc_prices = [
        41000.0, 41200.0, 41100.0, 41300.0, 41250.0,
        41400.0, 41350.0, 41500.0, 41450.0, 41600.0,
        41700.0, 41650.0, 41800.0, 41750.0, 41900.0,
        42000.0, 42100.0, 42050.0, 42200.0, 42300.0,
        42500.0, 42400.0, 42600.0, 42800.0, 43000.0,
    ];

    let bands_series = calculate_rolling_bollinger(&btc_prices, 10, 2.0);

    println!("Rolling Bollinger Bands (last 5 values):");
    println!("{:>10} {:>12} {:>12} {:>12} {:>10}",
             "Price", "Upper", "Middle", "Lower", "Position");

    for (i, bands) in bands_series.iter().rev().take(5).rev().enumerate() {
        let idx = btc_prices.len() - 5 + i;
        let price = btc_prices[idx];
        let position = calculate_position(price, bands);
        println!("{:>10.2} {:>12.2} {:>12.2} {:>12.2} {:>10.2}%",
                 price, bands.upper, bands.middle, bands.lower, position * 100.0);
    }
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

fn calculate_rolling_bollinger(prices: &[f64], period: usize, multiplier: f64) -> Vec<BollingerBands> {
    let mut result = Vec::new();

    for i in period..=prices.len() {
        let slice = &prices[i - period..i];

        let sma: f64 = slice.iter().sum::<f64>() / period as f64;
        let variance: f64 = slice
            .iter()
            .map(|p| (p - sma).powi(2))
            .sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();

        let upper = sma + (std_dev * multiplier);
        let lower = sma - (std_dev * multiplier);

        result.push(BollingerBands {
            upper,
            middle: sma,
            lower,
            bandwidth: (upper - lower) / sma,
        });
    }

    result
}

fn calculate_position(price: f64, bands: &BollingerBands) -> f64 {
    // Returns position within bands: 0.0 = lower band, 1.0 = upper band
    (price - bands.lower) / (bands.upper - bands.lower)
}
```

## Trading Signals with Bollinger Bands

```rust
fn main() {
    let prices = [
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
        42800.0, 42750.0, 42900.0, 42850.0, 43200.0,  // Price spike
    ];

    let period = 20;
    let multiplier = 2.0;

    if let Some(bands) = calculate_bollinger_bands(&prices, period, multiplier) {
        let current_price = prices[prices.len() - 1];
        let signal = generate_signal(current_price, &bands);

        println!("Current BTC Price: ${:.2}", current_price);
        println!("Bollinger Bands:");
        println!("  Upper:  ${:.2}", bands.upper);
        println!("  Middle: ${:.2}", bands.middle);
        println!("  Lower:  ${:.2}", bands.lower);
        println!("\nSignal: {:?}", signal);
        println!("Reason: {}", get_signal_reason(current_price, &bands));
    }
}

#[derive(Debug)]
enum TradingSignal {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

fn calculate_bollinger_bands(prices: &[f64], period: usize, multiplier: f64) -> Option<BollingerBands> {
    if prices.len() < period {
        return None;
    }

    let slice = &prices[prices.len() - period..];
    let sma: f64 = slice.iter().sum::<f64>() / period as f64;
    let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();

    let upper = sma + (std_dev * multiplier);
    let lower = sma - (std_dev * multiplier);

    Some(BollingerBands {
        upper,
        middle: sma,
        lower,
        bandwidth: (upper - lower) / sma,
    })
}

fn generate_signal(price: f64, bands: &BollingerBands) -> TradingSignal {
    let position = (price - bands.lower) / (bands.upper - bands.lower);

    if price < bands.lower {
        TradingSignal::StrongBuy  // Price below lower band - oversold
    } else if position < 0.2 {
        TradingSignal::Buy        // Near lower band
    } else if price > bands.upper {
        TradingSignal::StrongSell // Price above upper band - overbought
    } else if position > 0.8 {
        TradingSignal::Sell       // Near upper band
    } else {
        TradingSignal::Hold       // Price in middle range
    }
}

fn get_signal_reason(price: f64, bands: &BollingerBands) -> String {
    if price < bands.lower {
        format!("Price ${:.2} is below lower band ${:.2} - potential oversold condition",
                price, bands.lower)
    } else if price > bands.upper {
        format!("Price ${:.2} is above upper band ${:.2} - potential overbought condition",
                price, bands.upper)
    } else {
        let position = ((price - bands.lower) / (bands.upper - bands.lower)) * 100.0;
        format!("Price is at {:.1}% of band range - within normal bounds", position)
    }
}
```

## Bollinger Band Squeeze Detection

The "squeeze" occurs when bands narrow significantly, often preceding a big move:

```rust
fn main() {
    // Simulating a squeeze scenario
    let prices = [
        42000.0, 42050.0, 42025.0, 42075.0, 42050.0,
        42060.0, 42040.0, 42055.0, 42045.0, 42052.0,
        42048.0, 42051.0, 42049.0, 42050.0, 42050.0,  // Very tight range
        42051.0, 42049.0, 42050.0, 42050.0, 42050.0,  // Squeeze!
    ];

    let bands_history = calculate_bands_history(&prices, 10, 2.0);

    if let Some(squeeze) = detect_squeeze(&bands_history, 0.02) {
        println!("SQUEEZE DETECTED!");
        println!("Current bandwidth: {:.4}", squeeze.current_bandwidth);
        println!("Average bandwidth: {:.4}", squeeze.avg_bandwidth);
        println!("Squeeze ratio: {:.2}%", squeeze.squeeze_ratio * 100.0);
        println!("\nTrading implication: Expect increased volatility soon!");
    } else {
        println!("No squeeze detected - normal volatility");
    }
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

struct SqueezeInfo {
    current_bandwidth: f64,
    avg_bandwidth: f64,
    squeeze_ratio: f64,
}

fn calculate_bands_history(prices: &[f64], period: usize, multiplier: f64) -> Vec<BollingerBands> {
    let mut result = Vec::new();

    for i in period..=prices.len() {
        let slice = &prices[i - period..i];
        let sma: f64 = slice.iter().sum::<f64>() / period as f64;
        let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();

        let upper = sma + (std_dev * multiplier);
        let lower = sma - (std_dev * multiplier);

        result.push(BollingerBands {
            upper,
            middle: sma,
            lower,
            bandwidth: (upper - lower) / sma,
        });
    }

    result
}

fn detect_squeeze(bands_history: &[BollingerBands], threshold: f64) -> Option<SqueezeInfo> {
    if bands_history.is_empty() {
        return None;
    }

    let avg_bandwidth: f64 = bands_history.iter().map(|b| b.bandwidth).sum::<f64>()
                            / bands_history.len() as f64;
    let current_bandwidth = bands_history.last()?.bandwidth;
    let squeeze_ratio = current_bandwidth / avg_bandwidth;

    if squeeze_ratio < (1.0 - threshold) {
        Some(SqueezeInfo {
            current_bandwidth,
            avg_bandwidth,
            squeeze_ratio,
        })
    } else {
        None
    }
}
```

## %B Indicator — Position Within Bands

```rust
fn main() {
    let prices = [
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
        42800.0, 42750.0, 42900.0, 42850.0, 43000.0,
    ];

    let period = 20;
    let multiplier = 2.0;

    if let Some(bands) = calculate_bollinger_bands(&prices, period, multiplier) {
        let current_price = prices[prices.len() - 1];
        let percent_b = calculate_percent_b(current_price, &bands);

        println!("BTC Price: ${:.2}", current_price);
        println!("%B Value: {:.4}", percent_b);
        println!();
        interpret_percent_b(percent_b);
    }
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

fn calculate_bollinger_bands(prices: &[f64], period: usize, multiplier: f64) -> Option<BollingerBands> {
    if prices.len() < period {
        return None;
    }

    let slice = &prices[prices.len() - period..];
    let sma: f64 = slice.iter().sum::<f64>() / period as f64;
    let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();

    let upper = sma + (std_dev * multiplier);
    let lower = sma - (std_dev * multiplier);

    Some(BollingerBands {
        upper,
        middle: sma,
        lower,
        bandwidth: (upper - lower) / sma,
    })
}

fn calculate_percent_b(price: f64, bands: &BollingerBands) -> f64 {
    // %B = (Price - Lower Band) / (Upper Band - Lower Band)
    // Returns: 0.0 at lower band, 0.5 at middle, 1.0 at upper band
    (price - bands.lower) / (bands.upper - bands.lower)
}

fn interpret_percent_b(percent_b: f64) {
    println!("Interpretation:");
    if percent_b > 1.0 {
        println!("  > Price is ABOVE upper band ({:.1}%)", (percent_b - 1.0) * 100.0);
        println!("  > Strongly overbought - consider selling");
    } else if percent_b > 0.8 {
        println!("  > Price is near upper band");
        println!("  > Overbought territory - watch for reversal");
    } else if percent_b < 0.0 {
        println!("  > Price is BELOW lower band ({:.1}%)", percent_b.abs() * 100.0);
        println!("  > Strongly oversold - consider buying");
    } else if percent_b < 0.2 {
        println!("  > Price is near lower band");
        println!("  > Oversold territory - watch for bounce");
    } else {
        println!("  > Price is within normal range");
        println!("  > No extreme conditions detected");
    }
}
```

## Complete Trading Strategy with Bollinger Bands

```rust
fn main() {
    let btc_prices = [
        41000.0, 41200.0, 41100.0, 41300.0, 41250.0,
        41400.0, 41350.0, 41500.0, 41450.0, 41600.0,
        41700.0, 41650.0, 41800.0, 41750.0, 41900.0,
        42000.0, 42100.0, 42050.0, 42200.0, 42300.0,
        42500.0, 42400.0, 42600.0, 42800.0, 43000.0,
        43200.0, 43100.0, 43400.0, 43300.0, 40500.0,  // Sharp drop
    ];

    let mut strategy = BollingerStrategy::new(20, 2.0);
    let mut portfolio = Portfolio::new(10000.0);

    println!("Bollinger Bands Trading Simulation");
    println!("===================================\n");

    for (i, &price) in btc_prices.iter().enumerate() {
        if let Some(action) = strategy.analyze(price) {
            println!("Day {}: BTC ${:.2}", i + 1, price);
            println!("  Signal: {:?}", action.signal);
            println!("  %B: {:.4}", action.percent_b);

            match action.signal {
                Signal::Buy if portfolio.cash > 0.0 => {
                    let qty = (portfolio.cash * 0.5) / price;
                    portfolio.buy(price, qty);
                    println!("  ACTION: Bought {:.6} BTC", qty);
                }
                Signal::Sell if portfolio.btc > 0.0 => {
                    let qty = portfolio.btc * 0.5;
                    portfolio.sell(price, qty);
                    println!("  ACTION: Sold {:.6} BTC", qty);
                }
                _ => println!("  ACTION: Hold"),
            }
            println!("  Portfolio: ${:.2} + {:.6} BTC\n", portfolio.cash, portfolio.btc);
        }
    }

    let final_price = btc_prices[btc_prices.len() - 1];
    let total_value = portfolio.cash + portfolio.btc * final_price;
    println!("Final Portfolio Value: ${:.2}", total_value);
    println!("Return: {:.2}%", (total_value / 10000.0 - 1.0) * 100.0);
}

#[derive(Debug)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

struct TradeAction {
    signal: Signal,
    percent_b: f64,
}

struct BollingerStrategy {
    period: usize,
    multiplier: f64,
    prices: Vec<f64>,
}

impl BollingerStrategy {
    fn new(period: usize, multiplier: f64) -> Self {
        BollingerStrategy {
            period,
            multiplier,
            prices: Vec::new(),
        }
    }

    fn analyze(&mut self, price: f64) -> Option<TradeAction> {
        self.prices.push(price);

        if self.prices.len() < self.period {
            return None;
        }

        let slice = &self.prices[self.prices.len() - self.period..];
        let sma: f64 = slice.iter().sum::<f64>() / self.period as f64;
        let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / self.period as f64;
        let std_dev = variance.sqrt();

        let upper = sma + (std_dev * self.multiplier);
        let lower = sma - (std_dev * self.multiplier);
        let percent_b = (price - lower) / (upper - lower);

        let signal = if percent_b < 0.0 {
            Signal::Buy   // Below lower band
        } else if percent_b > 1.0 {
            Signal::Sell  // Above upper band
        } else {
            Signal::Hold
        };

        Some(TradeAction { signal, percent_b })
    }
}

struct Portfolio {
    cash: f64,
    btc: f64,
}

impl Portfolio {
    fn new(initial_cash: f64) -> Self {
        Portfolio { cash: initial_cash, btc: 0.0 }
    }

    fn buy(&mut self, price: f64, quantity: f64) {
        let cost = price * quantity;
        if cost <= self.cash {
            self.cash -= cost;
            self.btc += quantity;
        }
    }

    fn sell(&mut self, price: f64, quantity: f64) {
        if quantity <= self.btc {
            self.btc -= quantity;
            self.cash += price * quantity;
        }
    }
}
```

## Risk Management with Bollinger Bands

```rust
fn main() {
    let current_price = 42500.0;
    let portfolio_value = 50000.0;
    let risk_percent = 2.0;

    let prices = [
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
        42800.0, 42750.0, 42900.0, 42850.0, 42500.0,
    ];

    if let Some(risk) = calculate_position_with_bands(&prices, 20, 2.0, portfolio_value, risk_percent) {
        println!("Position Sizing with Bollinger Bands");
        println!("====================================");
        println!("Current Price: ${:.2}", current_price);
        println!("Stop Loss (Lower Band): ${:.2}", risk.stop_loss);
        println!("Risk Amount: ${:.2}", risk.risk_amount);
        println!("Position Size: {:.6} BTC", risk.position_size);
        println!("Position Value: ${:.2}", risk.position_value);
    }
}

struct PositionRisk {
    stop_loss: f64,
    risk_amount: f64,
    position_size: f64,
    position_value: f64,
}

fn calculate_position_with_bands(
    prices: &[f64],
    period: usize,
    multiplier: f64,
    portfolio_value: f64,
    risk_percent: f64,
) -> Option<PositionRisk> {
    if prices.len() < period {
        return None;
    }

    let slice = &prices[prices.len() - period..];
    let sma: f64 = slice.iter().sum::<f64>() / period as f64;
    let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();
    let lower_band = sma - (std_dev * multiplier);

    let current_price = prices[prices.len() - 1];
    let risk_per_unit = current_price - lower_band;

    if risk_per_unit <= 0.0 {
        return None;
    }

    let risk_amount = portfolio_value * (risk_percent / 100.0);
    let position_size = risk_amount / risk_per_unit;
    let position_value = position_size * current_price;

    Some(PositionRisk {
        stop_loss: lower_band,
        risk_amount,
        position_size,
        position_value,
    })
}
```

## What We Learned

| Concept | Description | Trading Use |
|---------|-------------|-------------|
| Middle Band | SMA of closing prices | Trend direction |
| Upper Band | SMA + (StdDev × multiplier) | Resistance / Overbought |
| Lower Band | SMA - (StdDev × multiplier) | Support / Oversold |
| Bandwidth | (Upper - Lower) / Middle | Volatility measure |
| %B | (Price - Lower) / (Upper - Lower) | Position within bands |
| Squeeze | Narrow bandwidth | Anticipate breakout |

## Homework

1. Implement a function `fn calculate_bollinger_with_ema(prices: &[f64], period: usize, multiplier: f64) -> Option<BollingerBands>` that uses EMA instead of SMA for the middle band

2. Create a function `fn detect_double_bottom(prices: &[f64], bands: &[BollingerBands]) -> Option<usize>` that detects when price touches the lower band twice — a classic reversal pattern

3. Write a backtesting function `fn backtest_bollinger_strategy(prices: &[f64], period: usize, multiplier: f64) -> BacktestResult` that simulates trading based on Bollinger Band signals and returns total return, win rate, and max drawdown

4. Implement a multi-timeframe analysis function `fn analyze_multi_timeframe(prices_1h: &[f64], prices_4h: &[f64], prices_1d: &[f64]) -> TradingSignal` that combines Bollinger Band signals from different timeframes for stronger confirmation

## Navigation

[← Previous day](../250-macd/en.md) | [Next day →](../252-atr/en.md)
