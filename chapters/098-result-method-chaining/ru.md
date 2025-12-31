# День 98: Цепочки методов Result

## Аналогия из трейдинга

Представь процесс исполнения сложного ордера на бирже: сначала нужно проверить баланс, затем валидировать цену, потом проверить лимиты, и наконец разместить ордер. Каждый шаг может провалиться. Вместо написания вложенных `match` блоков для каждой проверки, мы можем **связать** операции в цепочку — если любой шаг провалится, вся цепочка остановится и вернёт ошибку.

Это как конвейер на заводе: если деталь бракованная на любом этапе, она снимается с конвейера и не идёт дальше.

## Базовые методы Result для цепочек

### `and_then` — продолжить при успехе

```rust
fn main() {
    let result = parse_price("42000.50")
        .and_then(|price| validate_price(price))
        .and_then(|price| check_balance(price, 50000.0));

    match result {
        Ok(price) => println!("Цена валидна: ${:.2}", price),
        Err(e) => println!("Ошибка: {}", e),
    }
}

fn parse_price(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Невозможно распарсить '{}' как цену", s))
}

fn validate_price(price: f64) -> Result<f64, String> {
    if price <= 0.0 {
        Err(String::from("Цена должна быть положительной"))
    } else if price > 1_000_000.0 {
        Err(String::from("Цена слишком высокая"))
    } else {
        Ok(price)
    }
}

fn check_balance(price: f64, balance: f64) -> Result<f64, String> {
    if price > balance {
        Err(String::from("Недостаточно средств"))
    } else {
        Ok(price)
    }
}
```

### `map` — преобразовать успешное значение

```rust
fn main() {
    let result = parse_quantity("100")
        .map(|qty| qty * 1.1)  // Добавить 10% запас
        .map(|qty| qty.ceil() as u64);  // Округлить вверх

    println!("Количество с запасом: {:?}", result);
}

fn parse_quantity(s: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Невозможно распарсить '{}'", s))
}
```

### `map_err` — преобразовать ошибку

```rust
fn main() {
    let result: Result<f64, TradingError> = "invalid"
        .parse::<f64>()
        .map_err(|_| TradingError::InvalidPrice);

    match result {
        Ok(price) => println!("Цена: {}", price),
        Err(e) => println!("Ошибка: {:?}", e),
    }
}

#[derive(Debug)]
enum TradingError {
    InvalidPrice,
    InsufficientFunds,
    OrderRejected,
}
```

## Полный пайплайн обработки ордера

