# День 16: Комментарии — документируем торговую логику

## Аналогия из трейдинга

Представь торговый журнал трейдера. Каждая сделка записана с пометками: **почему** открыл позицию, **какой** был сигнал, **что** пошло не так. Без этих заметок через месяц ты не вспомнишь логику своих решений. Комментарии в коде — это такой же журнал для программиста: они объясняют **зачем** написан код, а не **что** он делает.

## Однострочные комментарии

Начинаются с `//` и продолжаются до конца строки:

```rust
fn main() {
    // Текущая цена Bitcoin
    let btc_price = 42000.0;

    let position_size = 0.5;  // Размер позиции в BTC

    // Расчёт стоимости позиции
    let position_value = btc_price * position_size;

    println!("Position value: ${}", position_value);
}
```

## Многострочные комментарии

Начинаются с `/*` и заканчиваются `*/`:

```rust
fn main() {
    /*
     * Стратегия скальпинга:
     * 1. Входим при пробое уровня
     * 2. Стоп-лосс 0.5% от входа
     * 3. Тейк-профит 1% от входа
     */

    let entry_price = 42000.0;
    let stop_loss = entry_price * 0.995;    // -0.5%
    let take_profit = entry_price * 1.01;   // +1%

    println!("Entry: {}, SL: {}, TP: {}", entry_price, stop_loss, take_profit);
}
```

## Документационные комментарии

### Для функций и структур: `///`

Генерируют документацию через `cargo doc`:

```rust
/// Рассчитывает прибыль/убыток сделки.
///
/// # Аргументы
///
/// * `entry_price` - Цена входа в позицию
/// * `exit_price` - Цена выхода из позиции
/// * `quantity` - Количество актива
///
/// # Возвращает
///
/// Прибыль (положительное число) или убыток (отрицательное)
///
/// # Пример
///
/// ```
/// let pnl = calculate_pnl(42000.0, 43500.0, 0.5);
/// assert!(pnl > 0.0);
/// ```
fn calculate_pnl(entry_price: f64, exit_price: f64, quantity: f64) -> f64 {
    (exit_price - entry_price) * quantity
}

fn main() {
    let pnl = calculate_pnl(42000.0, 43500.0, 0.5);
    println!("PnL: ${:.2}", pnl);
}
```

### Для модулей: `//!`

```rust
//! # Модуль управления рисками
//!
//! Этот модуль содержит функции для:
//! - Расчёта размера позиции
//! - Определения стоп-лосса
//! - Вычисления максимального риска

/// Рассчитывает размер позиции на основе риска.
fn calculate_position_size(balance: f64, risk_percent: f64, stop_distance: f64) -> f64 {
    let risk_amount = balance * (risk_percent / 100.0);
    risk_amount / stop_distance
}

fn main() {
    let size = calculate_position_size(10000.0, 2.0, 500.0);
    println!("Position size: {:.4}", size);
}
```

## Комментарии в трейдинговом коде

### Объяснение формул

```rust
fn main() {
    let prices = [42000.0, 42100.0, 42050.0, 42200.0, 42150.0];

    // SMA (Simple Moving Average) = сумма цен / количество периодов
    let sma: f64 = prices.iter().sum::<f64>() / prices.len() as f64;

    // RSI = 100 - (100 / (1 + RS))
    // где RS = средний рост / средний падение
    // Упрощённый расчёт для демонстрации
    let avg_gain = 75.0;
    let avg_loss = 25.0;
    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - (100.0 / (1.0 + rs));

    println!("SMA: {:.2}, RSI: {:.2}", sma, rsi);
}
```

### Объяснение торговой логики

```rust
fn main() {
    let current_price = 42500.0;
    let sma_20 = 42000.0;
    let rsi = 65.0;

    // Условия для лонга:
    // 1. Цена выше SMA-20 (восходящий тренд)
    // 2. RSI между 30 и 70 (не перекуплен/перепродан)
    // 3. RSI растёт (momentum)

    let above_sma = current_price > sma_20;
    let rsi_neutral = rsi > 30.0 && rsi < 70.0;

    if above_sma && rsi_neutral {
        println!("LONG signal");
    } else {
        println!("No signal");
    }
}
```

### Пометки TODO и FIXME

```rust
fn main() {
    let balance = 10000.0;
    let risk_percent = 2.0;

    // TODO: добавить проверку максимального размера позиции
    // TODO: учесть комиссию биржи

    let position_size = balance * (risk_percent / 100.0);

    // FIXME: деление на ноль при нулевом стоп-лоссе
    let stop_distance = 500.0;
    let lots = position_size / stop_distance;

    // HACK: временное решение, пока нет API биржи
    let simulated_fill_price = 42000.0 * 1.001;  // slippage 0.1%

    println!("Lots: {:.4}, Fill: {:.2}", lots, simulated_fill_price);
}
```

## Когда комментировать

### Хорошие комментарии

