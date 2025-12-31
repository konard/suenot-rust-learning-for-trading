# Day 83: HashMap Methods — insert, get, remove

## Trading Analogy

Think of your portfolio as a table of assets. You can:
- **insert** — add a new asset to the portfolio or update the quantity of an existing one
- **get** — check how much of a specific asset you have
- **remove** — completely sell an asset and remove it from the portfolio

These are three basic operations you perform every day as a trader!

## The insert Method — Adding or Updating

### Basic Usage

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Add assets to portfolio
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);
    portfolio.insert(String::from("SOL"), 100.0);

    println!("Portfolio: {:?}", portfolio);
}
```

### insert Returns the Old Value

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // First BTC purchase
    let old_value = portfolio.insert(String::from("BTC"), 0.5);
    println!("Previous BTC value: {:?}", old_value); // None

    // Buy more BTC — insert overwrites!
    let old_value = portfolio.insert(String::from("BTC"), 1.5);
    println!("Previous BTC value: {:?}", old_value); // Some(0.5)

    println!("Current BTC quantity: {:?}", portfolio.get("BTC")); // Some(1.5)
}
```

**Important:** `insert` completely replaces the value, it doesn't add to it!

### Proper Position Accumulation

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();

    // Function to buy an asset
    fn buy_asset(portfolio: &mut HashMap<String, f64>, asset: &str, quantity: f64) {
        let current = portfolio.get(asset).copied().unwrap_or(0.0);
        portfolio.insert(String::from(asset), current + quantity);
    }

    buy_asset(&mut portfolio, "BTC", 0.5);
    buy_asset(&mut portfolio, "BTC", 0.3);
    buy_asset(&mut portfolio, "ETH", 5.0);

    println!("BTC: {}", portfolio.get("BTC").unwrap()); // 0.8
    println!("ETH: {}", portfolio.get("ETH").unwrap()); // 5.0
}
```

## The get Method — Retrieving Values

### Basic Usage

```rust
use std::collections::HashMap;

fn main() {
    let mut prices: HashMap<String, f64> = HashMap::new();
    prices.insert(String::from("BTC"), 42000.0);
    prices.insert(String::from("ETH"), 2200.0);

    // get returns Option<&V>
    let btc_price = prices.get("BTC");
    println!("BTC price: {:?}", btc_price); // Some(42000.0)

    let unknown = prices.get("UNKNOWN");
    println!("Unknown asset: {:?}", unknown); // None
}
```

### Handling Option

```rust
use std::collections::HashMap;

fn main() {
    let mut prices: HashMap<String, f64> = HashMap::new();
    prices.insert(String::from("BTC"), 42000.0);

    // Method 1: match
    match prices.get("BTC") {
        Some(price) => println!("BTC costs ${}", price),
        None => println!("BTC not found"),
    }

    // Method 2: if let
    if let Some(price) = prices.get("ETH") {
        println!("ETH costs ${}", price);
    } else {
        println!("ETH not found in price list");
    }

    // Method 3: unwrap_or
    let sol_price = prices.get("SOL").unwrap_or(&0.0);
    println!("SOL costs ${}", sol_price);

    // Method 4: copied() + unwrap_or to copy the value
    let ada_price: f64 = prices.get("ADA").copied().unwrap_or(0.0);
    println!("ADA costs ${}", ada_price);
}
```

### get vs get_mut

```rust
use std::collections::HashMap;

fn main() {
    let mut balances: HashMap<String, f64> = HashMap::new();
    balances.insert(String::from("USD"), 10000.0);
    balances.insert(String::from("BTC"), 0.5);

    // get — read-only
    let usd = balances.get("USD");
    println!("USD balance: {:?}", usd);

    // get_mut — for modifying values
    if let Some(btc_balance) = balances.get_mut("BTC") {
        *btc_balance += 0.1; // Bought more BTC
    }

    println!("BTC after purchase: {:?}", balances.get("BTC")); // Some(0.6)
}
```

## The remove Method — Deleting Elements

### Basic Usage

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);
    portfolio.insert(String::from("DOGE"), 1000.0);

    println!("Before sale: {:?}", portfolio);

    // Sell all DOGE
    let removed = portfolio.remove("DOGE");
    println!("Sold DOGE: {:?}", removed); // Some(1000.0)

    println!("After sale: {:?}", portfolio);

    // Attempt to remove non-existent asset
    let not_found = portfolio.remove("SHIB");
    println!("SHIB not found: {:?}", not_found); // None
}
```

