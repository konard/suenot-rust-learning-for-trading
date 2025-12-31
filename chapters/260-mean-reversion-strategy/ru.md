# День 260: Стратегия: Возврат к среднему (Mean Reversion)

## Аналогия из трейдинга

Представь резинку, натянутую между двумя точками. Как бы сильно ты её ни растянул, она всегда стремится вернуться в своё естественное положение. Именно так работает возврат к среднему на финансовых рынках. Когда цена актива значительно отклоняется от своего исторического среднего, существует тенденция к "возврату" к этому среднему значению.

Представь трейдера, наблюдающего за ценой Bitcoin. Если BTC обычно торгуется около $50,000, но внезапно падает до $40,000 из-за краткосрочной паники, трейдер, использующий стратегию возврата к среднему, видит возможность. Он верит, что цена в конечном итоге вернётся к своему "нормальному" уровню, поэтому покупает по $40,000, ожидая прибыли при возврате к среднему.

В алгоритмической торговле стратегии возврата к среднему популярны потому, что:
- Они предоставляют чёткие сигналы входа и выхода на основе статистических показателей
- Они хорошо работают на рынках в боковике (без явного тренда)
- Их легко тестировать на исторических данных
- Они хорошо сочетаются с правилами управления рисками

## Что такое возврат к среднему?

Возврат к среднему основан на статистической концепции, что цены и доходности со временем стремятся вернуться к своему среднему значению. Ключевые компоненты:

1. **Среднее (Mean)** — центральное значение, вокруг которого колеблются цены
2. **Стандартное отклонение** — измеряет, насколько далеко цены обычно отклоняются от среднего
3. **Z-Score** — показывает, на сколько стандартных отклонений текущая цена отклонилась от среднего
4. **Полосы Боллинджера** — визуальное представление среднего и границ отклонения

## Базовый расчёт среднего в Rust

```rust
/// Вычисляет простую скользящую среднюю (SMA) цен
fn calculate_mean(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    let sum: f64 = prices.iter().sum();
    Some(sum / prices.len() as f64)
}

/// Вычисляет стандартное отклонение цен
fn calculate_std_dev(prices: &[f64], mean: f64) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    let variance: f64 = prices
        .iter()
        .map(|price| {
            let diff = price - mean;
            diff * diff
        })
        .sum::<f64>() / prices.len() as f64;

    Some(variance.sqrt())
}

/// Вычисляет Z-Score для заданной цены
fn calculate_z_score(price: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev == 0.0 {
        return 0.0;
    }
    (price - mean) / std_dev
}

fn main() {
    let btc_prices = vec![
        50000.0, 51000.0, 49500.0, 50500.0, 48000.0,
        47500.0, 49000.0, 50000.0, 51500.0, 52000.0,
        48500.0, 47000.0, 46000.0, 45000.0, 44000.0,
    ];

    let mean = calculate_mean(&btc_prices).unwrap();
    let std_dev = calculate_std_dev(&btc_prices, mean).unwrap();
    let current_price = 44000.0;
    let z_score = calculate_z_score(current_price, mean, std_dev);

    println!("Анализ цены:");
    println!("  Среднее: ${:.2}", mean);
    println!("  Станд. откл.: ${:.2}", std_dev);
    println!("  Текущая цена: ${:.2}", current_price);
    println!("  Z-Score: {:.2}", z_score);

    if z_score < -2.0 {
        println!("  Сигнал: СИЛЬНАЯ ПОКУПКА (цена значительно ниже среднего)");
    } else if z_score < -1.0 {
        println!("  Сигнал: ПОКУПКА (цена ниже среднего)");
    } else if z_score > 2.0 {
        println!("  Сигнал: СИЛЬНАЯ ПРОДАЖА (цена значительно выше среднего)");
    } else if z_score > 1.0 {
        println!("  Сигнал: ПРОДАЖА (цена выше среднего)");
    } else {
        println!("  Сигнал: ОЖИДАНИЕ (цена близка к среднему)");
    }
}
```

## Полная система торговли на возврате к среднему

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

#[derive(Debug, Clone, Copy)]
struct Position {
    symbol: &'static str,
    quantity: f64,
    entry_price: f64,
    entry_z_score: f64,
}

