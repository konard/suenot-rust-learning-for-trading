# День 257: Комбинирование индикаторов — построение полноценной торговой системы

## Аналогия из трейдинга

Профессиональные трейдеры редко полагаются на один индикатор. Представь пилота, сажающего самолёт: он не смотрит только на высоту — он одновременно отслеживает скорость, скорость снижения, топливо и положение относительно полосы. Аналогично, опытные трейдеры комбинируют несколько индикаторов, чтобы отфильтровать ложные сигналы и подтвердить торговые возможности. Один индикатор говорит «покупай», но остальные не согласны? Подожди. Все индикаторы совпадают? Это сигнал высокой надёжности.

## Трейт Indicator (повторение с Дня 256)

Перед комбинированием индикаторов установим фундамент — трейт `Indicator`:

```rust
trait Indicator {
    fn name(&self) -> &str;
    fn calculate(&mut self, price: f64) -> Option<f64>;
    fn is_ready(&self) -> bool;
}
```

Этот трейт предоставляет единый интерфейс для всех индикаторов, делая их компонуемыми.

## Простая скользящая средняя (SMA)

```rust
struct SMA {
    period: usize,
    prices: Vec<f64>,
}

impl SMA {
    fn new(period: usize) -> Self {
        SMA {
            period,
            prices: Vec::with_capacity(period),
        }
    }
}

impl Indicator for SMA {
    fn name(&self) -> &str {
        "SMA"
    }

    fn calculate(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }

        if self.prices.len() == self.period {
            Some(self.prices.iter().sum::<f64>() / self.period as f64)
        } else {
            None
        }
    }

    fn is_ready(&self) -> bool {
        self.prices.len() >= self.period
    }
}
```

## Экспоненциальная скользящая средняя (EMA)

```rust
struct EMA {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    count: usize,
}

impl EMA {
    fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        EMA {
            period,
            multiplier,
            current_ema: None,
            count: 0,
        }
    }
}

impl Indicator for EMA {
    fn name(&self) -> &str {
        "EMA"
    }

    fn calculate(&mut self, price: f64) -> Option<f64> {
        self.count += 1;

        self.current_ema = Some(match self.current_ema {
            None => price,
            Some(prev_ema) => (price - prev_ema) * self.multiplier + prev_ema,
        });

        if self.count >= self.period {
            self.current_ema
        } else {
            None
        }
    }

    fn is_ready(&self) -> bool {
        self.count >= self.period
    }
}
```

## Индекс относительной силы (RSI)

```rust
struct RSI {
    period: usize,
    gains: Vec<f64>,
    losses: Vec<f64>,
    prev_price: Option<f64>,
}

impl RSI {
    fn new(period: usize) -> Self {
        RSI {
            period,
            gains: Vec::new(),
            losses: Vec::new(),
            prev_price: None,
        }
    }
}

impl Indicator for RSI {
    fn name(&self) -> &str {
        "RSI"
    }

    fn calculate(&mut self, price: f64) -> Option<f64> {
        if let Some(prev) = self.prev_price {
            let change = price - prev;
            if change > 0.0 {
                self.gains.push(change);
                self.losses.push(0.0);
            } else {
                self.gains.push(0.0);
                self.losses.push(change.abs());
            }

            if self.gains.len() > self.period {
                self.gains.remove(0);
                self.losses.remove(0);
            }
        }
        self.prev_price = Some(price);

        if self.gains.len() == self.period {
            let avg_gain: f64 = self.gains.iter().sum::<f64>() / self.period as f64;
            let avg_loss: f64 = self.losses.iter().sum::<f64>() / self.period as f64;

            if avg_loss == 0.0 {
                Some(100.0)
            } else {
                let rs = avg_gain / avg_loss;
                Some(100.0 - (100.0 / (1.0 + rs)))
            }
        } else {
            None
        }
    }

    fn is_ready(&self) -> bool {
        self.gains.len() >= self.period
    }
}
```

