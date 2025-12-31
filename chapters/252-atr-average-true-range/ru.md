# День 252: ATR: Average True Range (Средний истинный диапазон)

## Аналогия из трейдинга

Представь, что ты трейдер, который пытается решить, где разместить стоп-лосс ордер. Если поставить его слишком близко к текущей цене, обычные рыночные колебания сработают его. Если поставить слишком далеко — рискуешь потерять слишком много на неудачной сделке. Как понять, что такое «нормальное» движение цены?

Здесь на помощь приходит **ATR (Average True Range — Средний истинный диапазон)**. Думай об ATR как о «термометре волатильности» рынка — он измеряет, насколько актив обычно движется за торговую сессию. Как ты одеваешься по-разному при 10°C и 30°C, так и стоп-лоссы нужно ставить на разном расстоянии для низковолатильного и высоковолатильного рынка.

В реальном трейдинге ATR используется для:
- **Размещения стоп-лоссов**: установка стопов на расстоянии 2-3 ATR от входа
- **Размера позиции**: торговля меньшими позициями при высокой волатильности
- **Подтверждения пробоев**: пробой с высоким ATR более значим
- **Сравнения волатильности**: BTC сейчас волатильнее ETH или нет?

## Что такое ATR?

ATR был разработан Дж. Уэллсом Уайлдером Младшим и представлен в его книге 1978 года «Новые концепции в технических торговых системах». Он измеряет волатильность рынка, вычисляя среднее значений **True Range (Истинный диапазон)** за указанный период.

### True Range (TR)

Истинный диапазон — это наибольшее из:
1. Текущий High минус текущий Low
2. Абсолютное значение (Текущий High минус предыдущий Close)
3. Абсолютное значение (Текущий Low минус предыдущий Close)

```
TR = max(High - Low, |High - Prev Close|, |Low - Prev Close|)
```

True Range учитывает гэпы — если акция закрылась на $100 и открылась на следующий день на $105, обычный диапазон (High - Low) не захватит этот гэп в $5, а True Range — захватит.

### Расчёт ATR

ATR обычно является 14-периодной скользящей средней значений True Range. Первый ATR — это простое среднее, затем последующие значения используют:

```
ATR = ((Предыдущий ATR × (n - 1)) + Текущий TR) / n
```

Где `n` — период (обычно 14).

## Базовая реализация ATR