### Complete Asset Sale

```rust
use std::collections::HashMap;

fn main() {
    let mut portfolio: HashMap<String, f64> = HashMap::new();
    portfolio.insert(String::from("BTC"), 0.5);
    portfolio.insert(String::from("ETH"), 10.0);

    fn sell_all(portfolio: &mut HashMap<String, f64>, asset: &str) -> Option<f64> {
        portfolio.remove(asset)
    }

    match sell_all(&mut portfolio, "ETH") {
        Some(quantity) => println!("Sold {} ETH", quantity),
        None => println!("ETH not in portfolio"),
    }

    println!("Portfolio after sale: {:?}", portfolio);
}
```

## Practical Example: Portfolio Management System

```rust
use std::collections::HashMap;

struct Portfolio {
    holdings: HashMap<String, f64>,
    prices: HashMap<String, f64>,
}

impl Portfolio {
    fn new() -> Self {
        Portfolio {
            holdings: HashMap::new(),
            prices: HashMap::new(),
        }
    }

    fn update_price(&mut self, asset: &str, price: f64) {
        self.prices.insert(String::from(asset), price);
    }

    fn buy(&mut self, asset: &str, quantity: f64) {
        let current = self.holdings.get(asset).copied().unwrap_or(0.0);
        self.holdings.insert(String::from(asset), current + quantity);
        println!("Bought {} {}", quantity, asset);
    }

    fn sell(&mut self, asset: &str, quantity: f64) -> Result<f64, String> {
        match self.holdings.get_mut(asset) {
            Some(holding) if *holding >= quantity => {
                *holding -= quantity;
                if *holding == 0.0 {
                    self.holdings.remove(asset);
                }
                Ok(quantity)
            }
            Some(holding) => Err(format!(
                "Insufficient {}: have {}, need {}",
                asset, holding, quantity
            )),
            None => Err(format!("{} not in portfolio", asset)),
        }
    }

    fn get_holding(&self, asset: &str) -> f64 {
        self.holdings.get(asset).copied().unwrap_or(0.0)
    }

    fn get_value(&self, asset: &str) -> f64 {
        let quantity = self.get_holding(asset);
        let price = self.prices.get(asset).copied().unwrap_or(0.0);
        quantity * price
    }

    fn total_value(&self) -> f64 {
        self.holdings.iter().map(|(asset, qty)| {
            let price = self.prices.get(asset).copied().unwrap_or(0.0);
            qty * price
        }).sum()
    }

    fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║           PORTFOLIO                   ║");
        println!("╠═══════════════════════════════════════╣");

        for (asset, quantity) in &self.holdings {
            let price = self.prices.get(asset).copied().unwrap_or(0.0);
            let value = quantity * price;
            println!("║ {:6} {:>10.4} @ ${:>10.2} = ${:>10.2} ║",
                     asset, quantity, price, value);
        }

        println!("╠═══════════════════════════════════════╣");
        println!("║ TOTAL:                     ${:>10.2} ║", self.total_value());
        println!("╚═══════════════════════════════════════╝");
    }
}

fn main() {
    let mut portfolio = Portfolio::new();

    // Update prices
    portfolio.update_price("BTC", 42000.0);
    portfolio.update_price("ETH", 2200.0);
    portfolio.update_price("SOL", 95.0);

    // Buy assets
    portfolio.buy("BTC", 0.5);
    portfolio.buy("ETH", 5.0);
    portfolio.buy("SOL", 50.0);
    portfolio.buy("BTC", 0.3); // Buy more BTC

    portfolio.print_summary();

    // Sell part of ETH
    match portfolio.sell("ETH", 2.0) {
        Ok(qty) => println!("\nSuccessfully sold {} ETH", qty),
        Err(e) => println!("\nError: {}", e),
    }

    portfolio.print_summary();
}
```

## Practical Example: Order Book

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    price: f64,
    quantity: f64,
    side: String,
}

