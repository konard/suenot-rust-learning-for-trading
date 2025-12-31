# День 249: RSI: индекс относительной силы

## Аналогия из трейдинга

Представь, что ты наблюдаешь за футбольным матчем. Одна команда атакует снова и снова — они в состоянии "перекупленности" (overbought), устали и скоро могут потерять темп. Другая команда только защищается — они в состоянии "перепроданности" (oversold), но скоро могут перейти в контратаку.

**RSI (Relative Strength Index)** — это индикатор, который показывает, насколько "уставшим" является текущий тренд. Если цена актива слишком долго росла, RSI покажет высокое значение (выше 70) — сигнал о перекупленности. Если цена слишком долго падала, RSI покажет низкое значение (ниже 30) — сигнал о перепроданности.

В реальном трейдинге RSI помогает:
- Определить моменты разворота тренда
- Найти точки входа и выхода из позиции
- Оценить силу текущего движения цены

## Что такое RSI?

RSI (Relative Strength Index — индекс относительной силы) — это осциллятор импульса, измеряющий скорость и величину изменения цены. Он был разработан Дж. Уэллсом Уайлдером в 1978 году.

### Формула RSI

```
RSI = 100 - (100 / (1 + RS))

где RS = Средний прирост / Средняя потеря
```

Значение RSI всегда находится между 0 и 100:
- **RSI > 70** — актив перекуплен (возможен разворот вниз)
- **RSI < 30** — актив перепродан (возможен разворот вверх)
- **RSI ≈ 50** — нейтральная зона

## Простой расчёт RSI

Рассмотрим, как рассчитать RSI для последовательности цен закрытия:

```rust
/// Рассчитывает изменения цен между последовательными периодами
fn calculate_price_changes(prices: &[f64]) -> Vec<f64> {
    prices.windows(2)
        .map(|w| w[1] - w[0])
        .collect()
}

/// Разделяет изменения на приросты (gains) и потери (losses)
fn separate_gains_losses(changes: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let gains: Vec<f64> = changes.iter()
        .map(|&c| if c > 0.0 { c } else { 0.0 })
        .collect();

    let losses: Vec<f64> = changes.iter()
        .map(|&c| if c < 0.0 { c.abs() } else { 0.0 })
        .collect();

    (gains, losses)
}

/// Рассчитывает простое среднее
fn simple_average(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Рассчитывает RSI для заданного периода
fn calculate_rsi(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period + 1 {
        return None; // Недостаточно данных
    }

    let changes = calculate_price_changes(prices);
    let (gains, losses) = separate_gains_losses(&changes);

    // Берём последние `period` значений
    let recent_gains = &gains[gains.len().saturating_sub(period)..];
    let recent_losses = &losses[losses.len().saturating_sub(period)..];

    let avg_gain = simple_average(recent_gains);
    let avg_loss = simple_average(recent_losses);

    if avg_loss == 0.0 {
        return Some(100.0); // Нет потерь — RSI = 100
    }

    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - (100.0 / (1.0 + rs));

    Some(rsi)
}

fn main() {
    // Цены закрытия BTC за 15 дней
    let btc_prices = vec![
        42000.0, 42500.0, 42300.0, 42800.0, 43200.0,
        43100.0, 43500.0, 44000.0, 43800.0, 44200.0,
        44500.0, 44300.0, 44100.0, 44600.0, 45000.0,
    ];

    println!("Цены закрытия BTC:");
    for (i, price) in btc_prices.iter().enumerate() {
        println!("  День {}: ${:.2}", i + 1, price);
    }

    if let Some(rsi) = calculate_rsi(&btc_prices, 14) {
        println!("\nRSI (14 периодов): {:.2}", rsi);

        if rsi > 70.0 {
            println!("Сигнал: Перекупленность — возможен разворот вниз");
        } else if rsi < 30.0 {
            println!("Сигнал: Перепроданность — возможен разворот вверх");
        } else {
            println!("Сигнал: Нейтральная зона");
        }
    }
}
```

## Структура RSI-калькулятора

Создадим структуру для отслеживания RSI в реальном времени:

