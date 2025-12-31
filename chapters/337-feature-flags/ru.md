# День 337: Feature flags: включаем функции

## Аналогия из трейдинга

Представь, что ты разрабатываешь торговую платформу, которой пользуются тысячи трейдеров. Ты хочешь внедрить новый высокочастотный алгоритм, но развернуть его сразу для всех рискованно. Что если в нём баг, который приведёт к огромным потерям?

**Без Feature Flags:**
Ты разворачиваешь новый алгоритм для всех пользователей. Если что-то пойдёт не так, придётся откатывать всё развёртывание, затрагивая всех и теряя деньги во время простоя.

**С Feature Flags:**
Ты разворачиваешь код, но новый алгоритм скрыт за переключателем. Сначала включаешь его только для внутренней команды тестирования. Затем постепенно раскатываешь на 1%, 10%, 50% пользователей, мониторя производительность на каждом шаге. При проблемах просто переключаешь флаг — без нового развёртывания.

| Сценарий | Без флагов | С флагами |
|----------|------------|-----------|
| **Тестирование новой стратегии** | Развернуть для всех или никого | Включить для выбранных аккаунтов |
| **Обнаружен баг** | Полный откат | Мгновенное отключение |
| **A/B тестирование** | Сложная инфраструктура | Встроенная возможность |
| **Региональный запуск** | Несколько развёртываний | Изменение конфигурации |
| **Экстренное отключение** | Развернуть старую версию | Переключить флаг |

## Что такое Feature Flags?

Feature flags (также называемые feature toggles) — это техника изменения поведения системы без изменения кода. Они позволяют:

1. **Постепенно раскатывать функции** — включать для процента пользователей
2. **A/B тестировать** — сравнивать разные реализации
3. **Kill switch** — мгновенно отключать проблемные функции
4. **Разное поведение для сред** — разные функции для dev/staging/prod
5. **Функции для конкретных пользователей** — премиум-функции, бета-тестеры

## Базовая реализация Feature Flags

```rust
use std::collections::HashMap;
use std::sync::RwLock;

/// Простое хранилище feature flags
pub struct FeatureFlags {
    flags: RwLock<HashMap<String, bool>>,
}

impl FeatureFlags {
    pub fn new() -> Self {
        FeatureFlags {
            flags: RwLock::new(HashMap::new()),
        }
    }

    /// Установить значение флага
    pub fn set(&self, feature: &str, enabled: bool) {
        let mut flags = self.flags.write().unwrap();
        flags.insert(feature.to_string(), enabled);
    }

    /// Проверить, включена ли функция
    pub fn is_enabled(&self, feature: &str) -> bool {
        let flags = self.flags.read().unwrap();
        *flags.get(feature).unwrap_or(&false)
    }

    /// Загрузить флаги из конфигурации
    pub fn load_from_config(config: &HashMap<String, bool>) -> Self {
        let flags = FeatureFlags::new();
        for (key, value) in config {
            flags.set(key, *value);
        }
        flags
    }
}

/// Торговая система с feature flags
struct TradingSystem {
    flags: FeatureFlags,
}

impl TradingSystem {
    fn new(flags: FeatureFlags) -> Self {
        TradingSystem { flags }
    }

    fn execute_trade(&self, symbol: &str, quantity: f64, price: f64) {
        println!("Исполнение сделки: {} {} @ ${:.2}", quantity, symbol, price);

        // Новая функция риск-менеджмента за флагом
        if self.flags.is_enabled("advanced_risk_check") {
            self.perform_advanced_risk_check(symbol, quantity, price);
        }

        // Новый умный роутинг за флагом
        if self.flags.is_enabled("smart_order_routing") {
            self.route_to_best_exchange(symbol, quantity, price);
        } else {
            self.route_to_default_exchange(symbol, quantity, price);
        }

        // Экспериментальные ML-предсказания
        if self.flags.is_enabled("ml_price_prediction") {
            let prediction = self.get_ml_prediction(symbol);
            println!("  ML-предсказание: ${:.2}", prediction);
        }
    }

    fn perform_advanced_risk_check(&self, symbol: &str, quantity: f64, price: f64) {
        let position_value = quantity * price;
        println!("  Расширенная проверка риска: объём позиции ${:.2}", position_value);

        if position_value > 100000.0 {
            println!("  ПРЕДУПРЕЖДЕНИЕ: Обнаружена крупная позиция!");
        }
    }

    fn route_to_best_exchange(&self, symbol: &str, _quantity: f64, _price: f64) {
        println!("  Умный роутинг: поиск лучшей биржи для {}", symbol);
    }

    fn route_to_default_exchange(&self, symbol: &str, _quantity: f64, _price: f64) {
        println!("  Стандартный роутинг: отправка на основную биржу для {}", symbol);
    }

    fn get_ml_prediction(&self, symbol: &str) -> f64 {
        // Имитация ML-предсказания
        match symbol {
            "BTCUSDT" => 51234.56,
            "ETHUSDT" => 3456.78,
            _ => 100.0,
        }
    }
}

fn main() {
    println!("=== Демо базовых Feature Flags ===\n");

    // Настройка feature flags
    let flags = FeatureFlags::new();
    flags.set("advanced_risk_check", true);
    flags.set("smart_order_routing", false);
    flags.set("ml_price_prediction", true);

    let system = TradingSystem::new(flags);

    // Выполнение сделок с разными комбинациями функций
    system.execute_trade("BTCUSDT", 0.5, 50000.0);
    println!();

    // Динамическое включение умного роутинга
    println!("--- Включаем умный роутинг ордеров ---\n");
    system.flags.set("smart_order_routing", true);

    system.execute_trade("ETHUSDT", 10.0, 3000.0);
}
```

