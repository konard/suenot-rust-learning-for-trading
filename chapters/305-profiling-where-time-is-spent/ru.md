# День 305: Профилирование: где тратится время

## Аналогия из трейдинга

Представь, что твоя торговая стратегия работает, но медленно. Ты обрабатываешь 1000 тиков в секунду, но конкуренты обрабатывают 10000. Где проблема? Может быть, ты тратишь слишком много времени на вычисление индикаторов? Или на сериализацию данных? А может, узкое место в обращении к базе данных?

Это как пытаться понять, почему твой торговый бот отстаёт от рынка. Ты видишь симптом (медленная работа), но не знаешь причину. **Профилирование** — это инструмент диагностики, который показывает, где именно тратится время:

- Какие функции вызываются чаще всего?
- Какие операции занимают больше всего времени?
- Где происходят неожиданные задержки?
- Какие части кода можно оптимизировать?

Как трейдер анализирует journal of trades, чтобы понять, где теряет деньги, программист использует профилировщик, чтобы понять, где теряется время.

## Что такое профилирование?

Профилирование (profiling) — это измерение производительности программы для выявления узких мест. Профилировщик собирает данные о:

1. **Времени выполнения** — сколько времени занимает каждая функция
2. **Частоте вызовов** — как часто вызывается каждая функция
3. **Дереве вызовов** — какие функции вызывают другие функции
4. **Потреблении памяти** — сколько памяти выделяется и где

## Типы профилирования

| Тип | Описание | Применение в трейдинге |
|-----|----------|------------------------|
| CPU профилирование | Где тратится процессорное время | Оптимизация вычислений индикаторов |
| Memory профилирование | Как используется память | Поиск утечек памяти в long-running ботах |
| I/O профилирование | Время на ввод-вывод | Оптимизация работы с базой данных |
| Sampling профилирование | Периодические снимки стека | Низкие накладные расходы |
| Instrumentation профилирование | Точные измерения каждого вызова | Детальный анализ критических путей |

## Базовое профилирование вручную

```rust
use std::time::Instant;

#[derive(Debug)]
struct PriceData {
    symbol: String,
    price: f64,
    volume: f64,
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();

    for i in period..=prices.len() {
        let sum: f64 = prices[i - period..i].iter().sum();
        sma.push(sum / period as f64);
    }

    sma
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    let mut ema = Vec::new();
    let multiplier = 2.0 / (period + 1) as f64;

    if prices.is_empty() {
        return ema;
    }

    // Первое значение - простое среднее
    if prices.len() >= period {
        let initial_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;
        ema.push(initial_sma);

        // Остальные значения - экспоненциальное среднее
        for i in period..prices.len() {
            let new_ema = (prices[i] - ema.last().unwrap()) * multiplier + ema.last().unwrap();
            ema.push(new_ema);
        }
    }

    ema
}

fn analyze_market(prices: &[f64]) -> f64 {
    let sma_20 = calculate_sma(prices, 20);
    let sma_50 = calculate_sma(prices, 50);
    let ema_12 = calculate_ema(prices, 12);
    let ema_26 = calculate_ema(prices, 26);

    // Простая логика: если быстрая скользящая выше медленной, сигнал к покупке
    if let (Some(&last_sma_20), Some(&last_sma_50)) = (sma_20.last(), sma_50.last()) {
        if last_sma_20 > last_sma_50 {
            return 1.0; // Buy signal
        }
    }

    0.0 // No signal
}

fn main() {
    // Генерируем тестовые данные
    let prices: Vec<f64> = (0..10000)
        .map(|i| 50000.0 + (i as f64 * 0.01).sin() * 1000.0)
        .collect();

    println!("=== Профилирование вручную ===\n");

    // Профилируем SMA
    let start = Instant::now();
    let _sma = calculate_sma(&prices, 20);
    let sma_duration = start.elapsed();
    println!("SMA(20): {:?}", sma_duration);

    // Профилируем EMA
    let start = Instant::now();
    let _ema = calculate_ema(&prices, 20);
    let ema_duration = start.elapsed();
    println!("EMA(20): {:?}", ema_duration);

    // Профилируем полный анализ
    let start = Instant::now();
    let _signal = analyze_market(&prices);
    let analysis_duration = start.elapsed();
    println!("Полный анализ: {:?}", analysis_duration);

    // Сравниваем
    println!("\nСравнение:");
    println!("EMA медленнее SMA в {:.2}x раз",
        ema_duration.as_nanos() as f64 / sma_duration.as_nanos() as f64);
}
```

## Структурированное профилирование с метриками

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ProfileMetrics {
    name: String,
    call_count: u64,
    total_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
}

impl ProfileMetrics {
    fn new(name: &str) -> Self {
        ProfileMetrics {
            name: name.to_string(),
            call_count: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
        }
    }