```rust
/// Калькулятор RSI с поддержкой потокового обновления
#[derive(Debug)]
struct RsiCalculator {
    period: usize,
    prices: Vec<f64>,
    avg_gain: f64,
    avg_loss: f64,
    initialized: bool,
}

impl RsiCalculator {
    /// Создаёт новый калькулятор RSI
    fn new(period: usize) -> Self {
        RsiCalculator {
            period,
            prices: Vec::new(),
            avg_gain: 0.0,
            avg_loss: 0.0,
            initialized: false,
        }
    }

    /// Добавляет новую цену и возвращает текущий RSI
    fn update(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);

        if self.prices.len() < self.period + 1 {
            return None; // Недостаточно данных
        }

        let change = price - self.prices[self.prices.len() - 2];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { change.abs() } else { 0.0 };

        if !self.initialized {
            // Первый расчёт — простое среднее
            let changes: Vec<f64> = self.prices.windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            let sum_gains: f64 = changes.iter()
                .map(|&c| if c > 0.0 { c } else { 0.0 })
                .sum();
            let sum_losses: f64 = changes.iter()
                .map(|&c| if c < 0.0 { c.abs() } else { 0.0 })
                .sum();

            self.avg_gain = sum_gains / self.period as f64;
            self.avg_loss = sum_losses / self.period as f64;
            self.initialized = true;
        } else {
            // Экспоненциальное сглаживание (метод Уайлдера)
            self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain)
                / self.period as f64;
            self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss)
                / self.period as f64;
        }

        self.calculate_rsi()
    }

    /// Рассчитывает текущий RSI
    fn calculate_rsi(&self) -> Option<f64> {
        if !self.initialized {
            return None;
        }

        if self.avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = self.avg_gain / self.avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    /// Возвращает интерпретацию текущего RSI
    fn interpret(&self) -> &'static str {
        match self.calculate_rsi() {
            Some(rsi) if rsi > 70.0 => "Перекупленность",
            Some(rsi) if rsi < 30.0 => "Перепроданность",
            Some(_) => "Нейтральная зона",
            None => "Недостаточно данных",
        }
    }
}

fn main() {
    let mut rsi_calc = RsiCalculator::new(14);

    // Симулируем поступление цен в реальном времени
    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42350.0,
        42300.0, 42450.0, 42600.0, 42550.0, 42700.0,
        42850.0, 43000.0, 43150.0, 43300.0, 43250.0,
        43400.0, 43550.0, 43700.0, 43650.0, 43800.0,
    ];

    println!("Потоковый расчёт RSI для BTC:\n");

    for (i, price) in prices.iter().enumerate() {
        if let Some(rsi) = rsi_calc.update(*price) {
            println!(
                "День {:2}: Цена ${:.2} | RSI: {:5.2} | {}",
                i + 1,
                price,
                rsi,
                rsi_calc.interpret()
            );
        } else {
            println!(
                "День {:2}: Цена ${:.2} | RSI: накопление данных...",
                i + 1,
                price
            );
        }
    }
}
```

## RSI с торговыми сигналами

Реализуем систему торговых сигналов на основе RSI:

