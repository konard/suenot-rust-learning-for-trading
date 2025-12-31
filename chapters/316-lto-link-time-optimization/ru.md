# День 316: LTO: Link Time Optimization

## Аналогия из трейдинга

Представь, что ты строишь высокочастотную торговую систему (HFT). У тебя есть несколько модулей:
- Модуль получения рыночных данных
- Модуль анализа сигналов
- Модуль исполнения ордеров
- Модуль управления рисками

При обычной компиляции каждый модуль компилируется отдельно, и компилятор не может оптимизировать взаимодействие между ними. Это как если бы каждый трейдер в команде работал изолированно, не зная о действиях других.

**LTO (Link Time Optimization)** — это как общий брифинг всей команды перед торговой сессией. Линкер видит весь код целиком и может:
- Убрать дублирующиеся функции (один аналитик вместо нескольких)
- Встроить функции из других модулей (прямая связь без посредников)
- Оптимизировать передачу данных между модулями (единая система коммуникации)

Результат — ваша торговая система работает быстрее, потому что каждая микросекунда на счету в HFT.

## Что такое LTO?

**LTO (Link Time Optimization)** — это техника оптимизации, при которой компилятор откладывает некоторые оптимизации до этапа линковки, когда виден весь код программы.

### Обычная компиляция vs LTO

```
Обычная компиляция:
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  module_a.rs│───►│ module_a.o  │    │             │
└─────────────┘    └─────────────┘    │             │
                                      │             │
┌─────────────┐    ┌─────────────┐    │   Linker    │───► binary
│  module_b.rs│───►│ module_b.o  │    │  (простая   │
└─────────────┘    └─────────────┘    │   сборка)   │
                                      │             │
┌─────────────┐    ┌─────────────┐    │             │
│  module_c.rs│───►│ module_c.o  │    │             │
└─────────────┘    └─────────────┘    └─────────────┘

LTO компиляция:
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  module_a.rs│───►│ module_a.bc │    │             │
└─────────────┘    └─────────────┘    │   Linker    │
                                      │     +       │
┌─────────────┐    ┌─────────────┐    │   LLVM      │───► binary
│  module_b.rs│───►│ module_b.bc │    │ оптимизации │    (быстрее!)
└─────────────┘    └─────────────┘    │             │
                                      │             │
┌─────────────┐    ┌─────────────┐    │             │
│  module_c.rs│───►│ module_c.bc │    │             │
└─────────────┘    └─────────────┘    └─────────────┘
```

### Почему это важно в трейдинге?

| Сценарий | Преимущество LTO |
|----------|------------------|
| **HFT системы** | Каждая микросекунда важна, LTO может сэкономить критичные циклы |
| **Анализ больших данных** | Оптимизация циклов обработки миллионов свечей |
| **Бэктестинг** | Ускорение тестов на исторических данных |
| **Риск-менеджмент** | Быстрые расчёты VaR в реальном времени |
| **Маркет-мейкинг** | Минимальная латентность для котирования |

## Включение LTO в Rust

### Базовая настройка в Cargo.toml

```toml
[package]
name = "trading_engine"
version = "0.1.0"
edition = "2021"

[profile.release]
lto = true  # Включаем полный LTO
```

### Варианты LTO

```toml
[profile.release]
# Вариант 1: Полный LTO (максимальная оптимизация, долгая компиляция)
lto = true

# Вариант 2: "Fat" LTO (аналогично true)
lto = "fat"

# Вариант 3: "Thin" LTO (баланс скорости компиляции и оптимизации)
lto = "thin"

# Вариант 4: Отключить LTO
lto = false

# Вариант 5: Только внутри crate (по умолчанию)
lto = "off"
```

### Сравнение режимов LTO

| Режим | Время компиляции | Размер бинарника | Производительность |
|-------|------------------|------------------|-------------------|
| `off` | Быстрое | Большой | Базовая |
| `thin` | Среднее | Средний | Хорошая |
| `fat`/`true` | Медленное | Маленький | Максимальная |

## Пример: Торговый движок с LTO

### Структура проекта

```
trading_engine/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── market_data.rs    # Модуль рыночных данных
│   ├── signals.rs        # Модуль сигналов
│   ├── execution.rs      # Модуль исполнения
│   └── risk.rs           # Модуль риск-менеджмента
```

### Cargo.toml с оптимальными настройками

