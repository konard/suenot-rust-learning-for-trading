# День 307: Бенчмарки: criterion crate

## Аналогия из трейдинга

Представьте трейдера, который тестирует два алгоритма исполнения ордеров:
- **Алгоритм А**: Отправляет ордера напрямую на биржу
- **Алгоритм Б**: Использует умную маршрутизацию ордеров через несколько бирж

Оба работают, но какой быстрее? Трейдеру нужны точные измерения:
- Сколько времени занимает исполнение каждого ордера?
- Какой алгоритм более стабилен?
- Ухудшается ли производительность при большом объёме ордеров?

Просто запустить каждый алгоритм один раз недостаточно — сетевые задержки варьируются, ответы биржи флуктуируют, случайные факторы вносят шум. Нужен **статистический бенчмаркинг** для получения надёжных измерений.

Именно это делает крейт **criterion** для кода на Rust: запускает ваши функции тысячи раз, собирает статистические данные, анализирует производительность, обнаруживает регрессии и генерирует подробные отчёты с графиками.

## Зачем нужны бенчмарки?

В алгоритмическом трейдинге производительность критична:

| Сценарий | Почему бенчмаркинг важен |
|----------|--------------------------|
| **Исполнение ордеров** | Микросекунды могут означать разницу между прибылью и убытком |
| **Расчёт цен** | Обработка тысяч тиков в секунду требует оптимизации |
| **Бэктестинг стратегий** | Быстрые бэктесты = больше итераций = лучшие стратегии |
| **Проверка рисков** | Лимиты позиций должны проверяться в реальном времени |
| **Обработка данных** | Потоки рыночных данных требуют эффективного парсинга |

### Что предоставляет criterion

```rust
// Без criterion: ручной замер времени (ненадёжно)
let start = std::time::Instant::now();
calculate_order_price(&order);
println!("Заняло: {:?}", start.elapsed());
// Проблема: Одно измерение, подвержено шуму, нет статистического анализа

// С criterion: профессиональный бенчмаркинг
c.bench_function("order_price_calculation", |b| {
    b.iter(|| calculate_order_price(black_box(&order)))
});
// Преимущества: Статистический анализ, обнаружение регрессий, HTML отчёты
```

## Настройка criterion

### 1. Добавление в Cargo.toml

```toml
[dev-dependencies]
criterion = { version = "0.7", features = ["html_reports"] }

[[bench]]
name = "trading_benchmarks"
harness = false
```

### 2. Создание файла бенчмарков

Создайте `benches/trading_benchmarks.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

// Простая структура ордера
#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    quantity: f64,
    price: f64,
}

// Функция для бенчмарка: расчёт общей стоимости ордера
fn calculate_order_value(order: &Order) -> f64 {
    order.quantity * order.price
}

// Функция для бенчмарка: расчёт стоимости ордера с комиссией
fn calculate_order_value_with_commission(order: &Order, commission_rate: f64) -> f64 {
    let base_value = order.quantity * order.price;
    base_value * (1.0 + commission_rate)
}

// Базовый бенчмарк
fn bench_order_value(c: &mut Criterion) {
    let order = Order {
        symbol: "BTCUSDT".to_string(),
        quantity: 1.5,
        price: 42000.0,
    };

    c.bench_function("calculate_order_value", |b| {
        b.iter(|| calculate_order_value(black_box(&order)))
    });
}

// Бенчмарк с разными входными данными
fn bench_order_value_with_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_value_by_size");

    for quantity in [0.1, 1.0, 10.0, 100.0].iter() {
        let order = Order {
            symbol: "BTCUSDT".to_string(),
            quantity: *quantity,
            price: 42000.0,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(quantity),
            &order,
            |b, order| {
                b.iter(|| calculate_order_value(black_box(order)))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_order_value, bench_order_value_with_sizes);
criterion_main!(benches);
```

### 3. Запуск бенчмарков

```bash
cargo bench
```

