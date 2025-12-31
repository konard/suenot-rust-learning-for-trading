# День 118: Паттерн: Fail Fast

## Аналогия из трейдинга

Представь, что ты торгуешь на бирже и получаешь данные о цене актива. Что лучше:
- **Продолжать работать** с невалидными данными и потерять деньги на неправильной сделке?
- **Сразу остановиться** при первой же ошибке и сообщить о проблеме?

Ответ очевиден: **fail fast** (падай быстро) — это паттерн, при котором мы немедленно прекращаем работу при обнаружении ошибки, вместо того чтобы пытаться "протащить" сломанные данные дальше.

В трейдинге это критически важно:
- Получили пустую цену? Лучше не торговать, чем торговать с ценой 0
- Баланс отрицательный? Немедленно остановить бота
- API вернул ошибку? Не пытаться угадать данные

## Что такое Fail Fast?

**Fail Fast** — это философия разработки, при которой:
1. Ошибки обнаруживаются как можно раньше
2. При ошибке программа сразу сообщает о проблеме
3. Не происходит "тихих" ошибок, которые накапливаются

```rust
// Плохо: пытаемся продолжить с невалидными данными
fn bad_calculate_position_size(balance: f64, risk_percent: f64) -> f64 {
    if balance <= 0.0 {
        return 0.0; // Тихо возвращаем 0, скрывая проблему
    }
    balance * risk_percent / 100.0
}

// Хорошо: fail fast — сразу сообщаем об ошибке
fn good_calculate_position_size(balance: f64, risk_percent: f64) -> Result<f64, String> {
    if balance <= 0.0 {
        return Err(format!("Невалидный баланс: {}. Баланс должен быть положительным", balance));
    }
    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return Err(format!("Невалидный риск: {}%. Должен быть от 0 до 100", risk_percent));
    }
    Ok(balance * risk_percent / 100.0)
}

fn main() {
    // Плохой подход — ошибка скрыта
    let size1 = bad_calculate_position_size(-1000.0, 2.0);
    println!("Размер позиции (плохой): {}", size1); // 0.0 — но почему?

    // Хороший подход — ошибка видна сразу
    match good_calculate_position_size(-1000.0, 2.0) {
        Ok(size) => println!("Размер позиции: {}", size),
        Err(e) => println!("Ошибка: {}", e), // Сразу понятно, что не так
    }
}
```

## Fail Fast при валидации ордера

```rust
#[derive(Debug)]
struct Order {
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: f64,
}

#[derive(Debug)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
enum OrderError {
    EmptySymbol,
    InvalidQuantity(f64),
    InvalidPrice(f64),
    InsufficientBalance { required: f64, available: f64 },
}

impl std::fmt::Display for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderError::EmptySymbol => write!(f, "Символ не может быть пустым"),
            OrderError::InvalidQuantity(q) => write!(f, "Невалидное количество: {}", q),
            OrderError::InvalidPrice(p) => write!(f, "Невалидная цена: {}", p),
            OrderError::InsufficientBalance { required, available } => {
                write!(f, "Недостаточно средств: требуется {}, доступно {}", required, available)
            }
        }
    }
}

/// Валидирует ордер по принципу fail fast.
/// При первой же ошибке возвращает Err.
fn validate_order(order: &Order, balance: f64) -> Result<(), OrderError> {
    // Проверка 1: символ не пустой
    if order.symbol.is_empty() {
        return Err(OrderError::EmptySymbol);
    }

    // Проверка 2: количество положительное
    if order.quantity <= 0.0 {
        return Err(OrderError::InvalidQuantity(order.quantity));
    }

    // Проверка 3: цена положительная
    if order.price <= 0.0 {
        return Err(OrderError::InvalidPrice(order.price));
    }

    // Проверка 4: достаточно средств
    let required = order.quantity * order.price;
    if required > balance {
        return Err(OrderError::InsufficientBalance {
            required,
            available: balance,
        });
    }

    Ok(())
}

fn main() {
    let balance = 10000.0;

    // Невалидный ордер — пустой символ
    let order1 = Order {
        symbol: String::new(),
        side: OrderSide::Buy,
        quantity: 0.5,
        price: 42000.0,
    };

    match validate_order(&order1, balance) {
        Ok(()) => println!("Ордер валиден"),
        Err(e) => println!("Fail fast: {}", e),
    }

    // Невалидный ордер — отрицательное количество
    let order2 = Order {
        symbol: "BTC".to_string(),
        side: OrderSide::Buy,
        quantity: -1.0,
        price: 42000.0,
    };

    match validate_order(&order2, balance) {
        Ok(()) => println!("Ордер валиден"),
        Err(e) => println!("Fail fast: {}", e),
    }

    // Валидный ордер
    let order3 = Order {
        symbol: "BTC".to_string(),
        side: OrderSide::Buy,
        quantity: 0.1,
        price: 42000.0,
    };

    match validate_order(&order3, balance) {
        Ok(()) => println!("Ордер валиден, можно отправлять"),
        Err(e) => println!("Fail fast: {}", e),
    }
}
```