```toml
[package]
name = "trading_engine"
version = "0.1.0"
edition = "2021"

[dependencies]

[profile.release]
lto = "fat"           # Максимальная оптимизация
codegen-units = 1     # Один юнит для лучшей оптимизации
panic = "abort"       # Меньше кода для паники
strip = true          # Удалить символы отладки

[profile.release-with-debug]
inherits = "release"
debug = true          # Для профилирования
lto = "thin"          # Быстрее компиляция с отладкой
```

### market_data.rs — Модуль рыночных данных

```rust
//! Модуль получения и обработки рыночных данных

/// Структура тика (минимальная единица рыночных данных)
#[derive(Debug, Clone, Copy)]
pub struct Tick {
    pub timestamp: u64,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
}

impl Tick {
    /// Создать новый тик
    #[inline]
    pub fn new(timestamp: u64, bid: f64, ask: f64, bid_size: f64, ask_size: f64) -> Self {
        Self {
            timestamp,
            bid,
            ask,
            bid_size,
            ask_size,
        }
    }

    /// Рассчитать средню цену (mid price)
    #[inline]
    pub fn mid_price(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }

    /// Рассчитать спред
    #[inline]
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Рассчитать спред в базисных пунктах
    #[inline]
    pub fn spread_bps(&self) -> f64 {
        (self.spread() / self.mid_price()) * 10_000.0
    }
}

/// Агрегатор тиков в свечи
pub struct CandleAggregator {
    period_seconds: u64,
    current_open: f64,
    current_high: f64,
    current_low: f64,
    current_close: f64,
    current_volume: f64,
    period_start: u64,
}

impl CandleAggregator {
    pub fn new(period_seconds: u64) -> Self {
        Self {
            period_seconds,
            current_open: 0.0,
            current_high: f64::MIN,
            current_low: f64::MAX,
            current_close: 0.0,
            current_volume: 0.0,
            period_start: 0,
        }
    }

    /// Обработать тик и вернуть свечу, если период завершён
    #[inline]
    pub fn process_tick(&mut self, tick: &Tick) -> Option<Candle> {
        let price = tick.mid_price();
        let tick_period = tick.timestamp / self.period_seconds;

        if self.period_start == 0 {
            // Первый тик
            self.period_start = tick_period;
            self.current_open = price;
            self.current_high = price;
            self.current_low = price;
            self.current_close = price;
            self.current_volume = tick.bid_size + tick.ask_size;
            return None;
        }

        if tick_period > self.period_start {
            // Новый период — создаём свечу
            let candle = Candle {
                timestamp: self.period_start * self.period_seconds,
                open: self.current_open,
                high: self.current_high,
                low: self.current_low,
                close: self.current_close,
                volume: self.current_volume,
            };

            // Сбрасываем для нового периода
            self.period_start = tick_period;
            self.current_open = price;
            self.current_high = price;
            self.current_low = price;
            self.current_close = price;
            self.current_volume = tick.bid_size + tick.ask_size;

            Some(candle)
        } else {
            // Обновляем текущий период
            self.current_high = self.current_high.max(price);
            self.current_low = self.current_low.min(price);
            self.current_close = price;
            self.current_volume += tick.bid_size + tick.ask_size;
            None
        }
    }
}

/// Структура свечи (OHLCV)
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}
```

### signals.rs — Модуль торговых сигналов

