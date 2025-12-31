# Day 241: Backups: Preserving History

## Trading Analogy

Imagine you're running a trading desk that has been accumulating years of valuable data: every trade executed, every price tick recorded, every portfolio rebalancing decision documented. One day, a server failure wipes out your database. Without backups, you lose not just data, but the ability to analyze past performance, audit trades for compliance, and learn from historical patterns.

In trading, backups are like having a safety vault for your trading journal. Just as professional traders keep meticulous records of their trades, a robust backup strategy ensures that:
- Your complete trading history survives any disaster
- You can reconstruct your portfolio state at any point in time
- Regulatory audits can be satisfied with historical data
- Strategy backtesting remains possible with real historical data

## What are Database Backups?

A database backup is a copy of your data that can be used to restore the original data in case of data loss. In trading systems, backups are critical because:

1. **Regulatory Compliance** — Financial regulations often require data retention for years
2. **Disaster Recovery** — Hardware failures, cyberattacks, or human errors can destroy data
3. **Point-in-Time Recovery** — Ability to restore data to a specific moment
4. **Data Migration** — Moving data between systems or environments

## Types of Backups

### 1. Full Backup
Complete copy of the entire database.

```rust
use std::fs;
use std::path::Path;
use std::io::{self, Write};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
struct Trade {
    id: u64,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct Portfolio {
    cash: f64,
    positions: Vec<(String, f64, f64)>, // (symbol, quantity, avg_price)
}

struct TradingDatabase {
    trades: Vec<Trade>,
    portfolio: Portfolio,
    backup_dir: String,
}

impl TradingDatabase {
    fn new(backup_dir: &str) -> Self {
        fs::create_dir_all(backup_dir).ok();
        TradingDatabase {
            trades: Vec::new(),
            portfolio: Portfolio {
                cash: 100_000.0,
                positions: Vec::new(),
            },
            backup_dir: backup_dir.to_string(),
        }
    }

    /// Creates a full backup of all trading data
    fn full_backup(&self) -> io::Result<String> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = format!("{}/full_backup_{}.json", self.backup_dir, timestamp);

        let backup_data = serde_json::json!({
            "backup_type": "full",
            "timestamp": Utc::now().to_rfc3339(),
            "trades_count": self.trades.len(),
            "portfolio": {
                "cash": self.portfolio.cash,
                "positions": self.portfolio.positions,
            },
            "trades": self.trades.iter().map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "symbol": t.symbol,
                    "side": t.side,
                    "quantity": t.quantity,
                    "price": t.price,
                    "timestamp": t.timestamp.to_rfc3339(),
                })
            }).collect::<Vec<_>>(),
        });

        let mut file = fs::File::create(&backup_path)?;
        file.write_all(backup_data.to_string().as_bytes())?;

        println!("Full backup created: {}", backup_path);
        println!("  - {} trades backed up", self.trades.len());
        println!("  - Portfolio cash: ${:.2}", self.portfolio.cash);

        Ok(backup_path)
    }
}
```

### 2. Incremental Backup
Only backs up data changed since the last backup.

