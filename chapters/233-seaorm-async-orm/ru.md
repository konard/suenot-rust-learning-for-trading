# День 233: SeaORM: Асинхронный ORM для Rust

## Аналогия из трейдинга

Представь, что ты управляешь крупной торговой платформой. У тебя есть тысячи ордеров, множество трейдеров, различные торговые пары и история всех сделок. Хранить всё это в памяти невозможно — нужна база данных. Но как работать с базой данных удобно и безопасно в Rust?

**SeaORM** — это как профессиональный бэк-офис для твоей торговой платформы. Вместо того чтобы писать сырые SQL-запросы (что похоже на ручное заполнение бумажных отчётов), ты работаешь с типизированными Rust-структурами. SeaORM автоматически:
- Проверяет типы на этапе компиляции (не купишь 0.5 акции, если нужно целое число)
- Генерирует эффективные SQL-запросы (как опытный аналитик знает, какие отчёты запросить)
- Работает асинхронно (обрабатывает множество запросов одновременно, как современная биржа)

В мире трейдинга SeaORM — это твой надёжный мост между быстрым Rust-кодом и персистентным хранилищем данных.

## Что такое SeaORM?

SeaORM — это асинхронный ORM (Object-Relational Mapping) фреймворк для Rust. Он позволяет:

1. **Описывать модели данных** как обычные Rust-структуры
2. **Выполнять CRUD-операции** без написания SQL вручную
3. **Работать асинхронно** с tokio или async-std
4. **Поддерживает миграции** для эволюции схемы базы данных
5. **Работает с PostgreSQL, MySQL, SQLite**

## Подключение SeaORM

Добавьте в `Cargo.toml`:

```toml
[dependencies]
sea-orm = { version = "0.12", features = ["sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
tokio = { version = "1", features = ["full"] }
```

## Определение сущностей (Entities)

### Модель ордера

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub trader_id: i64,
    pub symbol: String,
    pub side: String,        // "buy" или "sell"
    pub order_type: String,  // "market" или "limit"
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: String,      // "pending", "filled", "cancelled"
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::trader::Entity",
        from = "Column::TraderId",
        to = "super::trader::Column::Id"
    )]
    Trader,
    #[sea_orm(has_many = "super::trade::Entity")]
    Trades,
}

impl Related<super::trader::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Trader.def()
    }
}

impl Related<super::trade::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Trades.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### Модель трейдера

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "traders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub username: String,
    pub email: String,
    pub balance_usd: Decimal,
    pub balance_btc: Decimal,
    pub balance_eth: Decimal,
    pub is_active: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::order::Entity")]
    Orders,
    #[sea_orm(has_many = "super::portfolio::Entity")]
    Portfolio,
}

impl Related<super::order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Orders.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### Модель сделки (Trade)

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "trades")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub order_id: i64,
    pub counter_order_id: i64,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub total: Decimal,
    pub fee: Decimal,
    pub executed_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::order::Entity",
        from = "Column::OrderId",
        to = "super::order::Column::Id"
    )]
    Order,
}

impl Related<super::order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Order.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

## Подключение к базе данных

```rust
use sea_orm::{Database, DatabaseConnection};

async fn connect_to_database() -> Result<DatabaseConnection, sea_orm::DbErr> {
    // Подключение к PostgreSQL
    let database_url = "postgres://user:password@localhost/trading_db";
    let db = Database::connect(database_url).await?;

    println!("Подключено к базе данных trading_db");
    Ok(db)
}

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    let db = connect_to_database().await?;

    // Проверка подключения
    db.ping().await?;
    println!("База данных работает!");

    Ok(())
}
```

## CRUD-операции для торговой платформы

### Создание ордера (Create)

```rust
use sea_orm::{ActiveValue::Set, ActiveModelTrait};
use rust_decimal::Decimal;
use chrono::Utc;

async fn create_order(
    db: &DatabaseConnection,
    trader_id: i64,
    symbol: &str,
    side: &str,
    price: Decimal,
    quantity: Decimal,
) -> Result<order::Model, sea_orm::DbErr> {
    let now = Utc::now().naive_utc();

    let new_order = order::ActiveModel {
        trader_id: Set(trader_id),
        symbol: Set(symbol.to_string()),
        side: Set(side.to_string()),
        order_type: Set("limit".to_string()),
        price: Set(price),
        quantity: Set(quantity),
        filled_quantity: Set(Decimal::ZERO),
        status: Set("pending".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let order = new_order.insert(db).await?;

    println!(
        "Создан ордер #{}: {} {} {} @ {}",
        order.id, side, quantity, symbol, price
    );

    Ok(order)
}

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    let db = connect_to_database().await?;

    // Создаём ордер на покупку BTC
    let order = create_order(
        &db,
        1,
        "BTC/USD",
        "buy",
        Decimal::new(42500, 0),  // $42,500
        Decimal::new(1, 1),      // 0.1 BTC
    ).await?;

    println!("Ордер создан: {:?}", order);

    Ok(())
}
```

