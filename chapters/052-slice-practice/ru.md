# День 52: Практика слайсов: анализ части истории

## Аналогия из трейдинга

В алготрейдинге часто нужно анализировать **часть истории**, а не все данные:
- Последние 14 свечей для RSI
- Окно в 20 периодов для Bollinger Bands
- Последние 50 цен для скользящей средней
- Определённый торговый час для анализа объёмов

**Слайс (срез)** — это "окно" в массив данных, позволяющее работать с частью без копирования.

## Теория: что такое слайс?

Слайс — это ссылка на непрерывную последовательность элементов в массиве или векторе. В отличие от массива:
- Слайс **не владеет** данными (это ссылка)
- Размер слайса **не фиксирован** на этапе компиляции
- Слайс записывается как `&[T]`

```rust
fn main() {
    let prices = [100.0, 200.0, 300.0, 400.0, 500.0];

    // Полный массив: [f64; 5] — фиксированный размер
    // Слайс: &[f64] — динамический размер

    let slice: &[f64] = &prices[1..4];
    println!("Slice: {:?}", slice);  // [200.0, 300.0, 400.0]
}
```

## Синтаксис слайсов

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
                  42300.0, 42250.0, 42400.0, 42350.0, 42500.0];

    // [start..end] — от start до end (не включая end)
    let middle = &closes[3..7];
    println!("[3..7]: {:?}", middle);  // 4 элемента

    // [..end] — от начала до end
    let first_five = &closes[..5];
    println!("[..5]: {:?}", first_five);

    // [start..] — от start до конца
    let last_five = &closes[5..];
    println!("[5..]: {:?}", last_five);

    // [..] — весь массив как слайс
    let all = &closes[..];
    println!("All: {:?}", all);

    // [start..=end] — включая end
    let inclusive = &closes[0..=2];
    println!("[0..=2]: {:?}", inclusive);  // 3 элемента: индексы 0, 1, 2
}
```

## Слайсы и функции

Главное преимущество слайсов — универсальность функций:

```rust
fn main() {
    let array: [f64; 5] = [100.0, 200.0, 300.0, 400.0, 500.0];
    let vec: Vec<f64> = vec![100.0, 200.0, 300.0, 400.0, 500.0];

    // Одна функция работает и с массивом, и с вектором
    println!("Array SMA: {:.2}", calculate_sma(&array));
    println!("Vec SMA: {:.2}", calculate_sma(&vec));

    // И с частью данных
    println!("First 3 SMA: {:.2}", calculate_sma(&array[..3]));
    println!("Last 3 SMA: {:.2}", calculate_sma(&vec[2..]));
}

// Принимает слайс — работает с любым источником данных
fn calculate_sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Практический пример 1: Скользящее окно

Анализ цен с использованием скользящего окна:

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
                  42300.0, 42250.0, 42400.0, 42350.0, 42500.0];

    let window_size = 3;

    println!("=== Скользящее среднее (SMA-{}) ===", window_size);

    // Вычисляем SMA для каждого окна
    for i in 0..=(closes.len() - window_size) {
        let window = &closes[i..i + window_size];
        let sma = calculate_sma(window);
        println!("Window [{}-{}]: {:?} -> SMA: {:.2}",
                 i, i + window_size - 1, window, sma);
    }
}

