# День 217: INSERT: записываем сделку

## Аналогия из трейдинга

Каждая сделка на бирже — это история. Когда ты покупаешь или продаёшь актив, эта информация должна быть записана в базу данных: цена, объём, время, направление. Без записи сделок невозможно вести учёт прибыли и убытков, анализировать историю торговли или проводить аудит.

Представь, что у тебя торговый терминал, и ты только что исполнил ордер на покупку 0.5 BTC по цене $42,000. Эта сделка должна быть немедленно записана в базу данных:

```
Сделка #12345
├── Символ: BTC/USD
├── Направление: BUY
├── Количество: 0.5
├── Цена: 42000.00
├── Комиссия: 21.00
└── Время: 2024-01-15 10:30:45
```

SQL оператор `INSERT` — это ваш инструмент для записи каждой сделки в базу данных.

## Что такое INSERT?

`INSERT` — это SQL команда для добавления новых записей (строк) в таблицу базы данных. В контексте трейдинга мы используем INSERT для:

- Записи исполненных сделок
- Сохранения истории ордеров
- Логирования изменений портфеля
- Фиксации котировок

### Базовый синтаксис SQL

```sql
-- Вставка одной записи
INSERT INTO trades (symbol, side, quantity, price, timestamp)
VALUES ('BTC/USD', 'BUY', 0.5, 42000.00, '2024-01-15 10:30:45');

-- Вставка нескольких записей
INSERT INTO trades (symbol, side, quantity, price, timestamp)
VALUES
    ('BTC/USD', 'BUY', 0.5, 42000.00, '2024-01-15 10:30:45'),
    ('ETH/USD', 'SELL', 2.0, 2500.00, '2024-01-15 10:31:12'),
    ('BTC/USD', 'SELL', 0.25, 42100.00, '2024-01-15 10:32:00');
```

## Работа с базами данных в Rust

В Rust есть несколько популярных библиотек для работы с базами данных:

| Библиотека | Тип | Описание |
|------------|-----|----------|
| `rusqlite` | SQLite | Лёгкая встроенная БД |
| `sqlx` | Async | Асинхронный, compile-time проверка |
| `diesel` | ORM | Типобезопасный ORM |
| `tokio-postgres` | PostgreSQL | Асинхронный PostgreSQL |

Мы начнём с `rusqlite` — простой и понятной библиотеки для SQLite.

## Подготовка проекта

```toml
# Cargo.toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Создание таблицы для сделок

```rust
use rusqlite::{Connection, Result};

fn create_trades_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL,
            side TEXT NOT NULL CHECK(side IN ('BUY', 'SELL')),
            quantity REAL NOT NULL CHECK(quantity > 0),
            price REAL NOT NULL CHECK(price > 0),
            fee REAL DEFAULT 0,
            timestamp TEXT NOT NULL,
            strategy TEXT,
            notes TEXT
        )",
        [],
    )?;

    println!("Таблица trades создана успешно");
    Ok(())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;
    create_trades_table(&conn)?;
    Ok(())
}
```

## Простая вставка сделки

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;

#[derive(Debug)]
struct Trade {
    symbol: String,
    side: String,
    quantity: f64,
    price: f64,
    fee: f64,
    timestamp: String,
    strategy: Option<String>,
}

fn insert_trade(conn: &Connection, trade: &Trade) -> Result<i64> {
    conn.execute(
        "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            trade.symbol,
            trade.side,
            trade.quantity,
            trade.price,
            trade.fee,
            trade.timestamp,
            trade.strategy,
        ],
    )?;

    // Возвращаем ID вставленной записи
    Ok(conn.last_insert_rowid())
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;
    create_trades_table(&conn)?;

    let trade = Trade {
        symbol: "BTC/USD".to_string(),
        side: "BUY".to_string(),
        quantity: 0.5,
        price: 42000.0,
        fee: 21.0,
        timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        strategy: Some("momentum".to_string()),
    };

    let trade_id = insert_trade(&conn, &trade)?;
    println!("Сделка записана с ID: {}", trade_id);

    Ok(())
}
```

## Batch вставка нескольких сделок

