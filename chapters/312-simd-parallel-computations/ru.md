# День 312: SIMD: параллельные вычисления

## Аналогия из трейдинга

Представь, что тебе нужно обработать ордербук с тысячами заявок — рассчитать средневзвешенную цену для каждого из 1000 торговых инструментов. Обычный подход: взять инструмент, пройти по всем его заявкам, вычислить среднюю цену, перейти к следующему. Это как кассир в банке, который обслуживает клиентов строго по очереди.

**SIMD (Single Instruction, Multiple Data)** — это как если бы в банке было 4 кассира, которые одновременно выполняют одну и ту же операцию над разными клиентами: все четверо одновременно проверяют документы, все одновременно считают деньги, все одновременно выдают чеки.

В контексте трейдинга:
- Обычный процессор: обрабатывает цены по одной
- SIMD процессор: обрабатывает 4-8-16 цен одновременно за одну инструкцию
- Ускорение: в 4-8 раз при вычислении индикаторов, обработке тиков, расчёте волатильности

Типичный пример: вычисление скользящей средней по 1000 ценам:
- Без SIMD: 1000 операций сложения
- С SIMD (AVX-256): ~125 операций (8 цен за раз)
- Результат: ускорение в 8 раз!

## Что такое SIMD?

SIMD — это технология процессора, которая позволяет выполнять одну инструкцию одновременно над несколькими данными:

| Характеристика | Описание |
|----------------|----------|
| **SSE** | 128-bit регистры, 4 × f32 или 2 × f64 |
| **AVX** | 256-bit регистры, 8 × f32 или 4 × f64 |
| **AVX-512** | 512-bit регистры, 16 × f32 или 8 × f64 |

### Когда SIMD эффективен?

✅ **Подходит:**
- Обработка массивов цен, объёмов, индикаторов
- Математические операции: сложение, умножение, min/max
- Одинаковые операции над большим количеством данных
- Вычисление технических индикаторов (SMA, EMA, RSI)

❌ **Не подходит:**
- Разные операции для каждого элемента
- Сложная логика с ветвлениями
- Малые объёмы данных (накладные расходы больше выигрыша)

## Базовый пример: Вычисление PnL

```rust
// Без SIMD: обработка по одной сделке
fn calculate_pnl_scalar(entry_prices: &[f32], exit_prices: &[f32]) -> Vec<f32> {
    entry_prices.iter()
        .zip(exit_prices.iter())
        .map(|(entry, exit)| exit - entry)
        .collect()
}

// С SIMD: обработка 4 сделок одновременно (автоматически через итераторы)
fn calculate_pnl_simd(entry_prices: &[f32], exit_prices: &[f32]) -> Vec<f32> {
    // Rust может автоматически векторизовать простые операции
    entry_prices.iter()
        .zip(exit_prices.iter())
        .map(|(entry, exit)| exit - entry)
        .collect()
}

fn main() {
    let entries = vec![100.0, 101.5, 99.8, 102.3];
    let exits = vec![105.0, 100.0, 103.2, 101.5];

    let pnl = calculate_pnl_scalar(&entries, &exits);

    println!("=== Прибыль/убыток по сделкам ===");
    for (i, profit) in pnl.iter().enumerate() {
        println!("Сделка {}: {:.2}", i + 1, profit);
    }
}
```

## Явное использование SIMD с `std::simd`

Rust предоставляет портативный интерфейс для SIMD через экспериментальный модуль `std::simd`:

```rust
#![feature(portable_simd)]
use std::simd::f32x4;

/// Вычисление простой скользящей средней (SMA) с SIMD
fn calculate_sma_simd(prices: &[f32], window: usize) -> Vec<f32> {
    if prices.len() < window {
        return vec![];
    }

    let mut sma_values = Vec::with_capacity(prices.len() - window + 1);

    for i in 0..=prices.len() - window {
        let window_prices = &prices[i..i + window];
        let sum = calculate_sum_simd(window_prices);
        sma_values.push(sum / window as f32);
    }

    sma_values
}

/// Быстрая сумма с использованием SIMD
fn calculate_sum_simd(values: &[f32]) -> f32 {
    let mut sum = 0.0f32;

    // Обработка блоками по 4 элемента
    let chunks = values.chunks_exact(4);
    let remainder = chunks.remainder();

    let mut simd_sum = f32x4::splat(0.0);

    for chunk in chunks {
        // Загружаем 4 значения в SIMD регистр
        let simd_vals = f32x4::from_slice(chunk);
        // Складываем параллельно
        simd_sum += simd_vals;
    }

    // Суммируем 4 компонента SIMD регистра
    sum += simd_sum.reduce_sum();

    // Обрабатываем оставшиеся элементы
    sum += remainder.iter().sum::<f32>();

    sum
}

fn main() {
    let prices = vec![
        100.0, 101.0, 102.0, 103.0, 104.0,
        103.5, 102.0, 101.0, 100.5, 99.0,
        98.0, 99.5, 101.0, 102.5, 104.0,
        105.0, 106.0, 107.0, 106.5, 105.0,
    ];

    let sma = calculate_sma_simd(&prices, 5);

    println!("=== SMA-5 с SIMD ===");
    for (i, value) in sma.iter().enumerate() {
        println!("Позиция {}: SMA = {:.2}", i + 5, value);
    }
}
```

## Практический пример: Вычисление волатильности

```rust
#![feature(portable_simd)]
use std::simd::f32x8;

#[derive(Debug)]
struct VolatilityMetrics {
    std_dev: f32,
    variance: f32,
    mean: f32,
}

/// Расчёт волатильности с использованием SIMD
fn calculate_volatility_simd(returns: &[f32]) -> VolatilityMetrics {
    if returns.is_empty() {
        return VolatilityMetrics {
            std_dev: 0.0,
            variance: 0.0,
            mean: 0.0,
        };
    }

    // Шаг 1: Вычисление среднего с SIMD
    let mean = calculate_mean_simd(returns);

    // Шаг 2: Вычисление дисперсии
    let mut sum_squared_diff = 0.0f32;

    let chunks = returns.chunks_exact(8);
    let remainder = chunks.remainder();

    let mean_simd = f32x8::splat(mean);
    let mut simd_sum_sq = f32x8::splat(0.0);

    for chunk in chunks {
        let values = f32x8::from_slice(chunk);
        let diff = values - mean_simd;
        simd_sum_sq += diff * diff;
    }

    sum_squared_diff += simd_sum_sq.reduce_sum();

    // Обработка остатка
    for &value in remainder {
        let diff = value - mean;
        sum_squared_diff += diff * diff;
    }

    let variance = sum_squared_diff / returns.len() as f32;
    let std_dev = variance.sqrt();

    VolatilityMetrics {
        std_dev,
        variance,
        mean,
    }
}

fn calculate_mean_simd(values: &[f32]) -> f32 {
    let sum = calculate_sum_simd_f32x8(values);
    sum / values.len() as f32
}

fn calculate_sum_simd_f32x8(values: &[f32]) -> f32 {
    let chunks = values.chunks_exact(8);
    let remainder = chunks.remainder();

    let mut simd_sum = f32x8::splat(0.0);

    for chunk in chunks {
        simd_sum += f32x8::from_slice(chunk);
    }

    let mut sum = simd_sum.reduce_sum();
    sum += remainder.iter().sum::<f32>();

    sum
}

/// Генерация доходностей из цен
fn calculate_returns(prices: &[f32]) -> Vec<f32> {
    prices.windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect()
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.5, 102.0,
        104.0, 103.0, 105.5, 107.0, 106.0,
        108.0, 107.5, 109.0, 108.0, 110.0,
        111.0, 109.5, 110.5, 112.0, 111.0,
    ];

    let returns = calculate_returns(&prices);
    let volatility = calculate_volatility_simd(&returns);

    println!("=== Анализ волатильности с SIMD ===");
    println!("Средняя доходность: {:.4}", volatility.mean);
    println!("Дисперсия: {:.6}", volatility.variance);
    println!("Стандартное отклонение: {:.4}", volatility.std_dev);
    println!("Волатильность (годовая): {:.2}%",
             volatility.std_dev * (252.0f32).sqrt() * 100.0);
}
```

