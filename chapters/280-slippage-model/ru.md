# День 280: Модель проскальзывания (Slippage Model)

## Аналогия из трейдинга

Представь, что ты на оживлённом рынке пытаешься купить 100 яблок. На ценнике написано "1$ за яблоко", но когда ты хочешь купить все 100, продавец говорит: "У меня только 30 по $1. Следующие 40 будут стоить $1.10, а оставшиеся 30 — $1.25". Это движение цены, вызванное твоим собственным заказом, называется **проскальзывание (slippage)** — разница между ожидаемой ценой и реальной ценой исполнения.

В реальном трейдинге проскальзывание происходит потому что:
- Твой ордер потребляет доступную ликвидность по лучшей цене
- Рыночные условия меняются между размещением и исполнением ордера
- Крупные ордера двигают рынок против тебя
- Задержка сети приводит к изменению цены

Точное моделирование проскальзывания критически важно для бэктестинга — без него твоя симулированная прибыль может быть значительно выше, чем реальная!

## Что такое проскальзывание?

Проскальзывание — это разница между ожидаемой ценой сделки и ценой, по которой она реально исполняется. Оно может быть:

1. **Положительное проскальзывание** — исполнение по лучшей цене, чем ожидалось (редко, но возможно)
2. **Отрицательное проскальзывание** — исполнение по худшей цене, чем ожидалось (часто)
3. **Нулевое проскальзывание** — исполнение точно по ожидаемой цене (идеально, но нереалистично для крупных ордеров)

### Типы моделей проскальзывания

| Модель | Описание | Применение |
|--------|----------|------------|
| Фиксированная | Постоянные издержки на сделку | Простой бэктестинг |
| Процентная | Издержки как % от объёма сделки | Общие симуляции |
| Объёмная | Проскальзывание растёт с размером ордера | Реалистичное моделирование |
| На основе спреда | На основе спреда bid-ask | Высокочастотный трейдинг |
| Рыночное воздействие | Моделирует влияние крупных ордеров на цену | Институциональный трейдинг |

## Базовая модель проскальзывания

Начнём с простой структуры модели проскальзывания:

```rust
/// Представляет различные типы моделей проскальзывания
#[derive(Debug, Clone)]
pub enum SlippageModel {
    /// Без проскальзывания (идеальные условия)
    Zero,
    /// Фиксированные издержки на сделку в базисных пунктах (1 bp = 0.01%)
    Fixed { basis_points: f64 },
    /// Процент от объёма сделки
    Percentage { rate: f64 },
    /// Проскальзывание, зависящее от объёма
    VolumeImpact {
        base_spread: f64,      // Базовый спред bid-ask
        impact_factor: f64,    // Влияние на цену на единицу отношения объёма
    },
}

/// Сторона ордера для определения направления проскальзывания
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Представляет торговый ордер
#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub expected_price: f64,
}

/// Рыночные данные для расчёта проскальзывания
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub volume: f64,         // Текущий объём рынка
    pub avg_daily_volume: f64, // Средний дневной объём
}

impl SlippageModel {
    /// Рассчитать цену исполнения с учётом проскальзывания
    pub fn calculate_execution_price(
        &self,
        order: &Order,
        market: &MarketData,
    ) -> f64 {
        let slippage = self.calculate_slippage(order, market);

        match order.side {
            OrderSide::Buy => order.expected_price * (1.0 + slippage),
            OrderSide::Sell => order.expected_price * (1.0 - slippage),
        }
    }

    /// Рассчитать проскальзывание как десятичную дробь (0.01 = 1%)
    pub fn calculate_slippage(
        &self,
        order: &Order,
        market: &MarketData,
    ) -> f64 {
        match self {
            SlippageModel::Zero => 0.0,

            SlippageModel::Fixed { basis_points } => {
                basis_points / 10_000.0 // Конвертируем bp в десятичную дробь
            }

            SlippageModel::Percentage { rate } => *rate,

            SlippageModel::VolumeImpact {
                base_spread,
                impact_factor
            } => {
                // Рассчитываем отношение объёмов (размер ордера к среднедневному объёму)
                let volume_ratio = order.quantity / market.avg_daily_volume;

                // Проскальзывание = половина спреда + рыночное воздействие
                let half_spread = base_spread / 2.0;
                let market_impact = impact_factor * volume_ratio.sqrt();

                half_spread + market_impact
            }
        }
    }
}

fn main() {
    let order = Order {
        symbol: "BTC-USD".to_string(),
        side: OrderSide::Buy,
        quantity: 10.0,
        expected_price: 42000.0,
    };

    let market = MarketData {
        symbol: "BTC-USD".to_string(),
        bid: 41990.0,
        ask: 42010.0,
        volume: 1000.0,
        avg_daily_volume: 50000.0,
    };

    // Тестируем разные модели проскальзывания
    let models = vec![
        ("Нулевое", SlippageModel::Zero),
        ("Фикс. 5bp", SlippageModel::Fixed { basis_points: 5.0 }),
        ("0.1%", SlippageModel::Percentage { rate: 0.001 }),
        ("Объёмное", SlippageModel::VolumeImpact {
            base_spread: 0.0005,  // Спред 5 bp
            impact_factor: 0.01,  // 1% влияние при 100% ADV
        }),
    ];

    println!("Ордер: Покупка {} {} @ ${:.2}",
        order.quantity, order.symbol, order.expected_price);
    println!("\nСравнение моделей проскальзывания:");
    println!("{:-<65}", "");

    for (name, model) in &models {
        let exec_price = model.calculate_execution_price(&order, &market);
        let slippage_pct = model.calculate_slippage(&order, &market) * 100.0;
        let cost = (exec_price - order.expected_price) * order.quantity;

        println!(
            "{:<15} | Исп.: ${:.2} | Проск.: {:.3}% | Издержки: ${:.2}",
            name, exec_price, slippage_pct, cost
        );
    }
}
```

