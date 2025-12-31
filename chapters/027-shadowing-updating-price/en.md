# Day 27: Shadowing — Updating Price with Same Name

## Trading Analogy

Imagine a trading terminal where the asset price constantly updates. Every second a **new price** arrives that **replaces** the previous one. Yet we still call it "current price" — the name stays the same, but the value is completely new.

In Rust, this is called **shadowing** — we can declare a new variable with the same name, and it will "shadow" the previous one.

## What is Shadowing?

Shadowing allows you to redeclare a variable with the same name:

```rust
fn main() {
    let price = 42000;
    println!("Old price: {}", price);

    let price = 43500;  // New variable with the same name
    println!("New price: {}", price);
}
```

Output:
```
Old price: 42000
New price: 43500
```

**Important:** This is NOT modifying a variable (like with `mut`), but creating a **new** variable.

## Shadowing vs Mutability

What's the difference between shadowing and `mut`?

### With mut — modifying existing variable:

```rust
fn main() {
    let mut price = 42000.0;
    price = 43500.0;  // Change value of the same variable
    // You CAN'T change the type!
    // price = "forty-two thousand";  // ERROR!
}
```

### With shadowing — creating new variable:

```rust
fn main() {
    let price = "42000";         // String
    let price = price.len();     // Now it's a number (usize)!
    println!("String length: {}", price);
}
```

**Key difference:** Shadowing allows you to change the variable type!

## Why Shadowing in Trading?

### 1. Converting Data from API

Data often comes as strings:

```rust
fn main() {
    // Price came from API as string
    let btc_price = "42567.89";
    println!("Received price (string): {}", btc_price);

    // Convert to number for calculations
    let btc_price: f64 = btc_price.parse().unwrap();
    println!("Price for calculations: {} USDT", btc_price);

    // Now we can calculate
    let position_size = 0.1;
    let position_value = btc_price * position_size;
    println!("Position value: {} USDT", position_value);
}
```

### 2. Price Normalization

Exchanges may return prices in different formats:

```rust
fn main() {
    // Price in cents (like some APIs)
    let stock_price = 15099;  // $150.99 in cents
    println!("Price in cents: {}", stock_price);

    // Convert to dollars
    let stock_price = stock_price as f64 / 100.0;
    println!("Price in dollars: ${:.2}", stock_price);

    // Calculate commission
    let commission = stock_price * 0.001;
    println!("Commission: ${:.2}", commission);
}
```

### 3. Step-by-Step Data Processing

```rust
fn main() {
    // Raw data
    let ticker = "  BTC/USDT  ";
    println!("Raw ticker: '{}'", ticker);

    // Remove whitespace
    let ticker = ticker.trim();
    println!("After trim: '{}'", ticker);

    // Split into base and quote currency
    let parts: Vec<&str> = ticker.split('/').collect();
    let base_currency = parts[0];
    let quote_currency = parts[1];

    println!("Base currency: {}", base_currency);
    println!("Quote currency: {}", quote_currency);
}
```

## Practical Examples

### Example 1: Processing Exchange Data

```rust
fn main() {
    println!("=== Processing Exchange Data ===\n");

    // Data comes as strings from JSON
    let open = "42000.50";
    let high = "43500.75";
    let low = "41800.25";
    let close = "43200.00";
    let volume = "1234.56";

    // Convert to numbers for analysis
    let open: f64 = open.parse().unwrap();
    let high: f64 = high.parse().unwrap();
    let low: f64 = low.parse().unwrap();
    let close: f64 = close.parse().unwrap();
    let volume: f64 = volume.parse().unwrap();

    // Now we can analyze
    let price_range = high - low;
    let price_change = close - open;
    let change_percent = (price_change / open) * 100.0;

    println!("Open:   ${:.2}", open);
    println!("High:   ${:.2}", high);
    println!("Low:    ${:.2}", low);
    println!("Close:  ${:.2}", close);
    println!("Volume: {:.2} BTC", volume);
    println!("\n--- Analysis ---");
    println!("Day range: ${:.2}", price_range);
    println!("Change: ${:.2} ({:+.2}%)", price_change, change_percent);
}
```

### Example 2: Position Size Calculation

```rust
fn main() {
    println!("=== Position Size Calculation ===\n");

    // Input data
    let balance = 10000.0;  // USDT
    let risk_percent = 2.0; // 2%
    let entry_price = 42000.0;
    let stop_loss = 41000.0;

    // Calculate risk in dollars
    let risk = balance * (risk_percent / 100.0);
    println!("Risk per trade: ${:.2}", risk);

    // Calculate stop distance
    let stop_distance = entry_price - stop_loss;
    println!("Stop distance: ${:.2}", stop_distance);

    // Position size in BTC
    let position_size = risk / stop_distance;
    println!("Position size: {:.6} BTC", position_size);

    // Convert to value
    let position_size = position_size * entry_price;
    println!("Position value: ${:.2}", position_size);

    // Check leverage
    let leverage = position_size / balance;
    println!("Required leverage: {:.2}x", leverage);
}
```

