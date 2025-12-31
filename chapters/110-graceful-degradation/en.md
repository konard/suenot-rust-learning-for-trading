# Day 110: Graceful Degradation: Working Without Some Data

## Trading Analogy

Imagine you're trading on an exchange using several indicators: RSI, MACD, moving averages. Suddenly, your data provider stops delivering RSI. What do you do? Stop trading completely? An experienced trader **continues working** with the available data, perhaps reducing position size or using only reliable signals.

This is **Graceful Degradation** — the ability of a system to continue operating with limited functionality when some data or services are unavailable.

## The Concept of Graceful Degradation

In Rust, we use `Option` to represent data that may be absent. Graceful Degradation is a strategy for handling such situations:

```rust
fn main() {
    // Data may be partially unavailable
    let price = Some(42000.0);
    let rsi: Option<f64> = None;  // RSI unavailable
    let macd = Some(150.0);

    // Graceful degradation: work with what we have
    analyze_market(price, rsi, macd);
}

fn analyze_market(price: Option<f64>, rsi: Option<f64>, macd: Option<f64>) {
    let price = match price {
        Some(p) => p,
        None => {
            println!("Critical: no price data, analysis impossible");
            return;
        }
    };

    println!("Market analysis at price ${:.2}", price);

    // RSI — not critical, use if available
    match rsi {
        Some(r) => println!("  RSI: {:.1}", r),
        None => println!("  RSI: data unavailable, skipping"),
    }

    // MACD — not critical
    match macd {
        Some(m) => println!("  MACD: {:.2}", m),
        None => println!("  MACD: data unavailable, skipping"),
    }
}
```

## unwrap_or — Default Value

The simplest way to provide graceful degradation:

```rust
fn main() {
    let prices: Vec<Option<f64>> = vec![
        Some(42000.0),
        None,           // Missing price
        Some(42100.0),
        None,           // Missing price
        Some(42050.0),
    ];

    // Use last known price as fallback
    let mut last_known_price = 0.0;

    for (i, price_opt) in prices.iter().enumerate() {
        let price = price_opt.unwrap_or(last_known_price);

        if price_opt.is_some() {
            last_known_price = price;
        }

        println!("Candle {}: ${:.2} {}",
            i + 1,
            price,
            if price_opt.is_none() { "(interpolated)" } else { "" }
        );
    }
}
```

## unwrap_or_default — Zero Values by Default

For types implementing `Default`:

```rust
fn main() {
    let trade_volume: Option<f64> = None;
    let order_count: Option<u32> = None;
    let is_active: Option<bool> = None;

    // f64::default() = 0.0, u32::default() = 0, bool::default() = false
    println!("Volume: {}", trade_volume.unwrap_or_default());
    println!("Order count: {}", order_count.unwrap_or_default());
    println!("Active: {}", is_active.unwrap_or_default());
}
```

## unwrap_or_else — Lazy Fallback Computation

When fallback requires computation:

```rust
fn main() {
    let current_price: Option<f64> = None;
    let historical_prices = vec![41800.0, 41900.0, 42000.0];

    // If current price unavailable, compute historical average
    let price = current_price.unwrap_or_else(|| {
        println!("Current price unavailable, computing average...");
        let sum: f64 = historical_prices.iter().sum();
        sum / historical_prices.len() as f64
    });

    println!("Price used: ${:.2}", price);
}
```

## Practical Example: Trading Signal with Partial Data