```rust
//! Модуль генерации торговых сигналов

use crate::market_data::Candle;

/// Торговый сигнал
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Калькулятор SMA (Simple Moving Average)
pub struct SmaCalculator {
    period: usize,
    prices: Vec<f64>,
    sum: f64,
}

impl SmaCalculator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            prices: Vec::with_capacity(period),
            sum: 0.0,
        }
    }

    /// Добавить цену и получить текущее значение SMA
    #[inline]
    pub fn update(&mut self, price: f64) -> Option<f64> {
        if self.prices.len() < self.period {
            self.prices.push(price);
            self.sum += price;

            if self.prices.len() == self.period {
                Some(self.sum / self.period as f64)
            } else {
                None
            }
        } else {
            // Удаляем старую цену, добавляем новую
            let old_price = self.prices[0];
            self.sum = self.sum - old_price + price;

            // Сдвигаем массив
            self.prices.rotate_left(1);
            self.prices[self.period - 1] = price;

            Some(self.sum / self.period as f64)
        }
    }

    /// Текущее значение SMA (если доступно)
    #[inline]
    pub fn value(&self) -> Option<f64> {
        if self.prices.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

/// Калькулятор EMA (Exponential Moving Average)
pub struct EmaCalculator {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    count: usize,
    initial_sum: f64,
}

impl EmaCalculator {
    pub fn new(period: usize) -> Self {
        let multiplier = 2.0 / (period as f64 + 1.0);
        Self {
            period,
            multiplier,
            current_ema: None,
            count: 0,
            initial_sum: 0.0,
        }
    }

    /// Обновить EMA новой ценой
    #[inline]
    pub fn update(&mut self, price: f64) -> Option<f64> {
        self.count += 1;

        match self.current_ema {
            Some(ema) => {
                let new_ema = (price * self.multiplier) + (ema * (1.0 - self.multiplier));
                self.current_ema = Some(new_ema);
                Some(new_ema)
            }
            None => {
                self.initial_sum += price;
                if self.count >= self.period {
                    let sma = self.initial_sum / self.period as f64;
                    self.current_ema = Some(sma);
                    Some(sma)
                } else {
                    None
                }
            }
        }
    }

    /// Текущее значение EMA
    #[inline]
    pub fn value(&self) -> Option<f64> {
        self.current_ema
    }
}

/// Генератор сигналов на пересечении скользящих средних
pub struct CrossoverSignalGenerator {
    fast_ema: EmaCalculator,
    slow_ema: EmaCalculator,
    previous_fast: Option<f64>,
    previous_slow: Option<f64>,
}

impl CrossoverSignalGenerator {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_ema: EmaCalculator::new(fast_period),
            slow_ema: EmaCalculator::new(slow_period),
            previous_fast: None,
            previous_slow: None,
        }
    }

    /// Обработать свечу и получить сигнал
    #[inline]
    pub fn process_candle(&mut self, candle: &Candle) -> Signal {
        let price = candle.close;

        let fast = self.fast_ema.update(price);
        let slow = self.slow_ema.update(price);

        let signal = match (fast, slow, self.previous_fast, self.previous_slow) {
            (Some(f), Some(s), Some(pf), Some(ps)) => {
                // Пересечение снизу вверх — сигнал на покупку
                if pf <= ps && f > s {
                    Signal::Buy
                }
                // Пересечение сверху вниз — сигнал на продажу
                else if pf >= ps && f < s {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        self.previous_fast = fast;
        self.previous_slow = slow;

        signal
    }
}

/// Калькулятор RSI (Relative Strength Index)
pub struct RsiCalculator {
    period: usize,
    avg_gain: f64,
    avg_loss: f64,
    previous_price: Option<f64>,
    count: usize,
    initial_gains: Vec<f64>,
    initial_losses: Vec<f64>,
}

impl RsiCalculator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            avg_gain: 0.0,
            avg_loss: 0.0,
            previous_price: None,
            count: 0,
            initial_gains: Vec::with_capacity(period),
            initial_losses: Vec::with_capacity(period),
        }
    }

    /// Обновить RSI новой ценой
    #[inline]
    pub fn update(&mut self, price: f64) -> Option<f64> {
        if let Some(prev) = self.previous_price {
            let change = price - prev;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            self.count += 1;

            if self.count < self.period {
                self.initial_gains.push(gain);
                self.initial_losses.push(loss);
                self.previous_price = Some(price);
                return None;
            } else if self.count == self.period {
                self.initial_gains.push(gain);
                self.initial_losses.push(loss);
                self.avg_gain = self.initial_gains.iter().sum::<f64>() / self.period as f64;
                self.avg_loss = self.initial_losses.iter().sum::<f64>() / self.period as f64;
            } else {
                // Сглаживание по Уайлдеру
                self.avg_gain = (self.avg_gain * (self.period - 1) as f64 + gain) / self.period as f64;
                self.avg_loss = (self.avg_loss * (self.period - 1) as f64 + loss) / self.period as f64;
            }

            self.previous_price = Some(price);

            if self.avg_loss == 0.0 {
                Some(100.0)
            } else {
                let rs = self.avg_gain / self.avg_loss;
                Some(100.0 - (100.0 / (1.0 + rs)))
            }
        } else {
            self.previous_price = Some(price);
            None
        }
    }
}
```

### execution.rs — Модуль исполнения ордеров

