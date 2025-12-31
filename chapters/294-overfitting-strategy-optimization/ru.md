# День 294: Overfitting: переобучение стратегии

## Аналогия из трейдинга

Представь трейдера, который анализирует график биткоина за последний месяц и находит "идеальную" стратегию: покупать на каждом локальном минимуме и продавать на каждом максимуме. Он проверяет на исторических данных — прибыль 500%! Но когда применяет на реальном рынке — стратегия проваливается.

Это классический пример **переобучения (overfitting)**:
- Стратегия "заточена" под конкретный исторический период
- Она запомнила все случайные флуктуации рынка
- Вместо общих закономерностей она выучила шум
- На новых данных она бесполезна

Как студент, который зубрит конкретные задачи перед экзаменом вместо понимания принципов — он отлично решает знакомые задачи, но теряется при малейшем изменении условий.

## Почему переобучение опасно в алготрейдинге?

В машинном обучении и бэктестинге стратегий переобучение — это когда модель или алгоритм слишком точно подстраивается под обучающую выборку:

| Проблема | Причина | Последствие |
|----------|---------|-------------|
| Слишком много параметров | 50+ индикаторов, каждый настроен до сотых долей | Стратегия "запоминает" шум |
| Малая выборка данных | Тестирование на 2 неделях | Не учитывает разные рыночные условия |
| Look-ahead bias | Использование будущих данных | Невозможно воспроизвести в реальности |
| Data snooping | Перебор тысяч вариантов | Находится случайная корреляция |
| Survivorship bias | Тестирование только на живых активах | Игнорирование банкротств |

## Признаки переобучения

```rust
#[derive(Debug)]
struct BacktestResult {
    train_sharpe: f64,      // Sharpe ratio на обучающей выборке
    test_sharpe: f64,       // Sharpe ratio на тестовой выборке
    train_profit: f64,      // Прибыль на обучении
    test_profit: f64,       // Прибыль на тесте
    num_parameters: usize,  // Количество параметров стратегии
    num_trades: usize,      // Количество сделок
}

impl BacktestResult {
    /// Проверка на переобучение
    fn is_overfitted(&self) -> bool {
        // Признак 1: Sharpe ratio на тесте сильно хуже, чем на обучении
        let sharpe_degradation = (self.train_sharpe - self.test_sharpe) / self.train_sharpe;

        // Признак 2: Слишком много параметров относительно количества сделок
        let parameter_ratio = self.num_parameters as f64 / self.num_trades as f64;

        // Признак 3: Прибыль на тесте отрицательная при положительной на обучении
        let profit_reversal = self.train_profit > 0.0 && self.test_profit < 0.0;

        sharpe_degradation > 0.3 || parameter_ratio > 0.1 || profit_reversal
    }

    fn print_diagnosis(&self) {
        println!("=== Диагностика бэктеста ===");
        println!("Обучающая выборка:");
        println!("  Sharpe ratio: {:.2}", self.train_sharpe);
        println!("  Прибыль: {:.2}%", self.train_profit * 100.0);
        println!("\nТестовая выборка:");
        println!("  Sharpe ratio: {:.2}", self.test_sharpe);
        println!("  Прибыль: {:.2}%", self.test_profit * 100.0);
        println!("\nПараметры:");
        println!("  Количество параметров: {}", self.num_parameters);
        println!("  Количество сделок: {}", self.num_trades);
        println!("  Соотношение: {:.3}", self.num_parameters as f64 / self.num_trades as f64);

        if self.is_overfitted() {
            println!("\n⚠️  ВНИМАНИЕ: Обнаружены признаки переобучения!");
        } else {
            println!("\n✅ Стратегия выглядит устойчиво");
        }
    }
}

fn main() {
    // Хорошая стратегия
    let good_strategy = BacktestResult {
        train_sharpe: 1.8,
        test_sharpe: 1.6,
        train_profit: 0.35,
        test_profit: 0.28,
        num_parameters: 5,
        num_trades: 150,
    };

    good_strategy.print_diagnosis();
    println!();

    // Переобученная стратегия
    let overfitted_strategy = BacktestResult {
        train_sharpe: 3.5,
        test_sharpe: 0.8,
        train_profit: 0.85,
        test_profit: -0.12,
        num_parameters: 25,
        num_trades: 80,
    };

    overfitted_strategy.print_diagnosis();
}
```

## Пример: Оптимизация с защитой от переобучения

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct StrategyParams {
    ma_short: usize,      // Период короткой скользящей средней
    ma_long: usize,       // Период длинной скользящей средней
    stop_loss: f64,       // Стоп-лосс в процентах
    take_profit: f64,     // Тейк-профит в процентах
}

impl StrategyParams {
    fn count_params(&self) -> usize {
        4 // Количество настраиваемых параметров
    }
}

