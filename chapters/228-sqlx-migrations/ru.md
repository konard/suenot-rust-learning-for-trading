# День 228: sqlx Миграции

## Аналогия из трейдинга

Представь, что ты разрабатываешь торговую систему. Изначально тебе нужно хранить только цены активов. Затем появляется необходимость отслеживать объёмы торгов. Позже — история ордеров, потом — портфолио пользователей, управление рисками и торговые стратегии. Каждое такое изменение требует модификации структуры базы данных.

**Миграции базы данных** — это как эволюция торговой системы: контролируемый, версионированный способ изменения схемы данных. Подобно тому, как трейдер адаптирует свои стратегии к меняющимся рыночным условиям, миграции позволяют базе данных "адаптироваться" к новым требованиям бизнеса.

В реальном трейдинге это критично:
- Добавление новых типов ордеров без потери исторических данных
- Расширение аналитики цен с сохранением совместимости
- Эволюция системы управления рисками

## Что такое миграции?

Миграция — это файл с SQL-командами, который описывает изменение схемы базы данных. Каждая миграция имеет:

1. **Версию** — уникальный идентификатор (обычно timestamp)
2. **Up-скрипт** — команды для применения изменений
3. **Down-скрипт** — команды для отката изменений

```
migrations/
├── 20240101000000_create_assets.sql
├── 20240102000000_create_trades.sql
├── 20240103000000_add_volume_column.sql
└── 20240104000000_create_portfolio.sql
```

## Установка sqlx-cli

Для работы с миграциями нужен инструмент командной строки:

```bash
cargo install sqlx-cli --features postgres
```

Для поддержки нескольких баз данных:

```bash
cargo install sqlx-cli --features postgres,mysql,sqlite
```

## Настройка проекта

```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "macros"] }
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
chrono = { version = "0.4", features = ["serde"] }
```

Создаём файл `.env`:

```bash
DATABASE_URL=postgres://trader:password@localhost:5432/trading_db
```

## Создание первой миграции

```bash
# Инициализация директории миграций
sqlx migrate add create_assets
```

Это создаст файл `migrations/<timestamp>_create_assets.sql`:

```sql
-- migrations/20240101120000_create_assets.sql

-- Таблица активов для торговли
CREATE TABLE assets (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    asset_type VARCHAR(20) NOT NULL CHECK (asset_type IN ('crypto', 'stock', 'forex', 'commodity')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Индекс для быстрого поиска по символу
CREATE INDEX idx_assets_symbol ON assets(symbol);

-- Вставляем базовые активы
INSERT INTO assets (symbol, name, asset_type) VALUES
    ('BTC', 'Bitcoin', 'crypto'),
    ('ETH', 'Ethereum', 'crypto'),
    ('AAPL', 'Apple Inc.', 'stock'),
    ('EUR/USD', 'Euro/US Dollar', 'forex');
```

## Создание обратимых миграций

Для возможности отката используем reversible миграции:

```bash
sqlx migrate add -r create_trades
```

Это создаст два файла:

```sql
-- migrations/20240102120000_create_trades.up.sql

CREATE TABLE trades (
    id BIGSERIAL PRIMARY KEY,
    asset_id INTEGER REFERENCES assets(id),
    side VARCHAR(4) NOT NULL CHECK (side IN ('buy', 'sell')),
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    total_value DECIMAL(20, 8) GENERATED ALWAYS AS (price * quantity) STORED,
    fee DECIMAL(20, 8) DEFAULT 0,
    executed_at TIMESTAMPTZ DEFAULT NOW(),
    strategy VARCHAR(50),
    notes TEXT
);

-- Индексы для анализа торгов
CREATE INDEX idx_trades_asset ON trades(asset_id);
CREATE INDEX idx_trades_executed ON trades(executed_at);
CREATE INDEX idx_trades_strategy ON trades(strategy);
```

```sql
-- migrations/20240102120000_create_trades.down.sql

DROP INDEX IF EXISTS idx_trades_strategy;
DROP INDEX IF EXISTS idx_trades_executed;
DROP INDEX IF EXISTS idx_trades_asset;
DROP TABLE IF EXISTS trades;
```

## Применение миграций

```bash
# Применить все pending миграции
sqlx migrate run

# Проверить статус миграций
sqlx migrate info
```

## Миграция для портфолио и управления рисками

```sql
-- migrations/20240103120000_create_portfolio.up.sql

-- Портфолио трейдера
CREATE TABLE portfolio (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    asset_id INTEGER REFERENCES assets(id),
    quantity DECIMAL(20, 8) NOT NULL DEFAULT 0,
    avg_buy_price DECIMAL(20, 8),
    total_invested DECIMAL(20, 8) DEFAULT 0,
    unrealized_pnl DECIMAL(20, 8) DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, asset_id)
);

-- Лимиты риск-менеджмента
CREATE TABLE risk_limits (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    max_position_size DECIMAL(20, 8) NOT NULL,
    max_daily_loss DECIMAL(20, 8) NOT NULL,
    max_drawdown_percent DECIMAL(5, 2) NOT NULL,
    stop_loss_percent DECIMAL(5, 2),
    take_profit_percent DECIMAL(5, 2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Журнал событий риска
CREATE TABLE risk_events (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) CHECK (severity IN ('info', 'warning', 'critical')),
    message TEXT NOT NULL,
    metadata JSONB,
    occurred_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_risk_events_user ON risk_events(user_id);
CREATE INDEX idx_risk_events_occurred ON risk_events(occurred_at);
```