```rust
//! Модуль исполнения торговых ордеров

use crate::signals::Signal;

/// Тип ордера
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// Сторона ордера
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Ордер
#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub timestamp: u64,
}

/// Исполненная сделка
#[derive(Debug, Clone)]
pub struct Trade {
    pub order_id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub commission: f64,
    pub timestamp: u64,
}

/// Менеджер исполнения ордеров
pub struct ExecutionManager {
    next_order_id: u64,
    commission_rate: f64, // В процентах
    pending_orders: Vec<Order>,
}

impl ExecutionManager {
    pub fn new(commission_rate: f64) -> Self {
        Self {
            next_order_id: 1,
            commission_rate,
            pending_orders: Vec::new(),
        }
    }

    /// Создать рыночный ордер из сигнала
    #[inline]
    pub fn signal_to_order(
        &mut self,
        signal: Signal,
        symbol: &str,
        quantity: f64,
        timestamp: u64,
    ) -> Option<Order> {
        let side = match signal {
            Signal::Buy => OrderSide::Buy,
            Signal::Sell => OrderSide::Sell,
            Signal::Hold => return None,
        };

        let order = Order {
            id: self.next_order_id,
            symbol: symbol.to_string(),
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            timestamp,
        };

        self.next_order_id += 1;
        Some(order)
    }

    /// Исполнить рыночный ордер по текущей цене
    #[inline]
    pub fn execute_market_order(&self, order: &Order, current_price: f64) -> Trade {
        let execution_price = match order.side {
            OrderSide::Buy => current_price * 1.0001,  // Проскальзывание при покупке
            OrderSide::Sell => current_price * 0.9999, // Проскальзывание при продаже
        };

        let commission = order.quantity * execution_price * (self.commission_rate / 100.0);

        Trade {
            order_id: order.id,
            symbol: order.symbol.clone(),
            side: order.side,
            quantity: order.quantity,
            price: execution_price,
            commission,
            timestamp: order.timestamp,
        }
    }

    /// Добавить лимитный ордер в очередь
    pub fn add_limit_order(&mut self, mut order: Order, limit_price: f64) {
        order.order_type = OrderType::Limit;
        order.price = Some(limit_price);
        self.pending_orders.push(order);
    }

    /// Проверить и исполнить лимитные ордера
    #[inline]
    pub fn check_pending_orders(&mut self, bid: f64, ask: f64) -> Vec<Trade> {
        let mut trades = Vec::new();
        let mut executed_indices = Vec::new();

        for (i, order) in self.pending_orders.iter().enumerate() {
            if let Some(limit_price) = order.price {
                let should_execute = match order.side {
                    OrderSide::Buy => ask <= limit_price,   // Покупка, если ask ниже лимита
                    OrderSide::Sell => bid >= limit_price, // Продажа, если bid выше лимита
                };

                if should_execute {
                    let execution_price = match order.side {
                        OrderSide::Buy => ask,
                        OrderSide::Sell => bid,
                    };

                    let commission =
                        order.quantity * execution_price * (self.commission_rate / 100.0);

                    trades.push(Trade {
                        order_id: order.id,
                        symbol: order.symbol.clone(),
                        side: order.side,
                        quantity: order.quantity,
                        price: execution_price,
                        commission,
                        timestamp: order.timestamp,
                    });

                    executed_indices.push(i);
                }
            }
        }

        // Удаляем исполненные ордера (в обратном порядке для корректных индексов)
        for i in executed_indices.into_iter().rev() {
            self.pending_orders.remove(i);
        }

        trades
    }
}
```

### risk.rs — Модуль риск-менеджмента

