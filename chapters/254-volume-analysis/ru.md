# День 254: Анализ объёма торгов

## Аналогия из трейдинга

Представь, что ты трейдер, наблюдающий за потоком ордеров на бирже. Цена растёт, но сильное это движение или слабое? Ответ кроется в **объёме** — общем количестве акций, контрактов или монет, проторгованных за определённый период. Если цена растёт на высоком объёме, покупатели контролируют ситуацию и тренд силён. Если цена растёт на низком объёме, это может быть «ложный пробой» — слабое ралли, которое может развернуться.

Объём — как толпа на аукционе: растущая цена с громкими ставками (высокий объём) означает реальный спрос, а растущая цена в тихом зале (низкий объём) говорит о слабой уверенности участников.

В алготрейдинге анализ объёма помогает:
- Подтверждать ценовые тренды и пробои
- Обнаруживать потенциальные развороты (дивергенция)
- Идентифицировать фазы накопления и распределения
- Оценивать ликвидность рынка и издержки влияния

## Что такое анализ объёма?

Анализ объёма — это изучение торгового объёма вместе с движениями цены для принятия лучших торговых решений. Ключевые концепции:

1. **Свечи объёма** — объём торгов за каждый временной период
2. **Скользящая средняя объёма** — сглаженный объём для выявления активности выше/ниже среднего
3. **Связь объёма и цены** — подтверждение трендов при совместном движении цены и объёма
4. **Профиль объёма** — распределение объёма по ценовым уровням
5. **On-Balance Volume (OBV)** — кумулятивный индикатор, связывающий объём с направлением цены

## Базовая структура данных объёма

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct VolumeBar {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u32,
}

impl VolumeBar {
    pub fn new(timestamp: u64, price: f64, volume: f64) -> Self {
        VolumeBar {
            timestamp,
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            trade_count: 1,
        }
    }

    pub fn update(&mut self, price: f64, volume: f64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += volume;
        self.trade_count += 1;
    }

    /// Рассчитать типичную цену (среднее high, low, close)
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Средневзвешенная по объёму цена для этого бара
    pub fn vwap(&self) -> f64 {
        self.typical_price() // Упрощённо; реальный VWAP требует тиковых данных
    }

    /// Это бычий бар (close > open)?
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }
}

fn main() {
    let mut bar = VolumeBar::new(1704067200, 42000.0, 0.5);
    bar.update(42100.0, 0.3);
    bar.update(41950.0, 0.2);
    bar.update(42200.0, 0.8);

    println!("Бар объёма: {:?}", bar);
    println!("Общий объём: {:.2}", bar.volume);
    println!("Типичная цена: {:.2}", bar.typical_price());
    println!("Бычий бар: {}", bar.is_bullish());
}
```

## Скользящая средняя объёма

```rust
use std::collections::VecDeque;

pub struct VolumeMovingAverage {
    period: usize,
    volumes: VecDeque<f64>,
    sum: f64,
}

impl VolumeMovingAverage {
    pub fn new(period: usize) -> Self {
        VolumeMovingAverage {
            period,
            volumes: VecDeque::with_capacity(period),
            sum: 0.0,
        }
    }

    pub fn update(&mut self, volume: f64) -> Option<f64> {
        self.volumes.push_back(volume);
        self.sum += volume;

        if self.volumes.len() > self.period {
            if let Some(old) = self.volumes.pop_front() {
                self.sum -= old;
            }
        }

        if self.volumes.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }

    pub fn current(&self) -> Option<f64> {
        if self.volumes.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }

    /// Проверить, превышает ли текущий объём средний (сигнал силы)
    pub fn is_above_average(&self, current_volume: f64) -> Option<bool> {
        self.current().map(|avg| current_volume > avg)
    }

    /// Рассчитать коэффициент объёма (текущий / средний)
    pub fn volume_ratio(&self, current_volume: f64) -> Option<f64> {
        self.current().map(|avg| {
            if avg > 0.0 {
                current_volume / avg
            } else {
                1.0
            }
        })
    }
}

