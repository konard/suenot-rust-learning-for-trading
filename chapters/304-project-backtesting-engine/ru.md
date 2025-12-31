# День 304: Проект: Бэктестинг движок

## Аналогия из трейдинга

Представь, что ты опытный трейдер, который хочет разработать профессиональную торговую стратегию. Ты не можешь просто бросать деньги на случайные идеи на реальном рынке — это слишком рискованно и дорого. Вместо этого тебе нужен **бэктестинг движок**: система, которая тестирует твою стратегию на исторических данных, чтобы увидеть, как она работала бы.

Хороший бэктестинг движок — это как авиасимулятор для пилотов:
- Безопасная среда для тестирования рискованных идей
- Реалистичная симуляция со всеми комиссиями и проскальзываниями
- Детальные метрики производительности
- Валидация того, что стратегия работает на разных периодах
- Защита от переобучения

В этом месяце мы изучили все компоненты профессионального бэктестинга. Теперь мы собираем их воедино в полноценный, готовый к продакшену бэктестинг движок.

## Обзор проекта

Мы создаём комплексный бэктестинг движок, который включает:

1. **Основной движок**: Выполнение стратегий, симуляция ордеров, отслеживание позиций
2. **Комиссии и проскальзывание**: Реалистичное моделирование затрат
3. **Расчёт метрик**: Все ключевые показатели производительности
4. **Управление рисками**: Отслеживание просадок, расчёт размера позиций
5. **Валидация**: Walk-forward анализ, тестирование вне выборки
6. **Отчётность**: Детальные отчёты с визуализациями
7. **Оптимизация**: Подбор параметров с защитой от переобучения

## Архитектура проекта

```
backtesting_engine/
├── src/
│   ├── main.rs              # Точка входа и CLI
│   ├── lib.rs               # Публичное API
│   ├── engine/
│   │   ├── mod.rs           # Основной движок бэктестинга
│   │   ├── broker.rs        # Симулированный брокер
│   │   └── executor.rs      # Логика исполнения сделок
│   ├── data/
│   │   ├── mod.rs           # Обработка рыночных данных
│   │   ├── candle.rs        # OHLCV свечи
│   │   └── loader.rs        # Загрузка данных из CSV/JSON
│   ├── strategy/
│   │   ├── mod.rs           # Трейт стратегии
│   │   ├── moving_average.rs # Пример стратегии на скользящих средних
│   │   └── mean_reversion.rs # Пример стратегии возврата к среднему
│   ├── metrics/
│   │   ├── mod.rs           # Метрики производительности
│   │   ├── returns.rs       # Расчёт доходности
│   │   ├── risk.rs          # Метрики риска (Sharpe, просадка)
│   │   └── trade_stats.rs   # Статистика по сделкам
│   ├── validation/
│   │   ├── mod.rs           # Методы валидации
│   │   ├── walk_forward.rs  # Walk-forward анализ
│   │   ├── cross_validation.rs # K-fold кросс-валидация
│   │   └── monte_carlo.rs   # Монте-Карло симуляция
│   ├── optimization/
│   │   ├── mod.rs           # Оптимизация параметров
│   │   ├── grid_search.rs   # Grid search
│   │   └── genetic.rs       # Генетический алгоритм (бонус)
│   ├── report/
│   │   ├── mod.rs           # Генерация отчётов
│   │   ├── equity_curve.rs  # Визуализация кривой доходности
│   │   └── html.rs          # Генерация HTML отчётов
│   └── utils/
│       ├── mod.rs           # Утилиты
│       └── commissions.rs   # Модели комиссий
└── Cargo.toml
```

## Шаг 1: Структуры данных

