# День 220: DELETE: удаляем отменённый ордер

## Аналогия из трейдинга

Представь себе ситуацию на бирже: трейдер разместил ордер на покупку 100 акций по цене $150, но рыночные условия изменились. Может быть, вышла неожиданная новость, или цена ушла слишком далеко от желаемой точки входа. Трейдер решает отменить ордер.

Что происходит с отменённым ордером? В некоторых случаях мы хотим полностью удалить его из системы — как будто его никогда не было. Это особенно важно для:

- **Чистки устаревших данных** — удаление старых отменённых ордеров для экономии места
- **Соблюдения регуляторных требований** — удаление данных по истечении срока хранения
- **Защиты конфиденциальности** — удаление данных по запросу клиента

Операция `DELETE` в SQL — это именно такой инструмент. Она позволяет полностью удалить записи из базы данных.

## Что такое DELETE?

`DELETE` — это SQL-команда для удаления строк из таблицы. В отличие от `UPDATE`, который изменяет данные, `DELETE` полностью убирает записи из базы.

### Базовый синтаксис

```sql
DELETE FROM таблица
WHERE условие;
```

**Важно:** Без `WHERE` будут удалены **все** записи из таблицы!

## Простой пример DELETE

```rust
use rusqlite::{Connection, Result};

#[derive(Debug)]
struct Order {
    id: i64,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
}

fn main() -> Result<()> {
    // Создаём базу данных в памяти
    let conn = Connection::open_in_memory()?;

    // Создаём таблицу ордеров
    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            quantity REAL NOT NULL,
            price REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending'
        )",
        [],
    )?;

    // Добавляем тестовые ордера
    conn.execute(
        "INSERT INTO orders (symbol, side, quantity, price, status) VALUES
            ('BTC/USDT', 'buy', 0.5, 42000.0, 'filled'),
            ('BTC/USDT', 'buy', 0.3, 41500.0, 'cancelled'),
            ('ETH/USDT', 'sell', 2.0, 2200.0, 'pending'),
            ('BTC/USDT', 'sell', 0.1, 43000.0, 'cancelled')",
        [],
    )?;

    println!("Ордера до удаления:");
    print_orders(&conn)?;

    // Удаляем отменённые ордера
    let deleted_count = conn.execute(
        "DELETE FROM orders WHERE status = 'cancelled'",
        [],
    )?;

    println!("\nУдалено ордеров: {}", deleted_count);
    println!("\nОрдера после удаления:");
    print_orders(&conn)?;

    Ok(())
}

fn print_orders(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id, symbol, side, quantity, price, status FROM orders")?;
    let orders = stmt.query_map([], |row| {
        Ok(Order {
            id: row.get(0)?,
            symbol: row.get(1)?,
            side: row.get(2)?,
            quantity: row.get(3)?,
            price: row.get(4)?,
            status: row.get(5)?,
        })
    })?;

    for order in orders {
        let order = order?;
        println!(
            "  #{}: {} {} {} @ ${:.2} [{}]",
            order.id, order.side, order.quantity, order.symbol, order.price, order.status
        );
    }

    Ok(())
}
```

## Удаление с условиями

В торговле часто нужны более сложные условия для удаления:

```rust
use rusqlite::{Connection, Result};
use chrono::{Utc, Duration};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Таблица с временными метками
    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            quantity REAL NOT NULL,
            price REAL NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    // Добавляем ордера с разными датами
    let now = Utc::now();
    let old_date = (now - Duration::days(30)).to_rfc3339();
    let recent_date = (now - Duration::days(1)).to_rfc3339();

    conn.execute(
        "INSERT INTO orders (symbol, side, quantity, price, status, created_at) VALUES
            (?1, 'buy', 0.5, 42000.0, 'cancelled', ?2),
            (?1, 'buy', 0.3, 41500.0, 'cancelled', ?3),
            ('ETH/USDT', 'sell', 2.0, 2200.0, 'filled', ?3),
            (?1, 'sell', 0.1, 43000.0, 'pending', ?3)",
        rusqlite::params!["BTC/USDT", old_date, recent_date],
    )?;

    println!("Все ордера:");
    print_all_orders(&conn)?;

    // Удаляем только старые отменённые ордера (старше 7 дней)
    let cutoff_date = (now - Duration::days(7)).to_rfc3339();
    let deleted = conn.execute(
        "DELETE FROM orders
         WHERE status = 'cancelled'
         AND created_at < ?1",
        [&cutoff_date],
    )?;

    println!("\nУдалено старых отменённых ордеров: {}", deleted);
    println!("\nОставшиеся ордера:");
    print_all_orders(&conn)?;

    Ok(())
}

fn print_all_orders(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, symbol, status, created_at FROM orders"
    )?;
    let orders = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    for order in orders {
        let (id, symbol, status, created) = order?;
        println!("  #{}: {} [{}] - {}", id, symbol, status, created);
    }

    Ok(())
}
```