```rust
use std::collections::HashMap;

struct IncrementalBackupManager {
    last_backup_trade_id: u64,
    backup_history: Vec<BackupRecord>,
}

#[derive(Debug, Clone)]
struct BackupRecord {
    backup_id: u64,
    backup_type: String,
    timestamp: DateTime<Utc>,
    trades_from: u64,
    trades_to: u64,
    file_path: String,
}

impl IncrementalBackupManager {
    fn new() -> Self {
        IncrementalBackupManager {
            last_backup_trade_id: 0,
            backup_history: Vec::new(),
        }
    }

    /// Creates an incremental backup with only new trades
    fn incremental_backup(&mut self, db: &TradingDatabase) -> io::Result<Option<String>> {
        // Find trades newer than last backup
        let new_trades: Vec<&Trade> = db.trades
            .iter()
            .filter(|t| t.id > self.last_backup_trade_id)
            .collect();

        if new_trades.is_empty() {
            println!("No new trades since last backup");
            return Ok(None);
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = format!(
            "{}/incremental_backup_{}.json",
            db.backup_dir, timestamp
        );

        let backup_data = serde_json::json!({
            "backup_type": "incremental",
            "timestamp": Utc::now().to_rfc3339(),
            "previous_trade_id": self.last_backup_trade_id,
            "new_trades_count": new_trades.len(),
            "trades": new_trades.iter().map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "symbol": t.symbol,
                    "side": t.side,
                    "quantity": t.quantity,
                    "price": t.price,
                    "timestamp": t.timestamp.to_rfc3339(),
                })
            }).collect::<Vec<_>>(),
        });

        let mut file = fs::File::create(&backup_path)?;
        file.write_all(backup_data.to_string().as_bytes())?;

        // Update tracking
        let max_trade_id = new_trades.iter().map(|t| t.id).max().unwrap_or(0);

        self.backup_history.push(BackupRecord {
            backup_id: self.backup_history.len() as u64 + 1,
            backup_type: "incremental".to_string(),
            timestamp: Utc::now(),
            trades_from: self.last_backup_trade_id + 1,
            trades_to: max_trade_id,
            file_path: backup_path.clone(),
        });

        self.last_backup_trade_id = max_trade_id;

        println!("Incremental backup created: {}", backup_path);
        println!("  - {} new trades backed up", new_trades.len());

        Ok(Some(backup_path))
    }
}
```

### 3. Differential Backup
Backs up all changes since the last full backup.

```rust
struct DifferentialBackupManager {
    last_full_backup_trade_id: u64,
    last_full_backup_time: Option<DateTime<Utc>>,
}

impl DifferentialBackupManager {
    fn new() -> Self {
        DifferentialBackupManager {
            last_full_backup_trade_id: 0,
            last_full_backup_time: None,
        }
    }

    fn record_full_backup(&mut self, max_trade_id: u64) {
        self.last_full_backup_trade_id = max_trade_id;
        self.last_full_backup_time = Some(Utc::now());
    }

    /// Creates differential backup (all changes since last full backup)
    fn differential_backup(&self, db: &TradingDatabase) -> io::Result<String> {
        let trades_since_full: Vec<&Trade> = db.trades
            .iter()
            .filter(|t| t.id > self.last_full_backup_trade_id)
            .collect();

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = format!(
            "{}/differential_backup_{}.json",
            db.backup_dir, timestamp
        );

        let backup_data = serde_json::json!({
            "backup_type": "differential",
            "timestamp": Utc::now().to_rfc3339(),
            "base_full_backup_trade_id": self.last_full_backup_trade_id,
            "base_full_backup_time": self.last_full_backup_time,
            "trades_count": trades_since_full.len(),
            "trades": trades_since_full.iter().map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "symbol": t.symbol,
                    "side": t.side,
                    "quantity": t.quantity,
                    "price": t.price,
                    "timestamp": t.timestamp.to_rfc3339(),
                })
            }).collect::<Vec<_>>(),
        });

        let mut file = fs::File::create(&backup_path)?;
        file.write_all(backup_data.to_string().as_bytes())?;

        println!("Differential backup created: {}", backup_path);
        println!("  - {} trades since last full backup", trades_since_full.len());

        Ok(backup_path)
    }
}
```

## Point-in-Time Recovery for Trading

One of the most valuable backup features for trading systems is the ability to restore data to a specific point in time.