```rust
fn main() {
    // Успешный сценарий
    let order_result = process_order("42000.50", "0.5", 25000.0);
    println!("Результат 1: {:?}", order_result);

    // Ошибка парсинга
    let order_result = process_order("invalid", "0.5", 25000.0);
    println!("Результат 2: {:?}", order_result);

    // Недостаточно средств
    let order_result = process_order("42000.50", "0.5", 1000.0);
    println!("Результат 3: {:?}", order_result);
}

#[derive(Debug)]
struct Order {
    price: f64,
    quantity: f64,
    total: f64,
    fee: f64,
}

#[derive(Debug)]
enum OrderError {
    ParseError(String),
    ValidationError(String),
    InsufficientFunds { required: f64, available: f64 },
    RiskLimitExceeded,
}

fn process_order(price_str: &str, qty_str: &str, balance: f64) -> Result<Order, OrderError> {
    parse_order_params(price_str, qty_str)
        .and_then(|(price, qty)| validate_order(price, qty))
        .and_then(|(price, qty)| check_funds(price, qty, balance))
        .and_then(|(price, qty)| check_risk_limits(price, qty))
        .map(|(price, qty)| create_order(price, qty))
}

fn parse_order_params(price_str: &str, qty_str: &str) -> Result<(f64, f64), OrderError> {
    let price = price_str.parse::<f64>()
        .map_err(|_| OrderError::ParseError(format!("Неверная цена: {}", price_str)))?;

    let quantity = qty_str.parse::<f64>()
        .map_err(|_| OrderError::ParseError(format!("Неверное количество: {}", qty_str)))?;

    Ok((price, quantity))
}

fn validate_order(price: f64, quantity: f64) -> Result<(f64, f64), OrderError> {
    if price <= 0.0 {
        return Err(OrderError::ValidationError("Цена должна быть положительной".into()));
    }
    if quantity <= 0.0 {
        return Err(OrderError::ValidationError("Количество должно быть положительным".into()));
    }
    if quantity < 0.001 {
        return Err(OrderError::ValidationError("Минимальное количество: 0.001".into()));
    }
    Ok((price, quantity))
}

fn check_funds(price: f64, quantity: f64, balance: f64) -> Result<(f64, f64), OrderError> {
    let required = price * quantity * 1.001;  // +0.1% на комиссию
    if required > balance {
        Err(OrderError::InsufficientFunds { required, available: balance })
    } else {
        Ok((price, quantity))
    }
}

fn check_risk_limits(price: f64, quantity: f64) -> Result<(f64, f64), OrderError> {
    let position_value = price * quantity;
    if position_value > 100_000.0 {
        Err(OrderError::RiskLimitExceeded)
    } else {
        Ok((price, quantity))
    }
}

fn create_order(price: f64, quantity: f64) -> Order {
    let total = price * quantity;
    let fee = total * 0.001;
    Order { price, quantity, total, fee }
}
```

## Комбинаторы для сложных сценариев

### `or_else` — попробовать альтернативу при ошибке

```rust
fn main() {
    // Пробуем получить цену из разных источников
    let price = get_price_from_exchange("binance")
        .or_else(|_| get_price_from_exchange("coinbase"))
        .or_else(|_| get_cached_price());

    println!("Цена: {:?}", price);
}

fn get_price_from_exchange(exchange: &str) -> Result<f64, String> {
    match exchange {
        "binance" => Err("Binance недоступен".into()),
        "coinbase" => Ok(42150.0),
        _ => Err("Неизвестная биржа".into()),
    }
}

fn get_cached_price() -> Result<f64, String> {
    Ok(42000.0)  // Кешированная цена как fallback
}
```

### `unwrap_or_else` — значение по умолчанию с ленивым вычислением

```rust
fn main() {
    let price = parse_price("invalid")
        .unwrap_or_else(|_| get_default_price());

    println!("Цена: {}", price);
}

fn parse_price(s: &str) -> Result<f64, String> {
    s.parse().map_err(|_| "Ошибка парсинга".into())
}

fn get_default_price() -> f64 {
    println!("Получаем цену по умолчанию...");
    42000.0
}
```

### Комбинирование с `?` оператором

```rust
fn main() {
    match execute_trading_strategy() {
        Ok(profit) => println!("Прибыль: ${:.2}", profit),
        Err(e) => println!("Стратегия провалилась: {}", e),
    }
}

fn execute_trading_strategy() -> Result<f64, String> {
    let prices = fetch_prices()?;
    let signal = analyze_prices(&prices)?;
    let order = create_trade_order(signal)?;
    let result = execute_order(&order)?;
    Ok(result)
}

fn fetch_prices() -> Result<Vec<f64>, String> {
    Ok(vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0])
}

fn analyze_prices(prices: &[f64]) -> Result<TradeSignal, String> {
    if prices.is_empty() {
        return Err("Нет данных для анализа".into());
    }

    let avg: f64 = prices.iter().sum::<f64>() / prices.len() as f64;
    let last = prices.last().unwrap();

    if *last > avg * 1.01 {
        Ok(TradeSignal::Buy { price: *last })
    } else if *last < avg * 0.99 {
        Ok(TradeSignal::Sell { price: *last })
    } else {
        Err("Нет чёткого сигнала".into())
    }
}

#[derive(Debug)]
enum TradeSignal {
    Buy { price: f64 },
    Sell { price: f64 },
}

fn create_trade_order(signal: TradeSignal) -> Result<TradeOrder, String> {
    match signal {
        TradeSignal::Buy { price } => Ok(TradeOrder {
            side: "BUY".into(),
            price,
            quantity: 0.1,
        }),
        TradeSignal::Sell { price } => Ok(TradeOrder {
            side: "SELL".into(),
            price,
            quantity: 0.1,
        }),
    }
}

#[derive(Debug)]
struct TradeOrder {
    side: String,
    price: f64,
    quantity: f64,
}

fn execute_order(order: &TradeOrder) -> Result<f64, String> {
    println!("Исполняем ордер: {:?}", order);
    // Симуляция: небольшая прибыль
    Ok(order.price * order.quantity * 0.02)
}
```