#[derive(Debug)]
struct MeanReversionStrategy {
    symbol: &'static str,
    lookback_period: usize,
    entry_z_threshold: f64,   // Z-score для входа в сделку
    exit_z_threshold: f64,    // Z-score для выхода (возврат к среднему)
    price_history: VecDeque<f64>,
    position: Option<Position>,
    cash: f64,
    total_trades: u32,
    winning_trades: u32,
}

impl MeanReversionStrategy {
    fn new(symbol: &'static str, lookback_period: usize, initial_cash: f64) -> Self {
        MeanReversionStrategy {
            symbol,
            lookback_period,
            entry_z_threshold: 2.0,  // Входим при 2 станд. откл. от среднего
            exit_z_threshold: 0.5,   // Выходим близко к среднему
            price_history: VecDeque::with_capacity(lookback_period),
            position: None,
            cash: initial_cash,
            total_trades: 0,
            winning_trades: 0,
        }
    }

    fn add_price(&mut self, price: f64) {
        if self.price_history.len() >= self.lookback_period {
            self.price_history.pop_front();
        }
        self.price_history.push_back(price);
    }

    fn calculate_statistics(&self) -> Option<(f64, f64)> {
        if self.price_history.len() < self.lookback_period {
            return None;
        }

        let prices: Vec<f64> = self.price_history.iter().copied().collect();
        let mean: f64 = prices.iter().sum::<f64>() / prices.len() as f64;

        let variance: f64 = prices
            .iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;

        let std_dev = variance.sqrt();

        Some((mean, std_dev))
    }

    fn get_z_score(&self, price: f64) -> Option<f64> {
        self.calculate_statistics().map(|(mean, std_dev)| {
            if std_dev == 0.0 {
                0.0
            } else {
                (price - mean) / std_dev
            }
        })
    }

    fn generate_signal(&self, price: f64) -> Signal {
        match self.get_z_score(price) {
            Some(z) if z <= -self.entry_z_threshold => Signal::StrongBuy,
            Some(z) if z <= -1.0 => Signal::Buy,
            Some(z) if z >= self.entry_z_threshold => Signal::StrongSell,
            Some(z) if z >= 1.0 => Signal::Sell,
            _ => Signal::Hold,
        }
    }

    fn should_exit_position(&self, price: f64) -> bool {
        if let Some(ref position) = self.position {
            if let Some(z_score) = self.get_z_score(price) {
                // Выходим, когда цена возвращается к среднему
                if position.entry_z_score < 0.0 {
                    // Длинная позиция: выходим, когда z-score становится положительным
                    return z_score >= self.exit_z_threshold;
                } else {
                    // Короткая позиция: выходим, когда z-score становится отрицательным
                    return z_score <= -self.exit_z_threshold;
                }
            }
        }
        false
    }

    fn execute_trade(&mut self, price: f64, signal: Signal) -> Option<String> {
        // Проверяем, нужно ли закрыть существующую позицию
        if self.position.is_some() && self.should_exit_position(price) {
            return self.close_position(price);
        }

        // Открываем новую позицию, если её нет
        if self.position.is_none() {
            match signal {
                Signal::StrongBuy => {
                    return self.open_long(price);
                }
                Signal::StrongSell => {
                    return self.open_short(price);
                }
                _ => {}
            }
        }

        None
    }

    fn open_long(&mut self, price: f64) -> Option<String> {
        let quantity = (self.cash * 0.95) / price; // Используем 95% средств
        let z_score = self.get_z_score(price)?;

        self.position = Some(Position {
            symbol: self.symbol,
            quantity,
            entry_price: price,
            entry_z_score: z_score,
        });
        self.cash -= quantity * price;

        Some(format!(
            "ЛОНГ {} {:.4} @ ${:.2} (Z-Score: {:.2})",
            self.symbol, quantity, price, z_score
        ))
    }

    fn open_short(&mut self, price: f64) -> Option<String> {
        let quantity = (self.cash * 0.95) / price;
        let z_score = self.get_z_score(price)?;

        // Для простоты симулируем шорт через отрицательное количество
        self.position = Some(Position {
            symbol: self.symbol,
            quantity: -quantity,
            entry_price: price,
            entry_z_score: z_score,
        });
        self.cash += quantity * price; // Получаем деньги от короткой продажи

        Some(format!(
            "ШОРТ {} {:.4} @ ${:.2} (Z-Score: {:.2})",
            self.symbol, quantity, price, z_score
        ))
    }

