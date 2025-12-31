# День 76: Методы Result — обработка ошибок в трейдинге

## Аналогия из трейдинга

Представь, что ты отправляешь ордер на биржу. Биржа может ответить успехом (ордер исполнен) или ошибкой (недостаточно средств, неверная цена, биржа недоступна). Тип `Result` в Rust — это как ответ от биржи: либо `Ok(данные)`, либо `Err(ошибка)`. Методы `Result` — это различные стратегии обработки этих ответов:

- **unwrap** — "Я уверен, что ордер исполнится, иначе — паника!"
- **unwrap_or** — "Если ордер не исполнился, используй запасной вариант"
- **map** — "Преобразуй успешный результат"
- **?** — "Передай ошибку вызывающей функции"

## Базовые методы: is_ok() и is_err()

```rust
fn main() {
    let order_result: Result<f64, String> = execute_order("BTC", 0.1, 42000.0);

    // Проверка статуса
    if order_result.is_ok() {
        println!("Ордер успешно исполнен!");
    }

    if order_result.is_err() {
        println!("Ордер отклонён!");
    }

    // Ещё пример
    let balance_check = check_balance(1000.0, 500.0);
    println!("Баланс достаточен: {}", balance_check.is_ok());
}

fn execute_order(symbol: &str, qty: f64, price: f64) -> Result<f64, String> {
    if qty <= 0.0 {
        return Err(String::from("Количество должно быть положительным"));
    }
    if price <= 0.0 {
        return Err(String::from("Цена должна быть положительной"));
    }
    Ok(qty * price)  // Возвращаем стоимость ордера
}

fn check_balance(required: f64, available: f64) -> Result<(), String> {
    if available >= required {
        Ok(())
    } else {
        Err(format!("Недостаточно средств: нужно {}, есть {}", required, available))
    }
}
```

## unwrap() и expect() — когда уверен в успехе

```rust
fn main() {
    // unwrap — паника если Err
    let price = parse_price("42000.50").unwrap();
    println!("Цена: {}", price);

    // expect — паника с кастомным сообщением
    let volume = parse_volume("1.5")
        .expect("Критическая ошибка: не удалось распарсить объём");
    println!("Объём: {}", volume);

    // ОПАСНО! Это вызовет панику:
    // let bad_price = parse_price("invalid").unwrap();
}

fn parse_price(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Невозможно распарсить '{}' как цену", s))
}

fn parse_volume(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Невозможно распарсить '{}' как объём", s))
}
```

**Когда использовать:**
- В тестах
- Когда абсолютно уверен, что ошибка невозможна
- В прототипах (но потом замени на нормальную обработку!)

## unwrap_or() — значение по умолчанию

```rust
fn main() {
    // Если ошибка — используем значение по умолчанию
    let price = fetch_price("BTC").unwrap_or(0.0);
    println!("Цена BTC: ${}", price);

    // Полезно для опциональных настроек
    let risk_percent = parse_config_value("risk_percent").unwrap_or(2.0);
    println!("Риск: {}%", risk_percent);

    // Для ордеров с фолбэком
    let filled_qty = execute_market_order("ETH", 1.0).unwrap_or(0.0);
    if filled_qty == 0.0 {
        println!("Ордер не исполнен, пропускаем");
    } else {
        println!("Исполнено: {} ETH", filled_qty);
    }
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    // Имитация получения цены с биржи
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        _ => Err(format!("Неизвестный символ: {}", symbol)),
    }
}

fn parse_config_value(key: &str) -> Result<f64, String> {
    // Имитация чтения конфига
    Err(format!("Ключ '{}' не найден", key))
}

fn execute_market_order(symbol: &str, qty: f64) -> Result<f64, String> {
    if symbol == "ETH" {
        Ok(qty)
    } else {
        Err(String::from("Пара недоступна"))
    }
}
```

## unwrap_or_else() — ленивое вычисление

