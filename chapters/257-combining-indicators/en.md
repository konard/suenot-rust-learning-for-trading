# Day 257: Combining Indicators — Building a Complete Trading System

## Trading Analogy

Professional traders rarely rely on a single indicator. Imagine a pilot landing an aircraft: they don't just look at altitude — they monitor speed, descent rate, fuel, and runway alignment simultaneously. Similarly, experienced traders combine multiple indicators to filter out false signals and confirm trading opportunities. One indicator says "buy," but the others disagree? Hold off. All indicators align? That's a high-confidence signal.

## The Indicator Trait (Recap from Day 256)

Before combining indicators, let's establish our foundation — the `Indicator` trait:

```rust
trait Indicator {
    fn name(&self) -> &str;
    fn calculate(&mut self, price: f64) -> Option<f64>;
    fn is_ready(&self) -> bool;
}
```

This trait provides a unified interface for all indicators, making them composable.

## Simple Moving Average (SMA)

```rust
struct SMA {
    period: usize,
    prices: Vec<f64>,
}

impl SMA {
    fn new(period: usize) -> Self {
        SMA {
            period,
            prices: Vec::with_capacity(period),
        }
    }
}

impl Indicator for SMA {
    fn name(&self) -> &str {
        "SMA"
    }

    fn calculate(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }

        if self.prices.len() == self.period {
            Some(self.prices.iter().sum::<f64>() / self.period as f64)
        } else {
            None
        }
    }

    fn is_ready(&self) -> bool {
        self.prices.len() >= self.period
    }
}
```

## Exponential Moving Average (EMA)

```rust
struct EMA {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    count: usize,
}

impl EMA {
    fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        EMA {
            period,
            multiplier,
            current_ema: None,
            count: 0,
        }
    }
}

impl Indicator for EMA {
    fn name(&self) -> &str {
        "EMA"
    }

    fn calculate(&mut self, price: f64) -> Option<f64> {
        self.count += 1;

        self.current_ema = Some(match self.current_ema {
            None => price,
            Some(prev_ema) => (price - prev_ema) * self.multiplier + prev_ema,
        });

        if self.count >= self.period {
            self.current_ema
        } else {
            None
        }
    }

    fn is_ready(&self) -> bool {
        self.count >= self.period
    }
}
```

## Relative Strength Index (RSI)

```rust
struct RSI {
    period: usize,
    gains: Vec<f64>,
    losses: Vec<f64>,
    prev_price: Option<f64>,
}

impl RSI {
    fn new(period: usize) -> Self {
        RSI {
            period,
            gains: Vec::new(),
            losses: Vec::new(),
            prev_price: None,
        }
    }
}

impl Indicator for RSI {
    fn name(&self) -> &str {
        "RSI"
    }

    fn calculate(&mut self, price: f64) -> Option<f64> {
        if let Some(prev) = self.prev_price {
            let change = price - prev;
            if change > 0.0 {
                self.gains.push(change);
                self.losses.push(0.0);
            } else {
                self.gains.push(0.0);
                self.losses.push(change.abs());
            }

            if self.gains.len() > self.period {
                self.gains.remove(0);
                self.losses.remove(0);
            }
        }
        self.prev_price = Some(price);

        if self.gains.len() == self.period {
            let avg_gain: f64 = self.gains.iter().sum::<f64>() / self.period as f64;
            let avg_loss: f64 = self.losses.iter().sum::<f64>() / self.period as f64;

            if avg_loss == 0.0 {
                Some(100.0)
            } else {
                let rs = avg_gain / avg_loss;
                Some(100.0 - (100.0 / (1.0 + rs)))
            }
        } else {
            None
        }
    }

    fn is_ready(&self) -> bool {
        self.gains.len() >= self.period
    }
}
```

## Combining Indicators: The Signal Aggregator

Now let's build a system that combines multiple indicators:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Signal {
    StrongBuy,
    Buy,
    Neutral,
    Sell,
    StrongSell,
}

struct SignalAggregator {
    indicators: Vec<Box<dyn Indicator>>,
    weights: Vec<f64>,
}

impl SignalAggregator {
    fn new() -> Self {
        SignalAggregator {
            indicators: Vec::new(),
            weights: Vec::new(),
        }
    }

    fn add_indicator(&mut self, indicator: Box<dyn Indicator>, weight: f64) {
        self.indicators.push(indicator);
        self.weights.push(weight);
    }

    fn update(&mut self, price: f64) -> Vec<Option<f64>> {
        self.indicators
            .iter_mut()
            .map(|ind| ind.calculate(price))
            .collect()
    }

