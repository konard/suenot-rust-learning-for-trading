# Day 180: Thread Profiling

## Trading Analogy

Imagine you're running a trading system with dozens of parallel processes: one thread analyzes market data, another processes orders, a third monitors risks, and a fourth handles logging. Everything seems to work, but the system starts "lagging." Where's the problem? Which thread is "eating" all the resources?

It's like having a team of traders, and you don't know who's spending too much time on analysis and who's sitting idle. **Thread profiling** is a tool that lets you "look under the hood" of a multithreaded application and understand:

- How much time each thread spends working
- Which threads are blocked and on what
- Where bottlenecks occur
- Whether the load is properly distributed among threads

## What is Thread Profiling?

Thread profiling is the process of measuring and analyzing thread behavior in a multithreaded application. It helps answer key questions:

1. **CPU Usage** — which threads are actively using the processor?
2. **Wait Time** — how much time do threads spend in locks?
3. **Context Switches** — how often does the OS switch between threads?
4. **Resource Contention** — which threads compete for shared resources?

## Basic Profiling Using std::time

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Structure for storing thread metrics
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
    // Simulating a trading system with multiple threads
    let metrics = Arc::new(Mutex::new(Vec::<ThreadMetrics>::new()));

    let m1 = Arc::clone(&metrics);
    let m2 = Arc::clone(&metrics);
    let m3 = Arc::clone(&metrics);

    // Market analysis thread
    let market_analyzer = thread::spawn(move || {
        let mut thread_metrics = ThreadMetrics::new("MarketAnalyzer");

        for _ in 0..100 {
            let work_start = Instant::now();

            // Simulate market data analysis
            let _sum: f64 = (0..10000).map(|x| (x as f64).sin()).sum();

            thread_metrics.record_work(work_start.elapsed());
            thread::sleep(Duration::from_micros(100));
        }

        m1.lock().unwrap().push(thread_metrics);
    });

    // Order processing thread
    let order_processor = thread::spawn(move || {
        let mut thread_metrics = ThreadMetrics::new("OrderProcessor");

        for _ in 0..50 {
            let work_start = Instant::now();

            // Simulate order processing
            thread::sleep(Duration::from_millis(5));

            thread_metrics.record_work(work_start.elapsed());
        }

        m2.lock().unwrap().push(thread_metrics);
    });

    // Risk monitoring thread
    let risk_monitor = thread::spawn(move || {
        let mut thread_metrics = ThreadMetrics::new("RiskMonitor");

        for _ in 0..20 {
            let work_start = Instant::now();

            // Simulate risk checks
            thread::sleep(Duration::from_millis(10));

            thread_metrics.record_work(work_start.elapsed());
        }

        m3.lock().unwrap().push(thread_metrics);
    });

    market_analyzer.join().unwrap();
    order_processor.join().unwrap();
    risk_monitor.join().unwrap();

    // Output metrics
    println!("\n=== Thread Metrics ===\n");
    for metric in metrics.lock().unwrap().iter() {
        println!("Thread: {}", metric.thread_name);
        println!("  Total time: {:?}", metric.total_time());
        println!("  Work time: {:?}", metric.work_time);
        println!("  Operations: {}", metric.operations_count);
        println!("  Efficiency: {:.2}%", metric.efficiency());
        println!();
    }
}
```

## Mutex Lock Profiling

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Wrapper around Mutex with profiling
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

    // Create several threads competing for resources
    for i in 0..4 {
        let ob = Arc::clone(&order_book);
        let pf = Arc::clone(&portfolio);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                // Read from order book
                {
                    let book = ob.lock();
                    let _total: f64 = book.iter().map(|(_, p)| p).sum();
                }

                // Update portfolio
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

    // Output statistics
    println!("\n=== Lock Statistics ===\n");

    let ob_stats = order_book.stats();
    println!("Mutex: {}", ob_stats.name);
    println!("  Lock count: {}", ob_stats.lock_count);
    println!("  Total wait time: {:?}", ob_stats.total_wait_time);
    println!("  Total hold time: {:?}", ob_stats.total_hold_time);
    println!("  Average wait: {:?}",
        ob_stats.total_wait_time / ob_stats.lock_count.max(1) as u32);
    println!();

    let pf_stats = portfolio.stats();
    println!("Mutex: {}", pf_stats.name);
    println!("  Lock count: {}", pf_stats.lock_count);
    println!("  Total wait time: {:?}", pf_stats.total_wait_time);
    println!("  Total hold time: {:?}", pf_stats.total_hold_time);
    println!("  Average wait: {:?}",
        pf_stats.total_wait_time / pf_stats.lock_count.max(1) as u32);
}
```

