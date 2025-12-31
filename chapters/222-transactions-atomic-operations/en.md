# Day 222: Transactions: Atomic Operations

## Trading Analogy

Imagine you're executing an arbitrage trade: buying BTC on Exchange A for $42,000 and simultaneously selling it on Exchange B for $42,100. Profit — $100. But what if after buying on Exchange A, Exchange B goes down and you can't sell? You're left holding BTC that might drop in value.

In databases, this situation is solved with **transactions** — a group of operations that execute **atomically**: either all succeed, or none do. It's like the "all or nothing" rule in trading: a deal is either fully completed or fully cancelled.

In real trading, transactions are critical for:
- Order execution (deducting balance + adding position)
- Arbitrage trades (buy + sell must be atomic)
- Portfolio updates (modifying multiple positions simultaneously)
- Commission calculation (deducting fee + executing operation)

## What is a Transaction?

A transaction is a sequence of database operations that possesses **ACID** properties:

| Property | Description | Trading Example |
|----------|-------------|-----------------|
| **A**tomicity | All operations execute completely or not at all | Buying an asset: deduct funds + add position |
| **C**onsistency | Database transitions from one valid state to another | Total portfolio balance always equals sum of positions |
| **I**solation | Concurrent transactions don't interfere with each other | Two traders buying simultaneously — each sees their own balance |
| **D**urability | After confirmation, data is never lost | After trade confirmation, it's saved permanently |

## Basic Transaction Example

```rust
use rusqlite::{Connection, Result, Transaction};

#[derive(Debug)]
struct Trade {
    id: i64,
    symbol: String,
    side: String, // "buy" or "sell"
    quantity: f64,
    price: f64,
}

fn execute_trade(conn: &mut Connection, trade: &Trade) -> Result<()> {
    // Start a transaction
    let tx: Transaction = conn.transaction()?;

    // 1. Check sufficient funds
    let balance: f64 = tx.query_row(
        "SELECT balance FROM accounts WHERE id = 1",
        [],
        |row| row.get(0),
    )?;

    let cost = trade.quantity * trade.price;

    if trade.side == "buy" && balance < cost {
        // Rollback the transaction
        tx.rollback()?;
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    // 2. Update balance
    if trade.side == "buy" {
        tx.execute(
            "UPDATE accounts SET balance = balance - ?1 WHERE id = 1",
            [cost],
        )?;
    } else {
        tx.execute(
            "UPDATE accounts SET balance = balance + ?1 WHERE id = 1",
            [cost],
        )?;
    }

    // 3. Record the trade
    tx.execute(
        "INSERT INTO trades (symbol, side, quantity, price) VALUES (?1, ?2, ?3, ?4)",
        (&trade.symbol, &trade.side, trade.quantity, trade.price),
    )?;

    // 4. Update position
    let delta = if trade.side == "buy" { trade.quantity } else { -trade.quantity };
    tx.execute(
        "INSERT INTO positions (symbol, quantity) VALUES (?1, ?2)
         ON CONFLICT(symbol) DO UPDATE SET quantity = quantity + ?2",
        (&trade.symbol, delta),
    )?;

    // Commit the transaction — all changes are applied atomically
    tx.commit()?;

    println!("Trade executed: {:?}", trade);
    Ok(())
}

fn main() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;

    // Create tables
    conn.execute_batch(
        "CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance REAL);
         CREATE TABLE trades (id INTEGER PRIMARY KEY, symbol TEXT, side TEXT, quantity REAL, price REAL);
         CREATE TABLE positions (symbol TEXT PRIMARY KEY, quantity REAL);
         INSERT INTO accounts (id, balance) VALUES (1, 100000.0);"
    )?;

    let trade = Trade {
        id: 1,
        symbol: "BTC".to_string(),
        side: "buy".to_string(),
        quantity: 1.0,
        price: 42000.0,
    };

    execute_trade(&mut conn, &trade)?;

    // Verify the result
    let balance: f64 = conn.query_row(
        "SELECT balance FROM accounts WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    println!("Balance after trade: ${:.2}", balance);

    Ok(())
}
```

## Transaction Rollback Example