## Fail Fast с оператором ?

Оператор `?` в Rust — это идеальный инструмент для fail fast. Он немедленно прерывает выполнение функции при первой ошибке:

```rust
#[derive(Debug)]
struct TradeSignal {
    symbol: String,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
    position_size: f64,
}

#[derive(Debug)]
enum SignalError {
    InvalidPrice(String),
    StopLossAboveEntry,
    TakeProfitBelowEntry,
    InvalidRiskReward(f64),
}

impl std::fmt::Display for SignalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalError::InvalidPrice(msg) => write!(f, "Невалидная цена: {}", msg),
            SignalError::StopLossAboveEntry => write!(f, "Stop-loss выше цены входа"),
            SignalError::TakeProfitBelowEntry => write!(f, "Take-profit ниже цены входа"),
            SignalError::InvalidRiskReward(rr) => write!(f, "Плохой R:R = {:.2}, минимум 2.0", rr),
        }
    }
}

/// Создаёт торговый сигнал с проверками fail fast.
fn create_long_signal(
    symbol: &str,
    entry: f64,
    stop_loss: f64,
    take_profit: f64,
    position_size: f64,
) -> Result<TradeSignal, SignalError> {
    // Каждая проверка использует ?, чтобы немедленно вернуть ошибку
    validate_price(entry, "entry")?;
    validate_price(stop_loss, "stop_loss")?;
    validate_price(take_profit, "take_profit")?;
    validate_stop_loss_for_long(entry, stop_loss)?;
    validate_take_profit_for_long(entry, take_profit)?;
    validate_risk_reward(entry, stop_loss, take_profit)?;

    Ok(TradeSignal {
        symbol: symbol.to_string(),
        entry_price: entry,
        stop_loss,
        take_profit,
        position_size,
    })
}

fn validate_price(price: f64, name: &str) -> Result<(), SignalError> {
    if price <= 0.0 || price.is_nan() || price.is_infinite() {
        return Err(SignalError::InvalidPrice(format!("{} = {}", name, price)));
    }
    Ok(())
}

fn validate_stop_loss_for_long(entry: f64, stop_loss: f64) -> Result<(), SignalError> {
    if stop_loss >= entry {
        return Err(SignalError::StopLossAboveEntry);
    }
    Ok(())
}

fn validate_take_profit_for_long(entry: f64, take_profit: f64) -> Result<(), SignalError> {
    if take_profit <= entry {
        return Err(SignalError::TakeProfitBelowEntry);
    }
    Ok(())
}

fn validate_risk_reward(entry: f64, stop_loss: f64, take_profit: f64) -> Result<(), SignalError> {
    let risk = entry - stop_loss;
    let reward = take_profit - entry;
    let rr = reward / risk;
    if rr < 2.0 {
        return Err(SignalError::InvalidRiskReward(rr));
    }
    Ok(())
}

fn main() {
    // Попытка создать сигнал с невалидной ценой — fail fast на первой проверке
    match create_long_signal("BTC", -100.0, 40000.0, 45000.0, 0.1) {
        Ok(signal) => println!("Сигнал создан: {:?}", signal),
        Err(e) => println!("Ошибка создания сигнала: {}", e),
    }

    // Stop-loss выше entry — fail fast
    match create_long_signal("BTC", 42000.0, 43000.0, 45000.0, 0.1) {
        Ok(signal) => println!("Сигнал создан: {:?}", signal),
        Err(e) => println!("Ошибка создания сигнала: {}", e),
    }

    // Плохой Risk:Reward — fail fast
    match create_long_signal("BTC", 42000.0, 41000.0, 42500.0, 0.1) {
        Ok(signal) => println!("Сигнал создан: {:?}", signal),
        Err(e) => println!("Ошибка создания сигнала: {}", e),
    }

    // Валидный сигнал
    match create_long_signal("BTC", 42000.0, 41000.0, 45000.0, 0.1) {
        Ok(signal) => println!("Сигнал создан: {:?}", signal),
        Err(e) => println!("Ошибка создания сигнала: {}", e),
    }
}
```

## Fail Fast в конструкторах

Лучшее место для fail fast — конструктор. Если объект нельзя создать в валидном состоянии, лучше не создавать его вовсе:

```rust
#[derive(Debug)]
struct Portfolio {
    name: String,
    balance: f64,
    max_positions: usize,
    risk_per_trade: f64,
}

#[derive(Debug)]
enum PortfolioError {
    EmptyName,
    NegativeBalance(f64),
    ZeroPositions,
    InvalidRisk(f64),
}

impl std::fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortfolioError::EmptyName => write!(f, "Имя портфеля не может быть пустым"),
            PortfolioError::NegativeBalance(b) => write!(f, "Баланс не может быть отрицательным: {}", b),
            PortfolioError::ZeroPositions => write!(f, "Количество позиций должно быть больше 0"),
            PortfolioError::InvalidRisk(r) => write!(f, "Риск должен быть от 0.1% до 10%: {}%", r),
        }
    }
}

impl Portfolio {
    /// Конструктор с fail fast валидацией.
    /// Гарантирует, что Portfolio всегда в валидном состоянии.
    pub fn new(
        name: String,
        balance: f64,
        max_positions: usize,
        risk_per_trade: f64,
    ) -> Result<Self, PortfolioError> {
        // Fail fast: проверяем все инварианты сразу
        if name.trim().is_empty() {
            return Err(PortfolioError::EmptyName);
        }

        if balance < 0.0 {
            return Err(PortfolioError::NegativeBalance(balance));
        }

        if max_positions == 0 {
            return Err(PortfolioError::ZeroPositions);
        }

        if risk_per_trade < 0.1 || risk_per_trade > 10.0 {
            return Err(PortfolioError::InvalidRisk(risk_per_trade));
        }

        // Все проверки пройдены — создаём объект
        Ok(Portfolio {
            name,
            balance,
            max_positions,
            risk_per_trade,
        })
    }

    pub fn calculate_position_size(&self, entry: f64, stop_loss: f64) -> f64 {
        let risk_amount = self.balance * (self.risk_per_trade / 100.0);
        let price_risk = (entry - stop_loss).abs();
        if price_risk == 0.0 {
            0.0
        } else {
            risk_amount / price_risk
        }
    }
}

fn main() {
    // Попытки создать невалидный портфель — fail fast
    println!("Попытка 1: пустое имя");
    match Portfolio::new("".to_string(), 10000.0, 5, 2.0) {
        Ok(p) => println!("Создан: {:?}", p),
        Err(e) => println!("Ошибка: {}", e),
    }

    println!("\nПопытка 2: отрицательный баланс");
    match Portfolio::new("Main".to_string(), -5000.0, 5, 2.0) {
        Ok(p) => println!("Создан: {:?}", p),
        Err(e) => println!("Ошибка: {}", e),
    }

    println!("\nПопытка 3: слишком высокий риск");
    match Portfolio::new("Main".to_string(), 10000.0, 5, 50.0) {
        Ok(p) => println!("Создан: {:?}", p),
        Err(e) => println!("Ошибка: {}", e),
    }

    println!("\nПопытка 4: валидный портфель");
    match Portfolio::new("Main Portfolio".to_string(), 10000.0, 5, 2.0) {
        Ok(p) => {
            println!("Создан: {:?}", p);
            let size = p.calculate_position_size(42000.0, 41000.0);
            println!("Размер позиции: {:.6} BTC", size);
        }
        Err(e) => println!("Ошибка: {}", e),
    }
}
```

## Fail Fast vs. Collect All Errors

Иногда нужно собрать все ошибки сразу, а не останавливаться на первой. Это полезно для UI, где пользователю нужно показать все проблемы разом:

```rust
#[derive(Debug)]
struct OrderValidation {
    symbol: String,
    quantity: f64,
    price: f64,
    stop_loss: Option<f64>,
}

#[derive(Debug)]
struct ValidationErrors {
    errors: Vec<String>,
}

impl ValidationErrors {
    fn new() -> Self {
        ValidationErrors { errors: Vec::new() }
    }

    fn add(&mut self, error: String) {
        self.errors.push(error);
    }

    fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    fn into_result(self) -> Result<(), Self> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "  - {}", error)?;
        }
        Ok(())
    }
}

/// Собирает все ошибки валидации (не fail fast).
/// Полезно для показа всех проблем пользователю сразу.
fn validate_order_collect_all(order: &OrderValidation) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();

    if order.symbol.is_empty() {
        errors.add("Символ не может быть пустым".to_string());
    }

    if order.quantity <= 0.0 {
        errors.add(format!("Количество должно быть положительным: {}", order.quantity));
    }

    if order.price <= 0.0 {
        errors.add(format!("Цена должна быть положительной: {}", order.price));
    }

    if let Some(sl) = order.stop_loss {
        if sl <= 0.0 {
            errors.add(format!("Stop-loss должен быть положительным: {}", sl));
        }
        if sl >= order.price {
            errors.add(format!("Stop-loss ({}) должен быть ниже цены входа ({})", sl, order.price));
        }
    }

    errors.into_result()
}

fn main() {
    // Ордер с множеством ошибок
    let bad_order = OrderValidation {
        symbol: String::new(),
        quantity: -1.0,
        price: 0.0,
        stop_loss: Some(-100.0),
    };

    println!("Валидация плохого ордера:");
    match validate_order_collect_all(&bad_order) {
        Ok(()) => println!("Ордер валиден"),
        Err(errors) => {
            println!("Найдены ошибки:\n{}", errors);
        }
    }

    println!("\nВалидация хорошего ордера:");
    let good_order = OrderValidation {
        symbol: "BTC".to_string(),
        quantity: 0.5,
        price: 42000.0,
        stop_loss: Some(41000.0),
    };

    match validate_order_collect_all(&good_order) {
        Ok(()) => println!("Ордер валиден!"),
        Err(errors) => println!("Найдены ошибки:\n{}", errors),
    }
}
```

## Когда использовать Fail Fast

| Ситуация | Подход | Причина |
|----------|--------|---------|
| API запрос | Fail fast | Невалидные данные нельзя отправить |
| Конструктор объекта | Fail fast | Объект должен быть всегда валиден |
| Критические проверки | Fail fast | Безопасность важнее UX |
| Форма пользователя | Collect all | UX: показать все ошибки сразу |
| Batch обработка | Collect all | Обработать что можно, логировать ошибки |
| Парсинг конфига | Fail fast | Без конфига работать нельзя |

## Практический пример: Торговый бот с Fail Fast

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct TradingBot {
    name: String,
    api_key: String,
    balance: f64,
    positions: HashMap<String, f64>,
    max_positions: usize,
    risk_per_trade: f64,
}

#[derive(Debug)]
enum BotError {
    EmptyName,
    EmptyApiKey,
    NegativeBalance(f64),
    InvalidRisk(f64),
    TooManyPositions { current: usize, max: usize },
    InsufficientBalance { required: f64, available: f64 },
    PositionNotFound(String),
}

impl std::fmt::Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotError::EmptyName => write!(f, "Имя бота не может быть пустым"),
            BotError::EmptyApiKey => write!(f, "API ключ обязателен"),
            BotError::NegativeBalance(b) => write!(f, "Баланс не может быть отрицательным: {}", b),
            BotError::InvalidRisk(r) => write!(f, "Риск должен быть 0.1-5%: {}%", r),
            BotError::TooManyPositions { current, max } => {
                write!(f, "Превышен лимит позиций: {} из {}", current, max)
            }
            BotError::InsufficientBalance { required, available } => {
                write!(f, "Недостаточно средств: нужно ${:.2}, есть ${:.2}", required, available)
            }
            BotError::PositionNotFound(s) => write!(f, "Позиция не найдена: {}", s),
        }
    }
}

impl TradingBot {
    /// Конструктор с fail fast — все проверки до создания объекта.
    pub fn new(
        name: String,
        api_key: String,
        balance: f64,
        max_positions: usize,
        risk_per_trade: f64,
    ) -> Result<Self, BotError> {
        if name.trim().is_empty() {
            return Err(BotError::EmptyName);
        }
        if api_key.trim().is_empty() {
            return Err(BotError::EmptyApiKey);
        }
        if balance < 0.0 {
            return Err(BotError::NegativeBalance(balance));
        }
        if risk_per_trade < 0.1 || risk_per_trade > 5.0 {
            return Err(BotError::InvalidRisk(risk_per_trade));
        }

        Ok(TradingBot {
            name,
            api_key,
            balance,
            positions: HashMap::new(),
            max_positions,
            risk_per_trade,
        })
    }

    /// Открытие позиции с fail fast проверками.
    pub fn open_position(&mut self, symbol: &str, quantity: f64, price: f64) -> Result<(), BotError> {
        // Fail fast: проверяем лимит позиций
        if self.positions.len() >= self.max_positions {
            return Err(BotError::TooManyPositions {
                current: self.positions.len(),
                max: self.max_positions,
            });
        }

        // Fail fast: проверяем баланс
        let cost = quantity * price;
        if cost > self.balance {
            return Err(BotError::InsufficientBalance {
                required: cost,
                available: self.balance,
            });
        }

        // Все проверки пройдены — открываем позицию
        self.balance -= cost;
        *self.positions.entry(symbol.to_string()).or_insert(0.0) += quantity;

        println!("[{}] Открыта позиция: {} {} @ ${:.2}", self.name, quantity, symbol, price);
        Ok(())
    }

