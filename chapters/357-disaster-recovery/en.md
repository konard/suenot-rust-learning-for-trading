# Day 357: Disaster Recovery

## Trading Analogy

Imagine you're managing a large cryptocurrency hedge fund. You have backups (which we covered in the previous chapter), but what happens if:

- The New York data center completely fails due to a hurricane
- All three of your servers are simultaneously infected with ransomware
- A critical bug in the code zeros out all client balances
- An attack on the exchange infrastructure you work with

**Without a Disaster Recovery plan:**
- Panic in the team — no one knows what to do
- Hours or days to recover
- Loss of critical trading windows
- Clients lose money on open positions
- Reputational damage

**With a Disaster Recovery plan:**
- Clear roles and responsibilities
- Automatic switchover to backup data center
- RTO (Recovery Time Objective) — recovery in minutes
- RPO (Recovery Point Objective) — lose at most the last few seconds of data
- Clients don't even notice the problem

| Trading | Disaster Recovery |
|---------|-------------------|
| **Hedging** | Backup systems in different locations |
| **Stop-loss** | Automatic failover |
| **Risk diversification** | Multi-cloud strategy |
| **Trading plan** | DR Runbook |
| **Trading simulator** | DR testing |
| **Position management** | System recovery prioritization |

In trading, every second of downtime is potential losses. DR ensures you can continue trading under any scenario.

## Disaster Recovery Fundamentals

### Key DR Metrics

```rust
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Key Disaster Recovery metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DRMetrics {
    /// Recovery Time Objective — maximum allowable downtime
    pub rto: Duration,
    /// Recovery Point Objective — maximum allowable data loss
    pub rpo: Duration,
    /// Maximum Tolerable Downtime — critical downtime threshold
    pub mtd: Duration,
    /// Work Recovery Time — time to return to full productivity
    pub wrt: Duration,
}

impl DRMetrics {
    /// Create metrics for a critical trading system
    pub fn critical_trading() -> Self {
        DRMetrics {
            rto: Duration::from_secs(60),      // 1 minute
            rpo: Duration::from_secs(1),       // 1 second
            mtd: Duration::from_secs(300),     // 5 minutes
            wrt: Duration::from_secs(600),     // 10 minutes
        }
    }

    /// Create metrics for a standard system
    pub fn standard_system() -> Self {
        DRMetrics {
            rto: Duration::from_secs(3600),    // 1 hour
            rpo: Duration::from_secs(300),     // 5 minutes
            mtd: Duration::from_secs(14400),   // 4 hours
            wrt: Duration::from_secs(28800),   // 8 hours
        }
    }

    /// Validate if metrics are achievable
    pub fn validate(&self) -> Result<(), String> {
        if self.rto > self.mtd {
            return Err("RTO cannot exceed MTD".to_string());
        }
        if self.rto + self.wrt > self.mtd {
            return Err("RTO + WRT exceeds MTD".to_string());
        }
        Ok(())
    }
}

/// System classification by criticality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemTier {
    /// Tier 1: Critical systems (order execution, risk management)
    Critical,
    /// Tier 2: Important systems (market data, reporting)
    Important,
    /// Tier 3: Supporting systems (analytics, backtesting)
    Supporting,
    /// Tier 4: Non-critical systems (dev environments, documentation)
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
            SystemTier::Critical => "Critical systems — immediate recovery",
            SystemTier::Important => "Important systems — recovery within an hour",
            SystemTier::Supporting => "Supporting — recovery within a day",
            SystemTier::NonCritical => "Non-critical — recovery within a week",
        }
    }
}

/// System component for DR planning
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
    println!("=== Disaster Recovery Metrics ===\n");

    // Define metrics for trading system
    let trading_metrics = DRMetrics::critical_trading();
    println!("Critical trading system:");
    println!("  RTO: {:?}", trading_metrics.rto);
    println!("  RPO: {:?}", trading_metrics.rpo);
    println!("  MTD: {:?}", trading_metrics.mtd);
    println!("  WRT: {:?}", trading_metrics.wrt);

    // Validate metrics
    match trading_metrics.validate() {
        Ok(()) => println!("  ✓ Metrics are valid\n"),
        Err(e) => println!("  ✗ Error: {}\n", e),
    }

    // Metrics by criticality level
    println!("Metrics by criticality level:");
    for tier in [SystemTier::Critical, SystemTier::Important,
                 SystemTier::Supporting, SystemTier::NonCritical] {
        let metrics = tier.default_metrics();
        println!("\n{:?}: {}", tier, tier.description());
        println!("  RTO: {:?}, RPO: {:?}", metrics.rto, metrics.rpo);
    }
}
```

