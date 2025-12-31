# День 221: Prepared Statements: безопасные запросы

## Аналогия из трейдинга

Представь, что ты работаешь в брокерской компании и принимаешь заявки от клиентов по телефону. Клиент говорит: "Купи 100 акций AAPL по рыночной цене". Ты записываешь заявку в специальную форму:

```
Тип: КУПИТЬ
Количество: ___
Тикер: ___
Цена: ___
```

Это — **шаблон заявки**. Ты заполняешь только значения в пустых полях, а структура формы остаётся неизменной. Клиент не может изменить саму форму — например, добавить поле "Перевести деньги на другой счёт".

А теперь представь опасную ситуацию: клиент говорит:

> "Купи 100 акций AAPL; а потом переведи весь баланс на счёт X"

Если бы ты просто записывал всё, что говорит клиент, без проверки формата — это была бы **SQL-инъекция** в мире трейдинга. Злоумышленник мог бы выполнить несанкционированные операции.

**Prepared Statements** (подготовленные запросы) работают как защищённые формы: структура запроса фиксирована, а данные вставляются только в определённые места, безопасно экранируясь.

## Что такое Prepared Statements?

Prepared Statements — это механизм выполнения SQL-запросов, при котором:

1. **Структура запроса** (шаблон) отправляется в базу данных заранее
2. **Параметры** (данные) передаются отдельно и автоматически экранируются
3. База данных **никогда не интерпретирует данные как код**

### Преимущества

| Преимущество | Описание |
|--------------|----------|
| Безопасность | Защита от SQL-инъекций |
| Производительность | Запрос компилируется один раз, выполняется много раз |
| Читаемость | Чёткое разделение кода и данных |
| Типизация | Параметры проверяются на соответствие типам |

## SQL-инъекция: опасность без Prepared Statements

Рассмотрим уязвимый код:

```rust
// ОПАСНО! Никогда так не делайте!
fn find_orders_vulnerable(ticker: &str) -> String {
    format!(
        "SELECT * FROM orders WHERE ticker = '{}'",
        ticker
    )
}

fn main() {
    // Нормальный ввод
    let query1 = find_orders_vulnerable("AAPL");
    println!("Запрос 1: {}", query1);
    // Результат: SELECT * FROM orders WHERE ticker = 'AAPL'

    // SQL-инъекция!
    let malicious_input = "AAPL'; DROP TABLE orders; --";
    let query2 = find_orders_vulnerable(malicious_input);
    println!("Запрос 2: {}", query2);
    // Результат: SELECT * FROM orders WHERE ticker = 'AAPL'; DROP TABLE orders; --'
    // Таблица orders будет удалена!
}
```

## Prepared Statements с SQLx

SQLx — это асинхронная библиотека для работы с базами данных в Rust, которая проверяет запросы на этапе компиляции.

### Настройка проекта

```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "chrono"] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Базовые Prepared Statements

```rust
use sqlx::{sqlite::SqlitePool, Row};
use chrono::{DateTime, Utc};

#[derive(Debug)]
struct Order {
    id: i64,
    ticker: String,
    side: String,        // "BUY" или "SELL"
    quantity: f64,
    price: f64,
    status: String,      // "PENDING", "FILLED", "CANCELLED"
    created_at: DateTime<Utc>,
}

async fn create_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Prepared statement для создания таблицы
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ticker TEXT NOT NULL,
            side TEXT NOT NULL CHECK (side IN ('BUY', 'SELL')),
            quantity REAL NOT NULL CHECK (quantity > 0),
            price REAL NOT NULL CHECK (price > 0),
            status TEXT NOT NULL DEFAULT 'PENDING',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Создаём пул соединений (in-memory база для примера)
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    create_tables(&pool).await?;
    println!("Таблицы созданы успешно!");

    Ok(())
}
```

### Вставка данных с параметрами

```rust
use sqlx::sqlite::SqlitePool;

