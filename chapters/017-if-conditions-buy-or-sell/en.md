# Day 17: If Conditions — To Buy or To Sell?

## Trading Analogy

Imagine: you're looking at a chart and making a decision. **If** the price is below the moving average — buy. **If** RSI is above 70 — overbought, don't enter. Every trading decision is a condition. In programming, conditions work exactly the same way through the `if` operator.

## Basic if Syntax

```rust
fn main() {
    let btc_price = 42000.0;
    let buy_level = 40000.0;

    if btc_price < buy_level {
        println!("Price below buy level — BUY!");
    }
}
```

**Important:** The condition must return `bool` (true or false). Rust doesn't do implicit type conversions.

## Comparison Operators

```rust
fn main() {
    let current_price = 42000.0;
    let entry_price = 41500.0;
    let target_price = 43000.0;
    let stop_loss = 41000.0;

    // Equality and inequality
    if current_price == entry_price {
        println!("Price at entry level");
    }

    if current_price != entry_price {
        println!("Price changed since entry");
    }

    // Greater / less than
    if current_price > entry_price {
        println!("We're in profit!");
    }

    if current_price < stop_loss {
        println!("STOP-LOSS triggered!");
    }

    // Greater or equal / less or equal
    if current_price >= target_price {
        println!("Take profit reached!");
    }

    if current_price <= stop_loss {
        println!("Reached or broke through stop-loss");
    }
}
```

## Determining Trade Profitability

```rust
fn main() {
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;

    let pnl = (exit_price - entry_price) * quantity;

    if pnl > 0.0 {
        println!("Profitable trade! PnL: ${:.2}", pnl);
    }

    if pnl < 0.0 {
        println!("Losing trade. PnL: ${:.2}", pnl);
    }

    if pnl == 0.0 {
        println!("Breakeven");
    }
}
```

## Checking Market Conditions

```rust
fn main() {
    let market_open = true;
    let has_balance = true;
    let price_valid = true;

    if market_open {
        println!("Market is open, we can trade");
    }

    if !market_open {
        println!("Market is closed, waiting for open");
    }

    // Direct boolean checks
    if has_balance {
        println!("Balance is sufficient");
    }

    if price_valid {
        println!("Price is valid");
    }
}
```

## Logical Operators

### AND (&&) — all conditions must be true

```rust
fn main() {
    let price = 42000.0;
    let rsi = 35.0;
    let volume = 1500.0;
    let min_volume = 1000.0;

    // Buy only if ALL conditions are met
    if price < 43000.0 && rsi < 40.0 && volume > min_volume {
        println!("All buy conditions met — BUY!");
    }
}
```

### OR (||) — at least one condition is true

```rust
fn main() {
    let rsi = 75.0;
    let price_at_resistance = true;
    let bearish_divergence = false;

    // Sell signal if AT LEAST ONE condition is met
    if rsi > 70.0 || price_at_resistance || bearish_divergence {
        println!("Sell signal detected!");
    }
}
```

### NOT (!) — negation

```rust
fn main() {
    let is_weekend = false;
    let market_holiday = false;

    if !is_weekend && !market_holiday {
        println!("Trading day, ready to work");
    }
}
```

## Combining Conditions

```rust
fn main() {
    let price = 42000.0;
    let sma_20 = 41500.0;
    let sma_50 = 41000.0;
    let rsi = 45.0;
    let volume = 2000.0;
    let avg_volume = 1500.0;

    // Complex condition for long entry
    let price_above_sma = price > sma_20 && price > sma_50;
    let rsi_neutral = rsi > 40.0 && rsi < 60.0;
    let volume_confirmed = volume > avg_volume;

    if price_above_sma && rsi_neutral && volume_confirmed {
        println!("Strong buy signal!");
        println!("Price above SMA20 and SMA50");
        println!("RSI in neutral zone");
        println!("Volume confirms the move");
    }
}
```

## Order Validation

```rust
fn main() {
    let order_price = 42000.0;
    let order_quantity = 0.5;
    let balance = 25000.0;
    let min_order_size = 0.001;
    let max_order_size = 10.0;

    let order_value = order_price * order_quantity;

    // Validate order
    if order_price > 0.0 {
        println!("✓ Price is positive");
    }

    if order_quantity >= min_order_size && order_quantity <= max_order_size {
        println!("✓ Quantity within allowed range");
    }

    if order_value <= balance {
        println!("✓ Sufficient funds");
    }

    // Full validation
    if order_price > 0.0
        && order_quantity >= min_order_size
        && order_quantity <= max_order_size
        && order_value <= balance
    {
        println!("Order is valid — ready to submit!");
    }
}
```

