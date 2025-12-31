# День 285: Метрики: Win Rate (Процент прибыльных сделок)

## Аналогия из трейдинга

Представь, что ты играешь в баскетбол и пытаешься забросить мяч в корзину. Ты сделал 100 бросков, из которых 60 попали в цель, а 40 — промахнулись. Твой процент попаданий составляет 60%. В трейдинге **Win Rate** (процент прибыльных сделок) работает точно так же — это отношение количества прибыльных сделок к общему количеству сделок.

Важно понимать:
- Win Rate = 70% означает, что 7 из 10 сделок закрылись с прибылью
- Win Rate сам по себе **не гарантирует** прибыльность стратегии
- Стратегия с Win Rate 40% может быть прибыльнее стратегии с Win Rate 80%, если средний выигрыш значительно больше среднего проигрыша

Например:
- Стратегия А: Win Rate 90%, средний выигрыш $10, средний проигрыш $100 → убыточная
- Стратегия Б: Win Rate 40%, средний выигрыш $300, средний проигрыш $100 → прибыльная

## Что такое Win Rate?

**Win Rate** (коэффициент выигрыша, процент успешных сделок) — это базовая метрика эффективности торговой стратегии, которая показывает, какая доля сделок закрылась с прибылью.

Формула:
```
Win Rate = (Количество прибыльных сделок / Общее количество сделок) × 100%
```

### Типы Win Rate

1. **Общий Win Rate** — по всем сделкам
2. **Long Win Rate** — только по длинным позициям
3. **Short Win Rate** — только по коротким позициям
4. **Win Rate по инструментам** — отдельно для каждого торгового инструмента
5. **Win Rate по временным периодам** — по дням недели, времени суток и т.д.

## Базовая реализация на Rust

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TradeResult {
    Win,
    Loss,
    BreakEven,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub side: TradeSide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradeSide {
    Long,
    Short,
}

impl Trade {
    pub fn new(symbol: &str, entry_price: f64, exit_price: f64, quantity: f64, side: TradeSide) -> Self {
        Self {
            symbol: symbol.to_string(),
            entry_price,
            exit_price,
            quantity,
            side,
        }
    }

    pub fn pnl(&self) -> f64 {
        match self.side {
            TradeSide::Long => (self.exit_price - self.entry_price) * self.quantity,
            TradeSide::Short => (self.entry_price - self.exit_price) * self.quantity,
        }
    }

    pub fn result(&self) -> TradeResult {
        let pnl = self.pnl();
        if pnl > 0.0 {
            TradeResult::Win
        } else if pnl < 0.0 {
            TradeResult::Loss
        } else {
            TradeResult::BreakEven
        }
    }
}

pub struct WinRateCalculator {
    trades: Vec<Trade>,
}

impl WinRateCalculator {
    pub fn new() -> Self {
        Self { trades: Vec::new() }
    }

    pub fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    pub fn win_rate(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }

        let winning_trades = self.trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .count();

        (winning_trades as f64 / self.trades.len() as f64) * 100.0
    }

    pub fn total_trades(&self) -> usize {
        self.trades.len()
    }

    pub fn winning_trades(&self) -> usize {
        self.trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .count()
    }

    pub fn losing_trades(&self) -> usize {
        self.trades.iter()
            .filter(|t| t.result() == TradeResult::Loss)
            .count()
    }
}

fn main() {
    let mut calculator = WinRateCalculator::new();

    // Добавляем прибыльные сделки
    calculator.add_trade(Trade::new("BTC", 40000.0, 41000.0, 1.0, TradeSide::Long));
    calculator.add_trade(Trade::new("ETH", 2000.0, 2100.0, 10.0, TradeSide::Long));
    calculator.add_trade(Trade::new("BTC", 42000.0, 41000.0, 1.0, TradeSide::Short));

    // Добавляем убыточные сделки
    calculator.add_trade(Trade::new("BTC", 40000.0, 39000.0, 1.0, TradeSide::Long));
    calculator.add_trade(Trade::new("ETH", 2000.0, 1900.0, 10.0, TradeSide::Long));

    println!("Всего сделок: {}", calculator.total_trades());
    println!("Прибыльных: {}", calculator.winning_trades());
    println!("Убыточных: {}", calculator.losing_trades());
    println!("Win Rate: {:.2}%", calculator.win_rate());
}
```

**Вывод:**
```
Всего сделок: 5
Прибыльных: 3
Убыточных: 2
Win Rate: 60.00%
```

## Продвинутая аналитика Win Rate

Часто нужно анализировать Win Rate в разрезе различных параметров:

```rust
use std::collections::HashMap;

