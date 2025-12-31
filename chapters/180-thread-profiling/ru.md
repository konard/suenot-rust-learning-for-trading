# День 180: Профилирование потоков

## Аналогия из трейдинга

Представь, что ты управляешь торговой системой с десятками параллельных процессов: один поток анализирует рыночные данные, другой обрабатывает ордера, третий следит за рисками, четвёртый ведёт логирование. Всё вроде бы работает, но система начинает "тормозить". Где проблема? Какой поток "съедает" все ресурсы?

Это как если бы у тебя была команда трейдеров, и ты не знаешь, кто из них тратит слишком много времени на анализ, а кто простаивает. **Профилирование потоков** — это инструмент, который позволяет "заглянуть под капот" многопоточного приложения и понять:

- Сколько времени каждый поток тратит на работу
- Какие потоки блокируются и на чём
- Где возникают узкие места (bottlenecks)
- Правильно ли распределена нагрузка между потоками

## Что такое профилирование потоков?

Профилирование потоков — это процесс измерения и анализа поведения потоков в многопоточном приложении. Оно помогает ответить на ключевые вопросы:

1. **Использование CPU** — какие потоки активно используют процессор?
2. **Время ожидания** — сколько времени потоки проводят в блокировках?
3. **Контекстные переключения** — как часто ОС переключается между потоками?
4. **Конкуренция за ресурсы** — какие потоки конкурируют за общие ресурсы?

## Базовое профилирование с использованием std::time

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Структура для хранения метрик потока
#[derive(Debug, Clone)]
struct ThreadMetrics {
    thread_name: String,
    start_time: Instant,
    work_time: Duration,
    wait_time: Duration,
    operations_count: u64,
}

impl ThreadMetrics {
    fn new(name: &str) -> Self {
        ThreadMetrics {
            thread_name: name.to_string(),
            start_time: Instant::now(),
            work_time: Duration::ZERO,
            wait_time: Duration::ZERO,
            operations_count: 0,
        }
    }

    fn record_work(&mut self, duration: Duration) {
        self.work_time += duration;
        self.operations_count += 1;
    }

    fn record_wait(&mut self, duration: Duration) {
        self.wait_time += duration;
    }

    fn total_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn efficiency(&self) -> f64 {
        let total = self.total_time().as_secs_f64();
        if total > 0.0 {
            self.work_time.as_secs_f64() / total * 100.0
        } else {
            0.0
        }
    }
}