```rust
//! Модуль управления рисками

use crate::execution::{OrderSide, Trade};

/// Позиция по инструменту
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,       // Положительная — лонг, отрицательная — шорт
    pub average_price: f64,  // Средняя цена входа
    pub unrealized_pnl: f64, // Нереализованная прибыль/убыток
    pub realized_pnl: f64,   // Реализованная прибыль/убыток
}

impl Position {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            quantity: 0.0,
            average_price: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        }
    }

    /// Обновить позицию после сделки
    #[inline]
    pub fn update_from_trade(&mut self, trade: &Trade) {
        let trade_quantity = match trade.side {
            OrderSide::Buy => trade.quantity,
            OrderSide::Sell => -trade.quantity,
        };

        if self.quantity == 0.0 {
            // Открытие новой позиции
            self.quantity = trade_quantity;
            self.average_price = trade.price;
        } else if (self.quantity > 0.0) == (trade_quantity > 0.0) {
            // Увеличение позиции — пересчёт средней цены
            let total_cost = self.quantity * self.average_price + trade_quantity * trade.price;
            self.quantity += trade_quantity;
            self.average_price = total_cost / self.quantity;
        } else {
            // Уменьшение или разворот позиции
            let closing_quantity = trade_quantity.abs().min(self.quantity.abs());
            let pnl = closing_quantity * (trade.price - self.average_price) * self.quantity.signum();
            self.realized_pnl += pnl - trade.commission;

            self.quantity += trade_quantity;

            if self.quantity.abs() < 1e-10 {
                self.quantity = 0.0;
                self.average_price = 0.0;
            } else if self.quantity.signum() != (self.quantity - trade_quantity).signum() {
                // Разворот позиции
                self.average_price = trade.price;
            }
        }
    }

    /// Обновить нереализованный PnL по текущей цене
    #[inline]
    pub fn update_unrealized_pnl(&mut self, current_price: f64) {
        if self.quantity != 0.0 {
            self.unrealized_pnl = self.quantity * (current_price - self.average_price);
        } else {
            self.unrealized_pnl = 0.0;
        }
    }

    /// Общий PnL
    #[inline]
    pub fn total_pnl(&self) -> f64 {
        self.realized_pnl + self.unrealized_pnl
    }
}

/// Риск-менеджер
pub struct RiskManager {
    max_position_size: f64,    // Максимальный размер позиции
    max_drawdown_percent: f64, // Максимальная просадка в %
    daily_loss_limit: f64,     // Дневной лимит убытков
    peak_equity: f64,          // Пиковое значение эквити
    current_equity: f64,       // Текущее эквити
    daily_pnl: f64,            // Дневной PnL
    trading_halted: bool,      // Торговля остановлена
}

impl RiskManager {
    pub fn new(
        initial_equity: f64,
        max_position_size: f64,
        max_drawdown_percent: f64,
        daily_loss_limit: f64,
    ) -> Self {
        Self {
            max_position_size,
            max_drawdown_percent,
            daily_loss_limit,
            peak_equity: initial_equity,
            current_equity: initial_equity,
            daily_pnl: 0.0,
            trading_halted: false,
        }
    }

    /// Проверить, разрешена ли торговля
    #[inline]
    pub fn is_trading_allowed(&self) -> bool {
        !self.trading_halted
    }

    /// Проверить, допустим ли размер позиции
    #[inline]
    pub fn check_position_size(&self, quantity: f64) -> bool {
        quantity.abs() <= self.max_position_size
    }

    /// Обновить состояние риска после сделки
    #[inline]
    pub fn update_after_trade(&mut self, trade: &Trade, position: &Position) {
        // Обновляем дневной PnL
        self.daily_pnl = position.realized_pnl;

        // Обновляем эквити
        self.current_equity = self.peak_equity + position.total_pnl();

        // Обновляем пик
        if self.current_equity > self.peak_equity {
            self.peak_equity = self.current_equity;
        }

        // Проверяем условия остановки
        self.check_risk_limits();
    }

    /// Проверить лимиты риска
    #[inline]
    fn check_risk_limits(&mut self) {
        // Проверка дневного лимита
        if self.daily_pnl < -self.daily_loss_limit {
            self.trading_halted = true;
            println!("RISK: Дневной лимит убытков достигнут! PnL: {:.2}", self.daily_pnl);
            return;
        }

        // Проверка максимальной просадки
        let drawdown_percent =
            (self.peak_equity - self.current_equity) / self.peak_equity * 100.0;

        if drawdown_percent > self.max_drawdown_percent {
            self.trading_halted = true;
            println!(
                "RISK: Максимальная просадка достигнута! Drawdown: {:.2}%",
                drawdown_percent
            );
        }
    }

    /// Рассчитать оптимальный размер позиции (критерий Келли)
    #[inline]
    pub fn calculate_kelly_size(
        &self,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
    ) -> f64 {
        if avg_loss == 0.0 {
            return 0.0;
        }

        let win_loss_ratio = avg_win / avg_loss;
        let kelly = win_rate - (1.0 - win_rate) / win_loss_ratio;

        // Используем половину Келли для консервативности
        let half_kelly = kelly * 0.5;

        // Ограничиваем максимальным размером позиции
        half_kelly.max(0.0).min(self.max_position_size / self.current_equity)
    }

    /// Текущая просадка в процентах
    #[inline]
    pub fn current_drawdown_percent(&self) -> f64 {
        if self.peak_equity > 0.0 {
            (self.peak_equity - self.current_equity) / self.peak_equity * 100.0
        } else {
            0.0
        }
    }

    /// Сбросить дневную статистику
    pub fn reset_daily_stats(&mut self) {
        self.daily_pnl = 0.0;
        if !self.trading_halted || self.current_drawdown_percent() < self.max_drawdown_percent {
            self.trading_halted = false;
        }
    }
}

/// Калькулятор метрик производительности
pub struct PerformanceMetrics {
    trades: Vec<f64>,      // PnL каждой сделки
    equity_curve: Vec<f64>, // Кривая эквити
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            trades: Vec::new(),
            equity_curve: Vec::new(),
        }
    }

    /// Добавить результат сделки
    #[inline]
    pub fn add_trade(&mut self, pnl: f64, equity: f64) {
        self.trades.push(pnl);
        self.equity_curve.push(equity);
    }

    /// Рассчитать коэффициент Шарпа (упрощённый)
    #[inline]
    pub fn sharpe_ratio(&self, risk_free_rate: f64) -> Option<f64> {
        if self.trades.len() < 2 {
            return None;
        }

        let mean: f64 = self.trades.iter().sum::<f64>() / self.trades.len() as f64;
        let variance: f64 = self.trades
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (self.trades.len() - 1) as f64;

        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return None;
        }

        Some((mean - risk_free_rate) / std_dev)
    }

    /// Процент прибыльных сделок
    #[inline]
    pub fn win_rate(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }

        let wins = self.trades.iter().filter(|&&x| x > 0.0).count();
        Some(wins as f64 / self.trades.len() as f64 * 100.0)
    }

    /// Profit Factor
    #[inline]
    pub fn profit_factor(&self) -> Option<f64> {
        let gross_profit: f64 = self.trades.iter().filter(|&&x| x > 0.0).sum();
        let gross_loss: f64 = self.trades.iter().filter(|&&x| x < 0.0).sum::<f64>().abs();

        if gross_loss == 0.0 {
            return None;
        }

        Some(gross_profit / gross_loss)
    }

    /// Максимальная просадка
    #[inline]
    pub fn max_drawdown(&self) -> f64 {
        let mut peak = f64::MIN;
        let mut max_dd = 0.0f64;

        for &equity in &self.equity_curve {
            if equity > peak {
                peak = equity;
            }
            let dd = (peak - equity) / peak * 100.0;
            max_dd = max_dd.max(dd);
        }

        max_dd
    }
}
```