```rust
fn main() {
    // Scenario 1: all data available
    let signal1 = generate_signal(
        Some(42000.0),  // price
        Some(45.0),     // rsi
        Some(100.0),    // macd
        Some(41800.0),  // sma
    );
    println!("Signal 1: {:?}\n", signal1);

    // Scenario 2: RSI unavailable
    let signal2 = generate_signal(
        Some(42000.0),
        None,           // RSI unavailable
        Some(100.0),
        Some(41800.0),
    );
    println!("Signal 2: {:?}\n", signal2);

    // Scenario 3: only price and SMA
    let signal3 = generate_signal(
        Some(42000.0),
        None,
        None,
        Some(41800.0),
    );
    println!("Signal 3: {:?}\n", signal3);
}

#[derive(Debug)]
struct TradingSignal {
    action: String,
    confidence: f64,
    available_indicators: u8,
    warnings: Vec<String>,
}

fn generate_signal(
    price: Option<f64>,
    rsi: Option<f64>,
    macd: Option<f64>,
    sma: Option<f64>,
) -> Option<TradingSignal> {
    // Price is critical — cannot work without it
    let price = price?;

    let mut bullish_signals = 0;
    let mut bearish_signals = 0;
    let mut available = 0;
    let mut warnings = Vec::new();

    // RSI analysis (if available)
    if let Some(rsi_value) = rsi {
        available += 1;
        if rsi_value < 30.0 {
            bullish_signals += 1;  // Oversold
        } else if rsi_value > 70.0 {
            bearish_signals += 1;  // Overbought
        }
    } else {
        warnings.push("RSI unavailable".to_string());
    }

    // MACD analysis (if available)
    if let Some(macd_value) = macd {
        available += 1;
        if macd_value > 0.0 {
            bullish_signals += 1;
        } else {
            bearish_signals += 1;
        }
    } else {
        warnings.push("MACD unavailable".to_string());
    }

    // SMA analysis (if available)
    if let Some(sma_value) = sma {
        available += 1;
        if price > sma_value {
            bullish_signals += 1;
        } else {
            bearish_signals += 1;
        }
    } else {
        warnings.push("SMA unavailable".to_string());
    }

    // Determine action
    let action = if available == 0 {
        "HOLD".to_string()  // No data for analysis
    } else if bullish_signals > bearish_signals {
        "BUY".to_string()
    } else if bearish_signals > bullish_signals {
        "SELL".to_string()
    } else {
        "HOLD".to_string()
    };

    // Confidence depends on number of available indicators
    let max_indicators = 3.0;
    let confidence = (available as f64 / max_indicators) * 100.0;

    Some(TradingSignal {
        action,
        confidence,
        available_indicators: available,
        warnings,
    })
}
```

## Option Chains with map and and_then

```rust
fn main() {
    let raw_price: Option<&str> = Some("42000.50");

    // Chain of transformations with graceful degradation
    let processed = raw_price
        .map(|s| s.trim())                           // Remove whitespace
        .and_then(|s| s.parse::<f64>().ok())         // Parse (may fail)
        .map(|p| p * 1.001)                          // Add fee
        .unwrap_or(0.0);                             // Fallback

    println!("Processed price: ${:.2}", processed);

    // Example with invalid data
    let invalid_price: Option<&str> = Some("not_a_price");
    let result = invalid_price
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or_else(|| {
            println!("Failed to parse price, using 0");
            0.0
        });

    println!("Result: ${:.2}", result);
}
```

## Working with Partial Portfolio Data

```rust
fn main() {
    let portfolio = Portfolio {
        btc_balance: Some(1.5),
        eth_balance: None,  // Data unavailable
        usdt_balance: Some(10000.0),
    };

    let prices = Prices {
        btc: Some(42000.0),
        eth: Some(2200.0),  // Price exists, but no balance
        usdt: Some(1.0),
    };

    let report = calculate_portfolio_value(&portfolio, &prices);
    println!("{}", report);
}

struct Portfolio {
    btc_balance: Option<f64>,
    eth_balance: Option<f64>,
    usdt_balance: Option<f64>,
}

struct Prices {
    btc: Option<f64>,
    eth: Option<f64>,
    usdt: Option<f64>,
}

fn calculate_portfolio_value(portfolio: &Portfolio, prices: &Prices) -> String {
    let mut total = 0.0;
    let mut report = String::from("=== Portfolio Report ===\n");
    let mut warnings = Vec::new();

    // BTC
    match (portfolio.btc_balance, prices.btc) {
        (Some(bal), Some(price)) => {
            let value = bal * price;
            total += value;
            report.push_str(&format!("BTC: {:.4} x ${:.2} = ${:.2}\n", bal, price, value));
        }
        (None, _) => warnings.push("BTC balance unavailable"),
        (_, None) => warnings.push("BTC price unavailable"),
    }

    // ETH
    match (portfolio.eth_balance, prices.eth) {
        (Some(bal), Some(price)) => {
            let value = bal * price;
            total += value;
            report.push_str(&format!("ETH: {:.4} x ${:.2} = ${:.2}\n", bal, price, value));
        }
        (None, _) => warnings.push("ETH balance unavailable"),
        (_, None) => warnings.push("ETH price unavailable"),
    }

    // USDT
    match (portfolio.usdt_balance, prices.usdt) {
        (Some(bal), Some(price)) => {
            let value = bal * price;
            total += value;
            report.push_str(&format!("USDT: {:.2} x ${:.2} = ${:.2}\n", bal, price, value));
        }
        (None, _) => warnings.push("USDT balance unavailable"),
        (_, None) => warnings.push("USDT price unavailable"),
    }

    report.push_str(&format!("\nTOTAL: ${:.2}\n", total));

    if !warnings.is_empty() {
        report.push_str("\nWarnings:\n");
        for w in warnings {
            report.push_str(&format!("  * {}\n", w));
        }
        report.push_str("\n(Total may be incomplete)");
    }

    report
}
```

