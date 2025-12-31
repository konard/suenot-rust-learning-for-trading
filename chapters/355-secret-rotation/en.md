# Day 355: Secret Rotation

## Trading Analogy

Imagine you're managing a hedge fund with multiple trading desks. Each desk has access cards to the trading floor, passwords for terminals, and API keys for exchange connections.

**Why rotate secrets? Just like changing access cards:**

| Physical Security | Digital Secret Rotation |
|------------------|------------------------|
| **Changing access cards** after employee leaves | Rotating API keys after team changes |
| **Updating safe combinations** regularly | Rotating database passwords |
| **Rotating vault keys** on schedule | Rotating encryption keys |
| **Temporary visitor badges** | Short-lived tokens with expiration |
| **Security audit triggers** | Emergency key rotation after breach |

If an access card is stolen, you don't keep using it forever — you revoke it and issue new ones. The same applies to API keys: even without a known breach, regular rotation limits the window of exposure.

## What is Secret Rotation?

Secret rotation is the practice of periodically replacing sensitive credentials (API keys, passwords, tokens, encryption keys) with new ones. This:

1. **Limits exposure window** — if a secret is leaked, the damage is time-limited
2. **Enforces good hygiene** — prevents secrets from living forever
3. **Meets compliance requirements** — many regulations require regular rotation
4. **Reduces insider threat** — departed employees can't use old credentials

