# День 31: Проект — Калькулятор размера позиции

## Обзор проекта

**Position Size Calculator** — один из важнейших инструментов для трейдера. Правильный расчёт размера позиции помогает:
- Контролировать риск каждой сделки
- Защитить капитал от чрезмерных потерь
- Следовать торговой стратегии дисциплинированно

В этом проекте мы объединим всё, что изучили: переменные, типы данных, функции, структуры и обработку ошибок.

## Формула расчёта размера позиции

Классическая формула риск-менеджмента:

```
Position Size = Risk Amount / Risk Per Unit
             = (Account Balance × Risk %) / |Entry Price - Stop Loss|
```

**Пример:**
- Баланс: $10,000
- Риск на сделку: 2%
- Цена входа: $100
- Stop Loss: $95

```
Position Size = (10000 × 0.02) / |100 - 95| = 200 / 5 = 40 акций
```

## Базовая версия калькулятора

```rust
fn main() {
    // Входные данные
    let account_balance = 10000.0;  // $10,000
    let risk_percent = 2.0;          // 2%
    let entry_price = 100.0;         // $100
    let stop_loss = 95.0;            // $95

    // Расчёт
    let risk_amount = account_balance * (risk_percent / 100.0);
    let risk_per_share = (entry_price - stop_loss).abs();
    let position_size = risk_amount / risk_per_share;

    println!("╔═══════════════════════════════════════╗");
    println!("║     POSITION SIZE CALCULATOR          ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Account Balance: ${:>18.2} ║", account_balance);
    println!("║ Risk Percent:    {:>18.1}% ║", risk_percent);
    println!("║ Entry Price:     ${:>18.2} ║", entry_price);
    println!("║ Stop Loss:       ${:>18.2} ║", stop_loss);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Risk Amount:     ${:>18.2} ║", risk_amount);
    println!("║ Risk Per Share:  ${:>18.2} ║", risk_per_share);
    println!("║ Position Size:   {:>19.2} ║", position_size);
    println!("╚═══════════════════════════════════════╝");
}
```

## Структура для параметров

```rust
struct TradeSetup {
    symbol: String,
    account_balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
    take_profit: Option<f64>,
}

struct PositionInfo {
    size: f64,
    risk_amount: f64,
    potential_loss: f64,
    potential_profit: Option<f64>,
    risk_reward_ratio: Option<f64>,
}

fn main() {
    let setup = TradeSetup {
        symbol: String::from("AAPL"),
        account_balance: 10000.0,
        risk_percent: 2.0,
        entry_price: 150.0,
        stop_loss: 145.0,
        take_profit: Some(165.0),
    };

    let position = calculate_position(&setup);
    print_position_info(&setup, &position);
}

fn calculate_position(setup: &TradeSetup) -> PositionInfo {
    let risk_amount = setup.account_balance * (setup.risk_percent / 100.0);
    let risk_per_unit = (setup.entry_price - setup.stop_loss).abs();
    let size = risk_amount / risk_per_unit;
    let potential_loss = size * risk_per_unit;

    let (potential_profit, risk_reward_ratio) = match setup.take_profit {
        Some(tp) => {
            let profit = size * (tp - setup.entry_price).abs();
            let rr = profit / potential_loss;
            (Some(profit), Some(rr))
        }
        None => (None, None),
    };

    PositionInfo {
        size,
        risk_amount,
        potential_loss,
        potential_profit,
        risk_reward_ratio,
    }
}

fn print_position_info(setup: &TradeSetup, pos: &PositionInfo) {
    println!("\n╔═══════════════════════════════════════╗");
    println!("║   POSITION CALCULATOR: {:>14} ║", setup.symbol);
    println!("╠═══════════════════════════════════════╣");
    println!("║ Account:     ${:>22.2} ║", setup.account_balance);
    println!("║ Risk:        {:>22.1}% ║", setup.risk_percent);
    println!("║ Entry:       ${:>22.2} ║", setup.entry_price);
    println!("║ Stop Loss:   ${:>22.2} ║", setup.stop_loss);
    if let Some(tp) = setup.take_profit {
        println!("║ Take Profit: ${:>22.2} ║", tp);
    }
    println!("╠═══════════════════════════════════════╣");
    println!("║ POSITION SIZE: {:>21.2} ║", pos.size);
    println!("║ Risk Amount:   ${:>20.2} ║", pos.risk_amount);
    println!("║ Max Loss:      ${:>20.2} ║", pos.potential_loss);
    if let Some(profit) = pos.potential_profit {
        println!("║ Max Profit:    ${:>20.2} ║", profit);
    }
    if let Some(rr) = pos.risk_reward_ratio {
        println!("║ Risk/Reward:   {:>21.2} ║", rr);
    }
    println!("╚═══════════════════════════════════════╝");
}
```