#[derive(Debug)]
struct Trade {
    entry_price: f64,
    exit_price: f64,
    pnl: f64,
}

/// Простая MA crossover стратегия
fn backtest_strategy(prices: &[f64], params: &StrategyParams) -> Vec<Trade> {
    let mut trades = Vec::new();

    if prices.len() < params.ma_long {
        return trades;
    }

    let mut position_open = false;
    let mut entry_price = 0.0;

    for i in params.ma_long..prices.len() {
        // Считаем скользящие средние
        let short_ma: f64 = prices[i - params.ma_short..i].iter().sum::<f64>()
            / params.ma_short as f64;
        let long_ma: f64 = prices[i - params.ma_long..i].iter().sum::<f64>()
            / params.ma_long as f64;

        let prev_short_ma: f64 = prices[i - params.ma_short - 1..i - 1].iter().sum::<f64>()
            / params.ma_short as f64;
        let prev_long_ma: f64 = prices[i - params.ma_long - 1..i - 1].iter().sum::<f64>()
            / params.ma_long as f64;

        // Сигнал на покупку: короткая MA пересекает длинную снизу вверх
        if !position_open && prev_short_ma <= prev_long_ma && short_ma > long_ma {
            position_open = true;
            entry_price = prices[i];
        }

        // Проверка на закрытие позиции
        if position_open {
            let current_pnl = (prices[i] - entry_price) / entry_price;

            // Стоп-лосс или тейк-профит
            if current_pnl <= -params.stop_loss || current_pnl >= params.take_profit {
                trades.push(Trade {
                    entry_price,
                    exit_price: prices[i],
                    pnl: current_pnl,
                });
                position_open = false;
            }
            // Сигнал на продажу: короткая MA пересекает длинную сверху вниз
            else if prev_short_ma >= prev_long_ma && short_ma < long_ma {
                trades.push(Trade {
                    entry_price,
                    exit_price: prices[i],
                    pnl: current_pnl,
                });
                position_open = false;
            }
        }
    }

    trades
}

/// Вычисление Sharpe ratio
fn calculate_sharpe(trades: &[Trade]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }

    let returns: Vec<f64> = trades.iter().map(|t| t.pnl).collect();
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

    let variance = returns.iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>() / returns.len() as f64;

    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return 0.0;
    }

    // Annualized Sharpe (assuming 252 trading days)
    mean_return / std_dev * (252.0_f64).sqrt()
}

/// Walk-forward анализ для защиты от переобучения
fn walk_forward_analysis(prices: &[f64], params: &StrategyParams) -> BacktestResult {
    // Разделяем данные: 70% обучение, 30% тест
    let split_point = (prices.len() as f64 * 0.7) as usize;

    let train_prices = &prices[..split_point];
    let test_prices = &prices[split_point..];

    // Бэктест на обучающей выборке
    let train_trades = backtest_strategy(train_prices, params);
    let train_sharpe = calculate_sharpe(&train_trades);
    let train_profit: f64 = train_trades.iter().map(|t| t.pnl).sum();

    // Бэктест на тестовой выборке
    let test_trades = backtest_strategy(test_prices, params);
    let test_sharpe = calculate_sharpe(&test_trades);
    let test_profit: f64 = test_trades.iter().map(|t| t.pnl).sum();

    BacktestResult {
        train_sharpe,
        test_sharpe,
        train_profit,
        test_profit,
        num_parameters: params.count_params(),
        num_trades: train_trades.len() + test_trades.len(),
    }
}

/// Генерация тестовых данных (имитация цен)
fn generate_price_data(days: usize, start_price: f64) -> Vec<f64> {
    let mut prices = Vec::with_capacity(days);
    let mut price = start_price;

    for i in 0..days {
        // Простой тренд с шумом
        let trend = (i as f64 * 0.01).sin() * 10.0;
        let noise = ((i * 7) % 17) as f64 - 8.0; // Детерминированный "шум"
        price += trend + noise;
        prices.push(price);
    }

    prices
}

