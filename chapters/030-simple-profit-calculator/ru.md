# День 30: Простой калькулятор прибыли

## Аналогия из трейдинга

Каждый трейдер постоянно считает прибыль и убытки. Сколько заработал на сделке? Какой процент от депозита? Учёл ли комиссии? Сегодня мы создадим инструмент, который делает это автоматически — **калькулятор прибыли**. Это первый шаг к построению реального торгового софта.

## Базовый расчёт прибыли

Прибыль сделки = (Цена выхода - Цена входа) × Количество

```rust
fn main() {
    let entry_price = 42000.0;  // Купили BTC по $42,000
    let exit_price = 43500.0;   // Продали по $43,500
    let quantity = 0.5;         // 0.5 BTC

    let profit = (exit_price - entry_price) * quantity;
    println!("Прибыль: ${:.2}", profit);  // $750.00
}
```

## Функция расчёта прибыли

Вынесем логику в функцию:

```rust
fn main() {
    let profit1 = calculate_profit(42000.0, 43500.0, 0.5);
    let profit2 = calculate_profit(2500.0, 2400.0, 2.0);
    let profit3 = calculate_profit(95.0, 110.0, 10.0);

    println!("BTC trade: ${:.2}", profit1);   // $750.00
    println!("ETH trade: ${:.2}", profit2);   // -$200.00
    println!("SOL trade: ${:.2}", profit3);   // $150.00
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}
```

## Расчёт процента прибыли

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;

    let profit_pct = calculate_profit_percent(entry, exit);
    println!("Прибыль: {:.2}%", profit_pct);  // 3.57%
}

fn calculate_profit_percent(entry: f64, exit: f64) -> f64 {
    if entry == 0.0 {
        return 0.0;  // Защита от деления на ноль
    }
    ((exit - entry) / entry) * 100.0
}
```

## Учёт комиссий

Биржи берут комиссию за каждую сделку. Обычно это процент от объёма:

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;
    let fee_percent = 0.1;  // 0.1% комиссия (типично для бирж)

    let gross_profit = calculate_profit(entry, exit, quantity);
    let total_fees = calculate_total_fees(entry, exit, quantity, fee_percent);
    let net_profit = gross_profit - total_fees;

    println!("Валовая прибыль: ${:.2}", gross_profit);
    println!("Комиссии: ${:.2}", total_fees);
    println!("Чистая прибыль: ${:.2}", net_profit);
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_total_fees(entry: f64, exit: f64, quantity: f64, fee_pct: f64) -> f64 {
    let entry_value = entry * quantity;
    let exit_value = exit * quantity;
    (entry_value + exit_value) * (fee_pct / 100.0)
}
```

## ROI — Return on Investment

ROI показывает, какой процент прибыли получен относительно вложенных средств:

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;
    let fee_pct = 0.1;

    let roi = calculate_roi(entry, exit, quantity, fee_pct);
    println!("ROI: {:.2}%", roi);
}

fn calculate_roi(entry: f64, exit: f64, quantity: f64, fee_pct: f64) -> f64 {
    let investment = entry * quantity;
    if investment == 0.0 {
        return 0.0;
    }

    let gross = (exit - entry) * quantity;
    let fees = (entry + exit) * quantity * (fee_pct / 100.0);
    let net = gross - fees;

    (net / investment) * 100.0
}
```

## Определение статуса сделки

```rust
fn main() {
    analyze_trade(42000.0, 43500.0, 0.5);
    analyze_trade(42000.0, 41000.0, 0.5);
    analyze_trade(42000.0, 42000.0, 0.5);
}

fn analyze_trade(entry: f64, exit: f64, quantity: f64) {
    let profit = calculate_profit(entry, exit, quantity);
    let status = get_trade_status(profit);
    let emoji = get_status_emoji(profit);

    println!("{} {} ${:.2}", emoji, status, profit.abs());
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn get_trade_status(profit: f64) -> &'static str {
    if profit > 0.0 {
        "PROFIT"
    } else if profit < 0.0 {
        "LOSS"
    } else {
        "BREAKEVEN"
    }
}

