# День 248: EMA: экспоненциальная скользящая средняя

## Аналогия из трейдинга

В предыдущем уроке мы изучили SMA (простую скользящую среднюю), где все цены имеют одинаковый вес. Но представь ситуацию: ты анализируешь цену биткоина, и вчерашняя цена для тебя важнее, чем цена месяц назад. Рынок меняется, и свежие данные должны иметь больше влияния на твои решения.

**EMA (Exponential Moving Average)** — это скользящая средняя, которая придаёт **больший вес последним ценам**. Это как если бы при опросе трейдеров мнение тех, кто торговал вчера, значило больше, чем мнение тех, кто торговал месяц назад.

**Почему EMA важнее SMA для трейдинга:**
- Быстрее реагирует на изменения цены
- Лучше улавливает тренды
- Меньше запаздывает при резких движениях рынка
- Используется в популярных индикаторах (MACD, Bollinger Bands)

## Формула EMA

EMA рассчитывается по рекуррентной формуле:

```
EMA_сегодня = Цена_сегодня × k + EMA_вчера × (1 - k)
```

где **k** — коэффициент сглаживания (мультипликатор):

```
k = 2 / (период + 1)
```

Например, для EMA-10:
- k = 2 / (10 + 1) = 2 / 11 ≈ 0.1818

Это означает, что последняя цена получает ~18% веса, а все предыдущие значения EMA — ~82%.

## Простая реализация EMA

```rust
fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let mut ema_values = Vec::with_capacity(prices.len() - period + 1);

    // Коэффициент сглаживания
    let k = 2.0 / (period as f64 + 1.0);

    // Первое значение EMA = SMA за первый период
    let first_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;
    ema_values.push(first_sma);

    // Рассчитываем остальные значения EMA
    for i in period..prices.len() {
        let prev_ema = ema_values.last().unwrap();
        let current_ema = prices[i] * k + prev_ema * (1.0 - k);
        ema_values.push(current_ema);
    }

    ema_values
}

fn main() {
    // Цены закрытия BTC за 15 дней
    let prices = vec![
        42000.0, 42500.0, 42300.0, 42800.0, 43000.0,
        42700.0, 42900.0, 43200.0, 43500.0, 43300.0,
        43600.0, 43400.0, 43800.0, 44000.0, 44200.0,
    ];

    let ema_10 = calculate_ema(&prices, 10);

    println!("=== EMA-10 для BTC ===");
    for (i, ema) in ema_10.iter().enumerate() {
        println!("День {}: EMA = ${:.2}", i + 10, ema);
    }
}
```

## Сравнение EMA и SMA

```rust
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    prices
        .windows(period)
        .map(|window| window.iter().sum::<f64>() / period as f64)
        .collect()
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let k = 2.0 / (period as f64 + 1.0);
    let first_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;

    let mut ema_values = vec![first_sma];

    for i in period..prices.len() {
        let prev_ema = ema_values.last().unwrap();
        let current_ema = prices[i] * k + prev_ema * (1.0 - k);
        ema_values.push(current_ema);
    }

    ema_values
}

fn main() {
    // Симулируем резкий рост цены
    let prices = vec![
        100.0, 100.0, 100.0, 100.0, 100.0,  // Стабильная цена
        100.0, 100.0, 100.0, 100.0, 100.0,
        120.0, 125.0, 130.0, 128.0, 135.0,  // Резкий рост
    ];

    let sma_5 = calculate_sma(&prices, 5);
    let ema_5 = calculate_ema(&prices, 5);

    println!("=== Сравнение SMA-5 и EMA-5 при резком росте ===\n");
    println!("{:<10} {:>10} {:>10} {:>10}", "День", "Цена", "SMA-5", "EMA-5");
    println!("{}", "-".repeat(45));

    for i in 0..sma_5.len() {
        let day = i + 5;
        let price = prices[day - 1];
        println!(
            "{:<10} {:>10.2} {:>10.2} {:>10.2}",
            day, price, sma_5[i], ema_5[i]
        );
    }

    println!("\nОбрати внимание: EMA быстрее реагирует на рост цены!");
}
```

## Структура индикатора EMA

