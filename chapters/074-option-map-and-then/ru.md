# День 74: Option с map и and_then

## Аналогия из трейдинга

Представьте, что вы анализируете сделку по цепочке:
1. Получить текущую цену актива (может быть недоступна)
2. Если цена есть — рассчитать размер позиции
3. Если размер рассчитан — определить потенциальную прибыль
4. Если всё успешно — показать результат

Каждый шаг может завершиться неудачей: биржа не отвечает, недостаточно данных, ошибка расчёта. Вместо бесконечных проверок `if let Some(x)` Rust предлагает элегантные методы **map** и **and_then** для цепочки трансформаций.

**map** — трансформирует значение внутри Option, если оно есть (как пересчёт цены из одной валюты в другую).

**and_then** — применяет функцию, которая сама возвращает Option (как проверка, есть ли данные на следующем шаге).

## Метод map

`map` применяет функцию к значению внутри `Some`, оставляя `None` без изменений:

```rust
fn main() {
    let btc_price: Option<f64> = Some(42000.0);
    let no_price: Option<f64> = None;

    // Конвертируем цену в евро (курс 0.92)
    let btc_eur = btc_price.map(|price| price * 0.92);
    let no_eur = no_price.map(|price| price * 0.92);

    println!("BTC в EUR: {:?}", btc_eur);  // Some(38640.0)
    println!("No price в EUR: {:?}", no_eur);  // None
}
```

### Цепочка map

```rust
fn main() {
    let entry_price: Option<f64> = Some(42000.0);

    // Цепочка трансформаций
    let result = entry_price
        .map(|price| price * 1.05)      // +5% take profit
        .map(|tp| tp * 0.5)             // Размер позиции
        .map(|value| value - 50.0);     // Минус комиссия

    println!("Результат: {:?}", result);  // Some(21975.0)

    // Если исходное значение None, вся цепочка вернёт None
    let no_price: Option<f64> = None;
    let no_result = no_price
        .map(|price| price * 1.05)
        .map(|tp| tp * 0.5);

    println!("No result: {:?}", no_result);  // None
}
```

## Метод and_then

`and_then` (также известный как flatMap в других языках) применяет функцию, которая сама возвращает `Option`:

```rust
fn main() {
    let balance: Option<f64> = Some(10000.0);

    // Функция, которая возвращает Option
    fn calculate_position(balance: f64) -> Option<f64> {
        if balance >= 1000.0 {
            Some(balance * 0.02)  // 2% от баланса
        } else {
            None  // Недостаточно средств
        }
    }

    // and_then "разворачивает" вложенный Option
    let position = balance.and_then(calculate_position);
    println!("Позиция: {:?}", position);  // Some(200.0)

    // С маленьким балансом
    let small_balance: Option<f64> = Some(500.0);
    let no_position = small_balance.and_then(calculate_position);
    println!("No position: {:?}", no_position);  // None
}
```

