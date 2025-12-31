# День 241: Резервное копирование: сохраняя историю

## Аналогия из трейдинга

Представь, что ты управляешь торговым деском, который годами накапливал ценные данные: каждая исполненная сделка, каждое изменение цены, каждое решение о ребалансировке портфеля задокументировано. Однажды сбой сервера уничтожает твою базу данных. Без резервных копий ты теряешь не просто данные, но и возможность анализировать прошлые результаты, проводить аудит сделок для соответствия регуляторам и учиться на исторических паттернах.

В трейдинге резервные копии — это как сейф для твоего торгового журнала. Подобно тому, как профессиональные трейдеры ведут скрупулёзные записи своих сделок, надёжная стратегия резервного копирования гарантирует, что:
- Вся твоя торговая история переживёт любую катастрофу
- Ты сможешь восстановить состояние портфеля на любой момент времени
- Регуляторные аудиты будут удовлетворены историческими данными
- Бэктестирование стратегий останется возможным с реальными историческими данными

## Что такое резервное копирование базы данных?

Резервная копия базы данных — это копия твоих данных, которая может быть использована для восстановления исходных данных в случае их потери. В торговых системах резервные копии критически важны, потому что:

1. **Соответствие регуляторным требованиям** — финансовые регуляторы часто требуют хранения данных годами
2. **Восстановление после аварий** — аппаратные сбои, кибератаки или человеческие ошибки могут уничтожить данные
3. **Восстановление на момент времени** — возможность восстановить данные на конкретный момент
4. **Миграция данных** — перенос данных между системами или окружениями

## Типы резервных копий

### 1. Полное резервное копирование
Полная копия всей базы данных.

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

    /// Создаёт полную резервную копию всех торговых данных
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

        println!("Полная резервная копия создана: {}", backup_path);
        println!("  - {} сделок сохранено", self.trades.len());
        println!("  - Баланс портфеля: ${:.2}", self.portfolio.cash);

        Ok(backup_path)
    }
}
```

### 2. Инкрементное резервное копирование
Сохраняет только данные, изменившиеся с момента последней резервной копии.

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

    /// Создаёт инкрементную резервную копию только с новыми сделками
    fn incremental_backup(&mut self, db: &TradingDatabase) -> io::Result<Option<String>> {
        // Находим сделки новее последней резервной копии
        let new_trades: Vec<&Trade> = db.trades
            .iter()
            .filter(|t| t.id > self.last_backup_trade_id)
            .collect();

        if new_trades.is_empty() {
            println!("Нет новых сделок с момента последней резервной копии");
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

        // Обновляем отслеживание
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

        println!("Инкрементная резервная копия создана: {}", backup_path);
        println!("  - {} новых сделок сохранено", new_trades.len());

        Ok(Some(backup_path))
    }
}
```

### 3. Дифференциальное резервное копирование
Сохраняет все изменения с момента последней полной резервной копии.

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

    /// Создаёт дифференциальную резервную копию (все изменения с последней полной копии)
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

        println!("Дифференциальная резервная копия создана: {}", backup_path);
        println!("  - {} сделок с момента последней полной копии", trades_since_full.len());

        Ok(backup_path)
    }
}
```

## Восстановление на момент времени для трейдинга

Одна из самых ценных функций резервного копирования для торговых систем — возможность восстановить данные на конкретный момент времени.

```rust
use std::collections::BTreeMap;

struct PointInTimeRecovery {
    /// Снимки, индексированные по времени
    snapshots: BTreeMap<DateTime<Utc>, PortfolioSnapshot>,
    /// Лог транзакций для воспроизведения
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

