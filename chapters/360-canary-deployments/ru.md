# День 360: Канареечные деплои (Canary Deployments)

## Аналогия из трейдинга

Представь, что ты управляющий хедж-фондом с новым торговым алгоритмом. Ты уверен, что он хорошо работает на бэктестах, но развернуть его сразу для управления всем портфелем в $100 миллионов — страшно. А что если в нём баг, который приведёт к катастрофическим убыткам?

**Канареечный подход в трейдинге:**
Как шахтёры использовали канареек для обнаружения опасных газов, ты можешь использовать небольшую часть капитала, чтобы «проверить воздух» перед полным развёртыванием:

1. **Начни с малого**: Разверни новый алгоритм только с 1% капитала ($1М)
2. **Внимательно мониторь**: Следи за ключевыми метриками — P&L, проскальзывание, качество исполнения
3. **Постепенное увеличение**: Если метрики хорошие, увеличь до 5%, потом до 10%, затем до 25%
4. **Быстрый откат**: Если что-то пошло не так, мгновенно возвращайся к старому алгоритму

| Фаза | Доля капитала | Уровень риска | Действие при проблемах |
|------|---------------|---------------|------------------------|
| **Канарейка** | 1-5% | Минимальный | Мгновенный откат |
| **Ранние последователи** | 10-25% | Низкий | Быстрый откат |
| **Большинство** | 50-75% | Средний | Контролируемый откат |
| **Полный раскат** | 100% | Полный | Старая версия удалена |

**Почему «Канарейка»?**
В угольных шахтах канарейки были более чувствительны к токсичным газам, чем люди. Они начинали беспокоиться раньше, чем условия становились опасными для шахтёров. Аналогично, канареечный деплой направляет небольшую часть трафика на новую версию первой — если есть проблемы, они затронут только небольшое подмножество пользователей/капитала.

## Основы канареечного деплоя

### Паттерн канареечного развёртывания

