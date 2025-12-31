# День 92: Когда использовать panic! — невосстановимые ошибки

## Аналогия из трейдинга

Представь торгового бота, который внезапно обнаруживает, что его конфигурация содержит отрицательный размер позиции или что ключ API отсутствует при запуске. Это не ситуация, которую можно "обработать" — это **критическая ошибка**, при которой продолжение работы опасно.

В реальном трейдинге: если система риск-менеджмента обнаруживает нарушение критического инварианта (например, позиция превышает 100% депозита), она **немедленно останавливает торговлю**. Это и есть `panic!` в Rust — аварийная остановка программы при невосстановимой ошибке.

## Что такое panic!

`panic!` — это макрос, который немедленно завершает программу (или текущий поток) при обнаружении невосстановимой ошибки.

```rust
fn main() {
    panic!("Критическая ошибка: невозможно продолжить!");
}
```

При вызове `panic!`:
1. Выводится сообщение об ошибке
2. Раскручивается стек (unwinding) или программа немедленно прерывается (abort)
3. Программа завершается с ненулевым кодом выхода

## Когда использовать panic!

### 1. Нарушение инвариантов программы

```rust
fn main() {
    let balance = -1000.0;
    validate_balance(balance);
}

fn validate_balance(balance: f64) {
    if balance < 0.0 {
        panic!(
            "КРИТИЧЕСКАЯ ОШИБКА: Отрицательный баланс {} невозможен! \
             Возможна ошибка в расчётах.",
            balance
        );
    }
    println!("Баланс валиден: ${:.2}", balance);
}
```

### 2. Ошибки конфигурации при запуске

```rust
fn main() {
    let config = TradingConfig::load();
    println!("Конфигурация загружена: {:?}", config);
}

#[derive(Debug)]
struct TradingConfig {
    api_key: String,
    max_position_size: f64,
    risk_per_trade: f64,
}

impl TradingConfig {
    fn load() -> Self {
        // Имитация загрузки конфигурации
        let api_key = std::env::var("API_KEY").unwrap_or_default();
        let max_position = 10000.0;
        let risk = 2.0;

        // Критические проверки при запуске
        if api_key.is_empty() {
            panic!("API_KEY не установлен! Торговля невозможна.");
        }

        if max_position <= 0.0 {
            panic!(
                "Некорректный max_position_size: {}. Должен быть положительным.",
                max_position
            );
        }

        if risk <= 0.0 || risk > 100.0 {
            panic!(
                "Некорректный risk_per_trade: {}%. Допустимо: 0-100%.",
                risk
            );
        }

        TradingConfig {
            api_key,
            max_position_size: max_position,
            risk_per_trade: risk,
        }
    }
}
```

### 3. Недостижимый код (unreachable)

```rust
fn main() {
    let side = "BUY";
    let result = calculate_pnl(side, 100.0, 110.0, 1.0);
    println!("PnL: ${:.2}", result);
}

fn calculate_pnl(side: &str, entry: f64, exit: f64, qty: f64) -> f64 {
    match side {
        "BUY" => (exit - entry) * qty,
        "SELL" => (entry - exit) * qty,
        _ => panic!("Недопустимая сторона сделки: '{}'. Ожидается BUY или SELL.", side),
    }
}
```

### 4. Использование unreachable!()

```rust
fn main() {
    let order_type = OrderType::Market;
    process_order(order_type, 42000.0);
}

#[derive(Debug)]
enum OrderType {
    Market,
    Limit,
    StopLoss,
}

fn process_order(order_type: OrderType, price: f64) {
    match order_type {
        OrderType::Market => {
            println!("Исполнение рыночного ордера по текущей цене");
        }
        OrderType::Limit => {
            println!("Размещение лимитного ордера по цене ${:.2}", price);
        }
        OrderType::StopLoss => {
            println!("Установка стоп-лосса на ${:.2}", price);
        }
        // Если добавим новый вариант и забудем обработать:
        #[allow(unreachable_patterns)]
        _ => unreachable!("Все типы ордеров должны быть обработаны!"),
    }
}
```