fn main() {
    let prices = generate_price_data(500, 42000.0);

    println!("=== Тестирование стратегий на переобучение ===\n");

    // Простая стратегия с малым количеством параметров
    let simple_params = StrategyParams {
        ma_short: 10,
        ma_long: 50,
        stop_loss: 0.05,
        take_profit: 0.10,
    };

    println!("1. Простая стратегия (4 параметра):");
    let simple_result = walk_forward_analysis(&prices, &simple_params);
    simple_result.print_diagnosis();
    println!();

    // Сложная стратегия (имитация переобучения через тонкую настройку)
    let complex_params = StrategyParams {
        ma_short: 7,
        ma_long: 43,
        stop_loss: 0.0234,
        take_profit: 0.1567,
    };

    println!("2. Переоптимизированная стратегия (4 параметра, но слишком точная настройка):");
    println!("   (параметры подобраны слишком точно: 7, 43, 0.0234, 0.1567)");
    let complex_result = walk_forward_analysis(&prices, &complex_params);
    complex_result.print_diagnosis();
}
```

## Методы борьбы с переобучением

### 1. Cross-Validation (Кросс-валидация)

```rust
/// K-fold кросс-валидация для бэктестинга
fn k_fold_validation(prices: &[f64], params: &StrategyParams, k: usize) -> Vec<f64> {
    let fold_size = prices.len() / k;
    let mut sharpe_ratios = Vec::new();

    for i in 0..k {
        // Используем i-й fold как тест, остальные как обучение
        let test_start = i * fold_size;
        let test_end = (i + 1) * fold_size;

        let test_fold = &prices[test_start..test_end];
        let trades = backtest_strategy(test_fold, params);

        sharpe_ratios.push(calculate_sharpe(&trades));
    }

    sharpe_ratios
}

fn main() {
    let prices = generate_price_data(500, 42000.0);
    let params = StrategyParams {
        ma_short: 10,
        ma_long: 50,
        stop_loss: 0.05,
        take_profit: 0.10,
    };

    let sharpe_ratios = k_fold_validation(&prices, &params, 5);

    println!("=== 5-Fold Cross-Validation ===");
    for (i, sharpe) in sharpe_ratios.iter().enumerate() {
        println!("Fold {}: Sharpe = {:.2}", i + 1, sharpe);
    }

    let mean_sharpe = sharpe_ratios.iter().sum::<f64>() / sharpe_ratios.len() as f64;
    let std_sharpe = {
        let variance = sharpe_ratios.iter()
            .map(|s| (s - mean_sharpe).powi(2))
            .sum::<f64>() / sharpe_ratios.len() as f64;
        variance.sqrt()
    };

    println!("\nСредний Sharpe: {:.2} ± {:.2}", mean_sharpe, std_sharpe);

    if std_sharpe / mean_sharpe.abs() > 0.5 {
        println!("⚠️  Высокая вариативность — возможно переобучение!");
    } else {
        println!("✅ Стратегия стабильна на разных периодах");
    }
}
```

### 2. Регуляризация: ограничение сложности

```rust
#[derive(Debug)]
struct RegularizedStrategy {
    params: StrategyParams,
    complexity_penalty: f64,
}

impl RegularizedStrategy {
    /// Оценка с учётом штрафа за сложность (Akaike Information Criterion)
    fn calculate_aic(&self, trades: &[Trade]) -> f64 {
        let n = trades.len() as f64;
        let k = self.params.count_params() as f64;

        if n == 0.0 {
            return f64::INFINITY;
        }

        // Log-likelihood (упрощённо через среднюю прибыль)
        let mean_pnl = trades.iter().map(|t| t.pnl).sum::<f64>() / n;
        let log_likelihood = -n * mean_pnl.abs().ln();

        // AIC = 2k - 2ln(L)
        // Чем меньше AIC, тем лучше (баланс между точностью и простотой)
        2.0 * k - 2.0 * log_likelihood + self.complexity_penalty * k
    }

    fn evaluate(&self, prices: &[f64]) -> f64 {
        let trades = backtest_strategy(prices, &self.params);
        self.calculate_aic(&trades)
    }
}

fn main() {
    let prices = generate_price_data(500, 42000.0);

    let strategies = vec![
        RegularizedStrategy {
            params: StrategyParams {
                ma_short: 10,
                ma_long: 50,
                stop_loss: 0.05,
                take_profit: 0.10,
            },
            complexity_penalty: 1.0,
        },
        RegularizedStrategy {
            params: StrategyParams {
                ma_short: 7,
                ma_long: 43,
                stop_loss: 0.0234,
                take_profit: 0.1567,
            },
            complexity_penalty: 1.0,
        },
    ];

    println!("=== Сравнение стратегий с учётом сложности (AIC) ===");
    for (i, strategy) in strategies.iter().enumerate() {
        let aic = strategy.evaluate(&prices);
        println!("Стратегия {}: AIC = {:.2}", i + 1, aic);
    }
    println!("\nМеньшее значение AIC = лучше баланс точность/простота");
}
```

### 3. Out-of-Sample тестирование

```rust
/// Строгое разделение: обучение -> валидация -> тест
fn three_way_split_test(prices: &[f64], params: &StrategyParams) {
    let n = prices.len();
    let train_end = n * 50 / 100;  // 50% обучение
    let val_end = n * 75 / 100;    // 25% валидация
    // 25% тест

    let train = &prices[..train_end];
    let validation = &prices[train_end..val_end];
    let test = &prices[val_end..];

    let train_trades = backtest_strategy(train, params);
    let val_trades = backtest_strategy(validation, params);
    let test_trades = backtest_strategy(test, params);

    println!("=== Трёхстороннее разделение данных ===");
    println!("Обучение (50%):   Sharpe = {:.2}", calculate_sharpe(&train_trades));
    println!("Валидация (25%):  Sharpe = {:.2}", calculate_sharpe(&val_trades));
    println!("Тест (25%):       Sharpe = {:.2}", calculate_sharpe(&test_trades));

    let train_sharpe = calculate_sharpe(&train_trades);
    let test_sharpe = calculate_sharpe(&test_trades);

    if (train_sharpe - test_sharpe).abs() / train_sharpe > 0.3 {
        println!("\n⚠️  Производительность на тесте сильно отличается!");
    } else {
        println!("\n✅ Стратегия генерализуется хорошо");
    }
}

