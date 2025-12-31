# День 81: Итерация по Vec — перебираем все ордера

## Аналогия из трейдинга

Каждый день трейдер работает с множеством ордеров:
- **Просмотр всех открытых позиций** — для оценки текущего состояния портфеля
- **Анализ истории сделок** — для расчёта P&L
- **Проверка всех стоп-лоссов** — для управления рисками
- **Расчёт общей комиссии** — по всем сделкам

Итерация по вектору — это как обход всех ордеров в книге заявок, один за другим.

## Базовые способы итерации

### Простой цикл for

```rust
fn main() {
    let orders = vec!["BUY BTC", "SELL ETH", "BUY SOL", "SELL BTC"];

    println!("=== Все ордера ===");
    for order in &orders {
        println!("Ордер: {}", order);
    }

    // orders всё ещё доступен благодаря &
    println!("\nВсего ордеров: {}", orders.len());
}
```

### Итерация с владением (consuming)

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 42200.0];

    // into_iter() забирает владение
    for price in prices.into_iter() {
        println!("Цена: ${}", price);
    }

    // prices больше недоступен!
    // println!("{:?}", prices);  // Ошибка компиляции!
}
```

### Итерация с индексом (enumerate)

```rust
fn main() {
    let trades = vec![
        ("BTC", 100.0, 42000.0),
        ("ETH", 50.0, 2200.0),
        ("SOL", 200.0, 25.0),
    ];

    println!("=== История сделок ===");
    for (index, (symbol, amount, price)) in trades.iter().enumerate() {
        let value = amount * price;
        println!("#{}: {} {} шт. по ${} = ${:.2}",
                 index + 1, symbol, amount, price, value);
    }
}
```

## Методы итераторов для трейдинга

### Фильтрация ордеров

```rust
fn main() {
    let orders = vec![
        ("BTC", "BUY", 42000.0),
        ("ETH", "SELL", 2200.0),
        ("BTC", "SELL", 42500.0),
        ("SOL", "BUY", 25.0),
        ("BTC", "BUY", 41800.0),
    ];

    // Только BTC ордера
    println!("=== Ордера BTC ===");
    for order in orders.iter().filter(|(symbol, _, _)| *symbol == "BTC") {
        println!("{:?}", order);
    }

    // Только покупки
    println!("\n=== Все покупки ===");
    let buys: Vec<_> = orders.iter()
        .filter(|(_, side, _)| *side == "BUY")
        .collect();

    for buy in &buys {
        println!("{:?}", buy);
    }
}
```

### Расчёт суммы (sum)

```rust
fn main() {
    let trade_profits = vec![150.0, -50.0, 200.0, -30.0, 180.0, -20.0];

    // Общий P&L
    let total_pnl: f64 = trade_profits.iter().sum();
    println!("Общий P&L: ${:.2}", total_pnl);

    // Сумма прибыльных сделок
    let total_profit: f64 = trade_profits.iter()
        .filter(|&&p| p > 0.0)
        .sum();
    println!("Общая прибыль: ${:.2}", total_profit);

    // Сумма убыточных сделок
    let total_loss: f64 = trade_profits.iter()
        .filter(|&&p| p < 0.0)
        .sum();
    println!("Общий убыток: ${:.2}", total_loss);

    // Профит-фактор
    if total_loss != 0.0 {
        let profit_factor = total_profit / total_loss.abs();
        println!("Профит-фактор: {:.2}", profit_factor);
    }
}
```

### Преобразование (map)

```rust
fn main() {
    let prices_usd = vec![42000.0, 2200.0, 25.0, 0.35];
    let usd_to_eur = 0.92;

    // Конвертация в EUR
    let prices_eur: Vec<f64> = prices_usd.iter()
        .map(|price| price * usd_to_eur)
        .collect();

    println!("USD цены: {:?}", prices_usd);
    println!("EUR цены: {:?}", prices_eur);

    // Расчёт изменений в процентах
    let closes = vec![42000.0, 42500.0, 42200.0, 42800.0, 43000.0];

    let returns: Vec<f64> = closes.windows(2)
        .map(|w| (w[1] - w[0]) / w[0] * 100.0)
        .collect();

    println!("\nДневные доходности:");
    for (i, ret) in returns.iter().enumerate() {
        let sign = if *ret >= 0.0 { "+" } else { "" };
        println!("  День {}: {}{}%", i + 1, sign, ret);
    }
}
```

### Поиск экстремумов

```rust
fn main() {
    let daily_prices = vec![
        ("2024-01-01", 42000.0),
        ("2024-01-02", 42500.0),
        ("2024-01-03", 41800.0),
        ("2024-01-04", 43200.0),
        ("2024-01-05", 42900.0),
    ];

    // Максимальная цена
    if let Some((date, price)) = daily_prices.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    {
        println!("Максимум: {} - ${}", date, price);
    }

    // Минимальная цена
    if let Some((date, price)) = daily_prices.iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    {
        println!("Минимум: {} - ${}", date, price);
    }

    // Диапазон цен
    let prices: Vec<f64> = daily_prices.iter().map(|(_, p)| *p).collect();
    let max = prices.iter().cloned().fold(f64::MIN, f64::max);
    let min = prices.iter().cloned().fold(f64::MAX, f64::min);
    println!("Диапазон: ${:.2}", max - min);
}
```

## Изменяемая итерация

```rust
fn main() {
    let mut portfolio = vec![
        ("BTC", 1.5, 42000.0),   // (символ, количество, цена входа)
        ("ETH", 10.0, 2200.0),
        ("SOL", 100.0, 25.0),
    ];

    println!("=== До обновления цен ===");
    for (symbol, amount, price) in &portfolio {
        println!("{}: {} шт. по ${}", symbol, amount, price);
    }

    // Обновляем цены (симуляция рыночного движения)
    let price_changes = [1.05, 0.98, 1.12];  // +5%, -2%, +12%

    for (i, (_, _, price)) in portfolio.iter_mut().enumerate() {
        *price *= price_changes[i];
    }

    println!("\n=== После обновления цен ===");
    for (symbol, amount, price) in &portfolio {
        println!("{}: {} шт. по ${:.2}", symbol, amount, price);
    }
}
```

## Практический пример: анализ ордербука

```rust
#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    side: String,      // "BUY" или "SELL"
    price: f64,
    amount: f64,
    status: String,    // "OPEN", "FILLED", "CANCELLED"
}