```rust
// src/data/candle.rs
use serde::{Deserialize, Serialize};

/// OHLCV свеча
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    pub fn range(&self) -> f64 {
        self.high - self.low
    }
}

// src/engine/mod.rs
use std::collections::HashMap;

/// Торговая позиция
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub size: f64,           // Положительный = лонг, отрицательный = шорт
    pub entry_price: f64,
    pub entry_time: u64,
    pub realized_pnl: f64,   // Закрытая прибыль/убыток
    pub unrealized_pnl: f64, // Открытая прибыль/убыток
}

impl Position {
    pub fn update_unrealized_pnl(&mut self, current_price: f64) {
        self.unrealized_pnl = (current_price - self.entry_price) * self.size;
    }

    pub fn market_value(&self, current_price: f64) -> f64 {
        self.size * current_price
    }
}

/// Запись об исполненной сделке
#[derive(Debug, Clone)]
pub struct Trade {
    pub timestamp: u64,
    pub symbol: String,
    pub side: Side,
    pub price: f64,
    pub size: f64,
    pub commission: f64,
    pub pnl: f64,            // Для закрывающих сделок
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

/// Состояние портфеля в определённый момент времени
#[derive(Debug, Clone)]
pub struct PortfolioSnapshot {
    pub timestamp: u64,
    pub cash: f64,
    pub positions_value: f64,
    pub total_equity: f64,
    pub drawdown: f64,
}
```

## Шаг 2: Модели комиссий

```rust
// src/utils/commissions.rs

/// Модель комиссий для реалистичной симуляции затрат
#[derive(Debug, Clone)]
pub struct CommissionModel {
    pub taker_fee_percent: f64,
    pub maker_fee_percent: f64,
    pub min_fee: f64,
}

impl CommissionModel {
    pub fn binance_spot() -> Self {
        CommissionModel {
            taker_fee_percent: 0.1,  // 0.1%
            maker_fee_percent: 0.1,
            min_fee: 0.0,
        }
    }

    pub fn zero() -> Self {
        CommissionModel {
            taker_fee_percent: 0.0,
            maker_fee_percent: 0.0,
            min_fee: 0.0,
        }
    }

    pub fn calculate(&self, value: f64, is_maker: bool) -> f64 {
        let fee_percent = if is_maker {
            self.maker_fee_percent
        } else {
            self.taker_fee_percent
        };

        let fee = value * fee_percent / 100.0;
        fee.max(self.min_fee)
    }
}

/// Модель проскальзывания
#[derive(Debug, Clone)]
pub struct SlippageModel {
    pub fixed_percent: f64,  // Фиксированный процент проскальзывания
    pub volume_impact: f64,  // Влияние на основе размера ордера относительно объёма
}

impl SlippageModel {
    pub fn simple(percent: f64) -> Self {
        SlippageModel {
            fixed_percent: percent,
            volume_impact: 0.0,
        }
    }

    pub fn calculate(&self, price: f64, size: f64, volume: f64) -> f64 {
        let fixed = price * self.fixed_percent / 100.0;
        let impact = if volume > 0.0 {
            price * (size / volume) * self.volume_impact
        } else {
            0.0
        };
        fixed + impact
    }
}
```

## Шаг 3: Трейт стратегии

```rust
// src/strategy/mod.rs
use crate::data::Candle;
use crate::engine::Side;

/// Сигнал от стратегии
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Трейт стратегии, который должны реализовывать все торговые стратегии
pub trait Strategy: Send + Sync {
    /// Инициализация стратегии с параметрами
    fn initialize(&mut self, params: HashMap<String, f64>);

    /// Генерация сигнала на основе текущих рыночных данных
    /// - candles: Исторические данные по ценам (от старых к новым)
    /// - index: Индекс текущей свечи
    fn generate_signal(&mut self, candles: &[Candle], index: usize) -> Signal;

    /// Получить имя стратегии
    fn name(&self) -> &str;

    /// Получить текущие параметры
    fn parameters(&self) -> HashMap<String, f64>;
}
```

## Шаг 4: Пример стратегии - Пересечение скользящих средних