## Feature Flags времени компиляции с Cargo

Cargo в Rust поддерживает флаги времени компиляции через `Cargo.toml`:

```toml
# Cargo.toml
[package]
name = "trading_system"
version = "0.1.0"
edition = "2021"

[features]
default = ["basic_indicators"]

# Торговые функции
basic_indicators = []
advanced_indicators = ["basic_indicators"]
ml_predictions = ["dep:ndarray"]
real_time_data = ["dep:tokio", "dep:tokio-tungstenite"]
paper_trading = []
live_trading = []
backtesting = []

# Функции производительности
simd_optimization = []
parallel_processing = ["dep:rayon"]

# Функции отладки
detailed_logging = ["dep:tracing"]
performance_metrics = []

[dependencies]
ndarray = { version = "0.15", optional = true }
tokio = { version = "1.0", features = ["full"], optional = true }
tokio-tungstenite = { version = "0.20", optional = true }
rayon = { version = "1.8", optional = true }
tracing = { version = "0.1", optional = true }
```

Использование флагов времени компиляции в коде:

```rust
/// Расчёт ценовых индикаторов с feature flags
pub struct Indicators;

impl Indicators {
    /// Простая скользящая средняя — всегда доступна
    pub fn sma(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }
        let sum: f64 = prices[prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }

    /// Экспоненциальная скользящая средняя — требует advanced_indicators
    #[cfg(feature = "advanced_indicators")]
    pub fn ema(prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = Self::sma(&prices[..period], period)?;

        for price in &prices[period..] {
            ema = (price - ema) * multiplier + ema;
        }

        Some(ema)
    }

    /// Полосы Боллинджера — требует advanced_indicators
    #[cfg(feature = "advanced_indicators")]
    pub fn bollinger_bands(prices: &[f64], period: usize, std_dev: f64) -> Option<(f64, f64, f64)> {
        if prices.len() < period {
            return None;
        }

        let slice = &prices[prices.len() - period..];
        let sma = slice.iter().sum::<f64>() / period as f64;

        let variance = slice.iter()
            .map(|p| (p - sma).powi(2))
            .sum::<f64>() / period as f64;
        let std = variance.sqrt();

        Some((sma - std_dev * std, sma, sma + std_dev * std))
    }

    /// ML-предсказание — требует ml_predictions
    #[cfg(feature = "ml_predictions")]
    pub fn predict_next_price(prices: &[f64]) -> f64 {
        // Упрощённая линейная регрессия
        let n = prices.len() as f64;
        let sum_x: f64 = (0..prices.len()).map(|i| i as f64).sum();
        let sum_y: f64 = prices.iter().sum();
        let sum_xy: f64 = prices.iter().enumerate()
            .map(|(i, p)| i as f64 * p)
            .sum();
        let sum_x2: f64 = (0..prices.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;

        slope * n + intercept
    }
}

/// Выполнение стратегии с условным поведением
pub struct Strategy {
    name: String,
}

impl Strategy {
    pub fn new(name: &str) -> Self {
        Strategy { name: name.to_string() }
    }

    pub fn analyze(&self, prices: &[f64]) {
        println!("Стратегия '{}' анализирует {} цен", self.name, prices.len());

        // Базовая SMA всегда доступна
        if let Some(sma) = Indicators::sma(prices, 20) {
            println!("  SMA(20): {:.2}", sma);
        }

        // Расширенные индикаторы только с feature
        #[cfg(feature = "advanced_indicators")]
        {
            if let Some(ema) = Indicators::ema(prices, 20) {
                println!("  EMA(20): {:.2}", ema);
            }

            if let Some((lower, middle, upper)) = Indicators::bollinger_bands(prices, 20, 2.0) {
                println!("  Полосы Боллинджера: {:.2} | {:.2} | {:.2}", lower, middle, upper);
            }
        }

        // ML-предсказания только с feature
        #[cfg(feature = "ml_predictions")]
        {
            let prediction = Indicators::predict_next_price(prices);
            println!("  ML-предсказание: {:.2}", prediction);
        }

        // Детальное логирование
        #[cfg(feature = "detailed_logging")]
        {
            println!("  [DEBUG] Анализ завершён с детальным логированием");
        }
    }

    #[cfg(feature = "paper_trading")]
    pub fn execute_paper_trade(&self, symbol: &str, side: &str, quantity: f64, price: f64) {
        println!(
            "[PAPER] {} {} {} @ ${:.2}",
            side, quantity, symbol, price
        );
    }

    #[cfg(feature = "live_trading")]
    pub fn execute_live_trade(&self, symbol: &str, side: &str, quantity: f64, price: f64) {
        println!(
            "[LIVE] Исполнение {} {} {} @ ${:.2}",
            side, quantity, symbol, price
        );
        // В реальной реализации: подключение к API биржи
    }
}

fn main() {
    println!("=== Feature Flags времени компиляции ===\n");

    // Показываем, какие features скомпилированы
    println!("Скомпилированные features:");

    #[cfg(feature = "basic_indicators")]
    println!("  - basic_indicators");

    #[cfg(feature = "advanced_indicators")]
    println!("  - advanced_indicators");

    #[cfg(feature = "ml_predictions")]
    println!("  - ml_predictions");

    #[cfg(feature = "paper_trading")]
    println!("  - paper_trading");

    #[cfg(feature = "live_trading")]
    println!("  - live_trading");

    #[cfg(feature = "parallel_processing")]
    println!("  - parallel_processing");

    println!();

    // Генерируем тестовые данные цен
    let prices: Vec<f64> = (0..100)
        .map(|i| 50000.0 + (i as f64 * 0.1).sin() * 1000.0)
        .collect();

    let strategy = Strategy::new("TrendFollower");
    strategy.analyze(&prices);

    println!();

    // Условное выполнение на основе features
    #[cfg(feature = "paper_trading")]
    strategy.execute_paper_trade("BTCUSDT", "BUY", 0.1, 50000.0);

    #[cfg(feature = "live_trading")]
    strategy.execute_live_trade("BTCUSDT", "BUY", 0.1, 50000.0);

    #[cfg(not(any(feature = "paper_trading", feature = "live_trading")))]
    println!("Режим торговли не включён. Используйте --features paper_trading или live_trading");
}
```

