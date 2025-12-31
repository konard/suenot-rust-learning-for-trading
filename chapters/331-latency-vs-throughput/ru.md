# День 331: Latency vs Throughput

## Аналогия из трейдинга

Представь два разных типа торговых систем:

**Система A (High-Frequency Trading — HFT):**
- Обрабатывает один ордер за 10 микросекунд
- Но может обработать только 1000 ордеров в секунду
- Приоритет: **минимальная задержка** (latency)

**Система B (Batch Trading):**
- Обрабатывает один ордер за 100 миллисекунд
- Но может обработать 100 000 ордеров в секунду
- Приоритет: **максимальная пропускная способность** (throughput)

Это как разница между:
- **Гоночным болидом F1** — невероятно быстрый на одном круге, но может перевезти только одного человека
- **Грузовым поездом** — медленнее, но перевозит тысячи тонн груза за рейс

В трейдинге это критически важное архитектурное решение:
- HFT-системы жертвуют throughput ради минимального latency
- Системы массовой обработки жертвуют latency ради максимального throughput

## Что такое Latency и Throughput?

### Latency (Задержка)
Время от отправки запроса до получения ответа.

```
┌─────────────────────────────────────────────────────────────┐
│                    Latency                                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Отправка ордера → [Обработка] → Подтверждение              │
│        │                              │                      │
│        └──────── 10 мкс ──────────────┘                      │
│                                                              │
│  Измеряется в: микросекундах (мкс), миллисекундах (мс)      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Throughput (Пропускная способность)
Количество операций, обрабатываемых за единицу времени.

```
┌─────────────────────────────────────────────────────────────┐
│                    Throughput                                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                    │
│  │Ордер│ │Ордер│ │Ордер│ │Ордер│ │Ордер│  ──→ 5 ордеров/с  │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘                    │
│                                                              │
│  Измеряется в: операциях/секунду, запросах/секунду          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Соотношение между ними

| Характеристика | Низкий Latency | Высокий Throughput |
|----------------|----------------|---------------------|
| **Архитектура** | Последовательная обработка | Параллельная обработка |
| **Буферизация** | Минимальная | Агрессивная |
| **Пакетирование** | Нет | Да |
| **Пример в трейдинге** | HFT, арбитраж | Отчёты, бэктестинг |
| **Приоритет** | Время отклика | Объём обработки |

## Измерение Latency и Throughput в Rust

```rust
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Метрики производительности торговой системы
struct PerformanceMetrics {
    latencies: VecDeque<Duration>,
    window_size: usize,
    total_operations: u64,
    start_time: Instant,
}

impl PerformanceMetrics {
    fn new(window_size: usize) -> Self {
        PerformanceMetrics {
            latencies: VecDeque::with_capacity(window_size),
            window_size,
            total_operations: 0,
            start_time: Instant::now(),
        }
    }

    /// Записать latency операции
    fn record_latency(&mut self, latency: Duration) {
        if self.latencies.len() >= self.window_size {
            self.latencies.pop_front();
        }
        self.latencies.push_back(latency);
        self.total_operations += 1;
    }

    /// Средний latency
    fn avg_latency(&self) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.latencies.iter().sum();
        total / self.latencies.len() as u32
    }

    /// Percentile latency (p50, p95, p99)
    fn percentile_latency(&self, percentile: f64) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted: Vec<Duration> = self.latencies.iter().copied().collect();
        sorted.sort();

        let index = ((percentile / 100.0) * (sorted.len() - 1) as f64) as usize;
        sorted[index]
    }

    /// Текущий throughput (операций в секунду)
    fn throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_operations as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Минимальный latency
    fn min_latency(&self) -> Duration {
        self.latencies.iter().copied().min().unwrap_or(Duration::ZERO)
    }

    /// Максимальный latency
    fn max_latency(&self) -> Duration {
        self.latencies.iter().copied().max().unwrap_or(Duration::ZERO)
    }

    /// Вывести статистику
    fn print_stats(&self) {
        println!("=== Performance Metrics ===");
        println!("Total operations: {}", self.total_operations);
        println!("Throughput: {:.2} ops/sec", self.throughput());
        println!();
        println!("Latency:");
        println!("  Min: {:?}", self.min_latency());
        println!("  Avg: {:?}", self.avg_latency());
        println!("  p50: {:?}", self.percentile_latency(50.0));
        println!("  p95: {:?}", self.percentile_latency(95.0));
        println!("  p99: {:?}", self.percentile_latency(99.0));
        println!("  Max: {:?}", self.max_latency());
    }
}

fn main() {
    let mut metrics = PerformanceMetrics::new(1000);

    // Симулируем обработку ордеров
    for i in 0..1000 {
        let start = Instant::now();

        // Симуляция обработки ордера
        process_order(i);

        let latency = start.elapsed();
        metrics.record_latency(latency);
    }

    metrics.print_stats();
}

fn process_order(order_id: u32) {
    // Симуляция работы с разной нагрузкой
    let work = (order_id % 100) as u64 * 10;
    std::hint::black_box(work);
}
```

