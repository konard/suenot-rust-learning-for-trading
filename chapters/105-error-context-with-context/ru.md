# День 105: Контекст ошибок — with_context()

## Аналогия из трейдинга

Представь, что ты получаешь сообщение от брокера: "Ошибка выполнения ордера". Это неинформативно! А теперь представь другое сообщение: "Ошибка выполнения ордера BUY 0.5 BTC @ $42,000 на бирже Binance: недостаточно средств на счёте, баланс $15,000". Второе сообщение содержит **контекст** — всю информацию, нужную для понимания и решения проблемы.

Метод `with_context()` в Rust делает именно это — добавляет контекст к ошибкам, превращая непонятные сообщения в информативные.

## Зачем нужен контекст ошибок?

```rust
// Плохо: ошибка без контекста
Err("Failed to parse price")

// Хорошо: ошибка с контекстом
Err("Failed to parse price for BTCUSDT from API response: invalid format 'N/A'")
```

Без контекста:
- Непонятно, какой актив вызвал проблему
- Неизвестен источник данных
- Нет информации для отладки

С контекстом:
- Точно знаешь, где произошла ошибка
- Видишь входные данные
- Можешь быстро исправить проблему

## Библиотека anyhow

`with_context()` — это метод из популярной библиотеки `anyhow`, которая упрощает обработку ошибок:

```toml
# Cargo.toml
[dependencies]
anyhow = "1.0"
```

## Базовое использование with_context()

```rust
use anyhow::{Context, Result};
use std::fs;

fn main() -> Result<()> {
    let config = load_trading_config("config.json")?;
    println!("Loaded config: {:?}", config);
    Ok(())
}

fn load_trading_config(path: &str) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to load trading config from '{}'", path))
}
```

Без контекста ошибка выглядит так:
```
Error: No such file or directory (os error 2)
```

С контекстом:
```
Error: Failed to load trading config from 'config.json'

Caused by:
    No such file or directory (os error 2)
```

## Примеры из трейдинга

### 1. Парсинг рыночных данных

```rust
use anyhow::{Context, Result};

#[derive(Debug)]
struct MarketData {
    symbol: String,
    price: f64,
    volume: f64,
}

fn parse_market_data(symbol: &str, data: &str) -> Result<MarketData> {
    let parts: Vec<&str> = data.split(',').collect();

    if parts.len() < 2 {
        anyhow::bail!("Invalid data format for {}: expected 'price,volume'", symbol);
    }

    let price: f64 = parts[0]
        .trim()
        .parse()
        .with_context(|| format!(
            "Failed to parse price for {}: invalid value '{}'",
            symbol, parts[0]
        ))?;

    let volume: f64 = parts[1]
        .trim()
        .parse()
        .with_context(|| format!(
            "Failed to parse volume for {}: invalid value '{}'",
            symbol, parts[1]
        ))?;

    Ok(MarketData {
        symbol: symbol.to_string(),
        price,
        volume,
    })
}

fn main() -> Result<()> {
    // Успешный парсинг
    let btc = parse_market_data("BTCUSDT", "42000.50,1234.5")?;
    println!("BTC: {:?}", btc);

    // Ошибка с контекстом
    let invalid = parse_market_data("ETHUSDT", "N/A,500.0");
    if let Err(e) = invalid {
        println!("Error: {:?}", e);
    }

    Ok(())
}
```

### 2. Валидация торгового ордера

```rust
use anyhow::{Context, Result, ensure};

#[derive(Debug)]
struct Order {
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn validate_order(order: &Order, balance: f64, min_order_size: f64) -> Result<()> {
    // Проверка символа
    ensure!(
        !order.symbol.is_empty(),
        "Order validation failed: symbol cannot be empty"
    );

    // Проверка цены
    ensure!(
        order.price > 0.0,
        "Order validation failed for {}: price must be positive, got {}",
        order.symbol, order.price
    );

    // Проверка количества
    ensure!(
        order.quantity >= min_order_size,
        "Order validation failed for {}: quantity {} is below minimum {}",
        order.symbol, order.quantity, min_order_size
    );

    // Проверка баланса
    let required = order.price * order.quantity;
    ensure!(
        required <= balance,
        "Order validation failed for {}: required ${:.2} exceeds balance ${:.2}",
        order.symbol, required, balance
    );

    Ok(())
}

fn submit_order(order: Order, balance: f64) -> Result<String> {
    validate_order(&order, balance, 0.001)
        .with_context(|| format!(
            "Cannot submit {} order for {} @ ${:.2}",
            order.side, order.symbol, order.price
        ))?;

    Ok(format!("Order submitted: {:?}", order))
}

fn main() -> Result<()> {
    let order = Order {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        price: 42000.0,
        quantity: 0.5,
    };

    // Попытка с недостаточным балансом
    match submit_order(order, 10000.0) {
        Ok(msg) => println!("{}", msg),
        Err(e) => println!("Error:\n{:?}", e),
    }

    Ok(())
}
```

