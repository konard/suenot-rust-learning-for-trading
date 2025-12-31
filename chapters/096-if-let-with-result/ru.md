# День 96: if let с Result — когда нужен только успех

## Аналогия из трейдинга

Представь, что ты ждёшь исполнения ордера на бирже. Иногда тебя интересует **только успешный** результат — если ордер исполнился, ты хочешь увидеть детали сделки. Если не исполнился — просто идёшь дальше, без детального разбора причин.

Это как разница между:
- **match** — "Покажи мне всё: и успех, и причину отказа"
- **if let** — "Меня интересует только успех, остальное пропускаем"

## Проблема с match: иногда слишком много кода

Вчера мы учились использовать `match` для обработки `Result`:

```rust
fn main() {
    let order_result = execute_order("BTC/USDT", 0.1, 42000.0);

    match order_result {
        Ok(order_id) => println!("Ордер исполнен! ID: {}", order_id),
        Err(_) => {} // Нам не важна ошибка — просто игнорируем
    }
}

fn execute_order(pair: &str, amount: f64, price: f64) -> Result<String, String> {
    if price > 0.0 && amount > 0.0 {
        Ok(format!("ORD-{}-{}", pair, price as u64))
    } else {
        Err(String::from("Invalid parameters"))
    }
}
```

Видишь `Err(_) => {}`? Это пустая ветка! Мы обязаны её писать из-за exhaustive matching, но она ничего не делает.

## if let: элегантное решение

`if let` позволяет обрабатывать **только один вариант**, игнорируя остальные:

```rust
fn main() {
    let order_result = execute_order("BTC/USDT", 0.1, 42000.0);

    // Обрабатываем ТОЛЬКО успех
    if let Ok(order_id) = order_result {
        println!("Ордер исполнен! ID: {}", order_id);
    }
    // Ошибка? Просто идём дальше
}

fn execute_order(pair: &str, amount: f64, price: f64) -> Result<String, String> {
    if price > 0.0 && amount > 0.0 {
        Ok(format!("ORD-{}-{}", pair, price as u64))
    } else {
        Err(String::from("Invalid parameters"))
    }
}
```

Синтаксис: `if let Ok(переменная) = result { ... }`

## if let с else: обработка обоих случаев

Иногда нужно что-то сделать и при ошибке:

```rust
fn main() {
    let balance_result = get_balance("USDT");

    if let Ok(balance) = balance_result {
        println!("Баланс: ${:.2}", balance);

        if balance > 1000.0 {
            println!("Достаточно для торговли!");
        }
    } else {
        println!("Не удалось получить баланс, используем кеш");
        use_cached_balance();
    }
}

fn get_balance(asset: &str) -> Result<f64, String> {
    // Симуляция запроса к API
    if asset == "USDT" {
        Ok(5000.0)
    } else {
        Err(format!("Unknown asset: {}", asset))
    }
}

fn use_cached_balance() {
    println!("Используется кешированное значение: $4800.00");
}
```

## Получение значения ошибки в else

Если нужно узнать **что за ошибка**, используй `else if let`:

```rust
fn main() {
    let price_result = fetch_price("BTC/USDT");

    if let Ok(price) = price_result {
        println!("Текущая цена BTC: ${:.2}", price);
    } else if let Err(error) = price_result {
        println!("Ошибка получения цены: {}", error);
        // Можем попробовать альтернативный источник
    }
}

fn fetch_price(pair: &str) -> Result<f64, String> {
    Err(String::from("API timeout"))
}
```

Или более идиоматичный вариант:

```rust
fn main() {
    let price_result: Result<f64, String> = fetch_price("BTC/USDT");

    if let Ok(price) = &price_result {
        println!("Текущая цена BTC: ${:.2}", price);
    } else {
        // price_result всё ещё доступен, потому что мы заимствовали
        println!("Ошибка: {:?}", price_result.err().unwrap());
    }
}

fn fetch_price(pair: &str) -> Result<f64, String> {
    Err(String::from("API timeout"))
}
```

## Практические примеры

### Пример 1: Обновление портфеля только при успехе

```rust
fn main() {
    let mut portfolio_value = 10000.0;

    // Пытаемся получить текущие цены
    if let Ok(btc_price) = get_price("BTC") {
        let btc_holding = 0.5;
        portfolio_value += btc_price * btc_holding;
        println!("Добавлена стоимость BTC: ${:.2}", btc_price * btc_holding);
    }

    if let Ok(eth_price) = get_price("ETH") {
        let eth_holding = 10.0;
        portfolio_value += eth_price * eth_holding;
        println!("Добавлена стоимость ETH: ${:.2}", eth_price * eth_holding);
    }

    // SOL недоступен — просто пропускаем
    if let Ok(sol_price) = get_price("SOL") {
        let sol_holding = 100.0;
        portfolio_value += sol_price * sol_holding;
        println!("Добавлена стоимость SOL: ${:.2}", sol_price * sol_holding);
    }

    println!("Общая стоимость портфеля: ${:.2}", portfolio_value);
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(43500.0),
        "ETH" => Ok(2250.0),
        _ => Err(format!("Price not available for {}", symbol)),
    }
}
```

