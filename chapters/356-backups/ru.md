# День 356: Резервное копирование

## Аналогия из трейдинга

Представь, что ты управляешь крупным хедж-фондом. За годы работы ты накопил бесценные данные: историю всех сделок, настройки стратегий, обученные модели машинного обучения, конфигурации риск-менеджмента. Однажды утром ты приходишь в офис и обнаруживаешь, что сервер с данными сгорел.

**Без бэкапов:**
- Потеряны все торговые стратегии
- Нет истории P&L для отчётности инвесторам
- Невозможно восстановить позиции
- Клиенты уходят, репутация разрушена

**С бэкапами:**
- Восстановление за час
- Все данные на месте
- Бизнес продолжает работу
- Клиенты даже не заметили проблему

| Трейдинг | Резервное копирование |
|----------|----------------------|
| **Стоп-лосс** | Backup — защита от потери данных |
| **Хеджирование** | Репликация — копии в разных местах |
| **Диверсификация** | Разные типы бэкапов (полный, инкрементальный) |
| **Страхование** | Тестирование восстановления |
| **План на случай кризиса** | Disaster Recovery Plan |

В трейдинге мы всегда готовы к худшему. То же самое должно быть с данными.

## Стратегии резервного копирования

В Rust мы можем создать надёжную систему бэкапов для торговых данных:

```rust
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write, BufReader, BufWriter};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;

/// Типы резервных копий
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum BackupType {
    /// Полная копия — все данные
    Full,
    /// Инкрементальная — только изменения с последнего бэкапа
    Incremental,
    /// Дифференциальная — изменения с последнего полного бэкапа
    Differential,
}

/// Метаданные бэкапа
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

/// Конфигурация торговых данных для бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TradingDataBackup {
    /// История всех ордеров
    orders_history: Vec<Order>,
    /// Текущие открытые позиции
    open_positions: Vec<Position>,
    /// Настройки стратегий
    strategy_configs: Vec<StrategyConfig>,
    /// Параметры риск-менеджмента
    risk_parameters: RiskParameters,
    /// Временная метка создания
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

/// Менеджер резервного копирования для торговой системы
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
            compression_level: 6, // Баланс между скоростью и размером
        })
    }

    /// Создание полного бэкапа торговых данных
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

        println!("Создание полного бэкапа: {}", backup_id);

        // Сериализация данных
        let json_data = serde_json::to_vec(data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Вычисление контрольной суммы до сжатия
        let checksum = Self::calculate_checksum(&json_data);

        // Сжатие и запись
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

        // Сохраняем метаданные отдельно
        self.save_metadata(&metadata)?;

        println!(
            "Бэкап создан: {} ({} байт, сжатие: {:.1}%)",
            metadata.id,
            size_bytes,
            (1.0 - size_bytes as f64 / json_data.len() as f64) * 100.0
        );

        Ok(metadata)
    }

    /// Восстановление из бэкапа
    fn restore_from_backup(
        &self,
        backup_id: &str,
    ) -> io::Result<TradingDataBackup> {
        let backup_path = self.backup_dir.join(format!("{}.backup.gz", backup_id));

        println!("Восстановление из бэкапа: {}", backup_id);

        // Чтение и распаковка
        let file = File::open(&backup_path)?;
        let mut decoder = GzDecoder::new(BufReader::new(file));
        let mut json_data = Vec::new();
        decoder.read_to_end(&mut json_data)?;

        // Проверка контрольной суммы
        let metadata = self.load_metadata(backup_id)?;
        let current_checksum = Self::calculate_checksum(&json_data);

        if current_checksum != metadata.checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Контрольная сумма не совпадает! Бэкап повреждён.",
            ));
        }

        // Десериализация
        let data: TradingDataBackup = serde_json::from_slice(&json_data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        println!(
            "Восстановлено: {} ордеров, {} позиций, {} стратегий",
            data.orders_history.len(),
            data.open_positions.len(),
            data.strategy_configs.len()
        );

        Ok(data)
    }

    /// Список всех доступных бэкапов
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

        // Сортировка по дате (новые первыми)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Удаление старых бэкапов (политика удержания)
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

                println!("Удалён старый бэкап: {}", metadata.id);
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Верификация целостности бэкапа
    fn verify_backup(&self, backup_id: &str) -> io::Result<bool> {
        let backup_path = self.backup_dir.join(format!("{}.backup.gz", backup_id));
        let metadata = self.load_metadata(backup_id)?;

        // Чтение и распаковка для проверки
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
    println!("=== Система резервного копирования торговых данных ===\n");

    // Создаём тестовые торговые данные
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

    // Инициализация менеджера бэкапов
    let backup_manager = BackupManager::new(
        Path::new("./backups"),
        30, // Хранить бэкапы 30 дней
    )?;

    // Создание бэкапа
    let metadata = backup_manager.create_full_backup(
        &trading_data,
        "Ежедневный бэкап торговых данных",
    )?;

    println!("\n--- Верификация бэкапа ---");
    let is_valid = backup_manager.verify_backup(&metadata.id)?;
    println!("Бэкап {} валиден: {}", metadata.id, is_valid);

    println!("\n--- Восстановление ---");
    let restored_data = backup_manager.restore_from_backup(&metadata.id)?;
    println!(
        "Данные восстановлены на момент: {}",
        restored_data.timestamp
    );

    Ok(())
}
```

