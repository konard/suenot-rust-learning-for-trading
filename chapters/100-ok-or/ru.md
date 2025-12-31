# День 100: ok_or — превращаем Option в Result

## Аналогия из трейдинга

Представь, что ты запрашиваешь текущую цену актива из кэша. Кэш может вернуть **Option** — цена либо есть (`Some(price)`), либо её нет (`None`). Но твоя торговая система требует **Result** — чтобы в случае отсутствия данных была конкретная ошибка, которую можно залогировать и обработать.

Метод `ok_or` — это как конвертер между двумя форматами: "возможно есть" → "либо есть, либо вот такая ошибка".

## Базовый синтаксис ok_or

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);
    let result: Result<f64, &str> = price.ok_or("Price not found");

    println!("{:?}", result); // Ok(42000.0)

    let no_price: Option<f64> = None;
    let result: Result<f64, &str> = no_price.ok_or("Price not found");

    println!("{:?}", result); // Err("Price not found")
}
```

**Важно:** `ok_or` преобразует:
- `Some(value)` → `Ok(value)`
- `None` → `Err(error)`

## Зачем нужно преобразование Option → Result?

```rust
fn main() {
    // Option говорит: "значения может не быть"
    // Result говорит: "либо значение, либо КОНКРЕТНАЯ ошибка"

    let cached_price: Option<f64> = None;

    // Без контекста — непонятно почему нет цены
    match cached_price {
        Some(p) => println!("Price: {}", p),
        None => println!("No price"), // Почему нет? Таймаут? Ошибка API?
    }

    // С ok_or — есть конкретная ошибка
    let result = cached_price.ok_or("Cache miss: BTC price expired");
    match result {
        Ok(p) => println!("Price: {}", p),
        Err(e) => println!("Error: {}", e), // Ясно что произошло
    }
}
```

## Практические примеры из трейдинга

### Получение цены актива из кэша

```rust
use std::collections::HashMap;

fn main() {
    let mut price_cache: HashMap<&str, f64> = HashMap::new();
    price_cache.insert("BTC", 42000.0);
    price_cache.insert("ETH", 2200.0);

    // Используем ok_or для преобразования Option в Result
    let btc_result = get_price(&price_cache, "BTC");
    let doge_result = get_price(&price_cache, "DOGE");

    println!("BTC: {:?}", btc_result);   // Ok(42000.0)
    println!("DOGE: {:?}", doge_result); // Err("Asset DOGE not found in cache")
}

fn get_price(cache: &HashMap<&str, f64>, symbol: &str) -> Result<f64, String> {
    cache.get(symbol)
        .copied()
        .ok_or(format!("Asset {} not found in cache", symbol))
}
```

### Проверка баланса перед ордером

```rust
use std::collections::HashMap;

fn main() {
    let mut balances: HashMap<&str, f64> = HashMap::new();
    balances.insert("USDT", 10000.0);
    balances.insert("BTC", 0.5);

    match check_balance(&balances, "USDT", 5000.0) {
        Ok(available) => println!("Can trade, available: ${:.2}", available),
        Err(e) => println!("Cannot trade: {}", e),
    }

    match check_balance(&balances, "ETH", 1.0) {
        Ok(available) => println!("Can trade, available: {}", available),
        Err(e) => println!("Cannot trade: {}", e),
    }
}

fn check_balance(
    balances: &HashMap<&str, f64>,
    asset: &str,
    required: f64
) -> Result<f64, String> {
    let balance = balances.get(asset)
        .copied()
        .ok_or(format!("No {} balance found", asset))?;

    if balance < required {
        return Err(format!(
            "Insufficient {}: have {:.4}, need {:.4}",
            asset, balance, required
        ));
    }

    Ok(balance)
}
```

### Поиск ордера в книге заявок

```rust
struct Order {
    id: u64,
    symbol: String,
    price: f64,
    quantity: f64,
}

fn main() {
    let orders = vec![
        Order { id: 1, symbol: "BTC".into(), price: 42000.0, quantity: 0.5 },
        Order { id: 2, symbol: "ETH".into(), price: 2200.0, quantity: 10.0 },
    ];

    match find_order(&orders, 1) {
        Ok(order) => println!("Found: {} @ {}", order.symbol, order.price),
        Err(e) => println!("Error: {}", e),
    }

    match find_order(&orders, 999) {
        Ok(order) => println!("Found: {} @ {}", order.symbol, order.price),
        Err(e) => println!("Error: {}", e),
    }
}

