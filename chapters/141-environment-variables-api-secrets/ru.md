# День 141: Переменные окружения — секреты API

## Аналогия из трейдинга

Представь, что у тебя есть сейф в банке. Ты не носишь ключ от сейфа на виду — ты хранишь его в надёжном месте. API-ключи от бирж (Binance, Bybit, Coinbase) — это твои "ключи от сейфа" с деньгами. Хранить их прямо в коде — всё равно что написать пароль на стикере и приклеить на монитор.

**Переменные окружения** — это безопасный способ хранения секретов: ключей API, паролей, токенов. Они живут в операционной системе, а не в коде.

## Почему это критически важно

```rust
// ❌ НИКОГДА ТАК НЕ ДЕЛАЙ!
const API_KEY: &str = "sk-abc123xyz789";
const API_SECRET: &str = "super-secret-key";

fn main() {
    // Если ты загрузишь этот код на GitHub —
    // боты найдут ключи за секунды и опустошат счёт!
}
```

**Реальные случаи:**
- Трейдер потерял $50,000 за 2 минуты после случайного коммита API-ключей
- Боты сканируют GitHub 24/7 в поисках ключей от бирж
- Утечка ключей = полный доступ к твоему счёту

## Чтение переменных окружения

### Базовый способ: std::env

```rust
use std::env;

fn main() {
    // Получаем ключ API биржи
    match env::var("BINANCE_API_KEY") {
        Ok(key) => println!("API Key загружен: {}...", &key[..8]),
        Err(_) => println!("BINANCE_API_KEY не установлен!"),
    }
}
```

### Безопасная проверка наличия

```rust
use std::env;

fn main() {
    // Проверяем все необходимые переменные перед запуском бота
    let required_vars = [
        "EXCHANGE_API_KEY",
        "EXCHANGE_API_SECRET",
        "TELEGRAM_BOT_TOKEN",
    ];

    let mut missing = Vec::new();

    for var in &required_vars {
        if env::var(var).is_err() {
            missing.push(*var);
        }
    }

    if !missing.is_empty() {
        println!("Ошибка: отсутствуют переменные окружения:");
        for var in &missing {
            println!("  - {}", var);
        }
        std::process::exit(1);
    }

    println!("Все переменные загружены. Бот готов к работе.");
}
```

### Значения по умолчанию

```rust
use std::env;

fn main() {
    // Для некритичных настроек можно использовать значения по умолчанию
    let trading_mode = env::var("TRADING_MODE")
        .unwrap_or_else(|_| String::from("paper")); // paper trading по умолчанию

    let max_position_size: f64 = env::var("MAX_POSITION_SIZE")
        .unwrap_or_else(|_| String::from("1000.0"))
        .parse()
        .unwrap_or(1000.0);

    let log_level = env::var("LOG_LEVEL")
        .unwrap_or_else(|_| String::from("info"));

    println!("Режим: {}", trading_mode);
    println!("Макс. позиция: ${}", max_position_size);
    println!("Уровень логов: {}", log_level);
}
```

## Структура конфигурации торгового бота