### Чтение ордеров (Read)

```rust
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QueryOrder};

// Получить ордер по ID
async fn get_order_by_id(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<Option<order::Model>, sea_orm::DbErr> {
    order::Entity::find_by_id(order_id).one(db).await
}

// Получить все активные ордера трейдера
async fn get_active_orders(
    db: &DatabaseConnection,
    trader_id: i64,
) -> Result<Vec<order::Model>, sea_orm::DbErr> {
    order::Entity::find()
        .filter(order::Column::TraderId.eq(trader_id))
        .filter(order::Column::Status.eq("pending"))
        .order_by_desc(order::Column::CreatedAt)
        .all(db)
        .await
}

// Получить ордера по символу
async fn get_orders_by_symbol(
    db: &DatabaseConnection,
    symbol: &str,
    side: &str,
) -> Result<Vec<order::Model>, sea_orm::DbErr> {
    order::Entity::find()
        .filter(order::Column::Symbol.eq(symbol))
        .filter(order::Column::Side.eq(side))
        .filter(order::Column::Status.eq("pending"))
        .order_by_asc(order::Column::Price) // Сортировка по цене
        .all(db)
        .await
}

// Построение стакана заявок
async fn get_order_book(
    db: &DatabaseConnection,
    symbol: &str,
) -> Result<(Vec<order::Model>, Vec<order::Model>), sea_orm::DbErr> {
    // Заявки на покупку (bids) — сортировка по убыванию цены
    let bids = order::Entity::find()
        .filter(order::Column::Symbol.eq(symbol))
        .filter(order::Column::Side.eq("buy"))
        .filter(order::Column::Status.eq("pending"))
        .order_by_desc(order::Column::Price)
        .limit(10)
        .all(db)
        .await?;

    // Заявки на продажу (asks) — сортировка по возрастанию цены
    let asks = order::Entity::find()
        .filter(order::Column::Symbol.eq(symbol))
        .filter(order::Column::Side.eq("sell"))
        .filter(order::Column::Status.eq("pending"))
        .order_by_asc(order::Column::Price)
        .limit(10)
        .all(db)
        .await?;

    Ok((bids, asks))
}

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    let db = connect_to_database().await?;

    // Получаем стакан заявок для BTC/USD
    let (bids, asks) = get_order_book(&db, "BTC/USD").await?;

    println!("\n📊 Стакан заявок BTC/USD:");
    println!("--- ASKS (продажа) ---");
    for ask in asks.iter().rev() {
        println!("  {} @ ${}", ask.quantity, ask.price);
    }
    println!("----------------------");
    println!("--- BIDS (покупка) ---");
    for bid in &bids {
        println!("  {} @ ${}", bid.quantity, bid.price);
    }

    Ok(())
}
```

### Обновление ордера (Update)

```rust
use sea_orm::{ActiveValue::Set, ActiveModelTrait, IntoActiveModel};

// Частичное исполнение ордера
async fn fill_order_partially(
    db: &DatabaseConnection,
    order_id: i64,
    filled_amount: Decimal,
) -> Result<order::Model, sea_orm::DbErr> {
    let order = order::Entity::find_by_id(order_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(
            "Ордер не найден".to_string()
        ))?;

    let new_filled = order.filled_quantity + filled_amount;
    let new_status = if new_filled >= order.quantity {
        "filled"
    } else {
        "partial"
    };

    let mut active_order = order.into_active_model();
    active_order.filled_quantity = Set(new_filled);
    active_order.status = Set(new_status.to_string());
    active_order.updated_at = Set(Utc::now().naive_utc());

    let updated = active_order.update(db).await?;

    println!(
        "Ордер #{} обновлён: исполнено {}/{}, статус: {}",
        updated.id, updated.filled_quantity, updated.quantity, updated.status
    );

    Ok(updated)
}

// Отмена ордера
async fn cancel_order(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<order::Model, sea_orm::DbErr> {
    let order = order::Entity::find_by_id(order_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(
            "Ордер не найден".to_string()
        ))?;

    if order.status == "filled" {
        return Err(sea_orm::DbErr::Custom(
            "Нельзя отменить исполненный ордер".to_string()
        ));
    }

    let mut active_order = order.into_active_model();
    active_order.status = Set("cancelled".to_string());
    active_order.updated_at = Set(Utc::now().naive_utc());

    let updated = active_order.update(db).await?;

    println!("Ордер #{} отменён", updated.id);

    Ok(updated)
}
```