```rust
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Состояние канареечного деплоя
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanaryState {
    /// Нет активной канарейки, весь трафик на стабильную версию
    Stable,
    /// Канарейка работает с указанным процентом
    Active { percentage: u8 },
    /// Канарейка продвинута в стабильную версию
    Promoted,
    /// Канарейка откачена из-за проблем
    RolledBack,
}

/// Метрики, собираемые во время канареечного деплоя
#[derive(Debug, Default)]
pub struct CanaryMetrics {
    // Метрики стабильной версии
    pub stable_requests: AtomicU64,
    pub stable_errors: AtomicU64,
    pub stable_latency_sum_ms: AtomicU64,

    // Метрики канареечной версии
    pub canary_requests: AtomicU64,
    pub canary_errors: AtomicU64,
    pub canary_latency_sum_ms: AtomicU64,
}

impl CanaryMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_stable(&self, success: bool, latency_ms: u64) {
        self.stable_requests.fetch_add(1, Ordering::Relaxed);
        self.stable_latency_sum_ms.fetch_add(latency_ms, Ordering::Relaxed);
        if !success {
            self.stable_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_canary(&self, success: bool, latency_ms: u64) {
        self.canary_requests.fetch_add(1, Ordering::Relaxed);
        self.canary_latency_sum_ms.fetch_add(latency_ms, Ordering::Relaxed);
        if !success {
            self.canary_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn stable_error_rate(&self) -> f64 {
        let requests = self.stable_requests.load(Ordering::Relaxed);
        if requests == 0 { return 0.0; }
        let errors = self.stable_errors.load(Ordering::Relaxed);
        (errors as f64 / requests as f64) * 100.0
    }

    pub fn canary_error_rate(&self) -> f64 {
        let requests = self.canary_requests.load(Ordering::Relaxed);
        if requests == 0 { return 0.0; }
        let errors = self.canary_errors.load(Ordering::Relaxed);
        (errors as f64 / requests as f64) * 100.0
    }

    pub fn stable_avg_latency(&self) -> f64 {
        let requests = self.stable_requests.load(Ordering::Relaxed);
        if requests == 0 { return 0.0; }
        let sum = self.stable_latency_sum_ms.load(Ordering::Relaxed);
        sum as f64 / requests as f64
    }

    pub fn canary_avg_latency(&self) -> f64 {
        let requests = self.canary_requests.load(Ordering::Relaxed);
        if requests == 0 { return 0.0; }
        let sum = self.canary_latency_sum_ms.load(Ordering::Relaxed);
        sum as f64 / requests as f64
    }

    pub fn reset(&self) {
        self.stable_requests.store(0, Ordering::Relaxed);
        self.stable_errors.store(0, Ordering::Relaxed);
        self.stable_latency_sum_ms.store(0, Ordering::Relaxed);
        self.canary_requests.store(0, Ordering::Relaxed);
        self.canary_errors.store(0, Ordering::Relaxed);
        self.canary_latency_sum_ms.store(0, Ordering::Relaxed);
    }
}

/// Контроллер канареечного деплоя
pub struct CanaryDeployment {
    stable_version: String,
    canary_version: Option<String>,
    canary_percentage: AtomicU8,
    metrics: Arc<CanaryMetrics>,
    started_at: Option<Instant>,
}

impl CanaryDeployment {
    pub fn new(stable_version: String) -> Self {
        CanaryDeployment {
            stable_version,
            canary_version: None,
            canary_percentage: AtomicU8::new(0),
            metrics: Arc::new(CanaryMetrics::new()),
            started_at: None,
        }
    }

    /// Запустить канареечный деплой с начальным процентом
    pub fn start_canary(&mut self, version: String, initial_percentage: u8) {
        println!("Запуск канареечного деплоя:");
        println!("  Стабильная версия: {}", self.stable_version);
        println!("  Канареечная версия: {}", version);
        println!("  Начальный трафик: {}%", initial_percentage);

        self.canary_version = Some(version);
        self.canary_percentage.store(initial_percentage.min(100), Ordering::SeqCst);
        self.metrics.reset();
        self.started_at = Some(Instant::now());
    }

    /// Направить запрос на стабильную или канареечную версию
    pub fn route_request(&self, request_id: u64) -> &str {
        let percentage = self.canary_percentage.load(Ordering::Relaxed);

        if percentage == 0 || self.canary_version.is_none() {
            return &self.stable_version;
        }

        // Используем request_id для детерминированной маршрутизации
        let bucket = (request_id % 100) as u8;

        if bucket < percentage {
            self.canary_version.as_ref().unwrap()
        } else {
            &self.stable_version
        }
    }

    /// Проверить, должен ли запрос идти на канарейку
    pub fn is_canary_request(&self, request_id: u64) -> bool {
        let percentage = self.canary_percentage.load(Ordering::Relaxed);
        if percentage == 0 { return false; }
        (request_id % 100) as u8 < percentage
    }

    /// Увеличить процент канареечного трафика
    pub fn increase_traffic(&self, new_percentage: u8) -> u8 {
        let old = self.canary_percentage.load(Ordering::Relaxed);
        let new = new_percentage.min(100);
        self.canary_percentage.store(new, Ordering::SeqCst);
        println!("Канареечный трафик увеличен: {}% -> {}%", old, new);
        new
    }

    /// Продвинуть канарейку в стабильную версию (100% трафика)
    pub fn promote(&mut self) -> Result<String, &'static str> {
        match self.canary_version.take() {
            Some(version) => {
                println!("Продвижение канарейки {} в стабильную версию", version);
                self.stable_version = version.clone();
                self.canary_percentage.store(0, Ordering::SeqCst);
                self.started_at = None;
                Ok(version)
            }
            None => Err("Нет канареечной версии для продвижения"),
        }
    }

    /// Откатить канарейку (0% трафика, удалить канарейку)
    pub fn rollback(&mut self) -> Option<String> {
        if let Some(version) = self.canary_version.take() {
            println!("Откат канареечной версии: {}", version);
            self.canary_percentage.store(0, Ordering::SeqCst);
            self.started_at = None;
            Some(version)
        } else {
            None
        }
    }

    /// Получить текущий статус деплоя
    pub fn status(&self) -> CanaryStatus {
        let percentage = self.canary_percentage.load(Ordering::Relaxed);
        let duration = self.started_at.map(|s| s.elapsed());

        CanaryStatus {
            stable_version: self.stable_version.clone(),
            canary_version: self.canary_version.clone(),
            canary_percentage: percentage,
            duration,
            metrics: Arc::clone(&self.metrics),
        }
    }
}

#[derive(Debug)]
pub struct CanaryStatus {
    pub stable_version: String,
    pub canary_version: Option<String>,
    pub canary_percentage: u8,
    pub duration: Option<Duration>,
    pub metrics: Arc<CanaryMetrics>,
}

impl CanaryStatus {
    pub fn print_report(&self) {
        println!("\n=== Статус канареечного деплоя ===");
        println!("Стабильная версия: {}", self.stable_version);

        if let Some(ref canary) = self.canary_version {
            println!("Канареечная версия: {} ({}% трафика)", canary, self.canary_percentage);

            if let Some(duration) = self.duration {
                println!("Работает: {:.1} минут", duration.as_secs_f64() / 60.0);
            }

            println!("\nСравнение метрик:");
            println!("  Стабильная - Запросы: {}, Ошибки: {:.2}%, Средняя задержка: {:.1}мс",
                self.metrics.stable_requests.load(Ordering::Relaxed),
                self.metrics.stable_error_rate(),
                self.metrics.stable_avg_latency());
            println!("  Канарейка  - Запросы: {}, Ошибки: {:.2}%, Средняя задержка: {:.1}мс",
                self.metrics.canary_requests.load(Ordering::Relaxed),
                self.metrics.canary_error_rate(),
                self.metrics.canary_avg_latency());
        } else {
            println!("Нет активного канареечного деплоя");
        }
    }
}

fn main() {
    println!("=== Основы канареечного деплоя ===\n");

    let mut deployment = CanaryDeployment::new("v1.0.0".to_string());

    // Запуск канарейки с 10% трафика
    deployment.start_canary("v1.1.0".to_string(), 10);

    // Симуляция запросов
    println!("\nСимуляция 1000 запросов...");
    for i in 0..1000 {
        let is_canary = deployment.is_canary_request(i);
        let version = deployment.route_request(i);

        // Симуляция успеха/неудачи (канарейка немного лучше в этом примере)
        let success = if is_canary {
            i % 50 != 0  // 98% успехов
        } else {
            i % 25 != 0  // 96% успехов
        };

        let latency = if is_canary { 45 } else { 50 };

        if is_canary {
            deployment.status().metrics.record_canary(success, latency);
        } else {
            deployment.status().metrics.record_stable(success, latency);
        }
    }

    deployment.status().print_report();

    // Увеличение трафика
    println!("\n--- Увеличение канареечного трафика ---");
    deployment.increase_traffic(25);

    // Ещё запросы
    for i in 1000..2000 {
        let is_canary = deployment.is_canary_request(i);
        let success = if is_canary { i % 50 != 0 } else { i % 25 != 0 };
        let latency = if is_canary { 45 } else { 50 };

        if is_canary {
            deployment.status().metrics.record_canary(success, latency);
        } else {
            deployment.status().metrics.record_stable(success, latency);
        }
    }

    deployment.status().print_report();

    // Продвижение канарейки
    println!("\n--- Продвижение канарейки ---");
    deployment.promote().unwrap();

    deployment.status().print_report();
}
```

## Автоматический анализ канарейки для торговых систем

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Торговые метрики канарейки
#[derive(Debug)]
pub struct TradingCanaryMetrics {
    // Метрики исполнения ордеров
    pub stable_orders: AtomicU64,
    pub canary_orders: AtomicU64,
    pub stable_fills: AtomicU64,
    pub canary_fills: AtomicU64,
    pub stable_rejects: AtomicU64,
    pub canary_rejects: AtomicU64,

    // Отслеживание P&L (хранится в центах, смещение 1B для отрицательных)
    pub stable_pnl_cents: AtomicU64,
    pub canary_pnl_cents: AtomicU64,

    // Задержка (микросекунды)
    pub stable_latency_sum_us: AtomicU64,
    pub canary_latency_sum_us: AtomicU64,

    // Проскальзывание (базисные пункты * 100)
    pub stable_slippage_sum: AtomicU64,
    pub canary_slippage_sum: AtomicU64,
}

impl TradingCanaryMetrics {
    const PNL_OFFSET: i64 = 1_000_000_000_00; // Смещение $1B в центах

    pub fn new() -> Self {
        TradingCanaryMetrics {
            stable_orders: AtomicU64::new(0),
            canary_orders: AtomicU64::new(0),
            stable_fills: AtomicU64::new(0),
            canary_fills: AtomicU64::new(0),
            stable_rejects: AtomicU64::new(0),
            canary_rejects: AtomicU64::new(0),
            stable_pnl_cents: AtomicU64::new(Self::PNL_OFFSET as u64),
            canary_pnl_cents: AtomicU64::new(Self::PNL_OFFSET as u64),
            stable_latency_sum_us: AtomicU64::new(0),
            canary_latency_sum_us: AtomicU64::new(0),
            stable_slippage_sum: AtomicU64::new(0),
            canary_slippage_sum: AtomicU64::new(0),
        }
    }

