# Day 18: else and else if — Three Market Scenarios

## Trading Analogy

Markets are rarely "just bullish" or "just bearish". Usually there are **three scenarios**: uptrend, downtrend, and sideways. Your trading strategy must account for all three:
- Price above SMA → **buy**
- Price below SMA → **sell**
- Price around SMA → **wait**

In Rust, we use `if`, `else if`, and `else` for this.

## Basic else Syntax

```rust
fn main() {
    let price = 42500.0;
    let entry_price = 42000.0;

    if price > entry_price {
        println!("Position is in profit!");
    } else {
        println!("Position is at loss or breakeven");
    }
}
```

**Important:** `else` is "everything else". It executes when the `if` condition is false.

## else if — Multiple Conditions

```rust
fn main() {
    let price_change = 2.5; // percentage change

    if price_change > 5.0 {
        println!("Strong rally! Consider taking profits");
    } else if price_change > 0.0 {
        println!("Moderate gain. Hold position");
    } else if price_change > -5.0 {
        println!("Moderate decline. Analyze situation");
    } else {
        println!("Sharp decline! Check stop-loss");
    }
}
```

## Three Market Scenarios: Buy / Sell / Hold

```rust
fn main() {
    let current_price = 42500.0;
    let sma_20 = 42000.0;
    let threshold = 100.0; // threshold for "around"

    if current_price > sma_20 + threshold {
        println!("📈 SIGNAL: BUY — price above SMA");
    } else if current_price < sma_20 - threshold {
        println!("📉 SIGNAL: SELL — price below SMA");
    } else {
        println!("⏸️ SIGNAL: HOLD — price around SMA, wait");
    }
}
```

## Position Analysis: Profit, Loss, Breakeven

```rust
fn main() {
    let entry_price = 42000.0;
    let current_price = 42000.0;
    let quantity = 0.5;

    let pnl = (current_price - entry_price) * quantity;

    if pnl > 0.0 {
        println!("✅ Profit: ${:.2}", pnl);
    } else if pnl < 0.0 {
        println!("❌ Loss: ${:.2}", pnl.abs());
    } else {
        println!("➖ Breakeven");
    }
}
```

## Volatility Classification

```rust
fn main() {
    let daily_range_percent = 3.2;

    let volatility = if daily_range_percent < 1.0 {
        "low"
    } else if daily_range_percent < 3.0 {
        "medium"
    } else if daily_range_percent < 5.0 {
        "high"
    } else {
        "extreme"
    };

    println!("Volatility: {}", volatility);
}
```

**Note:** `if`/`else` is an expression! You can assign its result to a variable.

## Order Validation

```rust
fn main() {
    let balance = 10000.0;
    let order_price = 42000.0;
    let order_quantity = 0.3;
    let order_value = order_price * order_quantity;

    if order_price <= 0.0 {
        println!("❌ Error: price must be positive");
    } else if order_quantity <= 0.0 {
        println!("❌ Error: quantity must be positive");
    } else if order_value > balance {
        println!("❌ Error: insufficient funds (need ${:.2}, have ${:.2})",
                 order_value, balance);
    } else {
        println!("✅ Order valid: {} BTC at ${:.2}", order_quantity, order_price);
    }
}
```

## RSI Signals: Overbought / Oversold / Neutral

```rust
fn main() {
    let rsi = 72.5;

    let signal = if rsi >= 70.0 {
        ("OVERBOUGHT", "Possible correction ahead")
    } else if rsi <= 30.0 {
        ("OVERSOLD", "Possible bounce ahead")
    } else if rsi >= 50.0 {
        ("BULLISH", "Bullish trend")
    } else {
        ("BEARISH", "Bearish trend")
    };

    println!("RSI: {:.1} — {} ({})", rsi, signal.0, signal.1);
}
```

## Position Management Based on PnL

```rust
fn main() {
    let entry = 42000.0;
    let current = 43500.0;
    let stop_loss = 41000.0;
    let take_profit = 45000.0;

    if current <= stop_loss {
        println!("🛑 STOP LOSS triggered! Closing position at loss");
    } else if current >= take_profit {
        println!("🎯 TAKE PROFIT reached! Closing position at profit");
    } else if current > entry {
        let profit_percent = ((current - entry) / entry) * 100.0;
        println!("📈 In profit: +{:.2}%. Position open", profit_percent);
    } else if current < entry {
        let loss_percent = ((entry - current) / entry) * 100.0;
        println!("📉 In loss: -{:.2}%. Position open", loss_percent);
    } else {
        println!("➖ At entry point. Position open");
    }
}
```

## Trend Detection Using Multiple SMAs

```rust
fn main() {
    let price = 43000.0;
    let sma_20 = 42500.0;
    let sma_50 = 42000.0;
    let sma_200 = 40000.0;

    if price > sma_20 && sma_20 > sma_50 && sma_50 > sma_200 {
        println!("🚀 Strong uptrend (all SMAs aligned)");
    } else if price > sma_200 {
        println!("📈 General trend is up");
    } else if price < sma_20 && sma_20 < sma_50 && sma_50 < sma_200 {
        println!("💥 Strong downtrend (all SMAs aligned down)");
    } else if price < sma_200 {
        println!("📉 General trend is down");
    } else {
        println!("↔️ Sideways trend or uncertainty");
    }
}
```