    fn all_ready(&self) -> bool {
        self.indicators.iter().all(|ind| ind.is_ready())
    }
}
```

## Trend Following Strategy: SMA Crossover

A classic strategy combining two moving averages:

```rust
struct SMACrossover {
    fast_sma: SMA,
    slow_sma: SMA,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
}

impl SMACrossover {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        SMACrossover {
            fast_sma: SMA::new(fast_period),
            slow_sma: SMA::new(slow_period),
            prev_fast: None,
            prev_slow: None,
        }
    }

    fn update(&mut self, price: f64) -> Option<Signal> {
        let fast = self.fast_sma.calculate(price);
        let slow = self.slow_sma.calculate(price);

        let signal = match (fast, slow, self.prev_fast, self.prev_slow) {
            (Some(f), Some(s), Some(pf), Some(ps)) => {
                // Golden cross: fast crosses above slow
                if pf <= ps && f > s {
                    Some(Signal::Buy)
                }
                // Death cross: fast crosses below slow
                else if pf >= ps && f < s {
                    Some(Signal::Sell)
                } else {
                    Some(Signal::Neutral)
                }
            }
            _ => None,
        };

        self.prev_fast = fast;
        self.prev_slow = slow;
        signal
    }
}

fn main() {
    let mut crossover = SMACrossover::new(10, 20);

    let prices = [
        42000.0, 42100.0, 42300.0, 42500.0, 42400.0,
        42600.0, 42800.0, 43000.0, 43200.0, 43100.0,
        43300.0, 43500.0, 43400.0, 43600.0, 43800.0,
        44000.0, 43900.0, 43700.0, 43500.0, 43300.0,
        43100.0, 42900.0, 42700.0, 42500.0, 42300.0,
    ];

    println!("SMA Crossover Strategy:");
    println!("========================");

    for (i, &price) in prices.iter().enumerate() {
        if let Some(signal) = crossover.update(price) {
            println!("Day {}: Price ${:.2} -> {:?}", i + 1, price, signal);
        }
    }
}
```

## Multi-Indicator Confirmation System

Combining trend and momentum indicators for higher confidence:

```rust
struct MultiIndicatorStrategy {
    ema_fast: EMA,
    ema_slow: EMA,
    rsi: RSI,
    prev_ema_fast: Option<f64>,
    prev_ema_slow: Option<f64>,
}

impl MultiIndicatorStrategy {
    fn new() -> Self {
        MultiIndicatorStrategy {
            ema_fast: EMA::new(12),
            ema_slow: EMA::new(26),
            rsi: RSI::new(14),
            prev_ema_fast: None,
            prev_ema_slow: None,
        }
    }

    fn analyze(&mut self, price: f64) -> Option<(Signal, f64)> {
        let ema_fast = self.ema_fast.calculate(price);
        let ema_slow = self.ema_slow.calculate(price);
        let rsi = self.rsi.calculate(price);

        let result = match (ema_fast, ema_slow, rsi, self.prev_ema_fast, self.prev_ema_slow) {
            (Some(ef), Some(es), Some(r), Some(pef), Some(pes)) => {
                let trend_bullish = ef > es;
                let trend_bearish = ef < es;
                let ema_cross_up = pef <= pes && ef > es;
                let ema_cross_down = pef >= pes && ef < es;

                // Combine signals with confidence scoring
                let (signal, confidence) = if ema_cross_up && r < 70.0 {
                    // Bullish crossover with RSI not overbought
                    if r < 30.0 {
                        (Signal::StrongBuy, 0.9)
                    } else {
                        (Signal::Buy, 0.7)
                    }
                } else if ema_cross_down && r > 30.0 {
                    // Bearish crossover with RSI not oversold
                    if r > 70.0 {
                        (Signal::StrongSell, 0.9)
                    } else {
                        (Signal::Sell, 0.7)
                    }
                } else if trend_bullish && r < 30.0 {
                    // Oversold in uptrend - potential buy
                    (Signal::Buy, 0.6)
                } else if trend_bearish && r > 70.0 {
                    // Overbought in downtrend - potential sell
                    (Signal::Sell, 0.6)
                } else {
                    (Signal::Neutral, 0.5)
                };

                Some((signal, confidence))
            }
            _ => None,
        };

        self.prev_ema_fast = ema_fast;
        self.prev_ema_slow = ema_slow;
        result
    }
}