Вывод:
```
calculate_order_value  time:   [1.2345 ns 1.2567 ns 1.2789 ns]
order_value_by_size/0.1 time:   [1.2234 ns 1.2456 ns 1.2678 ns]
order_value_by_size/1   time:   [1.2345 ns 1.2567 ns 1.2789 ns]
order_value_by_size/10  time:   [1.2456 ns 1.2678 ns 1.2900 ns]
order_value_by_size/100 time:   [1.2567 ns 1.2789 ns 1.3011 ns]
```

## Реальный пример: Методы расчёта позиций

Давайте сравним разные подходы к расчёту позиций портфеля:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    average_price: f64,
}

// Метод 1: Используя Vec (простая итерация)
fn calculate_total_value_vec(positions: &Vec<Position>) -> f64 {
    positions.iter()
        .map(|p| p.quantity * p.average_price)
        .sum()
}

// Метод 2: Используя HashMap (быстрый поиск)
fn calculate_total_value_hashmap(positions: &HashMap<String, Position>) -> f64 {
    positions.values()
        .map(|p| p.quantity * p.average_price)
        .sum()
}

// Метод 3: Предварительно вычисленный кеш
struct PortfolioCache {
    positions: Vec<Position>,
    cached_total: f64,
    dirty: bool,
}

impl PortfolioCache {
    fn new(positions: Vec<Position>) -> Self {
        Self {
            positions,
            cached_total: 0.0,
            dirty: true,
        }
    }

    fn get_total_value(&mut self) -> f64 {
        if self.dirty {
            self.cached_total = self.positions.iter()
                .map(|p| p.quantity * p.average_price)
                .sum();
            self.dirty = false;
        }
        self.cached_total
    }
}

fn bench_position_calculations(c: &mut Criterion) {
    // Создание тестовых данных
    let positions_vec: Vec<Position> = (0..100)
        .map(|i| Position {
            symbol: format!("SYM{}", i),
            quantity: 100.0 + i as f64,
            average_price: 50.0 + (i as f64 * 0.5),
        })
        .collect();

    let positions_hashmap: HashMap<String, Position> = positions_vec
        .iter()
        .map(|p| (p.symbol.clone(), p.clone()))
        .collect();

    let mut cache = PortfolioCache::new(positions_vec.clone());

    let mut group = c.benchmark_group("position_calculations");

    group.bench_function("vec_iteration", |b| {
        b.iter(|| calculate_total_value_vec(black_box(&positions_vec)))
    });

    group.bench_function("hashmap_iteration", |b| {
        b.iter(|| calculate_total_value_hashmap(black_box(&positions_hashmap)))
    });

    group.bench_function("cached_calculation", |b| {
        b.iter(|| cache.get_total_value())
    });

    group.finish();
}

criterion_group!(benches, bench_position_calculations);
criterion_main!(benches);
```

## Бенчмаркинг расчёта ценовых индикаторов

Частая задача в трейдинге — расчёт технических индикаторов. Давайте сравним разные реализации:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// Простая скользящая средняя - наивный подход
fn sma_naive(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::new();

    for i in period - 1..prices.len() {
        let sum: f64 = prices[i - period + 1..=i].iter().sum();
        result.push(sum / period as f64);
    }

    result
}

// Простая скользящая средняя - оптимизированная с накопительной суммой
fn sma_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::with_capacity(prices.len() - period + 1);

    // Расчёт первой SMA
    let mut sum: f64 = prices[0..period].iter().sum();
    result.push(sum / period as f64);

    // Скользящий расчёт: добавляем новую цену, убираем старую
    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        result.push(sum / period as f64);
    }

    result
}

// Экспоненциальная скользящая средняя
fn ema(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::with_capacity(prices.len());
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Первая EMA — это SMA
    let first_sma: f64 = prices[0..period].iter().sum::<f64>() / period as f64;
    result.push(first_sma);

    // Расчёт последующих EMA
    for i in period..prices.len() {
        let ema_value = (prices[i] - result[result.len() - 1]) * multiplier
            + result[result.len() - 1];
        result.push(ema_value);
    }

    result
}

fn bench_indicators(c: &mut Criterion) {
    // Генерация тестовых ценовых данных
    let prices: Vec<f64> = (0..1000)
        .map(|i| 42000.0 + (i as f64 * 0.5).sin() * 1000.0)
        .collect();

    let mut group = c.benchmark_group("moving_averages");

    for period in [10, 20, 50, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("sma_naive", period),
            period,
            |b, &period| {
                b.iter(|| sma_naive(black_box(&prices), black_box(period)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sma_optimized", period),
            period,
            |b, &period| {
                b.iter(|| sma_optimized(black_box(&prices), black_box(period)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("ema", period),
            period,
            |b, &period| {
                b.iter(|| ema(black_box(&prices), black_box(period)))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_indicators);
criterion_main!(benches);
```