fn main() {
    // Имитация торговой системы с несколькими потоками
    let metrics = Arc::new(Mutex::new(Vec::<ThreadMetrics>::new()));

    let m1 = Arc::clone(&metrics);
    let m2 = Arc::clone(&metrics);
    let m3 = Arc::clone(&metrics);

    // Поток анализа рынка
    let market_analyzer = thread::spawn(move || {
        let mut thread_metrics = ThreadMetrics::new("MarketAnalyzer");

        for _ in 0..100 {
            let work_start = Instant::now();

            // Имитация анализа рыночных данных
            let _sum: f64 = (0..10000).map(|x| (x as f64).sin()).sum();

            thread_metrics.record_work(work_start.elapsed());
            thread::sleep(Duration::from_micros(100));
        }

        m1.lock().unwrap().push(thread_metrics);
    });

    // Поток обработки ордеров
    let order_processor = thread::spawn(move || {
        let mut thread_metrics = ThreadMetrics::new("OrderProcessor");

        for _ in 0..50 {
            let work_start = Instant::now();

            // Имитация обработки ордеров
            thread::sleep(Duration::from_millis(5));

            thread_metrics.record_work(work_start.elapsed());
        }

        m2.lock().unwrap().push(thread_metrics);
    });

    // Поток мониторинга рисков
    let risk_monitor = thread::spawn(move || {
        let mut thread_metrics = ThreadMetrics::new("RiskMonitor");

        for _ in 0..20 {
            let work_start = Instant::now();

            // Имитация проверки рисков
            thread::sleep(Duration::from_millis(10));

            thread_metrics.record_work(work_start.elapsed());
        }

        m3.lock().unwrap().push(thread_metrics);
    });

    market_analyzer.join().unwrap();
    order_processor.join().unwrap();
    risk_monitor.join().unwrap();

    // Вывод метрик
    println!("\n=== Метрики потоков ===\n");
    for metric in metrics.lock().unwrap().iter() {
        println!("Поток: {}", metric.thread_name);
        println!("  Общее время: {:?}", metric.total_time());
        println!("  Время работы: {:?}", metric.work_time);
        println!("  Операций: {}", metric.operations_count);
        println!("  Эффективность: {:.2}%", metric.efficiency());
        println!();
    }
}
```

## Профилирование блокировок Mutex

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Обёртка над Mutex с профилированием
struct ProfiledMutex<T> {
    inner: Mutex<T>,
    name: String,
    lock_count: Mutex<u64>,
    total_wait_time: Mutex<Duration>,
    total_hold_time: Mutex<Duration>,
}

impl<T> ProfiledMutex<T> {
    fn new(value: T, name: &str) -> Self {
        ProfiledMutex {
            inner: Mutex::new(value),
            name: name.to_string(),
            lock_count: Mutex::new(0),
            total_wait_time: Mutex::new(Duration::ZERO),
            total_hold_time: Mutex::new(Duration::ZERO),
        }
    }

    fn lock(&self) -> ProfiledMutexGuard<T> {
        let wait_start = Instant::now();
        let guard = self.inner.lock().unwrap();
        let wait_duration = wait_start.elapsed();

        *self.lock_count.lock().unwrap() += 1;
        *self.total_wait_time.lock().unwrap() += wait_duration;

        ProfiledMutexGuard {
            guard,
            hold_time_tracker: &self.total_hold_time,
            lock_start: Instant::now(),
        }
    }

    fn stats(&self) -> MutexStats {
        MutexStats {
            name: self.name.clone(),
            lock_count: *self.lock_count.lock().unwrap(),
            total_wait_time: *self.total_wait_time.lock().unwrap(),
            total_hold_time: *self.total_hold_time.lock().unwrap(),
        }
    }
}

struct ProfiledMutexGuard<'a, T> {
    guard: std::sync::MutexGuard<'a, T>,
    hold_time_tracker: &'a Mutex<Duration>,
    lock_start: Instant,
}

impl<T> Drop for ProfiledMutexGuard<'_, T> {
    fn drop(&mut self) {
        *self.hold_time_tracker.lock().unwrap() += self.lock_start.elapsed();
    }
}

impl<T> std::ops::Deref for ProfiledMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T> std::ops::DerefMut for ProfiledMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

#[derive(Debug)]
struct MutexStats {
    name: String,
    lock_count: u64,
    total_wait_time: Duration,
    total_hold_time: Duration,
}

fn main() {
    let order_book = Arc::new(ProfiledMutex::new(
        vec![("BTC", 42000.0), ("ETH", 2800.0)],
        "OrderBook",
    ));

    let portfolio = Arc::new(ProfiledMutex::new(
        vec![("BTC", 1.5), ("ETH", 10.0)],
        "Portfolio",
    ));

    let mut handles = vec![];

    // Создаём несколько потоков, конкурирующих за ресурсы
    for i in 0..4 {
        let ob = Arc::clone(&order_book);
        let pf = Arc::clone(&portfolio);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                // Читаем из стакана
                {
                    let book = ob.lock();
                    let _total: f64 = book.iter().map(|(_, p)| p).sum();
                }

                // Обновляем портфель
                {
                    let mut pf_guard = pf.lock();
                    if let Some((_, qty)) = pf_guard.get_mut(0) {
                        *qty += 0.001;
                    }
                }

                thread::sleep(Duration::from_micros(100 * (i + 1) as u64));
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Выводим статистику
    println!("\n=== Статистика блокировок ===\n");

    let ob_stats = order_book.stats();
    println!("Mutex: {}", ob_stats.name);
    println!("  Захватов: {}", ob_stats.lock_count);
    println!("  Общее время ожидания: {:?}", ob_stats.total_wait_time);
    println!("  Общее время удержания: {:?}", ob_stats.total_hold_time);
    println!("  Среднее ожидание: {:?}",
        ob_stats.total_wait_time / ob_stats.lock_count.max(1) as u32);
    println!();

    let pf_stats = portfolio.stats();
    println!("Mutex: {}", pf_stats.name);
    println!("  Захватов: {}", pf_stats.lock_count);
    println!("  Общее время ожидания: {:?}", pf_stats.total_wait_time);
    println!("  Общее время удержания: {:?}", pf_stats.total_hold_time);
    println!("  Среднее ожидание: {:?}",
        pf_stats.total_wait_time / pf_stats.lock_count.max(1) as u32);
}
```

