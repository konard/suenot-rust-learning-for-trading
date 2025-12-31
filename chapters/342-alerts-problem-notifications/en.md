# Day 342: Alerts: Problem Notifications

## Trading Analogy

Imagine you're running a trading bot that works 24/7. You can't constantly stare at the screen, but you need to know immediately about critical situations:

- **Lost exchange connection** — like your terminal suddenly disconnecting during active trading
- **Daily loss limit exceeded** — like a stop-loss for your entire portfolio
- **Abnormal volatility** — like an alarm on the trading floor
- **Order execution error** — like a broker notification about a trade problem

In production, alerts are your "eyes and ears" that watch the system when you can't do it yourself.

| Criteria | Logging | Alerts |
|----------|---------|--------|
| **Purpose** | Record all events | Immediate notification of problems |
| **Volume** | All events | Only critical ones |
| **Response** | Post-mortem analysis | Immediate action |
| **Channels** | Files, console | Email, Telegram, Slack, SMS |
| **Priority** | Low/Medium | High/Critical |

## Alert Types in Trading Systems

```rust
use std::fmt;

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational — for statistics
    Info,
    /// Warning — needs attention
    Warning,
    /// Error — requires intervention
    Error,
    /// Critical — requires immediate action
    Critical,
}

impl fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Error => write!(f, "ERROR"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Alert category for grouping
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertCategory {
    /// Connection issues
    Connectivity,
    /// Order problems
    OrderExecution,
    /// Risk management
    RiskManagement,
    /// System performance
    Performance,
    /// Market conditions
    MarketConditions,
}

/// Alert structure
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub title: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Alert {
    pub fn new(
        severity: AlertSeverity,
        category: AlertCategory,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            category,
            title: title.into(),
            message: message.into(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

fn main() {
    // Alert examples
    let connection_alert = Alert::new(
        AlertSeverity::Critical,
        AlertCategory::Connectivity,
        "Lost connection to Binance",
        "WebSocket connection dropped, attempting to reconnect...",
    )
    .with_metadata("exchange", "binance")
    .with_metadata("retry_count", "3");

    let risk_alert = Alert::new(
        AlertSeverity::Warning,
        AlertCategory::RiskManagement,
        "Approaching daily loss limit",
        "Current loss: -$450, limit: -$500",
    )
    .with_metadata("current_loss", "-450")
    .with_metadata("limit", "-500");

    println!("[{}] {}: {}", connection_alert.severity, connection_alert.title, connection_alert.message);
    println!("[{}] {}: {}", risk_alert.severity, risk_alert.title, risk_alert.message);
}
```

## Alert Sending System

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for notification channels
#[async_trait::async_trait]
pub trait AlertChannel: Send + Sync {
    /// Channel name
    fn name(&self) -> &str;

    /// Send alert
    async fn send(&self, alert: &Alert) -> Result<(), AlertError>;

    /// Does the channel support this severity level
    fn supports_severity(&self, severity: AlertSeverity) -> bool;
}

#[derive(Debug)]
pub struct AlertError {
    pub message: String,
    pub channel: String,
}

/// Console channel (for development)
pub struct ConsoleChannel;

#[async_trait::async_trait]
impl AlertChannel for ConsoleChannel {
    fn name(&self) -> &str {
        "console"
    }

    async fn send(&self, alert: &Alert) -> Result<(), AlertError> {
        let emoji = match alert.severity {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Error => "❌",
            AlertSeverity::Critical => "🚨",
        };

        println!(
            "\n{} [{}] {}\n   {}\n   Time: {}\n   Category: {:?}",
            emoji,
            alert.severity,
            alert.title,
            alert.message,
            alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.category
        );

        if !alert.metadata.is_empty() {
            println!("   Metadata:");
            for (key, value) in &alert.metadata {
                println!("     - {}: {}", key, value);
            }
        }

        Ok(())
    }

    fn supports_severity(&self, _severity: AlertSeverity) -> bool {
        true // Console accepts all levels
    }
}

/// Telegram sending channel
pub struct TelegramChannel {
    bot_token: String,
    chat_id: String,
    min_severity: AlertSeverity,
}

