# День 25: match — определяем тип ордера

## Аналогия из трейдинга

Когда ордер поступает в торговую систему, первым делом нужно определить его тип: **Market**, **Limit**, **Stop** или **Stop-Limit**. В зависимости от типа ордера система выполняет разные действия. Это как сортировщик на бирже — он смотрит на ордер и направляет его в нужный обработчик.

В Rust для такой "сортировки" используется выражение `match` — мощный инструмент сопоставления с образцом.

## Базовый синтаксис match

```rust
fn main() {
    let order_type = "limit";

    match order_type {
        "market" => println!("Исполнить немедленно по рыночной цене"),
        "limit" => println!("Поставить в стакан и ждать"),
        "stop" => println!("Активировать при достижении цены"),
        _ => println!("Неизвестный тип ордера"),
    }
}
```

**Важно:** `_` — это "wildcard", который ловит все остальные варианты. Rust требует, чтобы match был *исчерпывающим* — покрывал все возможные случаи.

## match возвращает значение

```rust
fn main() {
    let side = "buy";

    let direction = match side {
        "buy" => 1,
        "sell" => -1,
        _ => 0,
    };

    println!("Направление: {}", direction);

    // Используем для расчёта PnL
    let entry = 42000.0;
    let exit = 43500.0;
    let quantity = 0.5;

    let pnl = (exit - entry) * quantity * direction as f64;
    println!("PnL: ${:.2}", pnl);
}
```

## match с числами

```rust
fn main() {
    let rsi = 75;

    let signal = match rsi {
        0..=30 => "Перепроданность — сигнал на покупку",
        31..=70 => "Нейтральная зона — держим",
        71..=100 => "Перекупленность — сигнал на продажу",
        _ => "Некорректное значение RSI",
    };

    println!("RSI = {}: {}", rsi, signal);
}
```

## match с enum — идеальная пара

```rust
enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

fn main() {
    let order = OrderType::Limit;

    let description = match order {
        OrderType::Market => "Исполняется немедленно по лучшей цене",
        OrderType::Limit => "Исполняется по указанной цене или лучше",
        OrderType::Stop => "Становится рыночным при достижении стоп-цены",
        OrderType::StopLimit => "Становится лимитным при достижении стоп-цены",
    };

    println!("{}", description);
}
```

**Преимущество:** При использовании enum компилятор проверит, что вы обработали ВСЕ варианты. Если добавите новый тип ордера, код не скомпилируется, пока вы не обработаете его.

## match с данными внутри enum

```rust
enum OrderCommand {
    Buy { ticker: String, quantity: f64 },
    Sell { ticker: String, quantity: f64 },
    Cancel { order_id: u64 },
    ModifyPrice { order_id: u64, new_price: f64 },
}

fn main() {
    let command = OrderCommand::Buy {
        ticker: String::from("BTC"),
        quantity: 0.5,
    };

    match command {
        OrderCommand::Buy { ticker, quantity } => {
            println!("Покупаем {} {} единиц", ticker, quantity);
        }
        OrderCommand::Sell { ticker, quantity } => {
            println!("Продаём {} {} единиц", ticker, quantity);
        }
        OrderCommand::Cancel { order_id } => {
            println!("Отменяем ордер #{}", order_id);
        }
        OrderCommand::ModifyPrice { order_id, new_price } => {
            println!("Меняем цену ордера #{} на ${:.2}", order_id, new_price);
        }
    }
}
```

## Привязка значений с @

```rust
fn main() {
    let price_change_percent = 7.5;

    let alert = match price_change_percent {
        x @ 0.0..=2.0 => format!("Нормальное движение: {:.1}%", x),
        x @ 2.0..=5.0 => format!("Повышенная волатильность: {:.1}%", x),
        x @ 5.0..=10.0 => format!("ВНИМАНИЕ! Сильное движение: {:.1}%", x),
        x if x > 10.0 => format!("КРИТИЧНО! Экстремальное движение: {:.1}%", x),
        x => format!("Падение: {:.1}%", x),
    };

    println!("{}", alert);
}
```

