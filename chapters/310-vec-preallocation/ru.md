# День 310: Vec с предаллокацией

## Аналогия из трейдинга

Представь, что ты обрабатываешь тысячи тиков цен акций в режиме реального времени. Каждую секунду приходит поток данных, и ты добавляешь их в список для последующего анализа.

Если ты создаёшь пустой `Vec` и постепенно добавляешь элементы, Rust будет вынужден **многократно перевыделять память** — как если бы ты торговал на бирже с маленькой сумкой для денег, и каждый раз, когда сумка заполняется, тебе приходится бежать за новой, большей сумкой, перекладывать все деньги, а старую выбрасывать.

**Предаллокация (pre-allocation)** — это как прийти на биржу сразу с большим кейсом, в котором точно поместятся все твои деньги за день. Ты заранее резервируешь нужное количество места, и тебе не нужно тратить время на постоянную замену контейнера.

## Что такое предаллокация Vec?

Когда мы знаем (или можем оценить) количество элементов, которые будут добавлены в `Vec`, мы можем зарезервировать память заранее:

| Метод | Описание |
|-------|----------|
| `Vec::new()` | Создаёт пустой Vec с нулевой ёмкостью |
| `Vec::with_capacity(n)` | Создаёт Vec с предварительно зарезервированной ёмкостью для n элементов |
| `vec.reserve(n)` | Резервирует дополнительное место для как минимум n элементов |
| `vec.reserve_exact(n)` | Резервирует место точно для n дополнительных элементов |
| `vec.capacity()` | Возвращает текущую ёмкость Vec |
| `vec.len()` | Возвращает количество элементов в Vec |
| `vec.shrink_to_fit()` | Уменьшает ёмкость до текущей длины |

**Важно**: `capacity` (ёмкость) — это сколько элементов может поместиться без перевыделения памяти, а `len` (длина) — это сколько элементов фактически хранится.

## Пример без предаллокации

```rust
fn collect_price_ticks_slow(count: usize) -> Vec<f64> {
    let mut prices = Vec::new(); // Ёмкость: 0

    for i in 0..count {
        let price = 100.0 + (i as f64 * 0.01);
        prices.push(price);

        // Vec будет перевыделяться несколько раз:
        // capacity: 0 → 4 → 8 → 16 → 32 → 64 → 128 → ...
    }

    prices
}

fn main() {
    let ticks = collect_price_ticks_slow(1000);
    println!("Собрано {} тиков", ticks.len());
    println!("Ёмкость вектора: {}", ticks.capacity());
    // Capacity будет больше 1000 из-за стратегии роста
}
```

**Проблема**: При каждом переполнении Vec выделяет новый буфер (обычно в 2 раза больше), копирует все данные и освобождает старый буфер. Это медленно!

## Пример с предаллокацией

```rust
fn collect_price_ticks_fast(count: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(count); // Сразу резервируем память

    for i in 0..count {
        let price = 100.0 + (i as f64 * 0.01);
        prices.push(price);
        // Никаких перевыделений! Память уже зарезервирована
    }

    prices
}

fn main() {
    let ticks = collect_price_ticks_fast(1000);
    println!("Собрано {} тиков", ticks.len());
    println!("Ёмкость вектора: {}", ticks.capacity());
    // Capacity будет ровно 1000 (или чуть больше)
}
```

## Практический пример: сбор ордеров

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    quantity: u32,
    price: f64,
}

fn collect_orders_without_prealloc(order_count: usize) -> Vec<Order> {
    let mut orders = Vec::new(); // Плохая практика для известного размера

    for i in 0..order_count {
        orders.push(Order {
            id: i as u64,
            symbol: "BTCUSDT".to_string(),
            quantity: 1,
            price: 50000.0 + (i as f64 * 10.0),
        });
    }

    orders
}

fn collect_orders_with_prealloc(order_count: usize) -> Vec<Order> {
    let mut orders = Vec::with_capacity(order_count); // Хорошая практика

    for i in 0..order_count {
        orders.push(Order {
            id: i as u64,
            symbol: "BTCUSDT".to_string(),
            quantity: 1,
            price: 50000.0 + (i as f64 * 10.0),
        });
    }

    orders
}

