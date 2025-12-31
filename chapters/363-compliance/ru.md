# День 363: Compliance: соответствие требованиям

## Аналогия из трейдинга

Представь, что ты управляешь инвестиционным фондом. Каждый день регуляторы могут запросить:
- **Откуда деньги?** (AML — Anti-Money Laundering)
- **Кто твои клиенты?** (KYC — Know Your Customer)
- **Почему ты купил эти активы?** (Обоснование сделок)
- **Сколько ты платишь комиссий?** (Прозрачность издержек)

**Compliance в трейдинге — это как внутренний аудитор:**

| Регуляторные требования | Compliance в коде |
|------------------------|-------------------|
| **Аудит сделок** | Логирование всех операций с неизменяемой историей |
| **Лимиты позиций** | Валидация через типы и runtime проверки |
| **Отчётность** | Автоматическая генерация отчётов |
| **Идентификация** | Типобезопасные ID клиентов и счетов |
| **Разделение обязанностей** | Контроль доступа через типы и модули |

Несоблюдение compliance может стоить миллионы в штрафах. В коде — это баги, уязвимости и потеря данных.

## Типы для Compliance

Используем систему типов Rust для гарантии соблюдения правил:

```rust
use std::marker::PhantomData;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Маркеры состояния верификации
#[derive(Debug, Clone)]
pub struct Unverified;
#[derive(Debug, Clone)]
pub struct KycPending;
#[derive(Debug, Clone)]
pub struct KycVerified;
#[derive(Debug, Clone)]
pub struct AmlCleared;

/// Клиент с типобезопасным статусом верификации
#[derive(Debug, Clone)]
pub struct Client<Status> {
    id: ClientId,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
    _status: PhantomData<Status>,
}

/// Типобезопасный ID клиента
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
    /// Создание нового неверифицированного клиента
    pub fn new(id: ClientId, name: String, email: String) -> Self {
        Client {
            id,
            name,
            email,
            created_at: Utc::now(),
            _status: PhantomData,
        }
    }

    /// Начать процесс KYC верификации
    pub fn start_kyc(self) -> Client<KycPending> {
        println!("KYC процесс начат для клиента: {}", self.id.0);
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
    /// Завершить KYC верификацию
    pub fn complete_kyc(self, documents_verified: bool) -> Result<Client<KycVerified>, String> {
        if !documents_verified {
            return Err("Документы не прошли верификацию".to_string());
        }

        println!("KYC верификация завершена для клиента: {}", self.id.0);
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
    /// Пройти AML проверку
    pub fn clear_aml(self, risk_score: u8) -> Result<Client<AmlCleared>, String> {
        if risk_score > 70 {
            return Err(format!("Высокий риск AML: {}", risk_score));
        }

        println!("AML проверка пройдена для клиента: {}", self.id.0);
        Ok(Client {
            id: self.id,
            name: self.name,
            email: self.email,
            created_at: self.created_at,
            _status: PhantomData,
        })
    }
}

/// Только полностью верифицированные клиенты могут торговать
impl Client<AmlCleared> {
    pub fn place_order(&self, symbol: &str, quantity: f64, price: f64) -> Order {
        println!("Клиент {} размещает ордер: {} {} @ {}",
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
    println!("=== Типобезопасный Compliance ===\n");

    // Создаём неверифицированного клиента
    let client = Client::<Unverified>::new(
        ClientId::new("CLT-001"),
        "Иван Петров".to_string(),
        "ivan@example.com".to_string()
    );

    // Начинаем KYC
    let client = client.start_kyc();

    // Завершаем KYC
    let client = client.complete_kyc(true)
        .expect("KYC должен пройти");

    // Проходим AML проверку
    let client = client.clear_aml(25)
        .expect("AML должен пройти");

    // Теперь клиент может торговать
    let order = client.place_order("BTCUSDT", 0.5, 50000.0);
    println!("\nОрдер создан: {:?}", order);

    // Компилятор не позволит торговать неверифицированному клиенту:
    // let unverified = Client::<Unverified>::new(...);
    // unverified.place_order(...); // Ошибка компиляции!
}
```