```rust
fn main() {
    // Замыкание выполняется только при ошибке
    let price = fetch_live_price("BTC")
        .unwrap_or_else(|e| {
            println!("Ошибка получения цены: {}, используем кэш", e);
            get_cached_price("BTC")
        });
    println!("Цена: ${}", price);

    // Полезно когда дефолт дорого вычислять
    let position_size = calculate_position(10000.0, 42000.0)
        .unwrap_or_else(|_| calculate_safe_position());
    println!("Размер позиции: {}", position_size);
}

fn fetch_live_price(_symbol: &str) -> Result<f64, String> {
    Err(String::from("Биржа недоступна"))
}

fn get_cached_price(_symbol: &str) -> f64 {
    41500.0  // Кэшированное значение
}

fn calculate_position(balance: f64, price: f64) -> Result<f64, String> {
    if price == 0.0 {
        return Err(String::from("Цена не может быть нулевой"));
    }
    Ok(balance / price)
}

fn calculate_safe_position() -> f64 {
    println!("Вычисляем безопасную позицию...");
    0.01  // Минимальная безопасная позиция
}
```

## unwrap_or_default() — для типов с Default

```rust
fn main() {
    // Для чисел default = 0
    let volume: f64 = parse_volume("invalid").unwrap_or_default();
    println!("Объём: {}", volume);  // 0.0

    // Для String default = ""
    let symbol: String = get_symbol(999).unwrap_or_default();
    println!("Символ: '{}'", symbol);  // ""

    // Для Vec default = []
    let trades: Vec<f64> = get_recent_trades("UNKNOWN").unwrap_or_default();
    println!("Сделки: {:?}", trades);  // []

    // Для bool default = false
    let is_active: bool = check_market_active("NYSE").unwrap_or_default();
    println!("Рынок активен: {}", is_active);  // false
}

fn parse_volume(s: &str) -> Result<f64, String> {
    s.parse().map_err(|_| String::from("Ошибка парсинга"))
}

fn get_symbol(id: u32) -> Result<String, String> {
    Err(format!("Символ с id {} не найден", id))
}

fn get_recent_trades(_symbol: &str) -> Result<Vec<f64>, String> {
    Err(String::from("Нет данных"))
}

fn check_market_active(_market: &str) -> Result<bool, String> {
    Err(String::from("Рынок недоступен"))
}
```

## map() — преобразование успешного результата

```rust
fn main() {
    // Преобразуем цену в строку для отображения
    let formatted = fetch_price("BTC")
        .map(|price| format!("${:.2}", price));
    println!("{:?}", formatted);  // Ok("$42000.00")

    // Цепочка преобразований
    let trade_value = parse_trade_data("100.0,42000.0")
        .map(|(qty, price)| qty * price)
        .map(|value| value * 1.001);  // Добавляем комиссию
    println!("Стоимость с комиссией: {:?}", trade_value);

    // Расчёт PnL
    let pnl = calculate_exit_value(42000.0, 43500.0, 0.5)
        .map(|gross| gross - 50.0);  // Минус комиссия
    println!("Чистый PnL: {:?}", pnl);
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        _ => Err(String::from("Неизвестный символ")),
    }
}

fn parse_trade_data(s: &str) -> Result<(f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(String::from("Неверный формат"));
    }
    let qty = parts[0].parse().map_err(|_| String::from("Ошибка qty"))?;
    let price = parts[1].parse().map_err(|_| String::from("Ошибка price"))?;
    Ok((qty, price))
}

fn calculate_exit_value(entry: f64, exit: f64, qty: f64) -> Result<f64, String> {
    if qty <= 0.0 {
        return Err(String::from("Количество должно быть положительным"));
    }
    Ok((exit - entry) * qty)
}
```

## map_err() — преобразование ошибки

