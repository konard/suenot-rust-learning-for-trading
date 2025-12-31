# Day 233: SeaORM: Async ORM for Rust

## Trading Analogy

Imagine you're managing a large trading platform. You have thousands of orders, multiple traders, various trading pairs, and the history of all trades. Storing all this in memory is impossible — you need a database. But how do you work with a database conveniently and safely in Rust?

**SeaORM** is like a professional back-office for your trading platform. Instead of writing raw SQL queries (which is like manually filling out paper reports), you work with typed Rust structures. SeaORM automatically:
- Validates types at compile time (you can't buy 0.5 shares if an integer is required)
- Generates efficient SQL queries (like an experienced analyst knows which reports to request)
- Works asynchronously (handles multiple requests simultaneously, like a modern exchange)

In the trading world, SeaORM is your reliable bridge between fast Rust code and persistent data storage.

## What is SeaORM?

SeaORM is an asynchronous ORM (Object-Relational Mapping) framework for Rust. It allows you to:

1. **Describe data models** as regular Rust structures
2. **Perform CRUD operations** without writing SQL manually
3. **Work asynchronously** with tokio or async-std
4. **Supports migrations** for database schema evolution
5. **Works with PostgreSQL, MySQL, SQLite**

## Adding SeaORM

Add to your `Cargo.toml`:

```toml
[dependencies]
sea-orm = { version = "0.12", features = ["sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
tokio = { version = "1", features = ["full"] }
```

## Defining Entities

### Order Model

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "orders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub trader_id: i64,
    pub symbol: String,
    pub side: String,        // "buy" or "sell"
    pub order_type: String,  // "market" or "limit"
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

### Trader Model

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

### Trade Model

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

## Connecting to the Database

```rust
use sea_orm::{Database, DatabaseConnection};

async fn connect_to_database() -> Result<DatabaseConnection, sea_orm::DbErr> {
    // Connect to PostgreSQL
    let database_url = "postgres://user:password@localhost/trading_db";
    let db = Database::connect(database_url).await?;

    println!("Connected to trading_db database");
    Ok(db)
}

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    let db = connect_to_database().await?;

    // Verify connection
    db.ping().await?;
    println!("Database is working!");

    Ok(())
}
```

## CRUD Operations for Trading Platform

### Creating an Order (Create)

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
        "Created order #{}: {} {} {} @ {}",
        order.id, side, quantity, symbol, price
    );

    Ok(order)
}

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    let db = connect_to_database().await?;

    // Create a BTC buy order
    let order = create_order(
        &db,
        1,
        "BTC/USD",
        "buy",
        Decimal::new(42500, 0),  // $42,500
        Decimal::new(1, 1),      // 0.1 BTC
    ).await?;

    println!("Order created: {:?}", order);

    Ok(())
}
```

### Reading Orders (Read)

```rust
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QueryOrder};