struct OrderBook {
    orders: HashMap<u64, Order>,
    next_id: u64,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            orders: HashMap::new(),
            next_id: 1,
        }
    }

    fn place_order(&mut self, price: f64, quantity: f64, side: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let order = Order {
            id,
            price,
            quantity,
            side: String::from(side),
        };

        self.orders.insert(id, order);
        println!("Created order #{}: {} {} @ ${}", id, side, quantity, price);
        id
    }

    fn get_order(&self, id: u64) -> Option<&Order> {
        self.orders.get(&id)
    }

    fn cancel_order(&mut self, id: u64) -> Option<Order> {
        let removed = self.orders.remove(&id);
        if removed.is_some() {
            println!("Order #{} cancelled", id);
        }
        removed
    }

    fn modify_order(&mut self, id: u64, new_price: f64, new_quantity: f64) -> bool {
        if let Some(order) = self.orders.get_mut(&id) {
            order.price = new_price;
            order.quantity = new_quantity;
            println!("Order #{} modified: {} @ ${}", id, new_quantity, new_price);
            true
        } else {
            false
        }
    }

    fn print_orders(&self) {
        println!("\n=== Active Orders ===");
        for (id, order) in &self.orders {
            println!("#{}: {} {} @ ${}", id, order.side, order.quantity, order.price);
        }
        println!("Total orders: {}", self.orders.len());
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Place orders
    let order1 = book.place_order(42000.0, 0.5, "BUY");
    let order2 = book.place_order(42500.0, 0.3, "SELL");
    let order3 = book.place_order(41500.0, 1.0, "BUY");

    book.print_orders();

    // Check order
    if let Some(order) = book.get_order(order1) {
        println!("\nOrder #{}: {:?}", order1, order);
    }

    // Modify order
    book.modify_order(order2, 42800.0, 0.5);

    // Cancel order
    book.cancel_order(order3);

    book.print_orders();
}
```

## Useful HashMap Methods

```rust
use std::collections::HashMap;

fn main() {
    let mut data: HashMap<String, f64> = HashMap::new();
    data.insert(String::from("BTC"), 42000.0);
    data.insert(String::from("ETH"), 2200.0);
    data.insert(String::from("SOL"), 95.0);

    // contains_key — check if key exists
    if data.contains_key("BTC") {
        println!("BTC is in the data");
    }

    // len — number of elements
    println!("Number of assets: {}", data.len());

    // is_empty — check if empty
    println!("Empty: {}", data.is_empty());

    // keys — iterator over keys
    print!("Assets: ");
    for key in data.keys() {
        print!("{} ", key);
    }
    println!();

    // values — iterator over values
    let total: f64 = data.values().sum();
    println!("Sum of all prices: ${}", total);

    // iter — iterator over pairs
    for (asset, price) in data.iter() {
        println!("{}: ${}", asset, price);
    }

    // clear — clear the map
    data.clear();
    println!("After clearing: {} elements", data.len());
}
```

## What We Learned

| Method | Returns | Description |
|--------|---------|-------------|
| `insert(k, v)` | `Option<V>` | Inserts, returns old value |
| `get(&k)` | `Option<&V>` | Reference to value (read-only) |
| `get_mut(&k)` | `Option<&mut V>` | Mutable reference (for modification) |
| `remove(&k)` | `Option<V>` | Removes and returns value |
| `contains_key(&k)` | `bool` | Check if key exists |
| `len()` | `usize` | Number of elements |

## Exercises

1. **Price Tracker:** Create a structure that stores price history for each asset and allows retrieving the latest price

2. **Balance Cache:** Implement a balance cache for an exchange with `deposit`, `withdraw`, `get_balance` methods

3. **Trade Counter:** Create a system that counts the number of trades per asset

## Homework

1. Write a function `merge_portfolios(p1: &HashMap<String, f64>, p2: &HashMap<String, f64>) -> HashMap<String, f64>` that combines two portfolios

2. Create a `RiskManager` struct that tracks positions and prevents exceeding limits per asset

3. Implement an `OrderMatcher` that matches buy and sell orders by price

4. Write a function to find arbitrage opportunities between two HashMaps with prices on different exchanges

## Navigation

[← Previous day](../082-hashmap-portfolio-asset/en.md) | [Next day →](../084-entry-api-update-insert/en.md)
