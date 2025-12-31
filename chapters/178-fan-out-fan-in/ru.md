# День 178: Паттерн Fan-out Fan-in

## Аналогия из трейдинга

Представь, что ты управляешь торговой системой, которая должна анализировать данные с 10 бирж одновременно. Ты мог бы делать это последовательно — сначала проверить Binance, потом Kraken, затем Coinbase... Но это займёт много времени!

Вместо этого ты используешь паттерн **Fan-out Fan-in**:
- **Fan-out (разветвление)**: раздаёшь задачи нескольким воркерам параллельно — каждый воркер опрашивает свою биржу
- **Fan-in (схождение)**: собираешь результаты от всех воркеров в один поток для анализа

Это как если бы ты отправил 10 аналитиков проверять цены на разных биржах, а потом они все сообщают тебе результаты, и ты выбираешь лучшую цену.

```
                    ┌──> Воркер 1 (Binance)  ──┐
                    │                          │
Входные данные ─────┼──> Воркер 2 (Kraken)   ──┼───> Агрегация результатов
     (Fan-out)      │                          │         (Fan-in)
                    └──> Воркер 3 (Coinbase) ──┘
```

## Что такое Fan-out Fan-in?

**Fan-out Fan-in** — это паттерн параллельной обработки, где:

1. **Fan-out**: одна задача распределяется между несколькими параллельными обработчиками
2. **Fan-in**: результаты от всех обработчиков собираются в одно место

Этот паттерн идеален для:
- Агрегации данных с нескольких источников
- Параллельной обработки большого количества элементов
- Распределённых вычислений

## Простой пример: анализ цен с бирж

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct PriceData {
    exchange: String,
    symbol: String,
    price: f64,
    volume: f64,
}

fn main() {
    // Канал для сбора результатов (Fan-in)
    let (tx, rx) = mpsc::channel();

    let exchanges = vec![
        ("Binance", 42150.0, 1000.0),
        ("Kraken", 42145.0, 800.0),
        ("Coinbase", 42160.0, 1200.0),
        ("Bitstamp", 42140.0, 500.0),
        ("Gemini", 42155.0, 600.0),
    ];

    // Fan-out: запускаем воркер для каждой биржи
    for (exchange, price, volume) in exchanges {
        let tx = tx.clone();
        thread::spawn(move || {
            // Имитация сетевой задержки
            thread::sleep(Duration::from_millis(100 + (price as u64 % 50)));

            let data = PriceData {
                exchange: exchange.to_string(),
                symbol: "BTC/USD".to_string(),
                price,
                volume,
            };

            println!("[{}] Получена цена: ${:.2}", exchange, price);
            tx.send(data).unwrap();
        });
    }

    // Важно: удаляем оригинальный отправитель
    drop(tx);

    // Fan-in: собираем все результаты
    let mut prices: Vec<PriceData> = Vec::new();
    for data in rx {
        prices.push(data);
    }

    // Анализируем собранные данные
    println!("\n=== Анализ рынка ===");

    let best_bid = prices.iter()
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
        .unwrap();

    let best_ask = prices.iter()
        .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
        .unwrap();

    let avg_price: f64 = prices.iter().map(|p| p.price).sum::<f64>() / prices.len() as f64;
    let total_volume: f64 = prices.iter().map(|p| p.volume).sum();

    println!("Лучшая цена покупки: {} @ ${:.2}", best_bid.exchange, best_bid.price);
    println!("Лучшая цена продажи: {} @ ${:.2}", best_ask.exchange, best_ask.price);
    println!("Средняя цена: ${:.2}", avg_price);
    println!("Общий объём: {:.0} BTC", total_volume);
    println!("Спред: ${:.2}", best_bid.price - best_ask.price);
}
```

## Fan-out с пулом воркеров

В реальных системах мы ограничиваем количество параллельных воркеров:

```rust
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct TradeSignal {
    symbol: String,
    signal_type: String,
    strength: f64,
}

#[derive(Debug)]
struct AnalysisResult {
    symbol: String,
    recommendation: String,
    confidence: f64,
}

