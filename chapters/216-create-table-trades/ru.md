# День 216: CREATE TABLE: таблица сделок

## Аналогия из трейдинга

Представь, что ты трейдер, который ведёт журнал сделок. Каждая сделка имеет определённые характеристики: дату, тикер, направление (покупка или продажа), цену, количество, комиссию. Раньше трейдеры записывали всё в бумажный журнал, где каждая колонка — это определённый тип данных.

В базе данных **CREATE TABLE** — это создание такого журнала. Ты заранее определяешь:
- Какие колонки будут (название, тикер, цена...)
- Какой тип данных в каждой колонке (число, текст, дата...)
- Какие ограничения (цена не может быть отрицательной, количество — целое число)

Это как создание идеального торгового журнала, где каждая запись будет структурирована одинаково.

## Что такое CREATE TABLE?

`CREATE TABLE` — это SQL-команда для создания новой таблицы в базе данных. Таблица — это структурированное хранилище данных с заранее определёнными колонками (полями) и их типами.

### Базовый синтаксис SQL

```sql
CREATE TABLE имя_таблицы (
    колонка1 ТИП_ДАННЫХ ОГРАНИЧЕНИЯ,
    колонка2 ТИП_ДАННЫХ ОГРАНИЧЕНИЯ,
    ...
);
```

## Типы данных в SQLite

| Тип SQLite | Описание | Пример в трейдинге |
|------------|----------|-------------------|
| INTEGER | Целое число | Количество акций |
| REAL | Число с плавающей точкой | Цена актива |
| TEXT | Строка | Название тикера |
| BLOB | Бинарные данные | Сериализованные данные |
| NULL | Отсутствие значения | Нереализованная прибыль |

## Создание таблицы сделок в Rust

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // Создаём или открываем базу данных
    let conn = Connection::open("trading.db")?;

    // Создаём таблицу сделок
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            price REAL NOT NULL CHECK(price > 0),
            quantity REAL NOT NULL CHECK(quantity > 0),
            commission REAL DEFAULT 0.0,
            executed_at TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    println!("Таблица trades успешно создана!");
    Ok(())
}
```

## Разбор полей таблицы

| Поле | Тип | Назначение |
|------|-----|------------|
| `id` | INTEGER PRIMARY KEY | Уникальный идентификатор сделки |
| `symbol` | TEXT NOT NULL | Тикер инструмента (BTC, AAPL) |
| `side` | TEXT NOT NULL | Направление: BUY или SELL |
| `price` | REAL NOT NULL | Цена исполнения |
| `quantity` | REAL NOT NULL | Объём сделки |
| `commission` | REAL DEFAULT 0.0 | Комиссия биржи |
| `executed_at` | TEXT NOT NULL | Время исполнения |
| `created_at` | TEXT | Время создания записи |

## Ограничения (Constraints)

### PRIMARY KEY — уникальный идентификатор

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            client_order_id TEXT UNIQUE,
            symbol TEXT NOT NULL
        )",
        [],
    )?;

    // AUTOINCREMENT гарантирует уникальность даже после удаления
    conn.execute(
        "CREATE TABLE trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER,
            price REAL
        )",
        [],
    )?;

    println!("Таблицы с PRIMARY KEY созданы!");
    Ok(())
}
```

