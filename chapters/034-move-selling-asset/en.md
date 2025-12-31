# Day 34: Move — Selling the Asset

## Trading Analogy

Imagine: you hold 1 BTC in your wallet. You decide to sell — you transfer it to the buyer. **Now you no longer have that bitcoin**. It moved to the new owner. You can't spend it again — that's physically impossible.

In Rust, variables work the same way. When you pass a value somewhere, it **moves**, and the original variable becomes invalid. This is called **move semantics**.

## Why Does This Matter?

In trading, double spending is a catastrophe. If you could spend the same BTC twice, the entire system would collapse.

Rust protects against similar errors in code. If data has "moved", you won't be able to use the old variable — the compiler won't allow it.

## Move in Action

```rust
fn main() {
    let portfolio = String::from("BTC: 1.5, ETH: 10.0");

    // Sell (transfer) the portfolio to a new owner
    let new_owner = portfolio;

    // ERROR! portfolio no longer exists
    // println!("My portfolio: {}", portfolio);

    // Now only new_owner owns the data
    println!("New owner's portfolio: {}", new_owner);
}
```

The compiler will say:
```
error[E0382]: borrow of moved value: `portfolio`
 --> src/main.rs:8:35
  |
2 |     let portfolio = String::from("BTC: 1.5, ETH: 10.0");
  |         --------- move occurs because `portfolio` has type `String`
3 |     let new_owner = portfolio;
  |                     --------- value moved here
...
8 |     println!("My portfolio: {}", portfolio);
  |                                  ^^^^^^^^^ value borrowed here after move
```

**Analogy:** This is like trying to show your friend your bitcoin after you sold it. You don't have it anymore!

## Move When Passing to a Function

When you pass a value to a function, it also moves:

```rust
fn sell_asset(asset: String) {
    println!("Selling asset: {}", asset);
    // asset is destroyed at the end of the function
}

fn main() {
    let my_btc = String::from("1.0 BTC");

    sell_asset(my_btc);  // my_btc moved into the function

    // ERROR! my_btc no longer exists
    // println!("I have: {}", my_btc);
}
```

**Analogy:** You gave BTC to a broker to sell. The broker sold it and closed the deal. The bitcoin is gone forever.

## Practical Example: Passing an Order

```rust
struct Order {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

fn execute_order(order: Order) {
    println!("=== Order Execution ===");
    println!("Symbol: {}", order.symbol);
    println!("Side: {}", order.side);
    println!("Quantity: {}", order.quantity);
    println!("Price: {} USDT", order.price);
    println!("Order executed!");
    // order is destroyed here
}

fn main() {
    let buy_order = Order {
        symbol: String::from("BTCUSDT"),
        side: String::from("BUY"),
        quantity: 0.5,
        price: 42000.0,
    };

    // Pass the order for execution
    execute_order(buy_order);

    // ERROR! buy_order no longer exists
    // println!("Status: {}", buy_order.side);
}
```

## Move and Returning Values

A function can return ownership back:

```rust
struct Position {
    symbol: String,
    size: f64,
    entry_price: f64,
}

fn open_position(symbol: String, size: f64, price: f64) -> Position {
    Position {
        symbol,  // symbol moves into the struct
        size,
        entry_price: price,
    }
}

fn close_position(position: Position) -> f64 {
    let pnl = (42500.0 - position.entry_price) * position.size;
    println!("Closing position on {}", position.symbol);
    println!("PnL: {} USDT", pnl);
    pnl  // Return PnL, position is destroyed
}

fn main() {
    let ticker = String::from("BTCUSDT");

    // ticker moves into the function, Position is returned
    let position = open_position(ticker, 0.5, 42000.0);

    // position moves into close_position
    let profit = close_position(position);

    println!("Final profit: {} USDT", profit);
}
```

## Why Does String Move but Numbers Don't?

```rust
fn main() {
    // Numbers are COPIED, not moved
    let price = 42000.0;
    let copy_price = price;

    println!("Original: {}", price);      // Works!
    println!("Copy: {}", copy_price);     // Works!

    // String is MOVED
    let ticker = String::from("BTC");
    let moved_ticker = ticker;

    // println!("{}", ticker);  // ERROR!
    println!("{}", moved_ticker);  // Works
}
```

Simple types (numbers, bool, char) implement **Copy** — they're small and quick to copy. It's like cash — you can easily count out the same amount.

**String** and other complex types live on the heap. Copying would be expensive, so they move instead. It's like transferring cryptocurrency — the asset physically moves to the new owner.

## Move with Collections