```rust
// src/strategy/moving_average.rs
use super::{Signal, Strategy};
use crate::data::Candle;
use std::collections::HashMap;

pub struct MovingAverageCrossover {
    short_period: usize,
    long_period: usize,
    last_signal: Signal,
}

impl MovingAverageCrossover {
    pub fn new(short_period: usize, long_period: usize) -> Self {
        MovingAverageCrossover {
            short_period,
            long_period,
            last_signal: Signal::Hold,
        }
    }

    fn calculate_sma(&self, candles: &[Candle], period: usize) -> Option<f64> {
        if candles.len() < period {
            return None;
        }

        let sum: f64 = candles.iter()
            .rev()
            .take(period)
            .map(|c| c.close)
            .sum();

        Some(sum / period as f64)
    }
}

impl Strategy for MovingAverageCrossover {
    fn initialize(&mut self, params: HashMap<String, f64>) {
        if let Some(&short) = params.get("short_period") {
            self.short_period = short as usize;
        }
        if let Some(&long) = params.get("long_period") {
            self.long_period = long as usize;
        }
    }

    fn generate_signal(&mut self, candles: &[Candle], index: usize) -> Signal {
        let data = &candles[..=index];

        if data.len() < self.long_period {
            return Signal::Hold;
        }

        let short_ma = self.calculate_sma(data, self.short_period);
        let long_ma = self.calculate_sma(data, self.long_period);

        match (short_ma, long_ma) {
            (Some(short), Some(long)) => {
                let signal = if short > long && self.last_signal != Signal::Buy {
                    Signal::Buy
                } else if short < long && self.last_signal != Signal::Sell {
                    Signal::Sell
                } else {
                    Signal::Hold
                };

                if signal != Signal::Hold {
                    self.last_signal = signal;
                }
                signal
            }
            _ => Signal::Hold,
        }
    }

    fn name(&self) -> &str {
        "MA Crossover"
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("short_period".to_string(), self.short_period as f64);
        params.insert("long_period".to_string(), self.long_period as f64);
        params
    }
}
```

## Шаг 5: Основной движок бэктестинга