## Guards — дополнительные условия

```rust
fn main() {
    let price = 42500.0;
    let volume = 1000000.0;

    let market_condition = match (price, volume) {
        (p, v) if p > 50000.0 && v > 500000.0 => "Бычий рынок с высоким объёмом",
        (p, v) if p > 50000.0 && v <= 500000.0 => "Бычий рынок с низким объёмом",
        (p, v) if p <= 50000.0 && v > 500000.0 => "Медвежий рынок с высоким объёмом",
        (p, v) if p <= 50000.0 && v <= 500000.0 => "Медвежий рынок с низким объёмом",
        _ => "Неопределённое состояние",
    };

    println!("Состояние рынка: {}", market_condition);
}
```

## match с Option

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0];
    let target_price = 42500.0;

    let found = prices.iter().find(|&&p| p == target_price);

    match found {
        Some(price) => println!("Найдена цена: ${:.2}", price),
        None => println!("Цена не найдена"),
    }
}

fn get_stop_loss(entry: f64, risk_percent: f64) -> Option<f64> {
    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return None;
    }
    Some(entry * (1.0 - risk_percent / 100.0))
}
```

## match с Result

```rust
fn main() {
    let order_result = execute_order("BTC", 0.5, 42000.0);

    match order_result {
        Ok(order_id) => println!("Ордер исполнен! ID: {}", order_id),
        Err(error) => println!("Ошибка: {}", error),
    }
}

fn execute_order(ticker: &str, quantity: f64, price: f64) -> Result<u64, String> {
    if quantity <= 0.0 {
        return Err(String::from("Количество должно быть положительным"));
    }
    if price <= 0.0 {
        return Err(String::from("Цена должна быть положительной"));
    }
    // Симуляция успешного исполнения
    Ok(12345)
}
```

## Практический пример: классификация сделок

```rust
enum TradeResult {
    Profit(f64),
    Loss(f64),
    Breakeven,
}

fn main() {
    let trades = [
        calculate_trade_result(42000.0, 43500.0, 0.5),
        calculate_trade_result(42000.0, 41000.0, 0.3),
        calculate_trade_result(42000.0, 42000.0, 1.0),
        calculate_trade_result(50000.0, 52000.0, 0.2),
    ];

    let mut total_profit = 0.0;
    let mut total_loss = 0.0;
    let mut win_count = 0;
    let mut loss_count = 0;

    for trade in &trades {
        match trade {
            TradeResult::Profit(amount) => {
                println!("Прибыль: ${:.2}", amount);
                total_profit += amount;
                win_count += 1;
            }
            TradeResult::Loss(amount) => {
                println!("Убыток: ${:.2}", amount);
                total_loss += amount;
                loss_count += 1;
            }
            TradeResult::Breakeven => {
                println!("Безубыток");
            }
        }
    }

    println!("\n=== Статистика ===");
    println!("Всего прибыль: ${:.2}", total_profit);
    println!("Всего убыток: ${:.2}", total_loss);
    println!("Чистый результат: ${:.2}", total_profit - total_loss);
    println!("Win rate: {:.1}%", (win_count as f64 / trades.len() as f64) * 100.0);
}

fn calculate_trade_result(entry: f64, exit: f64, quantity: f64) -> TradeResult {
    let pnl = (exit - entry) * quantity;

    match pnl {
        p if p > 0.0 => TradeResult::Profit(p),
        p if p < 0.0 => TradeResult::Loss(p.abs()),
        _ => TradeResult::Breakeven,
    }
}
```

## Пример: торговый сигнал

```rust
enum TrendDirection {
    Up,
    Down,
    Sideways,
}