    fn record(&mut self, duration: Duration) {
        self.call_count += 1;
        self.total_duration += duration;

        if duration < self.min_duration {
            self.min_duration = duration;
        }

        if duration > self.max_duration {
            self.max_duration = duration;
        }
    }

    fn avg_duration(&self) -> Duration {
        if self.call_count > 0 {
            self.total_duration / self.call_count as u32
        } else {
            Duration::ZERO
        }
    }

    fn print_report(&self) {
        println!("Функция: {}", self.name);
        println!("  Вызовов: {}", self.call_count);
        println!("  Общее время: {:?}", self.total_duration);
        println!("  Среднее: {:?}", self.avg_duration());
        println!("  Мин: {:?}, Макс: {:?}", self.min_duration, self.max_duration);

        if self.call_count > 0 {
            let total_micros = self.total_duration.as_micros();
            let calls_per_sec = if total_micros > 0 {
                (self.call_count as f64 * 1_000_000.0) / total_micros as f64
            } else {
                0.0
            };
            println!("  Пропускная способность: {:.0} вызовов/сек", calls_per_sec);
        }
        println!();
    }
}

struct Profiler {
    metrics: HashMap<String, ProfileMetrics>,
}

impl Profiler {
    fn new() -> Self {
        Profiler {
            metrics: HashMap::new(),
        }
    }

    fn profile<F, R>(&mut self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        self.metrics
            .entry(name.to_string())
            .or_insert_with(|| ProfileMetrics::new(name))
            .record(duration);

        result
    }

    fn report(&self) {
        println!("\n=== Отчёт профилирования ===\n");

        // Сортируем по общему времени
        let mut metrics: Vec<_> = self.metrics.values().collect();
        metrics.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));

        let total_time: Duration = metrics.iter().map(|m| m.total_duration).sum();

        for metric in metrics {
            metric.print_report();

            // Процент от общего времени
            if total_time.as_nanos() > 0 {
                let percentage = (metric.total_duration.as_nanos() as f64
                    / total_time.as_nanos() as f64) * 100.0;
                println!("  Доля от общего времени: {:.2}%", percentage);
                println!();
            }
        }

        println!("ИТОГО: {:?}", total_time);
    }
}

// Торговая стратегия с профилированием
struct TradingStrategy {
    profiler: Profiler,
}

impl TradingStrategy {
    fn new() -> Self {
        TradingStrategy {
            profiler: Profiler::new(),
        }
    }

    fn fetch_market_data(&mut self) -> Vec<f64> {
        self.profiler.profile("fetch_market_data", || {
            // Имитация получения данных
            std::thread::sleep(Duration::from_micros(100));
            (0..1000).map(|i| 50000.0 + (i as f64).sin() * 100.0).collect()
        })
    }

    fn calculate_indicators(&mut self, prices: &[f64]) -> (Vec<f64>, Vec<f64>) {
        self.profiler.profile("calculate_indicators", || {
            let sma = self.profiler.profile("calculate_sma", || {
                calculate_sma(prices, 20)
            });

            let ema = self.profiler.profile("calculate_ema", || {
                calculate_ema(prices, 20)
            });

            (sma, ema)
        })
    }

    fn generate_signal(&mut self, sma: &[f64], ema: &[f64]) -> i32 {
        self.profiler.profile("generate_signal", || {
            if let (Some(&last_sma), Some(&last_ema)) = (sma.last(), ema.last()) {
                if last_ema > last_sma {
                    return 1; // Buy
                } else if last_ema < last_sma {
                    return -1; // Sell
                }
            }
            0 // Hold
        })
    }

    fn execute_trade(&mut self, signal: i32) {
        self.profiler.profile("execute_trade", || {
            if signal != 0 {
                // Имитация отправки ордера
                std::thread::sleep(Duration::from_micros(50));
            }
        });
    }

    fn run_strategy(&mut self, iterations: usize) {
        for _ in 0..iterations {
            let prices = self.fetch_market_data();
            let (sma, ema) = self.calculate_indicators(&prices);
            let signal = self.generate_signal(&sma, &ema);
            self.execute_trade(signal);
        }
    }

    fn report(&self) {
        self.profiler.report();
    }
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();
    for i in period..=prices.len() {
        let sum: f64 = prices[i - period..i].iter().sum();
        sma.push(sum / period as f64);
    }
    sma
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    let mut ema = Vec::new();
    let multiplier = 2.0 / (period + 1) as f64;

    if prices.len() >= period {
        let initial_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;
        ema.push(initial_sma);

        for i in period..prices.len() {
            let new_ema = (prices[i] - ema.last().unwrap()) * multiplier + ema.last().unwrap();
            ema.push(new_ema);
        }
    }

    ema
}

