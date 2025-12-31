# День 17: Условные выражения if — принятие торговых решений

## Аналогия из трейдинга

Трейдинг — это постоянное принятие решений на основе условий:
- **ЕСЛИ** цена выше скользящей средней, **ТО** рассматриваем покупку
- **ЕСЛИ** сработал стоп-лосс, **ТО** закрываем позицию
- **ЕСЛИ** RSI > 70, **ТО** рынок перекуплен, **ИНАЧЕ ЕСЛИ** RSI < 30, **ТО** перепродан, **ИНАЧЕ** нейтрально

В Rust это реализуется через конструкцию `if-else`.

## Базовый синтаксис if

```rust
fn main() {
    let btc_price = 42000.0;
    let entry_price = 41000.0;

    if btc_price > entry_price {
        println!("Позиция в прибыли!");
    }
}
```

**Важно:** Условие должно быть типа `bool`. В Rust нет неявного преобразования чисел в булевы значения.

```rust
fn main() {
    let balance = 1000.0;

    // Это НЕ сработает:
    // if balance { }  // Ошибка! f64 — не bool

    // Правильно:
    if balance > 0.0 {
        println!("На счету есть средства");
    }
}
```

## if-else — два варианта

```rust
fn main() {
    let current_price = 42000.0;
    let entry_price = 43000.0;

    if current_price > entry_price {
        println!("PROFIT: Цена выше входа");
    } else {
        println!("LOSS: Цена ниже или равна входу");
    }
}
```

### Применение в трейдинге: определение направления позиции

```rust
fn main() {
    let pnl = -150.0;

    if pnl >= 0.0 {
        println!("Сделка прибыльная: +${:.2}", pnl);
    } else {
        println!("Сделка убыточная: ${:.2}", pnl);
    }
}
```

## if-else if-else — множественные условия

```rust
fn main() {
    let rsi = 75.0;

    if rsi > 70.0 {
        println!("Перекуплено — возможен разворот вниз");
    } else if rsi < 30.0 {
        println!("Перепродано — возможен разворот вверх");
    } else {
        println!("Нейтральная зона");
    }
}
```

### Определение силы тренда

```rust
fn main() {
    let price_change_percent = 3.5;

    if price_change_percent > 5.0 {
        println!("Сильный рост!");
    } else if price_change_percent > 2.0 {
        println!("Умеренный рост");
    } else if price_change_percent > 0.0 {
        println!("Слабый рост");
    } else if price_change_percent > -2.0 {
        println!("Слабое падение");
    } else if price_change_percent > -5.0 {
        println!("Умеренное падение");
    } else {
        println!("Сильное падение!");
    }
}
```

## if как выражение

В Rust `if` — это выражение, которое возвращает значение:

```rust
fn main() {
    let price = 42000.0;
    let support = 41000.0;

    let signal = if price > support {
        "BUY"
    } else {
        "WAIT"
    };

    println!("Сигнал: {}", signal);
}
```

**Важно:**
- Обе ветки должны возвращать один тип
- Нет точки с запятой после значений внутри веток
- Точка с запятой в конце всего выражения

### Расчёт комиссии с учётом уровня

```rust
fn main() {
    let trade_volume = 50000.0;

    let fee_percent = if trade_volume > 100000.0 {
        0.05  // VIP: 0.05%
    } else if trade_volume > 10000.0 {
        0.08  // Средний уровень: 0.08%
    } else {
        0.1   // Базовый: 0.1%
    };

    let fee = trade_volume * (fee_percent / 100.0);
    println!("Объём: ${:.2}, Комиссия: {:.2}%, Сумма: ${:.2}",
             trade_volume, fee_percent, fee);
}
```

## Вложенные условия

```rust
fn main() {
    let has_position = true;
    let current_price = 43000.0;
    let entry_price = 42000.0;
    let stop_loss = 41000.0;
    let take_profit = 45000.0;

    if has_position {
        if current_price <= stop_loss {
            println!("СТОП-ЛОСС! Закрываем с убытком");
        } else if current_price >= take_profit {
            println!("ТЕЙК-ПРОФИТ! Фиксируем прибыль");
        } else if current_price > entry_price {
            println!("Позиция в прибыли, держим");
        } else {
            println!("Позиция в убытке, но стоп не сработал");
        }
    } else {
        println!("Нет открытых позиций");
    }
}
```

