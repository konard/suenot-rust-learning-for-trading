# Day 19: loop — Endless Price Monitoring

## Trading Analogy

Imagine a trading bot that **continuously** monitors the market. It doesn't stop after one check — it runs 24/7, constantly tracking prices, volumes, and signals. This is exactly how `loop` works in Rust — an infinite loop that runs until explicitly stopped.

In real trading, this includes:
- Real-time price monitoring
- Waiting for trade entry signals
- Maintaining WebSocket connections to exchanges
- Continuous stop-loss checking

## Basic loop Syntax

```rust
fn main() {
    let mut price = 42000.0;
    let mut iteration = 0;

    loop {
        iteration += 1;
        println!("Check #{}: BTC = ${:.2}", iteration, price);

        // Simulate price change
        price += 100.0;

        if iteration >= 5 {
            println!("Monitoring complete");
            break;  // Exit the loop
        }
    }

    println!("Final price: ${:.2}", price);
}
```

**Key point:** `loop` runs forever until it encounters `break`.

## break — Exiting the Loop

```rust
fn main() {
    let target_price = 43000.0;
    let mut current_price = 42000.0;
    let mut checks = 0;

    loop {
        checks += 1;
        current_price += 150.0;  // Simulate price increase

        println!("Check {}: ${:.2}", checks, current_price);

        if current_price >= target_price {
            println!("Target price reached!");
            break;
        }

        if checks > 100 {
            println!("Check limit exceeded");
            break;
        }
    }

    println!("Total checks: {}", checks);
}
```

## continue — Skipping an Iteration

```rust
fn main() {
    let prices = [42000.0, 0.0, 42500.0, -100.0, 43000.0, 42800.0];
    let mut index = 0;
    let mut valid_count = 0;
    let mut sum = 0.0;

    loop {
        if index >= prices.len() {
            break;
        }

        let price = prices[index];
        index += 1;

        // Skip invalid prices
        if price <= 0.0 {
            println!("Skipping invalid price: {}", price);
            continue;
        }

        valid_count += 1;
        sum += price;
        println!("Valid price #{}: ${:.2}", valid_count, price);
    }

    if valid_count > 0 {
        println!("Average price: ${:.2}", sum / valid_count as f64);
    }
}
```

## loop with Return Value

A unique Rust feature — `loop` can return a value through `break`:

```rust
fn main() {
    let mut price = 42000.0;
    let buy_threshold = 41000.0;
    let sell_threshold = 44000.0;

    let action = loop {
        // Simulate price change
        price += (price * 0.01) * if price > 43000.0 { -1.0 } else { 1.0 };

        println!("Current price: ${:.2}", price);

        if price <= buy_threshold {
            break "BUY";  // Return value
        }

        if price >= sell_threshold {
            break "SELL";
        }

        // Protection against infinite loop in example
        if price > 50000.0 || price < 35000.0 {
            break "HOLD";
        }
    };

    println!("Recommendation: {}", action);
}
```

## Nested Loops and Labels

```rust
fn main() {
    let exchanges = ["Binance", "Coinbase", "Kraken"];
    let assets = ["BTC", "ETH", "SOL"];

    'exchange_loop: loop {
        for exchange in &exchanges {
            for asset in &assets {
                let price = get_mock_price(exchange, asset);

                println!("{} on {}: ${:.2}", asset, exchange, price);

                if price > 50000.0 {
                    println!("Anomalous price found! Stopping.");
                    break 'exchange_loop;  // Exit outer loop
                }
            }
        }
        break;  // Exit after one iteration for the example
    }

    println!("Scanning complete");
}

fn get_mock_price(exchange: &str, asset: &str) -> f64 {
    // Mock prices
    match (exchange, asset) {
        ("Binance", "BTC") => 42150.0,
        ("Coinbase", "BTC") => 42200.0,
        ("Kraken", "BTC") => 55000.0,  // Anomaly
        (_, "ETH") => 2250.0,
        (_, "SOL") => 98.0,
        _ => 0.0,
    }
}
```

