# День 24: continue — пропускаем убыточные сделки в отчёте

## Аналогия из трейдинга

Представь, что ты готовишь отчёт о сделках за месяц. Тебе нужно вывести только прибыльные сделки, пропустив убыточные. Ты не прекращаешь работу над отчётом (как с `break`), а просто **пропускаешь** неподходящие строки и переходишь к следующей сделке.

Оператор `continue` делает именно это — пропускает текущую итерацию цикла и сразу переходит к следующей.

## Базовый синтаксис continue

```rust
fn main() {
    let pnl_list = [150.0, -50.0, 200.0, -30.0, 80.0, -10.0, 300.0];

    println!("=== Прибыльные сделки ===");

    for pnl in pnl_list {
        if pnl < 0.0 {
            continue;  // Пропускаем убыточные
        }
        println!("Прибыль: ${:.2}", pnl);
    }
}
```

**Вывод:**
```
=== Прибыльные сделки ===
Прибыль: $150.00
Прибыль: $200.00
Прибыль: $80.00
Прибыль: $300.00
```

## Разница между break и continue

```rust
fn main() {
    let prices = [100.0, 102.0, 98.0, 105.0, 95.0, 110.0];

    println!("=== С break (останавливаемся на первом падении) ===");
    for price in prices {
        if price < 100.0 {
            println!("Цена упала ниже 100, стоп!");
            break;  // Полностью выходим из цикла
        }
        println!("Цена: ${}", price);
    }

    println!("\n=== С continue (пропускаем падения) ===");
    for price in prices {
        if price < 100.0 {
            continue;  // Пропускаем эту итерацию
        }
        println!("Цена: ${}", price);
    }
}
```

**Вывод:**
```
=== С break (останавливаемся на первом падении) ===
Цена: $100
Цена: $102
Цена упала ниже 100, стоп!

=== С continue (пропускаем падения) ===
Цена: $100
Цена: $102
Цена: $105
Цена: $110
```

## Фильтрация сделок по критериям

```rust
fn main() {
    let trades = [
        ("BTC", 500.0, 0.1),    // тикер, PnL, объём
        ("ETH", -100.0, 0.5),
        ("BTC", 300.0, 0.05),   // маленький объём
        ("SOL", 200.0, 0.3),
        ("BTC", -50.0, 0.2),
        ("ETH", 400.0, 0.4),
    ];

    println!("=== Отчёт: прибыльные сделки BTC с объёмом >= 0.1 ===\n");

    let mut total_profit = 0.0;
    let mut count = 0;

    for (ticker, pnl, volume) in trades {
        // Пропускаем не-BTC
        if ticker != "BTC" {
            continue;
        }

        // Пропускаем убыточные
        if pnl <= 0.0 {
            continue;
        }

        // Пропускаем малый объём
        if volume < 0.1 {
            continue;
        }

        println!("{}: +${:.2} (объём: {})", ticker, pnl, volume);
        total_profit += pnl;
        count += 1;
    }

    println!("\nИтого: {} сделок, прибыль: ${:.2}", count, total_profit);
}
```

## Обработка ордеров с валидацией

```rust
fn main() {
    let orders = [
        (100.0, 10),    // цена, количество
        (0.0, 5),       // невалидная цена
        (50.0, 0),      // невалидное количество
        (75.0, -3),     // отрицательное количество
        (200.0, 20),    // валидный
        (150.0, 15),    // валидный
    ];

    println!("=== Обработка валидных ордеров ===\n");

    let mut processed = 0;
    let mut total_value = 0.0;

    for (i, (price, quantity)) in orders.iter().enumerate() {
        // Проверка валидности
        if *price <= 0.0 {
            println!("Ордер #{}: пропущен (невалидная цена)", i + 1);
            continue;
        }

        if *quantity <= 0 {
            println!("Ордер #{}: пропущен (невалидное количество)", i + 1);
            continue;
        }

        let value = price * (*quantity as f64);
        println!("Ордер #{}: {} шт. по ${:.2} = ${:.2}", i + 1, quantity, price, value);

        total_value += value;
        processed += 1;
    }

    println!("\nОбработано: {} ордеров на сумму ${:.2}", processed, total_value);
}
```

## continue в цикле while

```rust
fn main() {
    let mut prices = vec![100.0, 0.0, 105.0, -5.0, 110.0, 103.0];
    let mut index = 0;

    println!("=== Фильтрация аномальных цен ===\n");

    while index < prices.len() {
        let price = prices[index];

        // Пропускаем невалидные цены
        if price <= 0.0 {
            println!("Индекс {}: аномальная цена {}, пропускаем", index, price);
            index += 1;  // ВАЖНО: увеличиваем индекс перед continue!
            continue;
        }

        println!("Индекс {}: цена ${:.2} — OK", index, price);
        index += 1;
    }
}
```