// Get order by ID
async fn get_order_by_id(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<Option<order::Model>, sea_orm::DbErr> {
    order::Entity::find_by_id(order_id).one(db).await
}

// Get all active orders for a trader
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

// Get orders by symbol
async fn get_orders_by_symbol(
    db: &DatabaseConnection,
    symbol: &str,
    side: &str,
) -> Result<Vec<order::Model>, sea_orm::DbErr> {
    order::Entity::find()
        .filter(order::Column::Symbol.eq(symbol))
        .filter(order::Column::Side.eq(side))
        .filter(order::Column::Status.eq("pending"))
        .order_by_asc(order::Column::Price) // Sort by price
        .all(db)
        .await
}

// Building the order book
async fn get_order_book(
    db: &DatabaseConnection,
    symbol: &str,
) -> Result<(Vec<order::Model>, Vec<order::Model>), sea_orm::DbErr> {
    // Buy orders (bids) — sorted by descending price
    let bids = order::Entity::find()
        .filter(order::Column::Symbol.eq(symbol))
        .filter(order::Column::Side.eq("buy"))
        .filter(order::Column::Status.eq("pending"))
        .order_by_desc(order::Column::Price)
        .limit(10)
        .all(db)
        .await?;

    // Sell orders (asks) — sorted by ascending price
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

    // Get the order book for BTC/USD
    let (bids, asks) = get_order_book(&db, "BTC/USD").await?;

    println!("\n📊 Order Book BTC/USD:");
    println!("--- ASKS (sell) ---");
    for ask in asks.iter().rev() {
        println!("  {} @ ${}", ask.quantity, ask.price);
    }
    println!("-------------------");
    println!("--- BIDS (buy) ---");
    for bid in &bids {
        println!("  {} @ ${}", bid.quantity, bid.price);
    }

    Ok(())
}
```

### Updating an Order (Update)

```rust
use sea_orm::{ActiveValue::Set, ActiveModelTrait, IntoActiveModel};

// Partial order fill
async fn fill_order_partially(
    db: &DatabaseConnection,
    order_id: i64,
    filled_amount: Decimal,
) -> Result<order::Model, sea_orm::DbErr> {
    let order = order::Entity::find_by_id(order_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(
            "Order not found".to_string()
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
        "Order #{} updated: filled {}/{}, status: {}",
        updated.id, updated.filled_quantity, updated.quantity, updated.status
    );

    Ok(updated)
}

// Cancel an order
async fn cancel_order(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<order::Model, sea_orm::DbErr> {
    let order = order::Entity::find_by_id(order_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(
            "Order not found".to_string()
        ))?;

    if order.status == "filled" {
        return Err(sea_orm::DbErr::Custom(
            "Cannot cancel a filled order".to_string()
        ));
    }

    let mut active_order = order.into_active_model();
    active_order.status = Set("cancelled".to_string());
    active_order.updated_at = Set(Utc::now().naive_utc());

    let updated = active_order.update(db).await?;

    println!("Order #{} cancelled", updated.id);

    Ok(updated)
}
```

### Deleting (Delete)

```rust
use sea_orm::{EntityTrait, ModelTrait};

// Delete old filled orders
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

    println!("Deleted {} old orders", result.rows_affected);

    Ok(result.rows_affected)
}

// Delete a specific order
async fn delete_order(
    db: &DatabaseConnection,
    order_id: i64,
) -> Result<(), sea_orm::DbErr> {
    let order = order::Entity::find_by_id(order_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(
            "Order not found".to_string()
        ))?;

    order.delete(db).await?;
    println!("Order #{} deleted", order_id);

    Ok(())
}
```

## Transactions for Atomic Operations

```rust
use sea_orm::{TransactionTrait, DatabaseTransaction};

// Execute a trade with a transaction
async fn execute_trade(
    db: &DatabaseConnection,
    buy_order_id: i64,
    sell_order_id: i64,
    price: Decimal,
    quantity: Decimal,
) -> Result<trade::Model, sea_orm::DbErr> {
    // Begin transaction
    let txn = db.begin().await?;

    // Get both orders
    let buy_order = order::Entity::find_by_id(buy_order_id)
        .one(&txn)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound("Buy order not found".into()))?;

    let sell_order = order::Entity::find_by_id(sell_order_id)
        .one(&txn)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound("Sell order not found".into()))?;

    // Update orders
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

    // Update trader balances
    let buyer = trader::Entity::find_by_id(buy_order.trader_id)
        .one(&txn)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound("Buyer not found".into()))?;

    let seller = trader::Entity::find_by_id(sell_order.trader_id)
        .one(&txn)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound("Seller not found".into()))?;

    let total = price * quantity;
    let fee = total * Decimal::new(1, 3); // 0.1% fee

    // Buyer: -USD, +BTC
    let mut buyer_active = buyer.into_active_model();
    buyer_active.balance_usd = Set(buyer_active.balance_usd.unwrap() - total - fee);
    buyer_active.balance_btc = Set(buyer_active.balance_btc.unwrap() + quantity);
    buyer_active.update(&txn).await?;

    // Seller: +USD, -BTC
    let mut seller_active = seller.into_active_model();
    seller_active.balance_usd = Set(seller_active.balance_usd.unwrap() + total - fee);
    seller_active.balance_btc = Set(seller_active.balance_btc.unwrap() - quantity);
    seller_active.update(&txn).await?;

    // Create trade record
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

    // Commit transaction
    txn.commit().await?;

    println!(
        "Trade #{} executed: {} {} @ ${} = ${}",
        trade.id, quantity, buy_order.symbol, price, total
    );

    Ok(trade)
}
```

## Complex Queries for Analytics

```rust
use sea_orm::{FromQueryResult, QuerySelect, Condition};

