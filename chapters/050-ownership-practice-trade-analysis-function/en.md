# Day 50: Ownership Practice — Trade Analysis Function

## Trading Analogy

Imagine you have a **unique trading report** — the only copy of a document with detailed trade analysis. When you pass this report to an analyst:

1. **Transfer of ownership (move)**: You hand over the report — now the analyst has it, and you don't
2. **Borrowing (borrow)**: You let them look at the report — the analyst reads it, but the report remains yours
3. **Mutable borrowing**: You give them a pencil for notes — the analyst can make annotations, but only one person at a time

In Rust, these concepts apply to data in memory — and this is critical for safe handling of trading data!

## Theory: Ownership Rules in Rust

### Three Main Rules

```rust
// 1. Every value has an owner
let trade_data = String::from("BTC/USDT: +$1500");

// 2. There can only be one owner at a time
let analysis = trade_data;  // Ownership transferred to analysis
// println!("{}", trade_data);  // Error! trade_data no longer owns the data

// 3. When the owner goes out of scope, the value is dropped
{
    let temp_report = String::from("Temporary");
}  // temp_report is dropped here
```

### Why Is This Important for Trading?

In trading systems, it's critical to:
- **Not duplicate orders** — an accidental copy could lead to double execution
- **Not use stale data** — prices change every millisecond
- **Guarantee resource cleanup** — exchange connections must close properly

## Move: Transferring Trade Data Ownership

```rust
fn main() {
    // Create trade data
    let trade = create_trade("BTC/USDT", 42000.0, 0.5, "BUY");

    // Transfer ownership to analysis function
    let report = analyze_trade(trade);

    // trade is no longer accessible — ownership was transferred!
    // println!("{:?}", trade);  // Compile error

    println!("{}", report);
}

#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

fn create_trade(symbol: &str, price: f64, quantity: f64, side: &str) -> Trade {
    Trade {
        symbol: String::from(symbol),
        price,
        quantity,
        side: String::from(side),
    }
}

fn analyze_trade(trade: Trade) -> String {
    let value = trade.price * trade.quantity;
    format!(
        "Trade Analysis:\n  Symbol: {}\n  Side: {}\n  Value: ${:.2}",
        trade.symbol, trade.side, value
    )
}
```

## Borrow: Borrowing for Reading

```rust
fn main() {
    let portfolio = create_portfolio();

    // Borrow for reading — can create multiple references
    print_portfolio_summary(&portfolio);
    let total = calculate_total_value(&portfolio);
    let risk = assess_risk(&portfolio);

    // portfolio is still accessible!
    println!("\nTotal: ${:.2}, Risk: {}", total, risk);
    println!("Positions count: {}", portfolio.len());
}

#[derive(Debug)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

fn create_portfolio() -> Vec<Position> {
    vec![
        Position {
            symbol: String::from("BTC"),
            quantity: 0.5,
            entry_price: 42000.0,
            current_price: 43500.0,
        },
        Position {
            symbol: String::from("ETH"),
            quantity: 5.0,
            entry_price: 2800.0,
            current_price: 2650.0,
        },
        Position {
            symbol: String::from("SOL"),
            quantity: 100.0,
            entry_price: 95.0,
            current_price: 110.0,
        },
    ]
}

fn print_portfolio_summary(portfolio: &Vec<Position>) {
    println!("=== Portfolio Summary ===");
    for pos in portfolio {
        let pnl = (pos.current_price - pos.entry_price) * pos.quantity;
        let status = if pnl >= 0.0 { "+" } else { "" };
        println!("  {}: {}${:.2}", pos.symbol, status, pnl);
    }
}

fn calculate_total_value(portfolio: &Vec<Position>) -> f64 {
    portfolio.iter()
        .map(|pos| pos.current_price * pos.quantity)
        .sum()
}

fn assess_risk(portfolio: &Vec<Position>) -> &str {
    let losing_positions = portfolio.iter()
        .filter(|pos| pos.current_price < pos.entry_price)
        .count();

    let ratio = losing_positions as f64 / portfolio.len() as f64;

    if ratio > 0.5 {
        "HIGH"
    } else if ratio > 0.25 {
        "MEDIUM"
    } else {
        "LOW"
    }
}
```

## Mutable Borrow: Modifying Data

