# День 362: Пост-мортем: анализ инцидентов

## Аналогия из трейдинга

Представь, что ты управляешь торговым деском, и один из твоих алгоритмов внезапно исполняет 10 000 непреднамеренных сделок за 30 секунд, потеряв $2 миллиона. После того как кровотечение остановлено, что происходит дальше?

**Пост-мортем — это как расследование на торговой площадке после крупной потери:**

Как опытные трейдеры анализируют каждую деталь после рыночной катастрофы — что произошло, когда, почему и как предотвратить это в будущем — так и инженеры проводят пост-мортем анализ для понимания сбоев системы и предотвращения их повторения.

| Торговое расследование | Пост-мортем ПО |
|-----------------------|----------------|
| **Расчёт убытков** | Оценка влияния (простой, потеря данных, выручка) |
| **Реконструкция сделок** | Восстановление хронологии из логов |
| **Анализ первопричины** | Поиск реального бага/точки сбоя |
| **Отказ риск-контроля** | Почему алерты/предохранители не сработали? |
| **Улучшение процессов** | Действия для предотвращения повторения |

**Принцип безобвинения:**
Хороший пост-мортем фокусируется на системах, а не на людях. Точно так же, как ты не стал бы обвинять трейдера за следование ошибочным риск-моделям — вместо этого ты исправляешь модели.

## Структура пост-мортема

