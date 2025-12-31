# День 232: Diesel: Модели и Схемы

## Аналогия из трейдинга

Представь, что ты управляешь торговым терминалом. Каждый ордер, каждая сделка, каждое изменение баланса — всё это нужно где-то хранить. В реальном мире трейдинга данные хранятся в базах данных: история ордеров, портфели пользователей, котировки инструментов.

**Схема базы данных** — это как структура биржевого стакана: она определяет, какие поля есть у каждого ордера (цена, количество, направление, время). **Модель** — это Rust-структура, которая представляет одну запись из таблицы, как один конкретный ордер в стакане.

Diesel — это ORM (Object-Relational Mapping) для Rust, который позволяет:
- Описывать структуру базы данных на Rust
- Безопасно выполнять SQL-запросы с проверкой на этапе компиляции
- Работать с данными как с обычными Rust-структурами

## Что такое Diesel?

Diesel — это быстрый и безопасный ORM для Rust. Его главные преимущества:

1. **Проверка запросов на этапе компиляции** — ошибки в SQL обнаруживаются до запуска программы
2. **Нулевые накладные расходы** — производительность сравнима с ручным написанием SQL
3. **Типобезопасность** — невозможно случайно перепутать типы данных

## Установка и настройка

Добавим Diesel в проект:

```toml
# Cargo.toml
[dependencies]
diesel = { version = "2.1", features = ["postgres", "chrono"] }
dotenvy = "0.15"
chrono = { version = "0.4", features = ["serde"] }

[dependencies.diesel_migrations]
version = "2.1"
```

Установим CLI-инструмент Diesel:

```bash
cargo install diesel_cli --no-default-features --features postgres
```

Создадим файл `.env` с подключением к базе данных:

```bash
DATABASE_URL=postgres://trader:password@localhost/trading_db
```

Инициализируем проект:

```bash
diesel setup
diesel migration generate create_orders
```

## Создание схемы базы данных

### Миграция: создание таблицы ордеров

```sql
-- migrations/2024-01-01-000001_create_orders/up.sql
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(4) NOT NULL CHECK (side IN ('buy', 'sell')),
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_symbol ON orders(symbol);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);
```

```sql
-- migrations/2024-01-01-000001_create_orders/down.sql
DROP TABLE orders;
```

Применяем миграцию:

```bash
diesel migration run
```

## Генерация схемы

После миграции Diesel автоматически генерирует файл `src/schema.rs`:

```rust
// src/schema.rs (автоматически генерируется Diesel)
diesel::table! {
    orders (id) {
        id -> Int4,
        symbol -> Varchar,
        side -> Varchar,
        price -> Numeric,
        quantity -> Numeric,
        status -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
```

## Создание моделей

Модели — это Rust-структуры, которые соответствуют записям в таблице:

```rust
// src/models.rs
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;

// Модель для чтения данных из БД
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    pub id: i32,
    pub symbol: String,
    pub side: String,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Модель для создания новых записей
#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::orders)]
pub struct NewOrder<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub status: &'a str,
}

// Модель для обновления записей
#[derive(AsChangeset, Debug)]
#[diesel(table_name = crate::schema::orders)]
pub struct OrderUpdate {
    pub status: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}
```

## Атрибуты derive для моделей

| Атрибут | Назначение |
|---------|------------|
| `Queryable` | Позволяет читать данные из БД в структуру |
| `Selectable` | Автоматический выбор полей для SELECT |
| `Insertable` | Позволяет вставлять структуру в БД |
| `AsChangeset` | Позволяет обновлять записи |
| `Identifiable` | Указывает первичный ключ для связей |
| `Associations` | Определяет связи между таблицами |

## Практический пример: Торговая система

### Полная структура проекта