```rust
/// Представляет одну свечу (OHLC данные)
#[derive(Debug, Clone)]
struct Candle {
    timestamp: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Калькулятор индикатора ATR
struct ATRCalculator {
    period: usize,
    tr_values: Vec<f64>,
    current_atr: Option<f64>,
    prev_close: Option<f64>,
}

impl ATRCalculator {
    fn new(period: usize) -> Self {
        ATRCalculator {
            period,
            tr_values: Vec::with_capacity(period),
            current_atr: None,
            prev_close: None,
        }
    }

    /// Рассчитать True Range для свечи
    fn calculate_true_range(&self, candle: &Candle) -> f64 {
        match self.prev_close {
            Some(prev_close) => {
                let high_low = candle.high - candle.low;
                let high_prev_close = (candle.high - prev_close).abs();
                let low_prev_close = (candle.low - prev_close).abs();

                high_low.max(high_prev_close).max(low_prev_close)
            }
            None => {
                // Первая свеча: TR просто High - Low
                candle.high - candle.low
            }
        }
    }

    /// Обновить ATR новой свечой
    fn update(&mut self, candle: &Candle) -> Option<f64> {
        let tr = self.calculate_true_range(candle);
        self.prev_close = Some(candle.close);

        match self.current_atr {
            None => {
                // Накапливаем начальный ATR
                self.tr_values.push(tr);

                if self.tr_values.len() >= self.period {
                    // Рассчитываем первый ATR как простое среднее
                    let sum: f64 = self.tr_values.iter().sum();
                    self.current_atr = Some(sum / self.period as f64);
                    self.tr_values.clear(); // Больше не нужны
                }

                self.current_atr
            }
            Some(prev_atr) => {
                // Метод сглаживания Уайлдера
                let new_atr = ((prev_atr * (self.period - 1) as f64) + tr)
                    / self.period as f64;
                self.current_atr = Some(new_atr);
                self.current_atr
            }
        }
    }

    /// Получить текущее значение ATR
    fn value(&self) -> Option<f64> {
        self.current_atr
    }
}

fn main() {
    let mut atr = ATRCalculator::new(14);

    // Симулированные свечи BTC/USDT
    let candles = vec![
        Candle { timestamp: 1, open: 42000.0, high: 42500.0, low: 41800.0, close: 42300.0, volume: 100.0 },
        Candle { timestamp: 2, open: 42300.0, high: 42800.0, low: 42100.0, close: 42600.0, volume: 120.0 },
        Candle { timestamp: 3, open: 42600.0, high: 43100.0, low: 42400.0, close: 42900.0, volume: 150.0 },
        Candle { timestamp: 4, open: 42900.0, high: 43200.0, low: 42700.0, close: 43000.0, volume: 130.0 },
        Candle { timestamp: 5, open: 43000.0, high: 43500.0, low: 42800.0, close: 43200.0, volume: 140.0 },
        Candle { timestamp: 6, open: 43200.0, high: 43800.0, low: 43000.0, close: 43600.0, volume: 160.0 },
        Candle { timestamp: 7, open: 43600.0, high: 44000.0, low: 43400.0, close: 43800.0, volume: 145.0 },
        Candle { timestamp: 8, open: 43800.0, high: 44200.0, low: 43500.0, close: 44000.0, volume: 155.0 },
        Candle { timestamp: 9, open: 44000.0, high: 44500.0, low: 43800.0, close: 44300.0, volume: 170.0 },
        Candle { timestamp: 10, open: 44300.0, high: 44800.0, low: 44100.0, close: 44500.0, volume: 165.0 },
        Candle { timestamp: 11, open: 44500.0, high: 45000.0, low: 44300.0, close: 44700.0, volume: 175.0 },
        Candle { timestamp: 12, open: 44700.0, high: 45200.0, low: 44500.0, close: 45000.0, volume: 180.0 },
        Candle { timestamp: 13, open: 45000.0, high: 45500.0, low: 44800.0, close: 45300.0, volume: 190.0 },
        Candle { timestamp: 14, open: 45300.0, high: 45800.0, low: 45000.0, close: 45500.0, volume: 185.0 },
        Candle { timestamp: 15, open: 45500.0, high: 46000.0, low: 45200.0, close: 45800.0, volume: 195.0 },
    ];

    for candle in &candles {
        let atr_value = atr.update(candle);
        match atr_value {
            Some(val) => println!(
                "Свеча {}: Close=${:.2}, ATR=${:.2}",
                candle.timestamp, candle.close, val
            ),
            None => println!(
                "Свеча {}: Close=${:.2}, ATR=рассчитывается...",
                candle.timestamp, candle.close
            ),
        }
    }
}
```

## ATR для размещения стоп-лосса

Одно из самых практичных применений ATR — определение уровней стоп-лосса:

```rust
#[derive(Debug)]
struct Position {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    side: Side,
    stop_loss: f64,
    take_profit: f64,
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Long,
    Short,
}

struct ATRBasedRiskManager {
    atr_multiplier_stop: f64,  // Обычно 2.0-3.0
    atr_multiplier_target: f64, // Обычно 2.0-4.0
}

impl ATRBasedRiskManager {
    fn new(stop_multiplier: f64, target_multiplier: f64) -> Self {
        ATRBasedRiskManager {
            atr_multiplier_stop: stop_multiplier,
            atr_multiplier_target: target_multiplier,
        }
    }

    fn calculate_position(
        &self,
        symbol: &str,
        entry_price: f64,
        side: Side,
        atr: f64,
        risk_amount: f64,
    ) -> Position {
        let stop_distance = atr * self.atr_multiplier_stop;
        let target_distance = atr * self.atr_multiplier_target;

        let (stop_loss, take_profit) = match side {
            Side::Long => (
                entry_price - stop_distance,
                entry_price + target_distance,
            ),
            Side::Short => (
                entry_price + stop_distance,
                entry_price - target_distance,
            ),
        };

        // Размер позиции на основе риска
        let quantity = risk_amount / stop_distance;

        Position {
            symbol: symbol.to_string(),
            entry_price,
            quantity,
            side,
            stop_loss,
            take_profit,
        }
    }
}

fn main() {
    let risk_manager = ATRBasedRiskManager::new(2.0, 3.0);

    // Текущий ATR для BTC составляет $800
    let current_atr = 800.0;
    let entry_price = 45000.0;
    let risk_per_trade = 1000.0; // Риск $1000 на сделку

    let long_position = risk_manager.calculate_position(
        "BTC/USDT",
        entry_price,
        Side::Long,
        current_atr,
        risk_per_trade,
    );

    println!("=== Длинная позиция ===");
    println!("Вход: ${:.2}", long_position.entry_price);
    println!("Стоп-лосс: ${:.2} ({:.1}% от входа)",
        long_position.stop_loss,
        ((entry_price - long_position.stop_loss) / entry_price) * 100.0
    );
    println!("Тейк-профит: ${:.2} ({:.1}% от входа)",
        long_position.take_profit,
        ((long_position.take_profit - entry_price) / entry_price) * 100.0
    );
    println!("Количество: {:.6} BTC", long_position.quantity);
    println!("Соотношение риск/прибыль: 1:{:.1}",
        (long_position.take_profit - entry_price) / (entry_price - long_position.stop_loss)
    );
}
```

## Мультитаймфреймовый анализ ATR

Профессиональные трейдеры часто анализируют ATR на нескольких таймфреймах:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Timeframe {
    M15,   // 15 минут
    H1,    // 1 час
    H4,    // 4 часа
    D1,    // Дневной
}

impl Timeframe {
    fn name(&self) -> &str {
        match self {
            Timeframe::M15 => "15мин",
            Timeframe::H1 => "1час",
            Timeframe::H4 => "4часа",
            Timeframe::D1 => "Дневной",
        }
    }
}

struct MultiTimeframeATR {
    atrs: HashMap<Timeframe, ATRCalculator>,
}

impl MultiTimeframeATR {
    fn new(period: usize) -> Self {
        let mut atrs = HashMap::new();
        atrs.insert(Timeframe::M15, ATRCalculator::new(period));
        atrs.insert(Timeframe::H1, ATRCalculator::new(period));
        atrs.insert(Timeframe::H4, ATRCalculator::new(period));
        atrs.insert(Timeframe::D1, ATRCalculator::new(period));

        MultiTimeframeATR { atrs }
    }

    fn update(&mut self, timeframe: Timeframe, candle: &Candle) -> Option<f64> {
        self.atrs.get_mut(&timeframe)?.update(candle)
    }

    fn get_atr(&self, timeframe: Timeframe) -> Option<f64> {
        self.atrs.get(&timeframe)?.value()
    }

    fn analyze_volatility(&self) -> VolatilityAnalysis {
        let atrs: Vec<(Timeframe, f64)> = [
            Timeframe::M15,
            Timeframe::H1,
            Timeframe::H4,
            Timeframe::D1,
        ]
        .iter()
        .filter_map(|&tf| self.get_atr(tf).map(|atr| (tf, atr)))
        .collect();

        if atrs.is_empty() {
            return VolatilityAnalysis {
                trend: VolatilityTrend::Unknown,
                recommendation: "Недостаточно данных".to_string(),
            };
        }

        // Сравниваем краткосрочный и долгосрочный ATR
        let short_term = self.get_atr(Timeframe::M15).or(self.get_atr(Timeframe::H1));
        let long_term = self.get_atr(Timeframe::D1).or(self.get_atr(Timeframe::H4));

        match (short_term, long_term) {
            (Some(short), Some(long)) => {
                // Нормализуем, сравнивая соотношения
                let ratio = short / long;

                let (trend, recommendation) = if ratio > 1.5 {
                    (
                        VolatilityTrend::Increasing,
                        "Высокая краткосрочная волатильность. Рассмотрите более тесные стопы или уменьшение размера позиции.".to_string()
                    )
                } else if ratio < 0.5 {
                    (
                        VolatilityTrend::Decreasing,
                        "Низкая краткосрочная волатильность. Формируется потенциальный сетап на пробой.".to_string()
                    )
                } else {
                    (
                        VolatilityTrend::Stable,
                        "Стабильная волатильность. Нормальные торговые условия.".to_string()
                    )
                };

                VolatilityAnalysis { trend, recommendation }
            }
            _ => VolatilityAnalysis {
                trend: VolatilityTrend::Unknown,
                recommendation: "Ожидание дополнительных данных...".to_string(),
            },
        }
    }
}