## Инкрементальное резервное копирование

Для больших объёмов данных эффективнее использовать инкрементальные бэкапы:

```rust
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Запись об изменении файла
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

/// Снапшот состояния файловой системы
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

/// Система инкрементального бэкапа для торговых данных
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

    /// Создание снапшота текущего состояния
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

    /// Определение изменений между снапшотами
    fn detect_changes(
        &self,
        old_snapshot: &FileSystemSnapshot,
        new_snapshot: &FileSystemSnapshot,
    ) -> Vec<FileChange> {
        let mut changes = Vec::new();

        // Проверяем новые и изменённые файлы
        for (path, new_info) in &new_snapshot.files {
            match old_snapshot.files.get(path) {
                None => {
                    // Новый файл
                    changes.push(FileChange {
                        path: path.clone(),
                        change_type: ChangeType::Added,
                        checksum: new_info.checksum.clone(),
                        size: new_info.size,
                        modified_at: new_info.modified_at,
                    });
                }
                Some(old_info) if old_info.checksum != new_info.checksum => {
                    // Изменённый файл
                    changes.push(FileChange {
                        path: path.clone(),
                        change_type: ChangeType::Modified,
                        checksum: new_info.checksum.clone(),
                        size: new_info.size,
                        modified_at: new_info.modified_at,
                    });
                }
                _ => {} // Без изменений
            }
        }

        // Проверяем удалённые файлы
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

    /// Создание инкрементального бэкапа
    fn create_incremental_backup(&mut self) -> std::io::Result<IncrementalBackupResult> {
        let new_snapshot = self.create_snapshot()?;

        let changes = match &self.last_snapshot {
            Some(old) => self.detect_changes(old, &new_snapshot),
            None => {
                // Первый бэкап — все файлы новые
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

        // Сохраняем манифест изменений
        let manifest = IncrementalManifest {
            backup_id: backup_id.clone(),
            changes: changes.clone(),
            created_at: Utc::now(),
            parent_snapshot: self.last_snapshot.as_ref().map(|s| s.created_at),
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(backup_path.join("manifest.json"), manifest_json)?;

        // Обновляем последний снапшот
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
    println!("=== Инкрементальное резервное копирование ===\n");

    // Создаём тестовые данные
    let data_dir = Path::new("./trading_data");
    fs::create_dir_all(data_dir)?;

    // Имитация торговых данных
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

    // Первый бэкап
    let result = backup.create_incremental_backup()?;
    println!("Бэкап 1: {}", result.backup_id);
    println!(
        "  Изменений: {} (добавлено: {}, изменено: {}, удалено: {})",
        result.changes_count,
        result.added,
        result.modified,
        result.deleted
    );
    println!("  Размер: {} байт\n", result.total_size);

    // Изменяем данные
    fs::write(
        data_dir.join("orders.json"),
        r#"[{"id": "ORD-001", "symbol": "BTCUSDT"}, {"id": "ORD-002", "symbol": "ETHUSDT"}]"#,
    )?;
    fs::write(
        data_dir.join("strategies.json"),
        r#"{"momentum": {"enabled": true}}"#,
    )?;

    // Второй бэкап (инкрементальный)
    let result = backup.create_incremental_backup()?;
    println!("Бэкап 2: {}", result.backup_id);
    println!(
        "  Изменений: {} (добавлено: {}, изменено: {}, удалено: {})",
        result.changes_count,
        result.added,
        result.modified,
        result.deleted
    );
    println!("  Размер: {} байт", result.total_size);

    Ok(())
}
```

## Автоматическое расписание бэкапов

Торговая система должна делать бэкапы автоматически по расписанию:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{self, Duration, Instant};
use chrono::{Utc, Timelike, Weekday, Datelike};