## Комбинирование индикаторов: агрегатор сигналов

Теперь построим систему, объединяющую несколько индикаторов:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Signal {
    StrongBuy,
    Buy,
    Neutral,
    Sell,
    StrongSell,
}

struct SignalAggregator {
    indicators: Vec<Box<dyn Indicator>>,
    weights: Vec<f64>,
}

impl SignalAggregator {
    fn new() -> Self {
        SignalAggregator {
            indicators: Vec::new(),
            weights: Vec::new(),
        }
    }

    fn add_indicator(&mut self, indicator: Box<dyn Indicator>, weight: f64) {
        self.indicators.push(indicator);
        self.weights.push(weight);
    }

    fn update(&mut self, price: f64) -> Vec<Option<f64>> {
        self.indicators
            .iter_mut()
            .map(|ind| ind.calculate(price))
            .collect()
    }

    fn all_ready(&self) -> bool {
        self.indicators.iter().all(|ind| ind.is_ready())
    }
}
```

## Стратегия следования за трендом: пересечение SMA

Классическая стратегия, комбинирующая две скользящие средние:

```rust
struct SMACrossover {
    fast_sma: SMA,
    slow_sma: SMA,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
}

impl SMACrossover {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        SMACrossover {
            fast_sma: SMA::new(fast_period),
            slow_sma: SMA::new(slow_period),
            prev_fast: None,
            prev_slow: None,
        }
    }

    fn update(&mut self, price: f64) -> Option<Signal> {
        let fast = self.fast_sma.calculate(price);
        let slow = self.slow_sma.calculate(price);

        let signal = match (fast, slow, self.prev_fast, self.prev_slow) {
            (Some(f), Some(s), Some(pf), Some(ps)) => {
                // Золотой крест: быстрая пересекает медленную снизу вверх
                if pf <= ps && f > s {
                    Some(Signal::Buy)
                }
                // Мёртвый крест: быстрая пересекает медленную сверху вниз
                else if pf >= ps && f < s {
                    Some(Signal::Sell)
                } else {
                    Some(Signal::Neutral)
                }
            }
            _ => None,
        };

        self.prev_fast = fast;
        self.prev_slow = slow;
        signal
    }
}

fn main() {
    let mut crossover = SMACrossover::new(10, 20);

    let prices = [
        42000.0, 42100.0, 42300.0, 42500.0, 42400.0,
        42600.0, 42800.0, 43000.0, 43200.0, 43100.0,
        43300.0, 43500.0, 43400.0, 43600.0, 43800.0,
        44000.0, 43900.0, 43700.0, 43500.0, 43300.0,
        43100.0, 42900.0, 42700.0, 42500.0, 42300.0,
    ];

    println!("Стратегия пересечения SMA:");
    println!("==========================");

    for (i, &price) in prices.iter().enumerate() {
        if let Some(signal) = crossover.update(price) {
            println!("День {}: Цена ${:.2} -> {:?}", i + 1, price, signal);
        }
    }
}
```

## Система подтверждения несколькими индикаторами

Комбинирование трендовых и импульсных индикаторов для большей надёжности:

```rust
struct MultiIndicatorStrategy {
    ema_fast: EMA,
    ema_slow: EMA,
    rsi: RSI,
    prev_ema_fast: Option<f64>,
    prev_ema_slow: Option<f64>,
}

impl MultiIndicatorStrategy {
    fn new() -> Self {
        MultiIndicatorStrategy {
            ema_fast: EMA::new(12),
            ema_slow: EMA::new(26),
            rsi: RSI::new(14),
            prev_ema_fast: None,
            prev_ema_slow: None,
        }
    }

