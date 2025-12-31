# День 300: Stress testing: экстремальные условия

## Аналогия из трейдинга

Представь трейдера, который разработал отличную стратегию для обычных рыночных условий — она стабильно приносит 2-3% в месяц. Но вот наступает "черный лебедь": внезапный обвал рынка на 40% за один день, как это было в марте 2020 года.

Что происходит со стратегией?
- Стоп-лоссы не срабатывают из-за гэпов
- Ликвидность испаряется — невозможно закрыть позиции
- Кредитное плечо приводит к маржин-коллу
- За один день теряется весь капитал

Это классическая проблема: **стратегия не тестировалась на экстремальных условиях**. Она работала в "солнечную погоду", но не была готова к шторму.

**Stress testing** — это проверка торговой системы на устойчивость к экстремальным рыночным событиям:
- Обвалы и резкие всплески цен
- Периоды нулевой ликвидности
- Технические сбои биржи
- Флеш-крэши
- Череда убыточных сделок

## Зачем нужен stress testing?

В бэктестинге обычно тестируются "нормальные" условия — средняя волатильность, обычные объемы торгов. Но реальные рынки периодически сходят с ума:

| Событие | Дата | Что произошло |
|---------|------|---------------|
| Black Monday | 19.10.1987 | Dow Jones упал на 22% за день |
| Flash Crash | 06.05.2010 | Индексы обвалились на 9% за 5 минут |
| Swiss Franc Shock | 15.01.2015 | CHF вырос на 30% за минуты |
| COVID Crash | 12.03.2020 | S&P 500 упал на 12% за день |
| GameStop Short Squeeze | 28.01.2021 | GME вырос на 1900% за 2 недели |

Стресс-тестирование отвечает на вопросы:
- Выдержит ли стратегия обвал рынка на 30%?
- Что будет, если волатильность увеличится в 10 раз?
- Сколько подряд убыточных сделок может произойти?
- Какой максимальный убыток возможен?

## Виды стресс-тестов

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct MarketConditions {
    volatility_multiplier: f64,  // Множитель волатильности (1.0 = норма, 5.0 = стресс)
    liquidity_reduction: f64,    // Снижение ликвидности (0.0 = норма, 0.9 = 90% просадка)
    gap_probability: f64,        // Вероятность гэпов (0.0 - 1.0)
    max_drawdown: f64,           // Максимальная просадка рынка
}

impl MarketConditions {
    fn normal() -> Self {
        Self {
            volatility_multiplier: 1.0,
            liquidity_reduction: 0.0,
            gap_probability: 0.01,
            max_drawdown: 0.20,
        }
    }

    fn stress_crash() -> Self {
        Self {
            volatility_multiplier: 10.0,
            liquidity_reduction: 0.8,
            gap_probability: 0.3,
            max_drawdown: 0.50,
        }
    }

    fn stress_low_volatility() -> Self {
        Self {
            volatility_multiplier: 0.1,
            liquidity_reduction: 0.5,
            gap_probability: 0.0,
            max_drawdown: 0.05,
        }
    }

    fn stress_flash_crash() -> Self {
        Self {
            volatility_multiplier: 20.0,
            liquidity_reduction: 0.95,
            gap_probability: 0.5,
            max_drawdown: 0.30,
        }
    }
}

#[derive(Debug)]
struct StressTestResult {
    scenario: String,
    max_loss: f64,
    max_drawdown: f64,
    trades_executed: usize,
    trades_slipped: usize,
    final_balance: f64,
    survived: bool,
}

impl StressTestResult {
    fn print(&self) {
        println!("=== Сценарий: {} ===", self.scenario);
        println!("Максимальный убыток: {:.2}%", self.max_loss * 100.0);
        println!("Максимальная просадка: {:.2}%", self.max_drawdown * 100.0);
        println!("Сделок выполнено: {}", self.trades_executed);
        println!("Сделок с проскальзыванием: {}", self.trades_slipped);
        println!("Итоговый баланс: ${:.2}", self.final_balance);

        if self.survived {
            println!("✅ Стратегия выжила");
        } else {
            println!("❌ Маржин-колл / Полная потеря капитала");
        }
    }
}