impl TelegramChannel {
    pub fn new(bot_token: String, chat_id: String, min_severity: AlertSeverity) -> Self {
        Self {
            bot_token,
            chat_id,
            min_severity,
        }
    }
}

#[async_trait::async_trait]
impl AlertChannel for TelegramChannel {
    fn name(&self) -> &str {
        "telegram"
    }

    async fn send(&self, alert: &Alert) -> Result<(), AlertError> {
        let emoji = match alert.severity {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Error => "❌",
            AlertSeverity::Critical => "🚨",
        };

        let message = format!(
            "{} *{}*\n\n{}\n\n_Time: {}_\n_Category: {:?}_",
            emoji,
            alert.title,
            alert.message,
            alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.category
        );

        // In real code, this would be an HTTP request to Telegram API
        // let url = format!(
        //     "https://api.telegram.org/bot{}/sendMessage",
        //     self.bot_token
        // );
        //
        // let client = reqwest::Client::new();
        // client.post(&url)
        //     .json(&serde_json::json!({
        //         "chat_id": self.chat_id,
        //         "text": message,
        //         "parse_mode": "Markdown"
        //     }))
        //     .send()
        //     .await?;

        println!("[Telegram] Sent to chat {}: {}", self.chat_id, message);
        Ok(())
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity >= self.min_severity
    }
}

/// Alert manager
pub struct AlertManager {
    channels: Vec<Arc<dyn AlertChannel>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
    dedup_window_secs: u64,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            dedup_window_secs: 300, // 5 minutes
        }
    }

    pub fn add_channel(&mut self, channel: Arc<dyn AlertChannel>) {
        self.channels.push(channel);
    }

    /// Check if a similar alert was sent recently
    async fn is_duplicate(&self, alert: &Alert) -> bool {
        let history = self.alert_history.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(self.dedup_window_secs as i64);

        history.iter().any(|existing| {
            existing.title == alert.title
                && existing.category == alert.category
                && existing.timestamp > cutoff
        })
    }

    /// Send alert to all appropriate channels
    pub async fn send(&self, alert: Alert) -> Vec<Result<(), AlertError>> {
        // Check deduplication
        if self.is_duplicate(&alert).await {
            println!("[AlertManager] Duplicate alert skipped: {}", alert.title);
            return vec![];
        }

        // Save to history
        {
            let mut history = self.alert_history.write().await;
            history.push(alert.clone());

            // Clean up old entries
            let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
            history.retain(|a| a.timestamp > cutoff);
        }

        // Send to all appropriate channels
        let mut results = Vec::new();

        for channel in &self.channels {
            if channel.supports_severity(alert.severity) {
                let result = channel.send(&alert).await;
                if let Err(ref e) = result {
                    eprintln!(
                        "[AlertManager] Error sending to {}: {}",
                        channel.name(),
                        e.message
                    );
                }
                results.push(result);
            }
        }

        results
    }
}