### NOT NULL — обязательное поле

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE positions (
            symbol TEXT NOT NULL,
            quantity REAL NOT NULL,
            entry_price REAL NOT NULL,
            stop_loss REAL,  -- может быть NULL
            take_profit REAL -- может быть NULL
        )",
        [],
    )?;

    // Попытка вставить NULL в NOT NULL поле вызовет ошибку
    let result = conn.execute(
        "INSERT INTO positions (symbol, quantity, entry_price) VALUES (NULL, 100.0, 50000.0)",
        [],
    );

    match result {
        Ok(_) => println!("Вставка успешна"),
        Err(e) => println!("Ошибка: {}", e),
    }

    Ok(())
}
```

### CHECK — проверка условий

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    // CHECK ограничения для торговых данных
    conn.execute(
        "CREATE TABLE trades (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT CHECK(side IN ('BUY', 'SELL')),
            price REAL CHECK(price > 0),
            quantity REAL CHECK(quantity > 0),
            leverage INTEGER CHECK(leverage >= 1 AND leverage <= 100)
        )",
        [],
    )?;

    // Эта вставка успешна
    conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity, leverage)
         VALUES ('BTC', 'BUY', 50000.0, 0.1, 10)",
        [],
    )?;
    println!("Корректная сделка добавлена");

    // Эта вставка вызовет ошибку — отрицательная цена
    let result = conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity, leverage)
         VALUES ('BTC', 'BUY', -100.0, 0.1, 10)",
        [],
    );

    match result {
        Ok(_) => println!("Вставка успешна"),
        Err(e) => println!("Ошибка CHECK: {}", e),
    }

    Ok(())
}
```

### DEFAULT — значение по умолчанию

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            status TEXT DEFAULT 'PENDING',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            commission_rate REAL DEFAULT 0.001
        )",
        [],
    )?;

    // Вставляем только обязательные поля
    conn.execute(
        "INSERT INTO orders (symbol, side, price, quantity)
         VALUES ('ETH', 'BUY', 3000.0, 1.0)",
        [],
    )?;

    // Проверяем, что значения по умолчанию применились
    let mut stmt = conn.prepare("SELECT symbol, status, commission_rate FROM orders")?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let symbol: String = row.get(0)?;
        let status: String = row.get(1)?;
        let commission: f64 = row.get(2)?;
        println!("Символ: {}, Статус: {}, Комиссия: {}", symbol, status, commission);
    }

    Ok(())
}
```

### UNIQUE — уникальные значения

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    conn.execute(
        "CREATE TABLE api_keys (
            id INTEGER PRIMARY KEY,
            exchange TEXT NOT NULL,
            api_key TEXT UNIQUE NOT NULL,
            secret_hash TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Первый ключ добавляется успешно
    conn.execute(
        "INSERT INTO api_keys (exchange, api_key, secret_hash)
         VALUES ('Binance', 'key123', 'hash456')",
        [],
    )?;
    println!("Первый API ключ добавлен");

    // Попытка добавить дубликат вызовет ошибку
    let result = conn.execute(
        "INSERT INTO api_keys (exchange, api_key, secret_hash)
         VALUES ('Binance', 'key123', 'hash789')",
        [],
    );

    match result {
        Ok(_) => println!("Дубликат добавлен"),
        Err(e) => println!("Ошибка UNIQUE: {}", e),
    }

    Ok(())
}
```

## Полный пример: Торговая система

