# Day 70: Enum — OrderSide: Buy or Sell

## Trading Analogy

In trading, every order has a **direction**:
- **Buy** — open a long position or close a short one
- **Sell** — open a short position or close a long one

These are **mutually exclusive** states — an order cannot be both buy and sell at the same time. In Rust, we use **enums** to model such situations.

## What is an Enum?

An enum (enumeration) is a data type that can be one of several predefined values:

```rust
// Define an enum for order direction
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let my_order = OrderSide::Buy;
    let another_order = OrderSide::Sell;

    println!("Order created!");
}
```

## Creating Enums

```rust
enum OrderSide {
    Buy,   // Variant 1: buy
    Sell,  // Variant 2: sell
}

fn main() {
    // Create enum values
    let side1 = OrderSide::Buy;
    let side2 = OrderSide::Sell;

    println!("Orders created");
}
```

## Pattern Matching

Use the `match` construct to work with enums:

```rust
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let side = OrderSide::Buy;

    // match checks all variants
    match side {
        OrderSide::Buy => println!("Buying the asset"),
        OrderSide::Sell => println!("Selling the asset"),
    }
}
```

### Returning Values from match

```rust
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let side = OrderSide::Sell;

    let action = match side {
        OrderSide::Buy => "LONG",
        OrderSide::Sell => "SHORT",
    };

    println!("Position direction: {}", action);
}
```

## Enums with Data

Enum variants can contain data:

```rust
enum OrderType {
    Market,                      // Market order (no data)
    Limit(f64),                  // Limit order (with price)
    StopLoss { price: f64, trigger: f64 },  // Stop-loss (named fields)
}

fn main() {
    let order1 = OrderType::Market;
    let order2 = OrderType::Limit(42000.0);
    let order3 = OrderType::StopLoss {
        price: 41000.0,
        trigger: 41500.0
    };

    match order1 {
        OrderType::Market => println!("Market order — executes immediately"),
        OrderType::Limit(price) => println!("Limit order at price {}", price),
        OrderType::StopLoss { price, trigger } => {
            println!("Stop-loss: triggers at {}, executes at {}", trigger, price);
        }
    }
}
```

## Practical Example: OrderSide

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let side = OrderSide::Buy;
    let price = 42000.0;
    let quantity = 0.5;

    // Calculate position value
    let position_value = price * quantity;

    // Determine position sign
    let signed_quantity = match side {
        OrderSide::Buy => quantity,
        OrderSide::Sell => -quantity,
    };

    println!("╔════════════════════════════════╗");
    println!("║         ORDER DETAILS          ║");
    println!("╠════════════════════════════════╣");
    println!("║ Side:     {:?}                 ║", side);
    println!("║ Price:    ${:.2}           ║", price);
    println!("║ Quantity: {:.4} BTC           ║", quantity);
    println!("║ Value:    ${:.2}           ║", position_value);
    println!("║ Signed:   {:+.4}              ║", signed_quantity);
    println!("╚════════════════════════════════╝");
}
```

## Practical Example: PnL Calculation

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    // Open position
    let entry_side = OrderSide::Buy;
    let entry_price = 42000.0;
    let quantity = 0.5;

    // Current price
    let current_price = 43500.0;

    // PnL calculation depends on direction
    let pnl = calculate_pnl(entry_side, entry_price, current_price, quantity);

    println!("Entry: {:?} @ ${:.2}", entry_side, entry_price);
    println!("Current: ${:.2}", current_price);
    println!("PnL: ${:+.2}", pnl);
}

fn calculate_pnl(side: OrderSide, entry: f64, current: f64, qty: f64) -> f64 {
    match side {
        OrderSide::Buy => (current - entry) * qty,  // Long: price up = profit
        OrderSide::Sell => (entry - current) * qty, // Short: price down = profit
    }
}
```

## Practical Example: Trading Signal

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
enum Signal {
    Enter(OrderSide),
    Exit(OrderSide),
    Hold,
}

fn main() {
    let prices = [42000.0, 42500.0, 42300.0, 43000.0, 42800.0];

    println!("=== Trading Signals ===\n");

    for i in 1..prices.len() {
        let prev = prices[i - 1];
        let curr = prices[i];

        let signal = generate_signal(prev, curr);

        let action = match &signal {
            Signal::Enter(side) => format!("ENTER {:?}", side),
            Signal::Exit(side) => format!("EXIT {:?}", side),
            Signal::Hold => "HOLD".to_string(),
        };

        println!("Price: {:.2} -> {:.2} | Signal: {}", prev, curr, action);
    }
}

fn generate_signal(prev_price: f64, current_price: f64) -> Signal {
    let change_percent = (current_price - prev_price) / prev_price * 100.0;

    if change_percent > 1.0 {
        Signal::Enter(OrderSide::Buy)  // Strong rise — buy
    } else if change_percent < -1.0 {
        Signal::Enter(OrderSide::Sell) // Strong drop — sell
    } else {
        Signal::Hold // Sideways — wait
    }
}
```

## Practical Example: Order Book

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
struct Order {
    id: u64,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

fn main() {
    let orders = vec![
        Order { id: 1, side: OrderSide::Buy, price: 41990.0, quantity: 1.5 },
        Order { id: 2, side: OrderSide::Buy, price: 41980.0, quantity: 2.0 },
        Order { id: 3, side: OrderSide::Sell, price: 42010.0, quantity: 0.8 },
        Order { id: 4, side: OrderSide::Sell, price: 42020.0, quantity: 1.2 },
        Order { id: 5, side: OrderSide::Buy, price: 41970.0, quantity: 3.0 },
    ];

    // Separate orders by side
    let bids: Vec<&Order> = orders.iter()
        .filter(|o| o.side == OrderSide::Buy)
        .collect();

    let asks: Vec<&Order> = orders.iter()
        .filter(|o| o.side == OrderSide::Sell)
        .collect();

    println!("=== ORDER BOOK ===\n");

    println!("BIDS (buy orders):");
    for order in &bids {
        println!("  ${:.2} x {:.4}", order.price, order.quantity);
    }

    println!("\nASKS (sell orders):");
    for order in &asks {
        println!("  ${:.2} x {:.4}", order.price, order.quantity);
    }

    // Best prices
    let best_bid = bids.iter().map(|o| o.price).fold(f64::MIN, f64::max);
    let best_ask = asks.iter().map(|o| o.price).fold(f64::MAX, f64::min);

    println!("\n--- Spread ---");
    println!("Best Bid: ${:.2}", best_bid);
    println!("Best Ask: ${:.2}", best_ask);
    println!("Spread:   ${:.2}", best_ask - best_bid);
}
```