## panic! vs Result: когда что использовать

```rust
use std::collections::HashMap;

fn main() {
    // Пример 1: Result для восстановимых ошибок
    let mut portfolio: HashMap<String, f64> = HashMap::new();
    portfolio.insert("BTC".to_string(), 1.5);
    portfolio.insert("ETH".to_string(), 10.0);

    match get_position(&portfolio, "BTC") {
        Ok(qty) => println!("Позиция BTC: {} единиц", qty),
        Err(e) => println!("Ошибка: {}", e),
    }

    match get_position(&portfolio, "DOGE") {
        Ok(qty) => println!("Позиция DOGE: {} единиц", qty),
        Err(e) => println!("Предупреждение: {}", e),
    }

    // Пример 2: panic! для невосстановимых ошибок
    let critical_config = CriticalConfig {
        max_drawdown_percent: 25.0,
        emergency_stop: true,
    };
    validate_critical_config(&critical_config);
}

// Восстановимая ошибка - используем Result
fn get_position(portfolio: &HashMap<String, f64>, ticker: &str) -> Result<f64, String> {
    portfolio
        .get(ticker)
        .copied()
        .ok_or_else(|| format!("Тикер '{}' не найден в портфеле", ticker))
}

struct CriticalConfig {
    max_drawdown_percent: f64,
    emergency_stop: bool,
}

// Невосстановимая ошибка - используем panic!
fn validate_critical_config(config: &CriticalConfig) {
    if config.max_drawdown_percent <= 0.0 || config.max_drawdown_percent > 100.0 {
        panic!(
            "КРИТИЧЕСКАЯ ОШИБКА КОНФИГУРАЦИИ: \
             max_drawdown_percent = {}% недопустим!",
            config.max_drawdown_percent
        );
    }

    if !config.emergency_stop {
        panic!(
            "КРИТИЧЕСКАЯ ОШИБКА БЕЗОПАСНОСТИ: \
             emergency_stop должен быть включён!"
        );
    }

    println!("Критическая конфигурация валидна.");
}
```

## Методы, вызывающие panic!

```rust
fn main() {
    // unwrap() - паникует при None или Err
    let prices = vec![42000.0, 42500.0, 41800.0];

    // Опасно! Паникует если индекс вне диапазона
    // let price = prices[10]; // panic: index out of bounds

    // Безопасная альтернатива
    match prices.get(10) {
        Some(price) => println!("Цена: {}", price),
        None => println!("Индекс вне диапазона"),
    }

    // unwrap() - используй только когда уверен
    let valid_price: Option<f64> = Some(42000.0);
    let price = valid_price.unwrap(); // OK, мы знаем что Some
    println!("Цена: {}", price);

    // expect() - panic с пользовательским сообщением
    let api_response: Option<f64> = Some(42500.0);
    let current_price = api_response
        .expect("API должен вернуть текущую цену");
    println!("Текущая цена: ${:.2}", current_price);
}
```

## unwrap_or и альтернативы panic

```rust
fn main() {
    let maybe_price: Option<f64> = None;

    // Вместо panic - значение по умолчанию
    let price1 = maybe_price.unwrap_or(0.0);
    println!("Цена (или 0): {}", price1);

    // Ленивое вычисление значения по умолчанию
    let price2 = maybe_price.unwrap_or_else(|| {
        println!("Вычисляем значение по умолчанию...");
        fetch_default_price()
    });
    println!("Цена (или дефолт): {}", price2);

    // unwrap_or_default для типов с Default
    let maybe_qty: Option<f64> = None;
    let qty = maybe_qty.unwrap_or_default(); // 0.0 для f64
    println!("Количество: {}", qty);
}

fn fetch_default_price() -> f64 {
    42000.0 // Имитация получения цены
}
```

## Практический пример: валидация торговой системы