fn main() {
    println!("=== Стресс-тестирование торговой стратегии ===\n");

    let scenarios = vec![
        ("Нормальные условия", MarketConditions::normal()),
        ("Обвал рынка", MarketConditions::stress_crash()),
        ("Низкая волатильность", MarketConditions::stress_low_volatility()),
        ("Флеш-крэш", MarketConditions::stress_flash_crash()),
    ];

    for (name, conditions) in scenarios {
        println!("Условия: {:?}", conditions);
        println!();
    }
}
```

## Пример 1: Симуляция экстремальных цен

```rust
#[derive(Debug, Clone)]
struct PriceBar {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

/// Генератор цен с экстремальными условиями
struct StressDataGenerator {
    base_price: f64,
    base_volatility: f64,
}

impl StressDataGenerator {
    fn new(base_price: f64) -> Self {
        Self {
            base_price,
            base_volatility: 0.02, // 2% обычная волатильность
        }
    }

    /// Генерация нормальных цен
    fn generate_normal(&self, bars: usize) -> Vec<PriceBar> {
        let mut data = Vec::new();
        let mut price = self.base_price;

        for i in 0..bars {
            let change = self.deterministic_noise(i) * self.base_volatility;
            price *= 1.0 + change;

            let bar = PriceBar {
                timestamp: i as i64,
                open: price,
                high: price * 1.005,
                low: price * 0.995,
                close: price,
                volume: 1000000.0,
            };

            data.push(bar);
        }

        data
    }

    /// Генерация цен с крэшем (резкое падение)
    fn generate_crash(&self, bars: usize, crash_at: usize, crash_magnitude: f64) -> Vec<PriceBar> {
        let mut data = Vec::new();
        let mut price = self.base_price;

        for i in 0..bars {
            // В момент краша — резкое падение
            if i == crash_at {
                price *= 1.0 - crash_magnitude; // Падение на crash_magnitude %
            } else {
                let change = self.deterministic_noise(i) * self.base_volatility;
                price *= 1.0 + change;
            }

            // В момент краша — большой гэп между open и close
            let (open, close) = if i == crash_at {
                let prev_close = if i > 0 { data[i-1].close } else { price };
                (prev_close, price) // Гэп вниз
            } else {
                (price, price)
            };

            let bar = PriceBar {
                timestamp: i as i64,
                open,
                high: open.max(close) * 1.002,
                low: open.min(close) * 0.998,
                close,
                volume: if i == crash_at { 10000000.0 } else { 1000000.0 },
            };

            data.push(bar);
        }

        data
    }

    /// Генерация цен с флеш-крэшем (падение и быстрое восстановление)
    fn generate_flash_crash(&self, bars: usize, crash_at: usize) -> Vec<PriceBar> {
        let mut data = Vec::new();
        let mut price = self.base_price;

        for i in 0..bars {
            if i == crash_at {
                // Резкое падение на 20%
                price *= 0.80;
            } else if i == crash_at + 1 {
                // Быстрое восстановление на 15%
                price *= 1.15;
            } else {
                let change = self.deterministic_noise(i) * self.base_volatility;
                price *= 1.0 + change;
            }

            let bar = PriceBar {
                timestamp: i as i64,
                open: price,
                high: price * 1.005,
                low: price * 0.995,
                close: price,
                volume: if i == crash_at || i == crash_at + 1 { 50000000.0 } else { 1000000.0 },
            };

            data.push(bar);
        }

        data
    }

