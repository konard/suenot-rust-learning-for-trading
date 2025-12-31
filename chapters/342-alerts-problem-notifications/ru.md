# День 342: Алерты: уведомления о проблемах

## Аналогия из трейдинга

Представь, что ты управляешь торговым ботом, который работает 24/7. Ты не можешь постоянно смотреть на экран, но тебе нужно мгновенно узнавать о критических ситуациях:

- **Потеря соединения с биржей** — как если бы твой терминал внезапно отключился во время активной торговли
- **Превышение лимита убытков** — как стоп-лосс для всего портфеля
- **Аномальная волатильность** — как сирена на торговом этаже
- **Ошибка исполнения ордера** — как уведомление от брокера о проблеме со сделкой

В продакшене алерты — это твои «глаза и уши», которые следят за системой, когда ты не можешь этого делать сам.

| Критерий | Логирование | Алерты |
|----------|-------------|--------|
| **Цель** | Запись всех событий | Немедленное уведомление о проблемах |
| **Объём** | Все события | Только критические |
| **Реакция** | Анализ постфактум | Немедленное действие |
| **Каналы** | Файлы, консоль | Email, Telegram, Slack, SMS |
| **Приоритет** | Низкий/Средний | Высокий/Критический |

## Типы алертов в торговых системах

```rust
use std::fmt;

/// Уровень критичности алерта
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Информационный — для статистики
    Info,
    /// Предупреждение — требует внимания
    Warning,
    /// Ошибка — требует вмешательства
    Error,
    /// Критический — требует немедленного действия
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

/// Категория алерта для группировки
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertCategory {
    /// Проблемы с соединением
    Connectivity,
    /// Проблемы с ордерами
    OrderExecution,
    /// Риск-менеджмент
    RiskManagement,
    /// Производительность системы
    Performance,
    /// Рыночные условия
    MarketConditions,
}

/// Структура алерта
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
    // Примеры алертов
    let connection_alert = Alert::new(
        AlertSeverity::Critical,
        AlertCategory::Connectivity,
        "Потеря соединения с Binance",
        "WebSocket соединение разорвано, попытка переподключения...",
    )
    .with_metadata("exchange", "binance")
    .with_metadata("retry_count", "3");

    let risk_alert = Alert::new(
        AlertSeverity::Warning,
        AlertCategory::RiskManagement,
        "Приближение к дневному лимиту убытков",
        "Текущий убыток: -$450, лимит: -$500",
    )
    .with_metadata("current_loss", "-450")
    .with_metadata("limit", "-500");

    println!("[{}] {}: {}", connection_alert.severity, connection_alert.title, connection_alert.message);
    println!("[{}] {}: {}", risk_alert.severity, risk_alert.title, risk_alert.message);
}
```

## Система отправки алертов

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Трейт для каналов уведомлений
#[async_trait::async_trait]
pub trait AlertChannel: Send + Sync {
    /// Название канала
    fn name(&self) -> &str;

    /// Отправить алерт
    async fn send(&self, alert: &Alert) -> Result<(), AlertError>;

    /// Поддерживает ли канал данный уровень критичности
    fn supports_severity(&self, severity: AlertSeverity) -> bool;
}

#[derive(Debug)]
pub struct AlertError {
    pub message: String,
    pub channel: String,
}

/// Канал отправки в консоль (для разработки)
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
            "\n{} [{}] {}\n   {}\n   Время: {}\n   Категория: {:?}",
            emoji,
            alert.severity,
            alert.title,
            alert.message,
            alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.category
        );

        if !alert.metadata.is_empty() {
            println!("   Метаданные:");
            for (key, value) in &alert.metadata {
                println!("     - {}: {}", key, value);
            }
        }

        Ok(())
    }

    fn supports_severity(&self, _severity: AlertSeverity) -> bool {
        true // Консоль принимает все уровни
    }
}

/// Канал отправки в Telegram
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
            "{} *{}*\n\n{}\n\n_Время: {}_\n_Категория: {:?}_",
            emoji,
            alert.title,
            alert.message,
            alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.category
        );

        // В реальном коде здесь был бы HTTP запрос к Telegram API
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

        println!("[Telegram] Отправлено в чат {}: {}", self.chat_id, message);
        Ok(())
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity >= self.min_severity
    }
}