```rust
fn main() {
    let mut portfolio = Vec::new();

    let btc = String::from("BTC");
    let eth = String::from("ETH");

    portfolio.push(btc);  // btc moves into the vector
    portfolio.push(eth);  // eth moves into the vector

    // ERROR! btc and eth no longer exist
    // println!("{}, {}", btc, eth);

    // But we can get a reference from the vector
    println!("First asset: {}", portfolio[0]);
    println!("Second asset: {}", portfolio[1]);
}
```

## Pattern: Taking and Returning Ownership

```rust
struct Portfolio {
    assets: Vec<String>,
    total_value: f64,
}

fn add_asset(mut portfolio: Portfolio, asset: String, value: f64) -> Portfolio {
    println!("Adding {} worth {} USDT", asset, value);
    portfolio.assets.push(asset);
    portfolio.total_value += value;
    portfolio  // Return ownership
}

fn main() {
    let portfolio = Portfolio {
        assets: Vec::new(),
        total_value: 0.0,
    };

    // Each time we pass and receive back
    let portfolio = add_asset(portfolio, String::from("BTC"), 42000.0);
    let portfolio = add_asset(portfolio, String::from("ETH"), 2500.0);
    let portfolio = add_asset(portfolio, String::from("SOL"), 100.0);

    println!("\n=== Final Portfolio ===");
    for asset in &portfolio.assets {
        println!("- {}", asset);
    }
    println!("Total value: {} USDT", portfolio.total_value);
}
```

## Trading Session Simulation

```rust
struct Trade {
    id: u32,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
}

struct TradingSession {
    trades: Vec<Trade>,
    balance: f64,
}

fn execute_trade(mut session: TradingSession, trade: Trade) -> TradingSession {
    let trade_value = trade.quantity * trade.price;

    if trade.side == "BUY" {
        session.balance -= trade_value;
        println!("BOUGHT {} {} at {} = -{} USDT",
            trade.quantity, trade.symbol, trade.price, trade_value);
    } else {
        session.balance += trade_value;
        println!("SOLD {} {} at {} = +{} USDT",
            trade.quantity, trade.symbol, trade.price, trade_value);
    }

    session.trades.push(trade);  // trade moves into the vector
    session
}

fn close_session(session: TradingSession) {
    println!("\n=== Session Close ===");
    println!("Total trades: {}", session.trades.len());
    println!("Final balance: {} USDT", session.balance);

    println!("\nTrade history:");
    for trade in &session.trades {
        println!("  #{}: {} {} {} @ {}",
            trade.id, trade.side, trade.quantity, trade.symbol, trade.price);
    }
    // session is destroyed here
}

fn main() {
    let session = TradingSession {
        trades: Vec::new(),
        balance: 100000.0,
    };

    println!("=== Trading Session Start ===");
    println!("Initial balance: {} USDT\n", session.balance);

    let trade1 = Trade {
        id: 1,
        symbol: String::from("BTCUSDT"),
        side: String::from("BUY"),
        quantity: 1.0,
        price: 42000.0,
    };

    let trade2 = Trade {
        id: 2,
        symbol: String::from("ETHUSDT"),
        side: String::from("BUY"),
        quantity: 10.0,
        price: 2500.0,
    };

    let trade3 = Trade {
        id: 3,
        symbol: String::from("BTCUSDT"),
        side: String::from("SELL"),
        quantity: 0.5,
        price: 43000.0,
    };

    let session = execute_trade(session, trade1);
    let session = execute_trade(session, trade2);
    let session = execute_trade(session, trade3);

    close_session(session);
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Move | Transfer of data ownership |
| After move | Original variable becomes invalid |
| Protection | Compiler prevents using moved values |
| Copy types | Numbers, bool, char are copied, not moved |
| Return | Functions can return ownership |

## Homework

1. **Order System**: Create an `Order` struct and a function `submit_order(order: Order)` that "submits" the order. Verify that you can't use the order after submission.

2. **Portfolio Manager**: Implement:
   - `Asset` struct with fields `symbol: String` and `quantity: f64`
   - `Portfolio` struct with `Vec<Asset>` and `total_value: f64`
   - Function `add_to_portfolio(portfolio: Portfolio, asset: Asset) -> Portfolio`
   - Function `remove_from_portfolio(portfolio: Portfolio, index: usize) -> (Portfolio, Asset)`

3. **Trading Engine**: Create a simulation where:
   - There are 3 traders, each with their own balance
   - One asset can only belong to one trader
   - Implement function `transfer_asset(from: Trader, to: Trader, asset: Asset) -> (Trader, Trader)`

4. **Error Experiment**: Write code that:
   - Attempts to use a variable after move
   - Read the compiler error message
   - Fix the code in three different ways

## Navigation

[← Previous day](../033-ownership-who-holds/en.md) | [Next day →](../035-clone-copying-portfolio/en.md)