## Понимание black_box()

Функция `black_box()` критична для точных бенчмарков:

```rust
// БЕЗ black_box - компилятор может оптимизировать вычисления
c.bench_function("bad_benchmark", |b| {
    b.iter(|| calculate_price(&order))  // Компилятор может обнаружить, что результат не используется
});

// С black_box - предотвращает оптимизации компилятора
c.bench_function("good_benchmark", |b| {
    b.iter(|| calculate_price(black_box(&order)))  // Гарантирует выполнение вычислений
});
```

**Почему это важно**: Компилятор умный и может:
- Обнаружить, что результат не используется, и пропустить вычисление полностью
- Предварительно вычислить константные значения на этапе компиляции
- Встроить и оптимизировать вызов функции

`black_box()` говорит компилятору: "Считай это значение, как будто оно пришло извне — не оптимизируй его."

## Группы бенчмарков и сравнения

Сравнение нескольких реализаций бок о бок:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn calculate_vwap_v1(prices: &[f64], volumes: &[f64]) -> f64 {
    let total_volume: f64 = volumes.iter().sum();
    let weighted_sum: f64 = prices.iter()
        .zip(volumes.iter())
        .map(|(p, v)| p * v)
        .sum();
    weighted_sum / total_volume
}

fn calculate_vwap_v2(prices: &[f64], volumes: &[f64]) -> f64 {
    let (weighted_sum, total_volume) = prices.iter()
        .zip(volumes.iter())
        .fold((0.0, 0.0), |(sum, vol), (p, v)| {
            (sum + p * v, vol + v)
        });
    weighted_sum / total_volume
}

fn bench_vwap_comparison(c: &mut Criterion) {
    let prices: Vec<f64> = (0..1000).map(|i| 42000.0 + i as f64).collect();
    let volumes: Vec<f64> = (0..1000).map(|i| 1.0 + (i % 100) as f64).collect();

    let mut group = c.benchmark_group("vwap_comparison");

    group.bench_function("version_1_two_passes", |b| {
        b.iter(|| calculate_vwap_v1(black_box(&prices), black_box(&volumes)))
    });

    group.bench_function("version_2_single_fold", |b| {
        b.iter(|| calculate_vwap_v2(black_box(&prices), black_box(&volumes)))
    });

    group.finish();
}

criterion_group!(benches, bench_vwap_comparison);
criterion_main!(benches);
```

## Настройка конфигурации бенчмарков

Точная настройка параметров измерений:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

fn bench_with_custom_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_config");

    // Установка времени измерения
    group.measurement_time(Duration::from_secs(10));

    // Установка времени прогрева
    group.warm_up_time(Duration::from_secs(3));

    // Установка размера выборки
    group.sample_size(100);

    let order = Order {
        symbol: "BTCUSDT".to_string(),
        quantity: 1.0,
        price: 42000.0,
    };

    group.bench_function("order_calculation", |b| {
        b.iter(|| calculate_order_value(black_box(&order)))
    });

    group.finish();
}

criterion_group!(benches, bench_with_custom_config);
criterion_main!(benches);
```

## Интерпретация результатов

Criterion предоставляет подробный вывод:

```
order_calculation       time:   [1.2345 ns 1.2567 ns 1.2789 ns]
                        change: [-2.5432% -1.2345% +0.1234%] (p = 0.42 > 0.05)
                        No change in performance detected.
```

Разбираем:
- **time: [низкое медиана высокое]**: Доверительный интервал времени выполнения
- **change**: Процентное изменение по сравнению с предыдущим запуском
- **p-значение**: Статистическая значимость (< 0.05 означает обнаружено значимое изменение)
- **Вердикт производительности**: Обнаружено улучшение/регрессия или нет изменений

