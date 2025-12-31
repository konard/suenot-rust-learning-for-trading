# День 226: tokio-postgres: Асинхронный клиент

## Аналогия из трейдинга

Представь современную высокочастотную торговую систему, которая должна одновременно:
- Записывать каждую сделку в базу данных
- Запрашивать исторические данные для анализа
- Обновлять текущие позиции портфеля
- Отслеживать статус ордеров

Если использовать синхронный клиент базы данных, каждая операция будет блокировать поток — как если бы трейдер мог делать только одно действие за раз: либо смотреть на график, либо размещать ордер, либо проверять баланс. С **tokio-postgres** (асинхронным клиентом PostgreSQL) твоя система может выполнять множество операций с базой данных параллельно, не блокируя основной поток — как команда трейдеров, где каждый занят своей задачей, но все работают одновременно.

## Что такое tokio-postgres?

`tokio-postgres` — это асинхронный клиент PostgreSQL для Rust, построенный на runtime Tokio. Он позволяет:

- Выполнять запросы без блокировки потока
- Обрабатывать множество соединений параллельно
- Эффективно работать с пулом соединений
- Использовать подготовленные запросы для оптимизации

### Добавление зависимости

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-postgres = "0.7"
```

## Базовое подключение

```rust
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Строка подключения к PostgreSQL
    let connection_string = "host=localhost user=trader password=secret dbname=trading";

    // Устанавливаем соединение
    let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

    // Соединение нужно обрабатывать в отдельной задаче
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения с базой: {}", e);
        }
    });

    // Теперь можем использовать клиент для запросов
    let rows = client
        .query("SELECT symbol, price FROM prices LIMIT 5", &[])
        .await?;

    for row in rows {
        let symbol: &str = row.get(0);
        let price: f64 = row.get(1);
        println!("{}: ${:.2}", symbol, price);
    }

    Ok(())
}
```

## Создание таблиц для торговой системы

```rust
use tokio_postgres::{NoTls, Error};

async fn setup_trading_database(client: &tokio_postgres::Client) -> Result<(), Error> {
    // Таблица для хранения ценовых данных
    client.execute(
        "CREATE TABLE IF NOT EXISTS price_history (
            id SERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            price DECIMAL(18, 8) NOT NULL,
            volume DECIMAL(18, 8) NOT NULL,
            timestamp TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Таблица ордеров
    client.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            id SERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL,
            side VARCHAR(4) NOT NULL,
            order_type VARCHAR(10) NOT NULL,
            price DECIMAL(18, 8),
            quantity DECIMAL(18, 8) NOT NULL,
            status VARCHAR(20) DEFAULT 'pending',
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Таблица портфеля
    client.execute(
        "CREATE TABLE IF NOT EXISTS portfolio (
            id SERIAL PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL UNIQUE,
            quantity DECIMAL(18, 8) NOT NULL,
            avg_price DECIMAL(18, 8) NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;

    // Таблица сделок
    client.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id SERIAL PRIMARY KEY,
            order_id INTEGER REFERENCES orders(id),
            symbol VARCHAR(20) NOT NULL,
            side VARCHAR(4) NOT NULL,
            price DECIMAL(18, 8) NOT NULL,
            quantity DECIMAL(18, 8) NOT NULL,
            fee DECIMAL(18, 8) DEFAULT 0,
            executed_at TIMESTAMPTZ DEFAULT NOW()
        )",
        &[],
    ).await?;

    println!("Таблицы торговой системы созданы успешно!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=trader dbname=trading",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения: {}", e);
        }
    });

    setup_trading_database(&client).await?;

    Ok(())
}
```

## Анализ цен: Запись и чтение данных

```rust
use tokio_postgres::{NoTls, Error, Row};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct PriceData {
    symbol: String,
    price: Decimal,
    volume: Decimal,
}

impl PriceData {
    fn from_row(row: &Row) -> Self {
        PriceData {
            symbol: row.get("symbol"),
            price: row.get("price"),
            volume: row.get("volume"),
        }
    }
}

