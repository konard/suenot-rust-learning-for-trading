# День 361: A/B тестирование стратегий

## Аналогия из трейдинга

Представь, что ты управляешь хедж-фондом и хочешь улучшить свою торговую стратегию. Вместо того чтобы полностью заменить работающую стратегию на новую (рискуя потерять деньги), ты можешь:

**A/B тестирование в трейдинге:**
- Стратегия A (контрольная) — текущая проверенная стратегия
- Стратегия B (экспериментальная) — новая стратегия с потенциальными улучшениями
- Разделить капитал между ними и сравнить результаты

**Как работает A/B тестирование:**

| Аспект | Без A/B тестирования | С A/B тестированием |
|--------|----------------------|---------------------|
| **Риск** | Весь капитал под угрозой | Только часть капитала |
| **Данные** | Субъективные ощущения | Статистически значимые метрики |
| **Откат** | Сложный и долгий | Мгновенный переключатель |
| **Уверенность** | Надежда на лучшее | Доказанные результаты |

**Реальные сценарии A/B тестирования:**
- Тестирование нового алгоритма входа в позицию
- Сравнение разных размеров стоп-лосса
- Проверка нового источника данных
- Оценка влияния задержки на исполнение

## Основы A/B тестирования в Rust

### Структура эксперимента

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Вариант стратегии в A/B тесте
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrategyVariant {
    Control,     // Стратегия A — текущая
    Experiment,  // Стратегия B — новая
}

/// Метрики для одного варианта стратегии
#[derive(Debug)]
pub struct VariantMetrics {
    /// Количество сделок
    pub trades: AtomicU64,
    /// Количество прибыльных сделок
    pub winning_trades: AtomicU64,
    /// Общая прибыль в центах (со смещением для отрицательных)
    pub total_pnl_cents: AtomicU64,
    /// Максимальная просадка в центах
    pub max_drawdown_cents: AtomicU64,
    /// Сумма задержек исполнения в микросекундах
    pub latency_sum_us: AtomicU64,
    /// Количество измерений задержки
    pub latency_count: AtomicU64,
}

impl VariantMetrics {
    pub fn new() -> Self {
        VariantMetrics {
            trades: AtomicU64::new(0),
            winning_trades: AtomicU64::new(0),
            total_pnl_cents: AtomicU64::new(1_000_000_00), // Смещение для отрицательных
            max_drawdown_cents: AtomicU64::new(0),
            latency_sum_us: AtomicU64::new(0),
            latency_count: AtomicU64::new(0),
        }
    }

