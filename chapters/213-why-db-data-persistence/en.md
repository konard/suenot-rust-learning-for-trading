# Day 213: Why DB: Data Persistence

## Trading Analogy

Imagine you're running a trading bot. It works all day, executes hundreds of trades, earns profit. And then... you restart your computer. What happens? All the data — trade history, current positions, statistics — is gone.

It's like if your broker forgot every morning how many shares you own. Picture this: you own 100 Apple shares, turn off your computer for the night, and in the morning the broker says: "What shares? You don't own anything!"

**Data persistence** is the ability to save data between program runs. A database is like a safe for your trading information: reliable, organized, and always accessible.

## The Problem: Data in Memory is Temporary

When a program runs, all data is stored in RAM (Random Access Memory). But RAM is temporary storage:

```rust
fn main() {
    // This data only lives while the program is running
    let mut trades = Vec::new();

    trades.push(Trade {
        id: 1,
        symbol: "BTC".to_string(),
        price: 42000.0,
        quantity: 0.5,
    });

    trades.push(Trade {
        id: 2,
        symbol: "ETH".to_string(),
        price: 2800.0,
        quantity: 10.0,
    });

    println!("We have {} trades", trades.len());

    // When the program ends — all data disappears!
}

#[derive(Debug)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}
```

After the program terminates, the `trades` variable completely disappears. On the next run — an empty vector.

## What We Lose Without Persistence

### 1. Trade History

```rust
// Without database: every launch starts from zero
fn main() {
    let mut trade_history: Vec<Trade> = Vec::new();

    // Add today's trades
    trade_history.push(Trade {
        id: 1,
        symbol: "BTC".to_string(),
        price: 42000.0,
        quantity: 0.5,
        timestamp: 1699000000,
    });

    // Tomorrow after restart — history is empty!
    // Can't analyze past results
    // Can't calculate statistics
    // Can't report to tax authorities
}

#[derive(Debug)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
    timestamp: i64,
}
```

### 2. Positions and Balances

```rust
// Dangerous situation: forgot open positions
struct Portfolio {
    cash: f64,
    positions: std::collections::HashMap<String, f64>,
}

fn main() {
    let mut portfolio = Portfolio {
        cash: 10_000.0,
        positions: std::collections::HashMap::new(),
    };

    // Bought 0.5 BTC
    portfolio.positions.insert("BTC".to_string(), 0.5);
    portfolio.cash -= 21_000.0;

    println!("Balance: ${}, BTC: {}",
        portfolio.cash,
        portfolio.positions.get("BTC").unwrap_or(&0.0));

    // Program crashed or restarted...
    // On restart: cash = 10_000, positions = {}
    // We "forgot" that we spent $21k and own BTC!
}
```

### 3. Settings and Configuration

```rust
struct TradingConfig {
    max_position_size: f64,
    stop_loss_percent: f64,
    api_keys: std::collections::HashMap<String, String>,
    enabled_pairs: Vec<String>,
}

fn main() {
    // Configured the bot for an hour, then it restarted...
    let config = TradingConfig {
        max_position_size: 1000.0,
        stop_loss_percent: 2.0,
        api_keys: std::collections::HashMap::new(), // Added keys manually
        enabled_pairs: vec![
            "BTC/USDT".to_string(),
            "ETH/USDT".to_string(),
            // ... 20 more pairs
        ],
    };

    // After restart — configure everything again!
}
```

## The Solution: Persistence

Persistence means saving data to permanent storage (disk, SSD, cloud), from which it can be recovered on restart.

### Simplest Way: Files

```rust
use std::fs;
use std::io::{Write, BufRead, BufReader};

#[derive(Debug)]
struct Trade {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

impl Trade {
    // Save to CSV format
    fn to_csv_line(&self) -> String {
        format!("{},{},{},{}", self.id, self.symbol, self.price, self.quantity)
    }

    // Parse from CSV line
    fn from_csv_line(line: &str) -> Result<Trade, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Trade {
            id: parts[0].parse()?,
            symbol: parts[1].to_string(),
            price: parts[2].parse()?,
            quantity: parts[3].parse()?,
        })
    }
}

fn save_trades(trades: &[Trade], filename: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(filename)?;
    for trade in trades {
        writeln!(file, "{}", trade.to_csv_line())?;
    }
    Ok(())
}

fn load_trades(filename: &str) -> std::io::Result<Vec<Trade>> {
    let file = fs::File::open(filename)?;
    let reader = BufReader::new(file);
    let mut trades = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(trade) = Trade::from_csv_line(&line) {
            trades.push(trade);
        }
    }

    Ok(trades)
}

fn main() -> std::io::Result<()> {
    let filename = "trades.csv";

    // Load existing trades (if file exists)
    let mut trades = load_trades(filename).unwrap_or_else(|_| Vec::new());

    println!("Loaded {} trades from the past", trades.len());

    // Add a new trade
    let new_trade = Trade {
        id: trades.len() as u64 + 1,
        symbol: "BTC".to_string(),
        price: 42500.0,
        quantity: 0.1,
    };

    trades.push(new_trade);
    println!("Added new trade, total: {}", trades.len());

    // Save back
    save_trades(&trades, filename)?;
    println!("Trades saved to {}", filename);

    Ok(())
}
```

