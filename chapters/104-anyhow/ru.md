# День 104: anyhow — Простая обработка ошибок

## Аналогия из трейдинга

Представь, что ты управляешь торговой системой. Когда что-то идёт не так — API биржи недоступен, ордер отклонён, данные повреждены — тебе нужно быстро понять, **что** произошло и **где**. Крейт `anyhow` — это как единый журнал всех инцидентов в твоей торговой системе: он собирает все ошибки в унифицированном формате, добавляет контекст и позволяет отследить цепочку событий, приведших к проблеме.

## Зачем нужен anyhow?

В приложениях (в отличие от библиотек) нам часто не нужны строго типизированные ошибки. Нам важнее:
- Быстро понять, что пошло не так
- Получить читаемое сообщение об ошибке
- Увидеть цепочку ошибок (error chain)
- Минимизировать шаблонный код

```rust
// Стандартная библиотека: много типов ошибок
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // ...
}

// С anyhow: просто и единообразно
use anyhow::Result;

fn load_config() -> Result<Config> {
    // ...
}
```

## Основные возможности

### 1. Тип `anyhow::Result`

```rust
use anyhow::{Result, anyhow};

fn fetch_market_price(symbol: &str) -> Result<f64> {
    if symbol.is_empty() {
        return Err(anyhow!("Symbol cannot be empty"));
    }

    // Симуляция получения цены
    let price = match symbol {
        "BTC" => 42000.0,
        "ETH" => 2500.0,
        _ => return Err(anyhow!("Unknown symbol: {}", symbol)),
    };

    Ok(price)
}

fn main() {
    match fetch_market_price("BTC") {
        Ok(price) => println!("BTC price: ${}", price),
        Err(e) => eprintln!("Error: {}", e),
    }

    match fetch_market_price("UNKNOWN") {
        Ok(price) => println!("Price: ${}", price),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 2. Макрос `anyhow!`

Создаёт ошибку из строки с форматированием:

```rust
use anyhow::{Result, anyhow};

fn validate_order(price: f64, quantity: f64, balance: f64) -> Result<()> {
    if price <= 0.0 {
        return Err(anyhow!("Invalid price: {} (must be positive)", price));
    }

    if quantity <= 0.0 {
        return Err(anyhow!("Invalid quantity: {} (must be positive)", quantity));
    }

    let total = price * quantity;
    if total > balance {
        return Err(anyhow!(
            "Insufficient balance: need ${:.2}, have ${:.2}",
            total,
            balance
        ));
    }

    Ok(())
}

fn main() {
    let balance = 10000.0;

    match validate_order(42000.0, 0.5, balance) {
        Ok(()) => println!("Order validated successfully"),
        Err(e) => eprintln!("Validation failed: {}", e),
    }

    match validate_order(-100.0, 1.0, balance) {
        Ok(()) => println!("Order validated"),
        Err(e) => eprintln!("Validation failed: {}", e),
    }
}
```

### 3. Макрос `bail!`

Упрощённый способ вернуть ошибку (комбинация `return Err(anyhow!(...))`:

```rust
use anyhow::{Result, bail};

fn execute_trade(symbol: &str, side: &str, quantity: f64) -> Result<String> {
    if quantity <= 0.0 {
        bail!("Quantity must be positive, got: {}", quantity);
    }

    if side != "BUY" && side != "SELL" {
        bail!("Invalid side '{}', expected BUY or SELL", side);
    }

    // Симуляция исполнения
    let order_id = format!("ORD-{}-{}", symbol, rand_id());
    Ok(order_id)
}

fn rand_id() -> u32 {
    12345 // В реальности — генератор случайных чисел
}

fn main() {
    match execute_trade("BTC", "BUY", 0.5) {
        Ok(id) => println!("Order executed: {}", id),
        Err(e) => eprintln!("Trade failed: {}", e),
    }

    match execute_trade("ETH", "HOLD", 1.0) {
        Ok(id) => println!("Order executed: {}", id),
        Err(e) => eprintln!("Trade failed: {}", e),
    }
}
```

### 4. Макрос `ensure!`

Проверка условия с автоматическим возвратом ошибки:

```rust
use anyhow::{Result, ensure};

