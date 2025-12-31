# День 253: Стохастический осциллятор (Stochastic Oscillator)

## Аналогия из трейдинга

Представь, что ты наблюдаешь за мячом, который прыгает между полом и потолком комнаты. Когда мяч приближается к потолку, он теряет импульс и скоро начнёт падать. Когда он приближается к полу, он вот-вот оттолкнётся и начнёт подниматься.

**Стохастический осциллятор** работает точно так же для цен на рынке! Он измеряет, где находится текущая цена относительно диапазона цен за определённый период:
- Если цена близка к максимуму диапазона (около потолка) — актив **перекуплен** и скоро может пойти вниз
- Если цена близка к минимуму диапазона (около пола) — актив **перепродан** и скоро может пойти вверх

Этот индикатор был разработан Джорджем Лейном в 1950-х годах и до сих пор является одним из самых популярных инструментов технического анализа.

## Что такое Стохастический осциллятор?

Стохастический осциллятор — это индикатор импульса (momentum indicator), который сравнивает цену закрытия с диапазоном цен за определённый период.

### Формулы

Осциллятор состоит из двух линий:

**%K (быстрая линия)**:
```
%K = ((Текущее закрытие - Минимум за N периодов) / (Максимум за N периодов - Минимум за N периодов)) × 100
```

**%D (медленная линия)** — сглаженная версия %K:
```
%D = SMA(%K, M периодов)
```

Где:
- N — обычно 14 периодов
- M — обычно 3 периода для сглаживания

### Интерпретация значений

| Зона | Значение | Интерпретация |
|------|----------|---------------|
| Перекупленность | > 80 | Актив может быть переоценён, возможен разворот вниз |
| Нейтральная | 20-80 | Нормальное состояние рынка |
| Перепроданность | < 20 | Актив может быть недооценён, возможен разворот вверх |

## Базовая реализация