```rust
// src/main.rs
mod schema;
mod models;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenvy::dotenv;
use std::env;
use bigdecimal::BigDecimal;
use std::str::FromStr;

use models::{Order, NewOrder, OrderUpdate};
use schema::orders;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

// Создание нового ордера
pub fn create_order(
    conn: &mut PgConnection,
    symbol: &str,
    side: &str,
    price: &str,
    quantity: &str,
) -> Order {
    let new_order = NewOrder {
        symbol,
        side,
        price: BigDecimal::from_str(price).unwrap(),
        quantity: BigDecimal::from_str(quantity).unwrap(),
        status: "pending",
    };

    diesel::insert_into(orders::table)
        .values(&new_order)
        .returning(Order::as_returning())
        .get_result(conn)
        .expect("Error saving new order")
}

// Получение всех активных ордеров по символу
pub fn get_active_orders(
    conn: &mut PgConnection,
    trading_symbol: &str,
) -> Vec<Order> {
    orders::table
        .filter(orders::symbol.eq(trading_symbol))
        .filter(orders::status.eq("pending"))
        .order(orders::created_at.desc())
        .load::<Order>(conn)
        .expect("Error loading orders")
}

// Обновление статуса ордера
pub fn update_order_status(
    conn: &mut PgConnection,
    order_id: i32,
    new_status: &str,
) -> Order {
    diesel::update(orders::table.find(order_id))
        .set(OrderUpdate {
            status: Some(new_status.to_string()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
        })
        .returning(Order::as_returning())
        .get_result(conn)
        .expect("Error updating order")
}

// Удаление исполненных ордеров старше 30 дней
pub fn cleanup_old_orders(conn: &mut PgConnection) -> usize {
    use chrono::{Utc, Duration};

    let cutoff = Utc::now().naive_utc() - Duration::days(30);

    diesel::delete(
        orders::table
            .filter(orders::status.eq("filled"))
            .filter(orders::created_at.lt(cutoff))
    )
    .execute(conn)
    .expect("Error deleting old orders")
}

fn main() {
    let conn = &mut establish_connection();

    // Создаём новые ордера
    println!("=== Создание ордеров ===");

    let buy_order = create_order(
        conn, "BTC/USDT", "buy", "42000.50", "0.5"
    );
    println!("Создан ордер на покупку: {:?}", buy_order);

    let sell_order = create_order(
        conn, "BTC/USDT", "sell", "43000.00", "0.25"
    );
    println!("Создан ордер на продажу: {:?}", sell_order);

    // Получаем активные ордера
    println!("\n=== Активные ордера BTC/USDT ===");
    let active = get_active_orders(conn, "BTC/USDT");
    for order in &active {
        println!(
            "ID: {}, {} {} @ {} - Статус: {}",
            order.id, order.side, order.quantity, order.price, order.status
        );
    }

    // Исполняем ордер
    println!("\n=== Исполнение ордера ===");
    let filled = update_order_status(conn, buy_order.id, "filled");
    println!("Ордер исполнен: {:?}", filled);

    // Статистика
    println!("\n=== Статистика ===");
    let total_orders: i64 = orders::table
        .count()
        .get_result(conn)
        .unwrap();
    println!("Всего ордеров в системе: {}", total_orders);
}
```

## Расширенный пример: Портфель и позиции

### Добавляем таблицу позиций

```sql
-- migrations/2024-01-02-000001_create_positions/up.sql
CREATE TABLE positions (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL UNIQUE,
    quantity DECIMAL(20, 8) NOT NULL DEFAULT 0,
    avg_price DECIMAL(20, 8) NOT NULL DEFAULT 0,
    unrealized_pnl DECIMAL(20, 8) NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE trades (
    id SERIAL PRIMARY KEY,
    order_id INTEGER REFERENCES orders(id),
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(4) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    commission DECIMAL(20, 8) NOT NULL DEFAULT 0,
    executed_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

### Модели со связями

```rust
// src/models.rs (расширенная версия)
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use crate::schema::{orders, positions, trades};

#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: i32,
    pub symbol: String,
    pub side: String,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = positions)]
pub struct Position {
    pub id: i32,
    pub symbol: String,
    pub quantity: BigDecimal,
    pub avg_price: BigDecimal,
    pub unrealized_pnl: BigDecimal,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Debug)]
#[diesel(table_name = positions)]
pub struct PositionUpdate {
    pub symbol: String,
    pub quantity: BigDecimal,
    pub avg_price: BigDecimal,
    pub unrealized_pnl: BigDecimal,
    pub updated_at: NaiveDateTime,
}

// Сделка связана с ордером
#[derive(Queryable, Selectable, Associations, Debug, Clone)]
#[diesel(belongs_to(Order))]
#[diesel(table_name = trades)]
pub struct Trade {
    pub id: i32,
    pub order_id: Option<i32>,
    pub symbol: String,
    pub side: String,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub commission: BigDecimal,
    pub executed_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = trades)]
pub struct NewTrade {
    pub order_id: Option<i32>,
    pub symbol: String,
    pub side: String,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub commission: BigDecimal,
}
```

### Сложные запросы

```rust
use diesel::prelude::*;
use diesel::dsl::sum;
use bigdecimal::BigDecimal;

// Получение сделок по ордеру с использованием связей
pub fn get_trades_for_order(
    conn: &mut PgConnection,
    order: &Order,
) -> Vec<Trade> {
    Trade::belonging_to(order)
        .select(Trade::as_select())
        .load(conn)
        .expect("Error loading trades")
}