## Why Files Aren't Enough

Files are a good start, but they have serious limitations:

### 1. Performance

```rust
// Problem: to find a trade, you need to read the entire file
fn find_trade_by_id(filename: &str, target_id: u64) -> Option<Trade> {
    let trades = load_trades(filename).ok()?;

    // O(n) — reading ALL records even if we need the first one
    for trade in trades {
        if trade.id == target_id {
            return Some(trade);
        }
    }
    None
}

// If we have 1 million trades — this is slow!
```

### 2. Concurrent Access

```rust
// Problem: two threads writing simultaneously
use std::thread;

fn main() {
    let filename = "trades.csv";

    // Thread 1: adds a trade
    let handle1 = thread::spawn(move || {
        // Reading...
        // ... meanwhile thread 2 is also reading
        // Add trade
        // Save
        // ... thread 2 overwrites our changes!
    });

    // Thread 2: adds another trade
    let handle2 = thread::spawn(move || {
        // Same actions — race condition!
    });

    // Result: one of the trades is lost
}
```

### 3. Complex Queries

```rust
// Need to find: all BTC trades from last month with profit > $100
// With files — this is a nightmare:

fn find_profitable_btc_trades(filename: &str) -> Vec<Trade> {
    let trades = load_trades(filename).unwrap();
    let month_ago = current_timestamp() - 30 * 24 * 60 * 60;

    trades.into_iter()
        .filter(|t| t.symbol == "BTC")
        .filter(|t| t.timestamp > month_ago)
        .filter(|t| calculate_profit(t) > 100.0)
        .collect()
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn calculate_profit(_trade: &Trade) -> f64 {
    // Need to load more data about current prices...
    0.0
}

// With a database this is one SQL query!
```

### 4. Data Integrity

```rust
// Problem: program crashed in the middle of writing
fn save_trade_and_update_portfolio(trade: &Trade, portfolio: &mut Portfolio) {
    // Step 1: save the trade
    save_trade_to_file(trade);

    // PROGRAM CRASHED HERE!

    // Step 2: update balance — didn't execute
    portfolio.cash -= trade.price * trade.quantity;
    save_portfolio_to_file(portfolio);

    // Result: trade is saved, but balance not updated
    // Data is in an inconsistent state!
}

fn save_trade_to_file(_trade: &Trade) {}
fn save_portfolio_to_file(_portfolio: &Portfolio) {}

struct Portfolio {
    cash: f64,
}
```

## What a Database Provides

### 1. Indexes — Fast Search

A database creates special structures (indexes) that allow instant data search:

```rust
// With database: search by id — O(log n) or O(1)
// Instead of scanning all 1 million records — 20 operations!

// Pseudocode
fn find_trade_with_db(db: &Database, id: u64) -> Option<Trade> {
    // Index on id — instant search
    db.query("SELECT * FROM trades WHERE id = ?", &[id])
}
```

### 2. Transactions — Atomic Operations

```rust
// Database guarantees: either everything executes, or nothing
fn execute_trade_atomically(db: &Database, trade: &Trade, portfolio: &mut Portfolio) {
    db.transaction(|tx| {
        // Step 1: record the trade
        tx.execute("INSERT INTO trades ...", trade)?;

        // Step 2: update balance
        tx.execute("UPDATE portfolio SET cash = cash - ?",
                   trade.price * trade.quantity)?;

        // If any step fails — rollback everything
        Ok(())
    });
    // Guarantee: data is always consistent!
}

struct Database;
impl Database {
    fn transaction<F, R>(&self, _f: F) -> R
    where F: FnOnce(&Transaction) -> R {
        unimplemented!()
    }
}

struct Transaction;
impl Transaction {
    fn execute(&self, _query: &str, _params: impl std::fmt::Debug) -> Result<(), ()> {
        Ok(())
    }
}
```