```rust
use rusqlite::{Connection, Result};

fn create_trading_database() -> Result<Connection> {
    let conn = Connection::open("trading_system.db")?;

    // Таблица инструментов
    conn.execute(
        "CREATE TABLE IF NOT EXISTS instruments (
            id INTEGER PRIMARY KEY,
            symbol TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            instrument_type TEXT CHECK(instrument_type IN ('SPOT', 'FUTURES', 'OPTION')),
            base_currency TEXT NOT NULL,
            quote_currency TEXT NOT NULL,
            tick_size REAL NOT NULL CHECK(tick_size > 0),
            lot_size REAL NOT NULL CHECK(lot_size > 0),
            is_active INTEGER DEFAULT 1
        )",
        [],
    )?;

    // Таблица аккаунтов
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            exchange TEXT NOT NULL,
            balance REAL DEFAULT 0.0,
            currency TEXT DEFAULT 'USDT',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Таблица ордеров
    conn.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id INTEGER NOT NULL,
            instrument_id INTEGER NOT NULL,
            client_order_id TEXT UNIQUE,
            exchange_order_id TEXT,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            order_type TEXT NOT NULL CHECK(order_type IN ('MARKET', 'LIMIT', 'STOP', 'STOP_LIMIT')),
            price REAL,
            quantity REAL NOT NULL CHECK(quantity > 0),
            filled_quantity REAL DEFAULT 0.0,
            status TEXT DEFAULT 'NEW' CHECK(status IN ('NEW', 'PARTIALLY_FILLED', 'FILLED', 'CANCELLED', 'REJECTED')),
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (account_id) REFERENCES accounts(id),
            FOREIGN KEY (instrument_id) REFERENCES instruments(id)
        )",
        [],
    )?;

    // Таблица сделок (исполненные ордера)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER NOT NULL,
            account_id INTEGER NOT NULL,
            instrument_id INTEGER NOT NULL,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            price REAL NOT NULL CHECK(price > 0),
            quantity REAL NOT NULL CHECK(quantity > 0),
            commission REAL DEFAULT 0.0,
            commission_asset TEXT,
            realized_pnl REAL,
            executed_at TEXT NOT NULL,
            FOREIGN KEY (order_id) REFERENCES orders(id),
            FOREIGN KEY (account_id) REFERENCES accounts(id),
            FOREIGN KEY (instrument_id) REFERENCES instruments(id)
        )",
        [],
    )?;

    // Таблица позиций
    conn.execute(
        "CREATE TABLE IF NOT EXISTS positions (
            id INTEGER PRIMARY KEY,
            account_id INTEGER NOT NULL,
            instrument_id INTEGER NOT NULL,
            side TEXT CHECK(side IN ('LONG', 'SHORT')),
            quantity REAL NOT NULL DEFAULT 0.0,
            entry_price REAL,
            unrealized_pnl REAL,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (account_id) REFERENCES accounts(id),
            FOREIGN KEY (instrument_id) REFERENCES instruments(id),
            UNIQUE(account_id, instrument_id)
        )",
        [],
    )?;

    // Таблица истории цен (OHLCV)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS price_history (
            id INTEGER PRIMARY KEY,
            instrument_id INTEGER NOT NULL,
            timeframe TEXT NOT NULL CHECK(timeframe IN ('1m', '5m', '15m', '1h', '4h', '1d')),
            open_time TEXT NOT NULL,
            open REAL NOT NULL,
            high REAL NOT NULL,
            low REAL NOT NULL,
            close REAL NOT NULL,
            volume REAL NOT NULL,
            FOREIGN KEY (instrument_id) REFERENCES instruments(id),
            UNIQUE(instrument_id, timeframe, open_time)
        )",
        [],
    )?;

    println!("База данных торговой системы создана успешно!");
    Ok(conn)
}

fn main() -> Result<()> {
    let conn = create_trading_database()?;

    // Проверяем созданные таблицы
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    )?;

    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    println!("\nСозданные таблицы:");
    for table in tables {
        println!("  - {}", table);
    }

    Ok(())
}
```

## Внешние ключи (FOREIGN KEY)

Внешние ключи связывают таблицы между собой, обеспечивая целостность данных:

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    // Включаем поддержку внешних ключей (в SQLite выключена по умолчанию!)
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Создаём родительскую таблицу
    conn.execute(
        "CREATE TABLE portfolios (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT
        )",
        [],
    )?;

    // Создаём дочернюю таблицу с внешним ключом
    conn.execute(
        "CREATE TABLE holdings (
            id INTEGER PRIMARY KEY,
            portfolio_id INTEGER NOT NULL,
            symbol TEXT NOT NULL,
            quantity REAL NOT NULL,
            avg_price REAL NOT NULL,
            FOREIGN KEY (portfolio_id) REFERENCES portfolios(id)
                ON DELETE CASCADE
                ON UPDATE CASCADE
        )",
        [],
    )?;

    // Добавляем портфель
    conn.execute(
        "INSERT INTO portfolios (id, name, description) VALUES (1, 'Основной', 'Долгосрочные инвестиции')",
        [],
    )?;

    // Добавляем активы в портфель
    conn.execute(
        "INSERT INTO holdings (portfolio_id, symbol, quantity, avg_price) VALUES (1, 'BTC', 0.5, 45000.0)",
        [],
    )?;

    conn.execute(
        "INSERT INTO holdings (portfolio_id, symbol, quantity, avg_price) VALUES (1, 'ETH', 5.0, 3000.0)",
        [],
    )?;

    println!("Портфель и активы добавлены!");

    // Попытка добавить актив в несуществующий портфель вызовет ошибку
    let result = conn.execute(
        "INSERT INTO holdings (portfolio_id, symbol, quantity, avg_price) VALUES (999, 'SOL', 10.0, 100.0)",
        [],
    );

    match result {
        Ok(_) => println!("Актив добавлен"),
        Err(e) => println!("Ошибка FOREIGN KEY: {}", e),
    }

    Ok(())
}
```

## IF NOT EXISTS — безопасное создание

```rust
use rusqlite::{Connection, Result};