## Вычисление индикатора RSI с SIMD

```rust
#![feature(portable_simd)]
use std::simd::f32x4;

#[derive(Debug)]
struct RsiResult {
    rsi_values: Vec<f32>,
    avg_gain: f32,
    avg_loss: f32,
}

/// Вычисление RSI (Relative Strength Index) с использованием SIMD
fn calculate_rsi_simd(prices: &[f32], period: usize) -> RsiResult {
    if prices.len() < period + 1 {
        return RsiResult {
            rsi_values: vec![],
            avg_gain: 0.0,
            avg_loss: 0.0,
        };
    }

    // Шаг 1: Вычисление изменений цены
    let mut changes = Vec::with_capacity(prices.len() - 1);
    for i in 1..prices.len() {
        changes.push(prices[i] - prices[i - 1]);
    }

    // Шаг 2: Разделение на прибыли и убытки с SIMD
    let (gains, losses) = split_gains_losses_simd(&changes);

    // Шаг 3: Вычисление первых средних значений
    let first_avg_gain = gains[..period].iter().sum::<f32>() / period as f32;
    let first_avg_loss = losses[..period].iter().sum::<f32>() / period as f32;

    // Шаг 4: Сглаженные средние и RSI
    let mut avg_gain = first_avg_gain;
    let mut avg_loss = first_avg_loss;
    let mut rsi_values = Vec::new();

    // Первое значение RSI
    let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
    rsi_values.push(100.0 - (100.0 / (1.0 + rs)));

    // Последующие значения
    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f32 + gains[i]) / period as f32;
        avg_loss = (avg_loss * (period - 1) as f32 + losses[i]) / period as f32;

        let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
        rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
    }

    RsiResult {
        rsi_values,
        avg_gain,
        avg_loss,
    }
}

/// Разделение изменений на прибыли и убытки с SIMD
fn split_gains_losses_simd(changes: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let mut gains = vec![0.0f32; changes.len()];
    let mut losses = vec![0.0f32; changes.len()];

    let chunks = changes.chunks_exact(4);
    let remainder = chunks.remainder();

    let zero = f32x4::splat(0.0);

    for (i, chunk) in chunks.enumerate() {
        let values = f32x4::from_slice(chunk);

        // Параллельное сравнение: положительные значения
        let gain_mask = values.simd_gt(zero);
        let gain_values = gain_mask.select(values, zero);

        // Параллельное сравнение: отрицательные значения (берём модуль)
        let loss_mask = values.simd_lt(zero);
        let loss_values = loss_mask.select(-values, zero);

        let idx = i * 4;
        gain_values.copy_to_slice(&mut gains[idx..idx + 4]);
        loss_values.copy_to_slice(&mut losses[idx..idx + 4]);
    }

    // Обработка остатка
    for (i, &change) in remainder.iter().enumerate() {
        let idx = changes.len() - remainder.len() + i;
        if change > 0.0 {
            gains[idx] = change;
        } else {
            losses[idx] = -change;
        }
    }

    (gains, losses)
}

fn main() {
    let prices = vec![
        44.0, 44.34, 44.09, 43.61, 44.33,
        44.83, 45.10, 45.42, 45.84, 46.08,
        45.89, 46.03, 45.61, 46.28, 46.28,
        46.00, 46.03, 46.41, 46.22, 45.64,
        46.21, 46.25, 45.71, 46.45, 45.78,
        45.35, 44.03, 44.18, 44.22, 44.57,
        43.42, 42.66, 43.13,
    ];

    let rsi_result = calculate_rsi_simd(&prices, 14);

    println!("=== RSI-14 с использованием SIMD ===");
    println!("Средняя прибыль: {:.4}", rsi_result.avg_gain);
    println!("Средний убыток: {:.4}", rsi_result.avg_loss);
    println!("\nЗначения RSI:");

    for (i, rsi) in rsi_result.rsi_values.iter().enumerate() {
        let idx = i + 14;
        println!("День {}: RSI = {:.2}", idx + 1, rsi);

        if *rsi > 70.0 {
            println!("  ⚠️  Перекупленность!");
        } else if *rsi < 30.0 {
            println!("  ⚠️  Перепроданность!");
        }
    }
}
```