### 3. SQL — Powerful Query Language

```sql
-- A complex query that would take 50 lines of code with files:

SELECT
    symbol,
    COUNT(*) as trade_count,
    SUM(quantity * price) as total_volume,
    AVG(price) as avg_price
FROM trades
WHERE timestamp > datetime('now', '-30 days')
GROUP BY symbol
HAVING total_volume > 10000
ORDER BY total_volume DESC;
```

### 4. Parallel Access

```rust
// Database manages locking itself
use std::thread;

fn main() {
    // Both threads can safely work with the database simultaneously
    let handle1 = thread::spawn(|| {
        let db = connect_to_database();
        db.insert_trade(/* ... */);  // DB locks necessary rows itself
    });

    let handle2 = thread::spawn(|| {
        let db = connect_to_database();
        db.insert_trade(/* ... */);  // Works safely in parallel
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

fn connect_to_database() -> Database {
    Database
}

impl Database {
    fn insert_trade(&self) {}
}
```

## Practical Example: Trading Journal

Here's a complete example of a system that saves trading data persistently:

```rust
use std::fs;
use std::collections::HashMap;

/// Trading journal with file persistence
/// (In a real project, use a database!)
#[derive(Debug)]
struct TradingJournal {
    trades: Vec<Trade>,
    portfolio: Portfolio,
    filename: String,
}

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    side: TradeSide,
    price: f64,
    quantity: f64,
    timestamp: i64,
}

#[derive(Debug, Clone)]
enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct Portfolio {
    cash: f64,
    positions: HashMap<String, f64>,
}

impl TradingJournal {
    /// Creates a journal or loads an existing one
    fn new(filename: &str) -> Self {
        // Try to load existing data
        if let Ok(data) = fs::read_to_string(filename) {
            if let Some(journal) = Self::deserialize(&data) {
                println!("Loaded journal: {} trades, balance ${:.2}",
                    journal.trades.len(), journal.portfolio.cash);
                return journal;
            }
        }

        // Create new journal
        println!("Created new journal");
        TradingJournal {
            trades: Vec::new(),
            portfolio: Portfolio {
                cash: 100_000.0,  // Starting capital
                positions: HashMap::new(),
            },
            filename: filename.to_string(),
        }
    }

    /// Adds a trade and saves
    fn add_trade(&mut self, symbol: &str, side: TradeSide,
                 price: f64, quantity: f64) -> Result<u64, String> {
        let cost = price * quantity;

        // Check if trade is possible
        match &side {
            TradeSide::Buy => {
                if self.portfolio.cash < cost {
                    return Err(format!("Insufficient funds: need ${:.2}, have ${:.2}",
                        cost, self.portfolio.cash));
                }
            }
            TradeSide::Sell => {
                let position = self.portfolio.positions.get(symbol).unwrap_or(&0.0);
                if *position < quantity {
                    return Err(format!("Insufficient {}: need {}, have {}",
                        symbol, quantity, position));
                }
            }
        }

        // Create trade
        let trade = Trade {
            id: self.trades.len() as u64 + 1,
            symbol: symbol.to_string(),
            side: side.clone(),
            price,
            quantity,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        // Update portfolio
        match side {
            TradeSide::Buy => {
                self.portfolio.cash -= cost;
                *self.portfolio.positions
                    .entry(symbol.to_string())
                    .or_insert(0.0) += quantity;
            }
            TradeSide::Sell => {
                self.portfolio.cash += cost;
                if let Some(pos) = self.portfolio.positions.get_mut(symbol) {
                    *pos -= quantity;
                    if *pos <= 0.0 {
                        self.portfolio.positions.remove(symbol);
                    }
                }
            }
        }

        let trade_id = trade.id;
        self.trades.push(trade);

        // IMPORTANT: save after each operation
        self.save()?;

        Ok(trade_id)
    }

    /// Saves journal to file
    fn save(&self) -> Result<(), String> {
        let data = self.serialize();
        fs::write(&self.filename, data)
            .map_err(|e| format!("Save error: {}", e))
    }

    /// Serialization to simple text format
    fn serialize(&self) -> String {
        let mut lines = Vec::new();

        // Portfolio header
        lines.push(format!("CASH:{}", self.portfolio.cash));
        for (symbol, qty) in &self.portfolio.positions {
            lines.push(format!("POS:{}:{}", symbol, qty));
        }

        // Trades
        lines.push("TRADES".to_string());
        for trade in &self.trades {
            let side_str = match trade.side {
                TradeSide::Buy => "BUY",
                TradeSide::Sell => "SELL",
            };
            lines.push(format!("{}:{}:{}:{}:{}:{}",
                trade.id, trade.symbol, side_str,
                trade.price, trade.quantity, trade.timestamp));
        }

        lines.join("\n")
    }

    /// Deserialization from text
    fn deserialize(data: &str) -> Option<Self> {
        let mut cash = 100_000.0;
        let mut positions = HashMap::new();
        let mut trades = Vec::new();
        let mut reading_trades = false;

        for line in data.lines() {
            if line.starts_with("CASH:") {
                cash = line[5..].parse().ok()?;
            } else if line.starts_with("POS:") {
                let parts: Vec<&str> = line[4..].split(':').collect();
                if parts.len() == 2 {
                    positions.insert(
                        parts[0].to_string(),
                        parts[1].parse().ok()?
                    );
                }
            } else if line == "TRADES" {
                reading_trades = true;
            } else if reading_trades {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 6 {
                    let side = match parts[2] {
                        "BUY" => TradeSide::Buy,
                        "SELL" => TradeSide::Sell,
                        _ => continue,
                    };
                    trades.push(Trade {
                        id: parts[0].parse().ok()?,
                        symbol: parts[1].to_string(),
                        side,
                        price: parts[3].parse().ok()?,
                        quantity: parts[4].parse().ok()?,
                        timestamp: parts[5].parse().ok()?,
                    });
                }
            }
        }

        Some(TradingJournal {
            trades,
            portfolio: Portfolio { cash, positions },
            filename: String::new(),
        })
    }

    /// Shows status
    fn status(&self) {
        println!("\n=== Portfolio Status ===");
        println!("Cash: ${:.2}", self.portfolio.cash);
        println!("Positions:");
        for (symbol, qty) in &self.portfolio.positions {
            println!("  {}: {:.4}", symbol, qty);
        }
        println!("Total trades: {}", self.trades.len());

        if let Some(last) = self.trades.last() {
            let side = match last.side {
                TradeSide::Buy => "Buy",
                TradeSide::Sell => "Sell",
            };
            println!("Last trade: {} {} {} @ ${:.2}",
                side, last.quantity, last.symbol, last.price);
        }
    }
}

fn main() {
    // Journal is saved to file and loaded on restart
    let mut journal = TradingJournal::new("trading_journal.dat");

    journal.status();

    // Simulate trading
    println!("\n=== New Trades ===");

    match journal.add_trade("BTC", TradeSide::Buy, 42000.0, 0.5) {
        Ok(id) => println!("Trade #{}: Bought 0.5 BTC @ $42000", id),
        Err(e) => println!("Error: {}", e),
    }

    match journal.add_trade("ETH", TradeSide::Buy, 2800.0, 5.0) {
        Ok(id) => println!("Trade #{}: Bought 5 ETH @ $2800", id),
        Err(e) => println!("Error: {}", e),
    }

    // Try to sell what we don't have
    match journal.add_trade("SOL", TradeSide::Sell, 100.0, 10.0) {
        Ok(id) => println!("Trade #{}: Sold 10 SOL", id),
        Err(e) => println!("Error: {}", e),
    }

    journal.status();

    println!("\n=== Data will be saved when the program runs again! ===");
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Persistence | Saving data between program runs |
| RAM vs Disk | RAM is temporary, disk is permanent |
| Files | Simple approach, but with limitations |
| Database | Powerful solution: indexes, transactions, concurrency |
| Transactions | Atomic operations — all or nothing |
| Indexes | Fast search without scanning all records |

## Homework

1. **Configuration Saving**: Create a `BotConfig` struct with fields for API keys, trading pairs, and limits. Implement saving/loading to a JSON file. Verify that settings persist between runs.

2. **Trade History with Search**: Extend the `TradingJournal` example:
   - Add method `find_trades_by_symbol(symbol: &str)` — search trades by ticker
   - Add method `get_total_pnl()` — calculate total P&L
   - Add method `get_trades_after(timestamp: i64)` — trades after a specific time

3. **Atomic Saving**: Modify the `save()` method:
   - First write to a temporary `.tmp` file
   - Then rename to the main file
   - This protects against data loss during write failures

4. **Think About Databases**: Which operations in your example would be easier with an SQL database? Write down 3-5 example queries you'd like to execute.

## Navigation

[← Previous day](../212-project-realtime-price-monitor/en.md) | [Next day →](../214-sqlite-embedded-database/en.md)
