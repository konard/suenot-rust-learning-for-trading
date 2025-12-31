# День 73: Методы Option — unwrap, expect, unwrap_or

## Аналогия из трейдинга

Представь, что ты запрашиваешь цену актива у биржи. Иногда цена есть, а иногда — нет (торги приостановлены, актив делистингнут, сервер не отвечает). Тип `Option` в Rust представляет именно такую ситуацию: значение либо есть (`Some`), либо его нет (`None`).

Но что делать, когда нам нужно **извлечь** значение из `Option`? Для этого есть несколько методов:

- **unwrap** — "Я уверен, что цена есть. Если нет — пусть программа упадёт!"
- **expect** — "Я уверен, что цена есть. Если нет — упади с понятным сообщением!"
- **unwrap_or** — "Дай цену, а если её нет — возьми значение по умолчанию"

## Метод unwrap — опасный, но быстрый

`unwrap()` извлекает значение из `Some` или **паникует** при `None`:

```rust
fn main() {
    let price: Option<f64> = Some(42000.0);
    let value = price.unwrap();  // 42000.0
    println!("BTC price: ${}", value);

    let missing_price: Option<f64> = None;
    // let crash = missing_price.unwrap();  // ПАНИКА! thread 'main' panicked
}
```

### Когда использовать unwrap

```rust
fn main() {
    // 1. В тестах — падение теста = провал
    let test_price: Option<f64> = get_test_price();
    assert_eq!(test_price.unwrap(), 42000.0);

    // 2. Когда логически None невозможен
    let prices = vec![42000.0, 42100.0, 42050.0];
    let first = prices.first().unwrap();  // Вектор не пустой — 100% есть первый элемент
    println!("First price: ${}", first);

    // 3. В прототипах/быстрых скриптах
    let quick_calc = Some(100.0 * 1.05).unwrap();
    println!("Result: {}", quick_calc);
}

fn get_test_price() -> Option<f64> {
    Some(42000.0)
}
```

### Опасности unwrap в трейдинге

```rust
fn main() {
    let portfolio = vec!["BTC", "ETH"];

    // ОПАСНО: что если индекс выходит за границы?
    // let asset = portfolio.get(5).unwrap();  // ПАНИКА!

    // ОПАСНО в продакшене: API может вернуть None
    // let price = fetch_price("UNKNOWN_COIN").unwrap();  // ПАНИКА!
}
```

## Метод expect — unwrap с сообщением

`expect(msg)` работает как `unwrap`, но при панике выводит ваше сообщение:

```rust
fn main() {
    let btc_price: Option<f64> = Some(42000.0);
    let price = btc_price.expect("BTC price should exist in portfolio");
    println!("BTC: ${}", price);

    // Полезно для отладки
    let eth_price: Option<f64> = None;
    // let price = eth_price.expect("ETH price missing - check API connection");
    // thread 'main' panicked at 'ETH price missing - check API connection'
}
```

### Пример: загрузка конфигурации

```rust
fn main() {
    let config = load_trading_config();

    let api_key = config.api_key
        .expect("API key must be set in config.toml");

    let max_position = config.max_position_size
        .expect("Max position size required for risk management");

    println!("Config loaded: API key exists, max position: {}", max_position);
}

struct TradingConfig {
    api_key: Option<String>,
    max_position_size: Option<f64>,
}

fn load_trading_config() -> TradingConfig {
    TradingConfig {
        api_key: Some(String::from("secret_key_123")),
        max_position_size: Some(10000.0),
    }
}
```

## Метод unwrap_or — значение по умолчанию

`unwrap_or(default)` возвращает значение из `Some` или указанное значение по умолчанию:

```rust
fn main() {
    // Цена есть
    let btc_price: Option<f64> = Some(42000.0);
    let price = btc_price.unwrap_or(0.0);
    println!("BTC: ${}", price);  // 42000.0

    // Цены нет — используем дефолт
    let unknown_price: Option<f64> = None;
    let price = unknown_price.unwrap_or(0.0);
    println!("Unknown: ${}", price);  // 0.0
}
```

### Практическое применение

```rust
fn main() {
    // Комиссия биржи: если не указана, берём стандартную 0.1%
    let custom_fee: Option<f64> = None;
    let fee_percent = custom_fee.unwrap_or(0.1);
    println!("Fee: {}%", fee_percent);

    // Стоп-лосс: если не задан, используем 2% от цены входа
    let entry_price = 42000.0;
    let stop_loss: Option<f64> = None;
    let actual_stop = stop_loss.unwrap_or(entry_price * 0.98);
    println!("Stop-loss: ${}", actual_stop);  // 41160.0

    // Количество: минимум 1, если не указано
    let quantity: Option<u32> = None;
    let qty = quantity.unwrap_or(1);
    println!("Quantity: {}", qty);
}
```

