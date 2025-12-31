# День 227: sqlx: Проверка запросов на этапе компиляции

## Аналогия из трейдинга

Представь, что ты работаешь в инвестиционном банке, где каждая торговая заявка проходит через систему риск-менеджмента **до** того, как попадёт на биржу. Если заявка некорректна — неверный тикер, неправильный формат цены, несуществующий счёт — система отклонит её **мгновенно**, ещё до отправки на рынок.

Это именно то, что делает **sqlx** для SQL-запросов в Rust. Вместо того чтобы обнаружить ошибку в продакшене (когда запрос не находит колонку или возвращает неожиданный тип), sqlx проверяет все запросы **на этапе компиляции**:

- Существует ли таблица `trades`?
- Есть ли колонка `price` с типом `DECIMAL`?
- Совместим ли тип результата с твоей Rust-структурой?

Если что-то не так — код просто **не скомпилируется**. Это как иметь риск-менеджера, который проверяет каждую заявку до того, как ты нажмёшь кнопку "Отправить".

## Что такое sqlx?

**sqlx** — это асинхронный SQL-крейт для Rust со следующими особенностями:

1. **Проверка запросов на этапе компиляции** — sqlx подключается к базе данных во время компиляции и проверяет корректность запросов
2. **Чистый Rust** — без ORM, без магии, только SQL
3. **Асинхронность из коробки** — работает с tokio, async-std
4. **Поддержка PostgreSQL, MySQL, SQLite**

```rust
// Cargo.toml
// [dependencies]
// sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "macros", "chrono", "rust_decimal"] }
// tokio = { version = "1", features = ["full"] }
// chrono = { version = "0.4", features = ["serde"] }
// rust_decimal = { version = "1", features = ["db-postgres"] }
```

## Макрос query! — проверка на этапе компиляции

```rust
use sqlx::{PgPool, FromRow};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

// Эта структура будет заполняться данными из БД
#[derive(Debug, FromRow)]
struct Trade {
    id: i64,
    symbol: String,
    side: String,        // "BUY" или "SELL"
    price: Decimal,
    quantity: Decimal,
    executed_at: DateTime<Utc>,
}

async fn get_recent_trades(pool: &PgPool, symbol: &str) -> Result<Vec<Trade>, sqlx::Error> {
    // Макрос query_as! проверяет запрос во время компиляции!
    // Если таблица trades не существует или колонки не совпадают —
    // код НЕ скомпилируется
    let trades = sqlx::query_as!(
        Trade,
        r#"
        SELECT id, symbol, side, price, quantity, executed_at
        FROM trades
        WHERE symbol = $1
        ORDER BY executed_at DESC
        LIMIT 100
        "#,
        symbol
    )
    .fetch_all(pool)
    .await?;

    Ok(trades)
}
```

### Как это работает?

1. Во время компиляции sqlx читает переменную окружения `DATABASE_URL`
2. Подключается к базе данных
3. Выполняет `EXPLAIN` для каждого запроса
4. Проверяет типы колонок и сопоставляет их с Rust-типами
5. Если что-то не так — ошибка компиляции!

```bash
# Устанавливаем DATABASE_URL для проверки запросов
export DATABASE_URL="postgres://user:pass@localhost:5432/trading_db"

# Компилируем проект
cargo build
```

## Пример: Система учёта сделок