## Продвинутая модель объёмного воздействия

Для более реалистичного бэктестинга нужна сложная модель рыночного воздействия:

```rust
use std::collections::HashMap;

/// Продвинутая модель проскальзывания с учётом микроструктуры рынка
#[derive(Debug, Clone)]
pub struct AdvancedSlippageModel {
    /// Коэффициент временного воздействия (мгновенное влияние на цену)
    pub temporary_impact: f64,
    /// Коэффициент постоянного воздействия (длительное влияние на цену)
    pub permanent_impact: f64,
    /// Фактор волатильности
    pub volatility_factor: f64,
    /// Профили ликвидности по символам
    pub liquidity_profiles: HashMap<String, LiquidityProfile>,
}

#[derive(Debug, Clone)]
pub struct LiquidityProfile {
    pub avg_daily_volume: f64,
    pub avg_spread_bps: f64,
    pub volatility: f64,
    pub depth_at_best: f64, // Средний объём на лучшей цене bid/ask
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub order_id: u64,
    pub expected_price: f64,
    pub execution_price: f64,
    pub slippage_bps: f64,
    pub market_impact: f64,
    pub total_cost: f64,
}

impl AdvancedSlippageModel {
    pub fn new() -> Self {
        AdvancedSlippageModel {
            temporary_impact: 0.1,
            permanent_impact: 0.05,
            volatility_factor: 1.0,
            liquidity_profiles: HashMap::new(),
        }
    }

    pub fn with_liquidity_profile(
        mut self,
        symbol: &str,
        profile: LiquidityProfile,
    ) -> Self {
        self.liquidity_profiles.insert(symbol.to_string(), profile);
        self
    }

    /// Рассчитать исполнение по модели рыночного воздействия Almgren-Chriss
    pub fn simulate_execution(
        &self,
        order: &Order,
        market: &MarketData,
        order_id: u64,
    ) -> ExecutionResult {
        let profile = self.liquidity_profiles
            .get(&order.symbol)
            .cloned()
            .unwrap_or(LiquidityProfile {
                avg_daily_volume: market.avg_daily_volume,
                avg_spread_bps: 10.0,
                volatility: 0.02,
                depth_at_best: market.volume * 0.01,
            });

        // Доля участия в объёме
        let participation = order.quantity / profile.avg_daily_volume;

        // Издержки спреда (половина спреда)
        let spread_cost = profile.avg_spread_bps / 20_000.0; // Конвертируем в десятичную дробь

        // Временное воздействие: мгновенное движение цены
        let temp_impact = self.temporary_impact
            * profile.volatility
            * (participation).sqrt();

        // Постоянное воздействие: длительное движение цены
        let perm_impact = self.permanent_impact
            * profile.volatility
            * participation;

        // Общее проскальзывание
        let total_slippage = spread_cost + temp_impact + perm_impact;

        // Рассчитываем цену исполнения в зависимости от стороны
        let execution_price = match order.side {
            OrderSide::Buy => order.expected_price * (1.0 + total_slippage),
            OrderSide::Sell => order.expected_price * (1.0 - total_slippage),
        };

        let slippage_bps = total_slippage * 10_000.0;
        let total_cost = (execution_price - order.expected_price).abs()
            * order.quantity;

        ExecutionResult {
            order_id,
            expected_price: order.expected_price,
            execution_price,
            slippage_bps,
            market_impact: temp_impact + perm_impact,
            total_cost,
        }
    }
}

impl Default for AdvancedSlippageModel {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    // Создаём модель с профилем ликвидности BTC
    let model = AdvancedSlippageModel::new()
        .with_liquidity_profile("BTC-USD", LiquidityProfile {
            avg_daily_volume: 50_000.0, // 50k BTC в день
            avg_spread_bps: 5.0,        // Спред 5 bp
            volatility: 0.03,           // 3% дневная волатильность
            depth_at_best: 10.0,        // 10 BTC на лучшей цене
        });

    let market = MarketData {
        symbol: "BTC-USD".to_string(),
        bid: 41990.0,
        ask: 42010.0,
        volume: 1000.0,
        avg_daily_volume: 50_000.0,
    };

    // Тестируем с разными размерами ордеров
    let order_sizes = vec![1.0, 10.0, 100.0, 500.0, 1000.0];

    println!("Продвинутая модель проскальзывания - Влияние размера ордера");
    println!("{:-<75}", "");
    println!(
        "{:<10} | {:<14} | {:<12} | {:<10} | {:<12}",
        "Размер", "Цена исп.", "Проскальз.", "Воздейств.", "Издержки ($)"
    );
    println!("{:-<75}", "");

    for (i, &size) in order_sizes.iter().enumerate() {
        let order = Order {
            symbol: "BTC-USD".to_string(),
            side: OrderSide::Buy,
            quantity: size,
            expected_price: 42000.0,
        };

        let result = model.simulate_execution(&order, &market, i as u64);

        println!(
            "{:<10.1} | ${:<13.2} | {:<10.2} bp | {:<9.4}% | ${:<11.2}",
            size,
            result.execution_price,
            result.slippage_bps,
            result.market_impact * 100.0,
            result.total_cost
        );
    }
}
```

