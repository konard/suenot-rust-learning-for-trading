# Day 349: Code Formatting: rustfmt

## Trading Analogy

Imagine a trading floor at a major hedge fund. Each trader keeps their own records of trades, but if everyone uses their own format — one uses commas for thousands separators, another uses spaces, a third uses nothing — analyzing the overall position becomes chaotic.

That's why professional trading organizations have **documentation standards**:
- Unified price format (e.g., always 2 decimal places)
- Standard ticker symbols (BTCUSDT, not btc/usdt or Bitcoin-USDT)
- Consistent report formatting

**rustfmt** is the automatic "standardizer" for Rust code. It ensures consistent formatting style across the entire project, just like documentation standards ensure uniformity in trading reports.

| Trading | Programming |
|---------|-------------|
| Price format standard | Indentation style |
| Ticker format | Variable naming |
| Report template | Code structure |
| Document audit | Code review |

## What is rustfmt?

**rustfmt** is the official Rust code formatting tool. It automatically formats code according to the Rust Style Guide recommendations.

### Installation and Verification

rustfmt is typically installed along with Rust:

```bash
# Check installation
rustfmt --version

# If not installed
rustup component add rustfmt
```

## Basic Usage

### Formatting a File

```bash
# Format a single file
rustfmt src/main.rs

# Format with change output
rustfmt --check src/main.rs

# Format entire project via Cargo
cargo fmt

# Check without modifying files
cargo fmt --check
```

### Example: Before and After Formatting

Consider a trading module before formatting:

```rust
// Before formatting — chaotic style
use std::collections::HashMap;use std::time::{SystemTime,UNIX_EPOCH};

#[derive(Debug,Clone)]
struct Order{symbol:String,side:OrderSide,price:f64,quantity:f64,timestamp:u64}

#[derive(Debug,Clone,Copy)]
enum OrderSide{Buy,Sell}

impl Order{
fn new(symbol:&str,side:OrderSide,price:f64,quantity:f64)->Self{
let timestamp=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
Order{symbol:symbol.to_string(),side,price,quantity,timestamp}
}

fn total_value(&self)->f64{self.price*self.quantity}

fn is_buy(&self)->bool{matches!(self.side,OrderSide::Buy)}
}

fn calculate_portfolio_value(orders:&[Order],prices:&HashMap<String,f64>)->f64{
orders.iter().filter(|o|o.is_buy()).map(|o|{
let current_price=prices.get(&o.symbol).unwrap_or(&o.price);
o.quantity*current_price}).sum()
}

fn main(){
let mut prices=HashMap::new();
prices.insert("BTCUSDT".to_string(),50000.0);
prices.insert("ETHUSDT".to_string(),3000.0);

let orders=vec![
Order::new("BTCUSDT",OrderSide::Buy,49000.0,0.5),
Order::new("ETHUSDT",OrderSide::Buy,2900.0,2.0),
Order::new("BTCUSDT",OrderSide::Sell,51000.0,0.2),
];

let value=calculate_portfolio_value(&orders,&prices);
println!("Portfolio value: ${:.2}",value);
}
```

After running `cargo fmt`:

```rust
// After formatting — clean, readable code
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

impl Order {
    fn new(symbol: &str, side: OrderSide, price: f64, quantity: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Order {
            symbol: symbol.to_string(),
            side,
            price,
            quantity,
            timestamp,
        }
    }

    fn total_value(&self) -> f64 {
        self.price * self.quantity
    }

    fn is_buy(&self) -> bool {
        matches!(self.side, OrderSide::Buy)
    }
}

fn calculate_portfolio_value(orders: &[Order], prices: &HashMap<String, f64>) -> f64 {
    orders
        .iter()
        .filter(|o| o.is_buy())
        .map(|o| {
            let current_price = prices.get(&o.symbol).unwrap_or(&o.price);
            o.quantity * current_price
        })
        .sum()
}

fn main() {
    let mut prices = HashMap::new();
    prices.insert("BTCUSDT".to_string(), 50000.0);
    prices.insert("ETHUSDT".to_string(), 3000.0);

    let orders = vec![
        Order::new("BTCUSDT", OrderSide::Buy, 49000.0, 0.5),
        Order::new("ETHUSDT", OrderSide::Buy, 2900.0, 2.0),
        Order::new("BTCUSDT", OrderSide::Sell, 51000.0, 0.2),
    ];

    let value = calculate_portfolio_value(&orders, &prices);
    println!("Portfolio value: ${:.2}", value);
}
```