/// Расписание выполнения бэкапов
#[derive(Debug, Clone)]
enum BackupSchedule {
    /// Каждые N минут
    Interval(Duration),
    /// В определённое время каждый день
    Daily { hour: u32, minute: u32 },
    /// В определённые дни недели
    Weekly { days: Vec<Weekday>, hour: u32, minute: u32 },
    /// После каждых N сделок
    AfterTrades(usize),
}

/// Политика хранения бэкапов
#[derive(Debug, Clone)]
struct RetentionPolicy {
    /// Хранить все бэкапы за последние N дней
    keep_daily_for_days: u32,
    /// Хранить еженедельные бэкапы за N недель
    keep_weekly_for_weeks: u32,
    /// Хранить ежемесячные бэкапы за N месяцев
    keep_monthly_for_months: u32,
}

/// Статистика выполнения бэкапов
#[derive(Debug, Default)]
struct BackupStats {
    total_backups: u64,
    successful_backups: u64,
    failed_backups: u64,
    total_bytes_backed_up: u64,
    last_backup_time: Option<chrono::DateTime<Utc>>,
    last_backup_duration_ms: u64,
}

/// Планировщик автоматических бэкапов
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

    /// Запуск планировщика бэкапов
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
            // Вычисляем время до следующего запуска
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
                "Следующий бэкап запланирован на: {} (через {:?})",
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

            // Находим ближайший подходящий день
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

    /// Уведомление о новой сделке (для trade-based бэкапов)
    async fn notify_trade(&self) {
        let mut trades = self.trades_since_backup.write().await;
        *trades += 1;
    }

    async fn execute_backup(&self) {
        let start = Instant::now();

        println!("\n[{}] Запуск бэкапа...", Utc::now().format("%H:%M:%S"));

        // Имитация создания бэкапа
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
                    "[{}] Бэкап завершён: {} байт за {:?}",
                    Utc::now().format("%H:%M:%S"),
                    bytes,
                    duration
                );
            }
            Err(e) => {
                stats.failed_backups += 1;
                println!(
                    "[{}] Ошибка бэкапа: {}",
                    Utc::now().format("%H:%M:%S"),
                    e
                );
            }
        }
    }

    async fn perform_backup(&self) -> Result<u64, String> {
        // Имитация работы бэкапа
        time::sleep(Duration::from_millis(100)).await;

        // Имитация размера бэкапа
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
    println!("=== Планировщик автоматических бэкапов ===\n");

    // Создаём планировщик с интервалом в 5 секунд для демонстрации
    let scheduler = Arc::new(BackupScheduler::new(
        BackupSchedule::Interval(Duration::from_secs(5)),
        RetentionPolicy {
            keep_daily_for_days: 7,
            keep_weekly_for_weeks: 4,
            keep_monthly_for_months: 12,
        },
    ));

    // Запускаем планировщик в фоне
    let scheduler_clone = scheduler.clone();
    let backup_task = tokio::spawn(async move {
        scheduler_clone.start().await;
    });

    // Имитация торговой активности
    println!("Имитация торговли...\n");

    for i in 1..=15 {
        time::sleep(Duration::from_secs(1)).await;

        if i % 3 == 0 {
            println!("[{}] Сделка #{} выполнена", Utc::now().format("%H:%M:%S"), i / 3);
            scheduler.notify_trade().await;
        }

        if i % 10 == 0 {
            let stats = scheduler.get_stats().await;
            println!("\n--- Статистика бэкапов ---");
            println!("Всего: {}", stats.total_backups);
            println!("Успешных: {}", stats.successful_backups);
            println!("Ошибок: {}", stats.failed_backups);
            println!(
                "Объём: {} MB",
                stats.total_bytes_backed_up / 1024 / 1024
            );
            println!("---\n");
        }
    }

    // Останавливаем планировщик
    backup_task.abort();

    println!("\nПланировщик остановлен");
}
```

## Disaster Recovery для торговой системы

План восстановления после катастрофы критически важен:

```rust
use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Уровни критичности для восстановления
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum RecoveryPriority {
    /// Критические системы (должны работать в течение минут)
    Critical,
    /// Высокий приоритет (в течение часа)
    High,
    /// Средний приоритет (в течение дня)
    Medium,
    /// Низкий приоритет (в течение недели)
    Low,
}

/// Компонент системы для восстановления
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

/// Статус восстановления компонента
#[derive(Debug, Clone)]
enum RecoveryStatus {
    Pending,
    InProgress { started_at: DateTime<Utc> },
    Completed { duration_seconds: u64 },
    Failed { error: String },
}

