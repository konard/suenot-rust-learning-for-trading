# День 20: while — ждём пока цена достигнет цели

## Аналогия из трейдинга

Представь, что ты выставил лимитный ордер на покупку Bitcoin по цене $40,000. Рынок сейчас на уровне $42,000, и ты **ждёшь**, пока цена опустится до твоей цели. Ты не будешь проверять цену вечно — только **пока** она выше твоей цели.

Цикл `while` работает точно так же: выполняй действие, **пока** условие истинно.

## Базовый синтаксис while

```rust
fn main() {
    let target_price = 40000.0;
    let mut current_price = 42000.0;

    // Симулируем падение цены
    while current_price > target_price {
        println!("Текущая цена: ${:.2} — ждём снижения...", current_price);
        current_price -= 500.0; // Цена падает на $500
    }

    println!("Цена достигла ${:.2} — ПОКУПАЕМ!", current_price);
}
```

**Важно:** Условие проверяется **перед** каждой итерацией. Если условие изначально ложно — цикл не выполнится ни разу.

## while vs loop

```rust
fn main() {
    // loop — бесконечный цикл, требует break
    // while — проверяет условие, завершается автоматически

    let mut price = 100.0;
    let take_profit = 110.0;

    // while: проверяем условие на входе
    while price < take_profit {
        price += 2.0;
        println!("Цена растёт: ${:.2}", price);
    }

    println!("Тейк-профит достигнут!");
}
```

## Ожидание торгового сигнала

```rust
fn main() {
    let prices = [42000.0, 41800.0, 41500.0, 41200.0, 40800.0, 40500.0, 40000.0];
    let mut index = 0;
    let buy_signal_price = 41000.0;

    println!("Ожидаем сигнал на покупку (цена < ${:.2})", buy_signal_price);

    while index < prices.len() && prices[index] >= buy_signal_price {
        println!("  Цена ${:.2} — сигнала нет", prices[index]);
        index += 1;
    }

    if index < prices.len() {
        println!("СИГНАЛ! Цена ${:.2} — открываем позицию", prices[index]);
    } else {
        println!("Сигнал не получен за период наблюдения");
    }
}
```

## Накопление позиции (DCA)

Dollar Cost Averaging — покупка актива частями:

```rust
fn main() {
    let total_budget = 10000.0;   // Общий бюджет $10,000
    let order_size = 2000.0;      // Размер одной покупки
    let mut spent = 0.0;
    let mut btc_accumulated = 0.0;
    let mut order_number = 1;

    // Симулируем цены на момент каждой покупки
    let prices = [42000.0, 41500.0, 40000.0, 39500.0, 41000.0];
    let mut price_index = 0;

    while spent < total_budget && price_index < prices.len() {
        let price = prices[price_index];
        let btc_bought = order_size / price;

        println!(
            "Покупка #{}: ${:.2} по цене ${:.2} = {:.6} BTC",
            order_number, order_size, price, btc_bought
        );

        btc_accumulated += btc_bought;
        spent += order_size;
        order_number += 1;
        price_index += 1;
    }

    let avg_price = spent / btc_accumulated;
    println!("\n=== ИТОГО ===");
    println!("Потрачено: ${:.2}", spent);
    println!("Накоплено: {:.6} BTC", btc_accumulated);
    println!("Средняя цена: ${:.2}", avg_price);
}
```

## Мониторинг стоп-лосса

```rust
fn main() {
    let entry_price = 42000.0;
    let stop_loss = 40000.0;     // -4.76% от входа
    let mut current_price = entry_price;

    // Симулируем движение цены
    let price_changes = [-100.0, -200.0, 50.0, -500.0, -300.0, -400.0, -600.0];
    let mut change_index = 0;

    println!("Позиция открыта по ${:.2}", entry_price);
    println!("Стоп-лосс: ${:.2}\n", stop_loss);

    while current_price > stop_loss && change_index < price_changes.len() {
        current_price += price_changes[change_index];
        let pnl_percent = ((current_price - entry_price) / entry_price) * 100.0;

        println!(
            "Цена: ${:.2} | PnL: {:+.2}%",
            current_price, pnl_percent
        );

        change_index += 1;
    }

    if current_price <= stop_loss {
        let loss = entry_price - current_price;
        let loss_percent = (loss / entry_price) * 100.0;
        println!("\nСТОП-ЛОСС СРАБОТАЛ!");
        println!("Убыток: ${:.2} ({:.2}%)", loss, loss_percent);
    } else {
        println!("\nПозиция всё ещё открыта");
    }
}
```