fn main() {
    use std::time::Instant;

    // Тест без предаллокации
    let start = Instant::now();
    let orders1 = collect_orders_without_prealloc(10000);
    let duration1 = start.elapsed();

    // Тест с предаллокацией
    let start = Instant::now();
    let orders2 = collect_orders_with_prealloc(10000);
    let duration2 = start.elapsed();

    println!("Без предаллокации: {:?}", duration1);
    println!("С предаллокацией: {:?}", duration2);
    println!("Ускорение: {:.2}x", duration1.as_secs_f64() / duration2.as_secs_f64());
}
```

## Использование reserve() для динамического роста

Иногда мы не знаем точный размер заранее, но можем оценить:

```rust
fn process_market_data_stream() {
    let mut prices = Vec::new();

    // Получаем первую порцию данных
    let initial_batch_size = 1000;
    prices.reserve(initial_batch_size);

    for i in 0..initial_batch_size {
        prices.push(100.0 + i as f64);
    }

    println!("После первой порции: len={}, capacity={}",
             prices.len(), prices.capacity());

    // Получаем вторую порцию
    let second_batch_size = 5000;
    prices.reserve(second_batch_size);

    for i in 0..second_batch_size {
        prices.push(100.0 + i as f64);
    }

    println!("После второй порции: len={}, capacity={}",
             prices.len(), prices.capacity());
}

fn main() {
    process_market_data_stream();
}
```

## Пример: аггрегация сделок по биржевым стаканам

```rust
#[derive(Debug)]
struct Trade {
    price: f64,
    volume: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct OrderBookLevel {
    price: f64,
    total_volume: f64,
    trade_count: usize,
}

fn aggregate_trades_to_levels(trades: &[Trade], level_size: f64) -> Vec<OrderBookLevel> {
    if trades.is_empty() {
        return Vec::new();
    }

    // Оцениваем количество уровней цен
    let min_price = trades.iter().map(|t| t.price).fold(f64::INFINITY, f64::min);
    let max_price = trades.iter().map(|t| t.price).fold(f64::NEG_INFINITY, f64::max);
    let estimated_levels = ((max_price - min_price) / level_size).ceil() as usize + 1;

    // Предаллокация для уровней
    let mut levels = Vec::with_capacity(estimated_levels);

    let mut current_level_price = (min_price / level_size).floor() * level_size;
    let mut current_volume = 0.0;
    let mut current_count = 0;

    for trade in trades {
        let trade_level_price = (trade.price / level_size).floor() * level_size;

        if trade_level_price != current_level_price {
            // Сохраняем предыдущий уровень
            if current_count > 0 {
                levels.push(OrderBookLevel {
                    price: current_level_price,
                    total_volume: current_volume,
                    trade_count: current_count,
                });
            }

            // Начинаем новый уровень
            current_level_price = trade_level_price;
            current_volume = trade.volume;
            current_count = 1;
        } else {
            current_volume += trade.volume;
            current_count += 1;
        }
    }

    // Добавляем последний уровень
    if current_count > 0 {
        levels.push(OrderBookLevel {
            price: current_level_price,
            total_volume: current_volume,
            trade_count: current_count,
        });
    }

    levels
}

fn main() {
    // Генерируем тестовые сделки
    let mut trades = Vec::with_capacity(10000);
    for i in 0..10000 {
        trades.push(Trade {
            price: 50000.0 + (i as f64 % 100.0) * 0.1,
            volume: 0.1 + (i as f64 % 10.0) * 0.05,
            timestamp: i as u64,
        });
    }

    let levels = aggregate_trades_to_levels(&trades, 1.0);

    println!("Обработано {} сделок", trades.len());
    println!("Создано {} уровней цен", levels.len());
    println!("Первый уровень: {:?}", levels.first());
    println!("Последний уровень: {:?}", levels.last());
}
```

## Измерение производительности

```rust
use std::time::Instant;

fn benchmark_vec_allocations() {
    const SIZE: usize = 100_000;
    const ITERATIONS: usize = 100;

    // Тест 1: без предаллокации
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut vec = Vec::new();
        for i in 0..SIZE {
            vec.push(i as f64);
        }
    }
    let without_prealloc = start.elapsed();