### 3. Загрузка портфеля с цепочкой контекста

```rust
use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(Debug)]
struct Portfolio {
    positions: HashMap<String, f64>,
}

fn load_portfolio_file(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read portfolio file: {}", path))
}

fn parse_portfolio(content: &str) -> Result<Portfolio> {
    let mut positions = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let parts: Vec<&str> = line.split(':').collect();

        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid format at line {}: expected 'SYMBOL:QUANTITY'",
                line_num + 1
            );
        }

        let symbol = parts[0].trim().to_string();
        let quantity: f64 = parts[1]
            .trim()
            .parse()
            .with_context(|| format!(
                "Failed to parse quantity at line {} for symbol {}",
                line_num + 1, symbol
            ))?;

        positions.insert(symbol, quantity);
    }

    Ok(Portfolio { positions })
}

fn load_portfolio(path: &str) -> Result<Portfolio> {
    let content = load_portfolio_file(path)
        .with_context(|| "Failed to load portfolio")?;

    parse_portfolio(&content)
        .with_context(|| format!("Failed to parse portfolio from {}", path))
}

fn main() -> Result<()> {
    match load_portfolio("portfolio.txt") {
        Ok(portfolio) => println!("Portfolio: {:?}", portfolio),
        Err(e) => {
            // Цепочка ошибок с полным контекстом
            println!("Error chain:");
            for (i, cause) in e.chain().enumerate() {
                println!("  {}: {}", i, cause);
            }
        }
    }

    Ok(())
}
```

### 4. Расчёт риска с контекстом

```rust
use anyhow::{Context, Result, ensure};

#[derive(Debug)]
struct RiskMetrics {
    var_95: f64,
    max_drawdown: f64,
    sharpe_ratio: f64,
}

fn calculate_returns(prices: &[f64]) -> Result<Vec<f64>> {
    ensure!(
        prices.len() >= 2,
        "Need at least 2 prices to calculate returns, got {}",
        prices.len()
    );

    let returns: Vec<f64> = prices
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect();

    Ok(returns)
}

fn calculate_var(returns: &[f64], confidence: f64) -> Result<f64> {
    ensure!(
        !returns.is_empty(),
        "Cannot calculate VaR on empty returns"
    );
    ensure!(
        confidence > 0.0 && confidence < 1.0,
        "Confidence level must be between 0 and 1, got {}",
        confidence
    );

    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let index = ((1.0 - confidence) * sorted.len() as f64) as usize;
    Ok(sorted[index])
}

fn calculate_risk_metrics(symbol: &str, prices: &[f64]) -> Result<RiskMetrics> {
    let returns = calculate_returns(prices)
        .with_context(|| format!("Failed to calculate returns for {}", symbol))?;

    let var_95 = calculate_var(&returns, 0.95)
        .with_context(|| format!("Failed to calculate VaR for {}", symbol))?;

    // Упрощённый расчёт для примера
    let max_drawdown = returns.iter().cloned().fold(0.0_f64, f64::min).abs();

    let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance: f64 = returns.iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>() / returns.len() as f64;
    let std_dev = variance.sqrt();

    let sharpe_ratio = if std_dev > 0.0 { mean / std_dev } else { 0.0 };

    Ok(RiskMetrics {
        var_95,
        max_drawdown,
        sharpe_ratio,
    })
}

fn main() -> Result<()> {
    let btc_prices = vec![42000.0, 42500.0, 41800.0, 43000.0, 42200.0, 43500.0];

    match calculate_risk_metrics("BTCUSDT", &btc_prices) {
        Ok(metrics) => {
            println!("Risk Metrics for BTCUSDT:");
            println!("  VaR 95%: {:.4}", metrics.var_95);
            println!("  Max Drawdown: {:.4}", metrics.max_drawdown);
            println!("  Sharpe Ratio: {:.4}", metrics.sharpe_ratio);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Тест с недостаточными данными
    let insufficient_data = vec![42000.0];
    match calculate_risk_metrics("ETHUSDT", &insufficient_data) {
        Ok(_) => println!("Success"),
        Err(e) => println!("Expected error:\n{:?}", e),
    }

    Ok(())
}
```

## context() vs with_context()

```rust
use anyhow::{Context, Result};

fn example() -> Result<()> {
    // context() — статическое сообщение (быстрее)
    let _data = std::fs::read_to_string("config.json")
        .context("Failed to read config")?;

    // with_context() — динамическое сообщение (создаётся при ошибке)
    let path = "data.json";
    let _data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path))?;

    Ok(())
}
```