```rust
use std::collections::BTreeMap;

struct PointInTimeRecovery {
    /// Snapshots indexed by timestamp
    snapshots: BTreeMap<DateTime<Utc>, PortfolioSnapshot>,
    /// Transaction log for replay
    transaction_log: Vec<TransactionLogEntry>,
}

#[derive(Debug, Clone)]
struct PortfolioSnapshot {
    timestamp: DateTime<Utc>,
    cash: f64,
    positions: HashMap<String, (f64, f64)>, // symbol -> (quantity, avg_price)
    total_value: f64,
}

#[derive(Debug, Clone)]
struct TransactionLogEntry {
    timestamp: DateTime<Utc>,
    action: String,
    symbol: Option<String>,
    quantity: Option<f64>,
    price: Option<f64>,
    cash_change: f64,
}

impl PointInTimeRecovery {
    fn new() -> Self {
        PointInTimeRecovery {
            snapshots: BTreeMap::new(),
            transaction_log: Vec::new(),
        }
    }

    /// Takes a snapshot of current portfolio state
    fn take_snapshot(&mut self, portfolio: &Portfolio) {
        let mut positions_map = HashMap::new();
        for (symbol, qty, price) in &portfolio.positions {
            positions_map.insert(symbol.clone(), (*qty, *price));
        }

        let total_value = portfolio.cash + portfolio.positions
            .iter()
            .map(|(_, qty, price)| qty * price)
            .sum::<f64>();

        let snapshot = PortfolioSnapshot {
            timestamp: Utc::now(),
            cash: portfolio.cash,
            positions: positions_map,
            total_value,
        };

        self.snapshots.insert(snapshot.timestamp, snapshot);
        println!("Snapshot taken at {}", Utc::now());
    }

    /// Logs a transaction for replay capability
    fn log_transaction(
        &mut self,
        action: &str,
        symbol: Option<&str>,
        quantity: Option<f64>,
        price: Option<f64>,
        cash_change: f64,
    ) {
        self.transaction_log.push(TransactionLogEntry {
            timestamp: Utc::now(),
            action: action.to_string(),
            symbol: symbol.map(|s| s.to_string()),
            quantity,
            price,
            cash_change,
        });
    }

    /// Recovers portfolio state to a specific point in time
    fn recover_to_point(&self, target_time: DateTime<Utc>) -> Option<PortfolioSnapshot> {
        // Find the latest snapshot before target time
        let base_snapshot = self.snapshots
            .range(..=target_time)
            .last()
            .map(|(_, s)| s.clone())?;

        // Replay transactions between snapshot and target time
        let mut recovered = base_snapshot.clone();

        for entry in &self.transaction_log {
            if entry.timestamp > base_snapshot.timestamp && entry.timestamp <= target_time {
                // Apply transaction
                recovered.cash += entry.cash_change;

                if let (Some(symbol), Some(qty), Some(price)) =
                    (&entry.symbol, entry.quantity, entry.price)
                {
                    let position = recovered.positions
                        .entry(symbol.clone())
                        .or_insert((0.0, 0.0));

                    match entry.action.as_str() {
                        "BUY" => {
                            let total_qty = position.0 + qty;
                            let total_cost = position.0 * position.1 + qty * price;
                            position.1 = if total_qty > 0.0 { total_cost / total_qty } else { 0.0 };
                            position.0 = total_qty;
                        }
                        "SELL" => {
                            position.0 -= qty;
                            if position.0 <= 0.0 {
                                recovered.positions.remove(symbol);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Recalculate total value
        recovered.total_value = recovered.cash + recovered.positions
            .values()
            .map(|(qty, price)| qty * price)
            .sum::<f64>();

        recovered.timestamp = target_time;

        Some(recovered)
    }
}
```

## Automated Backup Scheduling

Trading systems need automated, reliable backup schedules.

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct BackupScheduler {
    db: Arc<Mutex<TradingDatabase>>,
    incremental_manager: Arc<Mutex<IncrementalBackupManager>>,
    full_backup_interval_hours: u64,
    incremental_backup_interval_minutes: u64,
    running: Arc<Mutex<bool>>,
}

impl BackupScheduler {
    fn new(
        db: Arc<Mutex<TradingDatabase>>,
        full_interval_hours: u64,
        incremental_interval_minutes: u64,
    ) -> Self {
        BackupScheduler {
            db,
            incremental_manager: Arc::new(Mutex::new(IncrementalBackupManager::new())),
            full_backup_interval_hours: full_interval_hours,
            incremental_backup_interval_minutes: incremental_interval_minutes,
            running: Arc::new(Mutex::new(false)),
        }
    }

