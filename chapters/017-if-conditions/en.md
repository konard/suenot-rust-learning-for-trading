# Day 17: If Conditions — Making Trading Decisions

## Trading Analogy

Trading is all about making decisions based on conditions:
- **IF** price is above the moving average, **THEN** consider buying
- **IF** stop-loss triggered, **THEN** close position
- **IF** RSI > 70, **THEN** market is overbought, **ELSE IF** RSI < 30, **THEN** oversold, **ELSE** neutral

In Rust, this is implemented using the `if-else` construct.

## Basic if Syntax

```rust
fn main() {
    let btc_price = 42000.0;
    let entry_price = 41000.0;

    if btc_price > entry_price {
        println!("Position is in profit!");
    }
}
```

**Important:** The condition must be of type `bool`. Rust doesn't implicitly convert numbers to booleans.

```rust
fn main() {
    let balance = 1000.0;

    // This WON'T work:
    // if balance { }  // Error! f64 is not bool

    // Correct:
    if balance > 0.0 {
        println!("Account has funds");
    }
}
```

## if-else — Two Options

```rust
fn main() {
    let current_price = 42000.0;
    let entry_price = 43000.0;

    if current_price > entry_price {
        println!("PROFIT: Price above entry");
    } else {
        println!("LOSS: Price at or below entry");
    }
}
```

### Trading Application: Determining Position Direction

```rust
fn main() {
    let pnl = -150.0;

    if pnl >= 0.0 {
        println!("Trade profitable: +${:.2}", pnl);
    } else {
        println!("Trade at loss: ${:.2}", pnl);
    }
}
```

## if-else if-else — Multiple Conditions

```rust
fn main() {
    let rsi = 75.0;

    if rsi > 70.0 {
        println!("Overbought — potential reversal down");
    } else if rsi < 30.0 {
        println!("Oversold — potential reversal up");
    } else {
        println!("Neutral zone");
    }
}
```

### Determining Trend Strength

```rust
fn main() {
    let price_change_percent = 3.5;

    if price_change_percent > 5.0 {
        println!("Strong rally!");
    } else if price_change_percent > 2.0 {
        println!("Moderate rise");
    } else if price_change_percent > 0.0 {
        println!("Slight rise");
    } else if price_change_percent > -2.0 {
        println!("Slight drop");
    } else if price_change_percent > -5.0 {
        println!("Moderate drop");
    } else {
        println!("Strong selloff!");
    }
}
```

## if as an Expression

In Rust, `if` is an expression that returns a value:

```rust
fn main() {
    let price = 42000.0;
    let support = 41000.0;

    let signal = if price > support {
        "BUY"
    } else {
        "WAIT"
    };

    println!("Signal: {}", signal);
}
```

**Important:**
- Both branches must return the same type
- No semicolon after values inside branches
- Semicolon at the end of the entire expression

### Fee Calculation Based on Tier

```rust
fn main() {
    let trade_volume = 50000.0;

    let fee_percent = if trade_volume > 100000.0 {
        0.05  // VIP: 0.05%
    } else if trade_volume > 10000.0 {
        0.08  // Mid tier: 0.08%
    } else {
        0.1   // Base: 0.1%
    };

    let fee = trade_volume * (fee_percent / 100.0);
    println!("Volume: ${:.2}, Fee: {:.2}%, Amount: ${:.2}",
             trade_volume, fee_percent, fee);
}
```

## Nested Conditions

```rust
fn main() {
    let has_position = true;
    let current_price = 43000.0;
    let entry_price = 42000.0;
    let stop_loss = 41000.0;
    let take_profit = 45000.0;

    if has_position {
        if current_price <= stop_loss {
            println!("STOP-LOSS! Closing with loss");
        } else if current_price >= take_profit {
            println!("TAKE-PROFIT! Securing gains");
        } else if current_price > entry_price {
            println!("Position in profit, holding");
        } else {
            println!("Position at loss, but stop not triggered");
        }
    } else {
        println!("No open positions");
    }
}
```

### Alternative: Combining Conditions

```rust
fn main() {
    let has_position = true;
    let current_price = 43000.0;
    let stop_loss = 41000.0;
    let take_profit = 45000.0;

    // Instead of nested if, use &&
    if has_position && current_price <= stop_loss {
        println!("Closing at stop-loss");
    } else if has_position && current_price >= take_profit {
        println!("Closing at take-profit");
    }
}
```

## Practical Example: Trading Signal

```rust
fn main() {
    // Market data
    let current_price = 42500.0;
    let sma_20 = 42000.0;
    let sma_50 = 41500.0;
    let rsi = 55.0;
    let volume = 1500.0;
    let avg_volume = 1000.0;

    // Trend analysis
    let trend = if sma_20 > sma_50 {
        "UPTREND"
    } else if sma_20 < sma_50 {
        "DOWNTREND"
    } else {
        "SIDEWAYS"
    };

    // RSI analysis
    let rsi_signal = if rsi > 70.0 {
        "OVERBOUGHT"
    } else if rsi < 30.0 {
        "OVERSOLD"
    } else {
        "NEUTRAL"
    };

    // Volume analysis
    let volume_signal = if volume > avg_volume * 1.5 {
        "HIGH"
    } else if volume < avg_volume * 0.5 {
        "LOW"
    } else {
        "NORMAL"
    };

    // Final signal
    let action = if trend == "UPTREND" && rsi_signal != "OVERBOUGHT" && volume_signal == "HIGH" {
        "STRONG BUY"
    } else if trend == "UPTREND" && rsi_signal != "OVERBOUGHT" {
        "BUY"
    } else if trend == "DOWNTREND" && rsi_signal != "OVERSOLD" {
        "SELL"
    } else {
        "HOLD"
    };

    println!("╔════════════════════════════════════╗");
    println!("║       TRADING SIGNAL ANALYSIS      ║");
    println!("╠════════════════════════════════════╣");
    println!("║ Price:        ${:>18.2} ║", current_price);
    println!("║ SMA-20:       ${:>18.2} ║", sma_20);
    println!("║ SMA-50:       ${:>18.2} ║", sma_50);
    println!("║ RSI:           {:>18.1} ║", rsi);
    println!("╠════════════════════════════════════╣");
    println!("║ Trend:         {:>18} ║", trend);
    println!("║ RSI Signal:    {:>18} ║", rsi_signal);
    println!("║ Volume:        {:>18} ║", volume_signal);
    println!("╠════════════════════════════════════╣");
    println!("║ >>> ACTION:    {:>18} ║", action);
    println!("╚════════════════════════════════════╝");
}
```