fn main() {
    let prices = generate_price_data(1000, 42000.0);
    let params = StrategyParams {
        ma_short: 10,
        ma_long: 50,
        stop_loss: 0.05,
        take_profit: 0.10,
    };

    three_way_split_test(&prices, &params);
}
```

## Практические рекомендации

### Правило 10:1
На каждый параметр должно быть минимум 10 сделок. Если у стратегии 5 параметров — нужно минимум 50 сделок для надёжного тестирования.

### Monte Carlo симуляция
Тестирование стратегии на тысячах случайных перестановок сделок:

```rust
use rand::seq::SliceRandom;
use rand::thread_rng;

fn monte_carlo_simulation(trades: &[Trade], iterations: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let mut results = Vec::new();

    for _ in 0..iterations {
        let mut shuffled = trades.to_vec();
        shuffled.shuffle(&mut rng);

        let total_pnl: f64 = shuffled.iter().map(|t| t.pnl).sum();
        results.push(total_pnl);
    }

    results.sort_by(|a, b| a.partial_cmp(b).unwrap());
    results
}

fn main() {
    let prices = generate_price_data(500, 42000.0);
    let params = StrategyParams {
        ma_short: 10,
        ma_long: 50,
        stop_loss: 0.05,
        take_profit: 0.10,
    };

    let trades = backtest_strategy(&prices, &params);
    let original_pnl: f64 = trades.iter().map(|t| t.pnl).sum();

    println!("=== Monte Carlo Simulation (1000 итераций) ===");
    println!("Оригинальная прибыль: {:.2}%", original_pnl * 100.0);

    let mc_results = monte_carlo_simulation(&trades, 1000);

    // 5-й и 95-й перцентили (90% confidence interval)
    let p5 = mc_results[50];
    let p95 = mc_results[950];

    println!("90% доверительный интервал: [{:.2}%, {:.2}%]", p5 * 100.0, p95 * 100.0);

    if original_pnl < p5 || original_pnl > p95 {
        println!("⚠️  Результат выходит за пределы доверительного интервала!");
        println!("    Возможно, порядок сделок имеет критическое значение (lucky streak)");
    } else {
        println!("✅ Результат в пределах нормы");
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Overfitting** | Переобучение — когда стратегия заточена под исторические данные |
| **Train/Test Split** | Разделение данных на обучающую и тестовую выборки |
| **Walk-Forward** | Последовательное тестирование на скользящем окне |
| **Cross-Validation** | K-fold валидация для оценки стабильности |
| **Sharpe Ratio** | Метрика риск-доходность для оценки стратегий |
| **AIC** | Информационный критерий с штрафом за сложность |
| **Monte Carlo** | Симуляция для оценки устойчивости результатов |
| **Правило 10:1** | Минимум 10 сделок на параметр |

## Домашнее задание

1. **Детектор переобучения**: Напиши функцию, которая:
   - Принимает результаты бэктеста
   - Проверяет 5+ признаков переобучения
   - Выдаёт оценку от 0 до 100 (риск переобучения)
   - Генерирует отчёт с рекомендациями

2. **Walk-Forward Optimizer**: Реализуй систему оптимизации:
   - Скользящее окно (6 месяцев обучение, 1 месяц тест)
   - Автоматический подбор параметров на обучающем окне
   - Тестирование на следующем месяце
   - Расчёт средней производительности по всем окнам

3. **Сравнение методов валидации**: Протестируй одну стратегию с помощью:
   - Simple train/test split (70/30)
   - K-fold cross-validation (k=5)
   - Walk-forward analysis
   - Monte Carlo simulation

   Сравни результаты и стабильность оценок.

4. **Регуляризованная оптимизация**: Создай оптимизатор параметров с:
   - Штрафом за большое количество параметров
   - Бонусом за большое количество сделок
   - Штрафом за длительные периоды без сделок
   - Балансом между прибылью и устойчивостью

## Навигация

[← Предыдущий день](../293-grid-search-parameter-sweep/ru.md) | [Следующий день →](../302-comparing-strategies/ru.md)