    /// Создаёт снимок текущего состояния портфеля
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
        println!("Снимок создан в {}", Utc::now());
    }

    /// Логирует транзакцию для возможности воспроизведения
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

    /// Восстанавливает состояние портфеля на конкретный момент времени
    fn recover_to_point(&self, target_time: DateTime<Utc>) -> Option<PortfolioSnapshot> {
        // Находим последний снимок перед целевым временем
        let base_snapshot = self.snapshots
            .range(..=target_time)
            .last()
            .map(|(_, s)| s.clone())?;

        // Воспроизводим транзакции между снимком и целевым временем
        let mut recovered = base_snapshot.clone();

        for entry in &self.transaction_log {
            if entry.timestamp > base_snapshot.timestamp && entry.timestamp <= target_time {
                // Применяем транзакцию
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

        // Пересчитываем общую стоимость
        recovered.total_value = recovered.cash + recovered.positions
            .values()
            .map(|(qty, price)| qty * price)
            .sum::<f64>();

        recovered.timestamp = target_time;

        Some(recovered)
    }
}
```

## Автоматическое планирование резервного копирования

Торговым системам необходимо автоматическое и надёжное расписание резервного копирования.

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

        // Поток полного резервного копирования
        let db_full = Arc::clone(&self.db);
        let running_full = Arc::clone(&self.running);
        let full_interval = self.full_backup_interval_hours;

        thread::spawn(move || {
            while *running_full.lock().unwrap() {
                thread::sleep(Duration::from_secs(full_interval * 3600));

                if *running_full.lock().unwrap() {
                    let db = db_full.lock().unwrap();
                    match db.full_backup() {
                        Ok(path) => println!("[Планировщик] Полная копия создана: {}", path),
                        Err(e) => eprintln!("[Планировщик] Ошибка полной копии: {}", e),
                    }
                }
            }
        });

        // Поток инкрементного резервного копирования
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
                            println!("[Планировщик] Инкрементная копия создана: {}", path);
                        }
                        Ok(None) => {
                            println!("[Планировщик] Нет новых данных для инкрементной копии");
                        }
                        Err(e) => eprintln!("[Планировщик] Ошибка инкрементной копии: {}", e),
                    }
                }
            }
        });

        println!("Планировщик резервного копирования запущен:");
        println!("  - Полная копия каждые {} часов", full_interval);
        println!("  - Инкрементная копия каждые {} минут", inc_interval);
    }

    fn stop(&self) {
        *self.running.lock().unwrap() = false;
        println!("Планировщик резервного копирования остановлен");
    }
}
```

## Проверка и целостность резервных копий

Всегда проверяйте, что ваши резервные копии можно успешно восстановить.

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

    /// Проверяет целостность и читаемость файла резервной копии
    fn verify_backup(&self, backup_path: &str) -> VerificationResult {
        let mut errors = Vec::new();
        let mut trades_count = 0;

        // Читаем файл
        let mut file = match File::open(backup_path) {
            Ok(f) => f,
            Err(e) => {
                return VerificationResult {
                    backup_path: backup_path.to_string(),
                    is_valid: false,
                    trades_count: 0,
                    checksum: String::new(),
                    errors: vec![format!("Не удаётся открыть файл: {}", e)],
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
                errors: vec![format!("Не удаётся прочитать файл: {}", e)],
            };
        }

        // Вычисляем контрольную сумму
        let checksum = format!("{:x}", md5::compute(&contents));

        // Парсим JSON
        let json: serde_json::Value = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(e) => {
                return VerificationResult {
                    backup_path: backup_path.to_string(),
                    is_valid: false,
                    trades_count: 0,
                    checksum,
                    errors: vec![format!("Некорректный JSON: {}", e)],
                };
            }
        };

        // Проверяем структуру
        if json.get("backup_type").is_none() {
            errors.push("Отсутствует поле backup_type".to_string());
        }

        if json.get("timestamp").is_none() {
            errors.push("Отсутствует поле timestamp".to_string());
        }

        if let Some(trades) = json.get("trades") {
            if let Some(arr) = trades.as_array() {
                trades_count = arr.len();

                // Проверяем, что каждая сделка имеет обязательные поля
                for (i, trade) in arr.iter().enumerate() {
                    if trade.get("id").is_none() {
                        errors.push(format!("Сделка {} не имеет id", i));
                    }
                    if trade.get("symbol").is_none() {
                        errors.push(format!("Сделка {} не имеет symbol", i));
                    }
                    if trade.get("price").is_none() {
                        errors.push(format!("Сделка {} не имеет price", i));
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

    /// Проверяет все резервные копии в директории
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

## Полный пример системы резервного копирования для трейдинга

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Utc;

fn main() {
    // Инициализируем торговую базу данных
    let db = Arc::new(Mutex::new(TradingDatabase::new("./trading_backups")));

    // Добавляем примеры сделок
    {
        let mut db = db.lock().unwrap();

        // Симулируем торговую активность
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

            // Обновляем портфель
            let cost = qty * price;
            if *side == "BUY" {
                db.portfolio.cash -= cost;
                db.portfolio.positions.push((symbol.to_string(), *qty, *price));
            } else {
                db.portfolio.cash += cost;
            }
        }
    }

    // Создаём полную резервную копию
    {
        let db = db.lock().unwrap();
        match db.full_backup() {
            Ok(path) => println!("\nПолная резервная копия успешно создана: {}", path),
            Err(e) => eprintln!("Ошибка резервного копирования: {}", e),
        }
    }

    // Инициализируем менеджер инкрементных копий
    let mut inc_manager = IncrementalBackupManager::new();
    inc_manager.last_backup_trade_id = 5; // Предполагаем, что первые 5 сделок уже сохранены

    // Добавляем ещё сделок
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

    // Создаём инкрементную резервную копию
    {
        let db = db.lock().unwrap();
        match inc_manager.incremental_backup(&db) {
            Ok(Some(path)) => println!("Инкрементная копия создана: {}", path),
            Ok(None) => println!("Нет новых данных для сохранения"),
            Err(e) => eprintln!("Ошибка инкрементного копирования: {}", e),
        }
    }

    // Проверяем резервные копии
    let verifier = BackupVerifier::new("./trading_backups");
    let results = verifier.verify_all_backups();

    println!("\n=== Отчёт о проверке резервных копий ===");
    for result in results {
        println!("\nФайл: {}", result.backup_path);
        println!("  Валидный: {}", result.is_valid);
        println!("  Сделок: {}", result.trades_count);
        println!("  Контрольная сумма: {}", result.checksum);
        if !result.errors.is_empty() {
            println!("  Ошибки:");
            for error in &result.errors {
                println!("    - {}", error);
            }
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Полная резервная копия | Полная копия всех данных базы |
| Инкрементная резервная копия | Только изменения с последней копии (любого типа) |
| Дифференциальная резервная копия | Все изменения с последней полной копии |
| Восстановление на момент времени | Восстановление данных на конкретный момент |
| Лог транзакций | Запись всех операций для воспроизведения |
| Проверка резервных копий | Подтверждение целостности и возможности восстановления |
| Планирование резервного копирования | Автоматическое регулярное создание копий |

## Домашнее задание

1. **Политика ротации резервных копий**: Реализуй систему ротации резервных копий, которая:
   - Хранит последние 7 ежедневных копий
   - Хранит последние 4 еженедельные копии
   - Хранит последние 12 ежемесячных копий
   - Автоматически удаляет более старые копии

2. **Сжатые резервные копии**: Расширь систему резервного копирования для:
   - Сжатия файлов резервных копий с использованием gzip или zstd
   - Вычисления и сохранения контрольных сумм для проверки целостности
   - Реализации распаковки при восстановлении

3. **Синхронизация с удалённым хранилищем**: Создай систему резервного копирования, которая:
   - Загружает резервные копии в удалённое место (симулируй локальной "удалённой" директорией)
   - Отслеживает, какие копии были синхронизированы
   - Реализует логику повторных попыток при сбоях загрузки
   - Предоставляет отчёт о статусе синхронизации

4. **Тестирование восстановления**: Реализуй автоматический тест восстановления, который:
   - Создаёт резервную копию
   - Восстанавливает её в отдельную тестовую базу данных
   - Проверяет, что все сделки и состояние портфеля совпадают
   - Сообщает о любых расхождениях
   - Выполняет эту проверку по расписанию

## Навигация

[← Предыдущий день](../240-migrations-evolving-schema/ru.md) | [Следующий день →](../242-replication-data-copies/ru.md)