## Профилирование с использованием каналов для сбора метрик

```rust
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
enum ProfileEvent {
    ThreadStart { thread_id: String, timestamp: Instant },
    ThreadEnd { thread_id: String, timestamp: Instant },
    TaskStart { thread_id: String, task_name: String, timestamp: Instant },
    TaskEnd { thread_id: String, task_name: String, timestamp: Instant },
    LockWait { thread_id: String, lock_name: String, duration: Duration },
}

fn main() {
    let (tx, rx) = mpsc::channel::<ProfileEvent>();

    // Профилировщик - собирает события
    let profiler = thread::spawn(move || {
        let mut events = Vec::new();

        while let Ok(event) = rx.recv_timeout(Duration::from_secs(2)) {
            events.push(event);
        }

        // Анализ собранных событий
        analyze_events(&events);
    });

    // Рабочие потоки
    let mut handles = vec![];

    for i in 0..3 {
        let tx_clone = tx.clone();
        let thread_id = format!("Worker-{}", i);

        let handle = thread::spawn(move || {
            tx_clone.send(ProfileEvent::ThreadStart {
                thread_id: thread_id.clone(),
                timestamp: Instant::now(),
            }).unwrap();

            // Задача 1: Анализ рынка
            tx_clone.send(ProfileEvent::TaskStart {
                thread_id: thread_id.clone(),
                task_name: "MarketAnalysis".to_string(),
                timestamp: Instant::now(),
            }).unwrap();

            thread::sleep(Duration::from_millis(50 + i as u64 * 20));

            tx_clone.send(ProfileEvent::TaskEnd {
                thread_id: thread_id.clone(),
                task_name: "MarketAnalysis".to_string(),
                timestamp: Instant::now(),
            }).unwrap();

            // Задача 2: Обработка сигналов
            tx_clone.send(ProfileEvent::TaskStart {
                thread_id: thread_id.clone(),
                task_name: "SignalProcessing".to_string(),
                timestamp: Instant::now(),
            }).unwrap();

            thread::sleep(Duration::from_millis(30 + i as u64 * 10));

            tx_clone.send(ProfileEvent::TaskEnd {
                thread_id: thread_id.clone(),
                task_name: "SignalProcessing".to_string(),
                timestamp: Instant::now(),
            }).unwrap();

            tx_clone.send(ProfileEvent::ThreadEnd {
                thread_id: thread_id.clone(),
                timestamp: Instant::now(),
            }).unwrap();
        });

        handles.push(handle);
    }

    // Ждём завершения рабочих потоков
    for handle in handles {
        handle.join().unwrap();
    }

    // Закрываем канал
    drop(tx);

    // Ждём завершения профилировщика
    profiler.join().unwrap();
}

fn analyze_events(events: &[ProfileEvent]) {
    use std::collections::HashMap;

    println!("\n=== Анализ профилирования ===\n");

    let mut thread_times: HashMap<String, Duration> = HashMap::new();
    let mut task_times: HashMap<(String, String), Duration> = HashMap::new();
    let mut task_starts: HashMap<(String, String), Instant> = HashMap::new();
    let mut thread_starts: HashMap<String, Instant> = HashMap::new();

    for event in events {
        match event {
            ProfileEvent::ThreadStart { thread_id, timestamp } => {
                thread_starts.insert(thread_id.clone(), *timestamp);
            }
            ProfileEvent::ThreadEnd { thread_id, timestamp } => {
                if let Some(start) = thread_starts.get(thread_id) {
                    thread_times.insert(thread_id.clone(), timestamp.duration_since(*start));
                }
            }
            ProfileEvent::TaskStart { thread_id, task_name, timestamp } => {
                task_starts.insert((thread_id.clone(), task_name.clone()), *timestamp);
            }
            ProfileEvent::TaskEnd { thread_id, task_name, timestamp } => {
                let key = (thread_id.clone(), task_name.clone());
                if let Some(start) = task_starts.get(&key) {
                    task_times.insert(key, timestamp.duration_since(*start));
                }
            }
            ProfileEvent::LockWait { thread_id, lock_name, duration } => {
                println!("Поток {} ожидал блокировку {} в течение {:?}",
                    thread_id, lock_name, duration);
            }
        }
    }

    println!("Время работы потоков:");
    for (thread_id, duration) in &thread_times {
        println!("  {}: {:?}", thread_id, duration);
    }

    println!("\nВремя выполнения задач:");
    for ((thread_id, task_name), duration) in &task_times {
        println!("  {} / {}: {:?}", thread_id, task_name, duration);
    }

    // Находим самую медленную задачу
    if let Some(((thread_id, task_name), max_duration)) =
        task_times.iter().max_by_key(|(_, d)| *d)
    {
        println!("\nСамая медленная задача: {} в потоке {} ({:?})",
            task_name, thread_id, max_duration);
    }
}
```