## Runtime Feature Flags с процентным раскатом

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Конфигурация feature flag с процентом раската
#[derive(Clone, Debug)]
pub struct FeatureConfig {
    pub enabled: bool,
    pub rollout_percentage: u8,  // 0-100
    pub allowed_users: Vec<String>,
    pub blocked_users: Vec<String>,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        FeatureConfig {
            enabled: false,
            rollout_percentage: 0,
            allowed_users: vec![],
            blocked_users: vec![],
        }
    }
}

/// Продвинутая система feature flags с таргетированием пользователей
pub struct FeatureFlagSystem {
    configs: RwLock<HashMap<String, FeatureConfig>>,
}

impl FeatureFlagSystem {
    pub fn new() -> Self {
        FeatureFlagSystem {
            configs: RwLock::new(HashMap::new()),
        }
    }

    /// Настроить функцию
    pub fn configure(&self, feature: &str, config: FeatureConfig) {
        let mut configs = self.configs.write().unwrap();
        configs.insert(feature.to_string(), config);
    }

    /// Проверить, включена ли функция для конкретного пользователя
    pub fn is_enabled_for_user(&self, feature: &str, user_id: &str) -> bool {
        let configs = self.configs.read().unwrap();

        let config = match configs.get(feature) {
            Some(c) => c,
            None => return false,
        };

        // Функция глобально отключена
        if !config.enabled {
            return false;
        }

        // Сначала проверяем заблокированных пользователей
        if config.blocked_users.contains(&user_id.to_string()) {
            return false;
        }

        // Проверяем разрешённых пользователей
        if config.allowed_users.contains(&user_id.to_string()) {
            return true;
        }

        // Процентный раскат на основе хэша пользователя
        if config.rollout_percentage >= 100 {
            return true;
        }

        let user_bucket = self.get_user_bucket(user_id, feature);
        user_bucket < config.rollout_percentage
    }

    /// Получить консистентный bucket (0-99) для комбинации пользователь/функция
    fn get_user_bucket(&self, user_id: &str, feature: &str) -> u8 {
        let mut hasher = DefaultHasher::new();
        format!("{}:{}", user_id, feature).hash(&mut hasher);
        (hasher.finish() % 100) as u8
    }

    /// Включить функцию для процента пользователей
    pub fn enable_for_percentage(&self, feature: &str, percentage: u8) {
        let mut configs = self.configs.write().unwrap();
        let config = configs.entry(feature.to_string()).or_insert(FeatureConfig::default());
        config.enabled = true;
        config.rollout_percentage = percentage.min(100);
    }

    /// Получить статистику раската (для мониторинга)
    pub fn get_rollout_stats(&self, feature: &str, all_users: &[&str]) -> (usize, usize) {
        let enabled_count = all_users
            .iter()
            .filter(|u| self.is_enabled_for_user(feature, u))
            .count();
        (enabled_count, all_users.len())
    }
}

/// Торговая стратегия с постепенным раскатом
struct TradingStrategy {
    name: String,
    flags: FeatureFlagSystem,
}

impl TradingStrategy {
    fn new(name: &str) -> Self {
        let flags = FeatureFlagSystem::new();

        // Настраиваем функции с процентами раската
        flags.configure("new_entry_logic", FeatureConfig {
            enabled: true,
            rollout_percentage: 25,  // Только 25% пользователей
            allowed_users: vec!["beta_tester_1".to_string()],
            blocked_users: vec![],
        });

        flags.configure("experimental_exit", FeatureConfig {
            enabled: true,
            rollout_percentage: 10,  // Только 10% пользователей
            allowed_users: vec![],
            blocked_users: vec!["risk_averse_user".to_string()],
        });

        TradingStrategy {
            name: name.to_string(),
            flags,
        }
    }

