# День 11: Кортежи — цена покупки и продажи вместе

## Аналогия из трейдинга

В стакане заявок всегда есть две цены:
- **Bid** (покупка) — лучшая цена, по которой кто-то готов купить
- **Ask** (продажа) — лучшая цена, по которой кто-то готов продать

Эти две цены всегда идут **вместе** — они связаны. В Rust для группировки связанных значений используются **кортежи** (tuples).

## Что такое кортеж?

Кортеж — это фиксированная группа значений, возможно разных типов:

```rust
fn main() {
    // (bid, ask) - цены в стакане
    let spread: (f64, f64) = (42000.0, 42010.0);

    println!("Bid: {}, Ask: {}", spread.0, spread.1);
}
```

## Создание кортежей

```rust
fn main() {
    // Разные типы данных в одном кортеже
    let trade: (&str, f64, f64, bool) = ("BTC/USDT", 42000.0, 0.5, true);

    println!("Symbol: {}", trade.0);
    println!("Price: {}", trade.1);
    println!("Quantity: {}", trade.2);
    println!("Is Long: {}", trade.3);
}
```

Кортеж может содержать до 12 элементов (больше — не рекомендуется).

## Доступ к элементам

### По индексу

```rust
fn main() {
    let candle = (42000.0, 42500.0, 41800.0, 42200.0); // O, H, L, C

    println!("Open: {}", candle.0);
    println!("High: {}", candle.1);
    println!("Low: {}", candle.2);
    println!("Close: {}", candle.3);
}
```

### Деструктуризация

```rust
fn main() {
    let candle = (42000.0, 42500.0, 41800.0, 42200.0);

    // Разбираем кортеж на переменные
    let (open, high, low, close) = candle;

    println!("Open: {}", open);
    println!("High: {}", high);
    println!("Low: {}", low);
    println!("Close: {}", close);

    // Расчёт
    let body = (close - open).abs();
    let range = high - low;

    println!("Body: {}", body);
    println!("Range: {}", range);
}
```

### Частичная деструктуризация

```rust
fn main() {
    let trade = ("BTC/USDT", 42000.0, 0.5, true, 123456);

    // Нам нужны только символ и цена
    let (symbol, price, ..) = trade;

    println!("Symbol: {}, Price: {}", symbol, price);

    // Или первый и последний
    let (sym, .., order_id) = trade;
    println!("Symbol: {}, Order ID: {}", sym, order_id);
}
```

## Изменяемые кортежи

```rust
fn main() {
    let mut position = ("BTC/USDT", 0.0, false); // (symbol, pnl, is_open)

    println!("Before: {:?}", position);

    // Открываем позицию
    position.2 = true;
    println!("Opened: {:?}", position);

    // Обновляем PnL
    position.1 = 150.50;
    println!("Updated PnL: {:?}", position);
}
```

## Вложенные кортежи

```rust
fn main() {
    // ((bid, ask), (bid_size, ask_size))
    let order_book_top: ((f64, f64), (f64, f64)) = (
        (42000.0, 42010.0),   // Цены
        (1.5, 2.3)            // Объёмы
    );

    let ((bid, ask), (bid_size, ask_size)) = order_book_top;

    println!("Bid: {} x {}", bid, bid_size);
    println!("Ask: {} x {}", ask, ask_size);

    let spread = ask - bid;
    println!("Spread: {}", spread);
}
```

## Кортежи как возвращаемые значения

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42050.0];

    let (min, max) = find_min_max(&prices);
    println!("Min: {}, Max: {}", min, max);
    println!("Range: {}", max - min);
}