```rust
/// Тип торгового сигнала
#[derive(Debug, Clone, PartialEq)]
enum Signal {
    StrongBuy,    // RSI < 20
    Buy,          // RSI < 30
    Hold,         // 30 <= RSI <= 70
    Sell,         // RSI > 70
    StrongSell,   // RSI > 80
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Signal::StrongBuy => write!(f, "СИЛЬНАЯ ПОКУПКА"),
            Signal::Buy => write!(f, "ПОКУПКА"),
            Signal::Hold => write!(f, "ДЕРЖАТЬ"),
            Signal::Sell => write!(f, "ПРОДАЖА"),
            Signal::StrongSell => write!(f, "СИЛЬНАЯ ПРОДАЖА"),
        }
    }
}

/// Торговая стратегия на основе RSI
struct RsiStrategy {
    rsi_calculator: RsiCalculator,
    oversold_level: f64,
    overbought_level: f64,
    strong_oversold: f64,
    strong_overbought: f64,
}

impl RsiStrategy {
    fn new(period: usize) -> Self {
        RsiStrategy {
            rsi_calculator: RsiCalculator::new(period),
            oversold_level: 30.0,
            overbought_level: 70.0,
            strong_oversold: 20.0,
            strong_overbought: 80.0,
        }
    }

    /// Обновляет стратегию и возвращает сигнал
    fn update(&mut self, price: f64) -> Option<(f64, Signal)> {
        let rsi = self.rsi_calculator.update(price)?;

        let signal = if rsi < self.strong_oversold {
            Signal::StrongBuy
        } else if rsi < self.oversold_level {
            Signal::Buy
        } else if rsi > self.strong_overbought {
            Signal::StrongSell
        } else if rsi > self.overbought_level {
            Signal::Sell
        } else {
            Signal::Hold
        };

        Some((rsi, signal))
    }
}

/// Калькулятор RSI (повторно используем предыдущую реализацию)
#[derive(Debug)]
struct RsiCalculator {
    period: usize,
    prices: Vec<f64>,
    avg_gain: f64,
    avg_loss: f64,
    initialized: bool,
}

impl RsiCalculator {
    fn new(period: usize) -> Self {
        RsiCalculator {
            period,
            prices: Vec::new(),
            avg_gain: 0.0,
            avg_loss: 0.0,
            initialized: false,
        }
    }

    fn update(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);

        if self.prices.len() < self.period + 1 {
            return None;
        }

        let change = price - self.prices[self.prices.len() - 2];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { change.abs() } else { 0.0 };

        if !self.initialized {
            let changes: Vec<f64> = self.prices.windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            let sum_gains: f64 = changes.iter()
                .map(|&c| if c > 0.0 { c } else { 0.0 })
                .sum();
            let sum_losses: f64 = changes.iter()
                .map(|&c| if c < 0.0 { c.abs() } else { 0.0 })
                .sum();

            self.avg_gain = sum_gains / self.period as f64;
            self.avg_loss = sum_losses / self.period as f64;
            self.initialized = true;
        } else {
            self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain)
                / self.period as f64;
            self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss)
                / self.period as f64;
        }

        if self.avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = self.avg_gain / self.avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
}

fn main() {
    let mut strategy = RsiStrategy::new(14);

    // Симулируем волатильный рынок
    let prices = vec![
        50000.0, 50500.0, 51000.0, 51500.0, 52000.0,
        52500.0, 53000.0, 53500.0, 54000.0, 54500.0,
        55000.0, 55500.0, 56000.0, 56500.0, 57000.0,
        // Резкий рост — RSI должен показать перекупленность
        58000.0, 59000.0, 60000.0, 61000.0, 62000.0,
        // Коррекция
        61000.0, 60000.0, 59000.0, 58000.0, 57000.0,
        56000.0, 55000.0, 54000.0, 53000.0, 52000.0,
        // Сильное падение — RSI должен показать перепроданность
        51000.0, 50000.0, 49000.0, 48000.0, 47000.0,
    ];

    println!("RSI-стратегия для BTC:\n");
    println!("{:>4} | {:>10} | {:>6} | {:>15}", "День", "Цена", "RSI", "Сигнал");
    println!("{}", "-".repeat(45));

    for (i, price) in prices.iter().enumerate() {
        if let Some((rsi, signal)) = strategy.update(*price) {
            println!(
                "{:>4} | ${:>9.0} | {:>5.1} | {}",
                i + 1,
                price,
                rsi,
                signal
            );
        }
    }
}
```

## RSI-дивергенция

Дивергенция — это расхождение между ценой и RSI, которое может сигнализировать о развороте тренда:

