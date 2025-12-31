# День 119: Паттерн: ошибка как значение

## Аналогия из трейдинга

Представь, что ты запрашиваешь данные с биржи. В большинстве языков программирования, если что-то идёт не так — сервер недоступен, неверный тикер, истёк токен — программа "взрывается" исключением. Это как если бы твой торговый терминал просто закрывался при любой ошибке.

В Rust подход другой. Когда ты вызываешь функцию получения цены, она возвращает тебе **результат в конверте**: либо цена, либо описание проблемы. Ты открываешь конверт и решаешь, что делать. Нет внезапных "взрывов" — всё под контролем.

Это как профессиональный риск-менеджмент: вместо паники при каждой проблеме, ты получаешь чёткий отчёт и принимаешь взвешенное решение.

## Философия "Ошибка как значение"

В Rust ошибки — это не исключительные ситуации, а **обычные значения**, которые возвращаются из функций:

```rust
// В других языках:
// price = get_price("BTC")  // Может "взорваться" в любой момент!

// В Rust:
// result = get_price("BTC")  // Возвращает Result<f64, Error>
// Ты ОБЯЗАН обработать оба варианта
```

## Result: Успех или Ошибка

`Result<T, E>` — это перечисление с двумя вариантами:

```rust
enum Result<T, E> {
    Ok(T),    // Успех — содержит значение типа T
    Err(E),   // Ошибка — содержит ошибку типа E
}
```

### Пример: Получение цены актива

```rust
fn main() {
    let symbols = ["BTC", "ETH", "INVALID", "SOL"];

    for symbol in symbols {
        match get_price(symbol) {
            Ok(price) => println!("{}: ${:.2}", symbol, price),
            Err(error) => println!("{}: Ошибка - {}", symbol, error),
        }
    }
}

fn get_price(symbol: &str) -> Result<f64, String> {
    // Симуляция получения цены с биржи
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        "SOL" => Ok(100.0),
        _ => Err(format!("Неизвестный тикер: {}", symbol)),
    }
}
```

## Option: Значение есть или нет

`Option<T>` — для случаев, когда отсутствие значения — это не ошибка, а нормальная ситуация:

```rust
enum Option<T> {
    Some(T),  // Значение есть
    None,     // Значения нет
}
```

### Пример: Поиск лучшей сделки

```rust
fn main() {
    let trades = vec![
        ("BTC", -150.0),
        ("ETH", 200.0),
        ("SOL", -50.0),
        ("DOGE", 500.0),
    ];

    match find_best_trade(&trades) {
        Some((symbol, pnl)) => {
            println!("Лучшая сделка: {} с прибылью ${:.2}", symbol, pnl);
        }
        None => println!("Нет прибыльных сделок"),
    }

    let empty_trades: Vec<(&str, f64)> = vec![];
    match find_best_trade(&empty_trades) {
        Some((symbol, pnl)) => println!("Лучшая: {} ${:.2}", symbol, pnl),
        None => println!("Нет сделок для анализа"),
    }
}

fn find_best_trade(trades: &[(&str, f64)]) -> Option<(&str, f64)> {
    trades
        .iter()
        .filter(|(_, pnl)| *pnl > 0.0)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .copied()
}
```

## Преимущества паттерна "Ошибка как значение"

### 1. Компилятор заставляет обрабатывать ошибки

```rust
fn main() {
    let price = get_price("BTC");

    // Ошибка компиляции! Нельзя использовать Result напрямую
    // let doubled = price * 2.0;  // Не скомпилируется

    // Правильно: явная обработка
    let doubled = match price {
        Ok(p) => p * 2.0,
        Err(_) => 0.0,  // Или другая логика обработки
    };

    println!("Удвоенная цена: ${:.2}", doubled);
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        _ => Err(String::from("Неизвестный тикер")),
    }
}
```

### 2. Ошибки видны в сигнатуре функции

```rust
// Сразу видно, что функция может вернуть ошибку
fn execute_order(
    symbol: &str,
    quantity: f64,
    price: f64,
) -> Result<OrderConfirmation, TradingError> {
    // ...
}

// Сразу видно, что результат может отсутствовать
fn find_stop_loss(position: &Position) -> Option<f64> {
    // ...
}

struct OrderConfirmation {
    order_id: String,
    filled_price: f64,
}

struct Position {
    symbol: String,
    size: f64,
}

#[derive(Debug)]
enum TradingError {
    InsufficientBalance,
    MarketClosed,
    InvalidQuantity,
}

fn execute_order(
    symbol: &str,
    quantity: f64,
    _price: f64,
) -> Result<OrderConfirmation, TradingError> {
    if quantity <= 0.0 {
        return Err(TradingError::InvalidQuantity);
    }
    Ok(OrderConfirmation {
        order_id: format!("ORD-{}-001", symbol),
        filled_price: 42000.0,
    })
}

fn find_stop_loss(position: &Position) -> Option<f64> {
    if position.size > 0.0 {
        Some(position.size * 0.95)  // 5% stop loss
    } else {
        None
    }
}

fn main() {
    // Примеры использования
    match execute_order("BTC", 0.5, 42000.0) {
        Ok(conf) => println!("Ордер исполнен: {}", conf.order_id),
        Err(e) => println!("Ошибка: {:?}", e),
    }

    let pos = Position { symbol: String::from("BTC"), size: 1.0 };
    if let Some(sl) = find_stop_loss(&pos) {
        println!("Stop Loss: {:.2}", sl);
    }
}
```

