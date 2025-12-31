# День 214: SQLite: встроенная база данных для бота

## Аналогия из трейдинга

Представь, что ты ведёшь торговый журнал. Каждый день ты записываешь в блокнот: какие сделки совершил, по каким ценам купил и продал, какой получился результат. Но блокнот легко потерять, в нём сложно быстро найти нужную запись, и невозможно сделать выборку "все прибыльные сделки за последний месяц".

**SQLite** — это как умный электронный журнал, который:
- Хранит все данные в одном файле (как блокнот)
- Позволяет мгновенно находить нужные записи (индексы)
- Умеет фильтровать и группировать данные (SQL-запросы)
- Не требует отдельного сервера (встроенная база)

В реальном трейдинге SQLite используется для:
- Хранения истории сделок локально
- Кэширования данных с биржи
- Сохранения настроек и состояния бота
- Логирования событий для анализа

## Что такое SQLite?

SQLite — это **встраиваемая** реляционная база данных. Это означает:

1. **Без сервера** — база данных работает как библиотека внутри твоего приложения
2. **Один файл** — вся база хранится в одном файле `.db`
3. **Нулевая конфигурация** — не нужно ничего устанавливать и настраивать
4. **ACID-совместимость** — надёжность как у "взрослых" баз данных
5. **Кроссплатформенность** — файл базы можно переносить между системами

## Почему SQLite идеален для торгового бота?

```
┌─────────────────────────────────────────────────────────┐
│                     Торговый бот                        │
│                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │   Стратегия │    │   Риск-     │    │   Отчёты    │ │
│  │   торговли  │    │ менеджмент  │    │  и анализ   │ │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘ │
│         │                  │                  │         │
│         └──────────────────┼──────────────────┘         │
│                            │                            │
│                   ┌────────▼────────┐                   │
│                   │     SQLite      │                   │
│                   │   trades.db     │                   │
│                   └─────────────────┘                   │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Сравнение SQLite с другими решениями

| Характеристика | Файл/JSON | SQLite | PostgreSQL |
|---------------|-----------|--------|------------|
| Настройка | Нет | Нет | Сложная |
| Сервер | Нет | Нет | Да |
| Поиск | Медленный | Быстрый | Очень быстрый |
| Транзакции | Нет | Да | Да |
| Параллельная запись | Нет | Ограничена | Да |
| Объём данных | Малый | Средний | Любой |
| Для бота | ❌ | ✅ | Для продакшена |

## Установка rusqlite

Добавь в `Cargo.toml`:

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
```

Флаг `bundled` означает, что SQLite будет скомпилирован вместе с программой — не нужно устанавливать SQLite в систему.

## Первое подключение к базе

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // Создаём или открываем базу данных
    let conn = Connection::open("trading.db")?;

    println!("База данных успешно открыта!");
    println!("SQLite версия: {}", rusqlite::version());

    Ok(())
}
```

## База данных в памяти

Для тестирования можно использовать базу в памяти:

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    // База данных только в RAM — исчезнет после завершения программы
    let conn = Connection::open_in_memory()?;

    println!("In-memory база создана!");

    Ok(())
}
```

## Создание таблицы для сделок

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Создаём таблицу для хранения сделок
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol      TEXT NOT NULL,
            side        TEXT NOT NULL CHECK(side IN ('buy', 'sell')),
            price       REAL NOT NULL,
            quantity    REAL NOT NULL,
            timestamp   INTEGER NOT NULL,
            pnl         REAL,
            status      TEXT DEFAULT 'open'
        )",
        (), // Нет параметров
    )?;

    println!("Таблица trades создана!");

    Ok(())
}
```

## Вставка сделки

```rust
use rusqlite::{Connection, Result, params};
use std::time::{SystemTime, UNIX_EPOCH};

struct Trade {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn insert_trade(conn: &Connection, trade: &Trade) -> Result<i64> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO trades (symbol, side, price, quantity, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![trade.symbol, trade.side, trade.price, trade.quantity, timestamp],
    )?;

    // Возвращаем ID вставленной записи
    Ok(conn.last_insert_rowid())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Создаём таблицу (если не существует)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol      TEXT NOT NULL,
            side        TEXT NOT NULL,
            price       REAL NOT NULL,
            quantity    REAL NOT NULL,
            timestamp   INTEGER NOT NULL,
            pnl         REAL,
            status      TEXT DEFAULT 'open'
        )",
        (),
    )?;

    // Вставляем сделку
    let trade = Trade {
        symbol: "BTC/USDT".to_string(),
        side: "buy".to_string(),
        price: 42000.0,
        quantity: 0.5,
    };

    let trade_id = insert_trade(&conn, &trade)?;
    println!("Сделка записана с ID: {}", trade_id);

    Ok(())
}
```

## Чтение сделок

```rust
use rusqlite::{Connection, Result, params};