pub struct AdvancedWinRateAnalyzer {
    trades: Vec<Trade>,
}

impl AdvancedWinRateAnalyzer {
    pub fn new(trades: Vec<Trade>) -> Self {
        Self { trades }
    }

    // Win Rate для длинных позиций
    pub fn long_win_rate(&self) -> f64 {
        let long_trades: Vec<_> = self.trades.iter()
            .filter(|t| t.side == TradeSide::Long)
            .collect();

        if long_trades.is_empty() {
            return 0.0;
        }

        let winning = long_trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .count();

        (winning as f64 / long_trades.len() as f64) * 100.0
    }

    // Win Rate для коротких позиций
    pub fn short_win_rate(&self) -> f64 {
        let short_trades: Vec<_> = self.trades.iter()
            .filter(|t| t.side == TradeSide::Short)
            .collect();

        if short_trades.is_empty() {
            return 0.0;
        }

        let winning = short_trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .count();

        (winning as f64 / short_trades.len() as f64) * 100.0
    }

    // Win Rate по каждому инструменту
    pub fn win_rate_by_symbol(&self) -> HashMap<String, f64> {
        let mut symbol_map: HashMap<String, Vec<&Trade>> = HashMap::new();

        for trade in &self.trades {
            symbol_map.entry(trade.symbol.clone())
                .or_insert_with(Vec::new)
                .push(trade);
        }

        symbol_map.into_iter()
            .map(|(symbol, trades)| {
                let winning = trades.iter()
                    .filter(|t| t.result() == TradeResult::Win)
                    .count();
                let wr = (winning as f64 / trades.len() as f64) * 100.0;
                (symbol, wr)
            })
            .collect()
    }

    // Средний размер выигрыша
    pub fn average_win(&self) -> f64 {
        let wins: Vec<_> = self.trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .collect();

        if wins.is_empty() {
            return 0.0;
        }

        let total_pnl: f64 = wins.iter().map(|t| t.pnl()).sum();
        total_pnl / wins.len() as f64
    }

    // Средний размер проигрыша
    pub fn average_loss(&self) -> f64 {
        let losses: Vec<_> = self.trades.iter()
            .filter(|t| t.result() == TradeResult::Loss)
            .collect();

        if losses.is_empty() {
            return 0.0;
        }

        let total_pnl: f64 = losses.iter().map(|t| t.pnl()).sum();
        total_pnl / losses.len() as f64
    }

    // Profit Factor = Средний выигрыш / Средний проигрыш (по абсолютной величине)
    pub fn profit_factor(&self) -> f64 {
        let avg_win = self.average_win();
        let avg_loss = self.average_loss().abs();

        if avg_loss == 0.0 {
            return f64::INFINITY;
        }

        avg_win / avg_loss
    }

    // Отчёт с полным анализом
    pub fn report(&self) {
        println!("=== Анализ Win Rate ===");
        println!("Общий Win Rate: {:.2}%", self.long_win_rate());
        println!("Long Win Rate: {:.2}%", self.long_win_rate());
        println!("Short Win Rate: {:.2}%", self.short_win_rate());
        println!();

        println!("Win Rate по инструментам:");
        for (symbol, wr) in self.win_rate_by_symbol() {
            println!("  {}: {:.2}%", symbol, wr);
        }
        println!();

        println!("Средний выигрыш: ${:.2}", self.average_win());
        println!("Средний проигрыш: ${:.2}", self.average_loss());
        println!("Profit Factor: {:.2}", self.profit_factor());
    }
}

