# День 276: Event-Driven Backtester: Архитектура на событиях

## Аналогия из трейдинга

Представь себе торговый зал биржи: каждое действие — это событие. Пришла новая котировка — событие. Трейдер разместил ордер — событие. Ордер исполнился — событие. Изменился баланс — событие. Всё, что происходит, можно описать как последовательность событий.

**Event-Driven Architecture (EDA)** — это архитектурный паттерн, где система реагирует на события, а не выполняет код последовательно. Это идеальный подход для бэктестинга торговых стратегий:

- **Реалистичность**: Реальный трейдинг работает на событиях — котировки, ордера, исполнения
- **Модульность**: Каждый компонент отвечает за свою часть логики
- **Тестируемость**: Легко подменять компоненты для тестирования
- **Переносимость**: Тот же код работает и в бэктесте, и в live-трейдинге

## Что такое Event-Driven Backtester?

Event-Driven Backtester состоит из нескольких ключевых компонентов:

```
┌─────────────────────────────────────────────────────────────────┐
│                        EVENT QUEUE                               │
│  [MarketEvent] → [SignalEvent] → [OrderEvent] → [FillEvent]     │
└─────────────────────────────────────────────────────────────────┘
       ↓                ↓                ↓              ↓
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ DataHandler  │ │   Strategy   │ │  Portfolio   │ │  Execution   │
│              │ │              │ │              │ │   Handler    │
│ Генерирует   │ │ Анализирует  │ │ Управляет    │ │ Исполняет    │
│ MarketEvent  │ │ и создаёт    │ │ позициями    │ │ ордера       │
│              │ │ SignalEvent  │ │              │ │              │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
```

## Определение типов событий

```rust
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Направление торговой операции
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Long,   // Покупка
    Short,  // Продажа (шорт)
}

/// Типы событий в системе
#[derive(Debug, Clone)]
pub enum Event {
    /// Новые рыночные данные (свеча, тик)
    Market(MarketEvent),
    /// Сигнал от стратегии
    Signal(SignalEvent),
    /// Ордер на исполнение
    Order(OrderEvent),
    /// Исполненная сделка
    Fill(FillEvent),
}

/// Рыночные данные (OHLCV свеча)
#[derive(Debug, Clone)]
pub struct MarketEvent {
    pub timestamp: u64,
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Сигнал от торговой стратегии
#[derive(Debug, Clone)]
pub struct SignalEvent {
    pub timestamp: u64,
    pub symbol: String,
    pub direction: Direction,
    pub strength: f64,  // Сила сигнала от 0.0 до 1.0
}

/// Ордер на покупку/продажу
#[derive(Debug, Clone)]
pub struct OrderEvent {
    pub timestamp: u64,
    pub symbol: String,
    pub direction: Direction,
    pub quantity: f64,
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderType {
    Market,
    Limit(f64),  // Лимитная цена
}

/// Исполненная сделка
#[derive(Debug, Clone)]
pub struct FillEvent {
    pub timestamp: u64,
    pub symbol: String,
    pub direction: Direction,
    pub quantity: f64,
    pub fill_price: f64,
    pub commission: f64,
}

impl MarketEvent {
    pub fn new(symbol: &str, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        MarketEvent {
            timestamp,
            symbol: symbol.to_string(),
            open,
            high,
            low,
            close,
            volume,
        }
    }
}
```

## Очередь событий

Центральный элемент архитектуры — очередь событий:

```rust
/// Очередь событий — сердце системы
pub struct EventQueue {
    events: VecDeque<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            events: VecDeque::new(),
        }
    }

    /// Добавить событие в очередь
    pub fn push(&mut self, event: Event) {
        self.events.push_back(event);
    }

    /// Получить следующее событие
    pub fn pop(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    /// Проверить, есть ли события
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Количество событий в очереди
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}
```

## Data Handler: Источник рыночных данных

