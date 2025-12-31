# День 224: Миграции: эволюция схемы

## Аналогия из трейдинга

Представь, что ты запустил торгового бота, который записывает сделки в базу данных. Сначала тебе нужны только цена и объём. Но потом ты понимаешь, что хочешь добавить комиссию, тип ордера, время исполнения. Ты не можешь просто удалить старую базу — там ценная история сделок! Тебе нужно **мигрировать** схему, сохранив все данные.

Это как модернизация торговой системы биржи: нужно добавить новые поля для новых типов ордеров, но нельзя остановить торговлю и потерять исторические данные. Миграции — это механизм безопасного обновления структуры базы данных.

В реальном трейдинге это происходит постоянно:
- Добавляем поле `stop_loss` к таблице ордеров
- Разделяем таблицу сделок на `trades` и `executions`
- Добавляем новые индексы для ускорения аналитических запросов
- Меняем тип данных цены с `REAL` на `DECIMAL` для точности

## Что такое миграции?

**Миграция** — это скрипт, который изменяет схему базы данных. Миграции:

1. **Версионируемые** — каждая миграция имеет уникальный идентификатор
2. **Последовательные** — применяются строго по порядку
3. **Обратимые** — можно откатить изменения (rollback)
4. **Идемпотентные** — повторное применение не ломает систему

```
Миграция 001: CREATE TABLE trades
Миграция 002: ADD COLUMN commission
Миграция 003: CREATE INDEX idx_trades_date
Миграция 004: ADD COLUMN order_type
```

## Простые миграции с rusqlite

Начнём с ручной реализации миграций:

```rust
use rusqlite::{Connection, Result};

/// Структура для отслеживания версии схемы
fn get_schema_version(conn: &Connection) -> Result<i32> {
    // Создаём таблицу версий если её нет
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        [],
    )?;

    // Получаем текущую версию
    let version: Result<i32> = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    );

    version.or(Ok(0))
}

fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        [version],
    )?;
    Ok(())
}

/// Миграция 1: Создаём базовую таблицу сделок
fn migration_001_create_trades(conn: &Connection) -> Result<()> {
    println!("Применяем миграцию 001: создание таблицы trades");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    Ok(())
}

/// Миграция 2: Добавляем поле комиссии
fn migration_002_add_commission(conn: &Connection) -> Result<()> {
    println!("Применяем миграцию 002: добавление поля commission");

    conn.execute(
        "ALTER TABLE trades ADD COLUMN commission REAL DEFAULT 0.0",
        [],
    )?;

    Ok(())
}

/// Миграция 3: Добавляем индекс по дате
fn migration_003_add_date_index(conn: &Connection) -> Result<()> {
    println!("Применяем миграцию 003: создание индекса по дате");

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_trades_created_at
         ON trades(created_at)",
        [],
    )?;

    Ok(())
}

/// Миграция 4: Добавляем тип ордера
fn migration_004_add_order_type(conn: &Connection) -> Result<()> {
    println!("Применяем миграцию 004: добавление поля order_type");

    conn.execute(
        "ALTER TABLE trades ADD COLUMN order_type TEXT DEFAULT 'MARKET'",
        [],
    )?;

    Ok(())
}

/// Применяем все миграции последовательно
fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version = get_schema_version(conn)?;
    println!("Текущая версия схемы: {}", current_version);

    let migrations: Vec<(i32, fn(&Connection) -> Result<()>)> = vec![
        (1, migration_001_create_trades),
        (2, migration_002_add_commission),
        (3, migration_003_add_date_index),
        (4, migration_004_add_order_type),
    ];

    for (version, migration_fn) in migrations {
        if version > current_version {
            // Выполняем миграцию в транзакции
            let tx = conn.unchecked_transaction()?;
            migration_fn(conn)?;
            set_schema_version(conn, version)?;
            tx.commit()?;
            println!("Миграция {} успешно применена", version);
        }
    }

    println!("Все миграции применены. Версия схемы: {}",
             get_schema_version(conn)?);
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    run_migrations(&conn)?;

    // Тестируем вставку данных
    conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity, commission, order_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        ("BTC/USDT", "BUY", 42000.0, 0.5, 21.0, "LIMIT"),
    )?;

    println!("Сделка успешно добавлена!");

    Ok(())
}
```

## Миграции с транзакциями и откатом

Для безопасности миграций важно использовать транзакции:

```rust
use rusqlite::{Connection, Result, Transaction};

struct Migration {
    version: i32,
    name: String,
    up: String,    // SQL для применения
    down: String,  // SQL для отката
}

impl Migration {
    fn new(version: i32, name: &str, up: &str, down: &str) -> Self {
        Migration {
            version,
            name: name.to_string(),
            up: up.to_string(),
            down: down.to_string(),
        }
    }
}

struct MigrationRunner {
    migrations: Vec<Migration>,
}

impl MigrationRunner {
    fn new() -> Self {
        MigrationRunner {
            migrations: Vec::new(),
        }
    }

    fn add_migration(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }

    fn init_schema_tracking(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    fn get_applied_versions(&self, conn: &Connection) -> Result<Vec<i32>> {
        let mut stmt = conn.prepare(
            "SELECT version FROM schema_migrations ORDER BY version"
        )?;

        let versions = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<i32>>>()?;

        Ok(versions)
    }

    fn apply_migration(&self, conn: &Connection, migration: &Migration) -> Result<()> {
        println!("Применяем миграцию {}: {}", migration.version, migration.name);

        // Выполняем SQL миграции
        conn.execute_batch(&migration.up)?;

        // Записываем применённую миграцию
        conn.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            (&migration.version, &migration.name),
        )?;

        Ok(())
    }

    fn rollback_migration(&self, conn: &Connection, migration: &Migration) -> Result<()> {
        println!("Откатываем миграцию {}: {}", migration.version, migration.name);

        // Выполняем откат
        conn.execute_batch(&migration.down)?;

        // Удаляем запись о миграции
        conn.execute(
            "DELETE FROM schema_migrations WHERE version = ?1",
            [migration.version],
        )?;

        Ok(())
    }

    fn migrate(&self, conn: &mut Connection) -> Result<()> {
        self.init_schema_tracking(conn)?;
        let applied = self.get_applied_versions(conn)?;

        for migration in &self.migrations {
            if !applied.contains(&migration.version) {
                let tx = conn.transaction()?;
                self.apply_migration(&tx, migration)?;
                tx.commit()?;
            }
        }

        Ok(())
    }

    fn rollback(&self, conn: &mut Connection, steps: usize) -> Result<()> {
        self.init_schema_tracking(conn)?;
        let mut applied = self.get_applied_versions(conn)?;
        applied.reverse(); // От новых к старым

        for (i, version) in applied.iter().enumerate() {
            if i >= steps {
                break;
            }

            if let Some(migration) = self.migrations.iter()
                .find(|m| m.version == *version)
            {
                let tx = conn.transaction()?;
                self.rollback_migration(&tx, migration)?;
                tx.commit()?;
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut runner = MigrationRunner::new();

    // Добавляем миграции для торговой системы
    runner.add_migration(Migration::new(
        1,
        "create_orders_table",
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            order_type TEXT NOT NULL,
            price REAL,
            quantity REAL NOT NULL,
            status TEXT DEFAULT 'NEW',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        "DROP TABLE orders",
    ));

    runner.add_migration(Migration::new(
        2,
        "create_positions_table",
        "CREATE TABLE positions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT UNIQUE NOT NULL,
            quantity REAL NOT NULL DEFAULT 0,
            avg_price REAL NOT NULL DEFAULT 0,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        "DROP TABLE positions",
    ));

    runner.add_migration(Migration::new(
        3,
        "add_order_client_id",
        "ALTER TABLE orders ADD COLUMN client_order_id TEXT",
        // В SQLite нельзя удалить колонку напрямую
        // Поэтому rollback здесь сложный - пересоздаём таблицу
        "CREATE TABLE orders_backup AS SELECT
            id, symbol, side, order_type, price, quantity, status, created_at
         FROM orders;
         DROP TABLE orders;
         ALTER TABLE orders_backup RENAME TO orders",
    ));

    runner.add_migration(Migration::new(
        4,
        "add_indexes",
        "CREATE INDEX idx_orders_symbol ON orders(symbol);
         CREATE INDEX idx_orders_status ON orders(status);
         CREATE INDEX idx_positions_symbol ON positions(symbol)",
        "DROP INDEX idx_orders_symbol;
         DROP INDEX idx_orders_status;
         DROP INDEX idx_positions_symbol",
    ));

    let mut conn = Connection::open("trading_advanced.db")?;

    // Применяем все миграции
    println!("=== Применяем миграции ===");
    runner.migrate(&mut conn)?;

    // Откатываем последнюю миграцию (для демонстрации)
    println!("\n=== Откатываем последнюю миграцию ===");
    runner.rollback(&mut conn, 1)?;

    // Снова применяем
    println!("\n=== Применяем миграции заново ===");
    runner.migrate(&mut conn)?;

    println!("\nМиграции завершены успешно!");

    Ok(())
}
```