fn main() {
    // Очередь задач (символы для анализа)
    let symbols = vec![
        "BTC/USD", "ETH/USD", "SOL/USD", "ADA/USD",
        "DOT/USD", "LINK/USD", "AVAX/USD", "MATIC/USD",
    ];

    let tasks: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(
        symbols.iter().map(|s| s.to_string()).collect()
    ));

    let (tx, rx) = mpsc::channel();

    // Создаём пул из 3 воркеров (Fan-out с ограничением)
    let num_workers = 3;
    let mut handles = vec![];

    for worker_id in 0..num_workers {
        let tasks = Arc::clone(&tasks);
        let tx = tx.clone();

        let handle = thread::spawn(move || {
            loop {
                // Получаем задачу из очереди
                let symbol = {
                    let mut queue = tasks.lock().unwrap();
                    queue.pop_front()
                };

                match symbol {
                    Some(symbol) => {
                        println!("Воркер {} анализирует {}", worker_id, symbol);

                        // Имитация анализа
                        thread::sleep(std::time::Duration::from_millis(150));

                        // Генерируем результат анализа
                        let strength = (worker_id as f64 * 0.1) + 0.7;
                        let result = AnalysisResult {
                            symbol: symbol.clone(),
                            recommendation: if strength > 0.75 {
                                "BUY".to_string()
                            } else {
                                "HOLD".to_string()
                            },
                            confidence: strength,
                        };

                        tx.send(result).unwrap();
                    }
                    None => break, // Очередь пуста
                }
            }
            println!("Воркер {} завершил работу", worker_id);
        });

        handles.push(handle);
    }

    drop(tx); // Закрываем канал после запуска всех воркеров

    // Fan-in: собираем результаты
    let mut results: Vec<AnalysisResult> = Vec::new();
    for result in rx {
        println!("  -> {} получен: {} ({:.0}%)",
            result.symbol,
            result.recommendation,
            result.confidence * 100.0
        );
        results.push(result);
    }

    // Ждём завершения всех воркеров
    for handle in handles {
        handle.join().unwrap();
    }

    // Итоговый анализ
    println!("\n=== Итоговые рекомендации ===");
    let buy_signals: Vec<_> = results.iter()
        .filter(|r| r.recommendation == "BUY")
        .collect();

    println!("Сигналы на покупку: {} из {}", buy_signals.len(), results.len());
    for signal in buy_signals {
        println!("  {} (уверенность: {:.0}%)", signal.symbol, signal.confidence * 100.0);
    }
}
```

## Параллельный анализ портфеля

Используем Fan-out Fan-in для расчёта метрик портфеля:

```rust
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
}

#[derive(Debug)]
struct PositionMetrics {
    symbol: String,
    pnl: f64,
    pnl_percent: f64,
    value: f64,
    risk_score: f64,
}

fn calculate_position_metrics(position: Position) -> PositionMetrics {
    // Имитация сложного расчёта
    thread::sleep(std::time::Duration::from_millis(50));

    let pnl = (position.current_price - position.entry_price) * position.quantity;
    let pnl_percent = ((position.current_price / position.entry_price) - 1.0) * 100.0;
    let value = position.current_price * position.quantity;

    // Простой расчёт риска на основе волатильности
    let risk_score = (pnl_percent.abs() / 10.0).min(1.0);

    PositionMetrics {
        symbol: position.symbol,
        pnl,
        pnl_percent,
        value,
        risk_score,
    }
}

