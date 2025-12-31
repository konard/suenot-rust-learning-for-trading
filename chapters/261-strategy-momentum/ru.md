# День 261: Стратегия: Momentum (Импульс)

## Аналогия из трейдинга

Представь, что ты наблюдаешь за оживлённым рынком. У некоторых прилавков растёт очередь покупателей — прошёл слух, что там продают что-то хорошее. У других прилавков покупатели стремительно уходят. Как опытный трейдер, ты замечаешь эти **тренды** и действуешь соответственно: инвестируешь в то, что набирает популярность, и избегаешь (или шортишь) то, что теряет спрос.

В этом и заключается суть **momentum-трейдинга** — стратегии, основанной на принципе, что активы, которые росли, склонны продолжать расти, а активы, которые падали, склонны продолжать падение. Это похоже на сёрфинг: ты ловишь волну, которая уже движется, и едешь в том же направлении.

На финансовых рынках momentum работает потому, что:
- **Устойчивость тренда**: Рыночные тренды часто продолжаются благодаря психологии инвесторов
- **Стадное поведение**: Трейдеры следуют за другими трейдерами, усиливая движение цен
- **Распространение информации**: Новости распространяются постепенно, создавая устойчивые движения
- **Институциональные потоки**: Крупным фондам нужно время, чтобы набрать или закрыть позиции

## Что такое Momentum?

Momentum (импульс) — это скорость изменения цены за определённый период. Он измеряет, как быстро и в каком направлении движется цена актива.

Ключевые концепции momentum:
- **Абсолютный momentum**: Текущая цена относительно прошлой цены
- **Относительный momentum**: Сравнение производительности нескольких активов
- **Rate of Change (ROC)**: Процентное изменение за N периодов
- **Скользящая средняя**: Сглаженная цена для определения направления тренда

## Базовый расчёт Momentum

```rust
/// Вычисляет индикатор Rate of Change (ROC)
fn calculate_roc(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() <= period {
        return vec![];
    }

    prices
        .iter()
        .skip(period)
        .zip(prices.iter())
        .map(|(current, past)| ((current - past) / past) * 100.0)
        .collect()
}

/// Вычисляет простую скользящую среднюю (SMA)
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    prices
        .windows(period)
        .map(|window| window.iter().sum::<f64>() / period as f64)
        .collect()
}

fn main() {
    // Дневные цены закрытия BTC (симуляция)
    let btc_prices = vec![
        42000.0, 42500.0, 43200.0, 43800.0, 44500.0,
        45200.0, 44800.0, 45500.0, 46200.0, 47000.0,
        47800.0, 48500.0, 48200.0, 49000.0, 50000.0,
    ];

    println!("=== Анализ Momentum для BTC ===\n");

    // Вычисляем 5-периодный ROC
    let roc_5 = calculate_roc(&btc_prices, 5);
    println!("Значения 5-периодного ROC:");
    for (i, roc) in roc_5.iter().enumerate() {
        let signal = if *roc > 5.0 {
            "СИЛЬНАЯ ПОКУПКА"
        } else if *roc > 0.0 {
            "ПОКУПКА"
        } else if *roc > -5.0 {
            "ПРОДАЖА"
        } else {
            "СИЛЬНАЯ ПРОДАЖА"
        };
        println!("  День {}: ROC = {:.2}% -> {}", i + 6, roc, signal);
    }

    // Вычисляем 5-периодную SMA
    let sma_5 = calculate_sma(&btc_prices, 5);
    println!("\nЗначения 5-периодной SMA:");
    for (i, sma) in sma_5.iter().enumerate() {
        let current_price = btc_prices[i + 4];
        let trend = if current_price > *sma { "ВЫШЕ (Бычий)" } else { "НИЖЕ (Медвежий)" };
        println!("  День {}: Цена ${:.0} vs SMA ${:.0} -> {}", i + 5, current_price, sma, trend);
    }
}
```

