# День 229: Connection Pool — пул соединений с базой данных

## Аналогия из трейдинга

Представь, что ты управляешь торговым деском с 100 трейдерами. Каждому трейдеру нужен прямой телефонный канал к бирже для отправки ордеров. Но аренда выделенной линии стоит дорого, и на установку соединения уходит 30 секунд.

**Плохой подход**: Каждый раз, когда трейдеру нужно отправить ордер, он звонит на биржу, ждёт соединения 30 секунд, отправляет ордер за 1 секунду и вешает трубку. При 1000 ордеров в день это 8+ часов только на ожидание соединения!

**Хороший подход**: Ты арендуешь 10 постоянных линий. Когда трейдеру нужно отправить ордер, он берёт свободную линию из "пула", отправляет ордер и возвращает линию обратно в пул. Следующий трейдер может использовать эту же линию мгновенно!

Это и есть **Connection Pool** — набор заранее установленных соединений с базой данных, которые приложение переиспользует вместо создания нового соединения для каждого запроса.

## Зачем нужен пул соединений?

В высоконагруженных торговых системах каждая миллисекунда на счету:

| Операция | Время без пула | Время с пулом |
|----------|----------------|---------------|
| Установка TCP-соединения | 1-5 мс | 0 мс |
| TLS handshake | 10-50 мс | 0 мс |
| Аутентификация в БД | 5-20 мс | 0 мс |
| Выполнение запроса | 1-10 мс | 1-10 мс |
| **Итого** | **17-85 мс** | **1-10 мс** |

При 10 000 запросов в секунду пул экономит до **750 секунд** процессорного времени ежесекундно!

## Основные концепции

### Параметры пула соединений

```rust
use std::time::Duration;

struct PoolConfig {
    // Минимальное количество соединений (всегда поддерживаются открытыми)
    min_connections: u32,

    // Максимальное количество соединений
    max_connections: u32,

    // Время ожидания свободного соединения
    connection_timeout: Duration,

    // Время жизни соединения (для защиты от утечек памяти)
    max_lifetime: Duration,

    // Время простоя перед закрытием лишнего соединения
    idle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            min_connections: 5,
            max_connections: 20,
            connection_timeout: Duration::from_secs(30),
            max_lifetime: Duration::from_secs(1800), // 30 минут
            idle_timeout: Duration::from_secs(600),  // 10 минут
        }
    }
}
```

## Пример с SQLx — популярным async драйвером

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::time::Duration;

#[derive(Debug, Clone)]
struct Trade {
    id: i64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

async fn create_trading_pool() -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .min_connections(5)           // Минимум 5 соединений
        .max_connections(50)          // Максимум 50 соединений
        .acquire_timeout(Duration::from_secs(3))  // Таймаут получения
        .idle_timeout(Duration::from_secs(600))   // Закрывать простаивающие
        .max_lifetime(Duration::from_secs(1800))  // Максимальное время жизни
        .connect("postgres://trader:secret@localhost/trading_db")
        .await?;

    println!("Пул соединений создан: min=5, max=50");
    Ok(pool)
}

async fn record_trade(pool: &PgPool, trade: &Trade) -> Result<i64, sqlx::Error> {
    // Соединение автоматически берётся из пула
    let row = sqlx::query(
        r#"
        INSERT INTO trades (symbol, price, quantity, side, timestamp)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#
    )
    .bind(&trade.symbol)
    .bind(trade.price)
    .bind(trade.quantity)
    .bind(&trade.side)
    .bind(trade.timestamp)
    .fetch_one(pool) // Соединение возвращается в пул после выполнения
    .await?;

    Ok(row.get("id"))
}