```rust
use rusqlite::{Connection, Result};

struct OrderExecution {
    order_id: i64,
    symbol: String,
    quantity: f64,
    price: f64,
    commission: f64,
}

fn execute_order_with_commission(
    conn: &mut Connection,
    execution: &OrderExecution,
) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Get current balance
    let balance: f64 = tx
        .query_row("SELECT balance FROM accounts WHERE id = 1", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let total_cost = execution.quantity * execution.price + execution.commission;

    // Check sufficient funds (including commission)
    if balance < total_cost {
        // Transaction will automatically rollback when exiting scope
        return Err(format!(
            "Insufficient funds: need ${:.2}, have ${:.2}",
            total_cost, balance
        ));
    }

    // Deduct trade cost
    tx.execute(
        "UPDATE accounts SET balance = balance - ?1 WHERE id = 1",
        [execution.quantity * execution.price],
    )
    .map_err(|e| e.to_string())?;

    // Deduct commission
    tx.execute(
        "UPDATE accounts SET balance = balance - ?1 WHERE id = 1",
        [execution.commission],
    )
    .map_err(|e| e.to_string())?;

    // Add position
    tx.execute(
        "INSERT INTO positions (symbol, quantity) VALUES (?1, ?2)
         ON CONFLICT(symbol) DO UPDATE SET quantity = quantity + ?2",
        (&execution.symbol, execution.quantity),
    )
    .map_err(|e| e.to_string())?;

    // Record commission
    tx.execute(
        "INSERT INTO commissions (order_id, amount) VALUES (?1, ?2)",
        [execution.order_id as i64, execution.commission as i64],
    )
    .map_err(|e| {
        // If commission recording fails — entire transaction rolls back
        format!("Commission recording error: {}", e)
    })?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn main() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;

    conn.execute_batch(
        "CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance REAL);
         CREATE TABLE positions (symbol TEXT PRIMARY KEY, quantity REAL);
         CREATE TABLE commissions (order_id INTEGER, amount REAL);
         INSERT INTO accounts (id, balance) VALUES (1, 50000.0);"
    )?;

    let execution = OrderExecution {
        order_id: 1,
        symbol: "ETH".to_string(),
        quantity: 10.0,
        price: 2500.0,
        commission: 25.0,
    };

    match execute_order_with_commission(&mut conn, &execution) {
        Ok(()) => println!("Order executed successfully"),
        Err(e) => println!("Execution error: {}", e),
    }

    // Check balance
    let balance: f64 = conn.query_row(
        "SELECT balance FROM accounts WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    println!("Final balance: ${:.2}", balance);

    Ok(())
}
```

## Arbitrage Trade with Transaction

```rust
use rusqlite::{Connection, Result};
use std::collections::HashMap;

struct ArbitrageTrade {
    buy_exchange: String,
    sell_exchange: String,
    symbol: String,
    quantity: f64,
    buy_price: f64,
    sell_price: f64,
}

fn execute_arbitrage(
    conn: &mut Connection,
    arb: &ArbitrageTrade,
) -> Result<f64, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Get balances on both exchanges
    let buy_exchange_balance: f64 = tx
        .query_row(
            "SELECT balance FROM exchange_balances WHERE exchange = ?1",
            [&arb.buy_exchange],
            |row| row.get(0),
        )
        .map_err(|e| format!("Error getting {} balance: {}", arb.buy_exchange, e))?;

    let sell_exchange_position: f64 = tx
        .query_row(
            "SELECT COALESCE(quantity, 0) FROM exchange_positions
             WHERE exchange = ?1 AND symbol = ?2",
            [&arb.sell_exchange, &arb.symbol],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let buy_cost = arb.quantity * arb.buy_price;
    let sell_revenue = arb.quantity * arb.sell_price;

    // Check conditions
    if buy_exchange_balance < buy_cost {
        return Err(format!(
            "Insufficient funds on {}: need ${:.2}, have ${:.2}",
            arb.buy_exchange, buy_cost, buy_exchange_balance
        ));
    }

    if sell_exchange_position < arb.quantity {
        return Err(format!(
            "Insufficient {} on {}: need {}, have {}",
            arb.symbol, arb.sell_exchange, arb.quantity, sell_exchange_position
        ));
    }

    // Execute buy on first exchange
    tx.execute(
        "UPDATE exchange_balances SET balance = balance - ?1 WHERE exchange = ?2",
        [buy_cost, arb.buy_exchange.parse().unwrap_or(0.0)],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO exchange_positions (exchange, symbol, quantity) VALUES (?1, ?2, ?3)
         ON CONFLICT(exchange, symbol) DO UPDATE SET quantity = quantity + ?3",
        [&arb.buy_exchange, &arb.symbol, &arb.quantity.to_string()],
    )
    .map_err(|e| e.to_string())?;

    // Execute sell on second exchange
    tx.execute(
        "UPDATE exchange_positions SET quantity = quantity - ?1
         WHERE exchange = ?2 AND symbol = ?3",
        [arb.quantity, arb.sell_exchange.parse().unwrap_or(0.0), arb.symbol.parse().unwrap_or(0.0)],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE exchange_balances SET balance = balance + ?1 WHERE exchange = ?2",
        [sell_revenue, arb.sell_exchange.parse().unwrap_or(0.0)],
    )
    .map_err(|e| e.to_string())?;

    // Record arbitrage trade
    let profit = sell_revenue - buy_cost;
    tx.execute(
        "INSERT INTO arbitrage_trades (buy_exchange, sell_exchange, symbol, quantity, profit)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            &arb.buy_exchange,
            &arb.sell_exchange,
            &arb.symbol,
            &arb.quantity.to_string(),
            &profit.to_string(),
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(profit)
}
```