fn get_status_emoji(profit: f64) -> &'static str {
    if profit > 0.0 {
        "[+]"
    } else if profit < 0.0 {
        "[-]"
    } else {
        "[=]"
    }
}
```

## Полный калькулятор прибыли

```rust
fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║     КАЛЬКУЛЯТОР ПРИБЫЛИ               ║");
    println!("╠═══════════════════════════════════════╣");

    // Данные сделки
    let symbol = "BTC/USDT";
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;
    let fee_percent = 0.1;

    // Расчёты
    let entry_value = entry_price * quantity;
    let exit_value = exit_price * quantity;

    let gross_profit = calculate_profit(entry_price, exit_price, quantity);
    let profit_percent = calculate_profit_percent(entry_price, exit_price);

    let entry_fee = calculate_fee(entry_value, fee_percent);
    let exit_fee = calculate_fee(exit_value, fee_percent);
    let total_fees = entry_fee + exit_fee;

    let net_profit = gross_profit - total_fees;
    let roi = calculate_roi_from_values(net_profit, entry_value);

    let status = get_trade_status(net_profit);

    // Вывод результатов
    println!("║ Пара:           {:>20} ║", symbol);
    println!("║ Цена входа:     ${:>18.2} ║", entry_price);
    println!("║ Цена выхода:    ${:>18.2} ║", exit_price);
    println!("║ Количество:     {:>19.4} ║", quantity);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Объём входа:    ${:>18.2} ║", entry_value);
    println!("║ Объём выхода:   ${:>18.2} ║", exit_value);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Валовая прибыль:${:>18.2} ║", gross_profit);
    println!("║ Изменение цены: {:>18.2}% ║", profit_percent);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Комиссия входа: ${:>18.2} ║", entry_fee);
    println!("║ Комиссия выхода:${:>18.2} ║", exit_fee);
    println!("║ Всего комиссий: ${:>18.2} ║", total_fees);
    println!("╠═══════════════════════════════════════╣");
    println!("║ ЧИСТАЯ ПРИБЫЛЬ: ${:>18.2} ║", net_profit);
    println!("║ ROI:            {:>18.2}% ║", roi);
    println!("║ Статус:         {:>20} ║", status);
    println!("╚═══════════════════════════════════════╝");
}

fn calculate_profit(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity
}

fn calculate_profit_percent(entry: f64, exit: f64) -> f64 {
    if entry == 0.0 { return 0.0; }
    ((exit - entry) / entry) * 100.0
}

fn calculate_fee(value: f64, fee_percent: f64) -> f64 {
    value * (fee_percent / 100.0)
}

fn calculate_roi_from_values(net_profit: f64, investment: f64) -> f64 {
    if investment == 0.0 { return 0.0; }
    (net_profit / investment) * 100.0
}

fn get_trade_status(profit: f64) -> &'static str {
    if profit > 0.0 {
        "PROFIT"
    } else if profit < 0.0 {
        "LOSS"
    } else {
        "BREAKEVEN"
    }
}
```

## Анализ нескольких сделок

```rust
fn main() {
    // Массив сделок: (entry, exit, quantity)
    let trades = [
        (42000.0, 43500.0, 0.5),   // BTC: прибыль
        (2500.0, 2400.0, 2.0),     // ETH: убыток
        (95.0, 110.0, 10.0),      // SOL: прибыль
        (0.65, 0.58, 1000.0),     // XRP: убыток
        (28000.0, 29500.0, 0.3),  // BTC: прибыль
    ];

    let fee_pct = 0.1;
    let mut total_profit = 0.0;
    let mut winning_trades = 0;
    let mut losing_trades = 0;

    println!("╔═════════════════════════════════════════╗");
    println!("║         PORTFOLIO ANALYSIS              ║");
    println!("╠═════════════════════════════════════════╣");

    for (i, &(entry, exit, qty)) in trades.iter().enumerate() {
        let gross = (exit - entry) * qty;
        let fees = (entry + exit) * qty * (fee_pct / 100.0);
        let net = gross - fees;

        total_profit += net;

        if net > 0.0 {
            winning_trades += 1;
        } else if net < 0.0 {
            losing_trades += 1;
        }

        let status = if net > 0.0 { "+" } else { "-" };
        println!("║ Trade #{}: {} ${:>10.2}              ║", i + 1, status, net.abs());
    }

    let total_trades = trades.len();
    let win_rate = (winning_trades as f64 / total_trades as f64) * 100.0;

    println!("╠═════════════════════════════════════════╣");
    println!("║ Всего сделок:    {:>5}                  ║", total_trades);
    println!("║ Прибыльных:      {:>5}                  ║", winning_trades);
    println!("║ Убыточных:       {:>5}                  ║", losing_trades);
    println!("║ Win Rate:        {:>5.1}%                 ║", win_rate);
    println!("╠═════════════════════════════════════════╣");
    let status = if total_profit > 0.0 { "PROFIT" } else { "LOSS" };
    println!("║ ИТОГО:           ${:>10.2} ({})   ║", total_profit, status);
    println!("╚═════════════════════════════════════════╝");
}
```

## Практические упражнения

### Упражнение 1: Расчёт точки безубыточности

Найдите цену выхода, при которой сделка покроет комиссии:

```rust
fn main() {
    let entry = 42000.0;
    let quantity = 0.5;
    let fee_pct = 0.1;

    let breakeven = calculate_breakeven(entry, quantity, fee_pct);
    println!("Точка безубыточности: ${:.2}", breakeven);
}