```rust
use std::env;

struct ExchangeConfig {
    api_key: String,
    api_secret: String,
    testnet: bool,
}

struct TradingConfig {
    max_position_usd: f64,
    risk_per_trade_percent: f64,
    allowed_symbols: Vec<String>,
}

struct BotConfig {
    exchange: ExchangeConfig,
    trading: TradingConfig,
    telegram_token: Option<String>,
}

impl BotConfig {
    fn from_env() -> Result<Self, String> {
        // Обязательные переменные
        let api_key = env::var("EXCHANGE_API_KEY")
            .map_err(|_| "EXCHANGE_API_KEY не установлен")?;

        let api_secret = env::var("EXCHANGE_API_SECRET")
            .map_err(|_| "EXCHANGE_API_SECRET не установлен")?;

        // Опциональные с дефолтами
        let testnet = env::var("USE_TESTNET")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true); // По умолчанию — тестовая сеть!

        let max_position_usd: f64 = env::var("MAX_POSITION_USD")
            .unwrap_or_else(|_| String::from("100.0"))
            .parse()
            .map_err(|_| "MAX_POSITION_USD должен быть числом")?;

        let risk_per_trade: f64 = env::var("RISK_PER_TRADE_PERCENT")
            .unwrap_or_else(|_| String::from("1.0"))
            .parse()
            .map_err(|_| "RISK_PER_TRADE_PERCENT должен быть числом")?;

        // Парсим список символов из переменной окружения
        let symbols_str = env::var("ALLOWED_SYMBOLS")
            .unwrap_or_else(|_| String::from("BTCUSDT,ETHUSDT"));
        let allowed_symbols: Vec<String> = symbols_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // Полностью опциональные
        let telegram_token = env::var("TELEGRAM_BOT_TOKEN").ok();

        Ok(BotConfig {
            exchange: ExchangeConfig {
                api_key,
                api_secret,
                testnet,
            },
            trading: TradingConfig {
                max_position_usd,
                risk_per_trade_percent: risk_per_trade,
                allowed_symbols,
            },
            telegram_token,
        })
    }
}

fn main() {
    match BotConfig::from_env() {
        Ok(config) => {
            println!("Конфигурация загружена:");
            println!("  API Key: {}...", &config.exchange.api_key[..8.min(config.exchange.api_key.len())]);
            println!("  Testnet: {}", config.exchange.testnet);
            println!("  Макс. позиция: ${}", config.trading.max_position_usd);
            println!("  Риск на сделку: {}%", config.trading.risk_per_trade_percent);
            println!("  Символы: {:?}", config.trading.allowed_symbols);
            println!("  Telegram: {}", if config.telegram_token.is_some() { "настроен" } else { "не настроен" });
        }
        Err(e) => {
            eprintln!("Ошибка конфигурации: {}", e);
            std::process::exit(1);
        }
    }
}
```

## Файл .env и библиотека dotenv

На практике переменные часто хранятся в файле `.env`:

```bash
# .env файл (НЕ КОММИТИТЬ В GIT!)
EXCHANGE_API_KEY=your-api-key-here
EXCHANGE_API_SECRET=your-secret-here
USE_TESTNET=true
MAX_POSITION_USD=500.0
RISK_PER_TRADE_PERCENT=2.0
ALLOWED_SYMBOLS=BTCUSDT,ETHUSDT,SOLUSDT
```

```rust
// Для использования .env файла добавь в Cargo.toml:
// [dependencies]
// dotenv = "0.15"

use std::env;

fn load_dotenv() {
    // Простая реализация загрузки .env без внешних зависимостей
    if let Ok(contents) = std::fs::read_to_string(".env") {
        for line in contents.lines() {
            let line = line.trim();

            // Пропускаем комментарии и пустые строки
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Разбираем KEY=VALUE
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();

                // Устанавливаем переменную, если она ещё не установлена
                if env::var(key).is_err() {
                    env::set_var(key, value);
                }
            }
        }
    }
}

fn main() {
    load_dotenv();

    // Теперь можно использовать переменные из .env
    if let Ok(key) = env::var("EXCHANGE_API_KEY") {
        println!("Ключ загружен из .env");
    }
}
```

## Маскирование секретов в логах

```rust
use std::env;

/// Маскирует секретную строку, показывая только первые и последние символы
fn mask_secret(secret: &str, visible_chars: usize) -> String {
    if secret.len() <= visible_chars * 2 {
        return "*".repeat(secret.len());
    }

    let start = &secret[..visible_chars];
    let end = &secret[secret.len() - visible_chars..];
    let middle = "*".repeat(secret.len() - visible_chars * 2);

    format!("{}{}{}", start, middle, end)
}

/// Безопасный вывод конфигурации API
fn log_api_config(api_key: &str, api_secret: &str) {
    println!("API Configuration:");
    println!("  Key:    {}", mask_secret(api_key, 4));
    println!("  Secret: {}", mask_secret(api_secret, 2));
}

fn main() {
    let api_key = env::var("API_KEY")
        .unwrap_or_else(|_| String::from("test-key-12345678"));
    let api_secret = env::var("API_SECRET")
        .unwrap_or_else(|_| String::from("super-secret-password-123"));

    log_api_config(&api_key, &api_secret);

    // Примеры маскирования
    println!("\nПримеры маскирования:");
    println!("  'abcdefghij' -> '{}'", mask_secret("abcdefghij", 2));
    println!("  'short' -> '{}'", mask_secret("short", 2));
}
```

## Валидация API-ключей