## Configuring rustfmt

### The rustfmt.toml File

Create a `rustfmt.toml` file in your project root to configure formatting:

```toml
# rustfmt.toml — configuration for trading system

# Maximum line width
max_width = 100

# Use tabs instead of spaces
hard_tabs = false

# Indentation size
tab_spaces = 4

# Indentation style for method chains
chain_width = 60

# Import formatting
imports_granularity = "Module"
group_imports = "StdExternalCrate"

# Line breaks in function declarations
fn_args_layout = "Tall"

# Brace style
brace_style = "SameLineWhere"

# Use field init shorthand
use_field_init_shorthand = true

# Use try! or ?
use_try_shorthand = true
```

### Example with Configuration

```rust
// With imports_granularity = "Module" and group_imports = "StdExternalCrate"
// Imports are automatically grouped and sorted

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::error::TradingError;
use crate::models::{Order, Position, Trade};

/// Trading engine with automatic formatting
#[derive(Debug)]
pub struct TradingEngine {
    orders: Arc<RwLock<HashMap<String, Order>>>,
    positions: Arc<RwLock<HashMap<String, Position>>>,
    trade_history: Vec<Trade>,
    started_at: Instant,
}

impl TradingEngine {
    /// Creates a new trading engine
    pub fn new() -> Self {
        // use_field_init_shorthand = true allows writing like this:
        Self {
            orders: Arc::new(RwLock::new(HashMap::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            trade_history: Vec::new(),
            started_at: Instant::now(),
        }
    }

    /// Places a new order
    pub fn place_order(
        &mut self,
        symbol: &str,
        side: OrderSide,
        price: f64,
        quantity: f64,
    ) -> Result<String, TradingError> {
        // fn_args_layout = "Tall" — arguments on separate lines
        // when they don't fit on one line

        let order_id = self.generate_order_id();

        let order = Order {
            id: order_id.clone(),
            symbol: symbol.to_string(),
            side,
            price,
            quantity,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
        };

        self.orders
            .write()
            .map_err(|_| TradingError::LockError)?
            .insert(order_id.clone(), order);

        Ok(order_id)
    }

    /// Gets all open positions
    pub fn get_open_positions(&self) -> Result<Vec<Position>, TradingError> {
        // chain_width = 60 determines when to break method chains
        let positions = self
            .positions
            .read()
            .map_err(|_| TradingError::LockError)?;

        Ok(positions
            .values()
            .filter(|p| p.quantity != 0.0)
            .cloned()
            .collect())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}
```

## Workflow Integration

### Pre-commit Hook

Create a file `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Pre-commit hook for format checking

echo "Checking code formatting..."

# Check formatting
if ! cargo fmt --check; then
    echo "Error: code is not formatted!"
    echo "Run 'cargo fmt' before committing."
    exit 1
fi

echo "Formatting OK!"
```

### CI/CD Integration

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  format:
    name: Check formatting
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --check

  build:
    name: Build and test
    runs-on: ubuntu-latest
    needs: format
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: cargo build --release

      - name: Test
        run: cargo test
```

## Advanced Features

### The #[rustfmt::skip] Attribute

Sometimes auto-formatting reduces readability. Use `#[rustfmt::skip]`:

