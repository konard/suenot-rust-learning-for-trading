# День 223: Индексы: быстрый поиск по дате

## Аналогия из трейдинга

Представь, что ты трейдер и хочешь найти все свои сделки за последнюю неделю. Если все твои сделки записаны в толстой тетради без какой-либо сортировки, тебе придётся просмотреть каждую страницу от начала до конца. Это медленно и неэффективно.

Теперь представь, что в этой тетради есть **оглавление по датам** — индекс, где указано: "Январь — страницы 1-30, Февраль — страницы 31-58...". Теперь поиск становится молниеносным: ты сразу открываешь нужный раздел!

В базах данных **индекс** работает точно так же — это специальная структура данных, которая ускоряет поиск по определённому полю. Индексы по дате особенно важны в трейдинге, потому что:
- Исторические данные цен всегда привязаны к времени
- Анализ часто требует выборки за определённый период
- Отчёты по P&L строятся по датам

## Что такое индекс?

Индекс в базе данных — это отсортированная структура данных, которая хранит указатели на строки таблицы. Самые распространённые типы индексов:

| Тип индекса | Описание | Применение |
|-------------|----------|------------|
| B-Tree | Сбалансированное дерево | Равенство и диапазоны |
| Hash | Хеш-таблица | Только точное равенство |
| GiST | Обобщённое дерево поиска | Геоданные, полнотекстовый поиск |
| BRIN | Block Range Index | Очень большие упорядоченные таблицы |

Для поиска по дате чаще всего используется **B-Tree индекс**, который отлично работает с диапазонами дат.

## Настройка проекта

Для работы с базами данных в Rust мы будем использовать библиотеки `sqlx` и `tokio`:

```toml
# Cargo.toml
[package]
name = "trading-indexes"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono"] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Создание таблицы с торговыми данными

```rust
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone)]
struct Trade {
    id: i64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,        // "buy" или "sell"
    executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct PriceCandle {
    id: i64,
    symbol: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    timestamp: DateTime<Utc>,
}

async fn create_tables(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // Таблица сделок
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS trades (
            id BIGSERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            price DECIMAL(20, 8) NOT NULL,
            quantity DECIMAL(20, 8) NOT NULL,
            side VARCHAR(4) NOT NULL,
            executed_at TIMESTAMPTZ NOT NULL
        )
    "#)
    .execute(pool)
    .await?;

    // Таблица свечей (OHLCV)
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS candles (
            id BIGSERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            open_price DECIMAL(20, 8) NOT NULL,
            high_price DECIMAL(20, 8) NOT NULL,
            low_price DECIMAL(20, 8) NOT NULL,
            close_price DECIMAL(20, 8) NOT NULL,
            volume DECIMAL(20, 8) NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL
        )
    "#)
    .execute(pool)
    .await?;

    println!("Таблицы успешно созданы!");
    Ok(())
}
```

## Создание индексов по дате

```rust
async fn create_indexes(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // Простой индекс по дате сделки
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_executed_at
        ON trades (executed_at)
    "#)
    .execute(pool)
    .await?;
    println!("Индекс idx_trades_executed_at создан");

    // Составной индекс: символ + дата (для фильтрации по инструменту и времени)
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_symbol_date
        ON trades (symbol, executed_at)
    "#)
    .execute(pool)
    .await?;
    println!("Индекс idx_trades_symbol_date создан");

    // Индекс по дате для свечей
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_candles_timestamp
        ON candles (timestamp)
    "#)
    .execute(pool)
    .await?;
    println!("Индекс idx_candles_timestamp создан");

    // Составной индекс для свечей: символ + время
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_candles_symbol_time
        ON candles (symbol, timestamp)
    "#)
    .execute(pool)
    .await?;
    println!("Индекс idx_candles_symbol_time создан");

    // Уникальный индекс для предотвращения дублей свечей
    sqlx::query(r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_candles_unique
        ON candles (symbol, timestamp)
    "#)
    .execute(pool)
    .await
    .ok(); // Игнорируем ошибку, если индекс уже существует
    println!("Уникальный индекс idx_candles_unique создан");

    Ok(())
}
```

## Заполнение тестовыми данными

```rust
use rand::Rng;

async fn seed_test_data(pool: &Pool<Postgres>, num_trades: i32) -> Result<(), sqlx::Error> {
    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT", "DOGE/USDT"];
    let base_prices = vec![42000.0, 2500.0, 100.0, 0.08];
    let sides = vec!["buy", "sell"];

    let mut rng = rand::thread_rng();
    let now = Utc::now();

    println!("Добавляем {} тестовых сделок...", num_trades);

    for i in 0..num_trades {
        let symbol_idx = rng.gen_range(0..symbols.len());
        let symbol = symbols[symbol_idx];
        let base_price = base_prices[symbol_idx];

        // Случайное отклонение цены ±5%
        let price = base_price * (1.0 + rng.gen_range(-0.05..0.05));
        let quantity = rng.gen_range(0.001..10.0);
        let side = sides[rng.gen_range(0..2)];

        // Распределяем сделки за последние 30 дней
        let days_ago = rng.gen_range(0..30);
        let hours_ago = rng.gen_range(0..24);
        let executed_at = now - Duration::days(days_ago) - Duration::hours(hours_ago);

        sqlx::query(r#"
            INSERT INTO trades (symbol, price, quantity, side, executed_at)
            VALUES ($1, $2, $3, $4, $5)
        "#)
        .bind(symbol)
        .bind(price)
        .bind(quantity)
        .bind(side)
        .bind(executed_at)
        .execute(pool)
        .await?;

        if (i + 1) % 1000 == 0 {
            println!("  Добавлено {} сделок", i + 1);
        }
    }

    println!("Тестовые данные успешно добавлены!");
    Ok(())
}
```

## Поиск с использованием индекса

```rust
use std::time::Instant;

async fn find_trades_by_date_range(
    pool: &Pool<Postgres>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Vec<Trade>, sqlx::Error> {
    let start = Instant::now();

    let trades: Vec<Trade> = sqlx::query_as!(
        Trade,
        r#"
        SELECT id, symbol, price as "price: f64", quantity as "quantity: f64",
               side, executed_at as "executed_at: DateTime<Utc>"
        FROM trades
        WHERE executed_at >= $1 AND executed_at < $2
        ORDER BY executed_at DESC
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    let elapsed = start.elapsed();
    println!(
        "Найдено {} сделок за {:?} (с {} по {})",
        trades.len(),
        elapsed,
        start_date.format("%Y-%m-%d"),
        end_date.format("%Y-%m-%d")
    );

    Ok(trades)
}

async fn find_trades_by_symbol_and_date(
    pool: &Pool<Postgres>,
    symbol: &str,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Vec<Trade>, sqlx::Error> {
    let start = Instant::now();

    let rows = sqlx::query(
        r#"
        SELECT id, symbol, price, quantity, side, executed_at
        FROM trades
        WHERE symbol = $1 AND executed_at >= $2 AND executed_at < $3
        ORDER BY executed_at DESC
        "#
    )
    .bind(symbol)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    let trades: Vec<Trade> = rows.iter().map(|row| {
        Trade {
            id: row.get("id"),
            symbol: row.get("symbol"),
            price: row.get::<sqlx::types::BigDecimal, _>("price")
                .to_string().parse().unwrap_or(0.0),
            quantity: row.get::<sqlx::types::BigDecimal, _>("quantity")
                .to_string().parse().unwrap_or(0.0),
            side: row.get("side"),
            executed_at: row.get("executed_at"),
        }
    }).collect();

    let elapsed = start.elapsed();
    println!(
        "Найдено {} сделок по {} за {:?}",
        trades.len(),
        symbol,
        elapsed
    );

    Ok(trades)
}
```

## Анализ плана выполнения запроса

Чтобы убедиться, что индекс используется, можно проанализировать план выполнения запроса:

```rust
async fn analyze_query_plan(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    println!("\n=== Анализ плана запроса ===\n");

    // Запрос БЕЗ использования индекса (полный перебор)
    let explain_no_index: Vec<(String,)> = sqlx::query_as(
        "EXPLAIN ANALYZE SELECT * FROM trades WHERE price > 40000"
    )
    .fetch_all(pool)
    .await?;

    println!("Запрос по цене (без индекса):");
    for row in &explain_no_index {
        println!("  {}", row.0);
    }

    // Запрос С использованием индекса по дате
    let explain_with_index: Vec<(String,)> = sqlx::query_as(
        "EXPLAIN ANALYZE SELECT * FROM trades WHERE executed_at > NOW() - INTERVAL '7 days'"
    )
    .fetch_all(pool)
    .await?;

    println!("\nЗапрос по дате (с индексом):");
    for row in &explain_with_index {
        println!("  {}", row.0);
    }

    // Запрос с составным индексом
    let explain_composite: Vec<(String,)> = sqlx::query_as(
        "EXPLAIN ANALYZE SELECT * FROM trades WHERE symbol = 'BTC/USDT' AND executed_at > NOW() - INTERVAL '7 days'"
    )
    .fetch_all(pool)
    .await?;

    println!("\nЗапрос по символу + дате (составной индекс):");
    for row in &explain_composite {
        println!("  {}", row.0);
    }

    Ok(())
}
```

## Типы индексов для дат

### 1. B-Tree (по умолчанию) — универсальный индекс

```rust
async fn create_btree_index(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // B-Tree отлично подходит для:
    // - Точного поиска: WHERE date = '2024-01-15'
    // - Диапазонов: WHERE date BETWEEN '2024-01-01' AND '2024-01-31'
    // - Сортировки: ORDER BY date DESC

    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_btree_date
        ON trades USING btree (executed_at)
    "#)
    .execute(pool)
    .await?;

    println!("B-Tree индекс создан");
    Ok(())
}
```

### 2. BRIN — для больших упорядоченных таблиц

```rust
async fn create_brin_index(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // BRIN (Block Range Index) идеален когда:
    // - Данные добавляются последовательно (по времени)
    // - Таблица очень большая (миллионы строк)
    // - Не нужна идеальная точность

    // BRIN занимает гораздо меньше места, чем B-Tree!
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_brin_date
        ON trades USING brin (executed_at)
        WITH (pages_per_range = 128)
    "#)
    .execute(pool)
    .await?;

    println!("BRIN индекс создан");
    Ok(())
}
```

### 3. Частичный индекс — только нужные данные

```rust
async fn create_partial_index(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // Частичный индекс индексирует только часть данных
    // Полезно для "горячих" данных — недавних сделок

    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_trades_recent
        ON trades (executed_at)
        WHERE executed_at > CURRENT_DATE - INTERVAL '30 days'
    "#)
    .execute(pool)
    .await?;

    println!("Частичный индекс для последних 30 дней создан");
    Ok(())
}
```

## Практический пример: торговый журнал

```rust
use std::collections::HashMap;

struct TradingJournal {
    pool: Pool<Postgres>,
}

impl TradingJournal {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(TradingJournal { pool })
    }

    /// Получить P&L за период
    async fn get_pnl_by_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<f64, sqlx::Error> {
        let row = sqlx::query(r#"
            SELECT
                COALESCE(SUM(
                    CASE
                        WHEN side = 'sell' THEN price * quantity
                        ELSE -price * quantity
                    END
                ), 0) as pnl
            FROM trades
            WHERE executed_at >= $1 AND executed_at < $2
        "#)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let pnl: sqlx::types::BigDecimal = row.get("pnl");
        Ok(pnl.to_string().parse().unwrap_or(0.0))
    }

    /// Получить дневную статистику
    async fn get_daily_stats(
        &self,
        date: DateTime<Utc>,
    ) -> Result<DailyStats, sqlx::Error> {
        let start = date.date_naive().and_hms_opt(0, 0, 0).unwrap()
            .and_utc();
        let end = start + Duration::days(1);

        let row = sqlx::query(r#"
            SELECT
                COUNT(*) as trade_count,
                COALESCE(SUM(quantity), 0) as total_volume,
                COALESCE(SUM(CASE WHEN side = 'buy' THEN 1 ELSE 0 END), 0) as buy_count,
                COALESCE(SUM(CASE WHEN side = 'sell' THEN 1 ELSE 0 END), 0) as sell_count
            FROM trades
            WHERE executed_at >= $1 AND executed_at < $2
        "#)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        Ok(DailyStats {
            date: start,
            trade_count: row.get::<i64, _>("trade_count") as u32,
            total_volume: row.get::<sqlx::types::BigDecimal, _>("total_volume")
                .to_string().parse().unwrap_or(0.0),
            buy_count: row.get::<i64, _>("buy_count") as u32,
            sell_count: row.get::<i64, _>("sell_count") as u32,
        })
    }

    /// Получить сделки по символу за последние N дней
    async fn get_recent_trades(
        &self,
        symbol: &str,
        days: i64,
    ) -> Result<Vec<Trade>, sqlx::Error> {
        let start = Utc::now() - Duration::days(days);

        let rows = sqlx::query(r#"
            SELECT id, symbol, price, quantity, side, executed_at
            FROM trades
            WHERE symbol = $1 AND executed_at >= $2
            ORDER BY executed_at DESC
            LIMIT 100
        "#)
        .bind(symbol)
        .bind(start)
        .fetch_all(&self.pool)
        .await?;

        let trades: Vec<Trade> = rows.iter().map(|row| {
            Trade {
                id: row.get("id"),
                symbol: row.get("symbol"),
                price: row.get::<sqlx::types::BigDecimal, _>("price")
                    .to_string().parse().unwrap_or(0.0),
                quantity: row.get::<sqlx::types::BigDecimal, _>("quantity")
                    .to_string().parse().unwrap_or(0.0),
                side: row.get("side"),
                executed_at: row.get("executed_at"),
            }
        }).collect();

        Ok(trades)
    }

    /// Группировка по дням с индексом
    async fn get_trades_grouped_by_day(
        &self,
        symbol: &str,
        days: i64,
    ) -> Result<HashMap<String, Vec<Trade>>, sqlx::Error> {
        let trades = self.get_recent_trades(symbol, days).await?;

        let mut grouped: HashMap<String, Vec<Trade>> = HashMap::new();
        for trade in trades {
            let date_key = trade.executed_at.format("%Y-%m-%d").to_string();
            grouped.entry(date_key).or_insert_with(Vec::new).push(trade);
        }

        Ok(grouped)
    }
}

#[derive(Debug)]
struct DailyStats {
    date: DateTime<Utc>,
    trade_count: u32,
    total_volume: f64,
    buy_count: u32,
    sell_count: u32,
}
```

## Оптимизация запросов с индексами

### 1. Избегайте функций в WHERE

```rust
// ПЛОХО: индекс не используется
// WHERE DATE(executed_at) = '2024-01-15'

// ХОРОШО: индекс используется
async fn find_by_date_efficient(
    pool: &Pool<Postgres>,
    date_str: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(r#"
        SELECT COUNT(*) as count
        FROM trades
        WHERE executed_at >= $1::date
          AND executed_at < ($1::date + INTERVAL '1 day')
    "#)
    .bind(date_str)
    .fetch_one(pool)
    .await?;

    Ok(row.get("count"))
}
```

### 2. Используйте правильный порядок столбцов в составном индексе

```rust
// Составной индекс (symbol, executed_at) эффективен для:
// - WHERE symbol = 'BTC' AND executed_at > '2024-01-01'
// - WHERE symbol = 'BTC'

// Но НЕ эффективен для:
// - WHERE executed_at > '2024-01-01' (без symbol)

async fn demonstrate_index_order(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // Этот запрос использует составной индекс эффективно
    let _ = sqlx::query(
        "SELECT COUNT(*) FROM trades WHERE symbol = $1 AND executed_at > $2"
    )
    .bind("BTC/USDT")
    .bind(Utc::now() - Duration::days(7))
    .fetch_one(pool)
    .await?;

    // Для запросов только по дате нужен отдельный индекс
    let _ = sqlx::query(
        "SELECT COUNT(*) FROM trades WHERE executed_at > $1"
    )
    .bind(Utc::now() - Duration::days(7))
    .fetch_one(pool)
    .await?;

    Ok(())
}
```

## Мониторинг индексов

```rust
async fn check_index_usage(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    println!("\n=== Статистика использования индексов ===\n");

    let rows = sqlx::query(r#"
        SELECT
            schemaname,
            tablename,
            indexname,
            idx_scan as index_scans,
            idx_tup_read as tuples_read,
            idx_tup_fetch as tuples_fetched
        FROM pg_stat_user_indexes
        WHERE tablename IN ('trades', 'candles')
        ORDER BY idx_scan DESC
    "#)
    .fetch_all(pool)
    .await?;

    for row in rows {
        let index_name: &str = row.get("indexname");
        let scans: i64 = row.get("index_scans");
        let tuples: i64 = row.get("tuples_read");

        println!(
            "Индекс: {} | Сканирований: {} | Кортежей прочитано: {}",
            index_name, scans, tuples
        );
    }

    Ok(())
}

async fn check_index_sizes(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    println!("\n=== Размеры индексов ===\n");

    let rows = sqlx::query(r#"
        SELECT
            indexname,
            pg_size_pretty(pg_relation_size(indexname::regclass)) as index_size
        FROM pg_indexes
        WHERE tablename IN ('trades', 'candles')
    "#)
    .fetch_all(pool)
    .await?;

    for row in rows {
        let name: &str = row.get("indexname");
        let size: &str = row.get("index_size");
        println!("Индекс: {} | Размер: {}", name, size);
    }

    Ok(())
}
```

## Полный пример

```rust
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use chrono::{DateTime, Utc, Duration};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost/trading".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Подключение к базе данных установлено!\n");

    // Создаём таблицы
    create_tables(&pool).await?;

    // Создаём индексы
    create_indexes(&pool).await?;

    // Заполняем тестовыми данными (10000 сделок)
    seed_test_data(&pool, 10000).await?;

    // Тестируем поиск
    println!("\n=== Тестирование поиска ===\n");

    let end = Utc::now();
    let start = end - Duration::days(7);

    // Поиск по диапазону дат
    let trades = find_trades_by_date_range(&pool, start, end).await?;
    println!("Первые 3 сделки:");
    for trade in trades.iter().take(3) {
        println!("  {:?}", trade);
    }

    // Поиск по символу и дате
    let btc_trades = find_trades_by_symbol_and_date(
        &pool, "BTC/USDT", start, end
    ).await?;
    println!("\nПервые 3 сделки по BTC:");
    for trade in btc_trades.iter().take(3) {
        println!("  {:?}", trade);
    }

    // Анализ планов запросов
    analyze_query_plan(&pool).await?;

    // Статистика индексов
    check_index_usage(&pool).await?;
    check_index_sizes(&pool).await?;

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Индекс | Структура данных для ускорения поиска |
| B-Tree | Универсальный индекс для равенства и диапазонов |
| BRIN | Компактный индекс для больших упорядоченных данных |
| Составной индекс | Индекс по нескольким столбцам |
| Частичный индекс | Индекс только для части данных |
| EXPLAIN ANALYZE | Анализ плана выполнения запроса |

## Домашнее задание

1. **Сравнение производительности**: Создай таблицу с 1 миллионом строк и сравни время выполнения запроса по диапазону дат с индексом и без него. Используй `EXPLAIN ANALYZE` для анализа.

2. **Составной индекс**: Создай составной индекс `(symbol, executed_at, side)` и определи, для каких запросов он будет эффективен, а для каких — нет.

3. **BRIN vs B-Tree**: Создай обе версии индекса на таблице с миллионом строк. Сравни:
   - Размер индекса
   - Время выполнения запроса
   - Когда какой индекс лучше использовать

4. **Торговая аналитика**: Используя индексы, напиши функции для:
   - Поиска самой прибыльной сделки за месяц
   - Подсчёта объёма торгов по часам
   - Нахождения "затишья" — периодов без сделок более 1 часа

## Навигация

[← Предыдущий день](../222-transactions-atomic-operations/ru.md) | [Следующий день →](../224-migrations-schema-evolution/ru.md)
