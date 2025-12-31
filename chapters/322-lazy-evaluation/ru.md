# День 322: Lazy Evaluation: Ленивые вычисления

## Аналогия из трейдинга

Представь, что ты управляешь торговым ботом, который отслеживает 1000 криптовалют. Каждую секунду ты получаешь обновления цен и должен пересчитывать индикаторы (SMA, EMA, RSI, MACD) для каждой пары.

**Жадный (eager) подход** — это как пересчитывать ВСЕ индикаторы для ВСЕХ пар каждую секунду:
- 1000 пар × 10 индикаторов × 60 секунд = 600,000 вычислений в минуту
- 99% из них не нужны, если цена не изменилась
- Ресурсы тратятся впустую

**Ленивый (lazy) подход** — как опытный трейдер:
- Пересчитывать индикатор только когда его значение РЕАЛЬНО нужно
- Если стратегия проверяет только BTC/USDT — не считать остальные 999 пар
- Если цена не изменилась — использовать закэшированное значение

В торговле lazy evaluation критически важен для:
- Экономии CPU в HFT системах
- Снижения латентности принятия решений
- Эффективной обработки больших объёмов данных
- Оптимизации backtesting на исторических данных

## Что такое Lazy Evaluation?

**Lazy Evaluation (ленивые вычисления)** — это стратегия оценки выражений, при которой вычисление откладывается до момента, когда результат действительно необходим.

### Сравнение подходов

```
┌─────────────────────────────────────────────────────────────────┐
│             Eager vs Lazy Evaluation                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  EAGER (Жадные вычисления):                                    │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │
│  │ Input 1 │───▶│ Compute │───▶│ Result 1│    │ Used?   │      │
│  └─────────┘    └─────────┘    └─────────┘    │ Maybe   │      │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    └─────────┘      │
│  │ Input 2 │───▶│ Compute │───▶│ Result 2│    │ Maybe   │      │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘      │
│                                                                 │
│  LAZY (Ленивые вычисления):                                    │
│  ┌─────────┐              ┌─────────┐    ┌─────────┐           │
│  │ Input 1 │─ ─ ─ ─ ─ ─ ▶│ Compute │───▶│ Result 1│           │
│  └─────────┘   (delayed)  └─────────┘    └─────────┘           │
│       │                        ▲                                │
│       │                        │                                │
│       └────────────────────────┘                                │
│          Only when result is needed                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Преимущества lazy evaluation

| Преимущество | Описание | Пример из трейдинга |
|--------------|----------|---------------------|
| **Экономия CPU** | Вычисления только по требованию | Расчёт индикаторов только для активных стратегий |
| **Экономия памяти** | Не храним ненужные результаты | Обработка 1M свечей без загрузки всех в память |
| **Бесконечные структуры** | Работа с потенциально бесконечными данными | Бесконечный поток цен с биржи |
| **Короткое замыкание** | Остановка вычислений при раннем результате | Стоп-лосс сработал — не считаем profit target |

## Lazy Evaluation в Rust

### Итераторы — главный инструмент

```rust
/// Демонстрация ленивых итераторов в контексте трейдинга
fn main() {
    // Имитация потока цен
    let prices: Vec<f64> = vec![
        50000.0, 50100.0, 49900.0, 50200.0, 50150.0,
        50300.0, 50250.0, 50400.0, 50350.0, 50500.0,
    ];

    // EAGER: Вычисляем ВСЕ процентные изменения сразу
    let all_changes_eager: Vec<f64> = prices
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0] * 100.0)
        .collect(); // <- collect() форсирует вычисление ВСЕХ элементов

    println!("Eager: вычислено {} изменений", all_changes_eager.len());

    // LAZY: Вычисляем только нужные изменения
    let significant_changes: Vec<f64> = prices
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0] * 100.0) // Ленивая операция
        .filter(|&change| change.abs() > 0.1)   // Ленивая операция
        .take(3)                                  // Ленивая операция
        .collect();                              // Только здесь начинается вычисление

    println!("Lazy: найдено {} значимых изменений: {:?}",
             significant_changes.len(), significant_changes);
}
```

### Ленивые адаптеры итераторов

```rust
/// Цепочка ленивых операций для анализа торговых данных
struct Trade {
    timestamp: i64,
    symbol: String,
    price: f64,
    quantity: f64,
    side: String,
}