    fn close_position(&mut self, price: f64) -> Option<String> {
        let position = self.position.take()?;
        let z_score = self.get_z_score(price).unwrap_or(0.0);

        let pnl = if position.quantity > 0.0 {
            // Закрываем длинную позицию
            let revenue = position.quantity * price;
            self.cash += revenue;
            revenue - (position.quantity * position.entry_price)
        } else {
            // Закрываем короткую позицию
            let cost = (-position.quantity) * price;
            self.cash -= cost;
            (position.entry_price - price) * (-position.quantity)
        };

        self.total_trades += 1;
        if pnl > 0.0 {
            self.winning_trades += 1;
        }

        Some(format!(
            "ЗАКРЫТИЕ {} @ ${:.2} | PnL: ${:.2} | Z-Score: {:.2}",
            self.symbol, price, pnl, z_score
        ))
    }

    fn get_portfolio_value(&self, current_price: f64) -> f64 {
        let position_value = match &self.position {
            Some(pos) if pos.quantity > 0.0 => pos.quantity * current_price,
            Some(pos) => {
                // Короткая позиция: прибыль при падении цены
                let short_qty = -pos.quantity;
                let initial_value = short_qty * pos.entry_price;
                let current_value = short_qty * current_price;
                initial_value - current_value + self.cash
            }
            None => 0.0,
        };

        if self.position.as_ref().map_or(true, |p| p.quantity > 0.0) {
            self.cash + position_value
        } else {
            position_value
        }
    }

    fn get_win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            (self.winning_trades as f64 / self.total_trades as f64) * 100.0
        }
    }
}

fn main() {
    // Симулированные данные цены BTC с характеристиками возврата к среднему
    let prices = vec![
        50000.0, 50500.0, 51000.0, 51500.0, 52000.0,  // Рост
        52500.0, 53000.0, 54000.0, 55000.0, 56000.0,  // Перекупленность
        55000.0, 54000.0, 52000.0, 50000.0, 49000.0,  // Возврат
        48000.0, 47000.0, 46000.0, 45000.0, 44000.0,  // Перепроданность
        45000.0, 46000.0, 48000.0, 49000.0, 50000.0,  // Возврат обратно
        50500.0, 51000.0, 50500.0, 50000.0, 49500.0,  // Стабилизация
    ];

    let mut strategy = MeanReversionStrategy::new("BTC", 10, 100_000.0);

    println!("=== Бэктест стратегии возврата к среднему ===\n");
    println!("Начальный капитал: $100,000.00");
    println!("Период расчёта: 10 периодов");
    println!("Порог Z-Score для входа: +/- 2.0");
    println!("Порог Z-Score для выхода: +/- 0.5\n");
    println!("{:-<60}", "");

    for (day, &price) in prices.iter().enumerate() {
        strategy.add_price(price);

        if let Some(z_score) = strategy.get_z_score(price) {
            let signal = strategy.generate_signal(price);

            print!("День {:2}: ${:.2} | Z: {:+.2} | ", day + 1, price, z_score);

            if let Some(action) = strategy.execute_trade(price, signal) {
                println!("{}", action);
            } else {
                println!("Сигнал: {:?}", signal);
            }
        } else {
            println!("День {:2}: ${:.2} | Сбор данных...", day + 1, price);
        }
    }

    let final_price = *prices.last().unwrap();
    println!("{:-<60}", "");
    println!("\n=== Результаты стратегии ===");
    println!("Итоговая стоимость портфеля: ${:.2}", strategy.get_portfolio_value(final_price));
    println!("Всего сделок: {}", strategy.total_trades);
    println!("Процент выигрышных: {:.1}%", strategy.get_win_rate());
    println!("Остаток средств: ${:.2}", strategy.cash);
}
```

## Реализация полос Боллинджера

```rust
#[derive(Debug, Clone)]
struct BollingerBands {
    period: usize,
    num_std_dev: f64,
    prices: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
struct BandValues {
    upper: f64,
    middle: f64,
    lower: f64,
    bandwidth: f64,
    percent_b: f64,
}

impl BollingerBands {
    fn new(period: usize, num_std_dev: f64) -> Self {
        BollingerBands {
            period,
            num_std_dev,
            prices: Vec::new(),
        }
    }

    fn add_price(&mut self, price: f64) {
        self.prices.push(price);
    }

