# День 251: Полосы Боллинджера — каналы волатильности для трейдинга

## Аналогия из трейдинга

Представь, что ты наблюдаешь за колебаниями цены BTC. Иногда цена движется в узком диапазоне (низкая волатильность), иногда резко скачет (высокая волатильность). Полосы Боллинджера — это **динамические границы**, которые расширяются и сужаются в зависимости от волатильности рынка — они помогают трейдерам определять, когда цена относительно высока или низка по сравнению с недавней историей.

Думай об этом как о реке: берега (верхняя и нижняя полосы) расширяются во время паводка (высокая волатильность) и сужаются в спокойные периоды. Когда цена касается верхнего берега, она может быть перекуплена; когда касается нижнего — может быть перепродана.

## Что такое полосы Боллинджера?

Полосы Боллинджера состоят из трёх линий:
1. **Средняя полоса**: простая скользящая средняя (обычно 20 периодов)
2. **Верхняя полоса**: средняя полоса + (стандартное отклонение × множитель)
3. **Нижняя полоса**: средняя полоса - (стандартное отклонение × множитель)

Стандартный множитель — 2.0, то есть полосы находятся на расстоянии 2 стандартных отклонений от среднего.

## Базовая реализация

```rust
fn main() {
    let prices = [
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
        42800.0, 42750.0, 42900.0, 42850.0, 43000.0,
    ];

    if let Some(bands) = calculate_bollinger_bands(&prices, 20, 2.0) {
        println!("Полосы Боллинджера BTC:");
        println!("  Верхняя полоса:  ${:.2}", bands.upper);
        println!("  Средняя полоса:  ${:.2}", bands.middle);
        println!("  Нижняя полоса:   ${:.2}", bands.lower);
        println!("  Ширина канала:   {:.4}", bands.bandwidth);
    }
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

fn calculate_bollinger_bands(prices: &[f64], period: usize, multiplier: f64) -> Option<BollingerBands> {
    if prices.len() < period {
        return None;
    }

    let slice = &prices[prices.len() - period..];

    // Расчёт SMA (средняя полоса)
    let sma: f64 = slice.iter().sum::<f64>() / period as f64;

    // Расчёт стандартного отклонения
    let variance: f64 = slice
        .iter()
        .map(|price| (price - sma).powi(2))
        .sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();

    // Расчёт полос
    let upper = sma + (std_dev * multiplier);
    let lower = sma - (std_dev * multiplier);
    let bandwidth = (upper - lower) / sma;  // Нормализованная ширина

    Some(BollingerBands {
        upper,
        middle: sma,
        lower,
        bandwidth,
    })
}
```

## Расчёт скользящих полос Боллинджера

Для реального трейдинга нужны полосы, рассчитанные в каждой точке времени:

```rust
fn main() {
    let btc_prices = [
        41000.0, 41200.0, 41100.0, 41300.0, 41250.0,
        41400.0, 41350.0, 41500.0, 41450.0, 41600.0,
        41700.0, 41650.0, 41800.0, 41750.0, 41900.0,
        42000.0, 42100.0, 42050.0, 42200.0, 42300.0,
        42500.0, 42400.0, 42600.0, 42800.0, 43000.0,
    ];

    let bands_series = calculate_rolling_bollinger(&btc_prices, 10, 2.0);

    println!("Скользящие полосы Боллинджера (последние 5 значений):");
    println!("{:>10} {:>12} {:>12} {:>12} {:>10}",
             "Цена", "Верхняя", "Средняя", "Нижняя", "Позиция");

    for (i, bands) in bands_series.iter().rev().take(5).rev().enumerate() {
        let idx = btc_prices.len() - 5 + i;
        let price = btc_prices[idx];
        let position = calculate_position(price, bands);
        println!("{:>10.2} {:>12.2} {:>12.2} {:>12.2} {:>10.2}%",
                 price, bands.upper, bands.middle, bands.lower, position * 100.0);
    }
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

fn calculate_rolling_bollinger(prices: &[f64], period: usize, multiplier: f64) -> Vec<BollingerBands> {
    let mut result = Vec::new();

    for i in period..=prices.len() {
        let slice = &prices[i - period..i];

        let sma: f64 = slice.iter().sum::<f64>() / period as f64;
        let variance: f64 = slice
            .iter()
            .map(|p| (p - sma).powi(2))
            .sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();

        let upper = sma + (std_dev * multiplier);
        let lower = sma - (std_dev * multiplier);

        result.push(BollingerBands {
            upper,
            middle: sma,
            lower,
            bandwidth: (upper - lower) / sma,
        });
    }

    result
}

fn calculate_position(price: f64, bands: &BollingerBands) -> f64 {
    // Возвращает позицию в полосах: 0.0 = нижняя полоса, 1.0 = верхняя полоса
    (price - bands.lower) / (bands.upper - bands.lower)
}
```

