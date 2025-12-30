# День 15: Возврат значений — функция возвращает PnL

## Аналогия из трейдинга

Когда ты просишь аналитика рассчитать прибыль сделки, он возвращает тебе **результат** — число. Функции в программировании работают так же: получают данные, обрабатывают и **возвращают** результат.

## Базовый возврат значения

```rust
fn main() {
    let pnl = calculate_pnl(42000.0, 43500.0, 0.5);
    println!("PnL: ${:.2}", pnl);
}

fn calculate_pnl(entry: f64, exit: f64, quantity: f64) -> f64 {
    (exit - entry) * quantity  // Последнее выражение — возвращаемое значение
}
```

**Важно:** Нет точки с запятой в конце! Это выражение, а не инструкция.

## Явный return

Используется для раннего выхода:

```rust
fn main() {
    println!("Result: {}", safe_divide(10.0, 2.0));
    println!("Result: {}", safe_divide(10.0, 0.0));
}

fn safe_divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        return 0.0;  // Ранний выход
    }
    a / b  // Нормальный возврат
}
```

## Возврат разных типов

### Числа

```rust
fn calculate_profit(entry: f64, exit: f64, qty: f64) -> f64 {
    (exit - entry) * qty
}

fn count_trades(trades: &[f64]) -> usize {
    trades.len()
}
```

### Булевы значения

```rust
fn is_profitable(entry: f64, exit: f64) -> bool {
    exit > entry
}

fn is_valid_order(price: f64, quantity: f64, balance: f64) -> bool {
    price > 0.0 && quantity > 0.0 && price * quantity <= balance
}
```

### Строки

```rust
fn get_trade_status(pnl: f64) -> String {
    if pnl > 0.0 {
        String::from("PROFIT")
    } else if pnl < 0.0 {
        String::from("LOSS")
    } else {
        String::from("BREAKEVEN")
    }
}

fn format_price(price: f64) -> String {
    format!("${:.2}", price)
}
```

## Возврат кортежей

```rust
fn main() {
    let (min, max) = find_price_range(&[42000.0, 42500.0, 41800.0]);
    println!("Range: {} - {}", min, max);

    let (gross, fees, net) = calculate_trade_results(42000.0, 43500.0, 0.5, 0.1);
    println!("Gross: {}, Fees: {}, Net: {}", gross, fees, net);
}

fn find_price_range(prices: &[f64]) -> (f64, f64) {
    let mut min = f64::MAX;
    let mut max = f64::MIN;

    for &price in prices {
        if price < min { min = price; }
        if price > max { max = price; }
    }

    (min, max)
}

fn calculate_trade_results(entry: f64, exit: f64, qty: f64, fee_pct: f64) -> (f64, f64, f64) {
    let gross = (exit - entry) * qty;
    let fees = (entry + exit) * qty * (fee_pct / 100.0);
    let net = gross - fees;
    (gross, fees, net)
}
```

## Возврат Option — значение может отсутствовать

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0];

    match calculate_sma(&prices, 3) {
        Some(sma) => println!("SMA-3: {:.2}", sma),
        None => println!("Not enough data"),
    }

    match calculate_sma(&prices, 10) {
        Some(sma) => println!("SMA-10: {:.2}", sma),
        None => println!("Not enough data for SMA-10"),
    }
}

fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period {
        return None;  // Недостаточно данных
    }

    let slice = &prices[prices.len() - period..];
    let sum: f64 = slice.iter().sum();
    Some(sum / period as f64)
}
```

## Возврат Result — операция может провалиться

```rust
fn main() {
    match calculate_position_size(10000.0, 2.0, 42000.0, 42000.0) {
        Ok(size) => println!("Position size: {}", size),
        Err(e) => println!("Error: {}", e),
    }

    match calculate_position_size(10000.0, 2.0, 42000.0, 41000.0) {
        Ok(size) => println!("Position size: {}", size),
        Err(e) => println!("Error: {}", e),
    }
}

fn calculate_position_size(
    balance: f64,
    risk_pct: f64,
    entry: f64,
    stop_loss: f64
) -> Result<f64, String> {
    if balance <= 0.0 {
        return Err(String::from("Balance must be positive"));
    }
    if risk_pct <= 0.0 || risk_pct > 100.0 {
        return Err(String::from("Invalid risk percentage"));
    }

    let risk_per_unit = (entry - stop_loss).abs();
    if risk_per_unit == 0.0 {
        return Err(String::from("Entry and stop loss cannot be equal"));
    }

    let risk_amount = balance * (risk_pct / 100.0);
    Ok(risk_amount / risk_per_unit)
}
```

## Практический пример: полный анализ сделки

```rust
fn main() {
    let analysis = analyze_trade(42000.0, 43500.0, 0.5, 0.1);
    print_analysis(&analysis);
}

struct TradeAnalysis {
    gross_pnl: f64,
    fees: f64,
    net_pnl: f64,
    roi_percent: f64,
    is_profitable: bool,
    status: String,
}

