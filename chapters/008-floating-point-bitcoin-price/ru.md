# День 8: Числа с плавающей точкой — точная цена биткоина

## Аналогия из трейдинга

Биткоин торгуется с точностью до 8 знаков после запятой:
- BTC: 42156.**78901234**
- ETH: 2250.**123456**
- USDT: 1.**0001**

Для работы с такими числами нужны **числа с плавающей точкой** (float).

## Типы в Rust

| Тип | Размер | Точность | Когда использовать |
|-----|--------|----------|-------------------|
| `f32` | 32 бита | ~7 цифр | Графики, быстрые расчёты |
| `f64` | 64 бита | ~15 цифр | Цены, деньги |

```rust
fn main() {
    let btc_price: f64 = 42156.78901234;
    let eth_price: f64 = 2250.50;

    println!("BTC: ${}", btc_price);
    println!("ETH: ${}", eth_price);
}
```

**Правило:** Для денег ВСЕГДА используй `f64`!

## Почему f64, а не f32?

```rust
fn main() {
    // Проблема с f32
    let balance_f32: f32 = 1_000_000.0;
    let small_trade_f32: f32 = 0.01;
    let result_f32 = balance_f32 + small_trade_f32;

    println!("f32: {} + {} = {}", balance_f32, small_trade_f32, result_f32);
    // Может вывести: 1000000.0 (потеряли копейку!)

    // Решение с f64
    let balance_f64: f64 = 1_000_000.0;
    let small_trade_f64: f64 = 0.01;
    let result_f64 = balance_f64 + small_trade_f64;

    println!("f64: {} + {} = {}", balance_f64, small_trade_f64, result_f64);
    // Выведет: 1000000.01 (точно!)
}
```

## Арифметика с float

```rust
fn main() {
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;

    // Основные операции
    let profit = (exit_price - entry_price) * quantity;
    let percent_change = ((exit_price - entry_price) / entry_price) * 100.0;

    println!("Прибыль: ${:.2}", profit);
    println!("Изменение: {:.2}%", percent_change);
}
```

## Специальные значения

```rust
fn main() {
    // Бесконечность
    let infinity = f64::INFINITY;
    let neg_infinity = f64::NEG_INFINITY;

    // Деление на ноль
    let result = 1.0 / 0.0;  // Infinity

    // NaN (Not a Number)
    let nan = f64::NAN;
    let also_nan = 0.0 / 0.0;

    println!("Infinity: {}", infinity);
    println!("1/0 = {}", result);
    println!("NaN: {}", nan);

    // Проверка на NaN
    println!("Is NaN: {}", nan.is_nan());
    println!("Is infinite: {}", infinity.is_infinite());
    println!("Is finite: {}", 42.0_f64.is_finite());
}
```

**В трейдинге:** NaN может появиться при делении 0/0 (например, процент от нулевого объёма). Всегда проверяй!

## Округление

```rust
fn main() {
    let price = 42156.789012;

    // Различные способы округления
    println!("Исходная: {}", price);
    println!("floor (вниз): {}", price.floor());     // 42156.0
    println!("ceil (вверх): {}", price.ceil());      // 42157.0
    println!("round (к ближайшему): {}", price.round()); // 42157.0
    println!("trunc (отбросить): {}", price.trunc()); // 42156.0

    // Округление до N знаков после запятой
    let rounded = (price * 100.0).round() / 100.0;
    println!("До 2 знаков: {}", rounded);  // 42156.79
}
```

## Форматирование вывода

```rust
fn main() {
    let btc_price = 42156.789012345;
    let eth_quantity = 1.23456789;

    // Количество знаков после запятой
    println!("Цена: ${:.2}", btc_price);      // $42156.79
    println!("Количество: {:.8}", eth_quantity); // 1.23456789

    // Ширина поля
    println!("Цена: ${:>12.2}", btc_price);   // $    42156.79
    println!("Цена: ${:<12.2}", btc_price);   // $42156.79

    // Заполнение нулями
    println!("ID: {:08.2}", 42.5);            // 00042.50
}
```

## Математические функции

```rust
fn main() {
    let price = 42000.0_f64;

    // Корень (для волатильности)
    let sqrt = price.sqrt();
    println!("Корень: {}", sqrt);

    // Степень
    let squared = price.powi(2);        // Целая степень
    let powered = price.powf(1.5);      // Дробная степень
    println!("Квадрат: {}", squared);

    // Логарифм (для log returns)
    let ln = price.ln();                // Натуральный
    let log10 = price.log10();          // Десятичный
    println!("ln(price): {}", ln);

    // Экспонента
    let exp = 0.05_f64.exp();           // e^0.05
    println!("e^0.05: {}", exp);

    // Абсолютное значение
    let loss = -500.0_f64;
    println!("Убыток: {}, Абс: {}", loss, loss.abs());

    // Мин/макс
    let a = 42000.0_f64;
    let b = 43000.0_f64;
    println!("Min: {}, Max: {}", a.min(b), a.max(b));
}
```

