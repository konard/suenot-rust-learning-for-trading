# Day 256: Pattern: Indicator as Trait

## Trading Analogy

Imagine you're building a trading dashboard that needs to display various technical indicators — SMA, EMA, RSI, MACD, Bollinger Bands. Each indicator takes price data and produces some output, but they all do it differently. Some calculate a single value, others produce multiple lines, and some generate signals.

In a trading terminal, you want a unified way to:
- **Update** any indicator with new price data
- **Get the current value** from any indicator
- **Reset** any indicator to start fresh

This is exactly what Rust traits provide: a **contract** that different indicator types must follow. Just like all exchanges have a standard API for placing orders (even though they implement it differently internally), all indicators can share a common interface through traits.

## What is a Trait?

A trait defines shared behavior that types can implement. Think of it as a "capability contract" — any type implementing a trait guarantees it can perform certain operations.

```rust
// Define what ANY indicator must be able to do
trait Indicator {
    fn update(&mut self, price: f64);
    fn value(&self) -> Option<f64>;
    fn reset(&mut self);
}
```

This says: "Any type that claims to be an Indicator MUST provide these three methods."

## Basic Indicator Trait Implementation

Let's implement our `Indicator` trait for Simple Moving Average (SMA):

```rust
trait Indicator {
    /// Feed a new price to the indicator
    fn update(&mut self, price: f64);

    /// Get the current indicator value (None if not enough data)
    fn value(&self) -> Option<f64>;

    /// Reset the indicator to initial state
    fn reset(&mut self);

    /// Get the name of the indicator
    fn name(&self) -> &str;
}

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
    fn update(&mut self, price: f64) {
        self.prices.push(price);
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }
    }

    fn value(&self) -> Option<f64> {
        if self.prices.len() < self.period {
            return None;
        }
        let sum: f64 = self.prices.iter().sum();
        Some(sum / self.period as f64)
    }

    fn reset(&mut self) {
        self.prices.clear();
    }

    fn name(&self) -> &str {
        "SMA"
    }
}

fn main() {
    let mut sma = SMA::new(3);

    // Feed prices
    for price in [100.0, 102.0, 104.0, 103.0, 105.0] {
        sma.update(price);
        match sma.value() {
            Some(v) => println!("{}: {:.2}", sma.name(), v),
            None => println!("{}: Calculating...", sma.name()),
        }
    }
}
```

Output:
```
SMA: Calculating...
SMA: Calculating...
SMA: 102.00
SMA: 103.00
SMA: 104.00
```

## Multiple Indicators with Same Trait

Now let's add EMA (Exponential Moving Average) using the same trait:

```rust
struct EMA {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    count: usize,
    initial_sum: f64,
}

impl EMA {
    fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        EMA {
            period,
            multiplier,
            current_ema: None,
            count: 0,
            initial_sum: 0.0,
        }
    }
}

impl Indicator for EMA {
    fn update(&mut self, price: f64) {
        self.count += 1;

        match self.current_ema {
            None => {
                // Accumulate prices for initial SMA
                self.initial_sum += price;
                if self.count >= self.period {
                    // Use SMA as first EMA value
                    self.current_ema = Some(self.initial_sum / self.period as f64);
                }
            }
            Some(prev_ema) => {
                // EMA formula: EMA = (Price - Previous EMA) * Multiplier + Previous EMA
                let new_ema = (price - prev_ema) * self.multiplier + prev_ema;
                self.current_ema = Some(new_ema);
            }
        }
    }

    fn value(&self) -> Option<f64> {
        self.current_ema
    }

    fn reset(&mut self) {
        self.current_ema = None;
        self.count = 0;
        self.initial_sum = 0.0;
    }

    fn name(&self) -> &str {
        "EMA"
    }
}

fn main() {
    let mut sma = SMA::new(3);
    let mut ema = EMA::new(3);

    let prices = [100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 106.0];

    println!("{:<6} {:>10} {:>10}", "Price", "SMA(3)", "EMA(3)");
    println!("{}", "-".repeat(28));

    for price in prices {
        sma.update(price);
        ema.update(price);

        let sma_val = sma.value().map(|v| format!("{:.2}", v)).unwrap_or("-".to_string());
        let ema_val = ema.value().map(|v| format!("{:.2}", v)).unwrap_or("-".to_string());

        println!("{:<6.1} {:>10} {:>10}", price, sma_val, ema_val);
    }
}
```