## Оптимизация для низкого Latency

### Избегание аллокаций в горячем пути

```rust
use std::time::Instant;

/// Ордер без динамических аллокаций
#[derive(Debug, Clone, Copy)]
struct LowLatencyOrder {
    id: u64,
    symbol_id: u32,      // Вместо String
    price: f64,
    quantity: f64,
    side: OrderSide,
    timestamp_ns: u64,   // Наносекунды вместо SystemTime
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

/// Пул заранее выделенных ордеров
struct OrderPool {
    orders: Vec<LowLatencyOrder>,
    free_indices: Vec<usize>,
}

impl OrderPool {
    fn new(capacity: usize) -> Self {
        let orders = (0..capacity)
            .map(|i| LowLatencyOrder {
                id: i as u64,
                symbol_id: 0,
                price: 0.0,
                quantity: 0.0,
                side: OrderSide::Buy,
                timestamp_ns: 0,
            })
            .collect();

        let free_indices = (0..capacity).rev().collect();

        OrderPool {
            orders,
            free_indices,
        }
    }

    /// Получить ордер из пула (O(1), без аллокации)
    #[inline(always)]
    fn acquire(&mut self) -> Option<&mut LowLatencyOrder> {
        self.free_indices.pop().map(|idx| &mut self.orders[idx])
    }

    /// Вернуть ордер в пул (O(1))
    #[inline(always)]
    fn release(&mut self, order: &LowLatencyOrder) {
        self.free_indices.push(order.id as usize);
    }

    fn available(&self) -> usize {
        self.free_indices.len()
    }
}

fn main() {
    let mut pool = OrderPool::new(10000);
    let mut latencies = Vec::with_capacity(1000);

    println!("=== Low Latency Order Pool ===\n");
    println!("Pool capacity: {}", pool.available());

    // Тест производительности
    for _ in 0..1000 {
        let start = Instant::now();

        if let Some(order) = pool.acquire() {
            order.symbol_id = 1;
            order.price = 42500.0;
            order.quantity = 0.1;
            order.side = OrderSide::Buy;
            order.timestamp_ns = start.elapsed().as_nanos() as u64;

            // Обработка...

            pool.release(order);
        }

        latencies.push(start.elapsed());
    }

    // Статистика
    latencies.sort();
    let avg: std::time::Duration = latencies.iter().sum::<std::time::Duration>() / 1000;

    println!("\nLatency statistics:");
    println!("  Min: {:?}", latencies.first().unwrap());
    println!("  Avg: {:?}", avg);
    println!("  p99: {:?}", latencies[990]);
    println!("  Max: {:?}", latencies.last().unwrap());
}
```

### Lock-free структуры данных

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Lock-free счётчик ордеров
struct OrderCounter {
    count: AtomicU64,
    total_value: AtomicU64, // В копейках для атомарности
}

impl OrderCounter {
    fn new() -> Self {
        OrderCounter {
            count: AtomicU64::new(0),
            total_value: AtomicU64::new(0),
        }
    }