    fn calculate(&self) -> Option<BandValues> {
        if self.prices.len() < self.period {
            return None;
        }

        let recent_prices: Vec<f64> = self.prices
            .iter()
            .rev()
            .take(self.period)
            .copied()
            .collect();

        let middle = recent_prices.iter().sum::<f64>() / self.period as f64;

        let variance: f64 = recent_prices
            .iter()
            .map(|p| (p - middle).powi(2))
            .sum::<f64>() / self.period as f64;

        let std_dev = variance.sqrt();
        let band_width = self.num_std_dev * std_dev;

        let upper = middle + band_width;
        let lower = middle - band_width;

        let current_price = *self.prices.last().unwrap();
        let bandwidth = (upper - lower) / middle * 100.0;
        let percent_b = (current_price - lower) / (upper - lower);

        Some(BandValues {
            upper,
            middle,
            lower,
            bandwidth,
            percent_b,
        })
    }

    fn get_signal(&self) -> Option<&'static str> {
        let bands = self.calculate()?;
        let current_price = *self.prices.last()?;

        if current_price <= bands.lower {
            Some("ПОКУПКА - Цена на нижней границе")
        } else if current_price >= bands.upper {
            Some("ПРОДАЖА - Цена на верхней границе")
        } else if bands.percent_b < 0.2 {
            Some("Рассмотреть ПОКУПКУ - Близко к нижней границе")
        } else if bands.percent_b > 0.8 {
            Some("Рассмотреть ПРОДАЖУ - Близко к верхней границе")
        } else {
            Some("ОЖИДАНИЕ - Цена внутри полос")
        }
    }
}

fn main() {
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0,
        104.0, 106.0, 108.0, 107.0, 105.0,
        103.0, 101.0, 99.0, 97.0, 95.0,
        96.0, 98.0, 100.0, 102.0, 104.0,
    ];

    let mut bb = BollingerBands::new(10, 2.0);

    println!("=== Анализ полос Боллинджера ===\n");

    for (day, &price) in prices.iter().enumerate() {
        bb.add_price(price);

        if let Some(bands) = bb.calculate() {
            println!("День {}: Цена ${:.2}", day + 1, price);
            println!("  Верхняя полоса:  ${:.2}", bands.upper);
            println!("  Средняя линия:   ${:.2}", bands.middle);
            println!("  Нижняя полоса:   ${:.2}", bands.lower);
            println!("  Ширина полос:    {:.2}%", bands.bandwidth);
            println!("  %B:              {:.2}", bands.percent_b);
            if let Some(signal) = bb.get_signal() {
                println!("  Сигнал:          {}", signal);
            }
            println!();
        }
    }
}
```

## Управление рисками для стратегии возврата к среднему

```rust
use std::collections::VecDeque;

#[derive(Debug)]
struct RiskManagedMeanReversion {
    symbol: String,
    lookback_period: usize,
    max_position_size: f64,      // Максимальный размер позиции в % от портфеля
    stop_loss_pct: f64,          // Процент стоп-лосса
    take_profit_pct: f64,        // Процент тейк-профита
    max_drawdown_pct: f64,       // Максимально допустимая просадка
    price_history: VecDeque<f64>,
    entry_price: Option<f64>,
    peak_portfolio_value: f64,
    current_portfolio_value: f64,
    is_trading_halted: bool,
}

impl RiskManagedMeanReversion {
    fn new(symbol: &str, initial_capital: f64) -> Self {
        RiskManagedMeanReversion {
            symbol: symbol.to_string(),
            lookback_period: 20,
            max_position_size: 0.25,      // 25% макс. позиция
            stop_loss_pct: 0.05,          // 5% стоп-лосс
            take_profit_pct: 0.10,        // 10% тейк-профит
            max_drawdown_pct: 0.15,       // 15% макс. просадка
            price_history: VecDeque::with_capacity(20),
            entry_price: None,
            peak_portfolio_value: initial_capital,
            current_portfolio_value: initial_capital,
            is_trading_halted: false,
        }
    }

    fn update_portfolio_value(&mut self, new_value: f64) {
        self.current_portfolio_value = new_value;
        if new_value > self.peak_portfolio_value {
            self.peak_portfolio_value = new_value;
        }

        // Проверяем просадку
        let drawdown = (self.peak_portfolio_value - self.current_portfolio_value)
            / self.peak_portfolio_value;

        if drawdown >= self.max_drawdown_pct {
            self.is_trading_halted = true;
            println!("ВНИМАНИЕ: Торговля остановлена из-за просадки {:.1}%!", drawdown * 100.0);
        }
    }

