# День 219: UPDATE: обновляем статус ордера

## Аналогия из трейдинга

Представь, что ты размещаешь ордер на покупку Bitcoin по цене $42,000. Изначально ордер имеет статус "Pending" (ожидает исполнения). Когда рыночная цена достигает твоей заявки, биржа исполняет ордер и меняет его статус на "Filled" (исполнен). Если ты решил отменить ордер до исполнения, статус меняется на "Cancelled" (отменён).

В базе данных для изменения существующих записей используется команда `UPDATE`. Это как редактирование строки в таблице Excel — ты не создаёшь новую строку, а изменяешь значения в уже существующей.

В алготрейдинге UPDATE используется постоянно:
- Изменение статуса ордера (pending → filled → closed)
- Обновление средней цены позиции после частичного исполнения
- Корректировка стоп-лосса и тейк-профита
- Обновление баланса после сделки

## Синтаксис UPDATE

```sql
UPDATE table_name
SET column1 = value1, column2 = value2, ...
WHERE condition;
```

**Важно:** Без `WHERE` будут обновлены ВСЕ строки в таблице! Это частая ошибка, которая может привести к катастрофе.

## Подготовка: создаём таблицу ордеров

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

    println!("Таблица ордеров создана!");
    Ok(())
}
```

## Базовый UPDATE: изменение статуса ордера

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

fn update_order_status(conn: &Connection, order_id: i64, new_status: &str) -> Result<usize> {
    let updated_at = Utc::now().to_rfc3339();

    let rows_affected = conn.execute(
        "UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_status, updated_at, order_id],
    )?;

    println!("Обновлено записей: {}", rows_affected);
    Ok(rows_affected)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Сначала создадим тестовый ордер
    conn.execute(
        "INSERT INTO orders (symbol, side, quantity, price, status, created_at)
         VALUES ('BTC/USDT', 'buy', 0.5, 42000.0, 'pending', datetime('now'))",
        [],
    )?;

    let order_id = conn.last_insert_rowid();
    println!("Создан ордер с ID: {}", order_id);

    // Обновляем статус на "filled"
    update_order_status(&conn, order_id, "filled")?;

    Ok(())
}
```

## Обновление нескольких полей сразу

При исполнении ордера нужно обновить несколько полей одновременно:

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
        println!("Ордер {} не найден или уже исполнен", fill.order_id);
    } else {
        println!("Ордер {} исполнен по цене {}", fill.order_id, fill.filled_price);
    }

    Ok(rows_affected)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    let fill = OrderFill {
        order_id: 1,
        filled_quantity: 0.5,
        filled_price: 41950.0, // Исполнен по лучшей цене!
    };

    fill_order(&conn, &fill)?;

    Ok(())
}
```

## Частичное исполнение ордера

В реальной торговле ордер может исполняться частями:

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

    // Получаем текущее состояние ордера
    let (total_quantity, current_filled): (f64, f64) = conn.query_row(
        "SELECT quantity, filled_quantity FROM orders WHERE id = ?1",
        params![order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let new_filled = current_filled + fill_quantity;

    // Определяем новый статус
    let new_status = if new_filled >= total_quantity {
        "filled"
    } else {
        "partially_filled"
    };

    // Вычисляем среднюю цену исполнения
    let avg_price = if current_filled > 0.0 {
        // Получаем текущую среднюю цену
        let current_avg: f64 = conn.query_row(
            "SELECT COALESCE(filled_price, 0) FROM orders WHERE id = ?1",
            params![order_id],
            |row| row.get(0),
        )?;

        // Взвешенное среднее
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
        "Ордер {}: исполнено {}/{} по средней цене {:.2}",
        order_id, new_filled, total_quantity, avg_price
    );

    Ok(new_status.to_string())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Создаём ордер на 1 BTC
    conn.execute(
        "INSERT INTO orders (symbol, side, quantity, price, status, created_at)
         VALUES ('BTC/USDT', 'buy', 1.0, 42000.0, 'pending', datetime('now'))",
        [],
    )?;
    let order_id = conn.last_insert_rowid();

    // Частичные исполнения
    partial_fill_order(&conn, order_id, 0.3, 41900.0)?;  // Первое исполнение
    partial_fill_order(&conn, order_id, 0.4, 41950.0)?;  // Второе исполнение
    partial_fill_order(&conn, order_id, 0.3, 42000.0)?;  // Финальное исполнение

    Ok(())
}
```