## Расчёт скользящей средней (SMA)

```rust
fn main() {
    let prices = [42000.0, 42100.0, 41900.0, 42300.0, 42500.0,
                  42400.0, 42600.0, 42800.0, 42700.0, 43000.0];
    let period = 5;

    println!("Расчёт SMA-{} для {} цен\n", period, prices.len());

    let mut i = period - 1;

    while i < prices.len() {
        // Суммируем последние `period` цен
        let mut sum = 0.0;
        let mut j = 0;

        while j < period {
            sum += prices[i - j];
            j += 1;
        }

        let sma = sum / period as f64;
        println!("Индекс {}: Цена ${:.2} | SMA-{}: ${:.2}", i, prices[i], period, sma);

        i += 1;
    }
}
```

## Ожидание подтверждения тренда

```rust
fn main() {
    let prices = [42000.0, 42100.0, 42200.0, 42150.0, 42300.0, 42500.0, 42700.0];
    let confirmations_needed = 3;
    let mut consecutive_up = 0;
    let mut i = 1;

    println!("Ожидаем {} последовательных свечей роста...\n", confirmations_needed);

    while i < prices.len() && consecutive_up < confirmations_needed {
        if prices[i] > prices[i - 1] {
            consecutive_up += 1;
            println!(
                "Свеча {}: ${:.2} > ${:.2} — рост #{}/{}",
                i, prices[i], prices[i - 1], consecutive_up, confirmations_needed
            );
        } else {
            consecutive_up = 0;
            println!(
                "Свеча {}: ${:.2} <= ${:.2} — сброс счётчика",
                i, prices[i], prices[i - 1]
            );
        }
        i += 1;
    }

    if consecutive_up >= confirmations_needed {
        println!("\nТРЕНД ПОДТВЕРЖДЁН! Открываем LONG позицию.");
    } else {
        println!("\nТренд не подтверждён.");
    }
}
```

## Управление портфелем с ребалансировкой

```rust
fn main() {
    let target_btc_percent = 50.0;
    let tolerance = 5.0; // Допустимое отклонение ±5%

    let mut btc_value = 6000.0;  // $6,000 в BTC
    let mut usd_value = 4000.0;  // $4,000 в USD

    // Симулируем изменение цены BTC
    let price_multipliers = [1.1, 1.15, 0.95, 1.2, 0.9];
    let mut step = 0;

    while step < price_multipliers.len() {
        btc_value *= price_multipliers[step];
        let total = btc_value + usd_value;
        let btc_percent = (btc_value / total) * 100.0;

        println!("\n=== Шаг {} ===", step + 1);
        println!("BTC: ${:.2} ({:.1}%)", btc_value, btc_percent);
        println!("USD: ${:.2} ({:.1}%)", usd_value, 100.0 - btc_percent);

        // Проверяем нужна ли ребалансировка
        let deviation = (btc_percent - target_btc_percent).abs();

        if deviation > tolerance {
            println!("Отклонение {:.1}% > {:.1}% — нужна ребалансировка!", deviation, tolerance);

            let target_btc_value = total * (target_btc_percent / 100.0);
            let adjustment = btc_value - target_btc_value;

            if adjustment > 0.0 {
                println!("Продаём ${:.2} BTC", adjustment);
                btc_value -= adjustment;
                usd_value += adjustment;
            } else {
                println!("Покупаем ${:.2} BTC", -adjustment);
                btc_value -= adjustment; // adjustment отрицательный
                usd_value += adjustment;
            }
        } else {
            println!("Отклонение {:.1}% — ребалансировка не нужна", deviation);
        }

        step += 1;
    }
}
```

## Анализ объёмов торгов