## Practical Example: Trading Bot Monitor

```rust
fn main() {
    println!("╔════════════════════════════════════╗");
    println!("║     PRICE MONITOR STARTED          ║");
    println!("╚════════════════════════════════════╝");

    let mut btc_price = 42000.0;
    let mut eth_price = 2200.0;

    let stop_loss_btc = 40000.0;
    let take_profit_btc = 45000.0;

    let mut tick = 0;
    let max_ticks = 20;

    let result = loop {
        tick += 1;

        // Simulate price changes
        btc_price += simulate_price_change(btc_price);
        eth_price += simulate_price_change(eth_price);

        println!("\n[Tick {}]", tick);
        println!("  BTC: ${:.2}", btc_price);
        println!("  ETH: ${:.2}", eth_price);

        // Check stop-loss
        if btc_price <= stop_loss_btc {
            break TradeSignal::StopLoss(btc_price);
        }

        // Check take-profit
        if btc_price >= take_profit_btc {
            break TradeSignal::TakeProfit(btc_price);
        }

        // Protection against infinite loop
        if tick >= max_ticks {
            break TradeSignal::Timeout(btc_price);
        }
    };

    println!("\n╔════════════════════════════════════╗");
    match result {
        TradeSignal::StopLoss(price) => {
            println!("║  STOP LOSS TRIGGERED              ║");
            println!("║  Exit price: ${:.2}           ║", price);
        }
        TradeSignal::TakeProfit(price) => {
            println!("║  TAKE PROFIT REACHED              ║");
            println!("║  Exit price: ${:.2}           ║", price);
        }
        TradeSignal::Timeout(price) => {
            println!("║  MONITORING TIMEOUT               ║");
            println!("║  Current price: ${:.2}         ║", price);
        }
    }
    println!("╚════════════════════════════════════╝");
}

enum TradeSignal {
    StopLoss(f64),
    TakeProfit(f64),
    Timeout(f64),
}

fn simulate_price_change(price: f64) -> f64 {
    // Simple simulation: random change ±2%
    let change_percent = ((price as i64 % 7) as f64 - 3.0) / 100.0;
    price * change_percent
}
```

## Monitoring Multiple Assets

```rust
fn main() {
    let mut portfolio = Portfolio {
        btc: Asset { symbol: "BTC", price: 42000.0, quantity: 0.5 },
        eth: Asset { symbol: "ETH", price: 2200.0, quantity: 5.0 },
        sol: Asset { symbol: "SOL", price: 95.0, quantity: 100.0 },
    };

    let mut tick = 0;

    loop {
        tick += 1;

        // Update prices
        update_prices(&mut portfolio, tick);

        // Calculate total value
        let total_value = calculate_portfolio_value(&portfolio);

        println!("\n═══ Tick {} ═══", tick);
        print_portfolio(&portfolio);
        println!("Total Value: ${:.2}", total_value);

        // Check exit conditions
        if total_value < 30000.0 {
            println!("\n⚠️  Portfolio value dropped below $30,000!");
            break;
        }

        if total_value > 50000.0 {
            println!("\n✓ Target portfolio value reached!");
            break;
        }

        if tick >= 10 {
            println!("\n— Monitoring session ended —");
            break;
        }
    }
}

struct Asset {
    symbol: &'static str,
    price: f64,
    quantity: f64,
}

struct Portfolio {
    btc: Asset,
    eth: Asset,
    sol: Asset,
}

fn update_prices(portfolio: &mut Portfolio, tick: i32) {
    // Simulate price changes
    portfolio.btc.price *= 1.0 + ((tick % 5) as f64 - 2.0) / 100.0;
    portfolio.eth.price *= 1.0 + ((tick % 4) as f64 - 1.5) / 100.0;
    portfolio.sol.price *= 1.0 + ((tick % 6) as f64 - 2.5) / 100.0;
}

fn calculate_portfolio_value(portfolio: &Portfolio) -> f64 {
    portfolio.btc.price * portfolio.btc.quantity
        + portfolio.eth.price * portfolio.eth.quantity
        + portfolio.sol.price * portfolio.sol.quantity
}

fn print_portfolio(portfolio: &Portfolio) {
    println!(
        "  {} ${:.2} x {} = ${:.2}",
        portfolio.btc.symbol,
        portfolio.btc.price,
        portfolio.btc.quantity,
        portfolio.btc.price * portfolio.btc.quantity
    );
    println!(
        "  {} ${:.2} x {} = ${:.2}",
        portfolio.eth.symbol,
        portfolio.eth.price,
        portfolio.eth.quantity,
        portfolio.eth.price * portfolio.eth.quantity
    );
    println!(
        "  {} ${:.2} x {} = ${:.2}",
        portfolio.sol.symbol,
        portfolio.sol.price,
        portfolio.sol.quantity,
        portfolio.sol.price * portfolio.sol.quantity
    );
}
```

