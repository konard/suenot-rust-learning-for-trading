# День 259: Стратегия: Пересечение SMA

## Аналогия из трейдинга

Представь, что ты наблюдаешь за оживлённым рынком. Цены постоянно колеблются — вверх, вниз, вбок. Как понять, когда покупать или продавать? Один из классических подходов — смотреть на «среднее настроение» рынка за разные периоды времени.

Это похоже на чтение погодных паттернов. Краткосрочное среднее (например, 10-дневная скользящая средняя) показывает недавние условия — как проверка сегодняшней погоды. Долгосрочное среднее (например, 50-дневная скользящая средняя) показывает общий тренд — как знание о времени года. Когда сегодняшняя погода теплее сезонной нормы, возможно, приближается весна. Когда краткосрочное среднее пересекает долгосрочное снизу вверх, возможно формирование бычьего тренда.

Это суть **стратегии пересечения SMA**:
- **Золотой крест**: Краткосрочная SMA пересекает долгосрочную SMA СНИЗУ ВВЕРХ → Сигнал на покупку (бычий)
- **Мёртвый крест**: Краткосрочная SMA пересекает долгосрочную SMA СВЕРХУ ВНИЗ → Сигнал на продажу (медвежий)

В реальном трейдинге эта стратегия используется:
- Свинг-трейдерами для поиска смены трендов
- Портфельными менеджерами для решений о распределении активов
- Алгоритмическими торговыми системами как базовая стратегия
- Риск-менеджерами для подтверждения направления тренда

## Что такое простая скользящая средняя (SMA)?

Простая скользящая средняя — это среднее арифметическое цен за определённый период:

```
SMA = (P1 + P2 + P3 + ... + Pn) / n
```

Где:
- `P1, P2, ... Pn` — цены (обычно цены закрытия)
- `n` — период (количество точек данных)

### Ключевые характеристики

| Свойство | Описание |
|----------|----------|
| Запаздывающий индикатор | Реагирует на прошлые движения цены |
| Эффект сглаживания | Фильтрует шум из ценовых данных |
| Чувствительность периода | Короче период = больше чувствительность, больше сигналов |
| Универсальность | Работает на любом таймфрейме (минуты, часы, дни) |

## Реализация SMA на Rust

Давай построим нашу стратегию пересечения SMA шаг за шагом.

### Шаг 1: Базовый расчёт SMA

```rust
/// Рассчитать простую скользящую среднюю из среза цен
fn calculate_sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let sum: f64 = prices.iter().rev().take(period).sum();
    Some(sum / period as f64)
}

fn main() {
    let prices = vec![
        42000.0, 42500.0, 42300.0, 42800.0, 43000.0,
        43200.0, 43100.0, 43500.0, 43800.0, 44000.0,
    ];

    if let Some(sma_5) = calculate_sma(&prices, 5) {
        println!("5-периодная SMA: ${:.2}", sma_5);
    }

    if let Some(sma_10) = calculate_sma(&prices, 10) {
        println!("10-периодная SMA: ${:.2}", sma_10);
    }
}
```

### Шаг 2: Калькулятор серии SMA

Для обнаружения пересечений нам нужна серия значений SMA:

```rust
/// Рассчитать серию значений SMA
fn calculate_sma_series(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    (0..prices.len())
        .map(|i| {
            if i + 1 >= period {
                let slice = &prices[i + 1 - period..=i];
                Some(slice.iter().sum::<f64>() / period as f64)
            } else {
                None // Ещё недостаточно данных
            }
        })
        .collect()
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0,
        104.0, 106.0, 108.0, 107.0, 110.0,
    ];

    let sma_3 = calculate_sma_series(&prices, 3);

    println!("Цена\t\tSMA(3)");
    println!("-----\t\t------");
    for (i, price) in prices.iter().enumerate() {
        match sma_3[i] {
            Some(sma) => println!("{:.2}\t\t{:.2}", price, sma),
            None => println!("{:.2}\t\t-", price),
        }
    }
}
```

### Шаг 3: Обнаружение сигналов пересечения

