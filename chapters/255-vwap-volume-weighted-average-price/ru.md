# День 255: VWAP: Средневзвешенная по объёму цена

## Аналогия из трейдинга

Представь, что ты крупный институциональный трейдер, которому нужно купить 100 000 акций в течение дня. Ты не хочешь двигать рынок одним гигантским ордером, поэтому распределяешь покупки на всю торговую сессию. В конце дня — как узнать, получил ли ты хорошую цену?

Здесь на помощь приходит **VWAP (Volume Weighted Average Price)** — средневзвешенная по объёму цена. VWAP — это как "справедливая цена", бенчмарк, который показывает среднюю цену, взвешенную по объёму торгов на каждом ценовом уровне. Если ты купил ниже VWAP — ты получил цену лучше средней. Если выше VWAP — заплатил премию.

В алгоритмическом трейдинге VWAP критически важен для:
- **Оценки исполнения** — Алгоритм купил/продал по справедливым ценам?
- **Стратегий исполнения ордеров** — VWAP-алгоритмы стремятся соответствовать или превзойти этот бенчмарк
- **Определения тренда** — Цена выше VWAP указывает на бычье давление, ниже — на медвежье
- **Управления рисками** — Обнаружение переплаты за крупные ордера

## Что такое VWAP?

VWAP рассчитывается по формуле:

```
VWAP = Σ(Цена × Объём) / Σ(Объём)
```

Или более детально:
```
VWAP = (P₁×V₁ + P₂×V₂ + ... + Pₙ×Vₙ) / (V₁ + V₂ + ... + Vₙ)
```

Где:
- `P` = Цена (обычно типичная цена: (High + Low + Close) / 3)
- `V` = Объём на этом ценовом уровне

## Базовая реализация VWAP

```rust
/// Представляет одну сделку или свечу
#[derive(Debug, Clone)]
struct PriceData {
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl PriceData {
    fn new(high: f64, low: f64, close: f64, volume: f64) -> Self {
        PriceData { high, low, close, volume }
    }

    /// Вычисляет типичную цену (среднее high, low, close)
    fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }
}

/// Калькулятор VWAP, накапливающий данные в течение сессии
#[derive(Debug)]
struct VwapCalculator {
    cumulative_volume: f64,
    cumulative_price_volume: f64,
}

impl VwapCalculator {
    fn new() -> Self {
        VwapCalculator {
            cumulative_volume: 0.0,
            cumulative_price_volume: 0.0,
        }
    }

    /// Добавляет новые данные и возвращает обновлённый VWAP
    fn add(&mut self, data: &PriceData) -> f64 {
        let typical_price = data.typical_price();

        self.cumulative_volume += data.volume;
        self.cumulative_price_volume += typical_price * data.volume;

        self.calculate()
    }

    /// Вычисляет текущий VWAP
    fn calculate(&self) -> f64 {
        if self.cumulative_volume == 0.0 {
            return 0.0;
        }
        self.cumulative_price_volume / self.cumulative_volume
    }

    /// Сбрасывает калькулятор для новой торговой сессии
    fn reset(&mut self) {
        self.cumulative_volume = 0.0;
        self.cumulative_price_volume = 0.0;
    }
}

fn main() {
    let mut vwap_calc = VwapCalculator::new();

    // Симулированные внутридневные торговые данные
    let trading_data = vec![
        PriceData::new(100.50, 99.80, 100.20, 10000.0),
        PriceData::new(100.30, 99.90, 100.10, 15000.0),
        PriceData::new(100.80, 100.00, 100.60, 20000.0),
        PriceData::new(101.20, 100.40, 101.00, 25000.0),
        PriceData::new(101.50, 100.80, 101.20, 18000.0),
    ];

    println!("=== Расчёт VWAP в течение дня ===\n");

    for (i, data) in trading_data.iter().enumerate() {
        let vwap = vwap_calc.add(data);
        let typical = data.typical_price();

        let position = if data.close > vwap {
            "выше"
        } else if data.close < vwap {
            "ниже"
        } else {
            "на уровне"
        };

        println!(
            "Период {}: Close=${:.2}, Typical=${:.2}, Объём={:.0}, VWAP=${:.4} (Цена {} VWAP)",
            i + 1, data.close, typical, data.volume, vwap, position
        );
    }

    println!("\nИтоговый VWAP: ${:.4}", vwap_calc.calculate());
}
```