```rust
// src/engine/mod.rs
use crate::data::Candle;
use crate::strategy::{Signal, Strategy};
use crate::utils::commissions::{CommissionModel, SlippageModel};

pub struct BacktestEngine {
    initial_capital: f64,
    cash: f64,
    positions: HashMap<String, Position>,
    trades: Vec<Trade>,
    equity_curve: Vec<PortfolioSnapshot>,
    commission_model: CommissionModel,
    slippage_model: SlippageModel,
    peak_equity: f64,
    max_drawdown: f64,
}

impl BacktestEngine {
    pub fn new(initial_capital: f64) -> Self {
        BacktestEngine {
            initial_capital,
            cash: initial_capital,
            positions: HashMap::new(),
            trades: Vec::new(),
            equity_curve: Vec::new(),
            commission_model: CommissionModel::binance_spot(),
            slippage_model: SlippageModel::simple(0.05), // 0.05% проскальзывание
            peak_equity: initial_capital,
            max_drawdown: 0.0,
        }
    }

    pub fn set_commission_model(&mut self, model: CommissionModel) {
        self.commission_model = model;
    }

    pub fn set_slippage_model(&mut self, model: SlippageModel) {
        self.slippage_model = model;
    }

    pub fn run(
        &mut self,
        symbol: String,
        candles: &[Candle],
        strategy: &mut dyn Strategy,
    ) -> BacktestResult {
        // Сброс состояния
        self.cash = self.initial_capital;
        self.positions.clear();
        self.trades.clear();
        self.equity_curve.clear();
        self.peak_equity = self.initial_capital;
        self.max_drawdown = 0.0;

        // Прогон по историческим данным
        for (index, candle) in candles.iter().enumerate() {
            let signal = strategy.generate_signal(candles, index);

            match signal {
                Signal::Buy => self.execute_buy(&symbol, candle),
                Signal::Sell => self.execute_sell(&symbol, candle),
                Signal::Hold => {}
            }

            // Обновление снимка портфеля
            self.update_snapshot(candle);
        }

        // Закрытие оставшихся позиций в конце
        if let Some(last_candle) = candles.last() {
            if self.positions.contains_key(&symbol) {
                self.execute_sell(&symbol, last_candle);
            }
        }

        self.generate_result()
    }

    fn execute_buy(&mut self, symbol: &str, candle: &Candle) {
        // Не покупаем, если уже есть позиция
        if self.positions.contains_key(symbol) {
            return;
        }

        let price = candle.close;
        let slippage = self.slippage_model.calculate(price, 1.0, candle.volume);
        let execution_price = price + slippage;

        // Используем 95% доступных средств (оставляем на комиссии)
        let available = self.cash * 0.95;
        let size = available / execution_price;
        let value = size * execution_price;

        let commission = self.commission_model.calculate(value, false);
        let total_cost = value + commission;

        if total_cost <= self.cash {
            self.cash -= total_cost;

            self.positions.insert(
                symbol.to_string(),
                Position {
                    symbol: symbol.to_string(),
                    size,
                    entry_price: execution_price,
                    entry_time: candle.timestamp,
                    realized_pnl: 0.0,
                    unrealized_pnl: 0.0,
                },
            );

            self.trades.push(Trade {
                timestamp: candle.timestamp,
                symbol: symbol.to_string(),
                side: Side::Buy,
                price: execution_price,
                size,
                commission,
                pnl: 0.0,
            });
        }
    }

    fn execute_sell(&mut self, symbol: &str, candle: &Candle) {
        if let Some(position) = self.positions.remove(symbol) {
            let price = candle.close;
            let slippage = self.slippage_model.calculate(price, position.size, candle.volume);
            let execution_price = price - slippage;

            let value = position.size * execution_price;
            let commission = self.commission_model.calculate(value, false);

            let pnl = (execution_price - position.entry_price) * position.size - commission;

            self.cash += value - commission;

            self.trades.push(Trade {
                timestamp: candle.timestamp,
                symbol: symbol.to_string(),
                side: Side::Sell,
                price: execution_price,
                size: position.size,
                commission,
                pnl,
            });
        }
    }

    fn update_snapshot(&mut self, candle: &Candle) {
        let mut positions_value = 0.0;

        for position in self.positions.values_mut() {
            position.update_unrealized_pnl(candle.close);
            positions_value += position.market_value(candle.close);
        }

        let total_equity = self.cash + positions_value;

        // Обновление пика и просадки
        if total_equity > self.peak_equity {
            self.peak_equity = total_equity;
        }

        let current_drawdown = (self.peak_equity - total_equity) / self.peak_equity;
        if current_drawdown > self.max_drawdown {
            self.max_drawdown = current_drawdown;
        }

        self.equity_curve.push(PortfolioSnapshot {
            timestamp: candle.timestamp,
            cash: self.cash,
            positions_value,
            total_equity,
            drawdown: current_drawdown,
        });
    }

    fn generate_result(&self) -> BacktestResult {
        let final_equity = self.equity_curve.last()
            .map(|s| s.total_equity)
            .unwrap_or(self.initial_capital);

        let total_return = (final_equity - self.initial_capital) / self.initial_capital;

        BacktestResult {
            initial_capital: self.initial_capital,
            final_equity,
            total_return,
            total_trades: self.trades.len(),
            winning_trades: self.trades.iter().filter(|t| t.pnl > 0.0).count(),
            losing_trades: self.trades.iter().filter(|t| t.pnl < 0.0).count(),
            max_drawdown: self.max_drawdown,
            equity_curve: self.equity_curve.clone(),
            trades: self.trades.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub initial_capital: f64,
    pub final_equity: f64,
    pub total_return: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub max_drawdown: f64,
    pub equity_curve: Vec<PortfolioSnapshot>,
    pub trades: Vec<Trade>,
}
```

## Шаг 6: Метрики производительности