fn main() {
    let trades = vec![
        // BTC Long trades
        Trade::new("BTC", 40000.0, 41000.0, 1.0, TradeSide::Long),
        Trade::new("BTC", 41000.0, 40500.0, 1.0, TradeSide::Long),
        Trade::new("BTC", 40500.0, 42000.0, 1.0, TradeSide::Long),

        // ETH Long trades
        Trade::new("ETH", 2000.0, 2100.0, 10.0, TradeSide::Long),
        Trade::new("ETH", 2100.0, 2050.0, 10.0, TradeSide::Long),

        // BTC Short trades
        Trade::new("BTC", 42000.0, 41000.0, 1.0, TradeSide::Short),
        Trade::new("BTC", 41000.0, 42000.0, 1.0, TradeSide::Short),
    ];

    let analyzer = AdvancedWinRateAnalyzer::new(trades);
    analyzer.report();
}
```

**Пример вывода:**
```
=== Анализ Win Rate ===
Общий Win Rate: 71.43%
Long Win Rate: 60.00%
Short Win Rate: 50.00%

Win Rate по инструментам:
  BTC: 60.00%
  ETH: 50.00%

Средний выигрыш: $1200.00
Средний проигрыш: $-750.00
Profit Factor: 1.60
```

## Win Rate в контексте риск-менеджмента

```rust
#[derive(Debug)]
pub struct RiskMetrics {
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub expectancy: f64, // Математическое ожидание на сделку
}

impl RiskMetrics {
    pub fn calculate(trades: &[Trade]) -> Self {
        let total = trades.len() as f64;
        if total == 0.0 {
            return Self {
                win_rate: 0.0,
                avg_win: 0.0,
                avg_loss: 0.0,
                profit_factor: 0.0,
                expectancy: 0.0,
            };
        }

        let wins: Vec<_> = trades.iter()
            .filter(|t| t.result() == TradeResult::Win)
            .collect();
        let losses: Vec<_> = trades.iter()
            .filter(|t| t.result() == TradeResult::Loss)
            .collect();

        let win_rate = (wins.len() as f64 / total) * 100.0;

        let avg_win = if wins.is_empty() {
            0.0
        } else {
            wins.iter().map(|t| t.pnl()).sum::<f64>() / wins.len() as f64
        };

        let avg_loss = if losses.is_empty() {
            0.0
        } else {
            losses.iter().map(|t| t.pnl()).sum::<f64>() / losses.len() as f64
        };

        let profit_factor = if avg_loss == 0.0 {
            f64::INFINITY
        } else {
            avg_win / avg_loss.abs()
        };

        // Expectancy = (Win% × Avg Win) - (Loss% × |Avg Loss|)
        let loss_rate = (losses.len() as f64 / total) * 100.0;
        let expectancy = (win_rate / 100.0 * avg_win) - (loss_rate / 100.0 * avg_loss.abs());

        Self {
            win_rate,
            avg_win,
            avg_loss,
            profit_factor,
            expectancy,
        }
    }

    pub fn is_profitable(&self) -> bool {
        self.expectancy > 0.0
    }

    pub fn required_win_rate_for_breakeven(&self) -> f64 {
        // При каком Win Rate стратегия будет в нуле?
        // Expectancy = 0
        // WR × AvgWin - (1 - WR) × |AvgLoss| = 0
        // WR × AvgWin = (1 - WR) × |AvgLoss|
        // WR × AvgWin = |AvgLoss| - WR × |AvgLoss|
        // WR × (AvgWin + |AvgLoss|) = |AvgLoss|
        // WR = |AvgLoss| / (AvgWin + |AvgLoss|)

        let avg_loss_abs = self.avg_loss.abs();
        if self.avg_win + avg_loss_abs == 0.0 {
            return 0.0;
        }

        (avg_loss_abs / (self.avg_win + avg_loss_abs)) * 100.0
    }
}