struct PriceAnalyzer {
    client: tokio_postgres::Client,
}

impl PriceAnalyzer {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Ошибка соединения: {}", e);
            }
        });

        Ok(PriceAnalyzer { client })
    }

    // Записать новую цену
    async fn record_price(&self, symbol: &str, price: Decimal, volume: Decimal) -> Result<(), Error> {
        self.client.execute(
            "INSERT INTO price_history (symbol, price, volume) VALUES ($1, $2, $3)",
            &[&symbol, &price, &volume],
        ).await?;

        println!("Записана цена {} для {}", price, symbol);
        Ok(())
    }

    // Получить последнюю цену
    async fn get_latest_price(&self, symbol: &str) -> Result<Option<PriceData>, Error> {
        let row = self.client.query_opt(
            "SELECT symbol, price, volume FROM price_history
             WHERE symbol = $1
             ORDER BY timestamp DESC
             LIMIT 1",
            &[&symbol],
        ).await?;

        Ok(row.map(|r| PriceData::from_row(&r)))
    }

    // Получить среднюю цену за период
    async fn get_average_price(&self, symbol: &str, hours: i32) -> Result<Option<Decimal>, Error> {
        let row = self.client.query_opt(
            "SELECT AVG(price) as avg_price FROM price_history
             WHERE symbol = $1
             AND timestamp > NOW() - INTERVAL '1 hour' * $2",
            &[&symbol, &hours],
        ).await?;

        Ok(row.and_then(|r| r.get("avg_price")))
    }

    // Получить максимум и минимум
    async fn get_price_range(&self, symbol: &str, hours: i32) -> Result<(Option<Decimal>, Option<Decimal>), Error> {
        let row = self.client.query_one(
            "SELECT MIN(price) as min_price, MAX(price) as max_price
             FROM price_history
             WHERE symbol = $1
             AND timestamp > NOW() - INTERVAL '1 hour' * $2",
            &[&symbol, &hours],
        ).await?;

        let min_price: Option<Decimal> = row.get("min_price");
        let max_price: Option<Decimal> = row.get("max_price");

        Ok((min_price, max_price))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let analyzer = PriceAnalyzer::new("host=localhost user=trader dbname=trading").await?;

    // Записываем несколько цен
    let btc_price = Decimal::from_str("42500.50").unwrap();
    let eth_price = Decimal::from_str("2250.75").unwrap();

    analyzer.record_price("BTC", btc_price, Decimal::from_str("1.5").unwrap()).await?;
    analyzer.record_price("ETH", eth_price, Decimal::from_str("10.0").unwrap()).await?;

    // Получаем последнюю цену
    if let Some(price) = analyzer.get_latest_price("BTC").await? {
        println!("Последняя цена BTC: {}", price.price);
    }

    // Получаем среднюю цену за 24 часа
    if let Some(avg) = analyzer.get_average_price("BTC", 24).await? {
        println!("Средняя цена BTC за 24ч: {}", avg);
    }

    Ok(())
}
```

## Управление ордерами

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
struct Order {
    id: Option<i32>,
    symbol: String,
    side: String,
    order_type: String,
    price: Option<Decimal>,
    quantity: Decimal,
    status: String,
}

struct OrderManager {
    client: tokio_postgres::Client,
}

impl OrderManager {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Ошибка соединения: {}", e);
            }
        });

        Ok(OrderManager { client })
    }

    // Создать новый ордер
    async fn create_order(&self, order: &Order) -> Result<i32, Error> {
        let row = self.client.query_one(
            "INSERT INTO orders (symbol, side, order_type, price, quantity, status)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id",
            &[&order.symbol, &order.side, &order.order_type,
              &order.price, &order.quantity, &order.status],
        ).await?;

        let id: i32 = row.get(0);
        println!("Создан ордер #{}: {} {} {} @ {:?}",
                 id, order.side, order.quantity, order.symbol, order.price);
        Ok(id)
    }

    // Обновить статус ордера
    async fn update_order_status(&self, order_id: i32, status: &str) -> Result<bool, Error> {
        let rows_affected = self.client.execute(
            "UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2",
            &[&status, &order_id],
        ).await?;

        if rows_affected > 0 {
            println!("Ордер #{} обновлён: статус = {}", order_id, status);
        }

        Ok(rows_affected > 0)
    }

    // Получить активные ордера
    async fn get_active_orders(&self, symbol: Option<&str>) -> Result<Vec<Order>, Error> {
        let rows = match symbol {
            Some(s) => {
                self.client.query(
                    "SELECT id, symbol, side, order_type, price, quantity, status
                     FROM orders
                     WHERE status IN ('pending', 'partial')
                     AND symbol = $1
                     ORDER BY created_at DESC",
                    &[&s],
                ).await?
            }
            None => {
                self.client.query(
                    "SELECT id, symbol, side, order_type, price, quantity, status
                     FROM orders
                     WHERE status IN ('pending', 'partial')
                     ORDER BY created_at DESC",
                    &[],
                ).await?
            }
        };

        let orders: Vec<Order> = rows.iter().map(|row| {
            Order {
                id: Some(row.get("id")),
                symbol: row.get("symbol"),
                side: row.get("side"),
                order_type: row.get("order_type"),
                price: row.get("price"),
                quantity: row.get("quantity"),
                status: row.get("status"),
            }
        }).collect();

        Ok(orders)
    }

    // Отменить ордер
    async fn cancel_order(&self, order_id: i32) -> Result<bool, Error> {
        let rows_affected = self.client.execute(
            "UPDATE orders SET status = 'cancelled', updated_at = NOW()
             WHERE id = $1 AND status IN ('pending', 'partial')",
            &[&order_id],
        ).await?;

        if rows_affected > 0 {
            println!("Ордер #{} отменён", order_id);
        }

        Ok(rows_affected > 0)
    }

    // Отменить все ордера по символу
    async fn cancel_all_orders(&self, symbol: &str) -> Result<u64, Error> {
        let rows_affected = self.client.execute(
            "UPDATE orders SET status = 'cancelled', updated_at = NOW()
             WHERE symbol = $1 AND status IN ('pending', 'partial')",
            &[&symbol],
        ).await?;

        println!("Отменено {} ордеров для {}", rows_affected, symbol);
        Ok(rows_affected)
    }
}
```

