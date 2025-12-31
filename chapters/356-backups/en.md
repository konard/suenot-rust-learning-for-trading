# Day 356: Backups

## Trading Analogy

Imagine you're managing a large hedge fund. Over years of work, you've accumulated invaluable data: history of all trades, strategy settings, trained machine learning models, risk management configurations. One morning you come to the office and discover that the data server has burned down.

**Without backups:**
- All trading strategies lost
- No P&L history for investor reporting
- Impossible to recover positions
- Clients leave, reputation destroyed

**With backups:**
- Recovery in an hour
- All data intact
- Business continues operating
- Clients didn't even notice the problem

| Trading | Backup |
|---------|--------|
| **Stop-loss** | Backup — protection against data loss |
| **Hedging** | Replication — copies in different locations |
| **Diversification** | Different backup types (full, incremental) |
| **Insurance** | Recovery testing |
| **Crisis plan** | Disaster Recovery Plan |

In trading, we're always prepared for the worst. The same should apply to data.

## Backup Strategies

In Rust, we can create a robust backup system for trading data:

```rust
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write, BufReader, BufWriter};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;

/// Types of backups
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum BackupType {
    /// Full backup — all data
    Full,
    /// Incremental — only changes since last backup
    Incremental,
    /// Differential — changes since last full backup
    Differential,
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupMetadata {
    id: String,
    backup_type: BackupType,
    created_at: DateTime<Utc>,
    size_bytes: u64,
    files_count: usize,
    checksum: String,
    parent_backup_id: Option<String>,
    description: String,
}

/// Trading data configuration for backup
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TradingDataBackup {
    /// History of all orders
    orders_history: Vec<Order>,
    /// Current open positions
    open_positions: Vec<Position>,
    /// Strategy settings
    strategy_configs: Vec<StrategyConfig>,
    /// Risk management parameters
    risk_parameters: RiskParameters,
    /// Creation timestamp
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    status: String,
    created_at: DateTime<Utc>,
    executed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    symbol: String,
    side: String,
    size: f64,
    entry_price: f64,
    unrealized_pnl: f64,
    opened_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StrategyConfig {
    name: String,
    parameters: std::collections::HashMap<String, String>,
    enabled: bool,
    last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RiskParameters {
    max_position_size: f64,
    max_daily_loss: f64,
    max_drawdown_percent: f64,
    position_limits: std::collections::HashMap<String, f64>,
}

/// Backup manager for trading system
struct BackupManager {
    backup_dir: PathBuf,
    retention_days: u32,
    compression_level: u32,
}

impl BackupManager {
    fn new(backup_dir: &Path, retention_days: u32) -> io::Result<Self> {
        fs::create_dir_all(backup_dir)?;

        Ok(BackupManager {
            backup_dir: backup_dir.to_path_buf(),
            retention_days,
            compression_level: 6, // Balance between speed and size
        })
    }

    /// Create full backup of trading data
    fn create_full_backup(
        &self,
        data: &TradingDataBackup,
        description: &str,
    ) -> io::Result<BackupMetadata> {
        let backup_id = format!(
            "full_{}",
            Utc::now().format("%Y%m%d_%H%M%S")
        );
        let backup_path = self.backup_dir.join(format!("{}.backup.gz", backup_id));

        println!("Creating full backup: {}", backup_id);

        // Serialize data
        let json_data = serde_json::to_vec(data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Calculate checksum before compression
        let checksum = Self::calculate_checksum(&json_data);

        // Compress and write
        let file = File::create(&backup_path)?;
        let mut encoder = GzEncoder::new(
            BufWriter::new(file),
            Compression::new(self.compression_level),
        );
        encoder.write_all(&json_data)?;
        encoder.finish()?;

        let size_bytes = fs::metadata(&backup_path)?.len();

        let metadata = BackupMetadata {
            id: backup_id,
            backup_type: BackupType::Full,
            created_at: Utc::now(),
            size_bytes,
            files_count: 1,
            checksum,
            parent_backup_id: None,
            description: description.to_string(),
        };

        // Save metadata separately
        self.save_metadata(&metadata)?;

        println!(
            "Backup created: {} ({} bytes, compression: {:.1}%)",
            metadata.id,
            size_bytes,
            (1.0 - size_bytes as f64 / json_data.len() as f64) * 100.0
        );

        Ok(metadata)
    }

    /// Restore from backup
    fn restore_from_backup(
        &self,
        backup_id: &str,
    ) -> io::Result<TradingDataBackup> {
        let backup_path = self.backup_dir.join(format!("{}.backup.gz", backup_id));

        println!("Restoring from backup: {}", backup_id);

        // Read and decompress
        let file = File::open(&backup_path)?;
        let mut decoder = GzDecoder::new(BufReader::new(file));
        let mut json_data = Vec::new();
        decoder.read_to_end(&mut json_data)?;

        // Verify checksum
        let metadata = self.load_metadata(backup_id)?;
        let current_checksum = Self::calculate_checksum(&json_data);

        if current_checksum != metadata.checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Checksum mismatch! Backup is corrupted.",
            ));
        }

        // Deserialize
        let data: TradingDataBackup = serde_json::from_slice(&json_data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        println!(
            "Restored: {} orders, {} positions, {} strategies",
            data.orders_history.len(),
            data.open_positions.len(),
            data.strategy_configs.len()
        );

        Ok(data)
    }

    /// List all available backups
    fn list_backups(&self) -> io::Result<Vec<BackupMetadata>> {
        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "meta") {
                let content = fs::read_to_string(&path)?;
                if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&content) {
                    backups.push(metadata);
                }
            }
        }

        // Sort by date (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Delete old backups (retention policy)
    fn cleanup_old_backups(&self) -> io::Result<usize> {
        let cutoff_date = Utc::now() - Duration::days(self.retention_days as i64);
        let mut deleted_count = 0;

        for metadata in self.list_backups()? {
            if metadata.created_at < cutoff_date {
                let backup_path = self.backup_dir.join(
                    format!("{}.backup.gz", metadata.id)
                );
                let meta_path = self.backup_dir.join(
                    format!("{}.meta", metadata.id)
                );

                if backup_path.exists() {
                    fs::remove_file(&backup_path)?;
                }
                if meta_path.exists() {
                    fs::remove_file(&meta_path)?;
                }

                println!("Deleted old backup: {}", metadata.id);
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Verify backup integrity
    fn verify_backup(&self, backup_id: &str) -> io::Result<bool> {
        let backup_path = self.backup_dir.join(format!("{}.backup.gz", backup_id));
        let metadata = self.load_metadata(backup_id)?;

        // Read and decompress for verification
        let file = File::open(&backup_path)?;
        let mut decoder = GzDecoder::new(BufReader::new(file));
        let mut json_data = Vec::new();
        decoder.read_to_end(&mut json_data)?;

        let current_checksum = Self::calculate_checksum(&json_data);

        Ok(current_checksum == metadata.checksum)
    }

    fn calculate_checksum(data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn save_metadata(&self, metadata: &BackupMetadata) -> io::Result<()> {
        let meta_path = self.backup_dir.join(format!("{}.meta", metadata.id));
        let content = serde_json::to_string_pretty(metadata)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(meta_path, content)
    }

    fn load_metadata(&self, backup_id: &str) -> io::Result<BackupMetadata> {
        let meta_path = self.backup_dir.join(format!("{}.meta", backup_id));
        let content = fs::read_to_string(meta_path)?;
        serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

fn main() -> io::Result<()> {
    println!("=== Trading Data Backup System ===\n");

    // Create test trading data
    let trading_data = TradingDataBackup {
        orders_history: vec![
            Order {
                id: "ORD-001".to_string(),
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                price: 50000.0,
                quantity: 0.5,
                status: "FILLED".to_string(),
                created_at: Utc::now() - Duration::hours(2),
                executed_at: Some(Utc::now() - Duration::hours(2)),
            },
            Order {
                id: "ORD-002".to_string(),
                symbol: "ETHUSDT".to_string(),
                side: "SELL".to_string(),
                price: 3000.0,
                quantity: 2.0,
                status: "FILLED".to_string(),
                created_at: Utc::now() - Duration::hours(1),
                executed_at: Some(Utc::now() - Duration::hours(1)),
            },
        ],
        open_positions: vec![
            Position {
                symbol: "BTCUSDT".to_string(),
                side: "LONG".to_string(),
                size: 0.5,
                entry_price: 50000.0,
                unrealized_pnl: 500.0,
                opened_at: Utc::now() - Duration::hours(2),
            },
        ],
        strategy_configs: vec![
            StrategyConfig {
                name: "MomentumTrader".to_string(),
                parameters: [
                    ("lookback_period".to_string(), "20".to_string()),
                    ("threshold".to_string(), "0.02".to_string()),
                ].into_iter().collect(),
                enabled: true,
                last_modified: Utc::now() - Duration::days(5),
            },
        ],
        risk_parameters: RiskParameters {
            max_position_size: 10.0,
            max_daily_loss: 5000.0,
            max_drawdown_percent: 10.0,
            position_limits: [
                ("BTCUSDT".to_string(), 5.0),
                ("ETHUSDT".to_string(), 50.0),
            ].into_iter().collect(),
        },
        timestamp: Utc::now(),
    };

    // Initialize backup manager
    let backup_manager = BackupManager::new(
        Path::new("./backups"),
        30, // Keep backups for 30 days
    )?;

    // Create backup
    let metadata = backup_manager.create_full_backup(
        &trading_data,
        "Daily trading data backup",
    )?;

    println!("\n--- Verifying backup ---");
    let is_valid = backup_manager.verify_backup(&metadata.id)?;
    println!("Backup {} is valid: {}", metadata.id, is_valid);

    println!("\n--- Restoring ---");
    let restored_data = backup_manager.restore_from_backup(&metadata.id)?;
    println!(
        "Data restored to point in time: {}",
        restored_data.timestamp
    );

    Ok(())
}
```