## Практический пример: Анализ портфеля

```rust
fn main() {
    let portfolio_data = vec![
        ("BTC", "42000.50", "0.5"),
        ("ETH", "2200.00", "5.0"),
        ("INVALID", "abc", "1.0"),  // Ошибочные данные
    ];

    let results: Vec<_> = portfolio_data
        .iter()
        .map(|(symbol, price, qty)| {
            analyze_position(symbol, price, qty)
        })
        .collect();

    for (symbol, result) in portfolio_data.iter().zip(results.iter()) {
        match result {
            Ok(analysis) => println!("{}: ${:.2}", symbol.0, analysis.value),
            Err(e) => println!("{}: Ошибка - {}", symbol.0, e),
        }
    }
}

#[derive(Debug)]
struct PositionAnalysis {
    symbol: String,
    price: f64,
    quantity: f64,
    value: f64,
    weight: f64,
}

fn analyze_position(symbol: &str, price_str: &str, qty_str: &str) -> Result<PositionAnalysis, String> {
    let price = price_str.parse::<f64>()
        .map_err(|_| format!("Неверная цена для {}", symbol))?;

    let quantity = qty_str.parse::<f64>()
        .map_err(|_| format!("Неверное количество для {}", symbol))?;

    let value = price * quantity;
    let weight = 0.0;  // Будет рассчитан позже

    Ok(PositionAnalysis {
        symbol: symbol.to_string(),
        price,
        quantity,
        value,
        weight,
    })
}
```

## Цепочка для риск-менеджмента