fn find_min_max(prices: &[f64]) -> (f64, f64) {
    let mut min = f64::MAX;
    let mut max = f64::MIN;

    for &price in prices {
        if price < min {
            min = price;
        }
        if price > max {
            max = price;
        }
    }

    (min, max)  // Возвращаем кортеж
}
```

## Практический пример: спред и его анализ

```rust
fn main() {
    // Данные стакана
    let bid = 42000.0;
    let ask = 42015.0;
    let bid_size = 2.5;
    let ask_size = 1.8;

    // Группируем связанные данные
    let best_bid: (f64, f64) = (bid, bid_size);
    let best_ask: (f64, f64) = (ask, ask_size);

    // Расчёты
    let spread = best_ask.0 - best_bid.0;
    let spread_percent = (spread / best_bid.0) * 100.0;
    let mid_price = (best_bid.0 + best_ask.0) / 2.0;

    println!("╔════════════════════════════════╗");
    println!("║      ORDER BOOK TOP            ║");
    println!("╠════════════════════════════════╣");
    println!("║ Best Bid: ${:.2} x {:.4}    ║", best_bid.0, best_bid.1);
    println!("║ Best Ask: ${:.2} x {:.4}    ║", best_ask.0, best_ask.1);
    println!("╠════════════════════════════════╣");
    println!("║ Spread:    ${:.2}            ║", spread);
    println!("║ Spread %:   {:.4}%           ║", spread_percent);
    println!("║ Mid Price: ${:.2}           ║", mid_price);
    println!("╚════════════════════════════════╝");
}
```

## Практический пример: OHLCV свеча

```rust
fn main() {
    // OHLCV: Open, High, Low, Close, Volume
    let candle: (f64, f64, f64, f64, f64) = (42000.0, 42500.0, 41800.0, 42200.0, 150.5);

    let (open, high, low, close, volume) = candle;

    // Анализ свечи
    let is_bullish = close > open;
    let body_size = (close - open).abs();
    let upper_shadow = high - close.max(open);
    let lower_shadow = close.min(open) - low;
    let total_range = high - low;

    println!("=== Candle Analysis ===");
    println!("Open: {}, Close: {}", open, close);
    println!("High: {}, Low: {}", high, low);
    println!("Volume: {}", volume);
    println!();
    println!("Type: {}", if is_bullish { "Bullish" } else { "Bearish" });
    println!("Body: {:.2}", body_size);
    println!("Upper Shadow: {:.2}", upper_shadow);
    println!("Lower Shadow: {:.2}", lower_shadow);
    println!("Range: {:.2}", total_range);
    println!("Body/Range: {:.1}%", (body_size / total_range) * 100.0);
}
```

## Практический пример: результат сделки

```rust
fn main() {
    // Симулируем несколько сделок
    let trades: [(f64, f64, f64); 5] = [
        (42000.0, 42500.0, 0.5),   // entry, exit, size
        (42500.0, 42200.0, 0.3),
        (42200.0, 42800.0, 0.4),
        (42800.0, 42600.0, 0.2),
        (42600.0, 43000.0, 0.6),
    ];

    let mut total_pnl = 0.0;
    let mut wins = 0;
    let mut losses = 0;

    println!("=== Trade Results ===");
    println!("{:<5} {:>10} {:>10} {:>8} {:>10}", "#", "Entry", "Exit", "Size", "PnL");
    println!("{}", "-".repeat(48));

    for (i, trade) in trades.iter().enumerate() {
        let (entry, exit, size) = *trade;
        let pnl = (exit - entry) * size;

        if pnl > 0.0 {
            wins += 1;
        } else {
            losses += 1;
        }

        total_pnl += pnl;

        println!("{:<5} {:>10.2} {:>10.2} {:>8.2} {:>+10.2}",
            i + 1, entry, exit, size, pnl);
    }

    println!("{}", "-".repeat(48));
    println!("Total PnL: {:+.2}", total_pnl);
    println!("Wins: {}, Losses: {}", wins, losses);
    println!("Win Rate: {:.1}%", (wins as f64 / trades.len() as f64) * 100.0);
}
```

## Unit-кортеж

Пустой кортеж `()` называется "unit" и означает "ничего":

```rust
fn main() {
    let nothing: () = ();

    println!("This prints nothing: {:?}", nothing);
}

// Функция без возвращаемого значения возвращает ()
fn do_something() {
    println!("Doing something...");
    // Неявно возвращает ()
}
```

## Сравнение кортежей

```rust
fn main() {
    let candle1 = (42000.0, 42500.0, 41800.0, 42200.0);
    let candle2 = (42000.0, 42500.0, 41800.0, 42200.0);
    let candle3 = (42100.0, 42500.0, 41800.0, 42200.0);

    println!("candle1 == candle2: {}", candle1 == candle2);  // true
    println!("candle1 == candle3: {}", candle1 == candle3);  // false

    // Сравнение по порядку
    let a = (1, 2);
    let b = (1, 3);
    println!("(1,2) < (1,3): {}", a < b);  // true
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `(T1, T2, ...)` | Создание кортежа |
| `tuple.0` | Доступ по индексу |
| `let (a, b) = tuple` | Деструктуризация |
| `(..)` | Игнорирование части элементов |
| `()` | Пустой кортеж (unit) |

## Домашнее задание

1. Создай кортеж для хранения данных ордера: (symbol, side, price, quantity, filled)

2. Напиши функцию, которая принимает OHLC кортеж и возвращает (is_bullish, body_size, range)

3. Создай массив из 5 кортежей (bid, ask) и найди минимальный/максимальный спред

4. Реализуй функцию расчёта PnL, которая принимает (entry, exit, size, fee_percent) и возвращает (gross_pnl, fee, net_pnl)

## Навигация

[← Предыдущий день](../010-strings-tickers/ru.md) | [Следующий день →](../012-arrays-closing-prices/ru.md)