**Важно:** В цикле `while` не забывай увеличивать счётчик **перед** `continue`, иначе получишь бесконечный цикл!

## continue в цикле loop

```rust
fn main() {
    let market_data = [
        Some(42000.0),
        None,           // данные недоступны
        Some(42100.0),
        None,
        Some(42050.0),
        Some(42200.0),
    ];

    let mut index = 0;
    let mut valid_prices = Vec::new();

    println!("=== Сбор валидных цен ===\n");

    loop {
        if index >= market_data.len() {
            break;
        }

        let data = market_data[index];
        index += 1;

        // Пропускаем пустые данные
        let price = match data {
            Some(p) => p,
            None => {
                println!("Данные #{}: недоступны, пропускаем", index);
                continue;
            }
        };

        println!("Данные #{}: цена ${:.2}", index, price);
        valid_prices.push(price);
    }

    println!("\nСобрано {} валидных цен: {:?}", valid_prices.len(), valid_prices);
}
```

## Метки циклов: continue с вложенными циклами

```rust
fn main() {
    let exchanges = ["Binance", "Coinbase", "Kraken"];
    let tickers = ["BTC", "INVALID", "ETH", "ERROR", "SOL"];

    println!("=== Проверка тикеров на биржах ===\n");

    'exchange: for exchange in exchanges {
        println!("Биржа: {}", exchange);

        for ticker in tickers {
            // Пропускаем невалидные тикеры
            if ticker == "INVALID" || ticker == "ERROR" {
                println!("  {} — невалидный тикер, пропуск", ticker);
                continue;  // Продолжаем с следующим тикером
            }

            // Пропускаем всю биржу если это Kraken и тикер SOL
            if exchange == "Kraken" && ticker == "SOL" {
                println!("  SOL не торгуется на Kraken, переходим к следующей бирже");
                continue 'exchange;  // Продолжаем с следующей биржей
            }

            println!("  {} — OK", ticker);
        }
        println!();
    }
}
```

## Практический пример: генерация торгового отчёта

```rust
fn main() {
    let trades = [
        ("2024-01-15", "BTC", "BUY", 42000.0, 0.5, 150.0),
        ("2024-01-15", "ETH", "BUY", 2200.0, 2.0, -50.0),   // убыток
        ("2024-01-16", "BTC", "SELL", 43000.0, 0.3, 300.0),
        ("2024-01-16", "SOL", "BUY", 95.0, 10.0, 0.0),      // безубыток
        ("2024-01-17", "ETH", "SELL", 2300.0, 1.5, 150.0),
        ("2024-01-17", "BTC", "BUY", 41500.0, 0.2, -80.0),  // убыток
        ("2024-01-18", "SOL", "SELL", 100.0, 8.0, 40.0),
    ];

    println!("╔════════════════════════════════════════════════════╗");
    println!("║         ОТЧЁТ: ПРИБЫЛЬНЫЕ СДЕЛКИ                   ║");
    println!("╠════════════════════════════════════════════════════╣");

    let mut total_profit = 0.0;
    let mut trade_count = 0;

    for (date, ticker, side, price, qty, pnl) in trades {
        // Пропускаем убыточные и нулевые сделки
        if pnl <= 0.0 {
            continue;
        }

        println!("║ {} │ {:>4} │ {:>4} │ ${:>8.2} │ {:>4.1} │ +${:>6.2} ║",
                 date, ticker, side, price, qty, pnl);

        total_profit += pnl;
        trade_count += 1;
    }

    println!("╠════════════════════════════════════════════════════╣");
    println!("║ Всего прибыльных сделок: {:>3}                       ║", trade_count);
    println!("║ Общая прибыль: ${:>10.2}                        ║", total_profit);
    println!("╚════════════════════════════════════════════════════╝");
}
```

## Анализ портфеля с фильтрацией