## Валидация входных данных

```rust
fn validate_trade_setup(setup: &TradeSetup) -> Result<(), String> {
    if setup.account_balance <= 0.0 {
        return Err(String::from("Account balance must be positive"));
    }

    if setup.risk_percent <= 0.0 || setup.risk_percent > 100.0 {
        return Err(format!(
            "Risk percent must be between 0 and 100, got {}",
            setup.risk_percent
        ));
    }

    if setup.entry_price <= 0.0 {
        return Err(String::from("Entry price must be positive"));
    }

    if setup.stop_loss <= 0.0 {
        return Err(String::from("Stop loss must be positive"));
    }

    if setup.entry_price == setup.stop_loss {
        return Err(String::from("Entry price and stop loss cannot be equal"));
    }

    if let Some(tp) = setup.take_profit {
        if tp <= 0.0 {
            return Err(String::from("Take profit must be positive"));
        }

        // Проверка логики: для лонга TP > Entry > SL
        let is_long = setup.entry_price > setup.stop_loss;
        if is_long && tp <= setup.entry_price {
            return Err(String::from("For long: take profit must be above entry"));
        }
        if !is_long && tp >= setup.entry_price {
            return Err(String::from("For short: take profit must be below entry"));
        }
    }

    Ok(())
}

fn main() {
    let setup = TradeSetup {
        symbol: String::from("BTCUSD"),
        account_balance: 50000.0,
        risk_percent: 1.0,
        entry_price: 42000.0,
        stop_loss: 40000.0,
        take_profit: Some(46000.0),
    };

    match validate_trade_setup(&setup) {
        Ok(()) => {
            let position = calculate_position(&setup);
            print_position_info(&setup, &position);
        }
        Err(e) => {
            println!("❌ Validation Error: {}", e);
        }
    }
}
```

## Поддержка Long и Short позиций

```rust
#[derive(Debug, Clone, Copy)]
enum TradeDirection {
    Long,
    Short,
}

struct TradeSetup {
    symbol: String,
    direction: TradeDirection,
    account_balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
    take_profit: Option<f64>,
}

impl TradeSetup {
    fn new_long(symbol: &str, balance: f64, risk: f64, entry: f64, sl: f64, tp: Option<f64>) -> Self {
        TradeSetup {
            symbol: String::from(symbol),
            direction: TradeDirection::Long,
            account_balance: balance,
            risk_percent: risk,
            entry_price: entry,
            stop_loss: sl,
            take_profit: tp,
        }
    }

    fn new_short(symbol: &str, balance: f64, risk: f64, entry: f64, sl: f64, tp: Option<f64>) -> Self {
        TradeSetup {
            symbol: String::from(symbol),
            direction: TradeDirection::Short,
            account_balance: balance,
            risk_percent: risk,
            entry_price: entry,
            stop_loss: sl,
            take_profit: tp,
        }
    }
}

fn detect_direction(entry: f64, stop_loss: f64) -> TradeDirection {
    if entry > stop_loss {
        TradeDirection::Long
    } else {
        TradeDirection::Short
    }
}

fn main() {
    // Long позиция на BTC
    let long_trade = TradeSetup::new_long(
        "BTCUSD",
        50000.0,
        1.0,
        42000.0,
        40000.0,
        Some(46000.0)
    );

    // Short позиция на ETH
    let short_trade = TradeSetup::new_short(
        "ETHUSD",
        50000.0,
        1.5,
        2200.0,
        2400.0,
        Some(1800.0)
    );

    for setup in [&long_trade, &short_trade] {
        if let Ok(()) = validate_trade_setup(setup) {
            let position = calculate_position(setup);
            print_position_info(setup, &position);
        }
    }
}
```