```rust
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};

/// Уровни серьёзности инцидентов
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Минимальное влияние, нет проблем для клиентов
    Low,
    /// Некоторые клиенты затронуты, есть обходной путь
    Medium,
    /// Значительное влияние, частичная деградация сервиса
    High,
    /// Критический сбой, полное отключение сервиса
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "НИЗКИЙ",
            Severity::Medium => "СРЕДНИЙ",
            Severity::High => "ВЫСОКИЙ",
            Severity::Critical => "КРИТИЧЕСКИЙ",
        }
    }
}

/// Статус инцидента
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentStatus {
    /// Инцидент расследуется
    Investigating,
    /// Первопричина определена
    Identified,
    /// Исправление разрабатывается
    Fixing,
    /// Исправление развёрнуто, мониторинг
    Monitoring,
    /// Инцидент разрешён
    Resolved,
    /// Пост-мортем завершён
    Closed,
}

/// Событие хронологии во время инцидента
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

/// Метрики влияния инцидента
#[derive(Debug, Clone)]
pub struct IncidentImpact {
    /// Продолжительность инцидента
    pub duration: Duration,
    /// Оценочная потеря выручки
    pub revenue_loss: f64,
    /// Количество затронутых клиентов
    pub affected_customers: u64,
    /// Количество неудачных транзакций
    pub failed_transactions: u64,
    /// Потеря данных (если была)
    pub data_loss: Option<String>,
    /// Нарушение SLA
    pub sla_breach: bool,
}

/// Категории первопричин
#[derive(Debug, Clone)]
pub enum RootCauseCategory {
    /// Баг или логическая ошибка в коде
    CodeDefect,
    /// Сбой инфраструктуры (железо, сеть)
    Infrastructure,
    /// Ошибка конфигурации
    Configuration,
    /// Сбой внешнего сервиса
    ExternalDependency,
    /// Человеческая ошибка при эксплуатации
    OperationalError,
    /// Проблемы мощности/масштабирования
    Capacity,
    /// Инцидент безопасности
    Security,
    /// Неизвестно (ещё расследуется)
    Unknown,
}

/// Элемент плана действий из пост-мортема
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
    P0, // Немедленно
    P1, // На этой неделе
    P2, // В этом месяце
    P3, // В бэклог
}

#[derive(Debug, Clone, Copy)]
pub enum ActionStatus {
    Open,
    InProgress,
    Done,
    WontFix,
}

/// Полный документ пост-мортема
#[derive(Debug, Clone)]
pub struct PostMortem {
    /// Уникальный идентификатор инцидента
    pub incident_id: String,
    /// Краткий заголовок, описывающий инцидент
    pub title: String,
    /// Подробное описание
    pub summary: String,
    /// Уровень серьёзности
    pub severity: Severity,
    /// Текущий статус
    pub status: IncidentStatus,
    /// Когда инцидент начался
    pub started_at: DateTime<Utc>,
    /// Когда инцидент был обнаружен
    pub detected_at: DateTime<Utc>,
    /// Когда инцидент был разрешён
    pub resolved_at: Option<DateTime<Utc>>,
    /// Хронология событий
    pub timeline: Vec<TimelineEvent>,
    /// Оценка влияния
    pub impact: IncidentImpact,
    /// Категория первопричины
    pub root_cause_category: RootCauseCategory,
    /// Детальный анализ первопричины
    pub root_cause_analysis: String,
    /// Что сработало хорошо
    pub what_went_well: Vec<String>,
    /// Что пошло не так
    pub what_went_wrong: Vec<String>,
    /// Где нам повезло
    pub where_we_got_lucky: Vec<String>,
    /// План действий
    pub action_items: Vec<ActionItem>,
    /// Автор пост-мортема
    pub author: String,
    /// Участники инцидента
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

        report.push_str(&format!("# Пост-мортем: {}\n\n", self.title));
        report.push_str(&format!("**ID инцидента:** {}\n", self.incident_id));
        report.push_str(&format!("**Серьёзность:** {}\n", self.severity.as_str()));
        report.push_str(&format!("**Статус:** {:?}\n", self.status));
        report.push_str(&format!("**Автор:** {}\n\n", self.author));

        report.push_str("## Краткое описание\n\n");
        report.push_str(&format!("{}\n\n", self.summary));

        report.push_str("## Хронология\n\n");
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

        report.push_str("## Влияние\n\n");
        report.push_str(&format!("- **Продолжительность:** {:?}\n", self.impact.duration));
        report.push_str(&format!("- **Потеря выручки:** ${:.2}\n", self.impact.revenue_loss));
        report.push_str(&format!("- **Затронутые клиенты:** {}\n", self.impact.affected_customers));
        report.push_str(&format!("- **Неудачные транзакции:** {}\n", self.impact.failed_transactions));
        report.push_str(&format!("- **Нарушение SLA:** {}\n\n", self.impact.sla_breach));

        report.push_str("## Анализ первопричины\n\n");
        report.push_str(&format!("**Категория:** {:?}\n\n", self.root_cause_category));
        report.push_str(&format!("{}\n\n", self.root_cause_analysis));

        report.push_str("## Что сработало хорошо\n\n");
        for item in &self.what_went_well {
            report.push_str(&format!("- {}\n", item));
        }
        report.push_str("\n");

        report.push_str("## Что пошло не так\n\n");
        for item in &self.what_went_wrong {
            report.push_str(&format!("- {}\n", item));
        }
        report.push_str("\n");

        report.push_str("## Где нам повезло\n\n");
        for item in &self.where_we_got_lucky {
            report.push_str(&format!("- {}\n", item));
        }
        report.push_str("\n");

        report.push_str("## План действий\n\n");
        for item in &self.action_items {
            report.push_str(&format!(
                "- [{}] **{}** (Ответственный: {}, Приоритет: {:?}): {}\n",
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
    println!("=== Демо структуры пост-мортема ===\n");

    let mut pm = PostMortem::new(
        "INC-2024-0142",
        "Отказ движка матчинга ордеров",
        Severity::Critical
    );

    pm.summary = "Движок матчинга ордеров стал недоступен на 47 минут \
                  из-за утечки памяти в логике восстановления ордербука, \
                  вызванной необычной последовательностью cancel-replace ордеров.".to_string();

    pm.author = "Иван Петров".to_string();
    pm.participants = vec!["Иван Петров".to_string(), "Мария Сидорова".to_string(), "Алексей Козлов".to_string()];

    pm.add_timeline_event(
        "Алерт мониторинга: Задержка обработки ордеров > 500мс",
        "PagerDuty",
        EventType::Alert
    );

    pm.add_timeline_event(
        "Дежурный инженер подтвердил алерт",
        "Алексей Козлов",
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
    pm.root_cause_analysis = "Утечка памяти была внесена в коммите abc123, когда \
        логика восстановления ордербука была изменена. Когда последовательность \
        cancel-replace ордеров превышала 1000 ордеров в окне 1 секунды, старые объекты \
        ордеров не освобождались должным образом.".to_string();

    pm.what_went_well = vec![
        "Алерт сработал в течение 2 минут после начала проблемы".to_string(),
        "Команда быстро собралась и начала расследование".to_string(),
        "Процедура отката сработала как ожидалось".to_string(),
    ];

    pm.what_went_wrong = vec![
        "Утечка памяти не была обнаружена при нагрузочном тестировании".to_string(),
        "Нет автоматического выключателя для высокой частоты cancel-replace".to_string(),
        "Начальное расследование пошло по неправильному пути".to_string(),
    ];

    pm.where_we_got_lucky = vec![
        "Инцидент произошёл в период низкого торгового объёма".to_string(),
        "Повреждения данных не произошло".to_string(),
    ];

    pm.action_items = vec![
        ActionItem {
            id: "AI-001".to_string(),
            description: "Добавить стресс-тест для последовательностей cancel-replace".to_string(),
            owner: "Алексей Козлов".to_string(),
            priority: Priority::P0,
            due_date: None,
            status: ActionStatus::Open,
        },
        ActionItem {
            id: "AI-002".to_string(),
            description: "Реализовать автоматический выключатель для ограничения скорости ордеров".to_string(),
            owner: "Мария Сидорова".to_string(),
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

## Обнаружение и отслеживание инцидентов

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};

/// Торговые метрики для обнаружения инцидентов
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

/// Конфигурация порогов для алертов
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

/// Сработавший алерт
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

/// Трекер инцидентов для торговых систем
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

    /// Проверить метрики и сгенерировать алерты
    pub fn check_metrics(&self, metrics: &TradingMetrics) -> Vec<Alert> {
        let mut new_alerts = Vec::new();

        // Проверка задержки ордеров
        if metrics.order_latency_ms > self.thresholds.max_order_latency_ms {
            new_alerts.push(self.create_alert(
                "high_latency",
                "Критическая задержка ордеров",
                AlertSeverity::Critical,
                format!(
                    "Задержка ордеров {}мс превышает порог {}мс",
                    metrics.order_latency_ms, self.thresholds.max_order_latency_ms
                ),
                metrics.order_latency_ms,
                self.thresholds.max_order_latency_ms,
            ));
        }

        // Проверка успешности
        if metrics.order_success_rate < self.thresholds.min_success_rate {
            new_alerts.push(self.create_alert(
                "low_success_rate",
                "Низкий процент успешных ордеров",
                AlertSeverity::Error,
                format!(
                    "Успешность {:.1}% ниже порога {:.1}%",
                    metrics.order_success_rate, self.thresholds.min_success_rate
                ),
                metrics.order_success_rate,
                self.thresholds.min_success_rate,
            ));
        }

        // Проверка на чрезмерную скорость ордеров (возможный сбой алгоритма)
        if metrics.orders_per_second > self.thresholds.max_orders_per_second {
            new_alerts.push(self.create_alert(
                "excessive_orders",
                "Чрезмерная скорость ордеров",
                AlertSeverity::Critical,
                format!(
                    "Скорость ордеров {:.0}/с превышает лимит безопасности {:.0}/с",
                    metrics.orders_per_second, self.thresholds.max_orders_per_second
                ),
                metrics.orders_per_second,
                self.thresholds.max_orders_per_second,
            ));
        }

        // Проверка порога убытков P&L
        if metrics.position_pnl < self.thresholds.max_loss_amount {
            new_alerts.push(self.create_alert(
                "excessive_loss",
                "Превышен порог убытков P&L",
                AlertSeverity::Critical,
                format!(
                    "P&L ${:.2} превышает лимит убытков ${:.2}",
                    metrics.position_pnl, self.thresholds.max_loss_amount
                ),
                metrics.position_pnl,
                self.thresholds.max_loss_amount,
            ));
        }

        // Проверка количества подключений
        if metrics.connection_count < self.thresholds.min_connections {
            new_alerts.push(self.create_alert(
                "low_connections",
                "Потеряны подключения к биржам",
                AlertSeverity::Critical,
                format!(
                    "Только {} активных подключений, требуется минимум {}",
                    metrics.connection_count, self.thresholds.min_connections
                ),
                metrics.connection_count as f64,
                self.thresholds.min_connections as f64,
            ));
        }

        // Проверка использования памяти
        if metrics.memory_usage_mb > self.thresholds.max_memory_mb {
            new_alerts.push(self.create_alert(
                "high_memory",
                "Высокое использование памяти",
                AlertSeverity::Warning,
                format!(
                    "Использование памяти {}МБ превышает порог {}МБ",
                    metrics.memory_usage_mb, self.thresholds.max_memory_mb
                ),
                metrics.memory_usage_mb,
                self.thresholds.max_memory_mb,
            ));
        }

        // Сохранение новых алертов
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

    /// Подтвердить алерт
    pub fn acknowledge_alert(&self, alert_id: &str, by: &str) {
        let mut active = self.active_alerts.write().unwrap();
        if let Some(alert) = active.get_mut(alert_id) {
            alert.acknowledged = true;
            println!("[{}] Алерт '{}' подтверждён пользователем {}", Utc::now().format("%H:%M:%S"), alert_id, by);
        }
    }

    /// Разрешить алерт
    pub fn resolve_alert(&self, alert_id: &str) {
        let mut active = self.active_alerts.write().unwrap();
        if let Some(alert) = active.remove(alert_id) {
            println!("[{}] Алерт '{}' разрешён после {:?}",
                Utc::now().format("%H:%M:%S"),
                alert_id,
                Utc::now().signed_duration_since(alert.triggered_at)
            );
        }
    }

    /// Получить все активные алерты
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.read().unwrap().values().cloned().collect()
    }

    /// Сгенерировать сводку инцидентов
    pub fn generate_summary(&self) -> String {
        let active = self.active_alerts.read().unwrap();
        let history = self.alert_history.read().unwrap();

        let critical_count = active.values().filter(|a| a.severity == AlertSeverity::Critical).count();
        let error_count = active.values().filter(|a| a.severity == AlertSeverity::Error).count();
        let warning_count = active.values().filter(|a| a.severity == AlertSeverity::Warning).count();

        format!(
            "Сводка инцидентов:\n\
             - Активные алерты: {} (Критических: {}, Ошибок: {}, Предупреждений: {})\n\
             - Всего алертов (история): {}\n\
             - Неподтверждённые: {}",
            active.len(), critical_count, error_count, warning_count,
            history.len(),
            active.values().filter(|a| !a.acknowledged).count()
        )
    }
}

fn main() {
    println!("=== Демо обнаружения инцидентов ===\n");

    let tracker = IncidentTracker::new(AlertThresholds::default());

    // Нормальные метрики — нет алертов
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
    println!("Проверка нормальных метрик: {} алертов", alerts.len());

    // Проблемные метрики — множественные алерты
    let problem_metrics = TradingMetrics {
        order_latency_ms: 250.0,        // Слишком высокая
        order_success_rate: 85.0,       // Слишком низкая
        orders_per_second: 15000.0,     // Сбой алгоритма?
        position_pnl: -150000.0,        // Чрезмерные убытки
        connection_count: 1,            // Потеряны подключения
        memory_usage_mb: 4096.0,
        cpu_usage_percent: 45.0,
    };

    let alerts = tracker.check_metrics(&problem_metrics);
    println!("\nПроверка проблемных метрик: {} алертов\n", alerts.len());

    for alert in &alerts {
        println!("АЛЕРТ [{:?}] {}: {}", alert.severity, alert.name, alert.message);
    }

    // Подтверждение некоторых алертов
    println!("\n--- Подтверждение алертов ---");
    tracker.acknowledge_alert("high_latency", "Дежурный инженер");
    tracker.acknowledge_alert("excessive_loss", "Риск-менеджер");

    // Показать сводку
    println!("\n{}", tracker.generate_summary());

    // Разрешить алерт
    println!("\n--- Разрешение алерта ---");
    tracker.resolve_alert("high_latency");

    println!("\n{}", tracker.generate_summary());
}
```