## Trading Signal Function

```rust
fn main() {
    let signals = [
        (42500.0, 42000.0, 45.0),  // price, sma, rsi
        (41500.0, 42000.0, 28.0),
        (42000.0, 42000.0, 50.0),
        (43000.0, 42000.0, 75.0),
    ];

    for (price, sma, rsi) in signals {
        let signal = get_trading_signal(price, sma, rsi);
        println!("Price: {}, SMA: {}, RSI: {:.0} → {}", price, sma, rsi, signal);
    }
}

fn get_trading_signal(price: f64, sma: f64, rsi: f64) -> &'static str {
    if rsi >= 70.0 {
        "SELL (RSI overbought)"
    } else if rsi <= 30.0 {
        "BUY (RSI oversold)"
    } else if price > sma * 1.02 {
        "BUY (price 2%+ above SMA)"
    } else if price < sma * 0.98 {
        "SELL (price 2%+ below SMA)"
    } else {
        "HOLD (no clear signal)"
    }
}
```

## Trade Risk Assessment

```rust
fn main() {
    let account_balance = 50000.0;
    let position_size = 15000.0;
    let risk_percent = (position_size / account_balance) * 100.0;

    let risk_level = if risk_percent > 20.0 {
        "CRITICAL"
    } else if risk_percent > 10.0 {
        "HIGH"
    } else if risk_percent > 5.0 {
        "MEDIUM"
    } else if risk_percent > 2.0 {
        "LOW"
    } else {
        "MINIMAL"
    };

    println!("Position size: ${:.0} ({:.1}% of balance)", position_size, risk_percent);
    println!("Risk level: {}", risk_level);

    if risk_percent > 10.0 {
        println!("⚠️ Recommendation: reduce position size!");
    }
}
```

## Nested Conditions

```rust
fn main() {
    let is_market_open = true;
    let has_signal = true;
    let has_balance = true;
    let risk_approved = true;

    if is_market_open {
        if has_signal {
            if has_balance {
                if risk_approved {
                    println!("✅ All conditions met — opening trade!");
                } else {
                    println!("❌ Risk exceeds limit");
                }
            } else {
                println!("❌ Insufficient funds");
            }
        } else {
            println!("⏸️ No trading signal");
        }
    } else {
        println!("🔒 Market is closed");
    }
}
```

**Avoid deep nesting!** Use early returns instead:

```rust
fn can_open_trade(market_open: bool, signal: bool, balance: bool, risk_ok: bool) -> bool {
    if !market_open {
        println!("🔒 Market closed");
        return false;
    }
    if !signal {
        println!("⏸️ No signal");
        return false;
    }
    if !balance {
        println!("❌ No balance");
        return false;
    }
    if !risk_ok {
        println!("❌ Risk exceeded");
        return false;
    }

    println!("✅ Can open trade");
    true
}

fn main() {
    can_open_trade(true, true, true, true);
    can_open_trade(true, true, false, true);
}
```

## What We Learned

| Construct | When to Use |
|-----------|-------------|
| `if` | Single condition |
| `if-else` | Two options (yes/no) |
| `if-else if-else` | Three or more options |
| Nested `if` | Dependent conditions (better avoid) |
| `if` as expression | When you need to return a value |

## Exercises

### Exercise 1: Trade Classification
Write code that classifies a trade by result:
- Loss more than 10% → "Disaster"
- Loss 5-10% → "Big loss"
- Loss up to 5% → "Small loss"
- Profit up to 5% → "Small profit"
- Profit 5-10% → "Good profit"
- Profit more than 10% → "Excellent trade"

### Exercise 2: Trading Session Time
Based on hour (0-23), determine trading session:
- 0-7 (UTC) → Asian session
- 8-12 (UTC) → European session
- 13-21 (UTC) → American session
- 22-23 (UTC) → Transition period

### Exercise 3: Position Size Recommendation
Based on volatility and trend, give recommendation:
- High volatility + downtrend → "Minimum size or don't trade"
- High volatility + uptrend → "Reduced size"
- Low volatility + any trend → "Standard size"

### Exercise 4: Multi-Level Stop Loss
Implement trailing stop with three levels:
- Profit 5%+ → move stop to breakeven
- Profit 10%+ → move stop to +5%
- Profit 15%+ → move stop to +10%

## Homework

1. **Create function `analyze_market_condition`** that takes price, SMA-20, SMA-50, and volume, and returns a string with detailed market analysis (trend, trend strength, recommendation).

2. **Implement a risk management system** that based on:
   - Account balance
   - Current open positions
   - Requested new position size

   Returns decision: "Allowed", "Allowed with reduction", "Denied" and explanation.

3. **Create order classification function** by type and conditions:
   - Market / limit / stop
   - Buy / sell
   - Urgency: immediate / day / GTC

   Function should return execution priority (1-10) and recommendations.

4. **Write a candlestick analyzer** (open, high, low, close) that determines:
   - Candle type (bullish/bearish/doji)
   - Body size (large/medium/small)
   - Shadow presence (upper/lower/both/none)
   - Pattern (hammer, hanging man, spinning top, etc.)

## Navigation

[← Previous day](../017-if-price-check/en.md) | [Next day →](../019-match-order-types/en.md)
