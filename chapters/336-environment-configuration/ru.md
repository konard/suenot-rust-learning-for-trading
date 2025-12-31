# День 336: Конфигурация для разных сред

## Аналогия из трейдинга

Представь, что у тебя есть торговый бот, который работает в трёх режимах:

**Разработка (Development):**
Ты тестируешь новые стратегии на исторических данных. Используешь тестовые API ключи, маленькие объёмы, подробное логирование. Ошибки — это нормально, ты учишься.

**Тестирование (Staging):**
Бот подключён к тестовой сети биржи (testnet). Торгует "бумажными" деньгами, но в реальном времени. Проверяешь, что всё работает перед выходом в продакшн.

**Продакшн (Production):**
Реальные деньги, реальные сделки. Минимальное логирование, максимальная производительность, строгие лимиты риска.

| Среда | API | Объёмы | Логирование | Риск |
|-------|-----|--------|-------------|------|
| **Development** | Тестовый | Минимальные | Подробное | Нулевой |
| **Staging** | Testnet | Средние | Умеренное | Низкий |
| **Production** | Реальный | Рабочие | Минимальное | Реальный |

## Переменные окружения

Основной способ конфигурации в Rust — переменные окружения:

```rust
use std::env;

/// Конфигурация торгового бота
#[derive(Debug, Clone)]
struct TradingConfig {
    /// Текущее окружение
    environment: Environment,
    /// API ключ биржи
    api_key: String,
    /// API секрет
    api_secret: String,
    /// Базовый URL биржи
    api_base_url: String,
    /// Максимальный размер позиции
    max_position_size: f64,
    /// Максимальный риск на сделку в процентах
    max_risk_percent: f64,
    /// Уровень логирования
    log_level: String,
}

#[derive(Debug, Clone, PartialEq)]
enum Environment {
    Development,
    Staging,
    Production,
}

impl TradingConfig {
    /// Загружает конфигурацию из переменных окружения
    fn from_env() -> Result<Self, String> {
        // Определяем окружение
        let env_str = env::var("TRADING_ENV")
            .unwrap_or_else(|_| "development".to_string());

        let environment = match env_str.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        };

        // Загружаем API ключи
        let api_key = env::var("TRADING_API_KEY")
            .map_err(|_| "TRADING_API_KEY не установлен")?;

        let api_secret = env::var("TRADING_API_SECRET")
            .map_err(|_| "TRADING_API_SECRET не установлен")?;

        // URL зависит от окружения
        let api_base_url = match environment {
            Environment::Production => {
                env::var("TRADING_API_URL")
                    .unwrap_or_else(|_| "https://api.exchange.com".to_string())
            }
            Environment::Staging => {
                "https://testnet.exchange.com".to_string()
            }
            Environment::Development => {
                "http://localhost:8080".to_string()
            }
        };

        // Лимиты риска зависят от окружения
        let (max_position_size, max_risk_percent) = match environment {
            Environment::Production => (10000.0, 2.0),
            Environment::Staging => (1000.0, 5.0),
            Environment::Development => (100.0, 10.0),
        };

        // Уровень логирования
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| {
            match environment {
                Environment::Production => "warn".to_string(),
                Environment::Staging => "info".to_string(),
                Environment::Development => "debug".to_string(),
            }
        });

        Ok(TradingConfig {
            environment,
            api_key,
            api_secret,
            api_base_url,
            max_position_size,
            max_risk_percent,
            log_level,
        })
    }

    /// Проверяет, разрешена ли реальная торговля
    fn is_live_trading_allowed(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Возвращает максимальный размер ордера для текущей среды
    fn max_order_size(&self) -> f64 {
        self.max_position_size * (self.max_risk_percent / 100.0)
    }
}

fn main() {
    // Для демонстрации установим переменные вручную
    env::set_var("TRADING_ENV", "staging");
    env::set_var("TRADING_API_KEY", "test_key_12345");
    env::set_var("TRADING_API_SECRET", "test_secret_67890");

    match TradingConfig::from_env() {
        Ok(config) => {
            println!("=== Конфигурация торгового бота ===\n");
            println!("Окружение: {:?}", config.environment);
            println!("API URL: {}", config.api_base_url);
            println!("Макс. позиция: ${:.2}", config.max_position_size);
            println!("Макс. риск: {:.1}%", config.max_risk_percent);
            println!("Макс. ордер: ${:.2}", config.max_order_size());
            println!("Уровень лога: {}", config.log_level);
            println!("\nРеальная торговля: {}",
                if config.is_live_trading_allowed() { "ДА" } else { "НЕТ" }
            );
        }
        Err(e) => {
            eprintln!("Ошибка конфигурации: {}", e);
        }
    }
}
```

## Библиотека dotenv

Для локальной разработки удобно использовать `.env` файлы:

```rust
use std::env;
use std::fs;
use std::collections::HashMap;

/// Простой парсер .env файлов
fn load_dotenv(path: &str) -> Result<HashMap<String, String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Не удалось прочитать {}: {}", path, e))?;

    let mut vars = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Пропускаем пустые строки и комментарии
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Разбираем KEY=VALUE
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            // Убираем кавычки если есть
            let value = value.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            vars.insert(key, value);
        }
    }

    Ok(vars)
}

/// Загружает .env файл и устанавливает переменные окружения
fn init_dotenv() {
    // Определяем какой файл загружать
    let env_file = match env::var("TRADING_ENV").ok() {
        Some(e) if e == "production" => ".env.production",
        Some(e) if e == "staging" => ".env.staging",
        _ => ".env.development",
    };

    // Сначала пробуем специфичный файл, потом общий .env
    let files_to_try = [env_file, ".env"];

    for file in files_to_try {
        if let Ok(vars) = load_dotenv(file) {
            println!("Загружен конфиг из: {}", file);
            for (key, value) in vars {
                // Не перезаписываем уже установленные переменные
                if env::var(&key).is_err() {
                    env::set_var(&key, &value);
                }
            }
            break;
        }
    }
}

/// Конфигурация биржевого подключения
#[derive(Debug)]
struct ExchangeConfig {
    name: String,
    api_key: String,
    api_secret: String,
    base_url: String,
    rate_limit: u32,      // запросов в секунду
    timeout_ms: u64,
}

impl ExchangeConfig {
    fn from_env(prefix: &str) -> Result<Self, String> {
        let get_var = |suffix: &str| -> Result<String, String> {
            let key = format!("{}_{}", prefix, suffix);
            env::var(&key).map_err(|_| format!("{} не установлен", key))
        };

        let get_var_or_default = |suffix: &str, default: &str| -> String {
            let key = format!("{}_{}", prefix, suffix);
            env::var(&key).unwrap_or_else(|_| default.to_string())
        };

        Ok(ExchangeConfig {
            name: prefix.to_lowercase(),
            api_key: get_var("API_KEY")?,
            api_secret: get_var("API_SECRET")?,
            base_url: get_var_or_default("BASE_URL", "https://api.exchange.com"),
            rate_limit: get_var_or_default("RATE_LIMIT", "10")
                .parse()
                .unwrap_or(10),
            timeout_ms: get_var_or_default("TIMEOUT_MS", "5000")
                .parse()
                .unwrap_or(5000),
        })
    }
}

fn main() {
    println!("=== Пример .env файла ===\n");

    // Создаём тестовый .env файл
    let env_content = r#"
# Настройки торгового бота
TRADING_ENV=staging

# Binance
BINANCE_API_KEY=binance_key_123
BINANCE_API_SECRET=binance_secret_456
BINANCE_BASE_URL=https://testnet.binance.vision
BINANCE_RATE_LIMIT=20

# Kraken
KRAKEN_API_KEY=kraken_key_789
KRAKEN_API_SECRET=kraken_secret_012
KRAKEN_RATE_LIMIT=15

# Общие настройки
LOG_LEVEL=debug
MAX_CONCURRENT_ORDERS=5
"#;

    // Записываем тестовый файл
    fs::write(".env.test", env_content).unwrap();

    // Загружаем
    if let Ok(vars) = load_dotenv(".env.test") {
        println!("Загружено {} переменных:\n", vars.len());
        for (key, value) in &vars {
            // Скрываем секреты
            let display_value = if key.contains("SECRET") {
                "***".to_string()
            } else {
                value.clone()
            };
            println!("  {} = {}", key, display_value);
        }

        // Устанавливаем переменные
        for (key, value) in vars {
            env::set_var(&key, &value);
        }
    }

    println!("\n=== Конфигурации бирж ===\n");

    // Загружаем конфигурации бирж
    for exchange in ["BINANCE", "KRAKEN"] {
        match ExchangeConfig::from_env(exchange) {
            Ok(config) => {
                println!("{}: URL={}, Rate Limit={}/s",
                    config.name, config.base_url, config.rate_limit);
            }
            Err(e) => println!("{}: Ошибка - {}", exchange, e),
        }
    }

    // Удаляем тестовый файл
    let _ = fs::remove_file(".env.test");
}
```

## Конфигурационные файлы (TOML, JSON)

Для сложных конфигураций используем структурированные файлы:

```rust
use std::fs;
use std::collections::HashMap;

/// Конфигурация торговой стратегии
#[derive(Debug, Clone)]
struct StrategyConfig {
    name: String,
    enabled: bool,
    symbols: Vec<String>,
    parameters: HashMap<String, f64>,
    risk_limits: RiskLimits,
}

#[derive(Debug, Clone)]
struct RiskLimits {
    max_position_pct: f64,
    max_drawdown_pct: f64,
    max_daily_loss: f64,
    stop_loss_pct: f64,
}

/// Полная конфигурация системы
#[derive(Debug)]
struct SystemConfig {
    environment: String,
    strategies: Vec<StrategyConfig>,
    exchanges: HashMap<String, ExchangeSettings>,
    database_url: String,
    metrics_port: u16,
}

#[derive(Debug)]
struct ExchangeSettings {
    enabled: bool,
    api_key_env: String,
    api_secret_env: String,
    base_url: String,
}

/// Простой парсер TOML (упрощённая версия)
fn parse_simple_toml(content: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Секция [section]
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].to_string();
            continue;
        }

        // Ключ = значение
        if let Some((key, value)) = line.split_once('=') {
            let full_key = if current_section.is_empty() {
                key.trim().to_string()
            } else {
                format!("{}.{}", current_section, key.trim())
            };

            let value = value.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            result.insert(full_key, value);
        }
    }

    result
}

fn main() {
    println!("=== Конфигурация через TOML ===\n");

    // Пример TOML конфигурации
    let toml_content = r#"
# Конфигурация торговой системы

environment = "staging"
database_url = "postgres://localhost/trading"
metrics_port = 9090

[exchange.binance]
enabled = true
api_key_env = "BINANCE_API_KEY"
api_secret_env = "BINANCE_API_SECRET"
base_url = "https://testnet.binance.vision"

[exchange.kraken]
enabled = false
api_key_env = "KRAKEN_API_KEY"
api_secret_env = "KRAKEN_API_SECRET"
base_url = "https://api.kraken.com"

[strategy.momentum]
enabled = true
symbols = "BTCUSDT,ETHUSDT,SOLUSDT"
lookback_period = 20
entry_threshold = 0.02
exit_threshold = 0.01

[strategy.momentum.risk]
max_position_pct = 10.0
max_drawdown_pct = 5.0
max_daily_loss = 1000.0
stop_loss_pct = 2.0

[strategy.mean_reversion]
enabled = true
symbols = "BTCUSDT"
window_size = 50
std_multiplier = 2.0

[strategy.mean_reversion.risk]
max_position_pct = 5.0
max_drawdown_pct = 3.0
max_daily_loss = 500.0
stop_loss_pct = 1.5
"#;

    // Парсим TOML
    let config = parse_simple_toml(toml_content);

    println!("Окружение: {}", config.get("environment").unwrap_or(&"unknown".to_string()));
    println!("Database: {}", config.get("database_url").unwrap_or(&"".to_string()));
    println!("Metrics port: {}", config.get("metrics_port").unwrap_or(&"".to_string()));

    println!("\n--- Биржи ---");
    for exchange in ["binance", "kraken"] {
        let enabled = config.get(&format!("exchange.{}.enabled", exchange))
            .map(|s| s == "true")
            .unwrap_or(false);
        let url = config.get(&format!("exchange.{}.base_url", exchange))
            .unwrap_or(&"N/A".to_string())
            .clone();
        println!("{}: {} ({})", exchange, if enabled { "ON" } else { "OFF" }, url);
    }

    println!("\n--- Стратегии ---");
    for strategy in ["momentum", "mean_reversion"] {
        let enabled = config.get(&format!("strategy.{}.enabled", strategy))
            .map(|s| s == "true")
            .unwrap_or(false);
        let symbols = config.get(&format!("strategy.{}.symbols", strategy))
            .unwrap_or(&"".to_string())
            .clone();
        let max_pos = config.get(&format!("strategy.{}.risk.max_position_pct", strategy))
            .unwrap_or(&"0".to_string())
            .clone();

        if enabled {
            println!("{}: symbols=[{}], max_position={}%",
                strategy, symbols, max_pos);
        }
    }
}
```

## Конфигурация через Builder Pattern

Для гибкой конфигурации используем Builder pattern:

```rust
use std::env;
use std::collections::HashMap;

/// Конфигурация торгового движка
#[derive(Debug, Clone)]
struct TradingEngine {
    name: String,
    environment: String,
    exchanges: Vec<String>,
    strategies: Vec<String>,
    risk_limits: RiskConfig,
    logging: LogConfig,
    features: HashMap<String, bool>,
}

#[derive(Debug, Clone, Default)]
struct RiskConfig {
    max_position_size: f64,
    max_daily_loss: f64,
    max_drawdown: f64,
    position_sizing: String,
}

#[derive(Debug, Clone, Default)]
struct LogConfig {
    level: String,
    file_path: Option<String>,
    json_format: bool,
}

/// Builder для создания конфигурации
#[derive(Default)]
struct TradingEngineBuilder {
    name: Option<String>,
    environment: Option<String>,
    exchanges: Vec<String>,
    strategies: Vec<String>,
    risk_limits: RiskConfig,
    logging: LogConfig,
    features: HashMap<String, bool>,
}

impl TradingEngineBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    fn environment(mut self, env: &str) -> Self {
        self.environment = Some(env.to_string());
        self
    }

    fn add_exchange(mut self, exchange: &str) -> Self {
        self.exchanges.push(exchange.to_string());
        self
    }

    fn add_strategy(mut self, strategy: &str) -> Self {
        self.strategies.push(strategy.to_string());
        self
    }

    fn max_position_size(mut self, size: f64) -> Self {
        self.risk_limits.max_position_size = size;
        self
    }

    fn max_daily_loss(mut self, loss: f64) -> Self {
        self.risk_limits.max_daily_loss = loss;
        self
    }

    fn max_drawdown(mut self, drawdown: f64) -> Self {
        self.risk_limits.max_drawdown = drawdown;
        self
    }

    fn position_sizing(mut self, method: &str) -> Self {
        self.risk_limits.position_sizing = method.to_string();
        self
    }

    fn log_level(mut self, level: &str) -> Self {
        self.logging.level = level.to_string();
        self
    }

    fn log_to_file(mut self, path: &str) -> Self {
        self.logging.file_path = Some(path.to_string());
        self
    }

    fn json_logging(mut self, enabled: bool) -> Self {
        self.logging.json_format = enabled;
        self
    }

    fn feature(mut self, name: &str, enabled: bool) -> Self {
        self.features.insert(name.to_string(), enabled);
        self
    }

    /// Создаёт конфигурацию на основе переменных окружения
    fn from_env(mut self) -> Self {
        // Окружение
        if let Ok(env) = env::var("TRADING_ENV") {
            self.environment = Some(env);
        }

        // Риск-лимиты из окружения
        if let Ok(val) = env::var("MAX_POSITION_SIZE") {
            if let Ok(size) = val.parse() {
                self.risk_limits.max_position_size = size;
            }
        }

        if let Ok(val) = env::var("MAX_DAILY_LOSS") {
            if let Ok(loss) = val.parse() {
                self.risk_limits.max_daily_loss = loss;
            }
        }

        // Логирование
        if let Ok(level) = env::var("LOG_LEVEL") {
            self.logging.level = level;
        }

        self
    }

    fn build(self) -> Result<TradingEngine, String> {
        let name = self.name.ok_or("Имя движка не указано")?;
        let environment = self.environment.unwrap_or_else(|| "development".to_string());

        if self.exchanges.is_empty() {
            return Err("Не указано ни одной биржи".to_string());
        }

        Ok(TradingEngine {
            name,
            environment,
            exchanges: self.exchanges,
            strategies: self.strategies,
            risk_limits: self.risk_limits,
            logging: self.logging,
            features: self.features,
        })
    }
}

/// Предустановленные конфигурации для разных окружений
impl TradingEngineBuilder {
    fn development() -> Self {
        Self::new()
            .environment("development")
            .max_position_size(100.0)
            .max_daily_loss(50.0)
            .max_drawdown(10.0)
            .position_sizing("fixed")
            .log_level("debug")
            .feature("paper_trading", true)
            .feature("backtesting", true)
            .feature("live_trading", false)
    }

    fn staging() -> Self {
        Self::new()
            .environment("staging")
            .max_position_size(1000.0)
            .max_daily_loss(200.0)
            .max_drawdown(5.0)
            .position_sizing("percent")
            .log_level("info")
            .feature("paper_trading", true)
            .feature("backtesting", true)
            .feature("live_trading", false)
    }

    fn production() -> Self {
        Self::new()
            .environment("production")
            .max_position_size(10000.0)
            .max_daily_loss(500.0)
            .max_drawdown(3.0)
            .position_sizing("kelly")
            .log_level("warn")
            .json_logging(true)
            .feature("paper_trading", false)
            .feature("backtesting", false)
            .feature("live_trading", true)
    }
}

fn main() {
    println!("=== Builder Pattern для конфигурации ===\n");

    // Создаём конфигурацию для разработки
    let dev_engine = TradingEngineBuilder::development()
        .name("DevBot")
        .add_exchange("binance_testnet")
        .add_strategy("momentum")
        .add_strategy("mean_reversion")
        .build()
        .unwrap();

    println!("Development:\n{:#?}\n", dev_engine);

    // Продакшн конфигурация
    let prod_engine = TradingEngineBuilder::production()
        .name("ProdBot")
        .add_exchange("binance")
        .add_exchange("kraken")
        .add_strategy("momentum")
        .log_to_file("/var/log/trading/bot.log")
        .from_env()  // Перезаписываем из окружения
        .build()
        .unwrap();

    println!("Production:\n{:#?}", prod_engine);
}
```

