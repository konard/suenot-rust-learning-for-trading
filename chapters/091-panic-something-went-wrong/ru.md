# День 91: panic! — Что-то пошло очень не так

## Аналогия из трейдинга

Представь ситуацию: твоя торговая система обнаруживает, что баланс счёта стал **отрицательным** при работе без маржи. Это невозможно в нормальных условиях — значит, произошла критическая ошибка. Продолжать торговать нельзя — нужно **немедленно остановить всё** и разобраться.

Макрос `panic!` в Rust делает именно это: немедленно останавливает программу при обнаружении ситуации, которая не должна была произойти.

## Базовое использование panic!

```rust
fn main() {
    println!("Запуск торговой системы...");

    panic!("Критическая ошибка: потеряно соединение с биржей!");

    println!("Этот код никогда не выполнится");
}
```

Вывод:
```
Запуск торговой системы...
thread 'main' panicked at 'Критическая ошибка: потеряно соединение с биржей!'
```

## Panic с форматированием

```rust
fn main() {
    let account_balance = -1500.0;
    let symbol = "BTC/USDT";

    if account_balance < 0.0 {
        panic!(
            "КРИТИЧЕСКАЯ ОШИБКА: Отрицательный баланс {} для {}!",
            account_balance,
            symbol
        );
    }
}
```

## Когда использовать panic!

### 1. Нарушение инвариантов системы

```rust
fn main() {
    let mut portfolio = Portfolio::new(10000.0);

    // Симуляция критической ошибки
    portfolio.balance = -500.0;

    portfolio.validate();
}

struct Portfolio {
    balance: f64,
    positions: Vec<Position>,
}

struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
}

impl Portfolio {
    fn new(initial_balance: f64) -> Self {
        Portfolio {
            balance: initial_balance,
            positions: Vec::new(),
        }
    }

    fn validate(&self) {
        // Баланс никогда не должен быть отрицательным без маржи
        if self.balance < 0.0 {
            panic!(
                "ИНВАРИАНТ НАРУШЕН: Баланс портфеля отрицательный: {}",
                self.balance
            );
        }

        // Проверка каждой позиции
        for position in &self.positions {
            if position.quantity <= 0.0 {
                panic!(
                    "ИНВАРИАНТ НАРУШЕН: Нулевое или отрицательное количество для {}",
                    position.symbol
                );
            }
            if position.entry_price <= 0.0 {
                panic!(
                    "ИНВАРИАНТ НАРУШЕН: Некорректная цена входа для {}",
                    position.symbol
                );
            }
        }
    }
}
```

### 2. Невозможные состояния в торговой логике

```rust
fn main() {
    let order_type = "INVALID";
    process_order(order_type, 100.0, 42000.0);
}

fn process_order(order_type: &str, quantity: f64, price: f64) {
    match order_type {
        "BUY" => {
            println!("Покупка {} по цене {}", quantity, price);
        }
        "SELL" => {
            println!("Продажа {} по цене {}", quantity, price);
        }
        "LIMIT_BUY" | "LIMIT_SELL" => {
            println!("Лимитный ордер: {} по {}", quantity, price);
        }
        _ => {
            // Неизвестный тип ордера — это баг в коде
            panic!("Неизвестный тип ордера: '{}'. Это баг!", order_type);
        }
    }
}
```

### 3. Критические ошибки при инициализации

```rust
fn main() {
    let config = TradingConfig::load();
    println!("Конфигурация загружена: {:?}", config);
}

#[derive(Debug)]
struct TradingConfig {
    api_key: String,
    api_secret: String,
    max_position_size: f64,
    risk_per_trade: f64,
}

impl TradingConfig {
    fn load() -> Self {
        // Имитация загрузки конфига
        let api_key = ""; // Пустой ключ
        let api_secret = "secret123";
        let max_position_size = 10000.0;
        let risk_per_trade = 2.0;

        // Проверки критических параметров
        if api_key.is_empty() {
            panic!("КРИТИЧЕСКАЯ ОШИБКА: API ключ не задан! Торговля невозможна.");
        }

        if api_secret.is_empty() {
            panic!("КРИТИЧЕСКАЯ ОШИБКА: API секрет не задан!");
        }

        if max_position_size <= 0.0 {
            panic!(
                "КРИТИЧЕСКАЯ ОШИБКА: Некорректный max_position_size: {}",
                max_position_size
            );
        }

        if risk_per_trade <= 0.0 || risk_per_trade > 100.0 {
            panic!(
                "КРИТИЧЕСКАЯ ОШИБКА: Некорректный risk_per_trade: {}%",
                risk_per_trade
            );
        }

        TradingConfig {
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
            max_position_size,
            risk_per_trade,
        }
    }
}
```