```rust
fn main() {
    // Создаём торговую систему с валидацией
    let system = TradingSystem::new(
        10000.0,  // начальный баланс
        2.0,      // риск на сделку %
        10.0,     // макс. просадка %
    );

    println!("{:?}", system);

    // Попытка создать систему с невалидными параметрами
    // Раскомментируй для теста panic:
    // let invalid_system = TradingSystem::new(-1000.0, 2.0, 10.0);
}

#[derive(Debug)]
struct TradingSystem {
    balance: f64,
    risk_per_trade: f64,
    max_drawdown: f64,
    is_active: bool,
}

impl TradingSystem {
    fn new(balance: f64, risk_per_trade: f64, max_drawdown: f64) -> Self {
        // Критические проверки - нарушение = panic
        if balance <= 0.0 {
            panic!(
                "КРИТИЧЕСКАЯ ОШИБКА: Начальный баланс должен быть положительным. \
                 Получено: ${:.2}",
                balance
            );
        }

        if risk_per_trade <= 0.0 {
            panic!(
                "КРИТИЧЕСКАЯ ОШИБКА: Риск на сделку должен быть положительным. \
                 Получено: {:.2}%",
                risk_per_trade
            );
        }

        if risk_per_trade > 10.0 {
            panic!(
                "КРИТИЧЕСКАЯ ОШИБКА: Риск на сделку превышает безопасный лимит 10%. \
                 Получено: {:.2}%",
                risk_per_trade
            );
        }

        if max_drawdown <= 0.0 || max_drawdown > 50.0 {
            panic!(
                "КРИТИЧЕСКАЯ ОШИБКА: Макс. просадка должна быть в диапазоне 0-50%. \
                 Получено: {:.2}%",
                max_drawdown
            );
        }

        println!("✓ Торговая система инициализирована успешно");
        println!("  Баланс: ${:.2}", balance);
        println!("  Риск на сделку: {:.2}%", risk_per_trade);
        println!("  Макс. просадка: {:.2}%", max_drawdown);

        TradingSystem {
            balance,
            risk_per_trade,
            max_drawdown,
            is_active: true,
        }
    }
}
```

## Перехват panic с catch_unwind

```rust
use std::panic;

fn main() {
    println!("Тестирование перехвата panic...\n");

    // Перехват panic для изоляции ошибок
    let result = panic::catch_unwind(|| {
        risky_calculation(-100.0)
    });

    match result {
        Ok(value) => println!("Результат: {}", value),
        Err(_) => println!("Функция запаниковала, но программа продолжает работу"),
    }

    println!("\nПрограмма продолжает выполнение после перехвата panic!");

    // Нормальный вызов
    let safe_result = panic::catch_unwind(|| {
        risky_calculation(100.0)
    });

    match safe_result {
        Ok(value) => println!("Безопасный результат: {}", value),
        Err(_) => println!("Неожиданная паника"),
    }
}

fn risky_calculation(value: f64) -> f64 {
    if value < 0.0 {
        panic!("Отрицательное значение недопустимо: {}", value);
    }
    value * 2.0
}
```

## assert! макросы для тестирования инвариантов

```rust
fn main() {
    let entry_price = 42000.0;
    let exit_price = 43500.0;
    let quantity = 0.5;

    // assert! - проверка условия
    assert!(entry_price > 0.0, "Цена входа должна быть положительной");
    assert!(quantity > 0.0, "Количество должно быть положительным");

    let pnl = calculate_pnl(entry_price, exit_price, quantity);
    println!("PnL: ${:.2}", pnl);

    // assert_eq! - проверка равенства
    let expected_pnl = 750.0;
    assert_eq!(pnl, expected_pnl, "PnL должен равняться $750");

    // assert_ne! - проверка неравенства
    assert_ne!(pnl, 0.0, "PnL не должен быть нулевым");

    println!("Все проверки пройдены!");
}

fn calculate_pnl(entry: f64, exit: f64, qty: f64) -> f64 {
    (exit - entry) * qty
}
```

## debug_assert! для проверок только в debug-режиме