## VWAP с полосами стандартного отклонения

Профессиональные трейдеры часто используют VWAP с полосами стандартного отклонения для определения перекупленности/перепроданности:

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct PriceData {
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl PriceData {
    fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }
}

/// VWAP с полосами стандартного отклонения
#[derive(Debug)]
struct VwapWithBands {
    cumulative_volume: f64,
    cumulative_price_volume: f64,
    price_data: VecDeque<(f64, f64)>, // (typical_price, volume)
    band_multiplier: f64,
}

impl VwapWithBands {
    fn new(band_multiplier: f64) -> Self {
        VwapWithBands {
            cumulative_volume: 0.0,
            cumulative_price_volume: 0.0,
            price_data: VecDeque::new(),
            band_multiplier,
        }
    }

    fn add(&mut self, data: &PriceData) {
        let typical_price = data.typical_price();

        self.cumulative_volume += data.volume;
        self.cumulative_price_volume += typical_price * data.volume;
        self.price_data.push_back((typical_price, data.volume));
    }

    fn vwap(&self) -> f64 {
        if self.cumulative_volume == 0.0 {
            return 0.0;
        }
        self.cumulative_price_volume / self.cumulative_volume
    }

    /// Вычисляет взвешенное по объёму стандартное отклонение
    fn std_deviation(&self) -> f64 {
        if self.cumulative_volume == 0.0 || self.price_data.is_empty() {
            return 0.0;
        }

        let vwap = self.vwap();
        let mut weighted_variance_sum = 0.0;

        for (price, volume) in &self.price_data {
            let deviation = price - vwap;
            weighted_variance_sum += deviation * deviation * volume;
        }

        (weighted_variance_sum / self.cumulative_volume).sqrt()
    }

    fn upper_band(&self) -> f64 {
        self.vwap() + self.band_multiplier * self.std_deviation()
    }

    fn lower_band(&self) -> f64 {
        self.vwap() - self.band_multiplier * self.std_deviation()
    }

    fn reset(&mut self) {
        self.cumulative_volume = 0.0;
        self.cumulative_price_volume = 0.0;
        self.price_data.clear();
    }
}