## Безопасное удаление с проверкой

Перед удалением важных данных всегда полезно проверить, что именно будет удалено:

```rust
use rusqlite::{Connection, Result, Transaction};

struct OrderManager {
    conn: Connection,
}

impl OrderManager {
    fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE orders (
                id INTEGER PRIMARY KEY,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL NOT NULL,
                status TEXT NOT NULL
            )",
            [],
        )?;

        Ok(OrderManager { conn })
    }

    fn add_order(&self, symbol: &str, side: &str, quantity: f64, price: f64) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO orders (symbol, side, quantity, price, status)
             VALUES (?1, ?2, ?3, ?4, 'pending')",
            rusqlite::params![symbol, side, quantity, price],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    fn cancel_order(&self, order_id: i64) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE orders SET status = 'cancelled' WHERE id = ?1 AND status = 'pending'",
            [order_id],
        )?;
        Ok(affected > 0)
    }

    /// Предпросмотр удаляемых ордеров
    fn preview_delete_cancelled(&self) -> Result<Vec<(i64, String, f64, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, quantity, price
             FROM orders
             WHERE status = 'cancelled'"
        )?;

        let orders = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;

        orders.collect()
    }

    /// Удаление отменённых ордеров с подтверждением
    fn delete_cancelled_orders(&self, confirm: bool) -> Result<usize> {
        if !confirm {
            println!("Удаление отменено. Используйте confirm=true для подтверждения.");
            return Ok(0);
        }

        let deleted = self.conn.execute(
            "DELETE FROM orders WHERE status = 'cancelled'",
            [],
        )?;

        Ok(deleted)
    }

    fn print_all(&self) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, side, quantity, price, status FROM orders"
        )?;
        let orders = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        for order in orders {
            let (id, symbol, side, qty, price, status) = order?;
            println!("  #{}: {} {} {} @ ${:.2} [{}]", id, side, qty, symbol, price, status);
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let manager = OrderManager::new()?;

    // Создаём несколько ордеров
    let order1 = manager.add_order("BTC/USDT", "buy", 0.5, 42000.0)?;
    let order2 = manager.add_order("BTC/USDT", "buy", 0.3, 41500.0)?;
    let order3 = manager.add_order("ETH/USDT", "sell", 2.0, 2200.0)?;

    // Отменяем некоторые ордера
    manager.cancel_order(order1)?;
    manager.cancel_order(order3)?;

    println!("Текущие ордера:");
    manager.print_all()?;

    // Предпросмотр удаления
    println!("\nОрдера для удаления:");
    let to_delete = manager.preview_delete_cancelled()?;
    for (id, symbol, qty, price) in &to_delete {
        println!("  #{}: {} {} @ ${:.2}", id, symbol, qty, price);
    }

    // Удаляем с подтверждением
    let deleted = manager.delete_cancelled_orders(true)?;
    println!("\nУдалено ордеров: {}", deleted);

    println!("\nОрдера после удаления:");
    manager.print_all()?;

    Ok(())
}
```

## Удаление с использованием транзакций

Для безопасного удаления используйте транзакции:

```rust
use rusqlite::{Connection, Result, Transaction};

fn delete_expired_orders_safely(conn: &mut Connection, days_old: i64) -> Result<usize> {
    let tx = conn.transaction()?;

    // Сначала архивируем удаляемые ордера
    let archived = tx.execute(
        "INSERT INTO orders_archive
         SELECT *, datetime('now') as archived_at
         FROM orders
         WHERE status = 'cancelled'
         AND julianday('now') - julianday(created_at) > ?1",
        [days_old],
    )?;

    // Затем удаляем
    let deleted = tx.execute(
        "DELETE FROM orders
         WHERE status = 'cancelled'
         AND julianday('now') - julianday(created_at) > ?1",
        [days_old],
    )?;

    // Фиксируем транзакцию
    tx.commit()?;

    println!("Заархивировано: {}, Удалено: {}", archived, deleted);
    Ok(deleted)
}

fn main() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;

    // Создаём таблицы
    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT,
            status TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE orders_archive (
            id INTEGER,
            symbol TEXT,
            status TEXT,
            created_at TEXT,
            archived_at TEXT
        )",
        [],
    )?;

    // Добавляем тестовые данные
    conn.execute(
        "INSERT INTO orders (symbol, status, created_at) VALUES
            ('BTC/USDT', 'cancelled', datetime('now', '-10 days')),
            ('ETH/USDT', 'cancelled', datetime('now', '-3 days')),
            ('BTC/USDT', 'filled', datetime('now', '-1 day'))",
        [],
    )?;

    println!("Ордера до очистки:");
    print_orders(&conn)?;

    // Удаляем ордера старше 7 дней с архивацией
    delete_expired_orders_safely(&mut conn, 7)?;

    println!("\nОрдера после очистки:");
    print_orders(&conn)?;

    println!("\nАрхив:");
    print_archive(&conn)?;

    Ok(())
}

fn print_orders(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id, symbol, status, created_at FROM orders")?;
    let orders = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    for order in orders {
        let (id, symbol, status, created) = order?;
        println!("  #{}: {} [{}] - {}", id, symbol, status, created);
    }
    Ok(())
}

fn print_archive(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, symbol, status, created_at, archived_at FROM orders_archive"
    )?;
    let orders = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;

    for order in orders {
        let (id, symbol, status, created, archived) = order?;
        println!("  #{}: {} [{}] created: {} archived: {}", id, symbol, status, created, archived);
    }
    Ok(())
}
```

## Каскадное удаление

Когда удаляется ордер, нужно также удалить связанные данные:

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Включаем поддержку внешних ключей
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Создаём таблицу ордеров
    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            status TEXT NOT NULL
        )",
        [],
    )?;

    // Создаём таблицу исполнений (fills) с каскадным удалением
    conn.execute(
        "CREATE TABLE fills (
            id INTEGER PRIMARY KEY,
            order_id INTEGER NOT NULL,
            quantity REAL NOT NULL,
            price REAL NOT NULL,
            FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Добавляем ордер и его исполнения
    conn.execute(
        "INSERT INTO orders (id, symbol, status) VALUES (1, 'BTC/USDT', 'cancelled')",
        [],
    )?;

    conn.execute(
        "INSERT INTO fills (order_id, quantity, price) VALUES
            (1, 0.1, 42000.0),
            (1, 0.2, 42100.0),
            (1, 0.2, 42050.0)",
        [],
    )?;

    println!("До удаления:");
    println!("Ордера: {:?}", count_orders(&conn)?);
    println!("Исполнения: {:?}", count_fills(&conn)?);

    // Удаляем ордер - исполнения удалятся автоматически
    conn.execute("DELETE FROM orders WHERE id = 1", [])?;

    println!("\nПосле удаления ордера:");
    println!("Ордера: {:?}", count_orders(&conn)?);
    println!("Исполнения: {:?}", count_fills(&conn)?);

    Ok(())
}

fn count_orders(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM orders", [], |row| row.get(0))
}

fn count_fills(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM fills", [], |row| row.get(0))
}
```

## Практический пример: Система очистки ордеров

```rust
use rusqlite::{Connection, Result};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy)]
enum CleanupPolicy {
    DeleteImmediately,
    ArchiveThenDelete,
    SoftDelete,
}

struct OrderCleanupService {
    conn: Connection,
    policy: CleanupPolicy,
}

impl OrderCleanupService {
    fn new(policy: CleanupPolicy) -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE orders (
                id INTEGER PRIMARY KEY,
                symbol TEXT NOT NULL,
                status TEXT NOT NULL,
                deleted_at TEXT,
                created_at TEXT DEFAULT (datetime('now'))
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE orders_archive (
                id INTEGER,
                symbol TEXT,
                status TEXT,
                created_at TEXT,
                archived_at TEXT
            )",
            [],
        )?;