#[derive(Debug)]
struct Trade {
    id: i64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: i64,
    pnl: Option<f64>,
    status: String,
}

fn get_all_trades(conn: &Connection) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT id, symbol, side, price, quantity, timestamp, pnl, status
         FROM trades
         ORDER BY timestamp DESC"
    )?;

    let trade_iter = stmt.query_map([], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            timestamp: row.get(5)?,
            pnl: row.get(6)?,
            status: row.get(7)?,
        })
    })?;

    let mut trades = Vec::new();
    for trade in trade_iter {
        trades.push(trade?);
    }

    Ok(trades)
}

fn get_trades_by_symbol(conn: &Connection, symbol: &str) -> Result<Vec<Trade>> {
    let mut stmt = conn.prepare(
        "SELECT id, symbol, side, price, quantity, timestamp, pnl, status
         FROM trades
         WHERE symbol = ?1
         ORDER BY timestamp DESC"
    )?;

    let trade_iter = stmt.query_map(params![symbol], |row| {
        Ok(Trade {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            price: row.get(3)?,
            quantity: row.get(4)?,
            timestamp: row.get(5)?,
            pnl: row.get(6)?,
            status: row.get(7)?,
        })
    })?;

    let mut trades = Vec::new();
    for trade in trade_iter {
        trades.push(trade?);
    }

    Ok(trades)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Получаем все сделки
    let all_trades = get_all_trades(&conn)?;
    println!("Всего сделок: {}", all_trades.len());

    for trade in &all_trades {
        println!("{:?}", trade);
    }

    // Получаем сделки по конкретному символу
    let btc_trades = get_trades_by_symbol(&conn, "BTC/USDT")?;
    println!("\nСделок по BTC/USDT: {}", btc_trades.len());

    Ok(())
}
```

## Обновление сделки

```rust
use rusqlite::{Connection, Result, params};

fn close_trade(conn: &Connection, trade_id: i64, pnl: f64) -> Result<usize> {
    let rows_updated = conn.execute(
        "UPDATE trades
         SET status = 'closed', pnl = ?1
         WHERE id = ?2",
        params![pnl, trade_id],
    )?;

    Ok(rows_updated)
}

fn update_trade_price(conn: &Connection, trade_id: i64, new_price: f64) -> Result<usize> {
    let rows_updated = conn.execute(
        "UPDATE trades
         SET price = ?1
         WHERE id = ?2 AND status = 'open'",
        params![new_price, trade_id],
    )?;

    Ok(rows_updated)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Закрываем сделку с прибылью
    let updated = close_trade(&conn, 1, 150.50)?;
    println!("Обновлено записей: {}", updated);

    Ok(())
}
```

## Удаление сделки

```rust
use rusqlite::{Connection, Result, params};

fn delete_trade(conn: &Connection, trade_id: i64) -> Result<usize> {
    let rows_deleted = conn.execute(
        "DELETE FROM trades WHERE id = ?1",
        params![trade_id],
    )?;

    Ok(rows_deleted)
}

fn delete_old_trades(conn: &Connection, before_timestamp: i64) -> Result<usize> {
    let rows_deleted = conn.execute(
        "DELETE FROM trades WHERE timestamp < ?1 AND status = 'closed'",
        params![before_timestamp],
    )?;

    Ok(rows_deleted)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Удаляем конкретную сделку
    let deleted = delete_trade(&conn, 5)?;
    println!("Удалено записей: {}", deleted);

    Ok(())
}
```

## Практический пример: Трейд-трекер

```rust
use rusqlite::{Connection, Result, params};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct Trade {
    id: Option<i64>,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
    timestamp: i64,
    pnl: Option<f64>,
    status: String,
}

struct TradeTracker {
    conn: Connection,
}

