# Day 362: Post-Mortem: Incident Analysis

## Trading Analogy

Imagine you're running a trading desk and one of your algorithms suddenly executes 10,000 unintended trades in 30 seconds, losing $2 million. After stopping the bleeding, what happens next?

**A Post-Mortem is like a trading floor investigation after a major loss:**

Just as experienced traders analyze every detail after a market disaster — what happened, when, why, and how to prevent it in the future — software engineers conduct post-mortem analyses to understand system failures and prevent them from recurring.

| Trading Investigation | Software Post-Mortem |
|----------------------|---------------------|
| **Loss calculation** | Impact assessment (downtime, data loss, revenue) |
| **Trade reconstruction** | Timeline reconstruction from logs |
| **Root cause analysis** | Finding the actual bug/failure point |
| **Risk control failure** | Why didn't alerts/safeguards catch it? |
| **Process improvement** | Action items to prevent recurrence |

**The Blameless Principle:**
A good post-mortem focuses on systems, not people. Just as you wouldn't blame a trader for following flawed risk models — you fix the models instead.

## Post-Mortem Structure

```rust
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};

/// Severity levels for incidents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Minor impact, no customer-facing issues
    Low,
    /// Some customers affected, workaround available
    Medium,
    /// Significant impact, partial service degradation
    High,
    /// Critical failure, full service outage
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "LOW",
            Severity::Medium => "MEDIUM",
            Severity::High => "HIGH",
            Severity::Critical => "CRITICAL",
        }
    }
}

/// Incident status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentStatus {
    /// Incident is being investigated
    Investigating,
    /// Root cause identified
    Identified,
    /// Fix is being implemented
    Fixing,
    /// Fix deployed, monitoring
    Monitoring,
    /// Incident resolved
    Resolved,
    /// Post-mortem complete
    Closed,
}

/// Timeline event during incident
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub actor: String,
    pub event_type: EventType,
}

#[derive(Debug, Clone)]
pub enum EventType {
    Detection,
    Alert,
    Investigation,
    Action,
    Communication,
    Resolution,
}

/// Impact metrics for the incident
#[derive(Debug, Clone)]
pub struct IncidentImpact {
    /// Duration of the incident
    pub duration: Duration,
    /// Estimated revenue loss
    pub revenue_loss: f64,
    /// Number of affected customers
    pub affected_customers: u64,
    /// Number of failed transactions
    pub failed_transactions: u64,
    /// Data loss (if any)
    pub data_loss: Option<String>,
    /// SLA breach
    pub sla_breach: bool,
}

/// Root cause categories
#[derive(Debug, Clone)]
pub enum RootCauseCategory {
    /// Code bug or logic error
    CodeDefect,
    /// Infrastructure failure (hardware, network)
    Infrastructure,
    /// Configuration error
    Configuration,
    /// External service failure
    ExternalDependency,
    /// Human error during operation
    OperationalError,
    /// Capacity/scaling issues
    Capacity,
    /// Security incident
    Security,
    /// Unknown (still investigating)
    Unknown,
}

/// Action item from post-mortem
#[derive(Debug, Clone)]
pub struct ActionItem {
    pub id: String,
    pub description: String,
    pub owner: String,
    pub priority: Priority,
    pub due_date: Option<DateTime<Utc>>,
    pub status: ActionStatus,
}

#[derive(Debug, Clone, Copy)]
pub enum Priority {
    P0, // Immediate
    P1, // This week
    P2, // This month
    P3, // Backlog
}

#[derive(Debug, Clone, Copy)]
pub enum ActionStatus {
    Open,
    InProgress,
    Done,
    WontFix,
}

/// Complete post-mortem document
#[derive(Debug, Clone)]
pub struct PostMortem {
    /// Unique incident identifier
    pub incident_id: String,
    /// Brief title describing the incident
    pub title: String,
    /// Detailed summary
    pub summary: String,
    /// Severity level
    pub severity: Severity,
    /// Current status
    pub status: IncidentStatus,
    /// When incident started
    pub started_at: DateTime<Utc>,
    /// When incident was detected
    pub detected_at: DateTime<Utc>,
    /// When incident was resolved
    pub resolved_at: Option<DateTime<Utc>>,
    /// Timeline of events
    pub timeline: Vec<TimelineEvent>,
    /// Impact assessment
    pub impact: IncidentImpact,
    /// Root cause category
    pub root_cause_category: RootCauseCategory,
    /// Detailed root cause analysis
    pub root_cause_analysis: String,
    /// What went well
    pub what_went_well: Vec<String>,
    /// What went wrong
    pub what_went_wrong: Vec<String>,
    /// Where we got lucky
    pub where_we_got_lucky: Vec<String>,
    /// Action items
    pub action_items: Vec<ActionItem>,
    /// Post-mortem author
    pub author: String,
    /// Participants in the incident
    pub participants: Vec<String>,
}

impl PostMortem {
    pub fn new(incident_id: &str, title: &str, severity: Severity) -> Self {
        PostMortem {
            incident_id: incident_id.to_string(),
            title: title.to_string(),
            summary: String::new(),
            severity,
            status: IncidentStatus::Investigating,
            started_at: Utc::now(),
            detected_at: Utc::now(),
            resolved_at: None,
            timeline: Vec::new(),
            impact: IncidentImpact {
                duration: Duration::from_secs(0),
                revenue_loss: 0.0,
                affected_customers: 0,
                failed_transactions: 0,
                data_loss: None,
                sla_breach: false,
            },
            root_cause_category: RootCauseCategory::Unknown,
            root_cause_analysis: String::new(),
            what_went_well: Vec::new(),
            what_went_wrong: Vec::new(),
            where_we_got_lucky: Vec::new(),
            action_items: Vec::new(),
            author: String::new(),
            participants: Vec::new(),
        }
    }

    pub fn add_timeline_event(&mut self, description: &str, actor: &str, event_type: EventType) {
        self.timeline.push(TimelineEvent {
            timestamp: Utc::now(),
            description: description.to_string(),
            actor: actor.to_string(),
            event_type,
        });
    }

    pub fn time_to_detection(&self) -> Duration {
        let started = self.started_at.timestamp() as u64;
        let detected = self.detected_at.timestamp() as u64;
        Duration::from_secs(detected.saturating_sub(started))
    }

    pub fn time_to_resolution(&self) -> Option<Duration> {
        self.resolved_at.map(|resolved| {
            let started = self.started_at.timestamp() as u64;
            let resolved_ts = resolved.timestamp() as u64;
            Duration::from_secs(resolved_ts.saturating_sub(started))
        })
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("# Post-Mortem: {}\n\n", self.title));
        report.push_str(&format!("**Incident ID:** {}\n", self.incident_id));
        report.push_str(&format!("**Severity:** {}\n", self.severity.as_str()));
        report.push_str(&format!("**Status:** {:?}\n", self.status));
        report.push_str(&format!("**Author:** {}\n\n", self.author));

        report.push_str("## Summary\n\n");
        report.push_str(&format!("{}\n\n", self.summary));

        report.push_str("## Timeline\n\n");
        for event in &self.timeline {
            report.push_str(&format!(
                "- **{}** [{}] {}: {}\n",
                event.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                format!("{:?}", event.event_type),
                event.actor,
                event.description
            ));
        }
        report.push_str("\n");

        report.push_str("## Impact\n\n");
        report.push_str(&format!("- **Duration:** {:?}\n", self.impact.duration));
        report.push_str(&format!("- **Revenue Loss:** ${:.2}\n", self.impact.revenue_loss));
        report.push_str(&format!("- **Affected Customers:** {}\n", self.impact.affected_customers));
        report.push_str(&format!("- **Failed Transactions:** {}\n", self.impact.failed_transactions));
        report.push_str(&format!("- **SLA Breach:** {}\n\n", self.impact.sla_breach));

        report.push_str("## Root Cause Analysis\n\n");
        report.push_str(&format!("**Category:** {:?}\n\n", self.root_cause_category));
        report.push_str(&format!("{}\n\n", self.root_cause_analysis));

        report.push_str("## What Went Well\n\n");
        for item in &self.what_went_well {
            report.push_str(&format!("- {}\n", item));
        }
        report.push_str("\n");

        report.push_str("## What Went Wrong\n\n");
        for item in &self.what_went_wrong {
            report.push_str(&format!("- {}\n", item));
        }
        report.push_str("\n");

        report.push_str("## Where We Got Lucky\n\n");
        for item in &self.where_we_got_lucky {
            report.push_str(&format!("- {}\n", item));
        }
        report.push_str("\n");

        report.push_str("## Action Items\n\n");
        for item in &self.action_items {
            report.push_str(&format!(
                "- [{}] **{}** (Owner: {}, Priority: {:?}): {}\n",
                if matches!(item.status, ActionStatus::Done) { "x" } else { " " },
                item.id,
                item.owner,
                item.priority,
                item.description
            ));
        }

        report
    }
}

fn main() {
    println!("=== Post-Mortem Structure Demo ===\n");

    let mut pm = PostMortem::new(
        "INC-2024-0142",
        "Order Matching Engine Outage",
        Severity::Critical
    );

    pm.summary = "The order matching engine became unresponsive for 47 minutes \
                  due to a memory leak in the order book reconstruction logic \
                  triggered by an unusual sequence of cancel-replace orders.".to_string();

    pm.author = "Jane Smith".to_string();
    pm.participants = vec!["Jane Smith".to_string(), "Bob Johnson".to_string(), "Alice Chen".to_string()];

    pm.add_timeline_event(
        "Monitoring alert: Order processing latency > 500ms",
        "PagerDuty",
        EventType::Alert
    );

    pm.add_timeline_event(
        "On-call engineer acknowledged alert",
        "Bob Johnson",
        EventType::Action
    );

    pm.impact = IncidentImpact {
        duration: Duration::from_secs(47 * 60),
        revenue_loss: 125000.0,
        affected_customers: 3247,
        failed_transactions: 15823,
        data_loss: None,
        sla_breach: true,
    };

    pm.root_cause_category = RootCauseCategory::CodeDefect;
    pm.root_cause_analysis = "A memory leak was introduced in commit abc123 when the \
        order book reconstruction logic was modified. When a cancel-replace order \
        sequence exceeded 1000 orders within a 1-second window, the old order objects \
        were not properly deallocated.".to_string();

    pm.what_went_well = vec![
        "Alert fired within 2 minutes of issue starting".to_string(),
        "Team assembled and started investigation quickly".to_string(),
        "Rollback procedure worked as expected".to_string(),
    ];

    pm.what_went_wrong = vec![
        "Memory leak was not caught in load testing".to_string(),
        "No circuit breaker for high cancel-replace rate".to_string(),
        "Initial investigation went down wrong path".to_string(),
    ];

    pm.where_we_got_lucky = vec![
        "Incident occurred during lower trading volume period".to_string(),
        "No data corruption occurred".to_string(),
    ];

    pm.action_items = vec![
        ActionItem {
            id: "AI-001".to_string(),
            description: "Add stress test for cancel-replace sequences".to_string(),
            owner: "Bob Johnson".to_string(),
            priority: Priority::P0,
            due_date: None,
            status: ActionStatus::Open,
        },
        ActionItem {
            id: "AI-002".to_string(),
            description: "Implement circuit breaker for order rate limiting".to_string(),
            owner: "Alice Chen".to_string(),
            priority: Priority::P1,
            due_date: None,
            status: ActionStatus::Open,
        },
    ];

    pm.status = IncidentStatus::Closed;
    pm.resolved_at = Some(Utc::now());

    println!("{}", pm.generate_report());
}
```