async fn get_recent_trades(pool: &PgPool, symbol: &str, limit: i32) -> Result<Vec<Trade>, sqlx::Error> {
    let trades = sqlx::query_as!(
        Trade,
        r#"
        SELECT id, symbol, price, quantity, side, timestamp
        FROM trades
        WHERE symbol = $1
        ORDER BY timestamp DESC
        LIMIT $2
        "#,
        symbol,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(trades)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_trading_pool().await?;

    // Симуляция высокой нагрузки — 1000 параллельных запросов
    let mut handles = vec![];

    for i in 0..1000 {
        let pool = pool.clone(); // Клонирование пула дёшево (это Arc)

        handles.push(tokio::spawn(async move {
            let trade = Trade {
                id: 0,
                symbol: "BTC/USD".to_string(),
                price: 42000.0 + (i as f64 * 0.1),
                quantity: 0.01,
                side: if i % 2 == 0 { "buy" } else { "sell" }.to_string(),
                timestamp: chrono::Utc::now(),
            };

            record_trade(&pool, &trade).await
        }));
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.await?.is_ok() {
            success_count += 1;
        }
    }

    println!("Успешно записано {} из 1000 сделок", success_count);

    // Статистика пула
    println!("Размер пула: {}", pool.size());
    println!("Простаивающих соединений: {}", pool.num_idle());

    Ok(())
}
```

## Реализация собственного простого пула

Для понимания механики создадим упрощённый пул:

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// Имитация соединения с базой данных
struct DatabaseConnection {
    id: u32,
    created_at: Instant,
}

impl DatabaseConnection {
    fn new(id: u32) -> Self {
        // Имитируем задержку установки соединения
        std::thread::sleep(Duration::from_millis(100));
        println!("Создано соединение #{}", id);

        DatabaseConnection {
            id,
            created_at: Instant::now(),
        }
    }

    fn execute(&self, query: &str) -> Result<String, String> {
        // Имитируем выполнение запроса
        std::thread::sleep(Duration::from_millis(10));
        Ok(format!("Соединение #{}: выполнен запрос '{}'", self.id, query))
    }

    fn is_healthy(&self) -> bool {
        // Проверяем, не устарело ли соединение
        self.created_at.elapsed() < Duration::from_secs(300)
    }
}

struct ConnectionPool {
    connections: Mutex<VecDeque<DatabaseConnection>>,
    available: Condvar,
    max_size: u32,
    current_size: Mutex<u32>,
    next_id: Mutex<u32>,
}

impl ConnectionPool {
    fn new(initial_size: u32, max_size: u32) -> Arc<Self> {
        let pool = Arc::new(ConnectionPool {
            connections: Mutex::new(VecDeque::new()),
            available: Condvar::new(),
            max_size,
            current_size: Mutex::new(0),
            next_id: Mutex::new(0),
        });

        // Создаём начальные соединения
        for _ in 0..initial_size {
            let conn = pool.create_connection();
            pool.connections.lock().unwrap().push_back(conn);
        }

        pool
    }

    fn create_connection(&self) -> DatabaseConnection {
        let mut next_id = self.next_id.lock().unwrap();
        let mut current_size = self.current_size.lock().unwrap();

        *next_id += 1;
        *current_size += 1;

        DatabaseConnection::new(*next_id)
    }

    fn get(&self, timeout: Duration) -> Option<PooledConnection> {
        let start = Instant::now();
        let mut connections = self.connections.lock().unwrap();

        loop {
            // Пробуем получить существующее соединение
            if let Some(conn) = connections.pop_front() {
                if conn.is_healthy() {
                    return Some(PooledConnection {
                        connection: Some(conn),
                        pool: self,
                    });
                }
                // Нездоровое соединение — уменьшаем счётчик и пробуем следующее
                *self.current_size.lock().unwrap() -= 1;
                continue;
            }

            // Можем ли создать новое соединение?
            let current = *self.current_size.lock().unwrap();
            if current < self.max_size {
                drop(connections); // Освобождаем лок перед долгой операцией
                let conn = self.create_connection();
                return Some(PooledConnection {
                    connection: Some(conn),
                    pool: self,
                });
            }

            // Ждём освобождения соединения
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return None; // Таймаут
            }

            let (guard, timeout_result) = self.available
                .wait_timeout(connections, remaining)
                .unwrap();
            connections = guard;

            if timeout_result.timed_out() {
                return None;
            }
        }
    }

    fn return_connection(&self, conn: DatabaseConnection) {
        if conn.is_healthy() {
            self.connections.lock().unwrap().push_back(conn);
            self.available.notify_one();
        } else {
            *self.current_size.lock().unwrap() -= 1;
        }
    }

    fn stats(&self) -> (u32, usize) {
        let current = *self.current_size.lock().unwrap();
        let available = self.connections.lock().unwrap().len();
        (current, available)
    }
}

// RAII-обёртка для автоматического возврата соединения
struct PooledConnection<'a> {
    connection: Option<DatabaseConnection>,
    pool: &'a ConnectionPool,
}

impl<'a> PooledConnection<'a> {
    fn execute(&self, query: &str) -> Result<String, String> {
        self.connection.as_ref().unwrap().execute(query)
    }
}

impl<'a> Drop for PooledConnection<'a> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            self.pool.return_connection(conn);
        }
    }
}

fn main() {
    let pool = ConnectionPool::new(3, 10);

    println!("\n=== Начальное состояние пула ===");
    let (total, available) = pool.stats();
    println!("Всего соединений: {}, доступно: {}", total, available);

    // Симуляция торговых запросов
    let pool = Arc::new(pool);
    let mut handles = vec![];

    for i in 0..20 {
        let pool = Arc::clone(&pool);

        handles.push(std::thread::spawn(move || {
            let trade_query = format!(
                "INSERT INTO trades (symbol, price) VALUES ('BTC', {})",
                42000.0 + i as f64
            );

            match pool.get(Duration::from_secs(5)) {
                Some(conn) => {
                    let result = conn.execute(&trade_query);
                    println!("Поток {}: {:?}", i, result);
                    // Соединение автоматически вернётся в пул при drop
                }
                None => {
                    println!("Поток {}: таймаут получения соединения!", i);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\n=== Финальное состояние пула ===");
    let (total, available) = Arc::try_unwrap(pool)
        .map(|p| p.stats())
        .unwrap_or((0, 0));
    println!("Всего соединений: {}, доступно: {}", total, available);
}
```

## Пул соединений в торговой системе

```rust
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

// Конфигурация для разных типов данных
struct TradingPoolManager {
    // Пул для записи сделок — высокая пропускная способность
    trades_pool: PgPool,

    // Пул для чтения рыночных данных — много читателей
    market_data_pool: PgPool,

    // Пул для управления рисками — критически важные операции
    risk_pool: PgPool,
}

impl TradingPoolManager {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // Пул для сделок — оптимизирован для записи
        let trades_pool = PgPoolOptions::new()
            .min_connections(10)
            .max_connections(100)
            .acquire_timeout(Duration::from_millis(500))
            .connect(database_url)
            .await?;

        // Пул для рыночных данных — больше соединений для чтения
        let market_data_pool = PgPoolOptions::new()
            .min_connections(20)
            .max_connections(200)
            .acquire_timeout(Duration::from_millis(100))
            .connect(database_url)
            .await?;

        // Пул для риск-менеджмента — меньше, но надёжнее
        let risk_pool = PgPoolOptions::new()
            .min_connections(5)
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(2)) // Больший таймаут для критичных операций
            .connect(database_url)
            .await?;

        Ok(TradingPoolManager {
            trades_pool,
            market_data_pool,
            risk_pool,
        })
    }

    // Быстрая запись сделки
    async fn record_trade(&self, symbol: &str, price: f64, qty: f64) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            "INSERT INTO trades (symbol, price, quantity) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(symbol)
        .bind(price)
        .bind(qty)
        .fetch_one(&self.trades_pool)
        .await
    }

    // Быстрое чтение рыночных данных
    async fn get_latest_price(&self, symbol: &str) -> Result<f64, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT price FROM market_data WHERE symbol = $1 ORDER BY timestamp DESC LIMIT 1"
        )
        .bind(symbol)
        .fetch_one(&self.market_data_pool)
        .await
    }

    // Критичная проверка лимитов
    async fn check_position_limit(&self, symbol: &str, new_qty: f64) -> Result<bool, sqlx::Error> {
        let current: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(quantity), 0) FROM positions WHERE symbol = $1"
        )
        .bind(symbol)
        .fetch_one(&self.risk_pool)
        .await?;

        let limit: f64 = sqlx::query_scalar(
            "SELECT max_position FROM risk_limits WHERE symbol = $1"
        )
        .bind(symbol)
        .fetch_one(&self.risk_pool)
        .await?;

        Ok(current + new_qty <= limit)
    }

    // Статистика всех пулов
    fn print_stats(&self) {
        println!("=== Статистика пулов соединений ===");
        println!("Trades Pool: size={}, idle={}",
            self.trades_pool.size(),
            self.trades_pool.num_idle());
        println!("Market Data Pool: size={}, idle={}",
            self.market_data_pool.size(),
            self.market_data_pool.num_idle());
        println!("Risk Pool: size={}, idle={}",
            self.risk_pool.size(),
            self.risk_pool.num_idle());
    }
}
```

## Мониторинг здоровья пула

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct PoolMetrics {
    connections_acquired: AtomicU64,
    connections_released: AtomicU64,
    connection_timeouts: AtomicU64,
    total_wait_time_ms: AtomicU64,
    queries_executed: AtomicU64,
}

impl PoolMetrics {
    fn new() -> Self {
        PoolMetrics {
            connections_acquired: AtomicU64::new(0),
            connections_released: AtomicU64::new(0),
            connection_timeouts: AtomicU64::new(0),
            total_wait_time_ms: AtomicU64::new(0),
            queries_executed: AtomicU64::new(0),
        }
    }

    fn record_acquire(&self, wait_time_ms: u64) {
        self.connections_acquired.fetch_add(1, Ordering::Relaxed);
        self.total_wait_time_ms.fetch_add(wait_time_ms, Ordering::Relaxed);
    }

    fn record_release(&self) {
        self.connections_released.fetch_add(1, Ordering::Relaxed);
    }

    fn record_timeout(&self) {
        self.connection_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    fn record_query(&self) {
        self.queries_executed.fetch_add(1, Ordering::Relaxed);
    }

    fn report(&self) -> PoolHealthReport {
        let acquired = self.connections_acquired.load(Ordering::Relaxed);
        let total_wait = self.total_wait_time_ms.load(Ordering::Relaxed);

        PoolHealthReport {
            total_acquired: acquired,
            total_released: self.connections_released.load(Ordering::Relaxed),
            total_timeouts: self.connection_timeouts.load(Ordering::Relaxed),
            avg_wait_time_ms: if acquired > 0 { total_wait / acquired } else { 0 },
            total_queries: self.queries_executed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
struct PoolHealthReport {
    total_acquired: u64,
    total_released: u64,
    total_timeouts: u64,
    avg_wait_time_ms: u64,
    total_queries: u64,
}

impl PoolHealthReport {
    fn is_healthy(&self) -> bool {
        // Алерт, если более 1% запросов получили таймаут
        let timeout_rate = if self.total_acquired > 0 {
            (self.total_timeouts as f64) / (self.total_acquired as f64)
        } else {
            0.0
        };

        // Алерт, если среднее время ожидания > 100 мс
        timeout_rate < 0.01 && self.avg_wait_time_ms < 100
    }

    fn print(&self) {
        println!("=== Отчёт о здоровье пула ===");
        println!("Соединений получено: {}", self.total_acquired);
        println!("Соединений возвращено: {}", self.total_released);
        println!("Таймаутов: {}", self.total_timeouts);
        println!("Среднее время ожидания: {} мс", self.avg_wait_time_ms);
        println!("Запросов выполнено: {}", self.total_queries);
        println!("Статус: {}", if self.is_healthy() { "ЗДОРОВ" } else { "ТРЕБУЕТ ВНИМАНИЯ" });
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Connection Pool | Набор переиспользуемых соединений с БД |
| min_connections | Минимум соединений, поддерживаемых открытыми |
| max_connections | Максимум соединений в пуле |
| acquire_timeout | Таймаут получения соединения из пула |
| idle_timeout | Время простоя до закрытия соединения |
| max_lifetime | Максимальное время жизни соединения |
| Метрики пула | Мониторинг здоровья и производительности |

## Практические упражнения

### Упражнение 1: Базовый пул
Реализуй простой пул соединений с возможностью:
- Установки min/max размера
- Получения соединения с таймаутом
- Автоматического возврата через RAII

### Упражнение 2: Мониторинг
Добавь к пулу сбор метрик:
- Количество успешных/неуспешных получений
- Среднее время ожидания соединения
- Текущая загрузка пула

### Упражнение 3: Разделение пулов
Создай систему с несколькими пулами для разных операций:
- Пул для записи сделок (высокая пропускная способность)
- Пул для чтения истории (много читателей)
- Пул для критических операций (гарантированный доступ)

### Упражнение 4: Health Check
Реализуй механизм проверки здоровья соединений:
- Периодический ping соединений
- Автоматическое удаление "мёртвых" соединений
- Восстановление минимального количества

## Домашнее задание

1. **Адаптивный пул**: Создай пул, который автоматически увеличивает размер при высокой нагрузке и уменьшает при простое. Используй метрики времени ожидания для принятия решений.

2. **Пул с приоритетами**: Реализуй пул, где критические операции (например, риск-менеджмент) получают соединения в первую очередь, даже если обычная очередь заполнена.

3. **Отказоустойчивый пул**: Создай пул с поддержкой нескольких серверов БД (primary + replicas). При отказе primary автоматически переключайся на replica для чтения.

4. **Бенчмарк**: Напиши тест производительности, сравнивающий:
   - Создание нового соединения для каждого запроса
   - Использование пула с разными настройками (min=1/max=10, min=5/max=50)

   Измерь пропускную способность и латентность при 100, 1000 и 10000 параллельных запросах.

## Навигация

[← Предыдущий день](../228-database-transactions/ru.md) | [Следующий день →](../230-connection-timeout-handling/ru.md)