## Panic в защитном программировании

### Проверка предусловий

```rust
fn main() {
    // Это вызовет панику
    let size = calculate_position_size(10000.0, 0.0, 42000.0, 41000.0);
    println!("Размер позиции: {}", size);
}

fn calculate_position_size(
    balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
) -> f64 {
    // Предусловия — если нарушены, это баг в вызывающем коде
    assert!(balance > 0.0, "Баланс должен быть положительным");
    assert!(
        risk_percent > 0.0 && risk_percent <= 100.0,
        "Риск должен быть от 0 до 100%"
    );
    assert!(entry_price > 0.0, "Цена входа должна быть положительной");
    assert!(stop_loss > 0.0, "Стоп-лосс должен быть положительным");
    assert!(
        entry_price != stop_loss,
        "Цена входа и стоп-лосс не могут быть равны"
    );

    let risk_amount = balance * (risk_percent / 100.0);
    let risk_per_unit = (entry_price - stop_loss).abs();

    risk_amount / risk_per_unit
}
```

### assert! vs panic!

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42050.0];

    // assert! — для проверки условий (выключается в релизе с debug_assertions)
    assert!(!prices.is_empty(), "Массив цен не может быть пустым");

    // debug_assert! — только в debug режиме
    debug_assert!(prices.len() >= 2, "Нужно минимум 2 цены для анализа");

    // panic! — всегда выполняется
    if prices.iter().any(|&p| p < 0.0) {
        panic!("Обнаружена отрицательная цена!");
    }

    let avg = calculate_average(&prices);
    println!("Средняя цена: {:.2}", avg);
}