### 3. Легко комбинировать проверки

```rust
fn main() {
    match validate_and_execute_trade("BTC", 0.5, 42000.0, 50000.0) {
        Ok(result) => println!("Сделка выполнена: {}", result),
        Err(e) => println!("Ошибка: {}", e),
    }
}

fn validate_and_execute_trade(
    symbol: &str,
    quantity: f64,
    price: f64,
    balance: f64,
) -> Result<String, String> {
    // Цепочка проверок с оператором ?
    validate_symbol(symbol)?;
    validate_quantity(quantity)?;
    validate_balance(price * quantity, balance)?;

    // Если все проверки прошли — выполняем сделку
    Ok(format!("Куплено {} {} по ${:.2}", quantity, symbol, price))
}

fn validate_symbol(symbol: &str) -> Result<(), String> {
    let valid_symbols = ["BTC", "ETH", "SOL"];
    if valid_symbols.contains(&symbol) {
        Ok(())
    } else {
        Err(format!("Неподдерживаемый тикер: {}", symbol))
    }
}

fn validate_quantity(quantity: f64) -> Result<(), String> {
    if quantity > 0.0 {
        Ok(())
    } else {
        Err(String::from("Количество должно быть положительным"))
    }
}

fn validate_balance(cost: f64, balance: f64) -> Result<(), String> {
    if cost <= balance {
        Ok(())
    } else {
        Err(format!("Недостаточно средств: нужно ${:.2}, есть ${:.2}", cost, balance))
    }
}
```

## Сравнение с исключениями

```rust
fn main() {
    // Rust: явная обработка через Result
    let result = divide_safely(100.0, 0.0);
    match result {
        Ok(value) => println!("Результат: {}", value),
        Err(msg) => println!("Ошибка: {}", msg),
    }

    // Мы ТОЧНО знаем, что ошибка обработана
    // Нет скрытых путей выполнения
}

fn divide_safely(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("Деление на ноль"))
    } else {
        Ok(a / b)
    }
}

// В языках с исключениями было бы:
// try {
//     result = divide(100, 0)  // Может "выбросить" исключение
// } catch (e) {
//     // Легко забыть обработать
// }
// // Исключение может "проскочить" на много уровней вверх
```

## Методы для работы с Result

```rust
fn main() {
    let prices: Vec<Result<f64, String>> = vec![
        Ok(42000.0),
        Err(String::from("API недоступен")),
        Ok(2500.0),
    ];

    // unwrap_or: значение по умолчанию
    for price in &prices {
        let value = price.clone().unwrap_or(0.0);
        println!("Цена: ${:.2}", value);
    }

    println!("---");

    // map: преобразование успешного значения
    for price in &prices {
        let doubled = price.clone().map(|p| p * 2.0);
        println!("Удвоенная: {:?}", doubled);
    }

    println!("---");

    // is_ok / is_err: проверка типа
    let success_count = prices.iter().filter(|p| p.is_ok()).count();
    println!("Успешных запросов: {}", success_count);
}
```

## Методы для работы с Option

```rust
fn main() {
    let stop_losses: Vec<Option<f64>> = vec![
        Some(41000.0),
        None,
        Some(2400.0),
    ];

    // unwrap_or: значение по умолчанию
    for sl in &stop_losses {
        let value = sl.unwrap_or(0.0);
        println!("Stop Loss: ${:.2}", value);
    }

    println!("---");

    // map: преобразование
    for sl in &stop_losses {
        let adjusted = sl.map(|s| s * 0.99);  // Снижаем на 1%
        println!("Скорректированный SL: {:?}", adjusted);
    }

    println!("---");

    // filter: условная фильтрация
    for sl in &stop_losses {
        let significant = sl.filter(|&s| s > 10000.0);
        println!("Значимый SL: {:?}", significant);
    }
}
```

## Практический пример: Обработка портфеля

