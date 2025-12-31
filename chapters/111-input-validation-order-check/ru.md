# День 111: Проверка ввода — валидация торгового ордера

## Аналогия из трейдинга

Представь, что ты работаешь на бирже. Перед тем как принять ордер от клиента, система **обязана** проверить его корректность:
- Цена положительная?
- Количество в допустимых пределах?
- Достаточно ли средств на счёте?
- Тикер существует?

Если хотя бы одна проверка не пройдена — ордер отклоняется **до** попытки исполнения. Это защищает и клиента, и биржу от ошибок. В программировании это называется **валидацией ввода** (input validation).

## Зачем нужна валидация?

```rust
// ❌ Опасно: нет проверок
fn execute_order_unsafe(price: f64, quantity: f64) -> f64 {
    price * quantity  // Что если price = -1000 или quantity = 0?
}

// ✅ Безопасно: с валидацией
fn execute_order_safe(price: f64, quantity: f64) -> Result<f64, String> {
    if price <= 0.0 {
        return Err(String::from("Цена должна быть положительной"));
    }
    if quantity <= 0.0 {
        return Err(String::from("Количество должно быть положительным"));
    }
    Ok(price * quantity)
}
```

## Базовые паттерны валидации

### 1. Проверка числовых значений

```rust
fn main() {
    // Валидация цены
    println!("{:?}", validate_price(42000.0));  // Ok
    println!("{:?}", validate_price(-100.0));   // Err
    println!("{:?}", validate_price(0.0));      // Err

    // Валидация количества
    println!("{:?}", validate_quantity(0.5, 0.001, 100.0));  // Ok
    println!("{:?}", validate_quantity(0.0001, 0.001, 100.0)); // Err: слишком мало
    println!("{:?}", validate_quantity(150.0, 0.001, 100.0));  // Err: слишком много
}

fn validate_price(price: f64) -> Result<f64, String> {
    if price.is_nan() {
        return Err(String::from("Цена не может быть NaN"));
    }
    if price.is_infinite() {
        return Err(String::from("Цена не может быть бесконечной"));
    }
    if price <= 0.0 {
        return Err(String::from("Цена должна быть положительной"));
    }
    Ok(price)
}

fn validate_quantity(qty: f64, min: f64, max: f64) -> Result<f64, String> {
    if qty.is_nan() || qty.is_infinite() {
        return Err(String::from("Некорректное значение количества"));
    }
    if qty < min {
        return Err(format!("Количество {} меньше минимума {}", qty, min));
    }
    if qty > max {
        return Err(format!("Количество {} больше максимума {}", qty, max));
    }
    Ok(qty)
}
```

### 2. Проверка строковых значений

```rust
fn main() {
    println!("{:?}", validate_ticker("BTCUSDT"));  // Ok
    println!("{:?}", validate_ticker(""));         // Err: пустой
    println!("{:?}", validate_ticker("btc@usdt")); // Err: спецсимволы
    println!("{:?}", validate_ticker("AB"));       // Err: слишком короткий
}

fn validate_ticker(ticker: &str) -> Result<&str, String> {
    if ticker.is_empty() {
        return Err(String::from("Тикер не может быть пустым"));
    }
    if ticker.len() < 3 {
        return Err(String::from("Тикер слишком короткий (минимум 3 символа)"));
    }
    if ticker.len() > 20 {
        return Err(String::from("Тикер слишком длинный (максимум 20 символов)"));
    }
    if !ticker.chars().all(|c| c.is_alphanumeric()) {
        return Err(String::from("Тикер может содержать только буквы и цифры"));
    }
    Ok(ticker)
}
```

### 3. Проверка диапазонов