## Profiling with Channels for Metrics Collection

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

    // Profiler - collects events
    let profiler = thread::spawn(move || {
        let mut events = Vec::new();

        while let Ok(event) = rx.recv_timeout(Duration::from_secs(2)) {
            events.push(event);
        }

        // Analyze collected events
        analyze_events(&events);
    });

    // Worker threads
    let mut handles = vec![];

    for i in 0..3 {
        let tx_clone = tx.clone();
        let thread_id = format!("Worker-{}", i);

        let handle = thread::spawn(move || {
            tx_clone.send(ProfileEvent::ThreadStart {
                thread_id: thread_id.clone(),
                timestamp: Instant::now(),
            }).unwrap();

            // Task 1: Market analysis
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

            // Task 2: Signal processing
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

    // Wait for worker threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Close the channel
    drop(tx);

    // Wait for profiler to finish
    profiler.join().unwrap();
}

fn analyze_events(events: &[ProfileEvent]) {
    use std::collections::HashMap;

    println!("\n=== Profiling Analysis ===\n");

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
                println!("Thread {} waited for lock {} for {:?}",
                    thread_id, lock_name, duration);
            }
        }
    }

    println!("Thread execution times:");
    for (thread_id, duration) in &thread_times {
        println!("  {}: {:?}", thread_id, duration);
    }

    println!("\nTask execution times:");
    for ((thread_id, task_name), duration) in &task_times {
        println!("  {} / {}: {:?}", thread_id, task_name, duration);
    }

    // Find the slowest task
    if let Some(((thread_id, task_name), max_duration)) =
        task_times.iter().max_by_key(|(_, d)| *d)
    {
        println!("\nSlowest task: {} in thread {} ({:?})",
            task_name, thread_id, max_duration);
    }
}
```

## Thread Activity Visualization

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

    // Visualization
    let total_duration = start_time.elapsed();
    println!("\n=== Thread Timeline ===\n");
    println!("Total duration: {:?}\n", total_duration);

    let scale = 50.0; // characters per 100ms

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

## Bottleneck Detection

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
                "[WARNING] High wait time for '{}': {:?}",
                resource_name, wait_time
            );
        }
    }

    fn report(&self) {
        let counts = self.contention_counts.lock().unwrap();
        let times = self.wait_times.lock().unwrap();

        println!("\n=== Bottleneck Report ===\n");

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

            println!("Resource: {}", resource);
            println!("  Contention count: {}", count);
            println!("  Total wait time: {:?}", total_time);
            println!("  Average wait time: {:?}", avg_time);

            if avg_time.as_millis() as u64 > self.threshold_ms {
                println!("  [!] BOTTLENECK DETECTED");
            }
            println!();
        }
    }
}

fn main() {
    let detector = Arc::new(BottleneckDetector::new(5)); // 5ms threshold

    // Simulate a "hot" lock - order book
    let order_book = Arc::new(Mutex::new(vec![(42000.0, 1.0)]));

    // "Cold" lock - configuration
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
                // Frequent access to order book
                {
                    let start = Instant::now();
                    let mut book = ob_clone.lock().unwrap();
                    let wait_time = start.elapsed();
                    detector_clone.record_contention("OrderBook", wait_time);

                    book.push((42000.0 + i as f64, 0.1));
                    thread::sleep(Duration::from_micros(100)); // hold the lock
                }

                // Rare access to configuration
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

## Trading Strategy Profiling

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
        println!("\n=== Trading Strategy Profile ===\n");

        let cycles = *self.total_cycles.lock().unwrap();
        println!("Total trading cycles: {}\n", cycles);

        self.report_times("Signal Generation", &self.signal_generation_times);
        self.report_times("Order Execution", &self.order_execution_times);
        self.report_times("Risk Check", &self.risk_check_times);
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

        // Calculate percentiles
        let mut sorted = times.clone();
        sorted.sort();
        let p50 = sorted[sorted.len() / 2];
        let p95 = sorted[(sorted.len() as f64 * 0.95) as usize];
        let p99 = sorted[(sorted.len() as f64 * 0.99) as usize];

        println!("{}:", name);
        println!("  Count: {}", times.len());
        println!("  Average: {:?}", avg);
        println!("  Min: {:?}, Max: {:?}", min, max);
        println!("  P50: {:?}, P95: {:?}, P99: {:?}", p50, p95, p99);
        println!("  Total time: {:?}", total);
        println!();
    }
}

fn simulate_signal_generation() -> f64 {
    // Simulate calculations
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

    // Start multiple strategy threads
    for _ in 0..4 {
        let profiler_clone = Arc::clone(&profiler);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                // Signal generation
                let start = Instant::now();
                let _signal = simulate_signal_generation();
                profiler_clone.record_signal_generation(start.elapsed());

                // Risk check
                let start = Instant::now();
                let risk_ok = simulate_risk_check();
                profiler_clone.record_risk_check(start.elapsed());

                // Order execution (if risk is OK)
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

## Using External Profiling Tools

### perf (Linux)

```bash
# Compile with debug symbols
cargo build --release

