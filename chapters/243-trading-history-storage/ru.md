# День 243: Проект: Хранилище торговой истории

## Аналогия из трейдинга

Представь себя управляющим крупного хедж-фонда. Каждый день через твои системы проходят тысячи сделок: покупки, продажи, стоп-лоссы, тейк-профиты. Через год ты захочешь проанализировать свою стратегию: какие сделки были прибыльными? В какое время дня ты торгуешь лучше всего? Какие активы приносят максимальную прибыль?

Без надёжного хранилища торговой истории ты не сможешь ответить ни на один из этих вопросов. **Хранилище торговой истории** — это твой "чёрный ящик" трейдера: оно записывает каждое действие, каждую сделку, каждое изменение портфеля, чтобы потом ты мог анализировать и улучшать свою торговлю.

В этом проекте мы объединим все знания о базах данных из восьмого месяца и построим полноценное хранилище для:
- Записи всех сделок с полной информацией
- Хранения истории изменения портфеля
- Кэширования актуальных цен
- Быстрого поиска и аналитики

## Архитектура системы

Наше хранилище будет использовать несколько технологий:

```
┌─────────────────────────────────────────────────────────────┐
│                    Trading History Storage                   │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Redis     │  │  PostgreSQL │  │      SQLite         │  │
│  │   (Кэш)     │  │ (Продакшн)  │  │   (Разработка)      │  │
│  │             │  │             │  │                     │  │
│  │ - Последние │  │ - Сделки    │  │ - Локальное         │  │
│  │   цены      │  │ - Портфель  │  │   тестирование      │  │
│  │ - Сессии    │  │ - Аналитика │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Структуры данных

Начнём с определения основных структур:

```rust
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Направление сделки
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "trade_side", rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Тип ордера
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_type", rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// Статус сделки
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "trade_status", rename_all = "lowercase")]
pub enum TradeStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
}