fn main() {
    let mut strategy = TradingStrategy::new();

    println!("Запуск стратегии с профилированием...\n");

    let start = Instant::now();
    strategy.run_strategy(100);
    let total_time = start.elapsed();

    println!("Общее время работы: {:?}\n", total_time);

    strategy.report();
}
```

## Профилирование с использованием flamegraph

Flamegraph — это визуализация того, где тратится время в программе. Каждая "полоса" представляет функцию, ширина полосы — время выполнения.

### Установка и использование

```bash
# Установка flamegraph
cargo install flamegraph

# Для Linux нужны права на perf
sudo sysctl -w kernel.perf_event_paranoid=-1

# Создание flamegraph
cargo flamegraph --bin trading_bot

# Откроется файл flamegraph.svg в браузере
```

### Пример кода для профилирования

```rust
// trading_bot/src/main.rs
use std::time::Duration;

fn calculate_heavy_indicator(prices: &[f64]) -> Vec<f64> {
    // Сложные вычисления, которые мы хотим профилировать
    prices
        .windows(50)
        .map(|window| {
            let sum: f64 = window.iter().sum();
            let mean = sum / window.len() as f64;
            let variance: f64 = window.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / window.len() as f64;
            variance.sqrt()
        })
        .collect()
}

fn process_market_data(data: &[f64]) -> f64 {
    let volatility = calculate_heavy_indicator(data);
    let avg_volatility: f64 = volatility.iter().sum::<f64>() / volatility.len() as f64;
    avg_volatility
}

fn main() {
    let prices: Vec<f64> = (0..10000)
        .map(|i| 50000.0 + (i as f64 * 0.01).sin() * 1000.0)
        .collect();

    // Запускаем много раз, чтобы профилировщик собрал статистику
    for _ in 0..1000 {
        let _result = process_market_data(&prices);
    }
}
```

## Профилирование с criterion (бенчмарки)

Criterion — это библиотека для точных измерений производительности.

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "indicator_benchmarks"
harness = false
```

```rust
// benches/indicator_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();
    for i in period..=prices.len() {
        let sum: f64 = prices[i - period..i].iter().sum();
        sma.push(sum / period as f64);
    }
    sma
}

fn calculate_sma_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::with_capacity(prices.len().saturating_sub(period - 1));

    if prices.len() < period {
        return sma;
    }

    // Первое значение
    let mut sum: f64 = prices[..period].iter().sum();
    sma.push(sum / period as f64);

    // Остальные значения используют скользящее окно
    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        sma.push(sum / period as f64);
    }

    sma
}

fn benchmark_sma(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMA");

    for size in [100, 1000, 10000].iter() {
        let prices: Vec<f64> = (0..*size)
            .map(|i| 50000.0 + (i as f64 * 0.01).sin() * 1000.0)
            .collect();

        group.bench_with_input(
            BenchmarkId::new("naive", size),
            &prices,
            |b, prices| {
                b.iter(|| calculate_sma(black_box(prices), black_box(20)))
            }
        );

        group.bench_with_input(
            BenchmarkId::new("optimized", size),
            &prices,
            |b, prices| {
                b.iter(|| calculate_sma_optimized(black_box(prices), black_box(20)))
            }
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_sma);
criterion_main!(benches);
```

Запуск бенчмарков:

```bash
cargo bench
```

Criterion создаст подробный отчёт с:
- Средним временем выполнения
- Стандартным отклонением
- Сравнением с предыдущими запусками
- HTML-отчётом с графиками

## Профилирование памяти

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct OrderBookEntry {
    price: f64,
    quantity: f64,
    timestamp: u64,
}

struct OrderBook {
    bids: Vec<OrderBookEntry>,
    asks: Vec<OrderBookEntry>,
}