fn main() {
    let orders = vec![
        Order { id: 1, symbol: "BTC".into(), side: "BUY".into(),
                price: 42000.0, amount: 0.5, status: "FILLED".into() },
        Order { id: 2, symbol: "ETH".into(), side: "SELL".into(),
                price: 2200.0, amount: 5.0, status: "OPEN".into() },
        Order { id: 3, symbol: "BTC".into(), side: "BUY".into(),
                price: 41500.0, amount: 0.3, status: "OPEN".into() },
        Order { id: 4, symbol: "BTC".into(), side: "SELL".into(),
                price: 43000.0, amount: 0.2, status: "CANCELLED".into() },
        Order { id: 5, symbol: "SOL".into(), side: "BUY".into(),
                price: 25.0, amount: 100.0, status: "FILLED".into() },
    ];

    // Статистика по статусам
    let filled_count = orders.iter().filter(|o| o.status == "FILLED").count();
    let open_count = orders.iter().filter(|o| o.status == "OPEN").count();
    let cancelled_count = orders.iter().filter(|o| o.status == "CANCELLED").count();

    println!("=== Статистика ордеров ===");
    println!("Исполнено: {}", filled_count);
    println!("Открыто: {}", open_count);
    println!("Отменено: {}", cancelled_count);

    // Общий объём исполненных ордеров по символам
    println!("\n=== Объёмы по символам ===");
    for symbol in ["BTC", "ETH", "SOL"] {
        let volume: f64 = orders.iter()
            .filter(|o| o.symbol == symbol && o.status == "FILLED")
            .map(|o| o.price * o.amount)
            .sum();
        if volume > 0.0 {
            println!("{}: ${:.2}", symbol, volume);
        }
    }

    // Открытые ордера на покупку
    println!("\n=== Открытые BUY ордера ===");
    for order in orders.iter().filter(|o| o.side == "BUY" && o.status == "OPEN") {
        println!("#{}: {} {} @ ${}", order.id, order.symbol, order.amount, order.price);
    }
}
```

## Практический пример: расчёт скользящей средней

```rust
fn main() {
    let prices = vec![
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
    ];

    let period = 5;

    // Расчёт SMA
    let sma: Vec<f64> = prices.windows(period)
        .map(|window| window.iter().sum::<f64>() / period as f64)
        .collect();

    println!("=== SMA-{} ===", period);
    for (i, value) in sma.iter().enumerate() {
        let price = prices[i + period - 1];
        let signal = if price > *value { "выше SMA" } else { "ниже SMA" };
        println!("Период {}: SMA={:.2}, Цена={:.2} ({})",
                 i + period, value, price, signal);
    }
}
```

## Практический пример: поиск торговых сигналов

```rust
fn main() {
    let candles = vec![
        (42000.0, 42500.0, 41800.0, 42300.0),  // (open, high, low, close)
        (42300.0, 42600.0, 42200.0, 42100.0),
        (42100.0, 42400.0, 42000.0, 42350.0),
        (42350.0, 43000.0, 42300.0, 42900.0),
        (42900.0, 43200.0, 42800.0, 43100.0),
    ];

    println!("=== Анализ свечей ===");

    for (i, (open, high, low, close)) in candles.iter().enumerate() {
        let body = (close - open).abs();
        let upper_shadow = high - f64::max(*open, *close);
        let lower_shadow = f64::min(*open, *close) - low;
        let range = high - low;

        let candle_type = if close > open { "бычья" } else { "медвежья" };

        // Определяем паттерны
        let pattern = if body < range * 0.1 {
            "Доджи"
        } else if lower_shadow > body * 2.0 && upper_shadow < body * 0.5 {
            "Молот"
        } else if upper_shadow > body * 2.0 && lower_shadow < body * 0.5 {
            "Падающая звезда"
        } else {
            "Обычная свеча"
        };

        println!("Свеча {}: {} {} (тело={:.0}, диапазон={:.0})",
                 i + 1, candle_type, pattern, body, range);
    }

    // Поиск разворотных паттернов
    println!("\n=== Сигналы ===");
    for i in 1..candles.len() {
        let prev = candles[i - 1];
        let curr = candles[i];

        // Бычье поглощение
        if prev.3 < prev.0 && curr.3 > curr.0 &&
           curr.0 < prev.3 && curr.3 > prev.0 {
            println!("Свеча {}: Бычье поглощение! Сигнал на покупку", i + 1);
        }

        // Медвежье поглощение
        if prev.3 > prev.0 && curr.3 < curr.0 &&
           curr.0 > prev.3 && curr.3 < prev.0 {
            println!("Свеча {}: Медвежье поглощение! Сигнал на продажу", i + 1);
        }
    }
}
```

## Цепочки итераторов

```rust
fn main() {
    let trades = vec![
        ("BTC", 100.0),
        ("ETH", -50.0),
        ("BTC", 200.0),
        ("SOL", -30.0),
        ("BTC", 150.0),
        ("ETH", 80.0),
    ];

    // Сложная цепочка: фильтрация + преобразование + агрегация
    let btc_profit: f64 = trades.iter()
        .filter(|(symbol, _)| *symbol == "BTC")  // Только BTC
        .map(|(_, profit)| profit)                // Извлекаем прибыль
        .filter(|&&p| p > 0.0)                    // Только положительные
        .sum();                                    // Суммируем

    println!("Прибыль по BTC: ${:.2}", btc_profit);

    // Топ-3 прибыльные сделки
    let mut sorted_trades = trades.clone();
    sorted_trades.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\nТоп-3 сделки:");
    for (symbol, profit) in sorted_trades.iter().take(3) {
        println!("  {}: ${:.2}", symbol, profit);
    }

    // Количество прибыльных сделок по каждому активу
    println!("\nПрибыльные сделки по активам:");
    for symbol in ["BTC", "ETH", "SOL"] {
        let count = trades.iter()
            .filter(|(s, p)| *s == symbol && *p > 0.0)
            .count();
        println!("  {}: {}", symbol, count);
    }
}
```

## Что мы узнали

| Метод | Описание |
|-------|----------|
| `iter()` | Итератор по ссылкам |
| `iter_mut()` | Итератор для изменения |
| `into_iter()` | Итератор с владением |
| `enumerate()` | Добавляет индексы |
| `filter()` | Фильтрация элементов |
| `map()` | Преобразование |
| `sum()` | Сумма элементов |
| `collect()` | Сборка в коллекцию |
| `windows(n)` | Скользящее окно |
| `take(n)` | Первые n элементов |

## Домашнее задание

1. **Анализ портфеля**: Создайте вектор позиций с полями (символ, количество, цена входа, текущая цена). Рассчитайте:
   - Общий P&L портфеля
   - Прибыльные и убыточные позиции отдельно
   - Процент прибыльных позиций

2. **История сделок**: Напишите функцию, которая принимает вектор сделок и возвращает:
   - Лучшую сделку
   - Худшую сделку
   - Среднюю прибыль
   - Профит-фактор

3. **Скользящие средние**: Реализуйте расчёт EMA (экспоненциальная скользящая средняя) через итерацию

4. **Детектор паттернов**: Напишите функцию поиска паттерна "три белых солдата" (три подряд бычьих свечи с повышающимися закрытиями)

## Навигация

[← Предыдущий день](../080-vec-modification/ru.md) | [Следующий день →](../082-vec-slices-trading/ru.md)
