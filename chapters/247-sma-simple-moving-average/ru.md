# День 247: SMA: простая скользящая средняя

## Аналогия из трейдинга

Представь, что ты следишь за ценой Bitcoin. Каждую минуту цена прыгает: $42000, $42150, $41980, $42200... Как понять, куда движется рынок — вверх или вниз? Смотреть на каждую отдельную цену бесполезно — слишком много шума.

**Простая скользящая средняя (SMA)** — это как "сглаживающие очки" для трейдера. Она берёт последние N цен и вычисляет их среднее значение. Это помогает увидеть общую тенденцию, отфильтровав случайные колебания.

Например:
- **SMA(5)** — средняя за последние 5 периодов (быстрая, чувствительная к изменениям)
- **SMA(20)** — средняя за последние 20 периодов (медленная, более гладкая)
- **SMA(200)** — средняя за последние 200 периодов (очень медленная, показывает долгосрочный тренд)

Когда быстрая SMA пересекает медленную снизу вверх — это сигнал к покупке ("золотой крест"). Когда сверху вниз — сигнал к продаже ("крест смерти").

## Что такое SMA?

**SMA (Simple Moving Average)** — это технический индикатор, который вычисляется как простое арифметическое среднее цен за определённый период:

```
SMA = (P1 + P2 + P3 + ... + Pn) / n
```

Где:
- `P1, P2, ..., Pn` — цены за последние n периодов
- `n` — период (окно) скользящей средней

### Почему "скользящая"?

Потому что с каждым новым значением цены окно "скользит" вперёд:

```
Цены:     [100, 102, 101, 103, 105, 104, 106]
SMA(3):         [101, 102, 103, 104, 105]
                  ↑    ↑    ↑    ↑    ↑
               100+   102+  101+  103+  105+
               102+   101+  103+  105+  104+
               101    103   105   104   106
               ───    ───   ───   ───   ───
                3      3     3     3     3
```

## Базовая реализация SMA

```rust
/// Вычисляет простую скользящую среднюю для вектора цен
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let mut sma_values = Vec::with_capacity(prices.len() - period + 1);

    for i in 0..=(prices.len() - period) {
        let window = &prices[i..i + period];
        let sum: f64 = window.iter().sum();
        let average = sum / period as f64;
        sma_values.push(average);
    }

    sma_values
}

fn main() {
    // Исторические цены BTC (в долларах)
    let btc_prices = vec![
        42000.0, 42150.0, 41980.0, 42200.0, 42350.0,
        42100.0, 42400.0, 42550.0, 42300.0, 42600.0,
    ];

    println!("Цены BTC: {:?}", btc_prices);
    println!();

    // Вычисляем SMA с разными периодами
    let sma_3 = calculate_sma(&btc_prices, 3);
    let sma_5 = calculate_sma(&btc_prices, 5);

    println!("SMA(3): {:?}", sma_3);
    println!("SMA(5): {:?}", sma_5);
}
```

## Оптимизированная реализация

Базовая реализация пересчитывает сумму для каждого окна. Это неэффективно! Можно использовать "скользящую сумму":

```rust
/// Оптимизированная SMA с использованием скользящей суммы
fn calculate_sma_optimized(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let mut sma_values = Vec::with_capacity(prices.len() - period + 1);

    // Вычисляем начальную сумму первого окна
    let mut window_sum: f64 = prices[..period].iter().sum();
    sma_values.push(window_sum / period as f64);

    // Скользим по массиву, обновляя сумму
    for i in period..prices.len() {
        // Добавляем новую цену, убираем старую
        window_sum += prices[i] - prices[i - period];
        sma_values.push(window_sum / period as f64);
    }

    sma_values
}

fn main() {
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0];

    let sma_naive = calculate_sma(&prices, 3);
    let sma_optimized = calculate_sma_optimized(&prices, 3);

    println!("Наивный алгоритм:       {:?}", sma_naive);
    println!("Оптимизированный:       {:?}", sma_optimized);

    // Проверяем, что результаты совпадают
    assert_eq!(sma_naive.len(), sma_optimized.len());
    for (a, b) in sma_naive.iter().zip(sma_optimized.iter()) {
        assert!((a - b).abs() < 1e-10);
    }
    println!("Результаты идентичны!");
}

/// Базовая реализация для сравнения
fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period || period == 0 {
        return vec![];
    }

    let mut sma_values = Vec::with_capacity(prices.len() - period + 1);

    for i in 0..=(prices.len() - period) {
        let window = &prices[i..i + period];
        let sum: f64 = window.iter().sum();
        sma_values.push(sum / period as f64);
    }

    sma_values
}
```

## Структура SMA-калькулятора

Для реального использования удобно создать структуру:

```rust
/// Калькулятор простой скользящей средней
#[derive(Debug)]
struct SmaCalculator {
    period: usize,
    prices: Vec<f64>,
    current_sum: f64,
}

impl SmaCalculator {
    /// Создаёт новый калькулятор SMA с заданным периодом
    fn new(period: usize) -> Self {
        SmaCalculator {
            period,
            prices: Vec::with_capacity(period),
            current_sum: 0.0,
        }
    }

    /// Добавляет новую цену и возвращает текущее значение SMA (если доступно)
    fn add_price(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);
        self.current_sum += price;

        if self.prices.len() > self.period {
            // Убираем самую старую цену
            let old_price = self.prices.remove(0);
            self.current_sum -= old_price;
        }

        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }

    /// Возвращает текущее значение SMA без добавления новой цены
    fn current_sma(&self) -> Option<f64> {
        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }

    /// Возвращает количество цен, необходимых для расчёта SMA
    fn prices_needed(&self) -> usize {
        if self.prices.len() >= self.period {
            0
        } else {
            self.period - self.prices.len()
        }
    }
}

fn main() {
    let mut sma = SmaCalculator::new(3);

    let incoming_prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];

    println!("Получение цен в реальном времени:");
    for price in incoming_prices {
        let sma_value = sma.add_price(price);
        match sma_value {
            Some(avg) => println!("Цена: {:.2}, SMA(3): {:.2}", price, avg),
            None => println!(
                "Цена: {:.2}, SMA(3): ожидаем ещё {} значений",
                price,
                sma.prices_needed()
            ),
        }
    }
}
```

## Торговая стратегия: пересечение SMA

Одна из классических стратегий — торговля на пересечении двух SMA:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug)]
struct SmaCrossoverStrategy {
    fast_sma: SmaCalculator,
    slow_sma: SmaCalculator,
    previous_fast: Option<f64>,
    previous_slow: Option<f64>,
}

impl SmaCrossoverStrategy {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period < slow_period, "Быстрый период должен быть меньше медленного");
        SmaCrossoverStrategy {
            fast_sma: SmaCalculator::new(fast_period),
            slow_sma: SmaCalculator::new(slow_period),
            previous_fast: None,
            previous_slow: None,
        }
    }

    fn update(&mut self, price: f64) -> Signal {
        let current_fast = self.fast_sma.add_price(price);
        let current_slow = self.slow_sma.add_price(price);

        let signal = match (self.previous_fast, self.previous_slow, current_fast, current_slow) {
            (Some(prev_f), Some(prev_s), Some(curr_f), Some(curr_s)) => {
                // Золотой крест: быстрая SMA пересекает медленную снизу вверх
                if prev_f <= prev_s && curr_f > curr_s {
                    Signal::Buy
                }
                // Крест смерти: быстрая SMA пересекает медленную сверху вниз
                else if prev_f >= prev_s && curr_f < curr_s {
                    Signal::Sell
                } else {
                    Signal::Hold
                }
            }
            _ => Signal::Hold,
        };

        self.previous_fast = current_fast;
        self.previous_slow = current_slow;

        signal
    }
}

/// Калькулятор простой скользящей средней
#[derive(Debug)]
struct SmaCalculator {
    period: usize,
    prices: Vec<f64>,
    current_sum: f64,
}

impl SmaCalculator {
    fn new(period: usize) -> Self {
        SmaCalculator {
            period,
            prices: Vec::with_capacity(period),
            current_sum: 0.0,
        }
    }

    fn add_price(&mut self, price: f64) -> Option<f64> {
        self.prices.push(price);
        self.current_sum += price;

        if self.prices.len() > self.period {
            let old_price = self.prices.remove(0);
            self.current_sum -= old_price;
        }

        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }
}

fn main() {
    let mut strategy = SmaCrossoverStrategy::new(3, 5);

    // Симулируем поток цен
    let prices = vec![
        100.0, 102.0, 101.0, 103.0, 105.0,  // Накопление данных
        108.0, 110.0, 109.0,                 // Рост
        106.0, 103.0, 100.0, 98.0,           // Падение
        101.0, 104.0, 107.0,                 // Восстановление
    ];

    println!("Стратегия пересечения SMA(3)/SMA(5):");
    println!("{:>8} {:>8}", "Цена", "Сигнал");
    println!("{}", "-".repeat(20));

    for price in prices {
        let signal = strategy.update(price);
        let signal_str = match signal {
            Signal::Buy => "ПОКУПКА",
            Signal::Sell => "ПРОДАЖА",
            Signal::Hold => "-",
        };
        println!("{:>8.2} {:>8}", price, signal_str);
    }
}
```

## Использование VecDeque для эффективности

`Vec::remove(0)` — это O(n) операция. Для лучшей производительности используем `VecDeque`:

```rust
use std::collections::VecDeque;

