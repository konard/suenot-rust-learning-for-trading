# День 77: Оператор ? — пробрасываем ошибку вверх

## Аналогия из трейдинга

Представь цепочку операций на бирже: получить данные с API → распарсить JSON → проверить цену → создать ордер. Если любой шаг провалился, нужно немедленно прервать всю цепочку и сообщить об ошибке. Оператор `?` — это как **автоматический стоп-лосс** для операций: при первой ошибке выполнение функции прекращается, и ошибка "всплывает" наверх.

## Проблема без оператора ?

Без `?` код превращается в лестницу из match:

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_portfolio_verbose() -> Result<String, io::Error> {
    let file_result = File::open("portfolio.json");

    let mut file = match file_result {
        Ok(f) => f,
        Err(e) => return Err(e),  // Если ошибка — возвращаем её
    };

    let mut contents = String::new();

    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => Err(e),
    }
}
```

Много повторяющегося кода! Каждый `Result` требует явной обработки.

## Оператор ? — элегантное решение

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_portfolio() -> Result<String, io::Error> {
    let mut file = File::open("portfolio.json")?;  // ? разворачивает Ok или возвращает Err
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    match read_portfolio() {
        Ok(data) => println!("Portfolio: {}", data),
        Err(e) => println!("Error: {}", e),
    }
}
```

**Оператор `?`** делает следующее:
- Если `Result` — это `Ok(value)`, извлекает `value` и продолжает выполнение
- Если `Result` — это `Err(e)`, немедленно возвращает `Err(e)` из функции

## Как работает ? под капотом

```rust
// Это:
let file = File::open("data.json")?;

// Эквивалентно этому:
let file = match File::open("data.json") {
    Ok(f) => f,
    Err(e) => return Err(e.into()),  // .into() для преобразования типов ошибок
};
```

## Цепочка операций с ?

```rust
use std::io::{self, Read};
use std::fs::File;

fn load_trading_config() -> Result<String, io::Error> {
    let mut contents = String::new();
    File::open("config.json")?
        .read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    match load_trading_config() {
        Ok(config) => println!("Config loaded: {} bytes", config.len()),
        Err(e) => eprintln!("Failed to load config: {}", e),
    }
}
```

## Оператор ? с Option

Оператор `?` работает и с `Option`:

```rust
fn get_best_bid(order_book: &[(f64, f64)]) -> Option<f64> {
    let (price, _quantity) = order_book.first()?;  // Вернёт None, если пусто
    Some(*price)
}

fn calculate_spread(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> Option<f64> {
    let best_bid = get_best_bid(bids)?;
    let best_ask = get_best_bid(asks)?;  // Переиспользуем функцию
    Some(best_ask - best_bid)
}

fn main() {
    let bids = [(42000.0, 1.5), (41990.0, 2.0)];
    let asks = [(42010.0, 1.0), (42020.0, 3.0)];

    match calculate_spread(&bids, &asks) {
        Some(spread) => println!("Spread: ${:.2}", spread),
        None => println!("Cannot calculate spread"),
    }

    // Пустой стакан
    let empty: [(f64, f64); 0] = [];
    println!("Empty spread: {:?}", calculate_spread(&empty, &asks));
}
```

## Практический пример: загрузка и парсинг данных

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
}

fn parse_trade_line(line: &str) -> Result<Trade, String> {
    let parts: Vec<&str> = line.split(',').collect();

    if parts.len() != 3 {
        return Err(format!("Invalid format: expected 3 fields, got {}", parts.len()));
    }

    let symbol = parts[0].to_string();
    let price = parts[1].parse::<f64>()
        .map_err(|_| format!("Invalid price: {}", parts[1]))?;
    let quantity = parts[2].parse::<f64>()
        .map_err(|_| format!("Invalid quantity: {}", parts[2]))?;

    Ok(Trade { symbol, price, quantity })
}

fn load_trades(filename: &str) -> Result<Vec<Trade>, String> {
    let file = File::open(filename)
        .map_err(|e| format!("Cannot open file: {}", e))?;

    let reader = BufReader::new(file);
    let mut trades = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result
            .map_err(|e| format!("Error reading line {}: {}", line_num + 1, e))?;

        // Пропускаем пустые строки и комментарии
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let trade = parse_trade_line(&line)
            .map_err(|e| format!("Line {}: {}", line_num + 1, e))?;

        trades.push(trade);
    }

    Ok(trades)
}