    pub fn record_trade(&self, pnl: f64, latency: Duration) {
        self.trades.fetch_add(1, Ordering::Relaxed);

        if pnl > 0.0 {
            self.winning_trades.fetch_add(1, Ordering::Relaxed);
        }

        // Обновляем P&L
        let pnl_cents = (pnl * 100.0) as i64;
        if pnl_cents >= 0 {
            self.total_pnl_cents.fetch_add(pnl_cents as u64, Ordering::Relaxed);
        } else {
            self.total_pnl_cents.fetch_sub((-pnl_cents) as u64, Ordering::Relaxed);
        }

        // Записываем задержку
        let latency_us = latency.as_micros() as u64;
        self.latency_sum_us.fetch_add(latency_us, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_drawdown(&self, drawdown: f64) {
        let drawdown_cents = (drawdown.abs() * 100.0) as u64;
        self.max_drawdown_cents.fetch_max(drawdown_cents, Ordering::Relaxed);
    }

    pub fn get_pnl(&self) -> f64 {
        let cents = self.total_pnl_cents.load(Ordering::Relaxed);
        (cents as f64 / 100.0) - 1_000_000.0
    }

    pub fn get_win_rate(&self) -> f64 {
        let trades = self.trades.load(Ordering::Relaxed);
        let wins = self.winning_trades.load(Ordering::Relaxed);
        if trades == 0 {
            return 0.0;
        }
        (wins as f64 / trades as f64) * 100.0
    }

    pub fn get_avg_latency(&self) -> Option<Duration> {
        let count = self.latency_count.load(Ordering::Relaxed);
        if count == 0 {
            return None;
        }
        let sum = self.latency_sum_us.load(Ordering::Relaxed);
        Some(Duration::from_micros(sum / count))
    }
}

/// A/B эксперимент для торговых стратегий
pub struct ABExperiment {
    pub name: String,
    pub control: Arc<VariantMetrics>,
    pub experiment: Arc<VariantMetrics>,
    pub traffic_split: f64,  // Доля трафика на эксперимент (0.0 - 1.0)
    pub start_time: Instant,
    pub min_trades_required: u64,
}

impl ABExperiment {
    pub fn new(name: &str, traffic_split: f64) -> Self {
        ABExperiment {
            name: name.to_string(),
            control: Arc::new(VariantMetrics::new()),
            experiment: Arc::new(VariantMetrics::new()),
            traffic_split: traffic_split.clamp(0.0, 1.0),
            start_time: Instant::now(),
            min_trades_required: 100,
        }
    }

    /// Выбор варианта на основе хеша (детерминированный)
    pub fn select_variant(&self, trade_id: u64) -> StrategyVariant {
        // Простой хеш для детерминированного распределения
        let hash = trade_id.wrapping_mul(2654435761) % 1000;
        let threshold = (self.traffic_split * 1000.0) as u64;

        if hash < threshold {
            StrategyVariant::Experiment
        } else {
            StrategyVariant::Control
        }
    }

    /// Получить метрики для варианта
    pub fn get_metrics(&self, variant: StrategyVariant) -> Arc<VariantMetrics> {
        match variant {
            StrategyVariant::Control => Arc::clone(&self.control),
            StrategyVariant::Experiment => Arc::clone(&self.experiment),
        }
    }

    /// Проверка статистической значимости
    pub fn is_statistically_significant(&self) -> bool {
        let control_trades = self.control.trades.load(Ordering::Relaxed);
        let experiment_trades = self.experiment.trades.load(Ordering::Relaxed);

        control_trades >= self.min_trades_required &&
        experiment_trades >= self.min_trades_required
    }

    /// Генерация отчёта
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("╔══════════════════════════════════════════════════╗\n"));
        report.push_str(&format!("║  A/B Test: {:37} ║\n", self.name));
        report.push_str(&format!("╠══════════════════════════════════════════════════╣\n"));

        let duration = self.start_time.elapsed();
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        report.push_str(&format!("║ Duration: {:02}h {:02}m {:30} ║\n", hours, minutes, ""));
        report.push_str(&format!("║ Traffic Split: {:.0}% experiment {:19} ║\n",
            self.traffic_split * 100.0, ""));

        report.push_str(&format!("╠══════════════════════════════════════════════════╣\n"));
        report.push_str(&format!("║ {:20} │ {:10} │ {:10} ║\n", "Metric", "Control", "Experiment"));
        report.push_str(&format!("╠══════════════════════════════════════════════════╣\n"));

        // Сделки
        let control_trades = self.control.trades.load(Ordering::Relaxed);
        let exp_trades = self.experiment.trades.load(Ordering::Relaxed);
        report.push_str(&format!("║ {:20} │ {:>10} │ {:>10} ║\n",
            "Trades", control_trades, exp_trades));

        // Win Rate
        let control_wr = self.control.get_win_rate();
        let exp_wr = self.experiment.get_win_rate();
        report.push_str(&format!("║ {:20} │ {:>9.1}% │ {:>9.1}% ║\n",
            "Win Rate", control_wr, exp_wr));

        // P&L
        let control_pnl = self.control.get_pnl();
        let exp_pnl = self.experiment.get_pnl();
        report.push_str(&format!("║ {:20} │ ${:>9.2} │ ${:>9.2} ║\n",
            "Total P&L", control_pnl, exp_pnl));

        // Средняя задержка
        if let (Some(ctrl_lat), Some(exp_lat)) =
            (self.control.get_avg_latency(), self.experiment.get_avg_latency())
        {
            report.push_str(&format!("║ {:20} │ {:>8.2}ms │ {:>8.2}ms ║\n",
                "Avg Latency",
                ctrl_lat.as_secs_f64() * 1000.0,
                exp_lat.as_secs_f64() * 1000.0));
        }

        report.push_str(&format!("╠══════════════════════════════════════════════════╣\n"));

        // Вывод
        let significant = self.is_statistically_significant();
        let winner = if exp_pnl > control_pnl { "EXPERIMENT" } else { "CONTROL" };
        let improvement = if control_pnl != 0.0 {
            ((exp_pnl - control_pnl) / control_pnl.abs()) * 100.0
        } else {
            0.0
        };

        if significant {
            report.push_str(&format!("║ Winner: {:12} ({:+.1}% improvement) {:6} ║\n",
                winner, improvement, ""));
            report.push_str(&format!("║ Status: Statistically significant {:14} ║\n", ""));
        } else {
            report.push_str(&format!("║ Status: Need more data {:25} ║\n", ""));
            let needed = self.min_trades_required.saturating_sub(
                control_trades.min(exp_trades)
            );
            report.push_str(&format!("║ Trades needed: ~{} more {:24} ║\n", needed, ""));
        }

        report.push_str(&format!("╚══════════════════════════════════════════════════╝\n"));

        report
    }
}

