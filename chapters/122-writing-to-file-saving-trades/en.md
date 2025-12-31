# Day 122: Writing to File — Saving Trades

## Trading Analogy

Every trader keeps a trade journal. At the end of each trading day, you record all completed operations: time, asset, entry price, exit price, volume, result. Without this journal, it's impossible to analyze your trading, find mistakes, and improve your strategy.

In programming, writing to a file is the digital equivalent of such a journal. We **save** data to disk so it persists after the program closes.

## Basic File Writing

```rust
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let mut file = File::create("trades.txt")?;

    writeln!(file, "BTCUSDT,BUY,42000.00,0.5")?;
    writeln!(file, "ETHUSDT,SELL,2500.00,2.0")?;

    println!("Trades saved to trades.txt");
    Ok(())
}
```

**Important:**
- `File::create` creates a new file or overwrites an existing one
- `?` operator propagates errors upward
- `writeln!` writes a string with a newline

## Writing Trade Structures

```rust
use std::fs::File;
use std::io::Write;

struct Trade {
    symbol: String,
    side: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

impl Trade {
    fn to_csv_line(&self) -> String {
        format!(
            "{},{},{:.2},{:.2},{:.4},{:.2}",
            self.symbol, self.side, self.entry_price,
            self.exit_price, self.quantity, self.pnl
        )
    }
}

fn main() -> std::io::Result<()> {
    let trades = vec![
        Trade {
            symbol: String::from("BTCUSDT"),
            side: String::from("LONG"),
            entry_price: 42000.0,
            exit_price: 43500.0,
            quantity: 0.5,
            pnl: 750.0,
        },
        Trade {
            symbol: String::from("ETHUSDT"),
            side: String::from("SHORT"),
            entry_price: 2500.0,
            exit_price: 2400.0,
            quantity: 2.0,
            pnl: 200.0,
        },
    ];

    let mut file = File::create("trade_log.csv")?;

    // CSV header
    writeln!(file, "symbol,side,entry_price,exit_price,quantity,pnl")?;

    // Write each trade
    for trade in &trades {
        writeln!(file, "{}", trade.to_csv_line())?;
    }

    println!("Saved {} trades to trade_log.csv", trades.len());
    Ok(())
}
```

## Appending to Existing File

```rust
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> std::io::Result<()> {
    // Open file for appending (create if doesn't exist)
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("trade_journal.txt")?;

    let new_trade = "2024-01-15,BTCUSDT,LONG,43000.00,43800.00,0.3,+240.00";
    writeln!(file, "{}", new_trade)?;

    println!("Trade appended to journal");
    Ok(())
}
```

## Buffered Writing

For many writes, use buffered output:

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> std::io::Result<()> {
    let file = File::create("price_history.csv")?;
    let mut writer = BufWriter::new(file);

    // Header
    writeln!(writer, "timestamp,price,volume")?;

    // Simulate writing large amounts of data
    let prices = vec![
        (1705305600, 42150.50, 125.5),
        (1705305660, 42165.75, 89.2),
        (1705305720, 42140.00, 210.8),
        (1705305780, 42180.25, 156.3),
        (1705305840, 42200.00, 178.9),
    ];

    for (ts, price, volume) in prices {
        writeln!(writer, "{},{:.2},{:.1}", ts, price, volume)?;
    }

    // Buffer flushes automatically on drop,
    // but can be done explicitly
    writer.flush()?;

    println!("Price history saved with buffering");
    Ok(())
}
```

## Saving in JSON Format

```rust
use std::fs::File;
use std::io::Write;

struct Portfolio {
    account_id: String,
    balance: f64,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

impl Position {
    fn unrealized_pnl(&self) -> f64 {
        (self.current_price - self.entry_price) * self.quantity
    }
}

fn portfolio_to_json(portfolio: &Portfolio) -> String {
    let mut json = String::from("{\n");
    json.push_str(&format!("  \"account_id\": \"{}\",\n", portfolio.account_id));
    json.push_str(&format!("  \"balance\": {:.2},\n", portfolio.balance));
    json.push_str("  \"positions\": [\n");

    for (i, pos) in portfolio.positions.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"symbol\": \"{}\",\n", pos.symbol));
        json.push_str(&format!("      \"quantity\": {:.4},\n", pos.quantity));
        json.push_str(&format!("      \"entry_price\": {:.2},\n", pos.entry_price));
        json.push_str(&format!("      \"current_price\": {:.2},\n", pos.current_price));
        json.push_str(&format!("      \"unrealized_pnl\": {:.2}\n", pos.unrealized_pnl()));
        json.push_str("    }");
        if i < portfolio.positions.len() - 1 {
            json.push_str(",");
        }
        json.push_str("\n");
    }

    json.push_str("  ]\n");
    json.push_str("}");
    json
}

