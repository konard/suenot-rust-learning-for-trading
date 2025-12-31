# День 242: Мониторинг производительности базы данных

## Аналогия из трейдинга

Представь, что ты управляешь торговой платформой с миллионами сделок в день. Каждая миллисекунда задержки при запросе к базе данных — это потенциально упущенная прибыль или, что ещё хуже, неправильно исполненный ордер. Мониторинг производительности базы данных похож на наблюдение за торговым терминалом: ты отслеживаешь латентность (как быстро исполняются запросы), пропускную способность (сколько запросов обрабатывается в секунду) и использование ресурсов (не перегружена ли система).

В алготрейдинге база данных хранит:
- Историю котировок и свечей
- Журнал всех сделок
- Состояние портфеля и позиций
- Настройки торговых стратегий
- Метрики риск-менеджмента

Медленная база данных = медленная торговля = убытки.

## Что такое мониторинг производительности БД?

Мониторинг производительности базы данных включает:

1. **Измерение времени выполнения запросов** — сколько времени занимает каждый запрос
2. **Отслеживание пула соединений** — эффективно ли используются соединения
3. **Мониторинг кэша** — какой процент запросов попадает в кэш
4. **Анализ медленных запросов** — выявление проблемных запросов
5. **Метрики ресурсов** — CPU, память, диск, сеть

## Базовая структура метрик

```rust
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Метрики производительности запросов к БД
#[derive(Debug)]
pub struct QueryMetrics {
    /// Общее количество выполненных запросов
    pub total_queries: AtomicU64,
    /// Количество успешных запросов
    pub successful_queries: AtomicU64,
    /// Количество неудачных запросов
    pub failed_queries: AtomicU64,
    /// Суммарное время выполнения всех запросов (в микросекундах)
    pub total_duration_us: AtomicU64,
    /// Минимальное время запроса (в микросекундах)
    pub min_duration_us: AtomicU64,
    /// Максимальное время запроса (в микросекундах)
    pub max_duration_us: AtomicU64,
}

impl QueryMetrics {
    pub fn new() -> Self {
        QueryMetrics {
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            total_duration_us: AtomicU64::new(0),
            min_duration_us: AtomicU64::new(u64::MAX),
            max_duration_us: AtomicU64::new(0),
        }
    }

    /// Записывает метрику успешного запроса
    pub fn record_success(&self, duration: Duration) {
        let duration_us = duration.as_micros() as u64;

        self.total_queries.fetch_add(1, Ordering::Relaxed);
        self.successful_queries.fetch_add(1, Ordering::Relaxed);
        self.total_duration_us.fetch_add(duration_us, Ordering::Relaxed);

        // Обновляем минимум
        self.min_duration_us.fetch_min(duration_us, Ordering::Relaxed);
        // Обновляем максимум
        self.max_duration_us.fetch_max(duration_us, Ordering::Relaxed);
    }

    /// Записывает метрику неудачного запроса
    pub fn record_failure(&self, duration: Duration) {
        let duration_us = duration.as_micros() as u64;

        self.total_queries.fetch_add(1, Ordering::Relaxed);
        self.failed_queries.fetch_add(1, Ordering::Relaxed);
        self.total_duration_us.fetch_add(duration_us, Ordering::Relaxed);
    }

    /// Возвращает среднее время запроса
    pub fn average_duration(&self) -> Duration {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            return Duration::ZERO;
        }
        let avg_us = self.total_duration_us.load(Ordering::Relaxed) / total;
        Duration::from_micros(avg_us)
    }

    /// Возвращает процент успешных запросов
    pub fn success_rate(&self) -> f64 {
        let total = self.total_queries.load(Ordering::Relaxed);
        if total == 0 {
            return 100.0;
        }
        let successful = self.successful_queries.load(Ordering::Relaxed);
        (successful as f64 / total as f64) * 100.0
    }
}

fn main() {
    let metrics = Arc::new(QueryMetrics::new());

    // Симулируем запросы к базе данных
    for i in 0..100 {
        let start = Instant::now();

        // Имитация запроса к БД
        std::thread::sleep(Duration::from_micros(100 + (i % 50) * 10));

        let duration = start.elapsed();

        if i % 10 == 0 {
            metrics.record_failure(duration);
        } else {
            metrics.record_success(duration);
        }
    }

    println!("=== Метрики производительности БД ===");
    println!("Всего запросов: {}", metrics.total_queries.load(Ordering::Relaxed));
    println!("Успешных: {}", metrics.successful_queries.load(Ordering::Relaxed));
    println!("Неудачных: {}", metrics.failed_queries.load(Ordering::Relaxed));
    println!("Среднее время: {:?}", metrics.average_duration());
    println!("Процент успеха: {:.2}%", metrics.success_rate());
}
```