fn main() {
    let mut vma = VolumeMovingAverage::new(5);

    let volumes = vec![100.0, 150.0, 120.0, 180.0, 200.0, 300.0, 250.0];

    for (i, &vol) in volumes.iter().enumerate() {
        let avg = vma.update(vol);
        let ratio = vma.volume_ratio(vol);

        println!(
            "Бар {}: Объём={:.0}, Среднее={:?}, Коэффициент={:.2}x",
            i + 1,
            vol,
            avg.map(|a| format!("{:.0}", a)),
            ratio.unwrap_or(1.0)
        );

        if let Some(true) = vma.is_above_average(vol) {
            println!("  -> Объём выше среднего! Потенциальный сигнал.");
        }
    }
}
```

## On-Balance Volume (OBV)

On-Balance Volume — кумулятивный индикатор, который прибавляет объём в дни роста и вычитает в дни падения:

```rust
#[derive(Debug)]
pub struct OnBalanceVolume {
    obv: f64,
    previous_close: Option<f64>,
    history: Vec<f64>,
}

impl OnBalanceVolume {
    pub fn new() -> Self {
        OnBalanceVolume {
            obv: 0.0,
            previous_close: None,
            history: Vec::new(),
        }
    }

    pub fn update(&mut self, close: f64, volume: f64) -> f64 {
        if let Some(prev_close) = self.previous_close {
            if close > prev_close {
                // Цена вверх: добавляем объём (покупатели контролируют)
                self.obv += volume;
            } else if close < prev_close {
                // Цена вниз: вычитаем объём (продавцы контролируют)
                self.obv -= volume;
            }
            // Если close == prev_close, OBV остаётся прежним
        }

        self.previous_close = Some(close);
        self.history.push(self.obv);
        self.obv
    }

    pub fn current(&self) -> f64 {
        self.obv
    }

    /// Обнаружить дивергенцию OBV с ценой
    /// Возвращает Some(true) для бычьей дивергенции, Some(false) для медвежьей
    pub fn check_divergence(&self, price_making_new_low: bool, price_making_new_high: bool) -> Option<bool> {
        if self.history.len() < 2 {
            return None;
        }

        let recent_obv = self.obv;
        let prev_obv = self.history[self.history.len() - 2];

        // Бычья дивергенция: цена делает новый минимум, но OBV — нет
        if price_making_new_low && recent_obv > prev_obv {
            return Some(true);
        }

        // Медвежья дивергенция: цена делает новый максимум, но OBV — нет
        if price_making_new_high && recent_obv < prev_obv {
            return Some(false);
        }

        None
    }
}

impl Default for OnBalanceVolume {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut obv = OnBalanceVolume::new();

    // Симулированные данные цены и объёма (восходящий тренд с подтверждением объёмом)
    let data = vec![
        (100.0, 1000.0),
        (102.0, 1500.0), // Вверх, добавляем объём
        (101.0, 800.0),  // Вниз, вычитаем объём
        (104.0, 2000.0), // Вверх, добавляем объём
        (106.0, 2500.0), // Вверх, добавляем объём
        (105.0, 1200.0), // Вниз, вычитаем объём
    ];

    println!("Анализ On-Balance Volume:");
    println!("{:-<50}", "");

    for (i, (close, volume)) in data.iter().enumerate() {
        let obv_value = obv.update(*close, *volume);
        let direction = if i > 0 && *close > data[i - 1].0 {
            "ВВЕРХ"
        } else if i > 0 && *close < data[i - 1].0 {
            "ВНИЗ"
        } else {
            "СТАРТ"
        };

        println!(
            "Бар {}: Close={:.2}, Объём={:.0}, Направление={}, OBV={:.0}",
            i + 1,
            close,
            volume,
            direction,
            obv_value
        );
    }

    println!("{:-<50}", "");
    println!("Итоговый OBV: {:.0}", obv.current());
    println!("Положительный OBV указывает на доминирование покупателей.");
}
```

## Профиль объёма

Профиль объёма показывает, сколько объёма было проторговано на каждом ценовом уровне:

```rust
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct VolumeProfile {
    /// Ценовой уровень -> Объём, проторгованный на этом уровне
    profile: BTreeMap<i64, f64>,
    tick_size: f64,
    total_volume: f64,
}

impl VolumeProfile {
    pub fn new(tick_size: f64) -> Self {
        VolumeProfile {
            profile: BTreeMap::new(),
            tick_size,
            total_volume: 0.0,
        }
    }

    /// Преобразовать цену в ценовой уровень (корзину)
    fn price_to_level(&self, price: f64) -> i64 {
        (price / self.tick_size).round() as i64
    }

    /// Добавить объём на определённой цене
    pub fn add_volume(&mut self, price: f64, volume: f64) {
        let level = self.price_to_level(price);
        *self.profile.entry(level).or_insert(0.0) += volume;
        self.total_volume += volume;
    }