    /// Добавить ордер (lock-free)
    #[inline(always)]
    fn add_order(&self, value_cents: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_value.fetch_add(value_cents, Ordering::Relaxed);
    }

    fn get_stats(&self) -> (u64, f64) {
        let count = self.count.load(Ordering::Relaxed);
        let value = self.total_value.load(Ordering::Relaxed) as f64 / 100.0;
        (count, value)
    }
}

/// Single-Producer Single-Consumer очередь (SPSC)
struct SpscQueue<T> {
    buffer: Vec<Option<T>>,
    head: AtomicU64,
    tail: AtomicU64,
    capacity: usize,
}

impl<T: Clone> SpscQueue<T> {
    fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        SpscQueue {
            buffer,
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            capacity,
        }
    }

    /// Добавить элемент (только для producer)
    fn push(&mut self, item: T) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % self.capacity as u64;

        if next_tail == self.head.load(Ordering::Acquire) {
            return false; // Очередь полна
        }

        self.buffer[tail as usize] = Some(item);
        self.tail.store(next_tail, Ordering::Release);
        true
    }

    /// Извлечь элемент (только для consumer)
    fn pop(&mut self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);

        if head == self.tail.load(Ordering::Acquire) {
            return None; // Очередь пуста
        }

        let item = self.buffer[head as usize].take();
        self.head.store((head + 1) % self.capacity as u64, Ordering::Release);
        item
    }
}

fn main() {
    let counter = Arc::new(OrderCounter::new());
    let running = Arc::new(AtomicBool::new(true));

    println!("=== Lock-free Order Counter ===\n");

    // Запускаем несколько потоков-продюсеров
    let mut handles = vec![];

    for thread_id in 0..4 {
        let counter = Arc::clone(&counter);
        let running = Arc::clone(&running);

        handles.push(thread::spawn(move || {
            let mut count = 0u64;
            while running.load(Ordering::Relaxed) {
                counter.add_order(100 + thread_id as u64);
                count += 1;
                if count >= 250000 {
                    break;
                }
            }
            count
        }));
    }

    // Даём поработать
    thread::sleep(Duration::from_secs(1));
    running.store(false, Ordering::Relaxed);

    // Собираем результаты
    let mut total_produced = 0u64;
    for handle in handles {
        total_produced += handle.join().unwrap();
    }

    let (count, value) = counter.get_stats();
    println!("Orders processed: {}", count);
    println!("Total value: ${:.2}", value);
    println!("Throughput: {:.0} orders/sec", count as f64);
}
```

## Оптимизация для высокого Throughput

### Пакетная обработка (Batching)

```rust
use std::time::{Duration, Instant};

/// Ордер для пакетной обработки
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

/// Результат обработки
#[derive(Debug)]
struct ProcessingResult {
    order_id: u64,
    success: bool,
    execution_price: f64,
}

/// Пакетный процессор ордеров
struct BatchProcessor {
    batch_size: usize,
    pending_orders: Vec<Order>,
    processed_count: u64,
}

impl BatchProcessor {
    fn new(batch_size: usize) -> Self {
        BatchProcessor {
            batch_size,
            pending_orders: Vec::with_capacity(batch_size),
            processed_count: 0,
        }
    }

    /// Добавить ордер в пакет
    fn add_order(&mut self, order: Order) -> Option<Vec<ProcessingResult>> {
        self.pending_orders.push(order);

        if self.pending_orders.len() >= self.batch_size {
            Some(self.flush())
        } else {
            None
        }
    }

    /// Обработать накопленный пакет
    fn flush(&mut self) -> Vec<ProcessingResult> {
        let orders = std::mem::take(&mut self.pending_orders);
        self.pending_orders = Vec::with_capacity(self.batch_size);

        // Пакетная обработка эффективнее по нескольким причинам:
        // 1. Меньше системных вызовов
        // 2. Лучшее использование кеша CPU
        // 3. Возможность SIMD оптимизаций
        // 4. Амортизация накладных расходов

        let results: Vec<ProcessingResult> = orders
            .iter()
            .map(|order| {
                self.processed_count += 1;
                ProcessingResult {
                    order_id: order.id,
                    success: true,
                    execution_price: order.price * 1.001, // Симуляция slippage
                }
            })
            .collect();

        results
    }