## Мониторинг пула соединений

В торговых системах важно эффективно управлять соединениями с БД:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Метрики пула соединений
#[derive(Debug)]
pub struct ConnectionPoolMetrics {
    /// Текущее количество активных соединений
    pub active_connections: AtomicUsize,
    /// Максимальное количество соединений
    pub max_connections: usize,
    /// Количество ожидающих соединения
    pub waiting_count: AtomicUsize,
    /// Общее количество полученных соединений
    pub connections_acquired: AtomicU64,
    /// Суммарное время ожидания соединения (мкс)
    pub total_wait_time_us: AtomicU64,
    /// Количество таймаутов при получении соединения
    pub connection_timeouts: AtomicU64,
}

impl ConnectionPoolMetrics {
    pub fn new(max_connections: usize) -> Self {
        ConnectionPoolMetrics {
            active_connections: AtomicUsize::new(0),
            max_connections,
            waiting_count: AtomicUsize::new(0),
            connections_acquired: AtomicU64::new(0),
            total_wait_time_us: AtomicU64::new(0),
            connection_timeouts: AtomicU64::new(0),
        }
    }

    /// Использование пула в процентах
    pub fn utilization(&self) -> f64 {
        let active = self.active_connections.load(Ordering::Relaxed);
        (active as f64 / self.max_connections as f64) * 100.0
    }

    /// Среднее время ожидания соединения
    pub fn average_wait_time(&self) -> Duration {
        let acquired = self.connections_acquired.load(Ordering::Relaxed);
        if acquired == 0 {
            return Duration::ZERO;
        }
        let avg_us = self.total_wait_time_us.load(Ordering::Relaxed) / acquired;
        Duration::from_micros(avg_us)
    }
}

/// Трейт для мониторируемого пула соединений
pub trait MonitoredPool {
    fn acquire(&self) -> Result<Connection, PoolError>;
    fn release(&self, conn: Connection);
    fn metrics(&self) -> &ConnectionPoolMetrics;
}

#[derive(Debug)]
pub struct Connection {
    id: u64,
    created_at: Instant,
}

#[derive(Debug)]
pub enum PoolError {
    Timeout,
    Exhausted,
}

/// Симуляция пула соединений для торговой системы
pub struct TradingDatabasePool {
    metrics: ConnectionPoolMetrics,
    next_conn_id: AtomicU64,
}

impl TradingDatabasePool {
    pub fn new(max_connections: usize) -> Self {
        TradingDatabasePool {
            metrics: ConnectionPoolMetrics::new(max_connections),
            next_conn_id: AtomicU64::new(0),
        }
    }

    pub fn acquire(&self) -> Result<Connection, PoolError> {
        let start = Instant::now();

        // Увеличиваем счётчик ожидающих
        self.metrics.waiting_count.fetch_add(1, Ordering::Relaxed);

        // Проверяем, есть ли свободные соединения
        let current = self.metrics.active_connections.load(Ordering::Relaxed);
        if current >= self.metrics.max_connections {
            self.metrics.waiting_count.fetch_sub(1, Ordering::Relaxed);
            self.metrics.connection_timeouts.fetch_add(1, Ordering::Relaxed);
            return Err(PoolError::Exhausted);
        }

        // Получаем соединение
        self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);
        self.metrics.waiting_count.fetch_sub(1, Ordering::Relaxed);
        self.metrics.connections_acquired.fetch_add(1, Ordering::Relaxed);

        let wait_time = start.elapsed();
        self.metrics.total_wait_time_us.fetch_add(
            wait_time.as_micros() as u64,
            Ordering::Relaxed
        );

        let id = self.next_conn_id.fetch_add(1, Ordering::Relaxed);