    /// Закрытие позиции с fail fast.
    pub fn close_position(&mut self, symbol: &str, price: f64) -> Result<f64, BotError> {
        // Fail fast: позиция должна существовать
        let quantity = self.positions.remove(symbol)
            .ok_or_else(|| BotError::PositionNotFound(symbol.to_string()))?;

        let proceeds = quantity * price;
        self.balance += proceeds;

        println!("[{}] Закрыта позиция: {} {} @ ${:.2}", self.name, quantity, symbol, price);
        Ok(proceeds)
    }

    pub fn status(&self) {
        println!("\n=== {} ===", self.name);
        println!("Баланс: ${:.2}", self.balance);
        println!("Позиции: {:?}", self.positions);
        println!("Риск на сделку: {}%", self.risk_per_trade);
    }
}

fn main() {
    println!("=== Создание бота ===\n");

    // Fail fast при создании бота
    let bot_result = TradingBot::new(
        "AlphaBot".to_string(),
        "secret-api-key".to_string(),
        10000.0,
        3,
        2.0,
    );

    let mut bot = match bot_result {
        Ok(b) => b,
        Err(e) => {
            println!("Не удалось создать бота: {}", e);
            return;
        }
    };

    bot.status();

    println!("\n=== Открытие позиций ===\n");

    // Успешное открытие
    if let Err(e) = bot.open_position("BTC", 0.1, 42000.0) {
        println!("Ошибка: {}", e);
    }

    if let Err(e) = bot.open_position("ETH", 2.0, 2500.0) {
        println!("Ошибка: {}", e);
    }

    if let Err(e) = bot.open_position("SOL", 50.0, 95.0) {
        println!("Ошибка: {}", e);
    }

    // Fail fast: превышен лимит позиций
    println!("\nПопытка открыть 4-ю позицию:");
    if let Err(e) = bot.open_position("DOGE", 1000.0, 0.08) {
        println!("Fail fast: {}", e);
    }

    bot.status();

    // Fail fast: недостаточно средств
    println!("\nПопытка открыть большую позицию:");
    if let Err(e) = bot.open_position("BTC", 1.0, 42000.0) {
        println!("Fail fast: {}", e);
    }

    println!("\n=== Закрытие позиций ===\n");

    // Успешное закрытие
    match bot.close_position("ETH", 2600.0) {
        Ok(proceeds) => println!("Получено: ${:.2}", proceeds),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Fail fast: позиция не найдена
    println!("\nПопытка закрыть несуществующую позицию:");
    match bot.close_position("MATIC", 1.0) {
        Ok(proceeds) => println!("Получено: ${:.2}", proceeds),
        Err(e) => println!("Fail fast: {}", e),
    }

    bot.status();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Fail Fast | Немедленное прекращение работы при ошибке |
| Валидация в конструкторе | Гарантирует валидность объекта |
| Оператор `?` | Инструмент для fail fast |
| Ранний return | `return Err(...)` при первой проблеме |
| Collect All | Альтернатива для UI — собрать все ошибки |

## Домашнее задание

1. **Валидатор торговой стратегии**
   Напиши функцию `validate_strategy()`, которая проверяет:
   - Имя стратегии не пустое
   - Таймфрейм валиден (1m, 5m, 15m, 1h, 4h, 1d)
   - Risk:Reward не меньше 2.0
   - Максимальный риск на сделку не больше 3%
   Используй fail fast подход с оператором `?`

2. **Конструктор биржевого подключения**
   Создай структуру `ExchangeConnection` с полями:
   - `exchange_name: String`
   - `api_key: String`
   - `api_secret: String`
   - `rate_limit: u32` (запросов в секунду)
   Реализуй `new()` с fail fast валидацией всех полей.

3. **Парсер ордера из JSON**
   Напиши функцию `parse_order(json: &str) -> Result<Order, OrderParseError>`, которая:
   - Парсит JSON
   - Проверяет наличие обязательных полей
   - Валидирует значения
   Используй fail fast — при первой ошибке парсинга возвращай Err.

4. **Система проверки риска**
   Создай `RiskManager` с методом `check_trade()`, который:
   - Проверяет максимальный размер позиции
   - Проверяет дневной лимит убытков
   - Проверяет коррелированность с открытыми позициями
   Если любая проверка не прошла — fail fast.

## Навигация

[← Предыдущий день](../117-errors-in-async-preview/ru.md) | [Следующий день →](../119-pattern-error-as-value/ru.md)