```rust
/// Источник исторических данных для бэктеста
pub struct DataHandler {
    symbol: String,
    data: Vec<MarketEvent>,
    current_index: usize,
}

impl DataHandler {
    pub fn new(symbol: &str, data: Vec<MarketEvent>) -> Self {
        DataHandler {
            symbol: symbol.to_string(),
            data,
            current_index: 0,
        }
    }

    /// Создать тестовые данные
    pub fn with_sample_data(symbol: &str) -> Self {
        let mut data = Vec::new();
        let base_price = 100.0;

        // Генерируем 100 свечей с трендом
        for i in 0..100 {
            let trend = (i as f64 * 0.1).sin() * 10.0;
            let noise = (i as f64 * 0.5).cos() * 2.0;
            let price = base_price + trend + noise;

            let event = MarketEvent {
                timestamp: 1000000 + i as u64 * 3600,
                symbol: symbol.to_string(),
                open: price - 0.5,
                high: price + 1.0,
                low: price - 1.0,
                close: price + 0.3,
                volume: 1000.0 + (i as f64 * 10.0),
            };
            data.push(event);
        }

        DataHandler::new(symbol, data)
    }

    /// Получить следующую свечу (генерирует MarketEvent)
    pub fn get_next_bar(&mut self, queue: &mut EventQueue) -> bool {
        if self.current_index >= self.data.len() {
            return false; // Данные закончились
        }

        let bar = self.data[self.current_index].clone();
        queue.push(Event::Market(bar));
        self.current_index += 1;
        true
    }

    /// Получить текущую цену закрытия
    pub fn get_latest_price(&self) -> Option<f64> {
        if self.current_index > 0 {
            Some(self.data[self.current_index - 1].close)
        } else {
            None
        }
    }

    /// Сбросить позицию для нового бэктеста
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}
```

## Торговая стратегия

```rust
/// Трейт для торговых стратегий
pub trait Strategy {
    fn calculate_signals(&mut self, event: &MarketEvent, queue: &mut EventQueue);
}

/// Простая стратегия на скользящих средних
pub struct MovingAverageCrossStrategy {
    symbol: String,
    short_window: usize,
    long_window: usize,
    prices: Vec<f64>,
    in_position: bool,
}

impl MovingAverageCrossStrategy {
    pub fn new(symbol: &str, short_window: usize, long_window: usize) -> Self {
        MovingAverageCrossStrategy {
            symbol: symbol.to_string(),
            short_window,
            long_window,
            prices: Vec::new(),
            in_position: false,
        }
    }

    fn calculate_sma(&self, window: usize) -> Option<f64> {
        if self.prices.len() < window {
            return None;
        }

        let sum: f64 = self.prices.iter().rev().take(window).sum();
        Some(sum / window as f64)
    }
}

impl Strategy for MovingAverageCrossStrategy {
    fn calculate_signals(&mut self, event: &MarketEvent, queue: &mut EventQueue) {
        // Добавляем новую цену
        self.prices.push(event.close);

        // Ждём достаточно данных
        if self.prices.len() < self.long_window {
            return;
        }

        // Вычисляем скользящие средние
        let short_sma = self.calculate_sma(self.short_window).unwrap();
        let long_sma = self.calculate_sma(self.long_window).unwrap();

        // Генерируем сигналы
        if short_sma > long_sma && !self.in_position {
            // Короткая MA пересекла длинную снизу вверх — покупаем
            let signal = SignalEvent {
                timestamp: event.timestamp,
                symbol: self.symbol.clone(),
                direction: Direction::Long,
                strength: (short_sma - long_sma) / long_sma, // Относительная сила
            };
            queue.push(Event::Signal(signal));
            self.in_position = true;
            println!("📈 Сигнал на покупку: SMA{}={:.2} > SMA{}={:.2}",
                     self.short_window, short_sma,
                     self.long_window, long_sma);
        } else if short_sma < long_sma && self.in_position {
            // Короткая MA пересекла длинную сверху вниз — продаём
            let signal = SignalEvent {
                timestamp: event.timestamp,
                symbol: self.symbol.clone(),
                direction: Direction::Short,
                strength: (long_sma - short_sma) / long_sma,
            };
            queue.push(Event::Signal(signal));
            self.in_position = false;
            println!("📉 Сигнал на продажу: SMA{}={:.2} < SMA{}={:.2}",
                     self.short_window, short_sma,
                     self.long_window, long_sma);
        }
    }
}
```