При активной торговле часто нужно записывать множество сделок одновременно. Batch вставка значительно эффективнее, чем вставка по одной записи.

```rust
use rusqlite::{Connection, Result, params};

fn insert_trades_batch(conn: &Connection, trades: &[Trade]) -> Result<Vec<i64>> {
    let mut ids = Vec::new();

    // Используем транзакцию для атомарности и производительности
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )?;

        for trade in trades {
            stmt.execute(params![
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.price,
                trade.fee,
                trade.timestamp,
                trade.strategy,
            ])?;
            ids.push(tx.last_insert_rowid());
        }
    }

    tx.commit()?;

    println!("Записано {} сделок", ids.len());
    Ok(ids)
}

fn main() -> Result<()> {
    let conn = Connection::open("trading.db")?;
    create_trades_table(&conn)?;

    let trades = vec![
        Trade {
            symbol: "BTC/USD".to_string(),
            side: "BUY".to_string(),
            quantity: 0.1,
            price: 42000.0,
            fee: 4.2,
            timestamp: "2024-01-15 10:30:00".to_string(),
            strategy: Some("scalping".to_string()),
        },
        Trade {
            symbol: "ETH/USD".to_string(),
            side: "SELL".to_string(),
            quantity: 2.0,
            price: 2500.0,
            fee: 5.0,
            timestamp: "2024-01-15 10:30:05".to_string(),
            strategy: Some("scalping".to_string()),
        },
        Trade {
            symbol: "BTC/USD".to_string(),
            side: "SELL".to_string(),
            quantity: 0.1,
            price: 42050.0,
            fee: 4.21,
            timestamp: "2024-01-15 10:30:10".to_string(),
            strategy: Some("scalping".to_string()),
        },
    ];

    let ids = insert_trades_batch(&conn, &trades)?;
    println!("ID сделок: {:?}", ids);

    Ok(())
}
```

## Торговый движок с записью сделок

Давайте создадим более реалистичный пример — торговый движок, который исполняет ордера и автоматически записывает сделки:

```rust
use rusqlite::{Connection, Result, params};
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    fn as_str(&self) -> &str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: f64,
}

struct TradingEngine {
    conn: Connection,
    balances: HashMap<String, f64>,
    positions: HashMap<String, f64>,
    fee_rate: f64,
}

impl TradingEngine {
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Создаём таблицы
        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL NOT NULL,
                fee REAL NOT NULL,
                total REAL NOT NULL,
                timestamp TEXT NOT NULL,
                pnl REAL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS balance_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                currency TEXT NOT NULL,
                amount REAL NOT NULL,
                change REAL NOT NULL,
                reason TEXT,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        let mut balances = HashMap::new();
        balances.insert("USD".to_string(), 100_000.0);

        Ok(TradingEngine {
            conn,
            balances,
            positions: HashMap::new(),
            fee_rate: 0.001, // 0.1% комиссия
        })
    }

    fn execute_order(&mut self, order: &Order) -> Result<i64> {
        let fee = order.quantity * order.price * self.fee_rate;
        let total = order.quantity * order.price;

        // Проверяем баланс
        match order.side {
            OrderSide::Buy => {
                let usd_balance = self.balances.get("USD").unwrap_or(&0.0);
                if *usd_balance < total + fee {
                    return Err(rusqlite::Error::InvalidQuery);
                }
            }
            OrderSide::Sell => {
                let position = self.positions.get(&order.symbol).unwrap_or(&0.0);
                if *position < order.quantity {
                    return Err(rusqlite::Error::InvalidQuery);
                }
            }
        }

        // Вычисляем PnL для продажи
        let pnl: Option<f64> = match order.side {
            OrderSide::Sell => {
                // Упрощённый расчёт PnL
                Some(0.0) // В реальности нужно учитывать среднюю цену входа
            }
            OrderSide::Buy => None,
        };

        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        // Записываем сделку в БД
        self.conn.execute(
            "INSERT INTO trades (symbol, side, quantity, price, fee, total, timestamp, pnl)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                order.symbol,
                order.side.as_str(),
                order.quantity,
                order.price,
                fee,
                total,
                timestamp,
                pnl,
            ],
        )?;

        let trade_id = self.conn.last_insert_rowid();

        // Обновляем балансы и позиции
        match order.side {
            OrderSide::Buy => {
                *self.balances.get_mut("USD").unwrap() -= total + fee;
                *self.positions.entry(order.symbol.clone()).or_insert(0.0) += order.quantity;
            }
            OrderSide::Sell => {
                *self.balances.get_mut("USD").unwrap() += total - fee;
                *self.positions.get_mut(&order.symbol).unwrap() -= order.quantity;
            }
        }

        // Логируем изменение баланса
        let usd_balance = *self.balances.get("USD").unwrap();
        let change = match order.side {
            OrderSide::Buy => -(total + fee),
            OrderSide::Sell => total - fee,
        };

        self.conn.execute(
            "INSERT INTO balance_log (currency, amount, change, reason, timestamp)
             VALUES ('USD', ?1, ?2, ?3, ?4)",
            params![
                usd_balance,
                change,
                format!("Trade #{}", trade_id),
                timestamp,
            ],
        )?;

        println!(
            "Сделка #{}: {} {} {} @ ${:.2} (комиссия: ${:.2})",
            trade_id,
            order.side.as_str(),
            order.quantity,
            order.symbol,
            order.price,
            fee
        );

        Ok(trade_id)
    }

    fn get_balance(&self, currency: &str) -> f64 {
        *self.balances.get(currency).unwrap_or(&0.0)
    }

    fn get_position(&self, symbol: &str) -> f64 {
        *self.positions.get(symbol).unwrap_or(&0.0)
    }
}

fn main() -> Result<()> {
    let mut engine = TradingEngine::new("trading_engine.db")?;

    println!("Начальный баланс: ${:.2}", engine.get_balance("USD"));

    // Серия сделок
    let orders = vec![
        Order {
            symbol: "BTC/USD".to_string(),
            side: OrderSide::Buy,
            quantity: 0.5,
            price: 42000.0,
        },
        Order {
            symbol: "ETH/USD".to_string(),
            side: OrderSide::Buy,
            quantity: 10.0,
            price: 2500.0,
        },
        Order {
            symbol: "BTC/USD".to_string(),
            side: OrderSide::Buy,
            quantity: 0.3,
            price: 41800.0,
        },
        Order {
            symbol: "BTC/USD".to_string(),
            side: OrderSide::Sell,
            quantity: 0.2,
            price: 42500.0,
        },
    ];

    for order in &orders {
        match engine.execute_order(order) {
            Ok(_) => {}
            Err(e) => println!("Ошибка исполнения ордера: {:?}", e),
        }
    }

    println!("\nИтоговый баланс USD: ${:.2}", engine.get_balance("USD"));
    println!("Позиция BTC: {:.4}", engine.get_position("BTC/USD"));
    println!("Позиция ETH: {:.4}", engine.get_position("ETH/USD"));

    Ok(())
}
```

## Обработка ошибок при INSERT

```rust
use rusqlite::{Connection, Result, Error};

fn insert_trade_safe(conn: &Connection, trade: &Trade) -> Result<i64> {
    // Валидация данных перед вставкой
    if trade.quantity <= 0.0 {
        return Err(Error::InvalidParameterName(
            "Количество должно быть положительным".to_string()
        ));
    }

    if trade.price <= 0.0 {
        return Err(Error::InvalidParameterName(
            "Цена должна быть положительной".to_string()
        ));
    }

    if trade.side != "BUY" && trade.side != "SELL" {
        return Err(Error::InvalidParameterName(
            "Направление должно быть BUY или SELL".to_string()
        ));
    }

    // Пробуем вставить
    match conn.execute(
        "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            trade.symbol,
            trade.side,
            trade.quantity,
            trade.price,
            trade.fee,
            trade.timestamp,
            trade.strategy,
        ],
    ) {
        Ok(_) => Ok(conn.last_insert_rowid()),
        Err(Error::SqliteFailure(err, msg)) => {
            eprintln!("Ошибка SQLite: {:?} - {:?}", err, msg);
            Err(Error::SqliteFailure(err, msg))
        }
        Err(e) => {
            eprintln!("Неизвестная ошибка: {:?}", e);
            Err(e)
        }
    }
}
```