fn main() {
    println!("=== A/B Testing Demo ===\n");

    // Создаём эксперимент: 30% трафика на новую стратегию
    let experiment = ABExperiment::new("New Entry Algorithm", 0.3);

    // Симуляция 200 сделок
    for trade_id in 0..200 {
        let variant = experiment.select_variant(trade_id);
        let metrics = experiment.get_metrics(variant);

        // Симуляция результатов
        let (pnl, latency) = match variant {
            StrategyVariant::Control => {
                // Контроль: стабильные результаты
                let pnl = if trade_id % 3 == 0 { -15.0 } else { 10.0 };
                let latency = Duration::from_millis(50);
                (pnl, latency)
            }
            StrategyVariant::Experiment => {
                // Эксперимент: немного лучше, но менее стабильно
                let pnl = if trade_id % 4 == 0 { -20.0 } else { 12.0 };
                let latency = Duration::from_millis(45);
                (pnl, latency)
            }
        };

        metrics.record_trade(pnl, latency);
    }

    // Генерация отчёта
    println!("{}", experiment.generate_report());
}
```

### Распределитель трафика

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Правило распределения трафика
#[derive(Debug, Clone)]
pub struct TrafficRule {
    pub variant: String,
    pub weight: u32,           // Вес (например, 70 = 70%)
    pub conditions: Vec<Condition>,
}

/// Условие для применения правила
#[derive(Debug, Clone)]
pub enum Condition {
    Symbol(String),           // Только для определённого символа
    TimeRange(u32, u32),      // Только в определённые часы
    AccountSize(f64, f64),    // Только для определённого размера счёта
    Always,                   // Всегда
}

/// Распределитель трафика для A/B тестирования
pub struct TrafficRouter {
    experiments: RwLock<HashMap<String, Vec<TrafficRule>>>,
}

impl TrafficRouter {
    pub fn new() -> Self {
        TrafficRouter {
            experiments: RwLock::new(HashMap::new()),
        }
    }

    /// Добавить эксперимент
    pub fn add_experiment(&self, name: &str, rules: Vec<TrafficRule>) {
        let mut experiments = self.experiments.write().unwrap();
        experiments.insert(name.to_string(), rules);
    }

    /// Выбрать вариант для эксперимента
    pub fn route(&self, experiment_name: &str, context: &TradeContext) -> Option<String> {
        let experiments = self.experiments.read().unwrap();
        let rules = experiments.get(experiment_name)?;

        // Фильтруем правила по условиям
        let applicable_rules: Vec<&TrafficRule> = rules
            .iter()
            .filter(|r| self.check_conditions(&r.conditions, context))
            .collect();

        if applicable_rules.is_empty() {
            return None;
        }

        // Вычисляем общий вес
        let total_weight: u32 = applicable_rules.iter().map(|r| r.weight).sum();

        // Детерминированный выбор на основе хеша
        let hash = self.hash_context(context);
        let point = (hash % total_weight as u64) as u32;

        let mut cumulative = 0u32;
        for rule in applicable_rules {
            cumulative += rule.weight;
            if point < cumulative {
                return Some(rule.variant.clone());
            }
        }

        None
    }

    fn check_conditions(&self, conditions: &[Condition], context: &TradeContext) -> bool {
        conditions.iter().all(|cond| match cond {
            Condition::Symbol(s) => &context.symbol == s,
            Condition::TimeRange(start, end) => {
                context.hour >= *start && context.hour < *end
            }
            Condition::AccountSize(min, max) => {
                context.account_size >= *min && context.account_size < *max
            }
            Condition::Always => true,
        })
    }

    fn hash_context(&self, context: &TradeContext) -> u64 {
        let mut hasher = DefaultHasher::new();
        context.trade_id.hash(&mut hasher);
        context.symbol.hash(&mut hasher);
        hasher.finish()
    }
}

/// Контекст сделки для маршрутизации
#[derive(Debug)]
pub struct TradeContext {
    pub trade_id: u64,
    pub symbol: String,
    pub hour: u32,
    pub account_size: f64,
}

fn main() {
    println!("=== Traffic Router Demo ===\n");

    let router = TrafficRouter::new();

    // Настраиваем эксперимент: новый алгоритм для BTC
    router.add_experiment("btc_algorithm_v2", vec![
        TrafficRule {
            variant: "control".to_string(),
            weight: 80,
            conditions: vec![Condition::Always],
        },
        TrafficRule {
            variant: "experiment".to_string(),
            weight: 20,
            conditions: vec![Condition::Always],
        },
    ]);

    // Настраиваем эксперимент с условиями
    router.add_experiment("night_strategy", vec![
        TrafficRule {
            variant: "aggressive".to_string(),
            weight: 30,
            conditions: vec![
                Condition::TimeRange(22, 6),  // Ночью
                Condition::Symbol("BTCUSDT".to_string()),
            ],
        },
        TrafficRule {
            variant: "conservative".to_string(),
            weight: 70,
            conditions: vec![Condition::Always],
        },
    ]);

    // Тестируем маршрутизацию
    let contexts = vec![
        TradeContext { trade_id: 1, symbol: "BTCUSDT".to_string(), hour: 14, account_size: 10000.0 },
        TradeContext { trade_id: 2, symbol: "BTCUSDT".to_string(), hour: 23, account_size: 10000.0 },
        TradeContext { trade_id: 3, symbol: "ETHUSDT".to_string(), hour: 10, account_size: 5000.0 },
    ];

    for ctx in &contexts {
        println!("Trade: {:?}", ctx);

        if let Some(variant) = router.route("btc_algorithm_v2", ctx) {
            println!("  btc_algorithm_v2 -> {}", variant);
        }

        if let Some(variant) = router.route("night_strategy", ctx) {
            println!("  night_strategy -> {}", variant);
        }

        println!();
    }
}
```

## Статистический анализ результатов