```sql
-- migrations/20240103120000_create_portfolio.down.sql

DROP TABLE IF EXISTS risk_events;
DROP TABLE IF EXISTS risk_limits;
DROP TABLE IF EXISTS portfolio;
```

## Программное применение миграций

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::migrate::Migrator;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Применение миграций из директории
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(&pool).await?;

    println!("Migrations applied successfully!");

    Ok(())
}
```

## Встроенные миграции (Embedded Migrations)

Для production-сборок можно встроить миграции в бинарник:

```rust
use sqlx::postgres::PgPoolOptions;

// Встраиваем миграции на этапе компиляции
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Применяем встроенные миграции
    MIGRATOR.run(&pool).await?;

    println!("Embedded migrations applied!");

    Ok(())
}
```

## Миграция данных: история цен

```sql
-- migrations/20240104120000_create_price_history.up.sql

-- Хранение истории цен для анализа
CREATE TABLE price_history (
    id BIGSERIAL PRIMARY KEY,
    asset_id INTEGER REFERENCES assets(id),
    open_price DECIMAL(20, 8) NOT NULL,
    high_price DECIMAL(20, 8) NOT NULL,
    low_price DECIMAL(20, 8) NOT NULL,
    close_price DECIMAL(20, 8) NOT NULL,
    volume DECIMAL(20, 8) NOT NULL,
    timeframe VARCHAR(10) NOT NULL CHECK (timeframe IN ('1m', '5m', '15m', '1h', '4h', '1d', '1w')),
    candle_time TIMESTAMPTZ NOT NULL,
    UNIQUE(asset_id, timeframe, candle_time)
);

-- Партиционирование для больших объёмов данных
CREATE INDEX idx_price_history_asset_time ON price_history(asset_id, candle_time);
CREATE INDEX idx_price_history_timeframe ON price_history(timeframe, candle_time);

-- Материализованное представление для дневной статистики
CREATE MATERIALIZED VIEW daily_stats AS
SELECT
    asset_id,
    DATE(candle_time) as trade_date,
    MIN(low_price) as day_low,
    MAX(high_price) as day_high,
    SUM(volume) as total_volume,
    (ARRAY_AGG(open_price ORDER BY candle_time))[1] as day_open,
    (ARRAY_AGG(close_price ORDER BY candle_time DESC))[1] as day_close
FROM price_history
WHERE timeframe = '1h'
GROUP BY asset_id, DATE(candle_time);

CREATE UNIQUE INDEX idx_daily_stats ON daily_stats(asset_id, trade_date);
```

## Добавление колонки к существующей таблице

```sql
-- migrations/20240105120000_add_strategy_params.up.sql

-- Добавляем параметры торговых стратегий
ALTER TABLE trades ADD COLUMN IF NOT EXISTS strategy_params JSONB;
ALTER TABLE trades ADD COLUMN IF NOT EXISTS signal_strength DECIMAL(5, 2);
ALTER TABLE trades ADD COLUMN IF NOT EXISTS market_condition VARCHAR(20);

-- Комментарии для документации
COMMENT ON COLUMN trades.strategy_params IS 'JSON параметры стратегии: stop_loss, take_profit, indicators';
COMMENT ON COLUMN trades.signal_strength IS 'Сила сигнала от 0.00 до 1.00';
COMMENT ON COLUMN trades.market_condition IS 'Состояние рынка: trending, ranging, volatile';
```

```sql
-- migrations/20240105120000_add_strategy_params.down.sql

ALTER TABLE trades DROP COLUMN IF EXISTS market_condition;
ALTER TABLE trades DROP COLUMN IF EXISTS signal_strength;
ALTER TABLE trades DROP COLUMN IF EXISTS strategy_params;
```

## Практический пример: Торговая система с миграциями

```rust
use sqlx::{PgPool, FromRow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, FromRow)]
struct Asset {
    id: i32,
    symbol: String,
    name: String,
    asset_type: String,
}

#[derive(Debug, FromRow)]
struct Trade {
    id: i64,
    asset_id: i32,
    side: String,
    price: rust_decimal::Decimal,
    quantity: rust_decimal::Decimal,
    total_value: rust_decimal::Decimal,
    executed_at: DateTime<Utc>,
    strategy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StrategyParams {
    stop_loss: f64,
    take_profit: f64,
    indicators: Vec<String>,
}

struct TradingSystem {
    pool: PgPool,
}

impl TradingSystem {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::PgPool::connect(database_url).await?;