```rust
// src/metrics/mod.rs
use crate::engine::{BacktestResult, Trade};

pub struct PerformanceMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub avg_trade_duration_hours: f64,
}

impl PerformanceMetrics {
    pub fn calculate(result: &BacktestResult, trading_days: usize) -> Self {
        let total_return = result.total_return;
        let years = trading_days as f64 / 365.0;
        let annualized_return = if years > 0.0 {
            ((1.0 + total_return).powf(1.0 / years)) - 1.0
        } else {
            0.0
        };

        let sharpe_ratio = Self::calculate_sharpe_ratio(&result.equity_curve);
        let (win_rate, profit_factor, avg_win, avg_loss, largest_win, largest_loss) =
            Self::calculate_trade_stats(&result.trades);

        let avg_duration = Self::calculate_avg_duration(&result.trades);

        PerformanceMetrics {
            total_return,
            annualized_return,
            sharpe_ratio,
            max_drawdown: result.max_drawdown,
            win_rate,
            profit_factor,
            average_win: avg_win,
            average_loss: avg_loss,
            largest_win,
            largest_loss,
            avg_trade_duration_hours: avg_duration,
        }
    }

    fn calculate_sharpe_ratio(equity_curve: &[crate::engine::PortfolioSnapshot]) -> f64 {
        if equity_curve.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = equity_curve
            .windows(2)
            .map(|w| (w[1].total_equity - w[0].total_equity) / w[0].total_equity)
            .collect();

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            // Годовой Sharpe (предполагаем дневные данные)
            mean / std_dev * (365.0_f64).sqrt()
        } else {
            0.0
        }
    }

    fn calculate_trade_stats(trades: &[Trade]) -> (f64, f64, f64, f64, f64, f64) {
        let closed_trades: Vec<&Trade> = trades
            .iter()
            .filter(|t| t.side == crate::engine::Side::Sell)
            .collect();

        if closed_trades.is_empty() {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        let winning: Vec<f64> = closed_trades
            .iter()
            .filter(|t| t.pnl > 0.0)
            .map(|t| t.pnl)
            .collect();

        let losing: Vec<f64> = closed_trades
            .iter()
            .filter(|t| t.pnl < 0.0)
            .map(|t| t.pnl.abs())
            .collect();

        let win_rate = winning.len() as f64 / closed_trades.len() as f64;

        let total_wins: f64 = winning.iter().sum();
        let total_losses: f64 = losing.iter().sum();
        let profit_factor = if total_losses > 0.0 {
            total_wins / total_losses
        } else {
            0.0
        };

        let avg_win = if !winning.is_empty() {
            winning.iter().sum::<f64>() / winning.len() as f64
        } else {
            0.0
        };

        let avg_loss = if !losing.is_empty() {
            losing.iter().sum::<f64>() / losing.len() as f64
        } else {
            0.0
        };

        let largest_win = winning.iter().cloned().fold(0.0, f64::max);
        let largest_loss = losing.iter().cloned().fold(0.0, f64::max);

        (win_rate, profit_factor, avg_win, avg_loss, largest_win, largest_loss)
    }

    fn calculate_avg_duration(trades: &[Trade]) -> f64 {
        if trades.len() < 2 {
            return 0.0;
        }

        let mut total_duration = 0u64;
        let mut count = 0;

        for window in trades.windows(2) {
            if window[0].side == crate::engine::Side::Buy
                && window[1].side == crate::engine::Side::Sell {
                total_duration += window[1].timestamp - window[0].timestamp;
                count += 1;
            }
        }

        if count > 0 {
            (total_duration as f64 / count as f64) / 3600.0 // Конвертация в часы
        } else {
            0.0
        }
    }

    pub fn print(&self) {
        println!("=== Метрики производительности ===");
        println!("Общая доходность:      {:.2}%", self.total_return * 100.0);
        println!("Годовая доходность:    {:.2}%", self.annualized_return * 100.0);
        println!("Sharpe Ratio:          {:.2}", self.sharpe_ratio);
        println!("Максимальная просадка: {:.2}%", self.max_drawdown * 100.0);
        println!("Win Rate:              {:.2}%", self.win_rate * 100.0);
        println!("Profit Factor:         {:.2}", self.profit_factor);
        println!("Средний выигрыш:       ${:.2}", self.average_win);
        println!("Средний проигрыш:      ${:.2}", self.average_loss);
        println!("Наибольший выигрыш:    ${:.2}", self.largest_win);
        println!("Наибольший проигрыш:   ${:.2}", self.largest_loss);
        println!("Средняя длит. сделки:  {:.1} часов", self.avg_trade_duration_hours);
    }
}
```