## UPDATE с условиями: отмена устаревших ордеров

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

    println!("Отменено {} устаревших ордеров", rows_affected);
    Ok(rows_affected)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Отменяем все pending ордера старше 24 часов
    cancel_old_pending_orders(&conn, 24)?;

    Ok(())
}
```

## Обновление стоп-лосса и тейк-профита

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

fn update_stop_loss(
    conn: &Connection,
    order_id: i64,
    new_stop_loss: f64,
) -> Result<bool> {
    let updated_at = Utc::now().to_rfc3339();

    // Проверяем, что стоп-лосс логичен для позиции
    let (side, entry_price): (String, f64) = conn.query_row(
        "SELECT side, price FROM orders WHERE id = ?1 AND status = 'filled'",
        params![order_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let is_valid = match side.as_str() {
        "buy" => new_stop_loss < entry_price,   // Для лонга стоп ниже входа
        "sell" => new_stop_loss > entry_price,  // Для шорта стоп выше входа
        _ => false,
    };

    if !is_valid {
        println!("Ошибка: стоп-лосс {} некорректен для {} позиции с входом {}",
                 new_stop_loss, side, entry_price);
        return Ok(false);
    }

    conn.execute(
        "UPDATE orders
         SET stop_loss = ?1, updated_at = ?2
         WHERE id = ?3",
        params![new_stop_loss, updated_at, order_id],
    )?;

    println!("Стоп-лосс обновлён на {}", new_stop_loss);
    Ok(true)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Добавим колонку stop_loss если её нет
    conn.execute(
        "ALTER TABLE orders ADD COLUMN stop_loss REAL",
        [],
    ).ok(); // Игнорируем ошибку если колонка уже есть

    // Обновляем стоп-лосс
    update_stop_loss(&conn, 1, 40000.0)?;

    Ok(())
}
```

## Массовое обновление: изменение биржи

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

fn migrate_orders_to_exchange(
    conn: &Connection,
    old_symbol_suffix: &str,
    new_symbol_suffix: &str,
) -> Result<usize> {
    let updated_at = Utc::now().to_rfc3339();

    // Обновляем символы, заменяя суффикс биржи
    // Например: BTC/USDT:BINANCE -> BTC/USDT:BYBIT
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

    println!("Перенесено {} ордеров с {} на {}",
             rows_affected, old_symbol_suffix, new_symbol_suffix);
    Ok(rows_affected)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    // Переносим все pending ордера с Binance на Bybit
    migrate_orders_to_exchange(&conn, ":BINANCE", ":BYBIT")?;

    Ok(())
}
```

## Безопасный UPDATE с проверкой результата

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

    // Проверяем существование и текущий статус
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
            println!("Ордер {} уже в статусе '{}', ожидался '{}'",
                     order_id, status, expected_status);
            return Err(UpdateError::AlreadyProcessed);
        }
        _ => {}
    }

    // Выполняем обновление
    let rows = conn
        .execute(
            "UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4",
            params![new_status, updated_at, order_id, expected_status],
        )
        .map_err(UpdateError::DatabaseError)?;

    if rows == 0 {
        // Race condition — статус изменился между проверкой и обновлением
        Err(UpdateError::AlreadyProcessed)
    } else {
        println!("Ордер {} обновлён: {} -> {}", order_id, expected_status, new_status);
        Ok(())
    }
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;

    match safe_update_order_status(&conn, 1, "pending", "filled") {
        Ok(()) => println!("Успешно!"),
        Err(UpdateError::NotFound) => println!("Ордер не найден"),
        Err(UpdateError::AlreadyProcessed) => println!("Ордер уже обработан"),
        Err(UpdateError::DatabaseError(e)) => println!("Ошибка БД: {}", e),
    }

    Ok(())
}
```