```rust
fn main() {
    let mut order_book = OrderBook::new();

    // Add orders — mutable borrow
    add_bid(&mut order_book, 41900.0, 1.5);
    add_bid(&mut order_book, 41800.0, 2.0);
    add_ask(&mut order_book, 42100.0, 1.0);
    add_ask(&mut order_book, 42200.0, 3.0);

    // Read — immutable borrow
    print_order_book(&order_book);

    // Execute order — mutable again
    execute_market_buy(&mut order_book, 0.5);

    println!("\nAfter execution:");
    print_order_book(&order_book);
}

struct OrderBook {
    bids: Vec<(f64, f64)>,  // (price, quantity)
    asks: Vec<(f64, f64)>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }
}

fn add_bid(book: &mut OrderBook, price: f64, quantity: f64) {
    book.bids.push((price, quantity));
    // Sort by descending price (best bid on top)
    book.bids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
}

fn add_ask(book: &mut OrderBook, price: f64, quantity: f64) {
    book.asks.push((price, quantity));
    // Sort by ascending price (best ask on top)
    book.asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
}

fn print_order_book(book: &OrderBook) {
    println!("╔══════════════════════════╗");
    println!("║      ORDER BOOK          ║");
    println!("╠══════════════════════════╣");
    println!("║ ASKS (Sell orders)       ║");
    for (price, qty) in book.asks.iter().rev() {
        println!("║   ${:.2} x {:.4}         ║", price, qty);
    }
    println!("╠══════════════════════════╣");
    println!("║ BIDS (Buy orders)        ║");
    for (price, qty) in &book.bids {
        println!("║   ${:.2} x {:.4}         ║", price, qty);
    }
    println!("╚══════════════════════════╝");
}

fn execute_market_buy(book: &mut OrderBook, quantity: f64) {
    let mut remaining = quantity;

    while remaining > 0.0 && !book.asks.is_empty() {
        let (price, available) = book.asks[0];

        if available <= remaining {
            println!("Filled {:.4} @ ${:.2}", available, price);
            remaining -= available;
            book.asks.remove(0);
        } else {
            println!("Filled {:.4} @ ${:.2}", remaining, price);
            book.asks[0].1 -= remaining;
            remaining = 0.0;
        }
    }

    if remaining > 0.0 {
        println!("Warning: {:.4} unfilled (no liquidity)", remaining);
    }
}
```

## Practical Example: Complete Trade Analyzer