enum Signal {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

fn main() {
    let trend = TrendDirection::Up;
    let rsi = 35;
    let price_above_sma = true;

    let signal = generate_signal(&trend, rsi, price_above_sma);

    let action = match signal {
        Signal::StrongBuy => "Открываем длинную позицию (полный размер)",
        Signal::Buy => "Открываем длинную позицию (половина размера)",
        Signal::Hold => "Ждём лучшей точки входа",
        Signal::Sell => "Закрываем часть позиции",
        Signal::StrongSell => "Закрываем всю позицию",
    };

    println!("Действие: {}", action);
}

fn generate_signal(trend: &TrendDirection, rsi: i32, price_above_sma: bool) -> Signal {
    match (trend, rsi, price_above_sma) {
        (TrendDirection::Up, r, true) if r < 30 => Signal::StrongBuy,
        (TrendDirection::Up, r, true) if r < 50 => Signal::Buy,
        (TrendDirection::Up, _, false) => Signal::Hold,
        (TrendDirection::Down, r, false) if r > 70 => Signal::StrongSell,
        (TrendDirection::Down, r, false) if r > 50 => Signal::Sell,
        (TrendDirection::Down, _, true) => Signal::Hold,
        (TrendDirection::Sideways, _, _) => Signal::Hold,
        _ => Signal::Hold,
    }
}
```

## if let — упрощённый match для одного варианта

```rust
fn main() {
    let order_status = Some("filled");

    // Вместо полного match:
    // match order_status {
    //     Some(status) => println!("Статус: {}", status),
    //     None => {},
    // }

    // Используем if let:
    if let Some(status) = order_status {
        println!("Статус ордера: {}", status);
    }

    // С else
    let price: Option<f64> = None;

    if let Some(p) = price {
        println!("Цена: ${:.2}", p);
    } else {
        println!("Цена недоступна");
    }
}
```

## let else — ранний выход

```rust
fn process_trade(trade_data: Option<(f64, f64, f64)>) {
    let Some((entry, exit, qty)) = trade_data else {
        println!("Нет данных о сделке");
        return;
    };

    let pnl = (exit - entry) * qty;
    println!("PnL: ${:.2}", pnl);
}

fn main() {
    process_trade(Some((42000.0, 43500.0, 0.5)));
    process_trade(None);
}
```

## Что мы узнали

| Конструкция | Использование |
|-------------|--------------|
| `match value { ... }` | Полное сопоставление с образцом |
| `_` | Wildcard — ловит все остальные варианты |
| `x @ pattern` | Привязка значения к переменной |
| `if guard` | Дополнительное условие в ветке |
| `if let` | Упрощённый match для одного варианта |
| `let else` | Ранний выход если не совпало |

## Домашнее задание

1. Создай enum `OrderStatus` с вариантами: `Pending`, `PartiallyFilled(f64)`, `Filled`, `Cancelled`, `Rejected(String)`. Напиши функцию, которая с помощью match выводит информацию о статусе.

2. Реализуй функцию `classify_market_move(change_percent: f64) -> &'static str`, которая классифицирует изменение цены:
   - < -5%: "Crash"
   - -5% до -2%: "Decline"
   - -2% до 2%: "Stable"
   - 2% до 5%: "Rally"
   - > 5%: "Moon"

3. Напиши функцию `get_position_action(current: f64, target: f64, stop: f64, price: f64) -> String`, которая определяет действие:
   - Если цена >= target: "Take Profit"
   - Если цена <= stop: "Stop Loss"
   - Если цена > current: "In Profit"
   - Если цена < current: "In Loss"
   - Иначе: "At Entry"

4. Создай систему определения комиссии на основе объёма торгов (match с диапазонами):
   - До $10,000: 0.1%
   - $10,000 - $50,000: 0.08%
   - $50,000 - $100,000: 0.06%
   - Более $100,000: 0.04%

## Навигация

[← Предыдущий день](../024-continue-skip-losing-trades/ru.md) | [Следующий день →](../026-constants-fixed-exchange-fee/ru.md)