    pub fn record_stable_order(&self, filled: bool, pnl_cents: i64, latency_us: u64, slippage_bp: f64) {
        self.stable_orders.fetch_add(1, Ordering::Relaxed);
        if filled {
            self.stable_fills.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stable_rejects.fetch_add(1, Ordering::Relaxed);
        }

        // Обновление P&L
        if pnl_cents >= 0 {
            self.stable_pnl_cents.fetch_add(pnl_cents as u64, Ordering::Relaxed);
        } else {
            self.stable_pnl_cents.fetch_sub((-pnl_cents) as u64, Ordering::Relaxed);
        }

        self.stable_latency_sum_us.fetch_add(latency_us, Ordering::Relaxed);
        self.stable_slippage_sum.fetch_add((slippage_bp * 100.0) as u64, Ordering::Relaxed);
    }

    pub fn record_canary_order(&self, filled: bool, pnl_cents: i64, latency_us: u64, slippage_bp: f64) {
        self.canary_orders.fetch_add(1, Ordering::Relaxed);
        if filled {
            self.canary_fills.fetch_add(1, Ordering::Relaxed);
        } else {
            self.canary_rejects.fetch_add(1, Ordering::Relaxed);
        }

        if pnl_cents >= 0 {
            self.canary_pnl_cents.fetch_add(pnl_cents as u64, Ordering::Relaxed);
        } else {
            self.canary_pnl_cents.fetch_sub((-pnl_cents) as u64, Ordering::Relaxed);
        }

        self.canary_latency_sum_us.fetch_add(latency_us, Ordering::Relaxed);
        self.canary_slippage_sum.fetch_add((slippage_bp * 100.0) as u64, Ordering::Relaxed);
    }

    pub fn stable_fill_rate(&self) -> f64 {
        let orders = self.stable_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let fills = self.stable_fills.load(Ordering::Relaxed);
        (fills as f64 / orders as f64) * 100.0
    }

    pub fn canary_fill_rate(&self) -> f64 {
        let orders = self.canary_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let fills = self.canary_fills.load(Ordering::Relaxed);
        (fills as f64 / orders as f64) * 100.0
    }

    pub fn stable_pnl(&self) -> f64 {
        let cents = self.stable_pnl_cents.load(Ordering::Relaxed) as i64;
        (cents - Self::PNL_OFFSET) as f64 / 100.0
    }

    pub fn canary_pnl(&self) -> f64 {
        let cents = self.canary_pnl_cents.load(Ordering::Relaxed) as i64;
        (cents - Self::PNL_OFFSET) as f64 / 100.0
    }

    pub fn stable_avg_latency_ms(&self) -> f64 {
        let orders = self.stable_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let sum = self.stable_latency_sum_us.load(Ordering::Relaxed);
        (sum as f64 / orders as f64) / 1000.0
    }

    pub fn canary_avg_latency_ms(&self) -> f64 {
        let orders = self.canary_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let sum = self.canary_latency_sum_us.load(Ordering::Relaxed);
        (sum as f64 / orders as f64) / 1000.0
    }

    pub fn stable_avg_slippage_bp(&self) -> f64 {
        let orders = self.stable_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let sum = self.stable_slippage_sum.load(Ordering::Relaxed);
        (sum as f64 / orders as f64) / 100.0
    }

    pub fn canary_avg_slippage_bp(&self) -> f64 {
        let orders = self.canary_orders.load(Ordering::Relaxed);
        if orders == 0 { return 0.0; }
        let sum = self.canary_slippage_sum.load(Ordering::Relaxed);
        (sum as f64 / orders as f64) / 100.0
    }
}

/// Решение по канарейке
#[derive(Debug, Clone, PartialEq)]
pub enum CanaryDecision {
    /// Продолжить мониторинг, недостаточно данных
    Continue,
    /// Увеличить процент трафика
    IncreaseTraffic(u8),
    /// Продвинуть канарейку в стабильную версию
    Promote,
    /// Немедленный откат канарейки
    Rollback(String),
}

/// Пороги для автоматических решений по канарейке
#[derive(Debug, Clone)]
pub struct CanaryThresholds {
    /// Минимум запросов перед принятием решений
    pub min_requests: u64,
    /// Максимально допустимое увеличение процента ошибок
    pub max_error_rate_increase: f64,
    /// Максимально допустимое увеличение задержки (проценты)
    pub max_latency_increase_pct: f64,
    /// Максимально допустимое увеличение проскальзывания (базисные пункты)
    pub max_slippage_increase_bp: f64,
    /// Минимальный P&L на ордер по сравнению со стабильной (как соотношение)
    pub min_pnl_ratio: f64,
    /// Шаги увеличения трафика
    pub traffic_steps: Vec<u8>,
}

impl Default for CanaryThresholds {
    fn default() -> Self {
        CanaryThresholds {
            min_requests: 100,
            max_error_rate_increase: 1.0,   // 1 процентный пункт
            max_latency_increase_pct: 20.0, // 20% медленнее допустимо
            max_slippage_increase_bp: 0.5,  // 0.5 базисных пункта
            min_pnl_ratio: 0.8,             // P&L канарейки минимум 80% от стабильной
            traffic_steps: vec![5, 10, 25, 50, 75, 100],
        }
    }
}

/// Автоматический анализатор канарейки для торговых систем
pub struct TradingCanaryAnalyzer {
    metrics: Arc<TradingCanaryMetrics>,
    thresholds: CanaryThresholds,
    current_percentage: u8,
    is_halted: AtomicBool,
}

impl TradingCanaryAnalyzer {
    pub fn new(thresholds: CanaryThresholds) -> Self {
        TradingCanaryAnalyzer {
            metrics: Arc::new(TradingCanaryMetrics::new()),
            thresholds,
            current_percentage: 0,
            is_halted: AtomicBool::new(false),
        }
    }

    pub fn metrics(&self) -> Arc<TradingCanaryMetrics> {
        Arc::clone(&self.metrics)
    }

    pub fn set_percentage(&mut self, pct: u8) {
        self.current_percentage = pct;
    }