#[derive(Debug)]
enum VolatilityTrend {
    Increasing,
    Decreasing,
    Stable,
    Unknown,
}

#[derive(Debug)]
struct VolatilityAnalysis {
    trend: VolatilityTrend,
    recommendation: String,
}

fn main() {
    let mut mtf_atr = MultiTimeframeATR::new(14);

    println!("=== Мультитаймфреймовый анализ ATR ===\n");

    // Симулируем данные свечей для разных таймфреймов
    // В реальном трейдинге они приходили бы из потока данных

    let daily_candle = Candle {
        timestamp: 1,
        open: 44000.0,
        high: 46000.0,
        low: 43500.0,
        close: 45500.0,
        volume: 10000.0,
    };

    let h4_candle = Candle {
        timestamp: 1,
        open: 45000.0,
        high: 45800.0,
        low: 44800.0,
        close: 45500.0,
        volume: 2500.0,
    };

    mtf_atr.update(Timeframe::D1, &daily_candle);
    mtf_atr.update(Timeframe::H4, &h4_candle);

    for tf in [Timeframe::M15, Timeframe::H1, Timeframe::H4, Timeframe::D1] {
        match mtf_atr.get_atr(tf) {
            Some(atr) => println!("{}: ATR = ${:.2}", tf.name(), atr),
            None => println!("{}: ATR = рассчитывается...", tf.name()),
        }
    }

    let analysis = mtf_atr.analyze_volatility();
    println!("\nТренд волатильности: {:?}", analysis.trend);
    println!("Рекомендация: {}", analysis.recommendation);
}
```

## Размер позиции на основе ATR

Размер позиции критически важен для управления рисками:

```rust
struct PortfolioManager {
    total_capital: f64,
    risk_per_trade_percent: f64,  // например, 1% или 2%
    max_positions: usize,
}

#[derive(Debug)]
struct TradeSetup {
    symbol: String,
    entry_price: f64,
    atr: f64,
    atr_multiplier: f64,
}

#[derive(Debug)]
struct PositionSize {
    quantity: f64,
    dollar_risk: f64,
    stop_loss: f64,
    position_value: f64,
    percent_of_portfolio: f64,
}

impl PortfolioManager {
    fn new(capital: f64, risk_percent: f64, max_positions: usize) -> Self {
        PortfolioManager {
            total_capital: capital,
            risk_per_trade_percent: risk_percent,
            max_positions,
        }
    }

    fn calculate_position_size(&self, setup: &TradeSetup) -> PositionSize {
        // Максимальный долларовый риск на сделку
        let max_risk = self.total_capital * (self.risk_per_trade_percent / 100.0);

        // Расстояние до стопа на основе ATR
        let stop_distance = setup.atr * setup.atr_multiplier;

        // Рассчитываем количество на основе риска
        let quantity = max_risk / stop_distance;

        // Стоимость позиции
        let position_value = quantity * setup.entry_price;

        // Процент от портфеля
        let percent_of_portfolio = (position_value / self.total_capital) * 100.0;

        PositionSize {
            quantity,
            dollar_risk: max_risk,
            stop_loss: setup.entry_price - stop_distance,
            position_value,
            percent_of_portfolio,
        }
    }

