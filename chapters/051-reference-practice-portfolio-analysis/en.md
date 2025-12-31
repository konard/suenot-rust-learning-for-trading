# Day 51: Reference Practice — Portfolio Analysis

## Trading Analogy

Imagine you have an investment portfolio worth millions of dollars. When you need a consultant's analysis, you don't physically hand over all your assets — you give them **view access**. The analyst looks at your portfolio, analyzes it, but the assets remain yours. This is exactly what **references** are in Rust — passing access without transferring ownership.

## Why Do We Need References in Trading?

When analyzing a portfolio, price history, or order list, we don't want to:
- Copy huge amounts of data (expensive in memory)
- Lose ownership of data after analysis (needed for further work)
- Give full modification access when read-only is enough

## Basic Portfolio Structure

```rust
#[derive(Debug)]
struct Position {
    ticker: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn new(ticker: &str, quantity: f64, entry_price: f64, current_price: f64) -> Self {
        Position {
            ticker: ticker.to_string(),
            quantity,
            entry_price,
            current_price,
        }
    }

    fn market_value(&self) -> f64 {
        self.quantity * self.current_price
    }

    fn unrealized_pnl(&self) -> f64 {
        self.quantity * (self.current_price - self.entry_price)
    }

    fn pnl_percent(&self) -> f64 {
        ((self.current_price - self.entry_price) / self.entry_price) * 100.0
    }
}

#[derive(Debug)]
struct Portfolio {
    name: String,
    positions: Vec<Position>,
    cash_balance: f64,
}

impl Portfolio {
    fn new(name: &str, cash_balance: f64) -> Self {
        Portfolio {
            name: name.to_string(),
            positions: Vec::new(),
            cash_balance,
        }
    }

    fn add_position(&mut self, position: Position) {
        self.positions.push(position);
    }
}
```

## Immutable References: Analysis Without Modification

### Calculating Total Portfolio Value

```rust
fn calculate_total_value(portfolio: &Portfolio) -> f64 {
    let positions_value: f64 = portfolio.positions
        .iter()
        .map(|p| p.market_value())
        .sum();

    positions_value + portfolio.cash_balance
}

fn main() {
    let mut portfolio = Portfolio::new("Main Trading Account", 10000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 45000.0));
    portfolio.add_position(Position::new("ETH", 10.0, 2200.0, 2500.0));
    portfolio.add_position(Position::new("AAPL", 50.0, 150.0, 175.0));

    // Passing a reference — portfolio stays with us
    let total = calculate_total_value(&portfolio);
    println!("Total portfolio value: ${:.2}", total);

    // We can still use portfolio!
    println!("Portfolio name: {}", portfolio.name);
}
```

### Analyzing All Positions

```rust
fn analyze_positions(portfolio: &Portfolio) {
    println!("\n=== Portfolio Analysis: {} ===", portfolio.name);
    println!("{:<8} {:>10} {:>12} {:>10} {:>8}",
             "Ticker", "Quantity", "Value", "PnL", "PnL%");
    println!("{}", "-".repeat(52));

    for position in &portfolio.positions {
        println!("{:<8} {:>10.2} {:>12.2} {:>10.2} {:>7.2}%",
                 position.ticker,
                 position.quantity,
                 position.market_value(),
                 position.unrealized_pnl(),
                 position.pnl_percent());
    }
}

fn main() {
    let mut portfolio = Portfolio::new("Crypto Portfolio", 5000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 45000.0));
    portfolio.add_position(Position::new("ETH", 10.0, 2200.0, 2500.0));

    analyze_positions(&portfolio);

    // Portfolio is still ours
    println!("\nCash balance: ${:.2}", portfolio.cash_balance);
}
```

## Multiple Immutable References

In Rust, you can have as many immutable references as you want simultaneously:

```rust
fn calculate_total_pnl(portfolio: &Portfolio) -> f64 {
    portfolio.positions
        .iter()
        .map(|p| p.unrealized_pnl())
        .sum()
}

fn calculate_win_rate(portfolio: &Portfolio) -> f64 {
    let total = portfolio.positions.len() as f64;
    if total == 0.0 { return 0.0; }

    let winners = portfolio.positions
        .iter()
        .filter(|p| p.unrealized_pnl() > 0.0)
        .count() as f64;

    (winners / total) * 100.0
}

fn find_best_performer(portfolio: &Portfolio) -> Option<&Position> {
    portfolio.positions
        .iter()
        .max_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
}

fn find_worst_performer(portfolio: &Portfolio) -> Option<&Position> {
    portfolio.positions
        .iter()
        .min_by(|a, b| a.pnl_percent().partial_cmp(&b.pnl_percent()).unwrap())
}

fn main() {
    let mut portfolio = Portfolio::new("Mixed Portfolio", 10000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 45000.0));  // Profit
    portfolio.add_position(Position::new("ETH", 5.0, 2800.0, 2500.0));   // Loss
    portfolio.add_position(Position::new("SOL", 100.0, 20.0, 35.0));     // Profit

    // All functions receive references — no conflicts!
    let total_pnl = calculate_total_pnl(&portfolio);
    let win_rate = calculate_win_rate(&portfolio);
    let best = find_best_performer(&portfolio);
    let worst = find_worst_performer(&portfolio);

    println!("Total PnL: ${:.2}", total_pnl);
    println!("Win Rate: {:.1}%", win_rate);

    if let Some(pos) = best {
        println!("Best: {} ({:+.2}%)", pos.ticker, pos.pnl_percent());
    }
    if let Some(pos) = worst {
        println!("Worst: {} ({:+.2}%)", pos.ticker, pos.pnl_percent());
    }
}
```