#[tokio::main]
async fn main() {
    let mut manager = AlertManager::new();

    // Add channels
    manager.add_channel(Arc::new(ConsoleChannel));
    manager.add_channel(Arc::new(TelegramChannel::new(
        "BOT_TOKEN".to_string(),
        "CHAT_ID".to_string(),
        AlertSeverity::Warning, // Only Warning and above
    )));

    // Send alerts
    let alert = Alert::new(
        AlertSeverity::Critical,
        AlertCategory::Connectivity,
        "Lost exchange connection",
        "Binance WebSocket disconnected, active orders may not update",
    )
    .with_metadata("exchange", "binance")
    .with_metadata("last_ping", "2024-01-15 10:30:00");

    manager.send(alert).await;
}
```

## Conditional Alerts Based on Metrics

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Rule for generating an alert
#[derive(Clone)]
pub struct AlertRule {
    pub name: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub message_template: String,
    pub cooldown_secs: u64,
}

#[derive(Clone)]
pub enum AlertCondition {
    /// Value greater than threshold
    GreaterThan(f64),
    /// Value less than threshold
    LessThan(f64),
    /// Value outside range
    OutOfRange { min: f64, max: f64 },
    /// Change over period greater than threshold (in percent)
    ChangeRate { threshold_pct: f64, window_secs: u64 },
}

impl AlertCondition {
    fn check(&self, current: f64, history: &[(chrono::DateTime<chrono::Utc>, f64)]) -> bool {
        match self {
            AlertCondition::GreaterThan(threshold) => current > *threshold,
            AlertCondition::LessThan(threshold) => current < *threshold,
            AlertCondition::OutOfRange { min, max } => current < *min || current > *max,
            AlertCondition::ChangeRate { threshold_pct, window_secs } => {
                let cutoff = chrono::Utc::now() - chrono::Duration::seconds(*window_secs as i64);
                if let Some((_, old_value)) = history.iter().find(|(ts, _)| *ts <= cutoff) {
                    if *old_value != 0.0 {
                        let change_pct = ((current - old_value) / old_value).abs() * 100.0;
                        return change_pct > *threshold_pct;
                    }
                }
                false
            }
        }
    }
}

/// Metric monitoring with alert generation
pub struct MetricAlertMonitor {
    rules: Vec<AlertRule>,
    metrics: Arc<RwLock<HashMap<String, Vec<(chrono::DateTime<chrono::Utc>, f64)>>>>,
    last_alerts: Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
}

impl MetricAlertMonitor {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            last_alerts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    /// Record metric value
    pub async fn record_metric(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        let history = metrics.entry(name.to_string()).or_insert_with(Vec::new);

        history.push((chrono::Utc::now(), value));

        // Keep only last 24 hours
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        history.retain(|(ts, _)| *ts > cutoff);
    }

    /// Check all rules and return triggered alerts
    pub async fn check_rules(&self) -> Vec<Alert> {
        let metrics = self.metrics.read().await;
        let mut last_alerts = self.last_alerts.write().await;
        let mut alerts = Vec::new();
        let now = chrono::Utc::now();

        for rule in &self.rules {
            // Check cooldown
            if let Some(last) = last_alerts.get(&rule.name) {
                if now - *last < chrono::Duration::seconds(rule.cooldown_secs as i64) {
                    continue;
                }
            }

            // Get current value and history
            if let Some(history) = metrics.get(&rule.metric_name) {
                if let Some((_, current)) = history.last() {
                    if rule.condition.check(*current, history) {
                        let message = rule.message_template
                            .replace("{value}", &format!("{:.2}", current))
                            .replace("{metric}", &rule.metric_name);

                        let alert = Alert::new(
                            rule.severity,
                            AlertCategory::Performance,
                            &rule.name,
                            message,
                        )
                        .with_metadata("metric", &rule.metric_name)
                        .with_metadata("value", &format!("{:.4}", current));

                        alerts.push(alert);
                        last_alerts.insert(rule.name.clone(), now);
                    }
                }
            }
        }

        alerts
    }
}

#[tokio::main]
async fn main() {
    let mut monitor = MetricAlertMonitor::new();

    // Rules for trading system
    monitor.add_rule(AlertRule {
        name: "high_latency".to_string(),
        metric_name: "order_execution_latency_ms".to_string(),
        condition: AlertCondition::GreaterThan(500.0),
        severity: AlertSeverity::Warning,
        message_template: "High order execution latency: {value}ms".to_string(),
        cooldown_secs: 300,
    });

    monitor.add_rule(AlertRule {
        name: "daily_loss_limit".to_string(),
        metric_name: "daily_pnl_usd".to_string(),
        condition: AlertCondition::LessThan(-500.0),
        severity: AlertSeverity::Critical,
        message_template: "Daily loss limit exceeded! PnL: ${value}".to_string(),
        cooldown_secs: 60,
    });

    monitor.add_rule(AlertRule {
        name: "price_spike".to_string(),
        metric_name: "btc_price".to_string(),
        condition: AlertCondition::ChangeRate {
            threshold_pct: 5.0,
            window_secs: 300,
        },
        severity: AlertSeverity::Warning,
        message_template: "Sharp BTC price change: ${value}".to_string(),
        cooldown_secs: 600,
    });

    // Simulate metric recording
    monitor.record_metric("order_execution_latency_ms", 150.0).await;
    monitor.record_metric("daily_pnl_usd", 100.0).await;
    monitor.record_metric("btc_price", 50000.0).await;

    // Check rules
    let alerts = monitor.check_rules().await;
    println!("Triggered alerts: {}", alerts.len());

    // Simulate problem
    monitor.record_metric("order_execution_latency_ms", 750.0).await;
    monitor.record_metric("daily_pnl_usd", -550.0).await;

    let alerts = monitor.check_rules().await;
    for alert in alerts {
        println!("[{}] {}: {}", alert.severity, alert.title, alert.message);
    }
}
```