```rust
use sqlx::{PgPool, postgres::PgPoolOptions, FromRow};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, FromRow)]
struct Position {
    symbol: String,
    quantity: Decimal,
    avg_price: Decimal,
    unrealized_pnl: Option<Decimal>,
}

#[derive(Debug, FromRow)]
struct OrderRecord {
    id: i64,
    symbol: String,
    order_type: String,
    side: String,
    price: Option<Decimal>,
    quantity: Decimal,
    status: String,
    created_at: DateTime<Utc>,
}

struct TradingDatabase {
    pool: PgPool,
}

impl TradingDatabase {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    // Все запросы проверяются на этапе компиляции!

    async fn insert_order(
        &self,
        symbol: &str,
        order_type: &str,
        side: &str,
        price: Option<Decimal>,
        quantity: Decimal,
    ) -> Result<i64, sqlx::Error> {
        // query_scalar! возвращает единственное значение
        let order_id = sqlx::query_scalar!(
            r#"
            INSERT INTO orders (symbol, order_type, side, price, quantity, status, created_at)
            VALUES ($1, $2, $3, $4, $5, 'NEW', NOW())
            RETURNING id
            "#,
            symbol,
            order_type,
            side,
            price,
            quantity
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(order_id)
    }

    async fn get_open_positions(&self) -> Result<Vec<Position>, sqlx::Error> {
        let positions = sqlx::query_as!(
            Position,
            r#"
            SELECT
                symbol,
                quantity,
                avg_price,
                unrealized_pnl
            FROM positions
            WHERE quantity != 0
            ORDER BY symbol
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(positions)
    }

    async fn get_orders_by_status(&self, status: &str) -> Result<Vec<OrderRecord>, sqlx::Error> {
        let orders = sqlx::query_as!(
            OrderRecord,
            r#"
            SELECT id, symbol, order_type, side, price, quantity, status, created_at
            FROM orders
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
            status
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(orders)
    }

    async fn update_order_status(
        &self,
        order_id: i64,
        new_status: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE orders
            SET status = $2, updated_at = NOW()
            WHERE id = $1 AND status != 'FILLED' AND status != 'CANCELLED'
            "#,
            order_id,
            new_status
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn get_daily_pnl(&self, date: chrono::NaiveDate) -> Result<Option<Decimal>, sqlx::Error> {
        let pnl = sqlx::query_scalar!(
            r#"
            SELECT SUM(realized_pnl) as "total_pnl"
            FROM trades
            WHERE DATE(executed_at) = $1
            "#,
            date
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(pnl)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // DATABASE_URL должен быть установлен для компиляции
    let db = TradingDatabase::new(&std::env::var("DATABASE_URL")?).await?;

    // Размещаем лимитный ордер
    let order_id = db.insert_order(
        "BTC/USDT",
        "LIMIT",
        "BUY",
        Some(Decimal::new(4200000, 2)), // 42000.00
        Decimal::new(1, 1),             // 0.1 BTC
    ).await?;

    println!("Создан ордер #{}", order_id);

    // Получаем открытые позиции
    let positions = db.get_open_positions().await?;
    for pos in positions {
        println!("{}: {} @ {} (PnL: {:?})",
            pos.symbol,
            pos.quantity,
            pos.avg_price,
            pos.unrealized_pnl
        );
    }

    // Получаем активные ордера
    let pending_orders = db.get_orders_by_status("NEW").await?;
    println!("Активных ордеров: {}", pending_orders.len());

    Ok(())
}
```

## Работа с транзакциями

Транзакции критически важны в трейдинге — исполнение ордера должно быть атомарным:

```rust
use sqlx::{PgPool, Transaction, Postgres};
use rust_decimal::Decimal;

struct TradeExecution {
    order_id: i64,
    executed_price: Decimal,
    executed_quantity: Decimal,
}

async fn execute_trade(
    pool: &PgPool,
    execution: TradeExecution,
) -> Result<(), sqlx::Error> {
    // Начинаем транзакцию
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    // 1. Получаем информацию об ордере
    let order = sqlx::query!(
        r#"
        SELECT symbol, side, quantity
        FROM orders
        WHERE id = $1 AND status = 'NEW'
        FOR UPDATE  -- Блокируем строку
        "#,
        execution.order_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| sqlx::Error::RowNotFound)?;

    // 2. Обновляем статус ордера
    sqlx::query!(
        r#"
        UPDATE orders
        SET status = 'FILLED',
            filled_quantity = $2,
            filled_price = $3,
            updated_at = NOW()
        WHERE id = $1
        "#,
        execution.order_id,
        execution.executed_quantity,
        execution.executed_price
    )
    .execute(&mut *tx)
    .await?;

    // 3. Создаём запись о сделке
    sqlx::query!(
        r#"
        INSERT INTO trades (order_id, symbol, side, price, quantity, executed_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#,
        execution.order_id,
        order.symbol,
        order.side,
        execution.executed_price,
        execution.executed_quantity
    )
    .execute(&mut *tx)
    .await?;

    // 4. Обновляем позицию
    let position_change = if order.side == "BUY" {
        execution.executed_quantity
    } else {
        -execution.executed_quantity
    };

    sqlx::query!(
        r#"
        INSERT INTO positions (symbol, quantity, avg_price, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (symbol)
        DO UPDATE SET
            quantity = positions.quantity + $2,
            avg_price = CASE
                WHEN $2 > 0 THEN
                    (positions.quantity * positions.avg_price + $2 * $3) / (positions.quantity + $2)
                ELSE positions.avg_price
            END,
            updated_at = NOW()
        "#,
        order.symbol,
        position_change,
        execution.executed_price
    )
    .execute(&mut *tx)
    .await?;

    // Коммитим транзакцию — все изменения атомарны
    tx.commit().await?;

    println!("Сделка исполнена: {} {} @ {}",
        order.symbol,
        execution.executed_quantity,
        execution.executed_price
    );

    Ok(())
}
```

## Оффлайн-режим с sqlx-data.json

Для CI/CD пайплайнов, где нет доступа к базе данных, sqlx поддерживает оффлайн-режим:

```bash
# Генерируем файл с метаданными запросов
cargo sqlx prepare

# Это создаст файл .sqlx/query-*.json для каждого запроса
# и файл sqlx-data.json в корне проекта
```

