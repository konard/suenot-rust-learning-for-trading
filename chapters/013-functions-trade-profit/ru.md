# День 13: Функции — рассчитываем прибыль сделки

## Аналогия из трейдинга

В трейдинге есть много повторяющихся вычислений:
- Расчёт прибыли сделки
- Расчёт размера позиции
- Вычисление индикаторов
- Проверка условий входа

Вместо того чтобы писать один и тот же код снова и снова, мы выносим его в **функции**. Функция — это именованный блок кода, который можно вызывать много раз.

## Простейшая функция

```rust
fn main() {
    say_hello();
    say_hello();
    say_hello();
}

fn say_hello() {
    println!("Hello, Trader!");
}
```

- `fn` — ключевое слово для объявления функции
- `say_hello` — имя функции (snake_case)
- `()` — параметры (пока пустые)
- `{}` — тело функции

## Функция с параметрами

```rust
fn main() {
    print_price("BTC", 42000.0);
    print_price("ETH", 2500.0);
    print_price("SOL", 95.0);
}

fn print_price(symbol: &str, price: f64) {
    println!("{}: ${:.2}", symbol, price);
}
```

Параметры указываются с типами: `имя: тип`

## Функция с возвращаемым значением

```rust
fn main() {
    let profit = calculate_profit(42000.0, 43500.0, 0.5);
    println!("Profit: ${:.2}", profit);
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}
```

- `-> f64` — тип возвращаемого значения
- Последнее выражение БЕЗ точки с запятой — это возвращаемое значение

## return для раннего выхода

```rust
fn main() {
    println!("Profit%: {:.2}%", calculate_profit_percent(42000.0, 43500.0));
    println!("Profit%: {:.2}%", calculate_profit_percent(42000.0, 0.0));
}

fn calculate_profit_percent(entry: f64, exit: f64) -> f64 {
    // Защита от деления на ноль
    if entry == 0.0 {
        return 0.0;
    }

    ((exit - entry) / entry) * 100.0
}
```

## Несколько функций

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;
    let fee_percent = 0.1;

    let gross_profit = calculate_gross_profit(entry, exit, quantity);
    let fees = calculate_fees(entry, exit, quantity, fee_percent);
    let net_profit = calculate_net_profit(gross_profit, fees);

    println!("Gross Profit: ${:.2}", gross_profit);
    println!("Fees: ${:.2}", fees);
    println!("Net Profit: ${:.2}", net_profit);
}

fn calculate_gross_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_fees(entry: f64, exit: f64, quantity: f64, fee_percent: f64) -> f64 {
    let entry_value = entry * quantity;
    let exit_value = exit * quantity;
    (entry_value + exit_value) * (fee_percent / 100.0)
}

fn calculate_net_profit(gross: f64, fees: f64) -> f64 {
    gross - fees
}
```

## Функции, вызывающие другие функции

```rust
fn main() {
    let result = full_trade_analysis(42000.0, 43500.0, 0.5, 0.1);
    println!("Net profit: ${:.2}", result);
}

fn full_trade_analysis(entry: f64, exit: f64, quantity: f64, fee_percent: f64) -> f64 {
    let gross = calculate_gross_profit(entry, exit, quantity);
    let fees = calculate_fees(entry, exit, quantity, fee_percent);
    calculate_net_profit(gross, fees)
}

fn calculate_gross_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_fees(entry: f64, exit: f64, quantity: f64, fee_percent: f64) -> f64 {
    (entry * quantity + exit * quantity) * (fee_percent / 100.0)
}

fn calculate_net_profit(gross: f64, fees: f64) -> f64 {
    gross - fees
}
```

## Документирование функций

```rust
/// Рассчитывает размер позиции на основе риск-менеджмента.
///
/// # Arguments
///
/// * `balance` - Доступный баланс
/// * `risk_percent` - Процент риска на сделку (например, 2.0 для 2%)
/// * `entry_price` - Цена входа
/// * `stop_loss` - Цена стоп-лосса
///
/// # Returns
///
/// Количество единиц актива для покупки
///
/// # Example
///
/// ```
/// let size = calculate_position_size(10000.0, 2.0, 42000.0, 41000.0);
/// assert_eq!(size, 0.2);
/// ```
fn calculate_position_size(
    balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64
) -> f64 {
    let risk_amount = balance * (risk_percent / 100.0);
    let price_risk = (entry_price - stop_loss).abs();

    if price_risk == 0.0 {
        return 0.0;
    }

    risk_amount / price_risk
}

