# День 355: Ротация секретов

## Аналогия из трейдинга

Представь, что ты управляешь хедж-фондом с несколькими торговыми отделами. У каждого отдела есть карты доступа на торговый этаж, пароли от терминалов и API-ключи для подключения к биржам.

**Зачем ротировать секреты? Как и смена карт доступа:**

| Физическая безопасность | Ротация цифровых секретов |
|------------------------|--------------------------|
| **Смена карт доступа** после ухода сотрудника | Ротация API-ключей после изменений в команде |
| **Обновление кодов сейфа** по расписанию | Ротация паролей базы данных |
| **Ротация ключей хранилища** | Ротация ключей шифрования |
| **Временные пропуска для гостей** | Короткоживущие токены с истечением |
| **Аудит безопасности** | Экстренная ротация после взлома |

Если карта доступа украдена, ты не продолжаешь её использовать — ты аннулируешь её и выпускаешь новые. То же самое с API-ключами: даже без известной утечки регулярная ротация ограничивает окно уязвимости.

## Что такое ротация секретов?

Ротация секретов — это практика периодической замены чувствительных учётных данных (API-ключей, паролей, токенов, ключей шифрования) новыми. Это позволяет:

1. **Ограничить окно уязвимости** — если секрет утёк, ущерб ограничен по времени
2. **Обеспечить хорошую гигиену** — предотвращает вечное существование секретов
3. **Соответствовать требованиям регуляторов** — многие нормативы требуют регулярной ротации
4. **Снизить внутренние угрозы** — уволенные сотрудники не смогут использовать старые учётные данные

## Архитектура управления секретами

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Представляет секрет с метаданными
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Secret {
    /// Само значение секрета
    value: String,
    /// Когда секрет был создан
    created_at: u64,
    /// Когда секрет истекает (0 = никогда)
    expires_at: u64,
    /// Номер версии для отслеживания
    version: u32,
    /// Активен ли секрет в данный момент
    is_active: bool,
}

impl Secret {
    fn new(value: String, ttl_seconds: u64, version: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Secret {
            value,
            created_at: now,
            expires_at: if ttl_seconds > 0 { now + ttl_seconds } else { 0 },
            version,
            is_active: true,
        }
    }

    fn is_expired(&self) -> bool {
        if self.expires_at == 0 {
            return false;
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.expires_at
    }

    fn time_until_expiry(&self) -> Option<Duration> {
        if self.expires_at == 0 {
            return None;
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now >= self.expires_at {
            Some(Duration::ZERO)
        } else {
            Some(Duration::from_secs(self.expires_at - now))
        }
    }
}

/// Типы секретов для торговой системы
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum SecretType {
    ExchangeApiKey(String),      // Название биржи
    ExchangeApiSecret(String),   // Название биржи
    DatabasePassword,
    EncryptionKey,
    JwtSigningKey,
    WebhookSecret,
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretType::ExchangeApiKey(ex) => write!(f, "api_key:{}", ex),
            SecretType::ExchangeApiSecret(ex) => write!(f, "api_secret:{}", ex),
            SecretType::DatabasePassword => write!(f, "db_password"),
            SecretType::EncryptionKey => write!(f, "encryption_key"),
            SecretType::JwtSigningKey => write!(f, "jwt_key"),
            SecretType::WebhookSecret => write!(f, "webhook_secret"),
        }
    }
}

/// Менеджер секретов с поддержкой ротации
struct SecretManager {
    /// Текущие активные секреты
    secrets: Arc<RwLock<HashMap<SecretType, Secret>>>,
    /// Предыдущие версии секретов (для плавной ротации)
    previous_versions: Arc<RwLock<HashMap<SecretType, Vec<Secret>>>>,
    /// Конфигурация ротации
    rotation_config: RotationConfig,
}

#[derive(Debug, Clone)]
struct RotationConfig {
    /// Сколько предыдущих версий хранить
    keep_versions: usize,
    /// Льготный период после ротации (старый ключ ещё работает)
    grace_period: Duration,
    /// TTL по умолчанию для новых секретов
    default_ttl: Duration,
}

impl Default for RotationConfig {
    fn default() -> Self {
        RotationConfig {
            keep_versions: 2,
            grace_period: Duration::from_secs(3600), // 1 час
            default_ttl: Duration::from_secs(86400 * 30), // 30 дней
        }
    }
}

impl SecretManager {
    fn new(config: RotationConfig) -> Self {
        SecretManager {
            secrets: Arc::new(RwLock::new(HashMap::new())),
            previous_versions: Arc::new(RwLock::new(HashMap::new())),
            rotation_config: config,
        }
    }

    /// Сохранить новый секрет
    fn store(&self, secret_type: SecretType, value: String) {
        let secret = Secret::new(
            value,
            self.rotation_config.default_ttl.as_secs(),
            1,
        );

        let mut secrets = self.secrets.write().unwrap();
        secrets.insert(secret_type, secret);
    }

