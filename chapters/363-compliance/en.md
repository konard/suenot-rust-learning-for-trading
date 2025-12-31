# Day 363: Compliance

## Trading Analogy

Imagine you're managing an investment fund. Every day regulators may request:
- **Where did the money come from?** (AML — Anti-Money Laundering)
- **Who are your clients?** (KYC — Know Your Customer)
- **Why did you buy these assets?** (Trade justification)
- **How much do you pay in fees?** (Cost transparency)

**Compliance in trading is like an internal auditor:**

| Regulatory Requirements | Compliance in Code |
|------------------------|-------------------|
| **Trade audit** | Logging all operations with immutable history |
| **Position limits** | Validation through types and runtime checks |
| **Reporting** | Automatic report generation |
| **Identification** | Type-safe client and account IDs |
| **Separation of duties** | Access control through types and modules |

Non-compliance can cost millions in fines. In code — it means bugs, vulnerabilities, and data loss.

## Types for Compliance

Using Rust's type system to guarantee compliance:

```rust
use std::marker::PhantomData;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Verification state markers
#[derive(Debug, Clone)]
pub struct Unverified;
#[derive(Debug, Clone)]
pub struct KycPending;
#[derive(Debug, Clone)]
pub struct KycVerified;
#[derive(Debug, Clone)]
pub struct AmlCleared;

/// Client with type-safe verification status
#[derive(Debug, Clone)]
pub struct Client<Status> {
    id: ClientId,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
    _status: PhantomData<Status>,
}

/// Type-safe client ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(String);

impl ClientId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl<Status> Client<Status> {
    pub fn id(&self) -> &ClientId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Client<Unverified> {
    /// Create a new unverified client
    pub fn new(id: ClientId, name: String, email: String) -> Self {
        Client {
            id,
            name,
            email,
            created_at: Utc::now(),
            _status: PhantomData,
        }
    }

    /// Start KYC verification process
    pub fn start_kyc(self) -> Client<KycPending> {
        println!("KYC process started for client: {}", self.id.0);
        Client {
            id: self.id,
            name: self.name,
            email: self.email,
            created_at: self.created_at,
            _status: PhantomData,
        }
    }
}

impl Client<KycPending> {
    /// Complete KYC verification
    pub fn complete_kyc(self, documents_verified: bool) -> Result<Client<KycVerified>, String> {
        if !documents_verified {
            return Err("Documents failed verification".to_string());
        }

        println!("KYC verification completed for client: {}", self.id.0);
        Ok(Client {
            id: self.id,
            name: self.name,
            email: self.email,
            created_at: self.created_at,
            _status: PhantomData,
        })
    }
}

impl Client<KycVerified> {
    /// Pass AML check
    pub fn clear_aml(self, risk_score: u8) -> Result<Client<AmlCleared>, String> {
        if risk_score > 70 {
            return Err(format!("High AML risk: {}", risk_score));
        }

        println!("AML check passed for client: {}", self.id.0);
        Ok(Client {
            id: self.id,
            name: self.name,
            email: self.email,
            created_at: self.created_at,
            _status: PhantomData,
        })
    }
}

/// Only fully verified clients can trade
impl Client<AmlCleared> {
    pub fn place_order(&self, symbol: &str, quantity: f64, price: f64) -> Order {
        println!("Client {} placing order: {} {} @ {}",
                 self.id.0, symbol, quantity, price);
        Order {
            client_id: self.id.clone(),
            symbol: symbol.to_string(),
            quantity,
            price,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Order {
    client_id: ClientId,
    symbol: String,
    quantity: f64,
    price: f64,
    timestamp: DateTime<Utc>,
}

fn main() {
    println!("=== Type-safe Compliance ===\n");

    // Create an unverified client
    let client = Client::<Unverified>::new(
        ClientId::new("CLT-001"),
        "John Smith".to_string(),
        "john@example.com".to_string()
    );

    // Start KYC
    let client = client.start_kyc();

    // Complete KYC
    let client = client.complete_kyc(true)
        .expect("KYC should pass");

    // Pass AML check
    let client = client.clear_aml(25)
        .expect("AML should pass");

    // Now the client can trade
    let order = client.place_order("BTCUSDT", 0.5, 50000.0);
    println!("\nOrder created: {:?}", order);

    // Compiler won't allow unverified clients to trade:
    // let unverified = Client::<Unverified>::new(...);
    // unverified.place_order(...); // Compile error!
}
```