fn calculate_average(prices: &[f64]) -> f64 {
    // assert_eq! — проверка равенства
    // assert_ne! — проверка неравенства
    assert_ne!(prices.len(), 0, "Деление на ноль!");

    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## unreachable!() — недостижимый код

```rust
fn main() {
    let signal = generate_signal(42000.0, 42500.0);
    execute_signal(&signal);
}

fn generate_signal(current_price: f64, target_price: f64) -> &'static str {
    if current_price < target_price {
        "BUY"
    } else if current_price > target_price {
        "SELL"
    } else {
        "HOLD"
    }
}

fn execute_signal(signal: &str) {
    match signal {
        "BUY" => println!("Выполняем покупку"),
        "SELL" => println!("Выполняем продажу"),
        "HOLD" => println!("Ждём"),
        _ => unreachable!("generate_signal возвращает только BUY, SELL или HOLD"),
    }
}
```

## todo!() и unimplemented!()

```rust
fn main() {
    let mut engine = TradingEngine::new();
    engine.start();
}

struct TradingEngine {
    is_running: bool,
}

impl TradingEngine {
    fn new() -> Self {
        TradingEngine { is_running: false }
    }

    fn start(&mut self) {
        self.is_running = true;
        println!("Движок запущен");

        // Это ещё не реализовано
        self.connect_to_exchange();
    }

    fn connect_to_exchange(&self) {
        // todo! — напоминание, что нужно реализовать
        todo!("Реализовать подключение к бирже");
    }

    #[allow(dead_code)]
    fn execute_trade(&self, _symbol: &str, _quantity: f64) {
        // unimplemented! — функционал намеренно не реализован
        unimplemented!("Торговля через API будет добавлена в следующей версии");
    }
}
```

## Практический пример: Валидатор ордеров

```rust
fn main() {
    let order1 = Order {
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        quantity: 0.5,
        price: Some(42000.0),
        order_type: OrderType::Limit,
    };

    let order2 = Order {
        symbol: "".to_string(), // Пустой символ — ошибка!
        side: OrderSide::Sell,
        quantity: 1.0,
        price: None,
        order_type: OrderType::Market,
    };

    validate_and_submit(&order1);
    validate_and_submit(&order2); // Это вызовет панику
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
enum OrderType {
    Market,
    Limit,
}

#[derive(Debug)]
struct Order {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: Option<f64>,
    order_type: OrderType,
}

fn validate_and_submit(order: &Order) {
    // Критические проверки — если не пройдены, это баг
    assert!(!order.symbol.is_empty(), "Символ ордера не может быть пустым");
    assert!(order.quantity > 0.0, "Количество должно быть положительным");

    // Проверка консистентности типа ордера и цены
    match order.order_type {
        OrderType::Limit => {
            if order.price.is_none() {
                panic!("Лимитный ордер требует указания цены!");
            }
            if order.price.unwrap() <= 0.0 {
                panic!("Цена лимитного ордера должна быть положительной!");
            }
        }
        OrderType::Market => {
            // Market ордер — цена не нужна, OK
        }
    }

    println!("Ордер валиден: {:?}", order);
    println!("Отправка ордера на биржу...");
}
```

## Panic vs Result: когда что использовать

```rust
fn main() {
    // panic! — для багов и невозможных ситуаций
    // Result — для ожидаемых ошибок

    // Пример с Result (правильно для пользовательского ввода)
    match parse_price("42000.50") {
        Ok(price) => println!("Цена: {}", price),
        Err(e) => println!("Ошибка парсинга: {}", e),
    }

    // Пример с panic! (для внутренних проверок)
    let prices = vec![42000.0, 42100.0];
    let avg = safe_average(&prices);
    println!("Средняя: {}", avg);

    // Это вызовет панику — внутренняя ошибка
    let empty: Vec<f64> = vec![];
    let _ = safe_average(&empty);
}

// Result — для ожидаемых ошибок (неверный ввод пользователя)
fn parse_price(input: &str) -> Result<f64, String> {
    input.parse::<f64>()
        .map_err(|_| format!("'{}' — некорректная цена", input))
}

// panic! — для багов (пустой массив — это баг вызывающего кода)
fn safe_average(prices: &[f64]) -> f64 {
    assert!(!prices.is_empty(), "BUG: массив цен не должен быть пустым");
    prices.iter().sum::<f64>() / prices.len() as f64
}
```

## Информативные сообщения о паниках

```rust
fn main() {
    let order_id = "ORD-12345";
    let expected_status = "FILLED";
    let actual_status = "REJECTED";

    check_order_status(order_id, expected_status, actual_status);
}

fn check_order_status(order_id: &str, expected: &str, actual: &str) {
    if expected != actual {
        panic!(
            "\n\
            ╔══════════════════════════════════════════╗\n\
            ║      КРИТИЧЕСКАЯ ОШИБКА ОРДЕРА           ║\n\
            ╠══════════════════════════════════════════╣\n\
            ║ Order ID: {:<30} ║\n\
            ║ Ожидался статус: {:<23} ║\n\
            ║ Получен статус:  {:<23} ║\n\
            ╚══════════════════════════════════════════╝",
            order_id, expected, actual
        );
    }
}
```

## Что мы узнали

| Макрос | Когда использовать |
|--------|-------------------|
| `panic!` | Критическая ошибка, программа не может продолжать |
| `assert!` | Проверка условия (отключается в release) |
| `assert_eq!` | Проверка равенства двух значений |
| `assert_ne!` | Проверка неравенства двух значений |
| `debug_assert!` | Проверка только в debug режиме |
| `unreachable!` | Код, который никогда не должен выполняться |
| `todo!` | Заглушка для нереализованного кода |
| `unimplemented!` | Функционал намеренно не реализован |

## Домашнее задание

1. Напиши структуру `RiskManager` с методом `validate_trade()`, который использует `panic!` при нарушении правил риск-менеджмента (превышение максимальной позиции, слишком большой риск на сделку)

2. Создай функцию `process_market_data()`, которая использует `assert!` для проверки входных данных (цены положительные, объёмы корректные, временные метки в правильном порядке)

3. Реализуй enum `TradingState` с состояниями (Idle, Trading, Paused, Error) и функцию `transition()`, которая использует `unreachable!` для невозможных переходов состояний

4. Напиши модуль с заглушками `todo!()` для будущей реализации: подключение к WebSocket, обработка стакана заявок, исполнение ордеров

## Навигация

[← Предыдущий день](../090-project-order-book/ru.md) | [Следующий день →](../092-when-to-panic/ru.md)