    /// Получить текущий активный секрет
    fn get(&self, secret_type: &SecretType) -> Option<String> {
        let secrets = self.secrets.read().unwrap();
        secrets.get(secret_type)
            .filter(|s| s.is_active && !s.is_expired())
            .map(|s| s.value.clone())
    }

    /// Ротировать секрет с новым значением
    fn rotate(&self, secret_type: SecretType, new_value: String) -> Result<u32, String> {
        let mut secrets = self.secrets.write().unwrap();
        let mut previous = self.previous_versions.write().unwrap();

        // Получить текущую версию
        let current_version = secrets.get(&secret_type)
            .map(|s| s.version)
            .unwrap_or(0);

        // Переместить текущий в предыдущие версии
        if let Some(mut old_secret) = secrets.remove(&secret_type) {
            old_secret.is_active = false;

            let versions = previous.entry(secret_type.clone()).or_insert_with(Vec::new);
            versions.push(old_secret);

            // Хранить только указанное количество версий
            while versions.len() > self.rotation_config.keep_versions {
                versions.remove(0);
            }
        }

        // Создать новый секрет
        let new_version = current_version + 1;
        let new_secret = Secret::new(
            new_value,
            self.rotation_config.default_ttl.as_secs(),
            new_version,
        );

        secrets.insert(secret_type, new_secret);

        Ok(new_version)
    }

    /// Проверить, какие секреты требуют ротации
    fn check_rotation_needed(&self) -> Vec<(SecretType, Duration)> {
        let secrets = self.secrets.read().unwrap();
        let mut needs_rotation = Vec::new();

        for (secret_type, secret) in secrets.iter() {
            if let Some(time_left) = secret.time_until_expiry() {
                // Предупреждать, если осталось менее 10% TTL
                let ttl = self.rotation_config.default_ttl;
                if time_left < ttl / 10 {
                    needs_rotation.push((secret_type.clone(), time_left));
                }
            }
        }

        needs_rotation
    }

    /// Валидировать секрет (проверяет текущий и версии в льготном периоде)
    fn validate(&self, secret_type: &SecretType, value: &str) -> bool {
        // Проверить текущий секрет
        let secrets = self.secrets.read().unwrap();
        if let Some(secret) = secrets.get(secret_type) {
            if secret.value == value && !secret.is_expired() {
                return true;
            }
        }
        drop(secrets);

        // Проверить предыдущие версии в льготном периоде
        let previous = self.previous_versions.read().unwrap();
        if let Some(versions) = previous.get(secret_type) {
            let grace_end = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() - self.rotation_config.grace_period.as_secs();

            for old_secret in versions.iter().rev() {
                if old_secret.value == value && old_secret.created_at > grace_end {
                    return true;
                }
            }
        }

        false
    }
}