## Сравнение производительности: SIMD vs Скаляр

```rust
use std::time::Instant;

/// Вычисление скользящей средней без SIMD
fn sma_scalar(prices: &[f32], window: usize) -> Vec<f32> {
    let mut result = Vec::new();

    for i in 0..=prices.len() - window {
        let sum: f32 = prices[i..i + window].iter().sum();
        result.push(sum / window as f32);
    }

    result
}

/// Генерация тестовых данных
fn generate_prices(count: usize, start: f32) -> Vec<f32> {
    let mut prices = Vec::with_capacity(count);
    let mut price = start;

    for i in 0..count {
        price += ((i % 17) as f32 - 8.0) * 0.1;
        prices.push(price);
    }

    prices
}

fn main() {
    let sizes = [1000, 10_000, 100_000, 1_000_000];

    println!("=== Сравнение производительности SIMD vs Скаляр ===\n");

    for &size in &sizes {
        let prices = generate_prices(size, 100.0);
        let window = 20;

        // Скалярная версия
        let start = Instant::now();
        let result_scalar = sma_scalar(&prices, window);
        let time_scalar = start.elapsed();

        // SIMD версия (с использованием calculate_sma_simd из предыдущих примеров)
        let start = Instant::now();
        let result_simd = calculate_sma_simd(&prices, window);
        let time_simd = start.elapsed();

        let speedup = time_scalar.as_secs_f64() / time_simd.as_secs_f64();

        println!("Размер данных: {} цен", size);
        println!("  Скаляр: {:?}", time_scalar);
        println!("  SIMD:   {:?}", time_simd);
        println!("  Ускорение: {:.2}x", speedup);
        println!();

        // Проверка корректности (первые несколько значений)
        assert_eq!(result_scalar.len(), result_simd.len());
        for i in 0..5.min(result_scalar.len()) {
            let diff = (result_scalar[i] - result_simd[i]).abs();
            assert!(diff < 0.001, "Значения отличаются!");
        }
    }
}
```

## Ограничения и подводные камни SIMD

### 1. Выравнивание данных

```rust
#![feature(portable_simd)]
use std::simd::f32x8;

fn demonstrate_alignment() {
    // Хорошо: данные выровнены
    let aligned = vec![1.0f32; 32];

    // Плохо: может быть не выровнено
    let unaligned = &aligned[1..25]; // начинается не с границы

    // SIMD требует выравнивания для оптимальной производительности
    println!("Aligned length: {}", aligned.len());
    println!("Unaligned length: {}", unaligned.len());
}
```

### 2. Остаточные элементы

```rust
fn handle_remainder_correctly(data: &[f32]) -> f32 {
    let mut sum = 0.0f32;

    // Обработка блоками
    let chunks = data.chunks_exact(8);
    let remainder = chunks.remainder();

    // SIMD обработка
    for chunk in chunks {
        sum += chunk.iter().sum::<f32>();
    }

    // Важно: не забыть обработать остаток!
    sum += remainder.iter().sum::<f32>();

    sum
}
```

### 3. Ветвления в SIMD