## Отслеживание портфеля

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: Decimal,
    avg_price: Decimal,
}

impl Position {
    fn market_value(&self, current_price: Decimal) -> Decimal {
        self.quantity * current_price
    }

    fn unrealized_pnl(&self, current_price: Decimal) -> Decimal {
        self.quantity * (current_price - self.avg_price)
    }
}

struct PortfolioTracker {
    client: tokio_postgres::Client,
}

impl PortfolioTracker {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Ошибка соединения: {}", e);
            }
        });

        Ok(PortfolioTracker { client })
    }

    // Обновить позицию после сделки
    async fn update_position(&self, symbol: &str, quantity: Decimal, price: Decimal) -> Result<(), Error> {
        // Используем UPSERT (INSERT ... ON CONFLICT)
        self.client.execute(
            "INSERT INTO portfolio (symbol, quantity, avg_price)
             VALUES ($1, $2, $3)
             ON CONFLICT (symbol) DO UPDATE SET
                quantity = portfolio.quantity + EXCLUDED.quantity,
                avg_price = CASE
                    WHEN EXCLUDED.quantity > 0 THEN
                        (portfolio.avg_price * portfolio.quantity + EXCLUDED.avg_price * EXCLUDED.quantity)
                        / (portfolio.quantity + EXCLUDED.quantity)
                    ELSE portfolio.avg_price
                END,
                updated_at = NOW()",
            &[&symbol, &quantity, &price],
        ).await?;

        println!("Позиция {} обновлена: +{} @ {}", symbol, quantity, price);
        Ok(())
    }

    // Получить позицию
    async fn get_position(&self, symbol: &str) -> Result<Option<Position>, Error> {
        let row = self.client.query_opt(
            "SELECT symbol, quantity, avg_price FROM portfolio WHERE symbol = $1",
            &[&symbol],
        ).await?;

        Ok(row.map(|r| Position {
            symbol: r.get("symbol"),
            quantity: r.get("quantity"),
            avg_price: r.get("avg_price"),
        }))
    }

    // Получить все позиции
    async fn get_all_positions(&self) -> Result<Vec<Position>, Error> {
        let rows = self.client.query(
            "SELECT symbol, quantity, avg_price FROM portfolio WHERE quantity != 0",
            &[],
        ).await?;

        let positions: Vec<Position> = rows.iter().map(|r| Position {
            symbol: r.get("symbol"),
            quantity: r.get("quantity"),
            avg_price: r.get("avg_price"),
        }).collect();

        Ok(positions)
    }

    // Рассчитать общую стоимость портфеля
    async fn calculate_portfolio_value(
        &self,
        prices: &HashMap<String, Decimal>
    ) -> Result<Decimal, Error> {
        let positions = self.get_all_positions().await?;

        let total: Decimal = positions.iter()
            .filter_map(|pos| prices.get(&pos.symbol).map(|&p| pos.market_value(p)))
            .sum();

        Ok(total)
    }

    // Рассчитать нереализованный PnL
    async fn calculate_unrealized_pnl(
        &self,
        prices: &HashMap<String, Decimal>
    ) -> Result<Decimal, Error> {
        let positions = self.get_all_positions().await?;

        let total_pnl: Decimal = positions.iter()
            .filter_map(|pos| prices.get(&pos.symbol).map(|&p| pos.unrealized_pnl(p)))
            .sum();

        Ok(total_pnl)
    }
}
```

## Управление рисками

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;
use std::str::FromStr;

struct RiskManager {
    client: tokio_postgres::Client,
    max_position_size: Decimal,
    max_daily_loss: Decimal,
}

impl RiskManager {
    async fn new(
        connection_string: &str,
        max_position_size: Decimal,
        max_daily_loss: Decimal,
    ) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Ошибка соединения: {}", e);
            }
        });

        Ok(RiskManager {
            client,
            max_position_size,
            max_daily_loss,
        })
    }

    // Проверить лимит позиции
    async fn check_position_limit(&self, symbol: &str, additional_quantity: Decimal) -> Result<bool, Error> {
        let row = self.client.query_opt(
            "SELECT quantity FROM portfolio WHERE symbol = $1",
            &[&symbol],
        ).await?;

        let current_quantity: Decimal = row
            .map(|r| r.get("quantity"))
            .unwrap_or(Decimal::ZERO);

        let new_quantity = current_quantity + additional_quantity;

        if new_quantity.abs() > self.max_position_size {
            println!("РИСК: Превышен лимит позиции для {}! Текущая: {}, Запрошено: +{}",
                     symbol, current_quantity, additional_quantity);
            return Ok(false);
        }

        Ok(true)
    }

    // Получить дневной PnL
    async fn get_daily_pnl(&self) -> Result<Decimal, Error> {
        let row = self.client.query_one(
            "SELECT COALESCE(SUM(
                CASE WHEN side = 'sell' THEN price * quantity - fee
                     ELSE -(price * quantity + fee)
                END
            ), 0) as daily_pnl
             FROM trades
             WHERE executed_at >= CURRENT_DATE",
            &[],
        ).await?;

        let daily_pnl: Decimal = row.get("daily_pnl");
        Ok(daily_pnl)
    }

    // Проверить дневной лимит убытков
    async fn check_daily_loss_limit(&self) -> Result<bool, Error> {
        let daily_pnl = self.get_daily_pnl().await?;

        if daily_pnl < -self.max_daily_loss {
            println!("РИСК: Превышен дневной лимит убытков! PnL: {}", daily_pnl);
            return Ok(false);
        }

        Ok(true)
    }

    // Проверить можно ли разместить ордер
    async fn can_place_order(
        &self,
        symbol: &str,
        side: &str,
        quantity: Decimal,
    ) -> Result<bool, Error> {
        // Проверяем дневной лимит
        if !self.check_daily_loss_limit().await? {
            return Ok(false);
        }

        // Для покупки проверяем увеличение позиции
        let quantity_change = if side == "buy" { quantity } else { -quantity };

        if !self.check_position_limit(symbol, quantity_change).await? {
            return Ok(false);
        }

        println!("Риск-проверка пройдена для {} {} {}", side, quantity, symbol);
        Ok(true)
    }

    // Получить отчёт о рисках
    async fn get_risk_report(&self) -> Result<String, Error> {
        let daily_pnl = self.get_daily_pnl().await?;

        let positions = self.client.query(
            "SELECT symbol, quantity, avg_price FROM portfolio WHERE quantity != 0",
            &[],
        ).await?;

        let mut report = String::from("=== ОТЧЁТ О РИСКАХ ===\n");
        report.push_str(&format!("Дневной PnL: {}\n", daily_pnl));
        report.push_str(&format!("Лимит дневных убытков: {}\n", self.max_daily_loss));
        report.push_str(&format!("Использовано: {:.2}%\n\n",
            (daily_pnl.abs() / self.max_daily_loss * Decimal::from(100))));

        report.push_str("Позиции:\n");
        for row in positions {
            let symbol: String = row.get("symbol");
            let quantity: Decimal = row.get("quantity");
            let utilization = (quantity.abs() / self.max_position_size * Decimal::from(100));
            report.push_str(&format!("  {}: {} ({:.2}% от лимита)\n", symbol, quantity, utilization));
        }

        Ok(report)
    }
}
```