## Фреймворк анализа первопричин

```rust
use std::collections::HashMap;

/// Техника "5 почему" для анализа первопричин
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
            format!("Почему {}?", self.incident_description)
        } else {
            format!("Почему {}?", self.whys.last().unwrap().answer)
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

        report.push_str("# Анализ первопричины методом 5 почему\n\n");
        report.push_str(&format!("**Инцидент:** {}\n\n", self.incident_description));

        for why in &self.whys {
            report.push_str(&format!("## Почему #{}\n\n", why.level));
            report.push_str(&format!("**В:** {}\n\n", why.question));
            report.push_str(&format!("**О:** {}\n\n", why.answer));

            if !why.evidence.is_empty() {
                report.push_str("**Доказательства:**\n");
                for e in &why.evidence {
                    report.push_str(&format!("- {}\n", e));
                }
                report.push_str("\n");
            }
        }

        if let Some(root_cause) = &self.root_cause {
            report.push_str(&format!("## Первопричина\n\n{}\n\n", root_cause));
        }

        if !self.contributing_factors.is_empty() {
            report.push_str("## Сопутствующие факторы\n\n");
            for factor in &self.contributing_factors {
                report.push_str(&format!("- {}\n", factor));
            }
        }

        report
    }
}

/// Диаграмма Исикавы (рыбья кость) для анализа причин
#[derive(Debug, Clone)]
pub struct FishboneDiagram {
    pub problem: String,
    pub categories: HashMap<CauseCategory, Vec<Cause>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CauseCategory {
    People,      // Человеческие факторы
    Process,     // Методология, процедуры
    Technology,  // Инструменты, системы, инфраструктура
    Data,        // Качество данных, доступность
    Environment, // Внешние факторы, рыночные условия
    Management,  // Организация, принятие решений
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
        println!("║                  ДИАГРАММА ИСИКАВЫ                        ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Проблема: {:48} ║", self.problem);
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
        println!("║ ★ = Первопричина                                         ║");
        println!("╚══════════════════════════════════════════════════════════╝");
    }
}

/// Анализ дерева отказов для торговых систем
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
    And,  // Все входы должны произойти
    Or,   // Любой вход может вызвать выход
}

#[derive(Debug, Clone)]
pub enum GateInput {
    BasicEvent { name: String, probability: f64 },
    Gate(String), // Ссылка на другой вентиль
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
        // Простой расчёт вероятностей (в реальности это было бы сложнее)
        for gate in &mut self.gates {
            let prob = match gate.gate_type {
                GateType::And => {
                    gate.inputs.iter().fold(1.0, |acc, input| {
                        match input {
                            GateInput::BasicEvent { probability, .. } => acc * probability,
                            GateInput::Gate(_) => acc, // Нужен рекурсивный поиск
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
        println!("║                    ДЕРЕВО ОТКАЗОВ                        ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Главное событие: {:39} ║", self.top_event);
        println!("╠══════════════════════════════════════════════════════════╣");

        for gate in &self.gates {
            let gate_symbol = match gate.gate_type {
                GateType::And => "И",
                GateType::Or => "ИЛИ",
            };
            let prob_str = gate.probability.map(|p| format!(" (P={:.6})", p)).unwrap_or_default();
            println!("║ [{}] {} {}{}", gate.id, gate_symbol, gate.description, prob_str);

            for input in &gate.inputs {
                match input {
                    GateInput::BasicEvent { name, probability } => {
                        println!("║   ├─ {} (P={:.4})", name, probability);
                    }
                    GateInput::Gate(id) => {
                        println!("║   ├─ [Вентиль: {}]", id);
                    }
                }
            }
        }

        println!("╚══════════════════════════════════════════════════════════╝");
    }
}

fn main() {
    println!("=== Фреймворк анализа первопричин ===\n");

    // Пример 5 почему
    println!("--- Анализ 5 почему ---\n");

    let mut five_whys = FiveWhysAnalysis::new("движок матчинга ордеров упал");

    five_whys.add_why(
        "процесс исчерпал память",
        vec!["Логи OOM killer показывают завершение процесса", "Графики использования памяти резко выросли до 100%"]
    );

    five_whys.add_why(
        "восстановление ордербука утекало память",
        vec!["Профилировщик кучи показывает неосвобождённые объекты Order", "Утечка началась после деплоя в 14:00"]
    );

    five_whys.add_why(
        "cancel-replace ордера не очищали старые объекты ордеров",
        vec!["Обзор кода нашёл отсутствующий вызов drop()", "Воспроизводится стресс-тестом cancel-replace"]
    );

    five_whys.add_why(
        "изменение кода в PR #4521 случайно удалило логику очистки",
        vec!["Git diff показывает удаление", "Автор подтверждает, что это было непреднамеренно"]
    );

    five_whys.add_why(
        "код-ревью не заметило баг, и тесты не покрывали этот путь",
        vec![
            "Нет теста для последовательностей cancel-replace > 1000",
            "Ревьюер сфокусировался на фиче, а не на коде очистки",
        ]
    );

    five_whys.set_root_cause(
        "Недостаточное покрытие тестами граничных случаев и процесс код-ревью \
         не включал чеклист по безопасности памяти."
    );

    five_whys.contributing_factors = vec![
        "Высокое давление по срокам выпуска фичи".to_string(),
        "Нет автоматического обнаружения утечек памяти в CI".to_string(),
        "Документация не упоминала требования по очистке".to_string(),
    ];

    println!("{}", five_whys.generate_report());

    // Пример диаграммы Исикавы
    println!("\n--- Диаграмма Исикавы ---\n");

    let mut fishbone = FishboneDiagram::new("Отказ торговой системы");

    fishbone.add_cause(CauseCategory::Technology, "Утечка памяти в ордербуке", true);
    fishbone.add_cause(CauseCategory::Technology, "Нет автовыключателя для памяти", false);
    fishbone.add_cause(CauseCategory::Process, "Недостаточное код-ревью", true);
    fishbone.add_cause(CauseCategory::Process, "Отсутствие покрытия тестами", false);
    fishbone.add_cause(CauseCategory::People, "Давление по срокам на разработчиков", false);
    fishbone.add_cause(CauseCategory::Management, "Агрессивный график релизов", false);

    fishbone.print_diagram();

    // Пример дерева отказов
    println!("\n--- Дерево отказов ---\n");

    let mut fault_tree = FaultTree::new("Отказ торговой системы");

    fault_tree.add_gate(
        "G1",
        GateType::Or,
        "Система недоступна",
        vec![
            GateInput::BasicEvent { name: "Исчерпание памяти".to_string(), probability: 0.01 },
            GateInput::BasicEvent { name: "Сбой сети".to_string(), probability: 0.005 },
            GateInput::BasicEvent { name: "Сбой БД".to_string(), probability: 0.002 },
        ]
    );

    fault_tree.add_gate(
        "G2",
        GateType::And,
        "Исчерпание памяти",
        vec![
            GateInput::BasicEvent { name: "Утечка памяти".to_string(), probability: 0.1 },
            GateInput::BasicEvent { name: "Высокая нагрузка".to_string(), probability: 0.2 },
            GateInput::BasicEvent { name: "Нет лимита памяти".to_string(), probability: 0.5 },
        ]
    );

    fault_tree.calculate_probabilities();
    fault_tree.print_tree();
}
```