```rust
fn main() {
    let trade = TradeRequest {
        symbol: "BTC".to_string(),
        price: 42000.0,
        quantity: 0.5,
        side: Side::Buy,
    };

    let account = Account {
        balance: 25000.0,
        max_position_size: 20000.0,
        daily_loss_limit: 1000.0,
        current_daily_loss: 200.0,
    };

    let result = validate_trade(&trade, &account)
        .and_then(|t| check_position_size(t, &account))
        .and_then(|t| check_daily_loss_limit(t, &account))
        .and_then(|t| calculate_risk_metrics(t))
        .map(|metrics| format_risk_report(&trade, &metrics));

    match result {
        Ok(report) => println!("{}", report),
        Err(e) => println!("Торговля отклонена: {:?}", e),
    }
}

#[derive(Debug, Clone)]
struct TradeRequest {
    symbol: String,
    price: f64,
    quantity: f64,
    side: Side,
}

#[derive(Debug, Clone)]
enum Side {
    Buy,
    Sell,
}

struct Account {
    balance: f64,
    max_position_size: f64,
    daily_loss_limit: f64,
    current_daily_loss: f64,
}

#[derive(Debug)]
enum RiskError {
    InsufficientBalance,
    PositionTooLarge,
    DailyLossLimitReached,
    InvalidTrade(String),
}

struct RiskMetrics {
    position_value: f64,
    risk_percentage: f64,
    max_loss: f64,
}

fn validate_trade(trade: &TradeRequest, account: &Account) -> Result<TradeRequest, RiskError> {
    let required = trade.price * trade.quantity;
    if required > account.balance {
        Err(RiskError::InsufficientBalance)
    } else {
        Ok(trade.clone())
    }
}

fn check_position_size(trade: TradeRequest, account: &Account) -> Result<TradeRequest, RiskError> {
    let position_value = trade.price * trade.quantity;
    if position_value > account.max_position_size {
        Err(RiskError::PositionTooLarge)
    } else {
        Ok(trade)
    }
}

fn check_daily_loss_limit(trade: TradeRequest, account: &Account) -> Result<TradeRequest, RiskError> {
    let remaining = account.daily_loss_limit - account.current_daily_loss;
    let potential_loss = trade.price * trade.quantity * 0.02;  // Макс 2% потери

    if potential_loss > remaining {
        Err(RiskError::DailyLossLimitReached)
    } else {
        Ok(trade)
    }
}

fn calculate_risk_metrics(trade: TradeRequest) -> Result<RiskMetrics, RiskError> {
    let position_value = trade.price * trade.quantity;
    let risk_percentage = 2.0;  // Фиксированный риск
    let max_loss = position_value * (risk_percentage / 100.0);

    Ok(RiskMetrics {
        position_value,
        risk_percentage,
        max_loss,
    })
}

fn format_risk_report(trade: &TradeRequest, metrics: &RiskMetrics) -> String {
    format!(
        "╔══════════════════════════════════╗\n\
         ║        РИСК-ОТЧЁТ                ║\n\
         ╠══════════════════════════════════╣\n\
         ║ Символ:        {:>17} ║\n\
         ║ Размер позиции: ${:>14.2} ║\n\
         ║ Риск:           {:>14.1}% ║\n\
         ║ Макс. убыток:   ${:>14.2} ║\n\
         ║ Статус:         {:>17} ║\n\
         ╚══════════════════════════════════╝",
        trade.symbol,
        metrics.position_value,
        metrics.risk_percentage,
        metrics.max_loss,
        "ОДОБРЕНО"
    )
}
```

## Что мы узнали

| Метод | Описание | Когда использовать |
|-------|----------|-------------------|
| `and_then` | Цепочка Result-returning функций | Последовательные операции |
| `map` | Преобразование Ok значения | Простые трансформации |
| `map_err` | Преобразование Err значения | Конвертация типов ошибок |
| `or_else` | Альтернатива при ошибке | Fallback стратегии |
| `unwrap_or_else` | Значение по умолчанию | Ленивые значения по умолчанию |
| `?` | Ранний возврат ошибки | Чистые цепочки в функциях |

## Упражнения

1. **Валидатор ордера**: Создай цепочку для валидации биржевого ордера с проверками: парсинг → валидация цены → валидация количества → проверка баланса → проверка лимитов.

2. **Мульти-биржевой парсер**: Напиши функцию, которая пытается получить цену с нескольких бирж по очереди, используя `or_else`.

3. **Калькулятор позиции**: Создай цепочку для расчёта размера позиции: парсинг параметров → расчёт риска → определение размера → валидация результата.

4. **Обработчик портфеля**: Напиши функцию, которая обрабатывает список позиций, собирая успешные результаты и логируя ошибки.

## Домашнее задание

1. Реализуй полный пайплайн исполнения торговой стратегии с цепочками методов: получение данных → анализ → генерация сигнала → расчёт размера → исполнение → отчёт.

2. Создай систему резервных источников данных, которая последовательно пробует: primary API → secondary API → кеш → значение по умолчанию.

3. Напиши функцию `batch_process_orders`, которая обрабатывает вектор ордеров, разделяя результаты на успешные и неуспешные.

4. Реализуй цепочку проверок для системы риск-менеджмента, где каждая проверка может добавить предупреждение даже при успехе.

## Навигация

[← Предыдущий день](../097-result-advanced/ru.md) | [Следующий день →](../099-custom-error-types/ru.md)