    fn analyze(&mut self, price: f64) -> Option<(Signal, f64)> {
        let ema_fast = self.ema_fast.calculate(price);
        let ema_slow = self.ema_slow.calculate(price);
        let rsi = self.rsi.calculate(price);

        let result = match (ema_fast, ema_slow, rsi, self.prev_ema_fast, self.prev_ema_slow) {
            (Some(ef), Some(es), Some(r), Some(pef), Some(pes)) => {
                let trend_bullish = ef > es;
                let trend_bearish = ef < es;
                let ema_cross_up = pef <= pes && ef > es;
                let ema_cross_down = pef >= pes && ef < es;

                // Комбинируем сигналы с оценкой уверенности
                let (signal, confidence) = if ema_cross_up && r < 70.0 {
                    // Бычье пересечение при RSI не в зоне перекупленности
                    if r < 30.0 {
                        (Signal::StrongBuy, 0.9)
                    } else {
                        (Signal::Buy, 0.7)
                    }
                } else if ema_cross_down && r > 30.0 {
                    // Медвежье пересечение при RSI не в зоне перепроданности
                    if r > 70.0 {
                        (Signal::StrongSell, 0.9)
                    } else {
                        (Signal::Sell, 0.7)
                    }
                } else if trend_bullish && r < 30.0 {
                    // Перепроданность в восходящем тренде - потенциальная покупка
                    (Signal::Buy, 0.6)
                } else if trend_bearish && r > 70.0 {
                    // Перекупленность в нисходящем тренде - потенциальная продажа
                    (Signal::Sell, 0.6)
                } else {
                    (Signal::Neutral, 0.5)
                };

                Some((signal, confidence))
            }
            _ => None,
        };

        self.prev_ema_fast = ema_fast;
        self.prev_ema_slow = ema_slow;
        result
    }
}

fn main() {
    let mut strategy = MultiIndicatorStrategy::new();

    let prices = [
        42000.0, 42050.0, 42100.0, 42080.0, 42150.0, 42200.0, 42180.0, 42250.0,
        42300.0, 42280.0, 42350.0, 42400.0, 42450.0, 42500.0, 42550.0, 42600.0,
        42650.0, 42700.0, 42800.0, 42900.0, 43000.0, 43100.0, 43200.0, 43300.0,
        43400.0, 43500.0, 43600.0, 43700.0, 43800.0, 43900.0, 44000.0, 44100.0,
    ];

    println!("Анализ мульти-индикаторной стратегии:");
    println!("=====================================");

    for (i, &price) in prices.iter().enumerate() {
        if let Some((signal, confidence)) = strategy.analyze(price) {
            if signal != Signal::Neutral {
                println!(
                    "День {}: ${:.2} -> {:?} (Уверенность: {:.0}%)",
                    i + 1,
                    price,
                    signal,
                    confidence * 100.0
                );
            }
        }
    }
}
```

## Взвешенная комбинация сигналов

Разные индикаторы имеют разную надёжность в различных рыночных условиях:

```rust
struct WeightedSignalCombiner {
    signals: Vec<(String, Signal, f64)>, // (имя, сигнал, вес)
}

impl WeightedSignalCombiner {
    fn new() -> Self {
        WeightedSignalCombiner { signals: Vec::new() }
    }

    fn add_signal(&mut self, name: &str, signal: Signal, weight: f64) {
        self.signals.push((name.to_string(), signal, weight));
    }

    fn clear(&mut self) {
        self.signals.clear();
    }

    fn get_combined_signal(&self) -> (Signal, f64) {
        if self.signals.is_empty() {
            return (Signal::Neutral, 0.0);
        }

        let mut score = 0.0;
        let mut total_weight = 0.0;

        for (_, signal, weight) in &self.signals {
            let signal_value = match signal {
                Signal::StrongBuy => 2.0,
                Signal::Buy => 1.0,
                Signal::Neutral => 0.0,
                Signal::Sell => -1.0,
                Signal::StrongSell => -2.0,
            };
            score += signal_value * weight;
            total_weight += weight;
        }

        let normalized_score = if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        };

