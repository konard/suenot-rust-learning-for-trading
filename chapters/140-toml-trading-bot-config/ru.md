# День 140: TOML — Конфигурация торгового бота

## Аналогия из трейдинга

Когда ты настраиваешь торгового бота, нужно указать множество параметров: API-ключи, адреса бирж, торговые пары, размеры позиций, лимиты риска и параметры стратегий. Хардкодить эти значения прямо в коде опасно — придётся перекомпилировать бота каждый раз, когда захочешь изменить стоп-лосс или переключиться на другую биржу.

**TOML** (Tom's Obvious, Minimal Language) — это как **торговый журнал-шаблон** — чистый, читаемый человеком формат для хранения конфигурации, понятный и тебе, и твоему боту. Как ты мог бы иметь чек-лист перед торговлей: "Проверить маржу, установить стоп-лосс 2%, максимальный размер позиции 0.5 BTC" — TOML позволяет записать эти правила в файл, который бот читает при запуске.

## Что такое TOML?

TOML — это формат конфигурационных файлов, который:
- **Читаемый человеком** — легко редактировать вручную
- **Строго типизированный** — числа остаются числами, строки строками
- **Иерархический** — поддерживает вложенные секции (таблицы)
- **Родной для Rust** — Cargo использует TOML для `Cargo.toml`!

## Настройка зависимостей

Добавь `toml` и `serde` в свой `Cargo.toml`:

```toml
[dependencies]
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

## Базовый синтаксис TOML

```toml
# bot_config.toml - Конфигурация торгового бота

# Простые пары ключ-значение
bot_name = "AlphaTrader"
version = "1.0.0"
debug_mode = false

# Числа
max_position_usd = 10000.0
max_open_trades = 5
risk_percent = 2.0

# Даты (TOML поддерживает ISO 8601)
start_date = 2024-01-01
```

## Парсинг простой конфигурации

```rust
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct BotConfig {
    bot_name: String,
    version: String,
    debug_mode: bool,
    max_position_usd: f64,
    max_open_trades: u32,
    risk_percent: f64,
}

fn main() {
    let config_str = r#"
        bot_name = "AlphaTrader"
        version = "1.0.0"
        debug_mode = false
        max_position_usd = 10000.0
        max_open_trades = 5
        risk_percent = 2.0
    "#;

    let config: BotConfig = toml::from_str(config_str).unwrap();

    println!("╔══════════════════════════════════════╗");
    println!("║       КОНФИГУРАЦИЯ БОТА              ║");
    println!("╠══════════════════════════════════════╣");
    println!("║ Имя:            {:>18} ║", config.bot_name);
    println!("║ Версия:         {:>18} ║", config.version);
    println!("║ Отладка:        {:>18} ║", config.debug_mode);
    println!("║ Макс. позиция:  ${:>16.2} ║", config.max_position_usd);
    println!("║ Макс. сделок:   {:>18} ║", config.max_open_trades);
    println!("║ Риск %:         {:>17.1}% ║", config.risk_percent);
    println!("╚══════════════════════════════════════╝");
}
```

## Таблицы TOML — Вложенная конфигурация

Таблицы в TOML создают вложенные структуры, идеально подходящие для организации настроек биржи:

```toml
# config.toml

[bot]
name = "AlphaTrader"
version = "1.0.0"

[exchange]
name = "binance"
base_url = "https://api.binance.com"
ws_url = "wss://stream.binance.com:9443"
rate_limit_per_minute = 1200

[trading]
symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
max_position_usd = 10000.0
max_open_trades = 5

[risk]
max_daily_loss_percent = 5.0
max_drawdown_percent = 10.0
stop_loss_percent = 2.0
take_profit_percent = 4.0
```

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    bot: BotSection,
    exchange: ExchangeSection,
    trading: TradingSection,
    risk: RiskSection,
}

#[derive(Debug, Deserialize)]
struct BotSection {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct ExchangeSection {
    name: String,
    base_url: String,
    ws_url: String,
    rate_limit_per_minute: u32,
}

#[derive(Debug, Deserialize)]
struct TradingSection {
    symbols: Vec<String>,
    max_position_usd: f64,
    max_open_trades: u32,
}

#[derive(Debug, Deserialize)]
struct RiskSection {
    max_daily_loss_percent: f64,
    max_drawdown_percent: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
}

fn main() {
    let config_str = r#"
        [bot]
        name = "AlphaTrader"
        version = "1.0.0"

        [exchange]
        name = "binance"
        base_url = "https://api.binance.com"
        ws_url = "wss://stream.binance.com:9443"
        rate_limit_per_minute = 1200

        [trading]
        symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
        max_position_usd = 10000.0
        max_open_trades = 5

        [risk]
        max_daily_loss_percent = 5.0
        max_drawdown_percent = 10.0
        stop_loss_percent = 2.0
        take_profit_percent = 4.0
    "#;

    let config: Config = toml::from_str(config_str).unwrap();

    println!("Бот: {} v{}", config.bot.name, config.bot.version);
    println!("Биржа: {}", config.exchange.name);
    println!("Торговые пары: {:?}", config.trading.symbols);
    println!("Стоп-лосс: {}%", config.risk.stop_loss_percent);
}
```

## Конфигурация стратегии с инлайн-таблицами

Инлайн-таблицы отлично подходят для компактных, связанных данных:

```toml
# strategies.toml

[strategy]
name = "momentum"
timeframe = "1h"
enabled = true

# Инлайн-таблицы для параметров индикаторов
[strategy.indicators]
sma_fast = { period = 10, source = "close" }
sma_slow = { period = 50, source = "close" }
rsi = { period = 14, overbought = 70, oversold = 30 }
```

```rust
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct StrategyConfig {
    strategy: Strategy,
}

#[derive(Debug, Deserialize)]
struct Strategy {
    name: String,
    timeframe: String,
    enabled: bool,
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    sma_fast: SmaConfig,
    sma_slow: SmaConfig,
    rsi: RsiConfig,
}

#[derive(Debug, Deserialize)]
struct SmaConfig {
    period: u32,
    source: String,
}

#[derive(Debug, Deserialize)]
struct RsiConfig {
    period: u32,
    overbought: u32,
    oversold: u32,
}

fn main() {
    let config_str = r#"
        [strategy]
        name = "momentum"
        timeframe = "1h"
        enabled = true

        [strategy.indicators]
        sma_fast = { period = 10, source = "close" }
        sma_slow = { period = 50, source = "close" }
        rsi = { period = 14, overbought = 70, oversold = 30 }
    "#;

    let config: StrategyConfig = toml::from_str(config_str).unwrap();
    let strategy = &config.strategy;

    println!("Стратегия: {} ({})", strategy.name, strategy.timeframe);
    println!("SMA быстрая: {} периодов", strategy.indicators.sma_fast.period);
    println!("SMA медленная: {} периодов", strategy.indicators.sma_slow.period);
    println!("RSI: {} (перекупленность: {}, перепроданность: {})",
        strategy.indicators.rsi.period,
        strategy.indicators.rsi.overbought,
        strategy.indicators.rsi.oversold
    );
}
```

## Массив таблиц — Несколько торговых пар

Используй `[[имя_массива]]` для массивов таблиц — идеально для нескольких торговых пар:

```toml
# pairs.toml

[[pairs]]
symbol = "BTCUSDT"
enabled = true
max_position = 0.5
stop_loss = 2.0
take_profit = 4.0

[[pairs]]
symbol = "ETHUSDT"
enabled = true
max_position = 5.0
stop_loss = 3.0
take_profit = 6.0

[[pairs]]
symbol = "SOLUSDT"
enabled = false
max_position = 100.0
stop_loss = 5.0
take_profit = 10.0
```

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PairsConfig {
    pairs: Vec<TradingPair>,
}

#[derive(Debug, Deserialize)]
struct TradingPair {
    symbol: String,
    enabled: bool,
    max_position: f64,
    stop_loss: f64,
    take_profit: f64,
}

fn main() {
    let config_str = r#"
        [[pairs]]
        symbol = "BTCUSDT"
        enabled = true
        max_position = 0.5
        stop_loss = 2.0
        take_profit = 4.0

        [[pairs]]
        symbol = "ETHUSDT"
        enabled = true
        max_position = 5.0
        stop_loss = 3.0
        take_profit = 6.0

        [[pairs]]
        symbol = "SOLUSDT"
        enabled = false
        max_position = 100.0
        stop_loss = 5.0
        take_profit = 10.0
    "#;

    let config: PairsConfig = toml::from_str(config_str).unwrap();

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║            КОНФИГУРАЦИЯ ТОРГОВЫХ ПАР                   ║");
    println!("╠══════════╦═════════╦════════════╦══════════╦═══════════╣");
    println!("║  Символ  ║ Активен ║  Макс поз  ║ Стоп-лосс║Тейк-профит║");
    println!("╠══════════╬═════════╬════════════╬══════════╬═══════════╣");

    for pair in &config.pairs {
        let enabled = if pair.enabled { "Да" } else { "Нет" };
        println!("║ {:>8} ║ {:>7} ║ {:>10.2} ║ {:>7.1}% ║ {:>8.1}% ║",
            pair.symbol, enabled, pair.max_position,
            pair.stop_loss, pair.take_profit);
    }
    println!("╚══════════╩═════════╩════════════╩══════════╩═══════════╝");

    // Фильтруем только активные пары
    let active_pairs: Vec<_> = config.pairs
        .iter()
        .filter(|p| p.enabled)
        .collect();

    println!("\nАктивных пар: {}", active_pairs.len());
}
```

## Опциональные поля со значениями по умолчанию

Используй `Option<T>` и `#[serde(default)]` для опциональной конфигурации:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BotConfig {
    name: String,

    // Опциональное поле - будет None, если отсутствует
    description: Option<String>,

    // Значение по умолчанию, если отсутствует
    #[serde(default = "default_max_retries")]
    max_retries: u32,

    // По умолчанию false, если отсутствует
    #[serde(default)]
    paper_trading: bool,

    // По умолчанию пустой вектор
    #[serde(default)]
    notification_emails: Vec<String>,
}