## Торговые сигналы с полосами Боллинджера

```rust
fn main() {
    let prices = [
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
        42800.0, 42750.0, 42900.0, 42850.0, 43200.0,  // Скачок цены
    ];

    let period = 20;
    let multiplier = 2.0;

    if let Some(bands) = calculate_bollinger_bands(&prices, period, multiplier) {
        let current_price = prices[prices.len() - 1];
        let signal = generate_signal(current_price, &bands);

        println!("Текущая цена BTC: ${:.2}", current_price);
        println!("Полосы Боллинджера:");
        println!("  Верхняя:  ${:.2}", bands.upper);
        println!("  Средняя:  ${:.2}", bands.middle);
        println!("  Нижняя:   ${:.2}", bands.lower);
        println!("\nСигнал: {:?}", signal);
        println!("Причина: {}", get_signal_reason(current_price, &bands));
    }
}

#[derive(Debug)]
enum TradingSignal {
    StrongBuy,   // Сильная покупка
    Buy,         // Покупка
    Hold,        // Удержание
    Sell,        // Продажа
    StrongSell,  // Сильная продажа
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

fn calculate_bollinger_bands(prices: &[f64], period: usize, multiplier: f64) -> Option<BollingerBands> {
    if prices.len() < period {
        return None;
    }

    let slice = &prices[prices.len() - period..];
    let sma: f64 = slice.iter().sum::<f64>() / period as f64;
    let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();

    let upper = sma + (std_dev * multiplier);
    let lower = sma - (std_dev * multiplier);

    Some(BollingerBands {
        upper,
        middle: sma,
        lower,
        bandwidth: (upper - lower) / sma,
    })
}

fn generate_signal(price: f64, bands: &BollingerBands) -> TradingSignal {
    let position = (price - bands.lower) / (bands.upper - bands.lower);

    if price < bands.lower {
        TradingSignal::StrongBuy  // Цена ниже нижней полосы - перепродано
    } else if position < 0.2 {
        TradingSignal::Buy        // Около нижней полосы
    } else if price > bands.upper {
        TradingSignal::StrongSell // Цена выше верхней полосы - перекуплено
    } else if position > 0.8 {
        TradingSignal::Sell       // Около верхней полосы
    } else {
        TradingSignal::Hold       // Цена в среднем диапазоне
    }
}

fn get_signal_reason(price: f64, bands: &BollingerBands) -> String {
    if price < bands.lower {
        format!("Цена ${:.2} ниже нижней полосы ${:.2} - возможное состояние перепроданности",
                price, bands.lower)
    } else if price > bands.upper {
        format!("Цена ${:.2} выше верхней полосы ${:.2} - возможное состояние перекупленности",
                price, bands.upper)
    } else {
        let position = ((price - bands.lower) / (bands.upper - bands.lower)) * 100.0;
        format!("Цена на {:.1}% диапазона полос - в пределах нормы", position)
    }
}
```

## Обнаружение сжатия полос Боллинджера

"Сжатие" происходит, когда полосы значительно сужаются, часто предшествуя сильному движению:

```rust
fn main() {
    // Симуляция сценария сжатия
    let prices = [
        42000.0, 42050.0, 42025.0, 42075.0, 42050.0,
        42060.0, 42040.0, 42055.0, 42045.0, 42052.0,
        42048.0, 42051.0, 42049.0, 42050.0, 42050.0,  // Очень узкий диапазон
        42051.0, 42049.0, 42050.0, 42050.0, 42050.0,  // Сжатие!
    ];

    let bands_history = calculate_bands_history(&prices, 10, 2.0);

    if let Some(squeeze) = detect_squeeze(&bands_history, 0.02) {
        println!("ОБНАРУЖЕНО СЖАТИЕ!");
        println!("Текущая ширина канала: {:.4}", squeeze.current_bandwidth);
        println!("Средняя ширина канала: {:.4}", squeeze.avg_bandwidth);
        println!("Коэффициент сжатия: {:.2}%", squeeze.squeeze_ratio * 100.0);
        println!("\nТорговое значение: Ожидайте увеличения волатильности!");
    } else {
        println!("Сжатие не обнаружено - нормальная волатильность");
    }
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

struct SqueezeInfo {
    current_bandwidth: f64,
    avg_bandwidth: f64,
    squeeze_ratio: f64,
}

fn calculate_bands_history(prices: &[f64], period: usize, multiplier: f64) -> Vec<BollingerBands> {
    let mut result = Vec::new();

    for i in period..=prices.len() {
        let slice = &prices[i - period..i];
        let sma: f64 = slice.iter().sum::<f64>() / period as f64;
        let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();

        let upper = sma + (std_dev * multiplier);
        let lower = sma - (std_dev * multiplier);

        result.push(BollingerBands {
            upper,
            middle: sma,
            lower,
            bandwidth: (upper - lower) / sma,
        });
    }

    result
}

fn detect_squeeze(bands_history: &[BollingerBands], threshold: f64) -> Option<SqueezeInfo> {
    if bands_history.is_empty() {
        return None;
    }

    let avg_bandwidth: f64 = bands_history.iter().map(|b| b.bandwidth).sum::<f64>()
                            / bands_history.len() as f64;
    let current_bandwidth = bands_history.last()?.bandwidth;
    let squeeze_ratio = current_bandwidth / avg_bandwidth;

    if squeeze_ratio < (1.0 - threshold) {
        Some(SqueezeInfo {
            current_bandwidth,
            avg_bandwidth,
            squeeze_ratio,
        })
    } else {
        None
    }
}
```

## Индикатор %B — позиция внутри полос

```rust
fn main() {
    let prices = [
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
        42800.0, 42750.0, 42900.0, 42850.0, 43000.0,
    ];

    let period = 20;
    let multiplier = 2.0;

    if let Some(bands) = calculate_bollinger_bands(&prices, period, multiplier) {
        let current_price = prices[prices.len() - 1];
        let percent_b = calculate_percent_b(current_price, &bands);

        println!("Цена BTC: ${:.2}", current_price);
        println!("Значение %B: {:.4}", percent_b);
        println!();
        interpret_percent_b(percent_b);
    }
}

struct BollingerBands {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
}

fn calculate_bollinger_bands(prices: &[f64], period: usize, multiplier: f64) -> Option<BollingerBands> {
    if prices.len() < period {
        return None;
    }

    let slice = &prices[prices.len() - period..];
    let sma: f64 = slice.iter().sum::<f64>() / period as f64;
    let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();

    let upper = sma + (std_dev * multiplier);
    let lower = sma - (std_dev * multiplier);

    Some(BollingerBands {
        upper,
        middle: sma,
        lower,
        bandwidth: (upper - lower) / sma,
    })
}

fn calculate_percent_b(price: f64, bands: &BollingerBands) -> f64 {
    // %B = (Цена - Нижняя полоса) / (Верхняя полоса - Нижняя полоса)
    // Возвращает: 0.0 на нижней полосе, 0.5 на средней, 1.0 на верхней
    (price - bands.lower) / (bands.upper - bands.lower)
}

fn interpret_percent_b(percent_b: f64) {
    println!("Интерпретация:");
    if percent_b > 1.0 {
        println!("  > Цена ВЫШЕ верхней полосы ({:.1}%)", (percent_b - 1.0) * 100.0);
        println!("  > Сильная перекупленность - рассмотрите продажу");
    } else if percent_b > 0.8 {
        println!("  > Цена около верхней полосы");
        println!("  > Зона перекупленности - следите за разворотом");
    } else if percent_b < 0.0 {
        println!("  > Цена НИЖЕ нижней полосы ({:.1}%)", percent_b.abs() * 100.0);
        println!("  > Сильная перепроданность - рассмотрите покупку");
    } else if percent_b < 0.2 {
        println!("  > Цена около нижней полосы");
        println!("  > Зона перепроданности - следите за отскоком");
    } else {
        println!("  > Цена в нормальном диапазоне");
        println!("  > Экстремальные условия не обнаружены");
    }
}
```