    fn pending_count(&self) -> usize {
        self.pending_orders.len()
    }

    fn processed_count(&self) -> u64 {
        self.processed_count
    }
}

/// Сравнение: поштучная vs пакетная обработка
fn compare_processing_methods() {
    println!("=== Throughput Comparison ===\n");

    let order_count = 100_000;

    // Генерируем тестовые ордера
    let orders: Vec<Order> = (0..order_count)
        .map(|i| Order {
            id: i,
            symbol: "BTCUSDT".to_string(),
            price: 42500.0 + (i as f64 * 0.01).sin() * 100.0,
            quantity: 0.1,
        })
        .collect();

    // Метод 1: Поштучная обработка
    let start = Instant::now();
    let mut individual_results = Vec::with_capacity(order_count as usize);

    for order in &orders {
        // Симуляция накладных расходов на каждый вызов
        let result = ProcessingResult {
            order_id: order.id,
            success: true,
            execution_price: order.price * 1.001,
        };
        individual_results.push(result);
    }

    let individual_time = start.elapsed();
    let individual_throughput = order_count as f64 / individual_time.as_secs_f64();

    println!("Individual processing:");
    println!("  Time: {:?}", individual_time);
    println!("  Throughput: {:.0} orders/sec", individual_throughput);

    // Метод 2: Пакетная обработка
    let start = Instant::now();
    let mut batch_processor = BatchProcessor::new(1000);
    let mut batch_results = Vec::new();

    for order in orders {
        if let Some(results) = batch_processor.add_order(order) {
            batch_results.extend(results);
        }
    }
    // Обрабатываем оставшиеся
    batch_results.extend(batch_processor.flush());

    let batch_time = start.elapsed();
    let batch_throughput = order_count as f64 / batch_time.as_secs_f64();

    println!("\nBatch processing (batch_size=1000):");
    println!("  Time: {:?}", batch_time);
    println!("  Throughput: {:.0} orders/sec", batch_throughput);
    println!("\nImprovement: {:.1}x", batch_throughput / individual_throughput);
}

fn main() {
    compare_processing_methods();
}
```

### Параллельная обработка с Rayon

```rust
use std::time::Instant;

// В Cargo.toml добавить: rayon = "1.8"
// use rayon::prelude::*;

/// OHLCV свеча
#[derive(Debug, Clone)]
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Расчёт SMA
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);
    let mut sum: f64 = prices[..period].iter().sum();
    result.push(sum / period as f64);

    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        result.push(sum / period as f64);
    }

    result
}

/// Расчёт RSI
fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period + 1 {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len() - period);

    for i in period..prices.len() {
        let mut gains = 0.0;
        let mut losses = 0.0;

        for j in (i - period + 1)..=i {
            let change = prices[j] - prices[j - 1];
            if change > 0.0 {
                gains += change;
            } else {
                losses -= change;
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
        };

        result.push(rsi);
    }

    result
}

/// Последовательная обработка множества символов
fn process_sequential(symbols_data: &[(String, Vec<f64>)]) -> Vec<(String, Vec<f64>, Vec<f64>)> {
    symbols_data
        .iter()
        .map(|(symbol, prices)| {
            let sma = calculate_sma(prices, 20);
            let rsi = calculate_rsi(prices, 14);
            (symbol.clone(), sma, rsi)
        })
        .collect()
}

/// Параллельная обработка (концептуально — без rayon для компиляции)
fn process_parallel_concept(symbols_data: &[(String, Vec<f64>)]) -> Vec<(String, Vec<f64>, Vec<f64>)> {
    // С rayon это выглядело бы так:
    // symbols_data
    //     .par_iter()
    //     .map(|(symbol, prices)| {
    //         let sma = calculate_sma(prices, 20);
    //         let rsi = calculate_rsi(prices, 14);
    //         (symbol.clone(), sma, rsi)
    //     })
    //     .collect()

    // Для демонстрации используем последовательную версию
    process_sequential(symbols_data)
}