    fn validate_position(&self, position: &PositionSize) -> Result<(), String> {
        // Проверяем, не слишком ли большая позиция
        if position.percent_of_portfolio > 25.0 {
            return Err(format!(
                "Позиция слишком большая: {:.1}% от портфеля (макс 25%)",
                position.percent_of_portfolio
            ));
        }

        Ok(())
    }
}

fn main() {
    let portfolio = PortfolioManager::new(100_000.0, 2.0, 5);

    let btc_setup = TradeSetup {
        symbol: "BTC/USDT".to_string(),
        entry_price: 45000.0,
        atr: 800.0,
        atr_multiplier: 2.0,
    };

    let eth_setup = TradeSetup {
        symbol: "ETH/USDT".to_string(),
        entry_price: 2500.0,
        atr: 60.0,
        atr_multiplier: 2.0,
    };

    println!("=== Размер позиции на основе ATR ===\n");
    println!("Портфель: ${:.0}", portfolio.total_capital);
    println!("Риск на сделку: {:.1}%\n", portfolio.risk_per_trade_percent);

    for setup in [&btc_setup, &eth_setup] {
        let position = portfolio.calculate_position_size(setup);

        println!("--- {} ---", setup.symbol);
        println!("Вход: ${:.2}", setup.entry_price);
        println!("ATR: ${:.2}", setup.atr);
        println!("Стоп-лосс: ${:.2}", position.stop_loss);
        println!("Количество: {:.6}", position.quantity);
        println!("Стоимость позиции: ${:.2}", position.position_value);
        println!("% портфеля: {:.1}%", position.percent_of_portfolio);
        println!("Долларовый риск: ${:.2}", position.dollar_risk);

        match portfolio.validate_position(&position) {
            Ok(()) => println!("Статус: ВАЛИДНА"),
            Err(e) => println!("Статус: НЕВАЛИДНА - {}", e),
        }
        println!();
    }
}
```

## Стратегия трейлинг-стопа на основе ATR

Динамический трейлинг-стоп на основе ATR:

```rust
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    side: Side,
    highest_price: f64,  // Для длинных позиций
    lowest_price: f64,   // Для коротких позиций
    trailing_stop: f64,
}

struct ATRTrailingStop {
    atr_multiplier: f64,
}

impl ATRTrailingStop {
    fn new(multiplier: f64) -> Self {
        ATRTrailingStop {
            atr_multiplier: multiplier,
        }
    }

    fn update_trailing_stop(&self, trade: &mut Trade, current_price: f64, atr: f64) -> TradeAction {
        let trail_distance = atr * self.atr_multiplier;

        match trade.side {
            Side::Long => {
                // Обновляем максимальную цену при новом хае
                if current_price > trade.highest_price {
                    trade.highest_price = current_price;
                    trade.trailing_stop = current_price - trail_distance;
                    return TradeAction::UpdateStop(trade.trailing_stop);
                }

                // Проверяем, сработал ли стоп
                if current_price <= trade.trailing_stop {
                    let pnl = (trade.trailing_stop - trade.entry_price) * trade.quantity;
                    return TradeAction::Exit {
                        exit_price: trade.trailing_stop,
                        pnl,
                        reason: "Сработал трейлинг-стоп".to_string(),
                    };
                }

                TradeAction::Hold
            }
            Side::Short => {
                // Обновляем минимальную цену при новом лоу
                if current_price < trade.lowest_price {
                    trade.lowest_price = current_price;
                    trade.trailing_stop = current_price + trail_distance;
                    return TradeAction::UpdateStop(trade.trailing_stop);
                }

                // Проверяем, сработал ли стоп
                if current_price >= trade.trailing_stop {
                    let pnl = (trade.entry_price - trade.trailing_stop) * trade.quantity;
                    return TradeAction::Exit {
                        exit_price: trade.trailing_stop,
                        pnl,
                        reason: "Сработал трейлинг-стоп".to_string(),
                    };
                }

                TradeAction::Hold
            }
        }
    }
}

