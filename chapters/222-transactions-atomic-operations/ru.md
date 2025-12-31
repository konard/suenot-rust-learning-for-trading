# День 222: Транзакции: атомарные операции

## Аналогия из трейдинга

Представь, что ты выполняешь арбитражную сделку: покупаешь BTC на бирже A за $42,000 и одновременно продаёшь его на бирже B за $42,100. Прибыль — $100. Но что если после покупки на бирже A биржа B упала, и ты не можешь продать? Ты остался с BTC, который может упасть в цене.

В базах данных такая ситуация решается **транзакциями** — группой операций, которые выполняются **атомарно**: либо все успешно, либо ни одна. Это как правило "всё или ничего" в трейдинге: сделка либо полностью завершена, либо полностью отменена.

В реальном трейдинге транзакции критичны для:
- Исполнения ордеров (списание баланса + добавление позиции)
- Арбитражных сделок (покупка + продажа должны быть атомарными)
- Обновления портфеля (изменение нескольких позиций одновременно)
- Расчёта комиссий (списание комиссии + выполнение операции)

## Что такое транзакция?

Транзакция — это последовательность операций с базой данных, которая обладает свойствами **ACID**:

| Свойство | Описание | Пример из трейдинга |
|----------|----------|---------------------|
| **A**tomicity (Атомарность) | Все операции выполняются целиком или не выполняются вообще | Покупка актива: списание денег + добавление позиции |
| **C**onsistency (Согласованность) | База данных переходит из одного корректного состояния в другое | Общий баланс портфеля всегда равен сумме позиций |
| **I**solation (Изолированность) | Параллельные транзакции не мешают друг другу | Два трейдера одновременно покупают — каждый видит свой баланс |
| **D**urability (Долговечность) | После подтверждения данные не теряются | После подтверждения сделки она сохраняется навсегда |

## Базовый пример транзакции

```rust
use rusqlite::{Connection, Result, Transaction};

#[derive(Debug)]
struct Trade {
    id: i64,
    symbol: String,
    side: String, // "buy" или "sell"
    quantity: f64,
    price: f64,
}

fn execute_trade(conn: &mut Connection, trade: &Trade) -> Result<()> {
    // Начинаем транзакцию
    let tx: Transaction = conn.transaction()?;

    // 1. Проверяем достаточность средств
    let balance: f64 = tx.query_row(
        "SELECT balance FROM accounts WHERE id = 1",
        [],
        |row| row.get(0),
    )?;

    let cost = trade.quantity * trade.price;

    if trade.side == "buy" && balance < cost {
        // Откатываем транзакцию
        tx.rollback()?;
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    // 2. Обновляем баланс
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

    // 3. Записываем сделку
    tx.execute(
        "INSERT INTO trades (symbol, side, quantity, price) VALUES (?1, ?2, ?3, ?4)",
        (&trade.symbol, &trade.side, trade.quantity, trade.price),
    )?;

    // 4. Обновляем позицию
    let delta = if trade.side == "buy" { trade.quantity } else { -trade.quantity };
    tx.execute(
        "INSERT INTO positions (symbol, quantity) VALUES (?1, ?2)
         ON CONFLICT(symbol) DO UPDATE SET quantity = quantity + ?2",
        (&trade.symbol, delta),
    )?;

    // Фиксируем транзакцию — все изменения применяются атомарно
    tx.commit()?;

    println!("Сделка выполнена: {:?}", trade);
    Ok(())
}

fn main() -> Result<()> {
    let mut conn = Connection::open_in_memory()?;

    // Создаём таблицы
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

    // Проверяем результат
    let balance: f64 = conn.query_row(
        "SELECT balance FROM accounts WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    println!("Баланс после сделки: ${:.2}", balance);

    Ok(())
}
```