## Audit and Immutable History

All operations must be recorded for audit:

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuditEvent {
    ClientCreated {
        client_id: String,
        timestamp: DateTime<Utc>,
    },
    KycCompleted {
        client_id: String,
        documents: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    OrderPlaced {
        client_id: String,
        order_id: String,
        symbol: String,
        side: String,
        quantity: f64,
        price: f64,
        timestamp: DateTime<Utc>,
    },
    OrderFilled {
        order_id: String,
        fill_price: f64,
        fill_quantity: f64,
        timestamp: DateTime<Utc>,
    },
    FundsTransferred {
        from_account: String,
        to_account: String,
        amount: f64,
        currency: String,
        timestamp: DateTime<Utc>,
    },
    RiskLimitBreached {
        client_id: String,
        limit_type: String,
        current_value: f64,
        max_value: f64,
        timestamp: DateTime<Utc>,
    },
}

/// Audit log entry with hash for integrity
#[derive(Debug, Clone, Serialize)]
pub struct AuditRecord {
    pub sequence: u64,
    pub event: AuditEvent,
    pub previous_hash: String,
    pub hash: String,
}

impl AuditRecord {
    fn calculate_hash(sequence: u64, event: &AuditEvent, previous_hash: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(sequence.to_be_bytes());
        hasher.update(serde_json::to_string(event).unwrap_or_default());
        hasher.update(previous_hash);
        format!("{:x}", hasher.finalize())
    }
}

/// Immutable audit log
pub struct AuditLog {
    records: RwLock<Vec<AuditRecord>>,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog {
            records: RwLock::new(Vec::new()),
        }
    }

    /// Add event to the log
    pub fn log(&self, event: AuditEvent) -> AuditRecord {
        let mut records = self.records.write().unwrap();

        let sequence = records.len() as u64;
        let previous_hash = records.last()
            .map(|r| r.hash.clone())
            .unwrap_or_else(|| "genesis".to_string());

        let hash = AuditRecord::calculate_hash(sequence, &event, &previous_hash);

        let record = AuditRecord {
            sequence,
            event,
            previous_hash,
            hash,
        };

        records.push(record.clone());
        record
    }

    /// Verify log integrity
    pub fn verify_integrity(&self) -> Result<(), String> {
        let records = self.records.read().unwrap();

        for (i, record) in records.iter().enumerate() {
            let expected_previous = if i == 0 {
                "genesis".to_string()
            } else {
                records[i - 1].hash.clone()
            };

            if record.previous_hash != expected_previous {
                return Err(format!(
                    "Integrity violation in record {}: invalid previous_hash",
                    record.sequence
                ));
            }

            let calculated_hash = AuditRecord::calculate_hash(
                record.sequence,
                &record.event,
                &record.previous_hash
            );

            if record.hash != calculated_hash {
                return Err(format!(
                    "Integrity violation in record {}: invalid hash",
                    record.sequence
                ));
            }
        }

        Ok(())
    }

    /// Get all records for a specific client
    pub fn get_client_history(&self, client_id: &str) -> Vec<AuditRecord> {
        let records = self.records.read().unwrap();
        records.iter()
            .filter(|r| {
                match &r.event {
                    AuditEvent::ClientCreated { client_id: id, .. } => id == client_id,
                    AuditEvent::KycCompleted { client_id: id, .. } => id == client_id,
                    AuditEvent::OrderPlaced { client_id: id, .. } => id == client_id,
                    AuditEvent::RiskLimitBreached { client_id: id, .. } => id == client_id,
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }

    /// Export for regulator
    pub fn export_for_regulator(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>
    ) -> Vec<AuditRecord> {
        let records = self.records.read().unwrap();
        records.iter()
            .filter(|r| {
                let timestamp = match &r.event {
                    AuditEvent::ClientCreated { timestamp, .. } => timestamp,
                    AuditEvent::KycCompleted { timestamp, .. } => timestamp,
                    AuditEvent::OrderPlaced { timestamp, .. } => timestamp,
                    AuditEvent::OrderFilled { timestamp, .. } => timestamp,
                    AuditEvent::FundsTransferred { timestamp, .. } => timestamp,
                    AuditEvent::RiskLimitBreached { timestamp, .. } => timestamp,
                };
                *timestamp >= from && *timestamp <= to
            })
            .cloned()
            .collect()
    }
}

fn main() {
    println!("=== Audit Log with Integrity Verification ===\n");

    let audit_log = AuditLog::new();

    // Record events
    let record1 = audit_log.log(AuditEvent::ClientCreated {
        client_id: "CLT-001".to_string(),
        timestamp: Utc::now(),
    });
    println!("Record 1: hash = {}...", &record1.hash[..16]);

    let record2 = audit_log.log(AuditEvent::KycCompleted {
        client_id: "CLT-001".to_string(),
        documents: vec!["passport.pdf".to_string(), "utility_bill.pdf".to_string()],
        timestamp: Utc::now(),
    });
    println!("Record 2: hash = {}...", &record2.hash[..16]);

    let record3 = audit_log.log(AuditEvent::OrderPlaced {
        client_id: "CLT-001".to_string(),
        order_id: "ORD-001".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        quantity: 0.5,
        price: 50000.0,
        timestamp: Utc::now(),
    });
    println!("Record 3: hash = {}...", &record3.hash[..16]);

    // Verify integrity
    println!("\nVerifying integrity...");
    match audit_log.verify_integrity() {
        Ok(()) => println!("Log integrity confirmed"),
        Err(e) => println!("Error: {}", e),
    }

    // Client history
    let history = audit_log.get_client_history("CLT-001");
    println!("\nClient CLT-001 history: {} records", history.len());
}
```

## Limits and Restrictions

Compliance requires limit enforcement:

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Limit types
#[derive(Debug, Clone)]
pub enum LimitType {
    /// Maximum position size
    MaxPositionSize { symbol: String, max_quantity: f64 },
    /// Maximum daily trading volume
    MaxDailyVolume { max_usd: f64 },
    /// Maximum daily loss
    MaxDailyLoss { max_usd: f64 },
    /// Maximum leverage
    MaxLeverage { max_ratio: f64 },
    /// Allowed instruments
    AllowedSymbols { symbols: Vec<String> },
}

/// Limit check result
#[derive(Debug)]
pub enum LimitCheckResult {
    Allowed,
    Denied { reason: String, limit_type: String },
    RequiresApproval { reason: String },
}

/// Limit manager
pub struct LimitManager {
    limits: RwLock<HashMap<String, Vec<LimitType>>>,
    daily_volumes: RwLock<HashMap<String, f64>>,
    daily_pnl: RwLock<HashMap<String, f64>>,
    positions: RwLock<HashMap<(String, String), f64>>, // (client_id, symbol) -> quantity
}

impl LimitManager {
    pub fn new() -> Self {
        LimitManager {
            limits: RwLock::new(HashMap::new()),
            daily_volumes: RwLock::new(HashMap::new()),
            daily_pnl: RwLock::new(HashMap::new()),
            positions: RwLock::new(HashMap::new()),
        }
    }

    /// Set limits for a client
    pub fn set_limits(&self, client_id: &str, limits: Vec<LimitType>) {
        let mut all_limits = self.limits.write().unwrap();
        all_limits.insert(client_id.to_string(), limits);
    }

    /// Check if order can be placed
    pub fn check_order(
        &self,
        client_id: &str,
        symbol: &str,
        quantity: f64,
        price: f64,
        is_buy: bool,
    ) -> LimitCheckResult {
        let limits = self.limits.read().unwrap();
        let Some(client_limits) = limits.get(client_id) else {
            return LimitCheckResult::Denied {
                reason: "Limits not configured".to_string(),
                limit_type: "NoLimits".to_string(),
            };
        };

        let order_value = quantity * price;

        for limit in client_limits {
            match limit {
                LimitType::MaxPositionSize { symbol: limit_symbol, max_quantity } => {
                    if limit_symbol == symbol {
                        let positions = self.positions.read().unwrap();
                        let current = positions
                            .get(&(client_id.to_string(), symbol.to_string()))
                            .copied()
                            .unwrap_or(0.0);

                        let new_position = if is_buy {
                            current + quantity
                        } else {
                            current - quantity
                        };

                        if new_position.abs() > *max_quantity {
                            return LimitCheckResult::Denied {
                                reason: format!(
                                    "Position limit exceeded: {} > {}",
                                    new_position.abs(), max_quantity
                                ),
                                limit_type: "MaxPositionSize".to_string(),
                            };
                        }
                    }
                }

                LimitType::MaxDailyVolume { max_usd } => {
                    let volumes = self.daily_volumes.read().unwrap();
                    let current = volumes.get(client_id).copied().unwrap_or(0.0);

                    if current + order_value > *max_usd {
                        return LimitCheckResult::Denied {
                            reason: format!(
                                "Daily volume exceeded: {} + {} > {}",
                                current, order_value, max_usd
                            ),
                            limit_type: "MaxDailyVolume".to_string(),
                        };
                    }
                }

                LimitType::MaxDailyLoss { max_usd } => {
                    let pnl = self.daily_pnl.read().unwrap();
                    let current_pnl = pnl.get(client_id).copied().unwrap_or(0.0);

                    if current_pnl < -(*max_usd) {
                        return LimitCheckResult::Denied {
                            reason: format!(
                                "Loss limit reached: {} < -{}",
                                current_pnl, max_usd
                            ),
                            limit_type: "MaxDailyLoss".to_string(),
                        };
                    }
                }

                LimitType::AllowedSymbols { symbols } => {
                    if !symbols.contains(&symbol.to_string()) {
                        return LimitCheckResult::Denied {
                            reason: format!(
                                "Instrument {} not allowed for trading",
                                symbol
                            ),
                            limit_type: "AllowedSymbols".to_string(),
                        };
                    }
                }

                _ => {}
            }
        }

        // Large orders require approval
        if order_value > 50000.0 {
            return LimitCheckResult::RequiresApproval {
                reason: format!("Large order: {} USD", order_value),
            };
        }

        LimitCheckResult::Allowed
    }

    /// Update position after execution
    pub fn update_position(&self, client_id: &str, symbol: &str, quantity: f64, is_buy: bool) {
        let mut positions = self.positions.write().unwrap();
        let key = (client_id.to_string(), symbol.to_string());
        let current = positions.get(&key).copied().unwrap_or(0.0);

        let new_position = if is_buy {
            current + quantity
        } else {
            current - quantity
        };

        positions.insert(key, new_position);
    }

    /// Update daily volume
    pub fn update_daily_volume(&self, client_id: &str, value: f64) {
        let mut volumes = self.daily_volumes.write().unwrap();
        let current = volumes.get(client_id).copied().unwrap_or(0.0);
        volumes.insert(client_id.to_string(), current + value);
    }

    /// Update daily P&L
    pub fn update_daily_pnl(&self, client_id: &str, pnl: f64) {
        let mut pnls = self.daily_pnl.write().unwrap();
        let current = pnls.get(client_id).copied().unwrap_or(0.0);
        pnls.insert(client_id.to_string(), current + pnl);
    }
}

fn main() {
    println!("=== Limit Checking System ===\n");

    let limit_manager = LimitManager::new();

    // Configure limits for client
    limit_manager.set_limits("CLT-001", vec![
        LimitType::MaxPositionSize {
            symbol: "BTCUSDT".to_string(),
            max_quantity: 10.0,
        },
        LimitType::MaxDailyVolume { max_usd: 100000.0 },
        LimitType::MaxDailyLoss { max_usd: 5000.0 },
        LimitType::AllowedSymbols {
            symbols: vec![
                "BTCUSDT".to_string(),
                "ETHUSDT".to_string(),
            ],
        },
    ]);

    // Check orders
    let orders = vec![
        ("BTCUSDT", 0.5, 50000.0, true),    // Small order
        ("BTCUSDT", 2.0, 50000.0, true),    // Large order (requires approval)
        ("BTCUSDT", 15.0, 50000.0, true),   // Exceeds position limit
        ("DOGEUSDT", 1000.0, 0.1, true),    // Disallowed instrument
    ];

    for (symbol, qty, price, is_buy) in orders {
        let result = limit_manager.check_order("CLT-001", symbol, qty, price, is_buy);
        println!("Order {} {} @ {}: {:?}", symbol, qty, price, result);
    }
}
```

## Regulatory Reporting

Generating reports in required formats:

```rust
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};

/// Trade Report
#[derive(Debug, Serialize)]
pub struct TradeReport {
    pub report_id: String,
    pub reporting_entity: String,
    pub report_date: NaiveDate,
    pub trades: Vec<TradeReportEntry>,
    pub summary: TradeSummary,
}

#[derive(Debug, Serialize)]
pub struct TradeReportEntry {
    pub trade_id: String,
    pub execution_time: DateTime<Utc>,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
    pub value: f64,
    pub client_id: String,
    pub venue: String,
    pub order_type: String,
}

#[derive(Debug, Serialize)]
pub struct TradeSummary {
    pub total_trades: u64,
    pub total_buy_value: f64,
    pub total_sell_value: f64,
    pub unique_clients: u64,
    pub unique_symbols: u64,
}

/// Position Report
#[derive(Debug, Serialize)]
pub struct PositionReport {
    pub report_id: String,
    pub report_date: NaiveDate,
    pub positions: Vec<PositionEntry>,
    pub total_exposure: f64,
}

#[derive(Debug, Serialize)]
pub struct PositionEntry {
    pub client_id: String,
    pub symbol: String,
    pub quantity: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub exposure_usd: f64,
}

/// Risk Report
#[derive(Debug, Serialize)]
pub struct RiskReport {
    pub report_id: String,
    pub report_date: NaiveDate,
    pub var_95: f64,      // Value at Risk 95%
    pub var_99: f64,      // Value at Risk 99%
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub position_concentration: HashMap<String, f64>,
    pub limit_utilization: Vec<LimitUtilization>,
}

#[derive(Debug, Serialize)]
pub struct LimitUtilization {
    pub client_id: String,
    pub limit_type: String,
    pub current_usage: f64,
    pub max_allowed: f64,
    pub utilization_pct: f64,
}

/// Report generator
pub struct ReportGenerator {
    entity_name: String,
}

impl ReportGenerator {
    pub fn new(entity_name: &str) -> Self {
        ReportGenerator {
            entity_name: entity_name.to_string(),
        }
    }

    /// Generate trade report
    pub fn generate_trade_report(
        &self,
        date: NaiveDate,
        trades: Vec<TradeReportEntry>,
    ) -> TradeReport {
        let total_trades = trades.len() as u64;

        let (buy_value, sell_value) = trades.iter().fold((0.0, 0.0), |acc, t| {
            if t.side == "BUY" {
                (acc.0 + t.value, acc.1)
            } else {
                (acc.0, acc.1 + t.value)
            }
        });

        let unique_clients: std::collections::HashSet<_> =
            trades.iter().map(|t| &t.client_id).collect();
        let unique_symbols: std::collections::HashSet<_> =
            trades.iter().map(|t| &t.symbol).collect();

        TradeReport {
            report_id: format!("TR-{}-{}", date.format("%Y%m%d"), uuid::Uuid::new_v4()),
            reporting_entity: self.entity_name.clone(),
            report_date: date,
            trades,
            summary: TradeSummary {
                total_trades,
                total_buy_value: buy_value,
                total_sell_value: sell_value,
                unique_clients: unique_clients.len() as u64,
                unique_symbols: unique_symbols.len() as u64,
            },
        }
    }

    /// Export to JSON for regulator
    pub fn export_to_json<T: Serialize>(&self, report: &T) -> String {
        serde_json::to_string_pretty(report).unwrap_or_default()
    }

    /// Export to CSV
    pub fn export_trades_to_csv(&self, report: &TradeReport) -> String {
        let mut csv = String::from(
            "trade_id,execution_time,symbol,side,quantity,price,value,client_id,venue,order_type\n"
        );

        for trade in &report.trades {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                trade.trade_id,
                trade.execution_time.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.price,
                trade.value,
                trade.client_id,
                trade.venue,
                trade.order_type,
            ));
        }

        csv
    }
}

fn main() {
    println!("=== Generating Reports for Regulators ===\n");

    let generator = ReportGenerator::new("TradingBot LLC");

    // Create test data
    let trades = vec![
        TradeReportEntry {
            trade_id: "TRD-001".to_string(),
            execution_time: Utc::now(),
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            quantity: 0.5,
            price: 50000.0,
            value: 25000.0,
            client_id: "CLT-001".to_string(),
            venue: "BINANCE".to_string(),
            order_type: "LIMIT".to_string(),
        },
        TradeReportEntry {
            trade_id: "TRD-002".to_string(),
            execution_time: Utc::now(),
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            quantity: 5.0,
            price: 3000.0,
            value: 15000.0,
            client_id: "CLT-002".to_string(),
            venue: "BINANCE".to_string(),
            order_type: "MARKET".to_string(),
        },
    ];

    let report = generator.generate_trade_report(
        Utc::now().date_naive(),
        trades,
    );

    println!("Trade Report:");
    println!("  ID: {}", report.report_id);
    println!("  Total trades: {}", report.summary.total_trades);
    println!("  Buy volume: ${:.2}", report.summary.total_buy_value);
    println!("  Sell volume: ${:.2}", report.summary.total_sell_value);

    println!("\nJSON for regulator:");
    println!("{}", generator.export_to_json(&report));

    println!("\nCSV format:");
    println!("{}", generator.export_trades_to_csv(&report));
}
```

## Personal Data Encryption

Protecting client data:

```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::generic_array::GenericArray;
use rand::RngCore;
use serde::{Serialize, Deserialize};

/// Encrypted personally identifiable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPII {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Encryption manager for personal data
pub struct PIIEncryption {
    cipher: Aes256Gcm,
}

impl PIIEncryption {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
        PIIEncryption { cipher }
    }

    /// Generate key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypt data
    pub fn encrypt(&self, data: &str) -> Result<EncryptedPII, String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| format!("Encryption error: {}", e))?;

        Ok(EncryptedPII {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedPII) -> Result<String, String> {
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = self.cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| format!("Decryption error: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| format!("Decoding error: {}", e))
    }
}

/// Secure client data
#[derive(Debug, Serialize, Deserialize)]
pub struct SecureClientData {
    pub client_id: String,
    pub encrypted_name: EncryptedPII,
    pub encrypted_email: EncryptedPII,
    pub encrypted_passport: EncryptedPII,
    pub kyc_status: String,
    pub created_at: String,
}

fn main() {
    println!("=== Personal Data Encryption ===\n");

    // Generate encryption key
    let key = PIIEncryption::generate_key();
    let encryption = PIIEncryption::new(&key);

    // Encrypt client data
    let name = "John Smith";
    let email = "john@example.com";
    let passport = "AB1234567";

    let encrypted_name = encryption.encrypt(name).expect("Name encryption");
    let encrypted_email = encryption.encrypt(email).expect("Email encryption");
    let encrypted_passport = encryption.encrypt(passport).expect("Passport encryption");

    println!("Original data:");
    println!("  Name: {}", name);
    println!("  Email: {}", email);
    println!("  Passport: {}", passport);

    println!("\nEncrypted data (first 32 bytes):");
    println!("  Name: {:?}...", &encrypted_name.ciphertext[..16.min(encrypted_name.ciphertext.len())]);
    println!("  Email: {:?}...", &encrypted_email.ciphertext[..16.min(encrypted_email.ciphertext.len())]);

    // Decrypt
    let decrypted_name = encryption.decrypt(&encrypted_name).expect("Decryption");
    let decrypted_email = encryption.decrypt(&encrypted_email).expect("Decryption");
    let decrypted_passport = encryption.decrypt(&encrypted_passport).expect("Decryption");

    println!("\nDecrypted data:");
    println!("  Name: {}", decrypted_name);
    println!("  Email: {}", decrypted_email);
    println!("  Passport: {}", decrypted_passport);

    // Create secure client record
    let secure_client = SecureClientData {
        client_id: "CLT-001".to_string(),
        encrypted_name,
        encrypted_email,
        encrypted_passport,
        kyc_status: "VERIFIED".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    println!("\nSecure Client Data (JSON):");
    println!("{}", serde_json::to_string_pretty(&secure_client).unwrap());
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **KYC/AML** | Type-safe client verification through phantom types |
| **Audit** | Immutable log with integrity verification (hash chain) |
| **Limits** | Trading restrictions checking system |
| **Reporting** | Report generation for regulators in JSON/CSV |
| **Encryption** | Personal data protection (PII) via AES-256-GCM |
| **Separation of duties** | Access control through types and modules |

## Practical Exercises

1. **Type-safe verification**: Extend the verification system:
   - Add `Suspended` state for blocked clients
   - Implement `KycExpired` transition when documents expire
   - Add status change history

2. **Extended audit**: Add to the audit log:
   - Record signing (ECDSA)
   - Periodic checkpoints
   - Old records archiving

3. **Limit system**: Implement:
   - Dynamic limit changes
   - Notifications when approaching limit (80%)
   - Automatic position closure on breach

4. **Report generator**: Create:
   - Suspicious Activity Report (SAR)
   - Currency Transaction Report (CTR)
   - Templates for different regulators (SEC, FCA, CySEC)

## Homework

1. **Complete Compliance System**: Create a comprehensive system:
   - Multi-level client verification (Individual, Corporate, Institutional)
   - Rule-based AML scoring
   - Automatic suspicious pattern detection
   - Daily reports for compliance officer
   - Integration with external sanctions databases

2. **Audit with Recovery**: Implement:
   - Audit log replication across multiple nodes
   - Recovery after failure with integrity verification
   - Merkle tree for efficient verification
   - API for external auditors

3. **GDPR Compliance**: Create a system for GDPR compliance:
   - Right to be forgotten (with hash retention for audit)
   - Client personal data export
   - Data processing consents
   - Logging all access to personal data

4. **Regulator Integration**: Implement:
   - FIX protocol for reporting
   - Automatic scheduled report submission
   - Regulator request handling
   - Dashboard for compliance status monitoring

## Navigation

[← Previous day](../362-post-mortem-incident-analysis/en.md) | [Next day →](../364-refactoring-system-evolution/en.md)