### HTML отчёты

Criterion генерирует подробные HTML отчёты в `target/criterion/`:
- Графики, показывающие производительность во времени
- Функции плотности вероятности
- Сравнительные графики между реализациями
- Детали статистического анализа

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| **criterion** | Библиотека для статистического бенчмаркинга в Rust |
| **black_box()** | Предотвращает оптимизации компилятора в бенчмарках |
| **criterion_group!** | Макрос для группировки связанных бенчмарков |
| **criterion_main!** | Точка входа для исполняемого файла бенчмарков |
| **BenchmarkId** | Идентифицирует параметрические бенчмарки |
| **Статистический анализ** | Автоматическое обнаружение регрессий производительности |
| **HTML отчёты** | Визуальное представление результатов бенчмарков |
| **Время прогрева** | Время, потраченное на стабилизацию перед измерением |

## Практические упражнения

### Упражнение 1: Бенчмарк операций с книгой ордеров

Создайте бенчмарки для операций с книгой ордеров:
- Добавление нового ордера в книгу
- Поиск лучшей цены bid/ask
- Сопоставление входящего ордера с книгой
- Отмена ордера по ID

Сравните производительность с 10, 100, 1000 и 10000 ордерами в книге.

### Упражнение 2: Сравнение алгоритмов сортировки

Сравните различные методы сортировки торговых сигналов по приоритету:
- Стандартный метод `sort()`
- Метод `sort_unstable()` (не сохраняет порядок равных элементов)
- Очередь с приоритетом на основе кучи
- Пользовательская сортировка с кешированными сравнениями

Протестируйте на 100, 1000 и 10000 сигналах.

### Упражнение 3: Оптимизация форматирования цен

Трейдерам нужны цены с правильной точностью. Сравните:
- Использование `format!("{:.2}", price)` для форматирования
- Предварительный расчёт строковых представлений
- Использование пользовательского форматтера чисел
- Кеширование недавно отформатированных цен

### Упражнение 4: Параллельная обработка ордеров

Сравните последовательную и параллельную валидацию ордеров:
- Обработка 1000 ордеров последовательно
- Обработка с использованием параллельного итератора `rayon`
- Обработка с использованием ручного пула потоков
- Обработка с использованием async/await с Tokio

## Домашнее задание

1. **Комплексный набор бенчмарков стратегии**: Создайте полный набор бенчмарков для торговой стратегии, который:
   - Тестирует генерацию сигналов (расчёт индикаторов)
   - Тестирует симуляцию исполнения ордеров
   - Тестирует расчёты управления рисками
   - Сравнивает минимум 3 разные реализации каждого компонента
   - Генерирует отчёт, определяющий самую быструю реализацию
   - Включает тесты с малыми (100 свечей), средними (1000) и большими (10000) наборами данных

2. **Система обнаружения регрессий**: Настройте автоматический мониторинг производительности:
   - Создайте базовый бенчмарк для ваших торговых функций
   - Запускайте бенчмарки после каждого изменения кода
   - Автоматически обнаруживайте регрессии производительности > 10%
   - Генерируйте предупреждения (вывод в консоль) при обнаружении регрессий
   - Документируйте, какие изменения кода вызвали улучшения/ухудшения производительности

3. **Компромисс память vs. скорость**: Сравните стратегии кеширования:
   - Реализуйте калькулятор ценовых индикаторов без кеширования
   - Реализуйте версию с LRU кешем (ограниченный размер)
   - Реализуйте версию с неограниченным кешем
   - Сравните использование памяти и скорость расчёта
   - Найдите оптимальный размер кеша для 10000 обновлений цен

4. **Реальный торговый сценарий**: Сравните полный поток обработки ордера:
   - Получение обновления рыночных данных
   - Расчёт технических индикаторов
   - Генерация торгового сигнала
   - Валидация ордера против правил риска
   - Форматирование и отправка ордера

   Оптимизируйте весь пайплайн для обработки менее чем за 1 миллисекунду. Определите узкое место с помощью бенчмарков criterion.

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md)