fn main() {
    // Создаём VWAP с полосами в 2 стандартных отклонения
    let mut vwap = VwapWithBands::new(2.0);

    let trading_data = vec![
        PriceData { high: 100.50, low: 99.80, close: 100.20, volume: 10000.0 },
        PriceData { high: 100.30, low: 99.90, close: 100.10, volume: 15000.0 },
        PriceData { high: 100.80, low: 100.00, close: 100.60, volume: 20000.0 },
        PriceData { high: 101.20, low: 100.40, close: 101.00, volume: 25000.0 },
        PriceData { high: 101.50, low: 100.80, close: 101.20, volume: 18000.0 },
        PriceData { high: 101.80, low: 101.00, close: 101.50, volume: 22000.0 },
        PriceData { high: 102.20, low: 101.20, close: 101.80, volume: 30000.0 },
    ];

    println!("=== VWAP с полосами стандартного отклонения ===\n");

    for (i, data) in trading_data.iter().enumerate() {
        vwap.add(data);

        let current_vwap = vwap.vwap();
        let upper = vwap.upper_band();
        let lower = vwap.lower_band();

        let signal = if data.close > upper {
            "ПЕРЕКУПЛЕНО - Рассмотри продажу"
        } else if data.close < lower {
            "ПЕРЕПРОДАНО - Рассмотри покупку"
        } else {
            "НЕЙТРАЛЬНО"
        };

        println!("Период {}:", i + 1);
        println!("  Close: ${:.2}", data.close);
        println!("  VWAP:  ${:.4}", current_vwap);
        println!("  Верхняя полоса (+2σ): ${:.4}", upper);
        println!("  Нижняя полоса (-2σ): ${:.4}", lower);
        println!("  Сигнал: {}\n", signal);
    }
}
```

## Алгоритм исполнения VWAP

Одно из самых распространённых применений VWAP в алгоритмическом трейдинге — это бенчмарк исполнения. Вот простой алгоритм исполнения VWAP:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    id: u64,
    symbol: String,
    quantity: f64,
    executed_quantity: f64,
    average_price: f64,
}

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    price: f64,
    volume: f64,
    typical_volume_profile: Vec<f64>, // Ожидаемый объём для каждого периода
}

/// Алгоритм исполнения VWAP
/// Стремится исполнять ордера в соответствии с ожидаемым распределением объёма
#[derive(Debug)]
struct VwapExecutor {
    target_quantity: f64,
    executed_quantity: f64,
    total_cost: f64,
    current_period: usize,
    volume_profile: Vec<f64>,
    order_id_counter: u64,
    orders: Vec<Order>,
}

impl VwapExecutor {
    fn new(target_quantity: f64, volume_profile: Vec<f64>) -> Self {
        VwapExecutor {
            target_quantity,
            executed_quantity: 0.0,
            total_cost: 0.0,
            current_period: 0,
            volume_profile,
            order_id_counter: 0,
            orders: Vec::new(),
        }
    }

    /// Вычисляет объём для торговли в текущем периоде на основе профиля объёма
    fn calculate_period_quantity(&self) -> f64 {
        if self.current_period >= self.volume_profile.len() {
            return 0.0;
        }

        let remaining_quantity = self.target_quantity - self.executed_quantity;
        let total_profile: f64 = self.volume_profile.iter().sum();
        let remaining_profile: f64 = self.volume_profile[self.current_period..].iter().sum();

        if remaining_profile == 0.0 {
            return remaining_quantity;
        }

        // Пропорциональное распределение на основе профиля объёма
        let period_weight = self.volume_profile[self.current_period] / remaining_profile;
        remaining_quantity * period_weight
    }

    /// Исполняет текущий период
    fn execute_period(&mut self, market_price: f64) -> Option<Order> {
        let quantity = self.calculate_period_quantity();

        if quantity <= 0.0 {
            self.current_period += 1;
            return None;
        }

        self.order_id_counter += 1;
        let order = Order {
            id: self.order_id_counter,
            symbol: "BTC".to_string(),
            quantity,
            executed_quantity: quantity,
            average_price: market_price,
        };

        self.executed_quantity += quantity;
        self.total_cost += quantity * market_price;
        self.orders.push(order.clone());
        self.current_period += 1;

        Some(order)
    }

    /// Получает среднюю цену исполнения
    fn average_execution_price(&self) -> f64 {
        if self.executed_quantity == 0.0 {
            return 0.0;
        }
        self.total_cost / self.executed_quantity
    }

    /// Проверяет качество исполнения относительно VWAP
    fn execution_quality(&self, market_vwap: f64) -> f64 {
        // Положительное значение означает, что мы сделали лучше VWAP (для покупок — чем ниже, тем лучше)
        market_vwap - self.average_execution_price()
    }

    fn progress(&self) -> f64 {
        (self.executed_quantity / self.target_quantity) * 100.0
    }
}

fn main() {
    // Профиль объёма: ожидаемое распределение объёма в течение дня
    // Большие значения = больше объёма ожидается в этом периоде
    let volume_profile = vec![
        0.15, // Открытие утром - высокий объём
        0.10, // Середина утра
        0.08, // Позднее утро
        0.05, // Обед - низкий объём
        0.05, // Начало дня
        0.07, // Середина дня
        0.10, // Позднее после обеда
        0.15, // Приближение к закрытию
        0.25, // Закрывающий аукцион - максимальный объём
    ];

    let mut executor = VwapExecutor::new(1000.0, volume_profile);

    // Симулированные рыночные цены для каждого периода
    let market_prices = vec![
        42000.0, 42150.0, 42100.0, 42050.0, 42200.0,
        42300.0, 42250.0, 42400.0, 42350.0,
    ];

    // Вычисляем рыночный VWAP (то, что мы пытаемся повторить)
    let market_volumes = vec![
        15000.0, 10000.0, 8000.0, 5000.0, 5000.0,
        7000.0, 10000.0, 15000.0, 25000.0,
    ];

    let total_volume: f64 = market_volumes.iter().sum();
    let market_vwap: f64 = market_prices.iter()
        .zip(market_volumes.iter())
        .map(|(p, v)| p * v)
        .sum::<f64>() / total_volume;

    println!("=== Алгоритм исполнения VWAP ===\n");
    println!("Цель: Купить 1000 BTC в течение дня");
    println!("Рыночный VWAP: ${:.2}\n", market_vwap);

    for (i, price) in market_prices.iter().enumerate() {
        if let Some(order) = executor.execute_period(*price) {
            println!(
                "Период {}: Исполнено {:.2} BTC @ ${:.2} (Прогресс: {:.1}%)",
                i + 1, order.quantity, order.average_price, executor.progress()
            );
        }
    }

    println!("\n=== Итоги исполнения ===");
    println!("Всего исполнено: {:.2} BTC", executor.executed_quantity);
    println!("Средняя цена: ${:.2}", executor.average_execution_price());
    println!("Рыночный VWAP: ${:.2}", market_vwap);

    let quality = executor.execution_quality(market_vwap);
    if quality > 0.0 {
        println!("Результат: ПРЕВЗОШЛИ VWAP на ${:.2} за единицу!", quality);
    } else {
        println!("Результат: Не дотянули до VWAP на ${:.2} за единицу", -quality);
    }
}
```