fn analyze_trades_lazy(trades: &[Trade]) {
    // Все эти операции ЛЕНИВЫЕ — ничего не вычисляется до collect/for_each
    let btc_buys = trades.iter()
        .filter(|t| t.symbol == "BTCUSDT")      // Lazy
        .filter(|t| t.side == "BUY")            // Lazy
        .map(|t| t.price * t.quantity)          // Lazy
        .take_while(|&volume| volume < 100000.0); // Lazy

    // Только сейчас начинается обработка
    let total_volume: f64 = btc_buys.sum();
    println!("BTC buy volume (до первой крупной сделки): ${:.2}", total_volume);
}

fn main() {
    let trades = vec![
        Trade { timestamp: 1704067200, symbol: "BTCUSDT".to_string(),
                price: 50000.0, quantity: 0.5, side: "BUY".to_string() },
        Trade { timestamp: 1704067201, symbol: "ETHUSDT".to_string(),
                price: 2500.0, quantity: 10.0, side: "SELL".to_string() },
        Trade { timestamp: 1704067202, symbol: "BTCUSDT".to_string(),
                price: 50100.0, quantity: 1.0, side: "BUY".to_string() },
        Trade { timestamp: 1704067203, symbol: "BTCUSDT".to_string(),
                price: 50200.0, quantity: 3.0, side: "BUY".to_string() },
    ];

    analyze_trades_lazy(&trades);
}
```

## Практический пример: Ленивый расчёт индикаторов

### Структура с отложенным вычислением

```rust
use std::cell::RefCell;

/// Ленивый калькулятор SMA — вычисляет только при обращении
struct LazySMA {
    prices: Vec<f64>,
    period: usize,
    // Кэш для избежания повторных вычислений
    cached_value: RefCell<Option<f64>>,
}

impl LazySMA {
    fn new(period: usize) -> Self {
        LazySMA {
            prices: Vec::new(),
            period,
            cached_value: RefCell::new(None),
        }
    }

    fn add_price(&mut self, price: f64) {
        self.prices.push(price);
        // Инвалидируем кэш при добавлении новой цены
        *self.cached_value.borrow_mut() = None;
    }

    /// Ленивое получение значения — вычисляем только если нужно
    fn get(&self) -> Option<f64> {
        // Проверяем кэш
        if let Some(cached) = *self.cached_value.borrow() {
            return Some(cached);
        }

        // Вычисляем только если достаточно данных
        if self.prices.len() < self.period {
            return None;
        }

        // Вычисляем SMA
        let sum: f64 = self.prices[self.prices.len() - self.period..].iter().sum();
        let sma = sum / self.period as f64;

        // Кэшируем результат
        *self.cached_value.borrow_mut() = Some(sma);

        Some(sma)
    }
}

fn main() {
    let mut sma = LazySMA::new(3);

    // Добавляем цены — SMA ещё не вычисляется
    sma.add_price(100.0);
    sma.add_price(102.0);
    sma.add_price(101.0);
    println!("Цены добавлены, SMA ещё не вычислена");

    // Только сейчас происходит вычисление
    println!("SMA(3) = {:?}", sma.get());

    // Повторный вызов использует кэш
    println!("SMA(3) из кэша = {:?}", sma.get());

    // Добавляем новую цену — кэш инвалидируется
    sma.add_price(103.0);
    println!("SMA(3) после новой цены = {:?}", sma.get());
}
```

### Ленивый индикатор с использованием замыканий

```rust
/// Универсальный ленивый индикатор
struct LazyIndicator<F>
where
    F: Fn(&[f64]) -> Option<f64>,
{
    prices: Vec<f64>,
    compute: F,
    cached: RefCell<Option<f64>>,
    dirty: RefCell<bool>,
}