### Разница между map и and_then

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);

    // Функция, возвращающая Option
    fn get_stop_loss(price: f64) -> Option<f64> {
        if price > 0.0 {
            Some(price * 0.95)  // -5%
        } else {
            None
        }
    }

    // map создаёт вложенный Option<Option<f64>>
    let nested = price.map(get_stop_loss);
    println!("С map: {:?}", nested);  // Some(Some(39900.0))

    // and_then "сплющивает" результат
    let flat = price.and_then(get_stop_loss);
    println!("С and_then: {:?}", flat);  // Some(39900.0)
}
```

## Комбинирование map и and_then

```rust
fn main() {
    let ticker: Option<&str> = Some("BTCUSDT");

    // Имитация получения цены по тикеру
    fn get_price(ticker: &str) -> Option<f64> {
        match ticker {
            "BTCUSDT" => Some(42000.0),
            "ETHUSDT" => Some(2500.0),
            _ => None,
        }
    }

    // Имитация проверки ликвидности
    fn check_liquidity(price: f64) -> Option<f64> {
        if price > 1000.0 {
            Some(price)
        } else {
            None  // Недостаточная ликвидность
        }
    }

    let result = ticker
        .and_then(get_price)           // Option<f64>
        .and_then(check_liquidity)     // Option<f64>
        .map(|price| price * 0.02)     // Размер позиции
        .map(|pos| format!("Позиция: ${:.2}", pos));

    println!("{:?}", result);  // Some("Позиция: $840.00")
}
```

## Практический пример: анализ ордера

```rust
fn main() {
    // Структура ордера
    struct Order {
        symbol: String,
        quantity: Option<f64>,
        price: Option<f64>,
    }

    let order = Order {
        symbol: "BTCUSDT".to_string(),
        quantity: Some(0.5),
        price: Some(42000.0),
    };

    // Расчёт стоимости ордера
    let order_value = order.quantity
        .and_then(|qty| {
            order.price.map(|price| qty * price)
        });

    println!("Стоимость ордера: {:?}", order_value);  // Some(21000.0)

    // Более элегантный способ с zip (Rust 1.46+)
    let order2 = Order {
        symbol: "ETHUSDT".to_string(),
        quantity: Some(10.0),
        price: Some(2500.0),
    };

    let value2 = order2.quantity
        .zip(order2.price)
        .map(|(qty, price)| qty * price);

    println!("Стоимость: {:?}", value2);  // Some(25000.0)
}
```

## Обработка портфеля

```rust
fn main() {
    struct Position {
        symbol: String,
        quantity: f64,
        entry_price: Option<f64>,
    }

    fn get_current_price(symbol: &str) -> Option<f64> {
        match symbol {
            "BTC" => Some(43000.0),
            "ETH" => Some(2600.0),
            "SOL" => Some(100.0),
            _ => None,
        }
    }

    fn calculate_pnl(position: &Position) -> Option<f64> {
        position.entry_price.and_then(|entry| {
            get_current_price(&position.symbol).map(|current| {
                (current - entry) * position.quantity
            })
        })
    }

    let positions = vec![
        Position { symbol: "BTC".to_string(), quantity: 0.5, entry_price: Some(42000.0) },
        Position { symbol: "ETH".to_string(), quantity: 5.0, entry_price: Some(2500.0) },
        Position { symbol: "UNKNOWN".to_string(), quantity: 100.0, entry_price: Some(10.0) },
    ];

    println!("╔═══════════════════════════════════════╗");
    println!("║          PORTFOLIO P&L                ║");
    println!("╚═══════════════════════════════════════╝\n");

    for pos in &positions {
        let pnl = calculate_pnl(pos);
        match pnl {
            Some(value) => {
                let sign = if value >= 0.0 { "+" } else { "" };
                println!("{}: {}${:.2}", pos.symbol, sign, value);
            }
            None => println!("{}: Нет данных", pos.symbol),
        }
    }
}
```

## filter в связке с map

```rust
fn main() {
    let prices: Vec<Option<f64>> = vec![
        Some(42000.0),
        None,
        Some(2500.0),
        Some(50.0),
        None,
        Some(100.0),
    ];

    // Фильтруем и трансформируем цены
    let high_value_positions: Vec<f64> = prices
        .into_iter()
        .filter_map(|price| {
            price
                .filter(|&p| p >= 100.0)  // Только цены >= 100
                .map(|p| p * 0.01)         // 1% позиция
        })
        .collect();

    println!("Позиции: {:?}", high_value_positions);
    // [420.0, 25.0, 1.0]
}
```

## Методы ok_or и ok_or_else

Преобразование `Option` в `Result`:

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);
    let no_price: Option<f64> = None;

    // ok_or преобразует Option в Result
    let result1 = price.ok_or("Цена недоступна");
    let result2 = no_price.ok_or("Цена недоступна");

    println!("Result 1: {:?}", result1);  // Ok(42000.0)
    println!("Result 2: {:?}", result2);  // Err("Цена недоступна")

    // ok_or_else с ленивым вычислением ошибки
    fn create_error() -> String {
        println!("Создаём сообщение об ошибке...");
        "API timeout".to_string()
    }

    let lazy_result = no_price.ok_or_else(create_error);
    println!("Lazy: {:?}", lazy_result);
}
```

## Практический пример: торговый валидатор