## Паттерны безопасных миграций

### 1. Добавление колонки с значением по умолчанию

```rust
use rusqlite::{Connection, Result};

/// Безопасное добавление новой колонки
fn add_stop_loss_column(conn: &Connection) -> Result<()> {
    // Сначала добавляем колонку с NULL
    conn.execute(
        "ALTER TABLE orders ADD COLUMN stop_loss REAL",
        [],
    )?;

    // Затем обновляем существующие записи
    conn.execute(
        "UPDATE orders SET stop_loss = price * 0.95 WHERE stop_loss IS NULL",
        [],
    )?;

    println!("Колонка stop_loss добавлена и заполнена");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    // Создаём таблицу
    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            price REAL NOT NULL
        )",
        [],
    )?;

    // Добавляем тестовые данные
    conn.execute("INSERT INTO orders (symbol, price) VALUES ('BTC', 42000.0)", [])?;
    conn.execute("INSERT INTO orders (symbol, price) VALUES ('ETH', 2800.0)", [])?;

    // Применяем миграцию
    add_stop_loss_column(&conn)?;

    // Проверяем результат
    let mut stmt = conn.prepare("SELECT symbol, price, stop_loss FROM orders")?;
    let orders = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, f64>(1)?,
            row.get::<_, f64>(2)?,
        ))
    })?;

    for order in orders {
        let (symbol, price, stop_loss) = order?;
        println!("{}: price={}, stop_loss={}", symbol, price, stop_loss);
    }

    Ok(())
}
```

### 2. Переименование колонки (через копирование)

```rust
use rusqlite::{Connection, Result};

/// SQLite не поддерживает RENAME COLUMN напрямую (до версии 3.25)
/// Используем пересоздание таблицы
fn rename_column_safe(conn: &Connection) -> Result<()> {
    // Начинаем транзакцию
    conn.execute("BEGIN TRANSACTION", [])?;

    // 1. Создаём новую таблицу с правильными именами
    conn.execute(
        "CREATE TABLE trades_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            entry_price REAL NOT NULL,  -- было 'price'
            quantity REAL NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 2. Копируем данные
    conn.execute(
        "INSERT INTO trades_new (id, symbol, side, entry_price, quantity, created_at)
         SELECT id, symbol, side, price, quantity, created_at FROM trades",
        [],
    )?;

    // 3. Удаляем старую таблицу
    conn.execute("DROP TABLE trades", [])?;

    // 4. Переименовываем новую
    conn.execute("ALTER TABLE trades_new RENAME TO trades", [])?;

    // 5. Пересоздаём индексы
    conn.execute(
        "CREATE INDEX idx_trades_symbol ON trades(symbol)",
        [],
    )?;

    conn.execute("COMMIT", [])?;

    println!("Колонка price переименована в entry_price");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity) VALUES ('BTC', 'BUY', 42000.0, 0.5)",
        [],
    )?;

    rename_column_safe(&conn)?;

    // Проверяем новую структуру
    let mut stmt = conn.prepare("SELECT symbol, entry_price FROM trades")?;
    let trades = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?;

    for trade in trades {
        let (symbol, entry_price) = trade?;
        println!("Trade: {} @ {}", symbol, entry_price);
    }

    Ok(())
}
```

### 3. Разделение таблицы (нормализация)

