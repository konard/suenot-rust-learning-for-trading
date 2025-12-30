# День 9: Булевы значения — рынок открыт или закрыт?

## Аналогия из трейдинга

В трейдинге постоянно нужно отвечать на вопросы "да или нет":
- Рынок открыт? **да/нет**
- Есть открытая позиция? **да/нет**
- Сработал стоп-лосс? **да/нет**
- Достаточно ли баланса? **да/нет**

Для таких вопросов существует тип `bool` (boolean) — он может быть только `true` (да) или `false` (нет).

## Базовое использование

```rust
fn main() {
    let market_open: bool = true;
    let has_position: bool = false;
    let is_profitable: bool = true;

    println!("Рынок открыт: {}", market_open);
    println!("Есть позиция: {}", has_position);
    println!("В прибыли: {}", is_profitable);
}
```

## Операторы сравнения

Сравнения возвращают `bool`:

```rust
fn main() {
    let btc_price = 42000.0;
    let entry_price = 41000.0;
    let stop_loss = 40000.0;
    let take_profit = 45000.0;

    // Сравнения
    let in_profit = btc_price > entry_price;
    let stop_triggered = btc_price <= stop_loss;
    let tp_reached = btc_price >= take_profit;
    let price_unchanged = btc_price == entry_price;
    let price_changed = btc_price != entry_price;

    println!("В прибыли: {}", in_profit);           // true
    println!("Стоп сработал: {}", stop_triggered);  // false
    println!("ТП достигнут: {}", tp_reached);       // false
    println!("Цена не изменилась: {}", price_unchanged); // false
    println!("Цена изменилась: {}", price_changed); // true
}
```

| Оператор | Значение | Пример |
|----------|----------|--------|
| `==` | Равно | `price == 42000.0` |
| `!=` | Не равно | `price != 0.0` |
| `>` | Больше | `price > stop_loss` |
| `<` | Меньше | `price < take_profit` |
| `>=` | Больше или равно | `balance >= min_order` |
| `<=` | Меньше или равно | `risk <= max_risk` |

## Логические операторы

### AND (И) — `&&`

Оба условия должны быть true:

```rust
fn main() {
    let has_balance = true;
    let market_open = true;
    let signal_buy = true;

    // Можем торговать только если ВСЕ условия выполнены
    let can_trade = has_balance && market_open && signal_buy;

    println!("Можем торговать: {}", can_trade);  // true
}
```

**Аналогия:** Чтобы открыть сделку, нужно И иметь деньги, И рынок открыт, И есть сигнал.

### OR (ИЛИ) — `||`

Хотя бы одно условие должно быть true:

```rust
fn main() {
    let stop_triggered = false;
    let tp_reached = true;
    let manual_close = false;

    // Закрываем если ЛЮБОЕ условие сработало
    let should_close = stop_triggered || tp_reached || manual_close;

    println!("Закрываем позицию: {}", should_close);  // true
}
```

**Аналогия:** Закрываем сделку ИЛИ по стопу, ИЛИ по тейку, ИЛИ вручную.

### NOT (НЕ) — `!`

Инвертирует значение:

```rust
fn main() {
    let market_open = true;
    let market_closed = !market_open;

    println!("Рынок открыт: {}", market_open);
    println!("Рынок закрыт: {}", market_closed);

    let has_position = false;
    let can_open_new = !has_position;  // Можем открыть, если нет позиции

    println!("Можем открыть новую: {}", can_open_new);
}
```

## Комбинирование условий

```rust
fn main() {
    let balance = 10000.0;
    let min_balance = 1000.0;
    let current_price = 42000.0;
    let entry_price = 41000.0;
    let stop_loss = 40000.0;
    let has_position = true;
    let market_open = true;

    // Сложное условие для закрытия позиции
    let should_close = has_position && (
        current_price <= stop_loss ||           // Стоп-лосс
        current_price >= entry_price * 1.05 ||  // +5% профит
        !market_open                            // Рынок закрывается
    );

    // Можем ли открыть новую позицию
    let can_open = !has_position &&
                   market_open &&
                   balance >= min_balance;

    println!("Закрываем: {}", should_close);
    println!("Можем открыть: {}", can_open);
}
```

## Приоритет операторов

От высшего к низшему:
1. `!` (NOT)
2. `&&` (AND)
3. `||` (OR)

```rust
fn main() {
    let a = true;
    let b = false;
    let c = true;

    // Это:
    let result1 = a || b && c;
    // Эквивалентно:
    let result2 = a || (b && c);  // && выполняется первым

    println!("a || b && c = {}", result1);  // true
    println!("a || (b && c) = {}", result2);  // true

    // Если нужен другой порядок — используй скобки:
    let result3 = (a || b) && c;
    println!("(a || b) && c = {}", result3);  // true
}
```

## Ленивые вычисления (Short-circuit)

Rust не проверяет второе условие, если результат уже известен:

```rust
fn main() {
    let has_position = false;
    let price = 42000.0;

    // Второе условие НЕ проверяется, т.к. первое уже false
    let should_sell = has_position && price > 45000.0;

    // Второе условие НЕ проверяется, т.к. первое уже true
    let market_closed = true;
    let should_wait = market_closed || price < 0.0;

    println!("Should sell: {}", should_sell);
    println!("Should wait: {}", should_wait);
}
```

**Важно для торговли:** Это можно использовать для защиты от ошибок:

```rust
fn main() {
    let quantity = 0.0;
    let price = 42000.0;

    // Безопасно: если quantity == 0, деление не выполнится
    let is_good_deal = quantity > 0.0 && (price / quantity) < 1000.0;
}
```

## Практический пример: сигнальная система

```rust
fn main() {
    // Данные рынка
    let current_price = 42500.0;
    let sma_20 = 42000.0;  // 20-периодная скользящая средняя
    let sma_50 = 41500.0;  // 50-периодная скользящая средняя
    let rsi = 65.0;        // RSI индикатор
    let volume = 1500.0;   // Объём
    let avg_volume = 1000.0;

    // Условия для сигналов
    let price_above_sma20 = current_price > sma_20;
    let price_above_sma50 = current_price > sma_50;
    let sma_bullish = sma_20 > sma_50;  // "Золотой крест"
    let rsi_not_overbought = rsi < 70.0;
    let high_volume = volume > avg_volume * 1.2;

    // Сигнал на покупку
    let buy_signal = price_above_sma20 &&
                     price_above_sma50 &&
                     sma_bullish &&
                     rsi_not_overbought &&
                     high_volume;

    // Условия для продажи
    let price_below_sma20 = current_price < sma_20;
    let rsi_oversold = rsi < 30.0;

    let sell_signal = price_below_sma20 || rsi_oversold;

    // Отчёт
    println!("╔══════════════════════════════════╗");
    println!("║        SIGNAL ANALYSIS           ║");
    println!("╠══════════════════════════════════╣");
    println!("║ Price > SMA20:      {:>12} ║", price_above_sma20);
    println!("║ Price > SMA50:      {:>12} ║", price_above_sma50);
    println!("║ SMA Bullish:        {:>12} ║", sma_bullish);
    println!("║ RSI OK:             {:>12} ║", rsi_not_overbought);
    println!("║ High Volume:        {:>12} ║", high_volume);
    println!("╠══════════════════════════════════╣");
    println!("║ BUY SIGNAL:         {:>12} ║", buy_signal);
    println!("║ SELL SIGNAL:        {:>12} ║", sell_signal);
    println!("╚══════════════════════════════════╝");
}
```

## Практический пример: проверка ордера

```rust
fn main() {
    // Параметры ордера
    let order_side = "BUY";
    let order_price = 42000.0;
    let order_quantity = 0.5;
    let order_value = order_price * order_quantity;

    // Параметры аккаунта и рынка
    let balance = 25000.0;
    let min_order_value = 10.0;
    let max_order_value = 100000.0;
    let market_open = true;
    let trading_enabled = true;

    // Валидация
    let has_enough_balance = balance >= order_value;
    let above_min = order_value >= min_order_value;
    let below_max = order_value <= max_order_value;
    let positive_quantity = order_quantity > 0.0;
    let positive_price = order_price > 0.0;
    let valid_side = order_side == "BUY" || order_side == "SELL";

    // Итоговая проверка
    let order_valid = has_enough_balance &&
                      above_min &&
                      below_max &&
                      positive_quantity &&
                      positive_price &&
                      valid_side &&
                      market_open &&
                      trading_enabled;

    println!("=== Order Validation ===");
    println!("Side: {}", order_side);
    println!("Price: ${}", order_price);
    println!("Quantity: {}", order_quantity);
    println!("Value: ${}", order_value);
    println!();
    println!("Balance OK: {}", has_enough_balance);
    println!("Above min: {}", above_min);
    println!("Below max: {}", below_max);
    println!("Valid quantity: {}", positive_quantity);
    println!("Valid price: {}", positive_price);
    println!("Valid side: {}", valid_side);
    println!("Market open: {}", market_open);
    println!("Trading enabled: {}", trading_enabled);
    println!();
    println!("ORDER VALID: {}", order_valid);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `bool` | true или false |
| `==`, `!=`, `<`, `>` | Сравнения |
| `&&` | Логическое И |
| `||` | Логическое ИЛИ |
| `!` | Логическое НЕ |
| Short-circuit | Ленивые вычисления |

## Домашнее задание

1. Создай систему проверки сигнала на покупку с 5+ условиями

2. Реализуй проверку: можно ли открыть позицию (проверь баланс, лимиты, статус рынка)

3. Напиши логику закрытия позиции по 3 условиям:
   - Стоп-лосс
   - Тейк-профит
   - Истечение времени

4. Поэкспериментируй с приоритетом операторов

## Навигация

[← Предыдущий день](../008-floating-point-bitcoin-price/ru.md) | [Следующий день →](../010-strings-tickers/ru.md)
