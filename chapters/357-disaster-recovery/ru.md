# День 357: Disaster Recovery

## Аналогия из трейдинга

Представь, что ты управляешь крупным криптовалютным хедж-фондом. У тебя есть бэкапы (которые мы рассмотрели в предыдущей главе), но что будет, если:

- Дата-центр в Нью-Йорке полностью выходит из строя из-за урагана
- Все три твоих сервера одновременно заражены ransomware
- Критическая ошибка в коде обнуляет балансы всех клиентов
- Атака на инфраструктуру биржи, с которой ты работаешь

**Без плана Disaster Recovery:**
- Паника в команде — никто не знает, что делать
- Часы или дни на восстановление
- Потеря критических торговых окон
- Клиенты теряют деньги на открытых позициях
- Репутационный ущерб

**С планом Disaster Recovery:**
- Чёткие роли и ответственности
- Автоматическое переключение на резервный дата-центр
- RTO (Recovery Time Objective) — восстановление за минуты
- RPO (Recovery Point Objective) — потеря максимум последних секунд данных
- Клиенты даже не замечают проблемы

| Трейдинг | Disaster Recovery |
|----------|-------------------|
| **Хеджирование** | Резервные системы в разных локациях |
| **Стоп-лосс** | Автоматический failover |
| **Диверсификация рисков** | Multi-cloud стратегия |
| **Торговый план** | DR Runbook |
| **Симулятор торговли** | DR-тестирование |
| **Управление позициями** | Приоритизация восстановления систем |

В трейдинге каждая секунда простоя — это потенциальные убытки. DR гарантирует, что ты сможешь продолжить торговлю при любом сценарии.

## Основы Disaster Recovery

### Ключевые метрики DR

```rust
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Ключевые метрики Disaster Recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DRMetrics {
    /// Recovery Time Objective — максимально допустимое время простоя
    pub rto: Duration,
    /// Recovery Point Objective — максимально допустимая потеря данных
    pub rpo: Duration,
    /// Maximum Tolerable Downtime — критический порог простоя
    pub mtd: Duration,
    /// Work Recovery Time — время на возврат к полной производительности
    pub wrt: Duration,
}

impl DRMetrics {
    /// Создать метрики для критической торговой системы
    pub fn critical_trading() -> Self {
        DRMetrics {
            rto: Duration::from_secs(60),      // 1 минута
            rpo: Duration::from_secs(1),       // 1 секунда
            mtd: Duration::from_secs(300),     // 5 минут
            wrt: Duration::from_secs(600),     // 10 минут
        }
    }

    /// Создать метрики для стандартной системы
    pub fn standard_system() -> Self {
        DRMetrics {
            rto: Duration::from_secs(3600),    // 1 час
            rpo: Duration::from_secs(300),     // 5 минут
            mtd: Duration::from_secs(14400),   // 4 часа
            wrt: Duration::from_secs(28800),   // 8 часов
        }
    }

    /// Проверить, достижимы ли метрики
    pub fn validate(&self) -> Result<(), String> {
        if self.rto > self.mtd {
            return Err("RTO не может превышать MTD".to_string());
        }
        if self.rto + self.wrt > self.mtd {
            return Err("RTO + WRT превышает MTD".to_string());
        }
        Ok(())
    }
}

/// Классификация систем по критичности
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemTier {
    /// Tier 1: Критические системы (order execution, risk management)
    Critical,
    /// Tier 2: Важные системы (market data, reporting)
    Important,
    /// Tier 3: Поддерживающие системы (analytics, backtesting)
    Supporting,
    /// Tier 4: Некритические системы (dev environments, documentation)
    NonCritical,
}

impl SystemTier {
    pub fn default_metrics(&self) -> DRMetrics {
        match self {
            SystemTier::Critical => DRMetrics {
                rto: Duration::from_secs(60),
                rpo: Duration::from_secs(1),
                mtd: Duration::from_secs(300),
                wrt: Duration::from_secs(600),
            },
            SystemTier::Important => DRMetrics {
                rto: Duration::from_secs(900),
                rpo: Duration::from_secs(60),
                mtd: Duration::from_secs(3600),
                wrt: Duration::from_secs(1800),
            },
            SystemTier::Supporting => DRMetrics {
                rto: Duration::from_secs(14400),
                rpo: Duration::from_secs(3600),
                mtd: Duration::from_secs(86400),
                wrt: Duration::from_secs(14400),
            },
            SystemTier::NonCritical => DRMetrics {
                rto: Duration::from_secs(86400),
                rpo: Duration::from_secs(86400),
                mtd: Duration::from_secs(604800),
                wrt: Duration::from_secs(86400),
            },
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SystemTier::Critical => "Критические системы — немедленное восстановление",
            SystemTier::Important => "Важные системы — восстановление в течение часа",
            SystemTier::Supporting => "Поддерживающие — восстановление в течение дня",
            SystemTier::NonCritical => "Некритические — восстановление в течение недели",
        }
    }
}

/// Компонент системы для DR-планирования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemComponent {
    pub name: String,
    pub tier: SystemTier,
    pub dependencies: Vec<String>,
    pub recovery_procedure: String,
    pub owner_team: String,
    pub contact_info: String,
}

fn main() {
    println!("=== Метрики Disaster Recovery ===\n");

    // Определение метрик для торговой системы
    let trading_metrics = DRMetrics::critical_trading();
    println!("Критическая торговая система:");
    println!("  RTO: {:?}", trading_metrics.rto);
    println!("  RPO: {:?}", trading_metrics.rpo);
    println!("  MTD: {:?}", trading_metrics.mtd);
    println!("  WRT: {:?}", trading_metrics.wrt);

    // Валидация метрик
    match trading_metrics.validate() {
        Ok(()) => println!("  ✓ Метрики валидны\n"),
        Err(e) => println!("  ✗ Ошибка: {}\n", e),
    }

    // Метрики по уровням критичности
    println!("Метрики по уровням критичности:");
    for tier in [SystemTier::Critical, SystemTier::Important,
                 SystemTier::Supporting, SystemTier::NonCritical] {
        let metrics = tier.default_metrics();
        println!("\n{:?}: {}", tier, tier.description());
        println!("  RTO: {:?}, RPO: {:?}", metrics.rto, metrics.rpo);
    }
}
```