    /// Анализировать текущие метрики и вернуть решение
    pub fn analyze(&self) -> CanaryDecision {
        // Проверка на приостановку
        if self.is_halted.load(Ordering::Relaxed) {
            return CanaryDecision::Continue;
        }

        let canary_requests = self.metrics.canary_orders.load(Ordering::Relaxed);
        let stable_requests = self.metrics.stable_orders.load(Ordering::Relaxed);

        // Ещё недостаточно данных
        if canary_requests < self.thresholds.min_requests {
            return CanaryDecision::Continue;
        }

        // Проверка процента отклонений
        let stable_fill_rate = self.metrics.stable_fill_rate();
        let canary_fill_rate = self.metrics.canary_fill_rate();
        let fill_rate_diff = stable_fill_rate - canary_fill_rate;

        if fill_rate_diff > self.thresholds.max_error_rate_increase {
            return CanaryDecision::Rollback(format!(
                "Процент исполнения ухудшился: стабильная {:.1}% vs канарейка {:.1}%",
                stable_fill_rate, canary_fill_rate
            ));
        }

        // Проверка задержки
        let stable_latency = self.metrics.stable_avg_latency_ms();
        let canary_latency = self.metrics.canary_avg_latency_ms();

        if stable_latency > 0.0 {
            let latency_increase = ((canary_latency - stable_latency) / stable_latency) * 100.0;
            if latency_increase > self.thresholds.max_latency_increase_pct {
                return CanaryDecision::Rollback(format!(
                    "Задержка выросла на {:.1}%: стабильная {:.2}мс vs канарейка {:.2}мс",
                    latency_increase, stable_latency, canary_latency
                ));
            }
        }

        // Проверка проскальзывания
        let stable_slippage = self.metrics.stable_avg_slippage_bp();
        let canary_slippage = self.metrics.canary_avg_slippage_bp();
        let slippage_diff = canary_slippage - stable_slippage;

        if slippage_diff > self.thresholds.max_slippage_increase_bp {
            return CanaryDecision::Rollback(format!(
                "Проскальзывание выросло: стабильная {:.2}bp vs канарейка {:.2}bp",
                stable_slippage, canary_slippage
            ));
        }

        // Проверка P&L (на ордер)
        if stable_requests > 0 && canary_requests > 0 {
            let stable_pnl_per_order = self.metrics.stable_pnl() / stable_requests as f64;
            let canary_pnl_per_order = self.metrics.canary_pnl() / canary_requests as f64;

            if stable_pnl_per_order > 0.0 {
                let pnl_ratio = canary_pnl_per_order / stable_pnl_per_order;
                if pnl_ratio < self.thresholds.min_pnl_ratio {
                    return CanaryDecision::Rollback(format!(
                        "P&L ухудшился: стабильная ${:.2}/ордер vs канарейка ${:.2}/ордер (соотношение: {:.2})",
                        stable_pnl_per_order, canary_pnl_per_order, pnl_ratio
                    ));
                }
            }
        }

        // Все проверки пройдены — определяем следующий шаг
        let next_step = self.thresholds.traffic_steps.iter()
            .find(|&&step| step > self.current_percentage);

        match next_step {
            Some(&100) => CanaryDecision::Promote,
            Some(&step) => CanaryDecision::IncreaseTraffic(step),
            None => CanaryDecision::Promote,
        }
    }

    /// Приостановить анализ канарейки (ручное вмешательство)
    pub fn halt(&self) {
        self.is_halted.store(true, Ordering::Relaxed);
        println!("Анализ канарейки ПРИОСТАНОВЛЕН — требуется ручное вмешательство");
    }

    /// Возобновить анализ канарейки
    pub fn resume(&self) {
        self.is_halted.store(false, Ordering::Relaxed);
        println!("Анализ канарейки ВОЗОБНОВЛЁН");
    }

    /// Сгенерировать детальный отчёт
    pub fn report(&self) -> String {
        let mut report = String::new();

        report.push_str("\n╔══════════════════════════════════════════════════╗\n");
        report.push_str("║    ОТЧЁТ КАНАРЕЕЧНОГО ДЕПЛОЯ ДЛЯ ТРЕЙДИНГА       ║\n");
        report.push_str("╠══════════════════════════════════════════════════╣\n");

        let stable_orders = self.metrics.stable_orders.load(Ordering::Relaxed);
        let canary_orders = self.metrics.canary_orders.load(Ordering::Relaxed);

        report.push_str(&format!("║ Разделение трафика: Стабильная {}% / Канарейка {}%\n",
            100 - self.current_percentage, self.current_percentage));
        report.push_str(&format!("║ Ордера: Стабильная {} / Канарейка {}          \n",
            stable_orders, canary_orders));

        report.push_str("╠══════════════════════════════════════════════════╣\n");
        report.push_str("║ МЕТРИКА         │ СТАБИЛЬНАЯ  │ КАНАРЕЙКА  │ РАЗН.\n");
        report.push_str("╠══════════════════════════════════════════════════╣\n");

        // Процент исполнения
        let stable_fill = self.metrics.stable_fill_rate();
        let canary_fill = self.metrics.canary_fill_rate();
        report.push_str(&format!("║ Исполнение      │ {:>6.1}%     │ {:>6.1}%    │ {:>+.1}%\n",
            stable_fill, canary_fill, canary_fill - stable_fill));

        // Задержка
        let stable_lat = self.metrics.stable_avg_latency_ms();
        let canary_lat = self.metrics.canary_avg_latency_ms();
        report.push_str(&format!("║ Задержка (мс)   │ {:>7.2}     │ {:>7.2}    │ {:>+.2}\n",
            stable_lat, canary_lat, canary_lat - stable_lat));

        // Проскальзывание
        let stable_slip = self.metrics.stable_avg_slippage_bp();
        let canary_slip = self.metrics.canary_avg_slippage_bp();
        report.push_str(&format!("║ Проскальз. (bp) │ {:>7.2}     │ {:>7.2}    │ {:>+.2}\n",
            stable_slip, canary_slip, canary_slip - stable_slip));

        // P&L
        let stable_pnl = self.metrics.stable_pnl();
        let canary_pnl = self.metrics.canary_pnl();
        report.push_str(&format!("║ Общий P&L       │ ${:>9.2}  │ ${:>9.2} │ ${:>+.2}\n",
            stable_pnl, canary_pnl, canary_pnl - stable_pnl));

        report.push_str("╚══════════════════════════════════════════════════╝\n");

        // Решение
        let decision = self.analyze();
        report.push_str(&format!("\nРешение: {:?}\n", decision));

        report
    }
}