## Integration with Prometheus Alertmanager

```rust
use std::collections::HashMap;

/// Alert in Prometheus Alertmanager format
#[derive(Debug, serde::Serialize)]
pub struct PrometheusAlert {
    pub status: String, // "firing" or "resolved"
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
    #[serde(rename = "endsAt", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<String>,
    #[serde(rename = "generatorURL", skip_serializing_if = "Option::is_none")]
    pub generator_url: Option<String>,
}

impl PrometheusAlert {
    pub fn firing(
        alertname: &str,
        severity: &str,
        summary: &str,
        description: &str,
    ) -> Self {
        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), alertname.to_string());
        labels.insert("severity".to_string(), severity.to_string());

        let mut annotations = HashMap::new();
        annotations.insert("summary".to_string(), summary.to_string());
        annotations.insert("description".to_string(), description.to_string());

        Self {
            status: "firing".to_string(),
            labels,
            annotations,
            starts_at: chrono::Utc::now().to_rfc3339(),
            ends_at: None,
            generator_url: None,
        }
    }

    pub fn resolved(alertname: &str) -> Self {
        let mut labels = HashMap::new();
        labels.insert("alertname".to_string(), alertname.to_string());

        Self {
            status: "resolved".to_string(),
            labels,
            annotations: HashMap::new(),
            starts_at: chrono::Utc::now().to_rfc3339(),
            ends_at: Some(chrono::Utc::now().to_rfc3339()),
            generator_url: None,
        }
    }

    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.insert(key.to_string(), value.to_string());
        self
    }
}

/// Client for sending alerts to Prometheus Alertmanager
pub struct AlertmanagerClient {
    url: String,
}

impl AlertmanagerClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    /// Send alerts to Alertmanager
    pub async fn send_alerts(&self, alerts: Vec<PrometheusAlert>) -> Result<(), String> {
        // In real code:
        // let client = reqwest::Client::new();
        // let response = client
        //     .post(&format!("{}/api/v1/alerts", self.url))
        //     .json(&alerts)
        //     .send()
        //     .await
        //     .map_err(|e| e.to_string())?;
        //
        // if !response.status().is_success() {
        //     return Err(format!("Alertmanager error: {}", response.status()));
        // }

        println!(
            "[Alertmanager] Sent {} alerts to {}",
            alerts.len(),
            self.url
        );
        for alert in &alerts {
            println!(
                "  - {} [{}]: {:?}",
                alert.labels.get("alertname").unwrap_or(&"unknown".to_string()),
                alert.status,
                alert.annotations.get("summary")
            );
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let client = AlertmanagerClient::new("http://localhost:9093");

    // Create alert about connection issue
    let connection_alert = PrometheusAlert::firing(
        "TradingBotConnectionLost",
        "critical",
        "Lost connection to Binance exchange",
        "WebSocket connection dropped. Last ping: 5 minutes ago.",
    )
    .with_label("exchange", "binance")
    .with_label("service", "trading-bot")
    .with_label("environment", "production");

    // Create alert about high latency
    let latency_alert = PrometheusAlert::firing(
        "HighOrderLatency",
        "warning",
        "High order execution latency",
        "Average latency over last 5 minutes: 850ms (threshold: 500ms)",
    )
    .with_label("exchange", "binance")
    .with_label("service", "trading-bot");

    client
        .send_alerts(vec![connection_alert, latency_alert])
        .await
        .expect("Error sending alerts");

    // Later, when problem is resolved
    let resolved = PrometheusAlert::resolved("TradingBotConnectionLost")
        .with_label("exchange", "binance")
        .with_label("service", "trading-bot");

    client
        .send_alerts(vec![resolved])
        .await
        .expect("Error sending resolved");
}
```

## Webhook Alerts for Integrations