```rust
/// Тип дивергенции
#[derive(Debug, PartialEq)]
enum Divergence {
    Bullish,   // Цена делает новый минимум, RSI — нет (бычья)
    Bearish,   // Цена делает новый максимум, RSI — нет (медвежья)
    None,
}

/// Детектор дивергенций
struct DivergenceDetector {
    price_highs: Vec<f64>,
    price_lows: Vec<f64>,
    rsi_at_highs: Vec<f64>,
    rsi_at_lows: Vec<f64>,
    lookback: usize,
}

impl DivergenceDetector {
    fn new(lookback: usize) -> Self {
        DivergenceDetector {
            price_highs: Vec::new(),
            price_lows: Vec::new(),
            rsi_at_highs: Vec::new(),
            rsi_at_lows: Vec::new(),
            lookback,
        }
    }

    /// Записывает локальный максимум
    fn record_high(&mut self, price: f64, rsi: f64) {
        self.price_highs.push(price);
        self.rsi_at_highs.push(rsi);

        // Сохраняем только последние значения
        if self.price_highs.len() > self.lookback {
            self.price_highs.remove(0);
            self.rsi_at_highs.remove(0);
        }
    }

    /// Записывает локальный минимум
    fn record_low(&mut self, price: f64, rsi: f64) {
        self.price_lows.push(price);
        self.rsi_at_lows.push(rsi);

        if self.price_lows.len() > self.lookback {
            self.price_lows.remove(0);
            self.rsi_at_lows.remove(0);
        }
    }

    /// Проверяет наличие медвежьей дивергенции
    fn check_bearish_divergence(&self) -> bool {
        if self.price_highs.len() < 2 {
            return false;
        }

        let len = self.price_highs.len();
        let prev_price = self.price_highs[len - 2];
        let curr_price = self.price_highs[len - 1];
        let prev_rsi = self.rsi_at_highs[len - 2];
        let curr_rsi = self.rsi_at_highs[len - 1];

        // Цена выше, но RSI ниже — медвежья дивергенция
        curr_price > prev_price && curr_rsi < prev_rsi
    }

    /// Проверяет наличие бычьей дивергенции
    fn check_bullish_divergence(&self) -> bool {
        if self.price_lows.len() < 2 {
            return false;
        }

        let len = self.price_lows.len();
        let prev_price = self.price_lows[len - 2];
        let curr_price = self.price_lows[len - 1];
        let prev_rsi = self.rsi_at_lows[len - 2];
        let curr_rsi = self.rsi_at_lows[len - 1];

        // Цена ниже, но RSI выше — бычья дивергенция
        curr_price < prev_price && curr_rsi > prev_rsi
    }

    /// Определяет текущую дивергенцию
    fn detect(&self) -> Divergence {
        if self.check_bullish_divergence() {
            Divergence::Bullish
        } else if self.check_bearish_divergence() {
            Divergence::Bearish
        } else {
            Divergence::None
        }
    }
}

fn main() {
    let mut detector = DivergenceDetector::new(5);

    // Симулируем медвежью дивергенцию:
    // Цена делает новые максимумы, но RSI падает
    println!("Демонстрация медвежьей дивергенции:\n");

    detector.record_high(50000.0, 75.0);
    println!("Максимум 1: Цена $50,000, RSI: 75.0");

    detector.record_high(52000.0, 72.0);
    println!("Максимум 2: Цена $52,000, RSI: 72.0");

    detector.record_high(54000.0, 68.0);
    println!("Максимум 3: Цена $54,000, RSI: 68.0");

    println!("\nАнализ: Цена растёт ($50K -> $52K -> $54K)");
    println!("        RSI падает (75 -> 72 -> 68)");
    println!("Результат: {:?}", detector.detect());
    println!("Интерпретация: Сила роста ослабевает, возможен разворот вниз");

    // Симулируем бычью дивергенцию
    let mut detector2 = DivergenceDetector::new(5);

    println!("\n{}\n", "=".repeat(50));
    println!("Демонстрация бычьей дивергенции:\n");

    detector2.record_low(50000.0, 25.0);
    println!("Минимум 1: Цена $50,000, RSI: 25.0");

    detector2.record_low(48000.0, 28.0);
    println!("Минимум 2: Цена $48,000, RSI: 28.0");

    detector2.record_low(46000.0, 32.0);
    println!("Минимум 3: Цена $46,000, RSI: 32.0");

    println!("\nАнализ: Цена падает ($50K -> $48K -> $46K)");
    println!("        RSI растёт (25 -> 28 -> 32)");
    println!("Результат: {:?}", detector2.detect());
    println!("Интерпретация: Сила падения ослабевает, возможен разворот вверх");
}
```