## The Power of Trait Objects

With traits, we can store different indicator types in the same collection using **trait objects**:

```rust
fn main() {
    // Store different indicators in one vector using trait objects
    let mut indicators: Vec<Box<dyn Indicator>> = vec![
        Box::new(SMA::new(3)),
        Box::new(SMA::new(5)),
        Box::new(EMA::new(3)),
        Box::new(EMA::new(5)),
    ];

    let prices = [100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 106.0, 108.0];

    for price in prices {
        println!("\nPrice: {:.1}", price);
        for indicator in &mut indicators {
            indicator.update(price);
            if let Some(value) = indicator.value() {
                println!("  {}: {:.2}", indicator.name(), value);
            }
        }
    }
}
```

This is incredibly powerful — we can add new indicator types without changing any of the processing code!

## RSI Implementation

Let's implement a more complex indicator — RSI (Relative Strength Index):

```rust
struct RSI {
    period: usize,
    gains: Vec<f64>,
    losses: Vec<f64>,
    prev_price: Option<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
}

impl RSI {
    fn new(period: usize) -> Self {
        RSI {
            period,
            gains: Vec::new(),
            losses: Vec::new(),
            prev_price: None,
            avg_gain: None,
            avg_loss: None,
        }
    }
}

impl Indicator for RSI {
    fn update(&mut self, price: f64) {
        if let Some(prev) = self.prev_price {
            let change = price - prev;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            match (&self.avg_gain, &self.avg_loss) {
                (None, None) => {
                    // Initial period: collect gains and losses
                    self.gains.push(gain);
                    self.losses.push(loss);

                    if self.gains.len() >= self.period {
                        self.avg_gain = Some(self.gains.iter().sum::<f64>() / self.period as f64);
                        self.avg_loss = Some(self.losses.iter().sum::<f64>() / self.period as f64);
                    }
                }
                (Some(prev_avg_gain), Some(prev_avg_loss)) => {
                    // Smoothed averages
                    let new_avg_gain = (prev_avg_gain * (self.period - 1) as f64 + gain) / self.period as f64;
                    let new_avg_loss = (prev_avg_loss * (self.period - 1) as f64 + loss) / self.period as f64;
                    self.avg_gain = Some(new_avg_gain);
                    self.avg_loss = Some(new_avg_loss);
                }
                _ => unreachable!(),
            }
        }
        self.prev_price = Some(price);
    }

    fn value(&self) -> Option<f64> {
        match (&self.avg_gain, &self.avg_loss) {
            (Some(avg_gain), Some(avg_loss)) => {
                if *avg_loss == 0.0 {
                    Some(100.0) // No losses means RSI is 100
                } else {
                    let rs = avg_gain / avg_loss;
                    Some(100.0 - (100.0 / (1.0 + rs)))
                }
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.gains.clear();
        self.losses.clear();
        self.prev_price = None;
        self.avg_gain = None;
        self.avg_loss = None;
    }

    fn name(&self) -> &str {
        "RSI"
    }
}
```

## Default Trait Methods

Traits can provide default implementations that can be overridden:

```rust
trait Indicator {
    fn update(&mut self, price: f64);
    fn value(&self) -> Option<f64>;
    fn reset(&mut self);
    fn name(&self) -> &str;

    /// Default implementation: check if indicator is ready
    fn is_ready(&self) -> bool {
        self.value().is_some()
    }

    /// Default implementation: get formatted value
    fn formatted_value(&self) -> String {
        match self.value() {
            Some(v) => format!("{}: {:.2}", self.name(), v),
            None => format!("{}: N/A", self.name()),
        }
    }

    /// Default implementation: update with multiple prices
    fn update_batch(&mut self, prices: &[f64]) {
        for &price in prices {
            self.update(price);
        }
    }
}
```

Now all indicators automatically get these methods:

```rust
fn main() {
    let mut rsi = RSI::new(14);

    // Use default update_batch method
    rsi.update_batch(&[100.0, 101.0, 99.0, 102.0, 103.0, 101.0, 104.0,
                       105.0, 103.0, 106.0, 107.0, 105.0, 108.0, 109.0, 110.0]);

    // Use default is_ready method
    if rsi.is_ready() {
        println!("{}", rsi.formatted_value());
    }
}
```

## Associated Types in Traits

For indicators that return different types of output, use associated types:

```rust
trait AdvancedIndicator {
    type Output;

    fn update(&mut self, price: f64);
    fn value(&self) -> Option<Self::Output>;
    fn name(&self) -> &str;
}

// Bollinger Bands return three values
#[derive(Debug, Clone)]
struct BollingerBandsOutput {
    upper: f64,
    middle: f64,
    lower: f64,
}

struct BollingerBands {
    period: usize,
    std_dev_multiplier: f64,
    prices: Vec<f64>,
}

impl BollingerBands {
    fn new(period: usize, std_dev_multiplier: f64) -> Self {
        BollingerBands {
            period,
            std_dev_multiplier,
            prices: Vec::with_capacity(period),
        }
    }
}

impl AdvancedIndicator for BollingerBands {
    type Output = BollingerBandsOutput;

    fn update(&mut self, price: f64) {
        self.prices.push(price);
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }
    }

    fn value(&self) -> Option<Self::Output> {
        if self.prices.len() < self.period {
            return None;
        }

        let sum: f64 = self.prices.iter().sum();
        let mean = sum / self.period as f64;

        let variance: f64 = self.prices.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>() / self.period as f64;
        let std_dev = variance.sqrt();

        Some(BollingerBandsOutput {
            upper: mean + self.std_dev_multiplier * std_dev,
            middle: mean,
            lower: mean - self.std_dev_multiplier * std_dev,
        })
    }

    fn name(&self) -> &str {
        "Bollinger Bands"
    }
}

fn main() {
    let mut bb = BollingerBands::new(20, 2.0);

    let prices: Vec<f64> = (0..25).map(|i| 100.0 + (i as f64 * 0.5) + (i % 3) as f64).collect();

    for price in &prices {
        bb.update(*price);
    }

    if let Some(bands) = bb.value() {
        println!("{}", bb.name());
        println!("  Upper:  {:.2}", bands.upper);
        println!("  Middle: {:.2}", bands.middle);
        println!("  Lower:  {:.2}", bands.lower);
    }
}
```

## Building an Indicator Engine

Let's create a complete indicator engine using traits:

```rust
use std::collections::HashMap;

trait Indicator {
    fn update(&mut self, price: f64);
    fn value(&self) -> Option<f64>;
    fn reset(&mut self);
    fn name(&self) -> &str;
}

struct IndicatorEngine {
    indicators: HashMap<String, Box<dyn Indicator>>,
    last_price: Option<f64>,
}

impl IndicatorEngine {
    fn new() -> Self {
        IndicatorEngine {
            indicators: HashMap::new(),
            last_price: None,
        }
    }

    fn add_indicator(&mut self, id: &str, indicator: Box<dyn Indicator>) {
        self.indicators.insert(id.to_string(), indicator);
    }

    fn remove_indicator(&mut self, id: &str) -> Option<Box<dyn Indicator>> {
        self.indicators.remove(id)
    }

    fn update(&mut self, price: f64) {
        self.last_price = Some(price);
        for indicator in self.indicators.values_mut() {
            indicator.update(price);
        }
    }

    fn get_value(&self, id: &str) -> Option<f64> {
        self.indicators.get(id)?.value()
    }

    fn get_all_values(&self) -> HashMap<String, Option<f64>> {
        self.indicators
            .iter()
            .map(|(id, ind)| (id.clone(), ind.value()))
            .collect()
    }

    fn reset_all(&mut self) {
        for indicator in self.indicators.values_mut() {
            indicator.reset();
        }
        self.last_price = None;
    }

    fn print_status(&self) {
        println!("\n=== Indicator Status ===");
        if let Some(price) = self.last_price {
            println!("Last Price: {:.2}", price);
        }
        for (id, indicator) in &self.indicators {
            let value_str = match indicator.value() {
                Some(v) => format!("{:.2}", v),
                None => "N/A".to_string(),
            };
            println!("{} ({}): {}", id, indicator.name(), value_str);
        }
    }
}

fn main() {
    let mut engine = IndicatorEngine::new();

    // Add various indicators
    engine.add_indicator("fast_sma", Box::new(SMA::new(5)));
    engine.add_indicator("slow_sma", Box::new(SMA::new(20)));
    engine.add_indicator("fast_ema", Box::new(EMA::new(5)));
    engine.add_indicator("slow_ema", Box::new(EMA::new(20)));
    engine.add_indicator("rsi_14", Box::new(RSI::new(14)));

    // Simulate price data
    let prices = [
        100.0, 101.5, 103.0, 102.5, 104.0, 105.5, 104.5, 106.0,
        107.5, 108.0, 107.0, 109.0, 110.5, 109.5, 111.0, 112.0,
        111.5, 113.0, 114.5, 113.5, 115.0, 116.5, 115.5, 117.0,
        118.0, 117.5, 119.0, 120.0, 119.0, 121.0,
    ];

    for &price in &prices {
        engine.update(price);
    }

    engine.print_status();

    // Check for SMA crossover signal
    let fast = engine.get_value("fast_sma");
    let slow = engine.get_value("slow_sma");

    if let (Some(f), Some(s)) = (fast, slow) {
        if f > s {
            println!("\nSignal: BULLISH (Fast SMA above Slow SMA)");
        } else {
            println!("\nSignal: BEARISH (Fast SMA below Slow SMA)");
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Traits | Shared behavior definition that types can implement |
| `impl Trait for Type` | Implementing a trait for a specific type |
| Trait Objects (`dyn Trait`) | Runtime polymorphism for different types |
| `Box<dyn Trait>` | Heap-allocated trait objects for collections |
| Default Methods | Trait methods with default implementations |
| Associated Types | Type placeholders within traits |

## Exercises

1. **MACD Indicator**: Implement the MACD indicator using the `Indicator` trait. MACD consists of:
   - MACD Line = 12-period EMA - 26-period EMA
   - Signal Line = 9-period EMA of MACD Line
   - Histogram = MACD Line - Signal Line

   Tip: You can compose existing EMA indicators!

2. **Stochastic Oscillator**: Implement the Stochastic indicator:
   - %K = (Current Close - Lowest Low) / (Highest High - Lowest Low) * 100
   - %D = 3-period SMA of %K

3. **ATR (Average True Range)**: Implement ATR which calculates:
   - True Range = max(High - Low, |High - Previous Close|, |Low - Previous Close|)
   - ATR = Moving Average of True Range

4. **Signal Generator Trait**: Create a `SignalGenerator` trait that works with indicators:
   ```rust
   trait SignalGenerator {
       fn generate_signal(&self) -> Signal;
   }

   enum Signal {
       Buy,
       Sell,
       Hold,
   }
   ```

## Homework

1. **Indicator Factory**: Create an `IndicatorFactory` that can create indicators by name:
   ```rust
   let sma = factory.create("SMA", &[("period", 20)]);
   let ema = factory.create("EMA", &[("period", 12)]);
   ```

2. **Composite Indicator**: Create a `CompositeIndicator` that combines multiple indicators with custom logic:
   ```rust
   let composite = CompositeIndicator::new()
       .add(SMA::new(10))
       .add(EMA::new(10))
       .combine(|values| values.iter().sum::<f64>() / values.len() as f64);
   ```

3. **Serializable Indicators**: Add serde support to save/load indicator state:
   ```rust
   let json = indicator.to_json();
   let restored: Box<dyn Indicator> = Indicator::from_json(&json);
   ```

4. **Indicator Backtester**: Create a simple backtester that tests indicator-based strategies:
   ```rust
   let strategy = CrossoverStrategy::new(SMA::new(10), SMA::new(50));
   let results = backtest(strategy, &historical_prices);
   println!("Total Return: {:.2}%", results.total_return);
   ```

## Navigation

[← Previous day](../255-vwap-volume-weighted-price/en.md) | [Next day →](../257-combining-indicators/en.md)