## Метод unwrap_or_else — ленивое вычисление

`unwrap_or_else(|| ...)` вычисляет значение по умолчанию только если `None`:

```rust
fn main() {
    let cached_price: Option<f64> = None;

    // Функция fetch_live_price вызовется ТОЛЬКО если cached_price == None
    let price = cached_price.unwrap_or_else(|| fetch_live_price("BTC"));
    println!("Price: ${}", price);

    // С unwrap_or функция вызывается ВСЕГДА (даже если есть Some)
    let cached = Some(42000.0);
    let price = cached.unwrap_or(fetch_live_price("BTC"));  // fetch вызовется!
    let price = cached.unwrap_or_else(|| fetch_live_price("BTC"));  // fetch НЕ вызовется
}

fn fetch_live_price(symbol: &str) -> f64 {
    println!("Fetching price for {}...", symbol);
    42500.0  // Имитация запроса к API
}
```

### Когда использовать unwrap_or_else

```rust
fn main() {
    // 1. Дорогие вычисления
    let cached_sma: Option<f64> = Some(42100.0);
    let sma = cached_sma.unwrap_or_else(|| calculate_sma(&get_prices(), 20));

    // 2. Побочные эффекты (логирование)
    let price: Option<f64> = None;
    let value = price.unwrap_or_else(|| {
        log_missing_price();
        0.0
    });

    // 3. Зависимость от других данных
    let balance = 10000.0;
    let position_size: Option<f64> = None;
    let size = position_size.unwrap_or_else(|| balance * 0.02);  // 2% от баланса
}

fn calculate_sma(prices: &[f64], period: usize) -> f64 {
    if prices.len() < period {
        return 0.0;
    }
    prices.iter().rev().take(period).sum::<f64>() / period as f64
}

fn get_prices() -> Vec<f64> {
    vec![42000.0, 42100.0, 42050.0, 42200.0, 42150.0]
}

fn log_missing_price() {
    println!("[WARN] Price is missing, using default");
}
```

## Метод unwrap_or_default — Default trait

`unwrap_or_default()` использует значение по умолчанию для типа:

```rust
fn main() {
    // Для чисел default = 0
    let volume: Option<f64> = None;
    let vol = volume.unwrap_or_default();  // 0.0
    println!("Volume: {}", vol);

    // Для строк default = ""
    let symbol: Option<String> = None;
    let sym = symbol.unwrap_or_default();  // ""
    println!("Symbol: '{}'", sym);

    // Для векторов default = []
    let trades: Option<Vec<f64>> = None;
    let t = trades.unwrap_or_default();  // []
    println!("Trades: {:?}", t);

    // Для bool default = false
    let is_active: Option<bool> = None;
    let active = is_active.unwrap_or_default();  // false
    println!("Active: {}", active);
}
```

## Сравнение методов

```rust
fn main() {
    let price: Option<f64> = None;

    // unwrap — паника при None
    // let v = price.unwrap();  // ПАНИКА!

    // expect — паника с сообщением
    // let v = price.expect("Price required");  // ПАНИКА с сообщением

    // unwrap_or — конкретное значение
    let v = price.unwrap_or(0.0);  // 0.0

    // unwrap_or_else — ленивое вычисление
    let v = price.unwrap_or_else(|| calculate_default());  // вызов функции

    // unwrap_or_default — Default trait
    let v = price.unwrap_or_default();  // 0.0 (default для f64)
}

fn calculate_default() -> f64 {
    println!("Calculating default...");
    42000.0
}
```

## Практический пример: Портфельный калькулятор

```rust
fn main() {
    let portfolio = Portfolio {
        btc: Some(0.5),
        eth: Some(10.0),
        sol: None,
        dot: Some(100.0),
    };

    let prices = Prices {
        btc: Some(42000.0),
        eth: Some(2200.0),
        sol: None,  // Нет данных о цене
        dot: Some(7.5),
    };

    let total = calculate_portfolio_value(&portfolio, &prices);
    println!("Total portfolio value: ${:.2}", total);
}

struct Portfolio {
    btc: Option<f64>,
    eth: Option<f64>,
    sol: Option<f64>,
    dot: Option<f64>,
}

struct Prices {
    btc: Option<f64>,
    eth: Option<f64>,
    sol: Option<f64>,
    dot: Option<f64>,
}

fn calculate_portfolio_value(portfolio: &Portfolio, prices: &Prices) -> f64 {
    let btc_value = portfolio.btc.unwrap_or(0.0) * prices.btc.unwrap_or(0.0);
    let eth_value = portfolio.eth.unwrap_or(0.0) * prices.eth.unwrap_or(0.0);
    let sol_value = portfolio.sol.unwrap_or(0.0) * prices.sol.unwrap_or(0.0);
    let dot_value = portfolio.dot.unwrap_or(0.0) * prices.dot.unwrap_or(0.0);

    btc_value + eth_value + sol_value + dot_value
}
```