```rust
#[derive(Debug, Clone, PartialEq)]
enum Signal {
    Buy,        // Золотой крест
    Sell,       // Мёртвый крест
    Hold,       // Нет пересечения
}

#[derive(Debug, Clone)]
struct CrossoverSignal {
    index: usize,
    signal: Signal,
    short_sma: f64,
    long_sma: f64,
    price: f64,
}

/// Обнаружить сигналы пересечения SMA
fn detect_crossovers(
    prices: &[f64],
    short_period: usize,
    long_period: usize,
) -> Vec<CrossoverSignal> {
    let short_sma = calculate_sma_series(prices, short_period);
    let long_sma = calculate_sma_series(prices, long_period);

    let mut signals = Vec::new();

    for i in 1..prices.len() {
        // Нужны обе SMA для текущего и предыдущего периода
        if let (Some(short_curr), Some(long_curr), Some(short_prev), Some(long_prev)) = (
            short_sma[i],
            long_sma[i],
            short_sma[i - 1],
            long_sma[i - 1],
        ) {
            let signal = if short_prev <= long_prev && short_curr > long_curr {
                Signal::Buy // Золотой крест
            } else if short_prev >= long_prev && short_curr < long_curr {
                Signal::Sell // Мёртвый крест
            } else {
                Signal::Hold
            };

            if signal != Signal::Hold {
                signals.push(CrossoverSignal {
                    index: i,
                    signal,
                    short_sma: short_curr,
                    long_sma: long_curr,
                    price: prices[i],
                });
            }
        }
    }

    signals
}

fn main() {
    // Симулированные цены BTC со сменой тренда
    let prices = vec![
        40000.0, 40500.0, 41000.0, 40800.0, 41500.0,
        42000.0, 42500.0, 43000.0, 43500.0, 44000.0,
        44500.0, 44200.0, 43800.0, 43500.0, 43000.0,
        42500.0, 42000.0, 41500.0, 41000.0, 40500.0,
    ];

    let signals = detect_crossovers(&prices, 3, 7);

    println!("=== Сигналы пересечения SMA ===\n");
    for signal in &signals {
        let signal_type = match signal.signal {
            Signal::Buy => "ПОКУПКА (Золотой крест)",
            Signal::Sell => "ПРОДАЖА (Мёртвый крест)",
            Signal::Hold => "ОЖИДАНИЕ",
        };

        println!(
            "День {}: {} по цене ${:.2}",
            signal.index, signal_type, signal.price
        );
        println!(
            "  Короткая SMA: ${:.2}, Длинная SMA: ${:.2}\n",
            signal.short_sma, signal.long_sma
        );
    }
}
```

## Полная реализация торговой стратегии

Теперь давай построим полную систему бэктестинга для стратегии пересечения SMA:

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct Trade {
    entry_price: f64,
    exit_price: Option<f64>,
    entry_index: usize,
    exit_index: Option<usize>,
    position_type: PositionType,
}

#[derive(Debug, Clone, PartialEq)]
enum PositionType {
    Long,
    Short,
}

#[derive(Debug)]
struct BacktestResult {
    trades: Vec<Trade>,
    total_return: f64,
    win_rate: f64,
    max_drawdown: f64,
    total_trades: usize,
}

struct SmaCrossoverStrategy {
    short_period: usize,
    long_period: usize,
    prices: Vec<f64>,
}

impl SmaCrossoverStrategy {
    fn new(short_period: usize, long_period: usize) -> Self {
        SmaCrossoverStrategy {
            short_period,
            long_period,
            prices: Vec::new(),
        }
    }

    fn load_prices(&mut self, prices: Vec<f64>) {
        self.prices = prices;
    }

    fn calculate_sma_at(&self, end_index: usize, period: usize) -> Option<f64> {
        if end_index + 1 < period {
            return None;
        }

        let start = end_index + 1 - period;
        let slice = &self.prices[start..=end_index];
        Some(slice.iter().sum::<f64>() / period as f64)
    }