fn main() {
    println!("=== Демо менеджера секретов ===\n");

    let config = RotationConfig {
        keep_versions: 2,
        grace_period: Duration::from_secs(3600),
        default_ttl: Duration::from_secs(86400 * 7), // 7 дней
    };

    let manager = SecretManager::new(config);

    // Сохранить начальные секреты
    let binance_key = SecretType::ExchangeApiKey("binance".to_string());
    manager.store(binance_key.clone(), "initial_api_key_12345".to_string());

    println!("Сохранён начальный API-ключ");
    println!("Текущий ключ: {:?}\n", manager.get(&binance_key));

    // Ротировать ключ
    let new_version = manager.rotate(
        binance_key.clone(),
        "rotated_api_key_67890".to_string()
    ).unwrap();

    println!("Ротация до версии {}", new_version);
    println!("Текущий ключ: {:?}", manager.get(&binance_key));

    // Старый ключ всё ещё валиден в льготный период
    println!(
        "Старый ключ валиден: {}",
        manager.validate(&binance_key, "initial_api_key_12345")
    );
    println!(
        "Новый ключ валиден: {}",
        manager.validate(&binance_key, "rotated_api_key_67890")
    );
}
```

## Ротация API-ключей биржи

Для торговых систем API-ключи нужно ротировать аккуратно, чтобы не нарушить активную торговлю:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Учётные данные биржи с поддержкой ротации
#[derive(Debug, Clone)]
struct ExchangeCredentials {
    api_key: String,
    api_secret: String,
    created_at: u64,
    version: u32,
}

/// Торговый бот с горячей заменой учётных данных
struct TradingBot {
    exchange: String,
    credentials: Arc<RwLock<ExchangeCredentials>>,
    /// Ожидающие верификации учётные данные
    pending_credentials: Arc<RwLock<Option<ExchangeCredentials>>>,
}

impl TradingBot {
    async fn new(exchange: &str, api_key: &str, api_secret: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        TradingBot {
            exchange: exchange.to_string(),
            credentials: Arc::new(RwLock::new(ExchangeCredentials {
                api_key: api_key.to_string(),
                api_secret: api_secret.to_string(),
                created_at: now,
                version: 1,
            })),
            pending_credentials: Arc::new(RwLock::new(None)),
        }
    }

    /// Подготовить новые учётные данные для ротации
    async fn prepare_rotation(&self, new_key: &str, new_secret: &str) {
        println!("[{}] Подготовка ротации учётных данных...", self.exchange);

        let current = self.credentials.read().await;
        let new_version = current.version + 1;
        drop(current);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let new_creds = ExchangeCredentials {
            api_key: new_key.to_string(),
            api_secret: new_secret.to_string(),
            created_at: now,
            version: new_version,
        };

        let mut pending = self.pending_credentials.write().await;
        *pending = Some(new_creds);

        println!("[{}] Новые учётные данные подготовлены (v{})", self.exchange, new_version);
    }

    /// Проверить новые учётные данные перед применением
    async fn verify_pending_credentials(&self) -> Result<(), String> {
        let pending = self.pending_credentials.read().await;

        let creds = pending.as_ref()
            .ok_or("Нет ожидающих верификации учётных данных")?;

        println!("[{}] Проверка новых учётных данных...", self.exchange);

        // В реальной реализации здесь был бы тестовый API-вызов
        let verification_result = self.test_api_call(&creds.api_key, &creds.api_secret).await;

        if verification_result {
            println!("[{}] Учётные данные успешно проверены", self.exchange);
            Ok(())
        } else {
            println!("[{}] Проверка учётных данных не пройдена!", self.exchange);
            Err("Проверка учётных данных не пройдена".to_string())
        }
    }

    /// Применить ротацию (атомарная замена учётных данных)
    async fn commit_rotation(&self) -> Result<u32, String> {
        // Сначала проверить
        self.verify_pending_credentials().await?;

        let mut pending = self.pending_credentials.write().await;
        let new_creds = pending.take()
            .ok_or("Нет ожидающих учётных данных")?;

        let new_version = new_creds.version;

        let mut current = self.credentials.write().await;
        *current = new_creds;

        println!("[{}] Ротация применена до v{}", self.exchange, new_version);

        Ok(new_version)
    }

    /// Откат, если ротация не удалась
    async fn rollback_rotation(&self) {
        let mut pending = self.pending_credentials.write().await;
        if pending.take().is_some() {
            println!("[{}] Ротация отменена", self.exchange);
        }
    }

    /// Имитация тестового API-вызова
    async fn test_api_call(&self, _api_key: &str, _api_secret: &str) -> bool {
        // Имитация сетевой задержки
        tokio::time::sleep(Duration::from_millis(100)).await;
        // В реальной реализации был бы вызов API биржи
        true
    }

    /// Выполнить сделку с текущими учётными данными
    async fn execute_trade(&self, symbol: &str, side: &str, amount: f64) -> Result<String, String> {
        let creds = self.credentials.read().await;

        println!(
            "[{}] Выполняю {} {} {} используя API-ключ v{}",
            self.exchange, side, amount, symbol, creds.version
        );

        // Имитация исполнения сделки
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(format!("TRADE-{}-{}", self.exchange.to_uppercase(), creds.version))
    }
}

#[tokio::main]
async fn main() {
    println!("=== Демо ротации API-ключей биржи ===\n");

    // Инициализация торгового бота
    let bot = TradingBot::new(
        "binance",
        "old_api_key_abc123",
        "old_api_secret_xyz789"
    ).await;

    // Выполнить несколько сделок со старыми учётными данными
    println!("--- Торговля с оригинальными учётными данными ---");
    let trade1 = bot.execute_trade("BTCUSDT", "BUY", 0.1).await.unwrap();
    println!("Сделка выполнена: {}\n", trade1);

    // Подготовить ротацию
    println!("--- Инициирование ротации ключей ---");
    bot.prepare_rotation(
        "new_api_key_def456",
        "new_api_secret_uvw012"
    ).await;

    // Торговля продолжается со старыми учётными данными во время подготовки
    let trade2 = bot.execute_trade("ETHUSDT", "SELL", 1.0).await.unwrap();
    println!("Сделка во время подготовки ротации: {}\n", trade2);

    // Применить ротацию
    println!("--- Применение ротации ---");
    match bot.commit_rotation().await {
        Ok(version) => println!("Успешная ротация до v{}\n", version),
        Err(e) => {
            println!("Ротация не удалась: {}", e);
            bot.rollback_rotation().await;
            return;
        }
    }

    // Продолжить торговлю с новыми учётными данными
    println!("--- Торговля с новыми учётными данными ---");
    let trade3 = bot.execute_trade("BTCUSDT", "BUY", 0.2).await.unwrap();
    println!("Сделка выполнена: {}", trade3);
}
```