## Аудит и неизменяемая история

Все операции должны быть записаны для аудита:

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Типы событий для аудита
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

/// Запись в журнале аудита с хэшем для целостности
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

/// Неизменяемый журнал аудита
pub struct AuditLog {
    records: RwLock<Vec<AuditRecord>>,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog {
            records: RwLock::new(Vec::new()),
        }
    }

    /// Добавить событие в журнал
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

    /// Проверить целостность журнала
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
                    "Нарушение целостности в записи {}: неверный previous_hash",
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
                    "Нарушение целостности в записи {}: неверный hash",
                    record.sequence
                ));
            }
        }

        Ok(())
    }

    /// Получить все записи для определённого клиента
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

    /// Экспорт для регулятора
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
    println!("=== Журнал аудита с проверкой целостности ===\n");

    let audit_log = AuditLog::new();

    // Записываем события
    let record1 = audit_log.log(AuditEvent::ClientCreated {
        client_id: "CLT-001".to_string(),
        timestamp: Utc::now(),
    });
    println!("Запись 1: hash = {}...", &record1.hash[..16]);

    let record2 = audit_log.log(AuditEvent::KycCompleted {
        client_id: "CLT-001".to_string(),
        documents: vec!["passport.pdf".to_string(), "utility_bill.pdf".to_string()],
        timestamp: Utc::now(),
    });
    println!("Запись 2: hash = {}...", &record2.hash[..16]);

    let record3 = audit_log.log(AuditEvent::OrderPlaced {
        client_id: "CLT-001".to_string(),
        order_id: "ORD-001".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        quantity: 0.5,
        price: 50000.0,
        timestamp: Utc::now(),
    });
    println!("Запись 3: hash = {}...", &record3.hash[..16]);

    // Проверяем целостность
    println!("\nПроверка целостности...");
    match audit_log.verify_integrity() {
        Ok(()) => println!("Целостность журнала подтверждена"),
        Err(e) => println!("Ошибка: {}", e),
    }

    // История клиента
    let history = audit_log.get_client_history("CLT-001");
    println!("\nИстория клиента CLT-001: {} записей", history.len());
}
```

## Лимиты и ограничения

Compliance требует соблюдения лимитов:

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Типы лимитов
#[derive(Debug, Clone)]
pub enum LimitType {
    /// Максимальный размер позиции
    MaxPositionSize { symbol: String, max_quantity: f64 },
    /// Максимальный дневной объём торгов
    MaxDailyVolume { max_usd: f64 },
    /// Максимальный убыток за день
    MaxDailyLoss { max_usd: f64 },
    /// Максимальное плечо
    MaxLeverage { max_ratio: f64 },
    /// Ограничение по инструментам
    AllowedSymbols { symbols: Vec<String> },
}

/// Результат проверки лимита
#[derive(Debug)]
pub enum LimitCheckResult {
    Allowed,
    Denied { reason: String, limit_type: String },
    RequiresApproval { reason: String },
}

/// Менеджер лимитов
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

    /// Установить лимиты для клиента
    pub fn set_limits(&self, client_id: &str, limits: Vec<LimitType>) {
        let mut all_limits = self.limits.write().unwrap();
        all_limits.insert(client_id.to_string(), limits);
    }

    /// Проверить возможность размещения ордера
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
                reason: "Лимиты не настроены".to_string(),
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
                                    "Превышен лимит позиции: {} > {}",
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
                                "Превышен дневной объём: {} + {} > {}",
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
                                "Достигнут лимит убытков: {} < -{}",
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
                                "Инструмент {} не разрешён для торговли",
                                symbol
                            ),
                            limit_type: "AllowedSymbols".to_string(),
                        };
                    }
                }

                _ => {}
            }
        }

        // Большие ордера требуют одобрения
        if order_value > 50000.0 {
            return LimitCheckResult::RequiresApproval {
                reason: format!("Крупный ордер: {} USD", order_value),
            };
        }

        LimitCheckResult::Allowed
    }

    /// Обновить позицию после исполнения
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

    /// Обновить дневной объём
    pub fn update_daily_volume(&self, client_id: &str, value: f64) {
        let mut volumes = self.daily_volumes.write().unwrap();
        let current = volumes.get(client_id).copied().unwrap_or(0.0);
        volumes.insert(client_id.to_string(), current + value);
    }

    /// Обновить дневной P&L
    pub fn update_daily_pnl(&self, client_id: &str, pnl: f64) {
        let mut pnls = self.daily_pnl.write().unwrap();
        let current = pnls.get(client_id).copied().unwrap_or(0.0);
        pnls.insert(client_id.to_string(), current + pnl);
    }
}

fn main() {
    println!("=== Система проверки лимитов ===\n");

    let limit_manager = LimitManager::new();

    // Настраиваем лимиты для клиента
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

    // Проверяем ордера
    let orders = vec![
        ("BTCUSDT", 0.5, 50000.0, true),    // Маленький ордер
        ("BTCUSDT", 2.0, 50000.0, true),    // Большой ордер (требует одобрения)
        ("BTCUSDT", 15.0, 50000.0, true),   // Превышает лимит позиции
        ("DOGEUSDT", 1000.0, 0.1, true),    // Неразрешённый инструмент
    ];

    for (symbol, qty, price, is_buy) in orders {
        let result = limit_manager.check_order("CLT-001", symbol, qty, price, is_buy);
        println!("Ордер {} {} @ {}: {:?}", symbol, qty, price, result);
    }
}
```