impl<F> LazyIndicator<F>
where
    F: Fn(&[f64]) -> Option<f64>,
{
    fn new(compute: F) -> Self {
        LazyIndicator {
            prices: Vec::new(),
            compute,
            cached: RefCell::new(None),
            dirty: RefCell::new(true),
        }
    }

    fn push(&mut self, price: f64) {
        self.prices.push(price);
        *self.dirty.borrow_mut() = true;
    }

    fn value(&self) -> Option<f64> {
        if !*self.dirty.borrow() {
            return *self.cached.borrow();
        }

        let result = (self.compute)(&self.prices);
        *self.cached.borrow_mut() = result;
        *self.dirty.borrow_mut() = false;
        result
    }
}

fn main() {
    // Ленивый RSI
    let rsi_compute = |prices: &[f64]| -> Option<f64> {
        if prices.len() < 15 {
            return None;
        }

        let period = 14;
        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in (prices.len() - period)..prices.len() {
            let change = prices[i] - prices[i - 1];
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
    };

    let mut rsi = LazyIndicator::new(rsi_compute);

    // Добавляем 20 цен
    for i in 0..20 {
        rsi.push(50000.0 + (i as f64 * 100.0 * ((i as f64 * 0.5).sin())));
    }

    // RSI вычисляется только сейчас
    println!("RSI = {:?}", rsi.value());
    // Повторный вызов — из кэша
    println!("RSI (cached) = {:?}", rsi.value());
}
```

## Ленивые итераторы для потоковой обработки

### Обработка миллионов свечей без загрузки в память

```rust
use std::io::{BufRead, BufReader};
use std::fs::File;

/// OHLCV свеча
#[derive(Debug)]
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    fn from_csv_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 6 {
            return None;
        }

        Some(Candle {
            timestamp: parts[0].parse().ok()?,
            open: parts[1].parse().ok()?,
            high: parts[2].parse().ok()?,
            low: parts[3].parse().ok()?,
            close: parts[4].parse().ok()?,
            volume: parts[5].parse().ok()?,
        })
    }

    fn body_percent(&self) -> f64 {
        ((self.close - self.open) / self.open).abs() * 100.0
    }
}

/// Ленивый итератор по свечам из файла
struct CandleIterator<R: BufRead> {
    reader: R,
    buffer: String,
}

impl<R: BufRead> CandleIterator<R> {
    fn new(reader: R) -> Self {
        CandleIterator {
            reader,
            buffer: String::new(),
        }
    }
}

impl<R: BufRead> Iterator for CandleIterator<R> {
    type Item = Candle;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        loop {
            match self.reader.read_line(&mut self.buffer) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    if let Some(candle) = Candle::from_csv_line(self.buffer.trim()) {
                        return Some(candle);
                    }
                    self.buffer.clear();
                    // Пропускаем невалидные строки
                }
                Err(_) => return None,
            }
        }
    }
}

fn analyze_large_file_lazy(filename: &str) {
    // Пример анализа большого файла
    // В реальном коде здесь был бы File::open(filename)

    // Симуляция данных для демонстрации
    let data = "1704067200,50000,50100,49900,50050,1000
1704070800,50050,50200,49950,50150,1500
1704074400,50150,50500,50100,50450,2000
1704078000,50450,50600,50300,50350,1800
1704081600,50350,50400,49800,49850,2500";

    let reader = BufReader::new(data.as_bytes());
    let candles = CandleIterator::new(reader);

    // Ленивая цепочка операций
    let strong_candles: Vec<_> = candles
        .filter(|c| c.volume > 1000.0)        // Только объёмные свечи
        .filter(|c| c.body_percent() > 0.5)   // Только с большим телом
        .take(2)                               // Берём первые 2
        .collect();

    println!("Найдено {} сильных свечей", strong_candles.len());
    for candle in &strong_candles {
        println!("  {:?}", candle);
    }
}

fn main() {
    analyze_large_file_lazy("candles.csv");
}
```

### Бесконечный поток цен

```rust
use std::time::{Duration, Instant};

/// Генератор бесконечного потока цен
struct PriceStream {
    current_price: f64,
    volatility: f64,
    tick: u64,
}

impl PriceStream {
    fn new(initial_price: f64, volatility: f64) -> Self {
        PriceStream {
            current_price: initial_price,
            volatility,
            tick: 0,
        }
    }
}