## Расширенные метрики

```rust
struct ExtendedPositionInfo {
    size: f64,
    total_value: f64,
    risk_amount: f64,
    potential_loss: f64,
    potential_profit: Option<f64>,
    risk_reward_ratio: Option<f64>,
    percent_of_account: f64,
    leverage_required: Option<f64>,
}

fn calculate_extended_position(setup: &TradeSetup) -> ExtendedPositionInfo {
    let risk_amount = setup.account_balance * (setup.risk_percent / 100.0);
    let risk_per_unit = (setup.entry_price - setup.stop_loss).abs();
    let size = risk_amount / risk_per_unit;
    let total_value = size * setup.entry_price;
    let potential_loss = size * risk_per_unit;
    let percent_of_account = (total_value / setup.account_balance) * 100.0;

    // Кредитное плечо (если позиция больше баланса)
    let leverage_required = if total_value > setup.account_balance {
        Some(total_value / setup.account_balance)
    } else {
        None
    };

    let (potential_profit, risk_reward_ratio) = match setup.take_profit {
        Some(tp) => {
            let profit_per_unit = (tp - setup.entry_price).abs();
            let profit = size * profit_per_unit;
            let rr = profit_per_unit / risk_per_unit;
            (Some(profit), Some(rr))
        }
        None => (None, None),
    };

    ExtendedPositionInfo {
        size,
        total_value,
        risk_amount,
        potential_loss,
        potential_profit,
        risk_reward_ratio,
        percent_of_account,
        leverage_required,
    }
}

fn print_extended_info(setup: &TradeSetup, pos: &ExtendedPositionInfo) {
    let direction = match setup.direction {
        TradeDirection::Long => "LONG",
        TradeDirection::Short => "SHORT",
    };

    println!("\n╔════════════════════════════════════════════╗");
    println!("║  {} {} POSITION", direction, setup.symbol);
    println!("╠════════════════════════════════════════════╣");
    println!("║ Position Size:    {:>22.4} ║", pos.size);
    println!("║ Total Value:      ${:>21.2} ║", pos.total_value);
    println!("║ % of Account:     {:>21.2}% ║", pos.percent_of_account);

    if let Some(lev) = pos.leverage_required {
        println!("║ Leverage Required:{:>22.1}x ║", lev);
    }

    println!("╠════════════════════════════════════════════╣");
    println!("║ Risk Amount:      ${:>21.2} ║", pos.risk_amount);
    println!("║ Potential Loss:   ${:>21.2} ║", pos.potential_loss);

    if let Some(profit) = pos.potential_profit {
        println!("║ Potential Profit: ${:>21.2} ║", profit);
    }

    if let Some(rr) = pos.risk_reward_ratio {
        let rr_status = if rr >= 2.0 { "Good" } else if rr >= 1.0 { "Fair" } else { "Poor" };
        println!("║ Risk/Reward:      {:>16.2} ({}) ║", rr, rr_status);
    }

    println!("╚════════════════════════════════════════════╝");
}
```

## Пакетный расчёт для нескольких сделок

```rust
fn main() {
    let account_balance = 100000.0;
    let risk_per_trade = 1.0;

    let trade_setups = vec![
        ("BTCUSD", 42000.0, 40000.0, Some(48000.0)),
        ("ETHUSD", 2200.0, 2100.0, Some(2500.0)),
        ("SOLUSD", 95.0, 90.0, Some(110.0)),
        ("AVAXUSD", 35.0, 32.0, Some(45.0)),
    ];

    println!("\n📊 PORTFOLIO POSITION SIZING");
    println!("Account Balance: ${:.2}", account_balance);
    println!("Risk per Trade: {}%", risk_per_trade);
    println!("{}", "═".repeat(50));

    let mut total_risk = 0.0;
    let mut total_value = 0.0;

    for (symbol, entry, sl, tp) in trade_setups {
        let setup = TradeSetup::new_long(
            symbol,
            account_balance,
            risk_per_trade,
            entry,
            sl,
            tp
        );

        let pos = calculate_extended_position(&setup);

        println!("\n{}: {:.4} units @ ${:.2}", symbol, pos.size, entry);
        println!("  Value: ${:.2} | Risk: ${:.2}", pos.total_value, pos.risk_amount);

        if let Some(rr) = pos.risk_reward_ratio {
            println!("  R/R: {:.2}", rr);
        }

        total_risk += pos.risk_amount;
        total_value += pos.total_value;
    }

    println!("\n{}", "═".repeat(50));
    println!("TOTAL RISK: ${:.2} ({:.2}% of account)",
        total_risk,
        (total_risk / account_balance) * 100.0
    );
    println!("TOTAL VALUE: ${:.2} ({:.2}% of account)",
        total_value,
        (total_value / account_balance) * 100.0
    );
}
```