## Incremental Backup

For large data volumes, incremental backups are more efficient:

```rust
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// File change record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileChange {
    path: PathBuf,
    change_type: ChangeType,
    checksum: String,
    size: u64,
    modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum ChangeType {
    Added,
    Modified,
    Deleted,
}

/// File system state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileSystemSnapshot {
    files: HashMap<PathBuf, FileInfo>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileInfo {
    checksum: String,
    size: u64,
    modified_at: DateTime<Utc>,
}

/// Incremental backup system for trading data
struct IncrementalBackup {
    data_dir: PathBuf,
    backup_dir: PathBuf,
    last_snapshot: Option<FileSystemSnapshot>,
}

impl IncrementalBackup {
    fn new(data_dir: &Path, backup_dir: &Path) -> std::io::Result<Self> {
        fs::create_dir_all(backup_dir)?;

        Ok(IncrementalBackup {
            data_dir: data_dir.to_path_buf(),
            backup_dir: backup_dir.to_path_buf(),
            last_snapshot: None,
        })
    }

    /// Create current state snapshot
    fn create_snapshot(&self) -> std::io::Result<FileSystemSnapshot> {
        let mut files = HashMap::new();

        self.scan_directory(&self.data_dir, &mut files)?;

        Ok(FileSystemSnapshot {
            files,
            created_at: Utc::now(),
        })
    }

    fn scan_directory(
        &self,
        dir: &Path,
        files: &mut HashMap<PathBuf, FileInfo>,
    ) -> std::io::Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = fs::metadata(&path)?;
                let content = fs::read(&path)?;

                let modified = metadata.modified()
                    .map(|t| DateTime::<Utc>::from(t))
                    .unwrap_or_else(|_| Utc::now());

                files.insert(
                    path.strip_prefix(&self.data_dir)
                        .unwrap_or(&path)
                        .to_path_buf(),
                    FileInfo {
                        checksum: Self::quick_checksum(&content),
                        size: metadata.len(),
                        modified_at: modified,
                    },
                );
            } else if path.is_dir() {
                self.scan_directory(&path, files)?;
            }
        }

        Ok(())
    }

    /// Detect changes between snapshots
    fn detect_changes(
        &self,
        old_snapshot: &FileSystemSnapshot,
        new_snapshot: &FileSystemSnapshot,
    ) -> Vec<FileChange> {
        let mut changes = Vec::new();

        // Check for new and modified files
        for (path, new_info) in &new_snapshot.files {
            match old_snapshot.files.get(path) {
                None => {
                    // New file
                    changes.push(FileChange {
                        path: path.clone(),
                        change_type: ChangeType::Added,
                        checksum: new_info.checksum.clone(),
                        size: new_info.size,
                        modified_at: new_info.modified_at,
                    });
                }
                Some(old_info) if old_info.checksum != new_info.checksum => {
                    // Modified file
                    changes.push(FileChange {
                        path: path.clone(),
                        change_type: ChangeType::Modified,
                        checksum: new_info.checksum.clone(),
                        size: new_info.size,
                        modified_at: new_info.modified_at,
                    });
                }
                _ => {} // No changes
            }
        }

        // Check for deleted files
        for path in old_snapshot.files.keys() {
            if !new_snapshot.files.contains_key(path) {
                changes.push(FileChange {
                    path: path.clone(),
                    change_type: ChangeType::Deleted,
                    checksum: String::new(),
                    size: 0,
                    modified_at: Utc::now(),
                });
            }
        }

        changes
    }

    /// Create incremental backup
    fn create_incremental_backup(&mut self) -> std::io::Result<IncrementalBackupResult> {
        let new_snapshot = self.create_snapshot()?;

        let changes = match &self.last_snapshot {
            Some(old) => self.detect_changes(old, &new_snapshot),
            None => {
                // First backup — all files are new
                new_snapshot.files.iter().map(|(path, info)| {
                    FileChange {
                        path: path.clone(),
                        change_type: ChangeType::Added,
                        checksum: info.checksum.clone(),
                        size: info.size,
                        modified_at: info.modified_at,
                    }
                }).collect()
            }
        };

        let backup_id = format!(
            "incr_{}",
            Utc::now().format("%Y%m%d_%H%M%S")
        );
        let backup_path = self.backup_dir.join(&backup_id);
        fs::create_dir_all(&backup_path)?;

        let mut copied_files = 0;
        let mut total_size = 0u64;

        for change in &changes {
            if change.change_type != ChangeType::Deleted {
                let source = self.data_dir.join(&change.path);
                let dest = backup_path.join(&change.path);

                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }

                if source.exists() {
                    fs::copy(&source, &dest)?;
                    copied_files += 1;
                    total_size += change.size;
                }
            }
        }

        // Save change manifest
        let manifest = IncrementalManifest {
            backup_id: backup_id.clone(),
            changes: changes.clone(),
            created_at: Utc::now(),
            parent_snapshot: self.last_snapshot.as_ref().map(|s| s.created_at),
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(backup_path.join("manifest.json"), manifest_json)?;

        // Update last snapshot
        self.last_snapshot = Some(new_snapshot);

        Ok(IncrementalBackupResult {
            backup_id,
            changes_count: changes.len(),
            added: changes.iter().filter(|c| c.change_type == ChangeType::Added).count(),
            modified: changes.iter().filter(|c| c.change_type == ChangeType::Modified).count(),
            deleted: changes.iter().filter(|c| c.change_type == ChangeType::Deleted).count(),
            total_size,
        })
    }

    fn quick_checksum(data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct IncrementalManifest {
    backup_id: String,
    changes: Vec<FileChange>,
    created_at: DateTime<Utc>,
    parent_snapshot: Option<DateTime<Utc>>,
}

#[derive(Debug)]
struct IncrementalBackupResult {
    backup_id: String,
    changes_count: usize,
    added: usize,
    modified: usize,
    deleted: usize,
    total_size: u64,
}

fn main() -> std::io::Result<()> {
    println!("=== Incremental Backup System ===\n");

    // Create test data
    let data_dir = Path::new("./trading_data");
    fs::create_dir_all(data_dir)?;

    // Simulated trading data
    fs::write(
        data_dir.join("orders.json"),
        r#"[{"id": "ORD-001", "symbol": "BTCUSDT"}]"#,
    )?;
    fs::write(
        data_dir.join("positions.json"),
        r#"[{"symbol": "BTCUSDT", "size": 0.5}]"#,
    )?;

    let mut backup = IncrementalBackup::new(
        data_dir,
        Path::new("./incremental_backups"),
    )?;

    // First backup
    let result = backup.create_incremental_backup()?;
    println!("Backup 1: {}", result.backup_id);
    println!(
        "  Changes: {} (added: {}, modified: {}, deleted: {})",
        result.changes_count,
        result.added,
        result.modified,
        result.deleted
    );
    println!("  Size: {} bytes\n", result.total_size);

    // Modify data
    fs::write(
        data_dir.join("orders.json"),
        r#"[{"id": "ORD-001", "symbol": "BTCUSDT"}, {"id": "ORD-002", "symbol": "ETHUSDT"}]"#,
    )?;
    fs::write(
        data_dir.join("strategies.json"),
        r#"{"momentum": {"enabled": true}}"#,
    )?;

    // Second backup (incremental)
    let result = backup.create_incremental_backup()?;
    println!("Backup 2: {}", result.backup_id);
    println!(
        "  Changes: {} (added: {}, modified: {}, deleted: {})",
        result.changes_count,
        result.added,
        result.modified,
        result.deleted
    );
    println!("  Size: {} bytes", result.total_size);

    Ok(())
}
```