## DR Runbook — Recovery Procedures

```rust
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Recovery procedure step
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

/// Runbook — set of recovery procedures
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

/// Types of disaster scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisasterScenario {
    /// Complete data center failure
    DatacenterFailure { location: String },
    /// Database failure
    DatabaseCorruption,
    /// Cyber attack
    CyberAttack { attack_type: String },
    /// Cloud provider outage
    CloudProviderOutage { provider: String },
    /// Critical application failure
    ApplicationFailure { component: String },
    /// Data loss
    DataLoss { scope: String },
}

/// Contact information for escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub role: String,
    pub phone: String,
    pub email: String,
    pub escalation_level: u8,
}

/// Runbook execution
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
    /// Create runbook for datacenter failure
    pub fn datacenter_failover(location: &str) -> Self {
        DRRunbook {
            name: format!("Datacenter Failover: {}", location),
            description: "Procedure for switching to backup data center".to_string(),
            scenario: DisasterScenario::DatacenterFailure {
                location: location.to_string(),
            },
            steps: vec![
                RecoveryStep {
                    order: 1,
                    name: "Assess Situation".to_string(),
                    description: "Confirm primary data center failure".to_string(),
                    responsible_role: "On-Call Engineer".to_string(),
                    estimated_duration_secs: 120,
                    commands: vec![
                        "ping primary-dc.trading.internal".to_string(),
                        "curl -s https://status.trading.internal/health".to_string(),
                    ],
                    verification: "Confirm primary DC is unreachable".to_string(),
                    rollback_procedure: None,
                },
                RecoveryStep {
                    order: 2,
                    name: "Activate DR Team".to_string(),
                    description: "Notify DR team and start incident".to_string(),
                    responsible_role: "Incident Commander".to_string(),
                    estimated_duration_secs: 300,
                    commands: vec![
                        "pagerduty trigger --severity critical".to_string(),
                        "slack notify #dr-team 'DC Failover initiated'".to_string(),
                    ],
                    verification: "All key participants in communication channel".to_string(),
                    rollback_procedure: None,
                },
                RecoveryStep {
                    order: 3,
                    name: "DNS Switchover".to_string(),
                    description: "Switch DNS to backup data center".to_string(),
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
                    name: "Activate DR Database".to_string(),
                    description: "Promote DB replica to primary".to_string(),
                    responsible_role: "DBA".to_string(),
                    estimated_duration_secs: 180,
                    commands: vec![
                        "pg_ctl promote -D /var/lib/postgresql/data".to_string(),
                        "psql -c 'SELECT pg_is_in_recovery()'".to_string(),
                    ],
                    verification: "Database is in primary mode".to_string(),
                    rollback_procedure: Some(
                        "Manual replication restoration with primary DC".to_string()
                    ),
                },
                RecoveryStep {
                    order: 5,
                    name: "Start Trading Services".to_string(),
                    description: "Start critical trading services in DR".to_string(),
                    responsible_role: "Platform Engineer".to_string(),
                    estimated_duration_secs: 120,
                    commands: vec![
                        "kubectl scale deployment order-engine --replicas=3".to_string(),
                        "kubectl scale deployment risk-manager --replicas=3".to_string(),
                        "kubectl rollout status deployment/order-engine".to_string(),
                    ],
                    verification: "All trading services are healthy".to_string(),
                    rollback_procedure: Some(
                        "kubectl scale deployment --all --replicas=0".to_string()
                    ),
                },
                RecoveryStep {
                    order: 6,
                    name: "Verify Functionality".to_string(),
                    description: "Check system is working correctly".to_string(),
                    responsible_role: "QA Engineer".to_string(),
                    estimated_duration_secs: 300,
                    commands: vec![
                        "./run-smoke-tests.sh".to_string(),
                        "./verify-order-execution.sh".to_string(),
                        "./check-market-data-feed.sh".to_string(),
                    ],
                    verification: "All smoke tests passed successfully".to_string(),
                    rollback_procedure: None,
                },
                RecoveryStep {
                    order: 7,
                    name: "Notify Stakeholders".to_string(),
                    description: "Inform clients and management".to_string(),
                    responsible_role: "Communications Lead".to_string(),
                    estimated_duration_secs: 60,
                    commands: vec![
                        "send-status-update --template dr-complete".to_string(),
                    ],
                    verification: "Notifications sent".to_string(),
                    rollback_procedure: None,
                },
            ],
            escalation_contacts: vec![
                Contact {
                    name: "John Smith".to_string(),
                    role: "CTO".to_string(),
                    phone: "+1-555-123-4567".to_string(),
                    email: "cto@trading.com".to_string(),
                    escalation_level: 1,
                },
                Contact {
                    name: "Jane Doe".to_string(),
                    role: "VP Engineering".to_string(),
                    phone: "+1-555-234-5678".to_string(),
                    email: "vp-eng@trading.com".to_string(),
                    escalation_level: 2,
                },
            ],
            last_tested: Some(Utc::now()),
            version: "2.1.0".to_string(),
        }
    }

    /// Calculate total recovery time
    pub fn estimated_recovery_time(&self) -> u64 {
        self.steps.iter().map(|s| s.estimated_duration_secs).sum()
    }

    /// Print runbook in readable format
    pub fn print_summary(&self) {
        println!("=== DR Runbook: {} ===", self.name);
        println!("Version: {}", self.version);
        println!("Description: {}", self.description);
        println!("Scenario: {:?}", self.scenario);
        println!("\nRecovery Steps:");

        for step in &self.steps {
            println!("\n{}. {} ({})", step.order, step.name, step.responsible_role);
            println!("   {}", step.description);
            println!("   Time: ~{} sec", step.estimated_duration_secs);
            println!("   Verification: {}", step.verification);
        }

        println!("\nTotal estimated time: {} minutes",
            self.estimated_recovery_time() / 60);

        println!("\nEscalation Contacts:");
        for contact in &self.escalation_contacts {
            println!("  L{}: {} ({}) - {}",
                contact.escalation_level, contact.name,
                contact.role, contact.phone);
        }
    }
}

impl RunbookExecution {
    pub fn start(runbook: DRRunbook) -> Self {
        println!("\n>>> STARTING DR PROCEDURE: {}", runbook.name);
        println!(">>> Start time: {}", Utc::now());

        RunbookExecution {
            runbook,
            started_at: Utc::now(),
            current_step: 0,
            step_results: Vec::new(),
            status: ExecutionStatus::InProgress,
        }
    }

    /// Execute next step
    pub fn execute_next_step(&mut self) -> Option<&StepResult> {
        if self.current_step >= self.runbook.steps.len() {
            return None;
        }

        let step = &self.runbook.steps[self.current_step];
        let started_at = Utc::now();

        println!("\n[Step {}/{}] {}",
            step.order, self.runbook.steps.len(), step.name);
        println!("  Responsible: {}", step.responsible_role);
        println!("  Executing commands:");

        for cmd in &step.commands {
            println!("    $ {}", cmd);
        }

        // Simulate execution (in reality, actual code would be here)
        let success = true; // Simulating success
        let completed_at = Utc::now();

        let result = StepResult {
            step_order: step.order,
            success,
            started_at,
            completed_at: Some(completed_at),
            output: format!("Step {} completed successfully", step.order),
            error: None,
        };

        println!("  Verification: {}", step.verification);
        println!("  Result: {}", if success { "✓ Success" } else { "✗ Error" });

        self.step_results.push(result);
        self.current_step += 1;

        // Check completion
        if self.current_step >= self.runbook.steps.len() {
            let duration = (Utc::now() - self.started_at).num_seconds() as u64;
            self.status = ExecutionStatus::Completed { duration_secs: duration };
            println!("\n>>> DR PROCEDURE COMPLETED");
            println!(">>> Total time: {} seconds", duration);
        }

        self.step_results.last()
    }

    /// Execute all steps
    pub fn execute_all(&mut self) {
        while self.execute_next_step().is_some() {}
    }
}

fn main() {
    println!("=== DR Runbook Demo ===\n");

    // Create runbook for datacenter failure
    let runbook = DRRunbook::datacenter_failover("NYC-DC-01");
    runbook.print_summary();

    // Simulate execution
    println!("\n" + &"=".repeat(50));
    let mut execution = RunbookExecution::start(runbook);
    execution.execute_all();
}
```