```rust
fn main() {
    // Create trade history
    let mut trade_history = TradeHistory::new();

    // Add trades
    record_trade(&mut trade_history, "BTC/USDT", 42000.0, 43500.0, 0.5);
    record_trade(&mut trade_history, "ETH/USDT", 2800.0, 2650.0, 5.0);
    record_trade(&mut trade_history, "BTC/USDT", 43000.0, 44200.0, 0.3);
    record_trade(&mut trade_history, "SOL/USDT", 95.0, 88.0, 100.0);
    record_trade(&mut trade_history, "BTC/USDT", 44000.0, 43800.0, 0.2);

    // Analyze — pass by reference
    let analysis = analyze_history(&trade_history);
    print_analysis(&analysis);

    // Filter profitable — another borrow
    let profitable = filter_profitable(&trade_history);
    println!("\nProfitable trades: {}", profitable.len());

    // Group by symbol
    let by_symbol = group_by_symbol(&trade_history);
    print_grouped_stats(&by_symbol);
}

struct TradeRecord {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

struct TradeHistory {
    trades: Vec<TradeRecord>,
}

impl TradeHistory {
    fn new() -> Self {
        TradeHistory { trades: Vec::new() }
    }
}

struct HistoryAnalysis {
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    total_pnl: f64,
    largest_win: f64,
    largest_loss: f64,
    win_rate: f64,
    average_pnl: f64,
}

fn record_trade(
    history: &mut TradeHistory,
    symbol: &str,
    entry: f64,
    exit: f64,
    quantity: f64,
) {
    let pnl = (exit - entry) * quantity;
    history.trades.push(TradeRecord {
        symbol: String::from(symbol),
        entry_price: entry,
        exit_price: exit,
        quantity,
        pnl,
    });
}

fn analyze_history(history: &TradeHistory) -> HistoryAnalysis {
    let total_trades = history.trades.len();

    if total_trades == 0 {
        return HistoryAnalysis {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_pnl: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
            win_rate: 0.0,
            average_pnl: 0.0,
        };
    }

    let winning_trades = history.trades.iter().filter(|t| t.pnl > 0.0).count();
    let losing_trades = history.trades.iter().filter(|t| t.pnl < 0.0).count();
    let total_pnl: f64 = history.trades.iter().map(|t| t.pnl).sum();

    let largest_win = history.trades.iter()
        .map(|t| t.pnl)
        .fold(0.0_f64, |a, b| a.max(b));

    let largest_loss = history.trades.iter()
        .map(|t| t.pnl)
        .fold(0.0_f64, |a, b| a.min(b));

    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;
    let average_pnl = total_pnl / total_trades as f64;

    HistoryAnalysis {
        total_trades,
        winning_trades,
        losing_trades,
        total_pnl,
        largest_win,
        largest_loss,
        win_rate,
        average_pnl,
    }
}

fn print_analysis(analysis: &HistoryAnalysis) {
    println!("╔════════════════════════════════════╗");
    println!("║      TRADING HISTORY ANALYSIS      ║");
    println!("╠════════════════════════════════════╣");
    println!("║ Total trades:      {:>15} ║", analysis.total_trades);
    println!("║ Winning trades:    {:>15} ║", analysis.winning_trades);
    println!("║ Losing trades:     {:>15} ║", analysis.losing_trades);
    println!("║ Win rate:          {:>14.1}% ║", analysis.win_rate);
    println!("╠════════════════════════════════════╣");
    println!("║ Total PnL:        ${:>14.2} ║", analysis.total_pnl);
    println!("║ Average PnL:      ${:>14.2} ║", analysis.average_pnl);
    println!("║ Largest win:      ${:>14.2} ║", analysis.largest_win);
    println!("║ Largest loss:     ${:>14.2} ║", analysis.largest_loss);
    println!("╚════════════════════════════════════╝");
}

fn filter_profitable(history: &TradeHistory) -> Vec<&TradeRecord> {
    history.trades.iter().filter(|t| t.pnl > 0.0).collect()
}

fn group_by_symbol(history: &TradeHistory) -> std::collections::HashMap<String, Vec<&TradeRecord>> {
    let mut groups: std::collections::HashMap<String, Vec<&TradeRecord>> = std::collections::HashMap::new();

    for trade in &history.trades {
        groups.entry(trade.symbol.clone())
            .or_insert_with(Vec::new)
            .push(trade);
    }

    groups
}

fn print_grouped_stats(groups: &std::collections::HashMap<String, Vec<&TradeRecord>>) {
    println!("\n=== Performance by Symbol ===");

    for (symbol, trades) in groups {
        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let wins = trades.iter().filter(|t| t.pnl > 0.0).count();
        let win_rate = (wins as f64 / trades.len() as f64) * 100.0;

        let status = if total_pnl >= 0.0 { "+" } else { "" };
        println!(
            "  {}: {} trades, {}${:.2}, {:.0}% win rate",
            symbol,
            trades.len(),
            status,
            total_pnl,
            win_rate
        );
    }
}
```

## Exercise 1: Position Tracker with Ownership

```rust
fn main() {
    // Create a PositionTracker struct
    // Implement:
    // - open_position(&mut self, ...) -> adds a position
    // - close_position(&mut self, symbol) -> closes and returns PnL
    // - get_position(&self, symbol) -> returns &Position or None
    // - calculate_exposure(&self) -> total position size

    let mut tracker = PositionTracker::new();

    tracker.open_position("BTC", 42000.0, 0.5);
    tracker.open_position("ETH", 2800.0, 5.0);

    println!("BTC position: {:?}", tracker.get_position("BTC"));
    println!("Total exposure: ${:.2}", tracker.calculate_exposure());

    if let Some(pnl) = tracker.close_position("BTC", 43000.0) {
        println!("Closed BTC with PnL: ${:.2}", pnl);
    }
}

// Your implementation here...
```

## Exercise 2: Order Validator

```rust
fn main() {
    // Create functions:
    // - validate_order(&Order) -> Result<(), String>
    // - enrich_order(&mut Order) -> adds timestamp and ID
    // - submit_order(Order) -> OrderReceipt (transfers ownership)

    let mut order = Order::new("BTC/USDT", 42000.0, 0.5, "LIMIT", "BUY");

    // Validation doesn't change the order
    match validate_order(&order) {
        Ok(()) => println!("Order valid"),
        Err(e) => println!("Invalid: {}", e),
    }

    // Enrichment modifies the order
    enrich_order(&mut order);

    // Submission takes ownership
    let receipt = submit_order(order);
    // order is no longer accessible!

    println!("Receipt: {:?}", receipt);
}

// Your implementation here...
```