## Полный проект

```rust
// position_calculator.rs

use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
enum TradeDirection {
    Long,
    Short,
}

struct TradeSetup {
    symbol: String,
    direction: TradeDirection,
    account_balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
    take_profit: Option<f64>,
}

struct PositionResult {
    size: f64,
    total_value: f64,
    risk_amount: f64,
    max_loss: f64,
    max_profit: Option<f64>,
    risk_reward: Option<f64>,
    percent_of_account: f64,
}

fn main() {
    println!("╔═══════════════════════════════════════╗");
    println!("║     POSITION SIZE CALCULATOR          ║");
    println!("║        for Algorithmic Trading        ║");
    println!("╚═══════════════════════════════════════╝\n");

    // Демо-режим с предустановленными значениями
    let setups = vec![
        create_setup("BTCUSD", 50000.0, 1.0, 43000.0, 41000.0, Some(47000.0)),
        create_setup("ETHUSD", 50000.0, 1.5, 2250.0, 2100.0, Some(2600.0)),
        create_setup("AAPL", 25000.0, 2.0, 185.0, 180.0, Some(200.0)),
    ];

    for setup in &setups {
        match validate(&setup) {
            Ok(()) => {
                let result = calculate(&setup);
                display(&setup, &result);
            }
            Err(e) => println!("Error for {}: {}\n", setup.symbol, e),
        }
    }

    // Сравнение рисков
    println!("\n📊 RISK COMPARISON");
    println!("{}", "─".repeat(60));
    println!("{:<10} {:>12} {:>12} {:>12} {:>12}",
        "Symbol", "Size", "Value", "Risk $", "R/R");
    println!("{}", "─".repeat(60));

    for setup in &setups {
        if let Ok(()) = validate(&setup) {
            let r = calculate(&setup);
            println!("{:<10} {:>12.4} {:>12.2} {:>12.2} {:>12.2}",
                setup.symbol,
                r.size,
                r.total_value,
                r.risk_amount,
                r.risk_reward.unwrap_or(0.0)
            );
        }
    }
}

fn create_setup(
    symbol: &str,
    balance: f64,
    risk: f64,
    entry: f64,
    sl: f64,
    tp: Option<f64>
) -> TradeSetup {
    let direction = if entry > sl {
        TradeDirection::Long
    } else {
        TradeDirection::Short
    };

    TradeSetup {
        symbol: String::from(symbol),
        direction,
        account_balance: balance,
        risk_percent: risk,
        entry_price: entry,
        stop_loss: sl,
        take_profit: tp,
    }
}

fn validate(setup: &TradeSetup) -> Result<(), String> {
    if setup.account_balance <= 0.0 {
        return Err(String::from("Account balance must be positive"));
    }
    if setup.risk_percent <= 0.0 || setup.risk_percent > 100.0 {
        return Err(String::from("Risk must be between 0-100%"));
    }
    if setup.entry_price <= 0.0 || setup.stop_loss <= 0.0 {
        return Err(String::from("Prices must be positive"));
    }
    if setup.entry_price == setup.stop_loss {
        return Err(String::from("Entry and SL cannot be equal"));
    }
    Ok(())
}

fn calculate(setup: &TradeSetup) -> PositionResult {
    let risk_amount = setup.account_balance * (setup.risk_percent / 100.0);
    let risk_per_unit = (setup.entry_price - setup.stop_loss).abs();
    let size = risk_amount / risk_per_unit;
    let total_value = size * setup.entry_price;
    let max_loss = size * risk_per_unit;
    let percent_of_account = (total_value / setup.account_balance) * 100.0;

    let (max_profit, risk_reward) = match setup.take_profit {
        Some(tp) => {
            let profit_per_unit = (tp - setup.entry_price).abs();
            let profit = size * profit_per_unit;
            let rr = profit_per_unit / risk_per_unit;
            (Some(profit), Some(rr))
        }
        None => (None, None),
    };

    PositionResult {
        size,
        total_value,
        risk_amount,
        max_loss,
        max_profit,
        risk_reward,
        percent_of_account,
    }
}

fn display(setup: &TradeSetup, result: &PositionResult) {
    let dir = match setup.direction {
        TradeDirection::Long => "LONG",
        TradeDirection::Short => "SHORT",
    };

    println!("┌─────────────────────────────────────────┐");
    println!("│ {} {} ", dir, setup.symbol);
    println!("├─────────────────────────────────────────┤");
    println!("│ Entry:        ${:>24.2} │", setup.entry_price);
    println!("│ Stop Loss:    ${:>24.2} │", setup.stop_loss);
    if let Some(tp) = setup.take_profit {
        println!("│ Take Profit:  ${:>24.2} │", tp);
    }
    println!("├─────────────────────────────────────────┤");
    println!("│ Position Size: {:>24.4} │", result.size);
    println!("│ Total Value:   ${:>23.2} │", result.total_value);
    println!("│ % of Account:  {:>23.1}% │", result.percent_of_account);
    println!("├─────────────────────────────────────────┤");
    println!("│ Risk Amount:   ${:>23.2} │", result.risk_amount);
    println!("│ Max Loss:      ${:>23.2} │", result.max_loss);
    if let Some(profit) = result.max_profit {
        println!("│ Max Profit:    ${:>23.2} │", profit);
    }
    if let Some(rr) = result.risk_reward {
        let quality = if rr >= 3.0 {
            "Excellent"
        } else if rr >= 2.0 {
            "Good"
        } else if rr >= 1.0 {
            "Fair"
        } else {
            "Poor"
        };
        println!("│ Risk/Reward:   {:>18.2} ({}) │", rr, quality);
    }
    println!("└─────────────────────────────────────────┘\n");
}
```

