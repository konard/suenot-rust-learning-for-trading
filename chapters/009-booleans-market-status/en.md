# Day 9: Booleans — Is the Market Open or Closed?

## Trading Analogy

In trading, we constantly need to answer yes/no questions:
- Is the market open? **yes/no**
- Do we have an open position? **yes/no**
- Did the stop-loss trigger? **yes/no**
- Is balance sufficient? **yes/no**

For such questions, there's the `bool` (boolean) type — it can only be `true` (yes) or `false` (no).

## Basic Usage

```rust
fn main() {
    let market_open: bool = true;
    let has_position: bool = false;
    let is_profitable: bool = true;

    println!("Market open: {}", market_open);
    println!("Has position: {}", has_position);
    println!("Profitable: {}", is_profitable);
}
```

## Comparison Operators

Comparisons return `bool`:

```rust
fn main() {
    let btc_price = 42000.0;
    let entry_price = 41000.0;
    let stop_loss = 40000.0;
    let take_profit = 45000.0;

    // Comparisons
    let in_profit = btc_price > entry_price;
    let stop_triggered = btc_price <= stop_loss;
    let tp_reached = btc_price >= take_profit;
    let price_unchanged = btc_price == entry_price;
    let price_changed = btc_price != entry_price;

    println!("In profit: {}", in_profit);           // true
    println!("Stop triggered: {}", stop_triggered); // false
    println!("TP reached: {}", tp_reached);         // false
    println!("Price unchanged: {}", price_unchanged); // false
    println!("Price changed: {}", price_changed);   // true
}
```

| Operator | Meaning | Example |
|----------|---------|---------|
| `==` | Equal | `price == 42000.0` |
| `!=` | Not equal | `price != 0.0` |
| `>` | Greater than | `price > stop_loss` |
| `<` | Less than | `price < take_profit` |
| `>=` | Greater or equal | `balance >= min_order` |
| `<=` | Less or equal | `risk <= max_risk` |

## Logical Operators

### AND — `&&`

Both conditions must be true:

```rust
fn main() {
    let has_balance = true;
    let market_open = true;
    let signal_buy = true;

    // Can only trade if ALL conditions are met
    let can_trade = has_balance && market_open && signal_buy;

    println!("Can trade: {}", can_trade);  // true
}
```

**Analogy:** To open a trade, you need to have money AND market is open AND there's a signal.

### OR — `||`

At least one condition must be true:

```rust
fn main() {
    let stop_triggered = false;
    let tp_reached = true;
    let manual_close = false;

    // Close if ANY condition is met
    let should_close = stop_triggered || tp_reached || manual_close;

    println!("Close position: {}", should_close);  // true
}
```

**Analogy:** Close the trade by stop OR by take-profit OR manually.

### NOT — `!`

Inverts the value:

```rust
fn main() {
    let market_open = true;
    let market_closed = !market_open;

    println!("Market open: {}", market_open);
    println!("Market closed: {}", market_closed);

    let has_position = false;
    let can_open_new = !has_position;  // Can open if no position

    println!("Can open new: {}", can_open_new);
}
```

## Combining Conditions

```rust
fn main() {
    let balance = 10000.0;
    let min_balance = 1000.0;
    let current_price = 42000.0;
    let entry_price = 41000.0;
    let stop_loss = 40000.0;
    let has_position = true;
    let market_open = true;

    // Complex condition for closing position
    let should_close = has_position && (
        current_price <= stop_loss ||           // Stop-loss
        current_price >= entry_price * 1.05 ||  // +5% profit
        !market_open                            // Market closing
    );

    // Can we open a new position
    let can_open = !has_position &&
                   market_open &&
                   balance >= min_balance;

    println!("Should close: {}", should_close);
    println!("Can open: {}", can_open);
}
```

## Operator Precedence

From highest to lowest:
1. `!` (NOT)
2. `&&` (AND)
3. `||` (OR)

```rust
fn main() {
    let a = true;
    let b = false;
    let c = true;

    // This:
    let result1 = a || b && c;
    // Is equivalent to:
    let result2 = a || (b && c);  // && executes first

    println!("a || b && c = {}", result1);  // true
    println!("a || (b && c) = {}", result2);  // true

    // If you need different order — use parentheses:
    let result3 = (a || b) && c;
    println!("(a || b) && c = {}", result3);  // true
}
```

## Short-circuit Evaluation

Rust doesn't check the second condition if the result is already known:

```rust
fn main() {
    let has_position = false;
    let price = 42000.0;

    // Second condition NOT checked because first is already false
    let should_sell = has_position && price > 45000.0;

    // Second condition NOT checked because first is already true
    let market_closed = true;
    let should_wait = market_closed || price < 0.0;

    println!("Should sell: {}", should_sell);
    println!("Should wait: {}", should_wait);
}
```

**Important for trading:** This can be used for error protection:

```rust
fn main() {
    let quantity = 0.0;
    let price = 42000.0;

    // Safe: if quantity == 0, division won't execute
    let is_good_deal = quantity > 0.0 && (price / quantity) < 1000.0;
}
```

## Practical Example: Signal System

```rust
fn main() {
    // Market data
    let current_price = 42500.0;
    let sma_20 = 42000.0;  // 20-period moving average
    let sma_50 = 41500.0;  // 50-period moving average
    let rsi = 65.0;        // RSI indicator
    let volume = 1500.0;   // Volume
    let avg_volume = 1000.0;

    // Signal conditions
    let price_above_sma20 = current_price > sma_20;
    let price_above_sma50 = current_price > sma_50;
    let sma_bullish = sma_20 > sma_50;  // "Golden cross"
    let rsi_not_overbought = rsi < 70.0;
    let high_volume = volume > avg_volume * 1.2;

    // Buy signal
    let buy_signal = price_above_sma20 &&
                     price_above_sma50 &&
                     sma_bullish &&
                     rsi_not_overbought &&
                     high_volume;

    // Sell conditions
    let price_below_sma20 = current_price < sma_20;
    let rsi_oversold = rsi < 30.0;

    let sell_signal = price_below_sma20 || rsi_oversold;

    // Report
    println!("╔══════════════════════════════════╗");
    println!("║        SIGNAL ANALYSIS           ║");
    println!("╠══════════════════════════════════╣");
    println!("║ Price > SMA20:      {:>12} ║", price_above_sma20);
    println!("║ Price > SMA50:      {:>12} ║", price_above_sma50);
    println!("║ SMA Bullish:        {:>12} ║", sma_bullish);
    println!("║ RSI OK:             {:>12} ║", rsi_not_overbought);
    println!("║ High Volume:        {:>12} ║", high_volume);
    println!("╠══════════════════════════════════╣");
    println!("║ BUY SIGNAL:         {:>12} ║", buy_signal);
    println!("║ SELL SIGNAL:        {:>12} ║", sell_signal);
    println!("╚══════════════════════════════════╝");
}
```

## Practical Example: Order Validation

```rust
fn main() {
    // Order parameters
    let order_side = "BUY";
    let order_price = 42000.0;
    let order_quantity = 0.5;
    let order_value = order_price * order_quantity;

    // Account and market parameters
    let balance = 25000.0;
    let min_order_value = 10.0;
    let max_order_value = 100000.0;
    let market_open = true;
    let trading_enabled = true;

    // Validation
    let has_enough_balance = balance >= order_value;
    let above_min = order_value >= min_order_value;
    let below_max = order_value <= max_order_value;
    let positive_quantity = order_quantity > 0.0;
    let positive_price = order_price > 0.0;
    let valid_side = order_side == "BUY" || order_side == "SELL";

    // Final check
    let order_valid = has_enough_balance &&
                      above_min &&
                      below_max &&
                      positive_quantity &&
                      positive_price &&
                      valid_side &&
                      market_open &&
                      trading_enabled;

    println!("=== Order Validation ===");
    println!("Side: {}", order_side);
    println!("Price: ${}", order_price);
    println!("Quantity: {}", order_quantity);
    println!("Value: ${}", order_value);
    println!();
    println!("Balance OK: {}", has_enough_balance);
    println!("Above min: {}", above_min);
    println!("Below max: {}", below_max);
    println!("Valid quantity: {}", positive_quantity);
    println!("Valid price: {}", positive_price);
    println!("Valid side: {}", valid_side);
    println!("Market open: {}", market_open);
    println!("Trading enabled: {}", trading_enabled);
    println!();
    println!("ORDER VALID: {}", order_valid);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `bool` | true or false |
| `==`, `!=`, `<`, `>` | Comparisons |
| `&&` | Logical AND |
| `||` | Logical OR |
| `!` | Logical NOT |
| Short-circuit | Lazy evaluation |

## Homework

1. Create a buy signal checking system with 5+ conditions

2. Implement a check: can we open a position (check balance, limits, market status)

3. Write position closing logic with 3 conditions:
   - Stop-loss
   - Take-profit
   - Time expiration

4. Experiment with operator precedence

## Navigation

[← Previous day](../008-floating-point-bitcoin-price/en.md) | [Next day →](../010-strings-tickers/en.md)
