# День 14: Параметры функций — передаём цену входа и выхода

## Аналогия из трейдинга

Когда ты рассчитываешь прибыль сделки, тебе нужны **входные данные**:
- Цена входа
- Цена выхода
- Объём
- Комиссия

Эти данные передаются в функцию как **параметры**. Функция использует их для вычислений и возвращает результат.

## Основы параметров

```rust
fn main() {
    // Передаём параметры в функцию
    let result = calculate_pnl(42000.0, 43500.0, 0.5);
    println!("PnL: ${:.2}", result);
}

// Объявляем параметры с типами
fn calculate_pnl(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}
```

Каждый параметр **обязательно** должен иметь тип!

## Передача по значению

Простые типы (числа, bool) передаются **по значению** — создаётся копия:

```rust
fn main() {
    let price = 42000.0;
    double_price(price);
    println!("Original price: {}", price);  // Всё ещё 42000.0!
}

fn double_price(mut p: f64) {
    p = p * 2.0;
    println!("Doubled: {}", p);  // 84000.0
}
```

Изменение внутри функции НЕ влияет на оригинал.

## Передача по ссылке

Для больших данных используем ссылки:

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];

    // Передаём ссылку на массив
    let avg = calculate_average(&prices);
    println!("Average: {:.2}", avg);
}

// &[f64] — ссылка на срез (любой размер)
fn calculate_average(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    let sum: f64 = prices.iter().sum();
    sum / prices.len() as f64
}
```

## Изменяемые ссылки

Если нужно изменить данные внутри функции:

```rust
fn main() {
    let mut balance = 10000.0;
    println!("Before: {}", balance);

    add_profit(&mut balance, 500.0);
    println!("After profit: {}", balance);

    add_profit(&mut balance, -200.0);  // Убыток как отрицательная прибыль
    println!("After loss: {}", balance);
}

// &mut f64 — изменяемая ссылка
fn add_profit(balance: &mut f64, profit: f64) {
    *balance += profit;  // * для разыменования
}
```

## Разные способы передачи

```rust
fn main() {
    // 1. По значению (для простых типов)
    let x = 42;
    takes_value(x);
    println!("x still exists: {}", x);

    // 2. По ссылке (для чтения)
    let prices = vec![1.0, 2.0, 3.0];
    takes_reference(&prices);
    println!("prices still exists: {:?}", prices);

    // 3. По изменяемой ссылке (для изменения)
    let mut balance = 100.0;
    takes_mut_reference(&mut balance);
    println!("balance changed: {}", balance);

    // 4. Перемещение (забираем владение)
    let data = String::from("BTC");
    takes_ownership(data);
    // println!("{}", data);  // ОШИБКА! data уже перемещена
}

fn takes_value(n: i32) {
    println!("Got value: {}", n);
}

fn takes_reference(v: &Vec<f64>) {
    println!("Got reference, len: {}", v.len());
}

fn takes_mut_reference(b: &mut f64) {
    *b += 50.0;
}

fn takes_ownership(s: String) {
    println!("Got ownership of: {}", s);
}
```

## Возврат нескольких значений через кортеж

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42150.0];

    let (min, max, avg) = analyze_prices(&prices);

    println!("Min: {:.2}", min);
    println!("Max: {:.2}", max);
    println!("Avg: {:.2}", avg);
    println!("Range: {:.2}", max - min);
}

fn analyze_prices(prices: &[f64]) -> (f64, f64, f64) {
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    let mut sum = 0.0;

    for &price in prices {
        if price < min { min = price; }
        if price > max { max = price; }
        sum += price;
    }

    let avg = if prices.is_empty() { 0.0 } else { sum / prices.len() as f64 };

    (min, max, avg)
}
```

## Параметры со значениями по умолчанию (через Builder)

В Rust нет значений по умолчанию для параметров, но можно использовать паттерн:

```rust
fn main() {
    // Вариант 1: полный набор параметров
    let pnl1 = calculate_net_pnl(42000.0, 43500.0, 0.5, 0.1);

    // Вариант 2: упрощённая версия (с дефолтной комиссией)
    let pnl2 = calculate_net_pnl_default(42000.0, 43500.0, 0.5);

    println!("With custom fee: {:.2}", pnl1);
    println!("With default fee: {:.2}", pnl2);
}

fn calculate_net_pnl(entry: f64, exit: f64, qty: f64, fee_percent: f64) -> f64 {
    let gross = (exit - entry) * qty;
    let fee = (entry * qty + exit * qty) * (fee_percent / 100.0);
    gross - fee
}

// Обёртка с дефолтным значением
fn calculate_net_pnl_default(entry: f64, exit: f64, qty: f64) -> f64 {
    calculate_net_pnl(entry, exit, qty, 0.1)  // 0.1% по умолчанию
}
```

## Практический пример: полный расчёт сделки