/// Запись о сделке
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Trade {
    pub id: i64,
    pub symbol: String,
    pub side: TradeSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Decimal,
    pub executed_quantity: Decimal,
    pub executed_price: Decimal,
    pub commission: Decimal,
    pub status: TradeStatus,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Снимок портфеля
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortfolioSnapshot {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub total_value_usd: Decimal,
    pub cash_balance: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
}

/// Позиция в портфеле
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Position {
    pub id: i64,
    pub snapshot_id: i64,
    pub symbol: String,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub current_price: Decimal,
    pub unrealized_pnl: Decimal,
}
```

## Создание схемы базы данных

SQL миграция для PostgreSQL:

```rust
use sqlx::{PgPool, Error};

pub async fn run_migrations(pool: &PgPool) -> Result<(), Error> {
    // Создаём типы enum
    sqlx::query(r#"
        DO $$ BEGIN
            CREATE TYPE trade_side AS ENUM ('buy', 'sell');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        DO $$ BEGIN
            CREATE TYPE order_type AS ENUM ('market', 'limit', 'stoploss', 'takeprofit');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        DO $$ BEGIN
            CREATE TYPE trade_status AS ENUM (
                'pending', 'filled', 'partiallyfilled', 'cancelled', 'rejected'
            );
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
    "#)
    .execute(pool)
    .await?;

    // Таблица сделок
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS trades (
            id BIGSERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            side trade_side NOT NULL,
            order_type order_type NOT NULL,
            quantity DECIMAL(20, 8) NOT NULL,
            price DECIMAL(20, 8) NOT NULL,
            executed_quantity DECIMAL(20, 8) NOT NULL DEFAULT 0,
            executed_price DECIMAL(20, 8) NOT NULL DEFAULT 0,
            commission DECIMAL(20, 8) NOT NULL DEFAULT 0,
            status trade_status NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            executed_at TIMESTAMPTZ,
            notes TEXT,

            -- Индексы для быстрого поиска
            CONSTRAINT positive_quantity CHECK (quantity > 0),
            CONSTRAINT positive_price CHECK (price > 0)
        );

        CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trades(symbol);
        CREATE INDEX IF NOT EXISTS idx_trades_created_at ON trades(created_at);
        CREATE INDEX IF NOT EXISTS idx_trades_status ON trades(status);
        CREATE INDEX IF NOT EXISTS idx_trades_symbol_created
            ON trades(symbol, created_at DESC);
    "#)
    .execute(pool)
    .await?;

    // Таблица снимков портфеля
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS portfolio_snapshots (
            id BIGSERIAL PRIMARY KEY,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            total_value_usd DECIMAL(20, 2) NOT NULL,
            cash_balance DECIMAL(20, 2) NOT NULL,
            unrealized_pnl DECIMAL(20, 2) NOT NULL,
            realized_pnl DECIMAL(20, 2) NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp
            ON portfolio_snapshots(timestamp DESC);
    "#)
    .execute(pool)
    .await?;

    // Таблица позиций
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS positions (
            id BIGSERIAL PRIMARY KEY,
            snapshot_id BIGINT NOT NULL REFERENCES portfolio_snapshots(id),
            symbol VARCHAR(20) NOT NULL,
            quantity DECIMAL(20, 8) NOT NULL,
            average_price DECIMAL(20, 8) NOT NULL,
            current_price DECIMAL(20, 8) NOT NULL,
            unrealized_pnl DECIMAL(20, 2) NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_positions_snapshot
            ON positions(snapshot_id);
        CREATE INDEX IF NOT EXISTS idx_positions_symbol
            ON positions(symbol);
    "#)
    .execute(pool)
    .await?;

    println!("Миграции успешно выполнены!");
    Ok(())
}
```

## Репозиторий для работы со сделками

```rust
use sqlx::PgPool;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

pub struct TradeRepository {
    pool: PgPool,
}

impl TradeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Создаёт новую сделку
    pub async fn create_trade(
        &self,
        symbol: &str,
        side: TradeSide,
        order_type: OrderType,
        quantity: Decimal,
        price: Decimal,
        notes: Option<&str>,
    ) -> Result<Trade, sqlx::Error> {
        sqlx::query_as::<_, Trade>(r#"
            INSERT INTO trades (symbol, side, order_type, quantity, price, notes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#)
        .bind(symbol)
        .bind(side)
        .bind(order_type)
        .bind(quantity)
        .bind(price)
        .bind(notes)
        .fetch_one(&self.pool)
        .await
    }

    /// Обновляет статус сделки при исполнении
    pub async fn execute_trade(
        &self,
        trade_id: i64,
        executed_quantity: Decimal,
        executed_price: Decimal,
        commission: Decimal,
    ) -> Result<Trade, sqlx::Error> {
        let status = if executed_quantity >= self.get_trade(trade_id).await?.quantity {
            TradeStatus::Filled
        } else {
            TradeStatus::PartiallyFilled
        };

        sqlx::query_as::<_, Trade>(r#"
            UPDATE trades
            SET executed_quantity = $2,
                executed_price = $3,
                commission = $4,
                status = $5,
                executed_at = NOW()
            WHERE id = $1
            RETURNING *
        "#)
        .bind(trade_id)
        .bind(executed_quantity)
        .bind(executed_price)
        .bind(commission)
        .bind(status)
        .fetch_one(&self.pool)
        .await
    }

    /// Отменяет сделку
    pub async fn cancel_trade(&self, trade_id: i64) -> Result<Trade, sqlx::Error> {
        sqlx::query_as::<_, Trade>(r#"
            UPDATE trades
            SET status = 'cancelled'
            WHERE id = $1 AND status = 'pending'
            RETURNING *
        "#)
        .bind(trade_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Получает сделку по ID
    pub async fn get_trade(&self, trade_id: i64) -> Result<Trade, sqlx::Error> {
        sqlx::query_as::<_, Trade>("SELECT * FROM trades WHERE id = $1")
            .bind(trade_id)
            .fetch_one(&self.pool)
            .await
    }

    /// Получает все сделки по символу
    pub async fn get_trades_by_symbol(
        &self,
        symbol: &str,
        limit: i64,
    ) -> Result<Vec<Trade>, sqlx::Error> {
        sqlx::query_as::<_, Trade>(r#"
            SELECT * FROM trades
            WHERE symbol = $1
            ORDER BY created_at DESC
            LIMIT $2
        "#)
        .bind(symbol)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Получает сделки за период
    pub async fn get_trades_in_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Trade>, sqlx::Error> {
        sqlx::query_as::<_, Trade>(r#"
            SELECT * FROM trades
            WHERE created_at >= $1 AND created_at <= $2
            ORDER BY created_at DESC
        "#)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
    }

    /// Получает статистику по сделкам
    pub async fn get_trade_statistics(
        &self,
        symbol: Option<&str>,
    ) -> Result<TradeStatistics, sqlx::Error> {
        let query = if symbol.is_some() {
            r#"
                SELECT
                    COUNT(*) as total_trades,
                    COUNT(*) FILTER (WHERE side = 'buy') as buy_trades,
                    COUNT(*) FILTER (WHERE side = 'sell') as sell_trades,
                    COUNT(*) FILTER (WHERE status = 'filled') as filled_trades,
                    COALESCE(SUM(executed_quantity * executed_price), 0) as total_volume,
                    COALESCE(SUM(commission), 0) as total_commission
                FROM trades
                WHERE symbol = $1
            "#
        } else {
            r#"
                SELECT
                    COUNT(*) as total_trades,
                    COUNT(*) FILTER (WHERE side = 'buy') as buy_trades,
                    COUNT(*) FILTER (WHERE side = 'sell') as sell_trades,
                    COUNT(*) FILTER (WHERE status = 'filled') as filled_trades,
                    COALESCE(SUM(executed_quantity * executed_price), 0) as total_volume,
                    COALESCE(SUM(commission), 0) as total_commission
                FROM trades
            "#
        };

        sqlx::query_as::<_, TradeStatistics>(query)
            .bind(symbol)
            .fetch_one(&self.pool)
            .await
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct TradeStatistics {
    pub total_trades: i64,
    pub buy_trades: i64,
    pub sell_trades: i64,
    pub filled_trades: i64,
    pub total_volume: Decimal,
    pub total_commission: Decimal,
}
```

## Кэширование с Redis

```rust
use redis::{AsyncCommands, Client};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPrice {
    pub symbol: String,
    pub price: Decimal,
    pub timestamp: i64,
}

pub struct PriceCache {
    client: Client,
}

impl PriceCache {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    /// Сохраняет цену в кэш
    pub async fn set_price(
        &self,
        symbol: &str,
        price: Decimal,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let cached = CachedPrice {
            symbol: symbol.to_string(),
            price,
            timestamp: chrono::Utc::now().timestamp(),
        };

        let key = format!("price:{}", symbol);
        let value = serde_json::to_string(&cached).unwrap();

        // Устанавливаем TTL 60 секунд
        conn.set_ex(&key, value, 60).await?;

        // Также добавляем в sorted set для истории
        let history_key = format!("price_history:{}", symbol);
        conn.zadd(&history_key, cached.timestamp, &cached.price.to_string()).await?;

        // Храним только последние 1000 записей
        conn.zremrangebyrank(&history_key, 0, -1001).await?;

        Ok(())
    }

    /// Получает последнюю цену из кэша
    pub async fn get_price(
        &self,
        symbol: &str,
    ) -> Result<Option<CachedPrice>, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let key = format!("price:{}", symbol);
        let value: Option<String> = conn.get(&key).await?;

        Ok(value.and_then(|v| serde_json::from_str(&v).ok()))
    }

    /// Получает историю цен
    pub async fn get_price_history(
        &self,
        symbol: &str,
        count: isize,
    ) -> Result<Vec<(Decimal, i64)>, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let key = format!("price_history:{}", symbol);
        let result: Vec<(String, i64)> = conn.zrevrange_withscores(&key, 0, count - 1).await?;

        Ok(result
            .into_iter()
            .filter_map(|(price, timestamp)| {
                price.parse::<Decimal>().ok().map(|p| (p, timestamp))
            })
            .collect())
    }

    /// Получает цены для нескольких символов
    pub async fn get_prices(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<Option<CachedPrice>>, redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let keys: Vec<String> = symbols
            .iter()
            .map(|s| format!("price:{}", s))
            .collect();

        let values: Vec<Option<String>> = conn.mget(&keys).await?;

        Ok(values
            .into_iter()
            .map(|v| v.and_then(|s| serde_json::from_str(&s).ok()))
            .collect())
    }

    /// Публикует обновление цены
    pub async fn publish_price_update(
        &self,
        symbol: &str,
        price: Decimal,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let message = serde_json::json!({
            "symbol": symbol,
            "price": price.to_string(),
            "timestamp": chrono::Utc::now().timestamp()
        });

        conn.publish("price_updates", message.to_string()).await?;
        Ok(())
    }
}
```

## Сервис хранилища торговой истории

Объединяем всё вместе в единый сервис:

```rust
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TradingHistoryStorage {
    trade_repo: TradeRepository,
    portfolio_repo: PortfolioRepository,
    price_cache: PriceCache,
    connection_pool: PgPool,
}

impl TradingHistoryStorage {
    pub async fn new(
        database_url: &str,
        redis_url: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Создаём пул соединений PostgreSQL
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(database_url)
            .await?;

        // Выполняем миграции
        run_migrations(&pool).await?;

        // Создаём Redis клиент
        let price_cache = PriceCache::new(redis_url)?;

        Ok(Self {
            trade_repo: TradeRepository::new(pool.clone()),
            portfolio_repo: PortfolioRepository::new(pool.clone()),
            price_cache,
            connection_pool: pool,
        })
    }

    /// Записывает новую сделку
    pub async fn record_trade(
        &self,
        symbol: &str,
        side: TradeSide,
        order_type: OrderType,
        quantity: Decimal,
        price: Decimal,
        notes: Option<&str>,
    ) -> Result<Trade, Box<dyn std::error::Error>> {
        let trade = self.trade_repo
            .create_trade(symbol, side, order_type, quantity, price, notes)
            .await?;

        // Обновляем кэш с последней ценой
        self.price_cache.set_price(symbol, price).await?;

        println!(
            "Записана сделка #{}: {} {} {} @ {}",
            trade.id,
            match side { TradeSide::Buy => "BUY", TradeSide::Sell => "SELL" },
            quantity,
            symbol,
            price
        );

        Ok(trade)
    }

    /// Исполняет сделку
    pub async fn execute_trade(
        &self,
        trade_id: i64,
        executed_quantity: Decimal,
        executed_price: Decimal,
        commission: Decimal,
    ) -> Result<Trade, Box<dyn std::error::Error>> {
        let trade = self.trade_repo
            .execute_trade(trade_id, executed_quantity, executed_price, commission)
            .await?;

        // Обновляем кэш и публикуем событие
        self.price_cache.set_price(&trade.symbol, executed_price).await?;
        self.price_cache.publish_price_update(&trade.symbol, executed_price).await?;

        println!(
            "Сделка #{} исполнена: {} @ {} (комиссия: {})",
            trade.id, executed_quantity, executed_price, commission
        );

        Ok(trade)
    }

    /// Получает текущую цену (сначала из кэша, потом из БД)
    pub async fn get_current_price(
        &self,
        symbol: &str,
    ) -> Result<Option<Decimal>, Box<dyn std::error::Error>> {
        // Сначала проверяем кэш
        if let Some(cached) = self.price_cache.get_price(symbol).await? {
            return Ok(Some(cached.price));
        }

        // Если в кэше нет, берём последнюю исполненную сделку
        let trades = self.trade_repo.get_trades_by_symbol(symbol, 1).await?;
        if let Some(trade) = trades.first() {
            if trade.status == TradeStatus::Filled {
                // Сохраняем в кэш для будущих запросов
                self.price_cache.set_price(symbol, trade.executed_price).await?;
                return Ok(Some(trade.executed_price));
            }
        }

        Ok(None)
    }

    /// Создаёт снимок портфеля
    pub async fn create_portfolio_snapshot(
        &self,
        cash_balance: Decimal,
        positions: Vec<(String, Decimal, Decimal)>, // (symbol, quantity, avg_price)
    ) -> Result<PortfolioSnapshot, Box<dyn std::error::Error>> {
        // Получаем текущие цены для всех позиций
        let symbols: Vec<&str> = positions.iter().map(|(s, _, _)| s.as_str()).collect();
        let prices = self.price_cache.get_prices(&symbols).await?;

        let mut total_value = cash_balance;
        let mut unrealized_pnl = Decimal::ZERO;

        let positions_with_prices: Vec<_> = positions
            .iter()
            .zip(prices.iter())
            .filter_map(|((symbol, qty, avg_price), cached_price)| {
                cached_price.as_ref().map(|cp| {
                    let current_value = *qty * cp.price;
                    let cost_basis = *qty * *avg_price;
                    let pnl = current_value - cost_basis;

                    total_value += current_value;
                    unrealized_pnl += pnl;

                    (symbol.clone(), *qty, *avg_price, cp.price, pnl)
                })
            })
            .collect();

        let snapshot = self.portfolio_repo
            .create_snapshot(total_value, cash_balance, unrealized_pnl, Decimal::ZERO)
            .await?;

        // Сохраняем позиции
        for (symbol, qty, avg_price, current_price, pnl) in positions_with_prices {
            self.portfolio_repo
                .add_position(snapshot.id, &symbol, qty, avg_price, current_price, pnl)
                .await?;
        }

        println!(
            "Создан снимок портфеля #{}: общая стоимость ${}, нереализованный PnL ${}",
            snapshot.id, total_value, unrealized_pnl
        );

        Ok(snapshot)
    }

    /// Генерирует отчёт по торговле за период
    pub async fn generate_trading_report(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<TradingReport, Box<dyn std::error::Error>> {
        let trades = self.trade_repo.get_trades_in_range(from, to).await?;
        let stats = self.trade_repo.get_trade_statistics(None).await?;

        let filled_trades: Vec<_> = trades
            .iter()
            .filter(|t| t.status == TradeStatus::Filled)
            .collect();

        let total_bought: Decimal = filled_trades
            .iter()
            .filter(|t| t.side == TradeSide::Buy)
            .map(|t| t.executed_quantity * t.executed_price)
            .sum();

        let total_sold: Decimal = filled_trades
            .iter()
            .filter(|t| t.side == TradeSide::Sell)
            .map(|t| t.executed_quantity * t.executed_price)
            .sum();

        let report = TradingReport {
            period_start: from,
            period_end: to,
            total_trades: trades.len() as i64,
            filled_trades: filled_trades.len() as i64,
            total_volume_bought: total_bought,
            total_volume_sold: total_sold,
            total_commission: stats.total_commission,
            net_flow: total_sold - total_bought,
        };

        Ok(report)
    }

    /// Получает статистику по символу
    pub async fn get_symbol_statistics(
        &self,
        symbol: &str,
    ) -> Result<TradeStatistics, Box<dyn std::error::Error>> {
        Ok(self.trade_repo.get_trade_statistics(Some(symbol)).await?)
    }

    /// Закрывает соединения
    pub async fn close(&self) {
        self.connection_pool.close().await;
        println!("Соединения с базами данных закрыты");
    }
}

#[derive(Debug, Clone)]
pub struct TradingReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_trades: i64,
    pub filled_trades: i64,
    pub total_volume_bought: Decimal,
    pub total_volume_sold: Decimal,
    pub total_commission: Decimal,
    pub net_flow: Decimal,
}

impl std::fmt::Display for TradingReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"
=== Торговый отчёт ===
Период: {} - {}
Всего сделок: {}
Исполнено: {}
Объём покупок: ${}
Объём продаж: ${}
Комиссии: ${}
Чистый поток: ${}
======================"#,
            self.period_start.format("%Y-%m-%d"),
            self.period_end.format("%Y-%m-%d"),
            self.total_trades,
            self.filled_trades,
            self.total_volume_bought,
            self.total_volume_sold,
            self.total_commission,
            self.net_flow
        )
    }
}
```

## Репозиторий для портфеля

```rust
pub struct PortfolioRepository {
    pool: PgPool,
}

impl PortfolioRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_snapshot(
        &self,
        total_value: Decimal,
        cash_balance: Decimal,
        unrealized_pnl: Decimal,
        realized_pnl: Decimal,
    ) -> Result<PortfolioSnapshot, sqlx::Error> {
        sqlx::query_as::<_, PortfolioSnapshot>(r#"
            INSERT INTO portfolio_snapshots
                (total_value_usd, cash_balance, unrealized_pnl, realized_pnl)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        "#)
        .bind(total_value)
        .bind(cash_balance)
        .bind(unrealized_pnl)
        .bind(realized_pnl)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn add_position(
        &self,
        snapshot_id: i64,
        symbol: &str,
        quantity: Decimal,
        average_price: Decimal,
        current_price: Decimal,
        unrealized_pnl: Decimal,
    ) -> Result<Position, sqlx::Error> {
        sqlx::query_as::<_, Position>(r#"
            INSERT INTO positions
                (snapshot_id, symbol, quantity, average_price, current_price, unrealized_pnl)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#)
        .bind(snapshot_id)
        .bind(symbol)
        .bind(quantity)
        .bind(average_price)
        .bind(current_price)
        .bind(unrealized_pnl)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_latest_snapshot(&self) -> Result<Option<PortfolioSnapshot>, sqlx::Error> {
        sqlx::query_as::<_, PortfolioSnapshot>(r#"
            SELECT * FROM portfolio_snapshots
            ORDER BY timestamp DESC
            LIMIT 1
        "#)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_snapshot_positions(
        &self,
        snapshot_id: i64,
    ) -> Result<Vec<Position>, sqlx::Error> {
        sqlx::query_as::<_, Position>(r#"
            SELECT * FROM positions
            WHERE snapshot_id = $1
        "#)
        .bind(snapshot_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_portfolio_history(
        &self,
        days: i32,
    ) -> Result<Vec<PortfolioSnapshot>, sqlx::Error> {
        sqlx::query_as::<_, PortfolioSnapshot>(r#"
            SELECT * FROM portfolio_snapshots
            WHERE timestamp >= NOW() - INTERVAL '1 day' * $1
            ORDER BY timestamp ASC
        "#)
        .bind(days)
        .fetch_all(&self.pool)
        .await
    }
}
```

## Пример использования

```rust
use rust_decimal_macros::dec;
use chrono::{Duration, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализируем хранилище
    let storage = TradingHistoryStorage::new(
        "postgres://user:password@localhost/trading",
        "redis://localhost:6379",
    ).await?;

    // Записываем несколько сделок
    let trade1 = storage.record_trade(
        "BTC/USDT",
        TradeSide::Buy,
        OrderType::Limit,
        dec!(0.5),
        dec!(42000.00),
        Some("Покупка на просадке"),
    ).await?;

    // Исполняем сделку
    storage.execute_trade(
        trade1.id,
        dec!(0.5),
        dec!(41950.00),
        dec!(10.49),
    ).await?;

    // Записываем ещё сделку
    let trade2 = storage.record_trade(
        "ETH/USDT",
        TradeSide::Buy,
        OrderType::Market,
        dec!(2.0),
        dec!(2800.00),
        None,
    ).await?;

    storage.execute_trade(
        trade2.id,
        dec!(2.0),
        dec!(2802.50),
        dec!(1.40),
    ).await?;

    // Продаём часть BTC
    let trade3 = storage.record_trade(
        "BTC/USDT",
        TradeSide::Sell,
        OrderType::Limit,
        dec!(0.25),
        dec!(43500.00),
        Some("Частичная фиксация прибыли"),
    ).await?;

    storage.execute_trade(
        trade3.id,
        dec!(0.25),
        dec!(43480.00),
        dec!(5.44),
    ).await?;

    // Создаём снимок портфеля
    let positions = vec![
        ("BTC/USDT".to_string(), dec!(0.25), dec!(41950.00)),
        ("ETH/USDT".to_string(), dec!(2.0), dec!(2802.50)),
    ];

    storage.create_portfolio_snapshot(dec!(10000.00), positions).await?;

    // Генерируем отчёт за последнюю неделю
    let report = storage.generate_trading_report(
        Utc::now() - Duration::days(7),
        Utc::now(),
    ).await?;

    println!("{}", report);

    // Получаем статистику по BTC
    let btc_stats = storage.get_symbol_statistics("BTC/USDT").await?;
    println!("\nСтатистика BTC/USDT:");
    println!("  Всего сделок: {}", btc_stats.total_trades);
    println!("  Покупок: {}", btc_stats.buy_trades);
    println!("  Продаж: {}", btc_stats.sell_trades);
    println!("  Общий объём: ${}", btc_stats.total_volume);

    // Закрываем соединения
    storage.close().await;

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Многоуровневая архитектура | PostgreSQL для персистентности, Redis для кэша |
| Connection Pool | Эффективное управление соединениями с БД |
| Миграции | Автоматическое создание схемы базы данных |
| Repository Pattern | Изоляция логики работы с данными |
| Кэширование | Быстрый доступ к часто используемым данным |
| Pub/Sub | Уведомления об изменениях в реальном времени |
| Транзакции | Атомарные операции для целостности данных |

## Практические упражнения

1. **Добавь поддержку SQLite**: Реализуй альтернативный репозиторий для SQLite, чтобы можно было использовать его в разработке без PostgreSQL.

2. **Реализуй агрегацию по времени**: Добавь метод для получения статистики сделок по часам/дням/неделям.

3. **Добавь подписку на обновления**: Используя Redis Pub/Sub, реализуй подписку на обновления цен в реальном времени.

4. **Экспорт в CSV**: Добавь функцию экспорта истории сделок в CSV файл для анализа в Excel.

## Домашнее задание

1. **Система алертов**: Добавь функциональность для создания алертов при достижении ценой определённого уровня. Храни алерты в Redis и проверяй их при каждом обновлении цены.

2. **Расчёт PnL**: Реализуй функцию расчёта реализованного PnL по методу FIFO (First In, First Out). При продаже актива нужно учитывать цену покупки самых старых позиций.

3. **Бэкап и восстановление**: Реализуй функции для создания бэкапа всех данных в JSON формате и восстановления из бэкапа.

4. **Мониторинг производительности**: Добавь логирование времени выполнения всех запросов к БД. Если запрос выполняется дольше 100мс, выводи предупреждение.

## Навигация

[← Предыдущий день](../242-database-performance-monitoring/ru.md) | [Следующий день →](../244-ohlcv-candles/ru.md)
