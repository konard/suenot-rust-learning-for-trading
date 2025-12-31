# День 49: Срезы массивов — часть ценовой серии

## Аналогия из трейдинга

В алгоритмическом трейдинге мы постоянно работаем с **частями данных**:
- Анализ последних 14 свечей для RSI
- Скользящая средняя за 20 периодов (SMA-20)
- Сравнение утренней и вечерней торговых сессий
- Выделение ценового диапазона за определённый период

**Срез (slice)** — это ссылка на непрерывную последовательность элементов. Это как "окно" в данные без копирования.

## Что такое срез?

Срез — это **заимствование части** массива или вектора:

```rust
fn main() {
    let prices = [100.0, 105.0, 103.0, 108.0, 110.0, 107.0, 112.0];

    // Срез: элементы с индекса 2 до 5 (не включая 5)
    let window: &[f64] = &prices[2..5];

    println!("Full array: {:?}", prices);
    println!("Slice [2..5]: {:?}", window);  // [103.0, 108.0, 110.0]
    println!("Slice length: {}", window.len());  // 3
}
```

**Ключевые особенности:**
- Срез не владеет данными — только ссылается на них
- Тип среза: `&[T]` (ссылка на срез)
- Не требует выделения памяти — это просто указатель + длина

## Синтаксис срезов

```rust
fn main() {
    let candles = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    // Полный синтаксис: [start..end]
    let slice1 = &candles[2..5];     // [3.0, 4.0, 5.0]

    // С начала: [..end]
    let first_three = &candles[..3]; // [1.0, 2.0, 3.0]

    // До конца: [start..]
    let last_four = &candles[6..];   // [7.0, 8.0, 9.0, 10.0]

    // Весь массив: [..]
    let all = &candles[..];          // Все 10 элементов

    // Включительный диапазон: [start..=end]
    let inclusive = &candles[2..=5]; // [3.0, 4.0, 5.0, 6.0]

    println!("slice1: {:?}", slice1);
    println!("first_three: {:?}", first_three);
    println!("last_four: {:?}", last_four);
    println!("inclusive [2..=5]: {:?}", inclusive);
}
```

## Срезы и функции

Главное преимущество срезов — **универсальность функций**:

```rust
fn main() {
    // Массив фиксированного размера
    let daily_closes: [f64; 5] = [42000.0, 42500.0, 42300.0, 42800.0, 43000.0];

    // Вектор динамического размера
    let hourly_closes: Vec<f64> = vec![42100.0, 42150.0, 42200.0, 42180.0];

    // Одна функция работает с обоими!
    println!("Daily SMA: {:.2}", calculate_sma(&daily_closes));
    println!("Hourly SMA: {:.2}", calculate_sma(&hourly_closes));

    // И с частью данных
    println!("Last 3 daily: {:.2}", calculate_sma(&daily_closes[2..]));
    println!("First 2 hourly: {:.2}", calculate_sma(&hourly_closes[..2]));
}

// Функция принимает срез — работает с массивами, векторами, частями данных
fn calculate_sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Практический пример: Анализ торговых сессий

```rust
fn main() {
    // Почасовые цены за торговый день (24 свечи)
    let hourly_prices: [f64; 24] = [
        // Азиатская сессия (00:00 - 08:00)
        42000.0, 42050.0, 42100.0, 42080.0, 42120.0, 42150.0, 42200.0, 42180.0,
        // Европейская сессия (08:00 - 16:00)
        42250.0, 42300.0, 42280.0, 42350.0, 42400.0, 42380.0, 42420.0, 42450.0,
        // Американская сессия (16:00 - 24:00)
        42500.0, 42550.0, 42480.0, 42520.0, 42600.0, 42650.0, 42700.0, 42680.0,
    ];

    // Выделяем срезы для каждой сессии
    let asian_session = &hourly_prices[0..8];
    let european_session = &hourly_prices[8..16];
    let american_session = &hourly_prices[16..24];

    println!("=== Анализ торговых сессий ===\n");

    println!("Азиатская сессия:");
    analyze_session(asian_session);

    println!("\nЕвропейская сессия:");
    analyze_session(european_session);

    println!("\nАмериканская сессия:");
    analyze_session(american_session);

    // Сравнение волатильности
    println!("\n=== Сравнение волатильности ===");
    println!("Азия: {:.2}%", calculate_volatility(asian_session));
    println!("Европа: {:.2}%", calculate_volatility(european_session));
    println!("Америка: {:.2}%", calculate_volatility(american_session));
}

fn analyze_session(prices: &[f64]) {
    let open = prices.first().unwrap();
    let close = prices.last().unwrap();
    let high = prices.iter().cloned().fold(f64::MIN, f64::max);
    let low = prices.iter().cloned().fold(f64::MAX, f64::min);
    let change = (close - open) / open * 100.0;

    println!("  Open: ${:.2}, Close: ${:.2}", open, close);
    println!("  High: ${:.2}, Low: ${:.2}", high, low);
    println!("  Change: {:+.2}%", change);
}

fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    let high = prices.iter().cloned().fold(f64::MIN, f64::max);
    let low = prices.iter().cloned().fold(f64::MAX, f64::min);
    let avg = prices.iter().sum::<f64>() / prices.len() as f64;

    (high - low) / avg * 100.0
}
```

## Практический пример: Скользящее окно для индикаторов

```rust
fn main() {
    let closes = [
        42000.0, 42100.0, 42050.0, 42200.0, 42300.0,
        42250.0, 42400.0, 42350.0, 42500.0, 42450.0,
        42600.0, 42550.0, 42700.0, 42650.0, 42800.0,
    ];

    let period = 5;

    println!("=== SMA-{} с использованием срезов ===\n", period);

    // Скользящее окно через срезы
    for i in 0..=(closes.len() - period) {
        let window = &closes[i..i + period];
        let sma = window.iter().sum::<f64>() / period as f64;

        println!(
            "Window [{:2}..{:2}]: {:?} => SMA: {:.2}",
            i, i + period, window, sma
        );
    }

    // Последнее значение SMA
    let last_window = &closes[closes.len() - period..];
    let current_sma = last_window.iter().sum::<f64>() / period as f64;
    println!("\nТекущий SMA-{}: {:.2}", period, current_sma);
}
```

## Изменяемые срезы

Срезы могут быть изменяемыми:

```rust
fn main() {
    let mut order_book = [100.0, 200.0, 150.0, 180.0, 220.0];

    println!("Before: {:?}", order_book);

    // Изменяемый срез первых трёх элементов
    let top_orders = &mut order_book[..3];

    // Применяем корректировку цен
    adjust_prices(top_orders, 1.05);  // +5%

    println!("After adjustment: {:?}", order_book);
}

fn adjust_prices(prices: &mut [f64], multiplier: f64) {
    for price in prices.iter_mut() {
        *price *= multiplier;
    }
}
```

## Практический пример: Разделение портфеля

```rust
fn main() {
    // Портфель: [символ_id, количество, цена]
    let mut portfolio: [(u32, f64, f64); 6] = [
        (1, 10.0, 42000.0),   // BTC
        (2, 100.0, 2800.0),   // ETH
        (3, 1000.0, 0.35),    // XRP
        (4, 50.0, 320.0),     // BNB
        (5, 200.0, 28.0),     // LINK
        (6, 500.0, 0.12),     // DOGE
    ];

    // Делим на две части: высоколиквидные и остальные
    let (high_cap, alt_coins) = portfolio.split_at_mut(3);

    println!("=== Высококапитализированные ===");
    for (id, qty, price) in high_cap.iter() {
        println!("Asset {}: {} @ ${:.2} = ${:.2}", id, qty, price, qty * price);
    }

    println!("\n=== Альткоины ===");
    for (id, qty, price) in alt_coins.iter() {
        println!("Asset {}: {} @ ${:.2} = ${:.2}", id, qty, price, qty * price);
    }

    // Обновляем цены в каждой части независимо
    update_prices(high_cap, 1.02);   // +2%
    update_prices(alt_coins, 0.95);  // -5%

    println!("\n=== После обновления цен ===");
    for (id, qty, price) in portfolio.iter() {
        println!("Asset {}: ${:.2}", id, qty * price);
    }
}

fn update_prices(assets: &mut [(u32, f64, f64)], multiplier: f64) {
    for (_, _, price) in assets.iter_mut() {
        *price *= multiplier;
    }
}
```

## Методы срезов

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42100.0, 42800.0];

    // Базовые методы
    println!("Length: {}", prices.len());
    println!("Is empty: {}", prices.is_empty());

    // Первый и последний элементы
    println!("First: {:?}", prices.first());
    println!("Last: {:?}", prices.last());

    // Разделение
    let (left, right) = prices.split_at(3);
    println!("Left half: {:?}", left);
    println!("Right half: {:?}", right);

    // Получение первого/последнего + остаток
    if let Some((first, rest)) = prices.split_first() {
        println!("First: {}, Rest: {:?}", first, rest);
    }

    if let Some((last, init)) = prices.split_last() {
        println!("Last: {}, Init: {:?}", last, init);
    }

    // Чанки (разбиение на части)
    println!("\nChunks of 2:");
    for chunk in prices.chunks(2) {
        println!("  {:?}", chunk);
    }

    // Окна (скользящие)
    println!("\nWindows of 3:");
    for window in prices.windows(3) {
        println!("  {:?}", window);
    }
}
```

## Практический пример: Поиск паттернов