    fn start(&self) {
        *self.running.lock().unwrap() = true;

        // Full backup thread
        let db_full = Arc::clone(&self.db);
        let running_full = Arc::clone(&self.running);
        let full_interval = self.full_backup_interval_hours;

        thread::spawn(move || {
            while *running_full.lock().unwrap() {
                thread::sleep(Duration::from_secs(full_interval * 3600));

                if *running_full.lock().unwrap() {
                    let db = db_full.lock().unwrap();
                    match db.full_backup() {
                        Ok(path) => println!("[Scheduler] Full backup completed: {}", path),
                        Err(e) => eprintln!("[Scheduler] Full backup failed: {}", e),
                    }
                }
            }
        });

        // Incremental backup thread
        let db_inc = Arc::clone(&self.db);
        let inc_manager = Arc::clone(&self.incremental_manager);
        let running_inc = Arc::clone(&self.running);
        let inc_interval = self.incremental_backup_interval_minutes;

        thread::spawn(move || {
            while *running_inc.lock().unwrap() {
                thread::sleep(Duration::from_secs(inc_interval * 60));

                if *running_inc.lock().unwrap() {
                    let db = db_inc.lock().unwrap();
                    let mut manager = inc_manager.lock().unwrap();
                    match manager.incremental_backup(&db) {
                        Ok(Some(path)) => {
                            println!("[Scheduler] Incremental backup completed: {}", path);
                        }
                        Ok(None) => {
                            println!("[Scheduler] No new data for incremental backup");
                        }
                        Err(e) => eprintln!("[Scheduler] Incremental backup failed: {}", e),
                    }
                }
            }
        });

        println!("Backup scheduler started:");
        println!("  - Full backup every {} hours", full_interval);
        println!("  - Incremental backup every {} minutes", inc_interval);
    }

    fn stop(&self) {
        *self.running.lock().unwrap() = false;
        println!("Backup scheduler stopped");
    }
}
```

## Backup Verification and Integrity

Always verify your backups can be restored successfully.

```rust
use std::fs::File;
use std::io::Read;

struct BackupVerifier {
    backup_dir: String,
}

#[derive(Debug)]
struct VerificationResult {
    backup_path: String,
    is_valid: bool,
    trades_count: usize,
    checksum: String,
    errors: Vec<String>,
}

impl BackupVerifier {
    fn new(backup_dir: &str) -> Self {
        BackupVerifier {
            backup_dir: backup_dir.to_string(),
        }
    }

    /// Verifies a backup file integrity and parsability
    fn verify_backup(&self, backup_path: &str) -> VerificationResult {
        let mut errors = Vec::new();
        let mut trades_count = 0;

        // Read file
        let mut file = match File::open(backup_path) {
            Ok(f) => f,
            Err(e) => {
                return VerificationResult {
                    backup_path: backup_path.to_string(),
                    is_valid: false,
                    trades_count: 0,
                    checksum: String::new(),
                    errors: vec![format!("Cannot open file: {}", e)],
                };
            }
        };

        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            return VerificationResult {
                backup_path: backup_path.to_string(),
                is_valid: false,
                trades_count: 0,
                checksum: String::new(),
                errors: vec![format!("Cannot read file: {}", e)],
            };
        }

        // Calculate checksum
        let checksum = format!("{:x}", md5::compute(&contents));