    /// Получить Point of Control (цена с максимальным объёмом)
    pub fn point_of_control(&self) -> Option<(f64, f64)> {
        self.profile
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(&level, &volume)| (level as f64 * self.tick_size, volume))
    }

    /// Получить Value Area (ценовой диапазон, содержащий 70% объёма)
    pub fn value_area(&self) -> Option<(f64, f64)> {
        if self.profile.is_empty() {
            return None;
        }

        let target_volume = self.total_volume * 0.70;
        let poc_level = self.profile
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(&l, _)| l)?;

        let mut included_volume = *self.profile.get(&poc_level).unwrap_or(&0.0);
        let mut low_level = poc_level;
        let mut high_level = poc_level;

        // Расширяем от POC пока не наберём 70% объёма
        while included_volume < target_volume {
            let below = self.profile.get(&(low_level - 1)).unwrap_or(&0.0);
            let above = self.profile.get(&(high_level + 1)).unwrap_or(&0.0);

            if below >= above && *below > 0.0 {
                low_level -= 1;
                included_volume += below;
            } else if *above > 0.0 {
                high_level += 1;
                included_volume += above;
            } else {
                break;
            }
        }

        Some((
            low_level as f64 * self.tick_size,
            high_level as f64 * self.tick_size,
        ))
    }

    /// Вывести профиль объёма как гистограмму
    pub fn print_histogram(&self, max_width: usize) {
        if self.profile.is_empty() {
            println!("Нет данных об объёме");
            return;
        }

        let max_volume = self.profile.values().cloned().fold(0.0_f64, f64::max);
        let poc = self.point_of_control();

        println!("\nПрофиль объёма:");
        println!("{:-<60}", "");

        for (&level, &volume) in self.profile.iter().rev() {
            let price = level as f64 * self.tick_size;
            let bar_len = ((volume / max_volume) * max_width as f64) as usize;
            let bar: String = "#".repeat(bar_len);
            let is_poc = poc.map(|(p, _)| (p - price).abs() < self.tick_size / 2.0).unwrap_or(false);

            println!(
                "${:>8.2} | {:width$} {:.0}{}",
                price,
                bar,
                volume,
                if is_poc { " <- POC" } else { "" },
                width = max_width
            );
        }
    }
}

