# День 333: Нагрузочное тестирование

## Аналогия из трейдинга

Представь, что ты управляешь высокочастотной торговой системой. Перед запуском в продакшн тебе нужно убедиться, что она выдержит пиковые нагрузки — например, во время важных новостей или волатильных движений рынка, когда поток ордеров резко возрастает.

**Нагрузочное тестирование — это как стресс-тест торговой системы:**
- Ты симулируешь сотни тысяч ордеров в секунду
- Проверяешь, как система ведёт себя под давлением
- Измеряешь латентность обработки каждого ордера
- Находишь узкие места до того, как они станут проблемой в реальной торговле

| Метрика | Описание | Пример из трейдинга |
|---------|----------|---------------------|
| **Throughput** | Количество операций в секунду | Ордеров/сек |
| **Latency** | Время обработки одной операции | Время от отправки до подтверждения |
| **P99 Latency** | Время для 99% запросов | Гарантированная скорость для большинства |
| **Error Rate** | Процент ошибок под нагрузкой | Отклонённые ордера |

## Инструменты для нагрузочного тестирования в Rust

### Criterion — бенчмаркинг с статистическим анализом

```rust
// Cargo.toml:
// [dev-dependencies]
// criterion = { version = "0.5", features = ["html_reports"] }
//
// [[bench]]
// name = "trading_bench"
// harness = false

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;

/// Структура ордера
#[derive(Clone, Debug)]
struct Order {
    id: u64,
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
    timestamp: u64,
}

#[derive(Clone, Debug, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

/// Книга ордеров — критичный компонент для тестирования
struct OrderBook {
    bids: Vec<Order>,  // Заявки на покупку (сортировка по убыванию цены)
    asks: Vec<Order>,  // Заявки на продажу (сортировка по возрастанию цены)
    orders_by_id: HashMap<u64, usize>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
            orders_by_id: HashMap::new(),
        }
    }

    fn add_order(&mut self, order: Order) {
        let orders = match order.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };

        // Находим позицию для вставки (поддерживаем сортировку)
        let pos = orders.iter().position(|o| {
            match order.side {
                OrderSide::Buy => o.price < order.price,
                OrderSide::Sell => o.price > order.price,
            }
        }).unwrap_or(orders.len());

        self.orders_by_id.insert(order.id, pos);
        orders.insert(pos, order);
    }

    fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|o| o.price)
    }

    fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|o| o.price)
    }

    fn spread(&self) -> Option<f64> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask - bid),
            _ => None,
        }
    }
}

/// Расчёт скользящей средней
fn calculate_sma(prices: &[f64], window: usize) -> Vec<f64> {
    if prices.len() < window {
        return vec![];
    }

    prices
        .windows(window)
        .map(|w| w.iter().sum::<f64>() / window as f64)
        .collect()
}

/// Расчёт экспоненциальной скользящей средней
fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() || period == 0 {
        return vec![];
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema = Vec::with_capacity(prices.len());

    // Первое значение EMA = SMA
    let sma: f64 = prices.iter().take(period).sum::<f64>() / period as f64;
    ema.push(sma);

    for price in prices.iter().skip(period) {
        let prev_ema = *ema.last().unwrap();
        let new_ema = (price - prev_ema) * multiplier + prev_ema;
        ema.push(new_ema);
    }

    ema
}

/// Расчёт волатильности (стандартное отклонение)
fn calculate_volatility(prices: &[f64], window: usize) -> Vec<f64> {
    if prices.len() < window {
        return vec![];
    }

    prices
        .windows(window)
        .map(|w| {
            let mean = w.iter().sum::<f64>() / window as f64;
            let variance = w.iter()
                .map(|p| (p - mean).powi(2))
                .sum::<f64>() / window as f64;
            variance.sqrt()
        })
        .collect()
}

fn benchmark_order_book(c: &mut Criterion) {
    let mut group = c.benchmark_group("OrderBook");

    // Тестируем разные размеры книги
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("add_order", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut book = OrderBook::new();
                    for i in 0..size {
                        let order = Order {
                            id: i as u64,
                            symbol: "BTCUSDT".to_string(),
                            side: if i % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell },
                            price: 50000.0 + (i as f64 * 0.01),
                            quantity: 0.1,
                            timestamp: i as u64,
                        };
                        book.add_order(order);
                    }
                    black_box(book.spread())
                });
            },
        );
    }

    group.finish();
}

fn benchmark_indicators(c: &mut Criterion) {
    let mut group = c.benchmark_group("Indicators");

    // Генерируем тестовые данные цен
    let prices: Vec<f64> = (0..10000)
        .map(|i| 50000.0 + (i as f64 * 0.001).sin() * 1000.0)
        .collect();

    // Бенчмарк SMA с разными периодами
    for period in [10, 50, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("SMA", period),
            period,
            |b, &period| {
                b.iter(|| calculate_sma(black_box(&prices), period))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("EMA", period),
            period,
            |b, &period| {
                b.iter(|| calculate_ema(black_box(&prices), period))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Volatility", period),
            period,
            |b, &period| {
                b.iter(|| calculate_volatility(black_box(&prices), period))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_order_book, benchmark_indicators);
criterion_main!(benches);
```