fn main() {
    let mut strategy = MultiIndicatorStrategy::new();

    let prices = [
        42000.0, 42050.0, 42100.0, 42080.0, 42150.0, 42200.0, 42180.0, 42250.0,
        42300.0, 42280.0, 42350.0, 42400.0, 42450.0, 42500.0, 42550.0, 42600.0,
        42650.0, 42700.0, 42800.0, 42900.0, 43000.0, 43100.0, 43200.0, 43300.0,
        43400.0, 43500.0, 43600.0, 43700.0, 43800.0, 43900.0, 44000.0, 44100.0,
    ];

    println!("Multi-Indicator Strategy Analysis:");
    println!("====================================");

    for (i, &price) in prices.iter().enumerate() {
        if let Some((signal, confidence)) = strategy.analyze(price) {
            if signal != Signal::Neutral {
                println!(
                    "Day {}: ${:.2} -> {:?} (Confidence: {:.0}%)",
                    i + 1,
                    price,
                    signal,
                    confidence * 100.0
                );
            }
        }
    }
}
```

## Weighted Signal Combination

Different indicators have different reliability in various market conditions:

```rust
struct WeightedSignalCombiner {
    signals: Vec<(String, Signal, f64)>, // (name, signal, weight)
}

impl WeightedSignalCombiner {
    fn new() -> Self {
        WeightedSignalCombiner { signals: Vec::new() }
    }

    fn add_signal(&mut self, name: &str, signal: Signal, weight: f64) {
        self.signals.push((name.to_string(), signal, weight));
    }

    fn clear(&mut self) {
        self.signals.clear();
    }

    fn get_combined_signal(&self) -> (Signal, f64) {
        if self.signals.is_empty() {
            return (Signal::Neutral, 0.0);
        }

        let mut score = 0.0;
        let mut total_weight = 0.0;

        for (_, signal, weight) in &self.signals {
            let signal_value = match signal {
                Signal::StrongBuy => 2.0,
                Signal::Buy => 1.0,
                Signal::Neutral => 0.0,
                Signal::Sell => -1.0,
                Signal::StrongSell => -2.0,
            };
            score += signal_value * weight;
            total_weight += weight;
        }

        let normalized_score = if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        };

        let final_signal = if normalized_score > 1.5 {
            Signal::StrongBuy
        } else if normalized_score > 0.5 {
            Signal::Buy
        } else if normalized_score < -1.5 {
            Signal::StrongSell
        } else if normalized_score < -0.5 {
            Signal::Sell
        } else {
            Signal::Neutral
        };

        let confidence = normalized_score.abs().min(2.0) / 2.0;
        (final_signal, confidence)
    }

    fn print_breakdown(&self) {
        println!("\nSignal Breakdown:");
        println!("-----------------");
        for (name, signal, weight) in &self.signals {
            println!("  {} ({:.1}): {:?}", name, weight, signal);
        }
        let (combined, confidence) = self.get_combined_signal();
        println!("-----------------");
        println!("  Combined: {:?} (Confidence: {:.0}%)", combined, confidence * 100.0);
    }
}

