# День 6: Типы данных — цена, количество, название тикера

## Аналогия из трейдинга

На бирже мы работаем с разными видами данных:
- **Цены** — числа с дробной частью (42156.78 USDT)
- **Количество** — тоже дробные (0.00123 BTC)
- **Объёмы торгов** — огромные целые числа (1,000,000,000)
- **Тикеры** — текст ("BTC", "ETH")
- **Статусы** — да/нет (рынок открыт?)

В Rust для каждого вида данных есть свой **тип**.

## Скалярные типы

Скалярные типы — это одиночные значения.

### 1. Целые числа (Integers)

| Тип | Размер | Диапазон | Пример в трейдинге |
|-----|--------|----------|-------------------|
| `i8` | 8 бит | -128 до 127 | — |
| `i16` | 16 бит | -32,768 до 32,767 | — |
| `i32` | 32 бит | ±2 миллиарда | Количество сделок за день |
| `i64` | 64 бит | ±9 квинтиллионов | Timestamp в миллисекундах |
| `u8` | 8 бит | 0 до 255 | Процент (0-100) |
| `u16` | 16 бит | 0 до 65,535 | Порт сервера |
| `u32` | 32 бит | 0 до 4 миллиардов | Order ID |
| `u64` | 64 бит | 0 до 18 квинтиллионов | Объём торгов в сатоши |

```rust
fn main() {
    let trade_count: i32 = 150;           // Сделок за день
    let order_id: u64 = 123456789012;     // ID ордера
    let timestamp: i64 = 1703865600000;   // Unix время в мс
    let risk_percent: u8 = 2;             // Риск 2%

    println!("Сделок: {}", trade_count);
    println!("Order ID: {}", order_id);
}
```

**Аналогия:**
- `i` = signed (со знаком) — может быть отрицательным (убыток)
- `u` = unsigned (без знака) — только положительные (ID, объём)

### 2. Числа с плавающей точкой (Floats)

| Тип | Размер | Точность | Пример |
|-----|--------|----------|--------|
| `f32` | 32 бит | ~7 цифр | Быстрые расчёты |
| `f64` | 64 бит | ~15 цифр | Точные цены |

```rust
fn main() {
    let btc_price: f64 = 42156.78901234;  // Точная цена
    let quick_calc: f32 = 42156.78;       // Менее точная

    println!("BTC: {}", btc_price);
    println!("Быстро: {}", quick_calc);
}
```

**Важно для трейдинга:** Всегда используй `f64` для денег! `f32` может терять точность.

```rust
fn main() {
    // Проблема f32
    let balance_f32: f32 = 1000000.0 + 0.01;
    println!("f32: {}", balance_f32);  // Может быть 1000000.0 !

    // Решение f64
    let balance_f64: f64 = 1000000.0 + 0.01;
    println!("f64: {}", balance_f64);  // Точно 1000000.01
}
```

### 3. Булевы значения (Boolean)

```rust
fn main() {
    let market_open: bool = true;
    let position_active: bool = false;
    let is_profitable: bool = true;

    println!("Рынок открыт: {}", market_open);
    println!("Есть позиция: {}", position_active);
}
```

**Аналогия:** Лампочки на панели — горит/не горит.

### 4. Символы (Characters)

```rust
fn main() {
    let direction: char = '↑';  // Направление тренда
    let status: char = '✓';     // Статус сделки
    let currency: char = '₿';   // Символ биткоина

    println!("Тренд: {}", direction);
    println!("Статус: {}", status);
}
```

## Составные типы

### 1. Кортежи (Tuples)

Группируют разные типы вместе:

```rust
fn main() {
    // (тикер, цена, объём)
    let trade: (&str, f64, f64) = ("BTC/USDT", 42000.0, 0.5);

    // Доступ по индексу
    println!("Тикер: {}", trade.0);
    println!("Цена: {}", trade.1);
    println!("Объём: {}", trade.2);

    // Деструктуризация
    let (symbol, price, volume) = trade;
    println!("{}: {} x {}", symbol, price, volume);
}
```

**Аналогия:** Строка в таблице сделок — несколько полей вместе.