```rust
fn main() {
    // Процент риска
    println!("{:?}", validate_risk_percent(2.0));   // Ok
    println!("{:?}", validate_risk_percent(-1.0));  // Err
    println!("{:?}", validate_risk_percent(150.0)); // Err

    // Stop-loss должен быть ниже entry для лонга
    println!("{:?}", validate_stop_loss(42000.0, 41000.0, true));  // Ok
    println!("{:?}", validate_stop_loss(42000.0, 43000.0, true));  // Err
}

fn validate_risk_percent(risk: f64) -> Result<f64, String> {
    if risk <= 0.0 {
        return Err(String::from("Риск должен быть положительным"));
    }
    if risk > 100.0 {
        return Err(String::from("Риск не может превышать 100%"));
    }
    if risk > 10.0 {
        // Предупреждение, но не ошибка
        println!("⚠️  Предупреждение: высокий риск {}%", risk);
    }
    Ok(risk)
}

fn validate_stop_loss(entry: f64, stop_loss: f64, is_long: bool) -> Result<f64, String> {
    if is_long {
        if stop_loss >= entry {
            return Err(format!(
                "Stop-loss ({}) должен быть ниже цены входа ({}) для длинной позиции",
                stop_loss, entry
            ));
        }
    } else {
        if stop_loss <= entry {
            return Err(format!(
                "Stop-loss ({}) должен быть выше цены входа ({}) для короткой позиции",
                stop_loss, entry
            ));
        }
    }
    Ok(stop_loss)
}
```

## Комплексная валидация ордера

```rust
fn main() {
    let order1 = OrderInput {
        ticker: String::from("BTCUSDT"),
        side: String::from("BUY"),
        price: 42000.0,
        quantity: 0.5,
        stop_loss: Some(41000.0),
        take_profit: Some(45000.0),
    };

    let order2 = OrderInput {
        ticker: String::from(""),
        side: String::from("INVALID"),
        price: -100.0,
        quantity: 0.0,
        stop_loss: None,
        take_profit: None,
    };

    match validate_order(&order1) {
        Ok(valid) => println!("✅ Ордер валиден: {:?}", valid),
        Err(errors) => println!("❌ Ошибки: {:?}", errors),
    }

    match validate_order(&order2) {
        Ok(valid) => println!("✅ Ордер валиден: {:?}", valid),
        Err(errors) => println!("❌ Ошибки: {:?}", errors),
    }
}

#[derive(Debug)]
struct OrderInput {
    ticker: String,
    side: String,      // "BUY" или "SELL"
    price: f64,
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

#[derive(Debug)]
struct ValidatedOrder {
    ticker: String,
    is_buy: bool,
    price: f64,
    quantity: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    total_value: f64,
}

fn validate_order(input: &OrderInput) -> Result<ValidatedOrder, Vec<String>> {
    let mut errors = Vec::new();

    // Валидация тикера
    if input.ticker.is_empty() {
        errors.push(String::from("Тикер не может быть пустым"));
    } else if input.ticker.len() < 3 {
        errors.push(String::from("Тикер слишком короткий"));
    }

    // Валидация стороны
    let is_buy = match input.side.to_uppercase().as_str() {
        "BUY" | "LONG" => true,
        "SELL" | "SHORT" => false,
        _ => {
            errors.push(format!("Неизвестная сторона: {}", input.side));
            true // значение по умолчанию для продолжения валидации
        }
    };

    // Валидация цены
    if input.price <= 0.0 {
        errors.push(String::from("Цена должна быть положительной"));
    }

    // Валидация количества
    if input.quantity <= 0.0 {
        errors.push(String::from("Количество должно быть положительным"));
    }

    // Валидация stop-loss (если указан)
    if let Some(sl) = input.stop_loss {
        if sl <= 0.0 {
            errors.push(String::from("Stop-loss должен быть положительным"));
        } else if is_buy && sl >= input.price {
            errors.push(String::from("Stop-loss должен быть ниже цены для покупки"));
        } else if !is_buy && sl <= input.price {
            errors.push(String::from("Stop-loss должен быть выше цены для продажи"));
        }
    }

    // Валидация take-profit (если указан)
    if let Some(tp) = input.take_profit {
        if tp <= 0.0 {
            errors.push(String::from("Take-profit должен быть положительным"));
        } else if is_buy && tp <= input.price {
            errors.push(String::from("Take-profit должен быть выше цены для покупки"));
        } else if !is_buy && tp >= input.price {
            errors.push(String::from("Take-profit должен быть ниже цены для продажи"));
        }
    }

    // Если есть ошибки, возвращаем их
    if !errors.is_empty() {
        return Err(errors);
    }

    // Создаём валидированный ордер
    Ok(ValidatedOrder {
        ticker: input.ticker.clone(),
        is_buy,
        price: input.price,
        quantity: input.quantity,
        stop_loss: input.stop_loss,
        take_profit: input.take_profit,
        total_value: input.price * input.quantity,
    })
}
```

## Валидация с накоплением ошибок

