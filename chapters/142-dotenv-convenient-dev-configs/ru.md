# День 142: dotenv: удобные конфиги разработки

## Аналогия из трейдинга

Представь, что у каждого трейдера в офисе есть личный блокнот с секретами: API ключи бирж, пароли, лимиты позиций. Когда трейдер садится за рабочее место, он достаёт этот блокнот и настраивает терминал. Файл `.env` — это такой "блокнот" для твоего торгового бота: все секреты и настройки в одном месте, но не в коде.

## Зачем нужен dotenv?

В предыдущей главе мы изучили переменные окружения. Но вводить их каждый раз в терминале неудобно:

```bash
BINANCE_API_KEY=abc123 BINANCE_SECRET=xyz789 RISK_PERCENT=2 cargo run
```

Файл `.env` позволяет записать все переменные один раз и автоматически загружать их при запуске.

## Подключение dotenv

Добавь зависимость в `Cargo.toml`:

```toml
[dependencies]
dotenvy = "0.15"
```

> **Примечание:** Мы используем `dotenvy` — активно поддерживаемый форк оригинальной библиотеки `dotenv`.

## Создание файла .env

Создай файл `.env` в корне проекта:

```env
# Настройки подключения к бирже
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET=your_secret_here

# Настройки риск-менеджмента
MAX_POSITION_SIZE=1000.0
RISK_PERCENT=2.0
MAX_DAILY_LOSS=500.0

# Настройки стратегии
SMA_FAST_PERIOD=10
SMA_SLOW_PERIOD=20
RSI_OVERSOLD=30
RSI_OVERBOUGHT=70

# Режим работы
TRADING_MODE=paper
LOG_LEVEL=debug
```

**Важно:** Добавь `.env` в `.gitignore`! Секреты не должны попадать в Git.

```gitignore
.env
.env.local
.env.*.local
```

## Базовое использование

```rust
use std::env;

fn main() {
    // Загружаем переменные из .env
    dotenvy::dotenv().ok();

    // Читаем настройки торговли
    let api_key = env::var("BINANCE_API_KEY")
        .expect("BINANCE_API_KEY must be set");

    let risk_percent: f64 = env::var("RISK_PERCENT")
        .unwrap_or_else(|_| "1.0".to_string())
        .parse()
        .expect("RISK_PERCENT must be a valid number");

    println!("API Key: {}...", &api_key[..8.min(api_key.len())]);
    println!("Risk: {}%", risk_percent);
}
```

## Структура конфигурации

Для удобства создадим структуру, хранящую все настройки:

```rust
use std::env;

#[derive(Debug)]
struct TradingConfig {
    // API credentials
    api_key: String,
    api_secret: String,

    // Risk management
    max_position_size: f64,
    risk_percent: f64,
    max_daily_loss: f64,

    // Strategy parameters
    sma_fast: usize,
    sma_slow: usize,
    rsi_oversold: f64,
    rsi_overbought: f64,

    // Mode
    is_paper_trading: bool,
}

impl TradingConfig {
    fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        Ok(TradingConfig {
            api_key: env::var("BINANCE_API_KEY")
                .map_err(|_| "BINANCE_API_KEY not set")?,

            api_secret: env::var("BINANCE_SECRET")
                .map_err(|_| "BINANCE_SECRET not set")?,

            max_position_size: env::var("MAX_POSITION_SIZE")
                .unwrap_or_else(|_| "1000.0".to_string())
                .parse()
                .map_err(|_| "Invalid MAX_POSITION_SIZE")?,

            risk_percent: env::var("RISK_PERCENT")
                .unwrap_or_else(|_| "2.0".to_string())
                .parse()
                .map_err(|_| "Invalid RISK_PERCENT")?,

            max_daily_loss: env::var("MAX_DAILY_LOSS")
                .unwrap_or_else(|_| "500.0".to_string())
                .parse()
                .map_err(|_| "Invalid MAX_DAILY_LOSS")?,

            sma_fast: env::var("SMA_FAST_PERIOD")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| "Invalid SMA_FAST_PERIOD")?,

            sma_slow: env::var("SMA_SLOW_PERIOD")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .map_err(|_| "Invalid SMA_SLOW_PERIOD")?,

            rsi_oversold: env::var("RSI_OVERSOLD")
                .unwrap_or_else(|_| "30.0".to_string())
                .parse()
                .map_err(|_| "Invalid RSI_OVERSOLD")?,

            rsi_overbought: env::var("RSI_OVERBOUGHT")
                .unwrap_or_else(|_| "70.0".to_string())
                .parse()
                .map_err(|_| "Invalid RSI_OVERBOUGHT")?,

            is_paper_trading: env::var("TRADING_MODE")
                .unwrap_or_else(|_| "paper".to_string())
                .to_lowercase() == "paper",
        })
    }
}

fn main() {
    match TradingConfig::from_env() {
        Ok(config) => {
            println!("Trading Configuration Loaded:");
            println!("  Paper trading: {}", config.is_paper_trading);
            println!("  Max position: ${:.2}", config.max_position_size);
            println!("  Risk: {}%", config.risk_percent);
            println!("  SMA periods: {}/{}", config.sma_fast, config.sma_slow);
            println!("  RSI levels: {}/{}", config.rsi_oversold, config.rsi_overbought);
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    }
}
```

