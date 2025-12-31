# День 110: Graceful Degradation: работаем без части данных

## Аналогия из трейдинга

Представь, что ты торгуешь на бирже и используешь несколько индикаторов: RSI, MACD, скользящие средние. Внезапно провайдер данных перестал отдавать RSI. Что делать? Останавливать торговлю полностью? Опытный трейдер **продолжит работу** с теми данными, которые есть, возможно, снизив размер позиции или используя только надёжные сигналы.

Это и есть **Graceful Degradation** — способность системы продолжать работу с ограниченной функциональностью, когда часть данных или сервисов недоступна.

## Концепция Graceful Degradation

В Rust мы используем `Option` для представления данных, которые могут отсутствовать. Graceful Degradation — это стратегия обработки таких ситуаций:

```rust
fn main() {
    // Данные могут быть частично недоступны
    let price = Some(42000.0);
    let rsi: Option<f64> = None;  // RSI недоступен
    let macd = Some(150.0);

    // Graceful degradation: работаем с тем, что есть
    analyze_market(price, rsi, macd);
}

fn analyze_market(price: Option<f64>, rsi: Option<f64>, macd: Option<f64>) {
    let price = match price {
        Some(p) => p,
        None => {
            println!("Критично: нет данных о цене, анализ невозможен");
            return;
        }
    };

    println!("Анализ рынка при цене ${:.2}", price);

    // RSI — не критичен, используем если есть
    match rsi {
        Some(r) => println!("  RSI: {:.1}", r),
        None => println!("  RSI: данные недоступны, пропускаем"),
    }

    // MACD — не критичен
    match macd {
        Some(m) => println!("  MACD: {:.2}", m),
        None => println!("  MACD: данные недоступны, пропускаем"),
    }
}
```

## unwrap_or — значение по умолчанию

Самый простой способ обеспечить graceful degradation:

```rust
fn main() {
    let prices: Vec<Option<f64>> = vec![
        Some(42000.0),
        None,           // Пропущенная цена
        Some(42100.0),
        None,           // Пропущенная цена
        Some(42050.0),
    ];

    // Используем последнюю известную цену как fallback
    let mut last_known_price = 0.0;

    for (i, price_opt) in prices.iter().enumerate() {
        let price = price_opt.unwrap_or(last_known_price);

        if price_opt.is_some() {
            last_known_price = price;
        }

        println!("Свеча {}: ${:.2} {}",
            i + 1,
            price,
            if price_opt.is_none() { "(interpolated)" } else { "" }
        );
    }
}
```

## unwrap_or_default — нулевые значения по умолчанию

Для типов с реализацией `Default`:

```rust
fn main() {
    let trade_volume: Option<f64> = None;
    let order_count: Option<u32> = None;
    let is_active: Option<bool> = None;

    // f64::default() = 0.0, u32::default() = 0, bool::default() = false
    println!("Объём: {}", trade_volume.unwrap_or_default());
    println!("Количество ордеров: {}", order_count.unwrap_or_default());
    println!("Активен: {}", is_active.unwrap_or_default());
}
```

## unwrap_or_else — ленивое вычисление fallback

Когда fallback требует вычислений:

```rust
fn main() {
    let current_price: Option<f64> = None;
    let historical_prices = vec![41800.0, 41900.0, 42000.0];

    // Если текущая цена недоступна, вычисляем среднюю историческую
    let price = current_price.unwrap_or_else(|| {
        println!("Текущая цена недоступна, вычисляем среднюю...");
        let sum: f64 = historical_prices.iter().sum();
        sum / historical_prices.len() as f64
    });

    println!("Используемая цена: ${:.2}", price);
}
```

## Практический пример: торговый сигнал с частичными данными