## Проскальзывание в движке бэктестинга

Вот как интегрировать проскальзывание в торговую симуляцию:

```rust
use std::collections::VecDeque;

/// Исполнение сделки с отслеживанием проскальзывания
#[derive(Debug, Clone)]
pub struct Trade {
    pub id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub expected_price: f64,
    pub execution_price: f64,
    pub slippage_cost: f64,
    pub timestamp: u64,
}

/// Движок бэктестинга с моделью проскальзывания
pub struct BacktestEngine {
    pub slippage_model: SlippageModel,
    pub trades: Vec<Trade>,
    pub cash: f64,
    pub positions: HashMap<String, f64>,
    pub total_slippage_cost: f64,
    trade_counter: u64,
}

impl BacktestEngine {
    pub fn new(initial_cash: f64, slippage_model: SlippageModel) -> Self {
        BacktestEngine {
            slippage_model,
            trades: Vec::new(),
            cash: initial_cash,
            positions: HashMap::new(),
            total_slippage_cost: 0.0,
            trade_counter: 0,
        }
    }

    /// Исполнить ордер с проскальзыванием
    pub fn execute_order(
        &mut self,
        order: Order,
        market: &MarketData,
        timestamp: u64,
    ) -> Result<Trade, String> {
        // Рассчитываем цену исполнения с проскальзыванием
        let execution_price = self.slippage_model
            .calculate_execution_price(&order, market);

        let trade_value = execution_price * order.quantity;
        let slippage_cost = (execution_price - order.expected_price).abs()
            * order.quantity;

        // Проверяем достаточность капитала/позиции
        match order.side {
            OrderSide::Buy => {
                if self.cash < trade_value {
                    return Err(format!(
                        "Недостаточно средств: нужно ${:.2}, есть ${:.2}",
                        trade_value, self.cash
                    ));
                }
                self.cash -= trade_value;
                *self.positions.entry(order.symbol.clone()).or_insert(0.0)
                    += order.quantity;
            }
            OrderSide::Sell => {
                let position = self.positions.get(&order.symbol).unwrap_or(&0.0);
                if *position < order.quantity {
                    return Err(format!(
                        "Недостаточно позиции: нужно {}, есть {}",
                        order.quantity, position
                    ));
                }
                self.cash += trade_value;
                *self.positions.entry(order.symbol.clone()).or_insert(0.0)
                    -= order.quantity;
            }
        }

        self.trade_counter += 1;
        self.total_slippage_cost += slippage_cost;

        let trade = Trade {
            id: self.trade_counter,
            symbol: order.symbol,
            side: order.side,
            quantity: order.quantity,
            expected_price: order.expected_price,
            execution_price,
            slippage_cost,
            timestamp,
        };

        self.trades.push(trade.clone());
        Ok(trade)
    }

    /// Получить статистику проскальзывания
    pub fn get_slippage_stats(&self) -> SlippageStats {
        if self.trades.is_empty() {
            return SlippageStats::default();
        }

        let slippages: Vec<f64> = self.trades.iter()
            .map(|t| {
                let slippage_pct = (t.execution_price - t.expected_price)
                    / t.expected_price * 100.0;
                match t.side {
                    OrderSide::Buy => slippage_pct,
                    OrderSide::Sell => -slippage_pct,
                }
            })
            .collect();

        let avg_slippage = slippages.iter().sum::<f64>() / slippages.len() as f64;
        let max_slippage = slippages.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_slippage = slippages.iter().cloned().fold(f64::INFINITY, f64::min);

        SlippageStats {
            total_trades: self.trades.len(),
            total_slippage_cost: self.total_slippage_cost,
            avg_slippage_pct: avg_slippage,
            max_slippage_pct: max_slippage,
            min_slippage_pct: min_slippage,
        }
    }
}

#[derive(Debug, Default)]
pub struct SlippageStats {
    pub total_trades: usize,
    pub total_slippage_cost: f64,
    pub avg_slippage_pct: f64,
    pub max_slippage_pct: f64,
    pub min_slippage_pct: f64,
}

fn main() {
    // Сравниваем результаты бэктеста с разными моделями проскальзывания
    let initial_cash = 100_000.0;

    let models = vec![
        ("Без проскальзывания", SlippageModel::Zero),
        ("Фикс. 10bp", SlippageModel::Fixed { basis_points: 10.0 }),
        ("Объёмное воздействие", SlippageModel::VolumeImpact {
            base_spread: 0.001,
            impact_factor: 0.02,
        }),
    ];

    let market = MarketData {
        symbol: "BTC-USD".to_string(),
        bid: 41990.0,
        ask: 42010.0,
        volume: 1000.0,
        avg_daily_volume: 50_000.0,
    };

    println!("Сравнение проскальзывания в бэктесте");
    println!("Симуляция 100 круговых сделок (покупка + продажа)\n");

    for (name, model) in models {
        let mut engine = BacktestEngine::new(initial_cash, model);

        // Симулируем 100 круговых сделок
        for i in 0..100 {
            let timestamp = i as u64;
            let price = 42000.0 + (i as f64 * 10.0).sin() * 500.0; // Вариация цены

            // Покупка
            let buy_order = Order {
                symbol: "BTC-USD".to_string(),
                side: OrderSide::Buy,
                quantity: 0.5,
                expected_price: price,
            };
            let _ = engine.execute_order(buy_order, &market, timestamp);

            // Продажа по слегка более высокой цене
            let sell_order = Order {
                symbol: "BTC-USD".to_string(),
                side: OrderSide::Sell,
                quantity: 0.5,
                expected_price: price * 1.002, // Цель прибыли 0.2%
            };
            let _ = engine.execute_order(sell_order, &market, timestamp);
        }

        let stats = engine.get_slippage_stats();
        println!("Модель: {}", name);
        println!("  Итоговые средства: ${:.2}", engine.cash);
        println!("  Общие издержки проскальзывания: ${:.2}", stats.total_slippage_cost);
        println!("  Среднее проскальзывание: {:.4}%", stats.avg_slippage_pct);
        println!("  Чистый P&L: ${:.2}", engine.cash - initial_cash);
        println!();
    }
}
```