/// Эффективный калькулятор SMA с использованием VecDeque
#[derive(Debug)]
struct EfficientSma {
    period: usize,
    prices: VecDeque<f64>,
    current_sum: f64,
}

impl EfficientSma {
    fn new(period: usize) -> Self {
        EfficientSma {
            period,
            prices: VecDeque::with_capacity(period),
            current_sum: 0.0,
        }
    }

    fn add_price(&mut self, price: f64) -> Option<f64> {
        self.prices.push_back(price);
        self.current_sum += price;

        if self.prices.len() > self.period {
            // O(1) операция благодаря VecDeque
            if let Some(old_price) = self.prices.pop_front() {
                self.current_sum -= old_price;
            }
        }

        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }

    fn current_sma(&self) -> Option<f64> {
        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }
}

fn main() {
    let mut sma = EfficientSma::new(5);

    let prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42250.0, 42400.0,
    ];

    println!("Эффективный расчёт SMA(5) для BTC:");
    for price in prices {
        match sma.add_price(price) {
            Some(avg) => println!("Цена: ${:.2} -> SMA: ${:.2}", price, avg),
            None => println!("Цена: ${:.2} -> SMA: накопление данных...", price),
        }
    }
}
```

## Практический пример: анализ портфеля

```rust
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
struct Asset {
    symbol: String,
    sma_short: EfficientSma,
    sma_long: EfficientSma,
    last_price: f64,
    position: f64, // количество актива
}

#[derive(Debug)]
struct Portfolio {
    assets: HashMap<String, Asset>,
    cash: f64,
}

#[derive(Debug)]
struct EfficientSma {
    period: usize,
    prices: VecDeque<f64>,
    current_sum: f64,
}

impl EfficientSma {
    fn new(period: usize) -> Self {
        EfficientSma {
            period,
            prices: VecDeque::with_capacity(period),
            current_sum: 0.0,
        }
    }

    fn add_price(&mut self, price: f64) -> Option<f64> {
        self.prices.push_back(price);
        self.current_sum += price;

        if self.prices.len() > self.period {
            if let Some(old_price) = self.prices.pop_front() {
                self.current_sum -= old_price;
            }
        }

        if self.prices.len() == self.period {
            Some(self.current_sum / self.period as f64)
        } else {
            None
        }
    }
}

impl Portfolio {
    fn new(initial_cash: f64) -> Self {
        Portfolio {
            assets: HashMap::new(),
            cash: initial_cash,
        }
    }

    fn add_asset(&mut self, symbol: &str, short_period: usize, long_period: usize) {
        self.assets.insert(symbol.to_string(), Asset {
            symbol: symbol.to_string(),
            sma_short: EfficientSma::new(short_period),
            sma_long: EfficientSma::new(long_period),
            last_price: 0.0,
            position: 0.0,
        });
    }

    fn update_price(&mut self, symbol: &str, price: f64) -> Option<String> {
        let asset = self.assets.get_mut(symbol)?;
        asset.last_price = price;

        let short_sma = asset.sma_short.add_price(price);
        let long_sma = asset.sma_long.add_price(price);

        match (short_sma, long_sma) {
            (Some(short), Some(long)) => {
                let trend = if short > long {
                    "ВОСХОДЯЩИЙ"
                } else if short < long {
                    "НИСХОДЯЩИЙ"
                } else {
                    "БОКОВОЙ"
                };

                Some(format!(
                    "{}: Цена=${:.2}, SMA(short)=${:.2}, SMA(long)=${:.2}, Тренд: {}",
                    symbol, price, short, long, trend
                ))
            }
            _ => Some(format!("{}: Цена=${:.2}, накопление данных...", symbol, price)),
        }
    }

    fn get_portfolio_value(&self) -> f64 {
        let positions_value: f64 = self.assets.values()
            .map(|a| a.position * a.last_price)
            .sum();
        self.cash + positions_value
    }
}