## Мультитаймфреймовый RSI

Анализ RSI на нескольких таймфреймах для более надёжных сигналов:

```rust
use std::collections::HashMap;

/// Таймфрейм
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum Timeframe {
    Minutes15,
    Hour1,
    Hours4,
    Daily,
}

impl std::fmt::Display for Timeframe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Timeframe::Minutes15 => write!(f, "15M"),
            Timeframe::Hour1 => write!(f, "1H"),
            Timeframe::Hours4 => write!(f, "4H"),
            Timeframe::Daily => write!(f, "1D"),
        }
    }
}

/// Мультитаймфреймовый анализатор RSI
struct MultiTimeframeRsi {
    analyzers: HashMap<Timeframe, RsiCalculator>,
}

impl MultiTimeframeRsi {
    fn new(period: usize) -> Self {
        let mut analyzers = HashMap::new();
        analyzers.insert(Timeframe::Minutes15, RsiCalculator::new(period));
        analyzers.insert(Timeframe::Hour1, RsiCalculator::new(period));
        analyzers.insert(Timeframe::Hours4, RsiCalculator::new(period));
        analyzers.insert(Timeframe::Daily, RsiCalculator::new(period));

        MultiTimeframeRsi { analyzers }
    }

    /// Обновляет RSI для указанного таймфрейма
    fn update(&mut self, timeframe: Timeframe, price: f64) -> Option<f64> {
        self.analyzers.get_mut(&timeframe)?.update(price)
    }

    /// Получает консенсус по всем таймфреймам
    fn get_consensus(&self) -> String {
        let mut bullish = 0;
        let mut bearish = 0;
        let mut neutral = 0;

        for (tf, calc) in &self.analyzers {
            if let Some(rsi) = calc.calculate_rsi() {
                if rsi < 30.0 {
                    bullish += 1;
                } else if rsi > 70.0 {
                    bearish += 1;
                } else {
                    neutral += 1;
                }
            }
        }

        if bullish > bearish && bullish > neutral {
            "Бычий консенсус (покупка)".to_string()
        } else if bearish > bullish && bearish > neutral {
            "Медвежий консенсус (продажа)".to_string()
        } else {
            "Смешанные сигналы (ожидание)".to_string()
        }
    }

    /// Выводит текущие значения RSI по всем таймфреймам
    fn print_status(&self) {
        println!("\nМультитаймфреймовый RSI:");
        println!("{}", "-".repeat(30));

        for tf in [Timeframe::Minutes15, Timeframe::Hour1,
                   Timeframe::Hours4, Timeframe::Daily] {
            if let Some(calc) = self.analyzers.get(&tf) {
                if let Some(rsi) = calc.calculate_rsi() {
                    let status = if rsi < 30.0 {
                        "Перепродан"
                    } else if rsi > 70.0 {
                        "Перекуплен"
                    } else {
                        "Нейтрален"
                    };
                    println!("{:>4}: RSI {:5.1} ({})", tf, rsi, status);
                }
            }
        }

        println!("\nКонсенсус: {}", self.get_consensus());
    }
}

/// Калькулятор RSI (используем предыдущую реализацию)
#[derive(Debug)]
struct RsiCalculator {
    period: usize,
    prices: Vec<f64>,
    avg_gain: f64,
    avg_loss: f64,
    initialized: bool,
}

impl RsiCalculator {
    fn new(period: usize) -> Self {
        RsiCalculator {
            period,
            prices: Vec::new(),
            avg_gain: 0.0,
            avg_loss: 0.0,
            initialized: false,
        }
    }

    fn update(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);

        if self.prices.len() < self.period + 1 {
            return None;
        }

        let change = price - self.prices[self.prices.len() - 2];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { change.abs() } else { 0.0 };

        if !self.initialized {
            let changes: Vec<f64> = self.prices.windows(2)
                .map(|w| w[1] - w[0])
                .collect();

            let sum_gains: f64 = changes.iter()
                .map(|&c| if c > 0.0 { c } else { 0.0 })
                .sum();
            let sum_losses: f64 = changes.iter()
                .map(|&c| if c < 0.0 { c.abs() } else { 0.0 })
                .sum();

            self.avg_gain = sum_gains / self.period as f64;
            self.avg_loss = sum_losses / self.period as f64;
            self.initialized = true;
        } else {
            self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain)
                / self.period as f64;
            self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss)
                / self.period as f64;
        }

        self.calculate_rsi()
    }

    fn calculate_rsi(&self) -> Option<f64> {
        if !self.initialized {
            return None;
        }

        if self.avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = self.avg_gain / self.avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
}

fn main() {
    let mut mtf_rsi = MultiTimeframeRsi::new(14);

    // Симулируем данные для разных таймфреймов
    // 15-минутные данные (более волатильные)
    let m15_prices = vec![
        100.0, 101.0, 100.5, 102.0, 103.0, 102.5, 104.0, 105.0,
        104.5, 106.0, 107.0, 106.5, 108.0, 109.0, 108.5, 110.0,
    ];

    // Часовые данные (средняя волатильность)
    let h1_prices = vec![
        100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 106.0, 108.0,
        110.0, 109.0, 111.0, 113.0, 112.0, 114.0, 116.0, 115.0,
    ];

    // 4-часовые данные (сглаженные)
    let h4_prices = vec![
        100.0, 105.0, 110.0, 108.0, 112.0, 115.0, 113.0, 118.0,
        120.0, 118.0, 122.0, 125.0, 123.0, 128.0, 130.0, 128.0,
    ];

    // Дневные данные (самые сглаженные)
    let daily_prices = vec![
        100.0, 110.0, 105.0, 115.0, 120.0, 118.0, 125.0, 130.0,
        128.0, 135.0, 140.0, 138.0, 145.0, 150.0, 148.0, 155.0,
    ];

    // Обновляем RSI для каждого таймфрейма
    for price in m15_prices {
        mtf_rsi.update(Timeframe::Minutes15, price);
    }

    for price in h1_prices {
        mtf_rsi.update(Timeframe::Hour1, price);
    }

    for price in h4_prices {
        mtf_rsi.update(Timeframe::Hours4, price);
    }

    for price in daily_prices {
        mtf_rsi.update(Timeframe::Daily, price);
    }

    mtf_rsi.print_status();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| RSI | Индекс относительной силы — осциллятор от 0 до 100 |
| Перекупленность | RSI > 70 — возможен разворот вниз |
| Перепроданность | RSI < 30 — возможен разворот вверх |
| RS | Relative Strength = Средний прирост / Средняя потеря |
| Сглаживание Уайлдера | Экспоненциальное сглаживание для расчёта средних |
| Дивергенция | Расхождение между ценой и RSI — сигнал разворота |
| Мультитаймфрейм | Анализ RSI на нескольких временных интервалах |

## Домашнее задание

1. **Калькулятор RSI**: Реализуйте структуру `RsiIndicator` с методами:
   - `new(period: usize)` — создание с заданным периодом
   - `add_price(price: f64)` — добавление новой цены
   - `get_rsi()` — получение текущего значения RSI
   - `get_signal()` — получение торгового сигнала (Buy/Sell/Hold)

2. **RSI с настраиваемыми уровнями**: Расширьте калькулятор, добавив:
   - Настраиваемые уровни перекупленности/перепроданности
   - Историю значений RSI за последние N периодов
   - Метод для определения тренда RSI (растёт/падает/боковое движение)

3. **Детектор дивергенций**: Создайте систему, которая:
   - Автоматически определяет локальные максимумы и минимумы цены
   - Сравнивает их с соответствующими значениями RSI
   - Выдаёт предупреждение при обнаружении дивергенции

4. **Бэктестинг RSI-стратегии**: Напишите программу, которая:
   - Загружает исторические данные (можно использовать массив цен)
   - Применяет RSI-стратегию (покупка при RSI < 30, продажа при RSI > 70)
   - Рассчитывает итоговую прибыль/убыток
   - Сравнивает с стратегией "купи и держи"

## Навигация

[← Предыдущий день](../248-ema-exponential-moving-average/ru.md) | [Следующий день →](../250-macd-moving-average-convergence-divergence/ru.md)