## Портфель: Управление позициями и риском

```rust
use std::collections::HashMap;

/// Позиция по инструменту
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

impl Position {
    pub fn new(symbol: &str) -> Self {
        Position {
            symbol: symbol.to_string(),
            quantity: 0.0,
            avg_price: 0.0,
            current_price: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
        }
    }

    pub fn update_price(&mut self, price: f64) {
        self.current_price = price;
        if self.quantity != 0.0 {
            self.unrealized_pnl = (price - self.avg_price) * self.quantity;
        }
    }
}

/// Портфель трейдера
pub struct Portfolio {
    initial_capital: f64,
    cash: f64,
    positions: HashMap<String, Position>,
    total_commission: f64,
    trade_count: u32,
    equity_curve: Vec<f64>,
}

impl Portfolio {
    pub fn new(initial_capital: f64) -> Self {
        Portfolio {
            initial_capital,
            cash: initial_capital,
            positions: HashMap::new(),
            total_commission: 0.0,
            trade_count: 0,
            equity_curve: vec![initial_capital],
        }
    }

    /// Обработка сигнала — создание ордера
    pub fn handle_signal(&mut self, signal: &SignalEvent, queue: &mut EventQueue, current_price: f64) {
        // Простой риск-менеджмент: инвестируем 10% от капитала
        let position_size = self.cash * 0.1;
        let quantity = position_size / current_price;

        let order = OrderEvent {
            timestamp: signal.timestamp,
            symbol: signal.symbol.clone(),
            direction: signal.direction,
            quantity,
            order_type: OrderType::Market,
        };

        queue.push(Event::Order(order));
    }

    /// Обработка исполненной сделки
    pub fn handle_fill(&mut self, fill: &FillEvent) {
        let position = self.positions
            .entry(fill.symbol.clone())
            .or_insert_with(|| Position::new(&fill.symbol));

        match fill.direction {
            Direction::Long => {
                // Покупка
                let total_cost = fill.fill_price * fill.quantity + fill.commission;

                if position.quantity > 0.0 {
                    // Усреднение позиции
                    let total_qty = position.quantity + fill.quantity;
                    position.avg_price =
                        (position.avg_price * position.quantity + fill.fill_price * fill.quantity)
                        / total_qty;
                    position.quantity = total_qty;
                } else if position.quantity < 0.0 {
                    // Закрытие шорта
                    let pnl = (position.avg_price - fill.fill_price) * fill.quantity.min(-position.quantity);
                    position.realized_pnl += pnl;
                    position.quantity += fill.quantity;
                    if position.quantity > 0.0 {
                        position.avg_price = fill.fill_price;
                    }
                } else {
                    position.avg_price = fill.fill_price;
                    position.quantity = fill.quantity;
                }

                self.cash -= total_cost;
            }
            Direction::Short => {
                // Продажа
                let revenue = fill.fill_price * fill.quantity - fill.commission;

                if position.quantity > 0.0 {
                    // Закрытие лонга
                    let pnl = (fill.fill_price - position.avg_price) * fill.quantity.min(position.quantity);
                    position.realized_pnl += pnl;
                    position.quantity -= fill.quantity;
                } else {
                    // Открытие или увеличение шорта
                    position.avg_price = fill.fill_price;
                    position.quantity -= fill.quantity;
                }

                self.cash += revenue;
            }
        }

        self.total_commission += fill.commission;
        self.trade_count += 1;

        // Обновляем equity curve
        self.update_equity(fill.fill_price);
    }

    /// Обновление equity при изменении цены
    pub fn update_market_value(&mut self, symbol: &str, price: f64) {
        if let Some(position) = self.positions.get_mut(symbol) {
            position.update_price(price);
        }
        self.update_equity(price);
    }

    fn update_equity(&mut self, _current_price: f64) {
        let positions_value: f64 = self.positions.values()
            .map(|p| p.quantity * p.current_price)
            .sum();
        let total_equity = self.cash + positions_value;
        self.equity_curve.push(total_equity);
    }

    /// Получить итоговую статистику
    pub fn get_stats(&self) -> PortfolioStats {
        let final_equity = self.equity_curve.last().copied().unwrap_or(self.initial_capital);
        let total_return = (final_equity - self.initial_capital) / self.initial_capital * 100.0;

        // Расчёт максимальной просадки
        let mut max_equity = self.initial_capital;
        let mut max_drawdown = 0.0;
        for &equity in &self.equity_curve {
            if equity > max_equity {
                max_equity = equity;
            }
            let drawdown = (max_equity - equity) / max_equity * 100.0;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        let realized_pnl: f64 = self.positions.values()
            .map(|p| p.realized_pnl)
            .sum();

        PortfolioStats {
            initial_capital: self.initial_capital,
            final_equity,
            total_return,
            max_drawdown,
            total_trades: self.trade_count,
            total_commission: self.total_commission,
            realized_pnl,
        }
    }
}

#[derive(Debug)]
pub struct PortfolioStats {
    pub initial_capital: f64,
    pub final_equity: f64,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub total_trades: u32,
    pub total_commission: f64,
    pub realized_pnl: f64,
}

impl std::fmt::Display for PortfolioStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "\n╔══════════════════════════════════════╗\n\
             ║       РЕЗУЛЬТАТЫ БЭКТЕСТА            ║\n\
             ╠══════════════════════════════════════╣\n\
             ║ Начальный капитал: {:>15.2}  ║\n\
             ║ Итоговый капитал:  {:>15.2}  ║\n\
             ║ Доходность:        {:>14.2}%  ║\n\
             ║ Макс. просадка:    {:>14.2}%  ║\n\
             ║ Количество сделок: {:>15}  ║\n\
             ║ Комиссии:          {:>15.2}  ║\n\
             ║ Реализованный P&L: {:>15.2}  ║\n\
             ╚══════════════════════════════════════╝",
            self.initial_capital,
            self.final_equity,
            self.total_return,
            self.max_drawdown,
            self.total_trades,
            self.total_commission,
            self.realized_pnl
        )
    }
}
```