```rust
fn main() {
    struct TradeRequest {
        symbol: Option<String>,
        side: Option<String>,
        quantity: Option<f64>,
        price: Option<f64>,
    }

    fn validate_symbol(symbol: &str) -> Option<String> {
        let valid_symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
        if valid_symbols.contains(&symbol) {
            Some(symbol.to_string())
        } else {
            None
        }
    }

    fn validate_quantity(qty: f64) -> Option<f64> {
        if qty > 0.0 && qty <= 1000.0 {
            Some(qty)
        } else {
            None
        }
    }

    fn validate_price(price: f64) -> Option<f64> {
        if price > 0.0 {
            Some(price)
        } else {
            None
        }
    }

    let request = TradeRequest {
        symbol: Some("BTCUSDT".to_string()),
        side: Some("BUY".to_string()),
        quantity: Some(0.5),
        price: Some(42000.0),
    };

    // Валидация через цепочку and_then
    let validated_order = request.symbol
        .as_ref()
        .and_then(|s| validate_symbol(s))
        .and_then(|symbol| {
            request.quantity
                .and_then(validate_quantity)
                .and_then(|qty| {
                    request.price
                        .and_then(validate_price)
                        .map(|price| {
                            format!(
                                "Order: {} {} @ ${:.2} (value: ${:.2})",
                                symbol, qty, price, qty * price
                            )
                        })
                })
        });

    match validated_order {
        Some(order) => println!("✓ Valid: {}", order),
        None => println!("✗ Invalid order"),
    }
}
```

## Что мы узнали

| Метод | Описание | Возвращает |
|-------|----------|------------|
| `map(f)` | Трансформирует значение внутри Some | `Option<U>` |
| `and_then(f)` | Применяет функцию, возвращающую Option | `Option<U>` |
| `filter(predicate)` | Сохраняет Some только если предикат true | `Option<T>` |
| `ok_or(err)` | Преобразует в Result | `Result<T, E>` |
| `ok_or_else(f)` | Ленивое преобразование в Result | `Result<T, E>` |
| `zip(other)` | Объединяет два Option в кортеж | `Option<(T, U)>` |

## Практические упражнения

### Упражнение 1: Калькулятор риска

Реализуйте функцию, которая:
- Принимает `Option<f64>` для баланса
- Принимает `Option<f64>` для процента риска
- Возвращает `Option<f64>` с размером риска

```rust
fn calculate_risk_amount(
    balance: Option<f64>,
    risk_percent: Option<f64>
) -> Option<f64> {
    // Ваш код здесь
    todo!()
}
```

### Упражнение 2: Цепочка анализа

Создайте функцию анализа возможности входа в позицию:
1. Получить цену (может быть None)
2. Проверить, что цена > 0
3. Рассчитать размер позиции (2% от баланса 10000)
4. Проверить, что размер >= 100
5. Вернуть финальное значение

### Упражнение 3: Обработка списка ордеров

```rust
struct Order {
    id: u64,
    price: Option<f64>,
    quantity: Option<f64>,
}

fn calculate_total_value(orders: Vec<Order>) -> f64 {
    // Суммируйте стоимость только валидных ордеров
    // (где оба поля price и quantity - Some)
    todo!()
}
```

### Упражнение 4: Конвертер валют

Создайте функцию, которая:
- Принимает сумму в USD (Option<f64>)
- Получает курс конвертации (может вернуть None)
- Применяет комиссию (может быть None = нет комиссии)
- Возвращает финальную сумму

## Домашнее задание

1. **Анализатор сигналов**: Создайте систему, которая получает торговый сигнал (Option), проверяет его валидность, рассчитывает параметры входа и возвращает готовый ордер.

2. **Портфельный калькулятор**: Реализуйте функцию, которая для списка позиций рассчитывает общий P&L, пропуская позиции с отсутствующими данными.

3. **API обёртка**: Напишите функции-обёртки для работы с биржевым API, где каждый вызов может вернуть None при ошибке, используя цепочки and_then.

4. **Валидатор стратегии**: Создайте валидатор торговой стратегии, который проверяет наличие и корректность всех параметров (entry, stop_loss, take_profit, position_size).

## Навигация

[← Предыдущий день](../073-option-unwrap-expect/ru.md) | [Следующий день →](../075-result-type/ru.md)