```rust
fn main() {
    // Коэффициент Шарпа > 1 считается хорошим,
    // > 2 — отличным, > 3 — исключительным
    let sharpe_ratio = 1.85;

    // Максимальная просадка не должна превышать 20%
    // согласно нашим риск-параметрам
    let max_drawdown = 0.15;

    // Используем 252 торговых дня в году
    // (стандарт для американских рынков)
    let trading_days = 252;

    let annual_return = 0.25;
    let annualized_volatility = annual_return / sharpe_ratio;

    println!("Volatility: {:.2}%", annualized_volatility * 100.0);
}
```

### Плохие комментарии (очевидные)

```rust
fn main() {
    // Объявляем переменную price
    let price = 42000.0;  // НЕ НУЖНО!

    // Увеличиваем quantity на 1
    let mut quantity = 5;
    quantity += 1;  // НЕ НУЖНО!

    // Вызываем функцию println
    println!("Price: {}, Qty: {}", price, quantity);  // НЕ НУЖНО!
}
```

### Лучше — понятные имена вместо комментариев

```rust
fn main() {
    // Плохо:
    let p = 42000.0;  // цена
    let q = 0.5;      // количество
    let f = 0.001;    // комиссия

    // Хорошо — имена говорят сами за себя:
    let bitcoin_price = 42000.0;
    let position_quantity = 0.5;
    let exchange_fee_rate = 0.001;

    let total_with_fee = bitcoin_price * position_quantity * (1.0 + exchange_fee_rate);
    println!("Total: ${:.2}", total_with_fee);
}
```

## Документирование торговых стратегий

```rust
/// Стратегия пересечения скользящих средних (Moving Average Crossover).
///
/// # Логика стратегии
///
/// - **Сигнал на покупку**: быстрая MA пересекает медленную MA снизу вверх
/// - **Сигнал на продажу**: быстрая MA пересекает медленную MA сверху вниз
///
/// # Параметры
///
/// * `fast_ma` - Значение быстрой скользящей средней (например, SMA-10)
/// * `slow_ma` - Значение медленной скользящей средней (например, SMA-50)
/// * `prev_fast_ma` - Предыдущее значение быстрой MA
/// * `prev_slow_ma` - Предыдущее значение медленной MA
///
/// # Возвращает
///
/// * `1` - сигнал на покупку (BUY)
/// * `-1` - сигнал на продажу (SELL)
/// * `0` - нет сигнала (HOLD)
///
/// # Пример
///
/// ```
/// let signal = ma_crossover_signal(42100.0, 42000.0, 41900.0, 42000.0);
/// assert_eq!(signal, 1); // Покупка: быстрая MA пересекла медленную снизу
/// ```
fn ma_crossover_signal(fast_ma: f64, slow_ma: f64, prev_fast_ma: f64, prev_slow_ma: f64) -> i32 {
    let currently_above = fast_ma > slow_ma;
    let was_below = prev_fast_ma <= prev_slow_ma;

    let currently_below = fast_ma < slow_ma;
    let was_above = prev_fast_ma >= prev_slow_ma;

    if currently_above && was_below {
        1  // Bullish crossover — покупаем
    } else if currently_below && was_above {
        -1  // Bearish crossover — продаём
    } else {
        0  // Нет пересечения — держим
    }
}

fn main() {
    let signal = ma_crossover_signal(42100.0, 42000.0, 41900.0, 42000.0);

    match signal {
        1 => println!("Signal: BUY"),
        -1 => println!("Signal: SELL"),
        _ => println!("Signal: HOLD"),
    }
}
```

## Комментарии для отключения кода

```rust
fn main() {
    let price = 42000.0;
    let quantity = 0.5;

    // Временно отключаем для отладки
    // let with_leverage = apply_leverage(price, 10);

    /*
    Старая логика расчёта комиссии:
    let fee = price * quantity * 0.001;
    let total = price * quantity + fee;
    */

    // Новая логика с учётом VIP-уровня
    let vip_fee_rate = 0.0005;
    let fee = price * quantity * vip_fee_rate;
    let total = price * quantity + fee;

    println!("Total with VIP fee: ${:.2}", total);
}
```

## Структурирование кода секциями

```rust
fn main() {
    // ============================================
    // КОНФИГУРАЦИЯ
    // ============================================

    let initial_balance = 10000.0;
    let risk_per_trade = 0.02;
    let max_positions = 5;

    // ============================================
    // РЫНОЧНЫЕ ДАННЫЕ
    // ============================================

    let btc_price = 42000.0;
    let eth_price = 2200.0;

    // ============================================
    // РАСЧЁТЫ
    // ============================================

    let btc_position_value = initial_balance * risk_per_trade;
    let btc_quantity = btc_position_value / btc_price;

    // ============================================
    // ВЫВОД РЕЗУЛЬТАТОВ
    // ============================================

    println!("BTC position: {:.6} BTC (${:.2})", btc_quantity, btc_position_value);
}
```

## Практический пример: полностью документированный калькулятор позиции

```rust
//! # Калькулятор размера позиции
//!
//! Модуль для расчёта оптимального размера позиции
//! на основе управления рисками.

