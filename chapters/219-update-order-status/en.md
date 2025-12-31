# Day 219: UPDATE: Updating Order Status

## Trading Analogy

Imagine you place an order to buy Bitcoin at $42,000. Initially, the order has status "Pending" (waiting for execution). When the market price reaches your order level, the exchange executes the order and changes its status to "Filled". If you decide to cancel the order before execution, the status changes to "Cancelled".

In databases, the `UPDATE` command is used to modify existing records. It's like editing a row in an Excel spreadsheet — you don't create a new row, you change values in an existing one.

In algorithmic trading, UPDATE is used constantly:
- Changing order status (pending → filled → closed)
- Updating average position price after partial fills
- Adjusting stop-loss and take-profit levels
- Updating balance after a trade

## UPDATE Syntax

```sql
UPDATE table_name
SET column1 = value1, column2 = value2, ...
WHERE condition;
```

**Important:** Without `WHERE`, ALL rows in the table will be updated! This is a common mistake that can lead to disaster.

## Preparation: Creating the Orders Table

```rust
use rusqlite::{Connection, Result};

fn create_orders_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL,
            quantity REAL NOT NULL,
            price REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            filled_quantity REAL DEFAULT 0,
            filled_price REAL,
            created_at TEXT NOT NULL,
            updated_at TEXT
        )",
        [],
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;
    create_orders_table(&conn)?;

    println!("Orders table created!");
    Ok(())
}
```

## Basic UPDATE: Changing Order Status

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

fn update_order_status(conn: &Connection, order_id: i64, new_status: &str) -> Result<usize> {
    let updated_at = Utc::now().to_rfc3339();

    let rows_affected = conn.execute(
        "UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_status, updated_at, order_id],
    )?;

    println!("Rows updated: {}", rows_affected);
    Ok(rows_affected)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // First create a test order
    conn.execute(
        "INSERT INTO orders (symbol, side, quantity, price, status, created_at)
         VALUES ('BTC/USDT', 'buy', 0.5, 42000.0, 'pending', datetime('now'))",
        [],
    )?;

    let order_id = conn.last_insert_rowid();
    println!("Created order with ID: {}", order_id);

    // Update status to "filled"
    update_order_status(&conn, order_id, "filled")?;

    Ok(())
}
```

## Updating Multiple Fields at Once

When filling an order, you need to update several fields simultaneously:

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

#[derive(Debug)]
struct OrderFill {
    order_id: i64,
    filled_quantity: f64,
    filled_price: f64,
}

fn fill_order(conn: &Connection, fill: &OrderFill) -> Result<usize> {
    let updated_at = Utc::now().to_rfc3339();

    let rows_affected = conn.execute(
        "UPDATE orders
         SET status = 'filled',
             filled_quantity = ?1,
             filled_price = ?2,
             updated_at = ?3
         WHERE id = ?4 AND status = 'pending'",
        params![fill.filled_quantity, fill.filled_price, updated_at, fill.order_id],
    )?;

    if rows_affected == 0 {
        println!("Order {} not found or already filled", fill.order_id);
    } else {
        println!("Order {} filled at price {}", fill.order_id, fill.filled_price);
    }

    Ok(rows_affected)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    let fill = OrderFill {
        order_id: 1,
        filled_quantity: 0.5,
        filled_price: 41950.0, // Filled at a better price!
    };

    fill_order(&conn, &fill)?;

    Ok(())
}
```

## Partial Order Fills

In real trading, orders can be filled in parts:

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