```rust
/// Структура для хранения данных свечи (OHLC)
#[derive(Debug, Clone)]
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    timestamp: u64,
}

/// Результат расчёта стохастического осциллятора
#[derive(Debug, Clone)]
struct StochasticResult {
    k_value: f64,  // Быстрая линия %K
    d_value: f64,  // Медленная линия %D
    signal: Signal,
}

#[derive(Debug, Clone, PartialEq)]
enum Signal {
    Buy,
    Sell,
    Neutral,
}

/// Калькулятор стохастического осциллятора
struct StochasticOscillator {
    k_period: usize,  // Период для расчёта %K (обычно 14)
    d_period: usize,  // Период для сглаживания %D (обычно 3)
    overbought: f64,  // Уровень перекупленности (обычно 80)
    oversold: f64,    // Уровень перепроданности (обычно 20)
}

impl StochasticOscillator {
    fn new(k_period: usize, d_period: usize) -> Self {
        StochasticOscillator {
            k_period,
            d_period,
            overbought: 80.0,
            oversold: 20.0,
        }
    }

    /// Находит максимум High за период
    fn highest_high(&self, candles: &[Candle]) -> f64 {
        candles
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Находит минимум Low за период
    fn lowest_low(&self, candles: &[Candle]) -> f64 {
        candles
            .iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, f64::min)
    }

    /// Рассчитывает значение %K для одного периода
    fn calculate_k(&self, candles: &[Candle]) -> Option<f64> {
        if candles.len() < self.k_period {
            return None;
        }

        let period_candles = &candles[candles.len() - self.k_period..];
        let highest = self.highest_high(period_candles);
        let lowest = self.lowest_low(period_candles);
        let current_close = candles.last()?.close;

        // Избегаем деления на ноль
        if (highest - lowest).abs() < f64::EPSILON {
            return Some(50.0); // Если диапазон нулевой, возвращаем середину
        }

        let k = ((current_close - lowest) / (highest - lowest)) * 100.0;
        Some(k.clamp(0.0, 100.0))
    }

    /// Рассчитывает SMA для сглаживания %D
    fn calculate_sma(&self, values: &[f64], period: usize) -> Option<f64> {
        if values.len() < period {
            return None;
        }
        let sum: f64 = values[values.len() - period..].iter().sum();
        Some(sum / period as f64)
    }

    /// Полный расчёт стохастика для серии свечей
    fn calculate(&self, candles: &[Candle]) -> Vec<StochasticResult> {
        let mut results = Vec::new();
        let mut k_values = Vec::new();

        // Рассчитываем %K для каждой позиции
        for i in self.k_period..=candles.len() {
            if let Some(k) = self.calculate_k(&candles[..i]) {
                k_values.push(k);
            }
        }

        // Рассчитываем %D и генерируем результаты
        for i in self.d_period..=k_values.len() {
            let k_value = k_values[i - 1];
            let d_value = self.calculate_sma(&k_values[..i], self.d_period)
                .unwrap_or(k_value);

            // Определяем сигнал
            let signal = self.generate_signal(k_value, d_value, &k_values, i);

            results.push(StochasticResult {
                k_value,
                d_value,
                signal,
            });
        }

        results
    }

    /// Генерирует торговый сигнал
    fn generate_signal(
        &self,
        k: f64,
        d: f64,
        k_history: &[f64],
        current_idx: usize,
    ) -> Signal {
        if current_idx < 2 {
            return Signal::Neutral;
        }

        let prev_k = k_history[current_idx - 2];
        let prev_d = if current_idx >= self.d_period + 1 {
            k_history[current_idx - self.d_period - 1..current_idx - 1]
                .iter()
                .sum::<f64>() / self.d_period as f64
        } else {
            prev_k
        };

        // Бычий сигнал: %K пересекает %D снизу вверх в зоне перепроданности
        if k > d && prev_k <= prev_d && k < self.oversold + 10.0 {
            return Signal::Buy;
        }

        // Медвежий сигнал: %K пересекает %D сверху вниз в зоне перекупленности
        if k < d && prev_k >= prev_d && k > self.overbought - 10.0 {
            return Signal::Sell;
        }

        Signal::Neutral
    }
}

fn main() {
    // Создаём тестовые данные — исторические свечи BTC/USDT
    let candles = vec![
        Candle { open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0, timestamp: 1 },
        Candle { open: 42300.0, high: 42800.0, low: 42100.0, close: 42600.0, timestamp: 2 },
        Candle { open: 42600.0, high: 43200.0, low: 42400.0, close: 43000.0, timestamp: 3 },
        Candle { open: 43000.0, high: 43500.0, low: 42800.0, close: 43400.0, timestamp: 4 },
        Candle { open: 43400.0, high: 44000.0, low: 43200.0, close: 43800.0, timestamp: 5 },
        Candle { open: 43800.0, high: 44200.0, low: 43500.0, close: 44100.0, timestamp: 6 },
        Candle { open: 44100.0, high: 44500.0, low: 43900.0, close: 44400.0, timestamp: 7 },
        Candle { open: 44400.0, high: 44800.0, low: 44200.0, close: 44600.0, timestamp: 8 },
        Candle { open: 44600.0, high: 44700.0, low: 44000.0, close: 44200.0, timestamp: 9 },
        Candle { open: 44200.0, high: 44400.0, low: 43800.0, close: 43900.0, timestamp: 10 },
        Candle { open: 43900.0, high: 44100.0, low: 43500.0, close: 43600.0, timestamp: 11 },
        Candle { open: 43600.0, high: 43800.0, low: 43200.0, close: 43300.0, timestamp: 12 },
        Candle { open: 43300.0, high: 43500.0, low: 42800.0, close: 42900.0, timestamp: 13 },
        Candle { open: 42900.0, high: 43100.0, low: 42500.0, close: 42600.0, timestamp: 14 },
        Candle { open: 42600.0, high: 42800.0, low: 42200.0, close: 42300.0, timestamp: 15 },
        Candle { open: 42300.0, high: 42600.0, low: 42100.0, close: 42500.0, timestamp: 16 },
        Candle { open: 42500.0, high: 43000.0, low: 42400.0, close: 42900.0, timestamp: 17 },
        Candle { open: 42900.0, high: 43400.0, low: 42800.0, close: 43300.0, timestamp: 18 },
    ];

    // Создаём осциллятор с периодами 14 и 3
    let stochastic = StochasticOscillator::new(14, 3);
    let results = stochastic.calculate(&candles);

    println!("=== Стохастический осциллятор BTC/USDT ===\n");
    println!("{:>8} {:>8} {:>10}", "%K", "%D", "Сигнал");
    println!("{:-<30}", "");

    for result in &results {
        let signal_str = match result.signal {
            Signal::Buy => "ПОКУПКА",
            Signal::Sell => "ПРОДАЖА",
            Signal::Neutral => "-",
        };
        println!(
            "{:>8.2} {:>8.2} {:>10}",
            result.k_value, result.d_value, signal_str
        );
    }
}
```

## Продвинутая реализация с торговой стратегией