## Execution Handler: Исполнение ордеров

```rust
/// Симулятор исполнения ордеров
pub struct ExecutionHandler {
    commission_rate: f64,  // Процент комиссии
    slippage: f64,         // Проскальзывание
}

impl ExecutionHandler {
    pub fn new(commission_rate: f64, slippage: f64) -> Self {
        ExecutionHandler {
            commission_rate,
            slippage,
        }
    }

    /// Симуляция исполнения ордера
    pub fn execute_order(&self, order: &OrderEvent, current_price: f64, queue: &mut EventQueue) {
        // Добавляем проскальзывание
        let fill_price = match order.direction {
            Direction::Long => current_price * (1.0 + self.slippage),
            Direction::Short => current_price * (1.0 - self.slippage),
        };

        // Рассчитываем комиссию
        let commission = fill_price * order.quantity * self.commission_rate;

        let fill = FillEvent {
            timestamp: order.timestamp,
            symbol: order.symbol.clone(),
            direction: order.direction,
            quantity: order.quantity,
            fill_price,
            commission,
        };

        println!("💰 Исполнен ордер: {:?} {} @ {:.2} (комиссия: {:.2})",
                 order.direction, order.symbol, fill_price, commission);

        queue.push(Event::Fill(fill));
    }
}
```

## Главный цикл бэктестера