fn main() {
    println!("=== Parallel Processing Throughput ===\n");

    // Генерируем данные для 100 символов
    let num_symbols = 100;
    let candles_per_symbol = 10_000;

    let symbols_data: Vec<(String, Vec<f64>)> = (0..num_symbols)
        .map(|i| {
            let symbol = format!("SYMBOL{:03}", i);
            let prices: Vec<f64> = (0..candles_per_symbol)
                .map(|j| 100.0 + (j as f64 * 0.01 + i as f64 * 0.1).sin() * 10.0)
                .collect();
            (symbol, prices)
        })
        .collect();

    println!("Data: {} symbols x {} candles = {} total candles\n",
             num_symbols, candles_per_symbol, num_symbols * candles_per_symbol);

    // Последовательная обработка
    let start = Instant::now();
    let results_seq = process_sequential(&symbols_data);
    let seq_time = start.elapsed();

    println!("Sequential processing:");
    println!("  Time: {:?}", seq_time);
    println!("  Throughput: {:.0} symbols/sec",
             num_symbols as f64 / seq_time.as_secs_f64());

    // Параллельная обработка (концептуально)
    let start = Instant::now();
    let results_par = process_parallel_concept(&symbols_data);
    let par_time = start.elapsed();

    println!("\nParallel processing (conceptual):");
    println!("  Time: {:?}", par_time);
    println!("  Throughput: {:.0} symbols/sec",
             num_symbols as f64 / par_time.as_secs_f64());

    // Проверка корректности
    assert_eq!(results_seq.len(), results_par.len());
    println!("\nResults validated: {} symbols processed", results_seq.len());

    println!("\nNote: With rayon, parallel processing would be ~4-8x faster");
    println!("      on a multi-core CPU depending on core count.");
}
```

## Компромисс: Latency vs Throughput

### Выбор стратегии для разных сценариев

```rust
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Тип торговой операции
#[derive(Debug, Clone, Copy)]
enum OperationType {
    /// Срочный арбитраж — нужен минимальный latency
    Arbitrage,
    /// Market making — баланс latency/throughput
    MarketMaking,
    /// Отчёты и аналитика — приоритет throughput
    Analytics,
    /// Бэктестинг — максимальный throughput
    Backtesting,
}

impl OperationType {
    fn recommended_batch_size(&self) -> usize {
        match self {
            OperationType::Arbitrage => 1,      // Без пакетирования
            OperationType::MarketMaking => 10,  // Небольшие пакеты
            OperationType::Analytics => 1000,   // Средние пакеты
            OperationType::Backtesting => 10000, // Большие пакеты
        }
    }

    fn max_latency(&self) -> Duration {
        match self {
            OperationType::Arbitrage => Duration::from_micros(100),
            OperationType::MarketMaking => Duration::from_millis(10),
            OperationType::Analytics => Duration::from_secs(1),
            OperationType::Backtesting => Duration::from_secs(60),
        }
    }
}

/// Адаптивный процессор с настраиваемым балансом
struct AdaptiveProcessor {
    operation_type: OperationType,
    pending: VecDeque<u64>,
    batch_size: usize,
    processed: u64,
    total_latency: Duration,
}

impl AdaptiveProcessor {
    fn new(operation_type: OperationType) -> Self {
        let batch_size = operation_type.recommended_batch_size();
        AdaptiveProcessor {
            operation_type,
            pending: VecDeque::with_capacity(batch_size),
            batch_size,
            processed: 0,
            total_latency: Duration::ZERO,
        }
    }