## Risk Checks

```rust
fn main() {
    let position_size = 5000.0;
    let portfolio_value = 100000.0;
    let max_position_percent = 5.0;
    let current_drawdown = 8.0;
    let max_drawdown = 10.0;

    let position_percent = (position_size / portfolio_value) * 100.0;

    // Position size check
    if position_percent > max_position_percent {
        println!("⚠ Position too large: {:.1}% > {:.1}%",
                 position_percent, max_position_percent);
    }

    // Drawdown check
    if current_drawdown > max_drawdown {
        println!("🛑 Max drawdown exceeded! Trading stopped.");
    }

    // Can we trade?
    if position_percent <= max_position_percent && current_drawdown <= max_drawdown {
        println!("✓ Risks within limits, ready to trade");
    }
}
```

## Practical Example: Entry Signal

```rust
fn main() {
    // Current market state
    let price = 42000.0;
    let sma_20 = 41500.0;
    let rsi = 35.0;
    let macd_histogram = 50.0;
    let volume = 2500.0;
    let avg_volume = 2000.0;

    println!("=== BTC/USDT Market Analysis ===");
    println!("Price: ${}", price);
    println!();

    // Check each condition separately
    let trend_up = price > sma_20;
    let oversold = rsi < 40.0;
    let macd_bullish = macd_histogram > 0.0;
    let volume_high = volume > avg_volume;

    println!("Condition checks:");

    if trend_up {
        println!("✓ Uptrend (price > SMA20)");
    }

    if oversold {
        println!("✓ RSI shows oversold");
    }

    if macd_bullish {
        println!("✓ MACD bullish");
    }

    if volume_high {
        println!("✓ Volume above average");
    }

    println!();

    // Final decision
    if trend_up && oversold && macd_bullish && volume_high {
        println!("🚀 SIGNAL: BUY!");
    }
}
```

## if as an Expression

In Rust, `if` can return a value:

```rust
fn main() {
    let pnl = 750.0;

    // if as expression
    let status = if pnl > 0.0 {
        "PROFIT"
    } else {
        "LOSS"
    };

    println!("Trade status: {}", status);

    // Fee calculation based on condition
    let trade_volume = 50000.0;
    let fee_percent = if trade_volume > 100000.0 {
        0.05  // VIP fee
    } else {
        0.1   // Standard fee
    };

    let fee = trade_volume * (fee_percent / 100.0);
    println!("Fee: ${:.2} ({:.2}%)", fee, fee_percent);
}
```

**Important:** When using `if` as an expression, both blocks must return the same type!

## Determining Trade Direction

```rust
fn main() {
    let entry_price = 42000.0;
    let exit_price = 43500.0;

    let direction = if exit_price > entry_price {
        "LONG"
    } else {
        "SHORT"
    };

    let profit = (exit_price - entry_price).abs();

    println!("Direction: {}", direction);
    println!("Profit: ${:.2}", profit);
}
```

## Nested Conditions

```rust
fn main() {
    let balance = 10000.0;
    let min_balance = 1000.0;
    let price = 42000.0;
    let quantity = 0.1;
    let order_value = price * quantity;

    if balance >= min_balance {
        println!("Balance sufficient for trading");

        if order_value <= balance {
            println!("Order can be executed");

            if quantity > 0.0 {
                println!("Quantity is valid");
                println!("✓ All checks passed!");
            }
        }
    }
}
```

## What We Learned

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `price == target` |
| `!=` | Not equal | `price != entry` |
| `>` | Greater than | `price > sma` |
| `<` | Less than | `rsi < 30` |
| `>=` | Greater or equal | `volume >= min_volume` |
| `<=` | Less or equal | `drawdown <= max_drawdown` |
| `&&` | Logical AND | `a > 0 && b > 0` |
| `\|\|` | Logical OR | `rsi > 70 \|\| rsi < 30` |
| `!` | Logical NOT | `!market_closed` |

## Homework

1. Write a program that checks if an entry price is favorable (below SMA and RSI < 40)

2. Create an order validator that checks:
   - Price > 0
   - Quantity > 0 and < maximum
   - Sufficient balance
   - Market is open

3. Implement risk checks:
   - Position size doesn't exceed 5% of portfolio
   - Drawdown doesn't exceed limit
   - Correlation with existing positions is below 0.8

4. Write a function that returns position size based on risk conditions (use `if` as expression)

## Navigation

[← Previous day](../015-return-values-pnl/en.md) | [Next day →](../018-else-market-scenarios/en.md)