```rust
use rusqlite::{Connection, Result};

/// Разделяем trades на trades + instruments
fn normalize_trades_table(conn: &Connection) -> Result<()> {
    // Создаём таблицу инструментов
    conn.execute(
        "CREATE TABLE IF NOT EXISTS instruments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT UNIQUE NOT NULL,
            base_asset TEXT NOT NULL,
            quote_asset TEXT NOT NULL,
            tick_size REAL DEFAULT 0.01,
            lot_size REAL DEFAULT 0.001
        )",
        [],
    )?;

    // Заполняем инструменты из существующих trades
    conn.execute(
        "INSERT OR IGNORE INTO instruments (symbol, base_asset, quote_asset)
         SELECT DISTINCT symbol,
                SUBSTR(symbol, 1, INSTR(symbol, '/') - 1),
                SUBSTR(symbol, INSTR(symbol, '/') + 1)
         FROM trades
         WHERE symbol LIKE '%/%'",
        [],
    )?;

    // Добавляем внешний ключ к trades
    conn.execute(
        "ALTER TABLE trades ADD COLUMN instrument_id INTEGER
         REFERENCES instruments(id)",
        [],
    )?;

    // Заполняем instrument_id
    conn.execute(
        "UPDATE trades SET instrument_id = (
            SELECT id FROM instruments WHERE instruments.symbol = trades.symbol
        )",
        [],
    )?;

    println!("Таблица trades нормализована");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE trades (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL
        )",
        [],
    )?;

    conn.execute("INSERT INTO trades (symbol, price, quantity) VALUES ('BTC/USDT', 42000.0, 0.5)", [])?;
    conn.execute("INSERT INTO trades (symbol, price, quantity) VALUES ('ETH/USDT', 2800.0, 2.0)", [])?;
    conn.execute("INSERT INTO trades (symbol, price, quantity) VALUES ('BTC/USDT', 42100.0, 0.3)", [])?;

    normalize_trades_table(&conn)?;

    // Проверяем результат
    let mut stmt = conn.prepare(
        "SELECT t.id, i.symbol, i.base_asset, t.price, t.quantity
         FROM trades t
         JOIN instruments i ON t.instrument_id = i.id"
    )?;

    let results = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, f64>(4)?,
        ))
    })?;

    println!("\nДанные после нормализации:");
    for result in results {
        let (id, symbol, base, price, qty) = result?;
        println!("  Trade #{}: {} ({}) @ {} qty={}", id, symbol, base, price, qty);
    }

    Ok(())
}
```

## Практический пример: Система миграций для торгового бота