## Отчётность для регуляторов

Генерация отчётов в требуемом формате:

```rust
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};

/// Отчёт о сделках (Trade Report)
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

/// Отчёт о позициях (Position Report)
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

/// Отчёт о рисках (Risk Report)
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

/// Генератор отчётов
pub struct ReportGenerator {
    entity_name: String,
}

impl ReportGenerator {
    pub fn new(entity_name: &str) -> Self {
        ReportGenerator {
            entity_name: entity_name.to_string(),
        }
    }

    /// Генерация отчёта о сделках
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

    /// Экспорт в JSON для регулятора
    pub fn export_to_json<T: Serialize>(&self, report: &T) -> String {
        serde_json::to_string_pretty(report).unwrap_or_default()
    }

    /// Экспорт в CSV
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
    println!("=== Генерация отчётов для регуляторов ===\n");

    let generator = ReportGenerator::new("TradingBot LLC");

    // Создаём тестовые данные
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

    println!("Отчёт о сделках:");
    println!("  ID: {}", report.report_id);
    println!("  Всего сделок: {}", report.summary.total_trades);
    println!("  Объём покупок: ${:.2}", report.summary.total_buy_value);
    println!("  Объём продаж: ${:.2}", report.summary.total_sell_value);

    println!("\nJSON для регулятора:");
    println!("{}", generator.export_to_json(&report));

    println!("\nCSV формат:");
    println!("{}", generator.export_trades_to_csv(&report));
}
```

## Шифрование персональных данных

Защита данных клиентов:

```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::generic_array::GenericArray;
use rand::RngCore;
use serde::{Serialize, Deserialize};

/// Зашифрованные персональные данные
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPII {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Менеджер шифрования для персональных данных
pub struct PIIEncryption {
    cipher: Aes256Gcm,
}

impl PIIEncryption {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
        PIIEncryption { cipher }
    }

    /// Генерация ключа
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Шифрование данных
    pub fn encrypt(&self, data: &str) -> Result<EncryptedPII, String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| format!("Ошибка шифрования: {}", e))?;

        Ok(EncryptedPII {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    /// Расшифровка данных
    pub fn decrypt(&self, encrypted: &EncryptedPII) -> Result<String, String> {
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = self.cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| format!("Ошибка расшифровки: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| format!("Ошибка декодирования: {}", e))
    }
}

/// Защищённые данные клиента
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
    println!("=== Шифрование персональных данных ===\n");

    // Генерируем ключ шифрования
    let key = PIIEncryption::generate_key();
    let encryption = PIIEncryption::new(&key);

    // Шифруем данные клиента
    let name = "Иван Петров";
    let email = "ivan@example.com";
    let passport = "1234 567890";

    let encrypted_name = encryption.encrypt(name).expect("Шифрование имени");
    let encrypted_email = encryption.encrypt(email).expect("Шифрование email");
    let encrypted_passport = encryption.encrypt(passport).expect("Шифрование паспорта");

    println!("Оригинальные данные:");
    println!("  Имя: {}", name);
    println!("  Email: {}", email);
    println!("  Паспорт: {}", passport);

    println!("\nЗашифрованные данные (первые 32 байта):");
    println!("  Имя: {:?}...", &encrypted_name.ciphertext[..16.min(encrypted_name.ciphertext.len())]);
    println!("  Email: {:?}...", &encrypted_email.ciphertext[..16.min(encrypted_email.ciphertext.len())]);

    // Расшифровываем
    let decrypted_name = encryption.decrypt(&encrypted_name).expect("Расшифровка");
    let decrypted_email = encryption.decrypt(&encrypted_email).expect("Расшифровка");
    let decrypted_passport = encryption.decrypt(&encrypted_passport).expect("Расшифровка");

    println!("\nРасшифрованные данные:");
    println!("  Имя: {}", decrypted_name);
    println!("  Email: {}", decrypted_email);
    println!("  Паспорт: {}", decrypted_passport);

    // Создаём защищённую запись клиента
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

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **KYC/AML** | Типобезопасная верификация клиентов через phantom types |
| **Аудит** | Неизменяемый журнал с проверкой целостности (хэш-цепочка) |
| **Лимиты** | Система проверки торговых ограничений |
| **Отчётность** | Генерация отчётов для регуляторов в JSON/CSV |
| **Шифрование** | Защита персональных данных (PII) через AES-256-GCM |
| **Разделение обязанностей** | Контроль доступа через типы и модули |

## Практические задания

1. **Типобезопасная верификация**: Расширь систему верификации:
   - Добавь состояние `Suspended` для заблокированных клиентов
   - Реализуй переход `KycExpired` при устаревании документов
   - Добавь историю изменений статуса

2. **Расширенный аудит**: Добавь в журнал аудита:
   - Подпись записей (ECDSA)
   - Периодические контрольные точки (checkpoints)
   - Архивирование старых записей

3. **Система лимитов**: Реализуй:
   - Динамическое изменение лимитов
   - Уведомления при приближении к лимиту (80%)
   - Автоматическое закрытие позиций при превышении

4. **Генератор отчётов**: Создай:
   - Отчёт о подозрительных операциях (SAR)
   - Отчёт о крупных сделках (CTR)
   - Шаблоны для разных регуляторов (SEC, FCA, CySEC)

## Домашнее задание

1. **Полная система Compliance**: Создай комплексную систему:
   - Многоуровневая верификация клиентов (Individual, Corporate, Institutional)
   - AML скоринг на основе правил
   - Автоматическое определение подозрительных паттернов
   - Ежедневные отчёты для compliance офицера
   - Интеграция с внешними базами санкций

2. **Аудит с восстановлением**: Реализуй:
   - Репликация журнала аудита на несколько узлов
   - Восстановление после сбоя с проверкой целостности
   - Merkle tree для эффективной верификации
   - API для внешних аудиторов

3. **GDPR Compliance**: Создай систему для соответствия GDPR:
   - Право на забвение (с сохранением хэша для аудита)
   - Экспорт персональных данных клиента
   - Согласия на обработку данных
   - Логирование всех доступов к персональным данным

4. **Интеграция с регуляторами**: Реализуй:
   - FIX протокол для отчётности
   - Автоматическая отправка отчётов по расписанию
   - Обработка запросов от регуляторов
   - Dashboard для мониторинга compliance статуса

## Навигация

[← Предыдущий день](../362-post-mortem-incident-analysis/ru.md) | [Следующий день →](../364-refactoring-system-evolution/ru.md)