```rust
fn main() {
    let volumes = [1500.0, 2000.0, 2500.0, 1800.0, 3500.0, 4000.0, 3800.0];
    let volume_threshold = 3000.0;
    let mut high_volume_count = 0;
    let mut i = 0;

    println!("Анализ объёмов (порог: ${:.0})\n", volume_threshold);

    while i < volumes.len() {
        let volume = volumes[i];
        let status = if volume >= volume_threshold {
            high_volume_count += 1;
            "ВЫСОКИЙ"
        } else {
            "обычный"
        };

        println!("Свеча {}: объём ${:.0} — {}", i + 1, volume, status);
        i += 1;
    }

    let high_volume_percent = (high_volume_count as f64 / volumes.len() as f64) * 100.0;
    println!("\nСвечей с высоким объёмом: {} из {} ({:.1}%)",
             high_volume_count, volumes.len(), high_volume_percent);
}
```

## Практический пример: симуляция торговой сессии

```rust
fn main() {
    let mut balance = 10000.0;
    let position_size = 0.1; // 0.1 BTC
    let mut btc_holdings = 0.0;
    let mut in_position = false;

    // Симуляция цен в течение дня
    let prices = [42000.0, 41800.0, 41500.0, 41200.0, 41500.0,
                  42000.0, 42500.0, 43000.0, 42800.0, 42500.0];

    let buy_threshold = 41300.0;
    let sell_threshold = 42800.0;

    let mut i = 0;

    println!("=== ТОРГОВАЯ СЕССИЯ ===\n");
    println!("Начальный баланс: ${:.2}", balance);
    println!("Размер позиции: {} BTC", position_size);
    println!("Покупка при < ${:.0}, продажа при > ${:.0}\n", buy_threshold, sell_threshold);

    while i < prices.len() {
        let price = prices[i];
        println!("Час {}: Цена ${:.2}", i + 1, price);

        // Логика покупки
        if !in_position && price < buy_threshold {
            let cost = price * position_size;
            if balance >= cost {
                balance -= cost;
                btc_holdings += position_size;
                in_position = true;
                println!("  → ПОКУПКА {} BTC по ${:.2} (потрачено ${:.2})", position_size, price, cost);
            }
        }

        // Логика продажи
        if in_position && price > sell_threshold {
            let revenue = price * btc_holdings;
            balance += revenue;
            println!("  → ПРОДАЖА {} BTC по ${:.2} (получено ${:.2})", btc_holdings, price, revenue);
            btc_holdings = 0.0;
            in_position = false;
        }

        i += 1;
    }

    // Итоги
    println!("\n=== ИТОГИ СЕССИИ ===");
    println!("Баланс USD: ${:.2}", balance);
    println!("Остаток BTC: {:.4}", btc_holdings);

    if btc_holdings > 0.0 {
        let btc_value = btc_holdings * prices[prices.len() - 1];
        println!("Стоимость BTC: ${:.2}", btc_value);
        println!("Общий портфель: ${:.2}", balance + btc_value);
    }

    let profit = balance - 10000.0 + (btc_holdings * prices[prices.len() - 1]);
    println!("Прибыль: ${:+.2}", profit);
}
```

## Что мы узнали

| Концепт | Описание | Применение в трейдинге |
|---------|----------|----------------------|
| `while condition {}` | Цикл пока условие истинно | Ожидание целевой цены |
| Условие проверяется первым | Цикл может не выполниться | Проверка возможности сделки |
| `while` с индексом | Проход по массиву | Анализ исторических данных |
| Множественные условия | `while a && b` | Стоп-лосс + тейк-профит |
| Изменение состояния | `mut` переменные в цикле | Накопление позиции |

## Домашнее задание

1. **Ожидание входа:** Напиши программу, которая ждёт, пока RSI опустится ниже 30 (перепроданность), используя массив значений RSI.

2. **Trailing Stop:** Реализуй trailing stop-loss, который следует за ценой на расстоянии 2% и срабатывает при развороте.

3. **Накопление объёма:** Создай симуляцию, где бот покупает по $1000 каждый раз, когда цена падает на 5% от предыдущей покупки, пока не потратит весь бюджет.

4. **Анализ консолидации:** Напиши программу, которая определяет период консолидации — когда цена находится в диапазоне ±2% в течение N свечей подряд.

## Навигация

[← Предыдущий день](../019-loop/ru.md) | [Следующий день →](../021-for/ru.md)