## Разные файлы для разных окружений

Создай отдельные файлы для разных сред:

```
.env                # Значения по умолчанию (можно коммитить)
.env.development    # Для разработки
.env.production     # Для продакшена
.env.local          # Локальные переопределения (не коммитить!)
```

```rust
use std::env;

fn load_environment_config() {
    // Сначала загружаем базовый .env
    dotenvy::dotenv().ok();

    // Определяем текущее окружение
    let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    // Загружаем файл для конкретного окружения
    let env_file = format!(".env.{}", environment);
    dotenvy::from_filename(&env_file).ok();

    // Локальные переопределения имеют высший приоритет
    dotenvy::from_filename(".env.local").ok();

    println!("Loaded configuration for: {}", environment);
}

fn main() {
    load_environment_config();

    let trading_mode = env::var("TRADING_MODE").unwrap_or_else(|_| "paper".to_string());
    println!("Trading mode: {}", trading_mode);
}
```

## Валидация конфигурации при запуске

```rust
use std::env;

struct ExchangeConfig {
    api_key: String,
    api_secret: String,
    base_url: String,
}

struct RiskConfig {
    max_position_usd: f64,
    max_loss_percent: f64,
    max_orders_per_minute: u32,
}

fn validate_config() -> Result<(ExchangeConfig, RiskConfig), Vec<String>> {
    dotenvy::dotenv().ok();

    let mut errors = Vec::new();

    // Проверяем обязательные поля
    let api_key = env::var("API_KEY").ok();
    let api_secret = env::var("API_SECRET").ok();

    if api_key.is_none() {
        errors.push("API_KEY is required".to_string());
    }
    if api_secret.is_none() {
        errors.push("API_SECRET is required".to_string());
    }

    // Проверяем числовые значения
    let max_position: Result<f64, _> = env::var("MAX_POSITION_USD")
        .unwrap_or_else(|_| "10000.0".to_string())
        .parse();

    if let Err(_) = max_position {
        errors.push("MAX_POSITION_USD must be a valid number".to_string());
    }

    let max_loss: Result<f64, _> = env::var("MAX_LOSS_PERCENT")
        .unwrap_or_else(|_| "5.0".to_string())
        .parse();

    if let Err(_) = max_loss {
        errors.push("MAX_LOSS_PERCENT must be a valid number".to_string());
    }

    // Проверяем допустимые диапазоны
    if let Ok(loss) = max_loss {
        if loss < 0.0 || loss > 100.0 {
            errors.push("MAX_LOSS_PERCENT must be between 0 and 100".to_string());
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok((
        ExchangeConfig {
            api_key: api_key.unwrap(),
            api_secret: api_secret.unwrap(),
            base_url: env::var("API_BASE_URL")
                .unwrap_or_else(|_| "https://api.exchange.com".to_string()),
        },
        RiskConfig {
            max_position_usd: max_position.unwrap(),
            max_loss_percent: max_loss.unwrap(),
            max_orders_per_minute: env::var("MAX_ORDERS_PER_MINUTE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        },
    ))
}

fn main() {
    println!("Validating trading bot configuration...\n");

    match validate_config() {
        Ok((exchange, risk)) => {
            println!("Configuration valid!");
            println!("Exchange URL: {}", exchange.base_url);
            println!("Max position: ${:.2}", risk.max_position_usd);
            println!("Max loss: {:.1}%", risk.max_loss_percent);
            println!("Rate limit: {} orders/min", risk.max_orders_per_minute);
        }
        Err(errors) => {
            eprintln!("Configuration errors found:");
            for error in errors {
                eprintln!("  - {}", error);
            }
            std::process::exit(1);
        }
    }
}
```