    /// Детерминированный шум (для воспроизводимости)
    fn deterministic_noise(&self, seed: usize) -> f64 {
        let x = ((seed * 1103515245 + 12345) % 2147483648) as f64;
        (x / 2147483648.0) * 2.0 - 1.0 // Диапазон [-1, 1]
    }
}

fn main() {
    let generator = StressDataGenerator::new(50000.0);

    println!("=== Генерация экстремальных рыночных данных ===\n");

    // Сценарий 1: Обычные условия
    let normal_data = generator.generate_normal(100);
    println!("1. Нормальные условия:");
    println!("   Начальная цена: ${:.2}", normal_data.first().unwrap().close);
    println!("   Конечная цена: ${:.2}", normal_data.last().unwrap().close);
    println!();

    // Сценарий 2: Крэш на 40%
    let crash_data = generator.generate_crash(100, 50, 0.40);
    println!("2. Крэш на 40% (бар 50):");
    println!("   Цена до краша: ${:.2}", crash_data[49].close);
    println!("   Цена после краша: ${:.2}", crash_data[50].close);
    println!("   Падение: {:.2}%", (crash_data[49].close - crash_data[50].close) / crash_data[49].close * 100.0);
    println!();

    // Сценарий 3: Флеш-крэш
    let flash_data = generator.generate_flash_crash(100, 50);
    println!("3. Флеш-крэш (бар 50):");
    println!("   Цена до: ${:.2}", flash_data[49].close);
    println!("   Цена в момент краша: ${:.2}", flash_data[50].close);
    println!("   Цена после восстановления: ${:.2}", flash_data[51].close);
    println!();
}
```

## Пример 2: Тестирование стоп-лоссов на гэпах

```rust
#[derive(Debug, Clone)]
struct Position {
    entry_price: f64,
    quantity: f64,
    stop_loss: f64,
    take_profit: f64,
}

impl Position {
    fn new(entry_price: f64, quantity: f64, stop_loss_pct: f64, take_profit_pct: f64) -> Self {
        Self {
            entry_price,
            quantity,
            stop_loss: entry_price * (1.0 - stop_loss_pct),
            take_profit: entry_price * (1.0 + take_profit_pct),
        }
    }

    /// Проверка закрытия позиции (с учётом гэпов)
    fn check_exit(&self, bar: &PriceBar) -> Option<(f64, &str)> {
        // Если открытие гэпом ниже стоп-лосса — выходим по open, а не по stop_loss
        if bar.open < self.stop_loss {
            return Some((bar.open, "Gap Stop-Loss"));
        }

        // Если low достиг стоп-лосса — выходим по стоп-лоссу
        if bar.low <= self.stop_loss {
            return Some((self.stop_loss, "Stop-Loss"));
        }

        // Если open гэпом выше тейк-профита
        if bar.open > self.take_profit {
            return Some((bar.open, "Gap Take-Profit"));
        }

        // Если high достиг тейк-профита
        if bar.high >= self.take_profit {
            return Some((self.take_profit, "Take-Profit"));
        }

        None
    }

    fn calculate_pnl(&self, exit_price: f64) -> f64 {
        (exit_price - self.entry_price) * self.quantity
    }
}

fn test_stop_loss_on_gaps() {
    let generator = StressDataGenerator::new(50000.0);

    println!("=== Тест стоп-лоссов на гэпах ===\n");

    // Тест 1: Нормальные условия
    println!("1. Нормальные условия (без гэпов):");
    let normal_data = generator.generate_normal(100);
    let position = Position::new(50000.0, 1.0, 0.05, 0.10);

    for (i, bar) in normal_data.iter().enumerate() {
        if let Some((exit_price, reason)) = position.check_exit(bar) {
            let pnl = position.calculate_pnl(exit_price);
            println!("   Закрытие на баре {}: {} по цене ${:.2}", i, reason, exit_price);
            println!("   P&L: ${:.2}", pnl);
            break;
        }
    }
    println!();

    // Тест 2: Крэш с гэпом
    println!("2. Крэш с гэпом (падение 40%):");
    let crash_data = generator.generate_crash(100, 10, 0.40);
    let position = Position::new(crash_data[5].close, 1.0, 0.05, 0.10);

    println!("   Вход по цене: ${:.2}", position.entry_price);
    println!("   Стоп-лосс установлен на: ${:.2}", position.stop_loss);

    for (i, bar) in crash_data.iter().skip(6).enumerate() {
        let actual_i = i + 6;
        if let Some((exit_price, reason)) = position.check_exit(bar) {
            let pnl = position.calculate_pnl(exit_price);
            let expected_loss = position.entry_price * 0.05;
            let actual_loss = -pnl;

            println!("   Закрытие на баре {}: {}", actual_i, reason);
            println!("   Цена выхода: ${:.2}", exit_price);
            println!("   Ожидаемый убыток: ${:.2} (5%)", expected_loss);
            println!("   Реальный убыток: ${:.2} ({:.1}%)", actual_loss, actual_loss / position.entry_price * 100.0);

            if actual_loss > expected_loss * 1.5 {
                println!("   ⚠️ ВНИМАНИЕ: Убыток в {:.1}x раз больше ожидаемого из-за гэпа!",
                         actual_loss / expected_loss);
            }
            break;
        }
    }
    println!();

    // Тест 3: Флеш-крэш
    println!("3. Флеш-крэш:");
    let flash_data = generator.generate_flash_crash(100, 10);
    let position = Position::new(flash_data[5].close, 1.0, 0.05, 0.10);

    println!("   Вход по цене: ${:.2}", position.entry_price);

    for (i, bar) in flash_data.iter().skip(6).enumerate() {
        let actual_i = i + 6;
        if let Some((exit_price, reason)) = position.check_exit(bar) {
            let pnl = position.calculate_pnl(exit_price);
            println!("   Закрытие на баре {}: {} по цене ${:.2}", actual_i, reason, exit_price);
            println!("   P&L: ${:.2}", pnl);
            break;
        }
    }
}

fn main() {
    test_stop_loss_on_gaps();
}
```

## Пример 3: Максимальная просадка и серия убытков

```rust
#[derive(Debug)]
struct TradingAccount {
    initial_balance: f64,
    current_balance: f64,
    peak_balance: f64,
    max_drawdown: f64,
    consecutive_losses: usize,
    max_consecutive_losses: usize,
}

impl TradingAccount {
    fn new(initial_balance: f64) -> Self {
        Self {
            initial_balance,
            current_balance: initial_balance,
            peak_balance: initial_balance,
            max_drawdown: 0.0,
            consecutive_losses: 0,
            max_consecutive_losses: 0,
        }
    }