### Альтернатива: комбинирование условий

```rust
fn main() {
    let has_position = true;
    let current_price = 43000.0;
    let stop_loss = 41000.0;
    let take_profit = 45000.0;

    // Вместо вложенных if можно использовать &&
    if has_position && current_price <= stop_loss {
        println!("Закрываем по стоп-лоссу");
    } else if has_position && current_price >= take_profit {
        println!("Закрываем по тейк-профиту");
    }
}
```

## Практический пример: торговый сигнал

```rust
fn main() {
    // Рыночные данные
    let current_price = 42500.0;
    let sma_20 = 42000.0;
    let sma_50 = 41500.0;
    let rsi = 55.0;
    let volume = 1500.0;
    let avg_volume = 1000.0;

    // Анализ тренда
    let trend = if sma_20 > sma_50 {
        "UPTREND"
    } else if sma_20 < sma_50 {
        "DOWNTREND"
    } else {
        "SIDEWAYS"
    };

    // Анализ RSI
    let rsi_signal = if rsi > 70.0 {
        "OVERBOUGHT"
    } else if rsi < 30.0 {
        "OVERSOLD"
    } else {
        "NEUTRAL"
    };

    // Анализ объёма
    let volume_signal = if volume > avg_volume * 1.5 {
        "HIGH"
    } else if volume < avg_volume * 0.5 {
        "LOW"
    } else {
        "NORMAL"
    };

    // Итоговый сигнал
    let action = if trend == "UPTREND" && rsi_signal != "OVERBOUGHT" && volume_signal == "HIGH" {
        "STRONG BUY"
    } else if trend == "UPTREND" && rsi_signal != "OVERBOUGHT" {
        "BUY"
    } else if trend == "DOWNTREND" && rsi_signal != "OVERSOLD" {
        "SELL"
    } else {
        "HOLD"
    };

    println!("╔════════════════════════════════════╗");
    println!("║       TRADING SIGNAL ANALYSIS      ║");
    println!("╠════════════════════════════════════╣");
    println!("║ Price:        ${:>18.2} ║", current_price);
    println!("║ SMA-20:       ${:>18.2} ║", sma_20);
    println!("║ SMA-50:       ${:>18.2} ║", sma_50);
    println!("║ RSI:           {:>18.1} ║", rsi);
    println!("╠════════════════════════════════════╣");
    println!("║ Trend:         {:>18} ║", trend);
    println!("║ RSI Signal:    {:>18} ║", rsi_signal);
    println!("║ Volume:        {:>18} ║", volume_signal);
    println!("╠════════════════════════════════════╣");
    println!("║ >>> ACTION:    {:>18} ║", action);
    println!("╚════════════════════════════════════╝");
}
```

## Практический пример: валидация ордера

```rust
fn main() {
    let order_type = "LIMIT";
    let side = "BUY";
    let price = 42000.0;
    let quantity = 0.5;
    let balance = 25000.0;
    let market_open = true;

    let order_value = price * quantity;

    // Валидация
    let is_valid = if !market_open {
        println!("Ошибка: Рынок закрыт");
        false
    } else if price <= 0.0 {
        println!("Ошибка: Некорректная цена");
        false
    } else if quantity <= 0.0 {
        println!("Ошибка: Некорректное количество");
        false
    } else if side == "BUY" && order_value > balance {
        println!("Ошибка: Недостаточно средств (нужно ${:.2}, есть ${:.2})",
                 order_value, balance);
        false
    } else if order_type != "LIMIT" && order_type != "MARKET" {
        println!("Ошибка: Неизвестный тип ордера");
        false
    } else {
        println!("Ордер прошёл валидацию");
        true
    };

    if is_valid {
        println!("\nОрдер принят:");
        println!("  Тип: {}", order_type);
        println!("  Сторона: {}", side);
        println!("  Цена: ${:.2}", price);
        println!("  Количество: {}", quantity);
        println!("  Сумма: ${:.2}", order_value);
    }
}
```