### main.rs — Основной модуль

```rust
//! Высокопроизводительный торговый движок с LTO оптимизацией

mod market_data;
mod signals;
mod execution;
mod risk;

use market_data::{Tick, CandleAggregator};
use signals::{CrossoverSignalGenerator, Signal, RsiCalculator};
use execution::ExecutionManager;
use risk::{Position, RiskManager, PerformanceMetrics};

use std::time::Instant;

fn main() {
    println!("=== Торговый движок с LTO оптимизацией ===\n");

    // Инициализация компонентов
    let mut aggregator = CandleAggregator::new(60); // 1-минутные свечи
    let mut signal_gen = CrossoverSignalGenerator::new(12, 26); // EMA 12/26
    let mut rsi = RsiCalculator::new(14);
    let mut execution = ExecutionManager::new(0.1); // 0.1% комиссия
    let mut position = Position::new("BTC/USD");
    let mut risk_manager = RiskManager::new(
        100_000.0,  // Начальный капитал
        10.0,       // Макс размер позиции
        20.0,       // Макс просадка 20%
        5_000.0,    // Дневной лимит убытков
    );
    let mut metrics = PerformanceMetrics::new();

    // Генерируем тестовые данные (имитация 100K тиков)
    let ticks = generate_test_ticks(100_000);

    println!("Обработка {} тиков...\n", ticks.len());

    let start = Instant::now();

    let mut trade_count = 0;

    for tick in &ticks {
        // Агрегируем тики в свечи
        if let Some(candle) = aggregator.process_tick(tick) {
            // Генерируем сигнал
            let signal = signal_gen.process_candle(&candle);
            let _rsi_value = rsi.update(candle.close);

            // Проверяем риски
            if !risk_manager.is_trading_allowed() {
                continue;
            }

            // Создаём и исполняем ордер
            if signal != Signal::Hold {
                if let Some(order) = execution.signal_to_order(
                    signal,
                    "BTC/USD",
                    0.1, // Размер позиции
                    candle.timestamp,
                ) {
                    // Проверяем размер позиции
                    if risk_manager.check_position_size(0.1) {
                        let trade = execution.execute_market_order(&order, tick.mid_price());

                        // Обновляем позицию
                        position.update_from_trade(&trade);
                        position.update_unrealized_pnl(tick.mid_price());

                        // Обновляем риск-менеджер
                        risk_manager.update_after_trade(&trade, &position);

                        // Записываем метрику
                        metrics.add_trade(
                            position.realized_pnl,
                            100_000.0 + position.total_pnl(),
                        );

                        trade_count += 1;
                    }
                }
            }
        }
    }

    let elapsed = start.elapsed();

    // Вывод результатов
    println!("=== Результаты ===\n");
    println!("Время обработки: {:?}", elapsed);
    println!("Тиков в секунду: {:.0}", ticks.len() as f64 / elapsed.as_secs_f64());
    println!("Всего сделок: {}", trade_count);
    println!();
    println!("Позиция: {:.4} BTC", position.quantity);
    println!("Средняя цена: ${:.2}", position.average_price);
    println!("Реализованный PnL: ${:.2}", position.realized_pnl);
    println!("Нереализованный PnL: ${:.2}", position.unrealized_pnl);
    println!("Общий PnL: ${:.2}", position.total_pnl());
    println!();

    if let Some(wr) = metrics.win_rate() {
        println!("Win Rate: {:.1}%", wr);
    }
    if let Some(pf) = metrics.profit_factor() {
        println!("Profit Factor: {:.2}", pf);
    }
    if let Some(sharpe) = metrics.sharpe_ratio(0.0) {
        println!("Sharpe Ratio: {:.2}", sharpe);
    }
    println!("Max Drawdown: {:.2}%", metrics.max_drawdown());
}

/// Генерация тестовых тиков
fn generate_test_ticks(count: usize) -> Vec<Tick> {
    let mut ticks = Vec::with_capacity(count);
    let mut price = 42000.0;
    let mut timestamp = 1700000000u64;

    for i in 0..count {
        // Случайное изменение цены
        let change = ((i * 17 + 13) % 100) as f64 / 100.0 - 0.5;
        price += change * 10.0;
        price = price.max(40000.0).min(45000.0);

        let spread = 0.5 + ((i * 7) % 10) as f64 / 10.0;
        let bid = price - spread / 2.0;
        let ask = price + spread / 2.0;

        ticks.push(Tick::new(
            timestamp,
            bid,
            ask,
            1.0 + (i % 5) as f64,
            1.0 + ((i + 3) % 5) as f64,
        ));

        timestamp += 100; // 100ms между тиками
    }

    ticks
}
```

