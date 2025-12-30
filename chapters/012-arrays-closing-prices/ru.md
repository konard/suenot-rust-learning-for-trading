# День 12: Массивы — последние 10 цен закрытия

## Аналогия из трейдинга

Для анализа рынка нужны **исторические данные**:
- Последние 10 цен закрытия
- 20 свечей для расчёта SMA
- 14 значений для RSI

Массив — это список значений **одного типа** с **фиксированным размером**.

## Создание массивов

```rust
fn main() {
    // Последние 5 цен закрытия
    let closes: [f64; 5] = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Массив нулей (10 элементов)
    let zeros: [f64; 10] = [0.0; 10];

    // Тип выводится автоматически
    let volumes = [1500.0, 2300.0, 1800.0, 2100.0, 1950.0];

    println!("Closes: {:?}", closes);
    println!("Zeros: {:?}", zeros);
    println!("Volumes: {:?}", volumes);
}
```

Синтаксис: `[тип; размер]` или `[значение; количество]`

## Доступ к элементам

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Индексы начинаются с 0
    println!("First: {}", closes[0]);   // 42000.0
    println!("Second: {}", closes[1]);  // 42100.0
    println!("Last: {}", closes[4]);    // 42150.0

    // Длина массива
    println!("Length: {}", closes.len());  // 5

    // ОШИБКА во время выполнения!
    // println!("{}", closes[10]);  // panic: index out of bounds
}
```

## Безопасный доступ с get()

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0];

    // get() возвращает Option
    match closes.get(1) {
        Some(price) => println!("Price at index 1: {}", price),
        None => println!("Index out of bounds"),
    }

    // Безопасно для любого индекса
    match closes.get(10) {
        Some(price) => println!("Price: {}", price),
        None => println!("No price at index 10"),
    }
}
```

## Итерация по массиву

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Простой for
    println!("All prices:");
    for price in closes {
        println!("  ${}", price);
    }

    // С индексом
    println!("\nWith index:");
    for (i, price) in closes.iter().enumerate() {
        println!("  [{}] ${}", i, price);
    }

    // Только первые 3
    for price in &closes[0..3] {
        println!("First 3: ${}", price);
    }
}
```

## Изменяемые массивы

```rust
fn main() {
    let mut prices = [0.0; 5];

    println!("Before: {:?}", prices);

    // Заполняем данными
    prices[0] = 42000.0;
    prices[1] = 42100.0;
    prices[2] = 41900.0;
    prices[3] = 42200.0;
    prices[4] = 42150.0;

    println!("After: {:?}", prices);

    // Обновляем последнюю цену
    prices[4] = 42300.0;
    println!("Updated: {:?}", prices);
}
```

## Слайсы (срезы)

Слайс — это "вид" на часть массива:

```rust
fn main() {
    let prices = [100.0, 200.0, 300.0, 400.0, 500.0];

    // Слайс от индекса 1 до 4 (не включая 4)
    let slice = &prices[1..4];
    println!("Slice [1..4]: {:?}", slice);  // [200.0, 300.0, 400.0]

    // С начала до индекса 3
    let first_three = &prices[..3];
    println!("First 3: {:?}", first_three);  // [100.0, 200.0, 300.0]

    // С индекса 2 до конца
    let last_three = &prices[2..];
    println!("Last 3: {:?}", last_three);  // [300.0, 400.0, 500.0]

    // Весь массив как слайс
    let all = &prices[..];
    println!("All: {:?}", all);
}
```

## Практический пример: расчёт SMA

```rust
fn main() {
    let closes = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
                  42300.0, 42250.0, 42400.0, 42350.0, 42500.0];

    // SMA-5 (простая скользящая средняя за 5 периодов)
    let sma5 = calculate_sma(&closes[5..]);  // Последние 5
    println!("SMA-5 (last 5): {:.2}", sma5);

    // SMA для разных окон
    let sma3 = calculate_sma(&closes[7..]);
    let sma10 = calculate_sma(&closes);

    println!("SMA-3: {:.2}", sma3);
    println!("SMA-10: {:.2}", sma10);
}