fn analyze_trade(entry: f64, exit: f64, qty: f64, fee_pct: f64) -> TradeAnalysis {
    let gross_pnl = (exit - entry) * qty;
    let fees = (entry + exit) * qty * (fee_pct / 100.0);
    let net_pnl = gross_pnl - fees;
    let investment = entry * qty;
    let roi_percent = if investment > 0.0 {
        (net_pnl / investment) * 100.0
    } else {
        0.0
    };
    let is_profitable = net_pnl > 0.0;
    let status = if net_pnl > 0.0 {
        String::from("PROFIT")
    } else if net_pnl < 0.0 {
        String::from("LOSS")
    } else {
        String::from("BREAKEVEN")
    };

    TradeAnalysis {
        gross_pnl,
        fees,
        net_pnl,
        roi_percent,
        is_profitable,
        status,
    }
}

fn print_analysis(a: &TradeAnalysis) {
    println!("╔═══════════════════════════════╗");
    println!("║      TRADE ANALYSIS           ║");
    println!("╠═══════════════════════════════╣");
    println!("║ Gross PnL:   ${:>14.2} ║", a.gross_pnl);
    println!("║ Fees:        ${:>14.2} ║", a.fees);
    println!("║ Net PnL:     ${:>14.2} ║", a.net_pnl);
    println!("║ ROI:          {:>13.2}% ║", a.roi_percent);
    println!("║ Profitable:  {:>14} ║", a.is_profitable);
    println!("║ Status:      {:>14} ║", a.status);
    println!("╚═══════════════════════════════╝");
}
```

## Цепочки вызовов

```rust
fn main() {
    let prices = [42000.0, 42500.0, 41800.0, 42200.0, 42100.0];

    // Цепочка вызовов
    let result = prices
        .iter()
        .map(|p| normalize_price(*p, 40000.0, 45000.0))
        .collect::<Vec<f64>>();

    println!("Normalized: {:?}", result);
}

fn normalize_price(price: f64, min: f64, max: f64) -> f64 {
    (price - min) / (max - min)
}
```

## Возврат замыкания (продвинуто)

```rust
fn main() {
    let fee_calculator = create_fee_calculator(0.1);

    println!("Fee for $1000: ${:.2}", fee_calculator(1000.0));
    println!("Fee for $5000: ${:.2}", fee_calculator(5000.0));
}

fn create_fee_calculator(fee_percent: f64) -> impl Fn(f64) -> f64 {
    move |value: f64| value * (fee_percent / 100.0)
}
```

## Паттерны возврата

```rust
// 1. Простой расчёт
fn calculate_value(price: f64, qty: f64) -> f64 {
    price * qty
}

// 2. Булева проверка
fn is_long_position(entry: f64, current: f64) -> bool {
    current > entry
}

// 3. Множественные значения
fn get_price_stats(prices: &[f64]) -> (f64, f64, f64) {
    // (min, max, avg)
    let min = prices.iter().cloned().fold(f64::MAX, f64::min);
    let max = prices.iter().cloned().fold(f64::MIN, f64::max);
    let avg = prices.iter().sum::<f64>() / prices.len() as f64;
    (min, max, avg)
}

// 4. Опциональный результат
fn find_first_profitable(trades: &[f64]) -> Option<f64> {
    trades.iter().cloned().find(|&pnl| pnl > 0.0)
}

// 5. Результат или ошибка
fn parse_price(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Cannot parse '{}' as price", s))
}

fn main() {
    println!("Value: {}", calculate_value(42000.0, 0.5));
    println!("Is long: {}", is_long_position(42000.0, 43000.0));

    let prices = [42000.0, 42500.0, 41800.0];
    let (min, max, avg) = get_price_stats(&prices);
    println!("Stats: min={}, max={}, avg={:.2}", min, max, avg);

    let trades = [-100.0, -50.0, 200.0, 150.0];
    println!("First profit: {:?}", find_first_profitable(&trades));

    println!("Parse: {:?}", parse_price("42000.50"));
    println!("Parse: {:?}", parse_price("invalid"));
}
```

## Что мы узнали

| Возврат | Синтаксис | Когда использовать |
|---------|-----------|-------------------|
| Простой | `-> T { value }` | Всегда есть результат |
| Ранний | `return value;` | Выход по условию |
| Кортеж | `-> (T, U)` | Несколько значений |
| Option | `-> Option<T>` | Может не быть значения |
| Result | `-> Result<T, E>` | Может быть ошибка |

## Домашнее задание

1. Напиши функцию `calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> Option<f64>`

2. Создай функцию `validate_and_calculate_position(...) -> Result<PositionInfo, String>`, которая проверяет входные данные и возвращает структуру с информацией о позиции

3. Реализуй функцию `get_trade_recommendation(price: f64, sma: f64, rsi: f64) -> (&str, f64)`, которая возвращает рекомендацию (BUY/SELL/HOLD) и уверенность (0.0-1.0)

4. Напиши цепочку функций для полного анализа массива цен: нормализация -> расчёт доходностей -> статистика

## Навигация

[← Предыдущий день](../014-function-parameters/ru.md) | [Следующий день →](../016-comments-trading-logic/ru.md)