fn calculate_breakeven(entry: f64, quantity: f64, fee_pct: f64) -> f64 {
    // entry_fee = entry * qty * fee_pct / 100
    // exit_fee = exit * qty * fee_pct / 100
    // profit = (exit - entry) * qty
    // net = profit - entry_fee - exit_fee = 0
    // Решаем уравнение для exit

    let fee_rate = fee_pct / 100.0;
    let numerator = entry * (1.0 + fee_rate);
    let denominator = 1.0 - fee_rate;

    numerator / denominator
}
```

### Упражнение 2: Сравнение длинной и короткой позиции

```rust
fn main() {
    let price1 = 42000.0;
    let price2 = 43500.0;
    let quantity = 0.5;

    let long_profit = calculate_long_profit(price1, price2, quantity);
    let short_profit = calculate_short_profit(price1, price2, quantity);

    println!("Long (buy at {}, sell at {}): ${:.2}", price1, price2, long_profit);
    println!("Short (sell at {}, buy at {}): ${:.2}", price1, price2, short_profit);
}

fn calculate_long_profit(entry: f64, exit: f64, qty: f64) -> f64 {
    (exit - entry) * qty
}

fn calculate_short_profit(entry: f64, exit: f64, qty: f64) -> f64 {
    (entry - exit) * qty  // В шорте прибыль, когда цена падает
}
```

### Упражнение 3: Калькулятор с учётом проскальзывания

```rust
fn main() {
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;
    let slippage_pct = 0.05;  // 0.05% проскальзывание

    let (actual_entry, actual_exit, profit) =
        calculate_with_slippage(entry, exit, quantity, slippage_pct);

    println!("Ожидаемый вход: ${:.2}", entry);
    println!("Фактический вход: ${:.2}", actual_entry);
    println!("Ожидаемый выход: ${:.2}", exit);
    println!("Фактический выход: ${:.2}", actual_exit);
    println!("Прибыль с учётом проскальзывания: ${:.2}", profit);
}

fn calculate_with_slippage(
    entry: f64,
    exit: f64,
    quantity: f64,
    slippage_pct: f64
) -> (f64, f64, f64) {
    let slippage_rate = slippage_pct / 100.0;

    // При покупке цена выше (проскальзывание вверх)
    let actual_entry = entry * (1.0 + slippage_rate);

    // При продаже цена ниже (проскальзывание вниз)
    let actual_exit = exit * (1.0 - slippage_rate);

    let profit = (actual_exit - actual_entry) * quantity;

    (actual_entry, actual_exit, profit)
}
```

### Упражнение 4: Расчёт прибыли в процентах от портфеля

```rust
fn main() {
    let portfolio_value = 10000.0;  // Общий размер портфеля
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.1;  // Размер позиции

    let profit = (exit - entry) * quantity;
    let position_value = entry * quantity;
    let portfolio_impact = (profit / portfolio_value) * 100.0;
    let position_size_pct = (position_value / portfolio_value) * 100.0;

    println!("Размер портфеля: ${:.2}", portfolio_value);
    println!("Размер позиции: ${:.2} ({:.1}% портфеля)", position_value, position_size_pct);
    println!("Прибыль: ${:.2}", profit);
    println!("Влияние на портфель: {:.2}%", portfolio_impact);
}
```

## Что мы узнали

| Концепция | Формула | Пример |
|-----------|---------|--------|
| Прибыль | (exit - entry) × qty | (43500 - 42000) × 0.5 = $750 |
| Прибыль % | ((exit - entry) / entry) × 100 | 3.57% |
| Комиссия | value × fee% / 100 | 21000 × 0.1% = $21 |
| ROI | (net_profit / investment) × 100 | 3.47% |
| Win Rate | wins / total × 100 | 60% |

## Домашнее задание

1. **Калькулятор с плечом**: Создай функцию `calculate_leveraged_pnl(entry, exit, quantity, leverage) -> f64`, которая учитывает кредитное плечо.

2. **Risk/Reward калькулятор**: Напиши функцию `calculate_risk_reward(entry, stop_loss, take_profit) -> f64`, которая возвращает соотношение риска к прибыли.

3. **Средняя цена входа**: Реализуй функцию `calculate_average_entry(entries: &[(f64, f64)]) -> f64`, которая считает среднюю цену входа при нескольких покупках (цена, количество).

4. **Симулятор серии сделок**: Создай программу, которая симулирует 10 сделок со случайными результатами и выводит общую статистику (total PnL, win rate, max drawdown).

## Навигация

[← Предыдущий день](../029-input-validation/ru.md) | [Следующий день →](../031-position-sizing/ru.md)