```rust
fn main() {
    // Добавляем контекст к ошибке
    let result = fetch_order_book("BTC")
        .map_err(|e| format!("Ошибка загрузки стакана: {}", e));
    println!("{:?}", result);

    // Преобразуем тип ошибки
    let trade_result = execute_trade("ETH", 1.0)
        .map_err(TradeError::from);
    println!("{:?}", trade_result);

    // Логирование ошибок
    let price = get_market_price("DOGE")
        .map_err(|e| {
            eprintln!("[ERROR] {}", e);
            e
        });
    println!("Результат: {:?}", price);
}

#[derive(Debug)]
enum TradeError {
    NetworkError(String),
    ValidationError(String),
    InsufficientFunds(String),
}

impl From<String> for TradeError {
    fn from(s: String) -> Self {
        TradeError::NetworkError(s)
    }
}

fn fetch_order_book(_symbol: &str) -> Result<Vec<(f64, f64)>, String> {
    Err(String::from("Таймаут соединения"))
}

fn execute_trade(_symbol: &str, _qty: f64) -> Result<String, String> {
    Err(String::from("Биржа недоступна"))
}

fn get_market_price(_symbol: &str) -> Result<f64, String> {
    Err(String::from("Символ не найден"))
}
```

## and_then() — цепочка операций с Result

```rust
fn main() {
    // Цепочка валидаций и операций
    let result = validate_symbol("BTC")
        .and_then(|s| fetch_price(&s))
        .and_then(|price| calculate_order_value(price, 0.1));

    match result {
        Ok(value) => println!("Стоимость ордера: ${:.2}", value),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Полный pipeline обработки ордера
    let order_result = parse_order_request("BTC,0.5,42000")
        .and_then(|(symbol, qty, price)| validate_order(&symbol, qty, price))
        .and_then(|(symbol, qty, price)| check_balance_for_order(&symbol, qty, price))
        .and_then(|(symbol, qty, price)| place_order(&symbol, qty, price));

    println!("Результат ордера: {:?}", order_result);
}

fn validate_symbol(symbol: &str) -> Result<String, String> {
    if symbol.len() >= 2 && symbol.len() <= 10 {
        Ok(symbol.to_uppercase())
    } else {
        Err(String::from("Неверный формат символа"))
    }
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        _ => Err(format!("Цена для {} недоступна", symbol)),
    }
}

fn calculate_order_value(price: f64, qty: f64) -> Result<f64, String> {
    if qty <= 0.0 {
        return Err(String::from("Количество должно быть положительным"));
    }
    Ok(price * qty)
}

fn parse_order_request(s: &str) -> Result<(String, f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err(String::from("Формат: SYMBOL,QTY,PRICE"));
    }
    let symbol = parts[0].to_string();
    let qty = parts[1].parse().map_err(|_| String::from("Неверное количество"))?;
    let price = parts[2].parse().map_err(|_| String::from("Неверная цена"))?;
    Ok((symbol, qty, price))
}

fn validate_order(symbol: &str, qty: f64, price: f64) -> Result<(String, f64, f64), String> {
    if qty <= 0.0 || price <= 0.0 {
        return Err(String::from("Количество и цена должны быть положительными"));
    }
    Ok((symbol.to_string(), qty, price))
}

fn check_balance_for_order(_symbol: &str, qty: f64, price: f64) -> Result<(String, f64, f64), String> {
    let required = qty * price;
    let balance = 50000.0;  // Симуляция баланса
    if required > balance {
        return Err(format!("Недостаточно средств: нужно {}, есть {}", required, balance));
    }
    Ok((_symbol.to_string(), qty, price))
}

fn place_order(symbol: &str, qty: f64, price: f64) -> Result<String, String> {
    Ok(format!("ORDER_{}_{:.4}@{:.2}", symbol, qty, price))
}
```

## Оператор ? — элегантное распространение ошибок