## Нагрузочное тестирование торговой системы

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;
use std::collections::VecDeque;

/// Метрики нагрузочного теста
#[derive(Debug, Clone)]
struct LoadTestMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_latency_ns: u64,
    min_latency_ns: u64,
    max_latency_ns: u64,
    latencies: Vec<u64>,  // Для расчёта перцентилей
}

impl LoadTestMetrics {
    fn new() -> Self {
        LoadTestMetrics {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_latency_ns: 0,
            min_latency_ns: u64::MAX,
            max_latency_ns: 0,
            latencies: Vec::new(),
        }
    }

    fn record(&mut self, latency_ns: u64, success: bool) {
        self.total_requests += 1;
        self.total_latency_ns += latency_ns;
        self.min_latency_ns = self.min_latency_ns.min(latency_ns);
        self.max_latency_ns = self.max_latency_ns.max(latency_ns);
        self.latencies.push(latency_ns);

        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
    }

    fn avg_latency_us(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.total_latency_ns as f64 / self.total_requests as f64) / 1000.0
    }

    fn percentile(&mut self, p: f64) -> u64 {
        if self.latencies.is_empty() {
            return 0;
        }
        self.latencies.sort_unstable();
        let idx = ((self.latencies.len() as f64 * p / 100.0) as usize)
            .min(self.latencies.len() - 1);
        self.latencies[idx]
    }

    fn throughput(&self, duration: Duration) -> f64 {
        self.total_requests as f64 / duration.as_secs_f64()
    }
}

/// Симулятор торговой системы для тестирования
struct TradingSystemSimulator {
    order_counter: AtomicU64,
    processed_orders: AtomicU64,
    is_running: AtomicBool,
}

impl TradingSystemSimulator {
    fn new() -> Self {
        TradingSystemSimulator {
            order_counter: AtomicU64::new(0),
            processed_orders: AtomicU64::new(0),
            is_running: AtomicBool::new(true),
        }
    }

    /// Симуляция обработки ордера
    fn process_order(&self, _symbol: &str, _side: &str, _price: f64, _qty: f64) -> Result<u64, String> {
        // Симулируем некоторую работу
        let order_id = self.order_counter.fetch_add(1, Ordering::SeqCst);

        // Симуляция валидации и обработки (небольшая задержка)
        std::hint::spin_loop();

        self.processed_orders.fetch_add(1, Ordering::SeqCst);
        Ok(order_id)
    }

    fn get_processed_count(&self) -> u64 {
        self.processed_orders.load(Ordering::SeqCst)
    }

    fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

/// Конфигурация нагрузочного теста
struct LoadTestConfig {
    num_threads: usize,
    duration: Duration,
    requests_per_second: Option<u64>,  // None = максимальная скорость
    warmup_duration: Duration,
}

/// Запуск нагрузочного теста
fn run_load_test(config: LoadTestConfig) -> LoadTestMetrics {
    let system = Arc::new(TradingSystemSimulator::new());
    let metrics = Arc::new(std::sync::Mutex::new(LoadTestMetrics::new()));
    let start_time = Instant::now();

    println!("=== Запуск нагрузочного теста ===");
    println!("Потоков: {}", config.num_threads);
    println!("Длительность: {:?}", config.duration);
    println!("Прогрев: {:?}", config.warmup_duration);
    println!();

    // Прогрев системы
    println!("Прогрев...");
    let warmup_end = Instant::now() + config.warmup_duration;
    while Instant::now() < warmup_end {
        let _ = system.process_order("BTCUSDT", "BUY", 50000.0, 0.1);
    }

    // Сброс счётчиков после прогрева
    system.processed_orders.store(0, Ordering::SeqCst);

    println!("Начало теста...");
    let test_start = Instant::now();
    let test_end = test_start + config.duration;

    // Запуск рабочих потоков
    let mut handles = vec![];

    for thread_id in 0..config.num_threads {
        let system = Arc::clone(&system);
        let metrics = Arc::clone(&metrics);
        let rps_limit = config.requests_per_second
            .map(|rps| rps / config.num_threads as u64);

        let handle = thread::spawn(move || {
            let mut local_metrics = LoadTestMetrics::new();
            let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
            let sides = ["BUY", "SELL"];
            let mut request_count = 0u64;
            let thread_start = Instant::now();

            while Instant::now() < test_end && system.is_running() {
                // Ограничение RPS если задано
                if let Some(limit) = rps_limit {
                    let expected_time = Duration::from_nanos(
                        request_count * 1_000_000_000 / limit
                    );
                    let elapsed = thread_start.elapsed();
                    if elapsed < expected_time {
                        thread::sleep(expected_time - elapsed);
                    }
                }

                let symbol = symbols[request_count as usize % symbols.len()];
                let side = sides[request_count as usize % sides.len()];
                let price = 50000.0 + (request_count as f64 * 0.01) % 1000.0;

                let op_start = Instant::now();
                let result = system.process_order(symbol, side, price, 0.1);
                let latency = op_start.elapsed().as_nanos() as u64;

                local_metrics.record(latency, result.is_ok());
                request_count += 1;
            }

            // Объединяем локальные метрики с глобальными
            let mut global_metrics = metrics.lock().unwrap();
            global_metrics.total_requests += local_metrics.total_requests;
            global_metrics.successful_requests += local_metrics.successful_requests;
            global_metrics.failed_requests += local_metrics.failed_requests;
            global_metrics.total_latency_ns += local_metrics.total_latency_ns;
            global_metrics.min_latency_ns = global_metrics.min_latency_ns
                .min(local_metrics.min_latency_ns);
            global_metrics.max_latency_ns = global_metrics.max_latency_ns
                .max(local_metrics.max_latency_ns);
            global_metrics.latencies.extend(local_metrics.latencies);
        });

        handles.push(handle);
    }

    // Ожидаем завершения всех потоков
    for handle in handles {
        handle.join().unwrap();
    }

    let total_duration = test_start.elapsed();
    system.stop();

    // Получаем финальные метрики
    let mut final_metrics = metrics.lock().unwrap().clone();

    // Выводим результаты
    println!("\n=== Результаты нагрузочного теста ===");
    println!("Общее время: {:?}", total_duration);
    println!("Всего запросов: {}", final_metrics.total_requests);
    println!("Успешных: {}", final_metrics.successful_requests);
    println!("Ошибок: {}", final_metrics.failed_requests);
    println!();
    println!("Throughput: {:.2} req/sec", final_metrics.throughput(total_duration));
    println!("Avg latency: {:.2} µs", final_metrics.avg_latency_us());
    println!("Min latency: {:.2} µs", final_metrics.min_latency_ns as f64 / 1000.0);
    println!("Max latency: {:.2} µs", final_metrics.max_latency_ns as f64 / 1000.0);
    println!("P50 latency: {:.2} µs", final_metrics.percentile(50.0) as f64 / 1000.0);
    println!("P95 latency: {:.2} µs", final_metrics.percentile(95.0) as f64 / 1000.0);
    println!("P99 latency: {:.2} µs", final_metrics.percentile(99.0) as f64 / 1000.0);

    final_metrics
}

fn main() {
    // Тест 1: Максимальная пропускная способность
    println!("\n### Тест 1: Максимальная пропускная способность ###\n");
    run_load_test(LoadTestConfig {
        num_threads: 4,
        duration: Duration::from_secs(5),
        requests_per_second: None,
        warmup_duration: Duration::from_secs(1),
    });

    // Тест 2: Контролируемая нагрузка
    println!("\n### Тест 2: Контролируемая нагрузка (10,000 RPS) ###\n");
    run_load_test(LoadTestConfig {
        num_threads: 4,
        duration: Duration::from_secs(5),
        requests_per_second: Some(10_000),
        warmup_duration: Duration::from_secs(1),
    });

    // Тест 3: Высокая конкурентность
    println!("\n### Тест 3: Высокая конкурентность (16 потоков) ###\n");
    run_load_test(LoadTestConfig {
        num_threads: 16,
        duration: Duration::from_secs(5),
        requests_per_second: None,
        warmup_duration: Duration::from_secs(1),
    });
}
```

## Профилирование под нагрузкой

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Профайлер для измерения времени выполнения секций кода
struct Profiler {
    sections: Arc<RwLock<HashMap<String, SectionStats>>>,
}

#[derive(Default, Clone)]
struct SectionStats {
    call_count: u64,
    total_time_ns: u64,
    min_time_ns: u64,
    max_time_ns: u64,
}

impl Profiler {
    fn new() -> Self {
        Profiler {
            sections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn section(&self, name: &str) -> ProfilerGuard {
        ProfilerGuard {
            name: name.to_string(),
            start: Instant::now(),
            profiler: Arc::clone(&self.sections),
        }
    }

    fn report(&self) {
        let sections = self.sections.read().unwrap();

        println!("\n=== Отчёт профилирования ===\n");
        println!("{:<30} {:>10} {:>12} {:>12} {:>12}",
                 "Секция", "Вызовов", "Avg (µs)", "Min (µs)", "Max (µs)");
        println!("{:-<78}", "");

        let mut entries: Vec<_> = sections.iter().collect();
        entries.sort_by(|a, b| b.1.total_time_ns.cmp(&a.1.total_time_ns));

        for (name, stats) in entries {
            let avg = if stats.call_count > 0 {
                stats.total_time_ns as f64 / stats.call_count as f64 / 1000.0
            } else {
                0.0
            };

            println!("{:<30} {:>10} {:>12.2} {:>12.2} {:>12.2}",
                     name,
                     stats.call_count,
                     avg,
                     stats.min_time_ns as f64 / 1000.0,
                     stats.max_time_ns as f64 / 1000.0);
        }
    }
}

struct ProfilerGuard {
    name: String,
    start: Instant,
    profiler: Arc<RwLock<HashMap<String, SectionStats>>>,
}

impl Drop for ProfilerGuard {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_nanos() as u64;
        let mut sections = self.profiler.write().unwrap();

        let stats = sections.entry(self.name.clone()).or_insert(SectionStats {
            call_count: 0,
            total_time_ns: 0,
            min_time_ns: u64::MAX,
            max_time_ns: 0,
        });

        stats.call_count += 1;
        stats.total_time_ns += elapsed;
        stats.min_time_ns = stats.min_time_ns.min(elapsed);
        stats.max_time_ns = stats.max_time_ns.max(elapsed);
    }
}

/// Торговый движок с профилированием
struct TradingEngine {
    profiler: Profiler,
    order_book: HashMap<String, Vec<(f64, f64)>>,  // symbol -> [(price, qty)]
    positions: HashMap<String, f64>,
}

impl TradingEngine {
    fn new(profiler: Profiler) -> Self {
        TradingEngine {
            profiler,
            order_book: HashMap::new(),
            positions: HashMap::new(),
        }
    }

    fn validate_order(&self, symbol: &str, price: f64, qty: f64) -> bool {
        let _guard = self.profiler.section("validate_order");

        // Симуляция валидации
        !symbol.is_empty() && price > 0.0 && qty > 0.0
    }

    fn check_risk(&self, symbol: &str, qty: f64) -> bool {
        let _guard = self.profiler.section("check_risk");

        // Симуляция проверки риска
        let current_position = self.positions.get(symbol).copied().unwrap_or(0.0);
        (current_position + qty).abs() < 100.0
    }

    fn update_order_book(&mut self, symbol: &str, price: f64, qty: f64) {
        let _guard = self.profiler.section("update_order_book");

        let orders = self.order_book.entry(symbol.to_string()).or_insert_with(Vec::new);
        orders.push((price, qty));

        // Сортировка книги ордеров
        orders.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    fn execute_order(&mut self, symbol: &str, side: &str, price: f64, qty: f64) -> Result<u64, String> {
        let _guard = self.profiler.section("execute_order");

        // Валидация
        if !self.validate_order(symbol, price, qty) {
            return Err("Invalid order".to_string());
        }

        // Проверка риска
        let risk_qty = if side == "BUY" { qty } else { -qty };
        if !self.check_risk(symbol, risk_qty) {
            return Err("Risk limit exceeded".to_string());
        }

        // Обновление позиции
        {
            let _guard = self.profiler.section("update_position");
            let position = self.positions.entry(symbol.to_string()).or_insert(0.0);
            *position += risk_qty;
        }

        // Обновление книги ордеров
        self.update_order_book(symbol, price, qty);

        Ok(1)
    }

    fn get_profiler(&self) -> &Profiler {
        &self.profiler
    }
}

fn main() {
    let profiler = Profiler::new();
    let mut engine = TradingEngine::new(profiler);

    println!("Запуск профилирования торгового движка...\n");

    // Симуляция нагрузки
    let start = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    for i in 0..100_000 {
        let symbol = ["BTCUSDT", "ETHUSDT", "SOLUSDT"][i % 3];
        let side = if i % 2 == 0 { "BUY" } else { "SELL" };
        let price = 50000.0 + (i as f64 * 0.01) % 1000.0;

        match engine.execute_order(symbol, side, price, 0.1) {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    let elapsed = start.elapsed();

    println!("Обработано 100,000 ордеров за {:?}", elapsed);
    println!("Успешно: {}, Ошибок: {}", success_count, error_count);
    println!("Throughput: {:.2} ордеров/сек", 100_000.0 / elapsed.as_secs_f64());

    // Отчёт профилирования
    engine.get_profiler().report();
}
```

## Тестирование пропускной способности WebSocket

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Симулятор WebSocket соединения для нагрузочного тестирования
struct WebSocketSimulator {
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
}

impl WebSocketSimulator {
    fn new() -> Self {
        WebSocketSimulator {
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
        }
    }

