# Day 220: DELETE: Removing Cancelled Order

## Trading Analogy

Imagine a situation on an exchange: a trader placed an order to buy 100 shares at $150, but market conditions changed. Perhaps unexpected news came out, or the price moved too far from the desired entry point. The trader decides to cancel the order.

What happens to the cancelled order? In some cases, we want to completely remove it from the system — as if it never existed. This is especially important for:

- **Cleaning up stale data** — removing old cancelled orders to save storage space
- **Regulatory compliance** — deleting data after the retention period expires
- **Privacy protection** — removing data upon client request

The `DELETE` operation in SQL is exactly such a tool. It allows you to completely remove records from the database.

## What is DELETE?

`DELETE` is an SQL command for removing rows from a table. Unlike `UPDATE`, which modifies data, `DELETE` completely removes records from the database.

### Basic Syntax

```sql
DELETE FROM table
WHERE condition;
```

**Important:** Without `WHERE`, **all** records from the table will be deleted!

## Simple DELETE Example

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
    // Create an in-memory database
    let conn = Connection::open_in_memory()?;

    // Create the orders table
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

    // Add test orders
    conn.execute(
        "INSERT INTO orders (symbol, side, quantity, price, status) VALUES
            ('BTC/USDT', 'buy', 0.5, 42000.0, 'filled'),
            ('BTC/USDT', 'buy', 0.3, 41500.0, 'cancelled'),
            ('ETH/USDT', 'sell', 2.0, 2200.0, 'pending'),
            ('BTC/USDT', 'sell', 0.1, 43000.0, 'cancelled')",
        [],
    )?;

    println!("Orders before deletion:");
    print_orders(&conn)?;

    // Delete cancelled orders
    let deleted_count = conn.execute(
        "DELETE FROM orders WHERE status = 'cancelled'",
        [],
    )?;

    println!("\nDeleted orders: {}", deleted_count);
    println!("\nOrders after deletion:");
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

## Deletion with Conditions

In trading, more complex deletion conditions are often needed:

```rust
use rusqlite::{Connection, Result};
use chrono::{Utc, Duration};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Table with timestamps
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

    // Add orders with different dates
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

    println!("All orders:");
    print_all_orders(&conn)?;

    // Delete only old cancelled orders (older than 7 days)
    let cutoff_date = (now - Duration::days(7)).to_rfc3339();
    let deleted = conn.execute(
        "DELETE FROM orders
         WHERE status = 'cancelled'
         AND created_at < ?1",
        [&cutoff_date],
    )?;

    println!("\nDeleted old cancelled orders: {}", deleted);
    println!("\nRemaining orders:");
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

## Safe Deletion with Verification

Before deleting important data, it's always useful to check what exactly will be deleted:

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

    /// Preview orders to be deleted
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

    /// Delete cancelled orders with confirmation
    fn delete_cancelled_orders(&self, confirm: bool) -> Result<usize> {
        if !confirm {
            println!("Deletion cancelled. Use confirm=true to confirm.");
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

    // Create some orders
    let order1 = manager.add_order("BTC/USDT", "buy", 0.5, 42000.0)?;
    let order2 = manager.add_order("BTC/USDT", "buy", 0.3, 41500.0)?;
    let order3 = manager.add_order("ETH/USDT", "sell", 2.0, 2200.0)?;

    // Cancel some orders
    manager.cancel_order(order1)?;
    manager.cancel_order(order3)?;

    println!("Current orders:");
    manager.print_all()?;

    // Preview deletion
    println!("\nOrders to be deleted:");
    let to_delete = manager.preview_delete_cancelled()?;
    for (id, symbol, qty, price) in &to_delete {
        println!("  #{}: {} {} @ ${:.2}", id, symbol, qty, price);
    }

    // Delete with confirmation
    let deleted = manager.delete_cancelled_orders(true)?;
    println!("\nDeleted orders: {}", deleted);

    println!("\nOrders after deletion:");
    manager.print_all()?;

    Ok(())
}
```

## Deletion Using Transactions

Use transactions for safe deletion:

