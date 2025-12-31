# День 290: Walk-forward анализ

## Аналогия из трейдинга

Представь, что ты разработал прибыльную торговую стратегию на исторических данных за 2020 год. Бэктест показывает потрясающие результаты — прибыль каждый месяц! В восторге ты запускаешь её в 2021 году, но она начинает терять деньги. Что произошло?

Стратегия была **переоптимизирована** под данные 2020 года. Она научилась торговать именно условия рынка 2020 года, а не универсальные торговые принципы. Это как готовиться к экзамену, изучая только прошлогодние вопросы, и ожидать тех же вопросов в этом году.

**Walk-forward анализ** — это как сдавать пробные экзамены на протяжении всего года, а не просто зубрить старые задачи. Ты:
1. Тренируешь стратегию на данных январь-март
2. Тестируешь её на апреле (вне выборки)
3. Заново тренируешь на данных февраль-апрель
4. Тестируешь на мае
5. Продолжаешь идти вперёд по времени

Это симулирует реальный трейдинг, где ты постоянно адаптируешь стратегию к новым рыночным данным, проверяя её на невиденных периодах.

## Что такое Walk-forward анализ?

Walk-forward анализ — это метод бэктестинга, который делит исторические данные на множество периодов обучения (in-sample) и тестирования (out-of-sample), которые перемещаются вперёд по времени. Он помогает:

1. **Обнаружить переоптимизацию** — стратегии, работающие только на исторических данных, проваливаются на периодах вне выборки
2. **Симулировать реальность** — реальный трейдинг предполагает постоянную ре-оптимизацию
3. **Проверить устойчивость** — надёжные стратегии работают стабильно во всех тестовых окнах
4. **Оценить реальную доходность** — результаты вне выборки лучше предсказывают торговлю вживую

### Процесс Walk-Forward

```
Время ────────────────────────────────────────────────────►

Окно 1:  [====Обучение====][Тест]
Окно 2:      [====Обучение====][Тест]
Окно 3:          [====Обучение====][Тест]
Окно 4:              [====Обучение====][Тест]
                                        ...
```

Каждое окно состоит из:
- **Период In-Sample** — оптимизация параметров стратегии
- **Период Out-of-Sample** — тестирование оптимизированных параметров без изменений

## Базовая реализация Walk-Forward

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    profit: f64,
}

#[derive(Debug, Clone)]
struct StrategyParams {
    fast_period: usize,
    slow_period: usize,
    stop_loss: f64,
}

#[derive(Debug)]
struct WalkForwardResult {
    window: usize,
    in_sample_profit: f64,
    out_sample_profit: f64,
    optimized_params: StrategyParams,
}

fn simple_moving_average(prices: &[f64], period: usize) -> Vec<f64> {
    let mut sma = Vec::new();

    for i in 0..prices.len() {
        if i < period - 1 {
            sma.push(0.0);
        } else {
            let sum: f64 = prices[i - period + 1..=i].iter().sum();
            sma.push(sum / period as f64);
        }
    }

    sma
}

fn backtest_strategy(
    prices: &[f64],
    params: &StrategyParams,
) -> f64 {
    let fast_sma = simple_moving_average(prices, params.fast_period);
    let slow_sma = simple_moving_average(prices, params.slow_period);

    let mut equity = 10_000.0;
    let mut position: Option<f64> = None;

    for i in params.slow_period..prices.len() {
        if fast_sma[i] == 0.0 || slow_sma[i] == 0.0 {
            continue;
        }

        // Сигнал покупки: быстрая пересекает медленную снизу вверх
        if position.is_none()
            && fast_sma[i] > slow_sma[i]
            && fast_sma[i - 1] <= slow_sma[i - 1]
        {
            position = Some(prices[i]);
        }
        // Сигнал продажи: быстрая пересекает медленную сверху вниз ИЛИ стоп-лосс
        else if let Some(entry_price) = position {
            let should_sell = fast_sma[i] < slow_sma[i] && fast_sma[i - 1] >= slow_sma[i - 1];
            let hit_stop_loss = (prices[i] - entry_price) / entry_price < -params.stop_loss;

            if should_sell || hit_stop_loss {
                let profit = (prices[i] - entry_price) / entry_price;
                equity *= 1.0 + profit;
                position = None;
            }
        }
    }

    // Закрываем открытую позицию
    if let Some(entry_price) = position {
        let profit = (prices[prices.len() - 1] - entry_price) / entry_price;
        equity *= 1.0 + profit;
    }

    (equity - 10_000.0) / 10_000.0 * 100.0 // Возвращаем как процент
}

fn optimize_parameters(prices: &[f64]) -> (StrategyParams, f64) {
    let mut best_params = StrategyParams {
        fast_period: 10,
        slow_period: 20,
        stop_loss: 0.05,
    };
    let mut best_profit = f64::MIN;

    // Перебор параметров
    for fast in [5, 10, 15, 20].iter() {
        for slow in [20, 30, 50, 100].iter() {
            for stop in [0.02, 0.05, 0.10].iter() {
                if fast >= slow {
                    continue;
                }

                let params = StrategyParams {
                    fast_period: *fast,
                    slow_period: *slow,
                    stop_loss: *stop,
                };

                let profit = backtest_strategy(prices, &params);

                if profit > best_profit {
                    best_profit = profit;
                    best_params = params;
                }
            }
        }
    }

    (best_params, best_profit)
}