## Transaction Isolation Levels

Databases support different isolation levels that determine how transactions interact with each other:

```rust
use rusqlite::{Connection, Result, TransactionBehavior};

fn demonstrate_isolation_levels(conn: &mut Connection) -> Result<()> {
    // DEFERRED — lock is deferred until first write
    // Suitable for operations that mostly read data
    let tx = conn.transaction_with_behavior(TransactionBehavior::Deferred)?;
    // ... operations ...
    tx.commit()?;

    // IMMEDIATE — immediate write lock
    // Other transactions can only read
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    // ... operations ...
    tx.commit()?;

    // EXCLUSIVE — full database lock
    // Other transactions can neither read nor write
    let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;
    // ... operations ...
    tx.commit()?;

    Ok(())
}

// Example usage for critical trading operations
fn execute_critical_trade(conn: &mut Connection) -> Result<()> {
    // Use EXCLUSIVE for critically important operations
    let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;

    // Now nobody can modify data while we're working
    let balance: f64 = tx.query_row(
        "SELECT balance FROM accounts WHERE id = 1",
        [],
        |row| row.get(0),
    )?;

    println!("Balance during exclusive transaction: ${:.2}", balance);

    // Execute critical operations...

    tx.commit()?;
    Ok(())
}
```

## Savepoints — Checkpoint Within Transaction

Savepoints allow you to create "checkpoints" within a transaction and rollback to them if needed:

```rust
use rusqlite::{Connection, Result, Savepoint};

fn execute_multi_leg_trade(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    // First leg of the trade
    tx.execute(
        "UPDATE accounts SET balance = balance - 10000 WHERE id = 1",
        [],
    )?;
    tx.execute(
        "INSERT INTO trades (symbol, side, quantity, price) VALUES ('BTC', 'buy', 0.25, 40000)",
        [],
    )?;

    // Create savepoint before second leg
    let mut sp = tx.savepoint()?;

    // Try to execute second leg
    let result = sp.execute(
        "UPDATE accounts SET balance = balance - 5000 WHERE id = 1",
        [],
    );

    match result {
        Ok(_) => {
            sp.execute(
                "INSERT INTO trades (symbol, side, quantity, price) VALUES ('ETH', 'buy', 2.0, 2500)",
                [],
            )?;
            // Accept savepoint — changes will be included in main transaction
            sp.commit()?;
        }
        Err(e) => {
            println!("Second leg failed: {}, rolling back to savepoint", e);
            // Rollback only second leg, first remains
            sp.rollback()?;
        }
    }

    // Commit entire transaction (with first leg, possibly without second)
    tx.commit()?;
    Ok(())
}

fn main() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;

    conn.execute_batch(
        "CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance REAL);
         CREATE TABLE trades (id INTEGER PRIMARY KEY, symbol TEXT, side TEXT, quantity REAL, price REAL);
         INSERT INTO accounts (id, balance) VALUES (1, 20000.0);"
    )?;

    execute_multi_leg_trade(&mut conn)?;

    // Check result
    let balance: f64 = conn.query_row(
        "SELECT balance FROM accounts WHERE id = 1",
        [],
        |row| row.get(0),
    )?;

    let trade_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM trades",
        [],
        |row| row.get(0),
    )?;

    println!("Balance: ${:.2}, Trades: {}", balance, trade_count);

    Ok(())
}
```

## Practical Example: Trading Engine with Transactions