## Практический пример: Система ордеров

```rust
fn main() {
    let order1 = Order {
        symbol: String::from("BTC"),
        price: Some(42000.0),
        quantity: 0.5,
        stop_loss: Some(41000.0),
        take_profit: None,
    };

    let order2 = Order {
        symbol: String::from("ETH"),
        price: None,  // Рыночный ордер
        quantity: 10.0,
        stop_loss: None,
        take_profit: Some(2500.0),
    };

    process_order(&order1);
    process_order(&order2);
}

struct Order {
    symbol: String,
    price: Option<f64>,      // None = рыночный ордер
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

fn process_order(order: &Order) {
    println!("\n=== Processing Order: {} ===", order.symbol);

    // Цена: используем рыночную если не указана
    let execution_price = order.price.unwrap_or_else(|| {
        println!("No limit price, fetching market price...");
        get_market_price(&order.symbol)
    });
    println!("Execution price: ${:.2}", execution_price);

    // Стоп-лосс: 5% ниже цены если не указан
    let stop = order.stop_loss.unwrap_or(execution_price * 0.95);
    println!("Stop-loss: ${:.2}", stop);

    // Тейк-профит: 10% выше цены если не указан
    let tp = order.take_profit.unwrap_or(execution_price * 1.10);
    println!("Take-profit: ${:.2}", tp);

    // Риск на сделку
    let risk = (execution_price - stop) * order.quantity;
    println!("Risk: ${:.2}", risk);
}

fn get_market_price(symbol: &str) -> f64 {
    match symbol {
        "BTC" => 42000.0,
        "ETH" => 2200.0,
        _ => 0.0,
    }
}
```

## Что мы узнали

| Метод | Поведение при None | Когда использовать |
|-------|-------------------|-------------------|
| `unwrap()` | Паника | Тесты, прототипы, 100% уверенность |
| `expect(msg)` | Паника с сообщением | Обязательные значения, отладка |
| `unwrap_or(val)` | Возвращает `val` | Простые дефолты |
| `unwrap_or_else(f)` | Вызывает `f()` | Дорогие вычисления, побочные эффекты |
| `unwrap_or_default()` | `Default::default()` | Стандартные нулевые значения |

## Упражнения

### Упражнение 1: Калькулятор комиссий
Напиши функцию, которая рассчитывает комиссию за сделку. Если комиссия не указана в ордере, используй стандартную 0.1%:

```rust
fn calculate_fee(trade_value: f64, custom_fee: Option<f64>) -> f64 {
    // Твой код здесь
}
```

### Упражнение 2: Безопасное получение цены
Создай функцию, которая возвращает цену актива из кэша или запрашивает с API:

```rust
fn get_price(symbol: &str, cache: &HashMap<String, f64>) -> f64 {
    // Используй unwrap_or_else для ленивой загрузки
}
```

### Упражнение 3: Валидация ордера
Напиши функцию проверки ордера, которая использует expect для обязательных полей:

```rust
fn validate_order(order: &Order) -> bool {
    // Используй expect для критически важных полей
}
```

### Упражнение 4: Статистика портфеля
Создай функцию расчёта статистики портфеля с дефолтными значениями:

```rust
fn portfolio_stats(assets: &[Option<f64>]) -> (f64, f64, f64) {
    // Верни (total, average, count), игнорируя None
}
```

## Домашнее задание

1. Реализуй систему кэширования цен с `unwrap_or_else` для ленивой загрузки

2. Создай конфигуратор торгового бота с `expect` для обязательных параметров и `unwrap_or` для опциональных

3. Напиши функцию расчёта PnL портфеля, где цены и количества могут отсутствовать

4. Реализуй парсер торгового лога, где некоторые поля могут быть пустыми

## Навигация

[← Предыдущий день: Option — цена может отсутствовать](../072-option-price-might-be-missing/ru.md) | [Следующий день: Option с map и and_then →](../074-option-map-and-then/ru.md)
