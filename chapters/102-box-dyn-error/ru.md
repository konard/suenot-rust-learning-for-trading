# День 102: Box<dyn Error> — Любая ошибка

## Аналогия из трейдинга

Представь, что ты управляешь торговой системой, которая получает данные из **разных источников**: биржевой API, файлы с историческими данными, база данных позиций, сетевые потоки ордеров. Каждый источник может выдать **свой тип ошибки**: сетевую ошибку, ошибку парсинга JSON, ошибку чтения файла, ошибку базы данных.

Вместо того чтобы обрабатывать каждый тип отдельно, тебе нужен **универсальный контейнер для любой ошибки** — как единая панель мониторинга, которая показывает проблемы из всех источников в одном формате.

`Box<dyn Error>` — это именно такой контейнер. Он может хранить **любую ошибку**, реализующую трейт `Error`.

## Зачем нужен Box<dyn Error>?

### Проблема: разные типы ошибок

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseFloatError;

// Эта функция может вернуть разные типы ошибок
fn load_prices_problem(path: &str) -> Vec<f64> {
    // io::Error при чтении файла
    // ParseFloatError при парсинге цены
    // Как вернуть Result с обоими типами?
    vec![]
}
```

### Решение: Box<dyn Error>

```rust
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn load_prices(path: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let file = File::open(path)?;  // io::Error автоматически конвертируется
    let reader = BufReader::new(file);
    let mut prices = Vec::new();

    for line in reader.lines() {
        let line = line?;  // io::Error
        let price: f64 = line.trim().parse()?;  // ParseFloatError
        prices.push(price);
    }

    Ok(prices)
}

fn main() {
    match load_prices("prices.txt") {
        Ok(prices) => println!("Loaded {} prices", prices.len()),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Как это работает

`Box<dyn Error>` состоит из двух частей:

1. **Box** — умный указатель, который хранит данные в куче
2. **dyn Error** — динамический трейт-объект, представляющий любой тип с трейтом `Error`

```rust
use std::error::Error;
use std::fmt;

// Пример создания собственной ошибки
#[derive(Debug)]
struct InsufficientBalanceError {
    required: f64,
    available: f64,
}

impl fmt::Display for InsufficientBalanceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Insufficient balance: need ${:.2}, have ${:.2}",
            self.required, self.available
        )
    }
}

impl Error for InsufficientBalanceError {}

fn check_balance(required: f64, available: f64) -> Result<(), Box<dyn Error>> {
    if required > available {
        return Err(Box::new(InsufficientBalanceError { required, available }));
    }
    Ok(())
}

fn main() {
    match check_balance(10000.0, 5000.0) {
        Ok(()) => println!("Balance OK"),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Практический пример: загрузка торговых данных

```rust
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

fn parse_trade(line: &str) -> Result<Trade, Box<dyn Error>> {
    let parts: Vec<&str> = line.split(',').collect();

    if parts.len() != 4 {
        return Err(format!("Invalid trade format: expected 4 fields, got {}", parts.len()).into());
    }

    Ok(Trade {
        symbol: parts[0].trim().to_string(),
        price: parts[1].trim().parse()?,
        quantity: parts[2].trim().parse()?,
        side: parts[3].trim().to_string(),
    })
}

fn load_trades(path: &str) -> Result<Vec<Trade>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut trades = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;  // Пропускаем пустые строки и комментарии
        }

        match parse_trade(&line) {
            Ok(trade) => trades.push(trade),
            Err(e) => {
                return Err(format!("Error on line {}: {}", line_num + 1, e).into());
            }
        }
    }

    Ok(trades)
}

fn calculate_portfolio_value(trades: &[Trade]) -> f64 {
    trades.iter()
        .filter(|t| t.side == "BUY")
        .map(|t| t.price * t.quantity)
        .sum()
}

fn main() -> Result<(), Box<dyn Error>> {
    let trades = load_trades("trades.csv")?;

    println!("Loaded {} trades", trades.len());
    println!("Portfolio value: ${:.2}", calculate_portfolio_value(&trades));

    for trade in &trades {
        println!(
            "  {} {} {} @ ${:.2}",
            trade.side, trade.quantity, trade.symbol, trade.price
        );
    }

    Ok(())
}
```

## Конвертация строки в Box<dyn Error>

```rust
use std::error::Error;