fn walk_forward_analysis(
    prices: &[f64],
    in_sample_size: usize,
    out_sample_size: usize,
) -> Vec<WalkForwardResult> {
    let mut results = Vec::new();
    let window_size = in_sample_size + out_sample_size;
    let num_windows = (prices.len() - in_sample_size) / out_sample_size;

    for window in 0..num_windows {
        let start_idx = window * out_sample_size;
        let in_sample_end = start_idx + in_sample_size;
        let out_sample_end = in_sample_end + out_sample_size;

        if out_sample_end > prices.len() {
            break;
        }

        // Оптимизация на in-sample
        let in_sample_data = &prices[start_idx..in_sample_end];
        let (optimized_params, in_sample_profit) = optimize_parameters(in_sample_data);

        // Тестирование на out-of-sample
        let out_sample_data = &prices[in_sample_end..out_sample_end];
        let out_sample_profit = backtest_strategy(out_sample_data, &optimized_params);

        results.push(WalkForwardResult {
            window: window + 1,
            in_sample_profit,
            out_sample_profit,
            optimized_params: optimized_params.clone(),
        });

        println!("Окно {}: In-sample: {:.2}%, Out-sample: {:.2}%",
            window + 1, in_sample_profit, out_sample_profit);
    }

    results
}

fn main() {
    // Симулированные данные цен (синусоида с трендом и шумом)
    let prices: Vec<f64> = (0..500)
        .map(|i| {
            let trend = 100.0 + i as f64 * 0.1;
            let cycle = 5.0 * (i as f64 * 0.1).sin();
            let noise = (i as f64 * 7.0).sin() * 2.0;
            trend + cycle + noise
        })
        .collect();

    println!("Запуск Walk-Forward анализа...\n");

    let results = walk_forward_analysis(
        &prices,
        200, // in-sample: 200 периодов
        50,  // out-of-sample: 50 периодов
    );

    // Расчёт статистики
    let total_windows = results.len() as f64;
    let avg_in_sample: f64 = results.iter()
        .map(|r| r.in_sample_profit)
        .sum::<f64>() / total_windows;
    let avg_out_sample: f64 = results.iter()
        .map(|r| r.out_sample_profit)
        .sum::<f64>() / total_windows;

    let profitable_windows = results.iter()
        .filter(|r| r.out_sample_profit > 0.0)
        .count();

    println!("\n=== Итоги Walk-Forward анализа ===");
    println!("Всего окон: {}", results.len());
    println!("Средняя прибыль in-sample: {:.2}%", avg_in_sample);
    println!("Средняя прибыль out-of-sample: {:.2}%", avg_out_sample);
    println!("Прибыльных окон: {}/{} ({:.1}%)",
        profitable_windows,
        results.len(),
        profitable_windows as f64 / total_windows * 100.0
    );

    // Коэффициент эффективности: out-of-sample / in-sample
    let efficiency = (avg_out_sample / avg_in_sample) * 100.0;
    println!("Коэффициент эффективности: {:.1}%", efficiency);

    if efficiency > 50.0 {
        println!("\n✓ Стратегия выглядит устойчивой (эффективность > 50%)");
    } else {
        println!("\n✗ Стратегия может быть переоптимизирована (эффективность < 50%)");
    }
}
```

## Продвинутый подход: Anchored Walk-Forward

В **anchored walk-forward** период обучения растёт со временем вместо скольжения:

```
Anchored:   [====Обучение====][Тест]
            [======Обучение======][Тест]
            [=========Обучение=========][Тест]

vs.

Sliding:    [====Обучение====][Тест]
                [====Обучение====][Тест]
                    [====Обучение====][Тест]
