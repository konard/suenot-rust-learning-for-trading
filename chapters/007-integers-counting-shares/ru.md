# День 7: Целые числа — считаем количество акций

## Аналогия из трейдинга

В трейдинге часто нужны **целые числа**:
- Количество акций (нельзя купить 1.5 акции Apple)
- ID ордера (уникальный номер)
- Номер свечи в истории
- Timestamp в миллисекундах
- Количество сделок за день

## Типы целых чисел в Rust

### Со знаком (могут быть отрицательными)

```rust
fn main() {
    let pnl_today: i32 = -500;      // Убыток $500
    let price_change: i64 = -1500;  // Цена упала на 1500 пунктов
    let temperature: i8 = -10;      // Для датацентра с серверами

    println!("PnL: {} USD", pnl_today);
}
```

| Тип | Минимум | Максимум |
|-----|---------|----------|
| `i8` | -128 | 127 |
| `i16` | -32,768 | 32,767 |
| `i32` | -2,147,483,648 | 2,147,483,647 |
| `i64` | -9.2 × 10¹⁸ | 9.2 × 10¹⁸ |
| `i128` | -1.7 × 10³⁸ | 1.7 × 10³⁸ |

### Без знака (только положительные)

```rust
fn main() {
    let order_id: u64 = 9876543210;
    let trade_count: u32 = 1500;
    let shares: u16 = 100;
    let percentage: u8 = 85;

    println!("Order #{}: {} shares", order_id, shares);
}
```

| Тип | Минимум | Максимум |
|-----|---------|----------|
| `u8` | 0 | 255 |
| `u16` | 0 | 65,535 |
| `u32` | 0 | 4,294,967,295 |
| `u64` | 0 | 18.4 × 10¹⁸ |
| `u128` | 0 | 3.4 × 10³⁸ |

### `isize` и `usize` — зависят от архитектуры

```rust
fn main() {
    let array_index: usize = 5;  // Индекс в массиве
    let length: usize = 100;     // Длина коллекции

    // На 64-битной системе: usize = u64
    // На 32-битной системе: usize = u32
}
```

**Важно:** Для индексов массивов ВСЕГДА используй `usize`.

## Литералы чисел

```rust
fn main() {
    // Десятичные
    let volume = 1_000_000;        // Подчёркивание для читаемости
    let price = 42_000;

    // Шестнадцатеричные (hex)
    let color = 0xFF00FF;          // Для цветов графика

    // Двоичные
    let flags = 0b1010_1010;       // Битовые флаги

    // Восьмеричные
    let permissions = 0o755;       // Unix права доступа

    // С суффиксом типа
    let small: i8 = 42i8;
    let big = 1_000_000u64;
}
```

## Арифметические операции

```rust
fn main() {
    let shares_bought = 100;
    let shares_sold = 30;
    let shares_left = shares_bought - shares_sold;

    println!("Осталось акций: {}", shares_left);

    // Все операции
    let a = 10;
    let b = 3;

    println!("Сложение: {} + {} = {}", a, b, a + b);
    println!("Вычитание: {} - {} = {}", a, b, a - b);
    println!("Умножение: {} * {} = {}", a, b, a * b);
    println!("Деление: {} / {} = {}", a, b, a / b);      // 3 (целочисленное!)
    println!("Остаток: {} % {} = {}", a, b, a % b);      // 1
}
```

**Важно:** Деление целых чисел даёт целое число! `10 / 3 = 3`, не 3.333...

## Переполнение

```rust
fn main() {
    let max_u8: u8 = 255;
    // let overflow: u8 = max_u8 + 1;  // ОШИБКА в debug режиме!

    // Безопасные операции
    let result = max_u8.checked_add(1);      // None если переполнение
    let wrapped = max_u8.wrapping_add(1);    // 0 (переполнение по кругу)
    let saturated = max_u8.saturating_add(1); // 255 (максимум)

    println!("Checked: {:?}", result);
    println!("Wrapped: {}", wrapped);
    println!("Saturated: {}", saturated);
}
```

**Аналогия:** Представь счётчик пробега в машине. Когда достигает 999999, следующий километр — или ошибка, или сбрасывается в 000000.

## Практический пример: подсчёт акций