impl Iterator for PriceStream {
    type Item = (u64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        // Бесконечный итератор — всегда возвращает Some
        self.tick += 1;

        // Симуляция случайного движения цены
        let change = ((self.tick as f64 * 0.1).sin() * self.volatility)
                   + ((self.tick as f64 * 0.03).cos() * self.volatility * 0.5);
        self.current_price *= 1.0 + change / 100.0;

        Some((self.tick, self.current_price))
    }
}

fn main() {
    println!("=== Бесконечный поток цен (берём только 10) ===\n");

    let stream = PriceStream::new(50000.0, 0.1);

    // Берём только первые 10 цен из бесконечного потока
    // Без take() программа бы работала вечно!
    let prices: Vec<_> = stream
        .take(10)
        .collect();

    for (tick, price) in &prices {
        println!("Tick {}: ${:.2}", tick, price);
    }

    println!("\n=== Поиск первого значительного движения ===\n");

    let stream = PriceStream::new(50000.0, 0.5);

    // Ленивый поиск — останавливаемся на первом найденном
    let big_move = stream
        .take(1000)
        .enumerate()
        .find(|(i, (_, price))| {
            if *i > 0 {
                let prev_price = 50000.0 * (1.0 + (*i as f64 * 0.001).sin() * 0.005);
                (price - prev_price).abs() / prev_price > 0.01
            } else {
                false
            }
        });

    match big_move {
        Some((idx, (tick, price))) => {
            println!("Значительное движение на тике {} (индекс {}): ${:.2}", tick, idx, price);
        }
        None => println!("Значительных движений не найдено"),
    }
}
```

## Ленивая инициализация с OnceCell и LazyCell

### Отложенная загрузка конфигурации

```rust
use std::sync::OnceLock;
use std::collections::HashMap;

/// Глобальная конфигурация с ленивой инициализацией
static CONFIG: OnceLock<TradingConfig> = OnceLock::new();

#[derive(Debug)]
struct TradingConfig {
    api_key: String,
    api_secret: String,
    symbols: Vec<String>,
    risk_limit: f64,
}

impl TradingConfig {
    fn load() -> Self {
        println!("[CONFIG] Загрузка конфигурации...");
        // Симуляция загрузки из файла/env
        TradingConfig {
            api_key: "your_api_key".to_string(),
            api_secret: "your_api_secret".to_string(),
            symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            risk_limit: 0.02,
        }
    }
}

fn get_config() -> &'static TradingConfig {
    CONFIG.get_or_init(|| TradingConfig::load())
}

fn main() {
    println!("Программа запущена");
    println!("CONFIG ещё не инициализирован");

    // Первый вызов — происходит инициализация
    let config = get_config();
    println!("\nКонфигурация загружена:");
    println!("  Symbols: {:?}", config.symbols);
    println!("  Risk limit: {:.1}%", config.risk_limit * 100.0);

    // Повторный вызов — используется закэшированное значение
    println!("\nПовторный вызов:");
    let config2 = get_config();
    println!("  Symbols: {:?}", config2.symbols);
}
```

### Ленивые вычисления для стратегии

```rust
use std::cell::OnceCell;

/// Торговая стратегия с ленивыми индикаторами
struct TradingStrategy {
    prices: Vec<f64>,
    // Ленивые индикаторы — вычисляются только при обращении
    sma_20: OnceCell<f64>,
    sma_50: OnceCell<f64>,
    rsi_14: OnceCell<f64>,
    volatility: OnceCell<f64>,
}

impl TradingStrategy {
    fn new(prices: Vec<f64>) -> Self {
        TradingStrategy {
            prices,
            sma_20: OnceCell::new(),
            sma_50: OnceCell::new(),
            rsi_14: OnceCell::new(),
            volatility: OnceCell::new(),
        }
    }