```

```rust
fn anchored_walk_forward(
    prices: &[f64],
    initial_in_sample: usize,
    out_sample_size: usize,
) -> Vec<WalkForwardResult> {
    let mut results = Vec::new();
    let num_windows = (prices.len() - initial_in_sample) / out_sample_size;

    for window in 0..num_windows {
        let in_sample_end = initial_in_sample + window * out_sample_size;
        let out_sample_end = in_sample_end + out_sample_size;

        if out_sample_end > prices.len() {
            break;
        }

        // In-sample растёт от начала
        let in_sample_data = &prices[0..in_sample_end];
        let (optimized_params, in_sample_profit) = optimize_parameters(in_sample_data);

        // Out-of-sample тестирование
        let out_sample_data = &prices[in_sample_end..out_sample_end];
        let out_sample_profit = backtest_strategy(out_sample_data, &optimized_params);

        results.push(WalkForwardResult {
            window: window + 1,
            in_sample_profit,
            out_sample_profit,
            optimized_params,
        });
    }

    results
}
```

## Реалистичный торговый движок с Walk-Forward

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    entry_time: usize,
}

#[derive(Debug, Clone)]
struct MarketData {
    timestamp: usize,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

#[derive(Debug)]
struct TradingEngine {
    cash: f64,
    position: Option<Position>,
    trades: Vec<Trade>,
}

impl TradingEngine {
    fn new(initial_cash: f64) -> Self {
        TradingEngine {
            cash: initial_cash,
            position: None,
            trades: Vec::new(),
        }
    }

    fn can_buy(&self, price: f64, quantity: f64) -> bool {
        self.position.is_none() && self.cash >= price * quantity
    }

    fn buy(&mut self, symbol: &str, price: f64, quantity: f64, time: usize) -> Result<(), String> {
        if !self.can_buy(price, quantity) {
            return Err("Невозможно купить: недостаточно средств или позиция уже открыта".to_string());
        }

        let cost = price * quantity;
        self.cash -= cost;
        self.position = Some(Position {
            symbol: symbol.to_string(),
            entry_price: price,
            quantity,
            entry_time: time,
        });

        Ok(())
    }

    fn sell(&mut self, price: f64, time: usize) -> Result<f64, String> {
        let position = self.position.take()
            .ok_or("Нет позиции для продажи".to_string())?;

        let revenue = price * position.quantity;
        self.cash += revenue;

        let profit = revenue - (position.entry_price * position.quantity);
        let profit_pct = profit / (position.entry_price * position.quantity) * 100.0;

        self.trades.push(Trade {
            entry_price: position.entry_price,
            exit_price: price,
            profit: profit_pct,
        });

        Ok(profit_pct)
    }

    fn get_equity(&self, current_price: f64) -> f64 {
        let mut equity = self.cash;
        if let Some(pos) = &self.position {
            equity += pos.quantity * current_price;
        }
        equity
    }

    fn get_performance(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.trades.is_empty() {
            return stats;
        }

        let total_profit: f64 = self.trades.iter().map(|t| t.profit).sum();
        let avg_profit = total_profit / self.trades.len() as f64;
        let winning_trades = self.trades.iter().filter(|t| t.profit > 0.0).count();
        let win_rate = winning_trades as f64 / self.trades.len() as f64 * 100.0;

        stats.insert("total_trades".to_string(), self.trades.len() as f64);
        stats.insert("avg_profit".to_string(), avg_profit);
        stats.insert("win_rate".to_string(), win_rate);
        stats.insert("total_profit".to_string(), total_profit);

        stats
    }
}

fn main() {
    println!("=== Walk-Forward анализ с торговым движком ===\n");

    // Генерация рыночных данных
    let market_data: Vec<MarketData> = (0..500)
        .map(|i| {
            let base = 100.0 + i as f64 * 0.1;
            let volatility = 2.0;
            MarketData {
                timestamp: i,
                open: base + (i as f64 * 3.0).sin() * volatility,
                high: base + (i as f64 * 3.0).sin() * volatility + 0.5,
                low: base + (i as f64 * 3.0).sin() * volatility - 0.5,
                close: base + (i as f64 * 3.0 + 0.5).sin() * volatility,
            }
        })
        .collect();

    let closes: Vec<f64> = market_data.iter().map(|m| m.close).collect();

    // Запуск walk-forward
    let results = walk_forward_analysis(&closes, 200, 50);

    println!("\n=== Детальные результаты ===");
    for result in &results {
        println!("Окно {}: Параметры(быстрая={}, медленная={}, стоп={:.2})",
            result.window,
            result.optimized_params.fast_period,
            result.optimized_params.slow_period,
            result.optimized_params.stop_loss
        );
        println!("  In-sample: {:.2}%, Out-sample: {:.2}%",
            result.in_sample_profit,
            result.out_sample_profit
        );
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Walk-Forward анализ | Скользящая оптимизация и тестирование по времени |
| Обнаружение переоптимизации | Плохие результаты вне выборки выявляют переоптимизацию |
| Период In-Sample | Данные для обучения и оптимизации параметров |
| Период Out-of-Sample | Данные для тестирования устойчивости стратегии |
| Коэффициент эффективности | Отношение доходности out-of-sample к in-sample |
| Anchored WFA | Растущее окно обучения против скользящего окна |

## Домашнее задание

1. **Реализовать Rolling Walk-Forward**: Создай функцию, выполняющую walk-forward анализ с фиксированным размером скользящего окна (sliding window подход). Сравни результаты с anchored подходом.

2. **Добавить больше метрик**: Расширь `WalkForwardResult`, чтобы отслеживать:
   - Максимальную просадку в каждом окне
   - Коэффициент Шарпа
   - Количество сделок
   - Среднюю длительность сделки

3. **Тестирование мульти-стратегии**: Реализуй walk-forward анализ для нескольких стратегий (например, momentum, mean-reversion, breakout) и сравни их коэффициенты эффективности.

4. **Стабильность оптимизации**: Отслеживай, как часто параметры меняются между окнами. Вычисли "оценку стабильности", которая штрафует стратегии, требующие частой ре-оптимизации.

## Навигация

[← Предыдущий день](../289-backtesting-framework/ru.md) | [Следующий день →](../291-monte-carlo-simulation/ru.md)