```rust
fn main() {
    // Сценарий 1: все данные доступны
    let signal1 = generate_signal(
        Some(42000.0),  // price
        Some(45.0),     // rsi
        Some(100.0),    // macd
        Some(41800.0),  // sma
    );
    println!("Сигнал 1: {:?}\n", signal1);

    // Сценарий 2: RSI недоступен
    let signal2 = generate_signal(
        Some(42000.0),
        None,           // RSI недоступен
        Some(100.0),
        Some(41800.0),
    );
    println!("Сигнал 2: {:?}\n", signal2);

    // Сценарий 3: только цена и SMA
    let signal3 = generate_signal(
        Some(42000.0),
        None,
        None,
        Some(41800.0),
    );
    println!("Сигнал 3: {:?}\n", signal3);
}

#[derive(Debug)]
struct TradingSignal {
    action: String,
    confidence: f64,
    available_indicators: u8,
    warnings: Vec<String>,
}

fn generate_signal(
    price: Option<f64>,
    rsi: Option<f64>,
    macd: Option<f64>,
    sma: Option<f64>,
) -> Option<TradingSignal> {
    // Цена критична — без неё не можем работать
    let price = price?;

    let mut bullish_signals = 0;
    let mut bearish_signals = 0;
    let mut available = 0;
    let mut warnings = Vec::new();

    // RSI анализ (если доступен)
    if let Some(rsi_value) = rsi {
        available += 1;
        if rsi_value < 30.0 {
            bullish_signals += 1;  // Перепроданность
        } else if rsi_value > 70.0 {
            bearish_signals += 1;  // Перекупленность
        }
    } else {
        warnings.push("RSI недоступен".to_string());
    }

    // MACD анализ (если доступен)
    if let Some(macd_value) = macd {
        available += 1;
        if macd_value > 0.0 {
            bullish_signals += 1;
        } else {
            bearish_signals += 1;
        }
    } else {
        warnings.push("MACD недоступен".to_string());
    }

    // SMA анализ (если доступен)
    if let Some(sma_value) = sma {
        available += 1;
        if price > sma_value {
            bullish_signals += 1;
        } else {
            bearish_signals += 1;
        }
    } else {
        warnings.push("SMA недоступен".to_string());
    }

    // Определяем действие
    let action = if available == 0 {
        "HOLD".to_string()  // Нет данных для анализа
    } else if bullish_signals > bearish_signals {
        "BUY".to_string()
    } else if bearish_signals > bullish_signals {
        "SELL".to_string()
    } else {
        "HOLD".to_string()
    };

    // Уверенность зависит от количества доступных индикаторов
    let max_indicators = 3.0;
    let confidence = (available as f64 / max_indicators) * 100.0;

    Some(TradingSignal {
        action,
        confidence,
        available_indicators: available,
        warnings,
    })
}
```

## Цепочки Option с map и and_then

```rust
fn main() {
    let raw_price: Option<&str> = Some("42000.50");

    // Цепочка преобразований с graceful degradation
    let processed = raw_price
        .map(|s| s.trim())                           // Убираем пробелы
        .and_then(|s| s.parse::<f64>().ok())         // Парсим (может не получиться)
        .map(|p| p * 1.001)                          // Добавляем комиссию
        .unwrap_or(0.0);                             // Fallback

    println!("Обработанная цена: ${:.2}", processed);

    // Пример с невалидными данными
    let invalid_price: Option<&str> = Some("not_a_price");
    let result = invalid_price
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or_else(|| {
            println!("Не удалось распарсить цену, используем 0");
            0.0
        });

    println!("Результат: ${:.2}", result);
}
```

## Работа с частичными данными портфеля

```rust
fn main() {
    let portfolio = Portfolio {
        btc_balance: Some(1.5),
        eth_balance: None,  // Данные недоступны
        usdt_balance: Some(10000.0),
    };

    let prices = Prices {
        btc: Some(42000.0),
        eth: Some(2200.0),  // Цена есть, но баланса нет
        usdt: Some(1.0),
    };

    let report = calculate_portfolio_value(&portfolio, &prices);
    println!("{}", report);
}

struct Portfolio {
    btc_balance: Option<f64>,
    eth_balance: Option<f64>,
    usdt_balance: Option<f64>,
}

struct Prices {
    btc: Option<f64>,
    eth: Option<f64>,
    usdt: Option<f64>,
}

fn calculate_portfolio_value(portfolio: &Portfolio, prices: &Prices) -> String {
    let mut total = 0.0;
    let mut report = String::from("=== Отчёт по портфелю ===\n");
    let mut warnings = Vec::new();

    // BTC
    match (portfolio.btc_balance, prices.btc) {
        (Some(bal), Some(price)) => {
            let value = bal * price;
            total += value;
            report.push_str(&format!("BTC: {:.4} × ${:.2} = ${:.2}\n", bal, price, value));
        }
        (None, _) => warnings.push("BTC баланс недоступен"),
        (_, None) => warnings.push("BTC цена недоступна"),
    }

    // ETH
    match (portfolio.eth_balance, prices.eth) {
        (Some(bal), Some(price)) => {
            let value = bal * price;
            total += value;
            report.push_str(&format!("ETH: {:.4} × ${:.2} = ${:.2}\n", bal, price, value));
        }
        (None, _) => warnings.push("ETH баланс недоступен"),
        (_, None) => warnings.push("ETH цена недоступна"),
    }

    // USDT
    match (portfolio.usdt_balance, prices.usdt) {
        (Some(bal), Some(price)) => {
            let value = bal * price;
            total += value;
            report.push_str(&format!("USDT: {:.2} × ${:.2} = ${:.2}\n", bal, price, value));
        }
        (None, _) => warnings.push("USDT баланс недоступен"),
        (_, None) => warnings.push("USDT цена недоступна"),
    }

    report.push_str(&format!("\nИТОГО: ${:.2}\n", total));

    if !warnings.is_empty() {
        report.push_str("\nПредупреждения:\n");
        for w in warnings {
            report.push_str(&format!("  ⚠ {}\n", w));
        }
        report.push_str("\n(Итого может быть неполным)");
    }

    report
}
```