## Параллельные запросы

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio::try_join;

struct TradingDashboard {
    client: tokio_postgres::Client,
}

impl TradingDashboard {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Ошибка соединения: {}", e);
            }
        });

        Ok(TradingDashboard { client })
    }

    // Получить полный дашборд — все запросы выполняются параллельно!
    async fn get_dashboard_data(&self) -> Result<DashboardData, Error> {
        // Запускаем все запросы параллельно
        let (positions, active_orders, recent_trades, daily_pnl) = try_join!(
            self.get_positions(),
            self.get_active_orders(),
            self.get_recent_trades(10),
            self.get_daily_pnl()
        )?;

        Ok(DashboardData {
            positions,
            active_orders,
            recent_trades,
            daily_pnl,
        })
    }

    async fn get_positions(&self) -> Result<Vec<PositionInfo>, Error> {
        let rows = self.client.query(
            "SELECT symbol, quantity, avg_price FROM portfolio WHERE quantity != 0",
            &[],
        ).await?;

        Ok(rows.iter().map(|r| PositionInfo {
            symbol: r.get("symbol"),
            quantity: r.get("quantity"),
            avg_price: r.get("avg_price"),
        }).collect())
    }

    async fn get_active_orders(&self) -> Result<Vec<OrderInfo>, Error> {
        let rows = self.client.query(
            "SELECT id, symbol, side, price, quantity, status
             FROM orders
             WHERE status IN ('pending', 'partial')
             ORDER BY created_at DESC",
            &[],
        ).await?;

        Ok(rows.iter().map(|r| OrderInfo {
            id: r.get("id"),
            symbol: r.get("symbol"),
            side: r.get("side"),
            price: r.get("price"),
            quantity: r.get("quantity"),
            status: r.get("status"),
        }).collect())
    }

    async fn get_recent_trades(&self, limit: i64) -> Result<Vec<TradeInfo>, Error> {
        let rows = self.client.query(
            "SELECT id, symbol, side, price, quantity, executed_at
             FROM trades
             ORDER BY executed_at DESC
             LIMIT $1",
            &[&limit],
        ).await?;

        Ok(rows.iter().map(|r| TradeInfo {
            id: r.get("id"),
            symbol: r.get("symbol"),
            side: r.get("side"),
            price: r.get("price"),
            quantity: r.get("quantity"),
        }).collect())
    }

    async fn get_daily_pnl(&self) -> Result<Decimal, Error> {
        let row = self.client.query_one(
            "SELECT COALESCE(SUM(
                CASE WHEN side = 'sell' THEN price * quantity - fee
                     ELSE -(price * quantity + fee)
                END
            ), 0) as pnl FROM trades WHERE executed_at >= CURRENT_DATE",
            &[],
        ).await?;

        Ok(row.get("pnl"))
    }
}