```rust
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Option<i64>,
    pub symbol: String,
    pub side: String, // "buy" or "sell"
    pub order_type: String, // "market" or "limit"
    pub quantity: f64,
    pub price: Option<f64>,
    pub status: String,
}

pub struct TradingEngine {
    conn: Arc<Mutex<Connection>>,
}

impl TradingEngine {
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        conn.execute_batch(
            "CREATE TABLE accounts (
                id INTEGER PRIMARY KEY,
                balance REAL NOT NULL,
                reserved REAL DEFAULT 0
            );
            CREATE TABLE positions (
                symbol TEXT PRIMARY KEY,
                quantity REAL NOT NULL,
                avg_price REAL NOT NULL
            );
            CREATE TABLE orders (
                id INTEGER PRIMARY KEY,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                order_type TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL,
                status TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE trades (
                id INTEGER PRIMARY KEY,
                order_id INTEGER,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL NOT NULL,
                executed_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            INSERT INTO accounts (id, balance) VALUES (1, 100000.0);"
        )?;

        Ok(TradingEngine {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn place_order(&self, order: &mut Order) -> Result<i64, String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // For limit buy orders, reserve funds
        if order.side == "buy" && order.order_type == "limit" {
            let price = order.price.ok_or("Limit order requires price")?;
            let cost = order.quantity * price;

            let balance: f64 = tx
                .query_row("SELECT balance - reserved FROM accounts WHERE id = 1", [], |r| r.get(0))
                .map_err(|e| e.to_string())?;

            if balance < cost {
                return Err(format!("Insufficient funds: need ${:.2}, available ${:.2}", cost, balance));
            }

            tx.execute(
                "UPDATE accounts SET reserved = reserved + ?1 WHERE id = 1",
                [cost],
            )
            .map_err(|e| e.to_string())?;
        }

        // Save order
        tx.execute(
            "INSERT INTO orders (symbol, side, order_type, quantity, price, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &order.symbol,
                &order.side,
                &order.order_type,
                order.quantity,
                order.price,
                "pending",
            ),
        )
        .map_err(|e| e.to_string())?;

        let order_id = tx.last_insert_rowid();
        order.id = Some(order_id);
        order.status = "pending".to_string();

        tx.commit().map_err(|e| e.to_string())?;

        Ok(order_id)
    }

    pub fn execute_order(&self, order_id: i64, execution_price: f64) -> Result<(), String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // Get order information
        let (symbol, side, quantity, price, status): (String, String, f64, Option<f64>, String) = tx
            .query_row(
                "SELECT symbol, side, quantity, price, status FROM orders WHERE id = ?1",
                [order_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .map_err(|e| format!("Order not found: {}", e))?;

        if status != "pending" {
            return Err(format!("Order already {}", status));
        }

        let cost = quantity * execution_price;

        if side == "buy" {
            // Check and deduct funds
            let available: f64 = tx
                .query_row("SELECT balance FROM accounts WHERE id = 1", [], |r| r.get(0))
                .map_err(|e| e.to_string())?;

            // If it was a limit order, release the reserve
            if let Some(limit_price) = price {
                let reserved_amount = quantity * limit_price;
                tx.execute(
                    "UPDATE accounts SET reserved = reserved - ?1 WHERE id = 1",
                    [reserved_amount],
                )
                .map_err(|e| e.to_string())?;
            }

            if available < cost {
                tx.execute(
                    "UPDATE orders SET status = 'rejected' WHERE id = ?1",
                    [order_id],
                )
                .map_err(|e| e.to_string())?;
                tx.commit().map_err(|e| e.to_string())?;
                return Err("Insufficient funds".to_string());
            }

            // Deduct cost
            tx.execute(
                "UPDATE accounts SET balance = balance - ?1 WHERE id = 1",
                [cost],
            )
            .map_err(|e| e.to_string())?;

            // Add position
            tx.execute(
                "INSERT INTO positions (symbol, quantity, avg_price) VALUES (?1, ?2, ?3)
                 ON CONFLICT(symbol) DO UPDATE SET
                    avg_price = (avg_price * quantity + ?3 * ?2) / (quantity + ?2),
                    quantity = quantity + ?2",
                (&symbol, quantity, execution_price),
            )
            .map_err(|e| e.to_string())?;
        } else {
            // Sell
            let current_qty: f64 = tx
                .query_row(
                    "SELECT COALESCE(quantity, 0) FROM positions WHERE symbol = ?1",
                    [&symbol],
                    |r| r.get(0),
                )
                .unwrap_or(0.0);

            if current_qty < quantity {
                tx.execute(
                    "UPDATE orders SET status = 'rejected' WHERE id = ?1",
                    [order_id],
                )
                .map_err(|e| e.to_string())?;
                tx.commit().map_err(|e| e.to_string())?;
                return Err(format!("Insufficient {}: have {}, need {}", symbol, current_qty, quantity));
            }

            // Decrease position
            tx.execute(
                "UPDATE positions SET quantity = quantity - ?1 WHERE symbol = ?2",
                (quantity, &symbol),
            )
            .map_err(|e| e.to_string())?;

            // Remove empty positions
            tx.execute(
                "DELETE FROM positions WHERE quantity <= 0",
                [],
            )
            .map_err(|e| e.to_string())?;

            // Credit funds
            tx.execute(
                "UPDATE accounts SET balance = balance + ?1 WHERE id = 1",
                [cost],
            )
            .map_err(|e| e.to_string())?;
        }

        // Update order status
        tx.execute(
            "UPDATE orders SET status = 'filled' WHERE id = ?1",
            [order_id],
        )
        .map_err(|e| e.to_string())?;

        // Record trade
        tx.execute(
            "INSERT INTO trades (order_id, symbol, side, quantity, price)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (order_id, &symbol, &side, quantity, execution_price),
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn cancel_order(&self, order_id: i64) -> Result<(), String> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        let (side, quantity, price, status): (String, f64, Option<f64>, String) = tx
            .query_row(
                "SELECT side, quantity, price, status FROM orders WHERE id = ?1",
                [order_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| format!("Order not found: {}", e))?;

        if status != "pending" {
            return Err(format!("Cannot cancel order with status: {}", status));
        }

        // Release reserve for limit buy orders
        if side == "buy" {
            if let Some(limit_price) = price {
                let reserved_amount = quantity * limit_price;
                tx.execute(
                    "UPDATE accounts SET reserved = reserved - ?1 WHERE id = 1",
                    [reserved_amount],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        tx.execute(
            "UPDATE orders SET status = 'cancelled' WHERE id = ?1",
            [order_id],
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_balance(&self) -> Result<(f64, f64), String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT balance, reserved FROM accounts WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())
    }

    pub fn get_positions(&self) -> Result<Vec<(String, f64, f64)>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT symbol, quantity, avg_price FROM positions")
            .map_err(|e| e.to_string())?;

        let positions = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(positions)
    }
}

fn main() -> Result<()> {
    let engine = TradingEngine::new()?;

    println!("=== Trading Engine with Transactions ===\n");

    // Place orders
    let mut buy_order = Order {
        id: None,
        symbol: "BTC".to_string(),
        side: "buy".to_string(),
        order_type: "limit".to_string(),
        quantity: 1.0,
        price: Some(42000.0),
        status: String::new(),
    };

    match engine.place_order(&mut buy_order) {
        Ok(id) => println!("Buy order placed: #{}", id),
        Err(e) => println!("Placement error: {}", e),
    }

    let (balance, reserved) = engine.get_balance()?;
    println!("Balance: ${:.2}, Reserved: ${:.2}\n", balance, reserved);

    // Execute order
    if let Some(order_id) = buy_order.id {
        match engine.execute_order(order_id, 41500.0) {
            Ok(()) => println!("Order #{} filled at $41,500", order_id),
            Err(e) => println!("Execution error: {}", e),
        }
    }

    let (balance, reserved) = engine.get_balance()?;
    println!("Balance: ${:.2}, Reserved: ${:.2}", balance, reserved);

    println!("\nPositions:");
    for (symbol, qty, avg_price) in engine.get_positions()? {
        println!("  {}: {} @ ${:.2}", symbol, qty, avg_price);
    }

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Transaction | Group of operations executed atomically |
| ACID | Atomicity, Consistency, Isolation, Durability |
| `transaction()` | Start a transaction |
| `commit()` | Commit changes |
| `rollback()` | Rollback changes |
| Savepoint | Checkpoint within a transaction |
| Isolation Levels | Deferred, Immediate, Exclusive |

## Homework

1. **Atomic Transfer**: Implement a funds transfer function between two accounts that:
   - Checks sufficient funds
   - Debits from one account
   - Credits to another
   - Records the transaction in history
   - Rolls back everything on any error

2. **Batch Order Execution**: Create a function that executes a list of orders in a single transaction:
   - If one order cannot be executed — all are rolled back
   - Use savepoints for partial rollback

3. **Trading Session**: Implement a system that:
   - Opens a trading session (transaction)
   - Allows executing multiple operations
   - Closes the session with commit or rollback
   - Maintains a journal of all operations in the session

4. **Failure Recovery**: Write a program that:
   - Simulates a failure in the middle of a transaction
   - Demonstrates that data remains in a consistent state
   - Shows which operations were rolled back

## Navigation

[← Previous day](../221-connection-pools/en.md) | [Next day →](../223-prepared-statements/en.md)