fn main() {
    let mut portfolio = Portfolio::new(100_000.0);

    // Добавляем активы с разными периодами SMA
    portfolio.add_asset("BTC", 5, 20);
    portfolio.add_asset("ETH", 5, 20);

    // Симулируем получение цен
    let btc_prices = vec![
        42000.0, 42100.0, 42050.0, 42200.0, 42150.0,
        42300.0, 42400.0, 42350.0, 42500.0, 42450.0,
        42600.0, 42700.0, 42650.0, 42800.0, 42750.0,
        42900.0, 43000.0, 42950.0, 43100.0, 43050.0,
        43200.0, 43300.0, 43250.0, 43400.0, 43350.0,
    ];

    let eth_prices = vec![
        2200.0, 2210.0, 2205.0, 2220.0, 2215.0,
        2230.0, 2240.0, 2235.0, 2250.0, 2245.0,
        2260.0, 2270.0, 2265.0, 2280.0, 2275.0,
        2290.0, 2300.0, 2295.0, 2310.0, 2305.0,
        2320.0, 2330.0, 2325.0, 2340.0, 2335.0,
    ];

    println!("=== Анализ портфеля с SMA ===\n");

    for i in 0..btc_prices.len() {
        println!("--- Тик {} ---", i + 1);

        if let Some(analysis) = portfolio.update_price("BTC", btc_prices[i]) {
            println!("{}", analysis);
        }

        if let Some(analysis) = portfolio.update_price("ETH", eth_prices[i]) {
            println!("{}", analysis);
        }

        println!();
    }

    println!("Общая стоимость портфеля: ${:.2}", portfolio.get_portfolio_value());
}
```

## Сравнение SMA с другими индикаторами

| Индикатор | Формула | Особенности |
|-----------|---------|-------------|
| SMA | (P1 + P2 + ... + Pn) / n | Простой, запаздывающий |
| EMA | α * P + (1-α) * EMA_prev | Больший вес новым данным |
| WMA | Σ(Pi * Wi) / Σ(Wi) | Линейные веса |

```rust
/// Сравнение SMA и EMA
fn main() {
    let prices = vec![
        100.0, 102.0, 104.0, 103.0, 105.0,
        107.0, 106.0, 108.0, 110.0, 109.0,
    ];

    let period = 5;
    let alpha = 2.0 / (period as f64 + 1.0); // Коэффициент сглаживания для EMA

    // Вычисляем SMA
    let sma_values = calculate_sma(&prices, period);

    // Вычисляем EMA
    let ema_values = calculate_ema(&prices, alpha);

    println!("{:>8} {:>10} {:>10}", "Цена", "SMA(5)", "EMA(5)");
    println!("{}", "-".repeat(30));

    for i in 0..prices.len() {
        let sma = if i >= period - 1 {
            format!("{:.2}", sma_values[i - period + 1])
        } else {
            "-".to_string()
        };

        let ema = format!("{:.2}", ema_values[i]);

        println!("{:>8.2} {:>10} {:>10}", prices[i], sma, ema);
    }
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![];
    }

    let mut result = Vec::new();
    for i in 0..=(prices.len() - period) {
        let sum: f64 = prices[i..i + period].iter().sum();
        result.push(sum / period as f64);
    }
    result
}

fn calculate_ema(prices: &[f64], alpha: f64) -> Vec<f64> {
    if prices.is_empty() {
        return vec![];
    }

    let mut ema_values = vec![prices[0]]; // Первое значение EMA = первой цене

    for i in 1..prices.len() {
        let ema = alpha * prices[i] + (1.0 - alpha) * ema_values[i - 1];
        ema_values.push(ema);
    }

    ema_values
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| SMA | Простая скользящая средняя — среднее арифметическое цен за период |
| Период | Количество значений для расчёта (окно скользящей средней) |
| Скользящая сумма | Оптимизация: обновляем сумму вместо полного пересчёта |
| VecDeque | Эффективная структура для реализации скользящего окна |
| Пересечение SMA | Торговая стратегия на основе двух SMA с разными периодами |
| Золотой крест | Сигнал покупки: быстрая SMA пересекает медленную снизу вверх |
| Крест смерти | Сигнал продажи: быстрая SMA пересекает медленную сверху вниз |

## Домашнее задание

1. **Множественные SMA**: Создай структуру `MultiSma`, которая одновременно отслеживает несколько SMA с разными периодами (например, 5, 10, 20, 50, 200). Реализуй метод, который возвращает текущую позицию цены относительно всех SMA.

2. **Детектор тренда**: Напиши функцию, которая анализирует историю цен и определяет силу тренда на основе расстояния между SMA(20) и SMA(50). Чем больше расстояние, тем сильнее тренд.

3. **Бэктестинг стратегии**: Реализуй простой бэктестер для стратегии пересечения SMA. Подсчитай:
   - Количество сделок
   - Общую прибыль/убыток
   - Процент выигрышных сделок
   - Максимальную просадку

4. **Адаптивная SMA**: Создай калькулятор, который автоматически подстраивает период SMA в зависимости от волатильности рынка: при высокой волатильности — короткий период, при низкой — длинный.

## Навигация

[← Предыдущий день](../246-trading-algorithms-intro/ru.md) | [Следующий день →](../248-ema-exponential-moving-average/ru.md)