## Практический пример: расчёт размера позиции

```rust
fn main() {
    let balance = 10000.0;
    let risk_percent = 2.0;
    let entry_price = 42000.0;
    let stop_loss = 40000.0;

    // Проверка входных данных
    let position_size = if balance <= 0.0 {
        println!("Ошибка: Нулевой баланс");
        0.0
    } else if risk_percent <= 0.0 || risk_percent > 100.0 {
        println!("Ошибка: Некорректный процент риска");
        0.0
    } else if stop_loss >= entry_price {
        println!("Ошибка: Стоп-лосс должен быть ниже цены входа для лонга");
        0.0
    } else {
        let risk_amount = balance * (risk_percent / 100.0);
        let risk_per_unit = entry_price - stop_loss;
        let size = risk_amount / risk_per_unit;

        println!("Риск на сделку: ${:.2}", risk_amount);
        println!("Риск на единицу: ${:.2}", risk_per_unit);

        size
    };

    if position_size > 0.0 {
        let position_value = position_size * entry_price;
        println!("\n=== Размер позиции ===");
        println!("Количество: {:.6} BTC", position_size);
        println!("Стоимость: ${:.2}", position_value);
        println!("Леверидж: {:.1}x", position_value / balance);
    }
}
```

## Паттерны использования if

### 1. Ранний выход

```rust
fn calculate_pnl(entry: f64, exit: f64, quantity: f64) -> f64 {
    if quantity == 0.0 {
        return 0.0;  // Ранний выход
    }
    (exit - entry) * quantity
}

fn main() {
    println!("PnL: {}", calculate_pnl(42000.0, 43000.0, 0.5));
    println!("PnL: {}", calculate_pnl(42000.0, 43000.0, 0.0));
}
```

### 2. Присвоение с условием

```rust
fn main() {
    let pnl = -500.0;

    let status = if pnl > 0.0 { "PROFIT" }
                 else if pnl < 0.0 { "LOSS" }
                 else { "BREAKEVEN" };

    println!("Status: {}", status);
}
```

### 3. Условное выполнение действий

```rust
fn main() {
    let alert_enabled = true;
    let price = 45000.0;
    let alert_price = 44000.0;

    if alert_enabled && price >= alert_price {
        println!("ALERT: Цена достигла ${:.2}!", price);
    }
}
```

## Что мы узнали

| Конструкция | Описание | Пример |
|-------------|----------|--------|
| `if` | Простое условие | `if price > 0.0 { ... }` |
| `if-else` | Два варианта | `if profit { buy } else { sell }` |
| `if-else if-else` | Множественные условия | Определение уровня RSI |
| `if` как выражение | Возвращает значение | `let x = if ... { a } else { b };` |
| Вложенные `if` | Условия внутри условий | Сложная логика |

## Домашнее задание

1. **Классификатор волатильности**
   Напиши программу, которая классифицирует волатильность на основе дневного диапазона цен:
   - < 1% — низкая
   - 1-3% — нормальная
   - 3-5% — повышенная
   - > 5% — высокая

2. **Система алертов**
   Создай систему, которая проверяет цену и выдаёт алерты:
   - Пробой уровня сопротивления
   - Пробой уровня поддержки
   - Достижение take-profit
   - Достижение stop-loss

3. **Калькулятор комиссий**
   Реализуй калькулятор комиссий биржи с учётом:
   - Maker/Taker ордеров
   - VIP-уровня (на основе объёма торгов)
   - Использования нативного токена (скидка 25%)

4. **Валидатор стратегии**
   Напиши валидатор параметров торговой стратегии:
   - Stop-loss обязателен
   - Take-profit должен быть больше entry (для лонга)
   - Риск на сделку не более 2% от депозита
   - Минимальное соотношение риск/прибыль 1:2

## Навигация

[← Предыдущий день](../016-comments-trading-logic/ru.md) | [Следующий день →](../018-else-if-order-types/ru.md)