        let final_signal = if normalized_score > 1.5 {
            Signal::StrongBuy
        } else if normalized_score > 0.5 {
            Signal::Buy
        } else if normalized_score < -1.5 {
            Signal::StrongSell
        } else if normalized_score < -0.5 {
            Signal::Sell
        } else {
            Signal::Neutral
        };

        let confidence = normalized_score.abs().min(2.0) / 2.0;
        (final_signal, confidence)
    }

    fn print_breakdown(&self) {
        println!("\nРазбор сигналов:");
        println!("-----------------");
        for (name, signal, weight) in &self.signals {
            println!("  {} ({:.1}): {:?}", name, weight, signal);
        }
        let (combined, confidence) = self.get_combined_signal();
        println!("-----------------");
        println!("  Итого: {:?} (Уверенность: {:.0}%)", combined, confidence * 100.0);
    }
}

fn main() {
    let mut combiner = WeightedSignalCombiner::new();

    // Сценарий: трендовый рынок с перепроданным RSI
    combiner.add_signal("EMA Crossover", Signal::Buy, 1.0);
    combiner.add_signal("RSI", Signal::StrongBuy, 0.8);
    combiner.add_signal("MACD", Signal::Buy, 0.9);
    combiner.add_signal("Volume", Signal::Neutral, 0.5);

    combiner.print_breakdown();
}
```

## Практический пример: полная торговая система

```rust
fn main() {
    // Строим полноценную мульти-индикаторную торговую систему
    let mut ema_12 = EMA::new(12);
    let mut ema_26 = EMA::new(26);
    let mut rsi_14 = RSI::new(14);
    let mut sma_50 = SMA::new(50);

    // Смоделированные ценовые данные (50+ периодов для SMA-50)
    let prices: Vec<f64> = (0..60)
        .map(|i| 42000.0 + (i as f64 * 50.0) + ((i as f64).sin() * 200.0))
        .collect();

    let mut prev_ema_12: Option<f64> = None;
    let mut prev_ema_26: Option<f64> = None;

    println!("Анализ полной торговой системы");
    println!("==============================\n");

    for (day, &price) in prices.iter().enumerate() {
        let e12 = ema_12.calculate(price);
        let e26 = ema_26.calculate(price);
        let rsi = rsi_14.calculate(price);
        let sma = sma_50.calculate(price);

        // Анализируем только когда все индикаторы готовы
        if let (Some(e12_val), Some(e26_val), Some(rsi_val), Some(sma_val)) =
            (e12, e26, rsi, sma)
        {
            // Определяем тренд по SMA-50
            let long_term_trend = if price > sma_val { "Бычий" } else { "Медвежий" };

            // Проверяем пересечение EMA
            let crossover = match (prev_ema_12, prev_ema_26) {
                (Some(pe12), Some(pe26)) => {
                    if pe12 <= pe26 && e12_val > e26_val {
                        Some("Золотой крест")
                    } else if pe12 >= pe26 && e12_val < e26_val {
                        Some("Мёртвый крест")
                    } else {
                        None
                    }
                }
                _ => None,
            };

            // Состояние RSI
            let rsi_condition = if rsi_val < 30.0 {
                "Перепродан"
            } else if rsi_val > 70.0 {
                "Перекуплен"
            } else {
                "Нейтрально"
            };

            // Генерируем торговое решение
            let decision = match (long_term_trend, crossover, rsi_condition) {
                ("Бычий", Some("Золотой крест"), "Перепродан") => "СИЛЬНАЯ ПОКУПКА",
                ("Бычий", Some("Золотой крест"), _) => "ПОКУПКА",
                ("Бычий", None, "Перепродан") => "ПОКУПКА (RSI)",
                ("Медвежий", Some("Мёртвый крест"), "Перекуплен") => "СИЛЬНАЯ ПРОДАЖА",
                ("Медвежий", Some("Мёртвый крест"), _) => "ПРОДАЖА",
                ("Медвежий", None, "Перекуплен") => "ПРОДАЖА (RSI)",
                _ => "ОЖИДАНИЕ",
            };

            if decision != "ОЖИДАНИЕ" {
                println!("День {:2}: ${:.2}", day + 1, price);
                println!("        Тренд: {} | RSI: {:.1} ({}) | Решение: {}",
                    long_term_trend, rsi_val, rsi_condition, decision);
                if let Some(cross) = crossover {
                    println!("        Сигнал: {}", cross);
                }
                println!();
            }
        }

        prev_ema_12 = e12;
        prev_ema_26 = e26;
    }
}
```

## Паттерн фильтрации: индикатор как привратник

Используй один индикатор для фильтрации сигналов другого:

```rust
struct FilteredStrategy<T: Indicator, F: Indicator> {
    signal_indicator: T,
    filter_indicator: F,
    filter_condition: Box<dyn Fn(f64) -> bool>,
}

