# Day 359: Kubernetes: Orchestration

## Trading Analogy

Imagine you're running a global trading operation across multiple exchanges in different time zones. You have trading bots in New York, London, Tokyo, and Singapore. Each bot needs to:

- **Stay alive**: If a bot crashes, someone needs to restart it immediately
- **Scale up**: During high volatility, you need more instances processing orders
- **Balance load**: Orders should be distributed evenly across available bots
- **Update smoothly**: Deploy new strategies without losing active positions

**This is exactly what Kubernetes does for your trading infrastructure:**

| Trading Operations | Kubernetes Concept |
|-------------------|-------------------|
| **Trading desk manager** | Kubernetes Control Plane |
| **Individual traders** | Pods (running containers) |
| **Team of traders** | Deployment (group of pods) |
| **Load balancer** | Service (routes traffic) |
| **Backup traders** | ReplicaSet (maintains pod count) |
| **Trading rules** | ConfigMaps & Secrets |
| **Resource limits** | Resource Quotas |

When a trader (pod) gets sick, the manager (Kubernetes) immediately brings in a replacement. When markets get volatile, more traders are added. Kubernetes is the ultimate trading floor manager.

## Kubernetes Fundamentals for Trading Systems

### Pod Definition for a Trading Bot

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kubernetes Pod specification for trading bot
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

/// Builder for creating trading bot pod specifications
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
    println!("=== Kubernetes Pod Specification for Trading Bot ===\n");

    let pod = TradingBotPodBuilder::new(
        "btc-momentum-bot",
        "trading/bot:v1.2.3"
    )
    .namespace("production")
    .exchange("binance")
    .strategy("momentum")
    .build();

    // Output as YAML (using serde_yaml in real code)
    println!("Pod Name: {}", pod.metadata.name);
    println!("Namespace: {}", pod.metadata.namespace);
    println!("Image: {}", pod.spec.containers[0].image);
    println!("\nLabels:");
    for (key, value) in &pod.metadata.labels {
        println!("  {}: {}", key, value);
    }
    println!("\nResource Limits:");
    println!("  CPU: {}", pod.spec.containers[0].resources.limits.cpu);
    println!("  Memory: {}", pod.spec.containers[0].resources.limits.memory);
    println!("\nProbes configured:");
    println!("  Liveness: /health on port 8080");
    println!("  Readiness: /ready on port 8080");
}
```

## Deployment for Scalable Trading

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kubernetes Deployment for trading bot fleet
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

/// Trading deployment manager
pub struct TradingDeploymentManager {
    deployments: HashMap<String, Deployment>,
}

impl TradingDeploymentManager {
    pub fn new() -> Self {
        TradingDeploymentManager {
            deployments: HashMap::new(),
        }
    }

    /// Create a new trading bot deployment
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

    /// Scale a deployment
    pub fn scale(&mut self, name: &str, replicas: u32) -> Result<(), String> {
        if let Some(deployment) = self.deployments.get_mut(name) {
            let old_replicas = deployment.spec.replicas;
            deployment.spec.replicas = replicas;
            println!(
                "Scaled deployment '{}': {} -> {} replicas",
                name, old_replicas, replicas
            );
            Ok(())
        } else {
            Err(format!("Deployment '{}' not found", name))
        }
    }

    /// Update deployment image
    pub fn update_image(&mut self, name: &str, new_image: &str) -> Result<(), String> {
        if let Some(deployment) = self.deployments.get_mut(name) {
            let old_image = deployment.spec.template.spec.containers[0].image.clone();
            deployment.spec.template.spec.containers[0].image = new_image.to_string();
            println!(
                "Updated deployment '{}' image: {} -> {}",
                name, old_image, new_image
            );
            Ok(())
        } else {
            Err(format!("Deployment '{}' not found", name))
        }
    }

    /// Get deployment status summary
    pub fn status(&self, name: &str) -> Option<String> {
        self.deployments.get(name).map(|d| {
            format!(
                "Deployment: {}\n  Replicas: {}\n  Image: {}\n  Strategy: {}",
                d.metadata.name,
                d.spec.replicas,
                d.spec.template.spec.containers[0].image,
                d.spec.strategy.strategy_type
            )
        })
    }
}

fn main() {
    println!("=== Kubernetes Deployment for Trading Bots ===\n");

    let mut manager = TradingDeploymentManager::new();

    // Create BTC trading deployment
    let btc_deployment = manager.create_deployment(
        "btc-trader",
        "trading/bot:v1.0.0",
        3,
        "binance",
        vec!["BTCUSDT".to_string(), "BTCBUSD".to_string()],
    );

    println!("Created deployment: {}", btc_deployment.metadata.name);
    println!("  Replicas: {}", btc_deployment.spec.replicas);
    println!("  Strategy: {}", btc_deployment.spec.strategy.strategy_type);

    // Create ETH trading deployment
    let eth_deployment = manager.create_deployment(
        "eth-trader",
        "trading/bot:v1.0.0",
        2,
        "binance",
        vec!["ETHUSDT".to_string()],
    );

    println!("\nCreated deployment: {}", eth_deployment.metadata.name);

    // Scale up during high volatility
    println!("\n--- High volatility detected, scaling up ---");
    manager.scale("btc-trader", 5).unwrap();

    // Update to new version
    println!("\n--- Deploying new version ---");
    manager.update_image("btc-trader", "trading/bot:v1.1.0").unwrap();

    // Check status
    println!("\n--- Current Status ---");
    if let Some(status) = manager.status("btc-trader") {
        println!("{}", status);
    }
}
```

