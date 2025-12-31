# День 95: match на Result — обрабатываем оба случая

## Аналогия из трейдинга

Представь, что ты отправляешь ордер на биржу. Есть два возможных исхода: ордер **исполнен** (успех) или **отклонён** (ошибка). Хороший трейдер не надеется на лучшее — он **продумывает оба сценария** заранее. Что делать, если ордер исполнился? Что делать, если отклонён?

Конструкция `match` в Rust заставляет тебя **явно обработать оба случая**. Это как чек-лист трейдера: "Если успех — делаю X, если неудача — делаю Y". Компилятор не даст забыть ни один сценарий!

## Синтаксис match для Result

```rust
fn main() {
    let result: Result<f64, String> = Ok(42000.0);

    match result {
        Ok(price) => println!("Цена получена: ${:.2}", price),
        Err(error) => println!("Ошибка: {}", error),
    }
}
```

**Ключевые моменты:**
- `Ok(value)` — извлекаем значение при успехе
- `Err(error)` — извлекаем ошибку при неудаче
- Компилятор требует обработать **оба** варианта

## Получение цены с биржи

```rust
fn main() {
    // Успешный запрос
    match fetch_price("BTC") {
        Ok(price) => {
            println!("Текущая цена BTC: ${:.2}", price);
            if price > 50000.0 {
                println!("Цена выше 50k — бычий рынок!");
            }
        }
        Err(e) => {
            println!("Не удалось получить цену: {}", e);
            println!("Используем последнюю известную цену...");
        }
    }

    // Неуспешный запрос
    match fetch_price("INVALID") {
        Ok(price) => println!("Цена: {}", price),
        Err(e) => println!("Ошибка: {}", e),
    }
}

fn fetch_price(ticker: &str) -> Result<f64, String> {
    match ticker {
        "BTC" => Ok(42500.0),
        "ETH" => Ok(2250.0),
        "SOL" => Ok(98.5),
        _ => Err(format!("Тикер '{}' не найден", ticker)),
    }
}
```

## Размещение ордера с полной обработкой

```rust
fn main() {
    let balance = 10000.0;

    // Попытка разместить ордер
    match place_order("BTC", 0.5, 42000.0, balance) {
        Ok(order_id) => {
            println!("Ордер размещён успешно!");
            println!("ID ордера: {}", order_id);
            println!("Ожидаем исполнения...");
        }
        Err(error) => {
            println!("Ордер отклонён!");
            println!("Причина: {}", error);
            println!("Проверьте баланс и параметры ордера.");
        }
    }

    // Попытка разместить слишком большой ордер
    match place_order("BTC", 10.0, 42000.0, balance) {
        Ok(order_id) => println!("Ордер {}", order_id),
        Err(error) => println!("Ошибка: {}", error),
    }
}

fn place_order(
    ticker: &str,
    quantity: f64,
    price: f64,
    balance: f64
) -> Result<String, String> {
    let total_cost = quantity * price;

    if quantity <= 0.0 {
        return Err(String::from("Количество должно быть положительным"));
    }

    if price <= 0.0 {
        return Err(String::from("Цена должна быть положительной"));
    }

    if total_cost > balance {
        return Err(format!(
            "Недостаточно средств. Требуется: ${:.2}, доступно: ${:.2}",
            total_cost, balance
        ));
    }

    // Генерируем ID ордера
    Ok(format!("ORD-{}-{}", ticker, 12345))
}
```

## Возврат значения из match

`match` — это выражение, поэтому оно возвращает значение:

```rust
fn main() {
    let result = calculate_profit(42000.0, 43500.0, 0.5);

    // match возвращает значение
    let message = match result {
        Ok(profit) => format!("Прибыль: ${:.2}", profit),
        Err(e) => format!("Ошибка расчёта: {}", e),
    };

    println!("{}", message);

    // Можно использовать напрямую в println!
    println!("Статус: {}", match result {
        Ok(_) => "Успех",
        Err(_) => "Неудача",
    });
}

fn calculate_profit(entry: f64, exit: f64, qty: f64) -> Result<f64, String> {
    if entry <= 0.0 || exit <= 0.0 {
        return Err(String::from("Цены должны быть положительными"));
    }
    if qty <= 0.0 {
        return Err(String::from("Количество должно быть положительным"));
    }
    Ok((exit - entry) * qty)
}
```

## Вложенная логика в ветках match

```rust
fn main() {
    let prices = vec![
        fetch_price_with_validation("BTC"),
        fetch_price_with_validation("ETH"),
        fetch_price_with_validation("INVALID"),
    ];

    for (i, result) in prices.iter().enumerate() {
        println!("\nЗапрос {}:", i + 1);
        match result {
            Ok(price) => {
                // Вложенная логика для успешного случая
                if *price > 10000.0 {
                    println!("  Высокая цена: ${:.2}", price);
                } else if *price > 1000.0 {
                    println!("  Средняя цена: ${:.2}", price);
                } else {
                    println!("  Низкая цена: ${:.2}", price);
                }
            }
            Err(error) => {
                // Можем анализировать тип ошибки
                if error.contains("не найден") {
                    println!("  Неизвестный тикер");
                } else if error.contains("сервер") {
                    println!("  Проблема с сервером, попробуйте позже");
                } else {
                    println!("  Неизвестная ошибка: {}", error);
                }
            }
        }
    }
}

fn fetch_price_with_validation(ticker: &str) -> Result<f64, String> {
    match ticker {
        "BTC" => Ok(42500.0),
        "ETH" => Ok(2250.0),
        _ => Err(format!("Тикер '{}' не найден", ticker)),
    }
}
```