## Упражнения

### Упражнение 1: Калькулятор с пользовательским вводом

Добавь интерактивный ввод данных от пользователя:

```rust
use std::io::{self, Write};

fn read_f64(prompt: &str) -> f64 {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse::<f64>() {
            Ok(value) => return value,
            Err(_) => println!("Invalid number, try again"),
        }
    }
}

// Реализуй функцию main() с интерактивным вводом
```

### Упражнение 2: Kelly Criterion

Реализуй расчёт размера позиции по критерию Келли:

```
Kelly % = W - [(1-W) / R]
где:
  W = вероятность выигрыша
  R = соотношение средней прибыли к среднему убытку
```

### Упражнение 3: Несколько уровней Take Profit

Добавь поддержку нескольких уровней Take Profit с частичным закрытием:

```rust
struct MultiTargetSetup {
    symbol: String,
    entry: f64,
    stop_loss: f64,
    targets: Vec<(f64, f64)>,  // (price, percentage)
}

// Пример: закрыть 50% на TP1, 30% на TP2, 20% на TP3
```

### Упражнение 4: Расчёт для криптовалют с разными лотами

Добавь поддержку минимального размера лота и округления:

```rust
fn round_to_lot_size(size: f64, lot_step: f64) -> f64 {
    // Реализуй округление до ближайшего допустимого размера
}
```

## Домашнее задание

1. **Position Sizer CLI** — создай полноценное CLI-приложение с аргументами командной строки

2. **Risk Report** — добавь генерацию отчёта о рисках для портфеля из нескольких позиций

3. **Volatility Adjustment** — добавь корректировку размера позиции на основе ATR (Average True Range)

4. **Position Scaling** — реализуй расчёт для наращивания позиции (pyramiding)

## Что мы изучили

| Концепция | Применение |
|-----------|-----------|
| Структуры | Организация данных сделки |
| Функции | Модульные расчёты |
| Result | Обработка ошибок валидации |
| Option | Опциональные параметры (TP) |
| Enum | Направление сделки (Long/Short) |
| impl | Конструкторы для структур |

## Навигация

[← Предыдущий день](../030-project-trade-analyzer/ru.md) | [Следующий день →](../032-project-risk-calculator/ru.md)