```rust
use std::collections::VecDeque;

/// Торговая позиция
#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    side: PositionSide,
    entry_price: f64,
    quantity: f64,
    stop_loss: f64,
    take_profit: f64,
}

#[derive(Debug, Clone, PartialEq)]
enum PositionSide {
    Long,
    Short,
}

/// Торговый движок на основе стохастика
struct StochasticTradingEngine {
    oscillator: StochasticOscillator,
    candle_buffer: VecDeque<Candle>,
    current_position: Option<Position>,
    balance: f64,
    trade_history: Vec<TradeResult>,
}

#[derive(Debug, Clone)]
struct TradeResult {
    symbol: String,
    side: PositionSide,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
    pnl_percent: f64,
}

impl StochasticTradingEngine {
    fn new(k_period: usize, d_period: usize, initial_balance: f64) -> Self {
        StochasticTradingEngine {
            oscillator: StochasticOscillator::new(k_period, d_period),
            candle_buffer: VecDeque::with_capacity(k_period + d_period),
            current_position: None,
            balance: initial_balance,
            trade_history: Vec::new(),
        }
    }

    /// Добавляет новую свечу и возвращает торговый сигнал
    fn on_candle(&mut self, candle: Candle) -> Option<Signal> {
        self.candle_buffer.push_back(candle.clone());

        // Ограничиваем размер буфера
        let max_size = self.oscillator.k_period + self.oscillator.d_period + 10;
        while self.candle_buffer.len() > max_size {
            self.candle_buffer.pop_front();
        }

        // Проверяем стоп-лосс и тейк-профит
        self.check_exit_conditions(&candle);

        // Рассчитываем стохастик
        let candles: Vec<Candle> = self.candle_buffer.iter().cloned().collect();
        let results = self.oscillator.calculate(&candles);

        results.last().map(|r| r.signal.clone())
    }

    /// Проверяет условия выхода из позиции
    fn check_exit_conditions(&mut self, candle: &Candle) {
        if let Some(position) = &self.current_position {
            let should_close = match position.side {
                PositionSide::Long => {
                    candle.low <= position.stop_loss || candle.high >= position.take_profit
                }
                PositionSide::Short => {
                    candle.high >= position.stop_loss || candle.low <= position.take_profit
                }
            };

            if should_close {
                let exit_price = match position.side {
                    PositionSide::Long => {
                        if candle.low <= position.stop_loss {
                            position.stop_loss
                        } else {
                            position.take_profit
                        }
                    }
                    PositionSide::Short => {
                        if candle.high >= position.stop_loss {
                            position.stop_loss
                        } else {
                            position.take_profit
                        }
                    }
                };

                self.close_position(exit_price);
            }
        }
    }

    /// Открывает новую позицию
    fn open_position(&mut self, symbol: &str, side: PositionSide, price: f64, risk_percent: f64) {
        if self.current_position.is_some() {
            println!("Позиция уже открыта!");
            return;
        }

        // Рассчитываем размер позиции на основе риска
        let risk_amount = self.balance * risk_percent;
        let stop_distance = price * 0.02; // 2% стоп-лосс

        let quantity = risk_amount / stop_distance;
        let position_value = quantity * price;

        if position_value > self.balance {
            println!("Недостаточно средств!");
            return;
        }

        let (stop_loss, take_profit) = match side {
            PositionSide::Long => (price - stop_distance, price + stop_distance * 2.0),
            PositionSide::Short => (price + stop_distance, price - stop_distance * 2.0),
        };

        let position = Position {
            symbol: symbol.to_string(),
            side: side.clone(),
            entry_price: price,
            quantity,
            stop_loss,
            take_profit,
        };

        println!(
            "Открыта {:?} позиция: {} @ {:.2}, SL: {:.2}, TP: {:.2}",
            side, symbol, price, stop_loss, take_profit
        );

        self.current_position = Some(position);
    }

    /// Закрывает текущую позицию
    fn close_position(&mut self, exit_price: f64) {
        if let Some(position) = self.current_position.take() {
            let pnl = match position.side {
                PositionSide::Long => (exit_price - position.entry_price) * position.quantity,
                PositionSide::Short => (position.entry_price - exit_price) * position.quantity,
            };

            let pnl_percent = (pnl / (position.entry_price * position.quantity)) * 100.0;

            self.balance += pnl;

            let trade = TradeResult {
                symbol: position.symbol.clone(),
                side: position.side.clone(),
                entry_price: position.entry_price,
                exit_price,
                quantity: position.quantity,
                pnl,
                pnl_percent,
            };

            println!(
                "Закрыта {:?} позиция: {} @ {:.2} -> {:.2}, PnL: {:.2} ({:.2}%)",
                position.side, position.symbol, position.entry_price, exit_price, pnl, pnl_percent
            );

            self.trade_history.push(trade);
        }
    }

    /// Возвращает статистику торговли
    fn get_statistics(&self) -> TradingStatistics {
        let total_trades = self.trade_history.len();
        let winning_trades = self.trade_history.iter().filter(|t| t.pnl > 0.0).count();
        let losing_trades = self.trade_history.iter().filter(|t| t.pnl < 0.0).count();

        let total_pnl: f64 = self.trade_history.iter().map(|t| t.pnl).sum();
        let gross_profit: f64 = self.trade_history.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();
        let gross_loss: f64 = self.trade_history.iter().filter(|t| t.pnl < 0.0).map(|t| t.pnl.abs()).sum();

        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        TradingStatistics {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_pnl,
            profit_factor,
            final_balance: self.balance,
        }
    }
}

#[derive(Debug)]
struct TradingStatistics {
    total_trades: usize,
    winning_trades: usize,
    losing_trades: usize,
    win_rate: f64,
    total_pnl: f64,
    profit_factor: f64,
    final_balance: f64,
}

fn main() {
    let mut engine = StochasticTradingEngine::new(14, 3, 10000.0);

    // Симулируем торговлю
    let candles = generate_sample_candles();

    println!("=== Торговая стратегия на основе стохастика ===\n");

    for candle in candles {
        if let Some(signal) = engine.on_candle(candle.clone()) {
            match signal {
                Signal::Buy if engine.current_position.is_none() => {
                    engine.open_position("BTC/USDT", PositionSide::Long, candle.close, 0.02);
                }
                Signal::Sell if engine.current_position.is_none() => {
                    engine.open_position("BTC/USDT", PositionSide::Short, candle.close, 0.02);
                }
                _ => {}
            }
        }
    }

    // Закрываем позицию в конце, если открыта
    if engine.current_position.is_some() {
        engine.close_position(42500.0);
    }

    let stats = engine.get_statistics();
    println!("\n=== Статистика торговли ===");
    println!("Всего сделок: {}", stats.total_trades);
    println!("Прибыльных: {}", stats.winning_trades);
    println!("Убыточных: {}", stats.losing_trades);
    println!("Win Rate: {:.2}%", stats.win_rate);
    println!("Общий PnL: ${:.2}", stats.total_pnl);
    println!("Profit Factor: {:.2}", stats.profit_factor);
    println!("Финальный баланс: ${:.2}", stats.final_balance);
}

fn generate_sample_candles() -> Vec<Candle> {
    // Генерируем реалистичные данные для тестирования
    let mut candles = Vec::new();
    let mut price = 42000.0;

    for i in 0..50 {
        let volatility = 200.0;
        let change = if i % 7 < 3 {
            volatility * 0.5
        } else if i % 7 < 5 {
            -volatility * 0.3
        } else {
            volatility * 0.1
        };

        let open = price;
        let close = price + change;
        let high = f64::max(open, close) + volatility * 0.3;
        let low = f64::min(open, close) - volatility * 0.3;

        candles.push(Candle {
            open,
            high,
            low,
            close,
            timestamp: i as u64,
        });

        price = close;
    }

    candles
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Стохастический осциллятор | Индикатор импульса, сравнивающий цену закрытия с диапазоном |
| %K линия | Быстрая линия, показывающая текущее положение цены |
| %D линия | Медленная линия (SMA от %K) для сглаживания |
| Перекупленность | Зона выше 80 — возможен разворот вниз |
| Перепроданность | Зона ниже 20 — возможен разворот вверх |
| Пересечение линий | Сигнал на вход/выход из позиции |

## Практические упражнения

1. **Расчёт вручную**: Дан массив цен закрытия `[100, 102, 98, 105, 103, 107, 104, 108]`. Рассчитай значение %K для последней свечи с периодом 5.

2. **Определение зоны**: Для значений стохастика `[78, 82, 85, 83, 79, 75]` определи, когда актив вошёл в зону перекупленности и когда вышел из неё.

3. **Поиск сигналов**: Даны значения %K `[15, 18, 25, 30]` и %D `[12, 16, 22, 28]`. Определи, есть ли сигнал на покупку.

## Домашнее задание

1. **Медленный стохастик**: Реализуй версию Slow Stochastic, где:
   - Slow %K = Fast %D (SMA от Fast %K)
   - Slow %D = SMA от Slow %K

   Это даёт более сглаженные сигналы.

2. **Дивергенция**: Добавь функцию обнаружения дивергенции:
   - Бычья дивергенция: цена делает новый минимум, а стохастик — нет
   - Медвежья дивергенция: цена делает новый максимум, а стохастик — нет

3. **Мультитаймфрейм анализ**: Создай структуру, которая анализирует стохастик на нескольких таймфреймах (1ч, 4ч, 1д) и генерирует сигнал только при согласованности всех таймфреймов.

4. **Бэктестинг**: Напиши полноценный бэктестер для стратегии на стохастике:
   - Загрузка исторических данных из CSV
   - Расчёт метрик: Sharpe Ratio, Maximum Drawdown, Win Rate
   - Вывод графика equity curve (можно текстом в консоль)

## Навигация

[← Предыдущий день](../252-rsi-indicator/ru.md) | [Следующий день →](../254-macd-indicator/ru.md)