fn main() {
    let portfolio = vec![
        Position { symbol: "BTC".to_string(), quantity: 2.5, entry_price: 40000.0, current_price: 42000.0 },
        Position { symbol: "ETH".to_string(), quantity: 30.0, entry_price: 2500.0, current_price: 2650.0 },
        Position { symbol: "SOL".to_string(), quantity: 100.0, entry_price: 100.0, current_price: 95.0 },
        Position { symbol: "ADA".to_string(), quantity: 5000.0, entry_price: 0.50, current_price: 0.55 },
        Position { symbol: "DOT".to_string(), quantity: 200.0, entry_price: 7.0, current_price: 7.5 },
        Position { symbol: "LINK".to_string(), quantity: 150.0, entry_price: 15.0, current_price: 14.0 },
    ];

    let (tx, rx) = mpsc::channel();

    println!("Запуск параллельного анализа {} позиций...\n", portfolio.len());
    let start = std::time::Instant::now();

    // Fan-out: каждая позиция анализируется в отдельном потоке
    for position in portfolio {
        let tx = tx.clone();
        thread::spawn(move || {
            let metrics = calculate_position_metrics(position);
            tx.send(metrics).unwrap();
        });
    }

    drop(tx);

    // Fan-in: собираем все метрики
    let mut all_metrics: Vec<PositionMetrics> = Vec::new();
    for metrics in rx {
        all_metrics.push(metrics);
    }

    let elapsed = start.elapsed();

    // Агрегированные метрики портфеля
    println!("=== Анализ портфеля (за {:?}) ===\n", elapsed);

    let total_value: f64 = all_metrics.iter().map(|m| m.value).sum();
    let total_pnl: f64 = all_metrics.iter().map(|m| m.pnl).sum();
    let avg_risk: f64 = all_metrics.iter().map(|m| m.risk_score).sum::<f64>()
        / all_metrics.len() as f64;

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║                 МЕТРИКИ ПОРТФЕЛЯ                     ║");
    println!("╠══════════════════════════════════════════════════════╣");

    for m in &all_metrics {
        let pnl_indicator = if m.pnl >= 0.0 { "+" } else { "" };
        println!("║ {:6} | PnL: {}${:>10.2} ({:>+6.2}%) | Risk: {:.2} ║",
            m.symbol,
            pnl_indicator,
            m.pnl.abs(),
            m.pnl_percent,
            m.risk_score
        );
    }

    println!("╠══════════════════════════════════════════════════════╣");
    println!("║ Общая стоимость:    ${:>12.2}                  ║", total_value);
    println!("║ Общий PnL:          ${:>12.2}                  ║", total_pnl);
    println!("║ Средний риск:       {:>13.2}                  ║", avg_risk);
    println!("╚══════════════════════════════════════════════════════╝");

    // Определяем позиции, требующие внимания
    let risky_positions: Vec<_> = all_metrics.iter()
        .filter(|m| m.risk_score > 0.5)
        .collect();

    if !risky_positions.is_empty() {
        println!("\n⚠️  Позиции с повышенным риском:");
        for pos in risky_positions {
            println!("   - {} (риск: {:.2})", pos.symbol, pos.risk_score);
        }
    }
}
```

## Сканер рынка с Fan-out Fan-in

Реалистичный пример сканера, который ищет торговые возможности:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct MarketData {
    symbol: String,
    price: f64,
    volume_24h: f64,
    change_24h: f64,
    rsi: f64,
}

#[derive(Debug)]
struct TradingOpportunity {
    symbol: String,
    signal: String,
    entry_price: f64,
    target_price: f64,
    stop_loss: f64,
    score: f64,
}

fn scan_symbol(data: MarketData) -> Option<TradingOpportunity> {
    // Имитация анализа
    thread::sleep(Duration::from_millis(100));

    // Условия для торговой возможности
    let is_oversold = data.rsi < 30.0;
    let is_overbought = data.rsi > 70.0;
    let high_volume = data.volume_24h > 1_000_000.0;
    let significant_drop = data.change_24h < -5.0;
    let significant_rise = data.change_24h > 5.0;

    if is_oversold && high_volume && significant_drop {
        // Сигнал на покупку
        Some(TradingOpportunity {
            symbol: data.symbol,
            signal: "BUY".to_string(),
            entry_price: data.price,
            target_price: data.price * 1.10, // +10%
            stop_loss: data.price * 0.95,    // -5%
            score: (30.0 - data.rsi) / 30.0 + (data.change_24h.abs() / 10.0),
        })
    } else if is_overbought && high_volume && significant_rise {
        // Сигнал на продажу
        Some(TradingOpportunity {
            symbol: data.symbol,
            signal: "SELL".to_string(),
            entry_price: data.price,
            target_price: data.price * 0.90, // -10%
            stop_loss: data.price * 1.05,    // +5%
            score: (data.rsi - 70.0) / 30.0 + (data.change_24h / 10.0),
        })
    } else {
        None
    }
}

fn main() {
    // Симуляция рыночных данных
    let market_data = vec![
        MarketData { symbol: "BTC".to_string(), price: 42000.0, volume_24h: 5_000_000.0, change_24h: -7.5, rsi: 25.0 },
        MarketData { symbol: "ETH".to_string(), price: 2600.0, volume_24h: 2_000_000.0, change_24h: 2.0, rsi: 55.0 },
        MarketData { symbol: "SOL".to_string(), price: 95.0, volume_24h: 1_500_000.0, change_24h: 8.0, rsi: 78.0 },
        MarketData { symbol: "ADA".to_string(), price: 0.55, volume_24h: 800_000.0, change_24h: -3.0, rsi: 42.0 },
        MarketData { symbol: "DOT".to_string(), price: 7.5, volume_24h: 1_200_000.0, change_24h: -6.0, rsi: 28.0 },
        MarketData { symbol: "AVAX".to_string(), price: 35.0, volume_24h: 900_000.0, change_24h: 1.5, rsi: 50.0 },
        MarketData { symbol: "LINK".to_string(), price: 14.0, volume_24h: 1_100_000.0, change_24h: 6.5, rsi: 72.0 },
        MarketData { symbol: "MATIC".to_string(), price: 0.85, volume_24h: 600_000.0, change_24h: -2.0, rsi: 38.0 },
    ];

    let (tx, rx) = mpsc::channel();

    println!("🔍 Сканирование рынка ({} символов)...\n", market_data.len());
    let start = std::time::Instant::now();

    // Fan-out: параллельное сканирование
    for data in market_data {
        let tx = tx.clone();
        thread::spawn(move || {
            let result = scan_symbol(data);
            tx.send(result).unwrap();
        });
    }

    drop(tx);

    // Fan-in: сбор результатов
    let mut opportunities: Vec<TradingOpportunity> = Vec::new();
    for result in rx {
        if let Some(opp) = result {
            opportunities.push(opp);
        }
    }

    let elapsed = start.elapsed();

    // Сортируем по силе сигнала
    opportunities.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    println!("=== Результаты сканирования (за {:?}) ===\n", elapsed);

    if opportunities.is_empty() {
        println!("Торговые возможности не найдены.");
    } else {
        println!("Найдено {} торговых возможностей:\n", opportunities.len());

        for (i, opp) in opportunities.iter().enumerate() {
            let emoji = if opp.signal == "BUY" { "🟢" } else { "🔴" };
            println!("{}. {} {} {}", i + 1, emoji, opp.signal, opp.symbol);
            println!("   Вход: ${:.4}", opp.entry_price);
            println!("   Цель: ${:.4}", opp.target_price);
            println!("   Стоп: ${:.4}", opp.stop_loss);
            println!("   Сила сигнала: {:.2}\n", opp.score);
        }
    }
}
```