```rust
use rusqlite::{Connection, Result, Transaction};

fn delete_expired_orders_safely(conn: &mut Connection, days_old: i64) -> Result<usize> {
    let tx = conn.transaction()?;

    // First archive the orders to be deleted
    let archived = tx.execute(
        "INSERT INTO orders_archive
         SELECT *, datetime('now') as archived_at
         FROM orders
         WHERE status = 'cancelled'
         AND julianday('now') - julianday(created_at) > ?1",
        [days_old],
    )?;

    // Then delete
    let deleted = tx.execute(
        "DELETE FROM orders
         WHERE status = 'cancelled'
         AND julianday('now') - julianday(created_at) > ?1",
        [days_old],
    )?;

    // Commit the transaction
    tx.commit()?;

    println!("Archived: {}, Deleted: {}", archived, deleted);
    Ok(deleted)
}

fn main() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;

    // Create tables
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

    // Add test data
    conn.execute(
        "INSERT INTO orders (symbol, status, created_at) VALUES
            ('BTC/USDT', 'cancelled', datetime('now', '-10 days')),
            ('ETH/USDT', 'cancelled', datetime('now', '-3 days')),
            ('BTC/USDT', 'filled', datetime('now', '-1 day'))",
        [],
    )?;

    println!("Orders before cleanup:");
    print_orders(&conn)?;

    // Delete orders older than 7 days with archiving
    delete_expired_orders_safely(&mut conn, 7)?;

    println!("\nOrders after cleanup:");
    print_orders(&conn)?;

    println!("\nArchive:");
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

## Cascade Deletion

When an order is deleted, related data should also be removed:

```rust
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    // Enable foreign key support
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Create the orders table
    conn.execute(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            status TEXT NOT NULL
        )",
        [],
    )?;

    // Create the fills table with cascade deletion
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

    // Add an order and its fills
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

    println!("Before deletion:");
    println!("Orders: {:?}", count_orders(&conn)?);
    println!("Fills: {:?}", count_fills(&conn)?);

    // Delete the order - fills will be deleted automatically
    conn.execute("DELETE FROM orders WHERE id = 1", [])?;

    println!("\nAfter order deletion:");
    println!("Orders: {:?}", count_orders(&conn)?);
    println!("Fills: {:?}", count_fills(&conn)?);

    Ok(())
}

fn count_orders(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM orders", [], |row| row.get(0))
}

fn count_fills(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM fills", [], |row| row.get(0))
}
```

## Practical Example: Order Cleanup System

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
        println!("Hard delete: {} records", deleted);
        Ok(deleted)
    }

    fn archive_and_delete(&self) -> Result<usize> {
        // First archive
        self.conn.execute(
            "INSERT INTO orders_archive (id, symbol, status, created_at, archived_at)
             SELECT id, symbol, status, created_at, datetime('now')
             FROM orders WHERE status = 'cancelled'",
            [],
        )?;

        // Then delete
        let deleted = self.conn.execute(
            "DELETE FROM orders WHERE status = 'cancelled'",
            [],
        )?;

        println!("Archived and deleted: {} records", deleted);
        Ok(deleted)
    }

    fn soft_delete(&self) -> Result<usize> {
        let updated = self.conn.execute(
            "UPDATE orders
             SET deleted_at = datetime('now')
             WHERE status = 'cancelled' AND deleted_at IS NULL",
            [],
        )?;
        println!("Soft delete: {} records", updated);
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
    println!("=== Hard Delete ===");
    let service1 = OrderCleanupService::new(CleanupPolicy::DeleteImmediately)?;
    service1.add_test_orders()?;
    println!("Before cleanup:");
    service1.print_orders()?;
    service1.cleanup_cancelled_orders()?;
    println!("After cleanup:");
    service1.print_orders()?;

    println!("\n=== Soft Delete ===");
    let service2 = OrderCleanupService::new(CleanupPolicy::SoftDelete)?;
    service2.add_test_orders()?;
    println!("Before cleanup:");
    service2.print_orders()?;
    service2.cleanup_cancelled_orders()?;
    println!("After cleanup:");
    service2.print_orders()?;

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| DELETE | SQL command for removing records from a table |
| WHERE | Condition for selecting records to delete |
| Transactions | Ensure atomicity of deletion operations |
| Cascade deletion | Automatic deletion of related records |
| Soft delete | Marking a record as deleted without physical removal |
| Archiving | Saving data before deletion |

## Practical Exercises

1. **Basic deletion**: Create a `trades` table and implement a function to delete trades by a specific symbol.

2. **Deletion with archiving**: Extend exercise 1 by adding archiving of deleted trades to a separate table before deletion.

3. **Time-based cleanup**: Implement a function that deletes all cancelled orders older than N days, but keeps at least the last 100 records for auditing.

4. **Safe deletion**: Create a deletion system with two-stage confirmation: first marking for deletion, then final deletion after 24 hours.

## Homework

1. **Portfolio cleanup system**: Implement a portfolio manager with functions:
   - `remove_position(symbol)` — remove a position (only with zero quantity)
   - `cleanup_empty_positions()` — remove all empty positions
   - `archive_closed_positions(days)` — archive and remove positions closed more than N days ago

2. **Price history management**: Create a historical price storage system with:
   - Automatic deletion of data older than a year
   - Saving aggregated data (OHLC) before deleting ticks
   - Ability to restore from archive

3. **Deletion audit**: Add to the order system:
   - A `deletion_log` table to record all deletion operations
   - A trigger that logs every DELETE
   - A function to view deletion history

4. **Cleanup policies**: Implement a configurable data cleanup system:
   - Different policies for different data types (orders, trades, logs)
   - Configurable retention periods
   - Report on completed cleanups

## Navigation

[← Previous day](../219-update-order-status/en.md) | [Next day →](../221-prepared-statements-safe-queries/en.md)
