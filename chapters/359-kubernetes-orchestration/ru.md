# День 359: Kubernetes: оркестрация

## Аналогия из трейдинга

Представь, что ты управляешь глобальной торговой операцией на нескольких биржах в разных часовых поясах. У тебя есть торговые боты в Нью-Йорке, Лондоне, Токио и Сингапуре. Каждый бот должен:

- **Оставаться живым**: Если бот падает, кто-то должен немедленно его перезапустить
- **Масштабироваться**: В периоды высокой волатильности нужно больше инстансов для обработки ордеров
- **Балансировать нагрузку**: Ордера должны равномерно распределяться между доступными ботами
- **Обновляться плавно**: Разворачивать новые стратегии без потери активных позиций

**Именно это делает Kubernetes для твоей торговой инфраструктуры:**

| Торговые операции | Концепция Kubernetes |
|------------------|---------------------|
| **Менеджер торгового зала** | Control Plane Kubernetes |
| **Отдельные трейдеры** | Pod'ы (запущенные контейнеры) |
| **Команда трейдеров** | Deployment (группа pod'ов) |
| **Балансировщик нагрузки** | Service (маршрутизация трафика) |
| **Запасные трейдеры** | ReplicaSet (поддержание количества pod'ов) |
| **Торговые правила** | ConfigMap и Secret |
| **Лимиты ресурсов** | Resource Quotas |

Когда трейдер (pod) заболевает, менеджер (Kubernetes) немедленно приводит замену. Когда рынки становятся волатильными, добавляется больше трейдеров. Kubernetes — это идеальный менеджер торгового зала.

## Основы Kubernetes для торговых систем

### Определение Pod для торгового бота

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Спецификация Kubernetes Pod для торгового бота
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSpec {
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: PodSpecDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub namespace: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSpecDetails {
    pub containers: Vec<Container>,
    pub restart_policy: String,
    pub service_account_name: Option<String>,
    pub node_selector: Option<HashMap<String, String>>,
    pub tolerations: Option<Vec<Toleration>>,
    pub affinity: Option<Affinity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub ports: Vec<ContainerPort>,
    pub env: Vec<EnvVar>,
    pub resources: ResourceRequirements,
    pub liveness_probe: Option<Probe>,
    pub readiness_probe: Option<Probe>,
    pub volume_mounts: Option<Vec<VolumeMount>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPort {
    pub container_port: u16,
    pub name: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: Option<String>,
    pub value_from: Option<EnvVarSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarSource {
    pub secret_key_ref: Option<SecretKeyRef>,
    pub config_map_key_ref: Option<ConfigMapKeyRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretKeyRef {
    pub name: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapKeyRef {
    pub name: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub limits: ResourceList,
    pub requests: ResourceList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceList {
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub http_get: Option<HttpGetAction>,
    pub tcp_socket: Option<TcpSocketAction>,
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
    pub timeout_seconds: u32,
    pub failure_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpGetAction {
    pub path: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpSocketAction {
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    pub mount_path: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toleration {
    pub key: String,
    pub operator: String,
    pub value: Option<String>,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affinity {
    pub node_affinity: Option<NodeAffinity>,
    pub pod_anti_affinity: Option<PodAntiAffinity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAffinity {
    pub required_during_scheduling_ignored_during_execution: Option<NodeSelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelector {
    pub node_selector_terms: Vec<NodeSelectorTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectorTerm {
    pub match_expressions: Vec<NodeSelectorRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectorRequirement {
    pub key: String,
    pub operator: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAntiAffinity {
    pub preferred_during_scheduling_ignored_during_execution: Vec<WeightedPodAffinityTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedPodAffinityTerm {
    pub weight: i32,
    pub pod_affinity_term: PodAffinityTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAffinityTerm {
    pub topology_key: String,
    pub label_selector: LabelSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelector {
    pub match_labels: HashMap<String, String>,
}

/// Билдер для создания спецификаций pod'ов торговых ботов
pub struct TradingBotPodBuilder {
    name: String,
    namespace: String,
    image: String,
    exchange: String,
    strategy: String,
    replicas_hint: u32,
}

impl TradingBotPodBuilder {
    pub fn new(name: &str, image: &str) -> Self {
        TradingBotPodBuilder {
            name: name.to_string(),
            namespace: "trading".to_string(),
            image: image.to_string(),
            exchange: "binance".to_string(),
            strategy: "momentum".to_string(),
            replicas_hint: 1,
        }
    }

    pub fn namespace(mut self, ns: &str) -> Self {
        self.namespace = ns.to_string();
        self
    }

    pub fn exchange(mut self, exchange: &str) -> Self {
        self.exchange = exchange.to_string();
        self
    }

    pub fn strategy(mut self, strategy: &str) -> Self {
        self.strategy = strategy.to_string();
        self
    }

    pub fn build(self) -> PodSpec {
        let mut labels = HashMap::new();
        labels.insert("app".to_string(), "trading-bot".to_string());
        labels.insert("exchange".to_string(), self.exchange.clone());
        labels.insert("strategy".to_string(), self.strategy.clone());
        labels.insert("version".to_string(), "v1".to_string());

        let mut annotations = HashMap::new();
        annotations.insert(
            "prometheus.io/scrape".to_string(),
            "true".to_string(),
        );
        annotations.insert(
            "prometheus.io/port".to_string(),
            "9090".to_string(),
        );

        PodSpec {
            api_version: "v1".to_string(),
            kind: "Pod".to_string(),
            metadata: Metadata {
                name: self.name,
                namespace: self.namespace,
                labels,
                annotations,
            },
            spec: PodSpecDetails {
                containers: vec![Container {
                    name: "trading-bot".to_string(),
                    image: self.image,
                    ports: vec![
                        ContainerPort {
                            container_port: 8080,
                            name: "http".to_string(),
                            protocol: "TCP".to_string(),
                        },
                        ContainerPort {
                            container_port: 9090,
                            name: "metrics".to_string(),
                            protocol: "TCP".to_string(),
                        },
                    ],
                    env: vec![
                        EnvVar {
                            name: "EXCHANGE".to_string(),
                            value: Some(self.exchange),
                            value_from: None,
                        },
                        EnvVar {
                            name: "STRATEGY".to_string(),
                            value: Some(self.strategy),
                            value_from: None,
                        },
                        EnvVar {
                            name: "API_KEY".to_string(),
                            value: None,
                            value_from: Some(EnvVarSource {
                                secret_key_ref: Some(SecretKeyRef {
                                    name: "exchange-credentials".to_string(),
                                    key: "api-key".to_string(),
                                }),
                                config_map_key_ref: None,
                            }),
                        },
                        EnvVar {
                            name: "API_SECRET".to_string(),
                            value: None,
                            value_from: Some(EnvVarSource {
                                secret_key_ref: Some(SecretKeyRef {
                                    name: "exchange-credentials".to_string(),
                                    key: "api-secret".to_string(),
                                }),
                                config_map_key_ref: None,
                            }),
                        },
                    ],
                    resources: ResourceRequirements {
                        limits: ResourceList {
                            cpu: "1000m".to_string(),
                            memory: "512Mi".to_string(),
                        },
                        requests: ResourceList {
                            cpu: "500m".to_string(),
                            memory: "256Mi".to_string(),
                        },
                    },
                    liveness_probe: Some(Probe {
                        http_get: Some(HttpGetAction {
                            path: "/health".to_string(),
                            port: 8080,
                        }),
                        tcp_socket: None,
                        initial_delay_seconds: 10,
                        period_seconds: 30,
                        timeout_seconds: 5,
                        failure_threshold: 3,
                    }),
                    readiness_probe: Some(Probe {
                        http_get: Some(HttpGetAction {
                            path: "/ready".to_string(),
                            port: 8080,
                        }),
                        tcp_socket: None,
                        initial_delay_seconds: 5,
                        period_seconds: 10,
                        timeout_seconds: 3,
                        failure_threshold: 3,
                    }),
                    volume_mounts: None,
                }],
                restart_policy: "Always".to_string(),
                service_account_name: Some("trading-bot-sa".to_string()),
                node_selector: None,
                tolerations: None,
                affinity: None,
            },
        }
    }
}

fn main() {
    println!("=== Спецификация Kubernetes Pod для торгового бота ===\n");

    let pod = TradingBotPodBuilder::new(
        "btc-momentum-bot",
        "trading/bot:v1.2.3"
    )
    .namespace("production")
    .exchange("binance")
    .strategy("momentum")
    .build();

    // Вывод в формате YAML (в реальном коде используется serde_yaml)
    println!("Имя Pod: {}", pod.metadata.name);
    println!("Namespace: {}", pod.metadata.namespace);
    println!("Image: {}", pod.spec.containers[0].image);
    println!("\nМетки:");
    for (key, value) in &pod.metadata.labels {
        println!("  {}: {}", key, value);
    }
    println!("\nЛимиты ресурсов:");
    println!("  CPU: {}", pod.spec.containers[0].resources.limits.cpu);
    println!("  Memory: {}", pod.spec.containers[0].resources.limits.memory);
    println!("\nНастроенные проверки:");
    println!("  Liveness: /health на порту 8080");
    println!("  Readiness: /ready на порту 8080");
}
```

## Deployment для масштабируемого трейдинга

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kubernetes Deployment для флота торговых ботов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub api_version: String,
    pub kind: String,
    pub metadata: DeploymentMetadata,
    pub spec: DeploymentSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetadata {
    pub name: String,
    pub namespace: String,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSpec {
    pub replicas: u32,
    pub selector: Selector,
    pub template: PodTemplate,
    pub strategy: DeploymentStrategy,
    pub min_ready_seconds: u32,
    pub revision_history_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selector {
    pub match_labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodTemplate {
    pub metadata: PodTemplateMetadata,
    pub spec: PodTemplateSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodTemplateMetadata {
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodTemplateSpec {
    pub containers: Vec<ContainerSpec>,
    pub volumes: Vec<Volume>,
    pub termination_grace_period_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
    pub name: String,
    pub image: String,
    pub ports: Vec<Port>,
    pub env: Vec<EnvVar>,
    pub resources: Resources,
    pub lifecycle: Option<Lifecycle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub container_port: u16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    pub limits: ResourceQuantity,
    pub requests: ResourceQuantity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuantity {
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    pub pre_stop: Option<LifecycleHandler>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHandler {
    pub exec: Option<ExecAction>,
    pub http_get: Option<HttpGet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecAction {
    pub command: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpGet {
    pub path: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub name: String,
    pub config_map: Option<ConfigMapVolumeSource>,
    pub secret: Option<SecretVolumeSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapVolumeSource {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretVolumeSource {
    pub secret_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStrategy {
    pub strategy_type: String,
    pub rolling_update: Option<RollingUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingUpdate {
    pub max_unavailable: String,
    pub max_surge: String,
}

/// Менеджер торговых деплойментов
pub struct TradingDeploymentManager {
    deployments: HashMap<String, Deployment>,
}

impl TradingDeploymentManager {
    pub fn new() -> Self {
        TradingDeploymentManager {
            deployments: HashMap::new(),
        }
    }

    /// Создать новый деплоймент торгового бота
    pub fn create_deployment(
        &mut self,
        name: &str,
        image: &str,
        replicas: u32,
        exchange: &str,
        symbols: Vec<String>,
    ) -> Deployment {
        let mut labels = HashMap::new();
        labels.insert("app".to_string(), "trading-bot".to_string());
        labels.insert("exchange".to_string(), exchange.to_string());
        labels.insert("managed-by".to_string(), "rust-operator".to_string());

        let mut annotations = HashMap::new();
        annotations.insert(
            "trading.io/symbols".to_string(),
            symbols.join(","),
        );
        annotations.insert(
            "trading.io/max-position".to_string(),
            "10000".to_string(),
        );

        let deployment = Deployment {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            metadata: DeploymentMetadata {
                name: name.to_string(),
                namespace: "trading".to_string(),
                labels: labels.clone(),
            },
            spec: DeploymentSpec {
                replicas,
                selector: Selector {
                    match_labels: labels.clone(),
                },
                template: PodTemplate {
                    metadata: PodTemplateMetadata {
                        labels,
                        annotations,
                    },
                    spec: PodTemplateSpec {
                        containers: vec![ContainerSpec {
                            name: "trading-bot".to_string(),
                            image: image.to_string(),
                            ports: vec![
                                Port {
                                    container_port: 8080,
                                    name: "http".to_string(),
                                },
                                Port {
                                    container_port: 9090,
                                    name: "metrics".to_string(),
                                },
                            ],
                            env: vec![
                                EnvVar {
                                    name: "EXCHANGE".to_string(),
                                    value: Some(exchange.to_string()),
                                },
                                EnvVar {
                                    name: "SYMBOLS".to_string(),
                                    value: Some(symbols.join(",")),
                                },
                                EnvVar {
                                    name: "LOG_LEVEL".to_string(),
                                    value: Some("info".to_string()),
                                },
                            ],
                            resources: Resources {
                                limits: ResourceQuantity {
                                    cpu: "2000m".to_string(),
                                    memory: "1Gi".to_string(),
                                },
                                requests: ResourceQuantity {
                                    cpu: "500m".to_string(),
                                    memory: "256Mi".to_string(),
                                },
                            },
                            lifecycle: Some(Lifecycle {
                                pre_stop: Some(LifecycleHandler {
                                    http_get: Some(HttpGet {
                                        path: "/shutdown".to_string(),
                                        port: 8080,
                                    }),
                                    exec: None,
                                }),
                            }),
                        }],
                        volumes: vec![
                            Volume {
                                name: "config".to_string(),
                                config_map: Some(ConfigMapVolumeSource {
                                    name: format!("{}-config", name),
                                }),
                                secret: None,
                            },
                            Volume {
                                name: "secrets".to_string(),
                                config_map: None,
                                secret: Some(SecretVolumeSource {
                                    secret_name: format!("{}-secrets", exchange),
                                }),
                            },
                        ],
                        termination_grace_period_seconds: 60,
                    },
                },
                strategy: DeploymentStrategy {
                    strategy_type: "RollingUpdate".to_string(),
                    rolling_update: Some(RollingUpdate {
                        max_unavailable: "25%".to_string(),
                        max_surge: "25%".to_string(),
                    }),
                },
                min_ready_seconds: 10,
                revision_history_limit: 5,
            },
        };

        self.deployments.insert(name.to_string(), deployment.clone());
        deployment
    }

    /// Масштабировать деплоймент
    pub fn scale(&mut self, name: &str, replicas: u32) -> Result<(), String> {
        if let Some(deployment) = self.deployments.get_mut(name) {
            let old_replicas = deployment.spec.replicas;
            deployment.spec.replicas = replicas;
            println!(
                "Масштабирован деплоймент '{}': {} -> {} реплик",
                name, old_replicas, replicas
            );
            Ok(())
        } else {
            Err(format!("Деплоймент '{}' не найден", name))
        }
    }

    /// Обновить образ деплоймента
    pub fn update_image(&mut self, name: &str, new_image: &str) -> Result<(), String> {
        if let Some(deployment) = self.deployments.get_mut(name) {
            let old_image = deployment.spec.template.spec.containers[0].image.clone();
            deployment.spec.template.spec.containers[0].image = new_image.to_string();
            println!(
                "Обновлён образ деплоймента '{}': {} -> {}",
                name, old_image, new_image
            );
            Ok(())
        } else {
            Err(format!("Деплоймент '{}' не найден", name))
        }
    }

    /// Получить сводку статуса деплоймента
    pub fn status(&self, name: &str) -> Option<String> {
        self.deployments.get(name).map(|d| {
            format!(
                "Deployment: {}\n  Реплики: {}\n  Образ: {}\n  Стратегия: {}",
                d.metadata.name,
                d.spec.replicas,
                d.spec.template.spec.containers[0].image,
                d.spec.strategy.strategy_type
            )
        })
    }
}

fn main() {
    println!("=== Kubernetes Deployment для торговых ботов ===\n");

    let mut manager = TradingDeploymentManager::new();

    // Создание деплоймента для BTC трейдинга
    let btc_deployment = manager.create_deployment(
        "btc-trader",
        "trading/bot:v1.0.0",
        3,
        "binance",
        vec!["BTCUSDT".to_string(), "BTCBUSD".to_string()],
    );

    println!("Создан деплоймент: {}", btc_deployment.metadata.name);
    println!("  Реплики: {}", btc_deployment.spec.replicas);
    println!("  Стратегия: {}", btc_deployment.spec.strategy.strategy_type);

    // Создание деплоймента для ETH трейдинга
    let eth_deployment = manager.create_deployment(
        "eth-trader",
        "trading/bot:v1.0.0",
        2,
        "binance",
        vec!["ETHUSDT".to_string()],
    );

    println!("\nСоздан деплоймент: {}", eth_deployment.metadata.name);

    // Масштабирование при высокой волатильности
    println!("\n--- Обнаружена высокая волатильность, масштабируем ---");
    manager.scale("btc-trader", 5).unwrap();

    // Обновление до новой версии
    println!("\n--- Разворачиваем новую версию ---");
    manager.update_image("btc-trader", "trading/bot:v1.1.0").unwrap();

    // Проверка статуса
    println!("\n--- Текущий статус ---");
    if let Some(status) = manager.status("btc-trader") {
        println!("{}", status);
    }
}
```

## Horizontal Pod Autoscaler для торговой нагрузки

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Метрики для принятия решений об автомасштабировании
#[derive(Debug, Clone)]
pub struct TradingMetrics {
    pub orders_per_second: f64,
    pub latency_p99_ms: f64,
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub pending_orders: u32,
    pub active_positions: u32,
}

/// Конфигурация Horizontal Pod Autoscaler
#[derive(Debug, Clone)]
pub struct HPAConfig {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: f64,
    pub target_memory_utilization: f64,
    pub scale_up_stabilization_seconds: u64,
    pub scale_down_stabilization_seconds: u64,
    pub custom_metrics: Vec<CustomMetricTarget>,
}

#[derive(Debug, Clone)]
pub struct CustomMetricTarget {
    pub name: String,
    pub target_value: f64,
    pub target_type: MetricTargetType,
}

#[derive(Debug, Clone)]
pub enum MetricTargetType {
    AverageValue,
    Value,
    Utilization,
}

/// Автомасштабировщик с учётом специфики трейдинга
pub struct TradingAutoscaler {
    config: HPAConfig,
    current_replicas: u32,
    metrics_history: Vec<(Instant, TradingMetrics)>,
    last_scale_up: Option<Instant>,
    last_scale_down: Option<Instant>,
}

impl TradingAutoscaler {
    pub fn new(config: HPAConfig) -> Self {
        TradingAutoscaler {
            current_replicas: config.min_replicas,
            config,
            metrics_history: Vec::new(),
            last_scale_up: None,
            last_scale_down: None,
        }
    }

    /// Записать метрики для принятия решений о масштабировании
    pub fn record_metrics(&mut self, metrics: TradingMetrics) {
        self.metrics_history.push((Instant::now(), metrics));

        // Храним только последние 5 минут метрик
        let cutoff = Instant::now() - Duration::from_secs(300);
        self.metrics_history.retain(|(time, _)| *time > cutoff);
    }

    /// Вычислить желаемое количество реплик на основе метрик
    pub fn calculate_desired_replicas(&self) -> u32 {
        if self.metrics_history.is_empty() {
            return self.current_replicas;
        }

        // Вычисляем средние метрики за последнее время
        let recent_metrics: Vec<&TradingMetrics> = self.metrics_history
            .iter()
            .filter(|(time, _)| time.elapsed() < Duration::from_secs(60))
            .map(|(_, m)| m)
            .collect();

        if recent_metrics.is_empty() {
            return self.current_replicas;
        }

        let avg_cpu: f64 = recent_metrics.iter()
            .map(|m| m.cpu_utilization)
            .sum::<f64>() / recent_metrics.len() as f64;

        let avg_latency: f64 = recent_metrics.iter()
            .map(|m| m.latency_p99_ms)
            .sum::<f64>() / recent_metrics.len() as f64;

        let avg_pending: f64 = recent_metrics.iter()
            .map(|m| m.pending_orders as f64)
            .sum::<f64>() / recent_metrics.len() as f64;

        // Масштабирование на основе CPU
        let cpu_ratio = avg_cpu / self.config.target_cpu_utilization;
        let cpu_desired = (self.current_replicas as f64 * cpu_ratio).ceil() as u32;

        // Масштабирование на основе задержки (если высокая — масштабируем вверх)
        let latency_multiplier = if avg_latency > 100.0 {
            1.5
        } else if avg_latency > 50.0 {
            1.2
        } else {
            1.0
        };

        // Масштабирование на основе очереди ордеров
        let pending_multiplier = if avg_pending > 100.0 {
            1.5
        } else if avg_pending > 50.0 {
            1.2
        } else {
            1.0
        };

        let desired = ((cpu_desired as f64) * latency_multiplier * pending_multiplier).ceil() as u32;

        // Ограничиваем min/max
        desired.clamp(self.config.min_replicas, self.config.max_replicas)
    }

    /// Оценить и потенциально масштабировать
    pub fn evaluate(&mut self) -> ScalingDecision {
        let desired = self.calculate_desired_replicas();

        if desired == self.current_replicas {
            return ScalingDecision::NoChange;
        }

        let now = Instant::now();

        if desired > self.current_replicas {
            // Масштабирование вверх
            if let Some(last) = self.last_scale_up {
                if now.duration_since(last).as_secs() < self.config.scale_up_stabilization_seconds {
                    return ScalingDecision::Stabilizing {
                        direction: "вверх",
                        wait_seconds: self.config.scale_up_stabilization_seconds
                            - now.duration_since(last).as_secs(),
                    };
                }
            }

            self.last_scale_up = Some(now);
            let old = self.current_replicas;
            self.current_replicas = desired;

            ScalingDecision::ScaleUp {
                from: old,
                to: desired,
                reason: self.get_scale_reason(),
            }
        } else {
            // Масштабирование вниз
            if let Some(last) = self.last_scale_down {
                if now.duration_since(last).as_secs() < self.config.scale_down_stabilization_seconds {
                    return ScalingDecision::Stabilizing {
                        direction: "вниз",
                        wait_seconds: self.config.scale_down_stabilization_seconds
                            - now.duration_since(last).as_secs(),
                    };
                }
            }

            self.last_scale_down = Some(now);
            let old = self.current_replicas;
            self.current_replicas = desired;

            ScalingDecision::ScaleDown {
                from: old,
                to: desired,
                reason: "Использование ресурсов ниже порога".to_string(),
            }
        }
    }

    fn get_scale_reason(&self) -> String {
        if let Some((_, metrics)) = self.metrics_history.last() {
            if metrics.cpu_utilization > self.config.target_cpu_utilization {
                return format!(
                    "Использование CPU {:.1}% превышает целевое {:.1}%",
                    metrics.cpu_utilization,
                    self.config.target_cpu_utilization
                );
            }
            if metrics.latency_p99_ms > 100.0 {
                return format!(
                    "P99 задержка {:.1}мс превышает порог",
                    metrics.latency_p99_ms
                );
            }
            if metrics.pending_orders > 50 {
                return format!(
                    "Очередь ордеров {} превышает порог",
                    metrics.pending_orders
                );
            }
        }
        "Масштабирование на основе комбинированных метрик".to_string()
    }

    pub fn current_replicas(&self) -> u32 {
        self.current_replicas
    }
}

#[derive(Debug)]
pub enum ScalingDecision {
    NoChange,
    ScaleUp { from: u32, to: u32, reason: String },
    ScaleDown { from: u32, to: u32, reason: String },
    Stabilizing { direction: &'static str, wait_seconds: u64 },
}

fn main() {
    println!("=== Торговый Horizontal Pod Autoscaler ===\n");

    let config = HPAConfig {
        min_replicas: 2,
        max_replicas: 10,
        target_cpu_utilization: 70.0,
        target_memory_utilization: 80.0,
        scale_up_stabilization_seconds: 30,
        scale_down_stabilization_seconds: 300,
        custom_metrics: vec![
            CustomMetricTarget {
                name: "orders_per_second".to_string(),
                target_value: 100.0,
                target_type: MetricTargetType::AverageValue,
            },
            CustomMetricTarget {
                name: "pending_orders".to_string(),
                target_value: 50.0,
                target_type: MetricTargetType::Value,
            },
        ],
    };

    let mut autoscaler = TradingAutoscaler::new(config);

    println!("Начальные реплики: {}\n", autoscaler.current_replicas());

    // Симуляция нормальной нагрузки
    println!("--- Нормальные торговые условия ---");
    autoscaler.record_metrics(TradingMetrics {
        orders_per_second: 50.0,
        latency_p99_ms: 25.0,
        cpu_utilization: 45.0,
        memory_utilization: 40.0,
        pending_orders: 10,
        active_positions: 50,
    });

    match autoscaler.evaluate() {
        ScalingDecision::NoChange => println!("Масштабирование не требуется"),
        decision => println!("Решение: {:?}", decision),
    }

    // Симуляция высокой нагрузки (волатильность рынка)
    println!("\n--- Обнаружена высокая волатильность! ---");
    autoscaler.record_metrics(TradingMetrics {
        orders_per_second: 500.0,
        latency_p99_ms: 150.0,
        cpu_utilization: 95.0,
        memory_utilization: 70.0,
        pending_orders: 200,
        active_positions: 150,
    });

    match autoscaler.evaluate() {
        ScalingDecision::ScaleUp { from, to, reason } => {
            println!("МАСШТАБИРОВАНИЕ ВВЕРХ: {} -> {} реплик", from, to);
            println!("Причина: {}", reason);
        }
        decision => println!("Решение: {:?}", decision),
    }

    println!("\nТекущие реплики после масштабирования: {}", autoscaler.current_replicas());

    // Симуляция снижения нагрузки
    println!("\n--- Рынок успокаивается ---");
    for _ in 0..5 {
        autoscaler.record_metrics(TradingMetrics {
            orders_per_second: 30.0,
            latency_p99_ms: 15.0,
            cpu_utilization: 25.0,
            memory_utilization: 30.0,
            pending_orders: 5,
            active_positions: 30,
        });
    }

    // В реальной реализации нужно ждать период стабилизации
    println!("Период стабилизации для масштабирования вниз: 300 секунд");
}
```

## Service Discovery и балансировка нагрузки

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::net::SocketAddr;

/// Типы Kubernetes Service
#[derive(Debug, Clone)]
pub enum ServiceType {
    ClusterIP,
    NodePort { node_port: u16 },
    LoadBalancer { external_ip: Option<String> },
    Headless,
}

/// Endpoint сервиса, представляющий pod
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub pod_name: String,
    pub ip: String,
    pub port: u16,
    pub ready: bool,
    pub zone: String,
}

/// Kubernetes Service для трейдинга
#[derive(Debug, Clone)]
pub struct TradingService {
    pub name: String,
    pub namespace: String,
    pub service_type: ServiceType,
    pub cluster_ip: Option<String>,
    pub ports: Vec<ServicePort>,
    pub selector: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ServicePort {
    pub name: String,
    pub port: u16,
    pub target_port: u16,
    pub protocol: String,
}

/// Реестр сервисов для торговых систем
pub struct ServiceRegistry {
    services: HashMap<String, TradingService>,
    endpoints: HashMap<String, Vec<Endpoint>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        ServiceRegistry {
            services: HashMap::new(),
            endpoints: HashMap::new(),
        }
    }

    /// Зарегистрировать сервис
    pub fn register_service(&mut self, service: TradingService) {
        println!("Регистрация сервиса: {}/{}", service.namespace, service.name);
        self.services.insert(service.name.clone(), service);
    }

    /// Обновить endpoints для сервиса
    pub fn update_endpoints(&mut self, service_name: &str, endpoints: Vec<Endpoint>) {
        let ready_count = endpoints.iter().filter(|e| e.ready).count();
        println!(
            "Обновление endpoints для '{}': {} всего, {} готовы",
            service_name,
            endpoints.len(),
            ready_count
        );
        self.endpoints.insert(service_name.to_string(), endpoints);
    }

    /// Получить готовые endpoints для сервиса
    pub fn get_ready_endpoints(&self, service_name: &str) -> Vec<&Endpoint> {
        self.endpoints
            .get(service_name)
            .map(|eps| eps.iter().filter(|e| e.ready).collect())
            .unwrap_or_default()
    }

    /// Получить ClusterIP сервиса
    pub fn get_cluster_ip(&self, service_name: &str) -> Option<&str> {
        self.services
            .get(service_name)
            .and_then(|s| s.cluster_ip.as_deref())
    }
}

/// Балансировщик нагрузки для торговых запросов
pub struct TradingLoadBalancer {
    registry: Arc<RwLock<ServiceRegistry>>,
    round_robin_counters: HashMap<String, usize>,
}

impl TradingLoadBalancer {
    pub fn new(registry: Arc<RwLock<ServiceRegistry>>) -> Self {
        TradingLoadBalancer {
            registry,
            round_robin_counters: HashMap::new(),
        }
    }

    /// Получить следующий endpoint используя round-robin
    pub fn get_endpoint(&mut self, service_name: &str) -> Option<Endpoint> {
        let registry = self.registry.read().unwrap();
        let endpoints = registry.get_ready_endpoints(service_name);

        if endpoints.is_empty() {
            return None;
        }

        let counter = self.round_robin_counters
            .entry(service_name.to_string())
            .or_insert(0);

        let idx = *counter % endpoints.len();
        *counter = counter.wrapping_add(1);

        Some(endpoints[idx].clone())
    }

    /// Получить endpoint с предпочтением зоны (предпочитать локальную)
    pub fn get_endpoint_with_affinity(
        &mut self,
        service_name: &str,
        preferred_zone: &str,
    ) -> Option<Endpoint> {
        let registry = self.registry.read().unwrap();
        let endpoints = registry.get_ready_endpoints(service_name);

        // Пытаемся найти endpoint в предпочтительной зоне
        let local_endpoints: Vec<_> = endpoints
            .iter()
            .filter(|e| e.zone == preferred_zone)
            .collect();

        if !local_endpoints.is_empty() {
            let counter = self.round_robin_counters
                .entry(format!("{}_{}", service_name, preferred_zone))
                .or_insert(0);
            let idx = *counter % local_endpoints.len();
            *counter = counter.wrapping_add(1);
            return Some((*local_endpoints[idx]).clone());
        }

        // Возвращаемся к любому доступному endpoint
        if !endpoints.is_empty() {
            let counter = self.round_robin_counters
                .entry(service_name.to_string())
                .or_insert(0);
            let idx = *counter % endpoints.len();
            *counter = counter.wrapping_add(1);
            return Some(endpoints[idx].clone());
        }

        None
    }
}

/// Service mesh для трейдинга
pub struct TradingServiceMesh {
    registry: Arc<RwLock<ServiceRegistry>>,
    load_balancer: TradingLoadBalancer,
}

impl TradingServiceMesh {
    pub fn new() -> Self {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let load_balancer = TradingLoadBalancer::new(Arc::clone(&registry));

        TradingServiceMesh {
            registry,
            load_balancer,
        }
    }

    /// Инициализировать торговые сервисы
    pub fn init_services(&self) {
        let mut registry = self.registry.write().unwrap();

        // Сервис исполнения ордеров
        registry.register_service(TradingService {
            name: "order-executor".to_string(),
            namespace: "trading".to_string(),
            service_type: ServiceType::ClusterIP,
            cluster_ip: Some("10.0.0.100".to_string()),
            ports: vec![
                ServicePort {
                    name: "grpc".to_string(),
                    port: 50051,
                    target_port: 50051,
                    protocol: "TCP".to_string(),
                },
            ],
            selector: {
                let mut s = HashMap::new();
                s.insert("app".to_string(), "order-executor".to_string());
                s
            },
        });

        // Сервис рыночных данных
        registry.register_service(TradingService {
            name: "market-data".to_string(),
            namespace: "trading".to_string(),
            service_type: ServiceType::ClusterIP,
            cluster_ip: Some("10.0.0.101".to_string()),
            ports: vec![
                ServicePort {
                    name: "ws".to_string(),
                    port: 8080,
                    target_port: 8080,
                    protocol: "TCP".to_string(),
                },
            ],
            selector: {
                let mut s = HashMap::new();
                s.insert("app".to_string(), "market-data".to_string());
                s
            },
        });

        // Сервис управления рисками
        registry.register_service(TradingService {
            name: "risk-manager".to_string(),
            namespace: "trading".to_string(),
            service_type: ServiceType::ClusterIP,
            cluster_ip: Some("10.0.0.102".to_string()),
            ports: vec![
                ServicePort {
                    name: "http".to_string(),
                    port: 8080,
                    target_port: 8080,
                    protocol: "TCP".to_string(),
                },
            ],
            selector: {
                let mut s = HashMap::new();
                s.insert("app".to_string(), "risk-manager".to_string());
                s
            },
        });

        // Симуляция обновления endpoints
        registry.update_endpoints("order-executor", vec![
            Endpoint {
                pod_name: "order-executor-abc123".to_string(),
                ip: "10.1.0.10".to_string(),
                port: 50051,
                ready: true,
                zone: "us-east-1a".to_string(),
            },
            Endpoint {
                pod_name: "order-executor-def456".to_string(),
                ip: "10.1.0.11".to_string(),
                port: 50051,
                ready: true,
                zone: "us-east-1b".to_string(),
            },
            Endpoint {
                pod_name: "order-executor-ghi789".to_string(),
                ip: "10.1.0.12".to_string(),
                port: 50051,
                ready: false, // Pod не готов
                zone: "us-east-1c".to_string(),
            },
        ]);

        registry.update_endpoints("market-data", vec![
            Endpoint {
                pod_name: "market-data-xyz111".to_string(),
                ip: "10.1.1.10".to_string(),
                port: 8080,
                ready: true,
                zone: "us-east-1a".to_string(),
            },
            Endpoint {
                pod_name: "market-data-xyz222".to_string(),
                ip: "10.1.1.11".to_string(),
                port: 8080,
                ready: true,
                zone: "us-east-1b".to_string(),
            },
        ]);
    }

    /// Маршрутизировать запрос к соответствующему сервису
    pub fn route_request(&mut self, service: &str, zone: Option<&str>) -> Option<String> {
        let endpoint = if let Some(z) = zone {
            self.load_balancer.get_endpoint_with_affinity(service, z)
        } else {
            self.load_balancer.get_endpoint(service)
        };

        endpoint.map(|e| format!("{}:{}", e.ip, e.port))
    }
}

fn main() {
    println!("=== Kubernetes Service Discovery для трейдинга ===\n");

    let mut mesh = TradingServiceMesh::new();
    mesh.init_services();

    // Маршрутизация запросов к order executor
    println!("Маршрутизация запросов к order-executor:");
    for i in 0..5 {
        if let Some(addr) = mesh.route_request("order-executor", None) {
            println!("  Запрос {}: направлен на {}", i + 1, addr);
        }
    }

    // Маршрутизация с предпочтением зоны
    println!("\nМаршрутизация с предпочтением зоны (us-east-1a):");
    for i in 0..3 {
        if let Some(addr) = mesh.route_request("order-executor", Some("us-east-1a")) {
            println!("  Запрос {}: направлен на {}", i + 1, addr);
        }
    }

    // Проверка сервиса market-data
    println!("\nEndpoints сервиса market-data:");
    let registry = mesh.registry.read().unwrap();
    for ep in registry.get_ready_endpoints("market-data") {
        println!("  {} -> {}:{} (зона: {})",
            ep.pod_name, ep.ip, ep.port, ep.zone);
    }
}
```

## ConfigMaps и Secrets для торговой конфигурации

```rust
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Kubernetes ConfigMap для торговой конфигурации
#[derive(Debug, Clone)]
pub struct ConfigMap {
    pub name: String,
    pub namespace: String,
    pub data: HashMap<String, String>,
    pub binary_data: HashMap<String, Vec<u8>>,
}

/// Kubernetes Secret для чувствительных торговых данных
#[derive(Debug, Clone)]
pub struct Secret {
    pub name: String,
    pub namespace: String,
    pub secret_type: SecretType,
    pub data: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum SecretType {
    Opaque,
    DockerConfigJson,
    TLS,
    ServiceAccountToken,
}

/// Менеджер торговой конфигурации
pub struct TradingConfigManager {
    config_maps: HashMap<String, ConfigMap>,
    secrets: HashMap<String, Secret>,
}

impl TradingConfigManager {
    pub fn new() -> Self {
        TradingConfigManager {
            config_maps: HashMap::new(),
            secrets: HashMap::new(),
        }
    }

    /// Создать конфигурацию торговой стратегии
    pub fn create_strategy_config(
        &mut self,
        name: &str,
        strategy_type: &str,
        parameters: HashMap<String, String>,
    ) -> ConfigMap {
        let mut data = HashMap::new();

        // Конфигурация стратегии
        data.insert("strategy.type".to_string(), strategy_type.to_string());

        // Добавляем все параметры
        for (key, value) in parameters {
            data.insert(format!("strategy.{}", key), value);
        }

        // Добавляем общие настройки трейдинга
        data.insert("trading.max_position_size".to_string(), "10000".to_string());
        data.insert("trading.max_daily_loss".to_string(), "5000".to_string());
        data.insert("trading.enable_paper_trading".to_string(), "false".to_string());

        let config = ConfigMap {
            name: name.to_string(),
            namespace: "trading".to_string(),
            data,
            binary_data: HashMap::new(),
        };

        self.config_maps.insert(name.to_string(), config.clone());
        config
    }

    /// Создать секрет с учётными данными биржи
    pub fn create_exchange_secret(
        &mut self,
        name: &str,
        api_key: &str,
        api_secret: &str,
        passphrase: Option<&str>,
    ) -> Secret {
        let mut data = HashMap::new();

        data.insert("api-key".to_string(), api_key.as_bytes().to_vec());
        data.insert("api-secret".to_string(), api_secret.as_bytes().to_vec());

        if let Some(pass) = passphrase {
            data.insert("passphrase".to_string(), pass.as_bytes().to_vec());
        }

        let secret = Secret {
            name: name.to_string(),
            namespace: "trading".to_string(),
            secret_type: SecretType::Opaque,
            data,
        };

        self.secrets.insert(name.to_string(), secret.clone());
        secret
    }

    /// Получить значение конфигурации
    pub fn get_config(&self, config_name: &str, key: &str) -> Option<&String> {
        self.config_maps
            .get(config_name)
            .and_then(|cm| cm.data.get(key))
    }

    /// Получить значение секрета (в реальном k8s будет декодировано из base64)
    pub fn get_secret(&self, secret_name: &str, key: &str) -> Option<String> {
        self.secrets
            .get(secret_name)
            .and_then(|s| s.data.get(key))
            .map(|v| String::from_utf8_lossy(v).to_string())
    }

    /// Сгенерировать YAML представление
    pub fn to_yaml(&self, config_name: &str) -> Option<String> {
        self.config_maps.get(config_name).map(|cm| {
            let mut yaml = format!(
                "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: {}\n  namespace: {}\ndata:\n",
                cm.name, cm.namespace
            );
            for (key, value) in &cm.data {
                yaml.push_str(&format!("  {}: \"{}\"\n", key, value));
            }
            yaml
        })
    }

    /// Сгенерировать YAML представление Secret
    pub fn secret_to_yaml(&self, secret_name: &str) -> Option<String> {
        self.secrets.get(secret_name).map(|s| {
            let mut yaml = format!(
                "apiVersion: v1\nkind: Secret\nmetadata:\n  name: {}\n  namespace: {}\ntype: Opaque\ndata:\n",
                s.name, s.namespace
            );
            for (key, value) in &s.data {
                let encoded = BASE64.encode(value);
                yaml.push_str(&format!("  {}: {}\n", key, encoded));
            }
            yaml
        })
    }
}

/// Загрузчик конфигурации окружения
pub struct EnvConfigLoader {
    config_manager: TradingConfigManager,
}

impl EnvConfigLoader {
    pub fn new(manager: TradingConfigManager) -> Self {
        EnvConfigLoader {
            config_manager: manager,
        }
    }

    /// Загрузить конфигурацию в структуру, подобную переменным окружения
    pub fn load_env(&self, config_name: &str, secret_name: &str) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Загружаем значения конфигурации
        if let Some(cm) = self.config_manager.config_maps.get(config_name) {
            for (key, value) in &cm.data {
                // Преобразуем формат ключа: strategy.type -> STRATEGY_TYPE
                let env_key = key.replace('.', "_").to_uppercase();
                env.insert(env_key, value.clone());
            }
        }

        // Загружаем значения секретов
        if let Some(s) = self.config_manager.secrets.get(secret_name) {
            for (key, value) in &s.data {
                let env_key = key.replace('-', "_").to_uppercase();
                env.insert(env_key, String::from_utf8_lossy(value).to_string());
            }
        }

        env
    }
}

fn main() {
    println!("=== Kubernetes ConfigMaps и Secrets для трейдинга ===\n");

    let mut manager = TradingConfigManager::new();

    // Создание конфигурации стратегии
    let mut params = HashMap::new();
    params.insert("lookback_period".to_string(), "20".to_string());
    params.insert("entry_threshold".to_string(), "0.02".to_string());
    params.insert("exit_threshold".to_string(), "0.01".to_string());
    params.insert("position_sizing".to_string(), "kelly".to_string());

    let config = manager.create_strategy_config(
        "momentum-strategy-config",
        "momentum",
        params,
    );

    println!("Создан ConfigMap: {}", config.name);
    println!("\nЗначения конфигурации:");
    for (key, value) in &config.data {
        println!("  {}: {}", key, value);
    }

    // Создание учётных данных биржи
    let secret = manager.create_exchange_secret(
        "binance-credentials",
        "my-api-key-12345",
        "my-secret-key-67890",
        None,
    );

    println!("\n\nСоздан Secret: {} (тип: {:?})", secret.name, secret.secret_type);
    println!("Ключи секрета: {:?}", secret.data.keys().collect::<Vec<_>>());

    // Генерация YAML
    println!("\n\n--- ConfigMap YAML ---");
    if let Some(yaml) = manager.to_yaml("momentum-strategy-config") {
        println!("{}", yaml);
    }

    println!("\n--- Secret YAML (base64 закодирован) ---");
    if let Some(yaml) = manager.secret_to_yaml("binance-credentials") {
        println!("{}", yaml);
    }

    // Загрузка как переменные окружения
    println!("\n--- Загруженные переменные окружения ---");
    let loader = EnvConfigLoader::new(manager);
    let env = loader.load_env("momentum-strategy-config", "binance-credentials");

    for (key, value) in &env {
        if key.contains("SECRET") || key.contains("KEY") {
            println!("  {}: ****", key);
        } else {
            println!("  {}: {}", key, value);
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Pod** | Минимальная развёртываемая единица, запускает контейнеры торговых ботов |
| **Deployment** | Управляет репликами pod'ов с rolling update'ами |
| **Service** | Балансировка нагрузки и service discovery для торговых сервисов |
| **HPA** | Автоматическое масштабирование на основе CPU, памяти или кастомных метрик |
| **ConfigMap** | Хранение параметров трейдинга и конфигурации стратегий |
| **Secret** | Безопасное хранение API ключей и учётных данных |
| **Probes** | Проверки здоровья для обеспечения работоспособности торговых ботов |
| **Labels** | Организация и выбор торговых ресурсов |

## Практические задания

1. **Деплоймент торгового бота**: Создай полный Kubernetes деплоймент, который:
   - Запускает несколько реплик торгового бота
   - Использует rolling update'ы для развёртывания без простоя
   - Включает liveness и readiness проверки
   - Монтирует конфигурацию из ConfigMap
   - Загружает учётные данные из Secret

2. **Автомасштабируемая торговая система**: Реализуй HPA, который:
   - Масштабируется на основе пропускной способности ордеров
   - Реагирует на всплески задержки
   - Имеет разные политики масштабирования вверх и вниз
   - Учитывает торговые часы (больше pod'ов во время работы рынка)

3. **Service Mesh для трейдинга**: Создай систему service discovery, которая:
   - Регистрирует все торговые микросервисы
   - Реализует балансировку нагрузки с учётом зон
   - Обрабатывает обновления состояния endpoints
   - Поддерживает circuit breaking для падающих сервисов

4. **Управление конфигурацией**: Построй систему, которая:
   - Управляет параметрами стратегий через ConfigMap
   - Обрабатывает ротацию API ключей через Secret
   - Поддерживает горячую перезагрузку конфигурации
   - Валидирует конфигурацию перед применением

## Домашнее задание

1. **Мультибиржевой деплоймент**: Создай Kubernetes архитектуру, которая:
   - Разворачивает отдельные торговые боты для каждой биржи
   - Использует namespace'ы для изоляции
   - Реализует межнеймспейсную коммуникацию
   - Управляет разными API credentials для каждой биржи
   - Обрабатывает специфичные для бирж требования к масштабированию

2. **Система аварийного восстановления**: Спроектируй деплоймент, который:
   - Охватывает несколько зон доступности
   - Реализует правила pod anti-affinity
   - Имеет автоматический failover между зонами
   - Сохраняет состояние позиций при перезапуске pod'ов
   - Включает процедуры резервного копирования и восстановления

3. **GitOps для трейдинга**: Реализуй CI/CD пайплайн, который:
   - Использует GitOps для управления деплойментами
   - Поддерживает канареечные деплои для стратегий
   - Включает автоматический откат при плохой производительности
   - Ведёт аудит-лог всех развёртываний
   - Интегрируется с системами мониторинга трейдинга

4. **Кастомный Kubernetes Operator**: Построй оператор, который:
   - Управляет кастомными ресурсами TradingBot
   - Автоматически создаёт Deployment, Service, ConfigMap
   - Мониторит метрики производительности трейдинга
   - Реализует автоматическое масштабирование на основе рыночных условий
   - Обрабатывает жизненный цикл стратегии (backtest -> paper -> live)

## Навигация

[← Предыдущий день](../354-production-logging/ru.md) | [Следующий день →](../360-canary-deployments/ru.md)