#[derive(Debug)]
struct DashboardData {
    positions: Vec<PositionInfo>,
    active_orders: Vec<OrderInfo>,
    recent_trades: Vec<TradeInfo>,
    daily_pnl: Decimal,
}

#[derive(Debug)]
struct PositionInfo {
    symbol: String,
    quantity: Decimal,
    avg_price: Decimal,
}

#[derive(Debug)]
struct OrderInfo {
    id: i32,
    symbol: String,
    side: String,
    price: Option<Decimal>,
    quantity: Decimal,
    status: String,
}

#[derive(Debug)]
struct TradeInfo {
    id: i32,
    symbol: String,
    side: String,
    price: Decimal,
    quantity: Decimal,
}
```

## Транзакции

```rust
use tokio_postgres::{NoTls, Error};
use rust_decimal::Decimal;

struct TradingEngine {
    client: tokio_postgres::Client,
}

impl TradingEngine {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Ошибка соединения: {}", e);
            }
        });

        Ok(TradingEngine { client })
    }

    // Исполнить ордер атомарно с использованием транзакции
    async fn execute_order(
        &self,
        order_id: i32,
        executed_price: Decimal,
        executed_quantity: Decimal,
    ) -> Result<i32, Error> {
        // Начинаем транзакцию
        let transaction = self.client.transaction().await?;

        // 1. Получаем информацию об ордере
        let order_row = transaction.query_one(
            "SELECT symbol, side, quantity, status FROM orders WHERE id = $1 FOR UPDATE",
            &[&order_id],
        ).await?;

        let symbol: String = order_row.get("symbol");
        let side: String = order_row.get("side");
        let status: String = order_row.get("status");

        if status != "pending" && status != "partial" {
            // Откатываем если ордер не активен
            transaction.rollback().await?;
            return Err(Error::__private_api_protocol_error("Ордер не активен"));
        }

        // 2. Создаём запись о сделке
        let trade_row = transaction.query_one(
            "INSERT INTO trades (order_id, symbol, side, price, quantity)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
            &[&order_id, &symbol, &side, &executed_price, &executed_quantity],
        ).await?;

        let trade_id: i32 = trade_row.get(0);

        // 3. Обновляем статус ордера
        transaction.execute(
            "UPDATE orders SET status = 'filled', updated_at = NOW() WHERE id = $1",
            &[&order_id],
        ).await?;

        // 4. Обновляем портфель
        let quantity_change = if side == "buy" {
            executed_quantity
        } else {
            -executed_quantity
        };

        transaction.execute(
            "INSERT INTO portfolio (symbol, quantity, avg_price)
             VALUES ($1, $2, $3)
             ON CONFLICT (symbol) DO UPDATE SET
                quantity = portfolio.quantity + EXCLUDED.quantity,
                avg_price = CASE
                    WHEN EXCLUDED.quantity > 0 THEN
                        (portfolio.avg_price * portfolio.quantity + EXCLUDED.avg_price * EXCLUDED.quantity)
                        / NULLIF(portfolio.quantity + EXCLUDED.quantity, 0)
                    ELSE portfolio.avg_price
                END,
                updated_at = NOW()",
            &[&symbol, &quantity_change, &executed_price],
        ).await?;

        // 5. Фиксируем транзакцию
        transaction.commit().await?;

        println!("Ордер #{} исполнен: {} {} {} @ {}",
                 order_id, side, executed_quantity, symbol, executed_price);

        Ok(trade_id)
    }
}
```

## Подготовленные запросы для оптимизации

```rust
use tokio_postgres::{NoTls, Error, Statement};
use rust_decimal::Decimal;