## Стратегия fallback для исторических данных

```rust
fn main() {
    let candles = vec![
        Candle { open: Some(42000.0), high: Some(42500.0), low: Some(41800.0), close: Some(42200.0) },
        Candle { open: Some(42200.0), high: None, low: Some(42000.0), close: Some(42300.0) },  // Нет high
        Candle { open: None, high: Some(42600.0), low: Some(42200.0), close: Some(42400.0) },  // Нет open
        Candle { open: Some(42400.0), high: Some(42700.0), low: None, close: None },  // Нет low и close
    ];

    for (i, candle) in candles.iter().enumerate() {
        let normalized = normalize_candle(candle, i, &candles);
        println!("Свеча {}: O={:.0} H={:.0} L={:.0} C={:.0}",
            i + 1, normalized.0, normalized.1, normalized.2, normalized.3);
    }
}

struct Candle {
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: Option<f64>,
}

fn normalize_candle(candle: &Candle, index: usize, all_candles: &[Candle]) -> (f64, f64, f64, f64) {
    // Стратегия: используем предыдущую свечу для fallback
    let prev_close = if index > 0 {
        all_candles[index - 1].close.unwrap_or(0.0)
    } else {
        0.0
    };

    let open = candle.open.unwrap_or(prev_close);
    let close = candle.close.unwrap_or(open);  // Если нет close, используем open

    // High должен быть >= max(open, close)
    let min_high = open.max(close);
    let high = candle.high.unwrap_or(min_high);

    // Low должен быть <= min(open, close)
    let max_low = open.min(close);
    let low = candle.low.unwrap_or(max_low);

    (open, high, low, close)
}
```

## Что мы узнали

| Метод | Использование | Пример |
|-------|--------------|--------|
| `unwrap_or(default)` | Фиксированное значение | `price.unwrap_or(0.0)` |
| `unwrap_or_default()` | Значение Default | `count.unwrap_or_default()` |
| `unwrap_or_else(fn)` | Ленивое вычисление | `price.unwrap_or_else(\|\| calc())` |
| `map(fn)` | Преобразование Some | `opt.map(\|x\| x * 2)` |
| `and_then(fn)` | Цепочка Option | `opt.and_then(\|x\| parse(x))` |
| `?` оператор | Ранний возврат None | `let x = opt?;` |

## Домашнее задание

1. **Анализатор ордербука с пропусками**: Создай функцию, которая анализирует ордербук, где некоторые уровни цен могут быть недоступны. Функция должна рассчитать приблизительный спред и глубину, используя доступные данные.

2. **Мультибиржевой агрегатор**: Напиши систему, которая получает цены с 3-х бирж (все `Option<f64>`). Если все биржи доступны — выводи среднюю цену. Если доступны 2 — среднюю из них. Если только 1 — её цену. Если ни одной — последнюю известную.

3. **Торговая стратегия с деградацией**: Реализуй стратегию, которая использует 5 индикаторов. При отсутствии индикатора стратегия должна снижать размер позиции пропорционально количеству недоступных индикаторов.

4. **Восстановление временного ряда**: Напиши функцию, которая принимает `Vec<Option<f64>>` (цены с пропусками) и возвращает `Vec<f64>`, где пропуски заполнены линейной интерполяцией между соседними известными значениями.

## Навигация

[← Предыдущий день](../109-circuit-breaker-cascade-failure/ru.md) | [Следующий день →](../111-input-validation/ru.md)