    // Тест 2: с предаллокацией
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut vec = Vec::with_capacity(SIZE);
        for i in 0..SIZE {
            vec.push(i as f64);
        }
    }
    let with_prealloc = start.elapsed();

    // Тест 3: collect() из итератора (оптимальный вариант)
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let vec: Vec<f64> = (0..SIZE).map(|i| i as f64).collect();
    }
    let with_collect = start.elapsed();

    println!("Результаты для {} элементов, {} итераций:", SIZE, ITERATIONS);
    println!("  Без предаллокации: {:?}", without_prealloc);
    println!("  С предаллокацией:  {:?}", with_prealloc);
    println!("  Через collect():   {:?}", with_collect);
    println!();
    println!("Ускорение:");
    println!("  with_capacity: {:.2}x быстрее",
             without_prealloc.as_secs_f64() / with_prealloc.as_secs_f64());
    println!("  collect():     {:.2}x быстрее",
             without_prealloc.as_secs_f64() / with_collect.as_secs_f64());
}

fn main() {
    benchmark_vec_allocations();
}
```

## Когда использовать предаллокацию?

| Ситуация | Рекомендация |
|----------|--------------|
| Знаем точный размер | `Vec::with_capacity(n)` |
| Можем оценить размер | `Vec::with_capacity(estimated)` |
| Размер неизвестен, но будет много добавлений | `vec.reserve(большое_число)` при первой возможности |
| Размер непредсказуем и мал | `Vec::new()` — предаллокация не критична |
| Собираем из итератора | Используйте `.collect()` — он сам оптимизирует |

## Что мы изучили

| Концепция | Описание |
|-----------|----------|
| Предаллокация | Резервирование памяти для Vec заранее |
| `Vec::with_capacity(n)` | Создание Vec с зарезервированной ёмкостью |
| `reserve(n)` | Резервирование дополнительного места |
| `capacity()` vs `len()` | Ёмкость vs фактическое количество элементов |
| Производительность | Предаллокация избегает множественных перевыделений памяти |
| Паттерн использования | Резервируй память, когда размер известен или предсказуем |

## Практические задания

1. **Базовая предаллокация**: Напиши функцию `collect_prices(count: usize) -> Vec<f64>`, которая собирает `count` случайных цен. Измерь разницу между версией с `Vec::new()` и `Vec::with_capacity(count)`.

2. **Динамическое резервирование**: Реализуй функцию, которая читает цены из потока данных порциями по 1000 элементов. Используй `reserve()` перед каждой порцией.

3. **Оптимизация агрегации**: Создай функцию для расчёта OHLC (Open, High, Low, Close) свечей из тиков. Используй предаллокацию для вектора свечей, если знаешь количество интервалов времени.

4. **Сравнение методов**: Сравни производительность трёх подходов для создания Vec из 1 миллиона элементов:
   - `Vec::new()` + `push()`
   - `Vec::with_capacity()` + `push()`
   - Итератор + `collect()`

## Домашнее задание

1. **Анализ orderbook**: Напиши функцию, которая принимает поток обновлений orderbook (каждое обновление — структура с ценой и объёмом) и агрегирует их в уровни цен с шагом 0.01. Используй предаллокацию для оптимизации.

2. **Кэширование исторических данных**: Реализуй структуру `PriceCache`, которая хранит последние N цен. При создании используй `Vec::with_capacity(N)` и добавь метод для ротации данных (удаление старых при добавлении новых без перевыделения).

3. **Batch обработка сделок**: Создай систему для обработки сделок порциями. Реализуй `TradeProcessor`, который собирает сделки в батчи фиксированного размера (например, 1000) с предаллокацией и обрабатывает их при заполнении.

4. **Бенчмарки**: Напиши набор бенчмарков для сравнения производительности различных стратегий создания Vec:
   - Без предаллокации
   - С точной предаллокацией
   - С избыточной предаллокацией (reserve 2x от нужного)
   - С недостаточной предаллокацией (reserve 0.5x, потом догоняем)

   Протестируй на размерах: 100, 1_000, 10_000, 100_000, 1_000_000 элементов.

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md) | [Следующий день →](../311-hashmap-preallocation/ru.md)