fn main() {
    println!("=== Автоматический анализ канарейки для трейдинга ===\n");

    let thresholds = CanaryThresholds {
        min_requests: 50,
        ..Default::default()
    };

    let mut analyzer = TradingCanaryAnalyzer::new(thresholds);
    analyzer.set_percentage(10);

    let metrics = analyzer.metrics();

    // Симуляция торговой активности
    println!("Симуляция трейдинга с 10% канареечного трафика...\n");

    for i in 0..500 {
        let is_canary = (i % 10) == 0;  // ~10% на канарейку

        // Симуляция исполнения ордера
        let filled = if is_canary {
            i % 12 != 0  // 91.7% исполнение для канарейки
        } else {
            i % 11 != 0  // 90.9% исполнение для стабильной
        };

        // Симуляция P&L (канарейка немного лучше)
        let pnl = if is_canary {
            if filled { 150 } else { -50 }  // Лучший P&L
        } else {
            if filled { 120 } else { -60 }
        };

        // Симуляция задержки (канарейка быстрее)
        let latency_us = if is_canary { 45_000 } else { 52_000 };

        // Симуляция проскальзывания
        let slippage_bp = if is_canary { 0.8 } else { 1.2 };

        if is_canary {
            metrics.record_canary_order(filled, pnl, latency_us, slippage_bp);
        } else {
            metrics.record_stable_order(filled, pnl, latency_us, slippage_bp);
        }
    }

    // Вывод отчёта и получение решения
    println!("{}", analyzer.report());

    // Симуляция увеличения трафика
    match analyzer.analyze() {
        CanaryDecision::IncreaseTraffic(pct) => {
            println!("\n--- Увеличение канареечного трафика до {}% ---", pct);
            analyzer.set_percentage(pct);

            // Ещё торговля...
            for i in 500..1000 {
                let is_canary = (i % 4) == 0;  // ~25% на канарейку
                let filled = if is_canary { i % 12 != 0 } else { i % 11 != 0 };
                let pnl = if is_canary {
                    if filled { 150 } else { -50 }
                } else {
                    if filled { 120 } else { -60 }
                };
                let latency_us = if is_canary { 45_000 } else { 52_000 };
                let slippage_bp = if is_canary { 0.8 } else { 1.2 };

                if is_canary {
                    metrics.record_canary_order(filled, pnl, latency_us, slippage_bp);
                } else {
                    metrics.record_stable_order(filled, pnl, latency_us, slippage_bp);
                }
            }

            println!("{}", analyzer.report());
        }
        decision => {
            println!("Неожиданное решение: {:?}", decision);
        }
    }
}
```

## Стратегия постепенного раската

```rust
use std::time::{Duration, Instant};

/// Конфигурация фазы раската
#[derive(Debug, Clone)]
pub struct RolloutPhase {
    pub name: String,
    pub traffic_percentage: u8,
    pub min_duration: Duration,
    pub min_requests: u64,
    pub success_criteria: SuccessCriteria,
}

/// Критерии успеха для перехода к следующей фазе
#[derive(Debug, Clone)]
pub struct SuccessCriteria {
    pub max_error_rate: f64,
    pub max_latency_p99_ms: f64,
    pub min_throughput: f64,
}

impl Default for SuccessCriteria {
    fn default() -> Self {
        SuccessCriteria {
            max_error_rate: 1.0,      // Максимум 1% ошибок
            max_latency_p99_ms: 100.0, // P99 задержка 100мс
            min_throughput: 100.0,     // Минимум 100 запросов/секунду
        }
    }
}

/// Менеджер постепенного раската для торговых систем
pub struct GradualRollout {
    phases: Vec<RolloutPhase>,
    current_phase: usize,
    phase_started: Option<Instant>,
    phase_requests: u64,
    phase_errors: u64,
    latencies: Vec<f64>,
}

impl GradualRollout {
    /// Создать стандартный раскат для торговой системы
    pub fn trading_standard() -> Self {
        let criteria = SuccessCriteria::default();

        GradualRollout {
            phases: vec![
                RolloutPhase {
                    name: "Канарейка".to_string(),
                    traffic_percentage: 1,
                    min_duration: Duration::from_secs(300),  // 5 минут
                    min_requests: 100,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Ранние последователи".to_string(),
                    traffic_percentage: 5,
                    min_duration: Duration::from_secs(600),  // 10 минут
                    min_requests: 500,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Ограниченный".to_string(),
                    traffic_percentage: 10,
                    min_duration: Duration::from_secs(900),  // 15 минут
                    min_requests: 1000,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Расширение".to_string(),
                    traffic_percentage: 25,
                    min_duration: Duration::from_secs(1200), // 20 минут
                    min_requests: 2500,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Большинство".to_string(),
                    traffic_percentage: 50,
                    min_duration: Duration::from_secs(1800), // 30 минут
                    min_requests: 5000,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Финальный".to_string(),
                    traffic_percentage: 100,
                    min_duration: Duration::from_secs(0),
                    min_requests: 0,
                    success_criteria: criteria,
                },
            ],
            current_phase: 0,
            phase_started: None,
            phase_requests: 0,
            phase_errors: 0,
            latencies: Vec::new(),
        }
    }

    /// Создать агрессивный раскат для низкорисковых изменений
    pub fn aggressive() -> Self {
        let criteria = SuccessCriteria {
            max_error_rate: 2.0,
            max_latency_p99_ms: 200.0,
            min_throughput: 50.0,
        };

        GradualRollout {
            phases: vec![
                RolloutPhase {
                    name: "Быстрый тест".to_string(),
                    traffic_percentage: 5,
                    min_duration: Duration::from_secs(60),
                    min_requests: 50,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Наращивание".to_string(),
                    traffic_percentage: 25,
                    min_duration: Duration::from_secs(120),
                    min_requests: 200,
                    success_criteria: criteria.clone(),
                },
                RolloutPhase {
                    name: "Полный".to_string(),
                    traffic_percentage: 100,
                    min_duration: Duration::from_secs(0),
                    min_requests: 0,
                    success_criteria: criteria,
                },
            ],
            current_phase: 0,
            phase_started: None,
            phase_requests: 0,
            phase_errors: 0,
            latencies: Vec::new(),
        }
    }

    /// Запустить раскат
    pub fn start(&mut self) {
        self.phase_started = Some(Instant::now());
        self.phase_requests = 0;
        self.phase_errors = 0;
        self.latencies.clear();

        let phase = &self.phases[self.current_phase];
        println!("Запуск фазы раската '{}' на {}% трафика",
            phase.name, phase.traffic_percentage);
    }