```rust
fn main() {
    match execute_trading_strategy() {
        Ok(pnl) => println!("Стратегия завершена. PnL: ${:.2}", pnl),
        Err(e) => println!("Ошибка стратегии: {}", e),
    }
}

fn execute_trading_strategy() -> Result<f64, String> {
    // ? автоматически возвращает Err если операция неудачна
    let btc_price = get_price("BTC")?;
    let eth_price = get_price("ETH")?;

    let signal = analyze_spread(btc_price, eth_price)?;

    let position = if signal > 0.0 {
        open_position("BTC", 0.1, btc_price)?
    } else {
        open_position("ETH", 1.0, eth_price)?
    };

    let exit_price = get_exit_price(&position.symbol)?;
    let pnl = calculate_pnl(&position, exit_price)?;

    Ok(pnl)
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        _ => Err(format!("Цена {} недоступна", symbol)),
    }
}

fn analyze_spread(price1: f64, price2: f64) -> Result<f64, String> {
    if price1 == 0.0 || price2 == 0.0 {
        return Err(String::from("Цены не могут быть нулевыми"));
    }
    Ok(price1 / price2 - 16.8)  // Сигнал отклонения от среднего
}

struct Position {
    symbol: String,
    qty: f64,
    entry_price: f64,
}

fn open_position(symbol: &str, qty: f64, price: f64) -> Result<Position, String> {
    Ok(Position {
        symbol: symbol.to_string(),
        qty,
        entry_price: price,
    })
}

fn get_exit_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(43000.0),
        "ETH" => Ok(2600.0),
        _ => Err(format!("Цена выхода для {} недоступна", symbol)),
    }
}

fn calculate_pnl(position: &Position, exit_price: f64) -> Result<f64, String> {
    Ok((exit_price - position.entry_price) * position.qty)
}
```

## ok() и err() — преобразование в Option

```rust
fn main() {
    // ok() — извлекает Ok значение как Some, Err становится None
    let price: Option<f64> = fetch_price("BTC").ok();
    println!("Цена: {:?}", price);  // Some(42000.0)

    let bad_price: Option<f64> = fetch_price("INVALID").ok();
    println!("Плохая цена: {:?}", bad_price);  // None

    // err() — извлекает Err значение как Some, Ok становится None
    let error: Option<String> = fetch_price("INVALID").err();
    println!("Ошибка: {:?}", error);  // Some("...")

    // Полезно с Option-методами
    let default_price = fetch_price("UNKNOWN")
        .ok()
        .unwrap_or(0.0);
    println!("Цена с дефолтом: {}", default_price);

    // Комбинирование с filter
    let valid_price = fetch_price("BTC")
        .ok()
        .filter(|&p| p > 10000.0);
    println!("Валидная цена: {:?}", valid_price);
}

fn fetch_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        _ => Err(format!("Неизвестный символ: {}", symbol)),
    }
}
```

## Практический пример: Торговый движок с обработкой ошибок

