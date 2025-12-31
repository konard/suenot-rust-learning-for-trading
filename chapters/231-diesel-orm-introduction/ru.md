# День 231: Diesel ORM: Введение

## Аналогия из трейдинга

Представь, что ты управляешь торговым терминалом, который должен хранить историю всех сделок, котировок и состояние портфеля. Ты мог бы записывать всё в текстовые файлы, но это медленно и ненадёжно. Лучше использовать базу данных — это как профессиональный архив биржи, где каждая транзакция записана, индексирована и может быть быстро найдена.

**Diesel ORM** — это "переводчик" между твоим Rust-кодом и базой данных. Вместо того чтобы писать сырые SQL-запросы вручную, ты описываешь свои данные как структуры Rust, а Diesel автоматически генерирует правильные запросы. Это как иметь персонального брокера, который понимает твои намерения и правильно оформляет все документы.

В трейдинге это критически важно:
- **Надёжность**: Компилятор проверяет твои запросы на этапе сборки
- **Скорость**: Оптимизированные запросы без накладных расходов
- **Безопасность**: Защита от SQL-инъекций "из коробки"

## Что такое ORM?

**ORM (Object-Relational Mapping)** — это техника программирования, которая связывает объекты в коде с таблицами в базе данных:

| Концепция Rust | Концепция БД |
|----------------|--------------|
| Struct | Таблица |
| Поле структуры | Колонка |
| Экземпляр структуры | Строка |
| Vec<Struct> | Результат SELECT |

## Почему Diesel?

Diesel выделяется среди других ORM несколькими особенностями:

1. **Проверка на этапе компиляции** — ошибки в запросах обнаруживаются до запуска программы
2. **Нулевой накладной расход** — производительность как у сырого SQL
3. **Типобезопасность** — невозможно случайно смешать типы данных
4. **Поддержка миграций** — версионирование схемы базы данных

## Установка Diesel

Для работы с Diesel нужно установить CLI-инструмент и добавить зависимости:

```bash
# Установка diesel_cli (для SQLite)
cargo install diesel_cli --no-default-features --features sqlite

# Или для PostgreSQL (рекомендуется для продакшена)
cargo install diesel_cli --no-default-features --features postgres
```

## Настройка проекта

### Cargo.toml

```toml
[package]
name = "trading_db"
version = "0.1.0"
edition = "2021"

[dependencies]
diesel = { version = "2.1", features = ["sqlite", "r2d2"] }
dotenvy = "0.15"
chrono = { version = "0.4", features = ["serde"] }
```

### Инициализация базы данных

```bash
# Создаём файл .env с путём к базе данных
echo DATABASE_URL=trading.db > .env

# Инициализируем Diesel
diesel setup
```

Эта команда создаст:
- Директорию `migrations/` для миграций
- Файл `diesel.toml` с настройками

## Создание первой миграции

Миграция — это версионированное изменение схемы базы данных:

```bash
diesel migration generate create_trades
```

Это создаст директорию `migrations/YYYY-MM-DD-HHMMSS_create_trades/` с двумя файлами:
- `up.sql` — применение миграции
- `down.sql` — откат миграции

### up.sql — Создание таблицы сделок

```sql
-- Таблица для хранения торговых сделок
CREATE TABLE trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol VARCHAR(10) NOT NULL,
    side VARCHAR(4) NOT NULL CHECK (side IN ('buy', 'sell')),
    quantity REAL NOT NULL CHECK (quantity > 0),
    price REAL NOT NULL CHECK (price > 0),
    executed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    commission REAL NOT NULL DEFAULT 0.0
);

-- Индекс для быстрого поиска по символу
CREATE INDEX idx_trades_symbol ON trades(symbol);

-- Индекс для поиска по времени
CREATE INDEX idx_trades_executed_at ON trades(executed_at);
```

### down.sql — Откат миграции

```sql
DROP TABLE trades;
```

### Применение миграции

```bash
diesel migration run
```

## Схема и модели

После применения миграции Diesel генерирует файл `src/schema.rs`:

```rust
// src/schema.rs (генерируется автоматически)
diesel::table! {
    trades (id) {
        id -> Integer,
        symbol -> Text,
        side -> Text,
        quantity -> Float,
        price -> Float,
        executed_at -> Timestamp,
        commission -> Float,
    }
}
```