fn validate_order(
    symbol: &str,
    price: f64,
    quantity: f64,
) -> Result<(), Box<dyn Error>> {
    if symbol.is_empty() {
        return Err("Symbol cannot be empty".into());  // &str -> Box<dyn Error>
    }

    if price <= 0.0 {
        return Err(format!("Invalid price: {}", price).into());  // String -> Box<dyn Error>
    }

    if quantity <= 0.0 {
        return Err("Quantity must be positive".into());
    }

    Ok(())
}

fn main() {
    match validate_order("", 100.0, 10.0) {
        Ok(()) => println!("Order valid"),
        Err(e) => println!("Validation error: {}", e),
    }
}
```

## Работа с несколькими источниками данных

```rust
use std::error::Error;
use std::collections::HashMap;

// Имитация разных источников данных
fn fetch_price_from_exchange(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // Может вернуть сетевую ошибку
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2800.0),
        _ => Err(format!("Unknown symbol: {}", symbol).into()),
    }
}

fn fetch_price_from_cache(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // Может вернуть ошибку кэша
    let cache: HashMap<&str, f64> = [("BTC", 41950.0), ("ETH", 2790.0)]
        .into_iter()
        .collect();

    cache
        .get(symbol)
        .copied()
        .ok_or_else(|| format!("Symbol {} not in cache", symbol).into())
}

fn fetch_price_from_file(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // Может вернуть io::Error или ParseFloatError
    // Упрощённая версия
    match symbol {
        "BTC" => Ok(41900.0),
        _ => Err("Price file not found".into()),
    }
}

fn get_best_price(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // Пробуем разные источники с разными типами ошибок
    let exchange_result = fetch_price_from_exchange(symbol);
    let cache_result = fetch_price_from_cache(symbol);
    let file_result = fetch_price_from_file(symbol);

    // Собираем все успешные результаты
    let mut prices = Vec::new();

    if let Ok(p) = exchange_result { prices.push(("exchange", p)); }
    if let Ok(p) = cache_result { prices.push(("cache", p)); }
    if let Ok(p) = file_result { prices.push(("file", p)); }

    if prices.is_empty() {
        return Err(format!("No price available for {}", symbol).into());
    }

    // Возвращаем среднюю цену
    let avg: f64 = prices.iter().map(|(_, p)| p).sum::<f64>() / prices.len() as f64;

    println!("Price sources for {}:", symbol);
    for (source, price) in &prices {
        println!("  {}: ${:.2}", source, price);
    }
    println!("  Average: ${:.2}", avg);

    Ok(avg)
}

fn main() -> Result<(), Box<dyn Error>> {
    let btc_price = get_best_price("BTC")?;
    println!("\nFinal BTC price: ${:.2}", btc_price);

    let eth_price = get_best_price("ETH")?;
    println!("Final ETH price: ${:.2}", eth_price);

    // Это вернёт ошибку
    match get_best_price("UNKNOWN") {
        Ok(p) => println!("Price: {}", p),
        Err(e) => println!("\nError: {}", e),
    }

    Ok(())
}
```

## Оператор ? с Box<dyn Error>

```rust
use std::error::Error;

fn process_order(
    symbol: &str,
    side: &str,
    quantity: f64,
    balance: f64,
) -> Result<f64, Box<dyn Error>> {
    // Каждая строка с ? может вернуть разный тип ошибки
    validate_symbol(symbol)?;           // Ошибка валидации
    validate_side(side)?;               // Ошибка валидации
    let price = get_market_price(symbol)?;  // Сетевая/данные
    let cost = calculate_cost(price, quantity)?;  // Вычисления
    check_sufficient_balance(cost, balance)?;     // Бизнес-логика

    Ok(cost)
}