fn default_max_retries() -> u32 {
    3
}

fn main() {
    // Минимальная конфигурация - без опциональных полей
    let minimal = r#"
        name = "SimpleBot"
    "#;

    let config: BotConfig = toml::from_str(minimal).unwrap();

    println!("Имя: {}", config.name);
    println!("Описание: {:?}", config.description);
    println!("Макс. попыток: {}", config.max_retries);
    println!("Бумажная торговля: {}", config.paper_trading);
    println!("Email-адреса: {:?}", config.notification_emails);

    // Полная конфигурация
    let full = r#"
        name = "AdvancedBot"
        description = "Мультистратегический торговый бот"
        max_retries = 5
        paper_trading = true
        notification_emails = ["trader@example.com", "alerts@example.com"]
    "#;

    let config: BotConfig = toml::from_str(full).unwrap();
    println!("\n--- Полная конфигурация ---");
    println!("{:#?}", config);
}
```

## Запись конфигурации в TOML

Сохраняй состояние бота или генерируй конфигурационные файлы:

```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Portfolio {
    last_updated: String,
    total_value_usd: f64,
    positions: Vec<Position>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

fn main() {
    let portfolio = Portfolio {
        last_updated: "2024-01-15T10:30:00Z".to_string(),
        total_value_usd: 52450.0,
        positions: vec![
            Position {
                symbol: "BTC".to_string(),
                quantity: 0.5,
                entry_price: 42000.0,
                current_price: 43500.0,
            },
            Position {
                symbol: "ETH".to_string(),
                quantity: 10.0,
                entry_price: 2200.0,
                current_price: 2350.0,
            },
        ],
    };

    // Сериализация в TOML-строку
    let toml_string = toml::to_string_pretty(&portfolio).unwrap();
    println!("Сгенерированный TOML:\n{}", toml_string);

    // Сохранение в файл
    // fs::write("portfolio.toml", &toml_string).unwrap();
}
```

Результат:
```toml
last_updated = "2024-01-15T10:30:00Z"
total_value_usd = 52450.0

[[positions]]
symbol = "BTC"
quantity = 0.5
entry_price = 42000.0
current_price = 43500.0

[[positions]]
symbol = "ETH"
quantity = 10.0
entry_price = 2200.0
current_price = 2350.0
```

## Обработка ошибок конфигурации

Всегда валидируй конфигурацию:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RiskConfig {
    max_position_usd: f64,
    stop_loss_percent: f64,
    max_daily_loss_percent: f64,
}

impl RiskConfig {
    fn validate(&self) -> Result<(), String> {
        if self.max_position_usd <= 0.0 {
            return Err("max_position_usd должен быть положительным".to_string());
        }
        if self.stop_loss_percent <= 0.0 || self.stop_loss_percent > 100.0 {
            return Err("stop_loss_percent должен быть между 0 и 100".to_string());
        }
        if self.max_daily_loss_percent <= 0.0 || self.max_daily_loss_percent > 100.0 {
            return Err("max_daily_loss_percent должен быть между 0 и 100".to_string());
        }
        Ok(())
    }
}

fn load_config(toml_str: &str) -> Result<RiskConfig, String> {
    // Парсинг TOML
    let config: RiskConfig = toml::from_str(toml_str)
        .map_err(|e| format!("Ошибка парсинга конфигурации: {}", e))?;

    // Валидация значений
    config.validate()?;

    Ok(config)
}

fn main() {
    // Валидная конфигурация
    let valid = r#"
        max_position_usd = 10000.0
        stop_loss_percent = 2.0
        max_daily_loss_percent = 5.0
    "#;

    match load_config(valid) {
        Ok(config) => println!("Конфигурация загружена: {:?}", config),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Невалидная конфигурация - отрицательный стоп-лосс
    let invalid = r#"
        max_position_usd = 10000.0
        stop_loss_percent = -2.0
        max_daily_loss_percent = 5.0
    "#;

    match load_config(invalid) {
        Ok(config) => println!("Конфигурация загружена: {:?}", config),
        Err(e) => println!("Ошибка валидации: {}", e),
    }

    // Ошибка парсинга - неправильный тип
    let parse_error = r#"
        max_position_usd = "не число"
        stop_loss_percent = 2.0
        max_daily_loss_percent = 5.0
    "#;

    match load_config(parse_error) {
        Ok(config) => println!("Конфигурация загружена: {:?}", config),
        Err(e) => println!("Ошибка парсинга: {}", e),
    }
}
```

## Практический пример: Полная конфигурация торгового бота

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct TradingBotConfig {
    bot: BotInfo,
    exchange: ExchangeConfig,
    trading: TradingConfig,
    risk: RiskManagement,
    strategies: Vec<StrategyConfig>,
    notifications: NotificationConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct BotInfo {
    name: String,
    version: String,
    #[serde(default)]
    paper_trading: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExchangeConfig {
    name: String,
    api_key_env: String,  // Имя переменной окружения, не сам ключ!
    secret_key_env: String,
    testnet: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct TradingConfig {
    symbols: Vec<String>,
    base_currency: String,
    timeframe: String,
    max_open_positions: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct RiskManagement {
    max_position_percent: f64,
    stop_loss_percent: f64,
    take_profit_percent: f64,
    max_daily_loss_percent: f64,
    max_drawdown_percent: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct StrategyConfig {
    name: String,
    enabled: bool,
    weight: f64,
    parameters: StrategyParameters,
}

#[derive(Debug, Deserialize, Serialize)]
struct StrategyParameters {
    fast_period: Option<u32>,
    slow_period: Option<u32>,
    signal_period: Option<u32>,
    rsi_period: Option<u32>,
    rsi_overbought: Option<f64>,
    rsi_oversold: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NotificationConfig {
    #[serde(default)]
    telegram_enabled: bool,
    telegram_bot_token_env: Option<String>,
    telegram_chat_id: Option<String>,
    #[serde(default)]
    email_enabled: bool,
    email_addresses: Vec<String>,
}

fn main() {
    let config_str = r#"
        [bot]
        name = "AlphaTrader Pro"
        version = "2.0.0"
        paper_trading = true

        [exchange]
        name = "binance"
        api_key_env = "BINANCE_API_KEY"
        secret_key_env = "BINANCE_SECRET_KEY"
        testnet = true

        [trading]
        symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"]
        base_currency = "USDT"
        timeframe = "4h"
        max_open_positions = 3

        [risk]
        max_position_percent = 10.0
        stop_loss_percent = 2.0
        take_profit_percent = 6.0
        max_daily_loss_percent = 5.0
        max_drawdown_percent = 15.0

        [[strategies]]
        name = "macd_crossover"
        enabled = true
        weight = 0.6
        [strategies.parameters]
        fast_period = 12
        slow_period = 26
        signal_period = 9

        [[strategies]]
        name = "rsi_reversal"
        enabled = true
        weight = 0.4
        [strategies.parameters]
        rsi_period = 14
        rsi_overbought = 70.0
        rsi_oversold = 30.0

        [notifications]
        telegram_enabled = true
        telegram_bot_token_env = "TELEGRAM_BOT_TOKEN"
        telegram_chat_id = "123456789"
        email_enabled = false
        email_addresses = []
    "#;

    let config: TradingBotConfig = toml::from_str(config_str).unwrap();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           КОНФИГУРАЦИЯ ТОРГОВОГО БОТА                    ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    println!("\n Информация о боте:");
    println!("   Имя: {} v{}", config.bot.name, config.bot.version);
    println!("   Режим: {}", if config.bot.paper_trading { "Бумажная торговля" } else { "Реальная торговля" });

    println!("\n Биржа:");
    println!("   Название: {}", config.exchange.name);
    println!("   Тестнет: {}", config.exchange.testnet);

    println!("\n Торговля:");
    println!("   Символы: {:?}", config.trading.symbols);
    println!("   Таймфрейм: {}", config.trading.timeframe);
    println!("   Макс. позиций: {}", config.trading.max_open_positions);

    println!("\n Риск-менеджмент:");
    println!("   Макс. позиция: {}%", config.risk.max_position_percent);
    println!("   Стоп-лосс: {}%", config.risk.stop_loss_percent);
    println!("   Тейк-профит: {}%", config.risk.take_profit_percent);
    println!("   Макс. дневной убыток: {}%", config.risk.max_daily_loss_percent);

    println!("\n Стратегии:");
    for strategy in &config.strategies {
        let status = if strategy.enabled { "v" } else { "x" };
        println!("   {} {} (вес: {:.0}%)", status, strategy.name, strategy.weight * 100.0);
    }

    println!("\n Уведомления:");
    println!("   Telegram: {}", if config.notifications.telegram_enabled { "Включён" } else { "Выключен" });
    println!("   Email: {}", if config.notifications.email_enabled { "Включён" } else { "Выключен" });
}
```

## Что мы узнали

| Фича TOML | Синтаксис | Применение |
|-----------|-----------|------------|
| Ключ-значение | `key = "value"` | Простые настройки |
| Таблица | `[section]` | Группировка настроек |
| Вложенная таблица | `[section.subsection]` | Иерархическая конфигурация |
| Инлайн-таблица | `{ key = "value" }` | Компактные структуры |
| Массив | `[1, 2, 3]` | Списки значений |
| Массив таблиц | `[[items]]` | Несколько однотипных объектов |
| Комментарии | `# комментарий` | Документация |

## Лучшие практики

1. **Никогда не храни секреты в TOML-файлах** — используй имена переменных окружения вместо этого
2. **Валидируй конфигурацию при запуске** — ловит ошибки раньше
3. **Используй значения по умолчанию для опциональных настроек** — `#[serde(default)]`
4. **Документируй формат конфигурации** — добавляй комментарии, объясняющие каждую опцию
5. **Версионируй формат конфигурации** — включай поле версии для миграций

## Домашнее задание

1. Создай TOML-конфигурацию для мультибиржевого торгового бота, поддерживающего Binance, Bybit и OKX с разными настройками для каждой биржи

2. Напиши валидатор конфигурации, который проверяет:
   - Стоп-лосс меньше тейк-профита
   - Сумма весов стратегий равна 1.0
   - Настроен хотя бы один торговый символ

3. Реализуй объединение конфигураций, которое берёт дефолтную конфигурацию и накладывает пользовательские переопределения

4. Создай TOML-конфигурацию для бэктестинга с:
   - Диапазоном дат для исторических данных
   - Списком стратегий для тестирования
   - Метриками производительности для расчёта

## Навигация

[← Предыдущий день](../139-time-formatting/ru.md) | [Следующий день →](../141-environment-variables-api-secrets/ru.md)