```rust
use std::collections::HashMap;

/// Exchange fee table
/// Format: (maker_fee, taker_fee)
#[rustfmt::skip]
const EXCHANGE_FEES: &[(&str, (f64, f64))] = &[
    ("Binance",  (0.001, 0.001)),
    ("Coinbase", (0.004, 0.006)),
    ("Kraken",   (0.002, 0.005)),
    ("Bybit",    (0.001, 0.001)),
    ("OKX",      (0.002, 0.005)),
];

/// Asset correlation matrix — formatting would break the structure
#[rustfmt::skip]
const CORRELATION_MATRIX: [[f64; 4]; 4] = [
    // BTC    ETH    SOL    ADA
    [ 1.00,  0.85,  0.72,  0.68],  // BTC
    [ 0.85,  1.00,  0.78,  0.75],  // ETH
    [ 0.72,  0.78,  1.00,  0.82],  // SOL
    [ 0.68,  0.75,  0.82,  1.00],  // ADA
];

/// Support and resistance levels
#[rustfmt::skip]
fn get_price_levels(symbol: &str) -> Vec<f64> {
    match symbol {
        "BTCUSDT" => vec![
            45000.0, 47500.0, 50000.0,  // Support
            52500.0, 55000.0, 60000.0,  // Resistance
        ],
        "ETHUSDT" => vec![
            2800.0, 3000.0, 3200.0,
            3500.0, 3800.0, 4000.0,
        ],
        _ => vec![],
    }
}

fn main() {
    // Display fees in a nice format
    println!("=== Exchange Fees ===");
    for (exchange, (maker, taker)) in EXCHANGE_FEES {
        println!("{:12} Maker: {:.2}% Taker: {:.2}%", exchange, maker * 100.0, taker * 100.0);
    }

    println!("\n=== Correlation Matrix ===");
    let assets = ["BTC", "ETH", "SOL", "ADA"];
    print!("     ");
    for asset in &assets {
        print!("{:>6}", asset);
    }
    println!();

    for (i, row) in CORRELATION_MATRIX.iter().enumerate() {
        print!("{:>4} ", assets[i]);
        for val in row {
            print!("{:>6.2}", val);
        }
        println!();
    }
}
```

### Formatting Macros

```rust
/// Macro for creating an order with formatting
macro_rules! order {
    ($symbol:expr, $side:ident, $price:expr, $qty:expr) => {
        Order {
            symbol: $symbol.to_string(),
            side: OrderSide::$side,
            price: $price,
            quantity: $qty,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    };
}

/// Macro for logging trades
macro_rules! log_trade {
    ($($arg:tt)*) => {
        println!(
            "[{}] {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            format!($($arg)*)
        );
    };
}

#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

fn main() {
    // Using macros with nice formatting
    let orders = vec![
        order!("BTCUSDT", Buy, 50000.0, 0.1),
        order!("ETHUSDT", Sell, 3000.0, 1.0),
        order!("SOLUSDT", Buy, 100.0, 10.0),
    ];

    for order in &orders {
        println!("{:?}", order);
    }
}
```

## Comparison with Other Formatters

| Feature | rustfmt | prettier (JS) | black (Python) |
|---------|---------|---------------|----------------|
| Official | Yes | No | No |
| Configuration | rustfmt.toml | .prettierrc | pyproject.toml |
| Style | Rust Style Guide | Custom | PEP 8 |
| IDE Integration | Excellent | Excellent | Good |
| Speed | Fast | Fast | Medium |

## What We Learned

| Concept | Description |
|---------|-------------|
| **rustfmt** | Official Rust code formatter |
| **cargo fmt** | Command to format entire project |
| **rustfmt.toml** | Formatting configuration file |
| **#[rustfmt::skip]** | Attribute to skip formatting |
| **--check** | Flag to check without modifying files |
| **Pre-commit hook** | Automatic check before commit |

## Practical Exercises

1. **Project Setup**: Configure rustfmt for your trading project:
   - Create `rustfmt.toml` with optimal settings
   - Set up import grouping
   - Set maximum line width to 100 characters
   - Add a pre-commit hook

2. **Formatting Existing Code**: Take unformatted code:
   - Run `cargo fmt --check` for analysis
   - Apply formatting
   - Compare the difference with `git diff`
   - Verify the code is more readable

3. **Using #[rustfmt::skip]**: Create a module with:
   - A constants table where formatting matters
   - A data matrix for analysis
   - Document reasons for using skip

4. **CI Integration**: Add formatting checks:
   - Create a GitHub Actions workflow
   - Add a formatting check step
   - Configure fail on unformatted code

## Homework

1. **Trading Project Standardization**: Create a complete formatting configuration:
   - Configure all `rustfmt.toml` parameters
   - Add examples with `#[rustfmt::skip]` for tables
   - Create an auto-formatting script
   - Document the chosen settings

2. **Automation**: Set up full automation:
   - Pre-commit hook for checking
   - Pre-push hook for formatting
   - VS Code integration (format on save)
   - CI/CD pipeline with checks

3. **Comparative Analysis**: Compare code readability:
   - Take a complex trading module
   - Measure time to understand code before formatting
   - Measure time after formatting
   - Prepare a report with conclusions

4. **Team Standards**: Create a team guide:
   - Describe all formatting settings
   - Explain when to use skip
   - Add examples of good and bad code
   - Create a code review checklist

## Navigation

[← Previous day](../326-async-vs-threading/en.md) | [Next day →](../350-*/en.md)