fn validate_symbol(symbol: &str) -> Result<(), Box<dyn Error>> {
    if symbol.len() < 2 || symbol.len() > 10 {
        return Err("Symbol must be 2-10 characters".into());
    }
    Ok(())
}

fn validate_side(side: &str) -> Result<(), Box<dyn Error>> {
    match side {
        "BUY" | "SELL" => Ok(()),
        _ => Err(format!("Invalid side: {}. Must be BUY or SELL", side).into()),
    }
}

fn get_market_price(symbol: &str) -> Result<f64, Box<dyn Error>> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2800.0),
        "SOL" => Ok(95.0),
        _ => Err(format!("No market data for {}", symbol).into()),
    }
}

fn calculate_cost(price: f64, quantity: f64) -> Result<f64, Box<dyn Error>> {
    if quantity <= 0.0 {
        return Err("Quantity must be positive".into());
    }
    Ok(price * quantity)
}

fn check_sufficient_balance(cost: f64, balance: f64) -> Result<(), Box<dyn Error>> {
    if cost > balance {
        return Err(format!(
            "Insufficient balance: need ${:.2}, have ${:.2}",
            cost, balance
        ).into());
    }
    Ok(())
}

fn main() {
    let test_cases = [
        ("BTC", "BUY", 0.5, 50000.0),
        ("ETH", "SELL", 10.0, 1000.0),
        ("X", "BUY", 1.0, 1000.0),      // Неверный символ
        ("BTC", "HOLD", 1.0, 50000.0),  // Неверная сторона
        ("BTC", "BUY", 1.0, 100.0),     // Недостаточно средств
    ];

    for (symbol, side, qty, balance) in test_cases {
        println!("\nOrder: {} {} {} with balance ${:.2}", side, qty, symbol, balance);
        match process_order(symbol, side, qty, balance) {
            Ok(cost) => println!("  ✓ Order cost: ${:.2}", cost),
            Err(e) => println!("  ✗ Error: {}", e),
        }
    }
}
```

## main() с Box<dyn Error>

```rust
use std::error::Error;

fn run_trading_bot() -> Result<(), Box<dyn Error>> {
    println!("Starting trading bot...");

    // Инициализация
    let config = load_config()?;
    let positions = load_positions()?;

    println!("Config loaded: {} pairs", config.len());
    println!("Positions loaded: {} open", positions.len());

    // Торговая логика
    for symbol in &config {
        let price = fetch_price(symbol)?;
        println!("{}: ${:.2}", symbol, price);
    }

    println!("Trading bot finished successfully");
    Ok(())
}

fn load_config() -> Result<Vec<String>, Box<dyn Error>> {
    Ok(vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()])
}

fn load_positions() -> Result<Vec<(String, f64)>, Box<dyn Error>> {
    Ok(vec![
        ("BTC".to_string(), 0.5),
        ("ETH".to_string(), 5.0),
    ])
}