    fn generate_signal(&self, user_id: &str, price: f64, sma: f64) -> &str {
        println!("\nГенерация сигнала для пользователя: {}", user_id);

        let signal = if self.flags.is_enabled_for_user("new_entry_logic", user_id) {
            println!("  Используется НОВАЯ логика входа (feature включён)");
            // Новая логика: более агрессивный вход
            if price < sma * 0.98 {
                "STRONG_BUY"
            } else if price < sma {
                "BUY"
            } else if price > sma * 1.02 {
                "STRONG_SELL"
            } else if price > sma {
                "SELL"
            } else {
                "HOLD"
            }
        } else {
            println!("  Используется СТАРАЯ логика входа (feature отключён)");
            // Старая логика: консервативная
            if price < sma {
                "BUY"
            } else if price > sma {
                "SELL"
            } else {
                "HOLD"
            }
        };

        // Проверяем feature логики выхода
        if self.flags.is_enabled_for_user("experimental_exit", user_id) {
            println!("  Экспериментальная логика выхода: ВКЛЮЧЕНА");
        } else {
            println!("  Экспериментальная логика выхода: ОТКЛЮЧЕНА");
        }

        println!("  Сигнал: {}", signal);
        signal
    }
}

fn main() {
    println!("=== Feature Flags с процентным раскатом ===\n");

    let strategy = TradingStrategy::new("MomentumStrategy");

    // Симулируем разных пользователей
    let users = [
        "user_001", "user_002", "user_003", "user_004", "user_005",
        "beta_tester_1", "risk_averse_user", "user_100", "user_200",
    ];

    let price = 50000.0;
    let sma = 50500.0;

    for user in &users {
        strategy.generate_signal(user, price, sma);
    }

    // Показываем статистику раската
    println!("\n=== Статистика раската ===");
    let user_refs: Vec<&str> = users.iter().copied().collect();

    let (enabled, total) = strategy.flags.get_rollout_stats("new_entry_logic", &user_refs);
    println!("new_entry_logic: {}/{} пользователей ({:.0}%)", enabled, total, enabled as f64 / total as f64 * 100.0);

    let (enabled, total) = strategy.flags.get_rollout_stats("experimental_exit", &user_refs);
    println!("experimental_exit: {}/{} пользователей ({:.0}%)", enabled, total, enabled as f64 / total as f64 * 100.0);

    // Демонстрируем постепенный раскат
    println!("\n=== Симуляция постепенного раската ===");
    let many_users: Vec<String> = (0..1000).map(|i| format!("user_{:04}", i)).collect();
    let user_refs: Vec<&str> = many_users.iter().map(|s| s.as_str()).collect();

    for percentage in [10, 25, 50, 75, 100] {
        strategy.flags.enable_for_percentage("gradual_feature", percentage);
        let (enabled, total) = strategy.flags.get_rollout_stats("gradual_feature", &user_refs);
        println!(
            "При раскате {}%: {}/{} пользователей реально включено ({:.1}%)",
            percentage, enabled, total, enabled as f64 / total as f64 * 100.0
        );
    }
}
```

## Feature Flags на основе окружения

```rust
use std::env;
use std::collections::HashMap;

/// Тип окружения
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    fn from_env() -> Self {
        match env::var("TRADING_ENV").as_deref() {
            Ok("production") | Ok("prod") => Environment::Production,
            Ok("staging") | Ok("stage") => Environment::Staging,
            _ => Environment::Development,
        }
    }
}

/// Feature flags с учётом окружения
pub struct EnvFeatureFlags {
    environment: Environment,
    overrides: HashMap<String, bool>,
}

impl EnvFeatureFlags {
    pub fn new() -> Self {
        EnvFeatureFlags {
            environment: Environment::from_env(),
            overrides: HashMap::new(),
        }
    }

    pub fn with_environment(environment: Environment) -> Self {
        EnvFeatureFlags {
            environment,
            overrides: HashMap::new(),
        }
    }

    /// Переопределить feature flag
    pub fn set_override(&mut self, feature: &str, enabled: bool) {
        self.overrides.insert(feature.to_string(), enabled);
    }

    /// Проверить, включена ли функция на основе окружения
    pub fn is_enabled(&self, feature: &str) -> bool {
        // Сначала проверяем переопределения
        if let Some(&enabled) = self.overrides.get(feature) {
            return enabled;
        }

        // Значения по умолчанию на основе окружения
        match (feature, self.environment) {
            // Функции отладки только в development
            ("debug_logging", Environment::Development) => true,
            ("debug_logging", _) => false,

            ("verbose_errors", Environment::Development) => true,
            ("verbose_errors", Environment::Staging) => true,
            ("verbose_errors", Environment::Production) => false,

            // Paper trading в dev/staging, реальная торговля в production
            ("paper_trading", Environment::Production) => false,
            ("paper_trading", _) => true,

            ("live_trading", Environment::Production) => true,
            ("live_trading", _) => false,

            // Лимиты риска строже в production
            ("relaxed_risk_limits", Environment::Development) => true,
            ("relaxed_risk_limits", _) => false,

            // Функции производительности в production
            ("query_caching", Environment::Production) => true,
            ("query_caching", Environment::Staging) => true,
            ("query_caching", Environment::Development) => false,

            // Экспериментальные функции только в development
            ("experimental_algorithm", Environment::Development) => true,
            ("experimental_algorithm", _) => false,

            // По умолчанию: отключено
            _ => false,
        }
    }

    pub fn environment(&self) -> Environment {
        self.environment
    }
}

/// Исполнитель ордеров с поведением на основе окружения
struct OrderExecutor {
    flags: EnvFeatureFlags,
}

impl OrderExecutor {
    fn new(flags: EnvFeatureFlags) -> Self {
        OrderExecutor { flags }
    }