// Structure for aggregated data
#[derive(Debug, FromQueryResult)]
struct TradingStats {
    symbol: String,
    total_volume: Decimal,
    trade_count: i64,
    avg_price: Decimal,
    min_price: Decimal,
    max_price: Decimal,
}

// Get trading stats for a period
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
            "No data for the specified period".to_string()
        ))?;

    Ok(stats)
}

// Top traders by volume
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
    // Using raw SQL for complex JOIN
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

// Get portfolio positions with current prices
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

## Database Migrations

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

## Complete Example: Trading Engine

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

    // Place a limit order
    pub async fn place_limit_order(
        &self,
        trader_id: i64,
        symbol: &str,
        side: &str,
        price: Decimal,
        quantity: Decimal,
    ) -> Result<order::Model, sea_orm::DbErr> {
        // Check trader's balance
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
                format!("Insufficient funds: need {}, available {}", required, available)
            ));
        }

        // Create order
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

        // Try to match with existing orders
        self.match_orders(order.id).await?;

        // Return updated order
        order::Entity::find_by_id(order.id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Order not found".into()))
    }

    // Order matching engine
    async fn match_orders(&self, order_id: i64) -> Result<(), sea_orm::DbErr> {
        let order = order::Entity::find_by_id(order_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Order not found".into()))?;

        if order.status != "pending" {
            return Ok(());
        }

        let opposite_side = if order.side == "buy" { "sell" } else { "buy" };

        // Find matching counter orders
        let mut matching_orders = if order.side == "buy" {
            // For buy, find sells at price <= ours
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
            // For sell, find buys at price >= ours
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

            // Execute trade at maker's price (the earlier order)
            let trade_price = matching_order.price;

            let (buy_order_id, sell_order_id) = if order.side == "buy" {
                (order.id, matching_order.id)
            } else {
                (matching_order.id, order.id)
            };

            // Use transaction for atomic execution
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

        // ... (trade execution logic from previous example)

        txn.commit().await?;

        // Get the created trade
        trade::Entity::find()
            .filter(trade::Column::OrderId.eq(buy_order_id))
            .filter(trade::Column::CounterOrderId.eq(sell_order_id))
            .order_by_desc(trade::Column::ExecutedAt)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Trade not found".into()))
    }

    // Get current order book
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

    // Place orders
    let buy_order = engine.place_limit_order(
        1,
        "BTC/USD",
        "buy",
        Decimal::new(42000, 0),
        Decimal::new(1, 1),
    ).await?;

    println!("Order placed: {:?}", buy_order);

    // Get order book
    let order_book = engine.get_order_book("BTC/USD", 10).await?;

    println!("\n📊 Order Book:");
    println!("Bids: {} orders", order_book.bids.len());
    println!("Asks: {} orders", order_book.asks.len());

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| SeaORM | Async ORM for Rust with PostgreSQL, MySQL, SQLite support |
| Entity | Data model corresponding to a database table |
| ActiveModel | Model for creating and updating records |
| Relations | Relationships between entities (has_one, has_many, belongs_to) |
| Transactions | Atomic operations for data consistency |
| Migrations | Versioned database schema changes |
| Query Builder | Type-safe SQL query builder |

## Exercises

1. **Portfolio Model**: Create a `Portfolio` entity with a relationship to `Trader` and methods for calculating total portfolio value.

2. **Price History**: Implement a `PriceHistory` entity to store historical prices and a method to get OHLCV data for a period.

3. **Risk Limits**: Add daily volume limit checks when creating an order using aggregating queries.

4. **Query Optimization**: Using `explain` in PostgreSQL, optimize the order book query by adding necessary indexes.

## Homework

1. **Notification System**: Implement a `Notification` entity and a mechanism for creating notifications when orders are executed using SeaORM hooks.

2. **Operations Audit**: Create an `AuditLog` table to record all balance and order changes. Use database triggers or SeaORM middleware.

3. **Multi-currency Wallet**: Extend the `Trader` model by adding support for an arbitrary number of currencies through a separate `Balances` table with a many-to-one relationship.

4. **Backtesting API**: Create functions to load historical trade data and simulate trading strategies using SeaORM for persisting results.

## Navigation

[← Previous day](../232-diesel-orm/en.md) | [Next day →](../234-sqlx-async-sql/en.md)