fn main() {
    match load_trades("trades.csv") {
        Ok(trades) => {
            println!("Loaded {} trades:", trades.len());
            for trade in &trades {
                println!("  {:?}", trade);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Преобразование типов ошибок с ?

Оператор `?` автоматически преобразует типы ошибок через трейт `From`:

```rust
use std::fs::File;
use std::io::{self, Read};
use std::num::ParseFloatError;

#[derive(Debug)]
enum TradingError {
    IoError(io::Error),
    ParseError(ParseFloatError),
    ValidationError(String),
}

// Реализуем From для автоматического преобразования
impl From<io::Error> for TradingError {
    fn from(err: io::Error) -> Self {
        TradingError::IoError(err)
    }
}

impl From<ParseFloatError> for TradingError {
    fn from(err: ParseFloatError) -> Self {
        TradingError::ParseError(err)
    }
}

fn read_and_parse_price(filename: &str) -> Result<f64, TradingError> {
    let mut contents = String::new();
    File::open(filename)?.read_to_string(&mut contents)?;  // io::Error -> TradingError

    let price: f64 = contents.trim().parse()?;  // ParseFloatError -> TradingError

    if price <= 0.0 {
        return Err(TradingError::ValidationError(
            "Price must be positive".to_string()
        ));
    }

    Ok(price)
}

fn main() {
    match read_and_parse_price("btc_price.txt") {
        Ok(price) => println!("BTC Price: ${:.2}", price),
        Err(TradingError::IoError(e)) => eprintln!("IO Error: {}", e),
        Err(TradingError::ParseError(e)) => eprintln!("Parse Error: {}", e),
        Err(TradingError::ValidationError(msg)) => eprintln!("Validation: {}", msg),
    }
}
```

## ? в функции main

Можно использовать `?` прямо в `main`, если указать возвращаемый тип:

```rust
use std::fs::File;
use std::io::{self, Read};

fn main() -> Result<(), io::Error> {
    let mut file = File::open("config.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("Config: {}", contents);
    Ok(())
}
```

При ошибке программа выведет сообщение и завершится с ненулевым кодом.

## Комбинирование ? с методами Result

```rust
fn fetch_price(symbol: &str) -> Result<f64, String> {
    // Симуляция API-запроса
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2800.0),
        _ => Err(format!("Unknown symbol: {}", symbol)),
    }
}

fn calculate_portfolio_value(holdings: &[(&str, f64)]) -> Result<f64, String> {
    let mut total = 0.0;

    for (symbol, quantity) in holdings {
        let price = fetch_price(symbol)?;
        total += price * quantity;
    }

    Ok(total)
}

fn get_portfolio_with_margin(holdings: &[(&str, f64)], margin: f64) -> Result<f64, String> {
    let value = calculate_portfolio_value(holdings)?;

    if margin < 0.0 || margin > 1.0 {
        return Err("Margin must be between 0 and 1".to_string());
    }

    Ok(value * (1.0 + margin))
}

fn main() {
    let holdings = [("BTC", 0.5), ("ETH", 2.0)];

    match get_portfolio_with_margin(&holdings, 0.1) {
        Ok(value) => println!("Portfolio value with 10% margin: ${:.2}", value),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Паттерн: ранний возврат с проверкой

```rust
fn execute_trade(
    symbol: &str,
    side: &str,
    quantity: f64,
    price: f64,
    balance: f64,
) -> Result<String, String> {
    // Валидация с ранним возвратом
    if quantity <= 0.0 {
        return Err("Quantity must be positive".to_string());
    }

    if price <= 0.0 {
        return Err("Price must be positive".to_string());
    }

    let cost = quantity * price;

    if side == "BUY" && cost > balance {
        return Err(format!(
            "Insufficient balance: need ${:.2}, have ${:.2}",
            cost, balance
        ));
    }

    // Основная логика
    Ok(format!(
        "Executed {} {} {} @ ${:.2} (total: ${:.2})",
        side, quantity, symbol, price, cost
    ))
}

fn process_orders(orders: &[(&str, &str, f64, f64)], balance: f64) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    let mut remaining_balance = balance;

    for (symbol, side, qty, price) in orders {
        let result = execute_trade(symbol, side, *qty, *price, remaining_balance)?;

        if *side == "BUY" {
            remaining_balance -= qty * price;
        } else {
            remaining_balance += qty * price;
        }

        results.push(result);
    }

    Ok(results)
}

fn main() {
    let orders = [
        ("BTC", "BUY", 0.1, 42000.0),
        ("ETH", "BUY", 1.0, 2800.0),
        ("BTC", "SELL", 0.05, 42500.0),
    ];

    match process_orders(&orders, 10000.0) {
        Ok(results) => {
            println!("All orders executed:");
            for r in results {
                println!("  {}", r);
            }
        }
        Err(e) => eprintln!("Order processing failed: {}", e),
    }
}
```

## Сравнение подходов

```rust
// ❌ Без ? — много boilerplate
fn without_question_mark() -> Result<i32, String> {
    let a = match step_one() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let b = match step_two(a) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    match step_three(b) {
        Ok(v) => Ok(v),
        Err(e) => Err(e),
    }
}

// ✅ С ? — чистый и понятный код
fn with_question_mark() -> Result<i32, String> {
    let a = step_one()?;
    let b = step_two(a)?;
    step_three(b)
}

fn step_one() -> Result<i32, String> { Ok(1) }
fn step_two(x: i32) -> Result<i32, String> { Ok(x + 1) }
fn step_three(x: i32) -> Result<i32, String> { Ok(x * 2) }

fn main() {
    println!("Result: {:?}", with_question_mark());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `?` с Result | Извлекает Ok или возвращает Err |
| `?` с Option | Извлекает Some или возвращает None |
| Цепочка `?` | Несколько операций подряд |
| From трейт | Автоматическое преобразование ошибок |
| `main() -> Result` | Использование ? в main |

## Домашнее задание

1. Напиши функцию `load_and_validate_orders(filename: &str) -> Result<Vec<Order>, String>`, которая читает файл с ордерами, парсит каждую строку и валидирует данные. Используй оператор `?` на каждом шаге.

2. Создай цепочку из трёх функций для обработки торговых данных: `fetch_data() -> Result<String, ApiError>`, `parse_data(data: &str) -> Result<Vec<Trade>, ParseError>`, `analyze_trades(trades: &[Trade]) -> Result<Report, AnalysisError>`. Объедини их в одну функцию с общим типом ошибки.

3. Реализуй функцию `calculate_portfolio_stats(filename: &str) -> Result<PortfolioStats, Box<dyn std::error::Error>>`, которая читает файл с позициями, парсит данные и вычисляет статистику (общая стоимость, PnL, процентные доли).

4. Напиши программу, которая использует `?` в `main()` для: чтения конфигурации, загрузки данных рынка, выполнения анализа и вывода результатов.

## Навигация

[← Предыдущий день](../076-result-methods/ru.md) | [Следующий день →](../078-custom-error-types/ru.md)
