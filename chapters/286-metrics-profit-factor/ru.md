# День 286: Метрики: Profit Factor

## Аналогия из трейдинга

Представьте, что вы анализируете свой торговый дневник за год. Вы выиграли в 60 сделках, заработав в сумме $15,000. Вы проиграли в 40 сделках, потеряв в сумме $5,000. Как оценить свою эффективность? Одна из простых метрик, которую используют трейдеры — это **Profit Factor** (коэффициент прибыльности) — он показывает, сколько долларов вы зарабатываете на каждый потерянный доллар.

В этом примере:
- Общая прибыль: $15,000
- Общие убытки: $5,000
- Profit Factor = $15,000 / $5,000 = 3.0

Это означает, что на каждый потерянный доллар вы заработали три доллара! Profit Factor выше 1.0 означает, что вы прибыльны. Чем выше число, тем лучше работает ваша стратегия — но остерегайтесь нереалистично высоких значений (выше 4.0), которые могут указывать на переобучение при бэктестинге.

## Что такое Profit Factor?

**Profit Factor** — одна из самых важных метрик при бэктестинге торговых стратегий. Это простое соотношение, которое отвечает на вопрос: "Сколько прибыли я получаю относительно своих убытков?"

### Формула

```
Profit Factor = Общая валовая прибыль / Общие валовые убытки
```

Где:
- **Общая валовая прибыль** = Сумма всех прибыльных сделок
- **Общие валовые убытки** = Сумма всех убыточных сделок (по модулю)

### Интерпретация

| Profit Factor | Значение |
|---------------|----------|
| < 1.0 | Убыточная стратегия (убытки превышают прибыль) |
| = 1.0 | Безубыточность (прибыль равна убыткам) |
| 1.0 - 1.5 | Незначительно прибыльная |
| 1.5 - 2.5 | Хорошая эффективность |
| 2.5 - 4.0 | Отличная эффективность |
| > 4.0 | Подозрительно высокое (возможное переобучение) |

## Простой пример: расчёт Profit Factor

```rust
fn main() {
    // Результаты торговли за месяц
    let winning_trades = vec![150.0, 200.0, 300.0, 120.0, 180.0];
    let losing_trades = vec![-80.0, -100.0, -60.0, -90.0];

    // Вычисляем общую валовую прибыль
    let total_profit: f64 = winning_trades.iter().sum();

    // Вычисляем общие валовые убытки (по модулю)
    let total_loss: f64 = losing_trades.iter()
        .map(|&x: &f64| x.abs())
        .sum();

    // Вычисляем Profit Factor
    let profit_factor = total_profit / total_loss;

    println!("Общая прибыль: ${:.2}", total_profit);
    println!("Общие убытки: ${:.2}", total_loss);
    println!("Profit Factor: {:.2}", profit_factor);

    // Интерпретация
    if profit_factor < 1.0 {
        println!("Стратегия теряет деньги!");
    } else if profit_factor >= 1.5 {
        println!("Стратегия работает хорошо!");
    } else {
        println!("Стратегия незначительно прибыльна.");
    }
}
```

Вывод:
```
Общая прибыль: $950.00
Общие убытки: $330.00
Profit Factor: 2.88
Стратегия работает хорошо!
```

## Создание структуры сделки

Построим более реалистичный пример с отдельными сделками:

```rust
#[derive(Debug, Clone)]
struct Trade {
    symbol: String,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    trade_type: TradeType,
}

#[derive(Debug, Clone, Copy)]
enum TradeType {
    Long,  // Длинная позиция (покупка)
    Short, // Короткая позиция (продажа)
}

impl Trade {
    fn new(symbol: &str, entry_price: f64, exit_price: f64, quantity: f64, trade_type: TradeType) -> Self {
        Trade {
            symbol: symbol.to_string(),
            entry_price,
            exit_price,
            quantity,
            trade_type,
        }
    }

    // Вычисляем прибыль/убыток для этой сделки
    fn pnl(&self) -> f64 {
        match self.trade_type {
            TradeType::Long => (self.exit_price - self.entry_price) * self.quantity,
            TradeType::Short => (self.entry_price - self.exit_price) * self.quantity,
        }
    }

    fn is_winner(&self) -> bool {
        self.pnl() > 0.0
    }
}

fn main() {
    let trades = vec![
        Trade::new("BTC", 40000.0, 42000.0, 1.0, TradeType::Long),   // +2000
        Trade::new("ETH", 2500.0, 2400.0, 10.0, TradeType::Long),    // -1000
        Trade::new("BTC", 41000.0, 43000.0, 0.5, TradeType::Long),   // +1000
        Trade::new("SOL", 100.0, 95.0, 20.0, TradeType::Long),       // -100
        Trade::new("BTC", 44000.0, 42000.0, 1.5, TradeType::Short),  // +3000
    ];

    let total_profit: f64 = trades.iter()
        .filter(|t| t.is_winner())
        .map(|t| t.pnl())
        .sum();

    let total_loss: f64 = trades.iter()
        .filter(|t| !t.is_winner())
        .map(|t| t.pnl().abs())
        .sum();

    let profit_factor = if total_loss > 0.0 {
        total_profit / total_loss
    } else {
        f64::INFINITY // Все сделки были прибыльными!
    };

    println!("Общая прибыль: ${:.2}", total_profit);
    println!("Общие убытки: ${:.2}", total_loss);
    println!("Profit Factor: {:.2}", profit_factor);
    println!("\nРазбивка по сделкам:");
    for (i, trade) in trades.iter().enumerate() {
        println!("  Сделка {}: {} {:?} - PnL: ${:.2}",
            i + 1,
            trade.symbol,
            trade.trade_type,
            trade.pnl()
        );
    }
}
```

## Построение фреймворка для бэктестинга

Создадим простую систему бэктестинга, которая вычисляет Profit Factor:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct BacktestResult {
    trades: Vec<Trade>,
    metrics: HashMap<String, f64>,
}

impl BacktestResult {
    fn new(trades: Vec<Trade>) -> Self {
        let mut result = BacktestResult {
            trades,
            metrics: HashMap::new(),
        };
        result.calculate_metrics();
        result
    }

    fn calculate_metrics(&mut self) {
        let total_profit: f64 = self.trades.iter()
            .filter(|t| t.is_winner())
            .map(|t| t.pnl())
            .sum();

        let total_loss: f64 = self.trades.iter()
            .filter(|t| !t.is_winner())
            .map(|t| t.pnl().abs())
            .sum();

        let profit_factor = if total_loss > 0.0 {
            total_profit / total_loss
        } else {
            f64::INFINITY
        };

        let win_count = self.trades.iter().filter(|t| t.is_winner()).count();
        let total_count = self.trades.len();
        let win_rate = win_count as f64 / total_count as f64;

        let net_profit = total_profit - total_loss;

        self.metrics.insert("total_profit".to_string(), total_profit);
        self.metrics.insert("total_loss".to_string(), total_loss);
        self.metrics.insert("profit_factor".to_string(), profit_factor);
        self.metrics.insert("win_rate".to_string(), win_rate);
        self.metrics.insert("net_profit".to_string(), net_profit);
        self.metrics.insert("total_trades".to_string(), total_count as f64);
    }

    fn print_report(&self) {
        println!("=== Отчёт по бэктестингу ===");
        println!("Всего сделок: {:.0}", self.metrics.get("total_trades").unwrap_or(&0.0));
        println!("Процент побед: {:.2}%", self.metrics.get("win_rate").unwrap_or(&0.0) * 100.0);
        println!("Общая прибыль: ${:.2}", self.metrics.get("total_profit").unwrap_or(&0.0));
        println!("Общие убытки: ${:.2}", self.metrics.get("total_loss").unwrap_or(&0.0));
        println!("Чистая прибыль: ${:.2}", self.metrics.get("net_profit").unwrap_or(&0.0));
        println!("Profit Factor: {:.2}", self.metrics.get("profit_factor").unwrap_or(&0.0));

        let pf = self.metrics.get("profit_factor").unwrap_or(&0.0);
        println!("\nИнтерпретация:");
        if *pf < 1.0 {
            println!("  ⚠️  Стратегия теряет деньги!");
        } else if *pf < 1.5 {
            println!("  ⚡ Стратегия незначительно прибыльна.");
        } else if *pf < 2.5 {
            println!("  ✅ Хорошая эффективность!");
        } else if *pf <= 4.0 {
            println!("  🎯 Отличная эффективность!");
        } else {
            println!("  ⚠️  Подозрительно высокое значение - проверьте на переобучение!");
        }
    }
}