### Удаление (Delete)

```rust
use sea_orm::{EntityTrait, ModelTrait};

// Удаление старых исполненных ордеров
async fn delete_old_filled_orders(
    db: &DatabaseConnection,
    days_old: i64,
) -> Result<u64, sea_orm::DbErr> {
    let cutoff_date = Utc::now().naive_utc() - chrono::Duration::days(days_old);

    let result = order::Entity::delete_many()
        .filter(order::Column::Status.eq("filled"))
        .filter(order::Column::UpdatedAt.lt(cutoff_date))
        .exec(db)
        .await?;

    println!("Удалено {} старых ордеров", result.rows_affected);

    Ok(result.rows_affected)
}

// Удаление конкретного ордера
async fn delete_order(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<(), sea_orm::DbErr> {
    let order = order::Entity::find_by_id(order_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(
            "Ордер не найден".to_string()
        ))?;

    order.delete(db).await?;
    println!("Ордер #{} удалён", order_id);

    Ok(())
}
```

## Транзакции для атомарных операций

```rust
use sea_orm::{TransactionTrait, DatabaseTransaction};

// Исполнение сделки с транзакцией
async fn execute_trade(
    db: &DatabaseConnection,
    buy_order_id: i64,
    sell_order_id: i64,
    price: Decimal,
    quantity: Decimal,
) -> Result<trade::Model, sea_orm::DbErr> {
    // Начинаем транзакцию
    let txn = db.begin().await?;

    // Получаем оба ордера
    let buy_order = order::Entity::find_by_id(buy_order_id)
        .one(&txn)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound("Buy order not found".into()))?;

    let sell_order = order::Entity::find_by_id(sell_order_id)
        .one(&txn)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound("Sell order not found".into()))?;

    // Обновляем ордера
    let mut buy_active = buy_order.clone().into_active_model();
    buy_active.filled_quantity = Set(buy_order.filled_quantity + quantity);
    buy_active.status = Set(if buy_order.filled_quantity + quantity >= buy_order.quantity {
        "filled"
    } else {
        "partial"
    }.to_string());
    buy_active.updated_at = Set(Utc::now().naive_utc());
    buy_active.update(&txn).await?;

    let mut sell_active = sell_order.clone().into_active_model();
    sell_active.filled_quantity = Set(sell_order.filled_quantity + quantity);
    sell_active.status = Set(if sell_order.filled_quantity + quantity >= sell_order.quantity {
        "filled"
    } else {
        "partial"
    }.to_string());
    sell_active.updated_at = Set(Utc::now().naive_utc());
    sell_active.update(&txn).await?;

    // Обновляем балансы трейдеров
    let buyer = trader::Entity::find_by_id(buy_order.trader_id)
        .one(&txn)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound("Buyer not found".into()))?;

    let seller = trader::Entity::find_by_id(sell_order.trader_id)
        .one(&txn)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound("Seller not found".into()))?;

    let total = price * quantity;
    let fee = total * Decimal::new(1, 3); // 0.1% комиссия

    // Покупатель: -USD, +BTC
    let mut buyer_active = buyer.into_active_model();
    buyer_active.balance_usd = Set(buyer_active.balance_usd.unwrap() - total - fee);
    buyer_active.balance_btc = Set(buyer_active.balance_btc.unwrap() + quantity);
    buyer_active.update(&txn).await?;

    // Продавец: +USD, -BTC
    let mut seller_active = seller.into_active_model();
    seller_active.balance_usd = Set(seller_active.balance_usd.unwrap() + total - fee);
    seller_active.balance_btc = Set(seller_active.balance_btc.unwrap() - quantity);
    seller_active.update(&txn).await?;

    // Создаём запись о сделке
    let new_trade = trade::ActiveModel {
        order_id: Set(buy_order_id),
        counter_order_id: Set(sell_order_id),
        symbol: Set(buy_order.symbol.clone()),
        price: Set(price),
        quantity: Set(quantity),
        total: Set(total),
        fee: Set(fee),
        executed_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let trade = new_trade.insert(&txn).await?;

    // Коммитим транзакцию
    txn.commit().await?;

    println!(
        "Сделка #{} исполнена: {} {} @ ${} = ${}",
        trade.id, quantity, buy_order.symbol, price, total
    );

    Ok(trade)
}
```