/// План восстановления после катастрофы
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

    /// Добавление компонента в план
    fn add_component(&mut self, component: SystemComponent) {
        self.status.insert(component.name.clone(), RecoveryStatus::Pending);
        self.components.insert(component.name.clone(), component);
        self.calculate_recovery_order();
    }

    /// Расчёт порядка восстановления на основе зависимостей и приоритетов
    fn calculate_recovery_order(&mut self) {
        let mut ordered = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Топологическая сортировка с учётом приоритетов
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
            // Сначала посещаем зависимости
            for dep in &component.dependencies {
                self.visit_component(dep, visited, ordered);
            }

            visited.insert(name.to_string());
            ordered.push(name.to_string());
        }
    }

    /// Запуск процедуры восстановления
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

        println!("\n=== ЗАПУСК DISASTER RECOVERY ===");
        println!("Время начала: {}", start_time.format("%Y-%m-%d %H:%M:%S"));
        println!("Компонентов к восстановлению: {}\n", self.recovery_order.len());

        for component_name in self.recovery_order.clone() {
            let component = self.components.get(&component_name).unwrap().clone();

            println!(
                "Восстановление [{}]: {} (RTO: {}с, приоритет: {:?})",
                self.recovery_order.iter().position(|n| n == &component_name).unwrap() + 1,
                component_name,
                component.rto_seconds,
                component.priority
            );

            self.status.insert(
                component_name.clone(),
                RecoveryStatus::InProgress { started_at: Utc::now() },
            );

            // Имитация восстановления
            let result = self.recover_component(&component).await;

            match result {
                Ok(duration) => {
                    let met_rto = duration <= component.rto_seconds;

                    self.status.insert(
                        component_name.clone(),
                        RecoveryStatus::Completed { duration_seconds: duration },
                    );

                    println!(
                        "  ✓ Восстановлено за {}с {}",
                        duration,
                        if met_rto { "(RTO соблюдён)" } else { "(RTO ПРЕВЫШЕН!)" }
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

                    println!("  ✗ Ошибка: {}", e);

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

        println!("\n=== ВОССТАНОВЛЕНИЕ ЗАВЕРШЕНО ===");
        println!("Общее время: {}с", report.total_duration_seconds);
        println!(
            "Успешно: {}, Ошибки: {}",
            report.components_recovered,
            report.components_failed
        );

        report
    }

    async fn recover_component(&self, component: &SystemComponent) -> Result<u64, String> {
        // Имитация времени восстановления
        let base_time = match component.priority {
            RecoveryPriority::Critical => 5,
            RecoveryPriority::High => 15,
            RecoveryPriority::Medium => 30,
            RecoveryPriority::Low => 60,
        };

        // Добавляем случайность
        let actual_time = base_time + (base_time / 5);

        tokio::time::sleep(Duration::from_millis(actual_time * 10)).await;

        // 95% успешных восстановлений
        if rand::random::<f32>() > 0.95 {
            Err("Ошибка при восстановлении из бэкапа".to_string())
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

    // Критические компоненты
    plan.add_component(SystemComponent {
        name: "database".to_string(),
        priority: RecoveryPriority::Critical,
        rto_seconds: 300,  // 5 минут
        rpo_seconds: 60,   // 1 минута потери данных
        dependencies: vec![],
        backup_location: "s3://backups/db/".to_string(),
        recovery_script: "scripts/restore_db.sh".to_string(),
    });

    plan.add_component(SystemComponent {
        name: "order_engine".to_string(),
        priority: RecoveryPriority::Critical,
        rto_seconds: 180,
        rpo_seconds: 0,  // Нельзя терять ордера
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

    // Высокий приоритет
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

    // Средний приоритет
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
        rpo_seconds: 86400, // Можем потерять день
        dependencies: vec!["database".to_string(), "market_data".to_string()],
        backup_location: "s3://backups/analytics/".to_string(),
        recovery_script: "scripts/restore_analytics.sh".to_string(),
    });

    // Низкий приоритет
    plan.add_component(SystemComponent {
        name: "backtest_engine".to_string(),
        priority: RecoveryPriority::Low,
        rto_seconds: 86400,
        rpo_seconds: 604800, // Неделя
        dependencies: vec!["market_data".to_string()],
        backup_location: "s3://backups/backtest/".to_string(),
        recovery_script: "scripts/restore_backtest.sh".to_string(),
    });

    plan
}

#[tokio::main]
async fn main() {
    println!("=== План Disaster Recovery для торговой системы ===\n");

    let mut plan = create_trading_system_dr_plan();

    println!("Порядок восстановления:");
    for (i, component) in plan.recovery_order.iter().enumerate() {
        let comp = plan.components.get(component).unwrap();
        println!(
            "  {}. {} (приоритет: {:?}, RTO: {}с)",
            i + 1,
            component,
            comp.priority,
            comp.rto_seconds
        );
    }

    // Запуск тестового восстановления
    let report = plan.execute_recovery().await;

    println!("\n=== ОТЧЁТ О ВОССТАНОВЛЕНИИ ===");
    println!("Начало: {}", report.started_at.format("%Y-%m-%d %H:%M:%S"));
    if let Some(end) = report.completed_at {
        println!("Конец: {}", end.format("%Y-%m-%d %H:%M:%S"));
    }
    println!("Общее время: {} секунд", report.total_duration_seconds);
    println!("Успешно восстановлено: {}", report.components_recovered);
    println!("С ошибками: {}", report.components_failed);

    let met_rto: Vec<_> = report.details.iter()
        .filter(|d| d.success && d.met_rto)
        .collect();
    let missed_rto: Vec<_> = report.details.iter()
        .filter(|d| d.success && !d.met_rto)
        .collect();

    println!("\nRTO соблюдён: {} компонентов", met_rto.len());
    println!("RTO превышен: {} компонентов", missed_rto.len());

    if !missed_rto.is_empty() {
        println!("\nКомпоненты с превышением RTO:");
        for detail in missed_rto {
            println!("  - {} ({}с)", detail.component, detail.duration_seconds);
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Полный бэкап** | Копия всех данных — базовый уровень восстановления |
| **Инкрементальный бэкап** | Только изменения с последнего бэкапа — экономит место |
| **Дифференциальный бэкап** | Изменения с последнего полного бэкапа |
| **RTO (Recovery Time Objective)** | Максимально допустимое время простоя |
| **RPO (Recovery Point Objective)** | Максимально допустимая потеря данных |
| **Retention Policy** | Политика хранения и удаления старых бэкапов |
| **Disaster Recovery** | План восстановления после катастрофы |
| **Checksums** | Проверка целостности бэкапов |

## Практические задания

1. **Система бэкапов для портфеля**: Создай систему, которая:
   - Делает полный бэкап всех позиций ежедневно
   - Сохраняет инкрементальные бэкапы после каждой сделки
   - Сжимает данные для экономии места
   - Проверяет целостность при восстановлении

2. **Geo-репликация**: Реализуй систему:
   - Синхронизация бэкапов в несколько регионов
   - Проверка консистентности между репликами
   - Автоматическое переключение на резервный регион
   - Мониторинг статуса репликации

3. **Версионирование данных**: Создай систему:
   - Хранение нескольких версий каждого файла
   - Возможность отката к любой точке во времени
   - Автоматическая очистка старых версий
   - Поиск по истории изменений

4. **Интеграция с облаком**: Реализуй:
   - Загрузка бэкапов в S3/GCS/Azure Blob
   - Шифрование данных перед загрузкой
   - Управление жизненным циклом объектов
   - Оповещения о статусе бэкапов

## Домашнее задание

1. **Комплексная система бэкапов**: Создай систему, которая:
   - Поддерживает полные, инкрементальные и дифференциальные бэкапы
   - Автоматически выбирает тип бэкапа по расписанию
   - Хранит бэкапы согласно политике retention (7 дней, 4 недели, 12 месяцев)
   - Отправляет уведомления о статусе в Telegram/Slack
   - Генерирует еженедельный отчёт о состоянии бэкапов

2. **DR-тестирование**: Напиши инструмент для:
   - Автоматического тестирования восстановления из бэкапов
   - Измерения фактического RTO/RPO
   - Сравнения с целевыми показателями
   - Генерации отчёта о готовности к DR
   - Рекомендаций по улучшению

3. **Система версионирования торговых стратегий**: Реализуй:
   - Версионирование конфигураций стратегий
   - История изменений с diff-просмотром
   - Возможность отката к любой версии
   - Сравнение производительности разных версий
   - Экспорт/импорт конфигураций

4. **Multi-cloud бэкапы**: Создай систему:
   - Одновременная загрузка в AWS, GCP и Azure
   - Проверка консистентности между провайдерами
   - Автоматическое восстановление из доступного источника
   - Мониторинг стоимости хранения
   - Оптимизация расходов на бэкапы

## Навигация

[← Предыдущий день](../355-secret-rotation/ru.md) | [Следующий день →](../357-disaster-recovery/ru.md)