fn main() {
    // Симулируем торговую стратегию
    let trades = vec![
        Trade::new("BTC", 40000.0, 42000.0, 1.0, TradeType::Long),
        Trade::new("ETH", 2500.0, 2400.0, 10.0, TradeType::Long),
        Trade::new("BTC", 41000.0, 43000.0, 0.5, TradeType::Long),
        Trade::new("SOL", 100.0, 95.0, 20.0, TradeType::Long),
        Trade::new("BTC", 44000.0, 42000.0, 1.5, TradeType::Short),
        Trade::new("ETH", 2600.0, 2800.0, 5.0, TradeType::Long),
        Trade::new("BTC", 43000.0, 41000.0, 2.0, TradeType::Short),
    ];

    let result = BacktestResult::new(trades);
    result.print_report();
}
```

## Сравнение стратегий

Сравним две разные торговые стратегии используя Profit Factor:

```rust
struct Strategy {
    name: String,
    trades: Vec<Trade>,
}

impl Strategy {
    fn new(name: &str) -> Self {
        Strategy {
            name: name.to_string(),
            trades: Vec::new(),
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    fn calculate_profit_factor(&self) -> f64 {
        let total_profit: f64 = self.trades.iter()
            .filter(|t| t.is_winner())
            .map(|t| t.pnl())
            .sum();

        let total_loss: f64 = self.trades.iter()
            .filter(|t| !t.is_winner())
            .map(|t| t.pnl().abs())
            .sum();

        if total_loss > 0.0 {
            total_profit / total_loss
        } else {
            f64::INFINITY
        }
    }

    fn print_summary(&self) {
        println!("\n{}", "=".repeat(50));
        println!("Стратегия: {}", self.name);
        println!("{}", "=".repeat(50));

        let result = BacktestResult::new(self.trades.clone());
        result.print_report();
    }
}

fn main() {
    // Стратегия 1: Агрессивная (высокий риск, высокая награда)
    let mut aggressive = Strategy::new("Агрессивный моментум");
    aggressive.add_trade(Trade::new("BTC", 40000.0, 45000.0, 2.0, TradeType::Long));  // +10000
    aggressive.add_trade(Trade::new("BTC", 45000.0, 42000.0, 2.0, TradeType::Long));  // -6000
    aggressive.add_trade(Trade::new("BTC", 42000.0, 48000.0, 1.5, TradeType::Long));  // +9000
    aggressive.add_trade(Trade::new("BTC", 48000.0, 44000.0, 1.5, TradeType::Long));  // -6000

    // Стратегия 2: Консервативная (низкий риск, стабильная прибыль)
    let mut conservative = Strategy::new("Консервативный возврат к среднему");
    conservative.add_trade(Trade::new("BTC", 40000.0, 41000.0, 1.0, TradeType::Long));  // +1000
    conservative.add_trade(Trade::new("BTC", 41000.0, 40500.0, 1.0, TradeType::Long));  // -500
    conservative.add_trade(Trade::new("BTC", 40500.0, 41500.0, 1.0, TradeType::Long));  // +1000
    conservative.add_trade(Trade::new("BTC", 41500.0, 41200.0, 1.0, TradeType::Long));  // -300
    conservative.add_trade(Trade::new("BTC", 41200.0, 42000.0, 1.0, TradeType::Long));  // +800
    conservative.add_trade(Trade::new("BTC", 42000.0, 41700.0, 1.0, TradeType::Long));  // -300

    aggressive.print_summary();
    conservative.print_summary();

    // Сравнение
    println!("\n{}", "=".repeat(50));
    println!("СРАВНЕНИЕ");
    println!("{}", "=".repeat(50));
    println!("Агрессивная PF: {:.2}", aggressive.calculate_profit_factor());
    println!("Консервативная PF: {:.2}", conservative.calculate_profit_factor());
}
```

## Продвинутый уровень: Profit Factor во времени

Отслеживайте, как Profit Factor изменяется по мере выполнения новых сделок:

```rust
struct PerformanceTracker {
    trades: Vec<Trade>,
}

impl PerformanceTracker {
    fn new() -> Self {
        PerformanceTracker {
            trades: Vec::new(),
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
        self.print_current_pf();
    }

    fn calculate_profit_factor_at(&self, end_index: usize) -> f64 {
        let trades_slice = &self.trades[0..=end_index];

        let total_profit: f64 = trades_slice.iter()
            .filter(|t| t.is_winner())
            .map(|t| t.pnl())
            .sum();

        let total_loss: f64 = trades_slice.iter()
            .filter(|t| !t.is_winner())
            .map(|t| t.pnl().abs())
            .sum();

        if total_loss > 0.0 {
            total_profit / total_loss
        } else {
            f64::INFINITY
        }
    }

    fn print_current_pf(&self) {
        if !self.trades.is_empty() {
            let pf = self.calculate_profit_factor_at(self.trades.len() - 1);
            println!("После {} сделок: PF = {:.2}", self.trades.len(), pf);
        }
    }

    fn plot_pf_curve(&self) {
        println!("\nЭволюция Profit Factor:");
        println!("{}", "=".repeat(60));

        for i in 0..self.trades.len() {
            let pf = self.calculate_profit_factor_at(i);
            let bar_length = if pf.is_finite() {
                (pf * 10.0).min(50.0) as usize
            } else {
                50
            };
            let bar = "█".repeat(bar_length);
            println!("Сделка {:2}: {:.2} | {}", i + 1, pf, bar);
        }
    }
}

fn main() {
    let mut tracker = PerformanceTracker::new();

    println!("Начинаем торговую сессию...\n");

    tracker.add_trade(Trade::new("BTC", 40000.0, 42000.0, 1.0, TradeType::Long));
    tracker.add_trade(Trade::new("ETH", 2500.0, 2400.0, 10.0, TradeType::Long));
    tracker.add_trade(Trade::new("BTC", 41000.0, 43000.0, 0.5, TradeType::Long));
    tracker.add_trade(Trade::new("SOL", 100.0, 95.0, 20.0, TradeType::Long));
    tracker.add_trade(Trade::new("BTC", 44000.0, 42000.0, 1.5, TradeType::Short));

    tracker.plot_pf_curve();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Profit Factor | Отношение общей прибыли к общим убыткам |
| Формула | `Общая валовая прибыль / Общие валовые убытки` |
| Интерпретация | > 1.0 = прибыльно, < 1.0 = убыточно |
| Хороший диапазон | 1.5 - 4.0 для реалистичных стратегий |
| Предупреждение о переобучении | PF > 4.0 может указывать на подгонку под кривую |
| Бэктестинг | Ключевая метрика для оценки стратегии |

## Практические упражнения

1. **Процент побед против Profit Factor**: Создайте две стратегии:
   - Стратегия А: 70% побед, средняя прибыль $100, средний убыток $200
   - Стратегия Б: 40% побед, средняя прибыль $400, средний убыток $100

   Вычислите Profit Factor для каждой. Какая лучше?

2. **Анализатор торгового дневника**: Напишите программу, которая:
   - Читает сделки из CSV файла (символ, вход, выход, количество, тип)
   - Вычисляет Profit Factor
   - Показывает разбивку по символам
   - Определяет, какие активы имеют лучший PF

3. **Месячная эффективность**: Отслеживайте сделки за несколько месяцев и вычислите:
   - Profit Factor за каждый месяц
   - Общий Profit Factor
   - Определите лучшие и худшие месяцы

4. **Risk-Adjusted PF**: Расширьте вычисление Profit Factor, включив:
   - Максимальную просадку
   - Коэффициент Шарпа
   - Создайте комбинированную оценку, сочетающую PF с другими метриками

## Домашнее задание

1. **Полная система бэктестинга**: Постройте систему бэктестинга, которая:
   - Симулирует 100 случайных сделок с реалистичными движениями цен
   - Вычисляет Profit Factor, процент побед и чистую прибыль
   - Генерирует отчёт, показывающий дневной, недельный и месячный PF
   - Предупреждает, если PF подозрительно высокий (> 4.0)

2. **Симуляция Монте-Карло**:
   - Возьмите стратегию с известным PF (например, 2.0)
   - Запустите 1000 симуляций со случайным порядком сделок
   - Вычислите распределение возможных значений PF
   - Постройте гистограмму результатов

3. **Мультиактивный портфель**: Создайте трекер портфеля, который:
   - Отслеживает сделки по BTC, ETH и SOL
   - Вычисляет Profit Factor для каждого актива отдельно
   - Вычисляет общий Profit Factor портфеля
   - Показывает, какой актив больше всего способствует общему PF

4. **Адаптивная стратегия**: Реализуйте стратегию, которая:
   - Следит за своим собственным Profit Factor в реальном времени
   - Если PF падает ниже 1.5, уменьшает размер позиции на 50%
   - Если PF поднимается выше 2.5, увеличивает размер позиции на 25%
   - Логирует все корректировки размера позиции

## Навигация

[← Предыдущий день](../285-previous-chapter/ru.md) | [Следующий день →](../287-next-chapter/ru.md)