```rust
use std::f64::consts::PI;

/// Статистический калькулятор для A/B тестов
pub struct StatisticalCalculator;

impl StatisticalCalculator {
    /// Расчёт Z-score для сравнения двух пропорций (win rate)
    pub fn z_score_proportions(
        control_wins: u64,
        control_total: u64,
        experiment_wins: u64,
        experiment_total: u64,
    ) -> f64 {
        if control_total == 0 || experiment_total == 0 {
            return 0.0;
        }

        let p1 = control_wins as f64 / control_total as f64;
        let p2 = experiment_wins as f64 / experiment_total as f64;

        // Объединённая пропорция
        let p_pooled = (control_wins + experiment_wins) as f64
            / (control_total + experiment_total) as f64;

        // Стандартная ошибка
        let se = (p_pooled * (1.0 - p_pooled) *
            (1.0 / control_total as f64 + 1.0 / experiment_total as f64)).sqrt();

        if se == 0.0 {
            return 0.0;
        }

        (p2 - p1) / se
    }

    /// Расчёт p-value из Z-score (двусторонний тест)
    pub fn p_value_from_z(z: f64) -> f64 {
        // Аппроксимация функции распределения нормального распределения
        let t = 1.0 / (1.0 + 0.2316419 * z.abs());
        let d = 0.3989423 * (-z * z / 2.0).exp();
        let p = d * t * (0.3193815 + t * (-0.3565638 + t *
            (1.781478 + t * (-1.821256 + t * 1.330274))));

        2.0 * if z > 0.0 { p } else { 1.0 - p }
    }

    /// Проверка статистической значимости
    pub fn is_significant(p_value: f64, alpha: f64) -> bool {
        p_value < alpha
    }

    /// Расчёт доверительного интервала для разницы пропорций
    pub fn confidence_interval(
        p1: f64,
        n1: u64,
        p2: f64,
        n2: u64,
        confidence: f64,
    ) -> (f64, f64) {
        let diff = p2 - p1;

        // Z-значение для уровня доверия
        let z = match confidence {
            x if x >= 0.99 => 2.576,
            x if x >= 0.95 => 1.96,
            x if x >= 0.90 => 1.645,
            _ => 1.96,
        };

        // Стандартная ошибка разницы
        let se = ((p1 * (1.0 - p1) / n1 as f64) +
                  (p2 * (1.0 - p2) / n2 as f64)).sqrt();

        let margin = z * se;
        (diff - margin, diff + margin)
    }

    /// Расчёт необходимого размера выборки
    pub fn required_sample_size(
        baseline_rate: f64,       // Текущий win rate
        minimum_effect: f64,       // Минимальное улучшение для обнаружения
        alpha: f64,                // Уровень значимости (обычно 0.05)
        power: f64,                // Статистическая мощность (обычно 0.8)
    ) -> u64 {
        // Z-значения
        let z_alpha = match alpha {
            x if x <= 0.01 => 2.576,
            x if x <= 0.05 => 1.96,
            _ => 1.645,
        };

        let z_beta = match power {
            x if x >= 0.9 => 1.282,
            x if x >= 0.8 => 0.842,
            _ => 0.524,
        };

        let p1 = baseline_rate;
        let p2 = baseline_rate + minimum_effect;
        let p_avg = (p1 + p2) / 2.0;

        let numerator = (z_alpha * (2.0 * p_avg * (1.0 - p_avg)).sqrt() +
                        z_beta * (p1 * (1.0 - p1) + p2 * (1.0 - p2)).sqrt()).powi(2);
        let denominator = (p2 - p1).powi(2);

        (numerator / denominator).ceil() as u64
    }
}

/// Результат статистического анализа A/B теста
#[derive(Debug)]
pub struct ABTestAnalysis {
    pub z_score: f64,
    pub p_value: f64,
    pub is_significant: bool,
    pub confidence_interval: (f64, f64),
    pub winner: Option<String>,
    pub lift: f64,  // Процентное улучшение
}

/// Анализатор A/B тестов
pub struct ABTestAnalyzer {
    pub alpha: f64,      // Уровень значимости
    pub confidence: f64, // Уровень доверия для интервала
}

impl ABTestAnalyzer {
    pub fn new() -> Self {
        ABTestAnalyzer {
            alpha: 0.05,
            confidence: 0.95,
        }
    }

    pub fn analyze(
        &self,
        control_wins: u64,
        control_total: u64,
        experiment_wins: u64,
        experiment_total: u64,
    ) -> ABTestAnalysis {
        let p1 = if control_total > 0 {
            control_wins as f64 / control_total as f64
        } else { 0.0 };

        let p2 = if experiment_total > 0 {
            experiment_wins as f64 / experiment_total as f64
        } else { 0.0 };

        let z_score = StatisticalCalculator::z_score_proportions(
            control_wins, control_total,
            experiment_wins, experiment_total
        );

        let p_value = StatisticalCalculator::p_value_from_z(z_score);
        let is_significant = StatisticalCalculator::is_significant(p_value, self.alpha);

        let confidence_interval = StatisticalCalculator::confidence_interval(
            p1, control_total,
            p2, experiment_total,
            self.confidence
        );

        let lift = if p1 > 0.0 { ((p2 - p1) / p1) * 100.0 } else { 0.0 };

        let winner = if is_significant {
            Some(if p2 > p1 { "Experiment".to_string() } else { "Control".to_string() })
        } else {
            None
        };

        ABTestAnalysis {
            z_score,
            p_value,
            is_significant,
            confidence_interval,
            winner,
            lift,
        }
    }

    pub fn print_analysis(&self, analysis: &ABTestAnalysis) {
        println!("╔══════════════════════════════════════════╗");
        println!("║     Statistical Analysis Results         ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║ Z-Score: {:>29.4} ║", analysis.z_score);
        println!("║ P-Value: {:>29.6} ║", analysis.p_value);
        println!("║ Significant (α={}): {:>18} ║",
            self.alpha,
            if analysis.is_significant { "YES" } else { "NO" });
        println!("║ 95% CI: [{:>8.4}, {:>8.4}] {:>8} ║",
            analysis.confidence_interval.0,
            analysis.confidence_interval.1, "");
        println!("║ Lift: {:>+29.2}% ║", analysis.lift);

        if let Some(winner) = &analysis.winner {
            println!("║ Winner: {:>28} ║", winner);
        } else {
            println!("║ Winner: {:>28} ║", "Inconclusive");
        }

        println!("╚══════════════════════════════════════════╝");
    }
}

fn main() {
    println!("=== Statistical Analysis Demo ===\n");

    let analyzer = ABTestAnalyzer::new();

    // Сценарий 1: Небольшая выборка — нет значимости
    println!("Scenario 1: Small sample");
    let analysis = analyzer.analyze(10, 20, 12, 20);
    analyzer.print_analysis(&analysis);

    // Сценарий 2: Большая выборка — есть значимость
    println!("\nScenario 2: Large sample with significant difference");
    let analysis = analyzer.analyze(450, 1000, 520, 1000);
    analyzer.print_analysis(&analysis);

    // Сценарий 3: Большая выборка — нет разницы
    println!("\nScenario 3: Large sample, no difference");
    let analysis = analyzer.analyze(500, 1000, 505, 1000);
    analyzer.print_analysis(&analysis);

    // Расчёт необходимого размера выборки
    println!("\n=== Sample Size Calculation ===");
    let sample_size = StatisticalCalculator::required_sample_size(
        0.45,   // Текущий win rate 45%
        0.05,   // Хотим обнаружить улучшение на 5%
        0.05,   // α = 0.05
        0.80,   // Мощность 80%
    );
    println!("Required sample size per group: {} trades", sample_size);
}
```