### 2. Массивы (Arrays)

Фиксированный список одинаковых типов:

```rust
fn main() {
    // Последние 5 цен закрытия
    let closes: [f64; 5] = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    println!("Первая цена: {}", closes[0]);
    println!("Последняя цена: {}", closes[4]);

    // Массив одинаковых значений
    let zeros: [f64; 10] = [0.0; 10];  // 10 нулей

    println!("Размер: {}", closes.len());
}
```

**Аналогия:** Свечи на графике за последние N периодов.

## Строки

В Rust есть два типа строк:

### `&str` — строковый срез (заимствованный текст)

```rust
fn main() {
    let ticker: &str = "BTC/USDT";
    let exchange: &str = "Binance";

    println!("Торгуем {} на {}", ticker, exchange);
}
```

### `String` — владеющая строка (можно менять)

```rust
fn main() {
    let mut message = String::from("Цена: ");
    message.push_str("42000");
    message.push_str(" USDT");

    println!("{}", message);
}
```

**Аналогия:**
- `&str` — бумажка с тикером на мониторе (только читаем)
- `String` — блокнот, где записываем заметки (можно менять)

## Практический пример: структура сделки

```rust
fn main() {
    // Данные сделки
    let symbol: &str = "ETH/USDT";
    let side: char = 'B';           // B = Buy, S = Sell
    let entry_price: f64 = 2250.50;
    let quantity: f64 = 1.5;
    let stop_loss: f64 = 2200.0;
    let take_profit: f64 = 2350.0;
    let is_active: bool = true;
    let order_id: u64 = 9876543210;

    // Расчёты
    let position_value: f64 = entry_price * quantity;
    let potential_loss: f64 = (entry_price - stop_loss) * quantity;
    let potential_profit: f64 = (take_profit - entry_price) * quantity;

    // Вывод
    println!("╔══════════════════════════════════╗");
    println!("║         TRADE INFO               ║");
    println!("╠══════════════════════════════════╣");
    println!("║ Symbol: {:>20}    ║", symbol);
    println!("║ Side: {:>22}    ║", if side == 'B' { "BUY" } else { "SELL" });
    println!("║ Order ID: {:>18}    ║", order_id);
    println!("║ Entry: {:>21.2}    ║", entry_price);
    println!("║ Quantity: {:>18.4}    ║", quantity);
    println!("║ Value: {:>21.2}    ║", position_value);
    println!("║ Stop Loss: {:>17.2}    ║", stop_loss);
    println!("║ Take Profit: {:>15.2}    ║", take_profit);
    println!("║ Pot. Loss: {:>17.2}    ║", potential_loss);
    println!("║ Pot. Profit: {:>15.2}    ║", potential_profit);
    println!("║ Active: {:>20}    ║", is_active);
    println!("╚══════════════════════════════════╝");
}
```

## Преобразование типов

```rust
fn main() {
    let price_int: i32 = 42000;
    let price_float: f64 = price_int as f64;  // i32 -> f64

    let big_number: u64 = 1000000;
    let small_number: u32 = big_number as u32;  // Осторожно!

    println!("Float: {}", price_float);
    println!("Small: {}", small_number);
}
```

## Что мы узнали

| Тип | Использование в трейдинге |
|-----|--------------------------|
| `i32`, `i64` | Счётчики, timestamp |
| `u32`, `u64` | ID, объёмы |
| `f64` | Цены, количества |
| `bool` | Флаги состояния |
| `&str` | Тикеры, названия |
| `(T, T, T)` | Группировка данных |
| `[T; N]` | Исторические данные |

## Домашнее задание

1. Создай переменные для описания ордера:
   - ID ордера (u64)
   - Тикер (&str)
   - Сторона (char: 'B' или 'S')
   - Цена (f64)
   - Количество (f64)
   - Исполнен ли (bool)

2. Создай массив из 7 цен закрытия за неделю

3. Создай кортеж с bid и ask ценой, рассчитай спред

4. Выведи всю информацию красиво отформатированной

## Навигация

[← Предыдущий день](../005-immutability-locked-price/ru.md) | [Следующий день →](../007-integers-counting-shares/ru.md)