    /// Записать результат запроса
    pub fn record_request(&mut self, success: bool, latency_ms: f64) {
        self.phase_requests += 1;
        if !success {
            self.phase_errors += 1;
        }
        self.latencies.push(latency_ms);
    }

    /// Получить текущий процент трафика
    pub fn current_percentage(&self) -> u8 {
        self.phases[self.current_phase].traffic_percentage
    }

    /// Проверить готовность к переходу на следующую фазу
    pub fn check_advance(&mut self) -> RolloutAction {
        let phase = &self.phases[self.current_phase];

        // Проверка минимальной продолжительности
        if let Some(started) = self.phase_started {
            if started.elapsed() < phase.min_duration {
                return RolloutAction::Wait(format!(
                    "Ожидание минимальной продолжительности: {:.0}с осталось",
                    (phase.min_duration - started.elapsed()).as_secs_f64()
                ));
            }
        } else {
            return RolloutAction::NotStarted;
        }

        // Проверка минимального количества запросов
        if self.phase_requests < phase.min_requests {
            return RolloutAction::Wait(format!(
                "Ожидание минимального количества запросов: {} / {}",
                self.phase_requests, phase.min_requests
            ));
        }

        // Проверка критериев успеха
        let error_rate = (self.phase_errors as f64 / self.phase_requests as f64) * 100.0;
        if error_rate > phase.success_criteria.max_error_rate {
            return RolloutAction::Rollback(format!(
                "Процент ошибок {:.2}% превышает порог {:.2}%",
                error_rate, phase.success_criteria.max_error_rate
            ));
        }

        // Проверка p99 задержки
        if !self.latencies.is_empty() {
            let mut sorted = self.latencies.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p99_idx = (sorted.len() as f64 * 0.99) as usize;
            let p99 = sorted.get(p99_idx.min(sorted.len() - 1)).copied().unwrap_or(0.0);

            if p99 > phase.success_criteria.max_latency_p99_ms {
                return RolloutAction::Rollback(format!(
                    "P99 задержка {:.2}мс превышает порог {:.2}мс",
                    p99, phase.success_criteria.max_latency_p99_ms
                ));
            }
        }

        // Все критерии выполнены — переход к следующей фазе
        if self.current_phase < self.phases.len() - 1 {
            self.current_phase += 1;
            self.phase_started = Some(Instant::now());
            self.phase_requests = 0;
            self.phase_errors = 0;
            self.latencies.clear();

            let next_phase = &self.phases[self.current_phase];
            RolloutAction::Advance(next_phase.traffic_percentage)
        } else {
            RolloutAction::Complete
        }
    }

    /// Получить название текущей фазы
    pub fn current_phase_name(&self) -> &str {
        &self.phases[self.current_phase].name
    }

    /// Вывести статус раската
    pub fn print_status(&self) {
        let phase = &self.phases[self.current_phase];

        println!("\n=== Статус раската ===");
        println!("Фаза: {} ({}/{})", phase.name, self.current_phase + 1, self.phases.len());
        println!("Трафик: {}%", phase.traffic_percentage);
        println!("Запросы: {} / {} требуется", self.phase_requests, phase.min_requests);

        if self.phase_requests > 0 {
            let error_rate = (self.phase_errors as f64 / self.phase_requests as f64) * 100.0;
            println!("Процент ошибок: {:.2}% (макс: {:.2}%)", error_rate, phase.success_criteria.max_error_rate);
        }

        if let Some(started) = self.phase_started {
            let elapsed = started.elapsed();
            let remaining = phase.min_duration.saturating_sub(elapsed);
            println!("Продолжительность: {:.0}с прошло, {:.0}с осталось",
                elapsed.as_secs_f64(), remaining.as_secs_f64());
        }
    }
}

#[derive(Debug)]
pub enum RolloutAction {
    NotStarted,
    Wait(String),
    Advance(u8),
    Rollback(String),
    Complete,
}

fn main() {
    println!("=== Стратегия постепенного раската ===\n");

    let mut rollout = GradualRollout::trading_standard();
    rollout.start();

    // Симуляция прохождения фаз
    for phase_num in 0..4 {
        println!("\n--- Симуляция фазы {} ---", phase_num + 1);
        rollout.print_status();

        // Симуляция запросов для этой фазы
        let requests_needed = 150 + (phase_num * 200) as u64;
        for i in 0..requests_needed {
            let success = i % 100 != 0;  // 99% успехов
            let latency = 30.0 + (i % 40) as f64;  // 30-70мс задержка
            rollout.record_request(success, latency);
        }

        // Проверка возможности перехода
        match rollout.check_advance() {
            RolloutAction::Advance(pct) => {
                println!("\nПереход на {}% трафика", pct);
            }
            RolloutAction::Wait(reason) => {
                println!("\nОжидание: {}", reason);
                // В реальной реализации здесь было бы ожидание и повторная попытка
            }
            RolloutAction::Rollback(reason) => {
                println!("\nОТКАТ: {}", reason);
                break;
            }
            RolloutAction::Complete => {
                println!("\nРаскат ЗАВЕРШЁН!");
                break;
            }
            RolloutAction::NotStarted => {
                println!("\nРаскат не запущен");
            }
        }
    }

    rollout.print_status();
}
```

## Интеграция с архитектурой торгового бота

```rust
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::collections::HashMap;

/// Идентификатор версии торговой стратегии
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StrategyVersion {
    pub name: String,
    pub version: String,
    pub hash: String,
}

impl StrategyVersion {
    pub fn new(name: &str, version: &str) -> Self {
        let hash = format!("{:x}", version.as_bytes().iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64)));
        StrategyVersion {
            name: name.to_string(),
            version: version.to_string(),
            hash,
        }
    }

    pub fn full_id(&self) -> String {
        format!("{}-{}-{}", self.name, self.version, &self.hash[..8])
    }
}

/// Интерфейс торговой стратегии
pub trait TradingStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &StrategyVersion;
    fn generate_signal(&self, symbol: &str, price: f64, volume: f64) -> Signal;
    fn calculate_position_size(&self, signal: &Signal, balance: f64) -> f64;
}

#[derive(Debug, Clone)]
pub enum Signal {
    Buy { strength: f64, target: f64, stop_loss: f64 },
    Sell { strength: f64, target: f64, stop_loss: f64 },
    Hold,
}