## Реалистичное проскальзывание с симуляцией стакана заявок

Для наиболее точного моделирования проскальзывания мы можем симулировать потребление стакана заявок:

```rust
use std::collections::BTreeMap;

/// Представляет ценовой уровень в стакане заявок
#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
}

/// Симулированный стакан заявок для реалистичного проскальзывания
#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: BTreeMap<i64, PriceLevel>, // Цена в центах как ключ (по убыванию)
    pub asks: BTreeMap<i64, PriceLevel>, // Цена в центах как ключ (по возрастанию)
}

impl OrderBook {
    pub fn new(symbol: &str) -> Self {
        OrderBook {
            symbol: symbol.to_string(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Создать пример стакана заявок с ликвидностью
    pub fn with_sample_liquidity(mut self, mid_price: f64) -> Self {
        // Генерируем уровни bid (по убыванию от середины)
        for i in 1..=10 {
            let price = mid_price - (i as f64 * 0.5);
            let quantity = 5.0 * i as f64; // Больше ликвидности дальше от середины
            self.bids.insert(
                (-price * 100.0) as i64, // Отрицательный для порядка по убыванию
                PriceLevel { price, quantity },
            );
        }

        // Генерируем уровни ask (по возрастанию от середины)
        for i in 1..=10 {
            let price = mid_price + (i as f64 * 0.5);
            let quantity = 5.0 * i as f64;
            self.asks.insert(
                (price * 100.0) as i64,
                PriceLevel { price, quantity },
            );
        }

        self
    }

    /// Симулировать исполнение рыночного ордера с потреблением уровней стакана
    pub fn execute_market_order(
        &self,
        side: OrderSide,
        quantity: f64,
    ) -> OrderBookExecution {
        let mut remaining = quantity;
        let mut fills: Vec<(f64, f64)> = Vec::new();
        let mut total_cost = 0.0;

        let levels: Vec<&PriceLevel> = match side {
            OrderSide::Buy => self.asks.values().collect(),
            OrderSide::Sell => self.bids.values().collect(),
        };

        for level in levels {
            if remaining <= 0.0 {
                break;
            }

            let fill_qty = remaining.min(level.quantity);
            fills.push((level.price, fill_qty));
            total_cost += level.price * fill_qty;
            remaining -= fill_qty;
        }

        let filled_quantity = quantity - remaining;
        let avg_price = if filled_quantity > 0.0 {
            total_cost / filled_quantity
        } else {
            0.0
        };

        let best_price = match side {
            OrderSide::Buy => self.asks.values().next().map(|l| l.price).unwrap_or(0.0),
            OrderSide::Sell => self.bids.values().next().map(|l| l.price).unwrap_or(0.0),
        };

        let slippage = if best_price > 0.0 {
            match side {
                OrderSide::Buy => (avg_price - best_price) / best_price,
                OrderSide::Sell => (best_price - avg_price) / best_price,
            }
        } else {
            0.0
        };

        OrderBookExecution {
            requested_quantity: quantity,
            filled_quantity,
            unfilled_quantity: remaining,
            avg_execution_price: avg_price,
            best_available_price: best_price,
            slippage_pct: slippage * 100.0,
            fills,
        }
    }
}

#[derive(Debug)]
pub struct OrderBookExecution {
    pub requested_quantity: f64,
    pub filled_quantity: f64,
    pub unfilled_quantity: f64,
    pub avg_execution_price: f64,
    pub best_available_price: f64,
    pub slippage_pct: f64,
    pub fills: Vec<(f64, f64)>, // пары (цена, количество)
}

fn main() {
    let order_book = OrderBook::new("BTC-USD")
        .with_sample_liquidity(42000.0);

    println!("Симуляция проскальзывания по стакану заявок");
    println!("Средняя цена: $42,000.00\n");

    // Показываем стакан заявок
    println!("Уровни Ask (продают тебе):");
    for level in order_book.asks.values().take(5) {
        println!("  ${:.2} x {:.1}", level.price, level.quantity);
    }
    println!("\nУровни Bid (покупают у тебя):");
    for level in order_book.bids.values().take(5) {
        println!("  ${:.2} x {:.1}", level.price, level.quantity);
    }

    println!("\n{:-<60}", "");
    println!("Исполнение рыночных ордеров:\n");

    // Тестируем разные размеры ордеров
    let order_sizes = vec![5.0, 20.0, 50.0, 100.0];

    for size in order_sizes {
        let execution = order_book.execute_market_order(OrderSide::Buy, size);

        println!("Покупка {} BTC:", size);
        println!("  Лучшая цена: ${:.2}", execution.best_available_price);
        println!("  Средняя цена исп.: ${:.2}", execution.avg_execution_price);
        println!("  Проскальзывание: {:.3}%", execution.slippage_pct);
        println!("  Исполнено: {:.1} / Не исполнено: {:.1}",
            execution.filled_quantity,
            execution.unfilled_quantity
        );
        println!("  Сделки: {:?}", execution.fills.iter()
            .map(|(p, q)| format!("{:.1}@${:.2}", q, p))
            .collect::<Vec<_>>()
        );
        println!();
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Проскальзывание | Разница между ожидаемой и реальной ценой исполнения |
| Фиксированная модель | Постоянное проскальзывание на сделку (просто, но нереалистично) |
| Объёмное воздействие | Проскальзывание растёт с размером ордера |
| Рыночное воздействие | Временные + постоянные эффекты на цену от торговли |
| Модель стакана | Самая реалистичная — симулирует потребление ликвидности |
| Издержки спреда | Половина спреда bid-ask добавляется к издержкам исполнения |

## Практические упражнения

1. **Сравнение моделей**: Используя базовую модель проскальзывания, рассчитай цену исполнения для ордера на покупку BTC на $1 миллион по цене $42,000 с каждым типом проскальзывания. Какая модель даёт самую консервативную (наихудшую) оценку?

2. **Корректировка на волатильность**: Модифицируй модель `VolumeImpact`, добавив множитель волатильности — в периоды высокой волатильности (волатильность > 5%) проскальзывание должно быть в 2 раза выше.

3. **Асимметричное проскальзывание**: Реализуй модель проскальзывания, где продажа имеет на 20% большее проскальзывание, чем покупка (отражая типичные рыночные условия во время распродаж).

4. **Фактор времени дня**: Создай модель проскальзывания, которая увеличивает проскальзывание на 50% во время открытия рынка (первые 30 минут) и закрытия (последние 30 минут), когда ликвидность обычно ниже.

## Домашнее задание

1. **Исторический анализ проскальзывания**: Создай структуру `SlippageAnalyzer`, которая:
   - Отслеживает ожидаемые vs реальные цены исполнения во времени
   - Рассчитывает скользящую статистику (среднее, стд. откл., макс. проскальзывание)
   - Выявляет паттерны (большее проскальзывание для крупных ордеров, определённое время)
   - Выводит сводный отчёт

2. **Адаптивная модель проскальзывания**: Реализуй модель проскальзывания, которая:
   - Обучается на исторических данных исполнения
   - Корректирует параметры на основе недавних рыночных условий
   - Использует скользящее среднее реализованного проскальзывания для калибровки прогнозов
   - Включает доверительный интервал для оценок проскальзывания

3. **Оптимизатор проскальзывания TWAP**: Создай симулятор исполнения Time-Weighted Average Price (TWAP), который:
   - Разбивает крупные ордера на меньшие части
   - Исполняет их за период времени для минимизации рыночного воздействия
   - Сравнивает проскальзывание между единичным исполнением и TWAP
   - Находит оптимальное количество частей для разных размеров ордеров

4. **Полная интеграция бэктеста**: Построй полноценный фреймворк бэктестинга, который:
   - Использует модель проскальзывания по стакану заявок
   - Симулирует реалистичное восстановление стакана между сделками
   - Отслеживает проскальзывание как отдельный компонент P&L
   - Генерирует отчёт, показывающий "идеальный P&L" vs "реалистичный P&L с проскальзыванием"

## Навигация

[← Предыдущий день](../279-position-sizing-model/ru.md) | [Следующий день →](../281-commission-model/ru.md)