fn calculate_position_size(
    balance: f64,
    risk_percent: f64,
    entry_price: f64,
    stop_loss: f64,
) -> Result<f64> {
    ensure!(balance > 0.0, "Balance must be positive: {}", balance);
    ensure!(
        risk_percent > 0.0 && risk_percent <= 100.0,
        "Risk percent must be between 0 and 100: {}",
        risk_percent
    );
    ensure!(entry_price > 0.0, "Entry price must be positive");
    ensure!(stop_loss > 0.0, "Stop loss must be positive");
    ensure!(
        entry_price != stop_loss,
        "Entry price and stop loss cannot be equal"
    );

    let risk_per_unit = (entry_price - stop_loss).abs();
    let risk_amount = balance * (risk_percent / 100.0);
    let position_size = risk_amount / risk_per_unit;

    Ok(position_size)
}

fn main() {
    match calculate_position_size(10000.0, 2.0, 42000.0, 41000.0) {
        Ok(size) => println!("Position size: {:.4} BTC", size),
        Err(e) => eprintln!("Error: {}", e),
    }

    match calculate_position_size(-1000.0, 2.0, 42000.0, 41000.0) {
        Ok(size) => println!("Position size: {:.4}", size),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Добавление контекста

### Метод `context()`

Добавляет контекст к ошибке, создавая цепочку:

```rust
use anyhow::{Result, Context};
use std::fs;

fn load_trading_config(path: &str) -> Result<String> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path))?;

    Ok(content)
}

fn parse_strategy_params(config: &str) -> Result<StrategyParams> {
    // Парсим конфигурацию
    let params = serde_json::from_str(config)
        .context("Failed to parse strategy parameters")?;

    Ok(params)
}

#[derive(Debug, serde::Deserialize)]
struct StrategyParams {
    symbol: String,
    timeframe: String,
    risk_percent: f64,
}

fn initialize_strategy(config_path: &str) -> Result<StrategyParams> {
    let config = load_trading_config(config_path)
        .context("Failed to load trading configuration")?;

    let params = parse_strategy_params(&config)
        .context("Failed to initialize strategy")?;

    Ok(params)
}

fn main() {
    match initialize_strategy("config.json") {
        Ok(params) => println!("Strategy initialized: {:?}", params),
        Err(e) => {
            eprintln!("Error: {}", e);
            // Вывод полной цепочки ошибок
            for cause in e.chain().skip(1) {
                eprintln!("Caused by: {}", cause);
            }
        }
    }
}
```

### Метод `with_context()`

Ленивое вычисление контекста (полезно для дорогих операций):

```rust
use anyhow::{Result, Context};

fn fetch_order_status(order_id: &str) -> Result<String> {
    // Симуляция API запроса
    if order_id.starts_with("ERR") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Order not found"
        ).into());
    }

    Ok("FILLED".to_string())
}

fn check_portfolio_orders(order_ids: &[&str]) -> Result<Vec<String>> {
    let mut statuses = Vec::new();

    for order_id in order_ids {
        let status = fetch_order_status(order_id)
            .with_context(|| format!("Failed to fetch status for order {}", order_id))?;
        statuses.push(status);
    }

    Ok(statuses)
}

fn main() {
    let orders = ["ORD-001", "ORD-002", "ERR-003"];

    match check_portfolio_orders(&orders) {
        Ok(statuses) => {
            for (id, status) in orders.iter().zip(statuses.iter()) {
                println!("{}: {}", id, status);
            }
        }
        Err(e) => {
            eprintln!("Portfolio check failed: {:#}", e);
        }
    }
}
```

## Форматирование ошибок

```rust
use anyhow::{Result, anyhow, Context};

fn risky_operation() -> Result<()> {
    Err(anyhow!("Connection timeout"))
        .context("Failed to connect to exchange API")
        .context("Cannot fetch market data")
}

fn main() {
    if let Err(e) = risky_operation() {
        // Только верхнее сообщение
        println!("Display: {}", e);

        // Полная цепочка (однострочно)
        println!("Debug: {:?}", e);

        // Полная цепочка (многострочно, красиво)
        println!("Alternate: {:#}", e);

        // Итерация по цепочке
        println!("\nError chain:");
        for (i, cause) in e.chain().enumerate() {
            println!("  {}: {}", i, cause);
        }
    }
}
```

## Практический пример: Торговый бот

```rust
use anyhow::{Result, Context, bail, ensure};
use std::collections::HashMap;

// Структуры данных
#[derive(Debug, Clone)]
struct Order {
    id: String,
    symbol: String,
    side: OrderSide,
    quantity: f64,
    price: f64,
}

#[derive(Debug, Clone, Copy)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
struct Portfolio {
    balance: f64,
    positions: HashMap<String, f64>,
}

// Симуляция API биржи
struct ExchangeClient {
    connected: bool,
}

impl ExchangeClient {
    fn new() -> Self {
        Self { connected: true }
    }

    fn place_order(&self, order: &Order) -> Result<String> {
        ensure!(self.connected, "Exchange client not connected");

        // Симуляция размещения ордера
        if order.symbol == "DELIST" {
            bail!("Symbol {} is delisted", order.symbol);
        }

        Ok(format!("EX-{}-{}", order.symbol, 12345))
    }

    fn get_price(&self, symbol: &str) -> Result<f64> {
        ensure!(self.connected, "Exchange client not connected");

        match symbol {
            "BTC" => Ok(42000.0),
            "ETH" => Ok(2500.0),
            "SOL" => Ok(100.0),
            _ => bail!("Unknown symbol: {}", symbol),
        }
    }
}

// Торговый бот
struct TradingBot {
    client: ExchangeClient,
    portfolio: Portfolio,
    max_position_size: f64,
}

impl TradingBot {
    fn new(initial_balance: f64) -> Self {
        Self {
            client: ExchangeClient::new(),
            portfolio: Portfolio {
                balance: initial_balance,
                positions: HashMap::new(),
            },
            max_position_size: 0.1, // 10% от баланса
        }
    }

    fn validate_trade(&self, symbol: &str, quantity: f64) -> Result<()> {
        ensure!(!symbol.is_empty(), "Symbol cannot be empty");
        ensure!(quantity > 0.0, "Quantity must be positive");

        let price = self.client.get_price(symbol)
            .context("Failed to validate trade")?;

        let trade_value = price * quantity;
        let max_value = self.portfolio.balance * self.max_position_size;

        ensure!(
            trade_value <= max_value,
            "Trade value ${:.2} exceeds maximum ${:.2}",
            trade_value,
            max_value
        );

        ensure!(
            trade_value <= self.portfolio.balance,
            "Insufficient balance: need ${:.2}, have ${:.2}",
            trade_value,
            self.portfolio.balance
        );

        Ok(())
    }

    fn buy(&mut self, symbol: &str, quantity: f64) -> Result<String> {
        self.validate_trade(symbol, quantity)
            .with_context(|| format!("Cannot buy {} {}", quantity, symbol))?;

        let price = self.client.get_price(symbol)
            .context("Failed to get price for buy order")?;

        let order = Order {
            id: String::new(),
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            quantity,
            price,
        };

        let order_id = self.client.place_order(&order)
            .with_context(|| format!("Failed to place buy order for {}", symbol))?;

        // Обновляем портфель
        let cost = price * quantity;
        self.portfolio.balance -= cost;
        *self.portfolio.positions.entry(symbol.to_string()).or_insert(0.0) += quantity;

        Ok(order_id)
    }

    fn sell(&mut self, symbol: &str, quantity: f64) -> Result<String> {
        let current_position = self.portfolio.positions.get(symbol).copied().unwrap_or(0.0);

        ensure!(
            current_position >= quantity,
            "Insufficient position: have {}, trying to sell {}",
            current_position,
            quantity
        );

        let price = self.client.get_price(symbol)
            .context("Failed to get price for sell order")?;

        let order = Order {
            id: String::new(),
            symbol: symbol.to_string(),
            side: OrderSide::Sell,
            quantity,
            price,
        };

        let order_id = self.client.place_order(&order)
            .with_context(|| format!("Failed to place sell order for {}", symbol))?;

        // Обновляем портфель
        let revenue = price * quantity;
        self.portfolio.balance += revenue;
        if let Some(pos) = self.portfolio.positions.get_mut(symbol) {
            *pos -= quantity;
        }

        Ok(order_id)
    }

    fn get_portfolio_value(&self) -> Result<f64> {
        let mut total = self.portfolio.balance;

        for (symbol, &quantity) in &self.portfolio.positions {
            if quantity > 0.0 {
                let price = self.client.get_price(symbol)
                    .with_context(|| format!("Failed to price position in {}", symbol))?;
                total += price * quantity;
            }
        }

        Ok(total)
    }
}

fn run_trading_session() -> Result<()> {
    let mut bot = TradingBot::new(100000.0);

    println!("Starting trading session...");
    println!("Initial balance: ${:.2}", bot.portfolio.balance);

    // Покупаем BTC
    let order1 = bot.buy("BTC", 0.1)
        .context("BTC purchase failed")?;
    println!("BTC order placed: {}", order1);

    // Покупаем ETH
    let order2 = bot.buy("ETH", 2.0)
        .context("ETH purchase failed")?;
    println!("ETH order placed: {}", order2);

    // Проверяем стоимость портфеля
    let value = bot.get_portfolio_value()
        .context("Failed to calculate portfolio value")?;
    println!("Portfolio value: ${:.2}", value);

    // Продаём часть BTC
    let order3 = bot.sell("BTC", 0.05)
        .context("BTC sale failed")?;
    println!("BTC sell order placed: {}", order3);

    // Финальное состояние
    let final_value = bot.get_portfolio_value()?;
    println!("Final portfolio value: ${:.2}", final_value);
    println!("Remaining balance: ${:.2}", bot.portfolio.balance);

    Ok(())
}

fn main() {
    if let Err(e) = run_trading_session() {
        eprintln!("Trading session failed!");
        eprintln!("Error: {:#}", e);

        // Детальный вывод для логов
        eprintln!("\nFull error chain:");
        for (i, cause) in e.chain().enumerate() {
            eprintln!("  [{}] {}", i, cause);
        }
    }
}
```

## Интеграция с другими типами ошибок

`anyhow::Error` автоматически конвертирует любые ошибки, реализующие `std::error::Error`:

```rust
use anyhow::{Result, Context};
use std::num::ParseIntError;
use std::io;

fn parse_trade_count(s: &str) -> Result<u32> {
    let count: u32 = s.parse()?;  // ParseIntError автоматически конвертируется
    Ok(count)
}

fn read_trade_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)?;  // io::Error конвертируется
    Ok(content)
}

fn process_trades(path: &str) -> Result<u32> {
    let content = read_trade_file(path)
        .context("Cannot read trade history")?;

    let count = parse_trade_count(&content.lines().count().to_string())
        .context("Cannot count trades")?;

    Ok(count)
}
```

## anyhow vs thiserror

| Аспект | anyhow | thiserror |
|--------|--------|-----------|
| Назначение | Приложения | Библиотеки |
| Типы ошибок | Единый `anyhow::Error` | Кастомные enum |
| Контекст | Встроенный `.context()` | Вручную |
| Цепочки ошибок | Автоматически | Вручную |
| Производительность | Чуть медленнее | Быстрее |
| Паттерн-матчинг | Ограниченный | Полный |

**Комбинация:** Используй `thiserror` для определения ошибок в библиотеках, а `anyhow` для обработки в приложении.

## Что мы узнали

| Инструмент | Описание |
|------------|----------|
| `anyhow::Result<T>` | Универсальный Result с anyhow::Error |
| `anyhow!("msg")` | Создание ошибки из строки |
| `bail!("msg")` | return Err(anyhow!("msg")) |
| `ensure!(cond, "msg")` | Проверка с ошибкой |
| `.context("msg")` | Добавление контекста |
| `.with_context(\|\| ...)` | Ленивый контекст |
| `e.chain()` | Итерация по цепочке ошибок |
| `{:#}` | Красивый вывод цепочки |

## Домашнее задание

1. Создай функцию `load_market_data(path: &str) -> Result<Vec<Candle>>`, которая читает CSV-файл со свечами, используя `context()` для добавления информации об ошибках

2. Реализуй валидатор торговой стратегии `validate_strategy(params: &StrategyParams) -> Result<()>` с использованием `ensure!` для проверки всех параметров

3. Напиши торговый симулятор, который может падать на разных этапах (загрузка данных, валидация, исполнение) и красиво выводит цепочку ошибок

4. Создай обработчик ордеров с несколькими уровнями вложенности, демонстрирующий работу `with_context()` для ленивого формирования сообщений об ошибках

## Навигация

[← День 103: thiserror](../103-thiserror/ru.md) | [День 105: Error Context →](../105-error-context/ru.md)