```rust
#[derive(Debug, Clone)]
pub struct EmaIndicator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    prices_count: usize,
    initial_sum: f64,
}

impl EmaIndicator {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Период должен быть больше 0");

        EmaIndicator {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current_ema: None,
            prices_count: 0,
            initial_sum: 0.0,
        }
    }

    /// Добавляет новую цену и возвращает текущее значение EMA (если доступно)
    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.prices_count += 1;

        match self.current_ema {
            None => {
                // Накапливаем данные для первого SMA
                self.initial_sum += price;

                if self.prices_count >= self.period {
                    // Первое значение EMA = SMA
                    let first_ema = self.initial_sum / self.period as f64;
                    self.current_ema = Some(first_ema);
                    Some(first_ema)
                } else {
                    None
                }
            }
            Some(prev_ema) => {
                // Рассчитываем новое значение EMA
                let new_ema = price * self.multiplier + prev_ema * (1.0 - self.multiplier);
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
        }
    }

    /// Возвращает текущее значение EMA без добавления новой цены
    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }

    /// Проверяет, готов ли индикатор (накоплено достаточно данных)
    pub fn is_ready(&self) -> bool {
        self.current_ema.is_some()
    }

    /// Сбрасывает индикатор
    pub fn reset(&mut self) {
        self.current_ema = None;
        self.prices_count = 0;
        self.initial_sum = 0.0;
    }
}

fn main() {
    let mut ema = EmaIndicator::new(5);

    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
    ];

    println!("=== Потоковый расчёт EMA-5 ===\n");

    for (i, &price) in prices.iter().enumerate() {
        let ema_value = ema.update(price);

        match ema_value {
            Some(value) => {
                println!("День {}: Цена = ${:.2}, EMA-5 = ${:.2}", i + 1, price, value);
            }
            None => {
                println!("День {}: Цена = ${:.2}, EMA-5 = (накопление данных...)", i + 1, price);
            }
        }
    }
}
```

## EMA Crossover: торговая стратегия

Одна из самых популярных стратегий — пересечение быстрой и медленной EMA:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug)]
pub struct EmaCrossover {
    fast_ema: EmaIndicator,
    slow_ema: EmaIndicator,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct EmaIndicator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    prices_count: usize,
    initial_sum: f64,
}

impl EmaIndicator {
    pub fn new(period: usize) -> Self {
        EmaIndicator {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current_ema: None,
            prices_count: 0,
            initial_sum: 0.0,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.prices_count += 1;

        match self.current_ema {
            None => {
                self.initial_sum += price;
                if self.prices_count >= self.period {
                    let first_ema = self.initial_sum / self.period as f64;
                    self.current_ema = Some(first_ema);
                    Some(first_ema)
                } else {
                    None
                }
            }
            Some(prev_ema) => {
                let new_ema = price * self.multiplier + prev_ema * (1.0 - self.multiplier);
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
        }
    }

    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }
}

impl EmaCrossover {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period < slow_period, "Быстрый период должен быть меньше медленного");