fn find_order(orders: &[Order], id: u64) -> Result<&Order, String> {
    orders.iter()
        .find(|o| o.id == id)
        .ok_or(format!("Order #{} not found", id))
}
```

## ok_or vs ok_or_else

```rust
fn main() {
    let value: Option<f64> = None;

    // ok_or — ошибка вычисляется ВСЕГДА (даже если Some)
    let _result = value.ok_or(expensive_error());

    // ok_or_else — ошибка вычисляется ТОЛЬКО если None (ленивое вычисление)
    let _result = value.ok_or_else(|| expensive_error());
}

fn expensive_error() -> String {
    println!("Computing error..."); // Это выполнится
    String::from("Expensive error message")
}
```

**Правило:** Используй `ok_or_else` когда создание ошибки "дорогое" (аллокации, вычисления, форматирование).

### Пример с форматированием

```rust
fn main() {
    let prices: Vec<f64> = vec![42000.0, 42100.0, 41900.0];

    // ok_or — строка создаётся всегда
    let _first = prices.first().ok_or(String::from("Empty prices"));

    // ok_or_else — строка создаётся только если None
    let _first = prices.first().ok_or_else(|| {
        format!("No prices available at {}", chrono_placeholder())
    });
}

fn chrono_placeholder() -> &'static str {
    "2024-01-15 10:30:00" // В реальности тут был бы вызов chrono
}
```

## Цепочки с оператором ?

```rust
use std::collections::HashMap;

fn main() {
    let result = execute_trade_flow();
    match result {
        Ok(msg) => println!("Success: {}", msg),
        Err(e) => println!("Failed: {}", e),
    }
}

fn execute_trade_flow() -> Result<String, String> {
    let mut balances: HashMap<&str, f64> = HashMap::new();
    balances.insert("USDT", 10000.0);

    let mut prices: HashMap<&str, f64> = HashMap::new();
    prices.insert("BTC", 42000.0);

    // Цепочка с ? — каждый ok_or может прервать выполнение
    let balance = balances.get("USDT")
        .copied()
        .ok_or("No USDT balance")?;

    let price = prices.get("BTC")
        .copied()
        .ok_or("No BTC price available")?;

    let quantity = balance / price;

    Ok(format!("Can buy {:.6} BTC at ${:.2}", quantity, price))
}
```

## Комбинирование с другими методами

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42050.0, 42200.0];

    // Расчёт SMA с валидацией
    match calculate_sma(&prices, 3) {
        Ok(sma) => println!("SMA-3: {:.2}", sma),
        Err(e) => println!("Error: {}", e),
    }

    match calculate_sma(&[], 3) {
        Ok(sma) => println!("SMA-3: {:.2}", sma),
        Err(e) => println!("Error: {}", e),
    }
}

fn calculate_sma(prices: &[f64], period: usize) -> Result<f64, String> {
    if prices.len() < period {
        return Err(format!(
            "Need {} prices, got {}",
            period, prices.len()
        ));
    }

    let slice = &prices[prices.len() - period..];
    let sum: f64 = slice.iter().sum();

    // Защита от деления на ноль (хотя period > 0 гарантировано выше)
    Some(sum / period as f64)
        .filter(|v| v.is_finite())
        .ok_or("SMA calculation resulted in invalid number".to_string())
}
```

## Типы ошибок

### Со строками

```rust
fn get_price_string(symbol: &str) -> Result<f64, String> {
    let prices = [("BTC", 42000.0), ("ETH", 2200.0)];

    prices.iter()
        .find(|(s, _)| *s == symbol)
        .map(|(_, p)| *p)
        .ok_or(format!("Price not found for {}", symbol))
}
```

### С собственным типом ошибки

```rust
#[derive(Debug)]
enum TradingError {
    PriceNotFound(String),
    InsufficientBalance { asset: String, have: f64, need: f64 },
    OrderNotFound(u64),
}

fn get_price_typed(symbol: &str) -> Result<f64, TradingError> {
    let prices = [("BTC", 42000.0), ("ETH", 2200.0)];

    prices.iter()
        .find(|(s, _)| *s == symbol)
        .map(|(_, p)| *p)
        .ok_or(TradingError::PriceNotFound(symbol.to_string()))
}

fn main() {
    match get_price_typed("DOGE") {
        Ok(price) => println!("Price: {}", price),
        Err(TradingError::PriceNotFound(s)) => {
            println!("Symbol {} not supported", s)
        }
        Err(_) => println!("Other error"),
    }
}
```

## Практические упражнения

### Упражнение 1: Валидация торгового сигнала