## Automated Backup Scheduling

A trading system should make backups automatically on schedule:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{self, Duration, Instant};
use chrono::{Utc, Timelike, Weekday, Datelike};

/// Backup execution schedule
#[derive(Debug, Clone)]
enum BackupSchedule {
    /// Every N minutes
    Interval(Duration),
    /// At a specific time every day
    Daily { hour: u32, minute: u32 },
    /// On specific days of the week
    Weekly { days: Vec<Weekday>, hour: u32, minute: u32 },
    /// After every N trades
    AfterTrades(usize),
}

/// Backup retention policy
#[derive(Debug, Clone)]
struct RetentionPolicy {
    /// Keep all backups for the last N days
    keep_daily_for_days: u32,
    /// Keep weekly backups for N weeks
    keep_weekly_for_weeks: u32,
    /// Keep monthly backups for N months
    keep_monthly_for_months: u32,
}

/// Backup execution statistics
#[derive(Debug, Default)]
struct BackupStats {
    total_backups: u64,
    successful_backups: u64,
    failed_backups: u64,
    total_bytes_backed_up: u64,
    last_backup_time: Option<chrono::DateTime<Utc>>,
    last_backup_duration_ms: u64,
}

/// Automatic backup scheduler
struct BackupScheduler {
    schedule: BackupSchedule,
    retention: RetentionPolicy,
    stats: Arc<RwLock<BackupStats>>,
    trades_since_backup: Arc<RwLock<usize>>,
}