## Валидация конфигурации

Критически важно проверять конфигурацию при запуске:

```rust
use std::env;

#[derive(Debug)]
struct ValidatedConfig {
    environment: String,
    api_key: String,
    max_position: f64,
    risk_per_trade: f64,
    allowed_symbols: Vec<String>,
}

#[derive(Debug)]
enum ConfigError {
    MissingRequired(String),
    InvalidValue { field: String, value: String, reason: String },
    SecurityRisk(String),
    InconsistentConfig(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingRequired(field) =>
                write!(f, "Обязательное поле отсутствует: {}", field),
            ConfigError::InvalidValue { field, value, reason } =>
                write!(f, "Некорректное значение {}: '{}' - {}", field, value, reason),
            ConfigError::SecurityRisk(msg) =>
                write!(f, "Риск безопасности: {}", msg),
            ConfigError::InconsistentConfig(msg) =>
                write!(f, "Несогласованная конфигурация: {}", msg),
        }
    }
}

struct ConfigValidator {
    errors: Vec<ConfigError>,
    warnings: Vec<String>,
}

impl ConfigValidator {
    fn new() -> Self {
        ConfigValidator {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn validate_required(&mut self, key: &str) -> Option<String> {
        match env::var(key) {
            Ok(value) if !value.is_empty() => Some(value),
            _ => {
                self.errors.push(ConfigError::MissingRequired(key.to_string()));
                None
            }
        }
    }

    fn validate_positive_f64(&mut self, key: &str, default: f64) -> f64 {
        match env::var(key) {
            Ok(value) => match value.parse::<f64>() {
                Ok(num) if num > 0.0 => num,
                Ok(num) => {
                    self.errors.push(ConfigError::InvalidValue {
                        field: key.to_string(),
                        value: num.to_string(),
                        reason: "должно быть положительным".to_string(),
                    });
                    default
                }
                Err(_) => {
                    self.errors.push(ConfigError::InvalidValue {
                        field: key.to_string(),
                        value,
                        reason: "не является числом".to_string(),
                    });
                    default
                }
            },
            Err(_) => default,
        }
    }

    fn validate_range(&mut self, key: &str, min: f64, max: f64, default: f64) -> f64 {
        let value = self.validate_positive_f64(key, default);
        if value < min || value > max {
            self.errors.push(ConfigError::InvalidValue {
                field: key.to_string(),
                value: value.to_string(),
                reason: format!("должно быть в диапазоне [{}, {}]", min, max),
            });
            default
        } else {
            value
        }
    }

    fn validate_api_key(&mut self, key: &str) -> Option<String> {
        match env::var(key) {
            Ok(value) => {
                // Проверяем формат ключа
                if value.len() < 16 {
                    self.errors.push(ConfigError::InvalidValue {
                        field: key.to_string(),
                        value: "***".to_string(),
                        reason: "слишком короткий (минимум 16 символов)".to_string(),
                    });
                    return None;
                }

                // Предупреждаем о тестовых ключах в продакшне
                if env::var("TRADING_ENV").unwrap_or_default() == "production" {
                    if value.starts_with("test") || value.starts_with("demo") {
                        self.errors.push(ConfigError::SecurityRisk(
                            format!("{} выглядит как тестовый ключ в продакшне", key)
                        ));
                    }
                }

                Some(value)
            }
            Err(_) => {
                self.errors.push(ConfigError::MissingRequired(key.to_string()));
                None
            }
        }
    }

    fn validate_symbols(&mut self, key: &str) -> Vec<String> {
        match env::var(key) {
            Ok(value) => {
                let symbols: Vec<String> = value
                    .split(',')
                    .map(|s| s.trim().to_uppercase())
                    .filter(|s| !s.is_empty())
                    .collect();

                // Проверяем формат символов
                for symbol in &symbols {
                    if !symbol.chars().all(|c| c.is_alphanumeric()) {
                        self.warnings.push(
                            format!("Символ '{}' содержит спец-символы", symbol)
                        );
                    }
                }

                if symbols.is_empty() {
                    self.warnings.push("Список символов пуст".to_string());
                }

                symbols
            }
            Err(_) => {
                self.warnings.push(
                    format!("{} не задан, используются символы по умолчанию", key)
                );
                vec!["BTCUSDT".to_string()]
            }
        }
    }

    fn cross_validate(&mut self, environment: &str, max_position: f64, risk: f64) {
        // Проверяем согласованность лимитов
        if environment == "production" && max_position > 50000.0 {
            self.warnings.push(
                "Очень высокий лимит позиции для продакшна".to_string()
            );
        }

        if environment == "development" && risk > 10.0 {
            self.errors.push(ConfigError::InconsistentConfig(
                "Риск > 10% для development — возможна ошибка".to_string()
            ));
        }

        // Production требует строгих лимитов
        if environment == "production" && risk > 5.0 {
            self.errors.push(ConfigError::SecurityRisk(
                "Риск > 5% в продакшне запрещён политикой".to_string()
            ));
        }
    }

    fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    fn print_report(&self) {
        if !self.warnings.is_empty() {
            println!("=== Предупреждения ===");
            for warning in &self.warnings {
                println!("  WARN: {}", warning);
            }
            println!();
        }

        if !self.errors.is_empty() {
            println!("=== Ошибки ===");
            for error in &self.errors {
                println!("  ERROR: {}", error);
            }
            println!();
        }
    }
}

fn load_validated_config() -> Result<ValidatedConfig, Vec<ConfigError>> {
    let mut validator = ConfigValidator::new();

    // Загружаем и валидируем
    let environment = validator.validate_required("TRADING_ENV")
        .unwrap_or_else(|| "development".to_string());

    let api_key = validator.validate_api_key("TRADING_API_KEY")
        .unwrap_or_default();

    let max_position = validator.validate_positive_f64("MAX_POSITION", 1000.0);
    let risk_per_trade = validator.validate_range("RISK_PER_TRADE", 0.1, 10.0, 2.0);
    let allowed_symbols = validator.validate_symbols("ALLOWED_SYMBOLS");

    // Кросс-валидация
    validator.cross_validate(&environment, max_position, risk_per_trade);

    // Выводим отчёт
    validator.print_report();

    if validator.is_valid() {
        Ok(ValidatedConfig {
            environment,
            api_key,
            max_position,
            risk_per_trade,
            allowed_symbols,
        })
    } else {
        Err(validator.errors)
    }
}

fn main() {
    println!("=== Валидация конфигурации ===\n");

    // Устанавливаем тестовые переменные
    env::set_var("TRADING_ENV", "staging");
    env::set_var("TRADING_API_KEY", "staging_key_1234567890");
    env::set_var("MAX_POSITION", "5000");
    env::set_var("RISK_PER_TRADE", "3.0");
    env::set_var("ALLOWED_SYMBOLS", "BTCUSDT, ETHUSDT, SOLUSDT");

    match load_validated_config() {
        Ok(config) => {
            println!("Конфигурация загружена успешно!");
            println!("Окружение: {}", config.environment);
            println!("Макс. позиция: ${:.2}", config.max_position);
            println!("Риск на сделку: {:.1}%", config.risk_per_trade);
            println!("Символы: {:?}", config.allowed_symbols);
        }
        Err(errors) => {
            println!("Конфигурация невалидна! {} ошибок:", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
            std::process::exit(1);
        }
    }
}
```