        Ok(OrderCleanupService { conn, policy })
    }

    fn cleanup_cancelled_orders(&self) -> Result<usize> {
        match self.policy {
            CleanupPolicy::DeleteImmediately => {
                self.hard_delete()
            }
            CleanupPolicy::ArchiveThenDelete => {
                self.archive_and_delete()
            }
            CleanupPolicy::SoftDelete => {
                self.soft_delete()
            }
        }
    }

    fn hard_delete(&self) -> Result<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM orders WHERE status = 'cancelled'",
            [],
        )?;
        println!("Жёсткое удаление: {} записей", deleted);
        Ok(deleted)
    }

    fn archive_and_delete(&self) -> Result<usize> {
        // Сначала архивируем
        self.conn.execute(
            "INSERT INTO orders_archive (id, symbol, status, created_at, archived_at)
             SELECT id, symbol, status, created_at, datetime('now')
             FROM orders WHERE status = 'cancelled'",
            [],
        )?;

        // Затем удаляем
        let deleted = self.conn.execute(
            "DELETE FROM orders WHERE status = 'cancelled'",
            [],
        )?;

        println!("Архивировано и удалено: {} записей", deleted);
        Ok(deleted)
    }

    fn soft_delete(&self) -> Result<usize> {
        let updated = self.conn.execute(
            "UPDATE orders
             SET deleted_at = datetime('now')
             WHERE status = 'cancelled' AND deleted_at IS NULL",
            [],
        )?;
        println!("Мягкое удаление: {} записей", updated);
        Ok(updated)
    }

    fn add_test_orders(&self) -> Result<()> {
        self.conn.execute(
            "INSERT INTO orders (symbol, status) VALUES
                ('BTC/USDT', 'filled'),
                ('BTC/USDT', 'cancelled'),
                ('ETH/USDT', 'cancelled'),
                ('SOL/USDT', 'pending')",
            [],
        )?;
        Ok(())
    }

    fn print_orders(&self) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, status, deleted_at FROM orders"
        )?;
        let orders = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;

        for order in orders {
            let (id, symbol, status, deleted) = order?;
            let deleted_str = deleted.map_or("active".to_string(), |d| format!("deleted: {}", d));
            println!("  #{}: {} [{}] {}", id, symbol, status, deleted_str);
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    println!("=== Жёсткое удаление ===");
    let service1 = OrderCleanupService::new(CleanupPolicy::DeleteImmediately)?;
    service1.add_test_orders()?;
    println!("До очистки:");
    service1.print_orders()?;
    service1.cleanup_cancelled_orders()?;
    println!("После очистки:");
    service1.print_orders()?;

    println!("\n=== Мягкое удаление ===");
    let service2 = OrderCleanupService::new(CleanupPolicy::SoftDelete)?;
    service2.add_test_orders()?;
    println!("До очистки:");
    service2.print_orders()?;
    service2.cleanup_cancelled_orders()?;
    println!("После очистки:");
    service2.print_orders()?;

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| DELETE | SQL-команда для удаления записей из таблицы |
| WHERE | Условие для выбора удаляемых записей |
| Транзакции | Обеспечивают атомарность операций удаления |
| Каскадное удаление | Автоматическое удаление связанных записей |
| Мягкое удаление | Пометка записи как удалённой без физического удаления |
| Архивация | Сохранение данных перед удалением |

## Практические задания

1. **Базовое удаление**: Создай таблицу `trades` (сделок) и реализуй функцию для удаления сделок по определённому символу.

2. **Удаление с архивацией**: Расширь задание 1, добавив архивацию удаляемых сделок в отдельную таблицу перед удалением.

3. **Очистка по времени**: Реализуй функцию, которая удаляет все отменённые ордера старше N дней, но сохраняет хотя бы последние 100 записей для аудита.

4. **Безопасное удаление**: Создай систему удаления с двухэтапным подтверждением: сначала пометка для удаления, затем окончательное удаление через 24 часа.

## Домашнее задание

1. **Система очистки портфеля**: Реализуй менеджер портфеля с функциями:
   - `remove_position(symbol)` — удаление позиции (только с нулевым количеством)
   - `cleanup_empty_positions()` — удаление всех пустых позиций
   - `archive_closed_positions(days)` — архивация и удаление позиций, закрытых более N дней назад

2. **Управление историей цен**: Создай систему хранения исторических цен с:
   - Автоматическим удалением данных старше года
   - Сохранением агрегированных данных (OHLC) перед удалением тиков
   - Возможностью восстановления из архива

3. **Аудит удалений**: Добавь в систему ордеров:
   - Таблицу `deletion_log` для записи всех операций удаления
   - Триггер, который записывает в лог при каждом DELETE
   - Функцию для просмотра истории удалений

4. **Политики очистки**: Реализуй конфигурируемую систему очистки данных:
   - Разные политики для разных типов данных (ордера, сделки, логи)
   - Настраиваемые сроки хранения
   - Отчёт о выполненных очистках

## Навигация

[← Предыдущий день](../219-update-order-status/ru.md) | [Следующий день →](../221-prepared-statements-safe-queries/ru.md)