```rust
fn main() {
    // Данные сделки
    let symbol = "BTC/USDT";
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;
    let fee_percent = 0.1;
    let is_long = true;

    // Расчёты
    let gross_pnl = calculate_gross_pnl(entry_price, exit_price, quantity, is_long);
    let (entry_fee, exit_fee, total_fees) = calculate_fees(
        entry_price, exit_price, quantity, fee_percent
    );
    let net_pnl = gross_pnl - total_fees;
    let roi = calculate_roi(entry_price, quantity, net_pnl);

    // Вывод
    print_trade_summary(
        symbol,
        entry_price,
        exit_price,
        quantity,
        is_long,
        gross_pnl,
        total_fees,
        net_pnl,
        roi
    );
}

fn calculate_gross_pnl(entry: f64, exit: f64, qty: f64, is_long: bool) -> f64 {
    if is_long {
        (exit - entry) * qty
    } else {
        (entry - exit) * qty  // Short: прибыль при падении
    }
}

fn calculate_fees(entry: f64, exit: f64, qty: f64, fee_pct: f64) -> (f64, f64, f64) {
    let entry_fee = entry * qty * (fee_pct / 100.0);
    let exit_fee = exit * qty * (fee_pct / 100.0);
    (entry_fee, exit_fee, entry_fee + exit_fee)
}

fn calculate_roi(entry: f64, qty: f64, net_pnl: f64) -> f64 {
    let investment = entry * qty;
    if investment == 0.0 { 0.0 } else { (net_pnl / investment) * 100.0 }
}

fn print_trade_summary(
    symbol: &str,
    entry: f64,
    exit: f64,
    qty: f64,
    is_long: bool,
    gross: f64,
    fees: f64,
    net: f64,
    roi: f64
) {
    println!("╔══════════════════════════════════════╗");
    println!("║         TRADE SUMMARY                ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ Symbol:     {:<24} ║", symbol);
    println!("║ Direction:  {:<24} ║", if is_long { "LONG" } else { "SHORT" });
    println!("║ Entry:      ${:<23.2} ║", entry);
    println!("║ Exit:       ${:<23.2} ║", exit);
    println!("║ Quantity:   {:<24.8} ║", qty);
    println!("╠══════════════════════════════════════╣");
    println!("║ Gross PnL:  ${:<23.2} ║", gross);
    println!("║ Fees:       ${:<23.2} ║", fees);
    println!("║ Net PnL:    ${:<23.2} ║", net);
    println!("║ ROI:        {:<23.2}% ║", roi);
    println!("╚══════════════════════════════════════╝");
}
```

## Общие паттерны параметров

```rust
// Паттерн 1: Цена и количество
fn process_order(price: f64, quantity: f64) -> f64 {
    price * quantity
}

// Паттерн 2: OHLC данные
fn analyze_candle(open: f64, high: f64, low: f64, close: f64) -> bool {
    close > open  // Bullish?
}

// Паттерн 3: Массив цен
fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;
    }
    let slice = &prices[prices.len() - period..];
    let sum: f64 = slice.iter().sum();
    Some(sum / period as f64)
}

// Паттерн 4: Изменение баланса
fn apply_trade_result(balance: &mut f64, pnl: f64) {
    *balance += pnl;
}

fn main() {
    // Использование всех паттернов
    let value = process_order(42000.0, 0.5);
    println!("Order value: {}", value);

    let is_bullish = analyze_candle(42000.0, 42500.0, 41800.0, 42300.0);
    println!("Bullish: {}", is_bullish);

    let prices = [42000.0, 42100.0, 41900.0, 42200.0, 42150.0];
    if let Some(sma) = calculate_sma(&prices, 3) {
        println!("SMA-3: {:.2}", sma);
    }

    let mut balance = 10000.0;
    apply_trade_result(&mut balance, 500.0);
    println!("New balance: {}", balance);
}
```

## Что мы узнали

| Передача | Синтаксис | Когда использовать |
|----------|-----------|-------------------|
| По значению | `fn f(x: i32)` | Простые типы (Copy) |
| По ссылке | `fn f(x: &T)` | Только чтение |
| Изменяемая ссылка | `fn f(x: &mut T)` | Нужно изменить |
| Перемещение | `fn f(x: String)` | Передать владение |

## Домашнее задание

1. Напиши функцию `update_portfolio(portfolio: &mut Vec<f64>, trade_result: f64)`, которая добавляет результат сделки в портфель

2. Создай функцию `validate_trade_params(entry: f64, stop: f64, take: f64) -> bool`, которая проверяет логичность параметров

3. Реализуй функцию `split_position(total: f64, parts: usize) -> Vec<f64>`, которая делит позицию на части

4. Напиши функцию `merge_candles(candles: &[(f64, f64, f64, f64)]) -> (f64, f64, f64, f64)`, которая объединяет несколько свечей в одну

## Навигация

[← Предыдущий день](../013-functions-trade-profit/ru.md) | [Следующий день →](../015-return-values-pnl/ru.md)