## Practical Example: Order Validation

```rust
fn main() {
    let order_type = "LIMIT";
    let side = "BUY";
    let price = 42000.0;
    let quantity = 0.5;
    let balance = 25000.0;
    let market_open = true;

    let order_value = price * quantity;

    // Validation
    let is_valid = if !market_open {
        println!("Error: Market is closed");
        false
    } else if price <= 0.0 {
        println!("Error: Invalid price");
        false
    } else if quantity <= 0.0 {
        println!("Error: Invalid quantity");
        false
    } else if side == "BUY" && order_value > balance {
        println!("Error: Insufficient funds (need ${:.2}, have ${:.2})",
                 order_value, balance);
        false
    } else if order_type != "LIMIT" && order_type != "MARKET" {
        println!("Error: Unknown order type");
        false
    } else {
        println!("Order validation passed");
        true
    };

    if is_valid {
        println!("\nOrder accepted:");
        println!("  Type: {}", order_type);
        println!("  Side: {}", side);
        println!("  Price: ${:.2}", price);
        println!("  Quantity: {}", quantity);
        println!("  Value: ${:.2}", order_value);
    }
}
```

## Practical Example: Position Size Calculator

```rust
fn main() {
    let balance = 10000.0;
    let risk_percent = 2.0;
    let entry_price = 42000.0;
    let stop_loss = 40000.0;

    // Input validation
    let position_size = if balance <= 0.0 {
        println!("Error: Zero balance");
        0.0
    } else if risk_percent <= 0.0 || risk_percent > 100.0 {
        println!("Error: Invalid risk percentage");
        0.0
    } else if stop_loss >= entry_price {
        println!("Error: Stop-loss must be below entry for long position");
        0.0
    } else {
        let risk_amount = balance * (risk_percent / 100.0);
        let risk_per_unit = entry_price - stop_loss;
        let size = risk_amount / risk_per_unit;

        println!("Risk amount: ${:.2}", risk_amount);
        println!("Risk per unit: ${:.2}", risk_per_unit);

        size
    };

    if position_size > 0.0 {
        let position_value = position_size * entry_price;
        println!("\n=== Position Size ===");
        println!("Quantity: {:.6} BTC", position_size);
        println!("Value: ${:.2}", position_value);
        println!("Leverage: {:.1}x", position_value / balance);
    }
}
```

## Patterns for Using if

### 1. Early Return

```rust
fn calculate_pnl(entry: f64, exit: f64, quantity: f64) -> f64 {
    if quantity == 0.0 {
        return 0.0;  // Early exit
    }
    (exit - entry) * quantity
}

fn main() {
    println!("PnL: {}", calculate_pnl(42000.0, 43000.0, 0.5));
    println!("PnL: {}", calculate_pnl(42000.0, 43000.0, 0.0));
}
```

### 2. Conditional Assignment

```rust
fn main() {
    let pnl = -500.0;

    let status = if pnl > 0.0 { "PROFIT" }
                 else if pnl < 0.0 { "LOSS" }
                 else { "BREAKEVEN" };

    println!("Status: {}", status);
}
```

### 3. Conditional Action Execution

```rust
fn main() {
    let alert_enabled = true;
    let price = 45000.0;
    let alert_price = 44000.0;

    if alert_enabled && price >= alert_price {
        println!("ALERT: Price reached ${:.2}!", price);
    }
}
```

## What We Learned

| Construct | Description | Example |
|-----------|-------------|---------|
| `if` | Simple condition | `if price > 0.0 { ... }` |
| `if-else` | Two options | `if profit { buy } else { sell }` |
| `if-else if-else` | Multiple conditions | RSI level detection |
| `if` as expression | Returns a value | `let x = if ... { a } else { b };` |
| Nested `if` | Conditions within conditions | Complex logic |

## Homework

1. **Volatility Classifier**
   Write a program that classifies volatility based on daily price range:
   - < 1% — low
   - 1-3% — normal
   - 3-5% — elevated
   - > 5% — high

2. **Alert System**
   Create a system that checks price and generates alerts:
   - Resistance level breakout
   - Support level breakdown
   - Take-profit reached
   - Stop-loss triggered

3. **Fee Calculator**
   Implement an exchange fee calculator considering:
   - Maker/Taker orders
   - VIP tier (based on trading volume)
   - Native token usage (25% discount)

4. **Strategy Validator**
   Write a validator for trading strategy parameters:
   - Stop-loss is mandatory
   - Take-profit must be above entry (for long)
   - Risk per trade no more than 2% of deposit
   - Minimum risk/reward ratio 1:2

## Navigation

[← Previous day](../016-comments-trading-logic/en.md) | [Next day →](../018-else-if-order-types/en.md)