    fn execute_order(&self, symbol: &str, side: &str, quantity: f64, price: f64) {
        println!("\n=== Исполнение ордера ===");
        println!("Окружение: {:?}", self.flags.environment());
        println!("Ордер: {} {} {} @ ${:.2}", side, quantity, symbol, price);

        // Debug-логирование только в development
        if self.flags.is_enabled("debug_logging") {
            println!("[DEBUG] Детали ордера:");
            println!("  - Символ: {}", symbol);
            println!("  - Сторона: {}", side);
            println!("  - Количество: {}", quantity);
            println!("  - Цена: {}", price);
            println!("  - Номинал: ${:.2}", quantity * price);
        }

        // Проверки риска
        let max_order_value = if self.flags.is_enabled("relaxed_risk_limits") {
            1_000_000.0  // $1M в dev
        } else {
            100_000.0    // $100K в production
        };

        let order_value = quantity * price;
        if order_value > max_order_value {
            println!("ОРДЕР ОТКЛОНЁН: Объём ${:.2} превышает лимит ${:.2}", order_value, max_order_value);
            return;
        }

        // Исполнение в зависимости от режима торговли
        if self.flags.is_enabled("paper_trading") {
            println!("[PAPER TRADE] Симулированное исполнение");
            println!("  Цена исполнения: ${:.2}", price);
            println!("  Статус: ИСПОЛНЕН (симуляция)");
        } else if self.flags.is_enabled("live_trading") {
            println!("[LIVE TRADE] Отправка на биржу...");
            println!("  Биржа: Основная");
            println!("  Статус: В ОБРАБОТКЕ");
        } else {
            println!("[НЕТ РЕЖИМА ТОРГОВЛИ] Ордер не исполнен");
        }

        // Расширенная обработка ошибок
        if self.flags.is_enabled("verbose_errors") {
            println!("[TRACE] Обработка ордера завершена без ошибок");
        }
    }
}