/// Стабильная версия моментум-стратегии (v1.0)
pub struct MomentumStrategyV1 {
    version: StrategyVersion,
}

impl MomentumStrategyV1 {
    pub fn new() -> Self {
        MomentumStrategyV1 {
            version: StrategyVersion::new("Momentum", "1.0.0"),
        }
    }
}

impl TradingStrategy for MomentumStrategyV1 {
    fn name(&self) -> &str { "Momentum" }
    fn version(&self) -> &StrategyVersion { &self.version }

    fn generate_signal(&self, _symbol: &str, price: f64, volume: f64) -> Signal {
        // Простая логика моментума
        let momentum = volume / 1000.0;
        if momentum > 1.5 {
            Signal::Buy {
                strength: (momentum - 1.0).min(1.0),
                target: price * 1.02,
                stop_loss: price * 0.98,
            }
        } else if momentum < 0.5 {
            Signal::Sell {
                strength: (1.0 - momentum).min(1.0),
                target: price * 0.98,
                stop_loss: price * 1.02,
            }
        } else {
            Signal::Hold
        }
    }

    fn calculate_position_size(&self, signal: &Signal, balance: f64) -> f64 {
        match signal {
            Signal::Buy { strength, .. } | Signal::Sell { strength, .. } => {
                balance * 0.01 * strength  // 1% баланса * сила сигнала
            }
            Signal::Hold => 0.0,
        }
    }
}

/// Канареечная версия с улучшенной логикой (v2.0)
pub struct MomentumStrategyV2 {
    version: StrategyVersion,
}

impl MomentumStrategyV2 {
    pub fn new() -> Self {
        MomentumStrategyV2 {
            version: StrategyVersion::new("Momentum", "2.0.0"),
        }
    }
}

impl TradingStrategy for MomentumStrategyV2 {
    fn name(&self) -> &str { "Momentum" }
    fn version(&self) -> &StrategyVersion { &self.version }

    fn generate_signal(&self, _symbol: &str, price: f64, volume: f64) -> Signal {
        // Улучшенная логика моментума с адаптивными порогами
        let momentum = volume / 1000.0;
        let volatility_factor = (price / 50000.0).min(2.0);  // Коррекция на уровень цены

        let buy_threshold = 1.3 * volatility_factor;
        let sell_threshold = 0.7 / volatility_factor;

        if momentum > buy_threshold {
            Signal::Buy {
                strength: ((momentum - buy_threshold) / buy_threshold).min(1.0),
                target: price * 1.025,  // Немного выше таргет
                stop_loss: price * 0.985,  // Более узкий стоп-лосс
            }
        } else if momentum < sell_threshold {
            Signal::Sell {
                strength: ((sell_threshold - momentum) / sell_threshold).min(1.0),
                target: price * 0.975,
                stop_loss: price * 1.015,
            }
        } else {
            Signal::Hold
        }
    }

    fn calculate_position_size(&self, signal: &Signal, balance: f64) -> f64 {
        match signal {
            Signal::Buy { strength, .. } | Signal::Sell { strength, .. } => {
                // Более агрессивный сайзинг в v2
                balance * 0.015 * strength  // 1.5% баланса * сила сигнала
            }
            Signal::Hold => 0.0,
        }
    }
}

/// Роутер стратегий с поддержкой канарейки
pub struct StrategyRouter {
    stable: Arc<dyn TradingStrategy>,
    canary: Option<Arc<dyn TradingStrategy>>,
    canary_percentage: AtomicU8,
    is_canary_enabled: AtomicBool,
}

impl StrategyRouter {
    pub fn new(stable: Arc<dyn TradingStrategy>) -> Self {
        println!("Инициализация роутера со стабильной стратегией: {}",
            stable.version().full_id());

        StrategyRouter {
            stable,
            canary: None,
            canary_percentage: AtomicU8::new(0),
            is_canary_enabled: AtomicBool::new(false),
        }
    }

    pub fn deploy_canary(&mut self, canary: Arc<dyn TradingStrategy>, percentage: u8) {
        println!("Развёртывание канареечной стратегии: {} на {}%",
            canary.version().full_id(), percentage);

        self.canary = Some(canary);
        self.canary_percentage.store(percentage.min(100), Ordering::SeqCst);
        self.is_canary_enabled.store(true, Ordering::SeqCst);
    }

    pub fn route(&self, account_id: u64) -> Arc<dyn TradingStrategy> {
        if !self.is_canary_enabled.load(Ordering::Relaxed) {
            return Arc::clone(&self.stable);
        }

        if let Some(ref canary) = self.canary {
            let percentage = self.canary_percentage.load(Ordering::Relaxed);
            let bucket = (account_id % 100) as u8;

            if bucket < percentage {
                return Arc::clone(canary);
            }
        }

        Arc::clone(&self.stable)
    }

    pub fn promote_canary(&mut self) -> Result<(), &'static str> {
        if let Some(canary) = self.canary.take() {
            println!("Продвижение канарейки {} в стабильную версию", canary.version().full_id());
            self.stable = canary;
            self.canary_percentage.store(0, Ordering::SeqCst);
            self.is_canary_enabled.store(false, Ordering::SeqCst);
            Ok(())
        } else {
            Err("Нет канарейки для продвижения")
        }
    }

    pub fn rollback_canary(&mut self) -> Option<String> {
        if let Some(canary) = self.canary.take() {
            let version_id = canary.version().full_id();
            println!("Откат канарейки: {}", version_id);
            self.canary_percentage.store(0, Ordering::SeqCst);
            self.is_canary_enabled.store(false, Ordering::SeqCst);
            Some(version_id)
        } else {
            None
        }
    }

    pub fn increase_canary_traffic(&self, new_percentage: u8) {
        let old = self.canary_percentage.load(Ordering::Relaxed);
        self.canary_percentage.store(new_percentage.min(100), Ordering::SeqCst);
        println!("Канареечный трафик: {}% -> {}%", old, new_percentage.min(100));
    }
}

/// Торговый бот с поддержкой канареечного деплоя
pub struct TradingBot {
    router: StrategyRouter,
    accounts: HashMap<u64, AccountState>,
}

#[derive(Debug)]
pub struct AccountState {
    pub balance: f64,
    pub positions: HashMap<String, f64>,
}

impl TradingBot {
    pub fn new(stable_strategy: Arc<dyn TradingStrategy>) -> Self {
        let router = StrategyRouter::new(stable_strategy);

        // Создание примерных аккаунтов
        let mut accounts = HashMap::new();
        for i in 0..10 {
            accounts.insert(i, AccountState {
                balance: 100_000.0,
                positions: HashMap::new(),
            });
        }

        TradingBot { router, accounts }
    }