## Сложные запросы для аналитики

```rust
use sea_orm::{FromQueryResult, QuerySelect, Condition};

// Структура для агрегированных данных
#[derive(Debug, FromQueryResult)]
struct TradingStats {
    symbol: String,
    total_volume: Decimal,
    trade_count: i64,
    avg_price: Decimal,
    min_price: Decimal,
    max_price: Decimal,
}

// Получение статистики торгов за период
async fn get_trading_stats(
    db: &DatabaseConnection,
    symbol: &str,
    from_date: DateTime,
    to_date: DateTime,
) -> Result<TradingStats, sea_orm::DbErr> {
    use sea_orm::sea_query::{Expr, Func};

    let stats = trade::Entity::find()
        .select_only()
        .column_as(trade::Column::Symbol, "symbol")
        .column_as(
            Expr::col(trade::Column::Quantity).sum(),
            "total_volume"
        )
        .column_as(Expr::cust("COUNT(*)"), "trade_count")
        .column_as(
            Expr::col(trade::Column::Price).avg(),
            "avg_price"
        )
        .column_as(
            Expr::col(trade::Column::Price).min(),
            "min_price"
        )
        .column_as(
            Expr::col(trade::Column::Price).max(),
            "max_price"
        )
        .filter(trade::Column::Symbol.eq(symbol))
        .filter(trade::Column::ExecutedAt.between(from_date, to_date))
        .group_by(trade::Column::Symbol)
        .into_model::<TradingStats>()
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(
            "Нет данных за указанный период".to_string()
        ))?;

    Ok(stats)
}

// Топ трейдеров по объёму
#[derive(Debug, FromQueryResult)]
struct TopTrader {
    trader_id: i64,
    username: String,
    total_volume: Decimal,
    trade_count: i64,
}

async fn get_top_traders(
    db: &DatabaseConnection,
    limit: u64,
) -> Result<Vec<TopTrader>, sea_orm::DbErr> {
    // Используем raw SQL для сложного JOIN
    let traders: Vec<TopTrader> = TopTrader::find_by_statement(
        sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT
                t.id as trader_id,
                t.username,
                COALESCE(SUM(tr.total), 0) as total_volume,
                COUNT(tr.id) as trade_count
            FROM traders t
            LEFT JOIN orders o ON t.id = o.trader_id
            LEFT JOIN trades tr ON o.id = tr.order_id
            WHERE t.is_active = true
            GROUP BY t.id, t.username
            ORDER BY total_volume DESC
            LIMIT $1
            "#,
            vec![limit.into()],
        )
    )
    .all(db)
    .await?;

    Ok(traders)
}

// Получение позиций портфеля с текущими ценами
#[derive(Debug, FromQueryResult)]
struct PortfolioPosition {
    symbol: String,
    quantity: Decimal,
    avg_price: Decimal,
    current_price: Decimal,
    pnl: Decimal,
    pnl_percent: Decimal,
}

async fn get_portfolio_with_pnl(
    db: &DatabaseConnection,
    trader_id: i64,
) -> Result<Vec<PortfolioPosition>, sea_orm::DbErr> {
    let positions: Vec<PortfolioPosition> = PortfolioPosition::find_by_statement(
        sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            WITH latest_prices AS (
                SELECT DISTINCT ON (symbol)
                    symbol,
                    price as current_price
                FROM trades
                ORDER BY symbol, executed_at DESC
            )
            SELECT
                p.symbol,
                p.quantity,
                p.avg_price,
                COALESCE(lp.current_price, p.avg_price) as current_price,
                (COALESCE(lp.current_price, p.avg_price) - p.avg_price) * p.quantity as pnl,
                ((COALESCE(lp.current_price, p.avg_price) - p.avg_price) / p.avg_price * 100) as pnl_percent
            FROM portfolio p
            LEFT JOIN latest_prices lp ON p.symbol = lp.symbol
            WHERE p.trader_id = $1 AND p.quantity > 0
            ORDER BY pnl DESC
            "#,
            vec![trader_id.into()],
        )
    )
    .all(db)
    .await?;

    Ok(positions)
}
```