fn main() {
    println!("=== Feature Flags на основе окружения ===\n");

    // Тестируем разные окружения
    for env in [Environment::Development, Environment::Staging, Environment::Production] {
        println!("\n==================================================");
        println!("Тестирование в окружении {:?}", env);
        println!("==================================================");

        let flags = EnvFeatureFlags::with_environment(env);
        let executor = OrderExecutor::new(flags);

        executor.execute_order("BTCUSDT", "BUY", 0.5, 50000.0);
    }

    // Тестируем с переопределениями
    println!("\n==================================================");
    println!("Тестирование с переопределениями в Production");
    println!("==================================================");

    let mut flags = EnvFeatureFlags::with_environment(Environment::Production);
    flags.set_override("debug_logging", true);  // Принудительный debug в production
    flags.set_override("paper_trading", true);  // Принудительный paper trading

    let executor = OrderExecutor::new(flags);
    executor.execute_order("ETHUSDT", "SELL", 10.0, 3000.0);
}
```

## Паттерн Feature Flag Service

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Feature flag с метаданными
#[derive(Clone, Debug)]
pub struct Feature {
    pub name: String,
    pub enabled: bool,
    pub description: String,
    pub owner: String,
    pub created_at: Instant,
    pub last_modified: Instant,
    pub tags: Vec<String>,
}

/// Централизованный сервис feature flags
pub struct FeatureFlagService {
    features: Arc<RwLock<HashMap<String, Feature>>>,
    cache_ttl: Duration,
    last_refresh: RwLock<Instant>,
}

impl FeatureFlagService {
    pub fn new(cache_ttl: Duration) -> Self {
        FeatureFlagService {
            features: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            last_refresh: RwLock::new(Instant::now()),
        }
    }

    /// Зарегистрировать новый feature flag
    pub fn register(&self, name: &str, enabled: bool, description: &str, owner: &str, tags: Vec<&str>) {
        let mut features = self.features.write().unwrap();
        let now = Instant::now();

        features.insert(name.to_string(), Feature {
            name: name.to_string(),
            enabled,
            description: description.to_string(),
            owner: owner.to_string(),
            created_at: now,
            last_modified: now,
            tags: tags.iter().map(|s| s.to_string()).collect(),
        });
    }

    /// Проверить, включена ли функция
    pub fn is_enabled(&self, name: &str) -> bool {
        self.maybe_refresh();

        let features = self.features.read().unwrap();
        features.get(name).map(|f| f.enabled).unwrap_or(false)
    }

    /// Переключить функцию
    pub fn toggle(&self, name: &str) -> Option<bool> {
        let mut features = self.features.write().unwrap();

        if let Some(feature) = features.get_mut(name) {
            feature.enabled = !feature.enabled;
            feature.last_modified = Instant::now();
            Some(feature.enabled)
        } else {
            None
        }
    }

    /// Включить функцию
    pub fn enable(&self, name: &str) -> bool {
        self.set_enabled(name, true)
    }

    /// Отключить функцию
    pub fn disable(&self, name: &str) -> bool {
        self.set_enabled(name, false)
    }

    fn set_enabled(&self, name: &str, enabled: bool) -> bool {
        let mut features = self.features.write().unwrap();

        if let Some(feature) = features.get_mut(name) {
            feature.enabled = enabled;
            feature.last_modified = Instant::now();
            true
        } else {
            false
        }
    }

    /// Получить все функции с определённым тегом
    pub fn get_by_tag(&self, tag: &str) -> Vec<Feature> {
        let features = self.features.read().unwrap();
        features
            .values()
            .filter(|f| f.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Получить сводку по функциям
    pub fn summary(&self) -> FeatureSummary {
        let features = self.features.read().unwrap();

        let total = features.len();
        let enabled = features.values().filter(|f| f.enabled).count();
        let by_owner = features.values().fold(HashMap::new(), |mut acc, f| {
            *acc.entry(f.owner.clone()).or_insert(0) += 1;
            acc
        });

        FeatureSummary {
            total,
            enabled,
            disabled: total - enabled,
            by_owner,
        }
    }

    /// Симуляция обновления кэша (в реальном приложении — запрос к серверу конфигурации)
    fn maybe_refresh(&self) {
        let last = *self.last_refresh.read().unwrap();
        if last.elapsed() > self.cache_ttl {
            // В реальной реализации: запрос к config-серверу
            *self.last_refresh.write().unwrap() = Instant::now();
        }
    }

    /// Получить список всех функций
    pub fn list_all(&self) -> Vec<Feature> {
        let features = self.features.read().unwrap();
        features.values().cloned().collect()
    }
}

#[derive(Debug)]
pub struct FeatureSummary {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub by_owner: HashMap<String, usize>,
}

/// Торговая платформа, использующая сервис feature flags
struct TradingPlatform {
    flags: Arc<FeatureFlagService>,
}

impl TradingPlatform {
    fn new(flags: Arc<FeatureFlagService>) -> Self {
        TradingPlatform { flags }
    }

    fn process_market_data(&self, symbol: &str, price: f64) {
        println!("\nОбработка рыночных данных: {} @ ${:.2}", symbol, price);

        if self.flags.is_enabled("real_time_alerts") {
            println!("  [ALERT] Получено обновление цены");
        }

        if self.flags.is_enabled("price_analytics") {
            println!("  [ANALYTICS] Запись цены для анализа");
        }

        if self.flags.is_enabled("ml_scoring") {
            println!("  [ML] Расчёт скоринга возможности...");
        }
    }

    fn execute_strategy(&self, symbol: &str) {
        println!("\nВыполнение стратегии для {}", symbol);

        if self.flags.is_enabled("stop_loss_v2") {
            println!("  Используется улучшенная логика стоп-лосса");
        }

        if self.flags.is_enabled("dynamic_position_sizing") {
            println!("  Используется динамический размер позиции");
        }

        if self.flags.is_enabled("multi_exchange_routing") {
            println!("  Проверка цен на разных биржах");
        }
    }
}

fn main() {
    println!("=== Паттерн Feature Flag Service ===\n");

    // Создаём сервис feature flags
    let service = Arc::new(FeatureFlagService::new(Duration::from_secs(60)));

    // Регистрируем функции
    service.register(
        "real_time_alerts",
        true,
        "Отправка оповещений о ценах в реальном времени",
        "alerts-team",
        vec!["notifications", "real-time"],
    );

    service.register(
        "price_analytics",
        true,
        "Запись ценовых данных для аналитики",
        "data-team",
        vec!["analytics", "data"],
    );

    service.register(
        "ml_scoring",
        false,
        "ML-скоринг торговых возможностей",
        "ml-team",
        vec!["ml", "experimental"],
    );

    service.register(
        "stop_loss_v2",
        true,
        "Улучшенный стоп-лосс с трейлингом",
        "trading-team",
        vec!["trading", "risk"],
    );

    service.register(
        "dynamic_position_sizing",
        false,
        "Размер позиции на основе волатильности",
        "trading-team",
        vec!["trading", "risk", "experimental"],
    );

    service.register(
        "multi_exchange_routing",
        true,
        "Роутинг ордеров на лучшую биржу",
        "execution-team",
        vec!["execution", "optimization"],
    );

    // Показываем сводку
    println!("Сводка по Feature Flags:");
    let summary = service.summary();
    println!("  Всего: {}", summary.total);
    println!("  Включено: {}", summary.enabled);
    println!("  Отключено: {}", summary.disabled);
    println!("  По владельцам: {:?}", summary.by_owner);

    // Список экспериментальных функций
    println!("\nЭкспериментальные функции:");
    for feature in service.get_by_tag("experimental") {
        println!(
            "  - {} ({}): {}",
            feature.name,
            if feature.enabled { "ВКЛ" } else { "ВЫКЛ" },
            feature.description
        );
    }

    // Создаём платформу и обрабатываем данные
    let platform = TradingPlatform::new(Arc::clone(&service));

    platform.process_market_data("BTCUSDT", 50000.0);
    platform.execute_strategy("BTCUSDT");

    // Переключаем функцию
    println!("\n--- Включаем ML-скоринг ---");
    service.enable("ml_scoring");

    platform.process_market_data("ETHUSDT", 3000.0);

    // Отключаем функцию
    println!("\n--- Отключаем оповещения в реальном времени ---");
    service.disable("real_time_alerts");

    platform.process_market_data("SOLUSDT", 100.0);
}
```

## A/B тестирование с Feature Flags

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Вариант A/B теста
#[derive(Clone, Debug)]
pub enum Variant {
    Control,
    Treatment(String),
}

/// Конфигурация A/B теста
#[derive(Clone, Debug)]
pub struct ABTest {
    pub name: String,
    pub variants: Vec<(String, u8)>,  // (название_варианта, процент)
    pub metrics: Vec<String>,
}

/// Система A/B тестирования для торговых стратегий
pub struct ABTestingSystem {
    tests: RwLock<HashMap<String, ABTest>>,
    assignments: RwLock<HashMap<(String, String), String>>,  // (тест, пользователь) -> вариант
    results: RwLock<HashMap<(String, String), Vec<f64>>>,    // (тест, вариант) -> значения метрик
}