    fn backtest(&self) -> BacktestResult {
        let mut trades: Vec<Trade> = Vec::new();
        let mut current_position: Option<Trade> = None;
        let mut equity_curve: Vec<f64> = Vec::new();
        let initial_capital = 10000.0;
        let mut capital = initial_capital;

        for i in self.long_period..self.prices.len() {
            let short_curr = self.calculate_sma_at(i, self.short_period).unwrap();
            let long_curr = self.calculate_sma_at(i, self.long_period).unwrap();
            let short_prev = self.calculate_sma_at(i - 1, self.short_period).unwrap();
            let long_prev = self.calculate_sma_at(i - 1, self.long_period).unwrap();

            // Золотой крест - Сигнал на покупку
            if short_prev <= long_prev && short_curr > long_curr {
                // Закрываем существующую короткую позицию
                if let Some(mut pos) = current_position.take() {
                    if pos.position_type == PositionType::Short {
                        pos.exit_price = Some(self.prices[i]);
                        pos.exit_index = Some(i);
                        let pnl = pos.entry_price - self.prices[i];
                        capital += pnl / pos.entry_price * capital;
                        trades.push(pos);
                    }
                }

                // Открываем длинную позицию
                current_position = Some(Trade {
                    entry_price: self.prices[i],
                    exit_price: None,
                    entry_index: i,
                    exit_index: None,
                    position_type: PositionType::Long,
                });
            }

            // Мёртвый крест - Сигнал на продажу
            if short_prev >= long_prev && short_curr < long_curr {
                // Закрываем существующую длинную позицию
                if let Some(mut pos) = current_position.take() {
                    if pos.position_type == PositionType::Long {
                        pos.exit_price = Some(self.prices[i]);
                        pos.exit_index = Some(i);
                        let pnl = self.prices[i] - pos.entry_price;
                        capital += pnl / pos.entry_price * capital;
                        trades.push(pos);
                    }
                }

                // Открываем короткую позицию
                current_position = Some(Trade {
                    entry_price: self.prices[i],
                    exit_price: None,
                    entry_index: i,
                    exit_index: None,
                    position_type: PositionType::Short,
                });
            }

            equity_curve.push(capital);
        }

        // Закрываем оставшуюся позицию в конце
        if let Some(mut pos) = current_position.take() {
            let final_price = *self.prices.last().unwrap();
            pos.exit_price = Some(final_price);
            pos.exit_index = Some(self.prices.len() - 1);

            let pnl = match pos.position_type {
                PositionType::Long => final_price - pos.entry_price,
                PositionType::Short => pos.entry_price - final_price,
            };
            capital += pnl / pos.entry_price * capital;
            trades.push(pos);
        }

        // Рассчитываем статистику
        let winning_trades = trades.iter().filter(|t| {
            if let Some(exit) = t.exit_price {
                match t.position_type {
                    PositionType::Long => exit > t.entry_price,
                    PositionType::Short => exit < t.entry_price,
                }
            } else {
                false
            }
        }).count();

        let total_trades = trades.len();
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        // Рассчитываем максимальную просадку
        let mut peak = initial_capital;
        let mut max_drawdown = 0.0;
        for equity in &equity_curve {
            if *equity > peak {
                peak = *equity;
            }
            let drawdown = (peak - equity) / peak * 100.0;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        let total_return = (capital - initial_capital) / initial_capital * 100.0;

        BacktestResult {
            trades,
            total_return,
            win_rate,
            max_drawdown,
            total_trades,
        }
    }
}

fn main() {
    // Генерируем примерные данные цен с трендами
    let mut prices: Vec<f64> = Vec::new();
    let mut price = 100.0;

    // Восходящий тренд
    for _ in 0..30 {
        price += (rand_simple() - 0.3) * 2.0;
        prices.push(price);
    }

    // Нисходящий тренд
    for _ in 0..30 {
        price += (rand_simple() - 0.7) * 2.0;
        prices.push(price);
    }

    // Ещё один восходящий тренд
    for _ in 0..30 {
        price += (rand_simple() - 0.3) * 2.0;
        prices.push(price);
    }

    let mut strategy = SmaCrossoverStrategy::new(5, 20);
    strategy.load_prices(prices);

    let result = strategy.backtest();

    println!("=== Результаты бэктеста SMA Crossover ===\n");
    println!("Период короткой SMA: 5");
    println!("Период длинной SMA: 20");
    println!("-----------------------------------");
    println!("Всего сделок: {}", result.total_trades);
    println!("Процент прибыльных: {:.2}%", result.win_rate);
    println!("Общая доходность: {:.2}%", result.total_return);
    println!("Максимальная просадка: {:.2}%", result.max_drawdown);

    println!("\n=== История сделок ===\n");
    for (i, trade) in result.trades.iter().enumerate() {
        let direction = match trade.position_type {
            PositionType::Long => "ЛОНГ",
            PositionType::Short => "ШОРТ",
        };

        if let Some(exit_price) = trade.exit_price {
            let pnl_pct = match trade.position_type {
                PositionType::Long => (exit_price - trade.entry_price) / trade.entry_price * 100.0,
                PositionType::Short => (trade.entry_price - exit_price) / trade.entry_price * 100.0,
            };

            println!(
                "Сделка {}: {} | Вход: ${:.2} | Выход: ${:.2} | P&L: {:.2}%",
                i + 1, direction, trade.entry_price, exit_price, pnl_pct
            );
        }
    }
}

// Простой псевдослучайный генератор для демонстрации
fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    static mut SEED: u64 = 0;
    unsafe {
        if SEED == 0 {
            SEED = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
        }
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        ((SEED >> 16) & 0x7fff) as f64 / 32768.0
    }
}
```

## Отслеживание SMA в реальном времени

Для живой торговли нужен эффективный калькулятор скользящей SMA:

```rust
use std::collections::VecDeque;

struct RollingSma {
    period: usize,
    prices: VecDeque<f64>,
    sum: f64,
}

impl RollingSma {
    fn new(period: usize) -> Self {
        RollingSma {
            period,
            prices: VecDeque::with_capacity(period),
            sum: 0.0,
        }
    }