struct OptimizedPriceRecorder {
    client: tokio_postgres::Client,
    insert_statement: Statement,
    select_latest_statement: Statement,
}

impl OptimizedPriceRecorder {
    async fn new(connection_string: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Ошибка соединения: {}", e);
            }
        });

        // Подготавливаем запросы заранее
        let insert_statement = client.prepare(
            "INSERT INTO price_history (symbol, price, volume) VALUES ($1, $2, $3)"
        ).await?;

        let select_latest_statement = client.prepare(
            "SELECT price FROM price_history WHERE symbol = $1 ORDER BY timestamp DESC LIMIT 1"
        ).await?;

        Ok(OptimizedPriceRecorder {
            client,
            insert_statement,
            select_latest_statement,
        })
    }

    // Быстрая запись цены с использованием подготовленного запроса
    async fn record_price(&self, symbol: &str, price: Decimal, volume: Decimal) -> Result<(), Error> {
        self.client.execute(&self.insert_statement, &[&symbol, &price, &volume]).await?;
        Ok(())
    }

    // Быстрое получение последней цены
    async fn get_latest_price(&self, symbol: &str) -> Result<Option<Decimal>, Error> {
        let row = self.client.query_opt(&self.select_latest_statement, &[&symbol]).await?;
        Ok(row.map(|r| r.get(0)))
    }

    // Массовая запись цен
    async fn record_prices_batch(&self, prices: &[(String, Decimal, Decimal)]) -> Result<(), Error> {
        for (symbol, price, volume) in prices {
            self.record_price(symbol, *price, *volume).await?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let recorder = OptimizedPriceRecorder::new(
        "host=localhost user=trader dbname=trading"
    ).await?;

    // Записываем много цен — подготовленные запросы работают быстрее
    for i in 0..1000 {
        let price = Decimal::from(42000 + i);
        recorder.record_price("BTC", price, Decimal::from(1)).await?;
    }

    println!("Записано 1000 ценовых точек");

    if let Some(price) = recorder.get_latest_price("BTC").await? {
        println!("Последняя цена BTC: {}", price);
    }

    Ok(())
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `tokio_postgres::connect` | Асинхронное подключение к PostgreSQL |
| `client.query()` | Выполнение запроса с возвратом строк |
| `client.execute()` | Выполнение запроса без возврата данных |
| `client.query_one()` | Запрос ровно одной строки |
| `client.query_opt()` | Запрос опционально одной строки |
| `client.transaction()` | Начало транзакции |
| `client.prepare()` | Создание подготовленного запроса |
| `try_join!` | Параллельное выполнение нескольких async операций |

## Домашнее задание

1. **Система мониторинга цен**: Создай сервис, который:
   - Каждую секунду записывает цену актива в базу
   - Рассчитывает скользящую среднюю за последние N записей
   - Отправляет уведомление если цена отклонилась более чем на X%

2. **Книга ордеров в базе данных**: Реализуй полноценную книгу ордеров:
   - Добавление лимитных ордеров
   - Сопоставление ордеров (матчинг)
   - Частичное исполнение
   - Получение лучших bid/ask

3. **Аудит торговых операций**: Создай систему аудита:
   - Логирование всех операций с портфелем
   - История изменений позиций
   - Генерация отчётов по периодам

4. **Пул соединений**: Используя `deadpool-postgres`, создай:
   - Пул из нескольких соединений
   - Параллельную обработку множества запросов
   - Graceful shutdown с закрытием всех соединений

## Навигация

[← Предыдущий день](../225-diesel-orm-rust/ru.md) | [Следующий день →](../227-sqlx-compile-time-sql/ru.md)