## Horizontal Pod Autoscaler for Trading Load

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Metrics for autoscaling decisions
#[derive(Debug, Clone)]
pub struct TradingMetrics {
    pub orders_per_second: f64,
    pub latency_p99_ms: f64,
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub pending_orders: u32,
    pub active_positions: u32,
}

/// Horizontal Pod Autoscaler configuration
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

/// Trading-aware autoscaler
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

    /// Record metrics for scaling decisions
    pub fn record_metrics(&mut self, metrics: TradingMetrics) {
        self.metrics_history.push((Instant::now(), metrics));

        // Keep only last 5 minutes of metrics
        let cutoff = Instant::now() - Duration::from_secs(300);
        self.metrics_history.retain(|(time, _)| *time > cutoff);
    }

    /// Calculate desired replica count based on metrics
    pub fn calculate_desired_replicas(&self) -> u32 {
        if self.metrics_history.is_empty() {
            return self.current_replicas;
        }

        // Calculate average metrics over recent history
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

        // CPU-based scaling
        let cpu_ratio = avg_cpu / self.config.target_cpu_utilization;
        let cpu_desired = (self.current_replicas as f64 * cpu_ratio).ceil() as u32;

        // Latency-based scaling (if latency is high, scale up)
        let latency_multiplier = if avg_latency > 100.0 {
            1.5
        } else if avg_latency > 50.0 {
            1.2
        } else {
            1.0
        };

        // Pending orders-based scaling
        let pending_multiplier = if avg_pending > 100.0 {
            1.5
        } else if avg_pending > 50.0 {
            1.2
        } else {
            1.0
        };

        let desired = ((cpu_desired as f64) * latency_multiplier * pending_multiplier).ceil() as u32;

        // Clamp to min/max
        desired.clamp(self.config.min_replicas, self.config.max_replicas)
    }

    /// Evaluate and potentially scale
    pub fn evaluate(&mut self) -> ScalingDecision {
        let desired = self.calculate_desired_replicas();

        if desired == self.current_replicas {
            return ScalingDecision::NoChange;
        }

        let now = Instant::now();

        if desired > self.current_replicas {
            // Scale up
            if let Some(last) = self.last_scale_up {
                if now.duration_since(last).as_secs() < self.config.scale_up_stabilization_seconds {
                    return ScalingDecision::Stabilizing {
                        direction: "up",
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
            // Scale down
            if let Some(last) = self.last_scale_down {
                if now.duration_since(last).as_secs() < self.config.scale_down_stabilization_seconds {
                    return ScalingDecision::Stabilizing {
                        direction: "down",
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
                reason: "Resource utilization below threshold".to_string(),
            }
        }
    }

    fn get_scale_reason(&self) -> String {
        if let Some((_, metrics)) = self.metrics_history.last() {
            if metrics.cpu_utilization > self.config.target_cpu_utilization {
                return format!(
                    "CPU utilization {:.1}% exceeds target {:.1}%",
                    metrics.cpu_utilization,
                    self.config.target_cpu_utilization
                );
            }
            if metrics.latency_p99_ms > 100.0 {
                return format!(
                    "P99 latency {:.1}ms exceeds threshold",
                    metrics.latency_p99_ms
                );
            }
            if metrics.pending_orders > 50 {
                return format!(
                    "Pending orders {} exceeds threshold",
                    metrics.pending_orders
                );
            }
        }
        "Scaling based on combined metrics".to_string()
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
    println!("=== Trading Horizontal Pod Autoscaler ===\n");

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

    println!("Initial replicas: {}\n", autoscaler.current_replicas());

    // Simulate normal load
    println!("--- Normal trading conditions ---");
    autoscaler.record_metrics(TradingMetrics {
        orders_per_second: 50.0,
        latency_p99_ms: 25.0,
        cpu_utilization: 45.0,
        memory_utilization: 40.0,
        pending_orders: 10,
        active_positions: 50,
    });

    match autoscaler.evaluate() {
        ScalingDecision::NoChange => println!("No scaling needed"),
        decision => println!("Decision: {:?}", decision),
    }

    // Simulate high load (market volatility)
    println!("\n--- High volatility detected! ---");
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
            println!("SCALE UP: {} -> {} replicas", from, to);
            println!("Reason: {}", reason);
        }
        decision => println!("Decision: {:?}", decision),
    }

    println!("\nCurrent replicas after scaling: {}", autoscaler.current_replicas());

    // Simulate load decrease
    println!("\n--- Market calming down ---");
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

    // Would need to wait for stabilization period in real implementation
    println!("Stabilization period for scale down: 300 seconds");
}
```

## Service Discovery and Load Balancing

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::net::SocketAddr;

/// Kubernetes Service types
#[derive(Debug, Clone)]
pub enum ServiceType {
    ClusterIP,
    NodePort { node_port: u16 },
    LoadBalancer { external_ip: Option<String> },
    Headless,
}

/// Service endpoint representing a pod
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub pod_name: String,
    pub ip: String,
    pub port: u16,
    pub ready: bool,
    pub zone: String,
}

/// Kubernetes Service for trading
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

/// Service registry for trading services
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

    /// Register a service
    pub fn register_service(&mut self, service: TradingService) {
        println!("Registering service: {}/{}", service.namespace, service.name);
        self.services.insert(service.name.clone(), service);
    }

    /// Update endpoints for a service
    pub fn update_endpoints(&mut self, service_name: &str, endpoints: Vec<Endpoint>) {
        let ready_count = endpoints.iter().filter(|e| e.ready).count();
        println!(
            "Updating endpoints for '{}': {} total, {} ready",
            service_name,
            endpoints.len(),
            ready_count
        );
        self.endpoints.insert(service_name.to_string(), endpoints);
    }

    /// Get ready endpoints for a service
    pub fn get_ready_endpoints(&self, service_name: &str) -> Vec<&Endpoint> {
        self.endpoints
            .get(service_name)
            .map(|eps| eps.iter().filter(|e| e.ready).collect())
            .unwrap_or_default()
    }

    /// Get service ClusterIP
    pub fn get_cluster_ip(&self, service_name: &str) -> Option<&str> {
        self.services
            .get(service_name)
            .and_then(|s| s.cluster_ip.as_deref())
    }
}

/// Load balancer for trading requests
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

    /// Get next endpoint using round-robin
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

    /// Get endpoint with zone affinity (prefer local zone)
    pub fn get_endpoint_with_affinity(
        &mut self,
        service_name: &str,
        preferred_zone: &str,
    ) -> Option<Endpoint> {
        let registry = self.registry.read().unwrap();
        let endpoints = registry.get_ready_endpoints(service_name);

        // Try to find endpoint in preferred zone
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

        // Fall back to any available endpoint
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

/// Trading service mesh
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

    /// Initialize trading services
    pub fn init_services(&self) {
        let mut registry = self.registry.write().unwrap();

        // Order execution service
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

        // Market data service
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

        // Risk manager service
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

        // Simulate endpoint updates
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
                ready: false, // Pod not ready
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

    /// Route request to appropriate service
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
    println!("=== Kubernetes Service Discovery for Trading ===\n");

    let mut mesh = TradingServiceMesh::new();
    mesh.init_services();

    // Route requests to order executor
    println!("Routing requests to order-executor:");
    for i in 0..5 {
        if let Some(addr) = mesh.route_request("order-executor", None) {
            println!("  Request {}: routed to {}", i + 1, addr);
        }
    }

    // Route with zone affinity
    println!("\nRouting with zone affinity (us-east-1a):");
    for i in 0..3 {
        if let Some(addr) = mesh.route_request("order-executor", Some("us-east-1a")) {
            println!("  Request {}: routed to {}", i + 1, addr);
        }
    }

    // Check market-data service
    println!("\nMarket data service endpoints:");
    let registry = mesh.registry.read().unwrap();
    for ep in registry.get_ready_endpoints("market-data") {
        println!("  {} -> {}:{} (zone: {})",
            ep.pod_name, ep.ip, ep.port, ep.zone);
    }
}
```

## ConfigMaps and Secrets for Trading Configuration

```rust
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Kubernetes ConfigMap for trading configuration
#[derive(Debug, Clone)]
pub struct ConfigMap {
    pub name: String,
    pub namespace: String,
    pub data: HashMap<String, String>,
    pub binary_data: HashMap<String, Vec<u8>>,
}

/// Kubernetes Secret for sensitive trading data
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

/// Trading configuration manager
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

    /// Create a trading strategy config
    pub fn create_strategy_config(
        &mut self,
        name: &str,
        strategy_type: &str,
        parameters: HashMap<String, String>,
    ) -> ConfigMap {
        let mut data = HashMap::new();

        // Strategy configuration
        data.insert("strategy.type".to_string(), strategy_type.to_string());

        // Add all parameters
        for (key, value) in parameters {
            data.insert(format!("strategy.{}", key), value);
        }

        // Add common trading settings
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

    /// Create exchange credentials secret
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

    /// Get config value
    pub fn get_config(&self, config_name: &str, key: &str) -> Option<&String> {
        self.config_maps
            .get(config_name)
            .and_then(|cm| cm.data.get(key))
    }

    /// Get secret value (would be base64 decoded in real k8s)
    pub fn get_secret(&self, secret_name: &str, key: &str) -> Option<String> {
        self.secrets
            .get(secret_name)
            .and_then(|s| s.data.get(key))
            .map(|v| String::from_utf8_lossy(v).to_string())
    }

    /// Generate YAML representation
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

    /// Generate Secret YAML representation
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

/// Environment configuration loader
pub struct EnvConfigLoader {
    config_manager: TradingConfigManager,
}

impl EnvConfigLoader {
    pub fn new(manager: TradingConfigManager) -> Self {
        EnvConfigLoader {
            config_manager: manager,
        }
    }

    /// Load configuration into environment-like structure
    pub fn load_env(&self, config_name: &str, secret_name: &str) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Load config values
        if let Some(cm) = self.config_manager.config_maps.get(config_name) {
            for (key, value) in &cm.data {
                // Convert key format: strategy.type -> STRATEGY_TYPE
                let env_key = key.replace('.', "_").to_uppercase();
                env.insert(env_key, value.clone());
            }
        }

        // Load secret values
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
    println!("=== Kubernetes ConfigMaps and Secrets for Trading ===\n");

    let mut manager = TradingConfigManager::new();

    // Create strategy configuration
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

    println!("Created ConfigMap: {}", config.name);
    println!("\nConfiguration values:");
    for (key, value) in &config.data {
        println!("  {}: {}", key, value);
    }

    // Create exchange credentials
    let secret = manager.create_exchange_secret(
        "binance-credentials",
        "my-api-key-12345",
        "my-secret-key-67890",
        None,
    );

    println!("\n\nCreated Secret: {} (type: {:?})", secret.name, secret.secret_type);
    println!("Secret keys: {:?}", secret.data.keys().collect::<Vec<_>>());

    // Generate YAML
    println!("\n\n--- ConfigMap YAML ---");
    if let Some(yaml) = manager.to_yaml("momentum-strategy-config") {
        println!("{}", yaml);
    }

    println!("\n--- Secret YAML (base64 encoded) ---");
    if let Some(yaml) = manager.secret_to_yaml("binance-credentials") {
        println!("{}", yaml);
    }

    // Load as environment
    println!("\n--- Loaded Environment Variables ---");
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

## What We Learned

| Concept | Description |
|---------|-------------|
| **Pod** | Smallest deployable unit, runs trading bot containers |
| **Deployment** | Manages pod replicas with rolling updates |
| **Service** | Load balancing and service discovery for trading services |
| **HPA** | Automatic scaling based on CPU, memory, or custom metrics |
| **ConfigMap** | Store trading parameters and strategy configuration |
| **Secret** | Secure storage for API keys and credentials |
| **Probes** | Health checks to ensure trading bots are alive and ready |
| **Labels** | Organize and select trading resources |

## Practical Exercises

1. **Trading Bot Deployment**: Create a complete Kubernetes deployment that:
   - Runs multiple trading bot replicas
   - Uses rolling updates for zero-downtime deployments
   - Includes liveness and readiness probes
   - Mounts configuration from ConfigMaps
   - Loads credentials from Secrets

2. **Autoscaling Trading System**: Implement an HPA that:
   - Scales based on order throughput
   - Responds to latency spikes
   - Has different scale-up and scale-down policies
   - Respects trading hours (more pods during market hours)

3. **Service Mesh for Trading**: Create a service discovery system that:
   - Registers all trading microservices
   - Implements zone-aware load balancing
   - Handles endpoint health updates
   - Supports circuit breaking for failing services

4. **Configuration Management**: Build a system that:
   - Manages strategy parameters via ConfigMaps
   - Handles API keys rotation via Secrets
   - Supports hot-reloading of configuration
   - Validates configuration before applying

## Homework

1. **Multi-Exchange Deployment**: Create a Kubernetes architecture that:
   - Deploys separate trading bots per exchange
   - Uses namespaces for isolation
   - Implements cross-namespace communication
   - Manages different API credentials per exchange
   - Handles exchange-specific scaling requirements

2. **Disaster Recovery System**: Design a deployment that:
   - Spans multiple availability zones
   - Implements pod anti-affinity rules
   - Has automatic failover between zones
   - Maintains position state during pod restarts
   - Includes backup and restore procedures

3. **GitOps for Trading**: Implement a CI/CD pipeline that:
   - Uses GitOps for deployment management
   - Supports canary deployments for strategies
   - Includes automated rollback on poor performance
   - Maintains audit log of all deployments
   - Integrates with trading monitoring systems

4. **Custom Kubernetes Operator**: Build an operator that:
   - Manages TradingBot custom resources
   - Automatically creates Deployments, Services, ConfigMaps
   - Monitors trading performance metrics
   - Implements automatic scaling based on market conditions
   - Handles strategy lifecycle (backtest -> paper -> live)

## Navigation

[← Previous day](../354-production-logging/en.md) | [Next day →](../360-canary-deployments/en.md)