impl ABTestingSystem {
    pub fn new() -> Self {
        ABTestingSystem {
            tests: RwLock::new(HashMap::new()),
            assignments: RwLock::new(HashMap::new()),
            results: RwLock::new(HashMap::new()),
        }
    }

    /// Создать новый A/B тест
    pub fn create_test(&self, name: &str, variants: Vec<(&str, u8)>, metrics: Vec<&str>) {
        let test = ABTest {
            name: name.to_string(),
            variants: variants.iter().map(|(n, p)| (n.to_string(), *p)).collect(),
            metrics: metrics.iter().map(|s| s.to_string()).collect(),
        };

        let mut tests = self.tests.write().unwrap();
        tests.insert(name.to_string(), test);
    }

    /// Получить вариант для пользователя (детерминированный на основе user_id)
    pub fn get_variant(&self, test_name: &str, user_id: &str) -> Option<String> {
        // Проверяем, есть ли уже назначение
        let key = (test_name.to_string(), user_id.to_string());
        {
            let assignments = self.assignments.read().unwrap();
            if let Some(variant) = assignments.get(&key) {
                return Some(variant.clone());
            }
        }

        // Получаем конфигурацию теста
        let tests = self.tests.read().unwrap();
        let test = tests.get(test_name)?;

        // Назначаем на основе хэша
        let bucket = self.get_bucket(user_id, test_name);
        let mut cumulative = 0u8;

        for (variant_name, percentage) in &test.variants {
            cumulative += percentage;
            if bucket < cumulative {
                let mut assignments = self.assignments.write().unwrap();
                assignments.insert(key, variant_name.clone());
                return Some(variant_name.clone());
            }
        }

        None
    }

    fn get_bucket(&self, user_id: &str, test_name: &str) -> u8 {
        let mut hasher = DefaultHasher::new();
        format!("{}:{}", user_id, test_name).hash(&mut hasher);
        (hasher.finish() % 100) as u8
    }

    /// Записать метрику для варианта
    pub fn record_metric(&self, test_name: &str, variant: &str, value: f64) {
        let key = (test_name.to_string(), variant.to_string());
        let mut results = self.results.write().unwrap();
        results.entry(key).or_insert_with(Vec::new).push(value);
    }

    /// Получить результаты теста
    pub fn get_results(&self, test_name: &str) -> HashMap<String, TestResults> {
        let tests = self.tests.read().unwrap();
        let results = self.results.read().unwrap();

        let test = match tests.get(test_name) {
            Some(t) => t,
            None => return HashMap::new(),
        };

        let mut variant_results = HashMap::new();

        for (variant_name, _) in &test.variants {
            let key = (test_name.to_string(), variant_name.clone());
            let values = results.get(&key).cloned().unwrap_or_default();

            let count = values.len();
            let mean = if count > 0 {
                values.iter().sum::<f64>() / count as f64
            } else {
                0.0
            };

            let std_dev = if count > 1 {
                let variance = values.iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / (count - 1) as f64;
                variance.sqrt()
            } else {
                0.0
            };

            variant_results.insert(variant_name.clone(), TestResults {
                variant: variant_name.clone(),
                count,
                mean,
                std_dev,
                values,
            });
        }

        variant_results
    }
}

#[derive(Debug)]
pub struct TestResults {
    pub variant: String,
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub values: Vec<f64>,
}

/// Торговая стратегия с A/B тестированием
struct TradingStrategyAB {
    ab_system: ABTestingSystem,
}

impl TradingStrategyAB {
    fn new() -> Self {
        let ab_system = ABTestingSystem::new();

        // Создаём A/B тесты для разных компонентов стратегии
        ab_system.create_test(
            "entry_timing",
            vec![("immediate", 50), ("wait_confirmation", 50)],
            vec!["profit_pct", "win_rate"],
        );

        ab_system.create_test(
            "position_size",
            vec![("fixed_1pct", 33), ("fixed_2pct", 33), ("dynamic", 34)],
            vec!["total_return", "max_drawdown"],
        );

        ab_system.create_test(
            "stop_loss_type",
            vec![("fixed", 50), ("trailing", 50)],
            vec!["avg_loss", "win_rate"],
        );

        TradingStrategyAB { ab_system }
    }

    fn execute_trade(&self, user_id: &str, symbol: &str, signal_strength: f64) {
        println!("\n=== Исполнение сделки для {} ===", user_id);
        println!("Символ: {}, Сигнал: {:.2}", symbol, signal_strength);

        // Получаем варианты для этого пользователя
        let entry_variant = self.ab_system.get_variant("entry_timing", user_id)
            .unwrap_or("immediate".to_string());
        let size_variant = self.ab_system.get_variant("position_size", user_id)
            .unwrap_or("fixed_1pct".to_string());
        let stop_variant = self.ab_system.get_variant("stop_loss_type", user_id)
            .unwrap_or("fixed".to_string());

        println!("A/B варианты:");
        println!("  - Вход: {}", entry_variant);
        println!("  - Размер позиции: {}", size_variant);
        println!("  - Стоп-лосс: {}", stop_variant);

        // Симулируем исполнение сделки с логикой для каждого варианта
        let entry_delay = match entry_variant.as_str() {
            "immediate" => 0,
            "wait_confirmation" => 100,
            _ => 0,
        };

        let position_pct = match size_variant.as_str() {
            "fixed_1pct" => 1.0,
            "fixed_2pct" => 2.0,
            "dynamic" => signal_strength * 2.0,
            _ => 1.0,
        };

        let stop_distance = match stop_variant.as_str() {
            "fixed" => 2.0,
            "trailing" => 1.5,
            _ => 2.0,
        };

        println!("Параметры сделки:");
        println!("  - Задержка входа: {}мс", entry_delay);
        println!("  - Позиция: {:.1}% от портфеля", position_pct);
        println!("  - Расстояние стопа: {:.1}%", stop_distance);

        // Симулируем результат и записываем метрики
        let profit_pct = (signal_strength - 0.5) * 4.0 + (rand_simple() - 0.5);
        let win = profit_pct > 0.0;

        println!("Результат: {:.2}% ({})", profit_pct, if win { "ПРИБЫЛЬ" } else { "УБЫТОК" });

        // Записываем результаты для A/B анализа
        self.ab_system.record_metric("entry_timing", &entry_variant, profit_pct);
        self.ab_system.record_metric("position_size", &size_variant, profit_pct);
        self.ab_system.record_metric("stop_loss_type", &stop_variant, if win { 1.0 } else { 0.0 });
    }