## Плановая ротация с фоновым воркером

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::time::Duration;

/// Политика ротации для разных типов секретов
#[derive(Debug, Clone)]
struct RotationPolicy {
    /// Как часто ротировать
    rotation_interval: Duration,
    /// Предупреждать за это время до истечения
    warning_threshold: Duration,
    /// Минимальный интервал между ротациями
    min_rotation_interval: Duration,
}

impl RotationPolicy {
    fn for_api_keys() -> Self {
        RotationPolicy {
            rotation_interval: Duration::from_secs(86400 * 30),  // 30 дней
            warning_threshold: Duration::from_secs(86400 * 7),   // 7 дней до
            min_rotation_interval: Duration::from_secs(3600),    // Минимум 1 час между
        }
    }

    fn for_database_passwords() -> Self {
        RotationPolicy {
            rotation_interval: Duration::from_secs(86400 * 90),  // 90 дней
            warning_threshold: Duration::from_secs(86400 * 14),  // 14 дней до
            min_rotation_interval: Duration::from_secs(86400),   // Минимум 1 день между
        }
    }

    fn for_jwt_keys() -> Self {
        RotationPolicy {
            rotation_interval: Duration::from_secs(86400 * 7),   // 7 дней
            warning_threshold: Duration::from_secs(86400),       // 1 день до
            min_rotation_interval: Duration::from_secs(3600),    // Минимум 1 час между
        }
    }
}

/// Секрет с метаданными ротации
#[derive(Debug, Clone)]
struct ManagedSecret {
    name: String,
    value: String,
    created_at: u64,
    last_rotated: u64,
    rotation_count: u32,
    policy: RotationPolicy,
}

impl ManagedSecret {
    fn new(name: &str, value: &str, policy: RotationPolicy) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ManagedSecret {
            name: name.to_string(),
            value: value.to_string(),
            created_at: now,
            last_rotated: now,
            rotation_count: 0,
            policy,
        }
    }

    fn needs_rotation(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let age = now - self.last_rotated;
        age >= self.policy.rotation_interval.as_secs()
    }

    fn should_warn(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let age = now - self.last_rotated;
        let warn_at = self.policy.rotation_interval.as_secs()
            - self.policy.warning_threshold.as_secs();

        age >= warn_at && !self.needs_rotation()
    }

    fn time_until_rotation(&self) -> Duration {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let age = now - self.last_rotated;
        let rotation_at = self.policy.rotation_interval.as_secs();

        if age >= rotation_at {
            Duration::ZERO
        } else {
            Duration::from_secs(rotation_at - age)
        }
    }

    fn rotate(&mut self, new_value: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.value = new_value;
        self.last_rotated = now;
        self.rotation_count += 1;
    }
}

/// События от воркера ротации
#[derive(Debug)]
enum RotationEvent {
    SecretRotated { name: String, new_version: u32 },
    RotationWarning { name: String, time_remaining: Duration },
    RotationFailed { name: String, error: String },
}

/// Фоновый воркер для автоматической ротации
struct RotationWorker {
    secrets: Arc<RwLock<HashMap<String, ManagedSecret>>>,
    event_tx: mpsc::Sender<RotationEvent>,
    check_interval: Duration,
}

impl RotationWorker {
    fn new(
        secrets: Arc<RwLock<HashMap<String, ManagedSecret>>>,
        event_tx: mpsc::Sender<RotationEvent>,
    ) -> Self {
        RotationWorker {
            secrets,
            event_tx,
            check_interval: Duration::from_secs(60), // Проверка каждую минуту
        }
    }

    async fn run(&self) {
        println!("[RotationWorker] Запуск фонового воркера ротации");

        loop {
            self.check_secrets().await;
            tokio::time::sleep(self.check_interval).await;
        }
    }

    async fn check_secrets(&self) {
        let mut secrets = self.secrets.write().await;

        for (name, secret) in secrets.iter_mut() {
            if secret.needs_rotation() {
                // Сгенерировать новое значение секрета
                let new_value = self.generate_new_secret(name);

                // Выполнить ротацию
                match self.perform_rotation(name, &new_value).await {
                    Ok(()) => {
                        secret.rotate(new_value);
                        let _ = self.event_tx.send(RotationEvent::SecretRotated {
                            name: name.clone(),
                            new_version: secret.rotation_count,
                        }).await;
                    }
                    Err(e) => {
                        let _ = self.event_tx.send(RotationEvent::RotationFailed {
                            name: name.clone(),
                            error: e,
                        }).await;
                    }
                }
            } else if secret.should_warn() {
                let _ = self.event_tx.send(RotationEvent::RotationWarning {
                    name: name.clone(),
                    time_remaining: secret.time_until_rotation(),
                }).await;
            }
        }
    }