## Полная торговая стратегия с полосами Боллинджера

```rust
fn main() {
    let btc_prices = [
        41000.0, 41200.0, 41100.0, 41300.0, 41250.0,
        41400.0, 41350.0, 41500.0, 41450.0, 41600.0,
        41700.0, 41650.0, 41800.0, 41750.0, 41900.0,
        42000.0, 42100.0, 42050.0, 42200.0, 42300.0,
        42500.0, 42400.0, 42600.0, 42800.0, 43000.0,
        43200.0, 43100.0, 43400.0, 43300.0, 40500.0,  // Резкое падение
    ];

    let mut strategy = BollingerStrategy::new(20, 2.0);
    let mut portfolio = Portfolio::new(10000.0);

    println!("Симуляция торговли по полосам Боллинджера");
    println!("==========================================\n");

    for (i, &price) in btc_prices.iter().enumerate() {
        if let Some(action) = strategy.analyze(price) {
            println!("День {}: BTC ${:.2}", i + 1, price);
            println!("  Сигнал: {:?}", action.signal);
            println!("  %B: {:.4}", action.percent_b);

            match action.signal {
                Signal::Buy if portfolio.cash > 0.0 => {
                    let qty = (portfolio.cash * 0.5) / price;
                    portfolio.buy(price, qty);
                    println!("  ДЕЙСТВИЕ: Куплено {:.6} BTC", qty);
                }
                Signal::Sell if portfolio.btc > 0.0 => {
                    let qty = portfolio.btc * 0.5;
                    portfolio.sell(price, qty);
                    println!("  ДЕЙСТВИЕ: Продано {:.6} BTC", qty);
                }
                _ => println!("  ДЕЙСТВИЕ: Удержание"),
            }
            println!("  Портфель: ${:.2} + {:.6} BTC\n", portfolio.cash, portfolio.btc);
        }
    }

    let final_price = btc_prices[btc_prices.len() - 1];
    let total_value = portfolio.cash + portfolio.btc * final_price;
    println!("Итоговая стоимость портфеля: ${:.2}", total_value);
    println!("Доходность: {:.2}%", (total_value / 10000.0 - 1.0) * 100.0);
}

#[derive(Debug)]
enum Signal {
    Buy,   // Покупка
    Sell,  // Продажа
    Hold,  // Удержание
}

struct TradeAction {
    signal: Signal,
    percent_b: f64,
}

struct BollingerStrategy {
    period: usize,
    multiplier: f64,
    prices: Vec<f64>,
}

impl BollingerStrategy {
    fn new(period: usize, multiplier: f64) -> Self {
        BollingerStrategy {
            period,
            multiplier,
            prices: Vec::new(),
        }
    }

    fn analyze(&mut self, price: f64) -> Option<TradeAction> {
        self.prices.push(price);

        if self.prices.len() < self.period {
            return None;
        }

        let slice = &self.prices[self.prices.len() - self.period..];
        let sma: f64 = slice.iter().sum::<f64>() / self.period as f64;
        let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / self.period as f64;
        let std_dev = variance.sqrt();

        let upper = sma + (std_dev * self.multiplier);
        let lower = sma - (std_dev * self.multiplier);
        let percent_b = (price - lower) / (upper - lower);

        let signal = if percent_b < 0.0 {
            Signal::Buy   // Ниже нижней полосы
        } else if percent_b > 1.0 {
            Signal::Sell  // Выше верхней полосы
        } else {
            Signal::Hold
        };

        Some(TradeAction { signal, percent_b })
    }
}

struct Portfolio {
    cash: f64,
    btc: f64,
}

impl Portfolio {
    fn new(initial_cash: f64) -> Self {
        Portfolio { cash: initial_cash, btc: 0.0 }
    }

    fn buy(&mut self, price: f64, quantity: f64) {
        let cost = price * quantity;
        if cost <= self.cash {
            self.cash -= cost;
            self.btc += quantity;
        }
    }

    fn sell(&mut self, price: f64, quantity: f64) {
        if quantity <= self.btc {
            self.btc -= quantity;
            self.cash += price * quantity;
        }
    }
}
```