    pub fn deploy_canary(&mut self, strategy: Arc<dyn TradingStrategy>, percentage: u8) {
        self.router.deploy_canary(strategy, percentage);
    }

    pub fn process_market_data(&self, symbol: &str, price: f64, volume: f64) {
        println!("\n--- Обработка {} @ ${:.2} (объём: {:.0}) ---", symbol, price, volume);

        for (&account_id, account) in &self.accounts {
            let strategy = self.router.route(account_id);
            let signal = strategy.generate_signal(symbol, price, volume);
            let position_size = strategy.calculate_position_size(&signal, account.balance);

            match signal {
                Signal::Buy { strength, target, stop_loss } if position_size > 0.0 => {
                    println!("Аккаунт {}: {} [{}] ПОКУПКА ${:.2} (сила: {:.2}, цель: {:.2}, стоп: {:.2})",
                        account_id, strategy.version().version, symbol,
                        position_size, strength, target, stop_loss);
                }
                Signal::Sell { strength, target, stop_loss } if position_size > 0.0 => {
                    println!("Аккаунт {}: {} [{}] ПРОДАЖА ${:.2} (сила: {:.2}, цель: {:.2}, стоп: {:.2})",
                        account_id, strategy.version().version, symbol,
                        position_size, strength, target, stop_loss);
                }
                _ => {
                    // Нет действия или hold
                }
            }
        }
    }

    pub fn increase_canary(&self, percentage: u8) {
        self.router.increase_canary_traffic(percentage);
    }

    pub fn promote(&mut self) -> Result<(), &'static str> {
        self.router.promote_canary()
    }

    pub fn rollback(&mut self) -> Option<String> {
        self.router.rollback_canary()
    }
}

fn main() {
    println!("=== Канареечный деплой торгового бота ===\n");

    // Создание торгового бота со стабильной стратегией
    let stable = Arc::new(MomentumStrategyV1::new());
    let mut bot = TradingBot::new(stable);

    // Обработка рыночных данных только со стабильной версией
    println!("=== Работа только со стабильной стратегией ===");
    bot.process_market_data("BTCUSDT", 50000.0, 2000.0);

    // Развёртывание канарейки
    println!("\n=== Развёртывание канареечной стратегии на 20% ===");
    let canary = Arc::new(MomentumStrategyV2::new());
    bot.deploy_canary(canary, 20);

    // Ещё рыночные данные
    bot.process_market_data("BTCUSDT", 50100.0, 2200.0);

    // Увеличение канареечного трафика
    println!("\n=== Увеличение канарейки до 50% ===");
    bot.increase_canary(50);

    bot.process_market_data("BTCUSDT", 50200.0, 1800.0);

    // Продвижение канарейки
    println!("\n=== Продвижение канарейки в стабильную версию ===");
    bot.promote().unwrap();

    bot.process_market_data("BTCUSDT", 50300.0, 2100.0);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Канареечный деплой** | Постепенное развёртывание новой версии на подмножество трафика |
| **Разделение трафика** | Маршрутизация запросов между стабильной и канареечной версиями |
| **Сравнение метрик** | Сравнение процента ошибок, задержек, P&L между версиями |
| **Автоматический анализ** | Принятие решений о продвижении/откате на основе метрик |
| **Постепенный раскат** | Пошаговое увеличение трафика с валидацией на каждом шаге |
| **Быстрый откат** | Мгновенный возврат к стабильной версии при проблемах |
| **Версионирование стратегий** | Идентификация и отслеживание разных версий стратегий |

## Практические задания

1. **Канарейка маршрутизации ордеров**: Реализуй систему, которая:
   - Направляет процент ордеров через новый алгоритм маршрутизации
   - Сравнивает качество исполнения (проскальзывание, исполнение, задержку)
   - Автоматически откатывается при ухудшении качества исполнения
   - Логирует все решения маршрутизации для аудита

2. **Канарейка риск-движка**: Создай канареечную систему для риск-менеджмента:
   - Тестирует новые расчёты риска на подмножестве аккаунтов
   - Сравнивает лимиты риска с продакшн-движком
   - Алертит о значительных различиях
   - Валидирует новые модели риска перед полным раскатом

3. **Мультибиржевая канарейка**: Построй систему деплоя, которая:
   - Тестирует новые коннекторы бирж с маленькими размерами ордеров
   - Мониторит стабильность соединения и задержку
   - Сравнивает исполнение ордеров между биржами
   - Поддерживает откат по отдельным биржам

4. **A/B со стратегией-канарейкой**: Реализуй гибридную систему:
   - Запускает A/B тесты внутри канареечного трафика
   - Отслеживает метрики производительности по вариантам
   - Продвигает выигрывающий вариант в стабильную версию
   - Предоставляет статистическую уверенность для решений

## Домашнее задание

1. **Полный канареечный пайплайн**: Построй систему end-to-end, которая:
   - Интегрируется с CI/CD для автоматических канареечных деплоев
   - Мониторит торговые метрики (P&L, коэффициент Шарпа, просадку)
   - Поддерживает запланированное увеличение трафика
   - Реализует автоматический откат при аномалиях
   - Отправляет алерты в Slack/PagerDuty при проблемах
   - Ведёт историю деплоев и аудит-логи

2. **Канарейка машинного обучения**: Создай систему для деплоя ML-моделей:
   - Сравнивает точность предсказаний между версиями
   - Мониторит дрифт признаков и деградацию модели
   - Поддерживает shadow-режим (предсказания без исполнения)
   - Валидирует задержку модели в продакшене
   - Реализует постепенное переключение трафика на основе производительности

3. **Глобальная оркестрация канарейки**: Спроектируй мультирегиональную систему:
   - Деплоит канарейку по географическим регионам
   - Мониторит регионально-специфические метрики
   - Поддерживает раскат по регионам
   - Обрабатывает торговые расписания с учётом часовых поясов
   - Координирует откат по всем регионам

4. **Интеграция с Chaos Engineering**: Объедини канарейку с хаос-тестированием:
   - Внедряет сбои во время канареечной фазы
   - Тестирует обработку ошибок и восстановление
   - Валидирует circuit breaker'ы и fallback'и
   - Измеряет устойчивость системы под нагрузкой
   - Генерирует отчёты о надёжности

## Навигация

[← Предыдущий день](../354-production-logging/ru.md) | [Следующий день →](../361-*/ru.md)