## Визуализация активности потоков

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct ThreadActivity {
    thread_name: String,
    activities: Vec<(Instant, Instant, String)>, // (start, end, activity_name)
}

fn main() {
    let start_time = Instant::now();
    let activities = Arc::new(Mutex::new(Vec::<ThreadActivity>::new()));

    let mut handles = vec![];

    let thread_configs = vec![
        ("PriceFeeder", vec![("FetchPrices", 50), ("ParseData", 30), ("Broadcast", 20)]),
        ("OrderMatcher", vec![("MatchOrders", 80), ("UpdateBook", 40)]),
        ("RiskEngine", vec![("CalcVaR", 100), ("CheckLimits", 30)]),
    ];

    for (name, tasks) in thread_configs {
        let activities_clone = Arc::clone(&activities);
        let thread_name = name.to_string();

        let handle = thread::spawn(move || {
            let mut thread_activity = ThreadActivity {
                thread_name: thread_name.clone(),
                activities: vec![],
            };

            for (task_name, duration_ms) in tasks {
                let task_start = Instant::now();
                thread::sleep(Duration::from_millis(duration_ms));
                let task_end = Instant::now();

                thread_activity.activities.push((
                    task_start,
                    task_end,
                    task_name.to_string(),
                ));
            }

            activities_clone.lock().unwrap().push(thread_activity);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Визуализация
    let total_duration = start_time.elapsed();
    println!("\n=== Временная диаграмма потоков ===\n");
    println!("Общее время: {:?}\n", total_duration);

    let scale = 50.0; // символов на 100ms

    for activity in activities.lock().unwrap().iter() {
        println!("{:15} |", activity.thread_name);

        for (start, end, task_name) in &activity.activities {
            let offset = start.duration_since(start_time).as_millis() as f64 / 100.0 * scale;
            let width = end.duration_since(*start).as_millis() as f64 / 100.0 * scale;

            let padding = " ".repeat(offset as usize);
            let bar = "=".repeat(width.max(1.0) as usize);

            println!("{:15} |{}{} {}", "", padding, bar, task_name);
        }
        println!();
    }
}
```

## Обнаружение узких мест (Bottleneck Detection)

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug)]
struct BottleneckDetector {
    contention_counts: Mutex<HashMap<String, u64>>,
    wait_times: Mutex<HashMap<String, Duration>>,
    threshold_ms: u64,
}

impl BottleneckDetector {
    fn new(threshold_ms: u64) -> Self {
        BottleneckDetector {
            contention_counts: Mutex::new(HashMap::new()),
            wait_times: Mutex::new(HashMap::new()),
            threshold_ms,
        }
    }

    fn record_contention(&self, resource_name: &str, wait_time: Duration) {
        let mut counts = self.contention_counts.lock().unwrap();
        *counts.entry(resource_name.to_string()).or_insert(0) += 1;

        let mut times = self.wait_times.lock().unwrap();
        *times.entry(resource_name.to_string()).or_insert(Duration::ZERO) += wait_time;

        if wait_time.as_millis() as u64 > self.threshold_ms {
            println!(
                "[ПРЕДУПРЕЖДЕНИЕ] Высокое время ожидания для '{}': {:?}",
                resource_name, wait_time
            );
        }
    }

    fn report(&self) {
        let counts = self.contention_counts.lock().unwrap();
        let times = self.wait_times.lock().unwrap();

        println!("\n=== Отчёт о узких местах ===\n");

        let mut resources: Vec<_> = counts.keys().collect();
        resources.sort_by_key(|k| std::cmp::Reverse(counts.get(*k).unwrap_or(&0)));

        for resource in resources {
            let count = counts.get(resource).unwrap_or(&0);
            let total_time = times.get(resource).unwrap_or(&Duration::ZERO);
            let avg_time = if *count > 0 {
                *total_time / *count as u32
            } else {
                Duration::ZERO
            };

            println!("Ресурс: {}", resource);
            println!("  Количество конфликтов: {}", count);
            println!("  Общее время ожидания: {:?}", total_time);
            println!("  Среднее время ожидания: {:?}", avg_time);

            if avg_time.as_millis() as u64 > self.threshold_ms {
                println!("  [!] УЗКОЕ МЕСТО ОБНАРУЖЕНО");
            }
            println!();
        }
    }
}

fn main() {
    let detector = Arc::new(BottleneckDetector::new(5)); // порог 5ms

    // Симулируем "горячую" блокировку - стакан ордеров
    let order_book = Arc::new(Mutex::new(vec![(42000.0, 1.0)]));

    // "Холодная" блокировка - конфигурация
    let config = Arc::new(Mutex::new(HashMap::from([
        ("max_position".to_string(), 100.0),
        ("risk_limit".to_string(), 0.02),
    ])));

    let mut handles = vec![];

    for i in 0..8 {
        let detector_clone = Arc::clone(&detector);
        let ob_clone = Arc::clone(&order_book);
        let cfg_clone = Arc::clone(&config);

        let handle = thread::spawn(move || {
            for _ in 0..50 {
                // Частый доступ к стакану
                {
                    let start = Instant::now();
                    let mut book = ob_clone.lock().unwrap();
                    let wait_time = start.elapsed();
                    detector_clone.record_contention("OrderBook", wait_time);

                    book.push((42000.0 + i as f64, 0.1));
                    thread::sleep(Duration::from_micros(100)); // удерживаем блокировку
                }

                // Редкий доступ к конфигурации
                if i % 4 == 0 {
                    let start = Instant::now();
                    let cfg = cfg_clone.lock().unwrap();
                    let wait_time = start.elapsed();
                    detector_clone.record_contention("Config", wait_time);

                    let _limit = cfg.get("risk_limit");
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    detector.report();
}
```

## Профилирование торговой стратегии

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct StrategyProfiler {
    signal_generation_times: Mutex<Vec<Duration>>,
    order_execution_times: Mutex<Vec<Duration>>,
    risk_check_times: Mutex<Vec<Duration>>,
    total_cycles: Mutex<u64>,
}

impl StrategyProfiler {
    fn new() -> Self {
        StrategyProfiler {
            signal_generation_times: Mutex::new(Vec::new()),
            order_execution_times: Mutex::new(Vec::new()),
            risk_check_times: Mutex::new(Vec::new()),
            total_cycles: Mutex::new(0),
        }
    }

    fn record_signal_generation(&self, duration: Duration) {
        self.signal_generation_times.lock().unwrap().push(duration);
    }

    fn record_order_execution(&self, duration: Duration) {
        self.order_execution_times.lock().unwrap().push(duration);
    }

    fn record_risk_check(&self, duration: Duration) {
        self.risk_check_times.lock().unwrap().push(duration);
    }

    fn increment_cycle(&self) {
        *self.total_cycles.lock().unwrap() += 1;
    }

    fn report(&self) {
        println!("\n=== Профиль торговой стратегии ===\n");

        let cycles = *self.total_cycles.lock().unwrap();
        println!("Всего торговых циклов: {}\n", cycles);

        self.report_times("Генерация сигналов", &self.signal_generation_times);
        self.report_times("Исполнение ордеров", &self.order_execution_times);
        self.report_times("Проверка рисков", &self.risk_check_times);
    }

    fn report_times(&self, name: &str, times: &Mutex<Vec<Duration>>) {
        let times = times.lock().unwrap();
        if times.is_empty() {
            return;
        }

        let total: Duration = times.iter().sum();
        let avg = total / times.len() as u32;
        let min = times.iter().min().unwrap();
        let max = times.iter().max().unwrap();

        // Вычисляем перцентили
        let mut sorted = times.clone();
        sorted.sort();
        let p50 = sorted[sorted.len() / 2];
        let p95 = sorted[(sorted.len() as f64 * 0.95) as usize];
        let p99 = sorted[(sorted.len() as f64 * 0.99) as usize];

        println!("{}:", name);
        println!("  Количество: {}", times.len());
        println!("  Среднее: {:?}", avg);
        println!("  Мин: {:?}, Макс: {:?}", min, max);
        println!("  P50: {:?}, P95: {:?}, P99: {:?}", p50, p95, p99);
        println!("  Общее время: {:?}", total);
        println!();
    }
}

fn simulate_signal_generation() -> f64 {
    // Имитация вычислений
    let prices: Vec<f64> = (0..100).map(|i| 42000.0 + (i as f64).sin() * 100.0).collect();
    let sma: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
    sma
}

fn simulate_order_execution() -> bool {
    thread::sleep(Duration::from_micros(rand_delay(50, 200)));
    true
}

fn simulate_risk_check() -> bool {
    thread::sleep(Duration::from_micros(rand_delay(10, 50)));
    true
}

fn rand_delay(min: u64, max: u64) -> u64 {
    min + (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64) % (max - min)
}

fn main() {
    let profiler = Arc::new(StrategyProfiler::new());

    let mut handles = vec![];

    // Запускаем несколько потоков стратегии
    for _ in 0..4 {
        let profiler_clone = Arc::clone(&profiler);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                // Генерация сигнала
                let start = Instant::now();
                let _signal = simulate_signal_generation();
                profiler_clone.record_signal_generation(start.elapsed());

                // Проверка рисков
                let start = Instant::now();
                let risk_ok = simulate_risk_check();
                profiler_clone.record_risk_check(start.elapsed());

                // Исполнение ордера (если риски в норме)
                if risk_ok {
                    let start = Instant::now();
                    let _executed = simulate_order_execution();
                    profiler_clone.record_order_execution(start.elapsed());
                }

                profiler_clone.increment_cycle();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    profiler.report();
}
```

## Использование внешних инструментов профилирования

### perf (Linux)

```bash
# Компиляция с символами отладки
cargo build --release

# Профилирование CPU
perf record -g ./target/release/trading_app
perf report

# Профилирование потоков
perf record -e sched:sched_switch -g ./target/release/trading_app
```

### flamegraph

```bash
# Установка
cargo install flamegraph

# Создание flamegraph
cargo flamegraph --bin trading_app
```

### Пример использования tracing crate

```rust
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// В реальном коде используйте:
// use tracing::{info, instrument, span, Level};

fn main() {
    // Инициализация tracing (упрощённо)
    // tracing_subscriber::fmt::init();

    println!("[TRACE] Запуск торговой системы");

    let mut handles = vec![];

    for i in 0..3 {
        let handle = thread::spawn(move || {
            println!("[TRACE] worker-{}: Начало работы", i);

            // В реальном коде:
            // #[instrument]
            process_market_data(i);

            println!("[TRACE] worker-{}: Завершение работы", i);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("[TRACE] Торговая система остановлена");
}

fn process_market_data(worker_id: usize) {
    println!("[TRACE] worker-{}: Обработка рыночных данных", worker_id);
    thread::sleep(Duration::from_millis(100));

    analyze_prices(worker_id);
    generate_signals(worker_id);
}

fn analyze_prices(worker_id: usize) {
    println!("[TRACE] worker-{}: Анализ цен", worker_id);
    thread::sleep(Duration::from_millis(50));
}

fn generate_signals(worker_id: usize) {
    println!("[TRACE] worker-{}: Генерация сигналов", worker_id);
    thread::sleep(Duration::from_millis(30));
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Профилирование времени | Измерение времени выполнения операций в потоках |
| Профилирование блокировок | Отслеживание времени ожидания и удержания мьютексов |
| Сбор метрик через каналы | Асинхронный сбор профилировочных данных |
| Обнаружение узких мест | Выявление ресурсов с высокой конкуренцией |
| Визуализация активности | Построение временных диаграмм потоков |
| Внешние инструменты | perf, flamegraph, tracing |

## Практические задания

1. **Профилировщик латентности**: Создай обёртку, которая автоматически профилирует каждый вызов функции и выводит предупреждение, если время выполнения превышает заданный порог.

2. **Тепловая карта блокировок**: Реализуй систему, которая собирает статистику по всем мьютексам в приложении и выводит "тепловую карту" — какие блокировки создают наибольшую конкуренцию.

3. **Профилировщик пула потоков**: Создай пул потоков с встроенным профилированием:
   - Время ожидания задач в очереди
   - Время выполнения задач
   - Загруженность каждого потока

## Домашнее задание

1. **Детектор регрессии производительности**: Создай систему, которая:
   - Сохраняет базовые метрики производительности
   - Сравнивает текущие метрики с базовыми
   - Автоматически предупреждает о деградации производительности

2. **Профилировщик торговой системы**: Реализуй полноценный профилировщик для торгового бота:
   - Время от получения рыночных данных до отправки ордера (end-to-end latency)
   - Распределение времени по этапам (парсинг, анализ, принятие решения, отправка)
   - Статистика по типам ордеров

3. **Адаптивный профилировщик**: Создай профилировщик, который:
   - Собирает детальные метрики только при обнаружении аномалий
   - Использует семплирование для снижения накладных расходов
   - Автоматически включает подробное профилирование при просадках производительности

4. **Визуализатор потоков**: Реализуй систему, которая:
   - Записывает активность всех потоков
   - Генерирует HTML-отчёт с интерактивной временной диаграммой
   - Показывает периоды работы, ожидания и блокировок

## Навигация

[← Предыдущий день](../179-thread-pools-custom/ru.md) | [Следующий день →](../181-concurrent-collections/ru.md)