fn main() {
    let mut combiner = WeightedSignalCombiner::new();

    // Scenario: Trending market with oversold RSI
    combiner.add_signal("EMA Crossover", Signal::Buy, 1.0);
    combiner.add_signal("RSI", Signal::StrongBuy, 0.8);
    combiner.add_signal("MACD", Signal::Buy, 0.9);
    combiner.add_signal("Volume", Signal::Neutral, 0.5);

    combiner.print_breakdown();
}
```

## Practical Example: Complete Trading System

```rust
fn main() {
    // Build a complete multi-indicator trading system
    let mut ema_12 = EMA::new(12);
    let mut ema_26 = EMA::new(26);
    let mut rsi_14 = RSI::new(14);
    let mut sma_50 = SMA::new(50);

    // Simulated price data (50+ periods for SMA-50)
    let prices: Vec<f64> = (0..60)
        .map(|i| 42000.0 + (i as f64 * 50.0) + ((i as f64).sin() * 200.0))
        .collect();

    let mut prev_ema_12: Option<f64> = None;
    let mut prev_ema_26: Option<f64> = None;

    println!("Complete Trading System Analysis");
    println!("=================================\n");

    for (day, &price) in prices.iter().enumerate() {
        let e12 = ema_12.calculate(price);
        let e26 = ema_26.calculate(price);
        let rsi = rsi_14.calculate(price);
        let sma = sma_50.calculate(price);

        // Only analyze when all indicators are ready
        if let (Some(e12_val), Some(e26_val), Some(rsi_val), Some(sma_val)) =
            (e12, e26, rsi, sma)
        {
            // Determine trend from SMA-50
            let long_term_trend = if price > sma_val { "Bullish" } else { "Bearish" };

            // Check for EMA crossover
            let crossover = match (prev_ema_12, prev_ema_26) {
                (Some(pe12), Some(pe26)) => {
                    if pe12 <= pe26 && e12_val > e26_val {
                        Some("Golden Cross")
                    } else if pe12 >= pe26 && e12_val < e26_val {
                        Some("Death Cross")
                    } else {
                        None
                    }
                }
                _ => None,
            };

            // RSI condition
            let rsi_condition = if rsi_val < 30.0 {
                "Oversold"
            } else if rsi_val > 70.0 {
                "Overbought"
            } else {
                "Neutral"
            };

            // Generate trading decision
            let decision = match (long_term_trend, crossover, rsi_condition) {
                ("Bullish", Some("Golden Cross"), "Oversold") => "STRONG BUY",
                ("Bullish", Some("Golden Cross"), _) => "BUY",
                ("Bullish", None, "Oversold") => "BUY (RSI)",
                ("Bearish", Some("Death Cross"), "Overbought") => "STRONG SELL",
                ("Bearish", Some("Death Cross"), _) => "SELL",
                ("Bearish", None, "Overbought") => "SELL (RSI)",
                _ => "HOLD",
            };

            if decision != "HOLD" {
                println!("Day {:2}: ${:.2}", day + 1, price);
                println!("        Trend: {} | RSI: {:.1} ({}) | Decision: {}",
                    long_term_trend, rsi_val, rsi_condition, decision);
                if let Some(cross) = crossover {
                    println!("        Signal: {}", cross);
                }
                println!();
            }
        }

        prev_ema_12 = e12;
        prev_ema_26 = e26;
    }
}
```

## Filter Pattern: Indicator as Gatekeeper

Use one indicator to filter signals from another:

```rust
struct FilteredStrategy<T: Indicator, F: Indicator> {
    signal_indicator: T,
    filter_indicator: F,
    filter_condition: Box<dyn Fn(f64) -> bool>,
}

impl<T: Indicator, F: Indicator> FilteredStrategy<T, F> {
    fn new(signal: T, filter: F, condition: Box<dyn Fn(f64) -> bool>) -> Self {
        FilteredStrategy {
            signal_indicator: signal,
            filter_indicator: filter,
            filter_condition: condition,
        }
    }

    fn update(&mut self, price: f64) -> Option<f64> {
        let signal_value = self.signal_indicator.calculate(price);
        let filter_value = self.filter_indicator.calculate(price);

        match (signal_value, filter_value) {
            (Some(sv), Some(fv)) if (self.filter_condition)(fv) => Some(sv),
            _ => None,
        }
    }
}

fn main() {
    // Only take EMA signals when RSI is not extreme
    let ema = EMA::new(20);
    let rsi = RSI::new(14);

    let mut strategy = FilteredStrategy::new(
        ema,
        rsi,
        Box::new(|rsi_val| rsi_val > 30.0 && rsi_val < 70.0),
    );

    let prices = [42000.0, 42100.0, 42200.0, 42300.0, 42400.0];

    for price in prices {
        match strategy.update(price) {
            Some(ema_val) => println!("Filtered EMA: {:.2}", ema_val),
            None => println!("Signal filtered out (RSI extreme)"),
        }
    }
}
```

## What We Learned

| Concept | Description | Use Case |
|---------|-------------|----------|
| Indicator Trait | Unified interface for all indicators | Polymorphism & composition |
| Signal Aggregator | Collects multiple indicator outputs | Multi-indicator analysis |
| Crossover Strategy | Detects when indicators cross | Trend reversal signals |
| Weighted Combination | Assigns importance to each signal | Confidence scoring |
| Filter Pattern | One indicator gates another | Reducing false signals |
| Confidence Scoring | Quantifies signal strength | Position sizing |

## Homework

1. Implement a `MACD` indicator (Moving Average Convergence Divergence) that combines two EMAs and a signal line. Use it with RSI to create a momentum-based trading strategy.

2. Create a `VolatilityFilter` that uses Bollinger Bands to filter out signals during low-volatility periods. Only allow trades when price touches the bands.

3. Build a `ConsensusSystem` where at least 3 out of 5 indicators must agree before generating a signal. Implement indicators: SMA, EMA, RSI, and add MACD and Stochastic.

4. Design a `DynamicWeightAdjuster` that changes indicator weights based on recent performance. If an indicator has been generating winning signals, increase its weight.

## Navigation

[← Previous day](../256-indicator-trait-pattern/en.md) | [Next day →](../258-signals-buy-sell/en.md)