    fn send_message(&self, data: &[u8]) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(data.len() as u64, Ordering::Relaxed);
    }

    fn receive_message(&self, data: &[u8]) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
        self.bytes_received.fetch_add(data.len() as u64, Ordering::Relaxed);
    }

    fn stats(&self) -> (u64, u64, u64, u64) {
        (
            self.messages_sent.load(Ordering::Relaxed),
            self.messages_received.load(Ordering::Relaxed),
            self.bytes_sent.load(Ordering::Relaxed),
            self.bytes_received.load(Ordering::Relaxed),
        )
    }
}

/// Генератор рыночных данных
struct MarketDataGenerator {
    symbols: Vec<String>,
    base_prices: Vec<f64>,
}

impl MarketDataGenerator {
    fn new() -> Self {
        MarketDataGenerator {
            symbols: vec![
                "BTCUSDT".to_string(),
                "ETHUSDT".to_string(),
                "SOLUSDT".to_string(),
                "ADAUSDT".to_string(),
                "DOTUSDT".to_string(),
            ],
            base_prices: vec![50000.0, 3000.0, 100.0, 0.5, 10.0],
        }
    }

    fn generate_tick(&self, index: usize) -> String {
        let symbol = &self.symbols[index % self.symbols.len()];
        let base_price = self.base_prices[index % self.base_prices.len()];

        // Генерируем случайное изменение цены
        let price_change = (index as f64 * 0.001).sin() * base_price * 0.001;
        let price = base_price + price_change;

        format!(
            r#"{{"symbol":"{}","price":{:.8},"qty":{},"time":{}}}"#,
            symbol, price, 0.1, index
        )
    }
}