## Интеграция в торговую систему

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Торговая стратегия — трейт для всех стратегий
pub trait TradingStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn should_buy(&self, price: f64, indicators: &Indicators) -> bool;
    fn should_sell(&self, price: f64, indicators: &Indicators) -> bool;
    fn calculate_position_size(&self, balance: f64, risk: f64) -> f64;
}

/// Технические индикаторы
#[derive(Debug, Clone)]
pub struct Indicators {
    pub rsi: f64,
    pub macd: f64,
    pub sma_20: f64,
    pub sma_50: f64,
    pub volume: f64,
}

/// Контрольная стратегия (текущая)
pub struct ControlStrategy;

impl TradingStrategy for ControlStrategy {
    fn name(&self) -> &str { "Control" }

    fn should_buy(&self, price: f64, indicators: &Indicators) -> bool {
        indicators.rsi < 30.0 && indicators.macd > 0.0
    }

    fn should_sell(&self, price: f64, indicators: &Indicators) -> bool {
        indicators.rsi > 70.0 || indicators.macd < 0.0
    }

    fn calculate_position_size(&self, balance: f64, risk: f64) -> f64 {
        balance * risk * 0.01
    }
}

/// Экспериментальная стратегия (новая)
pub struct ExperimentStrategy;

impl TradingStrategy for ExperimentStrategy {
    fn name(&self) -> &str { "Experiment" }

    fn should_buy(&self, price: f64, indicators: &Indicators) -> bool {
        // Более агрессивные условия входа
        indicators.rsi < 35.0 &&
        indicators.macd > 0.0 &&
        indicators.sma_20 > indicators.sma_50
    }

    fn should_sell(&self, price: f64, indicators: &Indicators) -> bool {
        // Более быстрый выход
        indicators.rsi > 65.0 ||
        (indicators.macd < 0.0 && indicators.sma_20 < indicators.sma_50)
    }

    fn calculate_position_size(&self, balance: f64, risk: f64) -> f64 {
        // Динамический размер на основе волатильности
        balance * risk * 0.015
    }
}

/// Менеджер A/B тестирования для торговой системы
pub struct TradingABManager {
    control: Arc<dyn TradingStrategy>,
    experiment: Arc<dyn TradingStrategy>,
    traffic_split: f64,
    control_metrics: Arc<StrategyMetrics>,
    experiment_metrics: Arc<StrategyMetrics>,
    is_active: AtomicBool,
    trade_counter: AtomicU64,
}

/// Метрики стратегии
pub struct StrategyMetrics {
    trades: AtomicU64,
    wins: AtomicU64,
    pnl_cents: AtomicU64,
    total_latency_us: AtomicU64,
}