```rust
use std::env;

#[derive(Debug)]
enum ApiKeyError {
    Missing(String),
    TooShort { key_name: String, min_length: usize },
    InvalidFormat(String),
}

fn validate_binance_key(key: &str) -> Result<(), ApiKeyError> {
    // Binance API ключи обычно 64 символа
    if key.len() < 64 {
        return Err(ApiKeyError::TooShort {
            key_name: String::from("BINANCE_API_KEY"),
            min_length: 64,
        });
    }

    // Проверяем, что ключ содержит только допустимые символы
    if !key.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ApiKeyError::InvalidFormat(
            String::from("Ключ должен содержать только буквы и цифры")
        ));
    }

    Ok(())
}

fn load_and_validate_api_keys() -> Result<(String, String), ApiKeyError> {
    let api_key = env::var("BINANCE_API_KEY")
        .map_err(|_| ApiKeyError::Missing(String::from("BINANCE_API_KEY")))?;

    let api_secret = env::var("BINANCE_API_SECRET")
        .map_err(|_| ApiKeyError::Missing(String::from("BINANCE_API_SECRET")))?;

    validate_binance_key(&api_key)?;
    validate_binance_key(&api_secret)?;

    Ok((api_key, api_secret))
}

fn main() {
    match load_and_validate_api_keys() {
        Ok((key, secret)) => {
            println!("API ключи загружены и провалидированы");
            println!("Key length: {}, Secret length: {}", key.len(), secret.len());
        }
        Err(ApiKeyError::Missing(name)) => {
            eprintln!("Ошибка: переменная {} не установлена", name);
        }
        Err(ApiKeyError::TooShort { key_name, min_length }) => {
            eprintln!("Ошибка: {} слишком короткий (мин. {} символов)", key_name, min_length);
        }
        Err(ApiKeyError::InvalidFormat(msg)) => {
            eprintln!("Ошибка формата: {}", msg);
        }
    }
}
```

## Работа с разными окружениями

```rust
use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    fn from_env() -> Self {
        match env::var("RUST_ENV").as_deref() {
            Ok("production") | Ok("prod") => Environment::Production,
            Ok("staging") | Ok("stage") => Environment::Staging,
            _ => Environment::Development,
        }
    }

    fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    fn api_base_url(&self) -> &'static str {
        match self {
            Environment::Development => "https://testnet.binance.vision",
            Environment::Staging => "https://testnet.binance.vision",
            Environment::Production => "https://api.binance.com",
        }
    }

    fn default_max_position(&self) -> f64 {
        match self {
            Environment::Development => 100.0,    // $100 для разработки
            Environment::Staging => 1000.0,       // $1000 для стейджинга
            Environment::Production => 10000.0,   // $10000 для продакшена
        }
    }
}

fn main() {
    let env = Environment::from_env();

    println!("Текущее окружение: {:?}", env);
    println!("API URL: {}", env.api_base_url());
    println!("Макс. позиция по умолчанию: ${}", env.default_max_position());

    if env.is_production() {
        println!("\n⚠️  ВНИМАНИЕ: Работа в PRODUCTION режиме!");
        println!("    Все сделки будут реальными!");
    }
}
```

## Практический пример: полный конфиг торгового бота