impl BackupScheduler {
    fn new(schedule: BackupSchedule, retention: RetentionPolicy) -> Self {
        BackupScheduler {
            schedule,
            retention,
            stats: Arc::new(RwLock::new(BackupStats::default())),
            trades_since_backup: Arc::new(RwLock::new(0)),
        }
    }

    /// Start the backup scheduler
    async fn start(&self) {
        match &self.schedule {
            BackupSchedule::Interval(duration) => {
                self.run_interval_schedule(*duration).await;
            }
            BackupSchedule::Daily { hour, minute } => {
                self.run_daily_schedule(*hour, *minute).await;
            }
            BackupSchedule::Weekly { days, hour, minute } => {
                self.run_weekly_schedule(days.clone(), *hour, *minute).await;
            }
            BackupSchedule::AfterTrades(count) => {
                self.run_trade_based_schedule(*count).await;
            }
        }
    }

    async fn run_interval_schedule(&self, interval: Duration) {
        let mut interval_timer = time::interval(interval);

        loop {
            interval_timer.tick().await;
            self.execute_backup().await;
        }
    }

    async fn run_daily_schedule(&self, target_hour: u32, target_minute: u32) {
        loop {
            // Calculate time until next run
            let now = Utc::now();
            let mut next_run = now
                .with_hour(target_hour).unwrap()
                .with_minute(target_minute).unwrap()
                .with_second(0).unwrap();

            if next_run <= now {
                next_run = next_run + chrono::Duration::days(1);
            }

            let wait_duration = (next_run - now).to_std().unwrap();

            println!(
                "Next backup scheduled for: {} (in {:?})",
                next_run.format("%Y-%m-%d %H:%M:%S"),
                wait_duration
            );

            time::sleep(wait_duration).await;
            self.execute_backup().await;
        }
    }