fn calculate_sma(prices: &[f64]) -> f64 {
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Практический пример 2: Анализ торговых сессий

Разделение дневных данных на торговые сессии:

```rust
fn main() {
    // 24 часовых цены (имитация суточных данных)
    let hourly_prices: [f64; 24] = [
        // Азиатская сессия (0:00-8:00 UTC)
        42000.0, 42050.0, 42100.0, 42080.0, 42120.0, 42150.0, 42130.0, 42180.0,
        // Европейская сессия (8:00-16:00 UTC)
        42200.0, 42250.0, 42300.0, 42280.0, 42350.0, 42400.0, 42380.0, 42450.0,
        // Американская сессия (16:00-24:00 UTC)
        42500.0, 42480.0, 42550.0, 42600.0, 42580.0, 42650.0, 42700.0, 42680.0,
    ];

    // Слайсы для каждой сессии
    let asian = &hourly_prices[0..8];
    let european = &hourly_prices[8..16];
    let american = &hourly_prices[16..24];

    println!("=== Анализ торговых сессий ===\n");

    analyze_session("Азиатская", asian);
    analyze_session("Европейская", european);
    analyze_session("Американская", american);

    // Сравнение средних цен
    println!("\n=== Сравнение сессий ===");
    let asian_avg = calculate_avg(asian);
    let european_avg = calculate_avg(european);
    let american_avg = calculate_avg(american);

    println!("Азия -> Европа: {:+.2}%",
             (european_avg - asian_avg) / asian_avg * 100.0);
    println!("Европа -> Америка: {:+.2}%",
             (american_avg - european_avg) / european_avg * 100.0);
}

fn analyze_session(name: &str, prices: &[f64]) {
    let min = find_min(prices);
    let max = find_max(prices);
    let avg = calculate_avg(prices);
    let range = max - min;

    println!("{} сессия:", name);
    println!("  Min: ${:.2}", min);
    println!("  Max: ${:.2}", max);
    println!("  Avg: ${:.2}", avg);
    println!("  Range: ${:.2} ({:.2}%)", range, range / min * 100.0);
}

fn find_min(prices: &[f64]) -> f64 {
    *prices.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

fn find_max(prices: &[f64]) -> f64 {
    *prices.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

fn calculate_avg(prices: &[f64]) -> f64 {
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Практический пример 3: RSI на слайсах

Расчёт RSI (Relative Strength Index) с использованием слайсов:

```rust
fn main() {
    let closes = [
        44000.0, 44200.0, 44100.0, 44400.0, 44300.0,
        44600.0, 44500.0, 44700.0, 44650.0, 44800.0,
        44750.0, 44900.0, 44850.0, 45000.0, 45100.0,
        44900.0, 44800.0, 44950.0, 45050.0, 45200.0,
    ];

    let period = 14;

    println!("=== RSI Analysis (period={}) ===\n", period);

    // Рассчитываем RSI для последних точек
    for i in period..closes.len() {
        let window = &closes[i - period..=i];
        let rsi = calculate_rsi(window);

        let signal = if rsi > 70.0 {
            "ПЕРЕКУПЛЕН"
        } else if rsi < 30.0 {
            "ПЕРЕПРОДАН"
        } else {
            "НЕЙТРАЛЬНО"
        };

        println!("Index {}: Price ${:.0}, RSI: {:.2} [{}]",
                 i, closes[i], rsi, signal);
    }
}

fn calculate_rsi(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 50.0;
    }

    let mut gains = 0.0;
    let mut losses = 0.0;
    let mut gain_count = 0;
    let mut loss_count = 0;

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains += change;
            gain_count += 1;
        } else if change < 0.0 {
            losses += change.abs();
            loss_count += 1;
        }
    }

    let avg_gain = if gain_count > 0 { gains / gain_count as f64 } else { 0.0 };
    let avg_loss = if loss_count > 0 { losses / loss_count as f64 } else { 0.0 };

    if avg_loss == 0.0 {
        return 100.0;
    }

    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}
```

## Практический пример 4: Поиск паттернов

Поиск торговых паттернов с помощью слайсов:

```rust
fn main() {
    let closes = [
        42000.0, 42100.0, 42050.0, 42200.0, 42300.0,  // Рост
        42250.0, 42150.0, 42100.0, 42000.0, 41900.0,  // Падение
        41950.0, 42000.0, 42100.0, 42200.0, 42350.0,  // Рост
        42400.0, 42380.0, 42360.0, 42340.0, 42320.0,  // Консолидация
    ];

    println!("=== Поиск паттернов (окно = 5 свечей) ===\n");

    let window_size = 5;

    for i in 0..=(closes.len() - window_size) {
        let window = &closes[i..i + window_size];
        let pattern = detect_pattern(window);

        println!("Window [{:2}-{:2}]: {:?}",
                 i, i + window_size - 1, pattern);
    }
}

#[derive(Debug)]
enum Pattern {
    Uptrend,
    Downtrend,
    Consolidation,
    Reversal,
}