## DR Runbook — Процедуры восстановления

```rust
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Шаг процедуры восстановления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStep {
    pub order: u32,
    pub name: String,
    pub description: String,
    pub responsible_role: String,
    pub estimated_duration_secs: u64,
    pub commands: Vec<String>,
    pub verification: String,
    pub rollback_procedure: Option<String>,
}

/// Runbook — набор процедур восстановления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DRRunbook {
    pub name: String,
    pub description: String,
    pub scenario: DisasterScenario,
    pub steps: Vec<RecoveryStep>,
    pub escalation_contacts: Vec<Contact>,
    pub last_tested: Option<DateTime<Utc>>,
    pub version: String,
}

/// Типы катастрофических сценариев
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisasterScenario {
    /// Полный отказ дата-центра
    DatacenterFailure { location: String },
    /// Отказ базы данных
    DatabaseCorruption,
    /// Сетевая атака
    CyberAttack { attack_type: String },
    /// Отказ облачного провайдера
    CloudProviderOutage { provider: String },
    /// Критическая ошибка в приложении
    ApplicationFailure { component: String },
    /// Потеря данных
    DataLoss { scope: String },
}

/// Контактная информация для эскалации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub role: String,
    pub phone: String,
    pub email: String,
    pub escalation_level: u8,
}

/// Выполнение runbook
#[derive(Debug)]
pub struct RunbookExecution {
    pub runbook: DRRunbook,
    pub started_at: DateTime<Utc>,
    pub current_step: usize,
    pub step_results: Vec<StepResult>,
    pub status: ExecutionStatus,
}

#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    InProgress,
    Completed { duration_secs: u64 },
    Failed { step: u32, error: String },
    RolledBack,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_order: u32,
    pub success: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: String,
    pub error: Option<String>,
}

impl DRRunbook {
    /// Создать runbook для отказа дата-центра
    pub fn datacenter_failover(location: &str) -> Self {
        DRRunbook {
            name: format!("Datacenter Failover: {}", location),
            description: "Процедура переключения на резервный дата-центр".to_string(),
            scenario: DisasterScenario::DatacenterFailure {
                location: location.to_string(),
            },
            steps: vec![
                RecoveryStep {
                    order: 1,
                    name: "Оценка ситуации".to_string(),
                    description: "Подтвердить отказ основного дата-центра".to_string(),
                    responsible_role: "On-Call Engineer".to_string(),
                    estimated_duration_secs: 120,
                    commands: vec![
                        "ping primary-dc.trading.internal".to_string(),
                        "curl -s https://status.trading.internal/health".to_string(),
                    ],
                    verification: "Подтверждение недоступности primary DC".to_string(),
                    rollback_procedure: None,
                },
                RecoveryStep {
                    order: 2,
                    name: "Активация DR-команды".to_string(),
                    description: "Оповестить DR-команду и начать инцидент".to_string(),
                    responsible_role: "Incident Commander".to_string(),
                    estimated_duration_secs: 300,
                    commands: vec![
                        "pagerduty trigger --severity critical".to_string(),
                        "slack notify #dr-team 'DC Failover initiated'".to_string(),
                    ],
                    verification: "Все ключевые участники в канале связи".to_string(),
                    rollback_procedure: None,
                },
                RecoveryStep {
                    order: 3,
                    name: "DNS-переключение".to_string(),
                    description: "Переключить DNS на резервный дата-центр".to_string(),
                    responsible_role: "Network Engineer".to_string(),
                    estimated_duration_secs: 60,
                    commands: vec![
                        "dns-failover --zone trading.com --target dr-dc".to_string(),
                        "dns-verify --zone trading.com".to_string(),
                    ],
                    verification: "DNS resolves to DR datacenter IPs".to_string(),
                    rollback_procedure: Some(
                        "dns-failover --zone trading.com --target primary-dc".to_string()
                    ),
                },
                RecoveryStep {
                    order: 4,
                    name: "Активация БД в DR".to_string(),
                    description: "Промоутить реплику БД до primary".to_string(),
                    responsible_role: "DBA".to_string(),
                    estimated_duration_secs: 180,
                    commands: vec![
                        "pg_ctl promote -D /var/lib/postgresql/data".to_string(),
                        "psql -c 'SELECT pg_is_in_recovery()'".to_string(),
                    ],
                    verification: "База данных в режиме primary".to_string(),
                    rollback_procedure: Some(
                        "Ручное восстановление репликации с primary DC".to_string()
                    ),
                },
                RecoveryStep {
                    order: 5,
                    name: "Запуск торговых сервисов".to_string(),
                    description: "Запустить критические торговые сервисы в DR".to_string(),
                    responsible_role: "Platform Engineer".to_string(),
                    estimated_duration_secs: 120,
                    commands: vec![
                        "kubectl scale deployment order-engine --replicas=3".to_string(),
                        "kubectl scale deployment risk-manager --replicas=3".to_string(),
                        "kubectl rollout status deployment/order-engine".to_string(),
                    ],
                    verification: "Все торговые сервисы healthy".to_string(),
                    rollback_procedure: Some(
                        "kubectl scale deployment --all --replicas=0".to_string()
                    ),
                },
                RecoveryStep {
                    order: 6,
                    name: "Верификация работоспособности".to_string(),
                    description: "Проверить корректность работы системы".to_string(),
                    responsible_role: "QA Engineer".to_string(),
                    estimated_duration_secs: 300,
                    commands: vec![
                        "./run-smoke-tests.sh".to_string(),
                        "./verify-order-execution.sh".to_string(),
                        "./check-market-data-feed.sh".to_string(),
                    ],
                    verification: "Все smoke-тесты пройдены успешно".to_string(),
                    rollback_procedure: None,
                },
                RecoveryStep {
                    order: 7,
                    name: "Уведомление стейкхолдеров".to_string(),
                    description: "Информировать клиентов и руководство".to_string(),
                    responsible_role: "Communications Lead".to_string(),
                    estimated_duration_secs: 60,
                    commands: vec![
                        "send-status-update --template dr-complete".to_string(),
                    ],
                    verification: "Уведомления отправлены".to_string(),
                    rollback_procedure: None,
                },
            ],
            escalation_contacts: vec![
                Contact {
                    name: "Иван Петров".to_string(),
                    role: "CTO".to_string(),
                    phone: "+7-999-123-4567".to_string(),
                    email: "cto@trading.com".to_string(),
                    escalation_level: 1,
                },
                Contact {
                    name: "Мария Сидорова".to_string(),
                    role: "VP Engineering".to_string(),
                    phone: "+7-999-234-5678".to_string(),
                    email: "vp-eng@trading.com".to_string(),
                    escalation_level: 2,
                },
            ],
            last_tested: Some(Utc::now()),
            version: "2.1.0".to_string(),
        }
    }

    /// Расчёт общего времени восстановления
    pub fn estimated_recovery_time(&self) -> u64 {
        self.steps.iter().map(|s| s.estimated_duration_secs).sum()
    }

    /// Вывод runbook в читаемом формате
    pub fn print_summary(&self) {
        println!("=== DR Runbook: {} ===", self.name);
        println!("Версия: {}", self.version);
        println!("Описание: {}", self.description);
        println!("Сценарий: {:?}", self.scenario);
        println!("\nШаги восстановления:");

        for step in &self.steps {
            println!("\n{}. {} ({})", step.order, step.name, step.responsible_role);
            println!("   {}", step.description);
            println!("   Время: ~{} сек", step.estimated_duration_secs);
            println!("   Проверка: {}", step.verification);
        }

        println!("\nОбщее расчётное время: {} минут",
            self.estimated_recovery_time() / 60);

        println!("\nКонтакты для эскалации:");
        for contact in &self.escalation_contacts {
            println!("  L{}: {} ({}) - {}",
                contact.escalation_level, contact.name,
                contact.role, contact.phone);
        }
    }
}

impl RunbookExecution {
    pub fn start(runbook: DRRunbook) -> Self {
        println!("\n>>> ЗАПУСК DR ПРОЦЕДУРЫ: {}", runbook.name);
        println!(">>> Время начала: {}", Utc::now());

        RunbookExecution {
            runbook,
            started_at: Utc::now(),
            current_step: 0,
            step_results: Vec::new(),
            status: ExecutionStatus::InProgress,
        }
    }

    /// Выполнить следующий шаг
    pub fn execute_next_step(&mut self) -> Option<&StepResult> {
        if self.current_step >= self.runbook.steps.len() {
            return None;
        }

        let step = &self.runbook.steps[self.current_step];
        let started_at = Utc::now();

        println!("\n[Шаг {}/{}] {}",
            step.order, self.runbook.steps.len(), step.name);
        println!("  Ответственный: {}", step.responsible_role);
        println!("  Выполняем команды:");

        for cmd in &step.commands {
            println!("    $ {}", cmd);
        }

        // Симуляция выполнения (в реальности здесь был бы настоящий код)
        let success = true; // Имитация успеха
        let completed_at = Utc::now();

        let result = StepResult {
            step_order: step.order,
            success,
            started_at,
            completed_at: Some(completed_at),
            output: format!("Шаг {} выполнен успешно", step.order),
            error: None,
        };

        println!("  Проверка: {}", step.verification);
        println!("  Результат: {}", if success { "✓ Успех" } else { "✗ Ошибка" });

        self.step_results.push(result);
        self.current_step += 1;

        // Проверка завершения
        if self.current_step >= self.runbook.steps.len() {
            let duration = (Utc::now() - self.started_at).num_seconds() as u64;
            self.status = ExecutionStatus::Completed { duration_secs: duration };
            println!("\n>>> DR ПРОЦЕДУРА ЗАВЕРШЕНА");
            println!(">>> Общее время: {} секунд", duration);
        }

        self.step_results.last()
    }

    /// Выполнить все шаги
    pub fn execute_all(&mut self) {
        while self.execute_next_step().is_some() {}
    }
}

fn main() {
    println!("=== DR Runbook Demo ===\n");

    // Создание runbook для отказа дата-центра
    let runbook = DRRunbook::datacenter_failover("NYC-DC-01");
    runbook.print_summary();

    // Симуляция выполнения
    println!("\n" + &"=".repeat(50));
    let mut execution = RunbookExecution::start(runbook);
    execution.execute_all();
}
```