/// Менеджер алертов
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
            dedup_window_secs: 300, // 5 минут
        }
    }

    pub fn add_channel(&mut self, channel: Arc<dyn AlertChannel>) {
        self.channels.push(channel);
    }

    /// Проверяет, не был ли похожий алерт отправлен недавно
    async fn is_duplicate(&self, alert: &Alert) -> bool {
        let history = self.alert_history.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(self.dedup_window_secs as i64);

        history.iter().any(|existing| {
            existing.title == alert.title
                && existing.category == alert.category
                && existing.timestamp > cutoff
        })
    }

    /// Отправить алерт во все подходящие каналы
    pub async fn send(&self, alert: Alert) -> Vec<Result<(), AlertError>> {
        // Проверяем дедупликацию
        if self.is_duplicate(&alert).await {
            println!("[AlertManager] Дубликат алерта пропущен: {}", alert.title);
            return vec![];
        }

        // Сохраняем в историю
        {
            let mut history = self.alert_history.write().await;
            history.push(alert.clone());

            // Очищаем старые записи
            let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
            history.retain(|a| a.timestamp > cutoff);
        }

        // Отправляем во все подходящие каналы
        let mut results = Vec::new();

        for channel in &self.channels {
            if channel.supports_severity(alert.severity) {
                let result = channel.send(&alert).await;
                if let Err(ref e) = result {
                    eprintln!(
                        "[AlertManager] Ошибка отправки в {}: {}",
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

    // Добавляем каналы
    manager.add_channel(Arc::new(ConsoleChannel));
    manager.add_channel(Arc::new(TelegramChannel::new(
        "BOT_TOKEN".to_string(),
        "CHAT_ID".to_string(),
        AlertSeverity::Warning, // Только Warning и выше
    )));

    // Отправляем алерты
    let alert = Alert::new(
        AlertSeverity::Critical,
        AlertCategory::Connectivity,
        "Потеря соединения с биржей",
        "Binance WebSocket отключён, активные ордера могут не обновляться",
    )
    .with_metadata("exchange", "binance")
    .with_metadata("last_ping", "2024-01-15 10:30:00");

    manager.send(alert).await;
}
```

## Условные алерты на основе метрик

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Правило для генерации алерта
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
    /// Значение больше порога
    GreaterThan(f64),
    /// Значение меньше порога
    LessThan(f64),
    /// Значение вне диапазона
    OutOfRange { min: f64, max: f64 },
    /// Изменение за период больше порога (в процентах)
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

/// Мониторинг метрик с генерацией алертов
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

    /// Записать значение метрики
    pub async fn record_metric(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        let history = metrics.entry(name.to_string()).or_insert_with(Vec::new);

        history.push((chrono::Utc::now(), value));

        // Храним только последние 24 часа
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        history.retain(|(ts, _)| *ts > cutoff);
    }

    /// Проверить все правила и вернуть сработавшие алерты
    pub async fn check_rules(&self) -> Vec<Alert> {
        let metrics = self.metrics.read().await;
        let mut last_alerts = self.last_alerts.write().await;
        let mut alerts = Vec::new();
        let now = chrono::Utc::now();

        for rule in &self.rules {
            // Проверяем cooldown
            if let Some(last) = last_alerts.get(&rule.name) {
                if now - *last < chrono::Duration::seconds(rule.cooldown_secs as i64) {
                    continue;
                }
            }

            // Получаем текущее значение и историю
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

    // Правила для торговой системы
    monitor.add_rule(AlertRule {
        name: "high_latency".to_string(),
        metric_name: "order_execution_latency_ms".to_string(),
        condition: AlertCondition::GreaterThan(500.0),
        severity: AlertSeverity::Warning,
        message_template: "Высокая задержка исполнения ордеров: {value}ms".to_string(),
        cooldown_secs: 300,
    });

    monitor.add_rule(AlertRule {
        name: "daily_loss_limit".to_string(),
        metric_name: "daily_pnl_usd".to_string(),
        condition: AlertCondition::LessThan(-500.0),
        severity: AlertSeverity::Critical,
        message_template: "Превышен дневной лимит убытков! PnL: ${value}".to_string(),
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
        message_template: "Резкое изменение цены BTC: ${value}".to_string(),
        cooldown_secs: 600,
    });

    // Симуляция записи метрик
    monitor.record_metric("order_execution_latency_ms", 150.0).await;
    monitor.record_metric("daily_pnl_usd", 100.0).await;
    monitor.record_metric("btc_price", 50000.0).await;

    // Проверяем правила
    let alerts = monitor.check_rules().await;
    println!("Сработавших алертов: {}", alerts.len());

    // Симулируем проблему
    monitor.record_metric("order_execution_latency_ms", 750.0).await;
    monitor.record_metric("daily_pnl_usd", -550.0).await;

    let alerts = monitor.check_rules().await;
    for alert in alerts {
        println!("[{}] {}: {}", alert.severity, alert.title, alert.message);
    }
}
```

## Интеграция с Prometheus Alertmanager

```rust
use std::collections::HashMap;

/// Алерт в формате Prometheus Alertmanager
#[derive(Debug, serde::Serialize)]
pub struct PrometheusAlert {
    pub status: String, // "firing" или "resolved"
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

/// Клиент для отправки алертов в Prometheus Alertmanager
pub struct AlertmanagerClient {
    url: String,
}

impl AlertmanagerClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    /// Отправить алерты в Alertmanager
    pub async fn send_alerts(&self, alerts: Vec<PrometheusAlert>) -> Result<(), String> {
        // В реальном коде:
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
            "[Alertmanager] Отправлено {} алертов на {}",
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

    // Создаём алерт о проблеме с соединением
    let connection_alert = PrometheusAlert::firing(
        "TradingBotConnectionLost",
        "critical",
        "Потеря соединения с биржей Binance",
        "WebSocket соединение разорвано. Последний пинг: 5 минут назад.",
    )
    .with_label("exchange", "binance")
    .with_label("service", "trading-bot")
    .with_label("environment", "production");

    // Создаём алерт о высокой латентности
    let latency_alert = PrometheusAlert::firing(
        "HighOrderLatency",
        "warning",
        "Высокая задержка исполнения ордеров",
        "Средняя латентность за последние 5 минут: 850ms (порог: 500ms)",
    )
    .with_label("exchange", "binance")
    .with_label("service", "trading-bot");

    client
        .send_alerts(vec![connection_alert, latency_alert])
        .await
        .expect("Ошибка отправки алертов");

    // Позже, когда проблема решена
    let resolved = PrometheusAlert::resolved("TradingBotConnectionLost")
        .with_label("exchange", "binance")
        .with_label("service", "trading-bot");

    client
        .send_alerts(vec![resolved])
        .await
        .expect("Ошибка отправки resolved");
}
```

## Webhook алерты для интеграций

```rust
use std::collections::HashMap;
use std::sync::Arc;

/// Формат webhook payload
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

/// Канал отправки webhook
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

        // В реальном коде:
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

/// Пример интеграции с PagerDuty
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

        // В реальном коде:
        // let client = reqwest::Client::new();
        // client
        //     .post("https://events.pagerduty.com/v2/enqueue")
        //     .json(&payload)
        //     .send()
        //     .await?;

        println!(
            "[PagerDuty] Создан инцидент: {}\n{}",
            alert.title,
            serde_json::to_string_pretty(&payload).unwrap()
        );

        Ok(())
    }

    fn supports_severity(&self, severity: AlertSeverity) -> bool {
        severity >= AlertSeverity::Error // Только Error и Critical
    }
}

fn main() {
    println!("Webhook integrations example");
}
```

## Полная система алертов для торгового бота

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Состояние торгового бота для мониторинга
#[derive(Default)]
pub struct TradingBotState {
    pub connected_exchanges: HashMap<String, bool>,
    pub daily_pnl: f64,
    pub open_positions: usize,
    pub order_latency_ms: f64,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    pub error_count_1h: usize,
}

/// Монитор здоровья торгового бота
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

    /// Обновить состояние подключения к бирже
    pub async fn update_exchange_connection(&self, exchange: &str, connected: bool) {
        let mut state = self.state.write().await;
        let was_connected = state.connected_exchanges.get(exchange).copied().unwrap_or(true);
        state.connected_exchanges.insert(exchange.to_string(), connected);

        // Генерируем алерт при потере соединения
        if was_connected && !connected {
            let alert = Alert::new(
                AlertSeverity::Critical,
                AlertCategory::Connectivity,
                format!("Потеря соединения: {}", exchange),
                format!("Соединение с биржей {} потеряно. Торговля приостановлена.", exchange),
            )
            .with_metadata("exchange", exchange);

            self.alert_manager.send(alert).await;
        }

        // Генерируем алерт при восстановлении
        if !was_connected && connected {
            let alert = Alert::new(
                AlertSeverity::Info,
                AlertCategory::Connectivity,
                format!("Соединение восстановлено: {}", exchange),
                format!("Соединение с биржей {} восстановлено.", exchange),
            )
            .with_metadata("exchange", exchange);

            self.alert_manager.send(alert).await;
        }
    }

    /// Обновить PnL
    pub async fn update_pnl(&self, pnl: f64) {
        let mut state = self.state.write().await;
        let previous_pnl = state.daily_pnl;
        state.daily_pnl = pnl;

        // Алерт при приближении к лимиту (80%)
        let warning_threshold = self.config.daily_loss_limit * 0.8;
        if pnl <= warning_threshold && previous_pnl > warning_threshold {
            let alert = Alert::new(
                AlertSeverity::Warning,
                AlertCategory::RiskManagement,
                "Приближение к лимиту убытков",
                format!(
                    "Текущий PnL: ${:.2}, лимит: ${:.2} (осталось: ${:.2})",
                    pnl,
                    self.config.daily_loss_limit,
                    self.config.daily_loss_limit - pnl
                ),
            )
            .with_metadata("current_pnl", &format!("{:.2}", pnl))
            .with_metadata("limit", &format!("{:.2}", self.config.daily_loss_limit));

            self.alert_manager.send(alert).await;
        }

        // Критический алерт при превышении лимита
        if pnl <= self.config.daily_loss_limit && previous_pnl > self.config.daily_loss_limit {
            let alert = Alert::new(
                AlertSeverity::Critical,
                AlertCategory::RiskManagement,
                "ПРЕВЫШЕН ЛИМИТ УБЫТКОВ",
                format!(
                    "Дневной PnL: ${:.2} превысил лимит ${:.2}. ТОРГОВЛЯ ДОЛЖНА БЫТЬ ОСТАНОВЛЕНА!",
                    pnl, self.config.daily_loss_limit
                ),
            )
            .with_metadata("current_pnl", &format!("{:.2}", pnl))
            .with_metadata("limit", &format!("{:.2}", self.config.daily_loss_limit));

            self.alert_manager.send(alert).await;
        }
    }

    /// Обновить латентность
    pub async fn update_latency(&self, latency_ms: f64) {
        let mut state = self.state.write().await;
        state.order_latency_ms = latency_ms;

        if latency_ms > self.config.max_latency_ms {
            let alert = Alert::new(
                AlertSeverity::Warning,
                AlertCategory::Performance,
                "Высокая латентность исполнения",
                format!(
                    "Текущая латентность: {:.0}ms (порог: {:.0}ms)",
                    latency_ms, self.config.max_latency_ms
                ),
            )
            .with_metadata("latency_ms", &format!("{:.0}", latency_ms))
            .with_metadata("threshold_ms", &format!("{:.0}", self.config.max_latency_ms));

            self.alert_manager.send(alert).await;
        }
    }

    /// Зарегистрировать heartbeat
    pub async fn heartbeat(&self) {
        let mut state = self.state.write().await;
        state.last_heartbeat = Some(chrono::Utc::now());
    }

    /// Проверить здоровье системы (вызывать периодически)
    pub async fn check_health(&self) {
        let state = self.state.read().await;

        // Проверяем heartbeat
        if let Some(last) = state.last_heartbeat {
            let elapsed = (chrono::Utc::now() - last).num_seconds() as u64;
            if elapsed > self.config.heartbeat_timeout_secs {
                let alert = Alert::new(
                    AlertSeverity::Critical,
                    AlertCategory::Performance,
                    "Бот не отвечает",
                    format!(
                        "Последний heartbeat {} секунд назад. Возможно бот завис.",
                        elapsed
                    ),
                )
                .with_metadata("last_heartbeat", &last.to_rfc3339())
                .with_metadata("elapsed_secs", &elapsed.to_string());

                self.alert_manager.send(alert).await;
            }
        }

        // Проверяем количество ошибок
        if state.error_count_1h > self.config.max_errors_per_hour {
            let alert = Alert::new(
                AlertSeverity::Error,
                AlertCategory::Performance,
                "Высокий уровень ошибок",
                format!(
                    "За последний час {} ошибок (порог: {})",
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
    println!("=== Система алертов торгового бота ===\n");

    // Создаём менеджер алертов
    let mut alert_manager = AlertManager::new();
    alert_manager.add_channel(Arc::new(ConsoleChannel));

    let alert_manager = Arc::new(alert_manager);

    // Создаём монитор
    let config = MonitorConfig {
        daily_loss_limit: -500.0,
        max_latency_ms: 300.0,
        max_errors_per_hour: 5,
        heartbeat_timeout_secs: 30,
    };

    let monitor = TradingBotHealthMonitor::new(Arc::clone(&alert_manager), config);

    // Симуляция работы
    println!("--- Симуляция нормальной работы ---");
    monitor.update_exchange_connection("binance", true).await;
    monitor.update_pnl(50.0).await;
    monitor.update_latency(150.0).await;
    monitor.heartbeat().await;

    println!("\n--- Симуляция проблем ---");

    // Потеря соединения
    monitor.update_exchange_connection("binance", false).await;

    // Ухудшение PnL
    monitor.update_pnl(-400.0).await;
    monitor.update_pnl(-520.0).await;

    // Высокая латентность
    monitor.update_latency(450.0).await;

    // Восстановление
    println!("\n--- Восстановление ---");
    monitor.update_exchange_connection("binance", true).await;
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Alert** | Структурированное уведомление о проблеме |
| **AlertChannel** | Канал доставки алерта (Telegram, Slack, PagerDuty) |
| **AlertManager** | Координатор отправки алертов с дедупликацией |
| **AlertRule** | Условие генерации алерта на основе метрики |
| **Severity** | Уровень критичности (Info, Warning, Error, Critical) |
| **Cooldown** | Период подавления повторных алертов |
| **Prometheus Alertmanager** | Стандарт индустрии для управления алертами |

## Практические задания

1. **Многоканальная система алертов**: Создай систему, которая:
   - Отправляет Info алерты только в логи
   - Warning алерты в Telegram
   - Error и Critical — в Telegram + Email + PagerDuty
   - Поддерживает эскалацию, если алерт не подтверждён за 10 минут

2. **Умные алерты для трейдинга**: Реализуй алерты:
   - Аномальный объём торгов (в 3 раза выше среднего)
   - Застрявшие ордера (не исполнены более 5 минут)
   - Расхождение цен между биржами более 0.5%
   - Превышение лимита открытых позиций

3. **Dashboard состояния**: Создай HTTP endpoint:
   - Показывает все активные алерты
   - История алертов за последние 24 часа
   - Статус всех каналов доставки
   - Метрики: количество алертов по категориям

4. **Тестирование алертов**: Напиши тесты:
   - Unit-тесты для правил AlertCondition
   - Mock-каналы для проверки отправки
   - Интеграционный тест полного цикла
   - Нагрузочный тест (1000 алертов/сек)

## Домашнее задание

1. **Система on-call**: Реализуй:
   - Расписание дежурств (кто получает алерты в какое время)
   - Автоматическую эскалацию при отсутствии ответа
   - Подтверждение алерта (acknowledge)
   - Отчёт о времени реакции

2. **Интеллектуальные алерты**: Создай систему:
   - Группировка связанных алертов (5 ошибок подключения = 1 алерт)
   - Корреляция алертов (высокая латентность + много ошибок = проблема с сетью)
   - Автоматическое определение ложных срабатываний
   - ML-модель для предсказания проблем

3. **Алерты с действиями**: Реализуй:
   - Алерты с кнопками действий (остановить бота, закрыть позиции)
   - Webhook для получения команд
   - Подтверждение опасных действий
   - Аудит-лог всех действий

4. **Мобильное приложение**: Создай:
   - Push-уведомления на телефон
   - Управление алертами из приложения
   - Графики метрик
   - Быстрые действия (acknowledge, resolve)

## Навигация

[← Предыдущий день](../341-distributed-tracing/ru.md) | [Следующий день →](../343-docker-containerization/ru.md)