## Практический пример: расчёт доходности

```rust
fn main() {
    let initial_price = 40000.0;
    let final_price = 42000.0;

    // Простая доходность
    let simple_return = (final_price - initial_price) / initial_price * 100.0;

    // Логарифмическая доходность (лучше для финансов)
    let log_return = (final_price / initial_price).ln() * 100.0;

    println!("Начальная цена: ${:.2}", initial_price);
    println!("Конечная цена: ${:.2}", final_price);
    println!("Простая доходность: {:.2}%", simple_return);
    println!("Log доходность: {:.2}%", log_return);
}
```

## Практический пример: расчёт волатильности

```rust
fn main() {
    // Дневные доходности (в процентах)
    let returns = [1.5, -0.8, 2.1, -1.2, 0.5, 1.8, -0.3];

    // Среднее
    let sum: f64 = returns.iter().sum();
    let mean = sum / returns.len() as f64;

    // Дисперсия
    let variance: f64 = returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / returns.len() as f64;

    // Стандартное отклонение (волатильность)
    let volatility = variance.sqrt();

    // Годовая волатильность (примерно 252 торговых дня)
    let annual_volatility = volatility * (252.0_f64).sqrt();

    println!("Средняя доходность: {:.2}%", mean);
    println!("Дневная волатильность: {:.2}%", volatility);
    println!("Годовая волатильность: {:.2}%", annual_volatility);
}
```

## Проблема сравнения float

```rust
fn main() {
    let a = 0.1 + 0.2;
    let b = 0.3;

    // НЕПРАВИЛЬНО!
    if a == b {
        println!("Равны");
    } else {
        println!("Не равны: {} != {}", a, b);  // Сюрприз!
    }

    // ПРАВИЛЬНО: сравнение с погрешностью
    let epsilon = 1e-10;
    if (a - b).abs() < epsilon {
        println!("Практически равны");
    }
}
```

**Аналогия:** Это как сравнивать цены с разных бирж — они никогда не будут точно равны, но могут быть "достаточно близки".

## Практический пример: торговый калькулятор

```rust
fn main() {
    // Входные данные
    let balance: f64 = 10_000.0;
    let risk_percent: f64 = 2.0;
    let entry_price: f64 = 42_000.0;
    let stop_loss: f64 = 41_000.0;
    let take_profit: f64 = 44_000.0;
    let fee_percent: f64 = 0.1;

    // Расчёты
    let risk_amount = balance * (risk_percent / 100.0);
    let price_risk = entry_price - stop_loss;
    let position_size = risk_amount / price_risk;
    let position_value = position_size * entry_price;

    let potential_loss = price_risk * position_size;
    let potential_profit = (take_profit - entry_price) * position_size;
    let risk_reward = potential_profit / potential_loss;

    let entry_fee = position_value * (fee_percent / 100.0);
    let exit_fee = position_size * take_profit * (fee_percent / 100.0);
    let total_fees = entry_fee + exit_fee;

    let net_profit = potential_profit - total_fees;

    // Вывод
    println!("╔══════════════════════════════════╗");
    println!("║      TRADING CALCULATOR          ║");
    println!("╠══════════════════════════════════╣");
    println!("║ Balance:        ${:>14.2} ║", balance);
    println!("║ Risk:              {:>11.1}% ║", risk_percent);
    println!("║ Entry:          ${:>14.2} ║", entry_price);
    println!("║ Stop Loss:      ${:>14.2} ║", stop_loss);
    println!("║ Take Profit:    ${:>14.2} ║", take_profit);
    println!("╠══════════════════════════════════╣");
    println!("║ Position Size:   {:>14.8} ║", position_size);
    println!("║ Position Value: ${:>14.2} ║", position_value);
    println!("║ Potential Loss: ${:>14.2} ║", potential_loss);
    println!("║ Potential Profit:${:>13.2} ║", potential_profit);
    println!("║ Risk/Reward:      {:>13.2} ║", risk_reward);
    println!("║ Total Fees:     ${:>14.2} ║", total_fees);
    println!("║ Net Profit:     ${:>14.2} ║", net_profit);
    println!("╚══════════════════════════════════╝");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `f64` | Основной тип для денег |
| `f32` | Для быстрых расчётов |
| Округление | floor, ceil, round, trunc |
| Форматирование | {:.2} для 2 знаков |
| Сравнение | Используй epsilon |

## Домашнее задание

1. Создай калькулятор комиссий для биржи:
   - Maker fee: 0.1%
   - Taker fee: 0.2%
   - Рассчитай комиссию для сделки любого размера

2. Рассчитай log return для серии из 10 цен

3. Реализуй функцию округления до нужного количества знаков после запятой

4. Напиши проверку: является ли число конечным и не NaN

## Навигация

[← Предыдущий день](../007-integers-counting-shares/ru.md) | [Следующий день →](../009-booleans-market-status/ru.md)