## Автоматический Failover

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Состояние узла
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Узел в кластере
#[derive(Debug)]
pub struct ClusterNode {
    pub id: String,
    pub address: String,
    pub role: NodeRole,
    pub state: RwLock<NodeState>,
    pub last_heartbeat: AtomicU64,
    pub consecutive_failures: AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Primary,
    Secondary,
    Witness,
}

impl ClusterNode {
    pub fn new(id: &str, address: &str, role: NodeRole) -> Self {
        ClusterNode {
            id: id.to_string(),
            address: address.to_string(),
            role,
            state: RwLock::new(NodeState::Unknown),
            last_heartbeat: AtomicU64::new(0),
            consecutive_failures: AtomicU64::new(0),
        }
    }

    pub async fn check_health(&self) -> NodeState {
        // Симуляция проверки здоровья
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let last_hb = self.last_heartbeat.load(Ordering::Relaxed);
        let age = now.saturating_sub(last_hb);

        let state = if age < 5 {
            NodeState::Healthy
        } else if age < 15 {
            NodeState::Degraded
        } else {
            NodeState::Unhealthy
        };

        *self.state.write().await = state;
        state
    }

    pub fn record_heartbeat(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_heartbeat.store(now, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    pub fn record_failure(&self) -> u64 {
        self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1
    }
}

/// Конфигурация failover
#[derive(Debug, Clone)]
pub struct FailoverConfig {
    /// Количество неудачных проверок до failover
    pub failure_threshold: u64,
    /// Интервал между проверками
    pub check_interval: Duration,
    /// Тайм-аут для проверки здоровья
    pub health_check_timeout: Duration,
    /// Время кулдауна между failover
    pub cooldown_period: Duration,
    /// Требуется ли подтверждение для failover
    pub require_confirmation: bool,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        FailoverConfig {
            failure_threshold: 3,
            check_interval: Duration::from_secs(5),
            health_check_timeout: Duration::from_secs(2),
            cooldown_period: Duration::from_secs(60),
            require_confirmation: false,
        }
    }
}

/// Событие failover
#[derive(Debug, Clone)]
pub struct FailoverEvent {
    pub timestamp: DateTime<Utc>,
    pub from_node: String,
    pub to_node: String,
    pub reason: String,
    pub duration_ms: u64,
    pub success: bool,
}

/// Менеджер автоматического failover
pub struct FailoverManager {
    nodes: Vec<Arc<ClusterNode>>,
    config: FailoverConfig,
    current_primary: RwLock<Option<String>>,
    is_failover_in_progress: AtomicBool,
    last_failover: RwLock<Option<DateTime<Utc>>>,
    failover_history: RwLock<Vec<FailoverEvent>>,
}

impl FailoverManager {
    pub fn new(config: FailoverConfig) -> Self {
        FailoverManager {
            nodes: Vec::new(),
            config,
            current_primary: RwLock::new(None),
            is_failover_in_progress: AtomicBool::new(false),
            last_failover: RwLock::new(None),
            failover_history: RwLock::new(Vec::new()),
        }
    }