```rust
fn main() {
    let portfolio = vec![
        ("BTC", 0.5),
        ("INVALID", 10.0),
        ("ETH", 2.0),
        ("FAKE", 100.0),
    ];

    println!("=== Анализ портфеля ===\n");

    let mut total_value = 0.0;
    let mut errors = Vec::new();

    for (symbol, quantity) in &portfolio {
        match get_position_value(symbol, *quantity) {
            Ok(value) => {
                println!("✓ {}: {} шт × ${:.2} = ${:.2}",
                    symbol, quantity, value / quantity, value);
                total_value += value;
            }
            Err(error) => {
                println!("✗ {}: {}", symbol, error);
                errors.push((symbol, error));
            }
        }
    }

    println!("\n=== Итоги ===");
    println!("Общая стоимость: ${:.2}", total_value);
    println!("Ошибок при расчёте: {}", errors.len());

    if !errors.is_empty() {
        println!("\nПроблемные позиции:");
        for (symbol, error) in errors {
            println!("  - {}: {}", symbol, error);
        }
    }
}

fn get_position_value(symbol: &str, quantity: f64) -> Result<f64, String> {
    let price = get_price(symbol)?;  // Оператор ? пробрасывает ошибку

    if quantity <= 0.0 {
        return Err(String::from("Некорректное количество"));
    }

    Ok(price * quantity)
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2500.0),
        "SOL" => Ok(100.0),
        _ => Err(format!("Неизвестный тикер: {}", symbol)),
    }
}
```

## Конвертация между Result и Option

```rust
fn main() {
    // Result -> Option (отбрасываем информацию об ошибке)
    let result: Result<f64, String> = Ok(42000.0);
    let option: Option<f64> = result.ok();
    println!("Result -> Option: {:?}", option);

    // Option -> Result (добавляем информацию об ошибке)
    let option: Option<f64> = Some(42000.0);
    let result: Result<f64, String> = option.ok_or(String::from("Значение отсутствует"));
    println!("Option -> Result: {:?}", result);

    // Пример в контексте трейдинга
    let price = find_cached_price("BTC")           // Option<f64>
        .ok_or(String::from("Цена не найдена в кеше"));  // Result<f64, String>

    println!("Кешированная цена: {:?}", price);
}

fn find_cached_price(symbol: &str) -> Option<f64> {
    match symbol {
        "BTC" => Some(42000.0),
        "ETH" => Some(2500.0),
        _ => None,
    }
}
```

## Когда использовать Result vs Option

```rust
// Result<T, E> — когда важно знать ПРИЧИНУ отсутствия значения
fn fetch_price_from_api(symbol: &str) -> Result<f64, ApiError> {
    // Может вернуть: NetworkError, TimeoutError, InvalidSymbol, и т.д.
    match symbol {
        "BTC" => Ok(42000.0),
        _ => Err(ApiError::InvalidSymbol(symbol.to_string())),
    }
}

// Option<T> — когда отсутствие значения нормально и не требует объяснения
fn find_cached_price(symbol: &str) -> Option<f64> {
    // Просто нет в кеше — это нормально
    match symbol {
        "BTC" => Some(42000.0),
        "ETH" => Some(2500.0),
        _ => None,
    }
}

#[derive(Debug)]
enum ApiError {
    NetworkError,
    Timeout,
    InvalidSymbol(String),
}

fn main() {
    // Разница в обработке

    // Result: обрабатываем конкретную ошибку
    match fetch_price_from_api("INVALID") {
        Ok(price) => println!("Цена: {}", price),
        Err(ApiError::NetworkError) => println!("Проблемы с сетью, повторите позже"),
        Err(ApiError::Timeout) => println!("Превышено время ожидания"),
        Err(ApiError::InvalidSymbol(s)) => println!("Неизвестный тикер: {}", s),
    }

    // Option: простая проверка наличия
    match find_cached_price("UNKNOWN") {
        Some(price) => println!("Из кеша: {}", price),
        None => println!("Запрашиваем с API..."),
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Ошибка как значение | Ошибки — обычные значения, не исключения |
| Result<T, E> | Успех или ошибка с информацией о причине |
| Option<T> | Есть значение или нет |
| Оператор ? | Пробрасывает ошибку вверх по стеку |
| Явная обработка | Компилятор требует обработать все варианты |
| Комбинаторы | map, unwrap_or, ok_or для преобразований |

## Практические задания

1. Напиши функцию `parse_trade_line(line: &str) -> Result<Trade, ParseError>`, которая парсит строку формата "BTC,0.5,42000.0" в структуру Trade

2. Реализуй функцию `find_trade_by_id(trades: &[Trade], id: u64) -> Option<&Trade>` для поиска сделки по ID

3. Создай цепочку обработки: получение цены → валидация → расчёт позиции, где каждый шаг может вернуть ошибку

## Домашнее задание

1. Реализуй систему обработки ордеров с разными типами ошибок:
   - `InsufficientBalance`
   - `InvalidQuantity`
   - `MarketClosed`
   - `PriceTooFar` (цена отклонилась от рыночной)

2. Напиши функцию `batch_get_prices(symbols: &[&str]) -> Vec<Result<f64, String>>`, которая возвращает цены для списка тикеров, где каждый результат независим

3. Создай обработчик портфеля, который:
   - Собирает все успешные значения
   - Логирует все ошибки
   - Возвращает итоговую статистику

4. Реализуй паттерн "Circuit Breaker": после N неудачных запросов к API функция начинает возвращать закешированные данные

## Навигация

[← Предыдущий день](../118-fail-fast/ru.md) | [Следующий день →](../120-robust-exchange-api-client/ru.md)