        EmaCrossover {
            fast_ema: EmaIndicator::new(fast_period),
            slow_ema: EmaIndicator::new(slow_period),
            prev_fast: None,
            prev_slow: None,
        }
    }

    pub fn update(&mut self, price: f64) -> Signal {
        let fast = self.fast_ema.update(price);
        let slow = self.slow_ema.update(price);

        let signal = match (fast, slow, self.prev_fast, self.prev_slow) {
            (Some(f), Some(s), Some(pf), Some(ps)) => {
                // Быстрая пересекает медленную снизу вверх = сигнал на покупку
                if pf <= ps && f > s {
                    Signal::Buy
                }
                // Быстрая пересекает медленную сверху вниз = сигнал на продажу
                else if pf >= ps && f < s {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        self.prev_fast = fast;
        self.prev_slow = slow;

        signal
    }

    pub fn fast_ema(&self) -> Option<f64> {
        self.fast_ema.value()
    }

    pub fn slow_ema(&self) -> Option<f64> {
        self.slow_ema.value()
    }
}

fn main() {
    let mut strategy = EmaCrossover::new(5, 10);

    // Симулируем движение цены: сначала рост, потом падение
    let prices = vec![
        // Начальный период
        100.0, 101.0, 102.0, 101.5, 103.0,
        104.0, 105.0, 106.0, 107.0, 108.0,
        // Рост (должен быть сигнал Buy)
        110.0, 112.0, 115.0, 118.0, 120.0,
        // Разворот и падение (должен быть сигнал Sell)
        118.0, 115.0, 112.0, 108.0, 105.0,
        102.0, 100.0, 98.0, 95.0, 92.0,
    ];

    println!("=== EMA Crossover Strategy (5/10) ===\n");
    println!("{:<6} {:>10} {:>12} {:>12} {:>10}", "День", "Цена", "EMA-5", "EMA-10", "Сигнал");
    println!("{}", "-".repeat(55));

    for (i, &price) in prices.iter().enumerate() {
        let signal = strategy.update(price);

        let fast_str = strategy.fast_ema()
            .map(|v| format!("{:.2}", v))
            .unwrap_or_else(|| "---".to_string());

        let slow_str = strategy.slow_ema()
            .map(|v| format!("{:.2}", v))
            .unwrap_or_else(|| "---".to_string());

        let signal_str = match signal {
            Signal::Buy => ">>> BUY <<<",
            Signal::Sell => "<<< SELL >>>",
            Signal::Hold => "",
        };

        println!(
            "{:<6} {:>10.2} {:>12} {:>12} {:>10}",
            i + 1, price, fast_str, slow_str, signal_str
        );
    }
}
```

## Мульти-таймфрейм EMA анализ

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EmaIndicator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    prices_count: usize,
    initial_sum: f64,
}

impl EmaIndicator {
    pub fn new(period: usize) -> Self {
        EmaIndicator {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current_ema: None,
            prices_count: 0,
            initial_sum: 0.0,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.prices_count += 1;

        match self.current_ema {
            None => {
                self.initial_sum += price;
                if self.prices_count >= self.period {
                    let first_ema = self.initial_sum / self.period as f64;
                    self.current_ema = Some(first_ema);
                    Some(first_ema)
                } else {
                    None
                }
            }
            Some(prev_ema) => {
                let new_ema = price * self.multiplier + prev_ema * (1.0 - self.multiplier);
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
        }
    }

    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }
}

#[derive(Debug)]
pub struct MultiEmaAnalyzer {
    emas: HashMap<usize, EmaIndicator>,
}

impl MultiEmaAnalyzer {
    pub fn new(periods: &[usize]) -> Self {
        let mut emas = HashMap::new();
        for &period in periods {
            emas.insert(period, EmaIndicator::new(period));
        }
        MultiEmaAnalyzer { emas }
    }

    pub fn update(&mut self, price: f64) {
        for ema in self.emas.values_mut() {
            ema.update(price);
        }
    }

    pub fn get_ema(&self, period: usize) -> Option<f64> {
        self.emas.get(&period).and_then(|ema| ema.value())
    }

    /// Определяет тренд на основе расположения EMA
    pub fn analyze_trend(&self) -> TrendAnalysis {
        let mut sorted_periods: Vec<_> = self.emas.keys().copied().collect();
        sorted_periods.sort();

        let mut values: Vec<(usize, f64)> = vec![];
        for &period in &sorted_periods {
            if let Some(value) = self.get_ema(period) {
                values.push((period, value));
            }
        }

        if values.len() < 2 {
            return TrendAnalysis::Unknown;
        }

        // Проверяем, отсортированы ли EMA по возрастанию (бычий тренд)
        // или по убыванию (медвежий тренд)
        let is_bullish = values.windows(2).all(|w| w[0].1 > w[1].1);
        let is_bearish = values.windows(2).all(|w| w[0].1 < w[1].1);

        if is_bullish {
            TrendAnalysis::Bullish
        } else if is_bearish {
            TrendAnalysis::Bearish
        } else {
            TrendAnalysis::Sideways
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendAnalysis {
    Bullish,   // Быстрые EMA выше медленных
    Bearish,   // Быстрые EMA ниже медленных
    Sideways,  // EMA переплетены
    Unknown,   // Недостаточно данных
}

fn main() {
    let mut analyzer = MultiEmaAnalyzer::new(&[9, 21, 55, 200]);

    // Симулируем устойчивый рост
    let prices: Vec<f64> = (0..250)
        .map(|i| 100.0 + (i as f64) * 0.5 + (i as f64 * 0.1).sin() * 5.0)
        .collect();

    println!("=== Мульти-EMA анализ ===\n");

    for (i, &price) in prices.iter().enumerate() {
        analyzer.update(price);

        // Выводим каждые 50 дней
        if (i + 1) % 50 == 0 {
            println!("День {}:", i + 1);
            println!("  Цена: ${:.2}", price);

            for &period in &[9, 21, 55, 200] {
                if let Some(ema) = analyzer.get_ema(period) {
                    println!("  EMA-{}: ${:.2}", period, ema);
                }
            }

            println!("  Тренд: {:?}", analyzer.analyze_trend());
            println!();
        }
    }
}
```

## Практический пример: торговый бот с EMA

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct EmaIndicator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    prices_count: usize,
    initial_sum: f64,
}

impl EmaIndicator {
    pub fn new(period: usize) -> Self {
        EmaIndicator {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
            current_ema: None,
            prices_count: 0,
            initial_sum: 0.0,
        }
    }

    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.prices_count += 1;

        match self.current_ema {
            None => {
                self.initial_sum += price;
                if self.prices_count >= self.period {
                    let first_ema = self.initial_sum / self.period as f64;
                    self.current_ema = Some(first_ema);
                    Some(first_ema)
                } else {
                    None
                }
            }
            Some(prev_ema) => {
                let new_ema = price * self.multiplier + prev_ema * (1.0 - self.multiplier);
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
        }
    }

    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Trade {
    entry_price: f64,
    exit_price: Option<f64>,
    position_size: f64,
    side: TradeSide,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSide {
    Long,
    Short,
}

impl Trade {
    pub fn pnl(&self) -> Option<f64> {
        self.exit_price.map(|exit| {
            match self.side {
                TradeSide::Long => (exit - self.entry_price) * self.position_size,
                TradeSide::Short => (self.entry_price - exit) * self.position_size,
            }
        })
    }
}

#[derive(Debug)]
pub struct EmaBot {
    fast_ema: EmaIndicator,
    slow_ema: EmaIndicator,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
    current_position: Option<Trade>,
    closed_trades: Vec<Trade>,
    initial_capital: f64,
    current_capital: f64,
    risk_per_trade: f64,  // Процент капитала на сделку
}

impl EmaBot {
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        initial_capital: f64,
        risk_per_trade: f64,
    ) -> Self {
        EmaBot {
            fast_ema: EmaIndicator::new(fast_period),
            slow_ema: EmaIndicator::new(slow_period),
            prev_fast: None,
            prev_slow: None,
            current_position: None,
            closed_trades: vec![],
            initial_capital,
            current_capital: initial_capital,
            risk_per_trade,
        }
    }

    pub fn on_price(&mut self, price: f64) {
        let fast = self.fast_ema.update(price);
        let slow = self.slow_ema.update(price);

        if let (Some(f), Some(s), Some(pf), Some(ps)) = (fast, slow, self.prev_fast, self.prev_slow) {
            // Golden Cross: быстрая EMA пересекает медленную снизу
            if pf <= ps && f > s {
                self.close_position_if_exists(price);
                self.open_long(price);
            }
            // Death Cross: быстрая EMA пересекает медленную сверху
            else if pf >= ps && f < s {
                self.close_position_if_exists(price);
                self.open_short(price);
            }
        }

        self.prev_fast = fast;
        self.prev_slow = slow;
    }

    fn open_long(&mut self, price: f64) {
        let position_value = self.current_capital * self.risk_per_trade;
        let size = position_value / price;

        self.current_position = Some(Trade {
            entry_price: price,
            exit_price: None,
            position_size: size,
            side: TradeSide::Long,
        });

        println!("  → ОТКРЫТА ДЛИННАЯ позиция: {} BTC @ ${:.2}", size, price);
    }

    fn open_short(&mut self, price: f64) {
        let position_value = self.current_capital * self.risk_per_trade;
        let size = position_value / price;

        self.current_position = Some(Trade {
            entry_price: price,
            exit_price: None,
            position_size: size,
            side: TradeSide::Short,
        });

        println!("  → ОТКРЫТА КОРОТКАЯ позиция: {} BTC @ ${:.2}", size, price);
    }

    fn close_position_if_exists(&mut self, price: f64) {
        if let Some(mut trade) = self.current_position.take() {
            trade.exit_price = Some(price);

            if let Some(pnl) = trade.pnl() {
                self.current_capital += pnl;
                println!(
                    "  ← ЗАКРЫТА позиция @ ${:.2}, PnL: ${:.2}",
                    price, pnl
                );
            }

            self.closed_trades.push(trade);
        }
    }

    pub fn close_all(&mut self, price: f64) {
        self.close_position_if_exists(price);
    }

    pub fn report(&self) {
        println!("\n=== ОТЧЁТ ===");
        println!("Начальный капитал: ${:.2}", self.initial_capital);
        println!("Текущий капитал: ${:.2}", self.current_capital);

        let total_pnl = self.current_capital - self.initial_capital;
        let return_pct = (total_pnl / self.initial_capital) * 100.0;

        println!("Общий PnL: ${:.2} ({:.2}%)", total_pnl, return_pct);
        println!("Всего сделок: {}", self.closed_trades.len());

        let winning = self.closed_trades.iter()
            .filter(|t| t.pnl().unwrap_or(0.0) > 0.0)
            .count();

        if !self.closed_trades.is_empty() {
            let win_rate = (winning as f64 / self.closed_trades.len() as f64) * 100.0;
            println!("Win Rate: {:.1}%", win_rate);
        }
    }
}

fn main() {
    let mut bot = EmaBot::new(9, 21, 10000.0, 0.5);  // 50% капитала на сделку

    // Симулируем волатильный рынок
    let prices: Vec<f64> = vec![
        // Начало
        100.0, 101.0, 99.0, 102.0, 100.0, 103.0, 101.0, 104.0, 102.0, 105.0,
        103.0, 106.0, 104.0, 107.0, 105.0, 108.0, 106.0, 109.0, 107.0, 110.0,
        108.0, 111.0,
        // Устойчивый рост
        112.0, 115.0, 118.0, 120.0, 123.0, 125.0, 128.0, 130.0, 133.0, 135.0,
        // Разворот и падение
        132.0, 128.0, 125.0, 120.0, 115.0, 110.0, 105.0, 100.0, 95.0, 90.0,
        // Восстановление
        92.0, 95.0, 98.0, 100.0, 103.0, 106.0, 110.0, 115.0, 120.0, 125.0,
    ];

    println!("=== EMA Trading Bot (9/21) ===\n");

    for (i, &price) in prices.iter().enumerate() {
        println!("День {}: Цена = ${:.2}", i + 1, price);
        bot.on_price(price);
    }

    // Закрываем все позиции по последней цене
    bot.close_all(*prices.last().unwrap());

    bot.report();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| EMA | Экспоненциальная скользящая средняя — даёт больший вес последним ценам |
| Мультипликатор k | k = 2 / (период + 1), определяет вес последней цены |
| EMA формула | EMA = Цена × k + EMA_пред × (1 - k) |
| EMA vs SMA | EMA быстрее реагирует на изменения цены |
| EMA Crossover | Стратегия на пересечении быстрой и медленной EMA |
| Golden Cross | Быстрая EMA пересекает медленную снизу — сигнал на покупку |
| Death Cross | Быстрая EMA пересекает медленную сверху — сигнал на продажу |

## Домашнее задание

1. **Оптимизация периодов**: Напиши функцию, которая перебирает разные комбинации периодов EMA (например, 5-10, 9-21, 12-26) на исторических данных и находит наиболее прибыльную комбинацию.

2. **EMA с фильтром тренда**: Добавь к стратегии EMA Crossover фильтр на основе EMA-200. Открывай только длинные позиции, когда цена выше EMA-200, и только короткие — когда ниже.

3. **Адаптивная EMA**: Реализуй KAMA (Kaufman Adaptive Moving Average), где период EMA автоматически изменяется в зависимости от волатильности рынка.

4. **Тройной EMA**: Создай стратегию с тремя EMA (быстрая, средняя, медленная). Сигнал на покупку появляется только когда все три EMA выстроены в правильном порядке (быстрая > средняя > медленная).

## Навигация

[← Предыдущий день](../247-sma-simple-moving-average/ru.md) | [Следующий день →](../249-rsi-relative-strength-index/ru.md)