fn calculate_sma(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    let sum: f64 = prices.iter().sum();
    sum / prices.len() as f64
}
```

## Практический пример: поиск min/max

```rust
fn main() {
    let daily_highs = [42500.0, 42800.0, 42300.0, 43100.0, 42900.0];
    let daily_lows = [41800.0, 42100.0, 41500.0, 42400.0, 42200.0];

    // Находим экстремумы
    let highest = find_max(&daily_highs);
    let lowest = find_min(&daily_lows);

    println!("Weekly High: ${:.2}", highest);
    println!("Weekly Low: ${:.2}", lowest);
    println!("Weekly Range: ${:.2}", highest - lowest);
}

fn find_max(prices: &[f64]) -> f64 {
    let mut max = prices[0];
    for &price in prices {
        if price > max {
            max = price;
        }
    }
    max
}

fn find_min(prices: &[f64]) -> f64 {
    let mut min = prices[0];
    for &price in prices {
        if price < min {
            min = price;
        }
    }
    min
}
```

## Практический пример: доходности

```rust
fn main() {
    let prices = [42000.0, 42500.0, 42200.0, 42800.0, 43000.0];

    // Рассчитываем дневные доходности
    let mut returns: [f64; 4] = [0.0; 4];

    for i in 1..prices.len() {
        returns[i - 1] = (prices[i] - prices[i - 1]) / prices[i - 1] * 100.0;
    }

    println!("Prices: {:?}", prices);
    println!("Daily returns (%):");
    for (i, ret) in returns.iter().enumerate() {
        let sign = if *ret >= 0.0 { "+" } else { "" };
        println!("  Day {}: {}{:.2}%", i + 1, sign, ret);
    }

    // Общая доходность
    let total_return = (prices[4] - prices[0]) / prices[0] * 100.0;
    println!("\nTotal return: {:.2}%", total_return);
}
```

## Двумерные массивы

```rust
fn main() {
    // OHLC данные за 3 дня
    // [день][значение]: O, H, L, C
    let ohlc: [[f64; 4]; 3] = [
        [42000.0, 42500.0, 41800.0, 42200.0],  // День 1
        [42200.0, 42800.0, 42100.0, 42600.0],  // День 2
        [42600.0, 43000.0, 42400.0, 42900.0],  // День 3
    ];

    println!("=== OHLC Data ===");
    for (day, candle) in ohlc.iter().enumerate() {
        println!("Day {}: O={}, H={}, L={}, C={}",
            day + 1, candle[0], candle[1], candle[2], candle[3]);
    }

    // Средняя цена закрытия
    let avg_close = (ohlc[0][3] + ohlc[1][3] + ohlc[2][3]) / 3.0;
    println!("\nAverage Close: {:.2}", avg_close);
}
```

## Полезные методы

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    // Проверки
    println!("Is empty: {}", prices.is_empty());
    println!("Length: {}", prices.len());

    // Первый и последний
    println!("First: {:?}", prices.first());
    println!("Last: {:?}", prices.last());

    // Содержит ли значение
    println!("Contains 42000: {}", prices.contains(&42000.0));

    // Итераторы
    let sum: f64 = prices.iter().sum();
    let max = prices.iter().cloned().fold(f64::MIN, f64::max);
    let min = prices.iter().cloned().fold(f64::MAX, f64::min);

    println!("Sum: {}", sum);
    println!("Max: {}", max);
    println!("Min: {}", min);
}
```

## Сортировка

```rust
fn main() {
    let mut prices = [42500.0, 41800.0, 42200.0, 42000.0, 42100.0];

    println!("Original: {:?}", prices);

    // Сортировка (для f64 нужен особый подход)
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("Sorted: {:?}", prices);

    // Обратная сортировка
    prices.sort_by(|a, b| b.partial_cmp(a).unwrap());
    println!("Reversed: {:?}", prices);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `[T; N]` | Массив из N элементов типа T |
| `arr[i]` | Доступ по индексу |
| `&arr[a..b]` | Слайс (срез) |
| `arr.len()` | Длина массива |
| `arr.iter()` | Итератор |

## Домашнее задание

1. Создай массив из 20 случайных цен и рассчитай SMA-5, SMA-10, SMA-20

2. Реализуй функцию нахождения волатильности (стандартное отклонение) для массива цен

3. Создай двумерный массив OHLCV за 5 дней и найди:
   - День с максимальным объёмом
   - День с самой большой свечой (high - low)

4. Напиши функцию, которая находит пересечение двух массивов SMA

## Навигация

[← Предыдущий день](../011-tuples-bid-ask/ru.md) | [Следующий день →](../013-functions-trade-profit/ru.md)