```rust
fn main() {
    let params = TradingParams {
        balance: -1000.0,      // Ошибка
        risk_percent: 150.0,   // Ошибка
        max_positions: 0,      // Ошибка
        min_trade_size: -1.0,  // Ошибка
    };

    match validate_trading_params(&params) {
        Ok(valid) => println!("Параметры валидны: {:?}", valid),
        Err(errors) => {
            println!("Найдено {} ошибок:", errors.len());
            for (i, err) in errors.iter().enumerate() {
                println!("  {}. {}", i + 1, err);
            }
        }
    }
}

#[derive(Debug)]
struct TradingParams {
    balance: f64,
    risk_percent: f64,
    max_positions: usize,
    min_trade_size: f64,
}

#[derive(Debug)]
struct ValidatedParams {
    balance: f64,
    risk_percent: f64,
    max_positions: usize,
    min_trade_size: f64,
    max_risk_per_trade: f64,
}

fn validate_trading_params(params: &TradingParams) -> Result<ValidatedParams, Vec<String>> {
    let mut errors = Vec::new();

    if params.balance <= 0.0 {
        errors.push(format!(
            "Баланс должен быть положительным, получено: {}",
            params.balance
        ));
    }

    if params.risk_percent <= 0.0 || params.risk_percent > 100.0 {
        errors.push(format!(
            "Процент риска должен быть от 0 до 100, получено: {}",
            params.risk_percent
        ));
    }

    if params.max_positions == 0 {
        errors.push(String::from("Максимум позиций должен быть больше 0"));
    }

    if params.min_trade_size <= 0.0 {
        errors.push(format!(
            "Минимальный размер сделки должен быть положительным, получено: {}",
            params.min_trade_size
        ));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(ValidatedParams {
        balance: params.balance,
        risk_percent: params.risk_percent,
        max_positions: params.max_positions,
        min_trade_size: params.min_trade_size,
        max_risk_per_trade: params.balance * (params.risk_percent / 100.0),
    })
}
```

## Валидация с использованием типов

```rust
fn main() {
    // Создание безопасных типов
    match Price::new(42000.0) {
        Ok(price) => println!("Цена: {}", price.value()),
        Err(e) => println!("Ошибка: {}", e),
    }

    match Quantity::new(0.5, 0.001, 100.0) {
        Ok(qty) => println!("Количество: {}", qty.value()),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Использование валидированных типов
    let price = Price::new(42000.0).unwrap();
    let qty = Quantity::new(0.5, 0.001, 100.0).unwrap();

    println!("Общая стоимость: {}", calculate_total(&price, &qty));
}

#[derive(Debug, Clone, Copy)]
struct Price(f64);

impl Price {
    fn new(value: f64) -> Result<Self, String> {
        if value.is_nan() || value.is_infinite() {
            return Err(String::from("Некорректное значение цены"));
        }
        if value <= 0.0 {
            return Err(String::from("Цена должна быть положительной"));
        }
        Ok(Price(value))
    }

    fn value(&self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
struct Quantity(f64);

impl Quantity {
    fn new(value: f64, min: f64, max: f64) -> Result<Self, String> {
        if value.is_nan() || value.is_infinite() {
            return Err(String::from("Некорректное значение количества"));
        }
        if value < min {
            return Err(format!("Количество меньше минимума: {} < {}", value, min));
        }
        if value > max {
            return Err(format!("Количество больше максимума: {} > {}", value, max));
        }
        Ok(Quantity(value))
    }

    fn value(&self) -> f64 {
        self.0
    }
}

// Функция принимает только валидированные типы
fn calculate_total(price: &Price, qty: &Quantity) -> f64 {
    price.value() * qty.value()
}
```

## Практический пример: система валидации ордеров