fn detect_pattern(prices: &[f64]) -> Pattern {
    if prices.len() < 2 {
        return Pattern::Consolidation;
    }

    let mut ups = 0;
    let mut downs = 0;

    for i in 1..prices.len() {
        if prices[i] > prices[i - 1] {
            ups += 1;
        } else if prices[i] < prices[i - 1] {
            downs += 1;
        }
    }

    let total_moves = ups + downs;
    if total_moves == 0 {
        return Pattern::Consolidation;
    }

    let up_ratio = ups as f64 / total_moves as f64;

    // Первая половина vs вторая половина
    let mid = prices.len() / 2;
    let first_half_trend = prices[mid] - prices[0];
    let second_half_trend = prices[prices.len() - 1] - prices[mid];

    // Разворот: тренды в разных направлениях
    if (first_half_trend > 0.0 && second_half_trend < 0.0) ||
       (first_half_trend < 0.0 && second_half_trend > 0.0) {
        return Pattern::Reversal;
    }

    if up_ratio >= 0.7 {
        Pattern::Uptrend
    } else if up_ratio <= 0.3 {
        Pattern::Downtrend
    } else {
        Pattern::Consolidation
    }
}
```

## Изменяемые слайсы

Слайсы могут быть изменяемыми для модификации данных:

```rust
fn main() {
    let mut prices = [100.0, 200.0, 300.0, 400.0, 500.0];

    println!("Before: {:?}", prices);

    // Изменяемый слайс
    let slice = &mut prices[1..4];

    // Применяем комиссию 1% к части цен
    apply_commission(slice, 0.01);

    println!("After commission: {:?}", prices);
}

fn apply_commission(prices: &mut [f64], rate: f64) {
    for price in prices.iter_mut() {
        *price *= 1.0 - rate;
    }
}
```

## Полезные методы слайсов

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];
    let slice = &prices[..];

    // Базовые методы
    println!("len: {}", slice.len());
    println!("is_empty: {}", slice.is_empty());
    println!("first: {:?}", slice.first());
    println!("last: {:?}", slice.last());

    // Разделение слайса
    let (left, right) = slice.split_at(2);
    println!("Left: {:?}", left);
    println!("Right: {:?}", right);

    // Окна (windows)
    println!("\nWindows of 3:");
    for window in slice.windows(3) {
        println!("  {:?}", window);
    }

    // Чанки (chunks)
    println!("\nChunks of 2:");
    for chunk in slice.chunks(2) {
        println!("  {:?}", chunk);
    }

    // Итераторы
    let sum: f64 = slice.iter().sum();
    let count = slice.iter().filter(|&&p| p > 42000.0).count();

    println!("\nSum: {}", sum);
    println!("Count > 42000: {}", count);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `&[T]` | Неизменяемый слайс |
| `&mut [T]` | Изменяемый слайс |
| `&arr[a..b]` | Слайс от a до b (не включая b) |
| `&arr[..b]` | От начала до b |
| `&arr[a..]` | От a до конца |
| `&arr[a..=b]` | От a до b (включая b) |
| `.windows(n)` | Скользящие окна размера n |
| `.chunks(n)` | Разбиение на части размера n |
| `.split_at(n)` | Разделение на две части |

## Домашнее задание

1. **Bollinger Bands**: Реализуй расчёт Bollinger Bands (SMA ± 2*стандартное отклонение) используя слайсы для скользящего окна.

2. **Сравнение периодов**: Напиши функцию, которая принимает массив цен и сравнивает первую половину со второй (средняя цена, волатильность, рост).

3. **Поиск локальных экстремумов**: Создай функцию, которая находит локальные максимумы и минимумы в массиве цен, используя окно из 3 элементов.

4. **Анализ объёмов по часам**: Имея массив из 168 значений объёмов (неделя по часам), найди:
   - Самый активный день недели
   - Самый активный час
   - Сравни объёмы выходных и будней

## Навигация

[← Предыдущий день](../051-reference-practice/ru.md) | [Следующий день →](../053-pattern-data-in-out/ru.md)