Теперь создадим модели для работы с данными:

```rust
// src/models.rs
use diesel::prelude::*;
use chrono::NaiveDateTime;

// Модель для чтения из базы данных
#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::trades)]
pub struct Trade {
    pub id: i32,
    pub symbol: String,
    pub side: String,
    pub quantity: f32,
    pub price: f32,
    pub executed_at: NaiveDateTime,
    pub commission: f32,
}

// Модель для вставки в базу данных
#[derive(Insertable)]
#[diesel(table_name = crate::schema::trades)]
pub struct NewTrade<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub quantity: f32,
    pub price: f32,
    pub commission: f32,
}

impl Trade {
    /// Рассчитывает общую стоимость сделки
    pub fn total_value(&self) -> f32 {
        self.quantity * self.price
    }

    /// Рассчитывает чистую стоимость с учётом комиссии
    pub fn net_value(&self) -> f32 {
        let value = self.total_value();
        match self.side.as_str() {
            "buy" => value + self.commission,
            "sell" => value - self.commission,
            _ => value,
        }
    }
}
```

## Подключение к базе данных

```rust
// src/lib.rs
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

pub mod models;
pub mod schema;

/// Устанавливает соединение с базой данных
pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL должен быть задан в .env файле");

    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Ошибка подключения к {}", database_url))
}
```

## Базовые CRUD операции

### Создание записи (Create)

```rust
use diesel::prelude::*;
use crate::models::{NewTrade, Trade};
use crate::schema::trades;

/// Записывает новую сделку в базу данных
pub fn create_trade(
    conn: &mut SqliteConnection,
    symbol: &str,
    side: &str,
    quantity: f32,
    price: f32,
    commission: f32,
) -> Trade {
    let new_trade = NewTrade {
        symbol,
        side,
        quantity,
        price,
        commission,
    };

    diesel::insert_into(trades::table)
        .values(&new_trade)
        .returning(Trade::as_returning())
        .get_result(conn)
        .expect("Ошибка сохранения сделки")
}

fn main() {
    let mut conn = establish_connection();

    // Записываем покупку Bitcoin
    let trade = create_trade(
        &mut conn,
        "BTC/USDT",
        "buy",
        0.5,
        42000.0,
        10.5, // комиссия $10.50
    );

    println!("Создана сделка #{}: {} {} {} @ {}",
        trade.id,
        trade.side,
        trade.quantity,
        trade.symbol,
        trade.price
    );
    println!("Общая стоимость: ${:.2}", trade.total_value());
    println!("С комиссией: ${:.2}", trade.net_value());
}
```

### Чтение записей (Read)

```rust
use diesel::prelude::*;
use crate::models::Trade;
use crate::schema::trades::dsl::*;

/// Получает все сделки по указанному символу
pub fn get_trades_by_symbol(
    conn: &mut SqliteConnection,
    ticker: &str,
) -> Vec<Trade> {
    trades
        .filter(symbol.eq(ticker))
        .order(executed_at.desc())
        .load::<Trade>(conn)
        .expect("Ошибка загрузки сделок")
}

/// Получает последние N сделок
pub fn get_recent_trades(
    conn: &mut SqliteConnection,
    limit: i64,
) -> Vec<Trade> {
    trades
        .order(executed_at.desc())
        .limit(limit)
        .load::<Trade>(conn)
        .expect("Ошибка загрузки сделок")
}

/// Получает сделку по ID
pub fn get_trade_by_id(
    conn: &mut SqliteConnection,
    trade_id: i32,
) -> Option<Trade> {
    trades
        .find(trade_id)
        .first(conn)
        .optional()
        .expect("Ошибка запроса к базе данных")
}

fn main() {
    let mut conn = establish_connection();

    // Получаем все сделки по BTC
    let btc_trades = get_trades_by_symbol(&mut conn, "BTC/USDT");
    println!("Найдено {} сделок по BTC/USDT", btc_trades.len());

    for trade in &btc_trades {
        println!("  #{}: {} {} @ ${:.2}",
            trade.id,
            trade.side,
            trade.quantity,
            trade.price
        );
    }

    // Получаем последние 5 сделок
    let recent = get_recent_trades(&mut conn, 5);
    println!("\nПоследние 5 сделок:");
    for trade in recent {
        println!("  {} {} {}", trade.symbol, trade.side, trade.quantity);
    }
}
```