    async fn run_weekly_schedule(
        &self,
        target_days: Vec<Weekday>,
        target_hour: u32,
        target_minute: u32,
    ) {
        loop {
            let now = Utc::now();

            // Find the nearest matching day
            let mut next_run = None;
            for days_ahead in 0..8 {
                let candidate = now + chrono::Duration::days(days_ahead);
                if target_days.contains(&candidate.weekday()) {
                    let mut run_time = candidate
                        .with_hour(target_hour).unwrap()
                        .with_minute(target_minute).unwrap()
                        .with_second(0).unwrap();

                    if run_time > now {
                        next_run = Some(run_time);
                        break;
                    }
                }
            }

            if let Some(next) = next_run {
                let wait_duration = (next - now).to_std().unwrap();
                time::sleep(wait_duration).await;
                self.execute_backup().await;
            }
        }
    }

    async fn run_trade_based_schedule(&self, threshold: usize) {
        loop {
            time::sleep(Duration::from_secs(1)).await;

            let trades = *self.trades_since_backup.read().await;
            if trades >= threshold {
                self.execute_backup().await;
                *self.trades_since_backup.write().await = 0;
            }
        }
    }

    /// Notify about a new trade (for trade-based backups)
    async fn notify_trade(&self) {
        let mut trades = self.trades_since_backup.write().await;
        *trades += 1;
    }

    async fn execute_backup(&self) {
        let start = Instant::now();

        println!("\n[{}] Starting backup...", Utc::now().format("%H:%M:%S"));

        // Simulate backup creation
        let backup_result = self.perform_backup().await;

        let duration = start.elapsed();
        let mut stats = self.stats.write().await;

        stats.total_backups += 1;
        stats.last_backup_time = Some(Utc::now());
        stats.last_backup_duration_ms = duration.as_millis() as u64;

        match backup_result {
            Ok(bytes) => {
                stats.successful_backups += 1;
                stats.total_bytes_backed_up += bytes;
                println!(
                    "[{}] Backup completed: {} bytes in {:?}",
                    Utc::now().format("%H:%M:%S"),
                    bytes,
                    duration
                );
            }
            Err(e) => {
                stats.failed_backups += 1;
                println!(
                    "[{}] Backup error: {}",
                    Utc::now().format("%H:%M:%S"),
                    e
                );
            }
        }
    }

    async fn perform_backup(&self) -> Result<u64, String> {
        // Simulate backup work
        time::sleep(Duration::from_millis(100)).await;

        // Simulated backup size
        Ok(1024 * 1024) // 1 MB
    }

    async fn get_stats(&self) -> BackupStats {
        self.stats.read().await.clone()
    }
}