### Пример 2: Проверка ордера перед отправкой

```rust
fn main() {
    let order = Order {
        symbol: String::from("ETH/USDT"),
        side: String::from("BUY"),
        amount: 5.0,
        price: 2200.0,
    };

    // Проверяем и отправляем только если валидно
    if let Ok(validated) = validate_order(&order) {
        println!("Ордер валиден, отправляем...");

        if let Ok(order_id) = send_to_exchange(&validated) {
            println!("Успех! Order ID: {}", order_id);
        } else {
            println!("Не удалось отправить на биржу");
        }
    }
    // Невалидный ордер? Ничего не делаем
}

struct Order {
    symbol: String,
    side: String,
    amount: f64,
    price: f64,
}

struct ValidatedOrder {
    symbol: String,
    side: String,
    amount: f64,
    price: f64,
    timestamp: u64,
}

fn validate_order(order: &Order) -> Result<ValidatedOrder, String> {
    if order.amount <= 0.0 {
        return Err(String::from("Amount must be positive"));
    }
    if order.price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }

    Ok(ValidatedOrder {
        symbol: order.symbol.clone(),
        side: order.side.clone(),
        amount: order.amount,
        price: order.price,
        timestamp: 1234567890,
    })
}

fn send_to_exchange(order: &ValidatedOrder) -> Result<String, String> {
    Ok(format!("ORD-{}-{}", order.symbol, order.timestamp))
}
```

### Пример 3: Условное логирование успешных сделок

```rust
fn main() {
    let trades = vec![
        execute_trade("BTC", 0.1, 43000.0),
        execute_trade("ETH", 0.0, 2200.0),  // Ошибка: нулевой объём
        execute_trade("SOL", 10.0, 95.0),
        execute_trade("DOGE", 1000.0, -0.1), // Ошибка: отрицательная цена
    ];

    println!("=== Успешные сделки ===");
    for (i, trade) in trades.iter().enumerate() {
        if let Ok(details) = trade {
            println!("Сделка #{}: {}", i + 1, details);
        }
    }

    println!("\n=== Неудавшиеся сделки ===");
    for (i, trade) in trades.iter().enumerate() {
        if let Err(error) = trade {
            println!("Сделка #{} провалилась: {}", i + 1, error);
        }
    }
}

fn execute_trade(symbol: &str, amount: f64, price: f64) -> Result<String, String> {
    if amount <= 0.0 {
        return Err(format!("{}: Invalid amount", symbol));
    }
    if price <= 0.0 {
        return Err(format!("{}: Invalid price", symbol));
    }

    Ok(format!("{} {} @ ${:.2}", symbol, amount, price))
}
```

### Пример 4: Парсинг торговых данных

```rust
fn main() {
    let raw_prices = vec!["42500.50", "invalid", "43100.00", "N/A", "42800.75"];

    let mut valid_prices = Vec::new();

    for raw in &raw_prices {
        if let Ok(price) = parse_price(raw) {
            valid_prices.push(price);
        }
        // Невалидные цены просто пропускаем
    }

    if !valid_prices.is_empty() {
        let avg: f64 = valid_prices.iter().sum::<f64>() / valid_prices.len() as f64;
        println!("Валидных цен: {}", valid_prices.len());
        println!("Средняя цена: ${:.2}", avg);
    }
}

fn parse_price(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Cannot parse '{}' as price", s))
}
```

## if let vs match: когда что использовать

```rust
fn main() {
    let result: Result<f64, String> = Ok(42000.0);

    // Используй if let когда:
    // 1. Нужен только один вариант
    if let Ok(price) = result {
        println!("Цена: {}", price);
    }

    // 2. Ошибка не важна — просто пропускаем
    if let Ok(price) = get_optional_indicator() {
        apply_indicator(price);
    }

    // Используй match когда:
    // 1. Нужны оба варианта с разной логикой
    match validate_trade(100.0, 42000.0) {
        Ok(trade) => execute(trade),
        Err(e) => log_error(e),
    }

    // 2. Нужно вернуть значение
    let status = match process_order() {
        Ok(_) => "SUCCESS",
        Err(_) => "FAILED",
    };
    println!("Status: {}", status);
}

fn get_optional_indicator() -> Result<f64, String> {
    Ok(50.0) // RSI value
}

fn apply_indicator(_value: f64) {
    println!("Applying indicator...");
}

fn validate_trade(_amount: f64, _price: f64) -> Result<String, String> {
    Ok(String::from("Trade validated"))
}

fn execute(_trade: String) {
    println!("Executing trade");
}

fn log_error(_e: String) {
    println!("Logging error");
}

fn process_order() -> Result<(), String> {
    Ok(())
}
```