impl StrategyMetrics {
    fn new() -> Self {
        StrategyMetrics {
            trades: AtomicU64::new(0),
            wins: AtomicU64::new(0),
            pnl_cents: AtomicU64::new(1_000_000_00),
            total_latency_us: AtomicU64::new(0),
        }
    }

    fn record(&self, pnl: f64, latency: Duration) {
        self.trades.fetch_add(1, Ordering::Relaxed);
        if pnl > 0.0 {
            self.wins.fetch_add(1, Ordering::Relaxed);
        }

        let cents = (pnl * 100.0) as i64;
        if cents >= 0 {
            self.pnl_cents.fetch_add(cents as u64, Ordering::Relaxed);
        } else {
            self.pnl_cents.fetch_sub((-cents) as u64, Ordering::Relaxed);
        }

        self.total_latency_us.fetch_add(latency.as_micros() as u64, Ordering::Relaxed);
    }

    fn get_summary(&self) -> (u64, u64, f64) {
        let trades = self.trades.load(Ordering::Relaxed);
        let wins = self.wins.load(Ordering::Relaxed);
        let pnl = (self.pnl_cents.load(Ordering::Relaxed) as f64 / 100.0) - 1_000_000.0;
        (trades, wins, pnl)
    }
}

impl TradingABManager {
    pub fn new(
        control: Arc<dyn TradingStrategy>,
        experiment: Arc<dyn TradingStrategy>,
        traffic_split: f64,
    ) -> Self {
        TradingABManager {
            control,
            experiment,
            traffic_split: traffic_split.clamp(0.0, 1.0),
            control_metrics: Arc::new(StrategyMetrics::new()),
            experiment_metrics: Arc::new(StrategyMetrics::new()),
            is_active: AtomicBool::new(true),
            trade_counter: AtomicU64::new(0),
        }
    }

    /// Получить стратегию для текущей сделки
    pub fn get_strategy(&self) -> (Arc<dyn TradingStrategy>, bool) {
        if !self.is_active.load(Ordering::Relaxed) {
            return (Arc::clone(&self.control), false);
        }

        let trade_id = self.trade_counter.fetch_add(1, Ordering::Relaxed);
        let hash = trade_id.wrapping_mul(2654435761) % 1000;
        let is_experiment = hash < (self.traffic_split * 1000.0) as u64;

        if is_experiment {
            (Arc::clone(&self.experiment), true)
        } else {
            (Arc::clone(&self.control), false)
        }
    }

    /// Записать результат сделки
    pub fn record_trade(&self, is_experiment: bool, pnl: f64, latency: Duration) {
        if is_experiment {
            self.experiment_metrics.record(pnl, latency);
        } else {
            self.control_metrics.record(pnl, latency);
        }
    }

    /// Остановить эксперимент
    pub fn stop_experiment(&self) {
        self.is_active.store(false, Ordering::Relaxed);
    }

    /// Выбрать победителя и переключиться на него
    pub fn select_winner(&self) -> &str {
        let (_, ctrl_wins, ctrl_pnl) = self.control_metrics.get_summary();
        let (_, exp_wins, exp_pnl) = self.experiment_metrics.get_summary();

        // Выбираем по P&L
        if exp_pnl > ctrl_pnl {
            "experiment"
        } else {
            "control"
        }
    }

    /// Генерация отчёта
    pub fn report(&self) -> String {
        let (ctrl_trades, ctrl_wins, ctrl_pnl) = self.control_metrics.get_summary();
        let (exp_trades, exp_wins, exp_pnl) = self.experiment_metrics.get_summary();

        let ctrl_wr = if ctrl_trades > 0 {
            (ctrl_wins as f64 / ctrl_trades as f64) * 100.0
        } else { 0.0 };

        let exp_wr = if exp_trades > 0 {
            (exp_wins as f64 / exp_trades as f64) * 100.0
        } else { 0.0 };

        format!(
            "A/B Test Report:\n\
             Control ({}):\n  Trades: {}, Win Rate: {:.1}%, P&L: ${:.2}\n\
             Experiment ({}):\n  Trades: {}, Win Rate: {:.1}%, P&L: ${:.2}\n\
             Winner: {}",
            self.control.name(), ctrl_trades, ctrl_wr, ctrl_pnl,
            self.experiment.name(), exp_trades, exp_wr, exp_pnl,
            self.select_winner()
        )
    }
}