```rust
/// Движок бэктестинга
pub struct Backtester {
    event_queue: EventQueue,
    data_handler: DataHandler,
    strategy: Box<dyn Strategy>,
    portfolio: Portfolio,
    execution_handler: ExecutionHandler,
}

impl Backtester {
    pub fn new(
        data_handler: DataHandler,
        strategy: Box<dyn Strategy>,
        initial_capital: f64,
        commission_rate: f64,
        slippage: f64,
    ) -> Self {
        Backtester {
            event_queue: EventQueue::new(),
            data_handler,
            strategy,
            portfolio: Portfolio::new(initial_capital),
            execution_handler: ExecutionHandler::new(commission_rate, slippage),
        }
    }

    /// Запустить бэктест
    pub fn run(&mut self) {
        println!("🚀 Запуск бэктеста...\n");

        // Основной цикл бэктестинга
        loop {
            // 1. Получаем новые рыночные данные
            if !self.data_handler.get_next_bar(&mut self.event_queue) {
                // Данные закончились
                break;
            }

            // 2. Обрабатываем все события в очереди
            while let Some(event) = self.event_queue.pop() {
                match event {
                    Event::Market(ref market_event) => {
                        // Обновляем рыночную стоимость портфеля
                        self.portfolio.update_market_value(
                            &market_event.symbol,
                            market_event.close
                        );

                        // Стратегия анализирует данные
                        self.strategy.calculate_signals(
                            market_event,
                            &mut self.event_queue
                        );
                    }
                    Event::Signal(ref signal_event) => {
                        // Портфель обрабатывает сигнал
                        if let Some(price) = self.data_handler.get_latest_price() {
                            self.portfolio.handle_signal(
                                signal_event,
                                &mut self.event_queue,
                                price
                            );
                        }
                    }
                    Event::Order(ref order_event) => {
                        // Исполняем ордер
                        if let Some(price) = self.data_handler.get_latest_price() {
                            self.execution_handler.execute_order(
                                order_event,
                                price,
                                &mut self.event_queue
                            );
                        }
                    }
                    Event::Fill(ref fill_event) => {
                        // Обновляем портфель
                        self.portfolio.handle_fill(fill_event);
                    }
                }
            }
        }

        println!("\n✅ Бэктест завершён!");
    }

    /// Получить результаты
    pub fn get_results(&self) -> PortfolioStats {
        self.portfolio.get_stats()
    }
}

fn main() {
    // Создаём компоненты
    let data_handler = DataHandler::with_sample_data("BTC/USDT");
    let strategy = Box::new(MovingAverageCrossStrategy::new("BTC/USDT", 5, 20));

    // Создаём бэктестер
    let mut backtester = Backtester::new(
        data_handler,
        strategy,
        100_000.0,  // Начальный капитал
        0.001,      // Комиссия 0.1%
        0.0005,     // Проскальзывание 0.05%
    );

    // Запускаем бэктест
    backtester.run();

    // Выводим результаты
    let stats = backtester.get_results();
    println!("{}", stats);
}
```

## Расширенный пример: Стратегия RSI