## Automatic Failover

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Node state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Cluster node
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
        // Simulate health check
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

/// Failover configuration
#[derive(Debug, Clone)]
pub struct FailoverConfig {
    /// Number of failed checks before failover
    pub failure_threshold: u64,
    /// Interval between checks
    pub check_interval: Duration,
    /// Timeout for health check
    pub health_check_timeout: Duration,
    /// Cooldown period between failovers
    pub cooldown_period: Duration,
    /// Whether confirmation is required for failover
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

/// Failover event
#[derive(Debug, Clone)]
pub struct FailoverEvent {
    pub timestamp: DateTime<Utc>,
    pub from_node: String,
    pub to_node: String,
    pub reason: String,
    pub duration_ms: u64,
    pub success: bool,
}

/// Automatic failover manager
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
            // Set primary when adding
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                *self.current_primary.write().await = Some(node.id.clone());
            });
        }
        self.nodes.push(node);
    }

    /// Get current primary node
    pub async fn get_primary(&self) -> Option<Arc<ClusterNode>> {
        let primary_id = self.current_primary.read().await;
        if let Some(id) = primary_id.as_ref() {
            self.nodes.iter().find(|n| &n.id == id).cloned()
        } else {
            None
        }
    }

    /// Check health of all nodes
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
                    println!("[HealthCheck] Node {} unreachable ({} consecutive)",
                        node.id, failures);

                    // Check if failover is needed
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

    /// Initiate failover
    pub async fn initiate_failover(&self, failed_node_id: &str) {
        // Check if failover is already in progress
        if self.is_failover_in_progress.swap(true, Ordering::SeqCst) {
            println!("[Failover] Failover already in progress, skipping");
            return;
        }

        // Check cooldown
        let last = self.last_failover.read().await;
        if let Some(last_time) = *last {
            let elapsed = Utc::now().signed_duration_since(last_time);
            if elapsed < chrono::Duration::from_std(self.config.cooldown_period).unwrap() {
                println!("[Failover] Cooldown period active, skipping");
                self.is_failover_in_progress.store(false, Ordering::SeqCst);
                return;
            }
        }
        drop(last);

        let start = Instant::now();
        println!("\n>>> FAILOVER INITIATED <<<");
        println!(">>> Failed node: {}", failed_node_id);

        // Find failover candidate
        let candidate = self.find_failover_candidate().await;

        match candidate {
            Some(new_primary) => {
                println!(">>> Selected new primary: {}", new_primary.id);

                // Perform failover
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
                    println!(">>> FAILOVER SUCCESSFUL ({}ms)", duration_ms);
                } else {
                    println!(">>> FAILOVER FAILED");
                }
            }
            None => {
                println!(">>> No available candidates for failover!");
            }
        }

        self.is_failover_in_progress.store(false, Ordering::SeqCst);
    }

    /// Find failover candidate
    async fn find_failover_candidate(&self) -> Option<Arc<ClusterNode>> {
        for node in &self.nodes {
            let state = *node.state.read().await;
            if node.role == NodeRole::Secondary && state == NodeState::Healthy {
                return Some(Arc::clone(node));
            }
        }
        None
    }

    /// Perform failover procedure
    async fn perform_failover(
        &self,
        _old_primary: &str,
        new_primary: &Arc<ClusterNode>,
    ) -> bool {
        println!("  1. Promoting {} to primary...", new_primary.id);
        // Simulate promotion
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("  2. Updating DNS...");
        tokio::time::sleep(Duration::from_millis(50)).await;

        println!("  3. Redirecting traffic...");
        tokio::time::sleep(Duration::from_millis(50)).await;

        println!("  4. Verifying...");
        tokio::time::sleep(Duration::from_millis(100)).await;

        true
    }

    /// Get failover statistics
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
        println!("\n=== Cluster Status ===");

        let primary = self.get_primary().await;
        println!("Primary: {}", primary.map_or("none".to_string(), |n| n.id.clone()));

        println!("\nNodes:");
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
        println!("\nFailover Statistics:");
        println!("  Total: {}", stats.total_failovers);
        println!("  Successful: {}", stats.successful_failovers);
        println!("  Failed: {}", stats.failed_failovers);
        println!("  Average duration: {}ms", stats.avg_duration_ms);
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
    println!("=== Automatic Failover ===\n");

    let config = FailoverConfig {
        failure_threshold: 2,
        check_interval: Duration::from_secs(2),
        cooldown_period: Duration::from_secs(30),
        ..Default::default()
    };

    let mut manager = FailoverManager::new(config);

    // Add cluster nodes
    let mut primary = ClusterNode::new("nyc-primary", "10.0.1.1:5432", NodeRole::Primary);
    primary.record_heartbeat(); // Simulate active node

    let mut secondary = ClusterNode::new("london-secondary", "10.0.2.1:5432", NodeRole::Secondary);
    secondary.record_heartbeat();

    let mut witness = ClusterNode::new("tokyo-witness", "10.0.3.1:5432", NodeRole::Witness);
    witness.record_heartbeat();

    manager.add_node(primary);
    manager.add_node(secondary);
    manager.add_node(witness);

    manager.print_status().await;

    // Simulate primary failure
    println!("\n>>> Simulating primary node failure...");

    // Simulate stale heartbeat
    if let Some(primary_node) = manager.nodes.iter().find(|n| n.role == NodeRole::Primary) {
        primary_node.last_heartbeat.store(0, Ordering::Relaxed);
    }

    // Health check (will trigger failover)
    for _ in 0..3 {
        println!("\n--- Health Check ---");
        let states = manager.check_all_nodes().await;
        for (id, state) in states {
            println!("  {} -> {:?}", id, state);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    manager.print_status().await;
}
```

## DR Testing

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// DR test type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DRTestType {
    /// Tabletop — plan discussion without actual execution
    Tabletop,
    /// Walkthrough — step-by-step procedure review
    Walkthrough,
    /// Simulation — simulation with partial execution
    Simulation,
    /// Parallel — parallel operation of DR system
    Parallel,
    /// Full — complete switchover to DR
    Full,
}

impl DRTestType {
    pub fn risk_level(&self) -> &'static str {
        match self {
            DRTestType::Tabletop => "Minimal",
            DRTestType::Walkthrough => "Low",
            DRTestType::Simulation => "Medium",
            DRTestType::Parallel => "Medium-High",
            DRTestType::Full => "High",
        }
    }

    pub fn production_impact(&self) -> &'static str {
        match self {
            DRTestType::Tabletop => "None",
            DRTestType::Walkthrough => "None",
            DRTestType::Simulation => "Minimal",
            DRTestType::Parallel => "Low",
            DRTestType::Full => "High",
        }
    }
}

/// Test scenario
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
    Critical,  // Must be met
    Major,     // Very important
    Minor,     // Desirable
}

/// DR test result
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

/// DR testing coordinator
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

    /// Create standard scenarios for trading system
    pub fn create_trading_scenarios(&mut self) {
        // Scenario 1: Primary DB failure
        self.add_scenario(TestScenario {
            name: "Database Failover Test".to_string(),
            description: "Testing switchover to backup database".to_string(),
            test_type: DRTestType::Simulation,
            objectives: vec![
                "Verify automatic PostgreSQL failover".to_string(),
                "Measure switchover time".to_string(),
                "Verify data integrity after failover".to_string(),
            ],
            success_criteria: vec![
                SuccessCriterion {
                    name: "RTO".to_string(),
                    description: "Recovery time".to_string(),
                    threshold: "< 60 seconds".to_string(),
                    priority: CriterionPriority::Critical,
                },
                SuccessCriterion {
                    name: "RPO".to_string(),
                    description: "Data loss".to_string(),
                    threshold: "< 1 second".to_string(),
                    priority: CriterionPriority::Critical,
                },
                SuccessCriterion {
                    name: "Data Integrity".to_string(),
                    description: "Data integrity".to_string(),
                    threshold: "100% transactions preserved".to_string(),
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

        // Scenario 2: Full region failure
        self.add_scenario(TestScenario {
            name: "Region Failover Test".to_string(),
            description: "Testing switchover to backup region".to_string(),
            test_type: DRTestType::Parallel,
            objectives: vec![
                "Verify DR region operability".to_string(),
                "Test DNS switchover".to_string(),
                "Verify data replication".to_string(),
            ],
            success_criteria: vec![
                SuccessCriterion {
                    name: "RTO".to_string(),
                    description: "Region recovery time".to_string(),
                    threshold: "< 5 minutes".to_string(),
                    priority: CriterionPriority::Critical,
                },
                SuccessCriterion {
                    name: "Service Availability".to_string(),
                    description: "Service availability".to_string(),
                    threshold: "100% critical services".to_string(),
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

        // Scenario 3: Cyber attack
        self.add_scenario(TestScenario {
            name: "Ransomware Recovery Test".to_string(),
            description: "Testing recovery after ransomware attack".to_string(),
            test_type: DRTestType::Tabletop,
            objectives: vec![
                "Verify system isolation procedure".to_string(),
                "Test recovery from air-gapped backups".to_string(),
                "Verify communication procedures".to_string(),
            ],
            success_criteria: vec![
                SuccessCriterion {
                    name: "Isolation Time".to_string(),
                    description: "Time to isolate infected systems".to_string(),
                    threshold: "< 15 minutes".to_string(),
                    priority: CriterionPriority::Critical,
                },
                SuccessCriterion {
                    name: "Recovery Time".to_string(),
                    description: "Full recovery".to_string(),
                    threshold: "< 4 hours".to_string(),
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

    /// Execute DR test
    pub fn execute_test(&mut self, scenario_name: &str) -> Option<DRTestResult> {
        let scenario = self.scenarios.iter()
            .find(|s| s.name == scenario_name)?
            .clone();

        println!("\n=== Starting DR Test: {} ===", scenario.name);
        println!("Type: {:?}", scenario.test_type);
        println!("Risk: {}", scenario.test_type.risk_level());
        println!("Production impact: {}", scenario.test_type.production_impact());

        println!("\nTest objectives:");
        for (i, obj) in scenario.objectives.iter().enumerate() {
            println!("  {}. {}", i + 1, obj);
        }

        println!("\nParticipants:");
        for p in &scenario.required_participants {
            println!("  - {}", p);
        }

        let started_at = Utc::now();
        let start_time = Instant::now();

        // Simulate test execution
        println!("\n--- Executing test ---");

        let mut criteria_results = HashMap::new();
        let mut issues = Vec::new();

        for criterion in &scenario.success_criteria {
            println!("\nChecking criterion: {}", criterion.name);

            // Simulate result
            let (passed, actual_value) = self.simulate_criterion_check(criterion);

            println!("  Threshold: {}", criterion.threshold);
            println!("  Result: {}", actual_value);
            println!("  Status: {}", if passed { "✓ PASS" } else { "✗ FAIL" });

            if !passed {
                issues.push(TestIssue {
                    severity: match criterion.priority {
                        CriterionPriority::Critical => IssueSeverity::Critical,
                        CriterionPriority::Major => IssueSeverity::High,
                        CriterionPriority::Minor => IssueSeverity::Medium,
                    },
                    description: format!("Criterion {} not met", criterion.name),
                    affected_component: "DR System".to_string(),
                    remediation: "Analysis and improvement required".to_string(),
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

        // Simulated measured RTO/RPO
        let actual_rto = Duration::from_secs(45);
        let actual_rpo = Duration::from_millis(500);

        let overall_success = criteria_results.values()
            .all(|r| r.passed);

        let recommendations = if overall_success {
            vec!["Continue regular testing".to_string()]
        } else {
            vec![
                "Improve failover automation".to_string(),
                "Increase DR drill frequency".to_string(),
                "Update runbook based on issues found".to_string(),
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

        println!("\n--- Test Results ---");
        println!("Status: {}", if result.overall_success { "SUCCESS" } else { "FAILURE" });
        println!("Actual RTO: {:?}", result.actual_rto);
        println!("Actual RPO: {:?}", result.actual_rpo);
        println!("Target RTO: {:?}", self.target_rto);
        println!("Target RPO: {:?}", self.target_rpo);

        if !result.issues_found.is_empty() {
            println!("\nIssues found:");
            for issue in &result.issues_found {
                println!("  [{:?}] {}", issue.severity, issue.description);
            }
        }

        self.test_history.push(result.clone());

        Some(result)
    }

    fn simulate_criterion_check(&self, criterion: &SuccessCriterion) -> (bool, String) {
        // Simulate criterion check
        match criterion.name.as_str() {
            "RTO" => {
                let actual = 45;
                (actual < 60, format!("{} seconds", actual))
            }
            "RPO" => {
                let actual = 0.5;
                (actual < 1.0, format!("{} seconds", actual))
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

    /// Generate DR readiness report
    pub fn generate_readiness_report(&self) {
        println!("\n" + &"=".repeat(50));
        println!("=== DR READINESS REPORT ===");
        println!("{}", "=".repeat(50));

        println!("\nTarget metrics:");
        println!("  RTO: {:?}", self.target_rto);
        println!("  RPO: {:?}", self.target_rpo);

        println!("\nTest history:");
        if self.test_history.is_empty() {
            println!("  No tests conducted");
        } else {
            for result in &self.test_history {
                let status = if result.overall_success { "✓" } else { "✗" };
                println!("\n  {} {} ({})",
                    status,
                    result.scenario_name,
                    result.started_at.format("%Y-%m-%d"));
                println!("    RTO: {:?} (target: {:?})",
                    result.actual_rto, self.target_rto);
                println!("    RPO: {:?} (target: {:?})",
                    result.actual_rpo, self.target_rpo);
            }
        }

        // Calculate overall readiness
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
        println!("DR READINESS SCORE: {:.0}%", readiness_score);
        println!("{}", "-".repeat(50));

        let status = if readiness_score >= 90.0 {
            "READY for DR"
        } else if readiness_score >= 70.0 {
            "PARTIALLY READY"
        } else {
            "NOT READY for DR"
        };
        println!("Status: {}", status);
    }
}

fn main() {
    println!("=== DR Testing for Trading System ===\n");

    let mut coordinator = DRTestCoordinator::new(
        Duration::from_secs(60),  // Target RTO: 1 minute
        Duration::from_secs(1),   // Target RPO: 1 second
    );

    // Create standard scenarios
    coordinator.create_trading_scenarios();

    // Execute test
    coordinator.execute_test("Database Failover Test");

    // Generate report
    coordinator.generate_readiness_report();
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **RTO (Recovery Time Objective)** | Maximum allowable system downtime |
| **RPO (Recovery Point Objective)** | Maximum allowable data loss |
| **MTD (Maximum Tolerable Downtime)** | Critical business downtime threshold |
| **DR Runbook** | Documented recovery procedure |
| **Failover** | Automatic switchover to backup system |
| **Failback** | Return to primary system after recovery |
| **DR Testing** | Regular disaster readiness verification |
| **Tier Classification** | System prioritization by criticality |

## Practical Exercises

1. **DR Dashboard**: Create a monitoring panel that:
   - Displays status of all critical systems
   - Shows current RTO/RPO metrics
   - Notifies about SLA violations
   - Visualizes incident history

2. **Automated Runbook**: Implement a system:
   - Automatic execution of recovery procedures
   - Validation of each step
   - Automatic rollback on errors
   - Integration with alerting system

3. **DR Orchestrator**: Build a system:
   - Coordination of failover between components
   - Dependency order compliance
   - Parallel recovery of independent systems
   - Recovery progress monitoring

4. **Chaos Engineering**: Create a platform:
   - Controlled fault injection
   - System resilience testing
   - Automatic recovery
   - Stability metrics and reports

## Homework

1. **Complete DR Infrastructure**: Develop a system that:
   - Supports multi-region failover
   - Automatically synchronizes data between regions
   - Determines optimal failover timing
   - Minimizes data loss during switchover
   - Ensures correct failback
   - Generates detailed post-mortem reports

2. **DR Simulator**: Write a tool for:
   - Modeling various disaster scenarios
   - Assessing impact on business processes
   - Calculating downtime cost
   - Comparing different recovery strategies
   - Recommendations for DR improvement

3. **Intelligent Failover**: Implement a system:
   - Failure pattern analysis
   - Predicting potential problems
   - Preventive failover before disaster
   - Machine learning for decision optimization
   - A/B testing of recovery strategies

4. **Compliance DR Framework**: Create a framework:
   - Regulatory compliance (SOC2, ISO 27001)
   - Automatic audit report generation
   - DR metrics tracking over time
   - Integration with risk management systems
   - DR testing planning and tracking

## Navigation

[← Previous day](../356-backups/en.md) | [Next day →](../358-horizontal-scaling/en.md)