/// Информация о рассчитанной позиции.
struct PositionInfo {
    /// Размер позиции в единицах актива
    size: f64,
    /// Стоимость позиции в валюте счёта
    value: f64,
    /// Риск в валюте счёта
    risk_amount: f64,
    /// Потенциальная прибыль
    potential_profit: f64,
    /// Соотношение риск/прибыль
    risk_reward_ratio: f64,
}

/// Рассчитывает размер позиции по методу фиксированного процента риска.
///
/// # Формула
///
/// `position_size = (balance * risk_percent) / |entry - stop_loss|`
///
/// # Аргументы
///
/// * `balance` - Баланс счёта
/// * `risk_percent` - Процент риска на сделку (например, 2.0 для 2%)
/// * `entry_price` - Цена входа
/// * `stop_loss` - Уровень стоп-лосса
/// * `take_profit` - Уровень тейк-профита
///
/// # Пример
///
/// ```
/// let position = calculate_position(10000.0, 2.0, 42000.0, 41500.0, 43000.0);
/// println!("Size: {:.6}", position.size);
/// ```
fn calculate_position(
    balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
) -> PositionInfo {
    // Риск в валюте счёта = баланс * процент_риска / 100
    let risk_amount = balance * (risk_percent / 100.0);

    // Расстояние до стоп-лосса в единицах цены
    let stop_distance = (entry_price - stop_loss).abs();

    // Размер позиции = риск / расстояние_до_стопа
    let size = risk_amount / stop_distance;

    // Стоимость позиции
    let value = size * entry_price;

    // Расстояние до тейк-профита
    let profit_distance = (take_profit - entry_price).abs();

    // Потенциальная прибыль
    let potential_profit = size * profit_distance;

    // Соотношение риск/прибыль (R:R)
    // R:R > 1 означает, что потенциальная прибыль больше риска
    let risk_reward_ratio = profit_distance / stop_distance;

    PositionInfo {
        size,
        value,
        risk_amount,
        potential_profit,
        risk_reward_ratio,
    }
}

/// Выводит информацию о позиции в читаемом формате.
fn print_position_info(info: &PositionInfo) {
    println!("╔════════════════════════════════════╗");
    println!("║      POSITION CALCULATOR           ║");
    println!("╠════════════════════════════════════╣");
    println!("║ Size:           {:>16.6} ║", info.size);
    println!("║ Value:          ${:>15.2} ║", info.value);
    println!("║ Risk:           ${:>15.2} ║", info.risk_amount);
    println!("║ Potential:      ${:>15.2} ║", info.potential_profit);
    println!("║ Risk/Reward:    {:>16.2} ║", info.risk_reward_ratio);
    println!("╚════════════════════════════════════╝");
}

fn main() {
    // Параметры сделки
    let balance = 10000.0;       // Баланс $10,000
    let risk = 2.0;              // Рискуем 2% на сделку
    let entry = 42000.0;         // Вход по $42,000
    let stop = 41500.0;          // Стоп на $41,500 (-1.2%)
    let target = 43500.0;        // Цель $43,500 (+3.6%)

    let position = calculate_position(balance, risk, entry, stop, target);
    print_position_info(&position);
}
```

## Что мы узнали

| Тип комментария | Синтаксис | Назначение |
|-----------------|-----------|------------|
| Однострочный | `// текст` | Краткое пояснение |
| Многострочный | `/* текст */` | Блок пояснений |
| Документация | `/// текст` | Документация функций/структур |
| Документация модуля | `//! текст` | Документация модуля |
| TODO | `// TODO:` | Запланированные доработки |
| FIXME | `// FIXME:` | Известные проблемы |

## Правила хороших комментариев

1. **Объясняй "зачем", а не "что"** — код показывает что происходит, комментарий объясняет почему
2. **Документируй формулы** — особенно в трейдинге, где много математики
3. **Обновляй комментарии** — устаревший комментарий хуже его отсутствия
4. **Используй TODO/FIXME** — помогает отслеживать технический долг
5. **Пиши doc-комментарии** — для публичных функций и API

## Домашнее задание

1. Добавь документационные комментарии к функции расчёта скользящей средней (SMA) с примерами использования

2. Напиши функцию `calculate_atr()` (Average True Range) с подробными комментариями, объясняющими формулу

3. Создай документированную структуру `TradeJournal` для ведения журнала сделок с doc-комментариями для каждого поля

4. Прокомментируй стратегию RSI-дивергенции: когда цена делает новый максимум, а RSI — нет (медвежья дивергенция)

## Навигация

[← Предыдущий день](../015-return-values-pnl/ru.md) | [Следующий день →](../017-control-flow-if/ru.md)