## Цепочки if let

```rust
fn main() {
    // Последовательные проверки
    if let Ok(balance) = get_balance() {
        if let Ok(price) = get_current_price() {
            if let Ok(max_amount) = calculate_max_position(balance, price) {
                println!("Можно купить до {} BTC", max_amount);
            }
        }
    }

    // Это может быть громоздко — тогда лучше использовать ?
    // или комбинаторы (об этом позже)
}

fn get_balance() -> Result<f64, String> {
    Ok(10000.0)
}

fn get_current_price() -> Result<f64, String> {
    Ok(43000.0)
}

fn calculate_max_position(balance: f64, price: f64) -> Result<f64, String> {
    if price > 0.0 {
        Ok(balance / price)
    } else {
        Err(String::from("Invalid price"))
    }
}
```

## Деструктуризация в if let

```rust
fn main() {
    let trade_result = get_trade_details();

    // Деструктуризация кортежа внутри Ok
    if let Ok((symbol, amount, price)) = trade_result {
        let value = amount * price;
        println!("{}: {} @ ${:.2} = ${:.2}", symbol, amount, price, value);
    }
}

fn get_trade_details() -> Result<(String, f64, f64), String> {
    Ok((String::from("BTC"), 0.5, 43000.0))
}
```

## Что мы узнали

| Конструкция | Когда использовать |
|-------------|-------------------|
| `if let Ok(x) = result` | Интересует только успех |
| `if let Err(e) = result` | Интересует только ошибка |
| `if let Ok(x) = result { } else { }` | Два случая, но логика для ошибки простая |
| `match result { Ok => ..., Err => ... }` | Нужна разная логика для обоих случаев |

## Упражнения

### Упражнение 1: Загрузка конфигурации
Напиши программу, которая пытается загрузить конфигурацию. Если успешно — использует её, если нет — применяет значения по умолчанию.

```rust
fn main() {
    // Твой код здесь
    // Используй if let с else для обработки load_config()
}

fn load_config() -> Result<TradingConfig, String> {
    // Реализуй
    todo!()
}

struct TradingConfig {
    max_position_size: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
}
```

### Упражнение 2: Фильтрация валидных ордеров
Дан вектор результатов парсинга ордеров. Используя `if let`, собери только успешно распарсенные ордера.

```rust
fn main() {
    let raw_orders = vec![
        "BUY,BTC,0.5,43000",
        "invalid order",
        "SELL,ETH,10,2200",
        "BUY,SOL,-5,95", // Невалидный: отрицательный объём
    ];

    // Твой код: собери валидные ордера в вектор
}
```

### Упражнение 3: Обновление позиций
Напиши функцию, которая обновляет позиции портфеля. Если получение цены не удалось — позиция не обновляется (но программа продолжает работать).

```rust
fn main() {
    let mut positions = vec![
        Position { symbol: "BTC".to_string(), amount: 0.5, value: 0.0 },
        Position { symbol: "ETH".to_string(), amount: 10.0, value: 0.0 },
        Position { symbol: "UNKNOWN".to_string(), amount: 100.0, value: 0.0 },
    ];

    update_portfolio_values(&mut positions);

    for pos in &positions {
        println!("{}: {} units = ${:.2}", pos.symbol, pos.amount, pos.value);
    }
}

struct Position {
    symbol: String,
    amount: f64,
    value: f64,
}

fn update_portfolio_values(positions: &mut Vec<Position>) {
    // Твой код здесь
    // Используй if let для обновления только тех позиций,
    // для которых удалось получить цену
}

fn get_price(symbol: &str) -> Result<f64, String> {
    match symbol {
        "BTC" => Ok(43000.0),
        "ETH" => Ok(2200.0),
        _ => Err(format!("Unknown symbol: {}", symbol)),
    }
}
```

## Домашнее задание

1. Создай систему мониторинга нескольких бирж. Используй `if let` для обработки ответов — если биржа ответила, выводи данные; если нет — пропускай без ошибок.

2. Реализуй функцию расчёта среднего PnL, которая принимает вектор `Result<f64, String>` (результаты сделок) и возвращает среднее только по успешным сделкам.

3. Напиши парсер торгового лога, который читает строки формата "TIMESTAMP,SYMBOL,SIDE,AMOUNT,PRICE" и собирает только валидные записи, игнорируя битые строки.

4. Создай функцию автоматического ребалансировщика портфеля, которая использует `if let` для проверки каждого этапа: получение балансов → расчёт долей → генерация ордеров.

## Навигация

[← Предыдущий день](../095-match-on-result/ru.md) | [Следующий день →](../097-while-let/ru.md)