fn main() {
    println!("=== Trading A/B Manager Demo ===\n");

    let control = Arc::new(ControlStrategy) as Arc<dyn TradingStrategy>;
    let experiment = Arc::new(ExperimentStrategy) as Arc<dyn TradingStrategy>;

    let manager = TradingABManager::new(control, experiment, 0.3);

    // Симуляция торговли
    let indicators = Indicators {
        rsi: 25.0,
        macd: 0.5,
        sma_20: 50100.0,
        sma_50: 50000.0,
        volume: 1000.0,
    };

    for i in 0..100 {
        let (strategy, is_experiment) = manager.get_strategy();

        // Симуляция сделки
        let should_buy = strategy.should_buy(50000.0, &indicators);
        let pnl = if should_buy {
            if i % 3 == 0 { -20.0 } else { 15.0 }
        } else {
            0.0
        };

        let latency = Duration::from_millis(45 + (i % 20) as u64);
        manager.record_trade(is_experiment, pnl, latency);
    }

    println!("{}", manager.report());
}
```

## Автоматическое завершение экспериментов

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Критерии завершения эксперимента
#[derive(Debug, Clone)]
pub struct StoppingCriteria {
    /// Минимальное количество сделок для каждого варианта
    pub min_trades: u64,
    /// Максимальная продолжительность эксперимента
    pub max_duration: Duration,
    /// Минимальный уровень значимости
    pub min_significance: f64,
    /// Остановка при критическом ухудшении
    pub stop_on_harm: bool,
    /// Порог вреда (процентное ухудшение P&L)
    pub harm_threshold: f64,
}

impl Default for StoppingCriteria {
    fn default() -> Self {
        StoppingCriteria {
            min_trades: 100,
            max_duration: Duration::from_secs(7 * 24 * 3600), // 1 неделя
            min_significance: 0.95,
            stop_on_harm: true,
            harm_threshold: -10.0, // -10%
        }
    }
}

/// Причина остановки эксперимента
#[derive(Debug, Clone)]
pub enum StopReason {
    /// Достигнута статистическая значимость
    SignificanceReached { winner: String, confidence: f64 },
    /// Истекло максимальное время
    TimeExpired,
    /// Обнаружен значительный вред
    HarmDetected { loss_percent: f64 },
    /// Ручная остановка
    ManualStop,
    /// Эксперимент всё ещё активен
    StillRunning,
}

/// Монитор автоматического завершения
pub struct ExperimentMonitor {
    start_time: Instant,
    criteria: StoppingCriteria,
    control_trades: AtomicU64,
    control_pnl_cents: AtomicU64,
    experiment_trades: AtomicU64,
    experiment_pnl_cents: AtomicU64,
    is_stopped: AtomicBool,
    stop_reason: std::sync::RwLock<StopReason>,
}

impl ExperimentMonitor {
    pub fn new(criteria: StoppingCriteria) -> Self {
        ExperimentMonitor {
            start_time: Instant::now(),
            criteria,
            control_trades: AtomicU64::new(0),
            control_pnl_cents: AtomicU64::new(1_000_000_00),
            experiment_trades: AtomicU64::new(0),
            experiment_pnl_cents: AtomicU64::new(1_000_000_00),
            is_stopped: AtomicBool::new(false),
            stop_reason: std::sync::RwLock::new(StopReason::StillRunning),
        }
    }

    pub fn record_control(&self, pnl: f64) {
        self.control_trades.fetch_add(1, Ordering::Relaxed);
        let cents = (pnl * 100.0) as i64;
        if cents >= 0 {
            self.control_pnl_cents.fetch_add(cents as u64, Ordering::Relaxed);
        } else {
            self.control_pnl_cents.fetch_sub((-cents) as u64, Ordering::Relaxed);
        }
    }

    pub fn record_experiment(&self, pnl: f64) {
        self.experiment_trades.fetch_add(1, Ordering::Relaxed);
        let cents = (pnl * 100.0) as i64;
        if cents >= 0 {
            self.experiment_pnl_cents.fetch_add(cents as u64, Ordering::Relaxed);
        } else {
            self.experiment_pnl_cents.fetch_sub((-cents) as u64, Ordering::Relaxed);
        }
    }

    /// Проверка критериев остановки
    pub fn check_stopping(&self) -> StopReason {
        if self.is_stopped.load(Ordering::Relaxed) {
            return self.stop_reason.read().unwrap().clone();
        }

        // Проверка времени
        if self.start_time.elapsed() >= self.criteria.max_duration {
            self.stop(StopReason::TimeExpired);
            return StopReason::TimeExpired;
        }

        let ctrl_trades = self.control_trades.load(Ordering::Relaxed);
        let exp_trades = self.experiment_trades.load(Ordering::Relaxed);

        // Проверка минимального количества сделок
        if ctrl_trades < self.criteria.min_trades ||
           exp_trades < self.criteria.min_trades {
            return StopReason::StillRunning;
        }

        let ctrl_pnl = (self.control_pnl_cents.load(Ordering::Relaxed) as f64 / 100.0) - 1_000_000.0;
        let exp_pnl = (self.experiment_pnl_cents.load(Ordering::Relaxed) as f64 / 100.0) - 1_000_000.0;

        // Проверка на вред
        if self.criteria.stop_on_harm && ctrl_pnl > 0.0 {
            let loss_percent = ((exp_pnl - ctrl_pnl) / ctrl_pnl) * 100.0;
            if loss_percent < self.criteria.harm_threshold {
                let reason = StopReason::HarmDetected { loss_percent };
                self.stop(reason.clone());
                return reason;
            }
        }

        // Упрощённая проверка значимости
        let improvement = if ctrl_pnl != 0.0 {
            ((exp_pnl - ctrl_pnl) / ctrl_pnl.abs()) * 100.0
        } else {
            0.0
        };

        // Если улучшение > 20% и достаточно данных
        if improvement.abs() > 20.0 && ctrl_trades >= 200 && exp_trades >= 200 {
            let winner = if exp_pnl > ctrl_pnl { "Experiment" } else { "Control" };
            let reason = StopReason::SignificanceReached {
                winner: winner.to_string(),
                confidence: 0.95,
            };
            self.stop(reason.clone());
            return reason;
        }

        StopReason::StillRunning
    }

    fn stop(&self, reason: StopReason) {
        self.is_stopped.store(true, Ordering::Relaxed);
        *self.stop_reason.write().unwrap() = reason;
    }

    pub fn is_running(&self) -> bool {
        !self.is_stopped.load(Ordering::Relaxed)
    }

    pub fn get_status(&self) -> String {
        let ctrl_trades = self.control_trades.load(Ordering::Relaxed);
        let exp_trades = self.experiment_trades.load(Ordering::Relaxed);
        let ctrl_pnl = (self.control_pnl_cents.load(Ordering::Relaxed) as f64 / 100.0) - 1_000_000.0;
        let exp_pnl = (self.experiment_pnl_cents.load(Ordering::Relaxed) as f64 / 100.0) - 1_000_000.0;
        let elapsed = self.start_time.elapsed();

        format!(
            "Experiment Status:\n\
             Duration: {:?}\n\
             Control: {} trades, P&L: ${:.2}\n\
             Experiment: {} trades, P&L: ${:.2}\n\
             Status: {:?}",
            elapsed, ctrl_trades, ctrl_pnl, exp_trades, exp_pnl,
            self.check_stopping()
        )
    }
}

fn main() {
    println!("=== Experiment Monitor Demo ===\n");

    let criteria = StoppingCriteria {
        min_trades: 50,
        max_duration: Duration::from_secs(3600),
        min_significance: 0.95,
        stop_on_harm: true,
        harm_threshold: -15.0,
    };

    let monitor = ExperimentMonitor::new(criteria);

    // Симуляция торговли
    for i in 0..300 {
        if !monitor.is_running() {
            println!("Experiment stopped at iteration {}", i);
            break;
        }

        // Контроль: стабильные результаты
        monitor.record_control(if i % 3 == 0 { -15.0 } else { 10.0 });

        // Эксперимент: лучшие результаты
        monitor.record_experiment(if i % 4 == 0 { -12.0 } else { 14.0 });

        // Проверяем каждые 50 сделок
        if i % 50 == 0 && i > 0 {
            println!("\n--- Check at {} trades ---", i);
            println!("{}", monitor.get_status());
        }
    }

    println!("\n=== Final Status ===");
    println!("{}", monitor.get_status());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **A/B тестирование** | Метод сравнения двух вариантов стратегии |
| **Контроль/Эксперимент** | Базовый вариант vs тестируемый вариант |
| **Распределение трафика** | Разделение сделок между вариантами |
| **Z-score** | Статистическая мера разницы между группами |
| **P-value** | Вероятность случайного получения результата |
| **Статистическая значимость** | Уверенность в том, что разница не случайна |
| **Доверительный интервал** | Диапазон вероятных значений истинной разницы |
| **Критерии остановки** | Условия для завершения эксперимента |

## Практические задания

1. **Базовый A/B тест**: Создай систему для сравнения двух стратегий:
   - Реализуй распределение трафика 50/50
   - Собирай метрики: win rate, P&L, средняя задержка
   - Генерируй отчёт сравнения

2. **Статистический анализ**: Расширь калькулятор:
   - Добавь t-тест для сравнения средних
   - Реализуй расчёт эффекта размера (effect size)
   - Добавь визуализацию доверительных интервалов

3. **Многовариантное тестирование**: Реализуй поддержку A/B/C/D:
   - Распределение трафика между 4+ вариантами
   - Корректировка для множественных сравнений
   - Иерархическое тестирование

4. **Автоматизация**: Создай полностью автоматическую систему:
   - Автоматический запуск экспериментов
   - Мониторинг в реальном времени
   - Автоматическое завершение и выбор победителя
   - Уведомления о результатах

## Домашнее задание

1. **Production A/B система**: Разработай полную систему:
   - Веб-интерфейс для создания экспериментов
   - REST API для записи событий
   - Дашборд с визуализацией результатов
   - Интеграция с Prometheus/Grafana

2. **Байесовское A/B тестирование**: Реализуй альтернативный подход:
   - Байесовская оценка вероятности победы
   - Динамическое распределение трафика (Multi-Armed Bandit)
   - Сравнение с классическим подходом

3. **A/B для высокочастотного трейдинга**: Оптимизируй для HFT:
   - Минимальные накладные расходы
   - Lock-free структуры данных
   - Микросекундные измерения
   - Streaming статистика

4. **Мульти-биржевой A/B тест**: Создай систему для:
   - Тестирование на разных биржах одновременно
   - Учёт специфики каждой биржи
   - Агрегация результатов
   - Кросс-биржевой анализ

5. **A/B с машинным обучением**: Интегрируй ML:
   - Автоматический выбор параметров эксперимента
   - Предсказание необходимого размера выборки
   - Раннее обнаружение победителя
   - Адаптивное распределение трафика

## Навигация

[← Предыдущий день](../360-canary-deployments/ru.md) | [Следующий день →](../362-post-mortem-incident-analysis/ru.md)