## Автоматизация реагирования на инциденты

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Автоматизированное действие реагирования
#[derive(Debug, Clone)]
pub enum AutomatedAction {
    /// Закрыть все открытые позиции
    FlattenPositions,
    /// Прекратить приём новых ордеров
    DisableOrderEntry,
    /// Уменьшить лимиты позиций
    ReduceLimits { factor: f64 },
    /// Переключиться на резервную систему
    Failover { target: String },
    /// Перезапустить сервис
    RestartService { service: String },
    /// Вызвать дежурного инженера
    PageOnCall { message: String },
    /// Отправить уведомление
    Notify { channel: String, message: String },
    /// Масштабировать ресурсы
    ScaleUp { resource: String, amount: u32 },
    /// Включить автовыключатель
    EnableCircuitBreaker { name: String },
}

/// Плейбук реагирования для типов инцидентов
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

/// Координатор реагирования на инциденты
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
        println!("Добавлен плейбук: {}", playbook.name);
        self.playbooks.push(playbook);
    }

    /// Создать стандартные торговые плейбуки
    pub fn with_standard_playbooks(mut self) -> Self {
        // Плейбук сбоя алгоритма
        self.add_playbook(ResponsePlaybook {
            name: "Сбой алгоритма".to_string(),
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
                    action: AutomatedAction::PageOnCall { message: "Обнаружен сбой алгоритма".to_string() },
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

        // Плейбук отключения биржи
        self.add_playbook(ResponsePlaybook {
            name: "Отключение биржи".to_string(),
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
                        message: "Потеряны подключения к бирже".to_string(),
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
                    action: AutomatedAction::PageOnCall { message: "Запущен failover биржи".to_string() },
                    delay: Duration::from_secs(0),
                    requires_approval: false,
                },
            ],
            escalation_timeout: Duration::from_secs(60),
        });

        // Плейбук чрезмерных убытков
        self.add_playbook(ResponsePlaybook {
            name: "Чрезмерные убытки".to_string(),
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
                    action: AutomatedAction::PageOnCall { message: "Превышен дневной лимит убытков".to_string() },
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

    /// Запустить плейбук по имени
    pub fn trigger_playbook(&self, playbook_name: &str) -> Result<(), String> {
        let playbook = self.playbooks.iter()
            .find(|p| p.name == playbook_name)
            .ok_or_else(|| format!("Плейбук '{}' не найден", playbook_name))?;

        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║ РЕАГИРОВАНИЕ НА ИНЦИДЕНТ: {:31} ║", playbook_name);
        println!("╚══════════════════════════════════════════════════════════╝\n");

        // Записать активное реагирование
        {
            let mut active = self.active_responses.write().unwrap();
            active.push(ActiveResponse {
                playbook_name: playbook_name.to_string(),
                started_at: Instant::now(),
                current_action_idx: 0,
                is_complete: false,
            });
        }

        // Выполнить действия
        for (idx, action) in playbook.actions.iter().enumerate() {
            if !action.delay.is_zero() {
                println!("[Ожидание {:?} перед следующим действием...]", action.delay);
            }

            let result = self.execute_action(action);
            self.log_action(playbook_name, &format!("{:?}", action.action), result.clone());

            match result {
                ActionResult::Success => {
                    println!("[OK] Действие выполнено: {:?}", action.action);
                }
                ActionResult::PendingApproval => {
                    println!("[ОЖИДАНИЕ] Действие требует подтверждения: {:?}", action.action);
                }
                ActionResult::Failed(err) => {
                    println!("[ОШИБКА] Действие не выполнено: {} - {:?}", err, action.action);
                }
                ActionResult::Skipped(reason) => {
                    println!("[ПРОПУЩЕНО] {}: {:?}", reason, action.action);
                }
            }

            self.actions_taken.fetch_add(1, Ordering::Relaxed);
        }

        // Отметить реагирование как завершённое
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

        // Симуляция выполнения действия
        match &action.action {
            AutomatedAction::FlattenPositions => {
                println!("  → Закрытие всех позиций...");
                ActionResult::Success
            }
            AutomatedAction::DisableOrderEntry => {
                println!("  → Отключение приёма ордеров...");
                ActionResult::Success
            }
            AutomatedAction::ReduceLimits { factor } => {
                println!("  → Уменьшение лимитов на коэффициент {}...", factor);
                ActionResult::Success
            }
            AutomatedAction::Failover { target } => {
                println!("  → Переключение на {}...", target);
                ActionResult::Success
            }
            AutomatedAction::RestartService { service } => {
                println!("  → Перезапуск сервиса {}...", service);
                ActionResult::Success
            }
            AutomatedAction::PageOnCall { message } => {
                println!("  → Вызов дежурного: {}", message);
                ActionResult::Success
            }
            AutomatedAction::Notify { channel, message } => {
                println!("  → Уведомление в #{}: {}", channel, message);
                ActionResult::Success
            }
            AutomatedAction::ScaleUp { resource, amount } => {
                println!("  → Масштабирование {} на {}", resource, amount);
                ActionResult::Success
            }
            AutomatedAction::EnableCircuitBreaker { name } => {
                println!("  → Включение автовыключателя: {}", name);
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

        // Хранить только последние 1000 записей
        while log.len() > 1000 {
            log.pop_front();
        }
    }

    /// Включить аварийный режим (остановить всю торговлю)
    pub fn enable_emergency_mode(&self) {
        self.is_emergency_mode.store(true, Ordering::SeqCst);
        println!("\n!!! АВАРИЙНЫЙ РЕЖИМ ВКЛЮЧЁН !!!\n");
    }

    /// Получить сводку реагирования
    pub fn get_summary(&self) -> String {
        let active = self.active_responses.read().unwrap();
        let log = self.action_log.read().unwrap();

        let active_count = active.iter().filter(|r| !r.is_complete).count();
        let completed_count = active.iter().filter(|r| r.is_complete).count();

        format!(
            "Сводка реагирования на инциденты:\n\
             - Активные реагирования: {}\n\
             - Завершённые реагирования: {}\n\
             - Всего выполнено действий: {}\n\
             - Аварийный режим: {}",
            active_count,
            completed_count,
            self.actions_taken.load(Ordering::Relaxed),
            self.is_emergency_mode.load(Ordering::Relaxed)
        )
    }
}

fn main() {
    println!("=== Автоматизация реагирования на инциденты ===\n");

    let responder = IncidentResponder::new().with_standard_playbooks();

    // Симуляция запуска разных плейбуков
    println!("Сценарий 1: Обнаружен сбой алгоритма");
    responder.trigger_playbook("Сбой алгоритма").unwrap();

    println!("\n\nСценарий 2: Потеряно подключение к бирже");
    responder.trigger_playbook("Отключение биржи").unwrap();

    println!("\n\nСценарий 3: Чрезмерные дневные убытки");
    responder.trigger_playbook("Чрезмерные убытки").unwrap();

    println!("\n\n{}", responder.get_summary());
}
```

## Метрики и обучение на инцидентах

```rust
use std::collections::HashMap;
use std::time::Duration;

/// Трекер метрик инцидентов для постоянного улучшения
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
        // Отслеживание MTTR по серьёзности
        self.mttr_by_severity
            .entry(record.severity.clone())
            .or_insert_with(Vec::new)
            .push(record.time_to_resolve);

        // Отслеживание повторений
        *self.recurrence_map
            .entry(record.category.clone())
            .or_insert(0) += 1;

        self.incidents.push(record);
    }

    /// Рассчитать среднее время обнаружения (MTTD)
    pub fn mttd(&self) -> Duration {
        if self.incidents.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.incidents.iter()
            .map(|i| i.time_to_detect)
            .sum();

        total / self.incidents.len() as u32
    }

    /// Рассчитать среднее время реагирования (первое действие)
    pub fn mttr_response(&self) -> Duration {
        if self.incidents.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.incidents.iter()
            .map(|i| i.time_to_respond)
            .sum();

        total / self.incidents.len() as u32
    }

    /// Рассчитать среднее время разрешения
    pub fn mttr_resolve(&self) -> Duration {
        if self.incidents.is_empty() {
            return Duration::from_secs(0);
        }

        let total: Duration = self.incidents.iter()
            .map(|i| i.time_to_resolve)
            .sum();

        total / self.incidents.len() as u32
    }

    /// Рассчитать частоту инцидентов по категориям
    pub fn incident_frequency(&self) -> Vec<(String, u32)> {
        let mut freq: Vec<_> = self.recurrence_map.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        freq.sort_by(|a, b| b.1.cmp(&a.1));
        freq
    }

    /// Рассчитать процент выполнения плана действий
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

    /// Рассчитать процент повторений
    pub fn recurrence_rate(&self) -> f64 {
        if self.incidents.is_empty() {
            return 0.0;
        }

        let recurring = self.incidents.iter()
            .filter(|i| i.was_recurring)
            .count();

        (recurring as f64 / self.incidents.len() as f64) * 100.0
    }

    /// Рассчитать общее влияние на клиентов
    pub fn total_customer_impact(&self) -> u64 {
        self.incidents.iter()
            .map(|i| i.affected_customers)
            .sum()
    }

    /// Рассчитать общее влияние на выручку
    pub fn total_revenue_impact(&self) -> f64 {
        self.incidents.iter()
            .map(|i| i.revenue_impact)
            .sum()
    }

    /// Сгенерировать полный отчёт
    pub fn generate_report(&self, period: &str) -> String {
        let mut report = String::new();

        report.push_str(&format!("╔══════════════════════════════════════════════════════════╗\n"));
        report.push_str(&format!("║        ОТЧЁТ ПО МЕТРИКАМ ИНЦИДЕНТОВ - {:14}  ║\n", period));
        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        report.push_str(&format!("║ Всего инцидентов: {:39} ║\n", self.incidents.len()));
        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        // Метрики времени
        report.push_str(&format!("║ МЕТРИКИ РЕАГИРОВАНИЯ                                     ║\n"));
        report.push_str(&format!("║   Среднее время обнаружения (MTTD): {:>20?} ║\n", self.mttd()));
        report.push_str(&format!("║   Среднее время реагирования:       {:>20?} ║\n", self.mttr_response()));
        report.push_str(&format!("║   Среднее время разрешения (MTTR):  {:>20?} ║\n", self.mttr_resolve()));

        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        // Метрики влияния
        report.push_str(&format!("║ МЕТРИКИ ВЛИЯНИЯ                                          ║\n"));
        report.push_str(&format!("║   Всего затронутых клиентов: {:>27} ║\n", self.total_customer_impact()));
        report.push_str(&format!("║   Общее влияние на выручку:  ${:>26.2} ║\n", self.total_revenue_impact()));

        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        // Метрики качества
        report.push_str(&format!("║ МЕТРИКИ КАЧЕСТВА                                         ║\n"));
        report.push_str(&format!("║   Выполнение плана действий: {:>24.1}% ║\n", self.action_item_completion_rate()));
        report.push_str(&format!("║   Процент повторений:        {:>24.1}% ║\n", self.recurrence_rate()));

        report.push_str(&format!("╠══════════════════════════════════════════════════════════╣\n"));

        // Топ категорий инцидентов
        report.push_str(&format!("║ ТОП КАТЕГОРИЙ ИНЦИДЕНТОВ                                 ║\n"));
        for (cat, count) in self.incident_frequency().iter().take(5) {
            report.push_str(&format!("║   {:40} {:>10} ║\n", cat, count));
        }

        report.push_str(&format!("╚══════════════════════════════════════════════════════════╝\n"));

        report
    }
}

/// База извлечённых уроков
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

        // Индексация по тегам
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
    println!("=== Метрики инцидентов и обучение ===\n");

    let mut tracker = IncidentMetricsTracker::new();

    // Добавление примеров инцидентов
    tracker.add_incident(IncidentRecord {
        id: "INC-001".to_string(),
        severity: "Критический".to_string(),
        category: "Утечка памяти".to_string(),
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
        severity: "Высокий".to_string(),
        category: "Таймаут сети".to_string(),
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
        severity: "Критический".to_string(),
        category: "Утечка памяти".to_string(),
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

    // Извлечённые уроки
    println!("\n--- База извлечённых уроков ---\n");

    let mut lessons_db = LessonsLearnedDatabase::new();

    lessons_db.add_lesson(Lesson {
        id: "LL-001".to_string(),
        incident_id: "INC-001".to_string(),
        title: "Профилирование памяти в CI".to_string(),
        description: "Добавить автоматическое обнаружение утечек памяти в CI".to_string(),
        tags: vec!["память".to_string(), "ci".to_string(), "тестирование".to_string()],
        recommendations: vec![
            "Интегрировать Valgrind или AddressSanitizer".to_string(),
            "Установить пороги роста памяти".to_string(),
        ],
        implemented: true,
    });

    lessons_db.add_lesson(Lesson {
        id: "LL-002".to_string(),
        incident_id: "INC-002".to_string(),
        title: "Обработка таймаутов сети".to_string(),
        description: "Реализовать правильную логику повторов с экспоненциальной задержкой".to_string(),
        tags: vec!["сеть".to_string(), "отказоустойчивость".to_string()],
        recommendations: vec![
            "Добавить паттерн автовыключателя".to_string(),
            "Реализовать пул соединений".to_string(),
        ],
        implemented: false,
    });

    println!("Всего уроков: {}", lessons_db.lessons.len());
    println!("Процент реализации: {:.1}%", lessons_db.implementation_rate());

    println!("\nУроки с тегом 'память':");
    for lesson in lessons_db.search_by_tag("память") {
        println!("  - {}: {}", lesson.id, lesson.title);
    }

    println!("\nНереализованные уроки:");
    for lesson in lessons_db.get_unimplemented() {
        println!("  - {}: {}", lesson.id, lesson.title);
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Пост-мортем** | Безобвинительный анализ инцидентов для предотвращения повторения |
| **Восстановление хронологии** | Построение детальной последовательности событий инцидента |
| **Анализ первопричины** | Поиск истинной причины, а не симптомов |
| **5 почему** | Итеративная техника опроса для поиска первопричины |
| **Диаграмма Исикавы** | Визуальный инструмент для организации потенциальных причин |
| **Плейбуки инцидентов** | Заранее определённые автоматизированные процедуры реагирования |
| **MTTD/MTTR** | Ключевые метрики для измерения реагирования на инциденты |
| **Извлечённые уроки** | Документирование знаний для будущего предотвращения |

## Практические задания

1. **Шаблон пост-мортема**: Создай комплексную систему пост-мортема, которая:
   - Автоматически генерирует шаблоны из данных инцидента
   - Отслеживает события хронологии по мере их происхождения
   - Автоматически рассчитывает метрики влияния
   - Экспортирует в различные форматы (Markdown, PDF, JIRA)

2. **Анализатор первопричин**: Построй инструмент, который:
   - Проводит пользователей через анализ 5 почему
   - Предлагает потенциальные причины на основе категории инцидента
   - Связывает связанные инциденты и паттерны
   - Генерирует визуализации (диаграммы Исикавы)

3. **Движок плейбуков реагирования**: Реализуй автоматизированную систему, которая:
   - Сопоставляет инциденты с плейбуками на основе метрик
   - Выполняет действия с соответствующими задержками
   - Обрабатывает рабочие процессы утверждения
   - Логирует все действия для аудита

4. **Дашборд метрик**: Создай дашборд, который:
   - Отслеживает тренды MTTD, MTTR во времени
   - Показывает частоту инцидентов по категориям
   - Мониторит выполнение плана действий
   - Алертит при деградации метрик

## Домашнее задание

1. **Полная система управления инцидентами**: Построй систему end-to-end:
   - Обнаружение инцидентов и алертинг в реальном времени
   - Автоматическое выполнение плейбуков
   - Генерация пост-мортемов с шаблонами
   - База извлечённых уроков с поиском
   - Интеграция со Slack/PagerDuty
   - Экспорт метрик в Prometheus

2. **ML-анализ первопричин**: Создай интеллектуальный анализатор:
   - Распознавание паттернов по историческим инцидентам
   - Автоматические предложения первопричин
   - Корреляция аномалий между сервисами
   - Предсказание вероятных режимов отказа

3. **Интеграция с Chaos Engineering**: Объедини с тестированием:
   - Генерация синтетических инцидентов для обучения
   - Тестирование эффективности плейбуков
   - Измерение времени реагирования команды
   - Выявление пробелов в мониторинге

4. **Торговый пост-мортем**: Построй специализированные инструменты для:
   - Реконструкции сделок во время инцидентов
   - Расчёта влияния на P&L
   - Соответствия регуляторной отчётности
   - Анализа корреляции с рыночными условиями

## Навигация

[← Предыдущий день](../361-ab-strategy-testing/ru.md) | [Следующий день →](../363-compliance/ru.md)