## Обработка разных типов ошибок

```rust
fn main() {
    // Тестируем разные сценарии
    let test_cases = vec![
        ("BTC", 1.0, 42000.0),    // Успех
        ("BTC", -1.0, 42000.0),   // Неверное количество
        ("BTC", 1.0, -100.0),     // Неверная цена
        ("XXX", 1.0, 100.0),      // Неверный тикер
    ];

    for (ticker, qty, price) in test_cases {
        println!("\nПопытка: {} x {} @ ${}", ticker, qty, price);

        match validate_order(ticker, qty, price) {
            Ok(order) => {
                println!("  Ордер валиден: {:?}", order);
            }
            Err(OrderError::InvalidTicker(t)) => {
                println!("  Тикер '{}' не поддерживается", t);
            }
            Err(OrderError::InvalidQuantity(q)) => {
                println!("  Неверное количество: {}", q);
            }
            Err(OrderError::InvalidPrice(p)) => {
                println!("  Неверная цена: {}", p);
            }
        }
    }
}

#[derive(Debug)]
struct Order {
    ticker: String,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum OrderError {
    InvalidTicker(String),
    InvalidQuantity(f64),
    InvalidPrice(f64),
}

fn validate_order(ticker: &str, quantity: f64, price: f64) -> Result<Order, OrderError> {
    // Проверяем тикер
    let valid_tickers = ["BTC", "ETH", "SOL"];
    if !valid_tickers.contains(&ticker) {
        return Err(OrderError::InvalidTicker(ticker.to_string()));
    }

    // Проверяем количество
    if quantity <= 0.0 {
        return Err(OrderError::InvalidQuantity(quantity));
    }

    // Проверяем цену
    if price <= 0.0 {
        return Err(OrderError::InvalidPrice(price));
    }

    Ok(Order {
        ticker: ticker.to_string(),
        quantity,
        price,
    })
}
```

## Guards в match (условия)

```rust
fn main() {
    let results = vec![
        Ok(150.0),   // Хорошая прибыль
        Ok(10.0),    // Маленькая прибыль
        Ok(-50.0),   // Убыток
        Err("Ошибка подключения".to_string()),
    ];

    for result in results {
        match result {
            Ok(pnl) if pnl > 100.0 => {
                println!("Отличная сделка! PnL: ${:.2}", pnl);
            }
            Ok(pnl) if pnl > 0.0 => {
                println!("Прибыльная сделка. PnL: ${:.2}", pnl);
            }
            Ok(pnl) if pnl < 0.0 => {
                println!("Убыточная сделка. PnL: ${:.2}", pnl);
            }
            Ok(pnl) => {
                println!("Безубыточная сделка. PnL: ${:.2}", pnl);
            }
            Err(e) => {
                println!("Ошибка: {}", e);
            }
        }
    }
}
```

## Практический пример: калькулятор позиции

```rust
fn main() {
    println!("=== Калькулятор размера позиции ===\n");

    let scenarios = vec![
        (10000.0, 2.0, 42000.0, 41000.0),  // Нормальный
        (10000.0, 0.0, 42000.0, 41000.0),  // Нулевой риск
        (10000.0, 2.0, 42000.0, 42000.0),  // Entry = Stop
        (-5000.0, 2.0, 42000.0, 41000.0),  // Отрицательный баланс
    ];

    for (balance, risk_pct, entry, stop) in scenarios {
        println!("Баланс: ${}, Риск: {}%, Entry: ${}, Stop: ${}",
                 balance, risk_pct, entry, stop);

        match calculate_position_size(balance, risk_pct, entry, stop) {
            Ok(size) => {
                let position_value = size * entry;
                println!("  Размер позиции: {:.6} BTC", size);
                println!("  Стоимость позиции: ${:.2}", position_value);
                println!("  Риск в долларах: ${:.2}", balance * risk_pct / 100.0);
            }
            Err(e) => {
                println!("  Ошибка: {}", e);
            }
        }
        println!();
    }
}

fn calculate_position_size(
    balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
) -> Result<f64, String> {
    if balance <= 0.0 {
        return Err(String::from("Баланс должен быть положительным"));
    }

    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return Err(String::from("Риск должен быть от 0 до 100%"));
    }

    if entry_price <= 0.0 {
        return Err(String::from("Цена входа должна быть положительной"));
    }

    let risk_per_unit = (entry_price - stop_loss).abs();
    if risk_per_unit == 0.0 {
        return Err(String::from("Цена входа и стоп-лосс не могут быть равны"));
    }

    let risk_amount = balance * (risk_percent / 100.0);
    Ok(risk_amount / risk_per_unit)
}
```