## Fan-out Fan-in с обработкой ошибок

В реальных системах важно обрабатывать ошибки:

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum FetchError {
    Timeout,
    NetworkError(String),
    ParseError,
}

#[derive(Debug)]
struct ExchangePrice {
    exchange: String,
    price: f64,
}

type FetchResult = Result<ExchangePrice, (String, FetchError)>;

fn fetch_price(exchange: &str, should_fail: bool) -> FetchResult {
    // Имитация сетевого запроса
    thread::sleep(Duration::from_millis(100));

    if should_fail {
        return Err((exchange.to_string(), FetchError::NetworkError("Connection refused".to_string())));
    }

    // Имитация успешного ответа
    let base_price = 42000.0;
    let variation = (exchange.len() as f64 * 10.0) - 30.0;

    Ok(ExchangePrice {
        exchange: exchange.to_string(),
        price: base_price + variation,
    })
}

fn main() {
    let exchanges = vec![
        ("Binance", false),
        ("Kraken", true),      // Имитация ошибки
        ("Coinbase", false),
        ("Bitstamp", false),
        ("Gemini", true),      // Имитация ошибки
    ];

    let (tx, rx) = mpsc::channel();

    println!("Получение цен с {} бирж...\n", exchanges.len());

    // Fan-out с обработкой ошибок
    for (exchange, should_fail) in exchanges {
        let tx = tx.clone();
        thread::spawn(move || {
            let result = fetch_price(exchange, should_fail);
            tx.send(result).unwrap();
        });
    }

    drop(tx);

    // Fan-in с разделением успешных и неуспешных результатов
    let mut successful: Vec<ExchangePrice> = Vec::new();
    let mut failed: Vec<(String, FetchError)> = Vec::new();

    for result in rx {
        match result {
            Ok(price) => {
                println!("✅ {}: ${:.2}", price.exchange, price.price);
                successful.push(price);
            }
            Err((exchange, error)) => {
                println!("❌ {}: {:?}", exchange, error);
                failed.push((exchange, error));
            }
        }
    }

    println!("\n=== Итог ===");
    println!("Успешно: {}", successful.len());
    println!("Ошибки: {}", failed.len());

    if !successful.is_empty() {
        let avg_price: f64 = successful.iter().map(|p| p.price).sum::<f64>()
            / successful.len() as f64;
        println!("Средняя цена (из успешных): ${:.2}", avg_price);
    }

    // Для критических систем можно требовать минимум N успешных ответов
    let min_required = 3;
    if successful.len() < min_required {
        println!("\n⚠️  Предупреждение: получено меньше {} ответов, данные могут быть неточными", min_required);
    }
}
```

## Визуализация паттерна

```
┌─────────────────────────────────────────────────────────────┐
│                     FAN-OUT FAN-IN                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Входные       Fan-out           Fan-in        Результат   │
│   данные                                                    │
│                                                             │
│                 ┌─────────┐                                 │
│              ┌──│ Воркер 1│──┐                              │
│              │  └─────────┘  │                              │
│   ┌──────┐   │  ┌─────────┐  │   ┌──────────┐   ┌────────┐ │
│   │Задачи│───┼──│ Воркер 2│──┼───│Агрегатор │───│Результат│ │
│   └──────┘   │  └─────────┘  │   └──────────┘   └────────┘ │
│              │  ┌─────────┐  │                              │
│              └──│ Воркер N│──┘                              │
│                 └─────────┘                                 │
│                                                             │
│   • Распределение   • Параллельная   • Объединение          │
│     нагрузки          обработка        результатов          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Fan-out | Распределение задачи между несколькими параллельными обработчиками |
| Fan-in | Сбор результатов от всех обработчиков в одно место |
| Пул воркеров | Ограниченное количество воркеров для контроля ресурсов |
| `mpsc::channel` | Основной инструмент для Fan-in в Rust |
| Обработка ошибок | Важно отслеживать успешные и неуспешные результаты |
| Агрегация | Вычисление итоговых метрик из собранных данных |

## Преимущества паттерна

1. **Параллелизм**: обрабатываем много задач одновременно
2. **Масштабируемость**: легко добавить больше воркеров
3. **Отказоустойчивость**: сбой одного воркера не блокирует весь процесс
4. **Эффективность**: используем все доступные ресурсы CPU

## Домашнее задание

1. **Мультибиржевой арбитраж**: Реализуй систему, которая параллельно получает цены с 5 бирж и находит арбитражные возможности (разницу в ценах > 0.5%).

2. **Анализ корреляций**: Создай систему, которая параллельно рассчитывает корреляцию между 10 торговыми парами и выводит матрицу корреляций.

3. **Стресс-тестер портфеля**: Реализуй Fan-out Fan-in систему, которая параллельно запускает 100 сценариев Monte-Carlo для оценки риска портфеля.

4. **Сканер паттернов**: Создай сканер, который параллельно ищет технические паттерны (голова-плечи, двойное дно и т.д.) на 20 торговых парах и возвращает найденные паттерны с уверенностью.

## Навигация

[← Предыдущий день](../177-pipeline-parallel-processing/ru.md) | [Следующий день →](../179-select-waiting-multiple-channels/ru.md)