    fn get_sma(&self, period: usize) -> Option<f64> {
        if self.prices.len() < period {
            return None;
        }
        let sum: f64 = self.prices[self.prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }

    fn sma_20(&self) -> Option<f64> {
        self.sma_20.get_or_init(|| {
            println!("[COMPUTE] Вычисление SMA(20)...");
            self.get_sma(20).unwrap_or(0.0)
        });
        self.sma_20.get().copied()
    }

    fn sma_50(&self) -> Option<f64> {
        self.sma_50.get_or_init(|| {
            println!("[COMPUTE] Вычисление SMA(50)...");
            self.get_sma(50).unwrap_or(0.0)
        });
        self.sma_50.get().copied()
    }

    fn rsi_14(&self) -> Option<f64> {
        self.rsi_14.get_or_init(|| {
            println!("[COMPUTE] Вычисление RSI(14)...");
            // Упрощённый расчёт RSI
            if self.prices.len() < 15 {
                return 50.0;
            }
            let mut gains = 0.0;
            let mut losses = 0.0;
            for i in (self.prices.len() - 14)..self.prices.len() {
                let change = self.prices[i] - self.prices[i - 1];
                if change > 0.0 { gains += change; }
                else { losses += change.abs(); }
            }
            if losses == 0.0 { 100.0 }
            else { 100.0 - (100.0 / (1.0 + gains / losses)) }
        });
        self.rsi_14.get().copied()
    }

    fn volatility(&self) -> Option<f64> {
        self.volatility.get_or_init(|| {
            println!("[COMPUTE] Вычисление волатильности...");
            if self.prices.len() < 20 {
                return 0.0;
            }
            let slice = &self.prices[self.prices.len() - 20..];
            let mean: f64 = slice.iter().sum::<f64>() / 20.0;
            let variance: f64 = slice.iter()
                .map(|p| (p - mean).powi(2))
                .sum::<f64>() / 20.0;
            variance.sqrt()
        });
        self.volatility.get().copied()
    }

    /// Генерация сигнала — использует только нужные индикаторы
    fn generate_signal(&self) -> &'static str {
        // Сначала проверяем RSI — если перекуплен/перепродан, остальное не важно
        if let Some(rsi) = self.rsi_14() {
            if rsi > 70.0 {
                return "SELL (RSI overbought)";
            }
            if rsi < 30.0 {
                return "BUY (RSI oversold)";
            }
        }

        // Проверяем пересечение SMA только если RSI нейтральный
        let sma_20 = self.sma_20();
        let sma_50 = self.sma_50();

        match (sma_20, sma_50) {
            (Some(fast), Some(slow)) if fast > slow => "BUY (SMA crossover)",
            (Some(fast), Some(slow)) if fast < slow => "SELL (SMA crossunder)",
            _ => "HOLD",
        }
    }
}

fn main() {
    println!("=== Ленивые индикаторы торговой стратегии ===\n");

    // Генерируем 100 цен
    let prices: Vec<f64> = (0..100)
        .map(|i| 50000.0 + (i as f64 * 0.1).sin() * 1000.0)
        .collect();

    let strategy = TradingStrategy::new(prices);

    println!("Стратегия создана, индикаторы ещё не вычислены\n");

    // Генерируем сигнал — вычисляются только нужные индикаторы
    println!("Генерация сигнала:");
    let signal = strategy.generate_signal();
    println!("Сигнал: {}\n", signal);

    // Повторный вызов — всё из кэша
    println!("Повторная генерация сигнала:");
    let signal2 = strategy.generate_signal();
    println!("Сигнал: {} (без вычислений)", signal2);
}
```

## Lazy Evaluation для оптимизации Backtesting

### Ленивый бэктестер

```rust
use std::collections::HashMap;

/// Результат сделки
#[derive(Debug, Clone)]
struct TradeResult {
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    pnl: f64,
}

/// Метрики стратегии — вычисляются лениво
struct BacktestMetrics {
    trades: Vec<TradeResult>,
    // Кэши для ленивых метрик
    total_pnl: OnceCell<f64>,
    win_rate: OnceCell<f64>,
    max_drawdown: OnceCell<f64>,
    sharpe_ratio: OnceCell<f64>,
    profit_factor: OnceCell<f64>,
}

use std::cell::OnceCell;

impl BacktestMetrics {
    fn new(trades: Vec<TradeResult>) -> Self {
        BacktestMetrics {
            trades,
            total_pnl: OnceCell::new(),
            win_rate: OnceCell::new(),
            max_drawdown: OnceCell::new(),
            sharpe_ratio: OnceCell::new(),
            profit_factor: OnceCell::new(),
        }
    }