## INSERT с возвратом данных (RETURNING)

SQLite 3.35+ поддерживает `RETURNING` для получения вставленных данных:

```rust
use rusqlite::{Connection, Result, params};

fn insert_trade_returning(conn: &Connection, trade: &Trade) -> Result<(i64, String)> {
    let mut stmt = conn.prepare(
        "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         RETURNING id, timestamp"
    )?;

    let (id, ts): (i64, String) = stmt.query_row(
        params![
            trade.symbol,
            trade.side,
            trade.quantity,
            trade.price,
            trade.fee,
            trade.timestamp,
            trade.strategy,
        ],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    println!("Вставлена сделка ID={} в {}", id, ts);
    Ok((id, ts))
}
```

## Производительность INSERT операций

| Метод | Скорость | Использование |
|-------|----------|---------------|
| Одиночный INSERT | Медленно | Редкие операции |
| Batch в транзакции | Быстро | Множество записей |
| Prepared Statement | Очень быстро | Повторяющиеся операции |
| Bulk INSERT (VALUES) | Максимум | Импорт данных |

### Пример оптимизированной массовой вставки

```rust
use rusqlite::{Connection, Result};

fn bulk_insert_trades(conn: &Connection, trades: &[Trade]) -> Result<()> {
    // Отключаем синхронизацию для максимальной скорости (осторожно!)
    conn.execute("PRAGMA synchronous = OFF", [])?;
    conn.execute("PRAGMA journal_mode = MEMORY", [])?;

    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO trades (symbol, side, quantity, price, fee, timestamp, strategy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )?;

        for trade in trades {
            stmt.execute(params![
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.price,
                trade.fee,
                trade.timestamp,
                trade.strategy,
            ])?;
        }
    }

    tx.commit()?;

    // Восстанавливаем безопасные настройки
    conn.execute("PRAGMA synchronous = FULL", [])?;
    conn.execute("PRAGMA journal_mode = DELETE", [])?;

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| INSERT | SQL команда для добавления записей |
| rusqlite | Rust библиотека для SQLite |
| params![] | Макрос для параметризованных запросов |
| Transaction | Группировка операций для атомарности |
| Prepared Statement | Предкомпилированный запрос для скорости |
| RETURNING | Возврат данных после вставки |
| last_insert_rowid() | Получение ID вставленной записи |

## Практические задания

1. **Журнал котировок**: Создай таблицу `quotes` и функцию для записи котировок (bid, ask, spread, timestamp). Реализуй batch вставку для записи 1000 котировок за секунду.

2. **История ордеров**: Создай систему записи ордеров с полями: id, symbol, side, type (LIMIT/MARKET), quantity, price, status, created_at, filled_at. Реализуй UPDATE статуса при исполнении.

3. **Аудит изменений**: Создай таблицу аудита, которая автоматически записывает все изменения в таблице trades с помощью триггера SQLite.

## Домашнее задание

1. **Торговый журнал**: Создай полноценный торговый журнал с функциями:
   - Запись сделок с расчётом комиссии
   - Запись депозитов и выводов
   - Подсчёт общего PnL
   - Экспорт в CSV

2. **Оптимизация производительности**: Измерь время вставки 10,000 сделок разными методами:
   - Одиночные INSERT
   - Batch в транзакции
   - С разными PRAGMA настройками

   Сравни результаты и сделай выводы.

3. **Многопоточная запись**: Создай систему с несколькими потоками, которые одновременно записывают сделки. Используй пул соединений или правильную синхронизацию.

4. **Миграции БД**: Реализуй систему миграций для эволюции схемы таблицы trades:
   - Версия 1: базовые поля
   - Версия 2: добавить поле exchange
   - Версия 3: добавить поле order_id

## Навигация

[← Предыдущий день](../216-select-query-trades/ru.md) | [Следующий день →](../218-update-modify-order/ru.md)