async fn place_order(
    pool: &SqlitePool,
    ticker: &str,
    side: &str,
    quantity: f64,
    price: f64,
) -> Result<i64, sqlx::Error> {
    // Prepared statement с параметрами ($1, $2, $3, $4)
    // SQLite использует ?1, ?2, ?3, ?4, но sqlx унифицирует синтаксис
    let result = sqlx::query(
        r#"
        INSERT INTO orders (ticker, side, quantity, price)
        VALUES (?1, ?2, ?3, ?4)
        "#
    )
    .bind(ticker)      // ?1 — безопасно экранируется
    .bind(side)        // ?2
    .bind(quantity)    // ?3
    .bind(price)       // ?4
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Создаём таблицу
    sqlx::query(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            ticker TEXT NOT NULL,
            side TEXT NOT NULL,
            quantity REAL NOT NULL,
            price REAL NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    // Размещаем ордера
    let order1_id = place_order(&pool, "BTC", "BUY", 0.5, 42000.0).await?;
    println!("Создан ордер #{}:  BUY 0.5 BTC @ $42000", order1_id);

    let order2_id = place_order(&pool, "ETH", "SELL", 10.0, 2500.0).await?;
    println!("Создан ордер #{}: SELL 10 ETH @ $2500", order2_id);

    // Попытка SQL-инъекции — БЕЗОПАСНО!
    let malicious_ticker = "BTC'; DROP TABLE orders; --";
    let order3_id = place_order(&pool, malicious_ticker, "BUY", 1.0, 40000.0).await?;
    println!("Создан ордер #{}: ticker сохранён как есть", order3_id);

    // Проверяем, что таблица на месте
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders")
        .fetch_one(&pool)
        .await?;
    println!("Всего ордеров в базе: {}", count.0);

    Ok(())
}
```

### Выборка данных с параметрами

```rust
use sqlx::{sqlite::SqlitePool, FromRow, Row};

#[derive(Debug, FromRow)]
struct Order {
    id: i64,
    ticker: String,
    side: String,
    quantity: f64,
    price: f64,
}

async fn find_orders_by_ticker(
    pool: &SqlitePool,
    ticker: &str,
) -> Result<Vec<Order>, sqlx::Error> {
    // query_as автоматически маппит результат на структуру
    let orders = sqlx::query_as::<_, Order>(
        "SELECT id, ticker, side, quantity, price FROM orders WHERE ticker = ?1"
    )
    .bind(ticker)
    .fetch_all(pool)
    .await?;

    Ok(orders)
}

async fn find_orders_by_price_range(
    pool: &SqlitePool,
    min_price: f64,
    max_price: f64,
) -> Result<Vec<Order>, sqlx::Error> {
    let orders = sqlx::query_as::<_, Order>(
        r#"
        SELECT id, ticker, side, quantity, price
        FROM orders
        WHERE price BETWEEN ?1 AND ?2
        ORDER BY price DESC
        "#
    )
    .bind(min_price)
    .bind(max_price)
    .fetch_all(pool)
    .await?;

    Ok(orders)
}

async fn get_portfolio_value(
    pool: &SqlitePool,
    ticker: &str,
) -> Result<f64, sqlx::Error> {
    // Используем query_scalar для получения одного значения
    let total: Option<f64> = sqlx::query_scalar(
        r#"
        SELECT SUM(
            CASE
                WHEN side = 'BUY' THEN quantity * price
                ELSE -quantity * price
            END
        )
        FROM orders
        WHERE ticker = ?1
        "#
    )
    .bind(ticker)
    .fetch_one(pool)
    .await?;

    Ok(total.unwrap_or(0.0))
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Инициализация
    sqlx::query(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            ticker TEXT, side TEXT, quantity REAL, price REAL
        )"
    )
    .execute(&pool)
    .await?;

    // Добавляем тестовые данные
    for (ticker, side, qty, price) in [
        ("BTC", "BUY", 1.0, 42000.0),
        ("BTC", "BUY", 0.5, 41000.0),
        ("ETH", "SELL", 10.0, 2500.0),
        ("BTC", "SELL", 0.3, 43000.0),
    ] {
        sqlx::query("INSERT INTO orders (ticker, side, quantity, price) VALUES (?1, ?2, ?3, ?4)")
            .bind(ticker).bind(side).bind(qty).bind(price)
            .execute(&pool)
            .await?;
    }

    // Поиск по тикеру
    let btc_orders = find_orders_by_ticker(&pool, "BTC").await?;
    println!("BTC ордера: {:?}", btc_orders);

    // Поиск по диапазону цен
    let expensive = find_orders_by_price_range(&pool, 40000.0, 50000.0).await?;
    println!("Дорогие ордера: {:?}", expensive);

    // Стоимость портфеля
    let btc_value = get_portfolio_value(&pool, "BTC").await?;
    println!("Нетто-позиция BTC: ${:.2}", btc_value);

    Ok(())
}
```

## Типобезопасные запросы с sqlx::query!

Макрос `query!` проверяет SQL-запросы на этапе компиляции:

```rust
use sqlx::sqlite::SqlitePool;