## Loop Usage Patterns

```rust
fn main() {
    // 1. Waiting for a condition
    let found_price = wait_for_price(42500.0);
    println!("Price reached: ${:.2}", found_price);

    // 2. Retry logic
    match fetch_with_retry(3) {
        Some(data) => println!("Data received: {}", data),
        None => println!("Failed to fetch data"),
    }

    // 3. Queue processing
    process_order_queue();
}

fn wait_for_price(target: f64) -> f64 {
    let mut current = 42000.0;

    loop {
        current += 50.0;  // Simulate growth

        if current >= target {
            break current;
        }
    }
}

fn fetch_with_retry(max_attempts: i32) -> Option<String> {
    let mut attempt = 0;

    loop {
        attempt += 1;
        println!("Attempt {} of {}", attempt, max_attempts);

        // Simulate success on 3rd attempt
        if attempt >= 3 {
            break Some(String::from("Market data"));
        }

        if attempt >= max_attempts {
            break None;
        }
    }
}

fn process_order_queue() {
    let mut orders = vec!["BUY BTC", "SELL ETH", "BUY SOL"];

    loop {
        if orders.is_empty() {
            println!("Order queue is empty");
            break;
        }

        let order = orders.remove(0);
        println!("Processing: {}", order);
    }
}
```

## What We Learned

| Construct | Description | Example |
|-----------|-------------|---------|
| `loop { }` | Infinite loop | Real-time monitoring |
| `break` | Exit loop | Condition triggered |
| `break value` | Exit with return | Search result |
| `continue` | Skip iteration | Data filtering |
| `'label: loop` | Labeled loop | Nested loops |

## Exercises

1. **Scalping Simulator:** Create a loop that buys when price drops by 0.5% and sells when it rises by 0.3%. Count the trades and calculate total PnL.

2. **Arbitrage Hunter:** Write a program that continuously compares prices on two exchanges and exits when it finds a difference greater than 1%.

3. **Position Accumulation:** Simulate DCA (Dollar Cost Averaging) — each iteration buy a fixed amount at the current price until you accumulate a target quantity.

4. **Alert System:** Create a monitor that tracks multiple price levels and outputs corresponding alerts when they're reached.

## Homework

1. Write a function `monitor_until_signal(prices: &[f64], buy_level: f64, sell_level: f64) -> TradeAction` that loops through prices and returns the first signal.

2. Create a monitoring system with multiple exit conditions: stop-loss, take-profit, timeout, and maximum drawdown.

3. Implement a market maker simulator that continuously updates bid/ask spread and exits at a certain profit level.

4. Write a trading bot with retry logic for exchange connection: the bot attempts to connect, waits on failure, and tries again (maximum N attempts).

## Navigation

[← Previous day](../018-if-else-trading-decisions/en.md) | [Next day →](../020-while-position-holding/en.md)