## Измерение влияния LTO

### Бенчмарк: сравнение режимов

```rust
//! Бенчмарк для сравнения производительности с разными настройками LTO

use std::time::Instant;

/// Расчёт SMA — типичная операция в трейдинге
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);
    let mut sum: f64 = prices[..period].iter().sum();

    result.push(sum / period as f64);

    for i in period..prices.len() {
        sum = sum - prices[i - period] + prices[i];
        result.push(sum / period as f64);
    }

    result
}

/// Расчёт EMA
fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.is_empty() {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len());
    let multiplier = 2.0 / (period as f64 + 1.0);

    result.push(prices[0]);

    for &price in &prices[1..] {
        let prev_ema = *result.last().unwrap();
        let ema = (price * multiplier) + (prev_ema * (1.0 - multiplier));
        result.push(ema);
    }

    result
}

/// Проверка пересечения
#[inline(always)]
fn check_crossover(fast: &[f64], slow: &[f64]) -> Vec<i32> {
    fast.iter()
        .zip(slow.iter())
        .zip(fast.iter().skip(1).zip(slow.iter().skip(1)))
        .map(|((pf, ps), (cf, cs))| {
            if pf <= ps && cf > cs {
                1  // Бычье пересечение
            } else if pf >= ps && cf < cs {
                -1 // Медвежье пересечение
            } else {
                0  // Нет пересечения
            }
        })
        .collect()
}

fn main() {
    // Генерируем данные
    let prices: Vec<f64> = (0..1_000_000)
        .map(|i| 100.0 + (i as f64 * 0.01).sin() * 10.0)
        .collect();

    println!("=== Бенчмарк LTO ===\n");
    println!("Данных: {} точек\n", prices.len());

    // Бенчмарк SMA
    let start = Instant::now();
    for _ in 0..10 {
        let _ = calculate_sma(&prices, 20);
    }
    let sma_time = start.elapsed() / 10;
    println!("SMA(20): {:?} за итерацию", sma_time);

    // Бенчмарк EMA
    let start = Instant::now();
    for _ in 0..10 {
        let _ = calculate_ema(&prices, 20);
    }
    let ema_time = start.elapsed() / 10;
    println!("EMA(20): {:?} за итерацию", ema_time);

    // Бенчмарк пересечения
    let fast_ema = calculate_ema(&prices, 12);
    let slow_ema = calculate_ema(&prices, 26);

    let start = Instant::now();
    for _ in 0..10 {
        let _ = check_crossover(&fast_ema, &slow_ema);
    }
    let cross_time = start.elapsed() / 10;
    println!("Crossover check: {:?} за итерацию", cross_time);

    println!("\nСовет: Скомпилируйте с разными настройками LTO и сравните!");
    println!("  cargo build --release                    # lto = false");
    println!("  cargo build --release --config 'profile.release.lto=\"thin\"'");
    println!("  cargo build --release --config 'profile.release.lto=\"fat\"'");
}
```