fn ensure_tables_exist(conn: &Connection) -> Result<()> {
    // Таблица создастся только если её нет
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            price REAL NOT NULL,
            quantity REAL NOT NULL,
            executed_at TEXT NOT NULL
        )",
        [],
    )?;

    // Можно вызывать многократно без ошибок
    println!("Таблица trades готова к использованию");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("safe_trading.db")?;

    // Первый вызов — создаёт таблицу
    ensure_tables_exist(&conn)?;

    // Второй вызов — просто проверяет, ничего не делает
    ensure_tables_exist(&conn)?;

    // Третий вызов — тоже без ошибок
    ensure_tables_exist(&conn)?;

    println!("Все вызовы успешны!");
    Ok(())
}
```

## DROP TABLE — удаление таблицы

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open(":memory:")?;

    // Создаём таблицу
    conn.execute(
        "CREATE TABLE temp_trades (
            id INTEGER PRIMARY KEY,
            data TEXT
        )",
        [],
    )?;
    println!("Таблица создана");

    // Удаляем таблицу (осторожно! Все данные будут потеряны)
    conn.execute("DROP TABLE temp_trades", [])?;
    println!("Таблица удалена");

    // DROP TABLE IF EXISTS — безопасное удаление
    conn.execute("DROP TABLE IF EXISTS temp_trades", [])?;
    println!("Безопасное удаление выполнено (таблица уже не существует)");

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| CREATE TABLE | Создание новой таблицы с определённой структурой |
| Типы данных | INTEGER, REAL, TEXT, BLOB, NULL |
| PRIMARY KEY | Уникальный идентификатор записи |
| NOT NULL | Обязательное поле |
| CHECK | Проверка условий при вставке |
| DEFAULT | Значение по умолчанию |
| UNIQUE | Уникальные значения в колонке |
| FOREIGN KEY | Связь между таблицами |
| IF NOT EXISTS | Безопасное создание таблицы |

## Домашнее задание

1. **Таблица биржи**: Создай таблицу `exchanges` с полями:
   - `id` — первичный ключ
   - `name` — название биржи (уникальное, обязательное)
   - `api_url` — URL API
   - `is_active` — активна ли биржа (по умолчанию 1)
   - `created_at` — дата добавления

2. **Таблица стратегий**: Создай таблицу `strategies` с полями:
   - `id` — первичный ключ
   - `name` — название стратегии
   - `description` — описание
   - `risk_level` — уровень риска (CHECK: LOW, MEDIUM, HIGH)
   - `max_position_size` — максимальный размер позиции (CHECK: > 0)
   - `is_active` — активна ли стратегия

3. **Связанные таблицы**: Создай две связанные таблицы:
   - `watchlists` — списки наблюдения (id, name, created_at)
   - `watchlist_items` — элементы списка (id, watchlist_id, symbol, added_at)

   Реализуй связь через FOREIGN KEY с каскадным удалением.

4. **Журнал сигналов**: Создай таблицу `signals` для хранения торговых сигналов:
   - `id`, `strategy_id`, `symbol`, `side` (BUY/SELL)
   - `entry_price`, `stop_loss`, `take_profit`
   - `confidence` (CHECK: от 0.0 до 1.0)
   - `created_at`, `expired_at`

   Добавь все необходимые ограничения для обеспечения валидности данных.

## Навигация

[← Предыдущий день](../215-rusqlite-connecting/ru.md) | [Следующий день →](../217-insert-recording-trade/ru.md)