## Mutable References: Modifying the Portfolio

When you need to modify the portfolio, use `&mut`:

```rust
fn update_prices(portfolio: &mut Portfolio, updates: &[(String, f64)]) {
    for (ticker, new_price) in updates {
        if let Some(position) = portfolio.positions
            .iter_mut()
            .find(|p| &p.ticker == ticker)
        {
            position.current_price = *new_price;
        }
    }
}

fn apply_deposit(portfolio: &mut Portfolio, amount: f64) {
    portfolio.cash_balance += amount;
    println!("Deposited ${:.2}. New balance: ${:.2}",
             amount, portfolio.cash_balance);
}

fn close_position(portfolio: &mut Portfolio, ticker: &str) -> Option<f64> {
    if let Some(idx) = portfolio.positions
        .iter()
        .position(|p| p.ticker == ticker)
    {
        let position = portfolio.positions.remove(idx);
        let realized_pnl = position.unrealized_pnl();
        let proceeds = position.market_value();

        portfolio.cash_balance += proceeds;

        println!("Closed {} position. Proceeds: ${:.2}, PnL: ${:.2}",
                 ticker, proceeds, realized_pnl);

        Some(realized_pnl)
    } else {
        println!("Position {} not found", ticker);
        None
    }
}

fn main() {
    let mut portfolio = Portfolio::new("Active Trading", 5000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 42000.0));
    portfolio.add_position(Position::new("ETH", 10.0, 2200.0, 2200.0));

    // Update prices
    let price_updates = vec![
        ("BTC".to_string(), 45000.0),
        ("ETH".to_string(), 2500.0),
    ];
    update_prices(&mut portfolio, &price_updates);

    // Add funds
    apply_deposit(&mut portfolio, 2000.0);

    analyze_positions(&portfolio);  // Immutable reference for analysis

    // Close a position
    close_position(&mut portfolio, "BTC");

    println!("\nFinal cash balance: ${:.2}", portfolio.cash_balance);
}
```

## The Rule: One Mutable OR Many Immutable

```rust
fn main() {
    let mut portfolio = Portfolio::new("Test", 1000.0);
    portfolio.add_position(Position::new("BTC", 0.1, 40000.0, 42000.0));

    // OK: multiple immutable references
    let ref1 = &portfolio;
    let ref2 = &portfolio;
    println!("Value via ref1: ${:.2}", calculate_total_value(ref1));
    println!("Value via ref2: ${:.2}", calculate_total_value(ref2));

    // OK: one mutable reference (after immutable refs are done)
    let ref_mut = &mut portfolio;
    apply_deposit(ref_mut, 500.0);

    // This will NOT compile:
    // let ref3 = &portfolio;      // immutable reference
    // apply_deposit(&mut portfolio, 100.0);  // mutable reference
    // println!("{:?}", ref3);     // using immutable ref
}
```

## Practical Example: Complete Portfolio Analyzer