## Генератор торговых сигналов на основе VWAP в реальном времени

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum TradingSignal {
    StrongBuy,
    Buy,
    Neutral,
    Sell,
    StrongSell,
}

impl TradingSignal {
    fn as_str(&self) -> &'static str {
        match self {
            TradingSignal::StrongBuy => "СИЛЬНАЯ ПОКУПКА",
            TradingSignal::Buy => "ПОКУПКА",
            TradingSignal::Neutral => "НЕЙТРАЛЬНО",
            TradingSignal::Sell => "ПРОДАЖА",
            TradingSignal::StrongSell => "СИЛЬНАЯ ПРОДАЖА",
        }
    }
}

#[derive(Debug)]
struct VwapSignalGenerator {
    cumulative_volume: f64,
    cumulative_price_volume: f64,
    squared_deviation_sum: f64,
    data_points: Vec<f64>,
    signal_threshold: f64,
}

impl VwapSignalGenerator {
    fn new(signal_threshold: f64) -> Self {
        VwapSignalGenerator {
            cumulative_volume: 0.0,
            cumulative_price_volume: 0.0,
            squared_deviation_sum: 0.0,
            data_points: Vec::new(),
            signal_threshold,
        }
    }

    fn update(&mut self, price: f64, volume: f64) {
        self.cumulative_volume += volume;
        self.cumulative_price_volume += price * volume;
        self.data_points.push(price);

        // Обновляем квадрат отклонения для расчёта std
        let vwap = self.vwap();
        self.squared_deviation_sum += (price - vwap).powi(2) * volume;
    }

    fn vwap(&self) -> f64 {
        if self.cumulative_volume == 0.0 {
            return 0.0;
        }
        self.cumulative_price_volume / self.cumulative_volume
    }

    fn std_deviation(&self) -> f64 {
        if self.cumulative_volume == 0.0 {
            return 0.0;
        }
        (self.squared_deviation_sum / self.cumulative_volume).sqrt()
    }

    fn generate_signal(&self, current_price: f64) -> TradingSignal {
        let vwap = self.vwap();
        let std = self.std_deviation();

        if std == 0.0 || vwap == 0.0 {
            return TradingSignal::Neutral;
        }

        let deviation = (current_price - vwap) / std;

        if deviation < -2.0 * self.signal_threshold {
            TradingSignal::StrongBuy
        } else if deviation < -self.signal_threshold {
            TradingSignal::Buy
        } else if deviation > 2.0 * self.signal_threshold {
            TradingSignal::StrongSell
        } else if deviation > self.signal_threshold {
            TradingSignal::Sell
        } else {
            TradingSignal::Neutral
        }
    }