## Управление рисками с полосами Боллинджера

```rust
fn main() {
    let current_price = 42500.0;
    let portfolio_value = 50000.0;
    let risk_percent = 2.0;

    let prices = [
        42000.0, 42100.0, 41900.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0, 42350.0, 42500.0,
        42450.0, 42600.0, 42550.0, 42700.0, 42650.0,
        42800.0, 42750.0, 42900.0, 42850.0, 42500.0,
    ];

    if let Some(risk) = calculate_position_with_bands(&prices, 20, 2.0, portfolio_value, risk_percent) {
        println!("Расчёт размера позиции с полосами Боллинджера");
        println!("==============================================");
        println!("Текущая цена: ${:.2}", current_price);
        println!("Стоп-лосс (нижняя полоса): ${:.2}", risk.stop_loss);
        println!("Сумма риска: ${:.2}", risk.risk_amount);
        println!("Размер позиции: {:.6} BTC", risk.position_size);
        println!("Стоимость позиции: ${:.2}", risk.position_value);
    }
}

struct PositionRisk {
    stop_loss: f64,
    risk_amount: f64,
    position_size: f64,
    position_value: f64,
}

fn calculate_position_with_bands(
    prices: &[f64],
    period: usize,
    multiplier: f64,
    portfolio_value: f64,
    risk_percent: f64,
) -> Option<PositionRisk> {
    if prices.len() < period {
        return None;
    }

    let slice = &prices[prices.len() - period..];
    let sma: f64 = slice.iter().sum::<f64>() / period as f64;
    let variance: f64 = slice.iter().map(|p| (p - sma).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();
    let lower_band = sma - (std_dev * multiplier);

    let current_price = prices[prices.len() - 1];
    let risk_per_unit = current_price - lower_band;

    if risk_per_unit <= 0.0 {
        return None;
    }

    let risk_amount = portfolio_value * (risk_percent / 100.0);
    let position_size = risk_amount / risk_per_unit;
    let position_value = position_size * current_price;

    Some(PositionRisk {
        stop_loss: lower_band,
        risk_amount,
        position_size,
        position_value,
    })
}
```

## Что мы узнали

| Концепция | Описание | Торговое применение |
|-----------|----------|---------------------|
| Средняя полоса | SMA цен закрытия | Направление тренда |
| Верхняя полоса | SMA + (СтОткл × множитель) | Сопротивление / Перекупленность |
| Нижняя полоса | SMA - (СтОткл × множитель) | Поддержка / Перепроданность |
| Ширина канала | (Верхняя - Нижняя) / Средняя | Мера волатильности |
| %B | (Цена - Нижняя) / (Верхняя - Нижняя) | Позиция внутри полос |
| Сжатие | Узкая ширина канала | Ожидание прорыва |

## Домашнее задание

1. Реализуй функцию `fn calculate_bollinger_with_ema(prices: &[f64], period: usize, multiplier: f64) -> Option<BollingerBands>`, которая использует EMA вместо SMA для средней полосы

2. Создай функцию `fn detect_double_bottom(prices: &[f64], bands: &[BollingerBands]) -> Option<usize>`, которая обнаруживает, когда цена дважды касается нижней полосы — классический паттерн разворота

3. Напиши функцию бэктестинга `fn backtest_bollinger_strategy(prices: &[f64], period: usize, multiplier: f64) -> BacktestResult`, которая симулирует торговлю на основе сигналов полос Боллинджера и возвращает общую доходность, процент успешных сделок и максимальную просадку

4. Реализуй функцию мультитаймфреймового анализа `fn analyze_multi_timeframe(prices_1h: &[f64], prices_4h: &[f64], prices_1d: &[f64]) -> TradingSignal`, которая комбинирует сигналы полос Боллинджера с разных таймфреймов для более сильного подтверждения

## Навигация

[← Предыдущий день](../250-macd/ru.md) | [Следующий день →](../252-atr/ru.md)