    /// Добавить операцию
    fn submit(&mut self, operation_id: u64) -> Option<Vec<u64>> {
        let start = Instant::now();
        self.pending.push_back(operation_id);

        // Проверяем, нужно ли обрабатывать пакет
        let should_process = match self.operation_type {
            OperationType::Arbitrage => true, // Всегда сразу
            _ => self.pending.len() >= self.batch_size,
        };

        if should_process {
            let results = self.process_batch();
            self.total_latency += start.elapsed();
            Some(results)
        } else {
            None
        }
    }

    /// Обработать пакет
    fn process_batch(&mut self) -> Vec<u64> {
        let batch: Vec<u64> = self.pending.drain(..).collect();
        self.processed += batch.len() as u64;

        // Симуляция обработки
        let work_per_item = match self.operation_type {
            OperationType::Arbitrage => 1,
            OperationType::MarketMaking => 10,
            OperationType::Analytics => 100,
            OperationType::Backtesting => 5,
        };

        for _ in 0..batch.len() * work_per_item {
            std::hint::black_box(0);
        }

        batch
    }

    /// Принудительная обработка оставшихся
    fn flush(&mut self) -> Vec<u64> {
        if self.pending.is_empty() {
            return vec![];
        }
        self.process_batch()
    }

    fn stats(&self) -> (u64, Duration) {
        (self.processed, self.total_latency)
    }
}

fn benchmark_operation_type(op_type: OperationType, num_operations: u64) {
    let mut processor = AdaptiveProcessor::new(op_type);
    let start = Instant::now();

    for i in 0..num_operations {
        let _ = processor.submit(i);
    }
    processor.flush();

    let elapsed = start.elapsed();
    let (processed, _) = processor.stats();
    let throughput = processed as f64 / elapsed.as_secs_f64();
    let avg_latency = elapsed / processed as u32;

    println!("{:?}:", op_type);
    println!("  Batch size: {}", op_type.recommended_batch_size());
    println!("  Throughput: {:.0} ops/sec", throughput);
    println!("  Avg latency: {:?}", avg_latency);
    println!("  Max acceptable latency: {:?}", op_type.max_latency());
    println!();
}

fn main() {
    println!("=== Latency vs Throughput Trade-offs ===\n");

    let num_operations = 100_000;

    benchmark_operation_type(OperationType::Arbitrage, num_operations);
    benchmark_operation_type(OperationType::MarketMaking, num_operations);
    benchmark_operation_type(OperationType::Analytics, num_operations);
    benchmark_operation_type(OperationType::Backtesting, num_operations);
}
```

## Практические рекомендации

### Когда оптимизировать Latency

| Сценарий | Причина | Техники |
|----------|---------|---------|
| **HFT** | Преимущество в скорости = прибыль | Избегать аллокаций, lock-free |
| **Арбитраж** | Окно возможности — миллисекунды | Pre-allocated buffers |
| **Market Making** | Быстрое обновление котировок | SPSC queues |
| **Стоп-лоссы** | Минимизация убытков | Inline критичный код |

### Когда оптимизировать Throughput

| Сценарий | Причина | Техники |
|----------|---------|---------|
| **Бэктестинг** | Миллионы операций | Batching, параллелизм |
| **Отчёты EOD** | Большие объёмы данных | Bulk operations |
| **Риск-расчёты** | Много инструментов | SIMD, rayon |
| **Агрегация данных** | Обработка потоков | Буферизация |

```rust
use std::time::{Duration, Instant};

/// Рекомендации по оптимизации
struct OptimizationGuide;