## Секреты и безопасность

Никогда не храните секреты в коде:

```rust
use std::env;
use std::fs;

/// Безопасный загрузчик секретов
struct SecretLoader {
    environment: String,
}

impl SecretLoader {
    fn new() -> Self {
        let environment = env::var("TRADING_ENV")
            .unwrap_or_else(|_| "development".to_string());
        SecretLoader { environment }
    }

    /// Загружает секрет из разных источников в зависимости от окружения
    fn load_secret(&self, name: &str) -> Result<String, String> {
        // 1. Сначала проверяем переменные окружения
        if let Ok(value) = env::var(name) {
            return Ok(value);
        }

        // 2. В продакшне — пробуем секретные файлы (Docker secrets, Kubernetes)
        if self.environment == "production" {
            let secret_path = format!("/run/secrets/{}", name.to_lowercase());
            if let Ok(value) = fs::read_to_string(&secret_path) {
                return Ok(value.trim().to_string());
            }
        }

        // 3. Для разработки — пробуем локальный файл секретов
        if self.environment == "development" {
            let local_path = format!(".secrets/{}", name.to_lowercase());
            if let Ok(value) = fs::read_to_string(&local_path) {
                return Ok(value.trim().to_string());
            }
        }

        Err(format!("Секрет {} не найден", name))
    }

    /// Маскирует секрет для логирования
    fn mask_secret(secret: &str) -> String {
        if secret.len() <= 8 {
            return "****".to_string();
        }
        let visible = &secret[..4];
        format!("{}****", visible)
    }
}

/// Конфигурация с секретами
struct SecureConfig {
    api_key: String,
    api_secret: String,
    database_password: String,
}

impl SecureConfig {
    fn load() -> Result<Self, String> {
        let loader = SecretLoader::new();

        Ok(SecureConfig {
            api_key: loader.load_secret("TRADING_API_KEY")?,
            api_secret: loader.load_secret("TRADING_API_SECRET")?,
            database_password: loader.load_secret("DATABASE_PASSWORD")?,
        })
    }

    /// Безопасный вывод для логирования
    fn log_safe(&self) -> String {
        format!(
            "SecureConfig {{ api_key: {}, api_secret: {}, db_password: {} }}",
            SecretLoader::mask_secret(&self.api_key),
            SecretLoader::mask_secret(&self.api_secret),
            SecretLoader::mask_secret(&self.database_password),
        )
    }
}

fn main() {
    println!("=== Безопасная загрузка секретов ===\n");

    // Создаём директорию для секретов
    let _ = fs::create_dir(".secrets");

    // Создаём тестовые файлы секретов
    fs::write(".secrets/trading_api_key", "dev_api_key_12345678").unwrap();
    fs::write(".secrets/trading_api_secret", "dev_secret_87654321").unwrap();
    fs::write(".secrets/database_password", "dev_db_pass_secure").unwrap();

    // Устанавливаем окружение
    env::set_var("TRADING_ENV", "development");

    match SecureConfig::load() {
        Ok(config) => {
            println!("Секреты загружены!");
            println!("Лог-безопасный вывод: {}", config.log_safe());

            // Никогда не делайте так в реальном коде!
            // println!("API Key: {}", config.api_key);
        }
        Err(e) => {
            println!("Ошибка загрузки секретов: {}", e);
        }
    }

    // Удаляем тестовые файлы
    let _ = fs::remove_dir_all(".secrets");
}
```