impl Clone for BackupStats {
    fn clone(&self) -> Self {
        BackupStats {
            total_backups: self.total_backups,
            successful_backups: self.successful_backups,
            failed_backups: self.failed_backups,
            total_bytes_backed_up: self.total_bytes_backed_up,
            last_backup_time: self.last_backup_time,
            last_backup_duration_ms: self.last_backup_duration_ms,
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Automatic Backup Scheduler ===\n");

    // Create scheduler with 5-second interval for demonstration
    let scheduler = Arc::new(BackupScheduler::new(
        BackupSchedule::Interval(Duration::from_secs(5)),
        RetentionPolicy {
            keep_daily_for_days: 7,
            keep_weekly_for_weeks: 4,
            keep_monthly_for_months: 12,
        },
    ));

    // Start scheduler in background
    let scheduler_clone = scheduler.clone();
    let backup_task = tokio::spawn(async move {
        scheduler_clone.start().await;
    });

    // Simulate trading activity
    println!("Simulating trading...\n");

    for i in 1..=15 {
        time::sleep(Duration::from_secs(1)).await;

        if i % 3 == 0 {
            println!("[{}] Trade #{} executed", Utc::now().format("%H:%M:%S"), i / 3);
            scheduler.notify_trade().await;
        }

        if i % 10 == 0 {
            let stats = scheduler.get_stats().await;
            println!("\n--- Backup Statistics ---");
            println!("Total: {}", stats.total_backups);
            println!("Successful: {}", stats.successful_backups);
            println!("Failed: {}", stats.failed_backups);
            println!(
                "Volume: {} MB",
                stats.total_bytes_backed_up / 1024 / 1024
            );
            println!("---\n");
        }
    }

    // Stop scheduler
    backup_task.abort();

    println!("\nScheduler stopped");
}
```

## Disaster Recovery for Trading System

A disaster recovery plan is critically important:

```rust
use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Recovery criticality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum RecoveryPriority {
    /// Critical systems (must work within minutes)
    Critical,
    /// High priority (within an hour)
    High,
    /// Medium priority (within a day)
    Medium,
    /// Low priority (within a week)
    Low,
}

/// System component for recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemComponent {
    name: String,
    priority: RecoveryPriority,
    rto_seconds: u64, // Recovery Time Objective
    rpo_seconds: u64, // Recovery Point Objective
    dependencies: Vec<String>,
    backup_location: String,
    recovery_script: String,
}

/// Component recovery status
#[derive(Debug, Clone)]
enum RecoveryStatus {
    Pending,
    InProgress { started_at: DateTime<Utc> },
    Completed { duration_seconds: u64 },
    Failed { error: String },
}

/// Disaster recovery plan
#[derive(Debug)]
struct DisasterRecoveryPlan {
    components: HashMap<String, SystemComponent>,
    recovery_order: Vec<String>,
    status: HashMap<String, RecoveryStatus>,
}

impl DisasterRecoveryPlan {
    fn new() -> Self {
        DisasterRecoveryPlan {
            components: HashMap::new(),
            recovery_order: Vec::new(),
            status: HashMap::new(),
        }
    }

    /// Add component to the plan
    fn add_component(&mut self, component: SystemComponent) {
        self.status.insert(component.name.clone(), RecoveryStatus::Pending);
        self.components.insert(component.name.clone(), component);
        self.calculate_recovery_order();
    }