fn main() {
    let trades = vec![
        Trade::new("BTC", 40000.0, 41000.0, 1.0, TradeSide::Long),  // +1000
        Trade::new("BTC", 41000.0, 40500.0, 1.0, TradeSide::Long),  // -500
        Trade::new("BTC", 40500.0, 42000.0, 1.0, TradeSide::Long),  // +1500
        Trade::new("ETH", 2000.0, 1900.0, 10.0, TradeSide::Long),   // -1000
        Trade::new("ETH", 2000.0, 2200.0, 10.0, TradeSide::Long),   // +2000
    ];

    let metrics = RiskMetrics::calculate(&trades);

    println!("=== Метрики риска ===");
    println!("Win Rate: {:.2}%", metrics.win_rate);
    println!("Средний выигрыш: ${:.2}", metrics.avg_win);
    println!("Средний проигрыш: ${:.2}", metrics.avg_loss);
    println!("Profit Factor: {:.2}", metrics.profit_factor);
    println!("Математическое ожидание: ${:.2}", metrics.expectancy);
    println!("Прибыльна: {}", if metrics.is_profitable() { "Да" } else { "Нет" });
    println!("Требуемый Win Rate для безубыточности: {:.2}%",
             metrics.required_win_rate_for_breakeven());
}
```

## Практические задания

### Задание 1: Базовый калькулятор Win Rate
Создай структуру `WinRateTracker`, которая:
- Хранит список сделок
- Вычисляет общий Win Rate
- Выводит количество выигрышных и проигрышных сделок

```rust
// Твой код здесь
```

### Задание 2: Win Rate по временным периодам
Расширь `Trade` структуру, добавив поле `timestamp: i64`. Реализуй функцию, которая вычисляет Win Rate:
- По дням недели (понедельник, вторник и т.д.)
- По часам дня (утро, день, вечер)

```rust
// Твой код здесь
```

### Задание 3: Симулятор стратегии
Создай функцию `simulate_strategy`, которая:
- Принимает параметры: `win_rate`, `avg_win`, `avg_loss`, `num_trades`
- Генерирует случайные сделки на основе этих параметров
- Возвращает итоговую прибыль/убыток

Подсказка: используй `rand` crate для генерации случайных результатов.

```rust
// Твой код здесь
```

### Задание 4: Анализатор серий
Реализуй функцию, которая находит:
- Максимальную серию выигрышей подряд
- Максимальную серию проигрышей подряд
- Среднюю длину серий выигрышей и проигрышей

```rust
// Твой код здесь
```

## Домашнее задание

1. **Калькулятор минимального Win Rate**: Напиши функцию, которая на основе желаемого соотношения риск/прибыль вычисляет минимально необходимый Win Rate для безубыточности стратегии. Например, если risk/reward = 1:2, какой должен быть Win Rate?

2. **Win Rate Tracker с персистентностью**: Создай структуру, которая:
   - Сохраняет сделки в JSON файл
   - Загружает сделки из файла
   - Отслеживает Win Rate в реальном времени
   - Экспортирует статистику в CSV формат

3. **Backtesting с Win Rate оптимизацией**: Реализуй простой бэктестер для стратегии Moving Average Crossover:
   - Тестируй разные периоды скользящих средних (например, MA(10)/MA(20), MA(20)/MA(50))
   - Для каждой комбинации вычисляй Win Rate, Profit Factor и Expectancy
   - Найди оптимальные параметры

4. **Win Rate Dashboard**: Создай консольную панель (используя библиотеку `tui` или `crossterm`), которая:
   - Отображает Win Rate в реальном времени
   - Показывает график Win Rate за последние N сделок
   - Выводит предупреждение, если Win Rate падает ниже порогового значения
   - Отображает топ-3 лучших и худших инструментов по Win Rate

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Win Rate | Процент прибыльных сделок от общего числа сделок |
| TradeResult | Enum для классификации результатов сделок (Win/Loss/BreakEven) |
| Profit Factor | Отношение среднего выигрыша к среднему проигрышу |
| Expectancy | Математическое ожидание прибыли/убытка на сделку |
| Breakeven Win Rate | Минимальный Win Rate для безубыточности стратегии |
| Advanced Analytics | Анализ Win Rate по инструментам, направлениям, временным периодам |

## Навигация

[← Предыдущий день](../284-backtesting-basics/ru.md) | [Следующий день →](../286-profit-factor-metric/ru.md)