```rust
struct TradingSignal {
    symbol: String,
    action: String, // "BUY" or "SELL"
    price: Option<f64>,
    quantity: Option<f64>,
}

// TODO: Реализуй функцию validate_signal
// Должна возвращать Result с ценой и количеством, или ошибку если чего-то нет
fn validate_signal(signal: &TradingSignal) -> Result<(f64, f64), String> {
    // Твой код здесь
    todo!()
}

fn main() {
    let valid_signal = TradingSignal {
        symbol: "BTC".to_string(),
        action: "BUY".to_string(),
        price: Some(42000.0),
        quantity: Some(0.5),
    };

    let invalid_signal = TradingSignal {
        symbol: "ETH".to_string(),
        action: "SELL".to_string(),
        price: Some(2200.0),
        quantity: None,
    };

    println!("{:?}", validate_signal(&valid_signal));
    println!("{:?}", validate_signal(&invalid_signal));
}
```

### Упражнение 2: Получение данных портфеля

```rust
use std::collections::HashMap;

struct Portfolio {
    balances: HashMap<String, f64>,
    prices: HashMap<String, f64>,
}

// TODO: Реализуй функцию get_position_value
// Должна вернуть стоимость позиции (balance * price) или ошибку
fn get_position_value(portfolio: &Portfolio, asset: &str) -> Result<f64, String> {
    // Твой код здесь
    todo!()
}

fn main() {
    let mut portfolio = Portfolio {
        balances: HashMap::new(),
        prices: HashMap::new(),
    };
    portfolio.balances.insert("BTC".to_string(), 0.5);
    portfolio.prices.insert("BTC".to_string(), 42000.0);

    println!("{:?}", get_position_value(&portfolio, "BTC"));
    println!("{:?}", get_position_value(&portfolio, "ETH"));
}
```

### Упражнение 3: Парсинг конфигурации стратегии

```rust
use std::collections::HashMap;

// TODO: Реализуй функцию parse_strategy_config
// Должна извлечь все необходимые параметры или вернуть ошибку
fn parse_strategy_config(
    config: &HashMap<String, String>
) -> Result<(f64, f64, u32), String> {
    // Нужно извлечь: stop_loss, take_profit, max_positions
    // Твой код здесь
    todo!()
}

fn main() {
    let mut config = HashMap::new();
    config.insert("stop_loss".to_string(), "2.5".to_string());
    config.insert("take_profit".to_string(), "5.0".to_string());
    config.insert("max_positions".to_string(), "3".to_string());

    println!("{:?}", parse_strategy_config(&config));
}
```

### Упражнение 4: Расчёт риска позиции

```rust
struct Position {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    stop_loss: Option<f64>,
}

// TODO: Реализуй функцию calculate_position_risk
// Должна вернуть размер риска в долларах или ошибку если stop_loss не установлен
fn calculate_position_risk(position: &Position) -> Result<f64, String> {
    // Твой код здесь
    todo!()
}

fn main() {
    let position_with_sl = Position {
        symbol: "BTC".to_string(),
        entry_price: 42000.0,
        quantity: 0.5,
        stop_loss: Some(40000.0),
    };

    let position_without_sl = Position {
        symbol: "ETH".to_string(),
        entry_price: 2200.0,
        quantity: 10.0,
        stop_loss: None,
    };

    println!("{:?}", calculate_position_risk(&position_with_sl));
    println!("{:?}", calculate_position_risk(&position_without_sl));
}
```

## Что мы узнали

| Метод | Описание | Когда использовать |
|-------|----------|-------------------|
| `ok_or(err)` | Option → Result | Ошибка простая (литерал, число) |
| `ok_or_else(\|\| err)` | Option → Result (ленивый) | Ошибка требует вычислений |
| `.ok_or()?` | С оператором ? | В цепочке Result-функций |

## Домашнее задание

1. **Order Book Lookup**: Напиши функцию поиска лучшей цены bid/ask в книге заявок, возвращающую `Result<f64, OrderBookError>`

2. **Portfolio Calculator**: Создай калькулятор общей стоимости портфеля, который собирает все ошибки отсутствующих цен

3. **Trading Signal Validator**: Реализуй полный валидатор торгового сигнала с проверкой всех полей и цепочкой `ok_or`

4. **Config Parser**: Напиши парсер конфигурации торговой стратегии, преобразующий `HashMap<String, String>` в типизированную структуру с использованием `ok_or_else`

## Навигация

[← Предыдущий день](../099-map-err/ru.md) | [Следующий день →](../101-multiple-error-types/ru.md)