```rust
fn main() {
    let engine = TradingEngine::new(10000.0);

    // Исполнение нескольких ордеров
    let orders = vec![
        ("BTC", 0.1, 42000.0),
        ("ETH", 2.0, 2500.0),
        ("INVALID", 1.0, 100.0),
        ("BTC", 0.05, 42500.0),
    ];

    for (symbol, qty, price) in orders {
        match engine.process_order(symbol, qty, price) {
            Ok(order_id) => println!("✓ Ордер исполнен: {}", order_id),
            Err(e) => println!("✗ Ошибка: {}", e),
        }
    }

    // Пакетная обработка с collect
    let results: Vec<Result<String, String>> = vec![
        engine.process_order("BTC", 0.1, 42000.0),
        engine.process_order("ETH", 1.0, 2500.0),
    ];

    let successful: Vec<String> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();
    println!("\nУспешные ордера: {:?}", successful);
}

struct TradingEngine {
    balance: f64,
}

impl TradingEngine {
    fn new(balance: f64) -> Self {
        TradingEngine { balance }
    }

    fn process_order(&self, symbol: &str, qty: f64, price: f64) -> Result<String, String> {
        // Валидация символа
        self.validate_symbol(symbol)?;

        // Валидация количества и цены
        self.validate_order_params(qty, price)?;

        // Проверка баланса
        self.check_sufficient_balance(qty, price)?;

        // Проверка лимитов
        self.check_order_limits(qty, price)?;

        // Генерация ID ордера
        Ok(format!("ORD-{}-{}", symbol, chrono_mock()))
    }

    fn validate_symbol(&self, symbol: &str) -> Result<(), String> {
        let valid_symbols = ["BTC", "ETH", "SOL", "DOGE"];
        if valid_symbols.contains(&symbol) {
            Ok(())
        } else {
            Err(format!("Неподдерживаемый символ: {}", symbol))
        }
    }

    fn validate_order_params(&self, qty: f64, price: f64) -> Result<(), String> {
        if qty <= 0.0 {
            return Err(String::from("Количество должно быть положительным"));
        }
        if price <= 0.0 {
            return Err(String::from("Цена должна быть положительной"));
        }
        Ok(())
    }

    fn check_sufficient_balance(&self, qty: f64, price: f64) -> Result<(), String> {
        let required = qty * price;
        if required > self.balance {
            Err(format!(
                "Недостаточно средств: требуется ${:.2}, доступно ${:.2}",
                required, self.balance
            ))
        } else {
            Ok(())
        }
    }

    fn check_order_limits(&self, qty: f64, price: f64) -> Result<(), String> {
        let value = qty * price;
        if value > 100000.0 {
            Err(String::from("Превышен лимит ордера ($100,000)"))
        } else {
            Ok(())
        }
    }
}

fn chrono_mock() -> u64 {
    1234567890  // Имитация timestamp
}
```

## Сравнение методов Result

| Метод | Описание | Пример использования |
|-------|----------|---------------------|
| `is_ok()` | Проверяет, содержит ли Ok | Условные проверки |
| `is_err()` | Проверяет, содержит ли Err | Условные проверки |
| `unwrap()` | Извлекает Ok или паника | Тесты, прототипы |
| `expect(msg)` | Извлекает Ok или паника с сообщением | Критические ошибки |
| `unwrap_or(val)` | Ok или значение по умолчанию | Простые фолбэки |
| `unwrap_or_else(f)` | Ok или вычисленное значение | Сложные фолбэки |
| `unwrap_or_default()` | Ok или Default::default() | Нулевые значения |
| `ok()` | Result -> Option (Ok) | Игнорирование ошибок |
| `err()` | Result -> Option (Err) | Извлечение ошибок |
| `map(f)` | Преобразует Ok значение | Трансформация данных |
| `map_err(f)` | Преобразует Err значение | Обогащение ошибок |
| `and_then(f)` | Цепочка Result операций | Pipeline обработки |
| `?` | Распространение ошибки | Элегантный код |

## Домашнее задание

1. **Валидатор ордеров**: Напиши функцию `validate_order_chain(symbol: &str, qty: f64, price: f64, balance: f64) -> Result<Order, OrderError>`, которая использует `and_then` для цепочки валидаций (символ, количество, цена, баланс).

2. **Парсер рыночных данных**: Создай функцию `parse_market_data(json: &str) -> Result<MarketData, ParseError>`, которая парсит строку с данными и использует `map` и `map_err` для преобразований.

3. **Торговая стратегия с ?**: Реализуй функцию `execute_strategy() -> Result<TradeResult, StrategyError>`, которая использует оператор `?` для последовательного выполнения: получение цены -> анализ -> открытие позиции -> закрытие.

4. **Обработчик портфеля**: Напиши функцию `process_portfolio(positions: Vec<&str>) -> Vec<Result<PositionValue, String>>`, которая обрабатывает список позиций и возвращает результаты для каждой, используя `map` для преобразования успешных и `filter_map` для сбора успешных.

## Навигация

[← Предыдущий день](../075-result-error-handling/ru.md) | [Следующий день →](../077-custom-error-types/ru.md)