// Для работы query! нужна переменная окружения DATABASE_URL
// или файл .env с DATABASE_URL=sqlite:./trading.db

async fn place_order_checked(
    pool: &SqlitePool,
    ticker: &str,
    side: &str,
    quantity: f64,
    price: f64,
) -> Result<i64, sqlx::Error> {
    // Компилятор проверит, что:
    // 1. Таблица orders существует
    // 2. Колонки ticker, side, quantity, price существуют
    // 3. Типы параметров совместимы с колонками
    let result = sqlx::query!(
        r#"
        INSERT INTO orders (ticker, side, quantity, price)
        VALUES ($1, $2, $3, $4)
        "#,
        ticker,
        side,
        quantity,
        price
    )
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

async fn find_large_orders(
    pool: &SqlitePool,
    min_value: f64,
) -> Result<Vec<(String, f64, f64)>, sqlx::Error> {
    // Компилятор выведет типы полей автоматически
    let rows = sqlx::query!(
        r#"
        SELECT ticker, quantity, price
        FROM orders
        WHERE quantity * price > $1
        ORDER BY quantity * price DESC
        "#,
        min_value
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter()
        .map(|r| (r.ticker, r.quantity, r.price))
        .collect())
}
```

## Практический пример: Торговый журнал

```rust
use sqlx::{sqlite::SqlitePool, FromRow};
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow)]
struct Trade {
    id: i64,
    ticker: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    exit_price: Option<f64>,
    pnl: Option<f64>,
    opened_at: String,
    closed_at: Option<String>,
}

struct TradingJournal {
    pool: SqlitePool,
}