## Шаг 7: Walk-Forward валидация

```rust
// src/validation/walk_forward.rs
use crate::data::Candle;
use crate::engine::BacktestEngine;
use crate::strategy::Strategy;

pub struct WalkForwardConfig {
    pub train_size: usize,   // Количество свечей для обучения
    pub test_size: usize,    // Количество свечей для тестирования
    pub step_size: usize,    // Шаг между окнами
}

pub struct WalkForwardResult {
    pub windows: Vec<WindowResult>,
    pub average_train_return: f64,
    pub average_test_return: f64,
    pub consistency_score: f64,
}

pub struct WindowResult {
    pub window_id: usize,
    pub train_return: f64,
    pub test_return: f64,
    pub train_sharpe: f64,
    pub test_sharpe: f64,
}

pub fn walk_forward_analysis(
    candles: &[Candle],
    strategy_factory: &dyn Fn() -> Box<dyn Strategy>,
    config: WalkForwardConfig,
    initial_capital: f64,
) -> WalkForwardResult {
    let mut windows = Vec::new();
    let mut position = 0;
    let mut window_id = 0;

    while position + config.train_size + config.test_size <= candles.len() {
        let train_data = &candles[position..position + config.train_size];
        let test_data = &candles[position + config.train_size..
                                 position + config.train_size + config.test_size];

        // Обучение на обучающем окне
        let mut strategy = strategy_factory();
        let mut engine = BacktestEngine::new(initial_capital);
        let train_result = engine.run("BTC".to_string(), train_data, strategy.as_mut());

        // Тестирование на следующем окне
        let mut engine = BacktestEngine::new(initial_capital);
        let test_result = engine.run("BTC".to_string(), test_data, strategy.as_mut());

        let train_metrics = crate::metrics::PerformanceMetrics::calculate(
            &train_result,
            config.train_size
        );
        let test_metrics = crate::metrics::PerformanceMetrics::calculate(
            &test_result,
            config.test_size
        );

        windows.push(WindowResult {
            window_id,
            train_return: train_result.total_return,
            test_return: test_result.total_return,
            train_sharpe: train_metrics.sharpe_ratio,
            test_sharpe: test_metrics.sharpe_ratio,
        });

        position += config.step_size;
        window_id += 1;
    }

    let avg_train = windows.iter().map(|w| w.train_return).sum::<f64>() / windows.len() as f64;
    let avg_test = windows.iter().map(|w| w.test_return).sum::<f64>() / windows.len() as f64;

    // Оценка консистентности: сколько окон имели положительную доходность на тесте
    let positive_windows = windows.iter().filter(|w| w.test_return > 0.0).count();
    let consistency = positive_windows as f64 / windows.len() as f64;

    WalkForwardResult {
        windows,
        average_train_return: avg_train,
        average_test_return: avg_test,
        consistency_score: consistency,
    }
}
```

## Шаг 8: Главный пример - Собираем всё вместе