## Структура Momentum-стратегии

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Signal {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Position {
    Long,
    Short,
    Flat,
}

#[derive(Debug)]
struct MomentumStrategy {
    lookback_period: usize,
    entry_threshold: f64,
    exit_threshold: f64,
    price_history: VecDeque<f64>,
    position: Position,
    entry_price: Option<f64>,
}

impl MomentumStrategy {
    fn new(lookback_period: usize, entry_threshold: f64, exit_threshold: f64) -> Self {
        MomentumStrategy {
            lookback_period,
            entry_threshold,
            exit_threshold,
            price_history: VecDeque::with_capacity(lookback_period + 1),
            position: Position::Flat,
            entry_price: None,
        }
    }

    fn update(&mut self, price: f64) -> Signal {
        self.price_history.push_back(price);

        // Поддерживаем только необходимую историю
        while self.price_history.len() > self.lookback_period + 1 {
            self.price_history.pop_front();
        }

        // Нужно достаточно данных для расчёта
        if self.price_history.len() <= self.lookback_period {
            return Signal::Hold;
        }

        let momentum = self.calculate_momentum();
        self.generate_signal(momentum, price)
    }

    fn calculate_momentum(&self) -> f64 {
        let current = *self.price_history.back().unwrap();
        let past = *self.price_history.front().unwrap();
        ((current - past) / past) * 100.0
    }

    fn generate_signal(&mut self, momentum: f64, current_price: f64) -> Signal {
        match self.position {
            Position::Flat => {
                if momentum > self.entry_threshold {
                    self.position = Position::Long;
                    self.entry_price = Some(current_price);
                    if momentum > self.entry_threshold * 2.0 {
                        Signal::StrongBuy
                    } else {
                        Signal::Buy
                    }
                } else if momentum < -self.entry_threshold {
                    self.position = Position::Short;
                    self.entry_price = Some(current_price);
                    if momentum < -self.entry_threshold * 2.0 {
                        Signal::StrongSell
                    } else {
                        Signal::Sell
                    }
                } else {
                    Signal::Hold
                }
            }
            Position::Long => {
                if momentum < self.exit_threshold {
                    self.position = Position::Flat;
                    self.entry_price = None;
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            Position::Short => {
                if momentum > -self.exit_threshold {
                    self.position = Position::Flat;
                    self.entry_price = None;
                    Signal::Buy
                } else {
                    Signal::Hold
                }
            }
        }
    }

    fn get_position(&self) -> Position {
        self.position
    }

    fn get_unrealized_pnl(&self, current_price: f64) -> Option<f64> {
        self.entry_price.map(|entry| {
            match self.position {
                Position::Long => ((current_price - entry) / entry) * 100.0,
                Position::Short => ((entry - current_price) / entry) * 100.0,
                Position::Flat => 0.0,
            }
        })
    }
}

fn main() {
    let mut strategy = MomentumStrategy::new(10, 5.0, 0.0);

    // Симулированные данные с трендовыми периодами
    let prices = vec![
        100.0, 101.0, 102.5, 104.0, 106.0, 108.5, 111.0, 114.0, 117.0, 120.0,  // Восходящий тренд
        121.0, 122.0, 121.5, 120.0, 118.0, 115.0, 112.0, 110.0, 108.0, 105.0,  // Нисходящий тренд
        104.0, 105.0, 107.0, 110.0, 113.0, 116.0, 120.0, 124.0, 128.0, 132.0,  // Новый восходящий
    ];

    println!("=== Бэктест Momentum-стратегии ===\n");
    println!("{:>4} {:>10} {:>12} {:>10} {:>12}", "День", "Цена", "Сигнал", "Позиция", "PnL %");
    println!("{}", "-".repeat(52));

    for (day, &price) in prices.iter().enumerate() {
        let signal = strategy.update(price);
        let position = strategy.get_position();
        let pnl = strategy.get_unrealized_pnl(price)
            .map(|p| format!("{:+.2}%", p))
            .unwrap_or_else(|| "N/A".to_string());

        println!("{:>4} {:>10.2} {:>12?} {:>10?} {:>12}",
            day + 1, price, signal, position, pnl);
    }
}
```

## Относительный Momentum: Сравнение активов

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Asset {
    symbol: String,
    prices: Vec<f64>,
}

impl Asset {
    fn new(symbol: &str, prices: Vec<f64>) -> Self {
        Asset {
            symbol: symbol.to_string(),
            prices,
        }
    }

    fn momentum(&self, period: usize) -> Option<f64> {
        if self.prices.len() <= period {
            return None;
        }

        let current = *self.prices.last()?;
        let past = self.prices[self.prices.len() - 1 - period];
        Some(((current - past) / past) * 100.0)
    }

    fn latest_price(&self) -> Option<f64> {
        self.prices.last().copied()
    }
}

#[derive(Debug)]
struct RelativeMomentumRanker {
    assets: Vec<Asset>,
    lookback_period: usize,
    top_n: usize,
}

impl RelativeMomentumRanker {
    fn new(lookback_period: usize, top_n: usize) -> Self {
        RelativeMomentumRanker {
            assets: Vec::new(),
            lookback_period,
            top_n,
        }
    }

    fn add_asset(&mut self, asset: Asset) {
        self.assets.push(asset);
    }

    fn rank_assets(&self) -> Vec<(&Asset, f64)> {
        let mut rankings: Vec<_> = self.assets
            .iter()
            .filter_map(|asset| {
                asset.momentum(self.lookback_period)
                    .map(|mom| (asset, mom))
            })
            .collect();

        // Сортируем по momentum по убыванию
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings
    }

    fn get_top_momentum(&self) -> Vec<(&Asset, f64)> {
        self.rank_assets()
            .into_iter()
            .take(self.top_n)
            .collect()
    }

    fn get_bottom_momentum(&self) -> Vec<(&Asset, f64)> {
        let mut rankings = self.rank_assets();
        rankings.reverse();
        rankings.into_iter().take(self.top_n).collect()
    }
}

fn main() {
    let mut ranker = RelativeMomentumRanker::new(5, 3);

    // Добавляем различные криптоактивы
    ranker.add_asset(Asset::new("BTC", vec![
        40000.0, 41000.0, 42500.0, 44000.0, 46000.0, 48000.0
    ]));

    ranker.add_asset(Asset::new("ETH", vec![
        2200.0, 2300.0, 2250.0, 2400.0, 2550.0, 2700.0
    ]));

    ranker.add_asset(Asset::new("SOL", vec![
        80.0, 85.0, 95.0, 110.0, 130.0, 150.0
    ]));

    ranker.add_asset(Asset::new("ADA", vec![
        0.50, 0.48, 0.45, 0.42, 0.40, 0.38
    ]));

    ranker.add_asset(Asset::new("DOGE", vec![
        0.08, 0.085, 0.082, 0.079, 0.075, 0.072
    ]));

    println!("=== Рейтинг относительного Momentum ===\n");

    println!("Все активы по 5-дневному momentum:");
    for (rank, (asset, momentum)) in ranker.rank_assets().iter().enumerate() {
        let direction = if *momentum > 0.0 { "+" } else { "" };
        println!("  {}. {} -> {}{:.2}%", rank + 1, asset.symbol, direction, momentum);
    }

    println!("\n--- Выбор портфеля ---");

    println!("\nКандидаты на ЛОНГ (топ momentum):");
    for (asset, momentum) in ranker.get_top_momentum() {
        println!("  ПОКУПАТЬ {} (momentum: +{:.2}%)", asset.symbol, momentum);
    }

    println!("\nКандидаты на ШОРТ (худший momentum):");
    for (asset, momentum) in ranker.get_bottom_momentum() {
        println!("  ПРОДАВАТЬ {} (momentum: {:.2}%)", asset.symbol, momentum);
    }
}
```

## Двойной Momentum (Dual Momentum)

Двойной momentum сочетает абсолютный и относительный momentum для лучшей доходности с учётом риска:

```rust
#[derive(Debug, Clone)]
struct DualMomentumStrategy {
    assets: Vec<String>,
    risk_free_rate: f64,  // Годовая ставка
    lookback_months: usize,
}

#[derive(Debug)]
struct MonthlyReturns {
    symbol: String,
    returns: Vec<f64>,
}

impl MonthlyReturns {
    fn new(symbol: &str, returns: Vec<f64>) -> Self {
        MonthlyReturns {
            symbol: symbol.to_string(),
            returns,
        }
    }

    fn cumulative_return(&self, months: usize) -> f64 {
        if self.returns.len() < months {
            return 0.0;
        }

        let start_idx = self.returns.len() - months;
        self.returns[start_idx..]
            .iter()
            .fold(1.0, |acc, r| acc * (1.0 + r / 100.0)) - 1.0
    }
}

impl DualMomentumStrategy {
    fn new(assets: Vec<String>, risk_free_rate: f64, lookback_months: usize) -> Self {
        DualMomentumStrategy {
            assets,
            risk_free_rate,
            lookback_months,
        }
    }

    fn select_asset(&self, asset_returns: &[MonthlyReturns]) -> Option<String> {
        // Шаг 1: Находим актив с лучшим относительным momentum
        let mut best_asset: Option<(&MonthlyReturns, f64)> = None;

        for returns in asset_returns {
            if self.assets.contains(&returns.symbol) {
                let cum_return = returns.cumulative_return(self.lookback_months);

                match &best_asset {
                    None => best_asset = Some((returns, cum_return)),
                    Some((_, best_return)) if cum_return > *best_return => {
                        best_asset = Some((returns, cum_return));
                    }
                    _ => {}
                }
            }
        }

        // Шаг 2: Проверяем абсолютный momentum (vs безрисковая ставка)
        if let Some((asset, momentum)) = best_asset {
            // Конвертируем годовую безрисковую ставку в ставку за период
            let period_rf_rate = (1.0 + self.risk_free_rate / 100.0)
                .powf(self.lookback_months as f64 / 12.0) - 1.0;

            if momentum > period_rf_rate {
                return Some(asset.symbol.clone());
            }
        }

        // Если ни один актив не превышает безрисковую ставку — уходим в кэш
        None
    }
}

fn main() {
    // Создаём стратегию двойного momentum
    let strategy = DualMomentumStrategy::new(
        vec!["SPY".to_string(), "EFA".to_string(), "BTC".to_string()],
        4.0,  // 4% годовая безрисковая ставка
        12,   // 12-месячный период
    );

    // Симулированные 12-месячные доходности для каждого актива
    let asset_returns = vec![
        MonthlyReturns::new("SPY", vec![
            1.5, 2.0, -0.5, 1.0, 2.5, 1.8, -1.0, 0.5, 2.0, 1.5, 0.8, 1.2
        ]),
        MonthlyReturns::new("EFA", vec![
            0.8, 1.5, -1.0, 0.5, 1.8, 2.0, -0.5, 0.2, 1.5, 0.8, 0.5, 0.8
        ]),
        MonthlyReturns::new("BTC", vec![
            5.0, 8.0, -3.0, 10.0, 15.0, -5.0, 2.0, 7.0, 12.0, -2.0, 5.0, 8.0
        ]),
    ];

    println!("=== Стратегия двойного Momentum ===\n");

    // Показываем накопленную доходность
    println!("12-месячная накопленная доходность:");
    for returns in &asset_returns {
        let cum_return = returns.cumulative_return(12) * 100.0;
        println!("  {}: {:.2}%", returns.symbol, cum_return);
    }

    // Вычисляем порог безрисковой ставки
    let rf_threshold = ((1.0 + 4.0 / 100.0_f64).powf(1.0) - 1.0) * 100.0;
    println!("\nПорог безрисковой ставки: {:.2}%", rf_threshold);

    // Получаем рекомендацию
    match strategy.select_asset(&asset_returns) {
        Some(asset) => {
            println!("\nРЕКОМЕНДАЦИЯ: Инвестировать в {}", asset);
            println!("Обоснование: Лучший относительный momentum И превышает безрисковую ставку");
        }
        None => {
            println!("\nРЕКОМЕНДАЦИЯ: Оставаться в КЭШЕ");
            println!("Обоснование: Ни один актив не превышает безрисковую ставку");
        }
    }
}
```

## Momentum с управлением рисками

```rust
#[derive(Debug, Clone)]
struct RiskManagedMomentum {
    symbol: String,
    position_size: f64,
    entry_price: Option<f64>,
    stop_loss_pct: f64,
    take_profit_pct: f64,
    trailing_stop_pct: f64,
    highest_price: f64,
}

#[derive(Debug)]
enum TradeAction {
    Enter { price: f64, size: f64 },
    Exit { price: f64, reason: String, pnl: f64 },
    Hold,
    UpdateTrailingStop { new_stop: f64 },
}

impl RiskManagedMomentum {
    fn new(symbol: &str, stop_loss_pct: f64, take_profit_pct: f64, trailing_stop_pct: f64) -> Self {
        RiskManagedMomentum {
            symbol: symbol.to_string(),
            position_size: 0.0,
            entry_price: None,
            stop_loss_pct,
            take_profit_pct,
            trailing_stop_pct,
            highest_price: 0.0,
        }
    }

    fn enter_position(&mut self, price: f64, size: f64) -> TradeAction {
        self.entry_price = Some(price);
        self.position_size = size;
        self.highest_price = price;
        TradeAction::Enter { price, size }
    }

    fn update(&mut self, current_price: f64) -> TradeAction {
        let entry = match self.entry_price {
            Some(p) => p,
            None => return TradeAction::Hold,
        };

        let pnl_pct = ((current_price - entry) / entry) * 100.0;

        // Обновляем максимальную цену для trailing stop
        if current_price > self.highest_price {
            self.highest_price = current_price;
        }

        // Вычисляем уровни стопов
        let fixed_stop = entry * (1.0 - self.stop_loss_pct / 100.0);
        let trailing_stop = self.highest_price * (1.0 - self.trailing_stop_pct / 100.0);
        let effective_stop = fixed_stop.max(trailing_stop);

        let take_profit = entry * (1.0 + self.take_profit_pct / 100.0);

        // Проверяем условия выхода
        if current_price <= effective_stop {
            let reason = if current_price <= fixed_stop {
                "Сработал стоп-лосс".to_string()
            } else {
                "Сработал trailing stop".to_string()
            };
            self.close_position();
            return TradeAction::Exit {
                price: current_price,
                reason,
                pnl: pnl_pct,
            };
        }

        if current_price >= take_profit {
            self.close_position();
            return TradeAction::Exit {
                price: current_price,
                reason: "Сработал тейк-профит".to_string(),
                pnl: pnl_pct,
            };
        }

        // Проверяем, был ли обновлён trailing stop
        if trailing_stop > fixed_stop {
            TradeAction::UpdateTrailingStop { new_stop: trailing_stop }
        } else {
            TradeAction::Hold
        }
    }

    fn close_position(&mut self) {
        self.entry_price = None;
        self.position_size = 0.0;
        self.highest_price = 0.0;
    }
}

fn main() {
    let mut trader = RiskManagedMomentum::new(
        "BTC",
        5.0,   // 5% стоп-лосс
        15.0,  // 15% тейк-профит
        3.0,   // 3% trailing stop
    );

    println!("=== Momentum-трейдинг с управлением рисками ===\n");

    // Открываем позицию
    let entry_action = trader.enter_position(50000.0, 1.0);
    println!("Вход: {:?}\n", entry_action);

    // Симулируем движение цен
    let prices = vec![
        50500.0, 51000.0, 52000.0, 53500.0, 55000.0,  // Восходящий тренд
        54000.0, 53000.0, 52500.0,  // Откат
        54000.0, 56000.0, 57000.0, 57500.0,  // Продолжение роста
        56500.0, 55000.0, 54000.0,  // Падение — может сработать trailing stop
    ];

    println!("{:>6} {:>12} {:>12} {:>10}", "Шаг", "Цена", "P&L %", "Действие");
    println!("{}", "-".repeat(45));

    for (i, &price) in prices.iter().enumerate() {
        let entry_price = 50000.0;
        let pnl = ((price - entry_price) / entry_price) * 100.0;
        let action = trader.update(price);

        let action_str = match &action {
            TradeAction::Hold => "Держать".to_string(),
            TradeAction::Exit { reason, .. } => format!("ВЫХОД: {}", reason),
            TradeAction::UpdateTrailingStop { new_stop } => {
                format!("Trail: ${:.0}", new_stop)
            }
            _ => "".to_string(),
        };

        println!("{:>6} {:>12.0} {:>+11.2}% {:>10}",
            i + 1, price, pnl, action_str);

        if matches!(action, TradeAction::Exit { .. }) {
            break;
        }
    }
}
```

## Полная торговая система на Momentum

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct PriceBar {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Debug)]
struct MomentumTradingSystem {
    symbol: String,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    bars: Vec<PriceBar>,
    position: f64,
    cash: f64,
    trades: Vec<Trade>,
}

#[derive(Debug, Clone)]
struct Trade {
    entry_time: u64,
    exit_time: Option<u64>,
    entry_price: f64,
    exit_price: Option<f64>,
    size: f64,
    pnl: Option<f64>,
}

impl MomentumTradingSystem {
    fn new(symbol: &str, initial_cash: f64) -> Self {
        MomentumTradingSystem {
            symbol: symbol.to_string(),
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            bars: Vec::new(),
            position: 0.0,
            cash: initial_cash,
            trades: Vec::new(),
        }
    }

    fn add_bar(&mut self, bar: PriceBar) {
        self.bars.push(bar);
        self.evaluate_signals();
    }

    fn calculate_ema(&self, period: usize) -> Vec<f64> {
        let closes: Vec<f64> = self.bars.iter().map(|b| b.close).collect();

        if closes.len() < period {
            return vec![];
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = Vec::with_capacity(closes.len());

        // Первая EMA — это SMA
        let first_sma: f64 = closes[..period].iter().sum::<f64>() / period as f64;
        ema.push(first_sma);

        for i in period..closes.len() {
            let new_ema = (closes[i] - ema.last().unwrap()) * multiplier + ema.last().unwrap();
            ema.push(new_ema);
        }

        ema
    }

    fn calculate_macd(&self) -> Option<(f64, f64, f64)> {
        let fast_ema = self.calculate_ema(self.fast_period);
        let slow_ema = self.calculate_ema(self.slow_period);

        if fast_ema.is_empty() || slow_ema.is_empty() {
            return None;
        }

        // Выравниваем EMA
        let offset = self.slow_period - self.fast_period;
        if fast_ema.len() <= offset {
            return None;
        }

        let macd_line: Vec<f64> = fast_ema[offset..]
            .iter()
            .zip(slow_ema.iter())
            .map(|(f, s)| f - s)
            .collect();

        if macd_line.len() < self.signal_period {
            return None;
        }

        // Вычисляем сигнальную линию (EMA от MACD)
        let multiplier = 2.0 / (self.signal_period as f64 + 1.0);
        let first_signal: f64 = macd_line[..self.signal_period].iter().sum::<f64>()
            / self.signal_period as f64;

        let mut signal_line = first_signal;
        for &macd in &macd_line[self.signal_period..] {
            signal_line = (macd - signal_line) * multiplier + signal_line;
        }

        let current_macd = *macd_line.last().unwrap();
        let histogram = current_macd - signal_line;

        Some((current_macd, signal_line, histogram))
    }

    fn calculate_rsi(&self, period: usize) -> Option<f64> {
        if self.bars.len() < period + 1 {
            return None;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        let start = self.bars.len() - period - 1;
        for i in start..self.bars.len() - 1 {
            let change = self.bars[i + 1].close - self.bars[i].close;
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    fn evaluate_signals(&mut self) {
        let current_bar = match self.bars.last() {
            Some(b) => b.clone(),
            None => return,
        };

        let macd = match self.calculate_macd() {
            Some(m) => m,
            None => return,
        };

        let rsi = match self.calculate_rsi(14) {
            Some(r) => r,
            None => return,
        };

        let (macd_line, signal_line, histogram) = macd;

        // Торговая логика
        if self.position == 0.0 {
            // Условия входа
            let macd_bullish = histogram > 0.0 && macd_line > signal_line;
            let rsi_not_overbought = rsi < 70.0;
            let rsi_recovering = rsi > 30.0;

            if macd_bullish && rsi_not_overbought && rsi_recovering {
                // Рассчитываем размер позиции (используем 95% кэша)
                let size = (self.cash * 0.95) / current_bar.close;
                self.position = size;
                self.cash -= size * current_bar.close;

                self.trades.push(Trade {
                    entry_time: current_bar.timestamp,
                    exit_time: None,
                    entry_price: current_bar.close,
                    exit_price: None,
                    size,
                    pnl: None,
                });

                println!("ПОКУПКА: {} единиц @ ${:.2} | RSI: {:.1} | MACD Hist: {:.4}",
                    size, current_bar.close, rsi, histogram);
            }
        } else {
            // Условия выхода
            let macd_bearish = histogram < 0.0;
            let rsi_overbought = rsi > 70.0;

            if macd_bearish || rsi_overbought {
                let exit_price = current_bar.close;
                let exit_value = self.position * exit_price;

                if let Some(trade) = self.trades.last_mut() {
                    let pnl = exit_value - (trade.size * trade.entry_price);
                    trade.exit_time = Some(current_bar.timestamp);
                    trade.exit_price = Some(exit_price);
                    trade.pnl = Some(pnl);

                    println!("ПРОДАЖА: {} единиц @ ${:.2} | PnL: ${:.2} | RSI: {:.1}",
                        self.position, exit_price, pnl, rsi);
                }

                self.cash += exit_value;
                self.position = 0.0;
            }
        }
    }

    fn get_performance(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        let total_pnl: f64 = self.trades
            .iter()
            .filter_map(|t| t.pnl)
            .sum();

        let winning_trades: Vec<_> = self.trades
            .iter()
            .filter(|t| t.pnl.map(|p| p > 0.0).unwrap_or(false))
            .collect();

        let total_trades = self.trades.iter().filter(|t| t.pnl.is_some()).count();
        let win_rate = if total_trades > 0 {
            winning_trades.len() as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        stats.insert("total_pnl".to_string(), total_pnl);
        stats.insert("total_trades".to_string(), total_trades as f64);
        stats.insert("win_rate".to_string(), win_rate);
        stats.insert("current_cash".to_string(), self.cash);
        stats.insert("position_value".to_string(),
            self.position * self.bars.last().map(|b| b.close).unwrap_or(0.0));

        stats
    }
}

fn main() {
    let mut system = MomentumTradingSystem::new("BTC", 100000.0);

    println!("=== Полная торговая система на Momentum ===\n");

    // Генерируем симулированные данные с трендами
    let mut price = 40000.0;
    for day in 0..100 {
        // Симулируем движение цены
        let trend = if day < 30 { 1.005 }
            else if day < 50 { 0.998 }
            else if day < 80 { 1.008 }
            else { 0.995 };

        let volatility = (rand_simple(day) - 0.5) * 0.02;
        price *= trend + volatility;

        let bar = PriceBar {
            timestamp: 1700000000 + day as u64 * 86400,
            open: price * 0.999,
            high: price * 1.01,
            low: price * 0.99,
            close: price,
            volume: 1000.0 + rand_simple(day) * 500.0,
        };

        system.add_bar(bar);
    }

    println!("\n=== Итоги производительности ===");
    for (key, value) in system.get_performance() {
        println!("{}: {:.2}", key, value);
    }
}

// Простой детерминированный "случайный" генератор для воспроизводимости
fn rand_simple(seed: u32) -> f64 {
    let x = seed.wrapping_mul(1103515245).wrapping_add(12345);
    (x as f64 / u32::MAX as f64).abs()
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Momentum | Скорость изменения цены за период времени |
| ROC (Rate of Change) | Процентное изменение между текущей и прошлой ценой |
| SMA/EMA | Скользящие средние для сглаживания цены и определения тренда |
| Относительный Momentum | Сравнение производительности нескольких активов |
| Двойной Momentum | Сочетание абсолютного и относительного momentum |
| MACD | Осциллятор momentum на основе пересечения EMA |
| RSI | Индекс относительной силы для перекупленности/перепроданности |
| Trailing Stop | Динамический стоп-лосс, следующий за ценой вверх |

## Практические упражнения

1. **Базовый калькулятор Momentum**: Напиши функцию, которая вычисляет 10-дневный и 20-дневный momentum для ценового ряда и возвращает сигнал на основе их пересечения.

2. **Ранжирование нескольких активов**: Создай программу, которая принимает ценовые данные для 5 криптовалют и ранжирует их по 7-дневному momentum, показывая топ-2 для потенциальных лонгов.

3. **Детектор разворота Momentum**: Реализуй функцию, которая определяет, когда momentum ослабевает (замедляется, даже если всё ещё положительный) как раннее предупреждение.

4. **Momentum, взвешенный по объёму**: Модифицируй базовый расчёт momentum, чтобы взвешивать изменения цены по объёму, придавая больше значения движениям с высоким объёмом.

## Домашнее задание

1. **Детектор дивергенции Momentum**: Реализуй систему, которая обнаруживает дивергенцию между ценой (новые максимумы) и momentum (более низкие максимумы). Это часто сигнализирует о развороте тренда.

2. **Стратегия ротации секторов**: Создай программу, которая:
   - Отслеживает momentum для 4 разных "секторов" (например, DeFi, L1, Мем-коины, Стейблкоины)
   - Ротирует капитал в сектор с наивысшим momentum
   - Включает режим "risk-off", когда все секторы имеют отрицательный momentum

3. **Адаптивный Momentum**: Построй momentum-стратегию, которая автоматически корректирует период lookback в зависимости от волатильности рынка (короче периоды при высокой волатильности, длиннее при низкой).

4. **Гибрид Momentum + Mean Reversion**: Спроектируй торговую систему, которая:
   - Использует momentum для определения основного направления тренда
   - Использует mean reversion для тайминга входа (покупка на откатах в восходящем тренде)
   - Включает правильный сайзинг позиций на основе волатильности

## Навигация

[← Предыдущий день](../260-strategy-market-making/ru.md) | [Следующий день →](../262-strategy-pairs-trading/ru.md)