    pub fn add_node(&mut self, node: ClusterNode) {
        let node = Arc::new(node);
        if node.role == NodeRole::Primary {
            // Устанавливаем primary при добавлении
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                *self.current_primary.write().await = Some(node.id.clone());
            });
        }
        self.nodes.push(node);
    }

    /// Получить текущий primary узел
    pub async fn get_primary(&self) -> Option<Arc<ClusterNode>> {
        let primary_id = self.current_primary.read().await;
        if let Some(id) = primary_id.as_ref() {
            self.nodes.iter().find(|n| &n.id == id).cloned()
        } else {
            None
        }
    }

    /// Проверить здоровье всех узлов
    pub async fn check_all_nodes(&self) -> HashMap<String, NodeState> {
        let mut states = HashMap::new();

        for node in &self.nodes {
            let state = node.check_health().await;
            states.insert(node.id.clone(), state);

            match state {
                NodeState::Healthy => {
                    node.record_heartbeat();
                }
                NodeState::Unhealthy => {
                    let failures = node.record_failure();
                    println!("[HealthCheck] Узел {} недоступен ({} подряд)",
                        node.id, failures);

                    // Проверить необходимость failover
                    if node.role == NodeRole::Primary
                        && failures >= self.config.failure_threshold
                    {
                        self.initiate_failover(&node.id).await;
                    }
                }
                _ => {}
            }
        }

        states
    }

    /// Инициировать failover
    pub async fn initiate_failover(&self, failed_node_id: &str) {
        // Проверка, не выполняется ли уже failover
        if self.is_failover_in_progress.swap(true, Ordering::SeqCst) {
            println!("[Failover] Failover уже выполняется, пропускаем");
            return;
        }

        // Проверка cooldown
        let last = self.last_failover.read().await;
        if let Some(last_time) = *last {
            let elapsed = Utc::now().signed_duration_since(last_time);
            if elapsed < chrono::Duration::from_std(self.config.cooldown_period).unwrap() {
                println!("[Failover] Cooldown period активен, пропускаем");
                self.is_failover_in_progress.store(false, Ordering::SeqCst);
                return;
            }
        }
        drop(last);

        let start = Instant::now();
        println!("\n>>> ИНИЦИИРОВАН FAILOVER <<<");
        println!(">>> Отказавший узел: {}", failed_node_id);

        // Найти кандидата для failover
        let candidate = self.find_failover_candidate().await;

        match candidate {
            Some(new_primary) => {
                println!(">>> Выбран новый primary: {}", new_primary.id);

                // Выполнить failover
                let success = self.perform_failover(failed_node_id, &new_primary).await;

                let duration_ms = start.elapsed().as_millis() as u64;

                let event = FailoverEvent {
                    timestamp: Utc::now(),
                    from_node: failed_node_id.to_string(),
                    to_node: new_primary.id.clone(),
                    reason: "Primary node health check failure".to_string(),
                    duration_ms,
                    success,
                };

                self.failover_history.write().await.push(event);
                *self.last_failover.write().await = Some(Utc::now());

                if success {
                    *self.current_primary.write().await = Some(new_primary.id.clone());
                    println!(">>> FAILOVER УСПЕШЕН ({}мс)", duration_ms);
                } else {
                    println!(">>> FAILOVER НЕУСПЕШЕН");
                }
            }
            None => {
                println!(">>> Нет доступных кандидатов для failover!");
            }
        }

        self.is_failover_in_progress.store(false, Ordering::SeqCst);
    }

    /// Найти кандидата для failover
    async fn find_failover_candidate(&self) -> Option<Arc<ClusterNode>> {
        for node in &self.nodes {
            let state = *node.state.read().await;
            if node.role == NodeRole::Secondary && state == NodeState::Healthy {
                return Some(Arc::clone(node));
            }
        }
        None
    }

    /// Выполнить процедуру failover
    async fn perform_failover(
        &self,
        _old_primary: &str,
        new_primary: &Arc<ClusterNode>,
    ) -> bool {
        println!("  1. Промоутим {} до primary...", new_primary.id);
        // Симуляция промоута
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("  2. Обновляем DNS...");
        tokio::time::sleep(Duration::from_millis(50)).await;

        println!("  3. Перенаправляем трафик...");
        tokio::time::sleep(Duration::from_millis(50)).await;

        println!("  4. Верифицируем...");
        tokio::time::sleep(Duration::from_millis(100)).await;

        true
    }

    /// Получить статистику failover
    pub async fn get_stats(&self) -> FailoverStats {
        let history = self.failover_history.read().await;

        let total = history.len();
        let successful = history.iter().filter(|e| e.success).count();
        let avg_duration = if total > 0 {
            history.iter().map(|e| e.duration_ms).sum::<u64>() / total as u64
        } else {
            0
        };

        FailoverStats {
            total_failovers: total,
            successful_failovers: successful,
            failed_failovers: total - successful,
            avg_duration_ms: avg_duration,
            last_failover: history.last().map(|e| e.timestamp),
        }
    }

    pub async fn print_status(&self) {
        println!("\n=== Статус кластера ===");

        let primary = self.get_primary().await;
        println!("Primary: {}", primary.map_or("нет".to_string(), |n| n.id.clone()));

        println!("\nУзлы:");
        for node in &self.nodes {
            let state = *node.state.read().await;
            let role = match node.role {
                NodeRole::Primary => "P",
                NodeRole::Secondary => "S",
                NodeRole::Witness => "W",
            };
            println!("  [{}] {} - {:?} ({})",
                role, node.id, state, node.address);
        }

        let stats = self.get_stats().await;
        println!("\nСтатистика failover:");
        println!("  Всего: {}", stats.total_failovers);
        println!("  Успешных: {}", stats.successful_failovers);
        println!("  Неуспешных: {}", stats.failed_failovers);
        println!("  Среднее время: {}мс", stats.avg_duration_ms);
    }
}