```rust
fn main() {
    let portfolio = [
        ("AAPL", 150.0, 10, 155.0),   // тикер, цена покупки, кол-во, текущая цена
        ("GOOGL", 140.0, 5, 135.0),   // убыток
        ("MSFT", 380.0, 8, 395.0),
        ("TSLA", 250.0, 3, 240.0),    // убыток
        ("NVDA", 450.0, 6, 520.0),
        ("META", 330.0, 4, 325.0),    // убыток
    ];

    println!("=== Позиции в плюсе ===\n");

    let mut total_unrealized = 0.0;

    for (ticker, buy_price, quantity, current_price) in portfolio {
        let pnl = (current_price - buy_price) * quantity as f64;

        // Пропускаем убыточные позиции
        if pnl <= 0.0 {
            continue;
        }

        let pnl_percent = ((current_price / buy_price) - 1.0) * 100.0;

        println!("{}: {} шт. | Покупка: ${:.2} | Сейчас: ${:.2}",
                 ticker, quantity, buy_price, current_price);
        println!("     Нереализованная прибыль: +${:.2} ({:+.2}%)\n",
                 pnl, pnl_percent);

        total_unrealized += pnl;
    }

    println!("Общая нереализованная прибыль: ${:.2}", total_unrealized);
}
```

## Подсчёт статистики с пропуском выбросов

```rust
fn main() {
    let daily_returns = [
        0.5, 1.2, -0.3, 0.8, 15.0,   // 15% — выброс
        -0.5, 0.2, -20.0, 0.7, 1.1,  // -20% — выброс
        0.3, -0.8, 0.9, 0.4, -0.2,
    ];

    let outlier_threshold = 10.0;  // Порог выброса: ±10%

    println!("=== Расчёт средней доходности (без выбросов) ===\n");

    let mut sum = 0.0;
    let mut count = 0;
    let mut outliers = 0;

    for ret in daily_returns {
        // Пропускаем выбросы
        if ret.abs() > outlier_threshold {
            println!("Выброс: {:+.1}% — пропускаем", ret);
            outliers += 1;
            continue;
        }

        sum += ret;
        count += 1;
    }

    let average = if count > 0 { sum / count as f64 } else { 0.0 };

    println!("\nВсего значений: {}", daily_returns.len());
    println!("Пропущено выбросов: {}", outliers);
    println!("Учтено значений: {}", count);
    println!("Средняя доходность: {:+.2}%", average);
}
```

## Паттерн: ранний continue для чистого кода

```rust
fn main() {
    let candles = [
        (100.0, 105.0, 98.0, 103.0, 1000),  // open, high, low, close, volume
        (103.0, 103.0, 103.0, 103.0, 0),    // нулевой объём — пропуск
        (103.0, 108.0, 102.0, 107.0, 1500),
        (107.0, 107.0, 100.0, 101.0, 800),
        (101.0, 101.0, 101.0, 101.0, 50),   // doji с малым объёмом — пропуск
        (101.0, 110.0, 100.0, 109.0, 2000),
    ];

    let min_volume = 100;

    println!("=== Анализ значимых свечей ===\n");

    for (i, (open, high, low, close, volume)) in candles.iter().enumerate() {
        // Ранние проверки с continue
        if *volume < min_volume {
            continue;  // Пропускаем свечи с малым объёмом
        }

        if open == close && high == low {
            continue;  // Пропускаем "пустые" свечи
        }

        // Основная логика для валидных свечей
        let body = (close - open).abs();
        let range = high - low;
        let body_ratio = if range > 0.0 { body / range * 100.0 } else { 0.0 };

        let candle_type = if close > open { "бычья" } else { "медвежья" };

        println!("Свеча #{}: {} | Диапазон: {:.2} | Тело: {:.1}%",
                 i + 1, candle_type, range, body_ratio);
    }
}
```

## Что мы узнали

| Концепция | Описание | Применение в трейдинге |
|-----------|----------|------------------------|
| `continue` | Пропуск текущей итерации | Пропуск убыточных сделок |
| `break` vs `continue` | Выход vs пропуск | Стоп-лосс vs фильтрация |
| `continue` в `while` | Не забыть инкремент! | Обработка потока данных |
| Метки циклов | `continue 'label` | Пропуск целой биржи |
| Ранний continue | Чистый код с проверками | Валидация данных |

## Домашнее задание

1. Напиши программу, которая проходит по списку сделок и выводит только те, где прибыль превышает $100 и объём больше 0.5

2. Создай цикл обработки рыночных данных, который пропускает:
   - Цены равные нулю
   - Отрицательные цены
   - Цены с аномальным спредом (> 5%)

3. Реализуй анализ портфеля с использованием `continue`:
   - Пропускай позиции с нулевым количеством
   - Пропускай позиции где убыток меньше 1%
   - Выводи только позиции требующие внимания (убыток > 5%)

4. Используя метки циклов, напиши программу которая:
   - Проходит по нескольким биржам
   - На каждой бирже проверяет несколько тикеров
   - Пропускает всю биржу если обнаружен невалидный API-ответ

## Навигация

[← Предыдущий день](../023-break-take-profit/ru.md) | [Следующий день →](../025-match-order-type/ru.md)