    fn generate_new_secret(&self, name: &str) -> String {
        // В продакшене использовать криптографически безопасную генерацию
        format!("{}_{}_v{}", name, uuid_stub(), timestamp())
    }

    async fn perform_rotation(&self, name: &str, _new_value: &str) -> Result<(), String> {
        // В продакшене здесь было бы:
        // 1. Обновление секрета в vault/KMS
        // 2. Обновление API-ключей на бирже
        // 3. Проверка работоспособности новых учётных данных
        println!("[RotationWorker] Ротация секрета: {}", name);
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
}

// Вспомогательные функции
fn uuid_stub() -> String {
    format!("{:08x}", rand_stub())
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn rand_stub() -> u32 {
    // В продакшене использовать настоящий random
    42
}

#[tokio::main]
async fn main() {
    println!("=== Демо воркера плановой ротации ===\n");

    let secrets = Arc::new(RwLock::new(HashMap::new()));
    let (tx, mut rx) = mpsc::channel(100);

    // Добавить секреты
    {
        let mut s = secrets.write().await;
        s.insert(
            "binance_api_key".to_string(),
            ManagedSecret::new(
                "binance_api_key",
                "initial_key",
                RotationPolicy::for_api_keys()
            )
        );
        s.insert(
            "db_password".to_string(),
            ManagedSecret::new(
                "db_password",
                "initial_password",
                RotationPolicy::for_database_passwords()
            )
        );
    }

    // Запустить воркер в фоне
    let worker = RotationWorker::new(secrets.clone(), tx);
    let worker_handle = tokio::spawn(async move {
        worker.run().await;
    });

    // Обработка событий
    println!("Ожидание событий ротации...\n");

    // В демо мы просто подождём немного и выйдем
    tokio::select! {
        _ = async {
            while let Some(event) = rx.recv().await {
                match event {
                    RotationEvent::SecretRotated { name, new_version } => {
                        println!("[СОБЫТИЕ] Секрет ротирован: {} -> v{}", name, new_version);
                    }
                    RotationEvent::RotationWarning { name, time_remaining } => {
                        println!(
                            "[СОБЫТИЕ] Предупреждение: {} истекает через {:?}",
                            name, time_remaining
                        );
                    }
                    RotationEvent::RotationFailed { name, error } => {
                        println!("[СОБЫТИЕ] Ротация не удалась: {} - {}", name, error);
                    }
                }
            }
        } => {}
        _ = tokio::time::sleep(Duration::from_secs(2)) => {
            println!("\nТаймаут демо - в продакшене это работало бы непрерывно");
        }
    }

    worker_handle.abort();
    println!("\n=== Демо завершено ===");
}
```

## Интеграция с Vault для хранения секретов

Для продакшен-систем секреты должны храниться в специализированном хранилище:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Трейт для бэкендов хранения секретов
#[async_trait::async_trait]
trait SecretVault: Send + Sync {
    /// Прочитать секрет
    async fn read(&self, path: &str) -> Result<String, VaultError>;

    /// Записать секрет
    async fn write(&self, path: &str, value: &str) -> Result<(), VaultError>;

    /// Удалить секрет
    async fn delete(&self, path: &str) -> Result<(), VaultError>;

    /// Список секретов по пути
    async fn list(&self, path: &str) -> Result<Vec<String>, VaultError>;

    /// Ротировать секрет (генерирует новое значение)
    async fn rotate(&self, path: &str) -> Result<String, VaultError>;
}

#[derive(Debug)]
enum VaultError {
    NotFound,
    AccessDenied,
    NetworkError(String),
    InvalidPath,
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::NotFound => write!(f, "Секрет не найден"),
            VaultError::AccessDenied => write!(f, "Доступ запрещён"),
            VaultError::NetworkError(e) => write!(f, "Сетевая ошибка: {}", e),
            VaultError::InvalidPath => write!(f, "Неверный путь"),
        }
    }
}

/// Реализация HashiCorp Vault (упрощённая)
struct HashiCorpVault {
    base_url: String,
    token: String,
    secrets: Arc<RwLock<HashMap<String, String>>>,
}

impl HashiCorpVault {
    fn new(base_url: &str, token: &str) -> Self {
        HashiCorpVault {
            base_url: base_url.to_string(),
            token: token.to_string(),
            secrets: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// Упрощённая имитация async_trait
mod async_trait {
    pub use std::marker::Sync;
    pub trait async_trait {}
}

impl HashiCorpVault {
    async fn read(&self, path: &str) -> Result<String, VaultError> {
        println!("[Vault] Чтение секрета: {}", path);

        let secrets = self.secrets.read().await;
        secrets.get(path)
            .cloned()
            .ok_or(VaultError::NotFound)
    }

    async fn write(&self, path: &str, value: &str) -> Result<(), VaultError> {
        println!("[Vault] Запись секрета: {}", path);

        let mut secrets = self.secrets.write().await;
        secrets.insert(path.to_string(), value.to_string());
        Ok(())
    }

    async fn rotate(&self, path: &str) -> Result<String, VaultError> {
        println!("[Vault] Ротация секрета: {}", path);

        // Сгенерировать новый секрет
        let new_value = format!("rotated_{}", generate_random_string(32));

        self.write(path, &new_value).await?;

        Ok(new_value)
    }
}

/// Клиент vault для торговой системы
struct TradingVaultClient {
    vault: HashiCorpVault,
    cache: Arc<RwLock<HashMap<String, CachedSecret>>>,
    cache_ttl: Duration,
}

#[derive(Clone)]
struct CachedSecret {
    value: String,
    fetched_at: std::time::Instant,
}

impl TradingVaultClient {
    fn new(vault: HashiCorpVault, cache_ttl: Duration) -> Self {
        TradingVaultClient {
            vault,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }

    /// Получить API-ключ биржи с кэшированием
    async fn get_exchange_api_key(&self, exchange: &str) -> Result<String, VaultError> {
        let path = format!("trading/exchanges/{}/api_key", exchange);
        self.get_cached(&path).await
    }

    /// Получить API-секрет биржи с кэшированием
    async fn get_exchange_api_secret(&self, exchange: &str) -> Result<String, VaultError> {
        let path = format!("trading/exchanges/{}/api_secret", exchange);
        self.get_cached(&path).await
    }

    async fn get_cached(&self, path: &str) -> Result<String, VaultError> {
        // Сначала проверить кэш
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(path) {
                if cached.fetched_at.elapsed() < self.cache_ttl {
                    return Ok(cached.value.clone());
                }
            }
        }

        // Получить из vault
        let value = self.vault.read(path).await?;

        // Обновить кэш
        {
            let mut cache = self.cache.write().await;
            cache.insert(path.to_string(), CachedSecret {
                value: value.clone(),
                fetched_at: std::time::Instant::now(),
            });
        }

        Ok(value)
    }

    /// Ротировать учётные данные биржи
    async fn rotate_exchange_credentials(&self, exchange: &str) -> Result<(String, String), VaultError> {
        let key_path = format!("trading/exchanges/{}/api_key", exchange);
        let secret_path = format!("trading/exchanges/{}/api_secret", exchange);

        println!("[TradingVault] Ротация учётных данных для {}", exchange);

        let new_key = self.vault.rotate(&key_path).await?;
        let new_secret = self.vault.rotate(&secret_path).await?;

        // Инвалидировать кэш
        {
            let mut cache = self.cache.write().await;
            cache.remove(&key_path);
            cache.remove(&secret_path);
        }

        Ok((new_key, new_secret))
    }
}

fn generate_random_string(len: usize) -> String {
    // В продакшене использовать криптографически безопасную генерацию
    (0..len).map(|_| 'x').collect()
}

#[tokio::main]
async fn main() {
    println!("=== Демо интеграции с Vault ===\n");

    // Инициализация vault
    let vault = HashiCorpVault::new(
        "https://vault.trading.local:8200",
        "s.trading-bot-token"
    );

    // Сохранить начальные учётные данные
    vault.write("trading/exchanges/binance/api_key", "binance_key_123").await.unwrap();
    vault.write("trading/exchanges/binance/api_secret", "binance_secret_456").await.unwrap();

    // Создать клиент с 5-минутным кэшем
    let client = TradingVaultClient::new(vault, Duration::from_secs(300));

    // Получить учётные данные
    println!("--- Получение учётных данных ---");
    let api_key = client.get_exchange_api_key("binance").await.unwrap();
    let api_secret = client.get_exchange_api_secret("binance").await.unwrap();

    println!("API Key: {}", api_key);
    println!("API Secret: {}\n", api_secret);

    // Получить снова (из кэша)
    println!("--- Получение из кэша ---");
    let _cached_key = client.get_exchange_api_key("binance").await.unwrap();
    println!("Получено из кэша\n");

    // Ротировать учётные данные
    println!("--- Ротация учётных данных ---");
    let (new_key, new_secret) = client.rotate_exchange_credentials("binance").await.unwrap();
    println!("Новый API Key: {}", new_key);
    println!("Новый API Secret: {}", new_secret);
}
```

## Экстренная ротация

Когда происходит инцидент безопасности, необходимо немедленно ротировать секреты:

```rust
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Типы инцидентов безопасности
#[derive(Debug, Clone)]
enum SecurityIncident {
    ApiKeyLeaked { exchange: String, key_prefix: String },
    EmployeeTerminated { employee_id: String },
    SuspiciousActivity { details: String },
    SystemCompromise { affected_systems: Vec<String> },
    ThirdPartyBreach { vendor: String },
}

/// Координатор экстренной ротации
struct EmergencyRotationCoordinator {
    /// Секреты, которые были ротированы
    rotated_secrets: Arc<RwLock<HashSet<String>>>,
    /// Активные торговые боты, которым нужно обновить учётные данные
    active_bots: Arc<RwLock<Vec<String>>>,
    /// Callback для уведомлений
    notify: Arc<dyn Fn(&str) + Send + Sync>,
}

impl EmergencyRotationCoordinator {
    fn new(notify: Arc<dyn Fn(&str) + Send + Sync>) -> Self {
        EmergencyRotationCoordinator {
            rotated_secrets: Arc::new(RwLock::new(HashSet::new())),
            active_bots: Arc::new(RwLock::new(vec![
                "bot-btc-1".to_string(),
                "bot-eth-1".to_string(),
                "bot-multi-1".to_string(),
            ])),
            notify,
        }
    }

    /// Обработать инцидент безопасности
    async fn handle_incident(&self, incident: SecurityIncident) {
        println!("\n[ЭКСТРЕННО] Обнаружен инцидент безопасности: {:?}", incident);
        (self.notify)("Обнаружен инцидент безопасности - запуск экстренной ротации");

        match incident {
            SecurityIncident::ApiKeyLeaked { exchange, key_prefix } => {
                self.rotate_exchange_keys(&exchange, &key_prefix).await;
            }
            SecurityIncident::EmployeeTerminated { employee_id } => {
                self.revoke_employee_access(&employee_id).await;
            }
            SecurityIncident::SuspiciousActivity { details } => {
                self.investigate_and_rotate(&details).await;
            }
            SecurityIncident::SystemCompromise { affected_systems } => {
                self.full_rotation(&affected_systems).await;
            }
            SecurityIncident::ThirdPartyBreach { vendor } => {
                self.rotate_vendor_secrets(&vendor).await;
            }
        }

        println!("[ЭКСТРЕННО] Экстренная ротация завершена");
        (self.notify)("Экстренная ротация завершена");
    }

    async fn rotate_exchange_keys(&self, exchange: &str, _key_prefix: &str) {
        println!("[ЭКСТРЕННО] Ротация API-ключей {}", exchange);

        // Шаг 1: Сгенерировать новые ключи на бирже
        let (new_key, new_secret) = self.generate_exchange_keys(exchange).await;

        // Шаг 2: Обновить vault
        println!("[ЭКСТРЕННО] Обновление vault с новыми учётными данными");

        // Шаг 3: Уведомить все боты об обновлении учётных данных
        let bots = self.active_bots.read().await;
        for bot in bots.iter() {
            println!("[ЭКСТРЕННО] Уведомление {} об обновлении учётных данных", bot);
        }

        // Шаг 4: Отозвать старые ключи на бирже
        println!("[ЭКСТРЕННО] Отзыв старых API-ключей на {}", exchange);

        // Шаг 5: Записать ротацию
        self.rotated_secrets.write().await
            .insert(format!("{}:api_key", exchange));

        println!("[ЭКСТРЕННО] Ключи {} успешно ротированы", exchange);
        println!("[ЭКСТРЕННО] Префикс нового ключа: {}...", &new_key[..8]);
    }

    async fn revoke_employee_access(&self, employee_id: &str) {
        println!("[ЭКСТРЕННО] Отзыв доступа для сотрудника: {}", employee_id);

        // Ротировать все секреты, к которым был доступ у сотрудника
        let secrets_to_rotate = vec![
            "binance:api_key",
            "bybit:api_key",
            "database:password",
        ];

        for secret in secrets_to_rotate {
            println!("[ЭКСТРЕННО] Ротация {}", secret);
            self.rotated_secrets.write().await.insert(secret.to_string());
        }

        // Отозвать JWT-токены
        println!("[ЭКСТРЕННО] Инвалидация всех сессий для {}", employee_id);
    }

    async fn investigate_and_rotate(&self, details: &str) {
        println!("[ЭКСТРЕННО] Расследование: {}", details);

        // Анализ паттернов активности
        // Определить, какие секреты могли быть скомпрометированы
        // Ротировать затронутые секреты

        println!("[ЭКСТРЕННО] Ротация потенциально скомпрометированных секретов");
    }

    async fn full_rotation(&self, affected_systems: &[String]) {
        println!("[ЭКСТРЕННО] ПОЛНАЯ РОТАЦИЯ для систем: {:?}", affected_systems);

        // Это ядерный вариант - ротировать всё

        // Шаг 1: Остановить торговлю
        println!("[ЭКСТРЕННО] ОСТАНОВКА ВСЕЙ ТОРГОВЛИ");

        // Шаг 2: Ротировать все секреты
        let all_secrets = vec![
            "binance:api_key",
            "binance:api_secret",
            "bybit:api_key",
            "bybit:api_secret",
            "database:password",
            "redis:password",
            "jwt:signing_key",
        ];

        for secret in all_secrets {
            println!("[ЭКСТРЕННО] Ротация {}", secret);
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.rotated_secrets.write().await.insert(secret.to_string());
        }

        // Шаг 3: Перезапустить все сервисы с новыми учётными данными
        println!("[ЭКСТРЕННО] Перезапуск всех сервисов");

        // Шаг 4: Возобновить торговлю
        println!("[ЭКСТРЕННО] ВОЗОБНОВЛЕНИЕ ТОРГОВЛИ");
    }

    async fn rotate_vendor_secrets(&self, vendor: &str) {
        println!("[ЭКСТРЕННО] Ротация секретов, связанных с вендором: {}", vendor);

        // Найти все секреты, связанные с этим вендором
        // Ротировать их
    }

    async fn generate_exchange_keys(&self, _exchange: &str) -> (String, String) {
        // В продакшене здесь был бы вызов API биржи
        tokio::time::sleep(Duration::from_millis(200)).await;
        (
            format!("new_key_{}", rand_stub()),
            format!("new_secret_{}", rand_stub()),
        )
    }
}

fn rand_stub() -> u32 {
    42
}

#[tokio::main]
async fn main() {
    println!("=== Демо экстренной ротации ===");

    let notify = Arc::new(|msg: &str| {
        println!("[АЛЕРТ] {}", msg);
    });

    let coordinator = EmergencyRotationCoordinator::new(notify);

    // Имитация утечки API-ключа
    coordinator.handle_incident(SecurityIncident::ApiKeyLeaked {
        exchange: "binance".to_string(),
        key_prefix: "abc123".to_string(),
    }).await;

    println!("\n---\n");

    // Имитация увольнения сотрудника
    coordinator.handle_incident(SecurityIncident::EmployeeTerminated {
        employee_id: "emp-456".to_string(),
    }).await;
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Ротация секретов** | Периодическая замена учётных данных новыми |
| **Льготный период** | Временное окно, когда работают и старый, и новый секреты |
| **Ротация без простоя** | Ротация без прерывания работы сервиса |
| **Интеграция с vault** | Использование специализированных систем управления секретами |
| **Политика ротации** | Правила, когда и как ротировать разные типы секретов |
| **Экстренная ротация** | Немедленная ротация в ответ на инциденты безопасности |
| **Версионирование секретов** | Отслеживание, какая версия секрета используется |

## Практические задания

1. **Базовый менеджер секретов**: Реализуй менеджер секретов, который:
   - Хранит API-ключи со сроком действия
   - Автоматически предупреждает перед истечением
   - Поддерживает ручную ротацию
   - Ведёт аудит-лог всех ротаций

2. **Ротация учётных данных биржи**: Построй систему, которая:
   - Ротирует API-ключи биржи без остановки торговли
   - Проверяет новые учётные данные перед применением
   - Откатывается при ошибке
   - Поддерживает несколько бирж

3. **Воркер плановой ротации**: Создай фоновый воркер, который:
   - Проверяет возраст секретов относительно политики
   - Отправляет предупреждения перед ротацией
   - Выполняет автоматическую ротацию
   - Отправляет уведомления о завершении

4. **Клиент Vault**: Реализуй клиент vault, который:
   - Кэширует секреты локально
   - Обновляет кэш при ротации
   - Корректно обрабатывает недоступность vault
   - Поддерживает несколько бэкендов секретов

## Домашнее задание

1. **Полноценная система ротации**: Построй production-ready систему ротации:
   - Разные типы секретов (API-ключи, пароли, токены)
   - Разные политики ротации для каждого типа
   - Веб-дашборд для мониторинга
   - Уведомления в Slack/Telegram
   - Аудит-логирование в базу данных
   - Метрики успешности ротации

2. **Аварийное восстановление**: Реализуй процедуры экстренной ротации:
   - Обнаружение утёкших учётных данных (мониторинг paste-сайтов)
   - Автоматический ответ на инцидент
   - Пауза торговли во время ротации
   - Генерация отчёта после инцидента
   - Интеграция с мониторингом безопасности

3. **Менеджер ключей для нескольких бирж**: Создай систему для управления несколькими биржами:
   - Унифицированный интерфейс для всех бирж
   - Специфичные для биржи процедуры ротации
   - Мониторинг здоровья учётных данных
   - Автоматическая перегенерация ключей
   - Управление IP-белыми списками

4. **Комплаенс-отчётность**: Реализуй функции соответствия требованиям:
   - Отслеживание истории ротации
   - Генерация комплаенс-отчётов
   - Принудительное соблюдение политик ротации
   - Алерты при нарушении политик
   - Интеграция с системами аудита

## Навигация

[← Предыдущий день](../354-production-logging/ru.md) | [Следующий день →](../360-canary-deployments/ru.md)