### Обновление записи (Update)

```rust
use diesel::prelude::*;
use crate::schema::trades::dsl::*;

/// Обновляет комиссию для сделки
pub fn update_trade_commission(
    conn: &mut SqliteConnection,
    trade_id: i32,
    new_commission: f32,
) -> usize {
    diesel::update(trades.find(trade_id))
        .set(commission.eq(new_commission))
        .execute(conn)
        .expect("Ошибка обновления сделки")
}

/// Помечает все сделки по символу как закрытые (обновляет комиссию)
pub fn close_all_positions(
    conn: &mut SqliteConnection,
    ticker: &str,
    closing_commission: f32,
) -> usize {
    diesel::update(trades.filter(symbol.eq(ticker)))
        .set(commission.eq(closing_commission))
        .execute(conn)
        .expect("Ошибка закрытия позиций")
}

fn main() {
    let mut conn = establish_connection();

    // Обновляем комиссию для сделки #1
    let updated = update_trade_commission(&mut conn, 1, 15.0);
    println!("Обновлено записей: {}", updated);
}
```

### Удаление записи (Delete)

```rust
use diesel::prelude::*;
use crate::schema::trades::dsl::*;

/// Удаляет сделку по ID
pub fn delete_trade(
    conn: &mut SqliteConnection,
    trade_id: i32,
) -> usize {
    diesel::delete(trades.find(trade_id))
        .execute(conn)
        .expect("Ошибка удаления сделки")
}

/// Удаляет все сделки старше указанной даты
pub fn delete_old_trades(
    conn: &mut SqliteConnection,
    before: NaiveDateTime,
) -> usize {
    diesel::delete(trades.filter(executed_at.lt(before)))
        .execute(conn)
        .expect("Ошибка удаления старых сделок")
}

fn main() {
    let mut conn = establish_connection();

    // Удаляем сделку #1
    let deleted = delete_trade(&mut conn, 1);
    println!("Удалено записей: {}", deleted);
}
```

## Практический пример: Торговый журнал