fn partial_fill_order(
    conn: &Connection,
    order_id: i64,
    fill_quantity: f64,
    fill_price: f64,
) -> Result<String> {
    let updated_at = Utc::now().to_rfc3339();

    // Get current order state
    let (total_quantity, current_filled): (f64, f64) = conn.query_row(
        "SELECT quantity, filled_quantity FROM orders WHERE id = ?1",
        params![order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let new_filled = current_filled + fill_quantity;

    // Determine new status
    let new_status = if new_filled >= total_quantity {
        "filled"
    } else {
        "partially_filled"
    };

    // Calculate weighted average fill price
    let avg_price = if current_filled > 0.0 {
        // Get current average price
        let current_avg: f64 = conn.query_row(
            "SELECT COALESCE(filled_price, 0) FROM orders WHERE id = ?1",
            params![order_id],
            |row| row.get(0),
        )?;

        // Weighted average
        (current_avg * current_filled + fill_price * fill_quantity) / new_filled
    } else {
        fill_price
    };

    conn.execute(
        "UPDATE orders
         SET status = ?1,
             filled_quantity = ?2,
             filled_price = ?3,
             updated_at = ?4
         WHERE id = ?5",
        params![new_status, new_filled, avg_price, updated_at, order_id],
    )?;

    println!(
        "Order {}: filled {}/{} at average price {:.2}",
        order_id, new_filled, total_quantity, avg_price
    );

    Ok(new_status.to_string())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Create an order for 1 BTC
    conn.execute(
        "INSERT INTO orders (symbol, side, quantity, price, status, created_at)
         VALUES ('BTC/USDT', 'buy', 1.0, 42000.0, 'pending', datetime('now'))",
        [],
    )?;
    let order_id = conn.last_insert_rowid();

    // Partial fills
    partial_fill_order(&conn, order_id, 0.3, 41900.0)?;  // First fill
    partial_fill_order(&conn, order_id, 0.4, 41950.0)?;  // Second fill
    partial_fill_order(&conn, order_id, 0.3, 42000.0)?;  // Final fill

    Ok(())
}
```

## UPDATE with Conditions: Cancelling Stale Orders

```rust
use rusqlite::{Connection, Result, params};
use chrono::{Utc, Duration};

fn cancel_old_pending_orders(conn: &Connection, max_age_hours: i64) -> Result<usize> {
    let cutoff_time = (Utc::now() - Duration::hours(max_age_hours)).to_rfc3339();
    let updated_at = Utc::now().to_rfc3339();

    let rows_affected = conn.execute(
        "UPDATE orders
         SET status = 'cancelled', updated_at = ?1
         WHERE status = 'pending' AND created_at < ?2",
        params![updated_at, cutoff_time],
    )?;

    println!("Cancelled {} stale orders", rows_affected);
    Ok(rows_affected)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Cancel all pending orders older than 24 hours
    cancel_old_pending_orders(&conn, 24)?;

    Ok(())
}
```

## Updating Stop-Loss and Take-Profit

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

fn update_stop_loss(
    conn: &Connection,
    order_id: i64,
    new_stop_loss: f64,
) -> Result<bool> {
    let updated_at = Utc::now().to_rfc3339();

    // Verify stop-loss is logical for the position
    let (side, entry_price): (String, f64) = conn.query_row(
        "SELECT side, price FROM orders WHERE id = ?1 AND status = 'filled'",
        params![order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let is_valid = match side.as_str() {
        "buy" => new_stop_loss < entry_price,   // For long, stop below entry
        "sell" => new_stop_loss > entry_price,  // For short, stop above entry
        _ => false,
    };

    if !is_valid {
        println!("Error: stop-loss {} is invalid for {} position with entry {}",
                 new_stop_loss, side, entry_price);
        return Ok(false);
    }

    conn.execute(
        "UPDATE orders
         SET stop_loss = ?1, updated_at = ?2
         WHERE id = ?3",
        params![new_stop_loss, updated_at, order_id],
    )?;

    println!("Stop-loss updated to {}", new_stop_loss);
    Ok(true)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Add stop_loss column if it doesn't exist
    conn.execute(
        "ALTER TABLE orders ADD COLUMN stop_loss REAL",
        [],
    ).ok(); // Ignore error if column already exists

    // Update stop-loss
    update_stop_loss(&conn, 1, 40000.0)?;

    Ok(())
}
```

## Bulk Update: Exchange Migration

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

fn migrate_orders_to_exchange(
    conn: &Connection,
    old_symbol_suffix: &str,
    new_symbol_suffix: &str,
) -> Result<usize> {
    let updated_at = Utc::now().to_rfc3339();

    // Update symbols by replacing the exchange suffix
    // Example: BTC/USDT:BINANCE -> BTC/USDT:BYBIT
    let rows_affected = conn.execute(
        "UPDATE orders
         SET symbol = REPLACE(symbol, ?1, ?2),
             updated_at = ?3
         WHERE symbol LIKE ?4 AND status = 'pending'",
        params![
            old_symbol_suffix,
            new_symbol_suffix,
            updated_at,
            format!("%{}", old_symbol_suffix)
        ],
    )?;

    println!("Migrated {} orders from {} to {}",
             rows_affected, old_symbol_suffix, new_symbol_suffix);
    Ok(rows_affected)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Migrate all pending orders from Binance to Bybit
    migrate_orders_to_exchange(&conn, ":BINANCE", ":BYBIT")?;

    Ok(())
}
```

## Safe UPDATE with Result Verification

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

#[derive(Debug)]
enum UpdateError {
    NotFound,
    AlreadyProcessed,
    DatabaseError(rusqlite::Error),
}

fn safe_update_order_status(
    conn: &Connection,
    order_id: i64,
    expected_status: &str,
    new_status: &str,
) -> std::result::Result<(), UpdateError> {
    let updated_at = Utc::now().to_rfc3339();

    // Check existence and current status
    let current_status: Option<String> = conn
        .query_row(
            "SELECT status FROM orders WHERE id = ?1",
            params![order_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => UpdateError::NotFound,
            other => UpdateError::DatabaseError(other),
        })?
        .into();

    match current_status {
        None => return Err(UpdateError::NotFound),
        Some(status) if status != expected_status => {
            println!("Order {} is already in status '{}', expected '{}'",
                     order_id, status, expected_status);
            return Err(UpdateError::AlreadyProcessed);
        }
        _ => {}
    }

    // Execute update
    let rows = conn
        .execute(
            "UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4",
            params![new_status, updated_at, order_id, expected_status],
        )
        .map_err(UpdateError::DatabaseError)?;

    if rows == 0 {
        // Race condition — status changed between check and update
        Err(UpdateError::AlreadyProcessed)
    } else {
        println!("Order {} updated: {} -> {}", order_id, expected_status, new_status);
        Ok(())
    }
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    match safe_update_order_status(&conn, 1, "pending", "filled") {
        Ok(()) => println!("Success!"),
        Err(UpdateError::NotFound) => println!("Order not found"),
        Err(UpdateError::AlreadyProcessed) => println!("Order already processed"),
        Err(UpdateError::DatabaseError(e)) => println!("Database error: {}", e),
    }

    Ok(())
}
```

## Complete Example: Order Management System

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

#[derive(Debug, Clone)]
struct Order {
    id: i64,
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    status: String,
    filled_quantity: f64,
    filled_price: Option<f64>,
}

struct OrderManager {
    conn: Connection,
}

impl OrderManager {
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                filled_quantity REAL DEFAULT 0,
                filled_price REAL,
                created_at TEXT NOT NULL,
                updated_at TEXT
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    fn create_order(&self, symbol: &str, side: &str, quantity: f64, price: f64) -> Result<i64> {
        let created_at = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO orders (symbol, side, quantity, price, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![symbol, side, quantity, price, created_at],
        )?;

        let order_id = self.conn.last_insert_rowid();
        println!("Created order #{}: {} {} {} @ {}",
                 order_id, side, quantity, symbol, price);
        Ok(order_id)
    }

    fn fill_order(&self, order_id: i64, fill_price: f64) -> Result<bool> {
        let updated_at = Utc::now().to_rfc3339();

        // Get quantity from order
        let quantity: f64 = self.conn.query_row(
            "SELECT quantity FROM orders WHERE id = ?1 AND status = 'pending'",
            params![order_id],
            |row| row.get(0),
        )?;

        let rows = self.conn.execute(
            "UPDATE orders
             SET status = 'filled',
                 filled_quantity = quantity,
                 filled_price = ?1,
                 updated_at = ?2
             WHERE id = ?3 AND status = 'pending'",
            params![fill_price, updated_at, order_id],
        )?;

        if rows > 0 {
            println!("Order #{} filled at price {}", order_id, fill_price);
            Ok(true)
        } else {
            println!("Order #{} not found or already processed", order_id);
            Ok(false)
        }
    }

    fn cancel_order(&self, order_id: i64) -> Result<bool> {
        let updated_at = Utc::now().to_rfc3339();

        let rows = self.conn.execute(
            "UPDATE orders
             SET status = 'cancelled', updated_at = ?1
             WHERE id = ?2 AND status = 'pending'",
            params![updated_at, order_id],
        )?;

        if rows > 0 {
            println!("Order #{} cancelled", order_id);
            Ok(true)
        } else {
            println!("Order #{} not found or already processed", order_id);
            Ok(false)
        }
    }

    fn get_order(&self, order_id: i64) -> Result<Order> {
        self.conn.query_row(
            "SELECT id, symbol, side, quantity, price, status, filled_quantity, filled_price
             FROM orders WHERE id = ?1",
            params![order_id],
            |row| {
                Ok(Order {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    side: row.get(2)?,
                    quantity: row.get(3)?,
                    price: row.get(4)?,
                    status: row.get(5)?,
                    filled_quantity: row.get(6)?,
                    filled_price: row.get(7)?,
                })
            },
        )
    }

    fn list_orders_by_status(&self, status: &str) -> Result<Vec<Order>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, side, quantity, price, status, filled_quantity, filled_price
             FROM orders WHERE status = ?1 ORDER BY id"
        )?;

        let orders = stmt.query_map(params![status], |row| {
            Ok(Order {
                id: row.get(0)?,
                symbol: row.get(1)?,
                side: row.get(2)?,
                quantity: row.get(3)?,
                price: row.get(4)?,
                status: row.get(5)?,
                filled_quantity: row.get(6)?,
                filled_price: row.get(7)?,
            })
        })?;

        orders.collect()
    }
}