## Fallback Strategy for Historical Data

```rust
fn main() {
    let candles = vec![
        Candle { open: Some(42000.0), high: Some(42500.0), low: Some(41800.0), close: Some(42200.0) },
        Candle { open: Some(42200.0), high: None, low: Some(42000.0), close: Some(42300.0) },  // No high
        Candle { open: None, high: Some(42600.0), low: Some(42200.0), close: Some(42400.0) },  // No open
        Candle { open: Some(42400.0), high: Some(42700.0), low: None, close: None },  // No low and close
    ];

    for (i, candle) in candles.iter().enumerate() {
        let normalized = normalize_candle(candle, i, &candles);
        println!("Candle {}: O={:.0} H={:.0} L={:.0} C={:.0}",
            i + 1, normalized.0, normalized.1, normalized.2, normalized.3);
    }
}

struct Candle {
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: Option<f64>,
}

fn normalize_candle(candle: &Candle, index: usize, all_candles: &[Candle]) -> (f64, f64, f64, f64) {
    // Strategy: use previous candle for fallback
    let prev_close = if index > 0 {
        all_candles[index - 1].close.unwrap_or(0.0)
    } else {
        0.0
    };

    let open = candle.open.unwrap_or(prev_close);
    let close = candle.close.unwrap_or(open);  // If no close, use open

    // High must be >= max(open, close)
    let min_high = open.max(close);
    let high = candle.high.unwrap_or(min_high);

    // Low must be <= min(open, close)
    let max_low = open.min(close);
    let low = candle.low.unwrap_or(max_low);

    (open, high, low, close)
}
```

## What We Learned

| Method | Usage | Example |
|--------|-------|---------|
| `unwrap_or(default)` | Fixed value | `price.unwrap_or(0.0)` |
| `unwrap_or_default()` | Default value | `count.unwrap_or_default()` |
| `unwrap_or_else(fn)` | Lazy computation | `price.unwrap_or_else(\|\| calc())` |
| `map(fn)` | Transform Some | `opt.map(\|x\| x * 2)` |
| `and_then(fn)` | Chain Options | `opt.and_then(\|x\| parse(x))` |
| `?` operator | Early return None | `let x = opt?;` |

## Homework

1. **Order Book Analyzer with Gaps**: Create a function that analyzes an order book where some price levels may be unavailable. The function should calculate approximate spread and depth using available data.

2. **Multi-Exchange Aggregator**: Write a system that receives prices from 3 exchanges (all `Option<f64>`). If all exchanges are available — output the average price. If 2 are available — average of those two. If only 1 — its price. If none — last known price.

3. **Trading Strategy with Degradation**: Implement a strategy that uses 5 indicators. When an indicator is missing, the strategy should reduce position size proportionally to the number of unavailable indicators.

4. **Time Series Recovery**: Write a function that takes `Vec<Option<f64>>` (prices with gaps) and returns `Vec<f64>`, where gaps are filled with linear interpolation between neighboring known values.

## Navigation

[← Previous day](../109-circuit-breaker-cascade-failure/en.md) | [Next day →](../111-input-validation/en.md)