```rust
use std::collections::HashMap;
use std::sync::Arc;

/// Webhook payload format
#[derive(Debug, serde::Serialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub timestamp: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub source: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Webhook sending channel
pub struct WebhookChannel {
    url: String,
    headers: HashMap<String, String>,
    name: String,
    min_severity: AlertSeverity,
}

impl WebhookChannel {
    pub fn new(name: &str, url: &str, min_severity: AlertSeverity) -> Self {
        Self {
            url: url.to_string(),
            headers: HashMap::new(),
            name: name.to_string(),
            min_severity,
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_auth_token(self, token: &str) -> Self {
        self.with_header("Authorization", &format!("Bearer {}", token))
    }
}

#[async_trait::async_trait]
impl AlertChannel for WebhookChannel {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, alert: &Alert) -> Result<(), AlertError> {
        let payload = WebhookPayload {
            event_type: "trading_alert".to_string(),
            timestamp: alert.timestamp.to_rfc3339(),
            severity: alert.severity.to_string().to_lowercase(),
            title: alert.title.clone(),
            message: alert.message.clone(),
            source: "trading-bot".to_string(),
            metadata: alert
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        };

        // In real code:
        // let client = reqwest::Client::new();
        // let mut request = client.post(&self.url).json(&payload);
        //
        // for (key, value) in &self.headers {
        //     request = request.header(key, value);
        // }
        //
        // let response = request.send().await.map_err(|e| AlertError {
        //     message: e.to_string(),
        //     channel: self.name.clone(),
        // })?;

        println!(
            "[Webhook:{}] POST {}\nPayload: {}",
            self.name,
            self.url,
            serde_json::to_string_pretty(&payload).unwrap()
        );

        Ok(())
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity >= self.min_severity
    }
}

/// PagerDuty integration example
pub struct PagerDutyChannel {
    routing_key: String,
}

impl PagerDutyChannel {
    pub fn new(routing_key: &str) -> Self {
        Self {
            routing_key: routing_key.to_string(),
        }
    }

    fn severity_to_pagerduty(&self, severity: AlertSeverity) -> &'static str {
        match severity {
            AlertSeverity::Info => "info",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Error => "error",
            AlertSeverity::Critical => "critical",
        }
    }
}

#[async_trait::async_trait]
impl AlertChannel for PagerDutyChannel {
    fn name(&self) -> &str {
        "pagerduty"
    }

    async fn send(&self, alert: &Alert) -> Result<(), AlertError> {
        let payload = serde_json::json!({
            "routing_key": self.routing_key,
            "event_action": "trigger",
            "dedup_key": format!("{}:{}", alert.category as u8, alert.title),
            "payload": {
                "summary": alert.title,
                "source": "trading-bot",
                "severity": self.severity_to_pagerduty(alert.severity),
                "custom_details": {
                    "message": alert.message,
                    "category": format!("{:?}", alert.category),
                    "metadata": alert.metadata,
                }
            }
        });

        // In real code:
        // let client = reqwest::Client::new();
        // client
        //     .post("https://events.pagerduty.com/v2/enqueue")
        //     .json(&payload)
        //     .send()
        //     .await?;

        println!(
            "[PagerDuty] Created incident: {}\n{}",
            alert.title,
            serde_json::to_string_pretty(&payload).unwrap()
        );

        Ok(())
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity >= AlertSeverity::Error // Only Error and Critical
    }
}

fn main() {
    println!("Webhook integrations example");
}
```

## Complete Alert System for Trading Bot

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Trading bot state for monitoring
#[derive(Default)]
pub struct TradingBotState {
    pub connected_exchanges: HashMap<String, bool>,
    pub daily_pnl: f64,
    pub open_positions: usize,
    pub order_latency_ms: f64,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    pub error_count_1h: usize,
}

/// Trading bot health monitor
pub struct TradingBotHealthMonitor {
    state: Arc<RwLock<TradingBotState>>,
    alert_manager: Arc<AlertManager>,
    config: MonitorConfig,
}

#[derive(Clone)]
pub struct MonitorConfig {
    pub daily_loss_limit: f64,
    pub max_latency_ms: f64,
    pub max_errors_per_hour: usize,
    pub heartbeat_timeout_secs: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            daily_loss_limit: -500.0,
            max_latency_ms: 500.0,
            max_errors_per_hour: 10,
            heartbeat_timeout_secs: 60,
        }
    }
}