```rust
#[derive(Debug)]
struct PortfolioAnalysis {
    total_value: f64,
    total_invested: f64,
    total_pnl: f64,
    total_pnl_percent: f64,
    win_rate: f64,
    best_performer: Option<String>,
    worst_performer: Option<String>,
    largest_position: Option<String>,
    risk_concentration: f64,
}

fn analyze_portfolio_full(portfolio: &Portfolio) -> PortfolioAnalysis {
    let total_value = calculate_total_value(portfolio);

    let total_invested: f64 = portfolio.positions
        .iter()
        .map(|p| p.quantity * p.entry_price)
        .sum();

    let total_pnl = calculate_total_pnl(portfolio);

    let total_pnl_percent = if total_invested > 0.0 {
        (total_pnl / total_invested) * 100.0
    } else {
        0.0
    };

    let win_rate = calculate_win_rate(portfolio);

    let best = find_best_performer(portfolio)
        .map(|p| p.ticker.clone());

    let worst = find_worst_performer(portfolio)
        .map(|p| p.ticker.clone());

    let largest = portfolio.positions
        .iter()
        .max_by(|a, b| a.market_value().partial_cmp(&b.market_value()).unwrap())
        .map(|p| p.ticker.clone());

    // Risk concentration: share of the largest position
    let risk_concentration = if let Some(max_pos) = portfolio.positions
        .iter()
        .max_by(|a, b| a.market_value().partial_cmp(&b.market_value()).unwrap())
    {
        (max_pos.market_value() / total_value) * 100.0
    } else {
        0.0
    };

    PortfolioAnalysis {
        total_value,
        total_invested,
        total_pnl,
        total_pnl_percent,
        win_rate,
        best_performer: best,
        worst_performer: worst,
        largest_position: largest,
        risk_concentration,
    }
}

fn print_analysis(analysis: &PortfolioAnalysis) {
    println!("\n╔════════════════════════════════════════╗");
    println!("║       PORTFOLIO ANALYSIS REPORT        ║");
    println!("╠════════════════════════════════════════╣");
    println!("║ Total Value:     ${:>18.2} ║", analysis.total_value);
    println!("║ Total Invested:  ${:>18.2} ║", analysis.total_invested);
    println!("║ Total PnL:       ${:>18.2} ║", analysis.total_pnl);
    println!("║ Return:          {:>18.2}% ║", analysis.total_pnl_percent);
    println!("║ Win Rate:        {:>18.2}% ║", analysis.win_rate);
    println!("╠════════════════════════════════════════╣");

    if let Some(ref best) = analysis.best_performer {
        println!("║ Best Performer:  {:>20} ║", best);
    }
    if let Some(ref worst) = analysis.worst_performer {
        println!("║ Worst Performer: {:>20} ║", worst);
    }
    if let Some(ref largest) = analysis.largest_position {
        println!("║ Largest Position:{:>20} ║", largest);
    }

    println!("╠════════════════════════════════════════╣");
    println!("║ Risk Concentration:{:>17.2}% ║", analysis.risk_concentration);

    if analysis.risk_concentration > 50.0 {
        println!("║ WARNING: High concentration risk!      ║");
    }

    println!("╚════════════════════════════════════════╝");
}

fn main() {
    let mut portfolio = Portfolio::new("Main Trading Portfolio", 15000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 42000.0, 48000.0));
    portfolio.add_position(Position::new("ETH", 10.0, 2200.0, 2800.0));
    portfolio.add_position(Position::new("SOL", 200.0, 25.0, 35.0));
    portfolio.add_position(Position::new("AAPL", 30.0, 170.0, 165.0));
    portfolio.add_position(Position::new("NVDA", 20.0, 450.0, 520.0));

    // Full analysis using references
    let analysis = analyze_portfolio_full(&portfolio);
    print_analysis(&analysis);

    // Portfolio is still available for further work
    println!("\nPositions count: {}", portfolio.positions.len());
}
```

## References in Closures

```rust
fn filter_profitable_positions(portfolio: &Portfolio) -> Vec<&Position> {
    portfolio.positions
        .iter()
        .filter(|p| p.unrealized_pnl() > 0.0)
        .collect()
}

fn filter_positions_by_pnl_threshold(
    portfolio: &Portfolio,
    threshold: f64
) -> Vec<&Position> {
    portfolio.positions
        .iter()
        .filter(|p| p.pnl_percent().abs() > threshold)
        .collect()
}

fn main() {
    let mut portfolio = Portfolio::new("Test", 5000.0);

    portfolio.add_position(Position::new("BTC", 0.5, 40000.0, 45000.0));  // +12.5%
    portfolio.add_position(Position::new("ETH", 10.0, 2800.0, 2500.0));  // -10.7%
    portfolio.add_position(Position::new("SOL", 100.0, 30.0, 31.0));     // +3.3%

    let profitable = filter_profitable_positions(&portfolio);
    println!("Profitable positions: {:?}",
             profitable.iter().map(|p| &p.ticker).collect::<Vec<_>>());

    // Positions with movement greater than 5%
    let significant_moves = filter_positions_by_pnl_threshold(&portfolio, 5.0);
    println!("Significant moves (>5%): {:?}",
             significant_moves.iter().map(|p| &p.ticker).collect::<Vec<_>>());
}
```

## What We Learned

| Concept | Syntax | When to Use |
|---------|--------|-------------|
| Immutable reference | `&T` | Reading without modification |
| Mutable reference | `&mut T` | Reading and modifying |
| Multiple `&T` | Allowed | Parallel reading |
| Single `&mut T` | Required | Exclusive access |
| Returning reference | `-> &T` | Part of owner's data |

## Homework

1. Write a function `calculate_sector_allocation(portfolio: &Portfolio, sectors: &HashMap<String, String>) -> HashMap<String, f64>` that calculates distribution by sectors (e.g., "BTC" -> "Crypto", "AAPL" -> "Tech")

2. Create a function `rebalance_suggestions(portfolio: &Portfolio, target_weights: &HashMap<String, f64>) -> Vec<(String, f64)>` that returns rebalancing recommendations (ticker, amount to buy/sell)

3. Implement a function `risk_metrics(portfolio: &Portfolio, benchmark: &[f64]) -> RiskMetrics` that calculates risk metrics: volatility, beta, maximum drawdown

4. Write an alert system `check_alerts(portfolio: &Portfolio, alerts: &[Alert]) -> Vec<String>` that checks conditions (price > X, PnL < Y) and returns triggered alerts

## Navigation

[← Previous day](../050-ownership-practice-trade-analysis/en.md) | [Next day →](../052-slice-practice-partial-history/en.md)