#[derive(Debug)]
enum TradeAction {
    Hold,
    UpdateStop(f64),
    Exit {
        exit_price: f64,
        pnl: f64,
        reason: String,
    },
}

fn main() {
    let trailing_stop = ATRTrailingStop::new(2.0);
    let atr = 800.0;

    let mut trade = Trade {
        symbol: "BTC/USDT".to_string(),
        entry_price: 45000.0,
        quantity: 0.1,
        side: Side::Long,
        highest_price: 45000.0,
        lowest_price: 45000.0,
        trailing_stop: 45000.0 - (atr * 2.0), // Начальный стоп
    };

    println!("=== Симуляция трейлинг-стопа на ATR ===\n");
    println!("Вход: ${:.2}", trade.entry_price);
    println!("Начальный стоп: ${:.2}", trade.trailing_stop);
    println!("ATR: ${:.2}\n", atr);

    // Симулируем движение цены
    let prices = vec![
        45500.0, 46000.0, 46500.0, 47000.0, 47500.0,
        47200.0, 46800.0, 46000.0, 45500.0, 45000.0,
    ];

    for (i, price) in prices.iter().enumerate() {
        let action = trailing_stop.update_trailing_stop(&mut trade, *price, atr);

        print!("Тик {}: Цена=${:.2} | ", i + 1, price);

        match action {
            TradeAction::Hold => {
                println!("Удержание (Стоп: ${:.2})", trade.trailing_stop);
            }
            TradeAction::UpdateStop(new_stop) => {
                println!("Стоп обновлён до ${:.2}", new_stop);
            }
            TradeAction::Exit { exit_price, pnl, reason } => {
                println!("{} по ${:.2}", reason, exit_price);
                println!("\n=== Сделка закрыта ===");
                println!("Вход: ${:.2}", trade.entry_price);
                println!("Выход: ${:.2}", exit_price);
                println!("PnL: ${:.2}", pnl);
                break;
            }
        }
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| True Range | Максимум из: (High-Low), \|High-PrevClose\|, \|Low-PrevClose\| |
| ATR | Среднее значений True Range за N периодов |
| Сглаживание Уайлдера | ATR = ((Пред ATR × (n-1)) + TR) / n |
| Размещение стопа | Обычно 2-3 ATR от цены входа |
| Размер позиции | Сумма риска / (ATR × Множитель) = Количество |
| Трейлинг-стоп | Стоп следует за ценой на расстоянии ATR |

## Домашнее задание

1. **ATR с разными периодами**: Реализуй калькулятор ATR, который может переключаться между разными периодами (7, 14, 21). Сравни, как они реагируют на одни и те же ценовые данные, и объясни, какой период лучше для:
   - Дейтрейдинга
   - Свинг-трейдинга
   - Долгосрочного инвестирования

2. **Стратегия пробоя на ATR**: Создай простую торговую стратегию, которая:
   - Рассчитывает 14-периодный ATR
   - Генерирует сигнал на покупку, когда цена пробивает вверх (Предыдущий Close + 1.5 × ATR)
   - Генерирует сигнал на продажу, когда цена пробивает вниз (Предыдущий Close - 1.5 × ATR)
   - Отслеживает гипотетические сделки и рассчитывает процент выигрышей

3. **Инструмент сравнения волатильности**: Создай инструмент, который:
   - Рассчитывает ATR для нескольких торговых пар (BTC, ETH, SOL)
   - Нормализует ATR как процент от цены (ATR / Цена × 100)
   - Ранжирует активы по волатильности
   - Предлагает корректировки размера позиции для каждого актива

4. **Риск-менеджер на основе ATR**: Создай полноценную систему управления рисками, которая:
   - Использует ATR для размещения стоп-лосса (2× ATR)
   - Использует ATR для уровней тейк-профита (3× ATR)
   - Рассчитывает размер позиции на основе риска счёта (1-2%)
   - Проверяет, что общий риск портфеля не превышает 10%
   - Логирует все расчёты для анализа

## Навигация

[← Предыдущий день](../251-stochastic-oscillator/ru.md) | [Следующий день →](../253-bollinger-bands/ru.md)