### Example 3: Order Validation and Normalization

```rust
fn main() {
    println!("=== Order Validation ===\n");

    // User data (may be incorrect)
    let quantity = "  0.001500  ";
    let price = "42000.123456789";
    let side = "BUY";

    println!("Input data:");
    println!("  Quantity: '{}'", quantity);
    println!("  Price: '{}'", price);
    println!("  Side: '{}'", side);

    // Clean and parse quantity
    let quantity = quantity.trim();
    let quantity: f64 = quantity.parse().unwrap();

    // Round to lot size step (0.0001 BTC)
    let quantity = (quantity * 10000.0).floor() / 10000.0;
    println!("\nCleaned quantity: {:.4} BTC", quantity);

    // Parse and round price
    let price = price.trim();
    let price: f64 = price.parse().unwrap();

    // Round to tick size (0.01 USDT)
    let price = (price * 100.0).floor() / 100.0;
    println!("Rounded price: ${:.2}", price);

    // Normalize side
    let side = side.trim().to_uppercase();
    println!("Normalized side: {}", side);

    // Final order
    println!("\n--- Final Order ---");
    println!("{} {:.4} BTC @ ${:.2}", side, quantity, price);
}
```

### Example 4: Trade History Analysis

```rust
fn main() {
    println!("=== Trade History Analysis ===\n");

    // Trade results (PnL)
    let trades = [150.0, -80.0, 200.0, -50.0, 300.0, -120.0, 180.0];

    // Calculate statistics
    let mut total_pnl = 0.0;
    let mut winning_pnl = 0.0;
    let mut losing_pnl = 0.0;
    let mut wins = 0;
    let mut losses = 0;

    for trade in trades {
        total_pnl += trade;
        if trade > 0.0 {
            winning_pnl += trade;
            wins += 1;
        } else {
            losing_pnl += trade;
            losses += 1;
        }
    }

    // Shadowing for metric calculations
    let win_rate = wins as f64 / (wins + losses) as f64;
    let win_rate = win_rate * 100.0;  // Convert to percentage

    let avg_win = winning_pnl / wins as f64;
    let avg_loss = (losing_pnl / losses as f64).abs();

    let profit_factor = winning_pnl / losing_pnl.abs();

    println!("Total trades: {}", trades.len());
    println!("Winners: {}", wins);
    println!("Losers: {}", losses);
    println!("\nTotal PnL: ${:.2}", total_pnl);
    println!("Win Rate: {:.1}%", win_rate);
    println!("Average win: ${:.2}", avg_win);
    println!("Average loss: ${:.2}", avg_loss);
    println!("Profit Factor: {:.2}", profit_factor);
}
```

## Shadowing in Nested Scopes

Shadowing also works in nested blocks:

```rust
fn main() {
    let price = 42000.0;
    println!("Outer: {}", price);

    {
        // This is a new variable only inside the block
        let price = 43000.0;
        println!("Inner: {}", price);
    }

    // Here the outer variable is visible again
    println!("Outer again: {}", price);
}
```

Output:
```
Outer: 42000
Inner: 43000
Outer again: 42000
```

## When to Use Shadowing?

### Use Shadowing when:
- Converting data from string to number
- Normalizing data (cents → dollars)
- Processing data step by step
- Logically it's "the same data, but in different form"

### DON'T use Shadowing when:
- You need to update value in a loop (use `mut`)
- You need access to both values at the same time
- It might confuse code readers

## What We Learned

| Concept | Description |
|---------|-------------|
| Shadowing | Redeclaring a variable with the same name |
| Difference from mut | Shadowing creates a new variable |
| Type change | Shadowing allows changing the type |
| Scopes | Shadowing works within blocks |

## Exercises

### Exercise 1: Currency Conversion
Write a program that:
- Receives an amount in rubles as string `"85000.50"`
- Converts to number
- Converts to dollars (rate 85)
- Converts to BTC (rate $42000)

### Exercise 2: Ticker Processing
Write a program that:
- Receives ticker `"  eth_usdt  "`
- Removes whitespace
- Converts to uppercase
- Replaces `_` with `/`
- Outputs result: `"ETH/USDT"`

### Exercise 3: Order Parsing
Write a program that processes an order:
```
price: "42150.123"
amount: "0.0015678"
side: "sell"
```
And outputs normalized order:
```
SELL 0.0015 BTC @ $42150.12
```

## Homework

1. Create an exchange data processing simulator:
   - Input: strings with OHLCV prices
   - Parse and convert to numbers
   - Calculate technical indicators (SMA, range)

2. Create a trading signal validator:
   - Input: raw signal as strings
   - Normalization (trim, uppercase)
   - Convert prices and volumes
   - Output ready signal for execution

3. Implement a cross-exchange converter:
   - Binance format: `"BTCUSDT"`
   - Kraken format: `"XBT/USD"`
   - Convert between formats

## Navigation

[← Previous day](../026-*/en.md) | [Next day →](../028-*/en.md)