## Пример: Файл конфигурации торгового бота

Пример полного `.env` файла для торгового бота:

```env
# ╔════════════════════════════════════════════════════════════════╗
# ║               TRADING BOT CONFIGURATION                        ║
# ╚════════════════════════════════════════════════════════════════╝

# ── Environment ──────────────────────────────────────────────────
APP_ENV=development
LOG_LEVEL=debug

# ── Exchange API ─────────────────────────────────────────────────
EXCHANGE=binance
API_KEY=your_api_key_here
API_SECRET=your_secret_key_here
API_PASSPHRASE=                     # For exchanges that require it

# ── Trading Mode ─────────────────────────────────────────────────
TRADING_MODE=paper                  # paper | live
DRY_RUN=true                        # true = log orders, don't execute

# ── Risk Management ──────────────────────────────────────────────
MAX_POSITION_SIZE_USD=1000.0        # Maximum single position
RISK_PER_TRADE_PERCENT=2.0          # % of capital per trade
MAX_DAILY_LOSS_USD=500.0            # Stop trading after this loss
MAX_OPEN_POSITIONS=5                # Maximum concurrent positions

# ── Strategy: SMA Crossover ──────────────────────────────────────
STRATEGY=sma_crossover
SMA_FAST_PERIOD=10
SMA_SLOW_PERIOD=20
MIN_VOLUME_USD=100000               # Minimum 24h volume to trade

# ── Strategy: RSI ────────────────────────────────────────────────
RSI_PERIOD=14
RSI_OVERSOLD=30
RSI_OVERBOUGHT=70

# ── Symbols ──────────────────────────────────────────────────────
TRADING_SYMBOLS=BTC/USDT,ETH/USDT,SOL/USDT
QUOTE_CURRENCY=USDT

# ── Execution ────────────────────────────────────────────────────
ORDER_TYPE=limit                    # limit | market
SLIPPAGE_TOLERANCE_PERCENT=0.5
MAX_RETRIES=3
RETRY_DELAY_MS=1000

# ── Notifications ────────────────────────────────────────────────
TELEGRAM_BOT_TOKEN=
TELEGRAM_CHAT_ID=
NOTIFY_ON_TRADE=true
NOTIFY_ON_ERROR=true

# ── Database ─────────────────────────────────────────────────────
DATABASE_URL=sqlite:./trading_bot.db
```

## Безопасность

### Никогда не коммить секреты!

```rust
use std::env;

fn mask_secret(secret: &str) -> String {
    if secret.len() <= 4 {
        "*".repeat(secret.len())
    } else {
        format!("{}...{}", &secret[..2], &secret[secret.len()-2..])
    }
}

fn print_safe_config() {
    dotenvy::dotenv().ok();

    let api_key = env::var("API_KEY").unwrap_or_else(|_| "NOT_SET".to_string());
    let api_secret = env::var("API_SECRET").unwrap_or_else(|_| "NOT_SET".to_string());

    println!("API Key: {}", mask_secret(&api_key));
    println!("API Secret: {}", mask_secret(&api_secret));
}

fn main() {
    print_safe_config();
}
```