fn main() -> std::io::Result<()> {
    let portfolio = Portfolio {
        account_id: String::from("ACC-001"),
        balance: 50000.0,
        positions: vec![
            Position {
                symbol: String::from("BTCUSDT"),
                quantity: 0.5,
                entry_price: 42000.0,
                current_price: 43500.0,
            },
            Position {
                symbol: String::from("ETHUSDT"),
                quantity: 5.0,
                entry_price: 2400.0,
                current_price: 2550.0,
            },
        ],
    };

    let json = portfolio_to_json(&portfolio);

    let mut file = File::create("portfolio.json")?;
    file.write_all(json.as_bytes())?;

    println!("Portfolio saved to portfolio.json");
    Ok(())
}
```

## Error Handling for Writing

```rust
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn save_trades(trades: &[String], filename: &str) -> Result<usize, String> {
    // Check if there's anything to save
    if trades.is_empty() {
        return Err(String::from("No trades to save"));
    }

    // Check directory
    let path = Path::new(filename);
    if let Some(parent) = path.parent() {
        if !parent.exists() && !parent.as_os_str().is_empty() {
            return Err(format!("Directory does not exist: {:?}", parent));
        }
    }

    // Try to create and write file
    let mut file = File::create(filename)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut count = 0;
    for trade in trades {
        writeln!(file, "{}", trade)
            .map_err(|e| format!("Failed to write trade: {}", e))?;
        count += 1;
    }

    Ok(count)
}