    fn total_pnl(&self) -> f64 {
        *self.total_pnl.get_or_init(|| {
            println!("[METRICS] Вычисление Total PnL...");
            self.trades.iter().map(|t| t.pnl).sum()
        })
    }

    fn win_rate(&self) -> f64 {
        *self.win_rate.get_or_init(|| {
            println!("[METRICS] Вычисление Win Rate...");
            if self.trades.is_empty() {
                return 0.0;
            }
            let wins = self.trades.iter().filter(|t| t.pnl > 0.0).count();
            wins as f64 / self.trades.len() as f64 * 100.0
        })
    }

    fn max_drawdown(&self) -> f64 {
        *self.max_drawdown.get_or_init(|| {
            println!("[METRICS] Вычисление Max Drawdown...");
            let mut peak = 0.0;
            let mut max_dd = 0.0;
            let mut equity = 0.0;

            for trade in &self.trades {
                equity += trade.pnl;
                if equity > peak {
                    peak = equity;
                }
                let dd = (peak - equity) / peak.max(1.0);
                if dd > max_dd {
                    max_dd = dd;
                }
            }
            max_dd * 100.0
        })
    }

    fn sharpe_ratio(&self) -> f64 {
        *self.sharpe_ratio.get_or_init(|| {
            println!("[METRICS] Вычисление Sharpe Ratio...");
            if self.trades.len() < 2 {
                return 0.0;
            }

            let returns: Vec<f64> = self.trades.iter()
                .map(|t| t.pnl / t.entry_price / t.quantity)
                .collect();

            let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance: f64 = returns.iter()
                .map(|r| (r - mean).powi(2))
                .sum::<f64>() / returns.len() as f64;
            let std_dev = variance.sqrt();

            if std_dev == 0.0 {
                return 0.0;
            }

            // Annualized (assuming daily returns)
            mean / std_dev * (252.0_f64).sqrt()
        })
    }

    fn profit_factor(&self) -> f64 {
        *self.profit_factor.get_or_init(|| {
            println!("[METRICS] Вычисление Profit Factor...");
            let gross_profit: f64 = self.trades.iter()
                .filter(|t| t.pnl > 0.0)
                .map(|t| t.pnl)
                .sum();
            let gross_loss: f64 = self.trades.iter()
                .filter(|t| t.pnl < 0.0)
                .map(|t| t.pnl.abs())
                .sum();

            if gross_loss == 0.0 {
                return f64::INFINITY;
            }

            gross_profit / gross_loss
        })
    }

    /// Быстрый отчёт — только основные метрики
    fn quick_report(&self) {
        println!("\n=== Quick Report ===");
        println!("Total PnL: ${:.2}", self.total_pnl());
        println!("Win Rate: {:.1}%", self.win_rate());
    }

    /// Полный отчёт — все метрики
    fn full_report(&self) {
        println!("\n=== Full Report ===");
        println!("Total PnL: ${:.2}", self.total_pnl());
        println!("Win Rate: {:.1}%", self.win_rate());
        println!("Max Drawdown: {:.1}%", self.max_drawdown());
        println!("Sharpe Ratio: {:.2}", self.sharpe_ratio());
        println!("Profit Factor: {:.2}", self.profit_factor());
    }
}

fn main() {
    println!("=== Ленивые метрики бэктеста ===\n");

    // Симуляция результатов торговли
    let trades: Vec<TradeResult> = (0..100)
        .map(|i| {
            let entry = 50000.0 + (i as f64 * 100.0);
            let exit = entry * (1.0 + ((i as f64 * 0.3).sin() * 0.02));
            let quantity = 0.1;
            TradeResult {
                entry_price: entry,
                exit_price: exit,
                quantity,
                pnl: (exit - entry) * quantity,
            }
        })
        .collect();

    let metrics = BacktestMetrics::new(trades);

    println!("Метрики созданы, ничего не вычислено\n");

    // Быстрый отчёт — вычисляем только 2 метрики
    println!("--- Быстрый отчёт ---");
    metrics.quick_report();

    println!("\n--- Полный отчёт ---");
    // Полный отчёт — TotalPnL и WinRate уже закэшированы
    metrics.full_report();
}
```

## Сравнение производительности Eager vs Lazy

```rust
use std::time::Instant;