**Когда использовать:**
- `context()` — когда сообщение не зависит от переменных
- `with_context()` — когда нужно включить динамические данные

## Практическое упражнение 1: API-клиент биржи

```rust
use anyhow::{Context, Result, bail};
use std::collections::HashMap;

struct ExchangeClient {
    name: String,
    connected: bool,
}

impl ExchangeClient {
    fn new(name: &str) -> Self {
        ExchangeClient {
            name: name.to_string(),
            connected: false,
        }
    }

    fn connect(&mut self) -> Result<()> {
        // Симуляция подключения
        self.connected = true;
        Ok(())
    }

    fn get_price(&self, symbol: &str) -> Result<f64> {
        if !self.connected {
            bail!("Not connected to exchange");
        }

        // Симуляция получения цены
        let prices: HashMap<&str, f64> = [
            ("BTCUSDT", 42000.0),
            ("ETHUSDT", 2200.0),
        ].into_iter().collect();

        prices.get(symbol)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Symbol not found"))
            .with_context(|| format!(
                "Failed to get price for {} from {}",
                symbol, self.name
            ))
    }

    fn place_order(&self, symbol: &str, side: &str, quantity: f64) -> Result<String> {
        if !self.connected {
            bail!("Not connected to exchange");
        }

        let price = self.get_price(symbol)
            .with_context(|| format!(
                "Cannot place {} order: failed to get current price",
                side
            ))?;

        Ok(format!(
            "Order placed on {}: {} {} {} @ ${:.2}",
            self.name, side, quantity, symbol, price
        ))
    }
}

fn main() -> Result<()> {
    let mut client = ExchangeClient::new("Binance");
    client.connect()?;

    // Успешный ордер
    let result = client.place_order("BTCUSDT", "BUY", 0.1)?;
    println!("{}", result);

    // Ордер с неизвестным символом
    match client.place_order("UNKNOWN", "BUY", 0.1) {
        Ok(r) => println!("{}", r),
        Err(e) => println!("Error:\n{:?}", e),
    }

    Ok(())
}
```

## Практическое упражнение 2: Парсер торговой истории

```rust
use anyhow::{Context, Result};

#[derive(Debug)]
struct Trade {
    timestamp: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn parse_trade_line(line: &str, line_num: usize) -> Result<Trade> {
    let parts: Vec<&str> = line.split(',').collect();

    if parts.len() != 5 {
        anyhow::bail!(
            "Invalid format at line {}: expected 5 fields, got {}",
            line_num, parts.len()
        );
    }

    let timestamp: u64 = parts[0]
        .trim()
        .parse()
        .with_context(|| format!("Invalid timestamp at line {}: '{}'", line_num, parts[0]))?;

    let symbol = parts[1].trim().to_string();
    let side = parts[2].trim().to_string();

    let price: f64 = parts[3]
        .trim()
        .parse()
        .with_context(|| format!(
            "Invalid price at line {} for {}: '{}'",
            line_num, symbol, parts[3]
        ))?;

    let quantity: f64 = parts[4]
        .trim()
        .parse()
        .with_context(|| format!(
            "Invalid quantity at line {} for {}: '{}'",
            line_num, symbol, parts[4]
        ))?;

    Ok(Trade { timestamp, symbol, side, price, quantity })
}

fn parse_trade_history(content: &str) -> Result<Vec<Trade>> {
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(i, line)| {
            parse_trade_line(line, i + 1)
                .with_context(|| "Failed to parse trade history")
        })
        .collect()
}

fn main() -> Result<()> {
    let valid_data = r#"
1703980800,BTCUSDT,BUY,42000.0,0.5
1703984400,BTCUSDT,SELL,42500.0,0.5
1703988000,ETHUSDT,BUY,2200.0,2.0
"#;

    let trades = parse_trade_history(valid_data)?;
    for trade in &trades {
        println!("{:?}", trade);
    }

    // Тест с ошибкой
    let invalid_data = "1703980800,BTCUSDT,BUY,invalid,0.5";
    match parse_trade_history(invalid_data) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("\nExpected error:\n{:?}", e),
    }

    Ok(())
}
```

## Практическое упражнение 3: Менеджер стратегий