## Миграции базы данных

```rust
// migrations/m20231201_000001_create_traders_table.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Traders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Traders::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Traders::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Traders::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Traders::BalanceUsd)
                            .decimal()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Traders::BalanceBtc)
                            .decimal()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Traders::BalanceEth)
                            .decimal()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Traders::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Traders::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Traders::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Traders {
    Table,
    Id,
    Username,
    Email,
    BalanceUsd,
    BalanceBtc,
    BalanceEth,
    IsActive,
    CreatedAt,
}
```

## Полный пример: Торговый движок

```rust
use sea_orm::{Database, DatabaseConnection, EntityTrait, ActiveModelTrait,
              QueryFilter, ColumnTrait, TransactionTrait, ActiveValue::Set};
use rust_decimal::Decimal;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

mod entities {
    pub mod order;
    pub mod trader;
    pub mod trade;
}

use entities::*;

struct TradingEngine {
    db: DatabaseConnection,
}

impl TradingEngine {
    pub async fn new(database_url: &str) -> Result<Self, sea_orm::DbErr> {
        let db = Database::connect(database_url).await?;
        Ok(Self { db })
    }

    // Размещение лимитного ордера
    pub async fn place_limit_order(
        &self,
        trader_id: i64,
        symbol: &str,
        side: &str,
        price: Decimal,
        quantity: Decimal,
    ) -> Result<order::Model, sea_orm::DbErr> {
        // Проверяем баланс трейдера
        let trader = trader::Entity::find_by_id(trader_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Trader not found".into()))?;

        let required = if side == "buy" {
            price * quantity
        } else {
            quantity
        };

        let available = if side == "buy" {
            trader.balance_usd
        } else {
            if symbol.starts_with("BTC") {
                trader.balance_btc
            } else {
                trader.balance_eth
            }
        };

        if available < required {
            return Err(sea_orm::DbErr::Custom(
                format!("Недостаточно средств: нужно {}, доступно {}", required, available)
            ));
        }

        // Создаём ордер
        let now = Utc::now().naive_utc();
        let new_order = order::ActiveModel {
            trader_id: Set(trader_id),
            symbol: Set(symbol.to_string()),
            side: Set(side.to_string()),
            order_type: Set("limit".to_string()),
            price: Set(price),
            quantity: Set(quantity),
            filled_quantity: Set(Decimal::ZERO),
            status: Set("pending".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let order = new_order.insert(&self.db).await?;

        // Пытаемся сопоставить с существующими ордерами
        self.match_orders(order.id).await?;

        // Возвращаем обновлённый ордер
        order::Entity::find_by_id(order.id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Order not found".into()))
    }

    // Сопоставление ордеров (matching engine)
    async fn match_orders(&self, order_id: i64) -> Result<(), sea_orm::DbErr> {
        let order = order::Entity::find_by_id(order_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Order not found".into()))?;

        if order.status != "pending" {
            return Ok(());
        }

        let opposite_side = if order.side == "buy" { "sell" } else { "buy" };

        // Находим подходящие встречные ордера
        let mut matching_orders = if order.side == "buy" {
            // Для покупки ищем продажи по цене <= нашей
            order::Entity::find()
                .filter(order::Column::Symbol.eq(&order.symbol))
                .filter(order::Column::Side.eq(opposite_side))
                .filter(order::Column::Status.eq("pending"))
                .filter(order::Column::Price.lte(order.price))
                .order_by_asc(order::Column::Price)
                .order_by_asc(order::Column::CreatedAt)
                .all(&self.db)
                .await?
        } else {
            // Для продажи ищем покупки по цене >= нашей
            order::Entity::find()
                .filter(order::Column::Symbol.eq(&order.symbol))
                .filter(order::Column::Side.eq(opposite_side))
                .filter(order::Column::Status.eq("pending"))
                .filter(order::Column::Price.gte(order.price))
                .order_by_desc(order::Column::Price)
                .order_by_asc(order::Column::CreatedAt)
                .all(&self.db)
                .await?
        };

        let mut remaining_quantity = order.quantity - order.filled_quantity;

        for matching_order in matching_orders {
            if remaining_quantity <= Decimal::ZERO {
                break;
            }

            let match_quantity = remaining_quantity.min(
                matching_order.quantity - matching_order.filled_quantity
            );

            // Исполняем сделку по цене мейкера (того, чей ордер был раньше)
            let trade_price = matching_order.price;

            let (buy_order_id, sell_order_id) = if order.side == "buy" {
                (order.id, matching_order.id)
            } else {
                (matching_order.id, order.id)
            };

            // Используем транзакцию для атомарного исполнения
            self.execute_trade_internal(
                buy_order_id,
                sell_order_id,
                trade_price,
                match_quantity,
            ).await?;

            remaining_quantity -= match_quantity;
        }

        Ok(())
    }

    async fn execute_trade_internal(
        &self,
        buy_order_id: i64,
        sell_order_id: i64,
        price: Decimal,
        quantity: Decimal,
    ) -> Result<trade::Model, sea_orm::DbErr> {
        let txn = self.db.begin().await?;

        // ... (логика исполнения сделки из предыдущего примера)

        txn.commit().await?;

        // Получаем созданную сделку
        trade::Entity::find()
            .filter(trade::Column::OrderId.eq(buy_order_id))
            .filter(trade::Column::CounterOrderId.eq(sell_order_id))
            .order_by_desc(trade::Column::ExecutedAt)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Trade not found".into()))
    }

    // Получение текущего стакана
    pub async fn get_order_book(
        &self,
        symbol: &str,
        depth: u64,
    ) -> Result<OrderBook, sea_orm::DbErr> {
        let bids = order::Entity::find()
            .filter(order::Column::Symbol.eq(symbol))
            .filter(order::Column::Side.eq("buy"))
            .filter(order::Column::Status.eq("pending"))
            .order_by_desc(order::Column::Price)
            .limit(depth)
            .all(&self.db)
            .await?;

        let asks = order::Entity::find()
            .filter(order::Column::Symbol.eq(symbol))
            .filter(order::Column::Side.eq("sell"))
            .filter(order::Column::Status.eq("pending"))
            .order_by_asc(order::Column::Price)
            .limit(depth)
            .all(&self.db)
            .await?;

        Ok(OrderBook { bids, asks })
    }
}

struct OrderBook {
    bids: Vec<order::Model>,
    asks: Vec<order::Model>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = TradingEngine::new("postgres://user:pass@localhost/trading").await?;

    // Размещаем ордера
    let buy_order = engine.place_limit_order(
        1,
        "BTC/USD",
        "buy",
        Decimal::new(42000, 0),
        Decimal::new(1, 1),
    ).await?;

    println!("Размещён ордер: {:?}", buy_order);

    // Получаем стакан
    let order_book = engine.get_order_book("BTC/USD", 10).await?;

    println!("\n📊 Стакан заявок:");
    println!("Bids: {} ордеров", order_book.bids.len());
    println!("Asks: {} ордеров", order_book.asks.len());

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| SeaORM | Асинхронный ORM для Rust с поддержкой PostgreSQL, MySQL, SQLite |
| Entity | Модель данных, соответствующая таблице в базе данных |
| ActiveModel | Модель для создания и обновления записей |
| Relations | Связи между сущностями (has_one, has_many, belongs_to) |
| Transactions | Атомарные операции для согласованности данных |
| Migrations | Версионированные изменения схемы базы данных |
| Query Builder | Типобезопасный построитель SQL-запросов |

## Упражнения

1. **Модель портфеля**: Создай сущность `Portfolio` со связью с `Trader` и методами для расчёта общей стоимости портфеля.

2. **История цен**: Реализуй сущность `PriceHistory` для хранения исторических цен и метод получения OHLCV-данных за период.

3. **Лимиты риска**: Добавь проверку дневных лимитов на объём торгов при создании ордера с использованием агрегирующих запросов.

4. **Оптимизация запросов**: Используя `explain` в PostgreSQL, оптимизируй запрос получения стакана заявок, добавив необходимые индексы.

## Домашнее задание

1. **Система уведомлений**: Реализуй сущность `Notification` и механизм создания уведомлений при исполнении ордеров с использованием хуков SeaORM.

2. **Аудит операций**: Создай таблицу `AuditLog` для записи всех изменений балансов и ордеров. Используй триггеры базы данных или middleware SeaORM.

3. **Мультивалютный кошелёк**: Расширь модель `Trader`, добавив поддержку произвольного количества валют через отдельную таблицу `Balances` со связью many-to-one.

4. **API для бэктестинга**: Создай функции для загрузки исторических данных о сделках и симуляции торговых стратегий с использованием SeaORM для персистентности результатов.

## Навигация

[← Предыдущий день](../232-diesel-orm/ru.md) | [Следующий день →](../234-sqlx-async-sql/ru.md)