    fn print_ab_results(&self) {
        println!("\n=== Результаты A/B тестов ===\n");

        for test_name in ["entry_timing", "position_size", "stop_loss_type"] {
            println!("Тест: {}", test_name);

            let results = self.ab_system.get_results(test_name);
            for (variant, data) in &results {
                println!(
                    "  {}: n={}, среднее={:.3}, std={:.3}",
                    variant, data.count, data.mean, data.std_dev
                );
            }
            println!();
        }
    }
}

/// Простой псевдослучайный генератор для демо (не для продакшна)
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

fn main() {
    println!("=== A/B тестирование с Feature Flags ===");

    let strategy = TradingStrategyAB::new();

    // Симулируем сделки для нескольких пользователей
    let users = ["user_001", "user_002", "user_003", "user_004", "user_005",
                 "user_006", "user_007", "user_008", "user_009", "user_010"];

    for user in &users {
        let signal = 0.3 + (rand_simple() * 0.4);  // Случайный сигнал 0.3-0.7
        strategy.execute_trade(user, "BTCUSDT", signal);
    }

    // Выводим результаты A/B тестов
    strategy.print_ab_results();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Feature Flags** | Переключатели времени выполнения для включения/отключения функций |
| **Флаги времени компиляции** | Cargo features, исключающие код при компиляции |
| **Процентный раскат** | Постепенное включение функций для подмножества пользователей |
| **Флаги окружения** | Разное поведение для dev/staging/production |
| **Kill Switch** | Возможность мгновенно отключить проблемные функции |
| **A/B тестирование** | Сравнение вариантов функций с метриками |
| **Feature Service** | Централизованное управление всеми feature flags |

## Практические задания

1. **Feature Flags для контроля рисков**: Создай систему, которая:
   - Имеет несколько уровней проверки риска (базовый, стандартный, строгий)
   - Использует feature flags для включения/отключения каждого уровня
   - Позволяет быстро отключить торговлю в экстренных ситуациях
   - Логирует все изменения флагов с временными метками

2. **A/B тестирование стратегий**: Реализуй систему:
   - Тестирует две версии логики входного сигнала
   - Отслеживает win rate и прибыль для каждого варианта
   - Автоматически раскатывает выигрывающий вариант
   - Предоставляет метрики статистической значимости

3. **Развёртывание с учётом окружения**: Создай конфигурацию:
   - Использует флаги времени компиляции для dev-зависимостей
   - Использует runtime-флаги для переключения функций
   - Разные подключения к биржам для каждого окружения
   - Автоматическая синхронизация флагов с config-сервером

4. **Дашборд Feature Flags**: Создай инструмент мониторинга:
   - Выводит список всех активных feature flags
   - Показывает процент пользователей в каждом варианте
   - Отображает метрики для каждой функции
   - Позволяет переключать функции через API

## Домашнее задание

1. **Мультитенантные Feature Flags**: Реализуй систему, которая:
   - Поддерживает конфигурацию функций для каждого аккаунта
   - Разрешает премиум-функции только для платных аккаунтов
   - Имеет бета-программу с ранним доступом к функциям
   - Отслеживает использование функций по тенантам
   - Генерирует биллинг-отчёты на основе используемых функций

2. **Система Canary-развёртывания**: Создай систему, которая:
   - Автоматически увеличивает процент раската
   - Мониторит частоту ошибок и откатывает при достижении порога
   - Отправляет оповещения об аномалиях во время раската
   - Ведёт журнал аудита всех развёртываний
   - Поддерживает мгновенный откат к предыдущему состоянию

3. **Фреймворк тестирования Feature Flags**: Создай фреймворк, который:
   - Тестирует все комбинации feature flags
   - Проверяет отсутствие конфликтов между флагами
   - Генерирует отчёт покрытия для флагов
   - Валидирует зависимости флагов
   - Находит устаревшие/неиспользуемые флаги

4. **Лаборатория торговых стратегий**: Разработай систему, которая:
   - Запускает несколько вариантов стратегий одновременно
   - Использует feature flags для контроля аллокации
   - Сравнивает производительность в реальном времени
   - Автоматически продвигает лучшие варианты
   - Предоставляет детальный аналитический дашборд

## Навигация

[← Предыдущий день](../326-async-vs-threading/ru.md) | [Следующий день →](../338-*/ru.md)