## Incident Detection and Tracking

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};

/// Trading-specific metrics for incident detection
#[derive(Debug, Clone)]
pub struct TradingMetrics {
    pub order_latency_ms: f64,
    pub order_success_rate: f64,
    pub orders_per_second: f64,
    pub position_pnl: f64,
    pub connection_count: u32,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

/// Threshold configuration for alerts
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_order_latency_ms: f64,
    pub min_success_rate: f64,
    pub max_orders_per_second: f64,
    pub max_loss_amount: f64,
    pub min_connections: u32,
    pub max_memory_mb: f64,
    pub max_cpu_percent: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        AlertThresholds {
            max_order_latency_ms: 100.0,
            min_success_rate: 95.0,
            max_orders_per_second: 10000.0,
            max_loss_amount: -100000.0,
            min_connections: 2,
            max_memory_mb: 8000.0,
            max_cpu_percent: 80.0,
        }
    }
}

/// Alert that was triggered
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub acknowledged: bool,
    pub resolved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertSeverity {
    Warning,
    Error,
    Critical,
}

/// Incident tracker for trading systems
pub struct IncidentTracker {
    thresholds: AlertThresholds,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
    incident_counter: Arc<RwLock<u64>>,
}

impl IncidentTracker {
    pub fn new(thresholds: AlertThresholds) -> Self {
        IncidentTracker {
            thresholds,
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            incident_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Check metrics and generate alerts
    pub fn check_metrics(&self, metrics: &TradingMetrics) -> Vec<Alert> {
        let mut new_alerts = Vec::new();

        // Check order latency
        if metrics.order_latency_ms > self.thresholds.max_order_latency_ms {
            new_alerts.push(self.create_alert(
                "high_latency",
                "Order Latency Critical",
                AlertSeverity::Critical,
                format!(
                    "Order latency {}ms exceeds threshold {}ms",
                    metrics.order_latency_ms, self.thresholds.max_order_latency_ms
                ),
                metrics.order_latency_ms,
                self.thresholds.max_order_latency_ms,
            ));
        }

        // Check success rate
        if metrics.order_success_rate < self.thresholds.min_success_rate {
            new_alerts.push(self.create_alert(
                "low_success_rate",
                "Order Success Rate Low",
                AlertSeverity::Error,
                format!(
                    "Success rate {:.1}% below threshold {:.1}%",
                    metrics.order_success_rate, self.thresholds.min_success_rate
                ),
                metrics.order_success_rate,
                self.thresholds.min_success_rate,
            ));
        }

        // Check for excessive order rate (possible runaway algo)
        if metrics.orders_per_second > self.thresholds.max_orders_per_second {
            new_alerts.push(self.create_alert(
                "excessive_orders",
                "Excessive Order Rate",
                AlertSeverity::Critical,
                format!(
                    "Order rate {:.0}/s exceeds safety limit {:.0}/s",
                    metrics.orders_per_second, self.thresholds.max_orders_per_second
                ),
                metrics.orders_per_second,
                self.thresholds.max_orders_per_second,
            ));
        }

        // Check P&L loss threshold
        if metrics.position_pnl < self.thresholds.max_loss_amount {
            new_alerts.push(self.create_alert(
                "excessive_loss",
                "P&L Loss Threshold Breached",
                AlertSeverity::Critical,
                format!(
                    "P&L ${:.2} exceeds loss limit ${:.2}",
                    metrics.position_pnl, self.thresholds.max_loss_amount
                ),
                metrics.position_pnl,
                self.thresholds.max_loss_amount,
            ));
        }

        // Check connection count
        if metrics.connection_count < self.thresholds.min_connections {
            new_alerts.push(self.create_alert(
                "low_connections",
                "Exchange Connections Lost",
                AlertSeverity::Critical,
                format!(
                    "Only {} connections active, minimum required {}",
                    metrics.connection_count, self.thresholds.min_connections
                ),
                metrics.connection_count as f64,
                self.thresholds.min_connections as f64,
            ));
        }

        // Check memory usage
        if metrics.memory_usage_mb > self.thresholds.max_memory_mb {
            new_alerts.push(self.create_alert(
                "high_memory",
                "High Memory Usage",
                AlertSeverity::Warning,
                format!(
                    "Memory usage {}MB exceeds threshold {}MB",
                    metrics.memory_usage_mb, self.thresholds.max_memory_mb
                ),
                metrics.memory_usage_mb,
                self.thresholds.max_memory_mb,
            ));
        }

        // Store new alerts
        let mut active = self.active_alerts.write().unwrap();
        let mut history = self.alert_history.write().unwrap();

        for alert in &new_alerts {
            if !active.contains_key(&alert.id) {
                active.insert(alert.id.clone(), alert.clone());
                history.push(alert.clone());
            }
        }

        new_alerts
    }

    fn create_alert(
        &self,
        id: &str,
        name: &str,
        severity: AlertSeverity,
        message: String,
        metric_value: f64,
        threshold_value: f64,
    ) -> Alert {
        Alert {
            id: id.to_string(),
            name: name.to_string(),
            severity,
            message,
            triggered_at: Utc::now(),
            metric_value,
            threshold_value,
            acknowledged: false,
            resolved: false,
        }
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&self, alert_id: &str, by: &str) {
        let mut active = self.active_alerts.write().unwrap();
        if let Some(alert) = active.get_mut(alert_id) {
            alert.acknowledged = true;
            println!("[{}] Alert '{}' acknowledged by {}", Utc::now().format("%H:%M:%S"), alert_id, by);
        }
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, alert_id: &str) {
        let mut active = self.active_alerts.write().unwrap();
        if let Some(alert) = active.remove(alert_id) {
            println!("[{}] Alert '{}' resolved after {:?}",
                Utc::now().format("%H:%M:%S"),
                alert_id,
                Utc::now().signed_duration_since(alert.triggered_at)
            );
        }
    }

    /// Get all active alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.read().unwrap().values().cloned().collect()
    }

    /// Generate incident summary
    pub fn generate_summary(&self) -> String {
        let active = self.active_alerts.read().unwrap();
        let history = self.alert_history.read().unwrap();

        let critical_count = active.values().filter(|a| a.severity == AlertSeverity::Critical).count();
        let error_count = active.values().filter(|a| a.severity == AlertSeverity::Error).count();
        let warning_count = active.values().filter(|a| a.severity == AlertSeverity::Warning).count();

        format!(
            "Incident Summary:\n\
             - Active Alerts: {} (Critical: {}, Error: {}, Warning: {})\n\
             - Total Alerts (historical): {}\n\
             - Unacknowledged: {}",
            active.len(), critical_count, error_count, warning_count,
            history.len(),
            active.values().filter(|a| !a.acknowledged).count()
        )
    }
}

fn main() {
    println!("=== Incident Detection Demo ===\n");

    let tracker = IncidentTracker::new(AlertThresholds::default());

    // Normal metrics - no alerts
    let normal_metrics = TradingMetrics {
        order_latency_ms: 25.0,
        order_success_rate: 99.5,
        orders_per_second: 500.0,
        position_pnl: 15000.0,
        connection_count: 5,
        memory_usage_mb: 4096.0,
        cpu_usage_percent: 45.0,
    };

    let alerts = tracker.check_metrics(&normal_metrics);
    println!("Normal metrics check: {} alerts", alerts.len());

    // Problematic metrics - multiple alerts
    let problem_metrics = TradingMetrics {
        order_latency_ms: 250.0,        // Too high
        order_success_rate: 85.0,       // Too low
        orders_per_second: 15000.0,     // Runaway algo?
        position_pnl: -150000.0,        // Excessive loss
        connection_count: 1,            // Lost connections
        memory_usage_mb: 4096.0,
        cpu_usage_percent: 45.0,
    };

    let alerts = tracker.check_metrics(&problem_metrics);
    println!("\nProblem metrics check: {} alerts\n", alerts.len());

    for alert in &alerts {
        println!("ALERT [{:?}] {}: {}", alert.severity, alert.name, alert.message);
    }

    // Acknowledge some alerts
    println!("\n--- Acknowledging alerts ---");
    tracker.acknowledge_alert("high_latency", "On-call Engineer");
    tracker.acknowledge_alert("excessive_loss", "Risk Manager");

    // Show summary
    println!("\n{}", tracker.generate_summary());

    // Resolve an alert
    println!("\n--- Resolving alert ---");
    tracker.resolve_alert("high_latency");

    println!("\n{}", tracker.generate_summary());
}
```

## Root Cause Analysis Framework

```rust
use std::collections::HashMap;

/// The "5 Whys" technique for root cause analysis
#[derive(Debug, Clone)]
pub struct FiveWhysAnalysis {
    pub incident_description: String,
    pub whys: Vec<WhyLevel>,
    pub root_cause: Option<String>,
    pub contributing_factors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WhyLevel {
    pub level: u8,
    pub question: String,
    pub answer: String,
    pub evidence: Vec<String>,
}

impl FiveWhysAnalysis {
    pub fn new(incident: &str) -> Self {
        FiveWhysAnalysis {
            incident_description: incident.to_string(),
            whys: Vec::new(),
            root_cause: None,
            contributing_factors: Vec::new(),
        }
    }