/// Бенчмарк сравнения eager и lazy подходов
fn benchmark_eager_vs_lazy() {
    const NUM_PRICES: usize = 1_000_000;
    const SAMPLE_SIZE: usize = 100;

    // Генерируем большой набор данных
    let prices: Vec<f64> = (0..NUM_PRICES)
        .map(|i| 50000.0 + (i as f64 * 0.001).sin() * 5000.0)
        .collect();

    println!("=== Benchmark: Eager vs Lazy ===\n");
    println!("Данных: {} цен", NUM_PRICES);
    println!("Ищем: первые {} цен > $52000\n", SAMPLE_SIZE);

    // EAGER подход: вычисляем всё, потом фильтруем
    let start = Instant::now();
    let filtered_eager: Vec<f64> = prices.clone()
        .into_iter()
        .filter(|&p| p > 52000.0)
        .collect();
    let result_eager: Vec<f64> = filtered_eager.into_iter().take(SAMPLE_SIZE).collect();
    let eager_time = start.elapsed();

    // LAZY подход: вычисляем только нужное
    let start = Instant::now();
    let result_lazy: Vec<f64> = prices.iter()
        .copied()
        .filter(|&p| p > 52000.0)
        .take(SAMPLE_SIZE)
        .collect();
    let lazy_time = start.elapsed();

    println!("EAGER:");
    println!("  Время: {:?}", eager_time);
    println!("  Результат: {} элементов", result_eager.len());

    println!("\nLAZY:");
    println!("  Время: {:?}", lazy_time);
    println!("  Результат: {} элементов", result_lazy.len());

    let speedup = eager_time.as_nanos() as f64 / lazy_time.as_nanos() as f64;
    println!("\nУскорение: {:.1}x", speedup);
}

fn main() {
    benchmark_eager_vs_lazy();
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Lazy Evaluation** | Отложенные вычисления до момента использования результата |
| **Итераторы** | Основной инструмент lazy evaluation в Rust |
| **OnceCell/OnceLock** | Ленивая инициализация с кэшированием |
| **Short-circuit** | Прерывание вычислений при достижении результата |
| **Infinite iterators** | Работа с бесконечными последовательностями |
| **Memoization** | Кэширование результатов для избежания повторных вычислений |

## Практические упражнения

1. **Ленивый OrderBook**: Создай структуру OrderBook, которая:
   - Вычисляет спред только при обращении
   - Кэширует лучший bid/ask
   - Инвалидирует кэш при изменении книги

2. **Ленивый парсер CSV**: Реализуй итератор, который:
   - Читает CSV файл построчно
   - Парсит строку только при вызове next()
   - Поддерживает filter/map/take без загрузки всего файла

3. **Ленивая стратегия**: Напиши стратегию, которая:
   - Имеет 10 разных индикаторов
   - Вычисляет индикаторы только если они нужны для сигнала
   - Использует short-circuit evaluation

4. **Бенчмарк**: Сравни eager и lazy подходы на:
   - 10M свечей
   - Поиск первых 10 свечей с условием
   - Измерь разницу в CPU и памяти

## Домашнее задание

1. **Ленивый Market Scanner**:
   - Сканирует 1000 криптовалют
   - Вычисляет индикаторы только для пар, прошедших первичный фильтр
   - Использует многоуровневую фильтрацию (объём → волатильность → паттерн)
   - Измерь экономию CPU по сравнению с eager подходом

2. **Streaming Backtester**:
   - Обрабатывает исторические данные как ленивый поток
   - Не загружает все данные в память
   - Поддерживает остановку по условию (достигнут лимит убытков)
   - Выводит метрики в реальном времени

3. **Lazy Config System**:
   - Конфигурация загружается только при первом обращении
   - Поддерживает hot-reload (перезагрузка при изменении файла)
   - Валидация происходит лениво
   - Логирование только реально используемых параметров

4. **Оптимизация существующего кода**:
   - Возьми свой trading bot или пример из предыдущих глав
   - Найди места с eager вычислениями
   - Преврати их в lazy
   - Измерь улучшение производительности

## Навигация

[← Предыдущий день](../319-memory-tracking-leaks/ru.md) | [Следующий день →](../323-*/ru.md)