## Комбинирование нескольких Result

```rust
fn main() {
    // Все операции успешны
    match execute_trade_pipeline("BTC", 0.1, 10000.0) {
        Ok(trade) => {
            println!("Сделка выполнена!");
            println!("  Тикер: {}", trade.ticker);
            println!("  Количество: {}", trade.quantity);
            println!("  Цена: ${:.2}", trade.price);
            println!("  ID: {}", trade.order_id);
        }
        Err(e) => {
            println!("Сделка не выполнена: {}", e);
        }
    }
}

#[derive(Debug)]
struct TradeResult {
    ticker: String,
    quantity: f64,
    price: f64,
    order_id: String,
}

fn execute_trade_pipeline(
    ticker: &str,
    quantity: f64,
    balance: f64,
) -> Result<TradeResult, String> {
    // Шаг 1: Получаем цену
    let price = match fetch_current_price(ticker) {
        Ok(p) => p,
        Err(e) => return Err(format!("Ошибка получения цены: {}", e)),
    };

    // Шаг 2: Проверяем баланс
    match check_balance(balance, quantity * price) {
        Ok(_) => {}
        Err(e) => return Err(format!("Ошибка баланса: {}", e)),
    };

    // Шаг 3: Размещаем ордер
    let order_id = match submit_order(ticker, quantity, price) {
        Ok(id) => id,
        Err(e) => return Err(format!("Ошибка размещения: {}", e)),
    };

    Ok(TradeResult {
        ticker: ticker.to_string(),
        quantity,
        price,
        order_id,
    })
}

fn fetch_current_price(ticker: &str) -> Result<f64, String> {
    match ticker {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2200.0),
        _ => Err(format!("Неизвестный тикер: {}", ticker)),
    }
}

fn check_balance(balance: f64, required: f64) -> Result<(), String> {
    if balance >= required {
        Ok(())
    } else {
        Err(format!("Недостаточно средств: нужно {:.2}, есть {:.2}", required, balance))
    }
}

fn submit_order(ticker: &str, quantity: f64, price: f64) -> Result<String, String> {
    if quantity > 0.0 && price > 0.0 {
        Ok(format!("ORD-{}-{}", ticker, 99999))
    } else {
        Err(String::from("Неверные параметры ордера"))
    }
}
```

## Сравнение: match vs другие подходы

```rust
fn main() {
    let result: Result<f64, String> = Ok(42000.0);

    // 1. match — полный контроль
    match &result {
        Ok(price) => println!("match: Цена ${:.2}", price),
        Err(e) => println!("match: Ошибка {}", e),
    }

    // 2. if let — когда важен только один случай
    if let Ok(price) = &result {
        println!("if let: Цена ${:.2}", price);
    }

    // 3. unwrap_or — значение по умолчанию
    let price = result.clone().unwrap_or(0.0);
    println!("unwrap_or: Цена ${:.2}", price);

    // 4. map — преобразование успешного значения
    let doubled = result.clone().map(|p| p * 2.0);
    println!("map: {:?}", doubled);
}
```

## Что мы узнали

| Концепция | Синтаксис | Применение |
|-----------|-----------|------------|
| Базовый match | `match result { Ok(v) => ..., Err(e) => ... }` | Полная обработка обоих случаев |
| Guard | `Ok(v) if v > 0.0 => ...` | Дополнительные условия |
| Вложенный match | `match { Ok(v) => match v {...} }` | Сложная логика |
| Match как выражение | `let x = match result {...}` | Получение значения |
| Enum ошибок | `Err(MyError::Type)` | Типизированные ошибки |

## Упражнения

1. **Валидатор ордера**: Напиши функцию, которая проверяет ордер и возвращает `Result<Order, OrderError>`. Используй `match` для обработки всех возможных ошибок.

2. **Конвертер валют**: Создай функцию конвертации валют, которая может вернуть ошибку при неизвестной валюте. Обработай результат с помощью `match`.

3. **Анализатор портфеля**: Напиши функцию, которая анализирует портфель и возвращает `Result<PortfolioStats, AnalysisError>`. Используй guards для классификации результатов.

## Домашнее задание

1. Реализуй функцию `execute_trade_with_retry`, которая пытается выполнить сделку и при неудаче повторяет попытку до 3 раз, используя `match` для анализа ошибок.

2. Создай тип `TradingError` с вариантами: `NetworkError`, `InsufficientFunds`, `InvalidOrder`, `ExchangeError`. Напиши функцию, которая возвращает `Result<Trade, TradingError>`, и обработай все варианты с помощью `match`.

3. Напиши функцию `analyze_multiple_trades(trades: Vec<Result<Trade, Error>>)`, которая подсчитывает количество успешных и неуспешных сделок, общую прибыль успешных сделок, и выводит статистику.

4. Реализуй chain of responsibility: функция `process_order` должна последовательно вызвать `validate_order`, `check_risk`, `check_balance`, `submit_to_exchange`, и при любой ошибке прервать цепочку с информативным сообщением.

## Навигация

[← Предыдущий день](../094-result-deeper/ru.md) | [Следующий день →](../096-if-let-result/ru.md)