```rust
fn main() {
    let validator = OrderValidator::new(
        10000.0,    // баланс
        100.0,      // макс. размер позиции
        0.001,      // мин. количество
        100.0,      // макс. количество
    );

    // Тестовые ордера
    let orders = vec![
        ("BTCUSDT", 42000.0, 0.1),
        ("BTCUSDT", 42000.0, 1.0),      // Превышает баланс
        ("ETHUSDT", -1500.0, 0.5),      // Отрицательная цена
        ("", 100.0, 1.0),               // Пустой тикер
    ];

    for (ticker, price, qty) in orders {
        println!("\n📋 Проверка: {} @ {} x {}", ticker, price, qty);
        match validator.validate(ticker, price, qty) {
            Ok(order) => {
                println!("  ✅ Принят");
                println!("  💰 Стоимость: ${:.2}", order.total_value);
            }
            Err(errors) => {
                println!("  ❌ Отклонён:");
                for err in errors {
                    println!("     - {}", err);
                }
            }
        }
    }
}

struct OrderValidator {
    balance: f64,
    max_position_value: f64,
    min_quantity: f64,
    max_quantity: f64,
}

struct ValidOrder {
    ticker: String,
    price: f64,
    quantity: f64,
    total_value: f64,
}

impl OrderValidator {
    fn new(balance: f64, max_position_value: f64, min_qty: f64, max_qty: f64) -> Self {
        OrderValidator {
            balance,
            max_position_value,
            min_quantity: min_qty,
            max_quantity: max_qty,
        }
    }

    fn validate(&self, ticker: &str, price: f64, quantity: f64) -> Result<ValidOrder, Vec<String>> {
        let mut errors = Vec::new();

        // Валидация тикера
        if ticker.is_empty() {
            errors.push(String::from("Тикер обязателен"));
        } else if !ticker.chars().all(|c| c.is_alphanumeric()) {
            errors.push(String::from("Тикер содержит недопустимые символы"));
        }

        // Валидация цены
        if price <= 0.0 {
            errors.push(String::from("Цена должна быть положительной"));
        } else if price.is_nan() || price.is_infinite() {
            errors.push(String::from("Некорректное значение цены"));
        }

        // Валидация количества
        if quantity < self.min_quantity {
            errors.push(format!(
                "Количество {} меньше минимума {}",
                quantity, self.min_quantity
            ));
        }
        if quantity > self.max_quantity {
            errors.push(format!(
                "Количество {} больше максимума {}",
                quantity, self.max_quantity
            ));
        }

        // Проверка стоимости (только если цена и количество валидны)
        if price > 0.0 && quantity > 0.0 {
            let total = price * quantity;

            if total > self.balance {
                errors.push(format!(
                    "Недостаточно средств: нужно ${:.2}, доступно ${:.2}",
                    total, self.balance
                ));
            }

            if total > self.max_position_value {
                errors.push(format!(
                    "Превышен лимит позиции: ${:.2} > ${:.2}",
                    total, self.max_position_value
                ));
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidOrder {
            ticker: ticker.to_string(),
            price,
            quantity,
            total_value: price * quantity,
        })
    }
}
```

## Что мы узнали

| Паттерн | Применение | Пример из трейдинга |
|---------|------------|---------------------|
| Раннее возвращение | Быстрая проверка критичных условий | Цена > 0 |
| Накопление ошибок | Показать все проблемы сразу | Все поля ордера |
| Типы-обёртки | Гарантия валидности на уровне типов | Price, Quantity |
| Комплексная валидация | Проверка связанных полей | Stop-loss vs Entry |

## Ключевые правила валидации в трейдинге

1. **Всегда проверяй на NaN и Infinity** — числа с плавающей точкой коварны
2. **Проверяй границы** — минимумы и максимумы для всех значений
3. **Валидируй связи** — stop-loss должен соответствовать направлению позиции
4. **Накапливай ошибки** — пользователю удобнее видеть все проблемы сразу
5. **Используй типы** — валидированные типы не требуют повторной проверки

## Домашнее задание

1. Напиши функцию `validate_portfolio_allocation(allocations: &[f64]) -> Result<(), String>`, которая проверяет, что сумма аллокаций равна 100%

2. Создай валидатор для параметров торговой стратегии:
   - SMA period (целое, от 1 до 200)
   - RSI period (целое, от 2 до 100)
   - Risk per trade (от 0.1% до 5%)
   - Take-profit ratio (от 1.0 до 10.0)

3. Реализуй тип `RiskPercentage`, который:
   - Принимает значения от 0.1 до 100.0
   - Имеет метод `of_balance(balance: f64) -> f64`
   - Автоматически округляет до 2 знаков после запятой

4. Напиши функцию валидации массива исторических цен:
   - Не пустой массив
   - Все значения положительные
   - Нет резких скачков (более 50% за одну свечу)
   - Возвращает `Vec<String>` со всеми найденными аномалиями

## Навигация

[← Предыдущий день](../110-error-matching-patterns/ru.md) | [Следующий день →](../112-error-custom-types/ru.md)