impl TradingBotHealthMonitor {
    pub fn new(alert_manager: Arc<AlertManager>, config: MonitorConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(TradingBotState::default())),
            alert_manager,
            config,
        }
    }

    /// Update exchange connection status
    pub async fn update_exchange_connection(&self, exchange: &str, connected: bool) {
        let mut state = self.state.write().await;
        let was_connected = state.connected_exchanges.get(exchange).copied().unwrap_or(true);
        state.connected_exchanges.insert(exchange.to_string(), connected);

        // Generate alert on connection loss
        if was_connected && !connected {
            let alert = Alert::new(
                AlertSeverity::Critical,
                AlertCategory::Connectivity,
                format!("Connection lost: {}", exchange),
                format!("Connection to {} exchange lost. Trading paused.", exchange),
            )
            .with_metadata("exchange", exchange);

            self.alert_manager.send(alert).await;
        }

        // Generate alert on reconnection
        if !was_connected && connected {
            let alert = Alert::new(
                AlertSeverity::Info,
                AlertCategory::Connectivity,
                format!("Connection restored: {}", exchange),
                format!("Connection to {} exchange restored.", exchange),
            )
            .with_metadata("exchange", exchange);

            self.alert_manager.send(alert).await;
        }
    }

    /// Update PnL
    pub async fn update_pnl(&self, pnl: f64) {
        let mut state = self.state.write().await;
        let previous_pnl = state.daily_pnl;
        state.daily_pnl = pnl;

        // Alert when approaching limit (80%)
        let warning_threshold = self.config.daily_loss_limit * 0.8;
        if pnl <= warning_threshold && previous_pnl > warning_threshold {
            let alert = Alert::new(
                AlertSeverity::Warning,
                AlertCategory::RiskManagement,
                "Approaching loss limit",
                format!(
                    "Current PnL: ${:.2}, limit: ${:.2} (remaining: ${:.2})",
                    pnl,
                    self.config.daily_loss_limit,
                    self.config.daily_loss_limit - pnl
                ),
            )
            .with_metadata("current_pnl", &format!("{:.2}", pnl))
            .with_metadata("limit", &format!("{:.2}", self.config.daily_loss_limit));

            self.alert_manager.send(alert).await;
        }

        // Critical alert when limit exceeded
        if pnl <= self.config.daily_loss_limit && previous_pnl > self.config.daily_loss_limit {
            let alert = Alert::new(
                AlertSeverity::Critical,
                AlertCategory::RiskManagement,
                "LOSS LIMIT EXCEEDED",
                format!(
                    "Daily PnL: ${:.2} exceeded limit ${:.2}. TRADING MUST BE STOPPED!",
                    pnl, self.config.daily_loss_limit
                ),
            )
            .with_metadata("current_pnl", &format!("{:.2}", pnl))
            .with_metadata("limit", &format!("{:.2}", self.config.daily_loss_limit));

            self.alert_manager.send(alert).await;
        }
    }

    /// Update latency
    pub async fn update_latency(&self, latency_ms: f64) {
        let mut state = self.state.write().await;
        state.order_latency_ms = latency_ms;

        if latency_ms > self.config.max_latency_ms {
            let alert = Alert::new(
                AlertSeverity::Warning,
                AlertCategory::Performance,
                "High execution latency",
                format!(
                    "Current latency: {:.0}ms (threshold: {:.0}ms)",
                    latency_ms, self.config.max_latency_ms
                ),
            )
            .with_metadata("latency_ms", &format!("{:.0}", latency_ms))
            .with_metadata("threshold_ms", &format!("{:.0}", self.config.max_latency_ms));

            self.alert_manager.send(alert).await;
        }
    }

    /// Register heartbeat
    pub async fn heartbeat(&self) {
        let mut state = self.state.write().await;
        state.last_heartbeat = Some(chrono::Utc::now());
    }

    /// Check system health (call periodically)
    pub async fn check_health(&self) {
        let state = self.state.read().await;

        // Check heartbeat
        if let Some(last) = state.last_heartbeat {
            let elapsed = (chrono::Utc::now() - last).num_seconds() as u64;
            if elapsed > self.config.heartbeat_timeout_secs {
                let alert = Alert::new(
                    AlertSeverity::Critical,
                    AlertCategory::Performance,
                    "Bot not responding",
                    format!(
                        "Last heartbeat {} seconds ago. Bot may be frozen.",
                        elapsed
                    ),
                )
                .with_metadata("last_heartbeat", &last.to_rfc3339())
                .with_metadata("elapsed_secs", &elapsed.to_string());

                self.alert_manager.send(alert).await;
            }
        }

        // Check error count
        if state.error_count_1h > self.config.max_errors_per_hour {
            let alert = Alert::new(
                AlertSeverity::Error,
                AlertCategory::Performance,
                "High error rate",
                format!(
                    "{} errors in the last hour (threshold: {})",
                    state.error_count_1h, self.config.max_errors_per_hour
                ),
            )
            .with_metadata("error_count", &state.error_count_1h.to_string());

            self.alert_manager.send(alert).await;
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Trading Bot Alert System ===\n");

    // Create alert manager
    let mut alert_manager = AlertManager::new();
    alert_manager.add_channel(Arc::new(ConsoleChannel));

    let alert_manager = Arc::new(alert_manager);

    // Create monitor
    let config = MonitorConfig {
        daily_loss_limit: -500.0,
        max_latency_ms: 300.0,
        max_errors_per_hour: 5,
        heartbeat_timeout_secs: 30,
    };

    let monitor = TradingBotHealthMonitor::new(Arc::clone(&alert_manager), config);

    // Simulate normal operation
    println!("--- Simulating normal operation ---");
    monitor.update_exchange_connection("binance", true).await;
    monitor.update_pnl(50.0).await;
    monitor.update_latency(150.0).await;
    monitor.heartbeat().await;

    println!("\n--- Simulating problems ---");

    // Connection loss
    monitor.update_exchange_connection("binance", false).await;

    // PnL degradation
    monitor.update_pnl(-400.0).await;
    monitor.update_pnl(-520.0).await;

    // High latency
    monitor.update_latency(450.0).await;

    // Recovery
    println!("\n--- Recovery ---");
    monitor.update_exchange_connection("binance", true).await;
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Alert** | Structured notification about a problem |
| **AlertChannel** | Alert delivery channel (Telegram, Slack, PagerDuty) |
| **AlertManager** | Alert sending coordinator with deduplication |
| **AlertRule** | Condition for generating an alert based on a metric |
| **Severity** | Criticality level (Info, Warning, Error, Critical) |
| **Cooldown** | Period for suppressing repeated alerts |
| **Prometheus Alertmanager** | Industry standard for alert management |

## Practical Exercises

1. **Multi-channel alert system**: Create a system that:
   - Sends Info alerts only to logs
   - Warning alerts to Telegram
   - Error and Critical — to Telegram + Email + PagerDuty
   - Supports escalation if alert is not acknowledged within 10 minutes

2. **Smart trading alerts**: Implement alerts for:
   - Abnormal trading volume (3x above average)
   - Stuck orders (not executed for more than 5 minutes)
   - Price divergence between exchanges over 0.5%
   - Open position limit exceeded

3. **Status dashboard**: Create an HTTP endpoint:
   - Shows all active alerts
   - Alert history for the last 24 hours
   - Status of all delivery channels
   - Metrics: alert counts by category

4. **Alert testing**: Write tests:
   - Unit tests for AlertCondition rules
   - Mock channels for send verification
   - Integration test for full cycle
   - Load test (1000 alerts/sec)

## Homework

1. **On-call system**: Implement:
   - Duty schedule (who receives alerts when)
   - Automatic escalation if no response
   - Alert acknowledgment
   - Response time report

2. **Intelligent alerts**: Create a system with:
   - Grouping related alerts (5 connection errors = 1 alert)
   - Alert correlation (high latency + many errors = network problem)
   - Automatic false positive detection
   - ML model for problem prediction

3. **Alerts with actions**: Implement:
   - Alerts with action buttons (stop bot, close positions)
   - Webhook for receiving commands
   - Dangerous action confirmation
   - Audit log of all actions

4. **Mobile application**: Create:
   - Push notifications to phone
   - Alert management from app
   - Metric graphs
   - Quick actions (acknowledge, resolve)

## Navigation

[← Previous day](../341-distributed-tracing/en.md) | [Next day →](../343-docker-containerization/en.md)