    fn distance_from_vwap(&self, current_price: f64) -> f64 {
        let vwap = self.vwap();
        if vwap == 0.0 {
            return 0.0;
        }
        ((current_price - vwap) / vwap) * 100.0
    }
}

fn main() {
    let mut signal_gen = VwapSignalGenerator::new(1.0);

    // Симулированный поток цен в реальном времени
    let price_feed = vec![
        (100.0, 1000.0),  // (цена, объём)
        (100.5, 1500.0),
        (101.0, 2000.0),
        (100.8, 1800.0),
        (101.5, 2200.0),
        (102.0, 2500.0),
        (103.5, 3000.0),  // Резкое движение вверх
        (104.0, 3500.0),
        (103.0, 2000.0),
        (102.5, 1500.0),
        (99.0, 4000.0),   // Резкое падение
        (98.5, 5000.0),
        (99.5, 2500.0),
    ];

    println!("=== Генератор сигналов VWAP в реальном времени ===\n");

    for (i, (price, volume)) in price_feed.iter().enumerate() {
        signal_gen.update(*price, *volume);

        let signal = signal_gen.generate_signal(*price);
        let vwap = signal_gen.vwap();
        let distance = signal_gen.distance_from_vwap(*price);

        println!("Тик {}: Цена=${:.2}, Объём={:.0}", i + 1, price, volume);
        println!("  VWAP: ${:.4}, Расстояние: {:+.2}%", vwap, distance);
        println!("  Сигнал: {}\n", signal.as_str());
    }
}
```

## Анализ портфеля по VWAP

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    quantity: f64,
    price: f64,
    volume: f64,
}

#[derive(Debug)]
struct PortfolioVwapAnalyzer {
    trades: HashMap<String, Vec<Trade>>,
    market_vwap: HashMap<String, f64>,
}

impl PortfolioVwapAnalyzer {
    fn new() -> Self {
        PortfolioVwapAnalyzer {
            trades: HashMap::new(),
            market_vwap: HashMap::new(),
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        self.trades
            .entry(trade.symbol.clone())
            .or_insert_with(Vec::new)
            .push(trade);
    }

    fn set_market_vwap(&mut self, symbol: &str, vwap: f64) {
        self.market_vwap.insert(symbol.to_string(), vwap);
    }

    /// Вычисляет VWAP исполнения для символа
    fn execution_vwap(&self, symbol: &str) -> Option<f64> {
        let trades = self.trades.get(symbol)?;

        if trades.is_empty() {
            return None;
        }

        let total_cost: f64 = trades.iter()
            .map(|t| t.price * t.quantity)
            .sum();
        let total_quantity: f64 = trades.iter()
            .map(|t| t.quantity)
            .sum();

        Some(total_cost / total_quantity)
    }

    /// Вычисляет качество исполнения (проскальзывание относительно VWAP)
    fn slippage(&self, symbol: &str) -> Option<f64> {
        let exec_vwap = self.execution_vwap(symbol)?;
        let market_vwap = self.market_vwap.get(symbol)?;

        // Положительное = худшее исполнение (заплатили больше за покупки)
        Some(exec_vwap - market_vwap)
    }

    /// Вычисляет проскальзывание в базисных пунктах
    fn slippage_bps(&self, symbol: &str) -> Option<f64> {
        let slippage = self.slippage(symbol)?;
        let market_vwap = self.market_vwap.get(symbol)?;

        Some((slippage / market_vwap) * 10000.0)
    }

    fn analyze_all(&self) {
        println!("=== Анализ портфеля по VWAP ===\n");

        for symbol in self.trades.keys() {
            let trades = &self.trades[symbol];
            let total_qty: f64 = trades.iter().map(|t| t.quantity).sum();
            let exec_vwap = self.execution_vwap(symbol).unwrap_or(0.0);
            let market_vwap = self.market_vwap.get(symbol).copied().unwrap_or(0.0);
            let slippage_bps = self.slippage_bps(symbol).unwrap_or(0.0);

            println!("Символ: {}", symbol);
            println!("  Всего сделок: {}", trades.len());
            println!("  Общее количество: {:.4}", total_qty);
            println!("  VWAP исполнения: ${:.4}", exec_vwap);
            println!("  Рыночный VWAP: ${:.4}", market_vwap);
            println!("  Проскальзывание: {:.2} bps", slippage_bps);

            if slippage_bps > 0.0 {
                println!("  Оценка: ХУЖЕ РЫНКА (заплатили премию)\n");
            } else if slippage_bps < 0.0 {
                println!("  Оценка: ЛУЧШЕ РЫНКА (получили скидку)\n");
            } else {
                println!("  Оценка: ТОЧНОЕ СООТВЕТСТВИЕ VWAP\n");
            }
        }
    }
}

fn main() {
    let mut analyzer = PortfolioVwapAnalyzer::new();

    // Добавляем сделки
    analyzer.add_trade(Trade {
        symbol: "BTC".to_string(),
        quantity: 0.5,
        price: 42100.0,
        volume: 1000.0,
    });
    analyzer.add_trade(Trade {
        symbol: "BTC".to_string(),
        quantity: 0.3,
        price: 42050.0,
        volume: 800.0,
    });
    analyzer.add_trade(Trade {
        symbol: "BTC".to_string(),
        quantity: 0.2,
        price: 42200.0,
        volume: 500.0,
    });

    analyzer.add_trade(Trade {
        symbol: "ETH".to_string(),
        quantity: 5.0,
        price: 2250.0,
        volume: 10000.0,
    });
    analyzer.add_trade(Trade {
        symbol: "ETH".to_string(),
        quantity: 3.0,
        price: 2230.0,
        volume: 6000.0,
    });

    // Устанавливаем рыночный VWAP для сравнения
    analyzer.set_market_vwap("BTC", 42150.0);
    analyzer.set_market_vwap("ETH", 2240.0);

    analyzer.analyze_all();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| VWAP | Volume Weighted Average Price — бенчмарк справедливой цены |
| Типичная цена | (High + Low + Close) / 3 — используется в расчёте VWAP |
| Полосы VWAP | Полосы стандартного отклонения вокруг VWAP для сигналов перекупленности/перепроданности |
| Исполнение VWAP | Алгоритм, исполняющий ордера в соответствии с ожидаемым распределением объёма |
| Проскальзывание | Разница между ценой исполнения и бенчмарком (VWAP) |
| Базисные пункты (bps) | 1 bps = 0.01% — используется для измерения качества исполнения |

## Домашнее задание

1. **Anchored VWAP (Привязанный VWAP)**: Реализуй калькулятор привязанного VWAP, который позволяет начинать расчёт VWAP с любой точки (например, от значимого ценового уровня или события). Добавь метод для сброса точки привязки.

2. **Мультитаймфреймовый VWAP**: Создай систему, которая вычисляет VWAP одновременно на нескольких таймфреймах (1 минута, 5 минут, 1 час). Сравни, как отличаются сигналы на разных таймфреймах.

3. **Стратегия пересечения VWAP**: Реализуй торговую стратегию, которая:
   - Генерирует сигналы ПОКУПКИ, когда цена пересекает VWAP снизу вверх с растущим объёмом
   - Генерирует сигналы ПРОДАЖИ, когда цена пересекает VWAP сверху вниз с растущим объёмом
   - Отслеживает результаты сделок и вычисляет коэффициент Шарпа

4. **Алгоритм участия VWAP**: Построй продвинутый алгоритм исполнения VWAP, который:
   - Корректирует уровень участия в зависимости от волатильности рынка
   - Реализует логику "догоняния", если отстаёт от цели
   - Включает лимиты проскальзывания и автоматические выключатели
   - Отчитывается о метриках качества исполнения в реальном времени

## Навигация

[← Предыдущий день](../254-twap-time-weighted-average-price/ru.md) | [Следующий день →](../256-ema-exponential-moving-average/ru.md)