```rust
fn main() {
    // Начальный портфель
    let mut apple_shares: u32 = 50;
    let mut google_shares: u32 = 20;
    let mut tesla_shares: u32 = 30;

    println!("=== Начало дня ===");
    println!("AAPL: {} акций", apple_shares);
    println!("GOOG: {} акций", google_shares);
    println!("TSLA: {} акций", tesla_shares);

    // Сделки за день
    apple_shares += 10;   // Докупили Apple
    google_shares -= 5;   // Продали часть Google
    tesla_shares += 15;   // Докупили Tesla

    println!("\n=== После торгов ===");
    println!("AAPL: {} акций", apple_shares);
    println!("GOOG: {} акций", google_shares);
    println!("TSLA: {} акций", tesla_shares);

    // Общее количество
    let total_shares = apple_shares + google_shares + tesla_shares;
    println!("\nВсего акций: {}", total_shares);
}
```

## Практический пример: Order ID генератор

```rust
fn main() {
    // Симулируем генератор ID ордеров
    let mut next_order_id: u64 = 1000000;

    // Создаём ордера
    let order1 = next_order_id;
    next_order_id += 1;

    let order2 = next_order_id;
    next_order_id += 1;

    let order3 = next_order_id;
    next_order_id += 1;

    println!("Order 1: #{}", order1);
    println!("Order 2: #{}", order2);
    println!("Order 3: #{}", order3);
    println!("Следующий ID: {}", next_order_id);
}
```

## Практический пример: расчёт лотов

```rust
fn main() {
    let balance_usd: u64 = 10_000;
    let apple_price: u64 = 185;
    let lot_size: u64 = 1;  // Минимальный лот

    // Сколько можем купить?
    let max_shares = balance_usd / apple_price;
    let remaining_usd = balance_usd % apple_price;

    println!("Баланс: ${}", balance_usd);
    println!("Цена AAPL: ${}", apple_price);
    println!("Можем купить: {} акций", max_shares);
    println!("Останется: ${}", remaining_usd);
    println!("Потратим: ${}", max_shares * apple_price);
}
```

## Преобразование между типами

```rust
fn main() {
    let small: u8 = 100;
    let medium: u32 = small as u32;    // Безопасно: u8 -> u32
    let large: u64 = medium as u64;    // Безопасно: u32 -> u64

    let big: u64 = 1000;
    let truncated: u8 = big as u8;     // Опасно! 1000 -> 232

    println!("Small: {}", small);
    println!("Large: {}", large);
    println!("Truncated (ОПАСНО!): {}", truncated);
}
```

**Правило:** Всегда можно безопасно преобразовать меньший тип в больший. Обратное — опасно!

## Побитовые операции

```rust
fn main() {
    // Флаги статуса ордера
    let filled: u8 = 0b0000_0001;    // Бит 0: исполнен
    let cancelled: u8 = 0b0000_0010; // Бит 1: отменён
    let partial: u8 = 0b0000_0100;   // Бит 2: частично исполнен

    let mut order_status: u8 = 0;

    // Устанавливаем флаг "частично исполнен"
    order_status |= partial;
    println!("Статус: {:08b}", order_status);

    // Потом полностью исполнен
    order_status |= filled;
    order_status &= !partial;  // Убираем partial
    println!("Статус: {:08b}", order_status);

    // Проверяем флаг
    if order_status & filled != 0 {
        println!("Ордер исполнен!");
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `i32`, `i64` | Числа со знаком (для PnL) |
| `u32`, `u64` | Числа без знака (для ID) |
| `usize` | Индексы массивов |
| `_` в числах | Для читаемости: 1_000_000 |
| Переполнение | Опасно! Используй checked/saturating |

## Домашнее задание

1. Создай симуляцию портфеля из 5 акций с количеством каждой

2. Реализуй функцию подсчёта общей стоимости:
   - Цены акций (целые доллары)
   - Количество каждой
   - Общая сумма

3. Поэкспериментируй с переполнением:
   - Создай `u8 = 255`
   - Попробуй прибавить 1 разными способами

4. Создай генератор Order ID, который начинается с текущего timestamp

## Навигация

[← Предыдущий день](../006-data-types/ru.md) | [Следующий день →](../008-floating-point-bitcoin-price/ru.md)