```rust
// В Cargo.toml добавляем:
// [features]
// offline = ["sqlx/offline"]

// Теперь можно компилировать без подключения к БД:
// SQLX_OFFLINE=true cargo build --features offline
```

## Динамические запросы с QueryBuilder

Иногда нужны динамические запросы (фильтры, сортировка). Для этого используем `QueryBuilder`:

```rust
use sqlx::{PgPool, QueryBuilder, postgres::Postgres};
use rust_decimal::Decimal;

#[derive(Debug)]
struct TradeFilter {
    symbol: Option<String>,
    side: Option<String>,
    min_price: Option<Decimal>,
    max_price: Option<Decimal>,
    limit: i64,
}

async fn search_trades(
    pool: &PgPool,
    filter: TradeFilter,
) -> Result<Vec<(i64, String, String, Decimal, Decimal)>, sqlx::Error> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT id, symbol, side, price, quantity FROM trades WHERE 1=1"
    );

    if let Some(symbol) = &filter.symbol {
        builder.push(" AND symbol = ");
        builder.push_bind(symbol);
    }

    if let Some(side) = &filter.side {
        builder.push(" AND side = ");
        builder.push_bind(side);
    }

    if let Some(min_price) = &filter.min_price {
        builder.push(" AND price >= ");
        builder.push_bind(min_price);
    }

    if let Some(max_price) = &filter.max_price {
        builder.push(" AND price <= ");
        builder.push_bind(max_price);
    }

    builder.push(" ORDER BY executed_at DESC LIMIT ");
    builder.push_bind(filter.limit);

    let query = builder.build_query_as::<(i64, String, String, Decimal, Decimal)>();

    query.fetch_all(pool).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    let filter = TradeFilter {
        symbol: Some("BTC/USDT".to_string()),
        side: Some("BUY".to_string()),
        min_price: Some(Decimal::new(4000000, 2)),
        max_price: None,
        limit: 50,
    };

    let trades = search_trades(&pool, filter).await?;

    for (id, symbol, side, price, qty) in trades {
        println!("#{}: {} {} {} @ {}", id, side, qty, symbol, price);
    }

    Ok(())
}
```

## Преимущества compile-time проверки

| Без проверки (runtime) | С проверкой (compile-time) |
|----------------------|---------------------------|
| Ошибки в продакшене | Ошибки при компиляции |
| Рефакторинг опасен | Рефакторинг безопасен |
| Требуется покрытие тестами | Компилятор — лучший тест |
| Опечатки в SQL ломают код | Опечатки не компилируются |
| Несовпадение типов в runtime | Типы проверяются сразу |

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `query!` | Макрос для проверки SQL на этапе компиляции |
| `query_as!` | Маппинг результатов в структуру с проверкой типов |
| `query_scalar!` | Получение единственного значения |
| Транзакции | Атомарные операции с `begin()` / `commit()` |
| QueryBuilder | Динамическое построение запросов |
| Оффлайн-режим | Компиляция без доступа к БД через `sqlx prepare` |

## Практические задания

1. **Система ордеров**: Создай структуру `OrderBook` с методами для размещения, отмены и получения ордеров. Все запросы должны использовать `query_as!` для compile-time проверки.

2. **Отчёт по позициям**: Напиши функцию, которая возвращает все позиции с их текущей рыночной стоимостью и нереализованным PnL. Используй JOIN между таблицами `positions` и `market_prices`.

3. **История сделок**: Реализуй пагинацию для истории сделок с использованием `OFFSET` и `LIMIT`. Добавь фильтрацию по дате и символу.

4. **Атомарный своп**: Напиши транзакцию, которая одновременно закрывает позицию по одному инструменту и открывает по другому (например, ребалансировка портфеля BTC -> ETH).

## Домашнее задание

1. **Риск-менеджмент с БД**: Создай систему, которая:
   - Хранит лимиты риска в таблице `risk_limits` (max_position_size, max_daily_loss, etc.)
   - Перед каждым ордером проверяет текущие позиции против лимитов
   - Записывает все проверки в таблицу `risk_checks` для аудита
   - Использует транзакции для атомарности проверки и создания ордера

2. **Мониторинг производительности**: Добавь логирование времени выполнения каждого запроса. Создай функцию-обёртку, которая замеряет время и записывает медленные запросы (>100ms) в таблицу `slow_queries`.

3. **Кэширование позиций**: Реализуй кэш позиций в памяти с инвалидацией при изменениях в БД. Используй `LISTEN/NOTIFY` PostgreSQL для уведомлений об изменениях.

4. **Оффлайн-подготовка**: Настрой свой проект для работы в оффлайн-режиме:
   - Запусти `cargo sqlx prepare`
   - Добавь сгенерированные файлы в git
   - Проверь, что проект компилируется с `SQLX_OFFLINE=true`

## Навигация

[← Предыдущий день](../226-tokio-postgres-async-client/ru.md) | [Следующий день →](../228-sqlx-migrations/ru.md)