### Скрипт для сравнения производительности

```bash
#!/bin/bash
# benchmark_lto.sh - Сравнение производительности с разными режимами LTO

echo "=== Бенчмарк LTO ==="

# Сборка без LTO
echo -e "\n[1/3] Сборка без LTO..."
cargo build --release 2>/dev/null
echo "Размер бинарника (без LTO):"
ls -lh target/release/trading_engine | awk '{print $5}'
echo "Запуск бенчмарка..."
time target/release/trading_engine

# Сборка с Thin LTO
echo -e "\n[2/3] Сборка с Thin LTO..."
cargo build --release --config 'profile.release.lto="thin"' 2>/dev/null
echo "Размер бинарника (Thin LTO):"
ls -lh target/release/trading_engine | awk '{print $5}'
echo "Запуск бенчмарка..."
time target/release/trading_engine

# Сборка с Fat LTO
echo -e "\n[3/3] Сборка с Fat LTO..."
cargo build --release --config 'profile.release.lto="fat"' --config 'profile.release.codegen-units=1' 2>/dev/null
echo "Размер бинарника (Fat LTO):"
ls -lh target/release/trading_engine | awk '{print $5}'
echo "Запуск бенчмарка..."
time target/release/trading_engine

echo -e "\n=== Сравнение завершено ==="
```

## Когда использовать LTO

### Рекомендации

| Ситуация | Рекомендация |
|----------|--------------|
| **Разработка** | `lto = false` — быстрая компиляция |
| **CI/CD тесты** | `lto = "thin"` — баланс скорости |
| **Production HFT** | `lto = "fat"` — максимальная скорость |
| **Бэктестинг** | `lto = "thin"` — хороший баланс |
| **Отладка** | `lto = false` — сохранение символов |

### Дополнительные оптимизации с LTO

```toml
[profile.release]
lto = "fat"
codegen-units = 1        # Один юнит компиляции
opt-level = 3            # Максимальная оптимизация
panic = "abort"          # Убрать код размотки стека
strip = true             # Убрать символы отладки
debug = false            # Без отладочной информации

[profile.release.package."*"]
opt-level = 3            # Оптимизировать зависимости тоже
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **LTO** | Link Time Optimization — оптимизация на этапе линковки |
| **Fat LTO** | Полная межмодульная оптимизация (медленная компиляция) |
| **Thin LTO** | Быстрая параллельная LTO с хорошими результатами |
| **codegen-units** | Количество параллельных единиц компиляции |
| **Inlining** | LTO позволяет инлайнить функции между модулями |
| **Dead code elimination** | Удаление неиспользуемого кода из всей программы |

## Практические задания

1. **Бенчмарк торгового индикатора**: Реализуй расчёт индикатора MACD и измерь производительность с разными режимами LTO. Используй `criterion` для точных измерений.

2. **Профилирование**: Используй `perf` или `flamegraph` для анализа того, какие функции были заинлайнены с LTO и без него. Сравни графы вызовов.

3. **Размер бинарника**: Создай таблицу сравнения размера бинарника твоего торгового приложения с разными настройками (`lto`, `strip`, `panic`).

4. **Кросс-модульная оптимизация**: Создай библиотеку торговых индикаторов и основное приложение. Измерь, насколько LTO улучшает производительность при вызове функций из библиотеки.

## Домашнее задание

1. **Оптимальный профиль сборки**: Создай несколько профилей сборки для разных сценариев использования торгового бота:
   - `dev` — быстрая разработка
   - `test` — для запуска тестов
   - `bench` — для бенчмарков
   - `production` — для боевого использования
   Измерь время компиляции и производительность для каждого.

2. **Сравнение с C++**: Напиши эквивалентный код расчёта SMA/EMA на C++ с `-flto`. Сравни производительность Rust LTO vs C++ LTO на идентичных данных.

3. **Анализ влияния LTO**: Используя `llvm-mca` или аналогичные инструменты, проанализируй сгенерированный машинный код для критичной функции (например, расчёт RSI) с LTO и без. Документируй различия.

4. **Интеграция с CI/CD**: Настрой GitHub Actions или аналогичную CI систему для автоматической сборки торгового приложения с разными профилями LTO и публикации артефактов. Добавь автоматические бенчмарки для отслеживания регрессий производительности.

## Навигация

[← Предыдущий день](../314-ffi-c-library-integration/ru.md) | [Следующий день →](../317-*/ru.md)