```rust
use anyhow::{Context, Result, ensure};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Strategy {
    name: String,
    enabled: bool,
    parameters: HashMap<String, f64>,
}

struct StrategyManager {
    strategies: HashMap<String, Strategy>,
}

impl StrategyManager {
    fn new() -> Self {
        StrategyManager {
            strategies: HashMap::new(),
        }
    }

    fn add_strategy(&mut self, strategy: Strategy) -> Result<()> {
        ensure!(
            !strategy.name.is_empty(),
            "Strategy name cannot be empty"
        );

        ensure!(
            !self.strategies.contains_key(&strategy.name),
            "Strategy '{}' already exists", strategy.name
        );

        self.strategies.insert(strategy.name.clone(), strategy);
        Ok(())
    }

    fn get_parameter(&self, strategy_name: &str, param: &str) -> Result<f64> {
        let strategy = self.strategies
            .get(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("Strategy not found"))
            .with_context(|| format!("Cannot get parameter '{}' from '{}'", param, strategy_name))?;

        strategy.parameters
            .get(param)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Parameter not found"))
            .with_context(|| format!(
                "Parameter '{}' not configured for strategy '{}'",
                param, strategy_name
            ))
    }

    fn execute_strategy(&self, strategy_name: &str, price: f64) -> Result<String> {
        let strategy = self.strategies
            .get(strategy_name)
            .ok_or_else(|| anyhow::anyhow!("Strategy not found"))
            .with_context(|| format!("Cannot execute strategy '{}'", strategy_name))?;

        ensure!(
            strategy.enabled,
            "Strategy '{}' is disabled", strategy_name
        );

        let threshold = self.get_parameter(strategy_name, "threshold")
            .with_context(|| format!("Cannot execute '{}': missing required parameter", strategy_name))?;

        let signal = if price > threshold { "BUY" } else { "SELL" };

        Ok(format!(
            "Strategy '{}' signal: {} (price: {:.2}, threshold: {:.2})",
            strategy_name, signal, price, threshold
        ))
    }
}

fn main() -> Result<()> {
    let mut manager = StrategyManager::new();

    let mut params = HashMap::new();
    params.insert("threshold".to_string(), 42000.0);
    params.insert("stop_loss".to_string(), 0.02);

    manager.add_strategy(Strategy {
        name: "BTC_Momentum".to_string(),
        enabled: true,
        parameters: params,
    })?;

    // Успешное выполнение
    let signal = manager.execute_strategy("BTC_Momentum", 42500.0)?;
    println!("{}", signal);

    // Ошибка: несуществующая стратегия
    match manager.execute_strategy("Unknown", 42500.0) {
        Ok(s) => println!("{}", s),
        Err(e) => println!("Error:\n{:?}", e),
    }

    Ok(())
}
```

## Лучшие практики

### 1. Добавляй контекст на границах модулей

```rust
// В модуле API
fn fetch_price(symbol: &str) -> Result<f64> {
    internal_fetch(symbol)
        .with_context(|| format!("API: Failed to fetch price for {}", symbol))
}

// В модуле стратегий
fn evaluate_signal(symbol: &str) -> Result<Signal> {
    let price = fetch_price(symbol)
        .with_context(|| format!("Strategy: Cannot evaluate signal for {}", symbol))?;
    // ...
}
```

### 2. Включай релевантные данные

```rust
// Плохо
.with_context(|| "Failed to process order")

// Хорошо
.with_context(|| format!(
    "Failed to process {} order for {} @ ${:.2}",
    order.side, order.symbol, order.price
))
```

### 3. Используй цепочку для отслеживания пути ошибки

```rust
fn main() -> Result<()> {
    if let Err(e) = run_trading_bot() {
        eprintln!("Trading bot error:");
        for (i, cause) in e.chain().enumerate() {
            eprintln!("  {}: {}", i, cause);
        }
    }
    Ok(())
}
```

## Что мы узнали

| Метод | Использование | Пример |
|-------|---------------|--------|
| `context()` | Статическое сообщение | `.context("Failed to load")` |
| `with_context()` | Динамическое сообщение | `.with_context(\|\| format!("Failed for {}", x))` |
| `ensure!()` | Проверка условия | `ensure!(x > 0, "Must be positive")` |
| `bail!()` | Немедленный возврат ошибки | `bail!("Invalid state")` |
| `.chain()` | Итерация по цепочке ошибок | `for cause in e.chain()` |

## Домашнее задание

1. Создай функцию `load_exchange_config(path: &str) -> Result<ExchangeConfig>`, которая загружает конфигурацию биржи из файла с подробным контекстом ошибок для каждого этапа (чтение файла, парсинг JSON, валидация полей)

2. Реализуй `execute_trading_strategy(name: &str, market_data: &MarketData) -> Result<Vec<Order>>`, где каждый шаг стратегии добавляет свой контекст к возможным ошибкам

3. Напиши функцию `reconcile_positions(local: &Portfolio, remote: &Portfolio) -> Result<ReconciliationReport>`, которая сравнивает позиции и возвращает детальные ошибки несоответствия с контекстом

4. Создай обёртку для API биржи с методами `get_balance()`, `place_order()`, `cancel_order()`, где каждый метод добавляет информативный контекст к ошибкам сети и API

## Навигация

[← Предыдущий день](../104-error-handling-best-practices/ru.md) | [Следующий день →](../106-custom-error-types/ru.md)