```rust
use diesel::prelude::*;
use chrono::{NaiveDateTime, Utc};

mod schema;
mod models;

use models::{Trade, NewTrade};
use schema::trades;

/// Структура для управления торговым журналом
pub struct TradingJournal {
    conn: SqliteConnection,
}

impl TradingJournal {
    pub fn new(database_url: &str) -> Self {
        let conn = SqliteConnection::establish(database_url)
            .expect("Не удалось подключиться к базе данных");
        TradingJournal { conn }
    }

    /// Записывает покупку
    pub fn record_buy(
        &mut self,
        symbol: &str,
        quantity: f32,
        price: f32,
        commission: f32,
    ) -> Trade {
        let new_trade = NewTrade {
            symbol,
            side: "buy",
            quantity,
            price,
            commission,
        };

        diesel::insert_into(trades::table)
            .values(&new_trade)
            .returning(Trade::as_returning())
            .get_result(&mut self.conn)
            .expect("Ошибка записи покупки")
    }

    /// Записывает продажу
    pub fn record_sell(
        &mut self,
        symbol: &str,
        quantity: f32,
        price: f32,
        commission: f32,
    ) -> Trade {
        let new_trade = NewTrade {
            symbol,
            side: "sell",
            quantity,
            price,
            commission,
        };

        diesel::insert_into(trades::table)
            .values(&new_trade)
            .returning(Trade::as_returning())
            .get_result(&mut self.conn)
            .expect("Ошибка записи продажи")
    }

    /// Рассчитывает общую прибыль/убыток по символу
    pub fn calculate_pnl(&mut self, ticker: &str) -> f32 {
        use schema::trades::dsl::*;

        let ticker_trades: Vec<Trade> = trades
            .filter(symbol.eq(ticker))
            .load(&mut self.conn)
            .expect("Ошибка загрузки сделок");

        let mut pnl = 0.0;
        for trade in ticker_trades {
            match trade.side.as_str() {
                "buy" => pnl -= trade.net_value(),
                "sell" => pnl += trade.net_value(),
                _ => {}
            }
        }
        pnl
    }

    /// Получает статистику по всем сделкам
    pub fn get_statistics(&mut self) -> TradingStats {
        use schema::trades::dsl::*;

        let all_trades: Vec<Trade> = trades
            .load(&mut self.conn)
            .expect("Ошибка загрузки сделок");

        let total_trades = all_trades.len();
        let buy_trades = all_trades.iter().filter(|t| t.side == "buy").count();
        let sell_trades = all_trades.iter().filter(|t| t.side == "sell").count();

        let total_volume: f32 = all_trades.iter()
            .map(|t| t.total_value())
            .sum();

        let total_commission: f32 = all_trades.iter()
            .map(|t| t.commission)
            .sum();

        TradingStats {
            total_trades,
            buy_trades,
            sell_trades,
            total_volume,
            total_commission,
        }
    }
}

#[derive(Debug)]
pub struct TradingStats {
    pub total_trades: usize,
    pub buy_trades: usize,
    pub sell_trades: usize,
    pub total_volume: f32,
    pub total_commission: f32,
}

fn main() {
    let mut journal = TradingJournal::new("trading.db");

    // Записываем серию сделок
    println!("=== Записываем сделки ===\n");

    let buy1 = journal.record_buy("BTC/USDT", 0.5, 40000.0, 10.0);
    println!("Покупка: {} BTC @ ${}", buy1.quantity, buy1.price);

    let buy2 = journal.record_buy("BTC/USDT", 0.3, 41000.0, 6.15);
    println!("Покупка: {} BTC @ ${}", buy2.quantity, buy2.price);

    let sell1 = journal.record_sell("BTC/USDT", 0.8, 43000.0, 17.2);
    println!("Продажа: {} BTC @ ${}", sell1.quantity, sell1.price);

    // Рассчитываем P&L
    println!("\n=== Анализ позиции ===\n");

    let pnl = journal.calculate_pnl("BTC/USDT");
    println!("P&L по BTC/USDT: ${:.2}", pnl);

    // Получаем статистику
    let stats = journal.get_statistics();
    println!("\n=== Статистика торговли ===\n");
    println!("Всего сделок: {}", stats.total_trades);
    println!("Покупок: {}", stats.buy_trades);
    println!("Продаж: {}", stats.sell_trades);
    println!("Общий объём: ${:.2}", stats.total_volume);
    println!("Общая комиссия: ${:.2}", stats.total_commission);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| ORM | Object-Relational Mapping — связь объектов кода с таблицами БД |
| Diesel | Типобезопасный ORM для Rust с проверкой на этапе компиляции |
| Миграции | Версионированные изменения схемы базы данных |
| `Queryable` | Trait для чтения данных из БД |
| `Insertable` | Trait для вставки данных в БД |
| `diesel::insert_into` | Функция для создания INSERT запроса |
| `diesel::update` | Функция для создания UPDATE запроса |
| `diesel::delete` | Функция для создания DELETE запроса |

## Практические упражнения

1. **Создай таблицу котировок**: Напиши миграцию для таблицы `quotes` с полями: `id`, `symbol`, `bid`, `ask`, `timestamp`. Создай соответствующие модели.

2. **Расширь торговый журнал**: Добавь метод `get_trades_in_range(start: NaiveDateTime, end: NaiveDateTime)` для получения сделок в указанном временном диапазоне.

3. **Агрегация данных**: Напиши функцию, которая группирует сделки по символу и возвращает общий объём для каждого.

## Домашнее задание

1. **Портфель трейдера**: Создай таблицу `portfolio` для хранения текущих позиций (symbol, quantity, avg_price). Реализуй методы для обновления позиции при каждой сделке.

2. **История котировок**: Создай систему для сохранения и загрузки исторических данных OHLCV (Open, High, Low, Close, Volume). Реализуй запрос для получения свечей за указанный период.

3. **Риск-менеджмент**: Создай таблицу `risk_limits` с полями: symbol, max_position, max_loss_daily. Напиши функцию проверки, не превышает ли новая сделка установленные лимиты.

4. **Журнал ордеров**: Расширь систему, добавив таблицу `orders` (id, symbol, side, quantity, price, status, created_at, filled_at). Реализуй lifecycle ордера: создание → исполнение → запись сделки.

## Навигация

[← Предыдущий день](../230-database-introduction/ru.md) | [Следующий день →](../232-diesel-queries/ru.md)