fn main() {
    let mut profile = VolumeProfile::new(100.0); // Корзины по $100

    // Симуляция сделок на различных ценовых уровнях
    let trades = vec![
        (41900.0, 50.0),
        (42000.0, 150.0),
        (42000.0, 200.0),
        (42100.0, 180.0),
        (42100.0, 120.0),
        (42100.0, 100.0),
        (42200.0, 80.0),
        (42200.0, 90.0),
        (42300.0, 40.0),
        (42000.0, 100.0),
    ];

    for (price, volume) in trades {
        profile.add_volume(price, volume);
    }

    profile.print_histogram(30);

    if let Some((poc_price, poc_volume)) = profile.point_of_control() {
        println!("\nPoint of Control: ${:.2} ({:.0} объёма)", poc_price, poc_volume);
    }

    if let Some((va_low, va_high)) = profile.value_area() {
        println!("Value Area: ${:.2} - ${:.2}", va_low, va_high);
    }
}
```

## Торговые сигналы на основе объёма

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum VolumeSignal {
    StrongBuy,      // Высокий объём + цена вверх
    WeakBuy,        // Низкий объём + цена вверх (потенциальный разворот)
    StrongSell,     // Высокий объём + цена вниз
    WeakSell,       // Низкий объём + цена вниз (потенциальный разворот)
    Accumulation,   // Высокий объём + стабильная цена (умные деньги покупают)
    Distribution,   // Высокий объём + стабильная цена (умные деньги продают)
    Neutral,
}

pub struct VolumeAnalyzer {
    volume_ma: VolumeMovingAverage,
    obv: OnBalanceVolume,
    volume_threshold: f64, // Множитель для "высокого объёма" (например, 1.5x от среднего)
}

impl VolumeAnalyzer {
    pub fn new(ma_period: usize, volume_threshold: f64) -> Self {
        VolumeAnalyzer {
            volume_ma: VolumeMovingAverage::new(ma_period),
            obv: OnBalanceVolume::new(),
            volume_threshold,
        }
    }

    pub fn analyze(&mut self, bar: &VolumeBar) -> VolumeSignal {
        // Обновляем индикаторы
        self.volume_ma.update(bar.volume);
        self.obv.update(bar.close, bar.volume);

        // Рассчитываем коэффициент объёма
        let volume_ratio = self.volume_ma.volume_ratio(bar.volume).unwrap_or(1.0);
        let is_high_volume = volume_ratio >= self.volume_threshold;

        // Рассчитываем изменение цены
        let price_change_pct = if bar.open > 0.0 {
            (bar.close - bar.open) / bar.open * 100.0
        } else {
            0.0
        };

        // Определяем сигнал на основе объёма и ценового действия
        match (is_high_volume, price_change_pct) {
            (true, pct) if pct > 0.5 => VolumeSignal::StrongBuy,
            (true, pct) if pct < -0.5 => VolumeSignal::StrongSell,
            (true, pct) if pct.abs() <= 0.5 => {
                // Высокий объём, но стабильная цена — накопление или распределение
                if self.obv.current() > 0.0 {
                    VolumeSignal::Accumulation
                } else {
                    VolumeSignal::Distribution
                }
            }
            (false, pct) if pct > 0.5 => VolumeSignal::WeakBuy,
            (false, pct) if pct < -0.5 => VolumeSignal::WeakSell,
            _ => VolumeSignal::Neutral,
        }
    }
}

// Включаем необходимые структуры из примеров выше
use std::collections::VecDeque;

pub struct VolumeMovingAverage {
    period: usize,
    volumes: VecDeque<f64>,
    sum: f64,
}

impl VolumeMovingAverage {
    pub fn new(period: usize) -> Self {
        VolumeMovingAverage {
            period,
            volumes: VecDeque::with_capacity(period),
            sum: 0.0,
        }
    }

    pub fn update(&mut self, volume: f64) -> Option<f64> {
        self.volumes.push_back(volume);
        self.sum += volume;
        if self.volumes.len() > self.period {
            if let Some(old) = self.volumes.pop_front() {
                self.sum -= old;
            }
        }
        if self.volumes.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }

    pub fn volume_ratio(&self, current_volume: f64) -> Option<f64> {
        if self.volumes.len() == self.period && self.sum > 0.0 {
            Some(current_volume / (self.sum / self.period as f64))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct OnBalanceVolume {
    obv: f64,
    previous_close: Option<f64>,
}

impl OnBalanceVolume {
    pub fn new() -> Self {
        OnBalanceVolume {
            obv: 0.0,
            previous_close: None,
        }
    }

    pub fn update(&mut self, close: f64, volume: f64) -> f64 {
        if let Some(prev) = self.previous_close {
            if close > prev {
                self.obv += volume;
            } else if close < prev {
                self.obv -= volume;
            }
        }
        self.previous_close = Some(close);
        self.obv
    }

    pub fn current(&self) -> f64 {
        self.obv
    }
}

#[derive(Debug, Clone)]
pub struct VolumeBar {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: u32,
}

fn main() {
    let mut analyzer = VolumeAnalyzer::new(5, 1.5);

    // Симуляция рыночных данных
    let bars = vec![
        VolumeBar { timestamp: 1, open: 100.0, high: 101.0, low: 99.5, close: 100.5, volume: 1000.0, trade_count: 50 },
        VolumeBar { timestamp: 2, open: 100.5, high: 102.0, low: 100.0, close: 101.8, volume: 1200.0, trade_count: 60 },
        VolumeBar { timestamp: 3, open: 101.8, high: 103.0, low: 101.5, close: 102.5, volume: 1100.0, trade_count: 55 },
        VolumeBar { timestamp: 4, open: 102.5, high: 103.5, low: 102.0, close: 103.2, volume: 1500.0, trade_count: 75 },
        VolumeBar { timestamp: 5, open: 103.2, high: 105.0, low: 103.0, close: 104.8, volume: 2500.0, trade_count: 120 }, // Пробой на высоком объёме
        VolumeBar { timestamp: 6, open: 104.8, high: 106.0, low: 104.5, close: 105.5, volume: 800.0, trade_count: 40 },  // Продолжение на низком объёме
        VolumeBar { timestamp: 7, open: 105.5, high: 106.0, low: 103.0, close: 103.5, volume: 3000.0, trade_count: 150 }, // Разворот на высоком объёме
    ];

    println!("Результаты анализа объёма:");
    println!("{:-<60}", "");

    for bar in &bars {
        let signal = analyzer.analyze(bar);
        let change = (bar.close - bar.open) / bar.open * 100.0;

        println!(
            "Бар {}: Close={:.2} ({:+.2}%), Объём={:.0} -> {:?}",
            bar.timestamp, bar.close, change, bar.volume, signal
        );
    }
}
```