```rust
// src/main.rs
use backtesting_engine::data::Candle;
use backtesting_engine::engine::BacktestEngine;
use backtesting_engine::strategy::moving_average::MovingAverageCrossover;
use backtesting_engine::metrics::PerformanceMetrics;
use backtesting_engine::validation::walk_forward::*;
use backtesting_engine::utils::commissions::CommissionModel;

fn main() {
    println!("=== Демонстрация бэктестинг движка ===\n");

    // Загрузка примерных данных (в реальном проекте загружать из CSV/базы данных)
    let candles = generate_sample_data();
    println!("Загружено {} свечей\n", candles.len());

    // Тест 1: Простой бэктест
    println!("ТЕСТ 1: Простой бэктест");
    println!("------------------------");
    run_simple_backtest(&candles);
    println!();

    // Тест 2: Walk-forward валидация
    println!("ТЕСТ 2: Walk-Forward валидация");
    println!("-------------------------------");
    run_walk_forward(&candles);
    println!();

    // Тест 3: Сравнение параметров
    println!("ТЕСТ 3: Сравнение параметров");
    println!("-----------------------------");
    compare_parameters(&candles);
}

fn run_simple_backtest(candles: &[Candle]) {
    let mut engine = BacktestEngine::new(10000.0);
    engine.set_commission_model(CommissionModel::binance_spot());

    let mut strategy = MovingAverageCrossover::new(10, 30);
    let result = engine.run("BTC".to_string(), candles, &mut strategy);

    println!("Начальный капитал: ${:.2}", result.initial_capital);
    println!("Конечный капитал:  ${:.2}", result.final_equity);
    println!("Общая доходность:  {:.2}%", result.total_return * 100.0);
    println!("Всего сделок:      {}", result.total_trades);
    println!("Прибыльных сделок: {}", result.winning_trades);
    println!("Убыточных сделок:  {}", result.losing_trades);

    let metrics = PerformanceMetrics::calculate(&result, candles.len());
    println!("\nДетальные метрики:");
    metrics.print();
}

fn run_walk_forward(candles: &[Candle]) {
    let config = WalkForwardConfig {
        train_size: 100,
        test_size: 50,
        step_size: 50,
    };

    let strategy_factory = || -> Box<dyn backtesting_engine::strategy::Strategy> {
        Box::new(MovingAverageCrossover::new(10, 30))
    };

    let wf_result = walk_forward_analysis(candles, &strategy_factory, config, 10000.0);

    println!("Количество окон: {}", wf_result.windows.len());
    println!("Средняя доходность на обучении: {:.2}%", wf_result.average_train_return * 100.0);
    println!("Средняя доходность на тесте:    {:.2}%", wf_result.average_test_return * 100.0);
    println!("Оценка консистентности:         {:.2}%", wf_result.consistency_score * 100.0);

    println!("\nПо каждому окну:");
    for window in &wf_result.windows {
        println!("  Окно {}: Обучение {:.2}%, Тест {:.2}%",
            window.window_id,
            window.train_return * 100.0,
            window.test_return * 100.0
        );
    }
}

fn compare_parameters(candles: &[Candle]) {
    let param_sets = [
        (5, 20),
        (10, 30),
        (20, 50),
        (10, 50),
    ];

    println!("Сравнение различных параметров MA:\n");
    println!("{:<15} {:<15} {:<15} {:<15}", "Параметры", "Доходность", "Sharpe", "Макс. просадка");
    println!("{}", "-".repeat(60));

    for (short, long) in param_sets.iter() {
        let mut engine = BacktestEngine::new(10000.0);
        let mut strategy = MovingAverageCrossover::new(*short, *long);
        let result = engine.run("BTC".to_string(), candles, &mut strategy);
        let metrics = PerformanceMetrics::calculate(&result, candles.len());

        println!("{:<15} {:<15.2}% {:<15.2} {:<15.2}%",
            format!("MA({},{})", short, long),
            result.total_return * 100.0,
            metrics.sharpe_ratio,
            result.max_drawdown * 100.0
        );
    }
}

// Генерация примерных данных для демонстрации
fn generate_sample_data() -> Vec<Candle> {
    let mut candles = Vec::new();
    let mut price = 40000.0;
    let base_time = 1609459200; // 2021-01-01

    for i in 0..500 {
        // Простое случайное блуждание с трендом
        let change = (rand::random::<f64>() - 0.48) * 100.0;
        price += change;
        price = price.max(30000.0).min(60000.0);

        let volatility = price * 0.005;
        let high = price + volatility;
        let low = price - volatility;

        candles.push(Candle {
            timestamp: base_time + (i * 3600), // 1-часовые свечи
            open: price,
            high,
            low,
            close: price,
            volume: 100.0 + rand::random::<f64>() * 50.0,
        });
    }

    candles
}

// Примечание: Добавь крейт rand в Cargo.toml:
// [dependencies]
// rand = "0.8"
// serde = { version = "1.0", features = ["derive"] }
```