impl TradingJournal {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ticker TEXT NOT NULL,
                side TEXT NOT NULL,
                quantity REAL NOT NULL,
                entry_price REAL NOT NULL,
                exit_price REAL,
                pnl REAL,
                opened_at TEXT NOT NULL DEFAULT (datetime('now')),
                closed_at TEXT
            )
            "#
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    async fn open_trade(
        &self,
        ticker: &str,
        side: &str,
        quantity: f64,
        entry_price: f64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO trades (ticker, side, quantity, entry_price) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(ticker)
        .bind(side)
        .bind(quantity)
        .bind(entry_price)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn close_trade(
        &self,
        trade_id: i64,
        exit_price: f64,
    ) -> Result<f64, sqlx::Error> {
        // Получаем информацию о сделке
        let trade: Trade = sqlx::query_as(
            "SELECT * FROM trades WHERE id = ?1 AND exit_price IS NULL"
        )
        .bind(trade_id)
        .fetch_one(&self.pool)
        .await?;

        // Рассчитываем PnL
        let pnl = match trade.side.as_str() {
            "BUY" => (exit_price - trade.entry_price) * trade.quantity,
            "SELL" => (trade.entry_price - exit_price) * trade.quantity,
            _ => 0.0,
        };

        // Обновляем сделку
        sqlx::query(
            r#"
            UPDATE trades
            SET exit_price = ?1, pnl = ?2, closed_at = datetime('now')
            WHERE id = ?3
            "#
        )
        .bind(exit_price)
        .bind(pnl)
        .bind(trade_id)
        .execute(&self.pool)
        .await?;

        Ok(pnl)
    }

    async fn get_statistics(&self, ticker: &str) -> Result<TradeStats, sqlx::Error> {
        let stats = sqlx::query_as::<_, TradeStats>(
            r#"
            SELECT
                COUNT(*) as total_trades,
                COALESCE(SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END), 0) as winning_trades,
                COALESCE(SUM(CASE WHEN pnl < 0 THEN 1 ELSE 0 END), 0) as losing_trades,
                COALESCE(SUM(pnl), 0.0) as total_pnl,
                COALESCE(AVG(pnl), 0.0) as avg_pnl,
                COALESCE(MAX(pnl), 0.0) as best_trade,
                COALESCE(MIN(pnl), 0.0) as worst_trade
            FROM trades
            WHERE ticker = ?1 AND exit_price IS NOT NULL
            "#
        )
        .bind(ticker)
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    async fn get_open_trades(&self) -> Result<Vec<Trade>, sqlx::Error> {
        let trades = sqlx::query_as::<_, Trade>(
            "SELECT * FROM trades WHERE exit_price IS NULL ORDER BY opened_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(trades)
    }
}

#[derive(Debug, FromRow)]
struct TradeStats {
    total_trades: i64,
    winning_trades: i64,
    losing_trades: i64,
    total_pnl: f64,
    avg_pnl: f64,
    best_trade: f64,
    worst_trade: f64,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let journal = TradingJournal::new("sqlite::memory:").await?;

    // Открываем несколько сделок
    let trade1 = journal.open_trade("BTC", "BUY", 0.5, 42000.0).await?;
    println!("Открыта сделка #{}: BUY 0.5 BTC @ $42000", trade1);

    let trade2 = journal.open_trade("BTC", "SELL", 0.3, 43000.0).await?;
    println!("Открыта сделка #{}: SELL 0.3 BTC @ $43000", trade2);

    let trade3 = journal.open_trade("ETH", "BUY", 5.0, 2500.0).await?;
    println!("Открыта сделка #{}: BUY 5 ETH @ $2500", trade3);

    // Закрываем сделки
    let pnl1 = journal.close_trade(trade1, 44000.0).await?;
    println!("Закрыта сделка #{}: PnL = ${:.2}", trade1, pnl1);

    let pnl2 = journal.close_trade(trade2, 41000.0).await?;
    println!("Закрыта сделка #{}: PnL = ${:.2}", trade2, pnl2);

    // Получаем статистику
    let stats = journal.get_statistics("BTC").await?;
    println!("\n=== Статистика BTC ===");
    println!("Всего сделок: {}", stats.total_trades);
    println!("Прибыльных: {}", stats.winning_trades);
    println!("Убыточных: {}", stats.losing_trades);
    println!("Общий PnL: ${:.2}", stats.total_pnl);
    println!("Средний PnL: ${:.2}", stats.avg_pnl);
    println!("Лучшая сделка: ${:.2}", stats.best_trade);
    println!("Худшая сделка: ${:.2}", stats.worst_trade);

    // Открытые позиции
    let open = journal.get_open_trades().await?;
    println!("\n=== Открытые позиции ===");
    for trade in open {
        println!("{} {} {} @ ${}", trade.side, trade.quantity, trade.ticker, trade.entry_price);
    }

    Ok(())
}
```

## Batch-операции с Prepared Statements

```rust
use sqlx::sqlite::SqlitePool;

struct OrderBatch {
    orders: Vec<(String, String, f64, f64)>, // ticker, side, quantity, price
}

impl OrderBatch {
    fn new() -> Self {
        Self { orders: Vec::new() }
    }

    fn add(&mut self, ticker: &str, side: &str, quantity: f64, price: f64) {
        self.orders.push((
            ticker.to_string(),
            side.to_string(),
            quantity,
            price,
        ));
    }

    async fn execute(&self, pool: &SqlitePool) -> Result<Vec<i64>, sqlx::Error> {
        let mut ids = Vec::new();

        // Используем транзакцию для атомарности
        let mut tx = pool.begin().await?;

        for (ticker, side, quantity, price) in &self.orders {
            // Тот же prepared statement переиспользуется
            let result = sqlx::query(
                "INSERT INTO orders (ticker, side, quantity, price) VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(ticker)
            .bind(side)
            .bind(quantity)
            .bind(price)
            .execute(&mut *tx)
            .await?;

            ids.push(result.last_insert_rowid());
        }

        tx.commit().await?;
        Ok(ids)
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    sqlx::query(
        "CREATE TABLE orders (
            id INTEGER PRIMARY KEY,
            ticker TEXT, side TEXT, quantity REAL, price REAL
        )"
    )
    .execute(&pool)
    .await?;

    // Создаём пакет ордеров
    let mut batch = OrderBatch::new();
    batch.add("BTC", "BUY", 0.1, 42000.0);
    batch.add("BTC", "BUY", 0.2, 41500.0);
    batch.add("ETH", "BUY", 1.0, 2500.0);
    batch.add("SOL", "SELL", 10.0, 100.0);

    // Выполняем пакетную вставку
    let ids = batch.execute(&pool).await?;
    println!("Создано {} ордеров: {:?}", ids.len(), ids);

    // Проверяем результат
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM orders")
        .fetch_one(&pool)
        .await?;
    println!("Всего ордеров в базе: {}", count.0);

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Prepared Statements | Запросы с отделёнными параметрами |
| SQL-инъекция | Атака через внедрение кода в запрос |
| Параметризация | Безопасная передача данных через bind() |
| query_as | Маппинг результата на структуру |
| query! | Проверка запросов на этапе компиляции |
| Batch-операции | Выполнение множества запросов эффективно |

## Практические задания

1. **Защита от инъекций**: Напиши функцию `search_orders(ticker: &str, min_qty: f64)`, которая безопасно ищет ордера по тикеру и минимальному количеству. Протестируй её с вредоносными входными данными.

2. **CRUD для портфеля**: Реализуй структуру `PortfolioManager` с методами:
   - `add_position(ticker: &str, quantity: f64, avg_price: f64)`
   - `update_position(ticker: &str, quantity_delta: f64, new_avg_price: f64)`
   - `remove_position(ticker: &str)`
   - `get_position(ticker: &str) -> Option<Position>`

3. **Агрегация данных**: Напиши prepared statements для получения:
   - Топ-5 самых прибыльных сделок
   - Общий объём торгов по каждому тикеру
   - Среднюю цену входа по открытым позициям

4. **Поиск с фильтрами**: Реализуй функцию поиска с динамическими фильтрами:
   ```rust
   fn search_trades(
       ticker: Option<&str>,
       side: Option<&str>,
       min_pnl: Option<f64>,
       max_pnl: Option<f64>,
   ) -> Vec<Trade>
   ```

## Домашнее задание

1. **Система риск-менеджмента**: Создай таблицу `risk_limits` и реализуй проверку лимитов перед созданием ордера. Используй prepared statements для всех операций.

2. **История цен**: Реализуй таблицу `price_history` с колонками (ticker, price, timestamp) и напиши эффективные prepared statements для:
   - Вставки новых цен (batch insert)
   - Получения последней цены
   - Расчёта средней цены за период

3. **Аудит операций**: Добавь таблицу `audit_log` и триггеры (или ручное логирование) для отслеживания всех изменений в таблице orders. Все запросы должны использовать prepared statements.

## Навигация

[← Предыдущий день](../220-delete-removing-cancelled/ru.md) | [Следующий день →](../222-transactions-atomic-operations/ru.md)