## Практический пример: Торговая стратегия на основе объёма

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Trade {
    pub timestamp: u64,
    pub side: TradeSide,
    pub price: f64,
    pub quantity: f64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct VolumeStrategy {
    // Конфигурация
    volume_ma_period: usize,
    volume_spike_threshold: f64,
    position_size: f64,

    // Состояние
    volume_history: VecDeque<f64>,
    price_history: VecDeque<f64>,
    position: f64,
    trades: Vec<Trade>,
}

impl VolumeStrategy {
    pub fn new(volume_ma_period: usize, spike_threshold: f64, position_size: f64) -> Self {
        VolumeStrategy {
            volume_ma_period,
            volume_spike_threshold: spike_threshold,
            position_size,
            volume_history: VecDeque::with_capacity(volume_ma_period),
            price_history: VecDeque::with_capacity(volume_ma_period),
            position: 0.0,
            trades: Vec::new(),
        }
    }

    fn average_volume(&self) -> Option<f64> {
        if self.volume_history.len() == self.volume_ma_period {
            Some(self.volume_history.iter().sum::<f64>() / self.volume_ma_period as f64)
        } else {
            None
        }
    }

    fn price_trend(&self) -> Option<f64> {
        if self.price_history.len() < 2 {
            return None;
        }
        let first = self.price_history.front()?;
        let last = self.price_history.back()?;
        Some((last - first) / first * 100.0)
    }

    pub fn on_bar(&mut self, timestamp: u64, close: f64, volume: f64) -> Option<Trade> {
        // Обновляем историю
        self.volume_history.push_back(volume);
        self.price_history.push_back(close);

        if self.volume_history.len() > self.volume_ma_period {
            self.volume_history.pop_front();
            self.price_history.pop_front();
        }

        // Нужно достаточно данных
        let avg_volume = self.average_volume()?;
        let trend = self.price_trend()?;

        let volume_ratio = volume / avg_volume;
        let is_volume_spike = volume_ratio >= self.volume_spike_threshold;

        // Логика стратегии
        let trade = if is_volume_spike && trend > 1.0 && self.position <= 0.0 {
            // Всплеск объёма с восходящим трендом — сигнал на покупку
            Some(Trade {
                timestamp,
                side: TradeSide::Buy,
                price: close,
                quantity: self.position_size,
                reason: format!(
                    "Всплеск объёма ({:.1}x от среднего) с восходящим трендом ({:+.2}%)",
                    volume_ratio, trend
                ),
            })
        } else if is_volume_spike && trend < -1.0 && self.position >= 0.0 {
            // Всплеск объёма с нисходящим трендом — сигнал на продажу
            Some(Trade {
                timestamp,
                side: TradeSide::Sell,
                price: close,
                quantity: self.position_size,
                reason: format!(
                    "Всплеск объёма ({:.1}x от среднего) с нисходящим трендом ({:+.2}%)",
                    volume_ratio, trend
                ),
            })
        } else if !is_volume_spike && self.position != 0.0 && trend.abs() < 0.5 {
            // Низкий объём и боковик — выход из позиции
            let side = if self.position > 0.0 { TradeSide::Sell } else { TradeSide::Buy };
            Some(Trade {
                timestamp,
                side,
                price: close,
                quantity: self.position.abs(),
                reason: "Выход на низком объёме — импульс затухает".to_string(),
            })
        } else {
            None
        };

        // Обновляем позицию
        if let Some(ref t) = trade {
            match t.side {
                TradeSide::Buy => self.position += t.quantity,
                TradeSide::Sell => self.position -= t.quantity,
            }
            self.trades.push(t.clone());
        }

        trade
    }

    pub fn get_trades(&self) -> &[Trade] {
        &self.trades
    }

    pub fn calculate_pnl(&self) -> f64 {
        let mut pnl = 0.0;
        let mut position = 0.0;
        let mut avg_entry = 0.0;

        for trade in &self.trades {
            match trade.side {
                TradeSide::Buy => {
                    if position < 0.0 {
                        // Закрытие шорта
                        pnl += (avg_entry - trade.price) * trade.quantity.min(-position);
                    }
                    let new_position = position + trade.quantity;
                    if new_position > 0.0 && position >= 0.0 {
                        avg_entry = (avg_entry * position + trade.price * trade.quantity)
                            / new_position;
                    } else if new_position > 0.0 {
                        avg_entry = trade.price;
                    }
                    position = new_position;
                }
                TradeSide::Sell => {
                    if position > 0.0 {
                        // Закрытие лонга
                        pnl += (trade.price - avg_entry) * trade.quantity.min(position);
                    }
                    let new_position = position - trade.quantity;
                    if new_position < 0.0 && position <= 0.0 {
                        avg_entry = (avg_entry * (-position) + trade.price * trade.quantity)
                            / (-new_position);
                    } else if new_position < 0.0 {
                        avg_entry = trade.price;
                    }
                    position = new_position;
                }
            }
        }

        pnl
    }
}

fn main() {
    let mut strategy = VolumeStrategy::new(5, 1.8, 1.0);

    // Симулированные рыночные данные: (timestamp, close, volume)
    let market_data = vec![
        (1, 100.0, 1000.0),
        (2, 100.5, 1100.0),
        (3, 101.0, 1050.0),
        (4, 101.5, 1200.0),
        (5, 102.0, 1150.0),
        (6, 103.0, 2500.0), // Всплеск объёма + восходящий тренд -> ПОКУПКА
        (7, 103.5, 1300.0),
        (8, 104.0, 1400.0),
        (9, 104.2, 800.0),  // Низкий объём
        (10, 104.0, 750.0), // Низкий объём, боковик -> ВЫХОД
        (11, 103.5, 1100.0),
        (12, 102.0, 2800.0), // Всплеск объёма + нисходящий тренд -> ПРОДАЖА
        (13, 101.0, 1500.0),
        (14, 100.5, 1200.0),
        (15, 100.8, 600.0),  // Низкий объём, боковик -> ВЫХОД
    ];

    println!("Торговая стратегия на основе объёма");
    println!("{:=<60}", "");

    for (timestamp, close, volume) in market_data {
        if let Some(trade) = strategy.on_bar(timestamp, close, volume) {
            println!(
                "\n[Бар {}] СДЕЛКА: {:?} {:.2} @ ${:.2}",
                timestamp, trade.side, trade.quantity, trade.price
            );
            println!("         Причина: {}", trade.reason);
        }
    }

    println!("\n{:=<60}", "");
    println!("Всего сделок: {}", strategy.get_trades().len());
    println!("P&L: ${:.2}", strategy.calculate_pnl());
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Volume Bar | Структура данных OHLCV для представления свечей |
| Скользящая средняя объёма | Сглаженный объём для выявления периодов высокой/низкой активности |
| On-Balance Volume (OBV) | Кумулятивный индикатор объёма, показывающий давление покупателей/продавцов |
| Профиль объёма | Распределение объёма по ценовым уровням |
| Point of Control (POC) | Ценовой уровень с наибольшим проторгованным объёмом |
| Value Area | Ценовой диапазон, содержащий 70% объёма |
| Всплеск объёма | Объём значительно выше среднего (сигнал интереса) |
| Дивергенция объёма и цены | Когда цена и объём не подтверждают друг друга |

## Домашнее задание

1. **Калькулятор VWAP**: Реализуй калькулятор Volume-Weighted Average Price (VWAP), который:
   - Принимает поток сделок (цена, объём, временная метка)
   - Рассчитывает скользящий VWAP в течение торгового дня
   - Сбрасывается на открытии рынка
   - Показывает, когда цена выше/ниже VWAP

2. **Детектор пробоев по объёму**: Создай систему, которая:
   - Мониторит объём в реальном времени
   - Обнаруживает, когда объём превышает 2x от 20-периодного среднего
   - Классифицирует пробои как бычьи/медвежьи на основе ценового действия
   - Логирует алерты с временными метками и коэффициентами объёма

3. **Торговые уровни по профилю объёма**: Построй `TradingLevelFinder`, который:
   - Строит профиль объёма по историческим данным
   - Определяет High Volume Nodes (HVN) как поддержку/сопротивление
   - Определяет Low Volume Nodes (LVN) как потенциальные зоны пробоя
   - Возвращает список ключевых ценовых уровней для торговли

4. **Мультитаймфреймовый анализ объёма**: Реализуй анализатор, который:
   - Агрегирует данные об объёме по нескольким таймфреймам (1м, 5м, 15м, 1ч)
   - Обнаруживает, когда сигналы объёма совпадают на разных таймфреймах
   - Генерирует более сильные сигналы при подтверждении на нескольких таймфреймах
   - Использует `Arc<Mutex<>>` для потокобезопасного общего состояния между таймфреймами

## Навигация

[← Предыдущий день](../253-momentum-indicators/ru.md) | [Следующий день →](../255-price-action-patterns/ru.md)