    fn update(&mut self, price: f64) -> Option<f64> {
        self.prices.push_back(price);
        self.sum += price;

        if self.prices.len() > self.period {
            if let Some(old_price) = self.prices.pop_front() {
                self.sum -= old_price;
            }
        }

        if self.prices.len() >= self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }

    fn current(&self) -> Option<f64> {
        if self.prices.len() >= self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

struct CrossoverTracker {
    short_sma: RollingSma,
    long_sma: RollingSma,
    prev_short: Option<f64>,
    prev_long: Option<f64>,
}

impl CrossoverTracker {
    fn new(short_period: usize, long_period: usize) -> Self {
        CrossoverTracker {
            short_sma: RollingSma::new(short_period),
            long_sma: RollingSma::new(long_period),
            prev_short: None,
            prev_long: None,
        }
    }

    fn update(&mut self, price: f64) -> Signal {
        let short_curr = self.short_sma.update(price);
        let long_curr = self.long_sma.update(price);

        let signal = match (short_curr, long_curr, self.prev_short, self.prev_long) {
            (Some(sc), Some(lc), Some(sp), Some(lp)) => {
                if sp <= lp && sc > lc {
                    Signal::Buy
                } else if sp >= lp && sc < lc {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        self.prev_short = short_curr;
        self.prev_long = long_curr;

        signal
    }
}

#[derive(Debug, PartialEq)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

fn main() {
    let mut tracker = CrossoverTracker::new(3, 7);

    // Симулируем входящие тики цен
    let price_stream = vec![
        100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 105.0,
        106.0, 107.0, 108.0, 107.5, 106.0, 105.0, 104.0,
        103.0, 102.0, 101.0, 100.0, 99.0, 98.0,
    ];

    println!("=== Отслеживание пересечений в реальном времени ===\n");
    println!("Цена\t\tСигнал");
    println!("-----\t\t------");

    for price in price_stream {
        let signal = tracker.update(price);

        let signal_str = match signal {
            Signal::Buy => ">>> ПОКУПКА <<<",
            Signal::Sell => "<<< ПРОДАЖА >>>",
            Signal::Hold => "-",
        };

        println!("{:.2}\t\t{}", price, signal_str);
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| SMA | Средняя цена за N периодов, сглаживает ценовое движение |
| Золотой крест | Короткая SMA пересекает длинную снизу вверх — бычий сигнал |
| Мёртвый крест | Короткая SMA пересекает длинную сверху вниз — медвежий сигнал |
| Запаздывающий индикатор | SMA реагирует на прошлые данные, сигналы приходят с задержкой |
| Выбор периода | Короче = более чувствительно, длиннее = более надёжно |
| Скользящий расчёт | Эффективное O(1) обновление с использованием скользящего окна |

## Упражнения

1. **Базовая SMA**: Напиши функцию, которая принимает вектор цен и период, возвращая SMA. Протестируй её с дневными ценами BTC.

2. **Несколько таймфреймов**: Создай функцию, которая рассчитывает SMA для нескольких периодов (5, 10, 20, 50) одновременно и отображает их в табличном формате.

3. **Счётчик сигналов**: Построй программу, которая читает данные о ценах и считает, сколько золотых и мёртвых крестов произошло в наборе данных.

4. **Сила тренда**: Реализуй функцию, которая измеряет «силу» пересечения, рассчитывая, как быстро короткая SMA расходится с длинной SMA после сигнала.

## Домашнее задание

1. **Улучшенная стратегия**: Модифицируй стратегию пересечения SMA, добавив:
   - Минимальное расстояние между SMA перед генерацией сигнала (фильтр ложных пересечений)
   - Подтверждение объёмом (принимать сигналы только в дни с объёмом выше среднего)
   - Уровни стоп-лосс и тейк-профит

2. **Сканер нескольких активов**: Создай программу, которая:
   - Принимает данные о ценах для нескольких активов (BTC, ETH, SOL и т.д.)
   - Отслеживает пересечения SMA по всем из них
   - Сообщает, какие активы в данный момент показывают сигналы покупки или продажи

3. **Оптимизация параметров**: Построй бэктестер, который:
   - Тестирует разные комбинации периодов короткой/длинной SMA (например, 5/20, 10/30, 15/50)
   - Сообщает, какая комбинация даёт лучшие результаты
   - Обрабатывает граничные случаи (недостаточно данных, нет сделок и т.д.)

4. **Живая панель мониторинга**: Создай терминальную панель, которая:
   - Непрерывно получает обновления цен
   - Отображает текущие значения короткой и длинной SMA
   - Показывает текущий статус сигнала (Лонг, Шорт или Нейтрально)
   - Отслеживает P&L от следования сигналам

## Навигация

[← Предыдущий день](../258-moving-averages-intro/ru.md) | [Следующий день →](../260-ema-exponential-moving-average/ru.md)
