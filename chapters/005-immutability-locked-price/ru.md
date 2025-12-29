# День 5: Неизменяемость — цена сделки зафиксирована

## Аналогия из трейдинга

Когда ты совершаешь сделку на бирже, цена **фиксируется** в момент исполнения. Ты купил BTC по $42,000 — эта цена входа никогда не изменится. Она записана в историю.

В Rust переменные работают так же — по умолчанию они **неизменяемые** (immutable). Это защищает от случайных изменений данных.

## Неизменяемость по умолчанию

```rust
fn main() {
    let btc_price = 42000;
    btc_price = 43000;  // ОШИБКА! Нельзя изменить
}
```

Компилятор скажет:
```
error[E0384]: cannot assign twice to immutable variable `btc_price`
```

**Аналогия:** Это как попытка изменить цену уже совершённой сделки в истории. Нельзя!

## Зачем нужна неизменяемость?

В трейдинге это критически важно:

```rust
fn main() {
    let entry_price = 42000.0;
    let stop_loss = entry_price - 1000.0;  // 41000

    // ... много кода ...

    // Представь, кто-то случайно написал:
    // entry_price = 50000.0;

    // Теперь stop_loss рассчитан неправильно!
    // В реальном боте это может стоить денег!
}
```

Rust **защищает** тебя от таких ошибок на этапе компиляции.

## Изменяемые переменные с `mut`

Когда изменение **действительно нужно**, используй `mut`:

```rust
fn main() {
    let mut balance = 10000.0;
    println!("Начальный баланс: {} USDT", balance);

    // Совершили прибыльную сделку
    balance = balance + 500.0;
    println!("После сделки: {} USDT", balance);

    // Ещё одна сделка
    balance = balance - 200.0;
    println!("После второй сделки: {} USDT", balance);
}
```

Вывод:
```
Начальный баланс: 10000 USDT
После сделки: 10500 USDT
После второй сделки: 10300 USDT
```

## Когда использовать mut?

### Используй `mut` когда:
- Баланс меняется после каждой сделки
- Текущая цена обновляется в реальном времени
- Счётчик сделок увеличивается
- Позиция открывается/закрывается

### НЕ используй `mut` когда:
- Цена входа в сделку (фиксирована)
- Комиссия биржи (обычно константа)
- Начальный депозит (для отчётности)
- Параметры стратегии (для бэктеста)

## Практические примеры

### Пример 1: Обновление цены

```rust
fn main() {
    let mut current_price = 42000.0;

    println!("Цена: {}", current_price);

    // Цена изменилась
    current_price = 42150.0;
    println!("Новая цена: {}", current_price);

    current_price = 42300.0;
    println!("Ещё новее: {}", current_price);
}
```

### Пример 2: Подсчёт сделок

```rust
fn main() {
    let mut trade_count = 0;

    // Совершили сделку
    trade_count = trade_count + 1;
    println!("Сделок: {}", trade_count);

    // Ещё сделка
    trade_count = trade_count + 1;
    println!("Сделок: {}", trade_count);

    // И ещё
    trade_count = trade_count + 1;
    println!("Всего сделок за день: {}", trade_count);
}
```

### Пример 3: Симуляция торговли

```rust
fn main() {
    // Фиксированные параметры (неизменяемые)
    let entry_price = 42000.0;
    let take_profit = 43000.0;
    let stop_loss = 41500.0;
    let position_size = 0.5;

    // Изменяемое состояние
    let mut current_price = entry_price;
    let mut pnl = 0.0;

    println!("=== Симуляция сделки ===");
    println!("Вход: {} USDT", entry_price);
    println!("Тейк-профит: {} USDT", take_profit);
    println!("Стоп-лосс: {} USDT", stop_loss);

    // Цена движется вверх
    current_price = 42500.0;
    pnl = (current_price - entry_price) * position_size;
    println!("\nЦена: {} | PnL: {} USDT", current_price, pnl);

    current_price = 43000.0;
    pnl = (current_price - entry_price) * position_size;
    println!("Цена: {} | PnL: {} USDT", current_price, pnl);
    println!("Тейк-профит достигнут!");
}
```

## Сокращённые операторы присваивания

Вместо `x = x + 1` можно писать `x += 1`:

```rust
fn main() {
    let mut balance = 10000.0;

    balance += 500.0;   // balance = balance + 500.0
    println!("После прибыли: {}", balance);

    balance -= 200.0;   // balance = balance - 200.0
    println!("После убытка: {}", balance);

    balance *= 1.1;     // balance = balance * 1.1 (увеличили на 10%)
    println!("После роста: {}", balance);

    balance /= 2.0;     // balance = balance / 2.0
    println!("Половина: {}", balance);
}
```

## Что нельзя изменить

Тип переменной изменить нельзя:

```rust
fn main() {
    let mut price = 42000;    // целое число
    price = 42000.5;          // ОШИБКА! Нельзя присвоить дробное
}
```

Для этого есть **shadowing** (следующая тема).

## Практический пример: простой торговый бот

```rust
fn main() {
    // Константы стратегии
    let initial_balance = 10000.0;
    let risk_per_trade = 0.02;  // 2%

    // Изменяемое состояние
    let mut balance = initial_balance;
    let mut total_trades = 0;
    let mut winning_trades = 0;

    println!("=== Торговый бот ===");
    println!("Начальный баланс: {} USDT\n", balance);

    // Сделка 1: прибыльная
    let trade_result = 150.0;
    balance += trade_result;
    total_trades += 1;
    winning_trades += 1;
    println!("Сделка {}: +{} USDT | Баланс: {}", total_trades, trade_result, balance);

    // Сделка 2: убыточная
    let trade_result = -80.0;
    balance += trade_result;
    total_trades += 1;
    println!("Сделка {}: {} USDT | Баланс: {}", total_trades, trade_result, balance);

    // Сделка 3: прибыльная
    let trade_result = 200.0;
    balance += trade_result;
    total_trades += 1;
    winning_trades += 1;
    println!("Сделка {}: +{} USDT | Баланс: {}", total_trades, trade_result, balance);

    // Итоги
    println!("\n=== Итоги ===");
    println!("Всего сделок: {}", total_trades);
    println!("Прибыльных: {}", winning_trades);
    println!("Начальный баланс: {} USDT", initial_balance);
    println!("Конечный баланс: {} USDT", balance);
    println!("Прибыль: {} USDT", balance - initial_balance);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Immutable | По умолчанию переменные нельзя изменять |
| `mut` | Делает переменную изменяемой |
| `+=`, `-=` | Сокращённые операторы |
| Безопасность | Защита от случайных изменений |

## Домашнее задание

1. Создай симуляцию 5 торговых сделок:
   - Начальный баланс: 5000 USDT (неизменяемый)
   - Текущий баланс: изменяется после каждой сделки
   - Счётчик сделок: увеличивается

2. Попробуй изменить неизменяемую переменную — прочитай сообщение об ошибке

3. Добавь подсчёт:
   - Общее количество прибыльных сделок
   - Общее количество убыточных сделок
   - Максимальная прибыль за одну сделку

## Навигация

[← Предыдущий день](../004-variables-asset-prices/ru.md) | [Следующий день →](../006-data-types/ru.md)