## Что мы изучили

Этот комплексный проект объединяет все концепции бэктестинга:

| Компонент | Назначение |
|-----------|------------|
| **Структуры данных** | OHLCV свечи, позиции, сделки, снимки портфеля |
| **Комиссии и проскальзывание** | Реалистичное моделирование затрат для точных результатов |
| **Трейт стратегии** | Гибкий интерфейс для любой торговой стратегии |
| **Движок бэктестинга** | Основная симуляция с исполнением ордеров |
| **Метрики производительности** | Комплексная статистика (Sharpe, просадка, win rate) |
| **Walk-Forward анализ** | Валидация вне выборки |
| **Управление позициями** | Вход, выход, отслеживание P&L |
| **Управление рисками** | Мониторинг просадок, расчёт размера позиций |

## Домашнее задание

1. **Добавить стратегию возврата к среднему**: Реализуй стратегию mean reversion, которая:
   - Рассчитывает полосы Боллинджера
   - Покупает при касании нижней полосы
   - Продаёт при касании верхней полосы
   - Сравни производительность со стратегией на скользящих средних

2. **Реализовать K-Fold кросс-валидацию**: Добавь модуль кросс-валидации:
   - Раздели данные на K блоков
   - Обучай на K-1 блоках, тестируй на оставшемся
   - Вычисли среднюю производительность по всем блокам
   - Обнаружь переобучение, сравнивая обучение и тест

3. **Добавить генератор HTML отчётов**: Создай детальные HTML отчёты с:
   - Графиком кривой доходности
   - Графиком просадок
   - Гистограммой распределения сделок
   - Таблицей месячной доходности
   - Диаграммой рассеяния риск/доходность

4. **Оптимизация параметров**: Реализуй grid search оптимизатор:
   - Тестируй множественные комбинации параметров
   - Ранжируй по Sharpe ratio или другой метрике
   - Применяй walk-forward к каждой комбинации
   - Найди устойчивые параметры на разных периодах

5. **Размер позиции с учётом риска**: Добавь расчёт размера позиций на основе:
   - Критерия Келли
   - Метода фиксированной доли
   - Расчёта на основе волатильности
   - Ограничения максимальной просадки

6. **Добавить больше метрик**: Реализуй дополнительные метрики:
   - Sortino ratio (отклонение вниз)
   - Calmar ratio (доходность/макс. просадка)
   - Omega ratio
   - Анализ серий выигрышей/проигрышей
   - Время восстановления от просадок

## Идеи для расширения

Для амбициозных:

1. **Поддержка нескольких активов**: Бэктестинг портфелей с несколькими инструментами
2. **Короткие продажи**: Добавить поддержку шортовых позиций
3. **Кредитное плечо**: Реализовать расчёты маржи и кредитного плеча
4. **Stop Loss / Take Profit**: Автоматические правила выхода
5. **Оптимизация генетическим алгоритмом**: Эволюция параметров стратегии
6. **Анализ транзакционных издержек**: Детальная разбивка комиссий
7. **Сравнение с бенчмарком**: Сравнение стратегии с buy-and-hold
8. **Монте-Карло симуляция**: Рандомизация порядка сделок для проверки устойчивости

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md) | [Следующий день →](../305-*/ru.md)