        Ok(Connection {
            id,
            created_at: Instant::now(),
        })
    }

    pub fn release(&self, _conn: Connection) {
        self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn metrics(&self) -> &ConnectionPoolMetrics {
        &self.metrics
    }
}

use std::sync::atomic::AtomicU64;

fn main() {
    let pool = Arc::new(TradingDatabasePool::new(10));

    println!("=== Симуляция торговой нагрузки на пул соединений ===\n");

    // Симуляция торговых операций
    let mut handles = vec![];

    for trader_id in 0..5 {
        let pool = Arc::clone(&pool);
        let handle = std::thread::spawn(move || {
            for order_id in 0..20 {
                match pool.acquire() {
                    Ok(conn) => {
                        // Симуляция запроса к БД для обработки ордера
                        std::thread::sleep(Duration::from_millis(5));
                        println!(
                            "Трейдер {}: ордер {} обработан (соединение #{})",
                            trader_id, order_id, conn.id
                        );
                        pool.release(conn);
                    }
                    Err(PoolError::Exhausted) => {
                        println!(
                            "Трейдер {}: ордер {} отложен - пул исчерпан",
                            trader_id, order_id
                        );
                    }
                    Err(PoolError::Timeout) => {
                        println!(
                            "Трейдер {}: ордер {} таймаут",
                            trader_id, order_id
                        );
                    }
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let metrics = pool.metrics();
    println!("\n=== Итоговые метрики пула ===");
    println!("Получено соединений: {}", metrics.connections_acquired.load(Ordering::Relaxed));
    println!("Таймаутов: {}", metrics.connection_timeouts.load(Ordering::Relaxed));
    println!("Среднее время ожидания: {:?}", metrics.average_wait_time());
    println!("Текущая загрузка: {:.1}%", metrics.utilization());
}
```

## Отслеживание медленных запросов

Для торговых систем критически важно находить медленные запросы:

```rust
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

/// Информация о медленном запросе
#[derive(Debug, Clone)]
pub struct SlowQuery {
    pub query: String,
    pub duration: Duration,
    pub timestamp: SystemTime,
    pub context: String, // Например, "order_execution" или "price_fetch"
}

/// Трекер медленных запросов для торговой системы
pub struct SlowQueryTracker {
    /// Порог для медленного запроса
    threshold: Duration,
    /// Последние медленные запросы (ограниченный буфер)
    slow_queries: Mutex<VecDeque<SlowQuery>>,
    /// Максимальное количество хранимых запросов
    max_queries: usize,
}

impl SlowQueryTracker {
    pub fn new(threshold: Duration, max_queries: usize) -> Self {
        SlowQueryTracker {
            threshold,
            slow_queries: Mutex::new(VecDeque::with_capacity(max_queries)),
            max_queries,
        }
    }

    /// Записывает запрос, если он медленный
    pub fn record(&self, query: &str, duration: Duration, context: &str) {
        if duration >= self.threshold {
            let slow_query = SlowQuery {
                query: query.to_string(),
                duration,
                timestamp: SystemTime::now(),
                context: context.to_string(),
            };

            let mut queries = self.slow_queries.lock().unwrap();

            if queries.len() >= self.max_queries {
                queries.pop_front();
            }

            queries.push_back(slow_query);
        }
    }

    /// Возвращает все медленные запросы
    pub fn get_slow_queries(&self) -> Vec<SlowQuery> {
        self.slow_queries.lock().unwrap().iter().cloned().collect()
    }

    /// Возвращает самый медленный запрос
    pub fn get_slowest(&self) -> Option<SlowQuery> {
        self.slow_queries
            .lock()
            .unwrap()
            .iter()
            .max_by_key(|q| q.duration)
            .cloned()
    }

    /// Возвращает количество медленных запросов
    pub fn count(&self) -> usize {
        self.slow_queries.lock().unwrap().len()
    }
}

/// Менеджер мониторинга для торговой БД
pub struct TradingDbMonitor {
    slow_query_tracker: SlowQueryTracker,
}

impl TradingDbMonitor {
    pub fn new() -> Self {
        TradingDbMonitor {
            // Порог 10мс для торговых систем — уже медленно!
            slow_query_tracker: SlowQueryTracker::new(
                Duration::from_millis(10),
                100
            ),
        }
    }

    /// Выполняет запрос с мониторингом
    pub fn execute_query<F, T>(&self, query: &str, context: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        self.slow_query_tracker.record(query, duration, context);

        result
    }

    pub fn report(&self) {
        println!("\n=== Отчёт о медленных запросах ===");
        println!("Всего медленных запросов: {}", self.slow_query_tracker.count());

        if let Some(slowest) = self.slow_query_tracker.get_slowest() {
            println!("\nСамый медленный запрос:");
            println!("  Контекст: {}", slowest.context);
            println!("  Запрос: {}", slowest.query);
            println!("  Время: {:?}", slowest.duration);
        }

        println!("\nПоследние медленные запросы:");
        for sq in self.slow_query_tracker.get_slow_queries().iter().take(5) {
            println!(
                "  [{:?}] {} - {:?}",
                sq.context, sq.query, sq.duration
            );
        }
    }
}

fn main() {
    let monitor = TradingDbMonitor::new();

    // Симуляция торговых запросов
    let trading_queries = vec![
        ("SELECT * FROM prices WHERE symbol = 'BTCUSDT' ORDER BY time DESC LIMIT 1", "price_fetch", 5),
        ("INSERT INTO orders (symbol, side, price, qty) VALUES ('BTCUSDT', 'BUY', 42000, 0.1)", "order_insert", 15),
        ("SELECT * FROM positions WHERE portfolio_id = 1", "position_check", 8),
        ("UPDATE balances SET amount = amount - 4200 WHERE currency = 'USDT'", "balance_update", 25),
        ("SELECT * FROM candles WHERE symbol = 'BTCUSDT' AND timeframe = '1h' ORDER BY time DESC LIMIT 100", "candle_fetch", 50),
        ("SELECT AVG(price) FROM trades WHERE symbol = 'BTCUSDT' AND time > NOW() - INTERVAL '1 hour'", "vwap_calc", 35),
        ("INSERT INTO trade_log (order_id, fill_price, qty, fee) VALUES (12345, 42001.5, 0.1, 0.42)", "trade_log", 3),
    ];

    println!("=== Симуляция торговых запросов ===\n");

    for (query, context, delay_ms) in trading_queries {
        monitor.execute_query(query, context, || {
            // Симуляция выполнения запроса
            std::thread::sleep(Duration::from_millis(delay_ms));
            println!("Выполнен: {} ({:?})", context, Duration::from_millis(delay_ms));
        });
    }

    monitor.report();
}
```

## Агрегация метрик по типам запросов

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Статистика по конкретному типу запроса
#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    pub count: u64,
    pub total_duration_us: u64,
    pub min_duration_us: u64,
    pub max_duration_us: u64,
    pub errors: u64,
}

impl QueryStats {
    pub fn new() -> Self {
        QueryStats {
            count: 0,
            total_duration_us: 0,
            min_duration_us: u64::MAX,
            max_duration_us: 0,
            errors: 0,
        }
    }

    pub fn record(&mut self, duration: Duration, success: bool) {
        let duration_us = duration.as_micros() as u64;

        self.count += 1;
        self.total_duration_us += duration_us;
        self.min_duration_us = self.min_duration_us.min(duration_us);
        self.max_duration_us = self.max_duration_us.max(duration_us);

        if !success {
            self.errors += 1;
        }
    }

    pub fn average_us(&self) -> u64 {
        if self.count == 0 { 0 } else { self.total_duration_us / self.count }
    }

    pub fn error_rate(&self) -> f64 {
        if self.count == 0 { 0.0 } else { (self.errors as f64 / self.count as f64) * 100.0 }
    }
}

/// Категории запросов в торговой системе
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum QueryCategory {
    PriceFetch,      // Получение котировок
    OrderInsert,     // Создание ордеров
    OrderUpdate,     // Обновление ордеров
    TradeLog,        // Логирование сделок
    BalanceCheck,    // Проверка балансов
    PositionQuery,   // Запросы позиций
    HistoricalData,  // Исторические данные
    RiskMetrics,     // Метрики риска
}

/// Агрегатор метрик по категориям
pub struct QueryMetricsAggregator {
    stats: Mutex<HashMap<QueryCategory, QueryStats>>,
}

impl QueryMetricsAggregator {
    pub fn new() -> Self {
        QueryMetricsAggregator {
            stats: Mutex::new(HashMap::new()),
        }
    }

    pub fn record(&self, category: QueryCategory, duration: Duration, success: bool) {
        let mut stats = self.stats.lock().unwrap();
        stats.entry(category)
            .or_insert_with(QueryStats::new)
            .record(duration, success);
    }

    pub fn get_stats(&self, category: &QueryCategory) -> Option<QueryStats> {
        self.stats.lock().unwrap().get(category).cloned()
    }

    pub fn report(&self) {
        let stats = self.stats.lock().unwrap();

        println!("\n{:=<70}", "");
        println!("{:^70}", "ОТЧЁТ ПО ПРОИЗВОДИТЕЛЬНОСТИ ТОРГОВОЙ БД");
        println!("{:=<70}", "");
        println!(
            "{:<20} {:>10} {:>12} {:>12} {:>10}",
            "Категория", "Запросов", "Сред.(мкс)", "Макс.(мкс)", "Ошибки %"
        );
        println!("{:-<70}", "");

        for (category, stat) in stats.iter() {
            println!(
                "{:<20} {:>10} {:>12} {:>12} {:>10.2}",
                format!("{:?}", category),
                stat.count,
                stat.average_us(),
                stat.max_duration_us,
                stat.error_rate()
            );
        }
        println!("{:=<70}", "");
    }
}

fn main() {
    let aggregator = Arc::new(QueryMetricsAggregator::new());

    // Симуляция торгового дня
    let test_data = vec![
        (QueryCategory::PriceFetch, 100, 2, false),
        (QueryCategory::PriceFetch, 150, 1, false),
        (QueryCategory::PriceFetch, 120, 3, false),
        (QueryCategory::OrderInsert, 5000, 15, false),
        (QueryCategory::OrderInsert, 4500, 12, false),
        (QueryCategory::OrderInsert, 6000, 20, true),
        (QueryCategory::BalanceCheck, 200, 5, false),
        (QueryCategory::BalanceCheck, 180, 4, false),
        (QueryCategory::TradeLog, 3000, 8, false),
        (QueryCategory::TradeLog, 2500, 6, false),
        (QueryCategory::HistoricalData, 50000, 150, false),
        (QueryCategory::HistoricalData, 45000, 120, false),
        (QueryCategory::RiskMetrics, 10000, 35, false),
        (QueryCategory::PositionQuery, 500, 8, false),
    ];

    println!("=== Симуляция торговых операций ===\n");

    for (category, duration_us, _expected_ms, is_error) in test_data {
        let duration = Duration::from_micros(duration_us);
        aggregator.record(category.clone(), duration, !is_error);

        if is_error {
            println!("ОШИБКА: {:?}", category);
        }
    }

    aggregator.report();
}
```

## Мониторинг в реальном времени

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Метрики в реальном времени для торговой системы
pub struct RealTimeMetrics {
    // Счётчики за текущую секунду
    queries_current_second: AtomicU64,
    // Счётчики за предыдущую секунду (для расчёта QPS)
    queries_per_second: AtomicU64,
    // Суммарная латентность за текущую секунду
    latency_sum_us: AtomicU64,
    // Флаг для остановки фонового потока
    running: AtomicBool,
}

impl RealTimeMetrics {
    pub fn new() -> Arc<Self> {
        let metrics = Arc::new(RealTimeMetrics {
            queries_current_second: AtomicU64::new(0),
            queries_per_second: AtomicU64::new(0),
            latency_sum_us: AtomicU64::new(0),
            running: AtomicBool::new(true),
        });

        // Запускаем фоновый поток для обновления QPS
        let metrics_clone = Arc::clone(&metrics);
        thread::spawn(move || {
            while metrics_clone.running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(1));

                // Переносим счётчик текущей секунды в QPS
                let current = metrics_clone.queries_current_second.swap(0, Ordering::Relaxed);
                metrics_clone.queries_per_second.store(current, Ordering::Relaxed);

                // Сбрасываем латентность
                metrics_clone.latency_sum_us.store(0, Ordering::Relaxed);
            }
        });

        metrics
    }

    pub fn record_query(&self, duration: Duration) {
        self.queries_current_second.fetch_add(1, Ordering::Relaxed);
        self.latency_sum_us.fetch_add(
            duration.as_micros() as u64,
            Ordering::Relaxed
        );
    }

    pub fn get_qps(&self) -> u64 {
        self.queries_per_second.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Дашборд мониторинга торговой БД
pub struct TradingDbDashboard {
    metrics: Arc<RealTimeMetrics>,
    start_time: Instant,
}

impl TradingDbDashboard {
    pub fn new() -> Self {
        TradingDbDashboard {
            metrics: RealTimeMetrics::new(),
            start_time: Instant::now(),
        }
    }

    pub fn record(&self, duration: Duration) {
        self.metrics.record_query(duration);
    }

    pub fn display(&self) {
        let uptime = self.start_time.elapsed();
        let qps = self.metrics.get_qps();

        println!("\n┌─────────────────────────────────────────┐");
        println!("│     МОНИТОРИНГ ТОРГОВОЙ БД              │");
        println!("├─────────────────────────────────────────┤");
        println!("│ Uptime: {:>10.1}s                      │", uptime.as_secs_f64());
        println!("│ QPS:    {:>10}                        │", qps);
        println!("│ Статус: {:>10}                        │",
            if qps > 100 { "ВЫСОКАЯ НАГРУЗКА" }
            else if qps > 10 { "НОРМАЛЬНЫЙ" }
            else { "НИЗКИЙ" }
        );
        println!("└─────────────────────────────────────────┘");
    }

    pub fn stop(&self) {
        self.metrics.stop();
    }
}

fn main() {
    let dashboard = Arc::new(TradingDbDashboard::new());

    println!("=== Запуск мониторинга торговой БД ===");

    // Симуляция нагрузки
    let dashboard_clone = Arc::clone(&dashboard);
    let load_handle = thread::spawn(move || {
        for i in 0..500 {
            let duration = Duration::from_micros(100 + (i % 100) * 10);
            dashboard_clone.record(duration);
            thread::sleep(Duration::from_millis(5));
        }
    });

    // Отображение дашборда каждую секунду
    for _ in 0..5 {
        thread::sleep(Duration::from_secs(1));
        dashboard.display();
    }

    load_handle.join().unwrap();
    dashboard.stop();

    println!("\nМониторинг остановлен.");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| QueryMetrics | Структура для сбора метрик выполнения запросов |
| ConnectionPoolMetrics | Мониторинг использования пула соединений |
| SlowQueryTracker | Выявление и логирование медленных запросов |
| QueryCategory | Категоризация запросов для агрегации |
| RealTimeMetrics | Метрики в реальном времени (QPS) |
| AtomicU64 | Атомарные счётчики для многопоточного доступа |

## Упражнения

1. **Расширенные метрики**: Добавь к `QueryMetrics` расчёт перцентилей (p50, p95, p99) для времени выполнения запросов. Используй гистограмму для хранения распределения.

2. **Алерты**: Реализуй систему алертов, которая отправляет уведомление, если:
   - QPS падает ниже определённого порога
   - Процент ошибок превышает 5%
   - Среднее время запроса превышает 100мс

3. **Экспорт метрик**: Создай функцию для экспорта метрик в формате Prometheus (текстовый формат метрик).

4. **Визуализация**: Реализуй простую ASCII-визуализацию нагрузки на БД в виде графика в терминале.

## Домашнее задание

1. **Полная система мониторинга**: Создай комплексную систему мониторинга БД для торговой платформы, которая включает:
   - Все типы метрик из урока
   - Автоматическое определение аномалий
   - Отчёты по расписанию
   - Историю метрик за последний час

2. **Симулятор нагрузки**: Напиши программу, которая симулирует различные паттерны торговой нагрузки:
   - Открытие рынка (всплеск активности)
   - Нормальная торговля
   - Высокая волатильность (много ордеров)
   - Закрытие рынка

3. **Оптимизатор запросов**: На основе собранных метрик создай анализатор, который:
   - Группирует похожие медленные запросы
   - Предлагает создание индексов
   - Определяет запросы-кандидаты для кэширования

4. **Многопоточный стресс-тест**: Реализуй стресс-тест пула соединений с различным количеством потоков и проанализируй, как меняются метрики при увеличении нагрузки.

## Навигация

[← Предыдущий день](../241-database-connection-pools/ru.md) | [Следующий день →](../243-database-query-optimization/ru.md)