fn main() -> Result<()> {
    let manager = OrderManager::new("trading_orders.db")?;

    // Create several orders
    let order1 = manager.create_order("BTC/USDT", "buy", 0.5, 42000.0)?;
    let order2 = manager.create_order("ETH/USDT", "buy", 2.0, 2500.0)?;
    let order3 = manager.create_order("BTC/USDT", "sell", 0.3, 43000.0)?;

    println!("\n--- All pending orders ---");
    for order in manager.list_orders_by_status("pending")? {
        println!("{:?}", order);
    }

    // Fill the first order
    println!("\n--- Filling order ---");
    manager.fill_order(order1, 41950.0)?;

    // Cancel the second order
    println!("\n--- Cancelling order ---");
    manager.cancel_order(order2)?;

    // Check statuses
    println!("\n--- Final statuses ---");
    println!("Order 1: {:?}", manager.get_order(order1)?);
    println!("Order 2: {:?}", manager.get_order(order2)?);
    println!("Order 3: {:?}", manager.get_order(order3)?);

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| `UPDATE ... SET` | Modifying values in existing records |
| `WHERE` clause | Filtering records for update (mandatory!) |
| Multiple fields | Updating several columns in one query |
| Conditional update | `UPDATE ... WHERE status = 'pending'` |
| Result checking | `rows_affected` shows number of changed records |
| Safe UPDATE | Verifying state before updating |

## Practical Exercises

1. **Basic Update**: Write a function `update_order_price` that changes the price of a pending order. Add validation that the new price differs from the current one.

2. **Partial Fills**: Implement a function `add_fill` that adds a partial fill to an order. It should:
   - Update `filled_quantity`
   - Recalculate `filled_price` as weighted average
   - Change status to `partially_filled` or `filled`

3. **Trailing Stop**: Write a function `update_trailing_stop` that:
   - Takes the current market price
   - Updates stop-loss if price moved favorably
   - For long: raises stop when price rises
   - For short: lowers stop when price falls

4. **Bulk Cancel**: Implement `cancel_orders_by_symbol` that cancels all pending orders for a given symbol. Return the list of cancelled order IDs.

## Homework

1. **Status System**: Extend the order system with statuses:
   - `pending` → `partially_filled` → `filled`
   - `pending` → `cancelled`
   - `filled` → `closed` (after closing position)

   Implement a `transition_status` function that validates allowed transitions.

2. **Change History**: Create an `order_history` table that records every order change. On each UPDATE, also add a record to history with previous and new state.

3. **Bulk Update with Transaction**: Write a function `fill_multiple_orders` that fills multiple orders atomically (all or nothing). Use transactions to ensure integrity.

4. **Auto-Cancel**: Implement an `auto_cancel_expired` function that:
   - Cancels orders older than a specified time
   - Cancels orders for symbols no longer in the active list
   - Logs all cancellations

## Navigation

[← Previous day](../218-select-reading-trade-history/en.md) | [Next day →](../220-delete-removing-cancelled-order/en.md)