    fn execute_trade(&mut self, pnl: f64) {
        self.current_balance += pnl;

        // Обновляем пик
        if self.current_balance > self.peak_balance {
            self.peak_balance = self.current_balance;
            self.consecutive_losses = 0; // Сброс серии убытков
        }

        // Вычисляем текущую просадку
        let current_drawdown = (self.peak_balance - self.current_balance) / self.peak_balance;

        if current_drawdown > self.max_drawdown {
            self.max_drawdown = current_drawdown;
        }

        // Отслеживаем серию убытков
        if pnl < 0.0 {
            self.consecutive_losses += 1;
            if self.consecutive_losses > self.max_consecutive_losses {
                self.max_consecutive_losses = self.consecutive_losses;
            }
        } else {
            self.consecutive_losses = 0;
        }
    }

    fn is_margin_call(&self, margin_call_level: f64) -> bool {
        let drawdown = (self.initial_balance - self.current_balance) / self.initial_balance;
        drawdown >= margin_call_level
    }

    fn print_stats(&self) {
        println!("=== Статистика счёта ===");
        println!("Начальный баланс: ${:.2}", self.initial_balance);
        println!("Текущий баланс: ${:.2}", self.current_balance);
        println!("Пиковый баланс: ${:.2}", self.peak_balance);
        println!("Максимальная просадка: {:.2}%", self.max_drawdown * 100.0);
        println!("Максимальная серия убытков: {}", self.max_consecutive_losses);
        println!("P&L: ${:.2} ({:.2}%)",
                 self.current_balance - self.initial_balance,
                 (self.current_balance - self.initial_balance) / self.initial_balance * 100.0);
    }
}

/// Симуляция торговли с разными условиями
fn simulate_extreme_losing_streak(num_trades: usize, win_rate: f64, avg_win: f64, avg_loss: f64) -> TradingAccount {
    let mut account = TradingAccount::new(10000.0);

    for i in 0..num_trades {
        // Детерминированная "случайность" для воспроизводимости
        let rand = ((i * 1103515245 + 12345) % 100) as f64 / 100.0;

        let pnl = if rand < win_rate {
            avg_win
        } else {
            -avg_loss
        };

        account.execute_trade(pnl);

        if account.is_margin_call(0.50) {
            println!("⚠️ Маржин-колл на сделке {}", i + 1);
            break;
        }
    }

    account
}

fn main() {
    println!("=== Стресс-тест: Максимальные серии убытков ===\n");

    // Сценарий 1: Нормальная стратегия (60% винрейт)
    println!("1. Нормальная стратегия (60% винрейт, 1:1.5 риск/профит):");
    let normal = simulate_extreme_losing_streak(1000, 0.60, 150.0, 100.0);
    normal.print_stats();
    println!();

    // Сценарий 2: Серия неудач (30% винрейт на 200 сделках)
    println!("2. Экстремальная серия неудач (30% винрейт):");
    let bad_streak = simulate_extreme_losing_streak(200, 0.30, 150.0, 100.0);
    bad_streak.print_stats();
    println!();

    // Сценарий 3: Большие убытки (плохой риск-менеджмент)
    println!("3. Большие убытки (плохой риск-менеджмент, 50% винрейт, но 1:0.5 риск/профит):");
    let bad_risk = simulate_extreme_losing_streak(500, 0.50, 100.0, 200.0);
    bad_risk.print_stats();
    println!();
}
```

## Практические рекомендации по стресс-тестированию

### 1. Исторические сценарии

Тестируйте стратегию на реальных исторических событиях:

```rust
fn test_historical_scenarios(strategy: &Strategy) {
    let scenarios = vec![
        ("Black Monday 1987", -0.22, "1987-10-19"),
        ("Flash Crash 2010", -0.09, "2010-05-06"),
        ("COVID Crash 2020", -0.12, "2020-03-12"),
    ];

    for (name, drop, date) in scenarios {
        println!("Тестирование: {}", name);
        // Загрузить данные за эту дату и запустить стратегию
        // test_strategy_on_date(strategy, date, drop);
    }
}
```

### 2. Синтетические экстремумы

Создавайте искусственные, но правдоподобные экстремальные условия:

- Волатильность × 10
- Объём торгов × 0.1 (низкая ликвидность)
- Серия из 20 убыточных сделок подряд
- Гэп в 15% между барами

### 3. Limits Testing

Проверяйте граничные значения:

```rust
fn test_position_limits() {
    // Что если размер позиции = 0?
    // Что если размер позиции = MAX?
    // Что если баланс = 0?
    // Что если цена = 0.00000001?
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Stress Testing** | Проверка стратегии на экстремальные условия |
| **Flash Crash** | Резкое падение цен с быстрым восстановлением |
| **Gap Risk** | Риск гэпов, когда стоп-лосс не сработает по заданной цене |
| **Maximum Drawdown** | Максимальная просадка от пика до минимума |
| **Consecutive Losses** | Серия убыточных сделок подряд |
| **Margin Call** | Принудительное закрытие позиций при критической просадке |
| **Liquidity Stress** | Тестирование при низкой ликвидности |

## Домашнее задание

1. **Стресс-тест генератор**: Создай генератор рыночных данных, который может симулировать:
   - Обычные условия
   - Крэш (резкое падение на N%)
   - Флеш-крэш (падение и восстановление)
   - Период низкой волатильности
   - Период высокой волатильности
   - Гэпы (случайные и в критические моменты)

2. **Анализатор устойчивости**: Напиши систему, которая:
   - Принимает торговую стратегию
   - Прогоняет её через 10+ стресс-сценариев
   - Вычисляет максимальную просадку для каждого
   - Определяет минимальный капитал для выживания
   - Генерирует отчёт с рекомендациями

3. **Monte Carlo стресс-тест**: Реализуй симуляцию:
   - Генерируй 1000 сценариев с разной волатильностью
   - Для каждого сценария запускай стратегию
   - Вычисли распределение максимальных просадок
   - Найди 95-й перцентиль (VaR - Value at Risk)
   - Оцени вероятность маржин-колла

4. **Gap Risk Analyzer**: Создай инструмент анализа гэп-риска:
   - Анализ исторических гэпов (частота, размер)
   - Симуляция стоп-лоссов при разных размерах гэпов
   - Расчёт реального ожидаемого убытка vs теоретического
   - Рекомендации по размеру позиции с учётом гэп-риска

## Навигация

[← Предыдущий день](../294-overfitting-strategy-optimization/ru.md)