    /// Calculate recovery order based on dependencies and priorities
    fn calculate_recovery_order(&mut self) {
        let mut ordered = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Topological sort considering priorities
        let mut components: Vec<_> = self.components.values().collect();
        components.sort_by(|a, b| {
            let priority_order = |p: &RecoveryPriority| match p {
                RecoveryPriority::Critical => 0,
                RecoveryPriority::High => 1,
                RecoveryPriority::Medium => 2,
                RecoveryPriority::Low => 3,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });

        for component in &components {
            self.visit_component(&component.name, &mut visited, &mut ordered);
        }

        self.recovery_order = ordered;
    }

    fn visit_component(
        &self,
        name: &str,
        visited: &mut std::collections::HashSet<String>,
        ordered: &mut Vec<String>,
    ) {
        if visited.contains(name) {
            return;
        }

        if let Some(component) = self.components.get(name) {
            // Visit dependencies first
            for dep in &component.dependencies {
                self.visit_component(dep, visited, ordered);
            }

            visited.insert(name.to_string());
            ordered.push(name.to_string());
        }
    }

    /// Execute recovery procedure
    async fn execute_recovery(&mut self) -> RecoveryReport {
        let start_time = Utc::now();
        let mut report = RecoveryReport {
            started_at: start_time,
            completed_at: None,
            components_recovered: 0,
            components_failed: 0,
            total_duration_seconds: 0,
            details: Vec::new(),
        };

        println!("\n=== STARTING DISASTER RECOVERY ===");
        println!("Start time: {}", start_time.format("%Y-%m-%d %H:%M:%S"));
        println!("Components to recover: {}\n", self.recovery_order.len());

        for component_name in self.recovery_order.clone() {
            let component = self.components.get(&component_name).unwrap().clone();

            println!(
                "Recovering [{}]: {} (RTO: {}s, priority: {:?})",
                self.recovery_order.iter().position(|n| n == &component_name).unwrap() + 1,
                component_name,
                component.rto_seconds,
                component.priority
            );

            self.status.insert(
                component_name.clone(),
                RecoveryStatus::InProgress { started_at: Utc::now() },
            );

            // Simulate recovery
            let result = self.recover_component(&component).await;

            match result {
                Ok(duration) => {
                    let met_rto = duration <= component.rto_seconds;

                    self.status.insert(
                        component_name.clone(),
                        RecoveryStatus::Completed { duration_seconds: duration },
                    );

                    println!(
                        "  ✓ Recovered in {}s {}",
                        duration,
                        if met_rto { "(RTO met)" } else { "(RTO EXCEEDED!)" }
                    );

                    report.components_recovered += 1;
                    report.details.push(RecoveryDetail {
                        component: component_name,
                        success: true,
                        duration_seconds: duration,
                        met_rto,
                        error: None,
                    });
                }
                Err(e) => {
                    self.status.insert(
                        component_name.clone(),
                        RecoveryStatus::Failed { error: e.clone() },
                    );

                    println!("  ✗ Error: {}", e);

                    report.components_failed += 1;
                    report.details.push(RecoveryDetail {
                        component: component_name,
                        success: false,
                        duration_seconds: 0,
                        met_rto: false,
                        error: Some(e),
                    });
                }
            }
        }

        let end_time = Utc::now();
        report.completed_at = Some(end_time);
        report.total_duration_seconds = (end_time - start_time).num_seconds() as u64;

        println!("\n=== RECOVERY COMPLETED ===");
        println!("Total time: {}s", report.total_duration_seconds);
        println!(
            "Successful: {}, Failed: {}",
            report.components_recovered,
            report.components_failed
        );

        report
    }

    async fn recover_component(&self, component: &SystemComponent) -> Result<u64, String> {
        // Simulate recovery time
        let base_time = match component.priority {
            RecoveryPriority::Critical => 5,
            RecoveryPriority::High => 15,
            RecoveryPriority::Medium => 30,
            RecoveryPriority::Low => 60,
        };

        // Add some randomness
        let actual_time = base_time + (base_time / 5);

        tokio::time::sleep(Duration::from_millis(actual_time * 10)).await;

        // 95% successful recoveries
        if rand::random::<f32>() > 0.95 {
            Err("Error restoring from backup".to_string())
        } else {
            Ok(actual_time)
        }
    }
}

#[derive(Debug)]
struct RecoveryReport {
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    components_recovered: usize,
    components_failed: usize,
    total_duration_seconds: u64,
    details: Vec<RecoveryDetail>,
}

#[derive(Debug)]
struct RecoveryDetail {
    component: String,
    success: bool,
    duration_seconds: u64,
    met_rto: bool,
    error: Option<String>,
}

fn create_trading_system_dr_plan() -> DisasterRecoveryPlan {
    let mut plan = DisasterRecoveryPlan::new();

    // Critical components
    plan.add_component(SystemComponent {
        name: "database".to_string(),
        priority: RecoveryPriority::Critical,
        rto_seconds: 300,  // 5 minutes
        rpo_seconds: 60,   // 1 minute data loss
        dependencies: vec![],
        backup_location: "s3://backups/db/".to_string(),
        recovery_script: "scripts/restore_db.sh".to_string(),
    });

    plan.add_component(SystemComponent {
        name: "order_engine".to_string(),
        priority: RecoveryPriority::Critical,
        rto_seconds: 180,
        rpo_seconds: 0,  // Cannot lose orders
        dependencies: vec!["database".to_string()],
        backup_location: "s3://backups/engine/".to_string(),
        recovery_script: "scripts/restore_engine.sh".to_string(),
    });

    plan.add_component(SystemComponent {
        name: "risk_manager".to_string(),
        priority: RecoveryPriority::Critical,
        rto_seconds: 180,
        rpo_seconds: 60,
        dependencies: vec!["database".to_string()],
        backup_location: "s3://backups/risk/".to_string(),
        recovery_script: "scripts/restore_risk.sh".to_string(),
    });

    // High priority
    plan.add_component(SystemComponent {
        name: "market_data".to_string(),
        priority: RecoveryPriority::High,
        rto_seconds: 600,
        rpo_seconds: 300,
        dependencies: vec![],
        backup_location: "s3://backups/market/".to_string(),
        recovery_script: "scripts/restore_market.sh".to_string(),
    });

    plan.add_component(SystemComponent {
        name: "api_gateway".to_string(),
        priority: RecoveryPriority::High,
        rto_seconds: 300,
        rpo_seconds: 0,
        dependencies: vec!["order_engine".to_string(), "risk_manager".to_string()],
        backup_location: "s3://backups/api/".to_string(),
        recovery_script: "scripts/restore_api.sh".to_string(),
    });

    // Medium priority
    plan.add_component(SystemComponent {
        name: "reporting".to_string(),
        priority: RecoveryPriority::Medium,
        rto_seconds: 3600,
        rpo_seconds: 3600,
        dependencies: vec!["database".to_string()],
        backup_location: "s3://backups/reports/".to_string(),
        recovery_script: "scripts/restore_reports.sh".to_string(),
    });

    plan.add_component(SystemComponent {
        name: "analytics".to_string(),
        priority: RecoveryPriority::Medium,
        rto_seconds: 7200,
        rpo_seconds: 86400, // Can lose a day
        dependencies: vec!["database".to_string(), "market_data".to_string()],
        backup_location: "s3://backups/analytics/".to_string(),
        recovery_script: "scripts/restore_analytics.sh".to_string(),
    });

    // Low priority
    plan.add_component(SystemComponent {
        name: "backtest_engine".to_string(),
        priority: RecoveryPriority::Low,
        rto_seconds: 86400,
        rpo_seconds: 604800, // A week
        dependencies: vec!["market_data".to_string()],
        backup_location: "s3://backups/backtest/".to_string(),
        recovery_script: "scripts/restore_backtest.sh".to_string(),
    });

    plan
}

#[tokio::main]
async fn main() {
    println!("=== Disaster Recovery Plan for Trading System ===\n");

    let mut plan = create_trading_system_dr_plan();

    println!("Recovery order:");
    for (i, component) in plan.recovery_order.iter().enumerate() {
        let comp = plan.components.get(component).unwrap();
        println!(
            "  {}. {} (priority: {:?}, RTO: {}s)",
            i + 1,
            component,
            comp.priority,
            comp.rto_seconds
        );
    }

    // Execute test recovery
    let report = plan.execute_recovery().await;

    println!("\n=== RECOVERY REPORT ===");
    println!("Start: {}", report.started_at.format("%Y-%m-%d %H:%M:%S"));
    if let Some(end) = report.completed_at {
        println!("End: {}", end.format("%Y-%m-%d %H:%M:%S"));
    }
    println!("Total time: {} seconds", report.total_duration_seconds);
    println!("Successfully recovered: {}", report.components_recovered);
    println!("Failed: {}", report.components_failed);

    let met_rto: Vec<_> = report.details.iter()
        .filter(|d| d.success && d.met_rto)
        .collect();
    let missed_rto: Vec<_> = report.details.iter()
        .filter(|d| d.success && !d.met_rto)
        .collect();

    println!("\nRTO met: {} components", met_rto.len());
    println!("RTO exceeded: {} components", missed_rto.len());

    if !missed_rto.is_empty() {
        println!("\nComponents with exceeded RTO:");
        for detail in missed_rto {
            println!("  - {} ({}s)", detail.component, detail.duration_seconds);
        }
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Full backup** | Copy of all data — baseline recovery level |
| **Incremental backup** | Only changes since last backup — saves space |
| **Differential backup** | Changes since last full backup |
| **RTO (Recovery Time Objective)** | Maximum allowable downtime |
| **RPO (Recovery Point Objective)** | Maximum allowable data loss |
| **Retention Policy** | Policy for storing and deleting old backups |
| **Disaster Recovery** | Post-disaster recovery plan |
| **Checksums** | Backup integrity verification |

## Practical Exercises

1. **Portfolio Backup System**: Create a system that:
   - Makes a full backup of all positions daily
   - Saves incremental backups after each trade
   - Compresses data to save space
   - Verifies integrity on restore

2. **Geo-replication**: Implement a system that:
   - Synchronizes backups to multiple regions
   - Checks consistency between replicas
   - Automatically switches to backup region
   - Monitors replication status

3. **Data Versioning**: Create a system that:
   - Stores multiple versions of each file
   - Allows rollback to any point in time
   - Automatically cleans up old versions
   - Searches through change history

4. **Cloud Integration**: Implement:
   - Upload backups to S3/GCS/Azure Blob
   - Encrypt data before upload
   - Manage object lifecycle
   - Notifications about backup status

## Homework

1. **Comprehensive Backup System**: Create a system that:
   - Supports full, incremental, and differential backups
   - Automatically selects backup type by schedule
   - Stores backups according to retention policy (7 days, 4 weeks, 12 months)
   - Sends status notifications to Telegram/Slack
   - Generates weekly report on backup status

2. **DR Testing**: Write a tool for:
   - Automatic testing of backup restoration
   - Measuring actual RTO/RPO
   - Comparing with target metrics
   - Generating DR readiness report
   - Improvement recommendations

3. **Trading Strategy Versioning System**: Implement:
   - Versioning of strategy configurations
   - Change history with diff view
   - Ability to rollback to any version
   - Performance comparison of different versions
   - Configuration export/import

4. **Multi-cloud Backups**: Create a system that:
   - Simultaneously uploads to AWS, GCP, and Azure
   - Checks consistency across providers
   - Automatically restores from available source
   - Monitors storage costs
   - Optimizes backup expenses

## Navigation

[← Previous day](../354-production-logging/en.md) | [Next day →](../360-canary-deployments/en.md)