/// Тест пропускной способности потока рыночных данных
fn test_market_data_throughput() {
    println!("=== Тест пропускной способности потока данных ===\n");

    let ws = Arc::new(WebSocketSimulator::new());
    let generator = MarketDataGenerator::new();

    let duration = Duration::from_secs(5);
    let start = Instant::now();
    let mut message_count = 0u64;

    // Буфер для батчинга сообщений
    let mut batch = Vec::with_capacity(100);

    while start.elapsed() < duration {
        // Генерируем батч сообщений
        for _ in 0..100 {
            let tick = generator.generate_tick(message_count as usize);
            batch.push(tick);
            message_count += 1;
        }

        // Отправляем батч
        for msg in batch.drain(..) {
            ws.send_message(msg.as_bytes());
        }
    }

    let elapsed = start.elapsed();
    let (sent, _, bytes_sent, _) = ws.stats();

    println!("Длительность: {:?}", elapsed);
    println!("Сообщений отправлено: {}", sent);
    println!("Байт отправлено: {} MB", bytes_sent / 1_000_000);
    println!("Throughput: {:.2} msg/sec", sent as f64 / elapsed.as_secs_f64());
    println!("Bandwidth: {:.2} MB/sec", bytes_sent as f64 / 1_000_000.0 / elapsed.as_secs_f64());
}