        // Применяем миграции при старте
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    async fn get_assets(&self) -> Result<Vec<Asset>, sqlx::Error> {
        sqlx::query_as!(
            Asset,
            "SELECT id, symbol, name, asset_type FROM assets ORDER BY symbol"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn execute_trade(
        &self,
        asset_id: i32,
        side: &str,
        price: f64,
        quantity: f64,
        strategy: Option<&str>,
        params: Option<StrategyParams>,
    ) -> Result<i64, sqlx::Error> {
        let params_json = params.map(|p| serde_json::to_value(p).unwrap());

        let result = sqlx::query!(
            r#"
            INSERT INTO trades (asset_id, side, price, quantity, strategy, strategy_params)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            asset_id,
            side,
            rust_decimal::Decimal::from_f64_retain(price),
            rust_decimal::Decimal::from_f64_retain(quantity),
            strategy,
            params_json
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.id)
    }

    async fn get_portfolio_value(&self, user_id: i32) -> Result<f64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(p.quantity * ph.close_price), 0) as "total_value!"
            FROM portfolio p
            JOIN LATERAL (
                SELECT close_price
                FROM price_history
                WHERE asset_id = p.asset_id
                ORDER BY candle_time DESC
                LIMIT 1
            ) ph ON true
            WHERE p.user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.total_value.to_string().parse().unwrap_or(0.0))
    }

    async fn check_risk_limits(&self, user_id: i32) -> Result<bool, sqlx::Error> {
        let limits = sqlx::query!(
            "SELECT max_daily_loss, max_drawdown_percent FROM risk_limits WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(limits) = limits {
            // Проверяем дневные убытки
            let daily_pnl = sqlx::query!(
                r#"
                SELECT COALESCE(SUM(
                    CASE WHEN side = 'sell' THEN total_value ELSE -total_value END
                ), 0) as "pnl!"
                FROM trades
                WHERE executed_at > NOW() - INTERVAL '1 day'
                "#
            )
            .fetch_one(&self.pool)
            .await?;

            let pnl: f64 = daily_pnl.pnl.to_string().parse().unwrap_or(0.0);
            let max_loss: f64 = limits.max_daily_loss.to_string().parse().unwrap_or(0.0);

            if pnl < -max_loss {
                // Логируем событие риска
                sqlx::query!(
                    r#"
                    INSERT INTO risk_events (user_id, event_type, severity, message)
                    VALUES ($1, 'daily_loss_exceeded', 'critical', $2)
                    "#,
                    user_id,
                    format!("Daily loss {} exceeds limit {}", pnl, max_loss)
                )
                .execute(&self.pool)
                .await?;

                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let trading = TradingSystem::new(&database_url).await?;

    // Получаем список активов
    let assets = trading.get_assets().await?;
    println!("Available assets:");
    for asset in &assets {
        println!("  {} - {} ({})", asset.symbol, asset.name, asset.asset_type);
    }

    // Выполняем сделку с параметрами стратегии
    let params = StrategyParams {
        stop_loss: 0.02,
        take_profit: 0.05,
        indicators: vec!["RSI".to_string(), "MACD".to_string()],
    };

    let trade_id = trading.execute_trade(
        1,
        "buy",
        42000.0,
        0.5,
        Some("momentum"),
        Some(params),
    ).await?;

    println!("Executed trade #{}", trade_id);

    // Проверяем лимиты риска
    let can_trade = trading.check_risk_limits(1).await?;
    println!("Can continue trading: {}", can_trade);

    Ok(())
}
```

## Откат миграций

```bash
# Откатить последнюю миграцию
sqlx migrate revert

# Откатить все миграции
sqlx migrate revert --target-version 0
```

## Лучшие практики

| Практика | Описание |
|----------|----------|
| Атомарность | Одна миграция = одно логическое изменение |
| Обратимость | Всегда пишите down-скрипты |
| Идемпотентность | Используйте IF NOT EXISTS / IF EXISTS |
| Тестирование | Проверяйте миграции на копии данных |
| Версионирование | Храните миграции в git |
| Документация | Комментируйте сложные изменения |

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Миграции | Версионированные изменения схемы БД |
| sqlx-cli | Инструмент командной строки для миграций |
| Up/Down | Скрипты применения и отката |
| Embedded | Встраивание миграций в бинарник |
| sqlx::migrate! | Макрос для compile-time проверки |

## Домашнее задание

1. **Система ордеров**: Создай миграции для полноценной системы ордеров с типами (market, limit, stop-loss, take-profit), статусами и историей изменений.

2. **Аналитика**: Добавь миграцию с материализованными представлениями для:
   - Ежедневной статистики по каждому активу
   - Топ-10 прибыльных сделок за месяц
   - Средней доходности по стратегиям

3. **Аудит**: Создай систему аудита изменений с помощью триггеров:
   - Таблица audit_log для записи всех изменений
   - Триггеры на INSERT/UPDATE/DELETE для trades и portfolio

4. **Миграция данных**: Напиши миграцию, которая:
   - Создаёт новую структуру таблицы
   - Переносит данные из старой структуры
   - Удаляет старую таблицу
   - Переименовывает новую таблицу

## Навигация

[← Предыдущий день](../227-sqlx-compile-time-queries/ru.md) | [Следующий день →](../229-connection-pool/ru.md)