```rust
use std::env;
use std::collections::HashMap;

#[derive(Debug)]
struct TradingBotConfig {
    // Биржа
    exchange_name: String,
    api_key: String,
    api_secret: String,
    is_testnet: bool,

    // Торговля
    symbols: Vec<String>,
    max_position_usd: f64,
    max_daily_trades: u32,
    risk_percent: f64,

    // Уведомления
    telegram_chat_id: Option<String>,
    telegram_bot_token: Option<String>,

    // Дополнительные настройки
    extra: HashMap<String, String>,
}

impl TradingBotConfig {
    fn from_env() -> Result<Self, String> {
        // Обязательные
        let exchange_name = env::var("EXCHANGE")
            .unwrap_or_else(|_| String::from("binance"));

        let api_key = env::var("API_KEY")
            .map_err(|_| "API_KEY is required")?;

        let api_secret = env::var("API_SECRET")
            .map_err(|_| "API_SECRET is required")?;

        // С дефолтами
        let is_testnet = env::var("TESTNET")
            .map(|v| v == "true" || v == "1" || v == "yes")
            .unwrap_or(true);

        let symbols: Vec<String> = env::var("TRADING_SYMBOLS")
            .unwrap_or_else(|_| String::from("BTCUSDT"))
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .collect();

        let max_position_usd: f64 = env::var("MAX_POSITION_USD")
            .unwrap_or_else(|_| String::from("100"))
            .parse()
            .unwrap_or(100.0);

        let max_daily_trades: u32 = env::var("MAX_DAILY_TRADES")
            .unwrap_or_else(|_| String::from("10"))
            .parse()
            .unwrap_or(10);

        let risk_percent: f64 = env::var("RISK_PERCENT")
            .unwrap_or_else(|_| String::from("1.0"))
            .parse()
            .unwrap_or(1.0)
            .min(5.0); // Не более 5% риска

        // Опциональные
        let telegram_chat_id = env::var("TELEGRAM_CHAT_ID").ok();
        let telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN").ok();

        // Собираем дополнительные настройки с префиксом BOT_
        let extra: HashMap<String, String> = env::vars()
            .filter(|(k, _)| k.starts_with("BOT_"))
            .map(|(k, v)| (k.strip_prefix("BOT_").unwrap().to_string(), v))
            .collect();

        Ok(TradingBotConfig {
            exchange_name,
            api_key,
            api_secret,
            is_testnet,
            symbols,
            max_position_usd,
            max_daily_trades,
            risk_percent,
            telegram_chat_id,
            telegram_bot_token,
            extra,
        })
    }

    fn print_summary(&self) {
        println!("╔═══════════════════════════════════════════╗");
        println!("║         TRADING BOT CONFIGURATION         ║");
        println!("╠═══════════════════════════════════════════╣");
        println!("║ Exchange:     {:>27} ║", self.exchange_name);
        println!("║ Mode:         {:>27} ║", if self.is_testnet { "TESTNET" } else { "PRODUCTION" });
        println!("║ API Key:      {:>27} ║", format!("{}...", &self.api_key[..8.min(self.api_key.len())]));
        println!("║ Symbols:      {:>27} ║", self.symbols.join(", "));
        println!("║ Max Position: {:>26}$ ║", self.max_position_usd);
        println!("║ Max Trades:   {:>27} ║", self.max_daily_trades);
        println!("║ Risk:         {:>26}% ║", self.risk_percent);
        println!("║ Telegram:     {:>27} ║",
            if self.telegram_bot_token.is_some() { "Enabled" } else { "Disabled" });
        println!("╚═══════════════════════════════════════════╝");

        if !self.extra.is_empty() {
            println!("\nДополнительные настройки:");
            for (key, value) in &self.extra {
                println!("  {}: {}", key, value);
            }
        }
    }
}

fn main() {
    println!("Загрузка конфигурации...\n");

    match TradingBotConfig::from_env() {
        Ok(config) => {
            config.print_summary();

            if !config.is_testnet {
                println!("\n🚨 ВНИМАНИЕ: PRODUCTION MODE! 🚨");
                println!("Все сделки будут РЕАЛЬНЫМИ!");
            }
        }
        Err(e) => {
            eprintln!("❌ Ошибка загрузки конфигурации: {}", e);
            eprintln!("\nПример настройки переменных окружения:");
            eprintln!("  export API_KEY=\"your-api-key\"");
            eprintln!("  export API_SECRET=\"your-api-secret\"");
            eprintln!("  export TESTNET=true");
            std::process::exit(1);
        }
    }
}
```

## Что мы узнали

| Концепция | Описание | Пример |
|-----------|----------|--------|
| `env::var()` | Чтение переменной | `env::var("API_KEY")` |
| `unwrap_or_else` | Значение по умолчанию | `var.unwrap_or_else(\|_\| default)` |
| `.env` файл | Локальное хранение | Добавь в `.gitignore`! |
| Маскирование | Скрытие в логах | `key[..4] + "***"` |
| Валидация | Проверка формата | Длина, символы, формат |

## Правила безопасности

1. **НИКОГДА** не коммить секреты в Git
2. Добавь `.env` в `.gitignore`
3. Используй разные ключи для dev/prod
4. Маскируй секреты в логах
5. По умолчанию — testnet режим

## Домашнее задание

1. Напиши функцию `load_multi_exchange_config()`, которая загружает конфигурацию для нескольких бирж (Binance, Bybit, Coinbase) из переменных с префиксами

2. Создай валидатор API-ключей, который проверяет формат ключей для разных бирж (у каждой свои правила)

3. Реализуй систему "секретного хранилища" с шифрованием: ключи хранятся в зашифрованном виде, и для расшифровки нужен мастер-пароль из переменной окружения

4. Напиши утилиту проверки безопасности, которая сканирует `.rs` файлы проекта на наличие захардкоженных секретов (строки, похожие на API-ключи)

## Навигация

[← Предыдущий день](../140-file-output-trade-exports/ru.md) | [Следующий день →](../142-file-reading-historical-data/ru.md)