```rust
/// Стратегия на основе RSI (Relative Strength Index)
pub struct RSIStrategy {
    symbol: String,
    period: usize,
    overbought: f64,
    oversold: f64,
    prices: Vec<f64>,
    in_position: bool,
}

impl RSIStrategy {
    pub fn new(symbol: &str, period: usize, overbought: f64, oversold: f64) -> Self {
        RSIStrategy {
            symbol: symbol.to_string(),
            period,
            overbought,
            oversold,
            prices: Vec::new(),
            in_position: false,
        }
    }

    fn calculate_rsi(&self) -> Option<f64> {
        if self.prices.len() < self.period + 1 {
            return None;
        }

        let changes: Vec<f64> = self.prices
            .windows(2)
            .map(|w| w[1] - w[0])
            .collect();

        let recent_changes: Vec<f64> = changes
            .iter()
            .rev()
            .take(self.period)
            .copied()
            .collect();

        let gains: f64 = recent_changes.iter()
            .filter(|&&x| x > 0.0)
            .sum();
        let losses: f64 = recent_changes.iter()
            .filter(|&&x| x < 0.0)
            .map(|x| x.abs())
            .sum();

        let avg_gain = gains / self.period as f64;
        let avg_loss = losses / self.period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Some(rsi)
    }
}

impl Strategy for RSIStrategy {
    fn calculate_signals(&mut self, event: &MarketEvent, queue: &mut EventQueue) {
        self.prices.push(event.close);

        if let Some(rsi) = self.calculate_rsi() {
            // RSI ниже oversold — сигнал на покупку
            if rsi < self.oversold && !self.in_position {
                let signal = SignalEvent {
                    timestamp: event.timestamp,
                    symbol: self.symbol.clone(),
                    direction: Direction::Long,
                    strength: (self.oversold - rsi) / self.oversold,
                };
                queue.push(Event::Signal(signal));
                self.in_position = true;
                println!("📈 RSI сигнал на покупку: RSI = {:.2} (перепродано)", rsi);
            }
            // RSI выше overbought — сигнал на продажу
            else if rsi > self.overbought && self.in_position {
                let signal = SignalEvent {
                    timestamp: event.timestamp,
                    symbol: self.symbol.clone(),
                    direction: Direction::Short,
                    strength: (rsi - self.overbought) / (100.0 - self.overbought),
                };
                queue.push(Event::Signal(signal));
                self.in_position = false;
                println!("📉 RSI сигнал на продажу: RSI = {:.2} (перекуплено)", rsi);
            }
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Event-Driven Architecture | Система реагирует на события, а не выполняет код последовательно |
| EventQueue | Центральная очередь для передачи событий между компонентами |
| MarketEvent | Событие с рыночными данными (OHLCV) |
| SignalEvent | Сигнал от торговой стратегии |
| OrderEvent | Ордер на исполнение |
| FillEvent | Исполненная сделка |
| DataHandler | Источник исторических данных |
| Strategy trait | Интерфейс для торговых стратегий |
| Portfolio | Управление позициями, балансом и риском |
| ExecutionHandler | Симуляция исполнения ордеров |

## Домашнее задание

### Упражнение 1: Стратегия Bollinger Bands

Реализуй стратегию на основе полос Боллинджера:
- Покупка, когда цена касается нижней полосы
- Продажа, когда цена касается верхней полосы

```rust
pub struct BollingerBandsStrategy {
    symbol: String,
    period: usize,
    num_std: f64,
    prices: Vec<f64>,
    in_position: bool,
}

// Реализуй методы:
// - calculate_bands(&self) -> Option<(f64, f64, f64)>  // (upper, middle, lower)
// - Strategy trait
```

### Упражнение 2: Stop-Loss и Take-Profit

Добавь в `Portfolio` поддержку стоп-лосса и тейк-профита:
- При открытии позиции устанавливай уровни SL/TP
- Автоматически закрывай позицию при достижении уровней

### Упражнение 3: Множество инструментов

Расшири `DataHandler` для работы с несколькими торговыми парами одновременно. Реализуй стратегию парного трейдинга.

### Упражнение 4: Оптимизация параметров

Создай функцию для оптимизации параметров стратегии:

```rust
fn optimize_strategy(
    data: &[MarketEvent],
    param_ranges: &[(f64, f64, f64)], // (min, max, step)
) -> (Vec<f64>, PortfolioStats) {
    // Перебери все комбинации параметров
    // Верни лучшие параметры и результаты
}
```

## Навигация

[← Предыдущий день](../275-backtesting-fundamentals/ru.md) | [Следующий день →](../277-backtester-data-handling/ru.md)