```rust
fn main() {
    let prices = vec![42000.0, 42100.0, 41900.0, 42300.0, 42200.0];

    let sma = calculate_sma(&prices, 3);
    println!("SMA-3: ${:.2}", sma);
}

fn calculate_sma(prices: &[f64], period: usize) -> f64 {
    // Эта проверка выполняется только в debug-режиме
    debug_assert!(
        !prices.is_empty(),
        "Массив цен не должен быть пустым"
    );
    debug_assert!(
        period > 0,
        "Период должен быть положительным"
    );
    debug_assert!(
        period <= prices.len(),
        "Период ({}) не может превышать количество цен ({})",
        period,
        prices.len()
    );

    let slice = &prices[prices.len() - period..];
    let sum: f64 = slice.iter().sum();
    sum / period as f64
}
```

## Рекомендации: когда panic, когда Result

```rust
fn main() {
    println!("=== Руководство по выбору между panic! и Result ===\n");

    // ИСПОЛЬЗУЙ panic! КОГДА:
    println!("Используй panic! когда:");
    println!("  • Нарушен критический инвариант программы");
    println!("  • Ошибка конфигурации при запуске");
    println!("  • Код теоретически недостижим");
    println!("  • В тестах для проверки условий");
    println!("  • Ошибка программиста (баг), а не пользователя\n");

    // ИСПОЛЬЗУЙ Result КОГДА:
    println!("Используй Result когда:");
    println!("  • Ошибка ожидаема и восстановима");
    println!("  • Пользователь может исправить ввод");
    println!("  • Сетевой запрос может не удасться");
    println!("  • Файл может не существовать");
    println!("  • Парсинг может провалиться\n");

    // Примеры
    println!("=== Примеры ===\n");

    // Это должен быть panic! - нарушение инварианта
    let portfolio_value = 50000.0;
    let position_value = 45000.0;
    let exposure = position_value / portfolio_value;

    if exposure > 1.0 {
        panic!("Невозможно: экспозиция > 100%");
    }
    println!("Экспозиция: {:.1}% - OK", exposure * 100.0);

    // Это должен быть Result - пользовательский ввод
    match parse_order_quantity("abc") {
        Ok(qty) => println!("Количество: {}", qty),
        Err(e) => println!("Ошибка ввода (ожидаемо): {}", e),
    }
}

fn parse_order_quantity(input: &str) -> Result<f64, String> {
    input
        .parse::<f64>()
        .map_err(|_| format!("'{}' не является числом", input))
}
```

## Практические упражнения

### Упражнение 1: Валидация ордера
Напиши функцию `validate_order`, которая паникует при критических ошибках и возвращает `Result` для восстановимых.

### Упражнение 2: Безопасный доступ к данным
Реализуй функцию получения исторических данных с правильным использованием `panic!` vs `Option`.

### Упражнение 3: Система проверок
Создай набор `assert!` проверок для валидации торговой стратегии.

### Упражнение 4: Изоляция panic
Используй `catch_unwind` для изоляции потенциально паникующего кода в торговом боте.

## Что мы узнали

| Инструмент | Когда использовать |
|------------|-------------------|
| `panic!()` | Невосстановимая ошибка, нарушение инварианта |
| `unreachable!()` | Код, который никогда не должен выполняться |
| `assert!()` | Проверка условия (паникует если false) |
| `assert_eq!()` | Проверка равенства |
| `debug_assert!()` | Проверка только в debug-режиме |
| `unwrap()` | Извлечение значения (паникует при ошибке) |
| `expect()` | `unwrap` с пользовательским сообщением |
| `catch_unwind()` | Перехват panic для изоляции |

## Домашнее задание

1. Создай структуру `RiskManager` с конструктором, который паникует при невалидных параметрах риска

2. Напиши функцию валидации портфеля, использующую комбинацию `panic!` для критических ошибок и `Result` для восстановимых

3. Реализуй набор `debug_assert!` проверок для функции расчёта позиции, которые не влияют на производительность в release-сборке

4. Создай "песочницу" для тестирования стратегий с использованием `catch_unwind` для изоляции паникующих стратегий

## Навигация

[← Предыдущий день](../091-propagating-errors/ru.md) | [Следующий день →](../093-custom-error-types/ru.md)