impl OptimizationGuide {
    /// Анализ требований и выбор стратегии
    fn analyze_requirements(
        target_latency: Duration,
        target_throughput: u64,
        operation_complexity: f64,
    ) -> String {
        let latency_critical = target_latency < Duration::from_millis(10);
        let throughput_critical = target_throughput > 100_000;

        let strategy = match (latency_critical, throughput_critical) {
            (true, false) => {
                "LATENCY-FOCUSED:\n\
                 - Use object pools to avoid allocations\n\
                 - Prefer lock-free data structures\n\
                 - Inline hot paths with #[inline(always)]\n\
                 - Avoid dynamic dispatch (dyn Trait)\n\
                 - Pre-compute what's possible\n\
                 - Use thread-per-core architecture"
            }
            (false, true) => {
                "THROUGHPUT-FOCUSED:\n\
                 - Use batch processing\n\
                 - Enable parallel processing (rayon)\n\
                 - Use async I/O for network operations\n\
                 - Buffer aggressively\n\
                 - Optimize for cache locality\n\
                 - Consider SIMD operations"
            }
            (true, true) => {
                "BALANCED (Hard!):\n\
                 - Use bounded queues between stages\n\
                 - Implement backpressure\n\
                 - Consider separate paths for fast/slow\n\
                 - Profile extensively\n\
                 - May need to relax one requirement"
            }
            (false, false) => {
                "RELAXED:\n\
                 - Focus on code clarity first\n\
                 - Standard Rust idioms are fine\n\
                 - Optimize only proven bottlenecks\n\
                 - Use profiler to find hot spots"
            }
        };

        format!("Requirements Analysis:\n\
                 - Target latency: {:?} (critical: {})\n\
                 - Target throughput: {} ops/sec (critical: {})\n\
                 - Operation complexity: {:.1}\n\n\
                 Recommended Strategy:\n{}",
                target_latency, latency_critical,
                target_throughput, throughput_critical,
                operation_complexity, strategy)
    }
}

fn main() {
    println!("=== Optimization Guide ===\n");

    // HFT сценарий
    println!("1. HFT Trading System:");
    println!("{}\n", OptimizationGuide::analyze_requirements(
        Duration::from_micros(100),
        10_000,
        0.5,
    ));

    // Бэктестинг
    println!("2. Backtesting Engine:");
    println!("{}\n", OptimizationGuide::analyze_requirements(
        Duration::from_secs(1),
        1_000_000,
        1.0,
    ));

    // Критичная система
    println!("3. Real-time Risk Engine:");
    println!("{}\n", OptimizationGuide::analyze_requirements(
        Duration::from_millis(5),
        500_000,
        2.0,
    ));
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Latency** | Время обработки одной операции |
| **Throughput** | Количество операций в единицу времени |
| **Trade-off** | Улучшение одного часто ухудшает другое |
| **Object Pool** | Повторное использование объектов |
| **Lock-free** | Структуры без блокировок |
| **Batching** | Пакетная обработка для throughput |
| **Percentile** | p50, p95, p99 — важнее среднего |

## Практические задания

1. **Измерение метрик**: Добавь в существующий торговый код сбор latency и throughput. Построй гистограмму latency.

2. **Object Pool**: Реализуй пул для структуры `Order` с методами `acquire()` и `release()`. Сравни производительность с обычным созданием.

3. **Adaptive Batching**: Создай систему, которая автоматически подбирает размер пакета для достижения целевого баланса latency/throughput.

4. **Lock-free Queue**: Реализуй MPSC (Multiple Producer Single Consumer) очередь без блокировок.

## Домашнее задание

1. **Профилирование**: Возьми свою торговую систему (или пример из главы) и профилируй её:
   - Измерь p50, p95, p99 latency
   - Измерь throughput
   - Определи узкие места
   - Предложи оптимизации

2. **Двухрежимный процессор**: Создай систему обработки ордеров с двумя режимами:
   - "Fast path" для VIP-клиентов (минимальный latency)
   - "Bulk path" для обычных ордеров (максимальный throughput)

3. **Симулятор нагрузки**: Напиши инструмент, который:
   - Генерирует ордера с заданным распределением (равномерное, пиковое)
   - Измеряет latency под разной нагрузкой
   - Строит график degradation (ухудшение latency при росте нагрузки)

4. **Comparison Benchmark**: Сравни три подхода обработки market data:
   - Последовательный (baseline)
   - Пакетный (разные размеры batch)
   - Параллельный (rayon)
   Определи точку пересечения, где каждый подход становится оптимальным.

## Навигация

[← Предыдущий день](../330-*/ru.md) | [Следующий день →](../332-*/ru.md)