/// Тест задержки обработки сообщений
fn test_message_latency() {
    println!("\n=== Тест задержки обработки сообщений ===\n");

    let generator = MarketDataGenerator::new();
    let mut latencies = Vec::with_capacity(10000);

    // Симуляция обработки сообщений
    for i in 0..10000 {
        let start = Instant::now();

        // Генерация сообщения
        let tick = generator.generate_tick(i);

        // Симуляция парсинга JSON
        let _: Vec<char> = tick.chars().collect();

        // Симуляция обновления книги ордеров
        let _price: f64 = 50000.0;
        let _update = (i, _price);

        let latency = start.elapsed().as_nanos() as u64;
        latencies.push(latency);
    }

    // Статистика
    latencies.sort();
    let avg = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
    let p50 = latencies[latencies.len() / 2];
    let p95 = latencies[latencies.len() * 95 / 100];
    let p99 = latencies[latencies.len() * 99 / 100];

    println!("Обработано сообщений: {}", latencies.len());
    println!("Avg latency: {:.2} ns", avg);
    println!("P50 latency: {} ns", p50);
    println!("P95 latency: {} ns", p95);
    println!("P99 latency: {} ns", p99);
}

fn main() {
    test_market_data_throughput();
    test_message_latency();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Throughput** | Количество операций в единицу времени |
| **Latency** | Время выполнения одной операции |
| **Перцентили (P50, P99)** | Статистика распределения задержек |
| **Criterion** | Библиотека для статистического бенчмаркинга |
| **Прогрев (Warmup)** | Разогрев системы перед измерениями |
| **Профилирование** | Анализ времени выполнения секций кода |
| **Батчинг** | Группировка операций для повышения throughput |

## Практические задания

1. **Бенчмарк книги ордеров**: Создай бенчмарк, который:
   - Измеряет время добавления ордера в книгу разного размера
   - Сравнивает разные структуры данных (Vec, BTreeMap, HashMap)
   - Измеряет время поиска лучшей цены
   - Генерирует HTML-отчёт с графиками

2. **Нагрузочный тест matching engine**: Реализуй тест:
   - Симулирует поток ордеров от нескольких клиентов
   - Измеряет задержку матчинга ордеров
   - Находит точку насыщения системы
   - Строит график latency vs throughput

3. **Профилировщик торговой стратегии**: Создай инструмент:
   - Измеряет время расчёта каждого индикатора
   - Находит узкие места в стратегии
   - Предлагает оптимизации
   - Сравнивает производительность до и после оптимизации

4. **Тест WebSocket потока**: Напиши тест:
   - Симулирует поток рыночных данных с высокой частотой
   - Измеряет задержку от получения до обработки
   - Определяет максимальную пропускную способность
   - Тестирует различные форматы сериализации

## Домашнее задание

1. **Полноценный нагрузочный тест системы**: Создай комплексный тест:
   - Включает все компоненты торговой системы
   - Поддерживает разные сценарии нагрузки
   - Генерирует детальный отчёт с графиками
   - Сравнивает результаты с baseline
   - Автоматически определяет регрессии производительности

2. **Chaos Engineering для трейдинга**: Реализуй инструмент:
   - Вносит случайные задержки в обработку
   - Симулирует отказы компонентов
   - Проверяет устойчивость системы
   - Измеряет время восстановления
   - Генерирует отчёт о найденных проблемах

3. **Сравнительный бенчмарк аллокаторов**: Создай тест:
   - Сравнивает System, Jemalloc, Mimalloc под нагрузкой
   - Измеряет throughput и latency для каждого
   - Анализирует влияние на память
   - Даёт рекомендации для разных сценариев
   - Включает долгосрочный тест на утечки памяти

4. **Автоматизированный CI/CD бенчмаркинг**: Разработай систему:
   - Запускает бенчмарки на каждый коммит
   - Сохраняет исторические данные
   - Обнаруживает регрессии производительности
   - Отправляет уведомления при проблемах
   - Интегрируется с GitHub Actions

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../334-*/ru.md)