        // Parse JSON
        let json: serde_json::Value = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(e) => {
                return VerificationResult {
                    backup_path: backup_path.to_string(),
                    is_valid: false,
                    trades_count: 0,
                    checksum,
                    errors: vec![format!("Invalid JSON: {}", e)],
                };
            }
        };

        // Verify structure
        if json.get("backup_type").is_none() {
            errors.push("Missing backup_type field".to_string());
        }

        if json.get("timestamp").is_none() {
            errors.push("Missing timestamp field".to_string());
        }

        if let Some(trades) = json.get("trades") {
            if let Some(arr) = trades.as_array() {
                trades_count = arr.len();

                // Verify each trade has required fields
                for (i, trade) in arr.iter().enumerate() {
                    if trade.get("id").is_none() {
                        errors.push(format!("Trade {} missing id", i));
                    }
                    if trade.get("symbol").is_none() {
                        errors.push(format!("Trade {} missing symbol", i));
                    }
                    if trade.get("price").is_none() {
                        errors.push(format!("Trade {} missing price", i));
                    }
                }
            }
        }

        VerificationResult {
            backup_path: backup_path.to_string(),
            is_valid: errors.is_empty(),
            trades_count,
            checksum,
            errors,
        }
    }

    /// Verifies all backups in the backup directory
    fn verify_all_backups(&self) -> Vec<VerificationResult> {
        let mut results = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.backup_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        let path = entry.path().to_string_lossy().to_string();
                        results.push(self.verify_backup(&path));
                    }
                }
            }
        }

        results
    }
}
```

## Complete Trading Backup System Example

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Utc;

fn main() {
    // Initialize trading database
    let db = Arc::new(Mutex::new(TradingDatabase::new("./trading_backups")));

    // Add some sample trades
    {
        let mut db = db.lock().unwrap();

        // Simulate trading activity
        let trades = vec![
            ("BTC", "BUY", 0.5, 42000.0),
            ("ETH", "BUY", 10.0, 2800.0),
            ("BTC", "SELL", 0.2, 43500.0),
            ("SOL", "BUY", 100.0, 95.0),
            ("ETH", "SELL", 5.0, 2950.0),
        ];

        for (i, (symbol, side, qty, price)) in trades.iter().enumerate() {
            db.trades.push(Trade {
                id: i as u64 + 1,
                symbol: symbol.to_string(),
                side: side.to_string(),
                quantity: *qty,
                price: *price,
                timestamp: Utc::now(),
            });

            // Update portfolio
            let cost = qty * price;
            if *side == "BUY" {
                db.portfolio.cash -= cost;
                db.portfolio.positions.push((symbol.to_string(), *qty, *price));
            } else {
                db.portfolio.cash += cost;
            }
        }
    }

    // Create full backup
    {
        let db = db.lock().unwrap();
        match db.full_backup() {
            Ok(path) => println!("\nFull backup created successfully: {}", path),
            Err(e) => eprintln!("Backup failed: {}", e),
        }
    }

    // Initialize incremental backup manager
    let mut inc_manager = IncrementalBackupManager::new();
    inc_manager.last_backup_trade_id = 5; // Assume we backed up first 5 trades

    // Add more trades
    {
        let mut db = db.lock().unwrap();
        for i in 6..=10 {
            db.trades.push(Trade {
                id: i,
                symbol: "DOGE".to_string(),
                side: "BUY".to_string(),
                quantity: 1000.0,
                price: 0.08,
                timestamp: Utc::now(),
            });
        }
    }

    // Create incremental backup
    {
        let db = db.lock().unwrap();
        match inc_manager.incremental_backup(&db) {
            Ok(Some(path)) => println!("Incremental backup created: {}", path),
            Ok(None) => println!("No new data to backup"),
            Err(e) => eprintln!("Incremental backup failed: {}", e),
        }
    }

    // Verify backups
    let verifier = BackupVerifier::new("./trading_backups");
    let results = verifier.verify_all_backups();

    println!("\n=== Backup Verification Report ===");
    for result in results {
        println!("\nFile: {}", result.backup_path);
        println!("  Valid: {}", result.is_valid);
        println!("  Trades: {}", result.trades_count);
        println!("  Checksum: {}", result.checksum);
        if !result.errors.is_empty() {
            println!("  Errors:");
            for error in &result.errors {
                println!("    - {}", error);
            }
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Full Backup | Complete copy of all database data |
| Incremental Backup | Only changes since last backup (any type) |
| Differential Backup | All changes since last full backup |
| Point-in-Time Recovery | Restore data to a specific moment |
| Transaction Log | Record of all operations for replay |
| Backup Verification | Confirming backup integrity and restorability |
| Backup Scheduling | Automated, regular backup creation |

## Homework

1. **Backup Rotation Policy**: Implement a backup rotation system that:
   - Keeps the last 7 daily backups
   - Keeps the last 4 weekly backups
   - Keeps the last 12 monthly backups
   - Automatically deletes older backups

2. **Compressed Backups**: Extend the backup system to:
   - Compress backup files using gzip or zstd
   - Calculate and store checksums for integrity verification
   - Implement decompression during restore

3. **Remote Backup Sync**: Create a backup system that:
   - Uploads backups to a remote location (simulate with a local "remote" directory)
   - Tracks which backups have been synced
   - Implements retry logic for failed uploads
   - Provides a sync status report

4. **Backup Restore Testing**: Implement an automated restore test that:
   - Takes a backup
   - Restores it to a separate test database
   - Verifies all trades and portfolio state match
   - Reports any discrepancies
   - Runs this verification on a schedule

## Navigation

[← Previous day](../240-migrations-evolving-schema/en.md) | [Next day →](../242-replication-data-copies/en.md)