impl OrderBook {
    fn new() -> Self {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    // Неоптимальная версия - создаёт много временных объектов
    fn get_best_bid_naive(&self) -> Option<f64> {
        let sorted_bids: Vec<_> = self.bids
            .iter()
            .map(|entry| entry.price)
            .collect();

        sorted_bids.iter().max().copied()
    }

    // Оптимизированная версия - не создаёт промежуточные коллекции
    fn get_best_bid_optimized(&self) -> Option<f64> {
        self.bids
            .iter()
            .map(|entry| entry.price)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

fn main() {
    let mut book = OrderBook::new();

    // Заполняем стакан
    for i in 0..10000 {
        book.bids.push(OrderBookEntry {
            price: 50000.0 + i as f64,
            quantity: 1.0,
            timestamp: i,
        });
    }

    println!("Размер стакана: {} записей", book.bids.len());
    println!("Память на одну запись: {} байт",
        std::mem::size_of::<OrderBookEntry>());
    println!("Общая память: {} КБ",
        book.bids.len() * std::mem::size_of::<OrderBookEntry>() / 1024);
}
```

Для детального профилирования памяти используйте:

```bash
# valgrind (Linux)
cargo build --release
valgrind --tool=massif ./target/release/trading_bot

# heaptrack (Linux)
heaptrack ./target/release/trading_bot

# Instruments (macOS)
# Используйте Xcode Instruments -> Allocations
```

## Практическое профилирование: оптимизация парсинга JSON

```rust
use std::time::Instant;

// Пример данных рынка в JSON
const MARKET_DATA_JSON: &str = r#"
{
    "symbol": "BTCUSDT",
    "price": 50000.0,
    "volume": 123.45,
    "timestamp": 1234567890
}
"#;

// Версия 1: Используем serde_json (медленнее, но удобнее)
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "use_serde")]
#[derive(Deserialize, Serialize, Debug)]
struct MarketDataSerde {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

// Версия 2: Ручной парсинг (быстрее, но более хрупкий)
#[derive(Debug)]
struct MarketDataManual {
    symbol: String,
    price: f64,
    volume: f64,
    timestamp: u64,
}

impl MarketDataManual {
    fn parse_simple(json: &str) -> Option<Self> {
        // Упрощённый парсер для демонстрации
        // В реальности используйте правильный JSON парсер
        let symbol = json.split("\"symbol\":").nth(1)?
            .split('"').nth(1)?
            .to_string();

        let price = json.split("\"price\":").nth(1)?
            .split(',').next()?
            .trim()
            .parse::<f64>()
            .ok()?;

        let volume = json.split("\"volume\":").nth(1)?
            .split(',').next()?
            .trim()
            .parse::<f64>()
            .ok()?;

        let timestamp = json.split("\"timestamp\":").nth(1)?
            .split('}').next()?
            .trim()
            .parse::<u64>()
            .ok()?;

        Some(MarketDataManual {
            symbol,
            price,
            volume,
            timestamp,
        })
    }
}

fn main() {
    let iterations = 100000;

    // Профилируем ручной парсинг
    let start = Instant::now();
    for _ in 0..iterations {
        let _data = MarketDataManual::parse_simple(MARKET_DATA_JSON);
    }
    let manual_duration = start.elapsed();

    println!("=== Профилирование парсинга ===");
    println!("Ручной парсинг: {:?} ({:.0} ops/sec)",
        manual_duration,
        iterations as f64 / manual_duration.as_secs_f64()
    );

    // В реальном коде здесь был бы бенчмарк serde_json
    println!("\nДля полного сравнения запустите с флагом use_serde");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Профилирование вручную | Использование `Instant` для измерения времени выполнения |
| Структурированные метрики | Сбор статистики по вызовам функций |
| Flamegraph | Визуализация горячих путей в коде |
| Criterion | Точные бенчмарки с статистическим анализом |
| Профилирование памяти | Выявление утечек и неэффективного использования памяти |
| Оптимизация на основе данных | Измерять перед оптимизацией |

## Практические задания

1. **Профилировщик горячих функций**: Создай макрос `profile!`, который автоматически замеряет время выполнения функции и собирает статистику в глобальный профилировщик.

2. **Сравнительный бенчмарк**: Реализуй несколько вариантов вычисления Bollinger Bands и сравни их производительность с помощью criterion:
   - Наивная версия с несколькими проходами по данным
   - Оптимизированная версия с одним проходом
   - Версия с использованием SIMD (если возможно)

3. **Профилировщик аллокаций**: Создай обёртку над `Vec`, которая отслеживает все выделения памяти и выводит отчёт о том, где программа больше всего выделяет память.

## Домашнее задание

1. **Профилирование real-time стратегии**: Создай торговую стратегию и профилируй её:
   - Измерь latency от получения тика до отправки ордера
   - Найди самые медленные компоненты
   - Оптимизируй узкие места
   - Документируй улучшения производительности

2. **Flamegraph анализ**: Возьми любой сложный алгоритм (например, бэктестер стратегии):
   - Создай flamegraph
   - Проанализируй, где тратится время
   - Оптимизируй топ-3 самых медленных функций
   - Создай новый flamegraph и сравни результаты

3. **Бенчмарк-сюита**: Создай набор бенчмарков для типичных операций в трейдинге:
   - Парсинг рыночных данных (JSON, MessagePack, Protocol Buffers)
   - Вычисление индикаторов (SMA, EMA, RSI, MACD)
   - Работа с order book (вставка, удаление, поиск лучшей цены)
   - Сравни разные подходы и выбери самые быстрые

4. **Профилировщик памяти**: Реализуй систему мониторинга использования памяти:
   - Отслеживание текущего использования heap
   - Детектирование утечек памяти
   - Предупреждения при превышении порога
   - Отчёты о самых "тяжёлых" структурах данных

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md)