## Secret Management Architecture

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Represents a secret with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Secret {
    /// The actual secret value
    value: String,
    /// When this secret was created
    created_at: u64,
    /// When this secret expires (0 = never)
    expires_at: u64,
    /// Version number for tracking
    version: u32,
    /// Whether this secret is currently active
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

/// Secret types for a trading system
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum SecretType {
    ExchangeApiKey(String),      // Exchange name
    ExchangeApiSecret(String),   // Exchange name
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

/// Secret manager with rotation support
struct SecretManager {
    /// Current active secrets
    secrets: Arc<RwLock<HashMap<SecretType, Secret>>>,
    /// Previous secret versions (for graceful rotation)
    previous_versions: Arc<RwLock<HashMap<SecretType, Vec<Secret>>>>,
    /// Rotation configuration
    rotation_config: RotationConfig,
}

#[derive(Debug, Clone)]
struct RotationConfig {
    /// How many previous versions to keep
    keep_versions: usize,
    /// Grace period after rotation (old key still works)
    grace_period: Duration,
    /// Default TTL for new secrets
    default_ttl: Duration,
}

impl Default for RotationConfig {
    fn default() -> Self {
        RotationConfig {
            keep_versions: 2,
            grace_period: Duration::from_secs(3600), // 1 hour
            default_ttl: Duration::from_secs(86400 * 30), // 30 days
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

    /// Store a new secret
    fn store(&self, secret_type: SecretType, value: String) {
        let secret = Secret::new(
            value,
            self.rotation_config.default_ttl.as_secs(),
            1,
        );

        let mut secrets = self.secrets.write().unwrap();
        secrets.insert(secret_type, secret);
    }

    /// Get the current active secret
    fn get(&self, secret_type: &SecretType) -> Option<String> {
        let secrets = self.secrets.read().unwrap();
        secrets.get(secret_type)
            .filter(|s| s.is_active && !s.is_expired())
            .map(|s| s.value.clone())
    }

    /// Rotate a secret with a new value
    fn rotate(&self, secret_type: SecretType, new_value: String) -> Result<u32, String> {
        let mut secrets = self.secrets.write().unwrap();
        let mut previous = self.previous_versions.write().unwrap();

        // Get current version
        let current_version = secrets.get(&secret_type)
            .map(|s| s.version)
            .unwrap_or(0);

        // Move current to previous versions
        if let Some(mut old_secret) = secrets.remove(&secret_type) {
            old_secret.is_active = false;

            let versions = previous.entry(secret_type.clone()).or_insert_with(Vec::new);
            versions.push(old_secret);

            // Keep only configured number of versions
            while versions.len() > self.rotation_config.keep_versions {
                versions.remove(0);
            }
        }

        // Create new secret
        let new_version = current_version + 1;
        let new_secret = Secret::new(
            new_value,
            self.rotation_config.default_ttl.as_secs(),
            new_version,
        );

        secrets.insert(secret_type, new_secret);

        Ok(new_version)
    }

    /// Check if any secrets need rotation
    fn check_rotation_needed(&self) -> Vec<(SecretType, Duration)> {
        let secrets = self.secrets.read().unwrap();
        let mut needs_rotation = Vec::new();

        for (secret_type, secret) in secrets.iter() {
            if let Some(time_left) = secret.time_until_expiry() {
                // Warn if less than 10% of TTL remains
                let ttl = self.rotation_config.default_ttl;
                if time_left < ttl / 10 {
                    needs_rotation.push((secret_type.clone(), time_left));
                }
            }
        }

        needs_rotation
    }

    /// Validate a secret (checks both current and grace-period versions)
    fn validate(&self, secret_type: &SecretType, value: &str) -> bool {
        // Check current secret
        let secrets = self.secrets.read().unwrap();
        if let Some(secret) = secrets.get(secret_type) {
            if secret.value == value && !secret.is_expired() {
                return true;
            }
        }
        drop(secrets);

        // Check previous versions within grace period
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
    println!("=== Secret Manager Demo ===\n");

    let config = RotationConfig {
        keep_versions: 2,
        grace_period: Duration::from_secs(3600),
        default_ttl: Duration::from_secs(86400 * 7), // 7 days
    };

    let manager = SecretManager::new(config);

    // Store initial secrets
    let binance_key = SecretType::ExchangeApiKey("binance".to_string());
    manager.store(binance_key.clone(), "initial_api_key_12345".to_string());

    println!("Stored initial API key");
    println!("Current key: {:?}\n", manager.get(&binance_key));

    // Rotate the key
    let new_version = manager.rotate(
        binance_key.clone(),
        "rotated_api_key_67890".to_string()
    ).unwrap();

    println!("Rotated to version {}", new_version);
    println!("Current key: {:?}", manager.get(&binance_key));

    // Old key still validates during grace period
    println!(
        "Old key valid: {}",
        manager.validate(&binance_key, "initial_api_key_12345")
    );
    println!(
        "New key valid: {}",
        manager.validate(&binance_key, "rotated_api_key_67890")
    );
}
```

## Exchange API Key Rotation

For trading systems, API keys need careful rotation to avoid disrupting active trading:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Exchange credentials with rotation support
#[derive(Debug, Clone)]
struct ExchangeCredentials {
    api_key: String,
    api_secret: String,
    created_at: u64,
    version: u32,
}

/// Trading bot with hot-swappable credentials
struct TradingBot {
    exchange: String,
    credentials: Arc<RwLock<ExchangeCredentials>>,
    /// Pending credentials waiting for verification
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

    /// Prepare new credentials for rotation
    async fn prepare_rotation(&self, new_key: &str, new_secret: &str) {
        println!("[{}] Preparing credential rotation...", self.exchange);

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

        println!("[{}] New credentials staged (v{})", self.exchange, new_version);
    }

    /// Verify new credentials work before committing
    async fn verify_pending_credentials(&self) -> Result<(), String> {
        let pending = self.pending_credentials.read().await;

        let creds = pending.as_ref()
            .ok_or("No pending credentials to verify")?;

        println!("[{}] Verifying new credentials...", self.exchange);

        // In real implementation, make a test API call
        // e.g., fetch account balance with new credentials
        let verification_result = self.test_api_call(&creds.api_key, &creds.api_secret).await;

        if verification_result {
            println!("[{}] Credentials verified successfully", self.exchange);
            Ok(())
        } else {
            println!("[{}] Credential verification failed!", self.exchange);
            Err("Credential verification failed".to_string())
        }
    }

    /// Commit the rotation (swap credentials atomically)
    async fn commit_rotation(&self) -> Result<u32, String> {
        // Verify first
        self.verify_pending_credentials().await?;

        let mut pending = self.pending_credentials.write().await;
        let new_creds = pending.take()
            .ok_or("No pending credentials")?;

        let new_version = new_creds.version;

        let mut current = self.credentials.write().await;
        *current = new_creds;

        println!("[{}] Rotation committed to v{}", self.exchange, new_version);

        Ok(new_version)
    }

    /// Rollback if rotation fails
    async fn rollback_rotation(&self) {
        let mut pending = self.pending_credentials.write().await;
        if pending.take().is_some() {
            println!("[{}] Rotation rolled back", self.exchange);
        }
    }

    /// Simulate API test call
    async fn test_api_call(&self, _api_key: &str, _api_secret: &str) -> bool {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        // In real implementation, call exchange API
        true
    }

    /// Execute a trade using current credentials
    async fn execute_trade(&self, symbol: &str, side: &str, amount: f64) -> Result<String, String> {
        let creds = self.credentials.read().await;

        println!(
            "[{}] Executing {} {} {} using API key v{}",
            self.exchange, side, amount, symbol, creds.version
        );

        // Simulate trade execution
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(format!("TRADE-{}-{}", self.exchange.to_uppercase(), creds.version))
    }
}

#[tokio::main]
async fn main() {
    println!("=== Exchange API Key Rotation Demo ===\n");

    // Initialize trading bot
    let bot = TradingBot::new(
        "binance",
        "old_api_key_abc123",
        "old_api_secret_xyz789"
    ).await;

    // Execute some trades with old credentials
    println!("--- Trading with original credentials ---");
    let trade1 = bot.execute_trade("BTCUSDT", "BUY", 0.1).await.unwrap();
    println!("Trade executed: {}\n", trade1);

    // Prepare rotation
    println!("--- Initiating key rotation ---");
    bot.prepare_rotation(
        "new_api_key_def456",
        "new_api_secret_uvw012"
    ).await;

    // Trading continues with old credentials during preparation
    let trade2 = bot.execute_trade("ETHUSDT", "SELL", 1.0).await.unwrap();
    println!("Trade executed during rotation prep: {}\n", trade2);

    // Commit rotation
    println!("--- Committing rotation ---");
    match bot.commit_rotation().await {
        Ok(version) => println!("Successfully rotated to v{}\n", version),
        Err(e) => {
            println!("Rotation failed: {}", e);
            bot.rollback_rotation().await;
            return;
        }
    }

    // Continue trading with new credentials
    println!("--- Trading with new credentials ---");
    let trade3 = bot.execute_trade("BTCUSDT", "BUY", 0.2).await.unwrap();
    println!("Trade executed: {}", trade3);
}
```

## Scheduled Rotation with Background Worker

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::time::Duration;

/// Rotation policy for different secret types
#[derive(Debug, Clone)]
struct RotationPolicy {
    /// How often to rotate
    rotation_interval: Duration,
    /// Warn this long before expiration
    warning_threshold: Duration,
    /// Minimum time between rotations
    min_rotation_interval: Duration,
}

impl RotationPolicy {
    fn for_api_keys() -> Self {
        RotationPolicy {
            rotation_interval: Duration::from_secs(86400 * 30),  // 30 days
            warning_threshold: Duration::from_secs(86400 * 7),   // 7 days before
            min_rotation_interval: Duration::from_secs(3600),    // Min 1 hour apart
        }
    }

    fn for_database_passwords() -> Self {
        RotationPolicy {
            rotation_interval: Duration::from_secs(86400 * 90),  // 90 days
            warning_threshold: Duration::from_secs(86400 * 14),  // 14 days before
            min_rotation_interval: Duration::from_secs(86400),   // Min 1 day apart
        }
    }

    fn for_jwt_keys() -> Self {
        RotationPolicy {
            rotation_interval: Duration::from_secs(86400 * 7),   // 7 days
            warning_threshold: Duration::from_secs(86400),       // 1 day before
            min_rotation_interval: Duration::from_secs(3600),    // Min 1 hour apart
        }
    }
}

/// Secret with rotation metadata
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

/// Events from the rotation worker
#[derive(Debug)]
enum RotationEvent {
    SecretRotated { name: String, new_version: u32 },
    RotationWarning { name: String, time_remaining: Duration },
    RotationFailed { name: String, error: String },
}

/// Background worker for automatic rotation
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
            check_interval: Duration::from_secs(60), // Check every minute
        }
    }

    async fn run(&self) {
        println!("[RotationWorker] Starting background rotation worker");

        loop {
            self.check_secrets().await;
            tokio::time::sleep(self.check_interval).await;
        }
    }

    async fn check_secrets(&self) {
        let mut secrets = self.secrets.write().await;

        for (name, secret) in secrets.iter_mut() {
            if secret.needs_rotation() {
                // Generate new secret value
                let new_value = self.generate_new_secret(name);

                // Perform rotation
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
        // In production, use cryptographically secure random generation
        format!("{}_{}_v{}", name, uuid_stub(), timestamp())
    }

    async fn perform_rotation(&self, name: &str, _new_value: &str) -> Result<(), String> {
        // In production, this would:
        // 1. Update the secret in vault/KMS
        // 2. Update the exchange API keys
        // 3. Verify the new credentials work
        println!("[RotationWorker] Rotating secret: {}", name);
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
}

// Helper stubs
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
    // In production, use proper random
    42
}

#[tokio::main]
async fn main() {
    println!("=== Scheduled Rotation Worker Demo ===\n");

    let secrets = Arc::new(RwLock::new(HashMap::new()));
    let (tx, mut rx) = mpsc::channel(100);

    // Add some secrets
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

    // Start worker in background
    let worker = RotationWorker::new(secrets.clone(), tx);
    let worker_handle = tokio::spawn(async move {
        worker.run().await;
    });

    // Process events
    println!("Waiting for rotation events...\n");

    // In demo, we'll just wait a bit and then exit
    tokio::select! {
        _ = async {
            while let Some(event) = rx.recv().await {
                match event {
                    RotationEvent::SecretRotated { name, new_version } => {
                        println!("[EVENT] Secret rotated: {} -> v{}", name, new_version);
                    }
                    RotationEvent::RotationWarning { name, time_remaining } => {
                        println!(
                            "[EVENT] Warning: {} expires in {:?}",
                            name, time_remaining
                        );
                    }
                    RotationEvent::RotationFailed { name, error } => {
                        println!("[EVENT] Rotation failed: {} - {}", name, error);
                    }
                }
            }
        } => {}
        _ = tokio::time::sleep(Duration::from_secs(2)) => {
            println!("\nDemo timeout - in production, this would run continuously");
        }
    }

    worker_handle.abort();
    println!("\n=== Demo Complete ===");
}
```

## Vault Integration for Secret Storage

For production systems, secrets should be stored in a dedicated vault:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Trait for secret vault backends
#[async_trait::async_trait]
trait SecretVault: Send + Sync {
    /// Read a secret
    async fn read(&self, path: &str) -> Result<String, VaultError>;

    /// Write a secret
    async fn write(&self, path: &str, value: &str) -> Result<(), VaultError>;

    /// Delete a secret
    async fn delete(&self, path: &str) -> Result<(), VaultError>;

    /// List secrets at path
    async fn list(&self, path: &str) -> Result<Vec<String>, VaultError>;

    /// Rotate a secret (generates new value)
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
            VaultError::NotFound => write!(f, "Secret not found"),
            VaultError::AccessDenied => write!(f, "Access denied"),
            VaultError::NetworkError(e) => write!(f, "Network error: {}", e),
            VaultError::InvalidPath => write!(f, "Invalid path"),
        }
    }
}

/// HashiCorp Vault-like implementation (simplified)
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

// Simplified async_trait simulation
mod async_trait {
    pub use std::marker::Sync;
    pub trait async_trait {}
}

impl HashiCorpVault {
    async fn read(&self, path: &str) -> Result<String, VaultError> {
        println!("[Vault] Reading secret: {}", path);

        let secrets = self.secrets.read().await;
        secrets.get(path)
            .cloned()
            .ok_or(VaultError::NotFound)
    }

    async fn write(&self, path: &str, value: &str) -> Result<(), VaultError> {
        println!("[Vault] Writing secret: {}", path);

        let mut secrets = self.secrets.write().await;
        secrets.insert(path.to_string(), value.to_string());
        Ok(())
    }

    async fn rotate(&self, path: &str) -> Result<String, VaultError> {
        println!("[Vault] Rotating secret: {}", path);

        // Generate new secret
        let new_value = format!("rotated_{}", generate_random_string(32));

        self.write(path, &new_value).await?;

        Ok(new_value)
    }
}

/// Trading system vault client
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

    /// Get exchange API key with caching
    async fn get_exchange_api_key(&self, exchange: &str) -> Result<String, VaultError> {
        let path = format!("trading/exchanges/{}/api_key", exchange);
        self.get_cached(&path).await
    }

    /// Get exchange API secret with caching
    async fn get_exchange_api_secret(&self, exchange: &str) -> Result<String, VaultError> {
        let path = format!("trading/exchanges/{}/api_secret", exchange);
        self.get_cached(&path).await
    }

    async fn get_cached(&self, path: &str) -> Result<String, VaultError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(path) {
                if cached.fetched_at.elapsed() < self.cache_ttl {
                    return Ok(cached.value.clone());
                }
            }
        }

        // Fetch from vault
        let value = self.vault.read(path).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(path.to_string(), CachedSecret {
                value: value.clone(),
                fetched_at: std::time::Instant::now(),
            });
        }

        Ok(value)
    }

    /// Rotate exchange credentials
    async fn rotate_exchange_credentials(&self, exchange: &str) -> Result<(String, String), VaultError> {
        let key_path = format!("trading/exchanges/{}/api_key", exchange);
        let secret_path = format!("trading/exchanges/{}/api_secret", exchange);

        println!("[TradingVault] Rotating credentials for {}", exchange);

        let new_key = self.vault.rotate(&key_path).await?;
        let new_secret = self.vault.rotate(&secret_path).await?;

        // Invalidate cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(&key_path);
            cache.remove(&secret_path);
        }

        Ok((new_key, new_secret))
    }
}

fn generate_random_string(len: usize) -> String {
    // In production, use cryptographically secure random
    (0..len).map(|_| 'x').collect()
}

#[tokio::main]
async fn main() {
    println!("=== Vault Integration Demo ===\n");

    // Initialize vault
    let vault = HashiCorpVault::new(
        "https://vault.trading.local:8200",
        "s.trading-bot-token"
    );

    // Store initial credentials
    vault.write("trading/exchanges/binance/api_key", "binance_key_123").await.unwrap();
    vault.write("trading/exchanges/binance/api_secret", "binance_secret_456").await.unwrap();

    // Create client with 5-minute cache
    let client = TradingVaultClient::new(vault, Duration::from_secs(300));

    // Get credentials
    println!("--- Fetching Credentials ---");
    let api_key = client.get_exchange_api_key("binance").await.unwrap();
    let api_secret = client.get_exchange_api_secret("binance").await.unwrap();

    println!("API Key: {}", api_key);
    println!("API Secret: {}\n", api_secret);

    // Fetch again (from cache)
    println!("--- Fetching from Cache ---");
    let _cached_key = client.get_exchange_api_key("binance").await.unwrap();
    println!("Retrieved from cache\n");

    // Rotate credentials
    println!("--- Rotating Credentials ---");
    let (new_key, new_secret) = client.rotate_exchange_credentials("binance").await.unwrap();
    println!("New API Key: {}", new_key);
    println!("New API Secret: {}", new_secret);
}
```

## Emergency Rotation

When a security incident occurs, you need to rotate secrets immediately:

```rust
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Security incident types
#[derive(Debug, Clone)]
enum SecurityIncident {
    ApiKeyLeaked { exchange: String, key_prefix: String },
    EmployeeTerminated { employee_id: String },
    SuspiciousActivity { details: String },
    SystemCompromise { affected_systems: Vec<String> },
    ThirdPartyBreach { vendor: String },
}

/// Emergency rotation coordinator
struct EmergencyRotationCoordinator {
    /// Secrets that have been rotated
    rotated_secrets: Arc<RwLock<HashSet<String>>>,
    /// Active trading bots that need credential updates
    active_bots: Arc<RwLock<Vec<String>>>,
    /// Notification callback
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

    /// Handle a security incident
    async fn handle_incident(&self, incident: SecurityIncident) {
        println!("\n[EMERGENCY] Security incident detected: {:?}", incident);
        (self.notify)("Security incident detected - initiating emergency rotation");

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

        println!("[EMERGENCY] Emergency rotation complete");
        (self.notify)("Emergency rotation completed");
    }

    async fn rotate_exchange_keys(&self, exchange: &str, _key_prefix: &str) {
        println!("[EMERGENCY] Rotating {} API keys", exchange);

        // Step 1: Generate new keys on exchange
        let (new_key, new_secret) = self.generate_exchange_keys(exchange).await;

        // Step 2: Update vault
        println!("[EMERGENCY] Updating vault with new credentials");

        // Step 3: Notify all bots to refresh credentials
        let bots = self.active_bots.read().await;
        for bot in bots.iter() {
            println!("[EMERGENCY] Notifying {} to refresh credentials", bot);
        }

        // Step 4: Revoke old keys on exchange
        println!("[EMERGENCY] Revoking old API keys on {}", exchange);

        // Step 5: Record rotation
        self.rotated_secrets.write().await
            .insert(format!("{}:api_key", exchange));

        println!("[EMERGENCY] {} keys rotated successfully", exchange);
        println!("[EMERGENCY] New key prefix: {}...", &new_key[..8]);
    }

    async fn revoke_employee_access(&self, employee_id: &str) {
        println!("[EMERGENCY] Revoking access for employee: {}", employee_id);

        // Rotate all secrets the employee had access to
        let secrets_to_rotate = vec![
            "binance:api_key",
            "bybit:api_key",
            "database:password",
        ];

        for secret in secrets_to_rotate {
            println!("[EMERGENCY] Rotating {}", secret);
            self.rotated_secrets.write().await.insert(secret.to_string());
        }

        // Revoke JWT tokens
        println!("[EMERGENCY] Invalidating all sessions for {}", employee_id);
    }

    async fn investigate_and_rotate(&self, details: &str) {
        println!("[EMERGENCY] Investigating: {}", details);

        // Analyze activity patterns
        // Determine which secrets might be compromised
        // Rotate affected secrets

        println!("[EMERGENCY] Rotating potentially compromised secrets");
    }

    async fn full_rotation(&self, affected_systems: &[String]) {
        println!("[EMERGENCY] FULL ROTATION for systems: {:?}", affected_systems);

        // This is the nuclear option - rotate everything

        // Step 1: Pause trading
        println!("[EMERGENCY] PAUSING ALL TRADING");

        // Step 2: Rotate all secrets
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
            println!("[EMERGENCY] Rotating {}", secret);
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.rotated_secrets.write().await.insert(secret.to_string());
        }

        // Step 3: Restart all services with new credentials
        println!("[EMERGENCY] Restarting all services");

        // Step 4: Resume trading
        println!("[EMERGENCY] RESUMING TRADING");
    }

    async fn rotate_vendor_secrets(&self, vendor: &str) {
        println!("[EMERGENCY] Rotating secrets related to vendor: {}", vendor);

        // Find all secrets associated with this vendor
        // Rotate them
    }

    async fn generate_exchange_keys(&self, _exchange: &str) -> (String, String) {
        // In production, this would call the exchange API
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
    println!("=== Emergency Rotation Demo ===");

    let notify = Arc::new(|msg: &str| {
        println!("[ALERT] {}", msg);
    });

    let coordinator = EmergencyRotationCoordinator::new(notify);

    // Simulate API key leak
    coordinator.handle_incident(SecurityIncident::ApiKeyLeaked {
        exchange: "binance".to_string(),
        key_prefix: "abc123".to_string(),
    }).await;

    println!("\n---\n");

    // Simulate employee termination
    coordinator.handle_incident(SecurityIncident::EmployeeTerminated {
        employee_id: "emp-456".to_string(),
    }).await;
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Secret rotation** | Periodically replacing credentials with new ones |
| **Grace period** | Time window where both old and new secrets are valid |
| **Zero-downtime rotation** | Rotating without service interruption |
| **Vault integration** | Using dedicated secret management systems |
| **Rotation policy** | Rules for when and how to rotate different secret types |
| **Emergency rotation** | Immediate rotation in response to security incidents |
| **Secret versioning** | Tracking which version of a secret is in use |

## Practical Exercises

1. **Basic Secret Manager**: Implement a secret manager that:
   - Stores API keys with expiration times
   - Automatically warns before expiration
   - Supports manual rotation
   - Keeps audit log of all rotations

2. **Exchange Credential Rotation**: Build a system that:
   - Rotates exchange API keys without stopping trading
   - Verifies new credentials before committing
   - Rolls back on failure
   - Supports multiple exchanges

3. **Scheduled Rotation Worker**: Create a background worker that:
   - Checks secret age against policy
   - Sends warnings before rotation
   - Performs automatic rotation
   - Sends notifications on completion

4. **Vault Client**: Implement a vault client that:
   - Caches secrets locally
   - Refreshes cache on rotation
   - Handles vault unavailability gracefully
   - Supports multiple secret backends

## Homework

1. **Complete Rotation System**: Build a production-ready rotation system:
   - Multiple secret types (API keys, passwords, tokens)
   - Different rotation policies per type
   - Web dashboard for monitoring
   - Slack/Telegram notifications
   - Audit logging to database
   - Metrics for rotation success rate

2. **Disaster Recovery**: Implement emergency rotation procedures:
   - Detect leaked credentials (monitor paste sites)
   - Automatic incident response
   - Trading pause during rotation
   - Post-incident report generation
   - Integration with security monitoring

3. **Multi-Exchange Key Manager**: Create a system for managing multiple exchanges:
   - Unified interface for all exchanges
   - Exchange-specific rotation procedures
   - Credential health monitoring
   - Automatic key regeneration
   - IP whitelist management

4. **Compliance Reporting**: Build compliance features:
   - Track rotation history
   - Generate compliance reports
   - Enforce rotation policies
   - Alert on policy violations
   - Integration with audit systems

## Navigation

[← Previous day](../354-production-logging/en.md) | [Next day →](../360-canary-deployments/en.md)