# CPU profiling
perf record -g ./target/release/trading_app
perf report

# Thread profiling
perf record -e sched:sched_switch -g ./target/release/trading_app
```

### flamegraph

```bash
# Installation
cargo install flamegraph

# Create flamegraph
cargo flamegraph --bin trading_app
```

### Example Using the tracing Crate

```rust
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// In real code, use:
// use tracing::{info, instrument, span, Level};

fn main() {
    // Initialize tracing (simplified)
    // tracing_subscriber::fmt::init();

    println!("[TRACE] Starting trading system");

    let mut handles = vec![];

    for i in 0..3 {
        let handle = thread::spawn(move || {
            println!("[TRACE] worker-{}: Starting work", i);

            // In real code:
            // #[instrument]
            process_market_data(i);

            println!("[TRACE] worker-{}: Finishing work", i);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("[TRACE] Trading system stopped");
}

fn process_market_data(worker_id: usize) {
    println!("[TRACE] worker-{}: Processing market data", worker_id);
    thread::sleep(Duration::from_millis(100));

    analyze_prices(worker_id);
    generate_signals(worker_id);
}

fn analyze_prices(worker_id: usize) {
    println!("[TRACE] worker-{}: Analyzing prices", worker_id);
    thread::sleep(Duration::from_millis(50));
}

fn generate_signals(worker_id: usize) {
    println!("[TRACE] worker-{}: Generating signals", worker_id);
    thread::sleep(Duration::from_millis(30));
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Time Profiling | Measuring operation execution time in threads |
| Lock Profiling | Tracking mutex wait and hold times |
| Channel-based Metrics | Asynchronous profiling data collection |
| Bottleneck Detection | Identifying resources with high contention |
| Activity Visualization | Building thread timelines |
| External Tools | perf, flamegraph, tracing |

## Practical Exercises

1. **Latency Profiler**: Create a wrapper that automatically profiles each function call and outputs a warning if execution time exceeds a given threshold.

2. **Lock Heatmap**: Implement a system that collects statistics on all mutexes in the application and outputs a "heatmap" — which locks create the most contention.

3. **Thread Pool Profiler**: Create a thread pool with built-in profiling:
   - Task queue wait time
   - Task execution time
   - Utilization of each thread

## Homework

1. **Performance Regression Detector**: Create a system that:
   - Saves baseline performance metrics
   - Compares current metrics with the baseline
   - Automatically warns about performance degradation

2. **Trading System Profiler**: Implement a full profiler for a trading bot:
   - Time from receiving market data to sending an order (end-to-end latency)
   - Time distribution across stages (parsing, analysis, decision-making, sending)
   - Statistics by order type

3. **Adaptive Profiler**: Create a profiler that:
   - Collects detailed metrics only when anomalies are detected
   - Uses sampling to reduce overhead
   - Automatically enables detailed profiling during performance drops

4. **Thread Visualizer**: Implement a system that:
   - Records activity of all threads
   - Generates an HTML report with an interactive timeline
   - Shows periods of work, waiting, and blocking

## Navigation

[← Previous day](../179-thread-pools-custom/en.md) | [Next day →](../181-concurrent-collections/en.md)
