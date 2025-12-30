# Day 7: Integers — Counting Shares

## Trading Analogy

In trading, we often need **whole numbers**:
- Number of shares (can't buy 1.5 Apple shares)
- Order ID (unique number)
- Candle number in history
- Timestamp in milliseconds
- Number of trades per day

## Integer Types in Rust

### Signed (can be negative)

```rust
fn main() {
    let pnl_today: i32 = -500;      // $500 loss
    let price_change: i64 = -1500;  // Price dropped 1500 points
    let temperature: i8 = -10;      // For server room datacenter

    println!("PnL: {} USD", pnl_today);
}
```

| Type | Minimum | Maximum |
|------|---------|---------|
| `i8` | -128 | 127 |
| `i16` | -32,768 | 32,767 |
| `i32` | -2,147,483,648 | 2,147,483,647 |
| `i64` | -9.2 × 10¹⁸ | 9.2 × 10¹⁸ |
| `i128` | -1.7 × 10³⁸ | 1.7 × 10³⁸ |

### Unsigned (positive only)

```rust
fn main() {
    let order_id: u64 = 9876543210;
    let trade_count: u32 = 1500;
    let shares: u16 = 100;
    let percentage: u8 = 85;

    println!("Order #{}: {} shares", order_id, shares);
}
```

| Type | Minimum | Maximum |
|------|---------|---------|
| `u8` | 0 | 255 |
| `u16` | 0 | 65,535 |
| `u32` | 0 | 4,294,967,295 |
| `u64` | 0 | 18.4 × 10¹⁸ |
| `u128` | 0 | 3.4 × 10³⁸ |

### `isize` and `usize` — Architecture-dependent

```rust
fn main() {
    let array_index: usize = 5;  // Array index
    let length: usize = 100;     // Collection length

    // On 64-bit system: usize = u64
    // On 32-bit system: usize = u32
}
```

**Important:** For array indices, ALWAYS use `usize`.

## Number Literals

```rust
fn main() {
    // Decimal
    let volume = 1_000_000;        // Underscores for readability
    let price = 42_000;

    // Hexadecimal
    let color = 0xFF00FF;          // For chart colors

    // Binary
    let flags = 0b1010_1010;       // Bit flags

    // Octal
    let permissions = 0o755;       // Unix permissions

    // With type suffix
    let small: i8 = 42i8;
    let big = 1_000_000u64;
}
```

## Arithmetic Operations

```rust
fn main() {
    let shares_bought = 100;
    let shares_sold = 30;
    let shares_left = shares_bought - shares_sold;

    println!("Remaining shares: {}", shares_left);

    // All operations
    let a = 10;
    let b = 3;

    println!("Addition: {} + {} = {}", a, b, a + b);
    println!("Subtraction: {} - {} = {}", a, b, a - b);
    println!("Multiplication: {} * {} = {}", a, b, a * b);
    println!("Division: {} / {} = {}", a, b, a / b);      // 3 (integer!)
    println!("Remainder: {} % {} = {}", a, b, a % b);     // 1
}
```

**Important:** Integer division yields an integer! `10 / 3 = 3`, not 3.333...

## Overflow

```rust
fn main() {
    let max_u8: u8 = 255;
    // let overflow: u8 = max_u8 + 1;  // ERROR in debug mode!

    // Safe operations
    let result = max_u8.checked_add(1);      // None if overflow
    let wrapped = max_u8.wrapping_add(1);    // 0 (wraps around)
    let saturated = max_u8.saturating_add(1); // 255 (maximum)

    println!("Checked: {:?}", result);
    println!("Wrapped: {}", wrapped);
    println!("Saturated: {}", saturated);
}
```

**Analogy:** Imagine a car odometer. When it reaches 999999, the next kilometer either errors or resets to 000000.

## Practical Example: Share Counting

```rust
fn main() {
    // Initial portfolio
    let mut apple_shares: u32 = 50;
    let mut google_shares: u32 = 20;
    let mut tesla_shares: u32 = 30;

    println!("=== Start of Day ===");
    println!("AAPL: {} shares", apple_shares);
    println!("GOOG: {} shares", google_shares);
    println!("TSLA: {} shares", tesla_shares);

    // Day's trades
    apple_shares += 10;   // Bought more Apple
    google_shares -= 5;   // Sold some Google
    tesla_shares += 15;   // Bought more Tesla

    println!("\n=== After Trading ===");
    println!("AAPL: {} shares", apple_shares);
    println!("GOOG: {} shares", google_shares);
    println!("TSLA: {} shares", tesla_shares);

    // Total
    let total_shares = apple_shares + google_shares + tesla_shares;
    println!("\nTotal shares: {}", total_shares);
}
```

## Practical Example: Order ID Generator

```rust
fn main() {
    // Simulate order ID generator
    let mut next_order_id: u64 = 1000000;

    // Create orders
    let order1 = next_order_id;
    next_order_id += 1;

    let order2 = next_order_id;
    next_order_id += 1;

    let order3 = next_order_id;
    next_order_id += 1;

    println!("Order 1: #{}", order1);
    println!("Order 2: #{}", order2);
    println!("Order 3: #{}", order3);
    println!("Next ID: {}", next_order_id);
}
```

## Practical Example: Lot Calculation

```rust
fn main() {
    let balance_usd: u64 = 10_000;
    let apple_price: u64 = 185;
    let lot_size: u64 = 1;  // Minimum lot

    // How many can we buy?
    let max_shares = balance_usd / apple_price;
    let remaining_usd = balance_usd % apple_price;

    println!("Balance: ${}", balance_usd);
    println!("AAPL price: ${}", apple_price);
    println!("Can buy: {} shares", max_shares);
    println!("Remaining: ${}", remaining_usd);
    println!("Will spend: ${}", max_shares * apple_price);
}
```

## Type Conversion

```rust
fn main() {
    let small: u8 = 100;
    let medium: u32 = small as u32;    // Safe: u8 -> u32
    let large: u64 = medium as u64;    // Safe: u32 -> u64

    let big: u64 = 1000;
    let truncated: u8 = big as u8;     // Dangerous! 1000 -> 232

    println!("Small: {}", small);
    println!("Large: {}", large);
    println!("Truncated (DANGEROUS!): {}", truncated);
}
```

**Rule:** You can always safely convert a smaller type to larger. The reverse is dangerous!

## Bitwise Operations

```rust
fn main() {
    // Order status flags
    let filled: u8 = 0b0000_0001;    // Bit 0: filled
    let cancelled: u8 = 0b0000_0010; // Bit 1: cancelled
    let partial: u8 = 0b0000_0100;   // Bit 2: partially filled

    let mut order_status: u8 = 0;

    // Set "partially filled" flag
    order_status |= partial;
    println!("Status: {:08b}", order_status);

    // Then fully filled
    order_status |= filled;
    order_status &= !partial;  // Remove partial
    println!("Status: {:08b}", order_status);

    // Check flag
    if order_status & filled != 0 {
        println!("Order filled!");
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `i32`, `i64` | Signed numbers (for PnL) |
| `u32`, `u64` | Unsigned numbers (for IDs) |
| `usize` | Array indices |
| `_` in numbers | For readability: 1_000_000 |
| Overflow | Dangerous! Use checked/saturating |

## Homework

1. Create a simulation of a portfolio with 5 stocks and quantity of each

2. Implement a function to calculate total value:
   - Stock prices (whole dollars)
   - Quantity of each
   - Total sum

3. Experiment with overflow:
   - Create `u8 = 255`
   - Try adding 1 in different ways

4. Create an Order ID generator that starts from current timestamp

## Navigation

[← Previous day](../006-data-types/en.md) | [Next day →](../008-floating-point-bitcoin-price/en.md)