fn fetch_price(symbol: &str) -> Result<f64, Box<dyn Error>> {
    match symbol {
        "BTC" => Ok(42000.0),
        "ETH" => Ok(2800.0),
        "SOL" => Ok(95.0),
        _ => Err(format!("Unknown symbol: {}", symbol).into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    run_trading_bot()
}
```

## Сравнение подходов обработки ошибок

| Подход | Когда использовать | Пример |
|--------|-------------------|--------|
| `Result<T, ConcreteError>` | Один известный тип ошибки | `Result<f64, ParseFloatError>` |
| `Result<T, String>` | Простые сообщения об ошибках | `Result<f64, String>` |
| `Result<T, Box<dyn Error>>` | Разные типы ошибок | Чтение файла + парсинг |
| Custom enum | Полный контроль над ошибками | `enum TradingError { ... }` |

## Ограничения Box<dyn Error>

```rust
use std::error::Error;

fn example() -> Result<(), Box<dyn Error>> {
    // Box<dyn Error> стирает конкретный тип ошибки
    // Мы не можем сделать pattern matching по типу

    let result: Result<i32, Box<dyn Error>> = Err("some error".into());

    match result {
        Ok(v) => println!("Value: {}", v),
        Err(e) => {
            // Можем только получить сообщение
            println!("Error: {}", e);

            // Можем проверить source (если есть)
            if let Some(source) = e.source() {
                println!("Caused by: {}", source);
            }
        }
    }

    Ok(())
}

fn main() {
    let _ = example();
}
```

## Упражнения

### Упражнение 1: Парсер биржевых данных

```rust
use std::error::Error;

// TODO: Реализуй функцию, которая парсит строку формата
// "BTC,42000.50,0.5,BUY,2024-01-15T10:30:00"
// и возвращает структуру Trade

struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
    timestamp: String,
}

fn parse_trade_line(line: &str) -> Result<Trade, Box<dyn Error>> {
    // Твой код здесь
    todo!()
}

fn main() -> Result<(), Box<dyn Error>> {
    let line = "BTC,42000.50,0.5,BUY,2024-01-15T10:30:00";
    let trade = parse_trade_line(line)?;
    println!("Parsed: {} {} {} @ {}",
        trade.side, trade.quantity, trade.symbol, trade.price);
    Ok(())
}
```

### Упражнение 2: Мульти-источник данных

```rust
use std::error::Error;

// TODO: Реализуй функцию, которая пробует получить цену из нескольких
// источников и возвращает первый успешный результат

fn get_price_with_fallback(symbol: &str) -> Result<f64, Box<dyn Error>> {
    // Источник 1: API (может упасть)
    // Источник 2: Кэш (может быть устаревшим)
    // Источник 3: Файл (может не существовать)
    // Если все упали — вернуть ошибку
    todo!()
}

fn main() {
    match get_price_with_fallback("BTC") {
        Ok(price) => println!("Got price: ${:.2}", price),
        Err(e) => println!("All sources failed: {}", e),
    }
}
```

### Упражнение 3: Валидатор портфеля

```rust
use std::error::Error;

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

// TODO: Реализуй функцию валидации портфеля, которая проверяет:
// 1. Все символы валидны (2-10 символов, только буквы)
// 2. Все количества положительные
// 3. Все цены положительные
// 4. Общая стоимость не превышает лимит

fn validate_portfolio(
    positions: &[Position],
    max_value: f64,
) -> Result<f64, Box<dyn Error>> {
    todo!()
}

fn main() -> Result<(), Box<dyn Error>> {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 0.5, entry_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 10.0, entry_price: 2800.0 },
    ];

    let total = validate_portfolio(&portfolio, 100000.0)?;
    println!("Portfolio value: ${:.2}", total);
    Ok(())
}
```

## Домашнее задание

1. **Загрузчик конфигурации**: Напиши функцию `load_trading_config(path: &str) -> Result<TradingConfig, Box<dyn Error>>`, которая читает JSON/TOML файл конфигурации и валидирует все поля.

2. **Обработчик ордеров**: Создай систему обработки ордеров, где функция `execute_order(order: Order) -> Result<ExecutionReport, Box<dyn Error>>` может вернуть ошибки валидации, сетевые ошибки, ошибки недостатка баланса.

3. **Агрегатор рыночных данных**: Реализуй функцию `aggregate_market_data(symbols: &[&str]) -> Result<MarketSnapshot, Box<dyn Error>>`, которая собирает данные из нескольких источников и обрабатывает частичные сбои.

4. **Риск-менеджер**: Напиши `check_risk_limits(portfolio: &Portfolio, new_order: &Order) -> Result<(), Box<dyn Error>>`, который проверяет лимиты позиций, exposure, drawdown и возвращает понятные сообщения об ошибках.

## Что мы узнали

- `Box<dyn Error>` позволяет возвращать любой тип ошибки
- Оператор `?` автоматически конвертирует ошибки в `Box<dyn Error>`
- Строки легко конвертируются через `.into()`
- `main()` может возвращать `Result<(), Box<dyn Error>>`
- Это удобно для прототипов и скриптов, но для production лучше использовать конкретные типы ошибок

## Навигация

[← Предыдущий день](../101-custom-error-types/ru.md) | [Следующий день →](../103-thiserror-crate/ru.md)