    fn calculate_position_size(&self, entry_price: f64) -> f64 {
        if self.is_trading_halted {
            return 0.0;
        }

        let max_investment = self.current_portfolio_value * self.max_position_size;
        let position_size = max_investment / entry_price;

        position_size
    }

    fn check_stop_loss(&self, current_price: f64) -> bool {
        if let Some(entry) = self.entry_price {
            let loss_pct = (entry - current_price) / entry;
            return loss_pct >= self.stop_loss_pct;
        }
        false
    }

    fn check_take_profit(&self, current_price: f64) -> bool {
        if let Some(entry) = self.entry_price {
            let profit_pct = (current_price - entry) / entry;
            return profit_pct >= self.take_profit_pct;
        }
        false
    }

    fn get_risk_metrics(&self) -> String {
        let drawdown = (self.peak_portfolio_value - self.current_portfolio_value)
            / self.peak_portfolio_value * 100.0;

        format!(
            "Портфель: ${:.2} | Пик: ${:.2} | Просадка: {:.2}% | Статус: {}",
            self.current_portfolio_value,
            self.peak_portfolio_value,
            drawdown,
            if self.is_trading_halted { "ОСТАНОВЛЕН" } else { "АКТИВЕН" }
        )
    }
}

fn main() {
    let mut strategy = RiskManagedMeanReversion::new("ETH", 50_000.0);

    println!("=== Возврат к среднему с управлением рисками ===\n");
    println!("Макс. размер позиции: {:.0}%", strategy.max_position_size * 100.0);
    println!("Стоп-лосс: {:.0}%", strategy.stop_loss_pct * 100.0);
    println!("Тейк-профит: {:.0}%", strategy.take_profit_pct * 100.0);
    println!("Макс. просадка: {:.0}%\n", strategy.max_drawdown_pct * 100.0);

    // Симулируем торговый сценарий
    strategy.entry_price = Some(2000.0);
    let position_size = strategy.calculate_position_size(2000.0);
    println!("Вход по $2000.00");
    println!("Размер позиции: {:.4} ETH (${:.2})\n",
        position_size, position_size * 2000.0);

    // Симулируем движение цены
    let prices = vec![2000.0, 1980.0, 1950.0, 1900.0, 1850.0, 2100.0, 2200.0];

    for price in prices {
        let position_value = position_size * price;
        strategy.update_portfolio_value(50_000.0 - (position_size * 2000.0) + position_value);

        println!("Цена: ${:.2}", price);
        println!("  {}", strategy.get_risk_metrics());

        if strategy.check_stop_loss(price) {
            println!("  ДЕЙСТВИЕ: Сработал стоп-лосс!");
        }
        if strategy.check_take_profit(price) {
            println!("  ДЕЙСТВИЕ: Сработал тейк-профит!");
        }
        println!();
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Возврат к среднему | Стратегия, основанная на возврате цен к историческому среднему |
| Z-Score | Измеряет расстояние от среднего в стандартных отклонениях |
| Полосы Боллинджера | Визуальные полосы, показывающие среднее и границы волатильности |
| Стандартное отклонение | Статистическая мера разброса цен |
| Сигналы входа/выхода | Правила открытия и закрытия позиций |
| Управление рисками | Правила стоп-лосса, тейк-профита и размера позиции |

## Домашнее задание

1. **Улучшенная стратегия Z-Score**: Модифицируй стратегию возврата к среднему, используя экспоненциальную скользящую среднюю (EMA) вместо простой. Сравни производительность обоих подходов на исторических данных.

2. **Мультиактивный возврат к среднему**: Реализуй стратегию, которая торгует парами коррелированных активов (например, BTC и ETH). Когда спред между ними отклоняется от исторического среднего, открывай позиции в ожидании конвергенции.

3. **Динамические пороги**: Создай систему, которая автоматически корректирует пороги Z-Score для входа и выхода на основе недавней волатильности рынка. Более высокая волатильность должна требовать больших отклонений для срабатывания сигналов.

4. **Фреймворк для бэктестирования**: Создай полноценный фреймворк для бэктестирования, который:
   - Читает исторические данные о ценах из файла
   - Запускает стратегию возврата к среднему
   - Рассчитывает метрики производительности (коэффициент Шарпа, максимальная просадка, процент выигрышных сделок)
   - Генерирует отчёт с анализом каждой сделки

## Навигация

[← Предыдущий день](../259-momentum-strategy/ru.md) | [Следующий день →](../261-arbitrage-strategy/ru.md)