```rust
use rusqlite::{Connection, Result};
use std::collections::HashMap;

/// Полноценная система миграций с метаданными
struct TradingMigrations {
    conn: Connection,
}

impl TradingMigrations {
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let migrations = TradingMigrations { conn };
        migrations.init()?;
        Ok(migrations)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version TEXT UNIQUE NOT NULL,
                description TEXT NOT NULL,
                applied_at TEXT DEFAULT CURRENT_TIMESTAMP,
                checksum TEXT,
                execution_time_ms INTEGER
            )",
            [],
        )?;
        Ok(())
    }

    fn is_applied(&self, version: &str) -> Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM _migrations WHERE version = ?1",
            [version],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    fn apply(&self, version: &str, description: &str, sql: &str) -> Result<()> {
        if self.is_applied(version)? {
            println!("Миграция {} уже применена, пропускаем", version);
            return Ok(());
        }

        println!("Применяем миграцию {}: {}", version, description);

        let start = std::time::Instant::now();

        // Выполняем миграцию
        self.conn.execute_batch(sql)?;

        let elapsed = start.elapsed().as_millis() as i64;

        // Записываем информацию о миграции
        self.conn.execute(
            "INSERT INTO _migrations (version, description, execution_time_ms)
             VALUES (?1, ?2, ?3)",
            (version, description, elapsed),
        )?;

        println!("  Выполнено за {} мс", elapsed);
        Ok(())
    }

    fn run_all(&self) -> Result<()> {
        // V001: Базовые таблицы
        self.apply(
            "V001",
            "Создание таблицы аккаунтов",
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                exchange TEXT NOT NULL,
                api_key_hash TEXT,
                is_active INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )"
        )?;

        // V002: Таблица балансов
        self.apply(
            "V002",
            "Создание таблицы балансов",
            "CREATE TABLE IF NOT EXISTS balances (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL REFERENCES accounts(id),
                asset TEXT NOT NULL,
                free REAL NOT NULL DEFAULT 0,
                locked REAL NOT NULL DEFAULT 0,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(account_id, asset)
            );
            CREATE INDEX idx_balances_account ON balances(account_id)"
        )?;

        // V003: Таблица ордеров
        self.apply(
            "V003",
            "Создание таблицы ордеров",
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL REFERENCES accounts(id),
                client_order_id TEXT UNIQUE,
                exchange_order_id TEXT,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
                order_type TEXT NOT NULL,
                price REAL,
                quantity REAL NOT NULL,
                filled_quantity REAL DEFAULT 0,
                status TEXT DEFAULT 'NEW',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX idx_orders_account ON orders(account_id);
            CREATE INDEX idx_orders_symbol ON orders(symbol);
            CREATE INDEX idx_orders_status ON orders(status)"
        )?;

        // V004: Таблица сделок
        self.apply(
            "V004",
            "Создание таблицы исполнений",
            "CREATE TABLE IF NOT EXISTS executions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL REFERENCES orders(id),
                exchange_trade_id TEXT,
                price REAL NOT NULL,
                quantity REAL NOT NULL,
                commission REAL DEFAULT 0,
                commission_asset TEXT,
                executed_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX idx_executions_order ON executions(order_id)"
        )?;

        // V005: Таблица позиций
        self.apply(
            "V005",
            "Создание таблицы позиций",
            "CREATE TABLE IF NOT EXISTS positions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL REFERENCES accounts(id),
                symbol TEXT NOT NULL,
                side TEXT NOT NULL CHECK(side IN ('LONG', 'SHORT')),
                quantity REAL NOT NULL,
                entry_price REAL NOT NULL,
                current_price REAL,
                unrealized_pnl REAL,
                realized_pnl REAL DEFAULT 0,
                opened_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(account_id, symbol)
            );
            CREATE INDEX idx_positions_account ON positions(account_id)"
        )?;

        // V006: Добавляем риск-менеджмент
        self.apply(
            "V006",
            "Добавление полей риск-менеджмента к ордерам",
            "ALTER TABLE orders ADD COLUMN stop_loss REAL;
             ALTER TABLE orders ADD COLUMN take_profit REAL;
             ALTER TABLE orders ADD COLUMN trailing_stop_percent REAL"
        )?;

        // V007: Таблица стратегий
        self.apply(
            "V007",
            "Создание таблицы стратегий",
            "CREATE TABLE IF NOT EXISTS strategies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                description TEXT,
                parameters TEXT,  -- JSON
                is_active INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            ALTER TABLE orders ADD COLUMN strategy_id INTEGER REFERENCES strategies(id)"
        )?;

        // V008: Добавляем аудит
        self.apply(
            "V008",
            "Создание таблицы аудита",
            "CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_id INTEGER NOT NULL,
                action TEXT NOT NULL,
                old_value TEXT,
                new_value TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX idx_audit_entity ON audit_log(entity_type, entity_id)"
        )?;

        Ok(())
    }

    fn status(&self) -> Result<()> {
        println!("\n=== Статус миграций ===");

        let mut stmt = self.conn.prepare(
            "SELECT version, description, applied_at, execution_time_ms
             FROM _migrations ORDER BY version"
        )?;

        let migrations = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;

        for migration in migrations {
            let (version, desc, applied_at, time_ms) = migration?;
            println!("{}: {} (применена: {}, {}ms)",
                     version, desc, applied_at, time_ms);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let migrations = TradingMigrations::new("trading_bot.db")?;

    // Применяем все миграции
    migrations.run_all()?;

    // Показываем статус
    migrations.status()?;

    // Проверяем структуру
    println!("\n=== Проверка структуры ===");

    let tables: Vec<String> = migrations.conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;

    println!("Созданные таблицы: {:?}", tables);

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Миграция | Скрипт изменения схемы БД |
| Версионирование | Каждая миграция имеет уникальный идентификатор |
| Транзакции | Миграции выполняются атомарно |
| Откат (Rollback) | Возможность отменить миграцию |
| Метаданные | Хранение информации о применённых миграциях |
| Безопасные паттерны | Добавление с default, переименование через копирование |

## Домашнее задание

1. **Система версий**: Реализуй систему миграций, которая поддерживает семантическое версионирование (V1.0.0, V1.1.0, V2.0.0). Добавь проверку совместимости при применении.

2. **Миграции с данными**: Создай миграцию, которая не только меняет схему, но и трансформирует данные. Например, разбивает поле `full_name` на `first_name` и `last_name`.

3. **Проверка целостности**: Добавь в систему миграций контрольную сумму SQL-кода. При повторном запуске проверяй, что код миграции не изменился с момента первого применения.

4. **Dry-run режим**: Реализуй режим, который показывает какие миграции будут применены без фактического выполнения. Полезно для проверки в production.

## Навигация

[← Предыдущий день](../223-indexes-fast-search/ru.md) | [Следующий день →](../225-postgresql-production/ru.md)