    pub fn add_why(&mut self, answer: &str, evidence: Vec<&str>) {
        let level = self.whys.len() as u8 + 1;
        let question = if level == 1 {
            format!("Why did {}?", self.incident_description)
        } else {
            format!("Why {}?", self.whys.last().unwrap().answer)
        };

        self.whys.push(WhyLevel {
            level,
            question,
            answer: answer.to_string(),
            evidence: evidence.into_iter().map(String::from).collect(),
        });
    }

    pub fn set_root_cause(&mut self, cause: &str) {
        self.root_cause = Some(cause.to_string());
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# 5 Whys Root Cause Analysis\n\n");
        report.push_str(&format!("**Incident:** {}\n\n", self.incident_description));

        for why in &self.whys {
            report.push_str(&format!("## Why #{}\n\n", why.level));
            report.push_str(&format!("**Q:** {}\n\n", why.question));
            report.push_str(&format!("**A:** {}\n\n", why.answer));

            if !why.evidence.is_empty() {
                report.push_str("**Evidence:**\n");
                for e in &why.evidence {
                    report.push_str(&format!("- {}\n", e));
                }
                report.push_str("\n");
            }
        }

        if let Some(root_cause) = &self.root_cause {
            report.push_str(&format!("## Root Cause\n\n{}\n\n", root_cause));
        }

        if !self.contributing_factors.is_empty() {
            report.push_str("## Contributing Factors\n\n");
            for factor in &self.contributing_factors {
                report.push_str(&format!("- {}\n", factor));
            }
        }

        report
    }
}

/// Fishbone (Ishikawa) diagram for cause analysis
#[derive(Debug, Clone)]
pub struct FishboneDiagram {
    pub problem: String,
    pub categories: HashMap<CauseCategory, Vec<Cause>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CauseCategory {
    People,      // Human factors
    Process,     // Methodology, procedures
    Technology,  // Tools, systems, infrastructure
    Data,        // Data quality, availability
    Environment, // External factors, market conditions
    Management,  // Organizational, decision-making
}

impl CauseCategory {
    pub fn all() -> Vec<CauseCategory> {
        vec![
            CauseCategory::People,
            CauseCategory::Process,
            CauseCategory::Technology,
            CauseCategory::Data,
            CauseCategory::Environment,
            CauseCategory::Management,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Cause {
    pub description: String,
    pub is_root_cause: bool,
    pub sub_causes: Vec<String>,
}

impl FishboneDiagram {
    pub fn new(problem: &str) -> Self {
        let mut categories = HashMap::new();
        for cat in CauseCategory::all() {
            categories.insert(cat, Vec::new());
        }

        FishboneDiagram {
            problem: problem.to_string(),
            categories,
        }
    }

    pub fn add_cause(&mut self, category: CauseCategory, description: &str, is_root: bool) {
        if let Some(causes) = self.categories.get_mut(&category) {
            causes.push(Cause {
                description: description.to_string(),
                is_root_cause: is_root,
                sub_causes: Vec::new(),
            });
        }
    }

    pub fn add_sub_cause(&mut self, category: CauseCategory, cause_idx: usize, sub_cause: &str) {
        if let Some(causes) = self.categories.get_mut(&category) {
            if let Some(cause) = causes.get_mut(cause_idx) {
                cause.sub_causes.push(sub_cause.to_string());
            }
        }
    }

    pub fn get_root_causes(&self) -> Vec<(&CauseCategory, &Cause)> {
        let mut roots = Vec::new();
        for (cat, causes) in &self.categories {
            for cause in causes {
                if cause.is_root_cause {
                    roots.push((cat, cause));
                }
            }
        }
        roots
    }

    pub fn print_diagram(&self) {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                    FISHBONE DIAGRAM                       ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Problem: {:48} ║", self.problem);
        println!("╠══════════════════════════════════════════════════════════╣");

        for cat in CauseCategory::all() {
            if let Some(causes) = self.categories.get(&cat) {
                if !causes.is_empty() {
                    println!("║ {:?}:", cat);
                    for cause in causes {
                        let marker = if cause.is_root_cause { "★" } else { "○" };
                        println!("║   {} {}", marker, cause.description);
                        for sub in &cause.sub_causes {
                            println!("║       └─ {}", sub);
                        }
                    }
                }
            }
        }

        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ ★ = Root Cause                                           ║");
        println!("╚══════════════════════════════════════════════════════════╝");
    }
}

/// Fault tree analysis for trading systems
#[derive(Debug, Clone)]
pub struct FaultTree {
    pub top_event: String,
    pub gates: Vec<Gate>,
}

#[derive(Debug, Clone)]
pub struct Gate {
    pub id: String,
    pub gate_type: GateType,
    pub description: String,
    pub inputs: Vec<GateInput>,
    pub probability: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum GateType {
    And,  // All inputs must occur
    Or,   // Any input can cause output
}

#[derive(Debug, Clone)]
pub enum GateInput {
    BasicEvent { name: String, probability: f64 },
    Gate(String), // Reference to another gate
}

impl FaultTree {
    pub fn new(top_event: &str) -> Self {
        FaultTree {
            top_event: top_event.to_string(),
            gates: Vec::new(),
        }
    }

    pub fn add_gate(&mut self, id: &str, gate_type: GateType, description: &str, inputs: Vec<GateInput>) {
        self.gates.push(Gate {
            id: id.to_string(),
            gate_type,
            description: description.to_string(),
            inputs,
            probability: None,
        });
    }

    pub fn calculate_probabilities(&mut self) {
        // Simple probability calculation (in reality, this would be more complex)
        for gate in &mut self.gates {
            let prob = match gate.gate_type {
                GateType::And => {
                    gate.inputs.iter().fold(1.0, |acc, input| {
                        match input {
                            GateInput::BasicEvent { probability, .. } => acc * probability,
                            GateInput::Gate(_) => acc, // Would need recursive lookup
                        }
                    })
                }
                GateType::Or => {
                    1.0 - gate.inputs.iter().fold(1.0, |acc, input| {
                        match input {
                            GateInput::BasicEvent { probability, .. } => acc * (1.0 - probability),
                            GateInput::Gate(_) => acc,
                        }
                    })
                }
            };
            gate.probability = Some(prob);
        }
    }

    pub fn print_tree(&self) {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                     FAULT TREE                           ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Top Event: {:45} ║", self.top_event);
        println!("╠══════════════════════════════════════════════════════════╣");

        for gate in &self.gates {
            let gate_symbol = match gate.gate_type {
                GateType::And => "AND",
                GateType::Or => "OR",
            };
            let prob_str = gate.probability.map(|p| format!(" (P={:.6})", p)).unwrap_or_default();
            println!("║ [{}] {} {}{}", gate.id, gate_symbol, gate.description, prob_str);

            for input in &gate.inputs {
                match input {
                    GateInput::BasicEvent { name, probability } => {
                        println!("║   ├─ {} (P={:.4})", name, probability);
                    }
                    GateInput::Gate(id) => {
                        println!("║   ├─ [Gate: {}]", id);
                    }
                }
            }
        }

        println!("╚══════════════════════════════════════════════════════════╝");
    }
}

fn main() {
    println!("=== Root Cause Analysis Framework ===\n");

    // 5 Whys Example
    println!("--- 5 Whys Analysis ---\n");

    let mut five_whys = FiveWhysAnalysis::new("the order matching engine crashed");

    five_whys.add_why(
        "the process ran out of memory",
        vec!["OOM killer logs show process terminated", "Memory usage graphs spike to 100%"]
    );

    five_whys.add_why(
        "the order book reconstruction was leaking memory",
        vec!["Heap profiler shows Order objects not being freed", "Leak started after deploy at 14:00"]
    );

    five_whys.add_why(
        "cancel-replace orders weren't properly cleaning up old order objects",
        vec!["Code review found missing drop() call", "Reproduces with cancel-replace stress test"]
    );

    five_whys.add_why(
        "the code change in PR #4521 removed the cleanup logic accidentally",
        vec!["Git diff shows removal", "Author confirms it was unintentional"]
    );

    five_whys.add_why(
        "code review didn't catch the bug and tests didn't cover this path",
        vec![
            "No test for cancel-replace sequences > 1000",
            "Reviewer focused on feature, not cleanup code",
        ]
    );

    five_whys.set_root_cause(
        "Insufficient test coverage for edge cases and code review process \
         didn't include memory safety checklist."
    );

    five_whys.contributing_factors = vec![
        "High pressure to ship feature quickly".to_string(),
        "No automated memory leak detection in CI".to_string(),
        "Documentation didn't mention cleanup requirements".to_string(),
    ];

    println!("{}", five_whys.generate_report());

    // Fishbone Diagram Example
    println!("\n--- Fishbone Diagram ---\n");

    let mut fishbone = FishboneDiagram::new("Trading System Outage");

    fishbone.add_cause(CauseCategory::Technology, "Memory leak in order book", true);
    fishbone.add_cause(CauseCategory::Technology, "No circuit breaker for memory", false);
    fishbone.add_cause(CauseCategory::Process, "Inadequate code review", true);
    fishbone.add_cause(CauseCategory::Process, "Missing test coverage", false);
    fishbone.add_cause(CauseCategory::People, "Time pressure on developers", false);
    fishbone.add_cause(CauseCategory::Management, "Aggressive release schedule", false);

    fishbone.print_diagram();

    // Fault Tree Example
    println!("\n--- Fault Tree ---\n");

    let mut fault_tree = FaultTree::new("Trading System Failure");

    fault_tree.add_gate(
        "G1",
        GateType::Or,
        "System Unavailable",
        vec![
            GateInput::BasicEvent { name: "Memory Exhaustion".to_string(), probability: 0.01 },
            GateInput::BasicEvent { name: "Network Failure".to_string(), probability: 0.005 },
            GateInput::BasicEvent { name: "Database Failure".to_string(), probability: 0.002 },
        ]
    );

    fault_tree.add_gate(
        "G2",
        GateType::And,
        "Memory Exhaustion",
        vec![
            GateInput::BasicEvent { name: "Memory Leak Present".to_string(), probability: 0.1 },
            GateInput::BasicEvent { name: "High Load".to_string(), probability: 0.2 },
            GateInput::BasicEvent { name: "No Memory Limit".to_string(), probability: 0.5 },
        ]
    );

    fault_tree.calculate_probabilities();
    fault_tree.print_tree();
}
```

## Incident Response Automation

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Automated response action
#[derive(Debug, Clone)]
pub enum AutomatedAction {
    /// Kill all open positions
    FlattenPositions,
    /// Stop accepting new orders
    DisableOrderEntry,
    /// Reduce position limits
    ReduceLimits { factor: f64 },
    /// Switch to backup system
    Failover { target: String },
    /// Restart service
    RestartService { service: String },
    /// Alert on-call engineer
    PageOnCall { message: String },
    /// Send notification
    Notify { channel: String, message: String },
    /// Scale up resources
    ScaleUp { resource: String, amount: u32 },
    /// Enable circuit breaker
    EnableCircuitBreaker { name: String },
}

/// Response playbook for incident types
#[derive(Debug, Clone)]
pub struct ResponsePlaybook {
    pub name: String,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub actions: Vec<PlaybookAction>,
    pub escalation_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct TriggerCondition {
    pub metric: String,
    pub operator: ComparisonOp,
    pub threshold: f64,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum ComparisonOp {
    GreaterThan,
    LessThan,
    Equal,
}

#[derive(Debug, Clone)]
pub struct PlaybookAction {
    pub action: AutomatedAction,
    pub delay: Duration,
    pub requires_approval: bool,
}

/// Incident response coordinator
pub struct IncidentResponder {
    playbooks: Vec<ResponsePlaybook>,
    active_responses: Arc<std::sync::RwLock<Vec<ActiveResponse>>>,
    action_log: Arc<std::sync::RwLock<VecDeque<ActionLogEntry>>>,
    is_emergency_mode: AtomicBool,
    actions_taken: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct ActiveResponse {
    pub playbook_name: String,
    pub started_at: Instant,
    pub current_action_idx: usize,
    pub is_complete: bool,
}

#[derive(Debug, Clone)]
pub struct ActionLogEntry {
    pub timestamp: Instant,
    pub playbook: String,
    pub action: String,
    pub result: ActionResult,
}

#[derive(Debug, Clone)]
pub enum ActionResult {
    Success,
    Failed(String),
    Skipped(String),
    PendingApproval,
}

impl IncidentResponder {
    pub fn new() -> Self {
        IncidentResponder {
            playbooks: Vec::new(),
            active_responses: Arc::new(std::sync::RwLock::new(Vec::new())),
            action_log: Arc::new(std::sync::RwLock::new(VecDeque::with_capacity(1000))),
            is_emergency_mode: AtomicBool::new(false),
            actions_taken: AtomicU64::new(0),
        }
    }

    pub fn add_playbook(&mut self, playbook: ResponsePlaybook) {
        println!("Added playbook: {}", playbook.name);
        self.playbooks.push(playbook);
    }

    /// Create standard trading playbooks
    pub fn with_standard_playbooks(mut self) -> Self {
        // Runaway algorithm playbook
        self.add_playbook(ResponsePlaybook {
            name: "Runaway Algorithm".to_string(),
            trigger_conditions: vec![
                TriggerCondition {
                    metric: "orders_per_second".to_string(),
                    operator: ComparisonOp::GreaterThan,
                    threshold: 10000.0,
                    duration: Duration::from_secs(5),
                },
            ],
            actions: vec![
                PlaybookAction {
                    action: AutomatedAction::DisableOrderEntry,
                    delay: Duration::from_secs(0),
                    requires_approval: false,
                },
                PlaybookAction {
                    action: AutomatedAction::PageOnCall { message: "Runaway algorithm detected".to_string() },
                    delay: Duration::from_secs(0),
                    requires_approval: false,
                },
                PlaybookAction {
                    action: AutomatedAction::FlattenPositions,
                    delay: Duration::from_secs(30),
                    requires_approval: true,
                },
            ],
            escalation_timeout: Duration::from_secs(300),
        });

        // Exchange disconnection playbook
        self.add_playbook(ResponsePlaybook {
            name: "Exchange Disconnection".to_string(),
            trigger_conditions: vec![
                TriggerCondition {
                    metric: "connection_count".to_string(),
                    operator: ComparisonOp::LessThan,
                    threshold: 2.0,
                    duration: Duration::from_secs(10),
                },
            ],
            actions: vec![
                PlaybookAction {
                    action: AutomatedAction::Notify {
                        channel: "trading-alerts".to_string(),
                        message: "Exchange connections lost".to_string(),
                    },
                    delay: Duration::from_secs(0),
                    requires_approval: false,
                },
                PlaybookAction {
                    action: AutomatedAction::Failover { target: "backup-gateway".to_string() },
                    delay: Duration::from_secs(5),
                    requires_approval: false,
                },
                PlaybookAction {
                    action: AutomatedAction::PageOnCall { message: "Exchange failover triggered".to_string() },
                    delay: Duration::from_secs(0),
                    requires_approval: false,
                },
            ],
            escalation_timeout: Duration::from_secs(60),
        });

        // Excessive loss playbook
        self.add_playbook(ResponsePlaybook {
            name: "Excessive Loss".to_string(),
            trigger_conditions: vec![
                TriggerCondition {
                    metric: "daily_pnl".to_string(),
                    operator: ComparisonOp::LessThan,
                    threshold: -100000.0,
                    duration: Duration::from_secs(0),
                },
            ],
            actions: vec![
                PlaybookAction {
                    action: AutomatedAction::ReduceLimits { factor: 0.5 },
                    delay: Duration::from_secs(0),
                    requires_approval: false,
                },
                PlaybookAction {
                    action: AutomatedAction::PageOnCall { message: "Daily loss limit breach".to_string() },
                    delay: Duration::from_secs(0),
                    requires_approval: false,
                },
                PlaybookAction {
                    action: AutomatedAction::DisableOrderEntry,
                    delay: Duration::from_secs(60),
                    requires_approval: true,
                },
            ],
            escalation_timeout: Duration::from_secs(120),
        });

        self
    }

    /// Trigger a playbook by name
    pub fn trigger_playbook(&self, playbook_name: &str) -> Result<(), String> {
        let playbook = self.playbooks.iter()
            .find(|p| p.name == playbook_name)
            .ok_or_else(|| format!("Playbook '{}' not found", playbook_name))?;

        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║ INCIDENT RESPONSE: {:37} ║", playbook_name);
        println!("╚══════════════════════════════════════════════════════════╝\n");

        // Record active response
        {
            let mut active = self.active_responses.write().unwrap();
            active.push(ActiveResponse {
                playbook_name: playbook_name.to_string(),
                started_at: Instant::now(),
                current_action_idx: 0,
                is_complete: false,
            });
        }

        // Execute actions
        for (idx, action) in playbook.actions.iter().enumerate() {
            if !action.delay.is_zero() {
                println!("[Waiting {:?} before next action...]", action.delay);
            }

            let result = self.execute_action(action);
            self.log_action(playbook_name, &format!("{:?}", action.action), result.clone());

            match result {
                ActionResult::Success => {
                    println!("[OK] Action completed: {:?}", action.action);
                }
                ActionResult::PendingApproval => {
                    println!("[PENDING] Action requires approval: {:?}", action.action);
                }
                ActionResult::Failed(err) => {
                    println!("[FAILED] Action failed: {} - {:?}", err, action.action);
                }
                ActionResult::Skipped(reason) => {
                    println!("[SKIPPED] {}: {:?}", reason, action.action);
                }
            }

            self.actions_taken.fetch_add(1, Ordering::Relaxed);
        }

        // Mark response as complete
        {
            let mut active = self.active_responses.write().unwrap();
            if let Some(response) = active.iter_mut().find(|r| r.playbook_name == playbook_name) {
                response.is_complete = true;
            }
        }

        Ok(())
    }

    fn execute_action(&self, action: &PlaybookAction) -> ActionResult {
        if action.requires_approval {
            return ActionResult::PendingApproval;
        }

        // Simulate action execution
        match &action.action {
            AutomatedAction::FlattenPositions => {
                println!("  → Flattening all positions...");
                ActionResult::Success
            }
            AutomatedAction::DisableOrderEntry => {
                println!("  → Disabling order entry...");
                ActionResult::Success
            }
            AutomatedAction::ReduceLimits { factor } => {
                println!("  → Reducing limits by factor {}...", factor);
                ActionResult::Success
            }
            AutomatedAction::Failover { target } => {
                println!("  → Failing over to {}...", target);
                ActionResult::Success
            }
            AutomatedAction::RestartService { service } => {
                println!("  → Restarting service {}...", service);
                ActionResult::Success
            }
            AutomatedAction::PageOnCall { message } => {
                println!("  → Paging on-call: {}", message);
                ActionResult::Success
            }
            AutomatedAction::Notify { channel, message } => {
                println!("  → Notifying #{}: {}", channel, message);
                ActionResult::Success
            }
            AutomatedAction::ScaleUp { resource, amount } => {
                println!("  → Scaling up {} by {}", resource, amount);
                ActionResult::Success
            }
            AutomatedAction::EnableCircuitBreaker { name } => {
                println!("  → Enabling circuit breaker: {}", name);
                ActionResult::Success
            }
        }
    }

    fn log_action(&self, playbook: &str, action: &str, result: ActionResult) {
        let mut log = self.action_log.write().unwrap();
        log.push_back(ActionLogEntry {
            timestamp: Instant::now(),
            playbook: playbook.to_string(),
            action: action.to_string(),
            result,
        });

        // Keep only last 1000 entries
        while log.len() > 1000 {
            log.pop_front();
        }
    }

    /// Enable emergency mode (stops all trading)
    pub fn enable_emergency_mode(&self) {
        self.is_emergency_mode.store(true, Ordering::SeqCst);
        println!("\n!!! EMERGENCY MODE ENABLED !!!\n");
    }

    /// Get response summary
    pub fn get_summary(&self) -> String {
        let active = self.active_responses.read().unwrap();
        let log = self.action_log.read().unwrap();

        let active_count = active.iter().filter(|r| !r.is_complete).count();
        let completed_count = active.iter().filter(|r| r.is_complete).count();

        format!(
            "Incident Response Summary:\n\
             - Active Responses: {}\n\
             - Completed Responses: {}\n\
             - Total Actions Taken: {}\n\
             - Emergency Mode: {}",
            active_count,
            completed_count,
            self.actions_taken.load(Ordering::Relaxed),
            self.is_emergency_mode.load(Ordering::Relaxed)
        )
    }
}

fn main() {
    println!("=== Incident Response Automation ===\n");

    let responder = IncidentResponder::new().with_standard_playbooks();

    // Simulate triggering different playbooks
    println!("Scenario 1: Runaway Algorithm Detected");
    responder.trigger_playbook("Runaway Algorithm").unwrap();

    println!("\n\nScenario 2: Exchange Connection Lost");
    responder.trigger_playbook("Exchange Disconnection").unwrap();

    println!("\n\nScenario 3: Excessive Daily Loss");
    responder.trigger_playbook("Excessive Loss").unwrap();

    println!("\n\n{}", responder.get_summary());
}
```

## Metrics and Learning from Incidents

```rust
use std::collections::HashMap;
use std::time::Duration;

/// Incident metrics tracker for continuous improvement
#[derive(Debug)]
pub struct IncidentMetricsTracker {
    incidents: Vec<IncidentRecord>,
    mttr_by_severity: HashMap<String, Vec<Duration>>,
    recurrence_map: HashMap<String, u32>,
}

#[derive(Debug, Clone)]
pub struct IncidentRecord {
    pub id: String,
    pub severity: String,
    pub category: String,
    pub time_to_detect: Duration,
    pub time_to_respond: Duration,
    pub time_to_resolve: Duration,
    pub affected_customers: u64,
    pub revenue_impact: f64,
    pub action_items_created: u32,
    pub action_items_completed: u32,
    pub was_recurring: bool,
}

impl IncidentMetricsTracker {
    pub fn new() -> Self {
        IncidentMetricsTracker {
            incidents: Vec::new(),
            mttr_by_severity: HashMap::new(),
            recurrence_map: HashMap::new(),
        }
    }

    pub fn add_incident(&mut self, record: IncidentRecord) {
        // Track MTTR by severity
        self.mttr_by_severity
            .entry(record.severity.clone())
            .or_insert_with(Vec::new)
            .push(record.time_to_resolve);

        // Track recurrence
        *self.recurrence_map
            .entry(record.category.clone())
            .or_insert(0) += 1;

        self.incidents.push(record);
    }

    /// Calculate Mean Time To Detect (MTTD)
    pub fn mttd(&self) -> Duration {
        if self.incidents.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.incidents.iter()
            .map(|i| i.time_to_detect)
            .sum();

        total / self.incidents.len() as u32
    }

    /// Calculate Mean Time To Respond (first action taken)
    pub fn mttr_response(&self) -> Duration {
        if self.incidents.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.incidents.iter()
            .map(|i| i.time_to_respond)
            .sum();

        total / self.incidents.len() as u32
    }

    /// Calculate Mean Time To Resolve
    pub fn mttr_resolve(&self) -> Duration {
        if self.incidents.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.incidents.iter()
            .map(|i| i.time_to_resolve)
            .sum();

        total / self.incidents.len() as u32
    }

    /// Calculate incident frequency by category
    pub fn incident_frequency(&self) -> Vec<(String, u32)> {
        let mut freq: Vec<_> = self.recurrence_map.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        freq.sort_by(|a, b| b.1.cmp(&a.1));
        freq
    }

    /// Calculate action item completion rate
    pub fn action_item_completion_rate(&self) -> f64 {
        let total_created: u32 = self.incidents.iter()
            .map(|i| i.action_items_created)
            .sum();

        let total_completed: u32 = self.incidents.iter()
            .map(|i| i.action_items_completed)
            .sum();

        if total_created == 0 {
            return 100.0;
        }

        (total_completed as f64 / total_created as f64) * 100.0
    }

    /// Calculate recurrence rate
    pub fn recurrence_rate(&self) -> f64 {
        if self.incidents.is_empty() {
            return 0.0;
        }

        let recurring = self.incidents.iter()
            .filter(|i| i.was_recurring)
            .count();

        (recurring as f64 / self.incidents.len() as f64) * 100.0
    }

    /// Calculate total customer impact
    pub fn total_customer_impact(&self) -> u64 {
        self.incidents.iter()
            .map(|i| i.affected_customers)
            .sum()
    }

    /// Calculate total revenue impact
    pub fn total_revenue_impact(&self) -> f64 {
        self.incidents.iter()
            .map(|i| i.revenue_impact)
            .sum()
    }

    /// Generate comprehensive report
    pub fn generate_report(&self, period: &str) -> String {
        let mut report = String::new();

        report.push_str(&format!("╔══════════════════════════════════════════════════════════╗\n"));
        report.push_str(&format!("║          INCIDENT METRICS REPORT - {:17}  ║\n", period));
        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        report.push_str(&format!("║ Total Incidents: {:39} ║\n", self.incidents.len()));
        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        // Time metrics
        report.push_str(&format!("║ RESPONSE METRICS                                         ║\n"));
        report.push_str(&format!("║   Mean Time To Detect (MTTD):  {:>24?} ║\n", self.mttd()));
        report.push_str(&format!("║   Mean Time To Respond:        {:>24?} ║\n", self.mttr_response()));
        report.push_str(&format!("║   Mean Time To Resolve (MTTR): {:>24?} ║\n", self.mttr_resolve()));

        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        // Impact metrics
        report.push_str(&format!("║ IMPACT METRICS                                           ║\n"));
        report.push_str(&format!("║   Total Affected Customers: {:>27} ║\n", self.total_customer_impact()));
        report.push_str(&format!("║   Total Revenue Impact:     ${:>26.2} ║\n", self.total_revenue_impact()));

        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        // Quality metrics
        report.push_str(&format!("║ QUALITY METRICS                                          ║\n"));
        report.push_str(&format!("║   Action Item Completion Rate: {:>24.1}% ║\n", self.action_item_completion_rate()));
        report.push_str(&format!("║   Recurrence Rate:             {:>24.1}% ║\n", self.recurrence_rate()));

        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        // Top incident categories
        report.push_str(&format!("║ TOP INCIDENT CATEGORIES                                  ║\n"));
        for (cat, count) in self.incident_frequency().iter().take(5) {
            report.push_str(&format!("║   {:40} {:>10} ║\n", cat, count));
        }

        report.push_str(&format!("╚══════════════════════════════════════════════════════════╝\n"));

        report
    }
}

/// Lessons learned database
#[derive(Debug)]
pub struct LessonsLearnedDatabase {
    lessons: Vec<Lesson>,
    tags_index: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Clone)]
pub struct Lesson {
    pub id: String,
    pub incident_id: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub recommendations: Vec<String>,
    pub implemented: bool,
}

impl LessonsLearnedDatabase {
    pub fn new() -> Self {
        LessonsLearnedDatabase {
            lessons: Vec::new(),
            tags_index: HashMap::new(),
        }
    }

    pub fn add_lesson(&mut self, lesson: Lesson) {
        let idx = self.lessons.len();

        // Index by tags
        for tag in &lesson.tags {
            self.tags_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }

        self.lessons.push(lesson);
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&Lesson> {
        self.tags_index.get(tag)
            .map(|indices| {
                indices.iter()
                    .map(|&i| &self.lessons[i])
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_unimplemented(&self) -> Vec<&Lesson> {
        self.lessons.iter()
            .filter(|l| !l.implemented)
            .collect()
    }

    pub fn implementation_rate(&self) -> f64 {
        if self.lessons.is_empty() {
            return 100.0;
        }

        let implemented = self.lessons.iter().filter(|l| l.implemented).count();
        (implemented as f64 / self.lessons.len() as f64) * 100.0
    }
}

fn main() {
    println!("=== Incident Metrics and Learning ===\n");

    let mut tracker = IncidentMetricsTracker::new();

    // Add sample incidents
    tracker.add_incident(IncidentRecord {
        id: "INC-001".to_string(),
        severity: "Critical".to_string(),
        category: "Memory Leak".to_string(),
        time_to_detect: Duration::from_secs(120),
        time_to_respond: Duration::from_secs(180),
        time_to_resolve: Duration::from_secs(2820),
        affected_customers: 3247,
        revenue_impact: 125000.0,
        action_items_created: 5,
        action_items_completed: 4,
        was_recurring: false,
    });

    tracker.add_incident(IncidentRecord {
        id: "INC-002".to_string(),
        severity: "High".to_string(),
        category: "Network Timeout".to_string(),
        time_to_detect: Duration::from_secs(60),
        time_to_respond: Duration::from_secs(120),
        time_to_resolve: Duration::from_secs(900),
        affected_customers: 521,
        revenue_impact: 15000.0,
        action_items_created: 3,
        action_items_completed: 3,
        was_recurring: true,
    });

    tracker.add_incident(IncidentRecord {
        id: "INC-003".to_string(),
        severity: "Critical".to_string(),
        category: "Memory Leak".to_string(),
        time_to_detect: Duration::from_secs(90),
        time_to_respond: Duration::from_secs(150),
        time_to_resolve: Duration::from_secs(1800),
        affected_customers: 1500,
        revenue_impact: 75000.0,
        action_items_created: 4,
        action_items_completed: 2,
        was_recurring: true,
    });

    println!("{}", tracker.generate_report("Q4 2024"));

    // Lessons learned
    println!("\n--- Lessons Learned Database ---\n");

    let mut lessons_db = LessonsLearnedDatabase::new();

    lessons_db.add_lesson(Lesson {
        id: "LL-001".to_string(),
        incident_id: "INC-001".to_string(),
        title: "Memory Profiling in CI Pipeline".to_string(),
        description: "Add automated memory leak detection to CI pipeline".to_string(),
        tags: vec!["memory".to_string(), "ci".to_string(), "testing".to_string()],
        recommendations: vec![
            "Integrate Valgrind or AddressSanitizer".to_string(),
            "Set memory growth thresholds".to_string(),
        ],
        implemented: true,
    });

    lessons_db.add_lesson(Lesson {
        id: "LL-002".to_string(),
        incident_id: "INC-002".to_string(),
        title: "Network Timeout Handling".to_string(),
        description: "Implement proper retry logic with exponential backoff".to_string(),
        tags: vec!["network".to_string(), "resilience".to_string()],
        recommendations: vec![
            "Add circuit breaker pattern".to_string(),
            "Implement connection pooling".to_string(),
        ],
        implemented: false,
    });

    println!("Total lessons: {}", lessons_db.lessons.len());
    println!("Implementation rate: {:.1}%", lessons_db.implementation_rate());

    println!("\nLessons tagged 'memory':");
    for lesson in lessons_db.search_by_tag("memory") {
        println!("  - {}: {}", lesson.id, lesson.title);
    }

    println!("\nUnimplemented lessons:");
    for lesson in lessons_db.get_unimplemented() {
        println!("  - {}: {}", lesson.id, lesson.title);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Post-Mortem** | Blameless analysis of incidents to prevent recurrence |
| **Timeline Reconstruction** | Building a detailed sequence of events during incident |
| **Root Cause Analysis** | Finding the underlying cause, not just symptoms |
| **5 Whys** | Iterative questioning technique to find root cause |
| **Fishbone Diagram** | Visual tool for organizing potential causes |
| **Incident Playbooks** | Predefined automated response procedures |
| **MTTD/MTTR** | Key metrics for measuring incident response |
| **Lessons Learned** | Documentation of insights for future prevention |

## Practical Exercises

1. **Post-Mortem Template**: Create a comprehensive post-mortem system that:
   - Generates templates automatically from incident data
   - Tracks timeline events as they happen
   - Calculates impact metrics automatically
   - Exports to various formats (Markdown, PDF, JIRA)

2. **Root Cause Analyzer**: Build a tool that:
   - Guides users through 5 Whys analysis
   - Suggests potential causes based on incident category
   - Links related incidents and patterns
   - Generates visualizations (fishbone diagrams)

3. **Response Playbook Engine**: Implement an automated system that:
   - Matches incidents to playbooks based on metrics
   - Executes actions with appropriate delays
   - Handles approval workflows
   - Logs all actions for audit trail

4. **Metrics Dashboard**: Create a dashboard that:
   - Tracks MTTD, MTTR trends over time
   - Shows incident frequency by category
   - Monitors action item completion
   - Alerts on metric degradation

## Homework

1. **Complete Incident Management System**: Build an end-to-end system:
   - Real-time incident detection and alerting
   - Automated playbook execution
   - Post-mortem generation with templates
   - Lessons learned database with search
   - Integration with Slack/PagerDuty
   - Prometheus metrics export

2. **ML-Powered Root Cause Analysis**: Create an intelligent analyzer:
   - Pattern recognition across historical incidents
   - Automatic root cause suggestions
   - Anomaly correlation across services
   - Prediction of likely failure modes

3. **Chaos Engineering Integration**: Combine with testing:
   - Generate synthetic incidents for training
   - Test playbook effectiveness
   - Measure team response times
   - Identify gaps in monitoring

4. **Trading-Specific Post-Mortem**: Build specialized tools for:
   - Trade reconstruction during incidents
   - P&L impact calculation
   - Regulatory reporting compliance
   - Market condition correlation analysis

## Navigation

[← Previous day](../361-ab-strategy-testing/en.md) | [Next day →](../363-compliance/en.md)