impl<T: Indicator, F: Indicator> FilteredStrategy<T, F> {
    fn new(signal: T, filter: F, condition: Box<dyn Fn(f64) -> bool>) -> Self {
        FilteredStrategy {
            signal_indicator: signal,
            filter_indicator: filter,
            filter_condition: condition,
        }
    }

    fn update(&mut self, price: f64) -> Option<f64> {
        let signal_value = self.signal_indicator.calculate(price);
        let filter_value = self.filter_indicator.calculate(price);

        match (signal_value, filter_value) {
            (Some(sv), Some(fv)) if (self.filter_condition)(fv) => Some(sv),
            _ => None,
        }
    }
}

fn main() {
    // Принимаем сигналы EMA только когда RSI не в экстремумах
    let ema = EMA::new(20);
    let rsi = RSI::new(14);

    let mut strategy = FilteredStrategy::new(
        ema,
        rsi,
        Box::new(|rsi_val| rsi_val > 30.0 && rsi_val < 70.0),
    );

    let prices = [42000.0, 42100.0, 42200.0, 42300.0, 42400.0];

    for price in prices {
        match strategy.update(price) {
            Some(ema_val) => println!("Отфильтрованный EMA: {:.2}", ema_val),
            None => println!("Сигнал отфильтрован (RSI в экстремуме)"),
        }
    }
}
```

## Что мы узнали

| Концепция | Описание | Применение |
|-----------|----------|------------|
| Трейт Indicator | Единый интерфейс для всех индикаторов | Полиморфизм и композиция |
| Агрегатор сигналов | Собирает выходы нескольких индикаторов | Мульти-индикаторный анализ |
| Стратегия пересечения | Обнаруживает пересечение индикаторов | Сигналы разворота тренда |
| Взвешенная комбинация | Назначает важность каждому сигналу | Оценка уверенности |
| Паттерн фильтрации | Один индикатор отсеивает сигналы другого | Уменьшение ложных сигналов |
| Оценка уверенности | Количественная оценка силы сигнала | Расчёт размера позиции |

## Домашнее задание

1. Реализуй индикатор `MACD` (Moving Average Convergence Divergence), который комбинирует две EMA и сигнальную линию. Используй его вместе с RSI для создания импульсной торговой стратегии.

2. Создай `VolatilityFilter`, который использует полосы Боллинджера для фильтрации сигналов в периоды низкой волатильности. Разрешай сделки только когда цена касается полос.

3. Построй `ConsensusSystem`, где минимум 3 из 5 индикаторов должны согласиться перед генерацией сигнала. Реализуй индикаторы: SMA, EMA, RSI, и добавь MACD и Stochastic.

4. Спроектируй `DynamicWeightAdjuster`, который изменяет веса индикаторов на основе их недавней эффективности. Если индикатор генерирует прибыльные сигналы, увеличивай его вес.

## Навигация

[← Предыдущий день](../256-indicator-trait-pattern/ru.md) | [Следующий день →](../258-signals-buy-sell/ru.md)