#[derive(Debug)]
pub struct FailoverStats {
    pub total_failovers: usize,
    pub successful_failovers: usize,
    pub failed_failovers: usize,
    pub avg_duration_ms: u64,
    pub last_failover: Option<DateTime<Utc>>,
}

#[tokio::main]
async fn main() {
    println!("=== Автоматический Failover ===\n");

    let config = FailoverConfig {
        failure_threshold: 2,
        check_interval: Duration::from_secs(2),
        cooldown_period: Duration::from_secs(30),
        ..Default::default()
    };

    let mut manager = FailoverManager::new(config);

    // Добавление узлов кластера
    let mut primary = ClusterNode::new("nyc-primary", "10.0.1.1:5432", NodeRole::Primary);
    primary.record_heartbeat(); // Симулируем активный узел

    let mut secondary = ClusterNode::new("london-secondary", "10.0.2.1:5432", NodeRole::Secondary);
    secondary.record_heartbeat();

    let mut witness = ClusterNode::new("tokyo-witness", "10.0.3.1:5432", NodeRole::Witness);
    witness.record_heartbeat();

    manager.add_node(primary);
    manager.add_node(secondary);
    manager.add_node(witness);

    manager.print_status().await;

    // Симуляция сбоя primary
    println!("\n>>> Симуляция сбоя primary узла...");

    // Имитируем устаревший heartbeat
    if let Some(primary_node) = manager.nodes.iter().find(|n| n.role == NodeRole::Primary) {
        primary_node.last_heartbeat.store(0, Ordering::Relaxed);
    }

    // Проверка здоровья (вызовет failover)
    for _ in 0..3 {
        println!("\n--- Проверка здоровья ---");
        let states = manager.check_all_nodes().await;
        for (id, state) in states {
            println!("  {} -> {:?}", id, state);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    manager.print_status().await;
}
```

## DR-тестирование

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Тип DR-теста
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DRTestType {
    /// Tabletop — обсуждение плана без реального выполнения
    Tabletop,
    /// Walkthrough — пошаговый проход процедуры
    Walkthrough,
    /// Simulation — симуляция с частичным выполнением
    Simulation,
    /// Parallel — параллельная работа DR-системы
    Parallel,
    /// Full — полное переключение на DR
    Full,
}

impl DRTestType {
    pub fn risk_level(&self) -> &'static str {
        match self {
            DRTestType::Tabletop => "Минимальный",
            DRTestType::Walkthrough => "Низкий",
            DRTestType::Simulation => "Средний",
            DRTestType::Parallel => "Средний-Высокий",
            DRTestType::Full => "Высокий",
        }
    }

    pub fn production_impact(&self) -> &'static str {
        match self {
            DRTestType::Tabletop => "Нет",
            DRTestType::Walkthrough => "Нет",
            DRTestType::Simulation => "Минимальный",
            DRTestType::Parallel => "Низкий",
            DRTestType::Full => "Высокий",
        }
    }
}

/// Сценарий тестирования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    pub name: String,
    pub description: String,
    pub test_type: DRTestType,
    pub objectives: Vec<String>,
    pub success_criteria: Vec<SuccessCriterion>,
    pub estimated_duration: Duration,
    pub required_participants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub name: String,
    pub description: String,
    pub threshold: String,
    pub priority: CriterionPriority,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CriterionPriority {
    Critical,  // Должен быть выполнен
    Major,     // Очень важен
    Minor,     // Желателен
}

/// Результат DR-теста
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DRTestResult {
    pub scenario_name: String,
    pub test_type: DRTestType,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub overall_success: bool,
    pub criteria_results: HashMap<String, CriterionResult>,
    pub actual_rto: Duration,
    pub actual_rpo: Duration,
    pub issues_found: Vec<TestIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult {
    pub passed: bool,
    pub actual_value: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub affected_component: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Координатор DR-тестирования
pub struct DRTestCoordinator {
    scenarios: Vec<TestScenario>,
    test_history: Vec<DRTestResult>,
    target_rto: Duration,
    target_rpo: Duration,
}

impl DRTestCoordinator {
    pub fn new(target_rto: Duration, target_rpo: Duration) -> Self {
        DRTestCoordinator {
            scenarios: Vec::new(),
            test_history: Vec::new(),
            target_rto,
            target_rpo,
        }
    }

    pub fn add_scenario(&mut self, scenario: TestScenario) {
        self.scenarios.push(scenario);
    }

    /// Создать стандартные сценарии для торговой системы
    pub fn create_trading_scenarios(&mut self) {
        // Сценарий 1: Отказ основной БД
        self.add_scenario(TestScenario {
            name: "Database Failover Test".to_string(),
            description: "Тестирование переключения на резервную БД".to_string(),
            test_type: DRTestType::Simulation,
            objectives: vec![
                "Проверить автоматический failover PostgreSQL".to_string(),
                "Измерить время переключения".to_string(),
                "Проверить целостность данных после failover".to_string(),
            ],
            success_criteria: vec![
                SuccessCriterion {
                    name: "RTO".to_string(),
                    description: "Время восстановления".to_string(),
                    threshold: "< 60 секунд".to_string(),
                    priority: CriterionPriority::Critical,
                },
                SuccessCriterion {
                    name: "RPO".to_string(),
                    description: "Потеря данных".to_string(),
                    threshold: "< 1 секунды".to_string(),
                    priority: CriterionPriority::Critical,
                },
                SuccessCriterion {
                    name: "Data Integrity".to_string(),
                    description: "Целостность данных".to_string(),
                    threshold: "100% транзакций сохранено".to_string(),
                    priority: CriterionPriority::Critical,
                },
            ],
            estimated_duration: Duration::from_secs(1800),
            required_participants: vec![
                "DBA".to_string(),
                "Platform Engineer".to_string(),
                "QA Engineer".to_string(),
            ],
        });

        // Сценарий 2: Полный отказ региона
        self.add_scenario(TestScenario {
            name: "Region Failover Test".to_string(),
            description: "Тестирование переключения на резервный регион".to_string(),
            test_type: DRTestType::Parallel,
            objectives: vec![
                "Проверить работоспособность DR-региона".to_string(),
                "Тестировать DNS-переключение".to_string(),
                "Проверить репликацию данных".to_string(),
            ],
            success_criteria: vec![
                SuccessCriterion {
                    name: "RTO".to_string(),
                    description: "Время восстановления региона".to_string(),
                    threshold: "< 5 минут".to_string(),
                    priority: CriterionPriority::Critical,
                },
                SuccessCriterion {
                    name: "Service Availability".to_string(),
                    description: "Доступность сервисов".to_string(),
                    threshold: "100% критических сервисов".to_string(),
                    priority: CriterionPriority::Critical,
                },
            ],
            estimated_duration: Duration::from_secs(7200),
            required_participants: vec![
                "Incident Commander".to_string(),
                "Network Engineer".to_string(),
                "DBA".to_string(),
                "Platform Team".to_string(),
            ],
        });

        // Сценарий 3: Кибератака
        self.add_scenario(TestScenario {
            name: "Ransomware Recovery Test".to_string(),
            description: "Тестирование восстановления после ransomware-атаки".to_string(),
            test_type: DRTestType::Tabletop,
            objectives: vec![
                "Проверить процедуру изоляции систем".to_string(),
                "Тестировать восстановление из air-gapped бэкапов".to_string(),
                "Проверить процедуры коммуникации".to_string(),
            ],
            success_criteria: vec![
                SuccessCriterion {
                    name: "Isolation Time".to_string(),
                    description: "Время изоляции заражённых систем".to_string(),
                    threshold: "< 15 минут".to_string(),
                    priority: CriterionPriority::Critical,
                },
                SuccessCriterion {
                    name: "Recovery Time".to_string(),
                    description: "Полное восстановление".to_string(),
                    threshold: "< 4 часа".to_string(),
                    priority: CriterionPriority::Major,
                },
            ],
            estimated_duration: Duration::from_secs(3600),
            required_participants: vec![
                "Security Team".to_string(),
                "Executive Team".to_string(),
                "Legal".to_string(),
                "Communications".to_string(),
            ],
        });
    }

    /// Выполнить DR-тест
    pub fn execute_test(&mut self, scenario_name: &str) -> Option<DRTestResult> {
        let scenario = self.scenarios.iter()
            .find(|s| s.name == scenario_name)?
            .clone();

        println!("\n=== Запуск DR-теста: {} ===", scenario.name);
        println!("Тип: {:?}", scenario.test_type);
        println!("Риск: {}", scenario.test_type.risk_level());
        println!("Влияние на production: {}", scenario.test_type.production_impact());

        println!("\nЦели теста:");
        for (i, obj) in scenario.objectives.iter().enumerate() {
            println!("  {}. {}", i + 1, obj);
        }

        println!("\nУчастники:");
        for p in &scenario.required_participants {
            println!("  - {}", p);
        }

        let started_at = Utc::now();
        let start_time = Instant::now();

        // Симуляция выполнения теста
        println!("\n--- Выполнение теста ---");

        let mut criteria_results = HashMap::new();
        let mut issues = Vec::new();

        for criterion in &scenario.success_criteria {
            println!("\nПроверка критерия: {}", criterion.name);

            // Симуляция результата
            let (passed, actual_value) = self.simulate_criterion_check(criterion);

            println!("  Порог: {}", criterion.threshold);
            println!("  Результат: {}", actual_value);
            println!("  Статус: {}", if passed { "✓ PASS" } else { "✗ FAIL" });

            if !passed {
                issues.push(TestIssue {
                    severity: match criterion.priority {
                        CriterionPriority::Critical => IssueSeverity::Critical,
                        CriterionPriority::Major => IssueSeverity::High,
                        CriterionPriority::Minor => IssueSeverity::Medium,
                    },
                    description: format!("Критерий {} не выполнен", criterion.name),
                    affected_component: "DR System".to_string(),
                    remediation: "Требуется анализ и улучшение".to_string(),
                });
            }

            criteria_results.insert(criterion.name.clone(), CriterionResult {
                passed,
                actual_value,
                notes: String::new(),
            });
        }

        let duration = start_time.elapsed();
        let completed_at = Utc::now();

        // Симуляция измеренных RTO/RPO
        let actual_rto = Duration::from_secs(45);
        let actual_rpo = Duration::from_millis(500);

        let overall_success = criteria_results.values()
            .all(|r| r.passed);

        let recommendations = if overall_success {
            vec!["Продолжать регулярное тестирование".to_string()]
        } else {
            vec![
                "Улучшить автоматизацию failover".to_string(),
                "Увеличить частоту DR-тренировок".to_string(),
                "Обновить runbook с учётом найденных проблем".to_string(),
            ]
        };

        let result = DRTestResult {
            scenario_name: scenario.name,
            test_type: scenario.test_type,
            started_at,
            completed_at,
            overall_success,
            criteria_results,
            actual_rto,
            actual_rpo,
            issues_found: issues,
            recommendations,
        };

        println!("\n--- Результаты теста ---");
        println!("Статус: {}", if result.overall_success { "УСПЕХ" } else { "НЕУДАЧА" });
        println!("Фактический RTO: {:?}", result.actual_rto);
        println!("Фактический RPO: {:?}", result.actual_rpo);
        println!("Целевой RTO: {:?}", self.target_rto);
        println!("Целевой RPO: {:?}", self.target_rpo);

        if !result.issues_found.is_empty() {
            println!("\nНайденные проблемы:");
            for issue in &result.issues_found {
                println!("  [{:?}] {}", issue.severity, issue.description);
            }
        }

        self.test_history.push(result.clone());

        Some(result)
    }

    fn simulate_criterion_check(&self, criterion: &SuccessCriterion) -> (bool, String) {
        // Симуляция проверки критерия
        match criterion.name.as_str() {
            "RTO" => {
                let actual = 45;
                (actual < 60, format!("{} секунд", actual))
            }
            "RPO" => {
                let actual = 0.5;
                (actual < 1.0, format!("{} секунд", actual))
            }
            "Data Integrity" => {
                let actual = 100.0;
                (actual == 100.0, format!("{}%", actual))
            }
            "Service Availability" => {
                let actual = 100.0;
                (actual == 100.0, format!("{}%", actual))
            }
            _ => (true, "OK".to_string()),
        }
    }

    /// Генерация отчёта о DR-готовности
    pub fn generate_readiness_report(&self) {
        println!("\n" + &"=".repeat(50));
        println!("=== ОТЧЁТ О DR-ГОТОВНОСТИ ===");
        println!("{}", "=".repeat(50));

        println!("\nЦелевые показатели:");
        println!("  RTO: {:?}", self.target_rto);
        println!("  RPO: {:?}", self.target_rpo);

        println!("\nИстория тестирования:");
        if self.test_history.is_empty() {
            println!("  Тесты не проводились");
        } else {
            for result in &self.test_history {
                let status = if result.overall_success { "✓" } else { "✗" };
                println!("\n  {} {} ({})",
                    status,
                    result.scenario_name,
                    result.started_at.format("%Y-%m-%d"));
                println!("    RTO: {:?} (цель: {:?})",
                    result.actual_rto, self.target_rto);
                println!("    RPO: {:?} (цель: {:?})",
                    result.actual_rpo, self.target_rpo);
            }
        }

        // Расчёт общей готовности
        let total_tests = self.test_history.len();
        let successful_tests = self.test_history.iter()
            .filter(|r| r.overall_success)
            .count();

        let readiness_score = if total_tests > 0 {
            (successful_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        println!("\n" + &"-".repeat(50));
        println!("ОЦЕНКА DR-ГОТОВНОСТИ: {:.0}%", readiness_score);
        println!("{}", "-".repeat(50));

        let status = if readiness_score >= 90.0 {
            "ГОТОВ к DR"
        } else if readiness_score >= 70.0 {
            "ЧАСТИЧНО ГОТОВ"
        } else {
            "НЕ ГОТОВ к DR"
        };
        println!("Статус: {}", status);
    }
}

fn main() {
    println!("=== DR-тестирование торговой системы ===\n");

    let mut coordinator = DRTestCoordinator::new(
        Duration::from_secs(60),  // Целевой RTO: 1 минута
        Duration::from_secs(1),   // Целевой RPO: 1 секунда
    );

    // Создание стандартных сценариев
    coordinator.create_trading_scenarios();

    // Выполнение теста
    coordinator.execute_test("Database Failover Test");

    // Генерация отчёта
    coordinator.generate_readiness_report();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **RTO (Recovery Time Objective)** | Максимально допустимое время простоя системы |
| **RPO (Recovery Point Objective)** | Максимально допустимая потеря данных |
| **MTD (Maximum Tolerable Downtime)** | Критический порог простоя для бизнеса |
| **DR Runbook** | Документированная процедура восстановления |
| **Failover** | Автоматическое переключение на резервную систему |
| **Failback** | Возврат на основную систему после восстановления |
| **DR-тестирование** | Регулярная проверка готовности к катастрофам |
| **Tier-классификация** | Приоритизация систем по критичности |

## Практические задания

1. **DR Dashboard**: Создай панель мониторинга, которая:
   - Отображает статус всех критических систем
   - Показывает текущие RTO/RPO метрики
   - Уведомляет о нарушении SLA
   - Визуализирует историю инцидентов

2. **Автоматический Runbook**: Реализуй систему:
   - Автоматическое выполнение процедур восстановления
   - Валидация каждого шага
   - Автоматический rollback при ошибках
   - Интеграция с системой оповещений

3. **DR-оркестратор**: Построй систему:
   - Координация failover между компонентами
   - Соблюдение порядка зависимостей
   - Параллельное восстановление независимых систем
   - Мониторинг прогресса восстановления

4. **Chaos Engineering**: Создай платформу:
   - Контролируемое внедрение сбоев
   - Тестирование устойчивости системы
   - Автоматическое восстановление
   - Метрики и отчёты о стабильности

## Домашнее задание

1. **Полная DR-инфраструктура**: Разработай систему, которая:
   - Поддерживает multi-region failover
   - Автоматически синхронизирует данные между регионами
   - Определяет оптимальный момент для failover
   - Минимизирует потерю данных при переключении
   - Обеспечивает корректный failback
   - Генерирует детальные post-mortem отчёты

2. **DR-симулятор**: Напиши инструмент для:
   - Моделирования различных сценариев катастроф
   - Оценки влияния на бизнес-процессы
   - Расчёта стоимости простоя
   - Сравнения разных стратегий восстановления
   - Рекомендаций по улучшению DR

3. **Интеллектуальный failover**: Реализуй систему:
   - Анализ паттернов сбоев
   - Предсказание потенциальных проблем
   - Превентивный failover до наступления катастрофы
   - Машинное обучение для оптимизации решений
   - A/B тестирование стратегий восстановления

4. **Compliance DR Framework**: Создай фреймворк:
   - Соответствие регуляторным требованиям (SOC2, ISO 27001)
   - Автоматическая генерация отчётов для аудита
   - Отслеживание DR-метрик во времени
   - Интеграция с системами управления рисками
   - Планирование и отслеживание DR-тестирования

## Навигация

[← Предыдущий день](../356-backups/ru.md) | [Следующий день →](../358-horizontal-scaling/ru.md)