## Полный пример: система управления ордерами

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
        println!("Создан ордер #{}: {} {} {} @ {}",
                 order_id, side, quantity, symbol, price);
        Ok(order_id)
    }

    fn fill_order(&self, order_id: i64, fill_price: f64) -> Result<bool> {
        let updated_at = Utc::now().to_rfc3339();

        // Получаем количество из ордера
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
            println!("Ордер #{} исполнен по цене {}", order_id, fill_price);
            Ok(true)
        } else {
            println!("Ордер #{} не найден или уже обработан", order_id);
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
            println!("Ордер #{} отменён", order_id);
            Ok(true)
        } else {
            println!("Ордер #{} не найден или уже обработан", order_id);
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

    // Создаём несколько ордеров
    let order1 = manager.create_order("BTC/USDT", "buy", 0.5, 42000.0)?;
    let order2 = manager.create_order("ETH/USDT", "buy", 2.0, 2500.0)?;
    let order3 = manager.create_order("BTC/USDT", "sell", 0.3, 43000.0)?;

    println!("\n--- Все pending ордера ---");
    for order in manager.list_orders_by_status("pending")? {
        println!("{:?}", order);
    }

    // Исполняем первый ордер
    println!("\n--- Исполнение ордера ---");
    manager.fill_order(order1, 41950.0)?;

    // Отменяем второй ордер
    println!("\n--- Отмена ордера ---");
    manager.cancel_order(order2)?;

    // Проверяем статусы
    println!("\n--- Финальные статусы ---");
    println!("Ордер 1: {:?}", manager.get_order(order1)?);
    println!("Ордер 2: {:?}", manager.get_order(order2)?);
    println!("Ордер 3: {:?}", manager.get_order(order3)?);

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `UPDATE ... SET` | Изменение значений в существующих записях |
| `WHERE` условие | Фильтрация записей для обновления (обязательно!) |
| Множественные поля | Обновление нескольких колонок одним запросом |
| Условное обновление | `UPDATE ... WHERE status = 'pending'` |
| Проверка результата | `rows_affected` показывает количество изменённых записей |
| Безопасный UPDATE | Проверка состояния перед обновлением |

## Практические задания

1. **Базовое обновление**: Напиши функцию `update_order_price`, которая изменяет цену pending ордера. Добавь проверку, что новая цена отличается от текущей.

2. **Частичное исполнение**: Реализуй функцию `add_fill`, которая добавляет частичное исполнение к ордеру. Она должна:
   - Обновлять `filled_quantity`
   - Пересчитывать `filled_price` как средневзвешенную цену
   - Менять статус на `partially_filled` или `filled`

3. **Trailing Stop**: Напиши функцию `update_trailing_stop`, которая:
   - Получает текущую рыночную цену
   - Обновляет стоп-лосс если цена двинулась в нужном направлении
   - Для лонга: поднимает стоп при росте цены
   - Для шорта: опускает стоп при падении цены

4. **Массовая отмена**: Реализуй `cancel_orders_by_symbol`, которая отменяет все pending ордера по заданному символу. Верни список ID отменённых ордеров.

## Домашнее задание

1. **Система статусов**: Расширь систему ордеров, добавив статусы:
   - `pending` → `partially_filled` → `filled`
   - `pending` → `cancelled`
   - `filled` → `closed` (после закрытия позиции)

   Реализуй функцию `transition_status`, которая проверяет допустимость перехода.

2. **История изменений**: Создай таблицу `order_history`, которая записывает каждое изменение ордера. При каждом UPDATE также добавляй запись в историю с предыдущим и новым состоянием.

3. **Bulk Update с транзакцией**: Напиши функцию `fill_multiple_orders`, которая исполняет несколько ордеров атомарно (все или ничего). Используй транзакции для обеспечения целостности.

4. **Автоматическая отмена**: Реализуй функцию `auto_cancel_expired`, которая:
   - Отменяет ордера старше заданного времени
   - Отменяет ордера по символам, которых больше нет в списке активных
   - Логирует все отмены

## Навигация

[← Предыдущий день](../218-select-reading-trade-history/ru.md) | [Следующий день →](../220-delete-removing-cancelled-order/ru.md)