## Exercise 3: Liquidity Analyzer

```rust
fn main() {
    // Implement functions with proper borrowing:
    // - calculate_spread(&OrderBook) -> f64
    // - calculate_depth(&OrderBook, levels: usize) -> (f64, f64)
    // - find_price_impact(&OrderBook, quantity: f64) -> f64
    // - merge_order_books(&OrderBook, &OrderBook) -> OrderBook

    let book1 = create_order_book_1();
    let book2 = create_order_book_2();

    let spread = calculate_spread(&book1);
    println!("Spread: ${:.2}", spread);

    let (bid_depth, ask_depth) = calculate_depth(&book1, 3);
    println!("Depth - Bids: ${:.2}, Asks: ${:.2}", bid_depth, ask_depth);

    let impact = find_price_impact(&book1, 2.0);
    println!("Price impact for 2 BTC: ${:.2}", impact);

    // Merging doesn't consume original books
    let merged = merge_order_books(&book1, &book2);
    println!("Merged book has {} bids", merged.bids.len());

    // book1 and book2 are still accessible
    println!("Original book1 spread: ${:.2}", calculate_spread(&book1));
}

// Your implementation here...
```

## Exercise 4: Risk Manager

```rust
fn main() {
    // Create RiskManager with ownership rules:
    // - new(max_position: f64, max_loss: f64) -> RiskManager
    // - check_order(&self, &Order, &Portfolio) -> Result<(), RiskError>
    // - update_limits(&mut self, new_max: f64)
    // - consume_and_report(self) -> RiskReport (consumes manager)

    let mut risk_manager = RiskManager::new(100000.0, 5000.0);
    let portfolio = create_test_portfolio();
    let order = create_test_order();

    // Check doesn't modify anything
    match risk_manager.check_order(&order, &portfolio) {
        Ok(()) => println!("Order passed risk check"),
        Err(e) => println!("Risk violation: {:?}", e),
    }

    // Update limits
    risk_manager.update_limits(150000.0);

    // Final report consumes the manager
    let report = risk_manager.consume_and_report();
    // risk_manager is no longer accessible!

    println!("Report: {:?}", report);
}

// Your implementation here...
```

## What We Learned

| Concept | Syntax | Trading Application |
|---------|--------|---------------------|
| Move | `fn process(data: Data)` | Sending order for execution |
| Borrow | `fn analyze(data: &Data)` | Reading portfolio for report |
| Mut Borrow | `fn update(data: &mut Data)` | Updating position |
| Return ownership | `fn create() -> Data` | Creating new order |
| Lifetime | `fn get<'a>(&'a self) -> &'a T` | References to struct data |

## Common Mistakes and How to Avoid Them

```rust
// ❌ Error: use after move
fn bad_example() {
    let order = Order::new();
    submit(order);
    println!("{:?}", order);  // Error!
}

// ✅ Correct: clone or use reference
fn good_example() {
    let order = Order::new();
    submit(order.clone());  // Send a copy
    println!("{:?}", order);  // Original is accessible
}

// ❌ Error: simultaneous mutable borrow
fn bad_borrow() {
    let mut book = OrderBook::new();
    let ref1 = &mut book;
    let ref2 = &mut book;  // Error!
}

// ✅ Correct: sequential borrows
fn good_borrow() {
    let mut book = OrderBook::new();
    {
        let ref1 = &mut book;
        update(ref1);
    }  // ref1 is released
    let ref2 = &mut book;  // Now allowed
}
```

## Homework

1. **Portfolio Tracker**: Create a `PortfolioTracker` that stores positions and properly manages ownership when adding/removing/updating positions

2. **Alert System**: Implement an `AlertSystem` where alerts are transferred by ownership when triggered, but checked by reference

3. **Data Aggregator**: Write functions for aggregating market data using only borrowing (no copying large arrays)

4. **Analysis Pipeline**: Create a chain of analysis functions where each function takes data by reference and returns a new struct with results

## Navigation

[← Previous day](../049-ownership-borrowing-trading-data/en.md) | [Next day →](../051-references-market-data-sharing/en.md)