// Агрегация: общий объём торгов по символу
pub fn get_total_volume(
    conn: &mut PgConnection,
    trading_symbol: &str,
) -> Option<BigDecimal> {
    use crate::schema::trades::dsl::*;

    trades
        .filter(symbol.eq(trading_symbol))
        .select(sum(quantity))
        .first::<Option<BigDecimal>>(conn)
        .expect("Error calculating volume")
}

// Транзакция: исполнение ордера с созданием сделки
pub fn execute_order(
    conn: &mut PgConnection,
    order_id: i32,
    exec_price: BigDecimal,
    exec_quantity: BigDecimal,
    commission_rate: BigDecimal,
) -> Result<Trade, diesel::result::Error> {
    conn.transaction(|conn| {
        // Получаем ордер
        let order = orders::table
            .find(order_id)
            .first::<Order>(conn)?;

        // Создаём сделку
        let commission = &exec_price * &exec_quantity * &commission_rate;
        let new_trade = NewTrade {
            order_id: Some(order_id),
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            price: exec_price,
            quantity: exec_quantity.clone(),
            commission,
        };

        let trade = diesel::insert_into(trades::table)
            .values(&new_trade)
            .returning(Trade::as_returning())
            .get_result(conn)?;

        // Обновляем статус ордера
        diesel::update(orders::table.find(order_id))
            .set(orders::status.eq("filled"))
            .execute(conn)?;

        // Обновляем позицию
        update_position(conn, &order.symbol, &order.side, &exec_quantity)?;

        Ok(trade)
    })
}

fn update_position(
    conn: &mut PgConnection,
    symbol: &str,
    side: &str,
    quantity: &BigDecimal,
) -> Result<(), diesel::result::Error> {
    use crate::schema::positions::dsl;

    let existing = dsl::positions
        .filter(dsl::symbol.eq(symbol))
        .first::<Position>(conn)
        .optional()?;

    match existing {
        Some(pos) => {
            let new_qty = if side == "buy" {
                &pos.quantity + quantity
            } else {
                &pos.quantity - quantity
            };

            diesel::update(dsl::positions.find(pos.id))
                .set(dsl::quantity.eq(new_qty))
                .execute(conn)?;
        }
        None => {
            let new_pos = PositionUpdate {
                symbol: symbol.to_string(),
                quantity: quantity.clone(),
                avg_price: BigDecimal::from(0),
                unrealized_pnl: BigDecimal::from(0),
                updated_at: chrono::Utc::now().naive_utc(),
            };

            diesel::insert_into(dsl::positions)
                .values(&new_pos)
                .execute(conn)?;
        }
    }

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Схема (Schema) | Описание структуры таблиц БД на Rust |
| Модель | Rust-структура, соответствующая записи в таблице |
| `Queryable` | Чтение данных из БД |
| `Insertable` | Вставка данных в БД |
| `AsChangeset` | Обновление данных в БД |
| `Associations` | Связи между таблицами |
| Миграции | Версионирование изменений БД |
| Транзакции | Атомарные операции над несколькими таблицами |

## Упражнения

1. **Таблица котировок**: Создай миграцию и модели для таблицы `quotes` с полями: `symbol`, `bid_price`, `ask_price`, `bid_size`, `ask_size`, `timestamp`. Реализуй функции для записи и чтения последних котировок.

2. **История баланса**: Создай таблицу `balance_history` для отслеживания изменений баланса пользователя. Добавь функцию, которая записывает изменение баланса при каждой сделке.

3. **Пагинация**: Реализуй функцию `get_orders_paginated(page: i64, per_page: i64)`, которая возвращает ордера с пагинацией и общее количество страниц.

4. **Статистика по символам**: Напиши функцию, которая возвращает агрегированную статистику по каждому символу: количество ордеров, общий объём, средняя цена.

## Домашнее задание

1. **Полная торговая система**: Создай модуль с таблицами `users`, `wallets`, `orders`, `trades`, `positions`. Реализуй:
   - Регистрацию пользователя
   - Пополнение кошелька
   - Размещение ордера с проверкой баланса
   - Исполнение ордера с обновлением позиции

2. **Риск-менеджмент**: Добавь таблицу `risk_limits` с лимитами на максимальную позицию и дневной убыток. Реализуй проверку лимитов перед размещением ордера.

3. **Аудит**: Создай таблицу `audit_log` для логирования всех изменений в критических таблицах. Используй Diesel для записи событий при каждой операции.

4. **Оптимизация запросов**: Проанализируй созданные запросы и добавь необходимые индексы. Реализуй кеширование часто запрашиваемых данных (последние котировки, активные позиции).

## Навигация

[← Предыдущий день](../231-diesel-connection-setup/ru.md) | [Следующий день →](../233-diesel-crud-operations/ru.md)