impl TradeTracker {
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Инициализация схемы
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol      TEXT NOT NULL,
                side        TEXT NOT NULL,
                price       REAL NOT NULL,
                quantity    REAL NOT NULL,
                timestamp   INTEGER NOT NULL,
                pnl         REAL,
                status      TEXT DEFAULT 'open'
            )",
            (),
        )?;

        // Создаём индекс для быстрого поиска
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_trades_symbol
             ON trades(symbol)",
            (),
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_trades_status
             ON trades(status)",
            (),
        )?;

        Ok(TradeTracker { conn })
    }

    fn record_trade(&self, symbol: &str, side: &str, price: f64, quantity: f64) -> Result<i64> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT INTO trades (symbol, side, price, quantity, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![symbol, side, price, quantity, timestamp],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    fn close_trade(&self, trade_id: i64, exit_price: f64) -> Result<f64> {
        // Получаем информацию о сделке
        let mut stmt = self.conn.prepare(
            "SELECT side, price, quantity FROM trades WHERE id = ?1 AND status = 'open'"
        )?;

        let trade_info: (String, f64, f64) = stmt.query_row(params![trade_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        let (side, entry_price, quantity) = trade_info;

        // Рассчитываем PnL
        let pnl = if side == "buy" {
            (exit_price - entry_price) * quantity
        } else {
            (entry_price - exit_price) * quantity
        };

        // Обновляем сделку
        self.conn.execute(
            "UPDATE trades SET status = 'closed', pnl = ?1 WHERE id = ?2",
            params![pnl, trade_id],
        )?;

        Ok(pnl)
    }

    fn get_open_trades(&self) -> Result<Vec<Trade>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, side, price, quantity, timestamp, pnl, status
             FROM trades
             WHERE status = 'open'
             ORDER BY timestamp DESC"
        )?;

        let trades = stmt.query_map([], |row| {
            Ok(Trade {
                id: Some(row.get(0)?),
                symbol: row.get(1)?,
                side: row.get(2)?,
                price: row.get(3)?,
                quantity: row.get(4)?,
                timestamp: row.get(5)?,
                pnl: row.get(6)?,
                status: row.get(7)?,
            })
        })?;

        trades.collect()
    }

    fn get_total_pnl(&self) -> Result<f64> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(SUM(pnl), 0.0) FROM trades WHERE status = 'closed'"
        )?;

        let total: f64 = stmt.query_row([], |row| row.get(0))?;
        Ok(total)
    }

    fn get_statistics(&self) -> Result<TradingStats> {
        let mut stmt = self.conn.prepare(
            "SELECT
                COUNT(*) as total,
                COALESCE(SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END), 0) as winning,
                COALESCE(SUM(CASE WHEN pnl < 0 THEN 1 ELSE 0 END), 0) as losing,
                COALESCE(SUM(pnl), 0.0) as total_pnl,
                COALESCE(MAX(pnl), 0.0) as best_trade,
                COALESCE(MIN(pnl), 0.0) as worst_trade
             FROM trades
             WHERE status = 'closed'"
        )?;

        let stats = stmt.query_row([], |row| {
            Ok(TradingStats {
                total_trades: row.get(0)?,
                winning_trades: row.get(1)?,
                losing_trades: row.get(2)?,
                total_pnl: row.get(3)?,
                best_trade: row.get(4)?,
                worst_trade: row.get(5)?,
            })
        })?;

        Ok(stats)
    }
}

#[derive(Debug)]
struct TradingStats {
    total_trades: i64,
    winning_trades: i64,
    losing_trades: i64,
    total_pnl: f64,
    best_trade: f64,
    worst_trade: f64,
}

impl TradingStats {
    fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        }
    }
}

fn main() -> Result<()> {
    let tracker = TradeTracker::new("trading_bot.db")?;

    // Записываем несколько сделок
    println!("=== Записываем сделки ===");

    let trade1 = tracker.record_trade("BTC/USDT", "buy", 42000.0, 0.5)?;
    println!("Открыта сделка #{}: BTC/USDT buy @ 42000", trade1);

    let trade2 = tracker.record_trade("ETH/USDT", "buy", 2800.0, 2.0)?;
    println!("Открыта сделка #{}: ETH/USDT buy @ 2800", trade2);

    let trade3 = tracker.record_trade("BTC/USDT", "sell", 43000.0, 0.3)?;
    println!("Открыта сделка #{}: BTC/USDT sell @ 43000", trade3);

    // Показываем открытые позиции
    println!("\n=== Открытые позиции ===");
    for trade in tracker.get_open_trades()? {
        println!("  #{}: {} {} {} @ {} (qty: {})",
            trade.id.unwrap(),
            trade.symbol,
            trade.side,
            trade.quantity,
            trade.price,
            trade.quantity
        );
    }

    // Закрываем сделки
    println!("\n=== Закрываем сделки ===");

    let pnl1 = tracker.close_trade(trade1, 43500.0)?;
    println!("Сделка #{} закрыта, PnL: ${:.2}", trade1, pnl1);

    let pnl2 = tracker.close_trade(trade2, 2750.0)?;
    println!("Сделка #{} закрыта, PnL: ${:.2}", trade2, pnl2);

    let pnl3 = tracker.close_trade(trade3, 42500.0)?;
    println!("Сделка #{} закрыта, PnL: ${:.2}", trade3, pnl3);

    // Показываем статистику
    println!("\n=== Статистика торговли ===");
    let stats = tracker.get_statistics()?;
    println!("Всего сделок: {}", stats.total_trades);
    println!("Прибыльных: {}", stats.winning_trades);
    println!("Убыточных: {}", stats.losing_trades);
    println!("Win Rate: {:.1}%", stats.win_rate());
    println!("Общий PnL: ${:.2}", stats.total_pnl);
    println!("Лучшая сделка: ${:.2}", stats.best_trade);
    println!("Худшая сделка: ${:.2}", stats.worst_trade);

    Ok(())
}
```

## Транзакции для атомарных операций

```rust
use rusqlite::{Connection, Result, params, Transaction};

