# День 22: Range — Цены от 100 до 200

## Аналогия из трейдинга

В трейдинге мы постоянно работаем с **диапазонами**:
- Цена в диапазоне от 100 до 200
- Последние 10 свечей
- Индексы от 0 до размера портфеля
- Уровни поддержки и сопротивления

Range в Rust — это способ выразить **последовательность значений** между начальной и конечной точкой.

## Типы Range

```rust
fn main() {
    // Range (не включает конец): a..b
    let price_range = 100..200;
    println!("Range 100..200");

    // RangeInclusive (включает конец): a..=b
    let inclusive_range = 100..=200;
    println!("Inclusive range 100..=200");

    // RangeFrom (от a до бесконечности): a..
    let from_100 = 100..;
    println!("From 100..");

    // RangeTo (от начала до b, не включая): ..b
    let to_100 = ..100;
    println!("To ..100");

    // RangeToInclusive (от начала до b, включая): ..=b
    let to_100_incl = ..=100;
    println!("To inclusive ..=100");

    // RangeFull (весь диапазон): ..
    let full = ..;
    println!("Full range ..");
}
```

## Range в циклах for

```rust
fn main() {
    println!("=== Анализ 10 свечей ===");

    // Индексы 0..10 (0,1,2,...,9)
    for i in 0..10 {
        println!("Candle {}: analyzing...", i);
    }

    println!("\n=== Цены от 100 до 105 включительно ===");

    // 100..=105 (100,101,102,103,104,105)
    for price in 100..=105 {
        println!("Price level: ${}", price);
    }
}
```