```rust
#![feature(portable_simd)]
use std::simd::{f32x4, Mask};

/// Пример: расчёт комиссий с условиями
fn calculate_fees_simd(volumes: &[f32], threshold: f32) -> Vec<f32> {
    let mut fees = vec![0.0f32; volumes.len()];

    let chunks = volumes.chunks_exact(4);
    let remainder = chunks.remainder();

    let threshold_simd = f32x4::splat(threshold);
    let high_fee = f32x4::splat(0.001); // 0.1%
    let low_fee = f32x4::splat(0.002);  // 0.2%

    for (i, chunk) in chunks.enumerate() {
        let vols = f32x4::from_slice(chunk);

        // Маска: объём >= порога?
        let is_high_volume = vols.simd_ge(threshold_simd);

        // Выбор комиссии на основе маски
        let fee_rate = is_high_volume.select(high_fee, low_fee);
        let fee_values = vols * fee_rate;

        let idx = i * 4;
        fee_values.copy_to_slice(&mut fees[idx..idx + 4]);
    }

    // Обработка остатка
    for (i, &vol) in remainder.iter().enumerate() {
        let idx = volumes.len() - remainder.len() + i;
        let rate = if vol >= threshold { 0.001 } else { 0.002 };
        fees[idx] = vol * rate;
    }

    fees
}

fn main() {
    let volumes = vec![
        10000.0, 5000.0, 50000.0, 3000.0,
        75000.0, 8000.0, 100000.0, 2000.0,
    ];

    let fees = calculate_fees_simd(&volumes, 10000.0);

    println!("=== Расчёт комиссий с SIMD ===");
    for (i, (vol, fee)) in volumes.iter().zip(fees.iter()).enumerate() {
        let rate = if *vol >= 10000.0 { "0.1%" } else { "0.2%" };
        println!("Сделка {}: объём = {:.0}, комиссия = {:.2} ({})",
                 i + 1, vol, fee, rate);
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **SIMD** | Single Instruction, Multiple Data — одна инструкция, много данных |
| **SSE/AVX** | Наборы инструкций SIMD в процессорах Intel/AMD |
| **f32x4, f32x8** | SIMD векторы для 4 и 8 чисел с плавающей точкой |
| **Векторизация** | Преобразование скалярного кода в SIMD |
| **Chunks** | Разбиение данных на блоки для SIMD обработки |
| **Маски** | Условные операции в SIMD через битовые маски |
| **Выравнивание** | Размещение данных на границах памяти для SIMD |
| **Remainder** | Обработка остаточных элементов, не кратных размеру SIMD |

## Практические задания

1. **Калькулятор технических индикаторов**: Реализуй с использованием SIMD:
   - Bollinger Bands (скользящая средняя + стандартное отклонение)
   - MACD (разница двух экспоненциальных скользящих средних)
   - ATR (Average True Range)

   Сравни производительность SIMD и скалярных версий.

2. **Поиск арбитражных возможностей**: Напиши функцию с SIMD:
   - Получает массивы цен bid/ask с разных бирж
   - Находит моменты, когда bid(биржа A) > ask(биржа B)
   - Вычисляет потенциальную прибыль с учётом комиссий
   - Использует маски SIMD для условий

3. **Портфельная оптимизация**: Создай SIMD-версию:
   - Расчёт корреляционной матрицы между активами
   - Вычисление весов портфеля
   - Симуляция Monte Carlo для оценки риска

   Измерь ускорение для портфеля из 50-100 активов.

## Домашнее задание

1. **Библиотека индикаторов с SIMD**: Создай модуль с:
   - Минимум 5 технических индикаторов
   - Каждый индикатор в двух версиях: скалярной и SIMD
   - Бенчмарки для сравнения производительности
   - Документация с примерами использования

2. **Анализатор паттернов свечей**: Реализуй с SIMD:
   - Поиск паттернов: doji, hammer, engulfing, etc.
   - Обработка OHLC данных параллельно
   - Векторные сравнения для определения паттернов
   - Статистика по найденным паттернам

3. **High-Frequency Data Processor**: Построй систему:
   - Обработка tick-данных в реальном времени
   - Агрегация в OHLC свечи с SIMD
   - Расчёт объёмных профилей
   - Детекция аномалий в ценах/объёмах
   - Вывод метрик производительности (тиков/сек)

4. **Сравнительный анализ**: Протестируй:
   - Разные размеры SIMD векторов (f32x4, f32x8, f32x16)
   - Влияние выравнивания данных
   - Overhead при малых объёмах данных
   - Масштабирование на больших датасетах (миллионы записей)

   Построй графики зависимости производительности от размера данных.

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md)