fn main() {
    let trades = vec![
        String::from("BTCUSDT,BUY,42000,0.5"),
        String::from("ETHUSDT,SELL,2500,2.0"),
        String::from("SOLUSDT,BUY,95,10.0"),
    ];

    match save_trades(&trades, "my_trades.csv") {
        Ok(count) => println!("Successfully saved {} trades", count),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Example with empty list
    let empty: Vec<String> = vec![];
    match save_trades(&empty, "empty.csv") {
        Ok(count) => println!("Saved {} trades", count),
        Err(e) => eprintln!("Expected error: {}", e),
    }
}
```

## Practical Example: Complete Trade Journal

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Clone)]
struct TradeEntry {
    timestamp: String,
    symbol: String,
    side: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    commission: f64,
}

impl TradeEntry {
    fn gross_pnl(&self) -> f64 {
        let multiplier = if self.side == "LONG" { 1.0 } else { -1.0 };
        (self.exit_price - self.entry_price) * self.quantity * multiplier
    }

    fn net_pnl(&self) -> f64 {
        self.gross_pnl() - self.commission
    }

    fn to_csv(&self) -> String {
        format!(
            "{},{},{},{:.2},{:.2},{:.4},{:.2},{:.2},{:.2}",
            self.timestamp,
            self.symbol,
            self.side,
            self.entry_price,
            self.exit_price,
            self.quantity,
            self.commission,
            self.gross_pnl(),
            self.net_pnl()
        )
    }
}

struct TradeJournal {
    filename: String,
    trades: Vec<TradeEntry>,
}

impl TradeJournal {
    fn new(filename: &str) -> Self {
        TradeJournal {
            filename: String::from(filename),
            trades: Vec::new(),
        }
    }

    fn add_trade(&mut self, trade: TradeEntry) {
        self.trades.push(trade);
    }

    fn save(&self) -> std::io::Result<()> {
        let file = File::create(&self.filename)?;
        let mut writer = BufWriter::new(file);

        // Header
        writeln!(writer, "timestamp,symbol,side,entry,exit,qty,commission,gross_pnl,net_pnl")?;

        // Data
        for trade in &self.trades {
            writeln!(writer, "{}", trade.to_csv())?;
        }

        writer.flush()?;
        Ok(())
    }

    fn save_summary(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;

        let total_trades = self.trades.len();
        let winning: Vec<_> = self.trades.iter().filter(|t| t.net_pnl() > 0.0).collect();
        let losing: Vec<_> = self.trades.iter().filter(|t| t.net_pnl() < 0.0).collect();

        let total_pnl: f64 = self.trades.iter().map(|t| t.net_pnl()).sum();
        let total_commission: f64 = self.trades.iter().map(|t| t.commission).sum();

        let win_rate = if total_trades > 0 {
            (winning.len() as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        writeln!(file, "╔════════════════════════════════════╗")?;
        writeln!(file, "║       TRADING JOURNAL SUMMARY      ║")?;
        writeln!(file, "╠════════════════════════════════════╣")?;
        writeln!(file, "║ Total Trades:      {:>15} ║", total_trades)?;
        writeln!(file, "║ Winning Trades:    {:>15} ║", winning.len())?;
        writeln!(file, "║ Losing Trades:     {:>15} ║", losing.len())?;
        writeln!(file, "║ Win Rate:          {:>14.1}% ║", win_rate)?;
        writeln!(file, "║ Total PnL:        ${:>14.2} ║", total_pnl)?;
        writeln!(file, "║ Total Commission: ${:>14.2} ║", total_commission)?;
        writeln!(file, "╚════════════════════════════════════╝")?;

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let mut journal = TradeJournal::new("trading_journal.csv");

    journal.add_trade(TradeEntry {
        timestamp: String::from("2024-01-15 09:30:00"),
        symbol: String::from("BTCUSDT"),
        side: String::from("LONG"),
        entry_price: 42000.0,
        exit_price: 43200.0,
        quantity: 0.5,
        commission: 42.6,
    });

    journal.add_trade(TradeEntry {
        timestamp: String::from("2024-01-15 11:45:00"),
        symbol: String::from("ETHUSDT"),
        side: String::from("SHORT"),
        entry_price: 2500.0,
        exit_price: 2450.0,
        quantity: 2.0,
        commission: 9.9,
    });

    journal.add_trade(TradeEntry {
        timestamp: String::from("2024-01-15 14:20:00"),
        symbol: String::from("SOLUSDT"),
        side: String::from("LONG"),
        entry_price: 95.0,
        exit_price: 92.0,
        quantity: 10.0,
        commission: 1.87,
    });

    journal.save()?;
    journal.save_summary("summary.txt")?;

    println!("Journal and summary saved successfully!");
    Ok(())
}
```

## Atomic Writing (Safe Saving)

```rust
use std::fs::{self, File};
use std::io::Write;

fn save_atomically(data: &str, filename: &str) -> std::io::Result<()> {
    let temp_filename = format!("{}.tmp", filename);

    // Write to temporary file
    {
        let mut file = File::create(&temp_filename)?;
        file.write_all(data.as_bytes())?;
        file.sync_all()?;  // Ensure data is written to disk
    }

    // Atomically rename
    fs::rename(&temp_filename, filename)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let portfolio_data = "BTC: 0.5\nETH: 5.0\nSOL: 100.0";

    save_atomically(portfolio_data, "portfolio_backup.txt")?;

    println!("Portfolio saved atomically");
    Ok(())
}
```

## What We Learned

| Operation | Method | When to Use |
|-----------|--------|-------------|
| Create file | `File::create` | New file or overwrite |
| Append | `OpenOptions::append` | Journal, trade log |
| Buffering | `BufWriter` | Many small writes |
| CSV format | `writeln!` with format | Data export |
| JSON format | Manual or serde | API, web services |
| Atomic write | temp + rename | Critical data |

## Practical Exercises

1. **Export Trades to CSV**
   Write a function that takes a vector of trades and saves them to a CSV file with headers.

2. **Order Journal**
   Create a function `append_order` that adds a new order to the end of a file without overwriting existing data.

3. **Backup**
   Implement a function that saves portfolio state and automatically creates a backup of the previous version.

4. **Daily Trading Report**
   Write a function that generates a text report with daily summaries: number of trades, total PnL, best and worst trades.

## Homework

1. Create a `PriceLogger` struct that writes prices to a file in real-time with buffering, flushing the buffer every N writes.

2. Implement a log rotation system: when a file exceeds a certain size, rename it with a timestamp suffix and start a new file.

3. Write a function `export_trades_report(trades: &[Trade], format: &str)` that exports trades in the specified format (csv, json, txt).

4. Create an auto-save mechanism: every N operations or after a certain interval, data is automatically saved to disk.

## Navigation

[← Previous day](../121-reading-from-file-parsing-market-data/en.md) | [Next day →](../123-file-error-handling-corrupted-data/en.md)