## Пример с откатом транзакции

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

    // Получаем текущий баланс
    let balance: f64 = tx
        .query_row("SELECT balance FROM accounts WHERE id = 1", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let total_cost = execution.quantity * execution.price + execution.commission;

    // Проверяем достаточность средств (включая комиссию)
    if balance < total_cost {
        // Транзакция автоматически откатится при выходе из scope
        return Err(format!(
            "Недостаточно средств: нужно ${:.2}, есть ${:.2}",
            total_cost, balance
        ));
    }

    // Списываем стоимость сделки
    tx.execute(
        "UPDATE accounts SET balance = balance - ?1 WHERE id = 1",
        [execution.quantity * execution.price],
    )
    .map_err(|e| e.to_string())?;

    // Списываем комиссию
    tx.execute(
        "UPDATE accounts SET balance = balance - ?1 WHERE id = 1",
        [execution.commission],
    )
    .map_err(|e| e.to_string())?;

    // Добавляем позицию
    tx.execute(
        "INSERT INTO positions (symbol, quantity) VALUES (?1, ?2)
         ON CONFLICT(symbol) DO UPDATE SET quantity = quantity + ?2",
        (&execution.symbol, execution.quantity),
    )
    .map_err(|e| e.to_string())?;

    // Записываем комиссию
    tx.execute(
        "INSERT INTO commissions (order_id, amount) VALUES (?1, ?2)",
        [execution.order_id as i64, execution.commission as i64],
    )
    .map_err(|e| {
        // Если не удалось записать комиссию — вся транзакция откатится
        format!("Ошибка записи комиссии: {}", e)
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
        Ok(()) => println!("Ордер исполнен успешно"),
        Err(e) => println!("Ошибка исполнения: {}", e),
    }

    // Проверяем баланс
    let balance: f64 = conn.query_row(
        "SELECT balance FROM accounts WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    println!("Итоговый баланс: ${:.2}", balance);

    Ok(())
}
```

## Арбитражная сделка с транзакцией

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

    // Получаем балансы на обеих биржах
    let buy_exchange_balance: f64 = tx
        .query_row(
            "SELECT balance FROM exchange_balances WHERE exchange = ?1",
            [&arb.buy_exchange],
            |row| row.get(0),
        )
        .map_err(|e| format!("Ошибка получения баланса {}: {}", arb.buy_exchange, e))?;

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

    // Проверяем условия
    if buy_exchange_balance < buy_cost {
        return Err(format!(
            "Недостаточно средств на {}: нужно ${:.2}, есть ${:.2}",
            arb.buy_exchange, buy_cost, buy_exchange_balance
        ));
    }

    if sell_exchange_position < arb.quantity {
        return Err(format!(
            "Недостаточно {} на {}: нужно {}, есть {}",
            arb.symbol, arb.sell_exchange, arb.quantity, sell_exchange_position
        ));
    }

    // Выполняем покупку на первой бирже
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

    // Выполняем продажу на второй бирже
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

    // Записываем арбитражную сделку
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

## Уровни изоляции транзакций

В базах данных существуют разные уровни изоляции, которые определяют, как транзакции взаимодействуют друг с другом:

```rust
use rusqlite::{Connection, Result, TransactionBehavior};

fn demonstrate_isolation_levels(conn: &mut Connection) -> Result<()> {
    // DEFERRED — блокировка откладывается до первой записи
    // Подходит для операций, которые в основном читают данные
    let tx = conn.transaction_with_behavior(TransactionBehavior::Deferred)?;
    // ... операции ...
    tx.commit()?;

    // IMMEDIATE — немедленная блокировка на запись
    // Другие транзакции могут только читать
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    // ... операции ...
    tx.commit()?;

    // EXCLUSIVE — полная блокировка базы данных
    // Другие транзакции не могут ни читать, ни писать
    let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;
    // ... операции ...
    tx.commit()?;

    Ok(())
}

// Пример использования для критичных торговых операций
fn execute_critical_trade(conn: &mut Connection) -> Result<()> {
    // Используем EXCLUSIVE для критически важных операций
    let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;

    // Теперь никто не может изменить данные, пока мы работаем
    let balance: f64 = tx.query_row(
        "SELECT balance FROM accounts WHERE id = 1",
        [],
        |row| row.get(0),
    )?;

    println!("Баланс во время эксклюзивной транзакции: ${:.2}", balance);

    // Выполняем критичные операции...

    tx.commit()?;
    Ok(())
}
```

## Savepoints — точки сохранения

Savepoints позволяют создавать "контрольные точки" внутри транзакции и откатываться к ним при необходимости:

```rust
use rusqlite::{Connection, Result, Savepoint};

fn execute_multi_leg_trade(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    // Первая нога сделки
    tx.execute(
        "UPDATE accounts SET balance = balance - 10000 WHERE id = 1",
        [],
    )?;
    tx.execute(
        "INSERT INTO trades (symbol, side, quantity, price) VALUES ('BTC', 'buy', 0.25, 40000)",
        [],
    )?;

    // Создаём точку сохранения перед второй ногой
    let mut sp = tx.savepoint()?;

    // Пытаемся выполнить вторую ногу
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
            // Принимаем savepoint — изменения войдут в основную транзакцию
            sp.commit()?;
        }
        Err(e) => {
            println!("Вторая нога не удалась: {}, откатываемся к savepoint", e);
            // Откатываем только вторую ногу, первая остаётся
            sp.rollback()?;
        }
    }

    // Фиксируем всю транзакцию (с первой ногой, возможно без второй)
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

    // Проверяем результат
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

    println!("Баланс: ${:.2}, Сделок: {}", balance, trade_count);

    Ok(())
}
```

## Практический пример: торговый движок с транзакциями

```rust
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Option<i64>,
    pub symbol: String,
    pub side: String, // "buy" или "sell"
    pub order_type: String, // "market" или "limit"
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

        // Для лимитных ордеров на покупку резервируем средства
        if order.side == "buy" && order.order_type == "limit" {
            let price = order.price.ok_or("Лимитный ордер требует цену")?;
            let cost = order.quantity * price;

            let balance: f64 = tx
                .query_row("SELECT balance - reserved FROM accounts WHERE id = 1", [], |r| r.get(0))
                .map_err(|e| e.to_string())?;

            if balance < cost {
                return Err(format!("Недостаточно средств: нужно ${:.2}, доступно ${:.2}", cost, balance));
            }

            tx.execute(
                "UPDATE accounts SET reserved = reserved + ?1 WHERE id = 1",
                [cost],
            )
            .map_err(|e| e.to_string())?;
        }

        // Сохраняем ордер
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

        // Получаем информацию об ордере
        let (symbol, side, quantity, price, status): (String, String, f64, Option<f64>, String) = tx
            .query_row(
                "SELECT symbol, side, quantity, price, status FROM orders WHERE id = ?1",
                [order_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .map_err(|e| format!("Ордер не найден: {}", e))?;

        if status != "pending" {
            return Err(format!("Ордер уже {}", status));
        }

        let cost = quantity * execution_price;

        if side == "buy" {
            // Проверяем и списываем средства
            let available: f64 = tx
                .query_row("SELECT balance FROM accounts WHERE id = 1", [], |r| r.get(0))
                .map_err(|e| e.to_string())?;

            // Если был лимитный ордер, освобождаем резерв
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
                return Err("Недостаточно средств".to_string());
            }

            // Списываем стоимость
            tx.execute(
                "UPDATE accounts SET balance = balance - ?1 WHERE id = 1",
                [cost],
            )
            .map_err(|e| e.to_string())?;

            // Добавляем позицию
            tx.execute(
                "INSERT INTO positions (symbol, quantity, avg_price) VALUES (?1, ?2, ?3)
                 ON CONFLICT(symbol) DO UPDATE SET
                    avg_price = (avg_price * quantity + ?3 * ?2) / (quantity + ?2),
                    quantity = quantity + ?2",
                (&symbol, quantity, execution_price),
            )
            .map_err(|e| e.to_string())?;
        } else {
            // Продажа
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
                return Err(format!("Недостаточно {}: есть {}, нужно {}", symbol, current_qty, quantity));
            }

            // Уменьшаем позицию
            tx.execute(
                "UPDATE positions SET quantity = quantity - ?1 WHERE symbol = ?2",
                (quantity, &symbol),
            )
            .map_err(|e| e.to_string())?;

            // Удаляем пустые позиции
            tx.execute(
                "DELETE FROM positions WHERE quantity <= 0",
                [],
            )
            .map_err(|e| e.to_string())?;

            // Начисляем средства
            tx.execute(
                "UPDATE accounts SET balance = balance + ?1 WHERE id = 1",
                [cost],
            )
            .map_err(|e| e.to_string())?;
        }

        // Обновляем статус ордера
        tx.execute(
            "UPDATE orders SET status = 'filled' WHERE id = ?1",
            [order_id],
        )
        .map_err(|e| e.to_string())?;

        // Записываем сделку
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
            .map_err(|e| format!("Ордер не найден: {}", e))?;

        if status != "pending" {
            return Err(format!("Нельзя отменить ордер со статусом: {}", status));
        }

        // Освобождаем резерв для лимитных ордеров на покупку
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

    println!("=== Торговый движок с транзакциями ===\n");

    // Размещаем ордера
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
        Ok(id) => println!("Размещён ордер на покупку: #{}", id),
        Err(e) => println!("Ошибка размещения: {}", e),
    }

    let (balance, reserved) = engine.get_balance()?;
    println!("Баланс: ${:.2}, Резерв: ${:.2}\n", balance, reserved);

    // Исполняем ордер
    if let Some(order_id) = buy_order.id {
        match engine.execute_order(order_id, 41500.0) {
            Ok(()) => println!("Ордер #{} исполнен по $41,500", order_id),
            Err(e) => println!("Ошибка исполнения: {}", e),
        }
    }

    let (balance, reserved) = engine.get_balance()?;
    println!("Баланс: ${:.2}, Резерв: ${:.2}", balance, reserved);

    println!("\nПозиции:");
    for (symbol, qty, avg_price) in engine.get_positions()? {
        println!("  {}: {} @ ${:.2}", symbol, qty, avg_price);
    }

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Транзакция | Группа операций, выполняемых атомарно |
| ACID | Atomicity, Consistency, Isolation, Durability |
| `transaction()` | Начало транзакции |
| `commit()` | Фиксация изменений |
| `rollback()` | Откат изменений |
| Savepoint | Контрольная точка внутри транзакции |
| Уровни изоляции | Deferred, Immediate, Exclusive |

## Домашнее задание

1. **Атомарный перевод**: Реализуй функцию перевода средств между двумя счетами, которая:
   - Проверяет достаточность средств
   - Списывает с одного счёта
   - Зачисляет на другой
   - Записывает транзакцию в историю
   - Откатывает всё при любой ошибке

2. **Массовое исполнение ордеров**: Создай функцию, которая исполняет список ордеров в одной транзакции:
   - Если один ордер не может быть исполнен — откатываются все
   - Используй savepoints для частичного отката

3. **Торговая сессия**: Реализуй систему, которая:
   - Открывает торговую сессию (транзакция)
   - Позволяет выполнять множество операций
   - Закрывает сессию с commit или rollback
   - Поддерживает журнал всех операций в сессии

4. **Восстановление после сбоя**: Напиши программу, которая:
   - Имитирует сбой в середине транзакции
   - Демонстрирует, что данные остаются в согласованном состоянии
   - Показывает, какие операции были откачены

## Навигация

[← Предыдущий день](../221-connection-pools/ru.md) | [Следующий день →](../223-prepared-statements/ru.md)