fn transfer_funds(conn: &mut Connection, from_asset: &str, to_asset: &str, amount: f64) -> Result<()> {
    // Начинаем транзакцию
    let tx = conn.transaction()?;

    // Списываем с одного актива
    tx.execute(
        "UPDATE portfolio SET balance = balance - ?1 WHERE asset = ?2",
        params![amount, from_asset],
    )?;

    // Зачисляем на другой
    tx.execute(
        "UPDATE portfolio SET balance = balance + ?1 WHERE asset = ?2",
        params![amount, to_asset],
    )?;

    // Коммитим только если оба запроса успешны
    tx.commit()?;

    println!("Перевод {} {} -> {} выполнен", amount, from_asset, to_asset);
    Ok(())
}

fn execute_order_atomic(conn: &mut Connection, symbol: &str, side: &str, price: f64, quantity: f64) -> Result<i64> {
    let tx = conn.transaction()?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Записываем сделку
    tx.execute(
        "INSERT INTO trades (symbol, side, price, quantity, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![symbol, side, price, quantity, timestamp],
    )?;

    let trade_id = tx.last_insert_rowid();

    // Обновляем позицию
    let qty_change = if side == "buy" { quantity } else { -quantity };

    tx.execute(
        "INSERT INTO positions (symbol, quantity) VALUES (?1, ?2)
         ON CONFLICT(symbol) DO UPDATE SET quantity = quantity + ?2",
        params![symbol, qty_change],
    )?;

    // Логируем событие
    tx.execute(
        "INSERT INTO trade_log (trade_id, event, timestamp) VALUES (?1, 'executed', ?2)",
        params![trade_id, timestamp],
    )?;

    tx.commit()?;

    Ok(trade_id)
}

fn main() -> Result<()> {
    let mut conn = Connection::open("trading.db")?;

    // Создаём таблицы
    conn.execute(
        "CREATE TABLE IF NOT EXISTS portfolio (
            asset TEXT PRIMARY KEY,
            balance REAL NOT NULL DEFAULT 0
        )",
        (),
    )?;

    // Инициализируем баланс
    conn.execute("INSERT OR REPLACE INTO portfolio (asset, balance) VALUES ('USDT', 10000)", ())?;
    conn.execute("INSERT OR REPLACE INTO portfolio (asset, balance) VALUES ('BTC', 0)", ())?;

    // Выполняем перевод
    transfer_funds(&mut conn, "USDT", "BTC", 1000.0)?;

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| SQLite | Встраиваемая база данных в одном файле |
| rusqlite | Rust библиотека для работы с SQLite |
| Connection | Соединение с базой данных |
| execute() | Выполнение SQL без результата |
| prepare() + query_map() | Выполнение SELECT запросов |
| params![] | Безопасная передача параметров |
| Transaction | Атомарные операции |

## Домашнее задание

1. **Журнал ордеров**: Создай таблицу `orders` с полями: id, symbol, side, type (market/limit), price, quantity, status, created_at, filled_at. Реализуй CRUD операции.

2. **Портфель с историей**: Создай систему из двух таблиц:
   - `portfolio` — текущие балансы активов
   - `portfolio_history` — история изменений (триггер на INSERT/UPDATE)

   Реализуй функцию получения баланса на определённую дату.

3. **Анализ сделок**: Напиши SQL-запросы для:
   - Топ-5 самых прибыльных символов
   - Средний PnL по дням недели
   - Сделки с аномально большим объёмом (> 2 стандартных отклонения)

4. **Кеш стакана**: Создай структуру для хранения стакана заявок в SQLite:
   - Таблица `order_book` с полями: symbol, side, price, quantity, updated_at
   - Функции для обновления и получения топ-N уровней
   - Очистка устаревших записей (старше 1 минуты)

## Навигация

[← Предыдущий день](../213-why-database-persistence/ru.md) | [Следующий день →](../215-rusqlite-connecting/ru.md)