## Feature Flags для торговли

Управление функциями через конфигурацию:

```rust
use std::env;
use std::collections::HashMap;

/// Флаги функций для торговой системы
#[derive(Debug, Clone)]
struct FeatureFlags {
    flags: HashMap<String, bool>,
    environment: String,
}

impl FeatureFlags {
    fn new() -> Self {
        let environment = env::var("TRADING_ENV")
            .unwrap_or_else(|_| "development".to_string());

        let mut flags = HashMap::new();

        // Устанавливаем дефолты в зависимости от окружения
        match environment.as_str() {
            "production" => {
                flags.insert("live_trading".to_string(), true);
                flags.insert("paper_trading".to_string(), false);
                flags.insert("debug_logging".to_string(), false);
                flags.insert("experimental_strategies".to_string(), false);
                flags.insert("margin_trading".to_string(), false);
                flags.insert("auto_compound".to_string(), true);
            }
            "staging" => {
                flags.insert("live_trading".to_string(), false);
                flags.insert("paper_trading".to_string(), true);
                flags.insert("debug_logging".to_string(), true);
                flags.insert("experimental_strategies".to_string(), true);
                flags.insert("margin_trading".to_string(), false);
                flags.insert("auto_compound".to_string(), true);
            }
            _ => {
                flags.insert("live_trading".to_string(), false);
                flags.insert("paper_trading".to_string(), true);
                flags.insert("debug_logging".to_string(), true);
                flags.insert("experimental_strategies".to_string(), true);
                flags.insert("margin_trading".to_string(), true);
                flags.insert("auto_compound".to_string(), false);
            }
        }

        // Переопределяем из переменных окружения
        for (flag, _) in flags.clone().iter() {
            let env_key = format!("FEATURE_{}", flag.to_uppercase());
            if let Ok(value) = env::var(&env_key) {
                let enabled = value == "1" || value.to_lowercase() == "true";
                flags.insert(flag.clone(), enabled);
            }
        }

        FeatureFlags { flags, environment }
    }

    fn is_enabled(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    fn require(&self, flag: &str) -> Result<(), String> {
        if self.is_enabled(flag) {
            Ok(())
        } else {
            Err(format!("Feature '{}' отключена в окружении '{}'",
                flag, self.environment))
        }
    }
}

/// Торговый движок с feature flags
struct TradingEngine {
    features: FeatureFlags,
    balance: f64,
}

impl TradingEngine {
    fn new(initial_balance: f64) -> Self {
        TradingEngine {
            features: FeatureFlags::new(),
            balance: initial_balance,
        }
    }

    fn execute_trade(&mut self, symbol: &str, side: &str, amount: f64) -> Result<String, String> {
        // Проверяем можем ли торговать
        if self.features.is_enabled("live_trading") {
            println!("[LIVE] Исполняем реальную сделку: {} {} {}", side, amount, symbol);
            // Реальная логика торговли
            Ok(format!("LIVE-ORDER-{}", chrono_mock()))
        } else if self.features.is_enabled("paper_trading") {
            println!("[PAPER] Симулируем сделку: {} {} {}", side, amount, symbol);
            // Обновляем виртуальный баланс
            if side == "BUY" {
                self.balance -= amount * 50000.0; // примерная цена
            } else {
                self.balance += amount * 50000.0;
            }
            Ok(format!("PAPER-ORDER-{}", chrono_mock()))
        } else {
            Err("Ни live_trading, ни paper_trading не включены".to_string())
        }
    }

    fn enable_margin(&mut self) -> Result<(), String> {
        self.features.require("margin_trading")?;
        println!("Маржинальная торговля активирована");
        Ok(())
    }

    fn run_experimental_strategy(&self, name: &str) -> Result<(), String> {
        self.features.require("experimental_strategies")?;
        println!("Запуск экспериментальной стратегии: {}", name);
        Ok(())
    }

    fn status(&self) {
        println!("\n=== Статус движка ===");
        println!("Баланс: ${:.2}", self.balance);
        println!("\nВключённые функции:");
        for (flag, enabled) in &self.features.flags {
            let status = if *enabled { "ON" } else { "OFF" };
            println!("  {}: {}", flag, status);
        }
    }
}

fn chrono_mock() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn main() {
    println!("=== Feature Flags для торговли ===\n");

    // Устанавливаем staging окружение
    env::set_var("TRADING_ENV", "staging");
    // Включаем экспериментальную функцию
    env::set_var("FEATURE_EXPERIMENTAL_STRATEGIES", "true");

    let mut engine = TradingEngine::new(10000.0);

    // Статус
    engine.status();

    println!("\n=== Операции ===\n");

    // Пробуем торговать
    match engine.execute_trade("BTCUSDT", "BUY", 0.1) {
        Ok(order_id) => println!("Ордер создан: {}", order_id),
        Err(e) => println!("Ошибка: {}", e),
    }

    // Пробуем включить маржу (должно не сработать в staging)
    match engine.enable_margin() {
        Ok(_) => println!("Маржа включена"),
        Err(e) => println!("Маржа недоступна: {}", e),
    }

    // Экспериментальная стратегия (должна работать)
    match engine.run_experimental_strategy("quantum_prediction") {
        Ok(_) => println!("Стратегия запущена"),
        Err(e) => println!("Стратегия недоступна: {}", e),
    }

    engine.status();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Переменные окружения** | Основной способ конфигурации через `std::env` |
| **dotenv** | Файлы `.env` для локальной разработки |
| **TOML/JSON** | Структурированные конфигурационные файлы |
| **Builder Pattern** | Гибкое создание конфигурации |
| **Валидация** | Проверка конфигурации при запуске |
| **Секреты** | Безопасное хранение API ключей |
| **Feature Flags** | Управление функциями через конфигурацию |

## Практические задания

1. **Мульти-биржевой конфигуратор**: Создай систему, которая:
   - Загружает конфигурации для 5+ бирж
   - Поддерживает разные форматы (env, TOML, JSON)
   - Валидирует API ключи каждой биржи
   - Показывает статус подключения

2. **Динамический переключатель стратегий**: Реализуй систему:
   - Загружает список стратегий из конфига
   - Позволяет включать/отключать стратегии без перезапуска
   - Отслеживает изменения конфигурации в реальном времени
   - Логирует все изменения конфигурации

3. **Система A/B тестирования**: Создай инструмент:
   - Распределяет сделки между версиями стратегии
   - Настраивается через feature flags
   - Собирает метрики каждой версии
   - Генерирует отчёты сравнения

4. **Миграция конфигурации**: Реализуй утилиту:
   - Конвертирует старый формат конфига в новый
   - Валидирует совместимость
   - Создаёт бэкап старой конфигурации
   - Показывает diff изменений

## Домашнее задание

1. **Конфигурационный движок**: Напиши систему, которая:
   - Поддерживает наследование конфигураций (base → environment)
   - Позволяет переопределять настройки через CLI аргументы
   - Выводит итоговую конфигурацию с указанием источника каждого значения
   - Экспортирует конфиг в разные форматы
   - Имеет режим dry-run для проверки без запуска

2. **Менеджер секретов**: Реализуй систему:
   - Шифрует локальные секреты с мастер-паролем
   - Ротирует API ключи по расписанию
   - Интегрируется с HashiCorp Vault (или его эмуляцией)
   - Аудит логирует все обращения к секретам
   - Поддерживает аварийную смену всех ключей

3. **Конфигурационный сервер**: Создай сервис:
   - Централизованно хранит конфигурации для всех ботов
   - Поддерживает версионирование конфигураций
   - Позволяет откатывать изменения
   - Отправляет уведомления при изменениях
   - REST API для управления

4. **Тестирование конфигураций**: Разработай фреймворк:
   - Автоматически тестирует все комбинации feature flags
   - Проверяет совместимость окружений
   - Находит конфликтующие настройки
   - Генерирует матрицу совместимости
   - Интегрируется с CI/CD

## Навигация

[← День 326: Async vs Threading](../326-async-vs-threading/ru.md) | [День 337 →](../337-*/ru.md)