## Methods for Enums

You can add methods to enums using `impl`:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    // Opposite side
    fn opposite(&self) -> OrderSide {
        match self {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        }
    }

    // Sign for calculations
    fn sign(&self) -> f64 {
        match self {
            OrderSide::Buy => 1.0,
            OrderSide::Sell => -1.0,
        }
    }

    // String representation
    fn as_str(&self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }
}

fn main() {
    let side = OrderSide::Buy;

    println!("Side: {}", side.as_str());
    println!("Sign: {}", side.sign());
    println!("Opposite: {}", side.opposite().as_str());

    // Use in calculations
    let quantity = 0.5;
    let signed_qty = quantity * side.sign();
    println!("Signed quantity: {:+.4}", signed_qty);
}
```

## Complete Example: Order System

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
}

#[derive(Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    filled: f64,
    status: OrderStatus,
}

impl Order {
    fn new(id: u64, symbol: &str, side: OrderSide, price: f64, quantity: f64) -> Self {
        Order {
            id,
            symbol: symbol.to_string(),
            side,
            price,
            quantity,
            filled: 0.0,
            status: OrderStatus::Pending,
        }
    }

    fn fill(&mut self, amount: f64) {
        self.filled += amount;

        if self.filled >= self.quantity {
            self.filled = self.quantity;
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }

    fn cancel(&mut self) {
        if self.status != OrderStatus::Filled {
            self.status = OrderStatus::Cancelled;
        }
    }

    fn remaining(&self) -> f64 {
        self.quantity - self.filled
    }

    fn fill_percent(&self) -> f64 {
        (self.filled / self.quantity) * 100.0
    }
}

fn main() {
    let mut order = Order::new(1, "BTC/USDT", OrderSide::Buy, 42000.0, 1.0);

    println!("=== Order Lifecycle ===\n");

    println!("Created: {:?}", order.status);
    println!("Remaining: {:.4}\n", order.remaining());

    // Partial fill
    order.fill(0.3);
    println!("After partial fill:");
    println!("  Status: {:?}", order.status);
    println!("  Filled: {:.1}%", order.fill_percent());
    println!("  Remaining: {:.4}\n", order.remaining());

    // Full fill
    order.fill(0.7);
    println!("After full fill:");
    println!("  Status: {:?}", order.status);
    println!("  Filled: {:.1}%", order.fill_percent());

    // Attempt to cancel filled order
    order.cancel();
    println!("\nAfter cancel attempt: {:?}", order.status);
}
```

## Comparing Enums

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    let side1 = OrderSide::Buy;
    let side2 = OrderSide::Buy;
    let side3 = OrderSide::Sell;

    println!("Buy == Buy: {}", side1 == side2);   // true
    println!("Buy == Sell: {}", side1 == side3);  // false
    println!("Buy != Sell: {}", side1 != side3);  // true

    // Use in conditions
    if side1 == OrderSide::Buy {
        println!("This is a buy order");
    }
}
```

## if let — Simplified match

When you only need to check one variant:

```rust
#[derive(Debug)]
enum OrderType {
    Market,
    Limit(f64),
    StopLoss(f64),
}

fn main() {
    let order = OrderType::Limit(42000.0);

    // Instead of full match
    if let OrderType::Limit(price) = order {
        println!("Limit order at price: ${:.2}", price);
    }

    // With else
    let order2 = OrderType::Market;

    if let OrderType::Limit(price) = order2 {
        println!("Limit order: {}", price);
    } else {
        println!("Not a limit order");
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `enum Name { A, B }` | Define an enumeration |
| `Name::A` | Access a variant |
| `match value { ... }` | Pattern matching |
| `enum Name { A(T) }` | Variant with data |
| `impl Name { }` | Methods for enum |
| `if let Pattern = value` | Simplified match |
| `#[derive(PartialEq)]` | Compare variants |

## Homework

1. Create an enum `TimeInForce` with variants: `GTC` (Good Till Cancel), `IOC` (Immediate Or Cancel), `FOK` (Fill Or Kill). Add a method `description()` that returns a description for each type.

2. Create an enum `OrderType` with variants:
   - `Market` — no data
   - `Limit(f64)` — with price
   - `StopMarket(f64)` — with trigger price
   - `StopLimit { trigger: f64, price: f64 }` — with two prices

   Write a function `describe_order` that prints the order description.

3. Implement a function `process_signals` that takes an array of `Signal::Buy` or `Signal::Sell` signals and counts how many of each type there are.

4. Create a complete `Position` struct with enum `PositionSide { Long, Short, Flat }`. Add methods:
   - `open(side, price, quantity)` — open a position
   - `close(price)` — close the position and return PnL
   - `is_open()` — check if position is open

## Navigation

[← Previous day](../069-review-week-9/en.md) | [Next day →](../071-enum-order-type/en.md)