fn main() {
    let size = calculate_position_size(10000.0, 2.0, 42000.0, 41000.0);
    println!("Position size: {} BTC", size);
}
```

## Практический пример: торговый калькулятор

```rust
fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║       TRADING CALCULATOR              ║");
    println!("╚═══════════════════════════════════════╝\n");

    let balance = 10000.0;
    let risk_percent = 2.0;
    let entry = 42000.0;
    let stop_loss = 41000.0;
    let take_profit = 44000.0;
    let fee_percent = 0.1;

    // Расчёт размера позиции
    let position_size = calculate_position_size(balance, risk_percent, entry, stop_loss);
    let position_value = position_size * entry;

    // Расчёт рисков и прибыли
    let max_loss = calculate_pnl(entry, stop_loss, position_size);
    let max_profit = calculate_pnl(entry, take_profit, position_size);
    let risk_reward = calculate_risk_reward(entry, stop_loss, take_profit);

    // Комиссии
    let entry_fee = calculate_fee(position_value, fee_percent);
    let exit_fee_loss = calculate_fee(position_size * stop_loss, fee_percent);
    let exit_fee_profit = calculate_fee(position_size * take_profit, fee_percent);

    // Чистый результат
    let net_loss = max_loss - entry_fee - exit_fee_loss;
    let net_profit = max_profit - entry_fee - exit_fee_profit;

    // Вывод
    print_section("INPUT DATA");
    println!("Balance:      ${:.2}", balance);
    println!("Risk:         {:.1}%", risk_percent);
    println!("Entry:        ${:.2}", entry);
    println!("Stop Loss:    ${:.2}", stop_loss);
    println!("Take Profit:  ${:.2}", take_profit);

    print_section("POSITION");
    println!("Size:         {:.8} BTC", position_size);
    println!("Value:        ${:.2}", position_value);

    print_section("RISK/REWARD");
    println!("Max Loss:     ${:.2}", max_loss);
    println!("Max Profit:   ${:.2}", max_profit);
    println!("R:R Ratio:    1:{:.2}", risk_reward);

    print_section("FEES (@ {:.1}%)", fee_percent);
    println!("Entry fee:    ${:.2}", entry_fee);
    println!("Exit (loss):  ${:.2}", exit_fee_loss);
    println!("Exit (profit):${:.2}", exit_fee_profit);

    print_section("NET RESULT");
    println!("Net Loss:     ${:.2}", net_loss);
    println!("Net Profit:   ${:.2}", net_profit);
}

fn calculate_position_size(balance: f64, risk_percent: f64, entry: f64, stop_loss: f64) -> f64 {
    let risk_amount = balance * (risk_percent / 100.0);
    let price_risk = (entry - stop_loss).abs();
    if price_risk == 0.0 { 0.0 } else { risk_amount / price_risk }
}

fn calculate_pnl(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_risk_reward(entry: f64, stop_loss: f64, take_profit: f64) -> f64 {
    let risk = (entry - stop_loss).abs();
    let reward = (take_profit - entry).abs();
    if risk == 0.0 { 0.0 } else { reward / risk }
}

fn calculate_fee(value: f64, fee_percent: f64) -> f64 {
    value * (fee_percent / 100.0)
}

fn print_section(title: &str) {
    println!("\n--- {} ---", title);
}
```

## Выражения vs инструкции

```rust
fn main() {
    // Инструкция (statement) - не возвращает значение
    let x = 5;  // Это инструкция

    // Выражение (expression) - возвращает значение
    let y = {
        let temp = 3;
        temp + 1  // Это выражение (нет ;)
    };

    println!("y = {}", y);  // 4

    // В функциях то же самое
    let result = add(2, 3);
    println!("result = {}", result);
}

fn add(a: i32, b: i32) -> i32 {
    a + b  // Выражение - возвращается
}

fn add_with_return(a: i32, b: i32) -> i32 {
    return a + b;  // Явный return
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `fn name()` | Объявление функции |
| `fn name(x: T)` | Функция с параметром |
| `-> T` | Возвращаемый тип |
| Без `;` | Возвращаемое выражение |
| `return` | Ранний выход |
| `///` | Документационный комментарий |

## Домашнее задание

1. Напиши функции для полного анализа сделки:
   - `calculate_position_value(price, quantity) -> f64`
   - `calculate_pnl(entry, exit, quantity) -> f64`
   - `calculate_pnl_percent(entry, exit) -> f64`
   - `is_profitable(entry, exit) -> bool`

2. Создай функцию расчёта Kelly Criterion:
   `kelly(win_rate, avg_win, avg_loss) -> f64`

3. Напиши функцию форматирования цены:
   `format_price(price, decimals) -> String`

4. Реализуй функцию проверки валидности ордера:
   `is_valid_order(price, quantity, balance) -> bool`

## Навигация

[← Предыдущий день](../012-arrays-closing-prices/ru.md) | [Следующий день →](../014-function-parameters/ru.md)