## Range для доступа к элементам массива

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
                  42300.0, 42250.0, 42400.0, 42350.0, 42500.0];

    // Первые 5 свечей (индексы 0,1,2,3,4)
    let first_five = &closes[0..5];
    println!("First 5: {:?}", first_five);

    // Последние 3 свечи (индексы 7,8,9)
    let last_three = &closes[7..10];
    println!("Last 3: {:?}", last_three);

    // От индекса 3 до конца
    let from_third = &closes[3..];
    println!("From index 3: {:?}", from_third);

    // От начала до индекса 4 (не включая)
    let to_fourth = &closes[..4];
    println!("Up to index 4: {:?}", to_fourth);

    // Весь массив
    let all = &closes[..];
    println!("All: {:?}", all);
}
```

## Практический пример: Скользящее окно для SMA

```rust
fn main() {
    let prices = [100.0, 102.0, 104.0, 103.0, 105.0,
                  107.0, 106.0, 108.0, 110.0, 109.0];

    let window_size = 3;

    println!("=== SMA-{} Calculation ===", window_size);

    // Проходим по всем возможным окнам
    for i in 0..=(prices.len() - window_size) {
        let window = &prices[i..i + window_size];
        let sma: f64 = window.iter().sum::<f64>() / window_size as f64;
        println!("Window [{}-{}]: {:?} -> SMA: {:.2}",
                 i, i + window_size - 1, window, sma);
    }
}
```

## Практический пример: Поиск уровней поддержки/сопротивления

```rust
fn main() {
    let prices = [100.0, 105.0, 103.0, 108.0, 106.0,
                  110.0, 107.0, 112.0, 109.0, 115.0];

    // Определяем ценовой диапазон
    let min_price = 100;
    let max_price = 120;
    let step = 5;

    println!("=== Price Level Analysis ===");

    // Анализируем каждый уровень
    for level in (min_price..=max_price).step_by(step) {
        let level_f = level as f64;
        let touches = prices.iter()
            .filter(|&&p| (p - level_f).abs() < 2.0)
            .count();

        if touches > 0 {
            println!("Level ${}: {} touches", level, touches);
        }
    }
}
```

## Практический пример: Торговые сессии

```rust
fn main() {
    // Часы торговых сессий (0-23)
    let asian_session = 0..9;      // 00:00 - 08:59
    let european_session = 7..16;  // 07:00 - 15:59
    let american_session = 13..22; // 13:00 - 21:59

    let current_hour = 14;

    println!("Current hour: {}:00", current_hour);

    if asian_session.contains(&current_hour) {
        println!("Asian session is active");
    }
    if european_session.contains(&current_hour) {
        println!("European session is active");
    }
    if american_session.contains(&current_hour) {
        println!("American session is active");
    }
}
```

## Методы Range

```rust
fn main() {
    let range = 1..10;

    // contains() - проверка вхождения
    println!("Range 1..10 contains 5: {}", range.contains(&5));
    println!("Range 1..10 contains 10: {}", range.contains(&10));

    let inclusive = 1..=10;
    println!("Range 1..=10 contains 10: {}", inclusive.contains(&10));

    // is_empty() - проверка на пустоту
    let empty_range = 5..5;
    println!("Range 5..5 is empty: {}", empty_range.is_empty());

    let normal_range = 5..10;
    println!("Range 5..10 is empty: {}", normal_range.is_empty());
}
```

## Практический пример: Фильтрация ордеров по цене

```rust
fn main() {
    let orders = [
        ("BUY", 100.0),
        ("SELL", 150.0),
        ("BUY", 120.0),
        ("SELL", 200.0),
        ("BUY", 180.0),
        ("SELL", 130.0),
    ];

    // Диапазон интересующих нас цен
    let price_range = 100.0..=150.0;

    println!("=== Orders in range ${:.0} - ${:.0} ===", 100.0, 150.0);

    for (i, (side, price)) in orders.iter().enumerate() {
        // Проверяем, попадает ли цена в диапазон
        if *price >= 100.0 && *price <= 150.0 {
            println!("Order {}: {} @ ${:.2}", i, side, price);
        }
    }

    // Подсчёт ордеров в диапазоне
    let count = orders.iter()
        .filter(|(_, price)| *price >= 100.0 && *price <= 150.0)
        .count();
    println!("\nTotal orders in range: {}", count);
}
```

## Практический пример: Анализ волатильности по периодам

```rust
fn main() {
    let hourly_volatility = [
        0.5, 0.4, 0.3, 0.2, 0.2, 0.3,  // 00:00 - 05:59
        0.6, 0.8, 1.2, 1.5, 1.3, 1.1,  // 06:00 - 11:59
        0.9, 1.4, 1.8, 2.0, 1.7, 1.5,  // 12:00 - 17:59
        1.2, 0.9, 0.7, 0.5, 0.4, 0.3,  // 18:00 - 23:59
    ];

    // Анализируем разные сессии
    let sessions = [
        ("Night", 0..6),
        ("Morning", 6..12),
        ("Afternoon", 12..18),
        ("Evening", 18..24),
    ];

    println!("=== Volatility by Session ===");

    for (name, range) in sessions {
        let session_vol = &hourly_volatility[range.clone()];
        let avg_vol: f64 = session_vol.iter().sum::<f64>() / session_vol.len() as f64;
        let max_vol = session_vol.iter().cloned().fold(0.0_f64, f64::max);

        println!("{:10}: Avg={:.2}%, Max={:.2}%", name, avg_vol, max_vol);
    }
}
```

## Практический пример: Разбиение данных на периоды

```rust
fn main() {
    let daily_closes: [f64; 20] = [
        100.0, 102.0, 101.0, 103.0, 105.0,  // Неделя 1
        104.0, 106.0, 108.0, 107.0, 109.0,  // Неделя 2
        110.0, 108.0, 109.0, 111.0, 113.0,  // Неделя 3
        112.0, 114.0, 115.0, 113.0, 116.0,  // Неделя 4
    ];

    println!("=== Weekly Performance ===");

    // Разбиваем на недели по 5 дней
    for week in 0..4 {
        let start = week * 5;
        let end = start + 5;
        let week_data = &daily_closes[start..end];

        let open = week_data[0];
        let close = week_data[4];
        let change = (close - open) / open * 100.0;

        let max = week_data.iter().cloned().fold(f64::MIN, f64::max);
        let min = week_data.iter().cloned().fold(f64::MAX, f64::min);

        println!("Week {}: Open={:.0}, Close={:.0}, Change={:+.2}%, Range={:.0}-{:.0}",
                 week + 1, open, close, change, min, max);
    }
}
```

## Обратный Range с rev()

```rust
fn main() {
    println!("=== Countdown to Market Open ===");

    // Обратный отсчёт
    for seconds in (1..=10).rev() {
        println!("{}...", seconds);
    }
    println!("Market is OPEN!");

    println!("\n=== Last 5 Trades (newest first) ===");

    let trades = ["Trade A", "Trade B", "Trade C", "Trade D", "Trade E"];

    for i in (0..trades.len()).rev() {
        println!("{}: {}", trades.len() - i, trades[i]);
    }
}
```

## step_by() для кастомного шага

```rust
fn main() {
    // Уровни Фибоначчи (каждые 10%)
    println!("=== Fibonacci-style Levels ===");
    for level in (0..=100).step_by(10) {
        println!("{}% retracement", level);
    }

    // Временные интервалы (каждые 15 минут)
    println!("\n=== 15-minute Candles in 1 Hour ===");
    for minute in (0..60).step_by(15) {
        println!("Candle at :{:02}", minute);
    }

    // Ценовые уровни с шагом 50
    println!("\n=== Price Levels ===");
    for price in (1000..=1500).step_by(50) {
        println!("Support/Resistance at ${}", price);
    }
}
```

## Range с разными типами

```rust
fn main() {
    // Range с i32
    let int_range: std::ops::Range<i32> = -10..10;
    println!("Integer range: {:?}", int_range);

    // Range с char
    for c in 'A'..='Z' {
        print!("{}", c);
    }
    println!(" <- Ticker symbols");

    // Важно: Range<f64> не поддерживает итерацию!
    // Для float нужно использовать другие подходы

    let start = 100.0_f64;
    let end = 110.0_f64;
    let step = 0.5_f64;

    println!("\nPrice levels (float):");
    let mut price = start;
    while price <= end {
        println!("  ${:.1}", price);
        price += step;
    }
}
```

## Что мы узнали

| Синтаксис | Тип | Описание |
|-----------|-----|----------|
| `a..b` | `Range` | От a до b (не включая b) |
| `a..=b` | `RangeInclusive` | От a до b (включая b) |
| `a..` | `RangeFrom` | От a до бесконечности |
| `..b` | `RangeTo` | От начала до b |
| `..=b` | `RangeToInclusive` | От начала до b включительно |
| `..` | `RangeFull` | Весь диапазон |

| Метод | Описание |
|-------|----------|
| `contains(&x)` | Проверяет, содержит ли диапазон x |
| `is_empty()` | Проверяет, пуст ли диапазон |
| `rev()` | Обратная итерация |
| `step_by(n)` | Итерация с шагом n |

## Домашнее задание

1. **Торговые окна**: Создай массив из 24 значений объёма (по часам). Найди 3 самых активных 4-часовых окна, используя Range для выбора данных.

2. **Анализ тренда**: Дан массив из 50 цен закрытия. Используя Range, раздели его на 5 периодов по 10 дней. Определи, какой период показал лучший рост.

3. **Сетка ордеров**: Напиши функцию, которая создаёт сетку лимитных ордеров от цены A до цены B с шагом S. Используй Range и step_by().

4. **Поиск паттерна**: Дан массив OHLC данных. Используя Range, найди все свечи, где цена закрытия была в диапазоне ±1% от цены открытия (doji-свечи).

## Навигация

[← Предыдущий день](../021-loops-market-scanner/ru.md) | [Следующий день →](../023-if-let-option/ru.md)