### Создай шаблон .env.example

Создай файл `.env.example` с пустыми значениями для документации:

```env
# Copy this file to .env and fill in your values
API_KEY=
API_SECRET=
TRADING_MODE=paper
RISK_PERCENT=2.0
```

## Практический пример: Мультибиржевой конфиг

```rust
use std::env;
use std::collections::HashMap;

#[derive(Debug)]
struct ExchangeCredentials {
    name: String,
    api_key: String,
    api_secret: String,
    is_testnet: bool,
}

fn load_exchange_configs() -> Vec<ExchangeCredentials> {
    dotenvy::dotenv().ok();

    let exchanges = ["BINANCE", "BYBIT", "OKX"];
    let mut configs = Vec::new();

    for exchange in exchanges {
        let key_var = format!("{}_API_KEY", exchange);
        let secret_var = format!("{}_API_SECRET", exchange);
        let testnet_var = format!("{}_TESTNET", exchange);

        if let (Ok(key), Ok(secret)) = (env::var(&key_var), env::var(&secret_var)) {
            configs.push(ExchangeCredentials {
                name: exchange.to_lowercase(),
                api_key: key,
                api_secret: secret,
                is_testnet: env::var(&testnet_var)
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(true),
            });
        }
    }

    configs
}

fn main() {
    let exchanges = load_exchange_configs();

    println!("Loaded {} exchange configurations:", exchanges.len());
    for ex in &exchanges {
        println!(
            "  {} ({}): API key {}...{}",
            ex.name,
            if ex.is_testnet { "testnet" } else { "mainnet" },
            &ex.api_key[..4],
            &ex.api_key[ex.api_key.len()-4..]
        );
    }
}
```

Файл `.env` для этого примера:

```env
# Binance
BINANCE_API_KEY=abc123...
BINANCE_API_SECRET=xyz789...
BINANCE_TESTNET=true

# Bybit
BYBIT_API_KEY=def456...
BYBIT_API_SECRET=uvw012...
BYBIT_TESTNET=true

# OKX (not configured yet, will be skipped)
# OKX_API_KEY=
# OKX_API_SECRET=
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `.env` файл | Файл с переменными окружения |
| `dotenvy` | Библиотека для загрузки `.env` |
| `.env.example` | Шаблон для документации |
| Разные окружения | `.env.development`, `.env.production` |
| Валидация | Проверка конфига при старте |
| Безопасность | Не коммитить секреты, маскировать вывод |

## Практические задания

1. **Конфиг позиции:** Создай `.env` файл с настройками для расчёта размера позиции (баланс, риск, комиссия) и программу, которая их загружает и валидирует.

2. **Мульти-окружение:** Создай три файла `.env.development`, `.env.staging`, `.env.production` с разными настройками и программу, которая загружает нужный на основе переменной `APP_ENV`.

3. **Безопасный вывод:** Напиши функцию `print_config_summary()`, которая выводит все настройки, но маскирует секретные значения (API ключи, пароли).

## Домашнее задание

1. Создай полный конфигурационный файл для торгового бота со всеми необходимыми параметрами: биржа, стратегия, риск-менеджмент, уведомления.

2. Напиши структуру `TradingBotConfig` с методом `from_env()`, который загружает все параметры из `.env` с валидацией и значениями по умолчанию.

3. Реализуй функцию `validate_trading_config()`, которая проверяет:
   - Все обязательные поля заполнены
   - Числовые значения в допустимых диапазонах
   - API ключи имеют правильный формат
   - Возвращает список всех найденных ошибок

4. Создай модуль для работы с конфигами нескольких бирж одновременно, с возможностью переключения между ними.

## Навигация

[← Предыдущий день](../141-environment-variables-api-secrets/ru.md) | [Следующий день →](../143-command-line-arguments-clap/ru.md)