```rust
fn main() {
    let prices = [
        100.0, 102.0, 101.0, 103.0, 105.0,  // Рост
        104.0, 103.0, 101.0, 99.0, 97.0,    // Падение
        98.0, 100.0, 102.0, 104.0, 106.0,   // Рост
    ];

    println!("=== Поиск трендов через окна ===\n");

    // Анализируем окнами по 3 свечи
    for (i, window) in prices.windows(3).enumerate() {
        let trend = detect_trend(window);
        println!(
            "Candles [{:2}-{:2}]: {:?} => {}",
            i, i + 2, window, trend
        );
    }

    // Подсчёт трендов
    let mut uptrends = 0;
    let mut downtrends = 0;

    for window in prices.windows(3) {
        match detect_trend(window) {
            "Uptrend" => uptrends += 1,
            "Downtrend" => downtrends += 1,
            _ => {}
        }
    }

    println!("\n=== Статистика ===");
    println!("Uptrends: {}", uptrends);
    println!("Downtrends: {}", downtrends);
    println!("Trend ratio: {:.2}", uptrends as f64 / downtrends as f64);
}

fn detect_trend(window: &[f64]) -> &'static str {
    if window.len() < 2 {
        return "Unknown";
    }

    let is_uptrend = window.windows(2).all(|w| w[1] > w[0]);
    let is_downtrend = window.windows(2).all(|w| w[1] < w[0]);

    if is_uptrend {
        "Uptrend"
    } else if is_downtrend {
        "Downtrend"
    } else {
        "Sideways"
    }
}
```

## Безопасность срезов

```rust
fn main() {
    let prices = [42000.0, 42100.0, 42200.0];

    // Безопасный доступ через get()
    match prices.get(1..4) {
        Some(slice) => println!("Slice: {:?}", slice),
        None => println!("Index out of bounds!"),
    }

    // Проверка перед созданием среза
    let start = 0;
    let end = 5;

    if end <= prices.len() {
        let safe_slice = &prices[start..end];
        println!("Safe slice: {:?}", safe_slice);
    } else {
        println!("Cannot create slice: end ({}) > len ({})", end, prices.len());

        // Используем доступную часть
        let available = &prices[start..];
        println!("Available data: {:?}", available);
    }
}
```

## Упражнения

### Упражнение 1: Сравнение периодов

```rust
fn main() {
    let monthly_returns = [
        2.5, -1.2, 3.8, 0.5, -2.1, 4.2,   // Первое полугодие
        1.8, -0.5, 2.9, -1.5, 3.1, 2.0,   // Второе полугодие
    ];

    // TODO: Разделите данные на два полугодия и сравните:
    // - Среднюю доходность
    // - Максимальную просадку (минимальное значение)
    // - Количество прибыльных месяцев
}
```

### Упражнение 2: Скользящий максимум

```rust
fn main() {
    let highs = [42500.0, 42800.0, 42300.0, 43100.0, 42900.0, 43500.0, 43200.0];

    // TODO: Найдите максимум за каждое окно из 3 элементов
    // Используйте метод windows()
}
```

### Упражнение 3: Нормализация данных

```rust
fn main() {
    let mut prices = [100.0, 150.0, 120.0, 180.0, 140.0];

    // TODO: Нормализуйте цены в диапазон [0, 1]
    // Используйте изменяемый срез
    // Формула: (price - min) / (max - min)
}
```

### Упражнение 4: Разделение ордеров

```rust
fn main() {
    let orders = [
        ("BTC", 1.5, "buy"),
        ("ETH", 10.0, "sell"),
        ("BTC", 0.5, "buy"),
        ("SOL", 100.0, "sell"),
        ("ETH", 5.0, "buy"),
    ];

    // TODO: Используя chunks(), обработайте ордера пакетами по 2
    // и выведите информацию о каждом пакете
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `&[T]` | Неизменяемый срез типа T |
| `&mut [T]` | Изменяемый срез |
| `[a..b]` | Диапазон от a до b (не включая b) |
| `[a..=b]` | Диапазон от a до b (включая b) |
| `[..b]` / `[a..]` / `[..]` | Сокращённые формы |
| `split_at()` | Разделение на две части |
| `chunks()` | Разбиение на части фиксированного размера |
| `windows()` | Скользящие окна |

## Домашнее задание

1. **RSI через срезы**: Реализуйте расчёт RSI (Relative Strength Index) используя срезы для анализа изменений цен за 14 периодов.

2. **Bollinger Bands**: Создайте функцию, которая принимает срез цен и период, и возвращает верхнюю и нижнюю полосы Боллинджера.

3. **Анализ объёмов**: Дан массив объёмов торгов за месяц. Разделите на недели и найдите неделю с максимальным средним объёмом.

4. **Детектор паттернов**: Напишите функцию, которая ищет паттерн "три белых солдата" (три последовательные бычьи свечи) в массиве цен закрытия.

## Навигация

[← День 48](../048-ownership-transfer/ru.md) | [День 50 →](../050-string-slices/ru.md)
