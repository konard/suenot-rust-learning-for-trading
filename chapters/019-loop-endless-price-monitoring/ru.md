# День 19: loop — Бесконечный мониторинг цен

## Аналогия из трейдинга

Представь торгового бота, который **непрерывно** следит за рынком. Он не останавливается после одной проверки — он работает 24/7, постоянно отслеживая цены, объёмы и сигналы. Именно так работает `loop` в Rust — бесконечный цикл, который выполняется, пока его явно не остановят.

В реальной торговле это:
- Мониторинг цен в реальном времени
- Ожидание сигнала на вход в сделку
- Поддержание WebSocket-соединения с биржей
- Постоянная проверка стоп-лоссов

## Базовый синтаксис loop

```rust
fn main() {
    let mut price = 42000.0;
    let mut iteration = 0;

    loop {
        iteration += 1;
        println!("Проверка #{}: BTC = ${:.2}", iteration, price);

        // Симуляция изменения цены
        price += 100.0;

        if iteration >= 5 {
            println!("Мониторинг завершён");
            break;  // Выход из цикла
        }
    }

    println!("Итоговая цена: ${:.2}", price);
}
```

**Ключевой момент:** `loop` работает вечно, пока не встретит `break`.

## break — выход из цикла

```rust
fn main() {
    let target_price = 43000.0;
    let mut current_price = 42000.0;
    let mut checks = 0;

    loop {
        checks += 1;
        current_price += 150.0;  // Симуляция роста цены

        println!("Проверка {}: ${:.2}", checks, current_price);

        if current_price >= target_price {
            println!("Целевая цена достигнута!");
            break;
        }

        if checks > 100 {
            println!("Превышен лимит проверок");
            break;
        }
    }

    println!("Всего проверок: {}", checks);
}
```

## continue — пропуск итерации

```rust
fn main() {
    let prices = [42000.0, 0.0, 42500.0, -100.0, 43000.0, 42800.0];
    let mut index = 0;
    let mut valid_count = 0;
    let mut sum = 0.0;

    loop {
        if index >= prices.len() {
            break;
        }

        let price = prices[index];
        index += 1;

        // Пропускаем невалидные цены
        if price <= 0.0 {
            println!("Пропуск невалидной цены: {}", price);
            continue;
        }

        valid_count += 1;
        sum += price;
        println!("Валидная цена #{}: ${:.2}", valid_count, price);
    }

    if valid_count > 0 {
        println!("Средняя цена: ${:.2}", sum / valid_count as f64);
    }
}
```

## loop с возвратом значения

Уникальная особенность Rust — `loop` может возвращать значение через `break`:

```rust
fn main() {
    let mut price = 42000.0;
    let buy_threshold = 41000.0;
    let sell_threshold = 44000.0;

    let action = loop {
        // Симуляция изменения цены
        price += (price * 0.01) * if price > 43000.0 { -1.0 } else { 1.0 };

        println!("Текущая цена: ${:.2}", price);

        if price <= buy_threshold {
            break "BUY";  // Возвращаем значение
        }

        if price >= sell_threshold {
            break "SELL";
        }

        // Защита от бесконечного цикла в примере
        if price > 50000.0 || price < 35000.0 {
            break "HOLD";
        }
    };

    println!("Рекомендация: {}", action);
}
```

## Вложенные циклы и метки

```rust
fn main() {
    let exchanges = ["Binance", "Coinbase", "Kraken"];
    let assets = ["BTC", "ETH", "SOL"];

    'exchange_loop: loop {
        for exchange in &exchanges {
            for asset in &assets {
                let price = get_mock_price(exchange, asset);

                println!("{} на {}: ${:.2}", asset, exchange, price);

                if price > 50000.0 {
                    println!("Найдена аномальная цена! Остановка.");
                    break 'exchange_loop;  // Выход из внешнего цикла
                }
            }
        }
        break;  // Выходим после одной итерации для примера
    }

    println!("Сканирование завершено");
}

fn get_mock_price(exchange: &str, asset: &str) -> f64 {
    // Симуляция цен
    match (exchange, asset) {
        ("Binance", "BTC") => 42150.0,
        ("Coinbase", "BTC") => 42200.0,
        ("Kraken", "BTC") => 55000.0,  // Аномалия
        (_, "ETH") => 2250.0,
        (_, "SOL") => 98.0,
        _ => 0.0,
    }
}
```

## Практический пример: Торговый бот-монитор

```rust
fn main() {
    println!("╔════════════════════════════════════╗");
    println!("║     PRICE MONITOR STARTED          ║");
    println!("╚════════════════════════════════════╝");

    let mut btc_price = 42000.0;
    let mut eth_price = 2200.0;

    let stop_loss_btc = 40000.0;
    let take_profit_btc = 45000.0;

    let mut tick = 0;
    let max_ticks = 20;

    let result = loop {
        tick += 1;

        // Симуляция изменения цен
        btc_price += simulate_price_change(btc_price);
        eth_price += simulate_price_change(eth_price);

        println!("\n[Tick {}]", tick);
        println!("  BTC: ${:.2}", btc_price);
        println!("  ETH: ${:.2}", eth_price);

        // Проверка стоп-лосса
        if btc_price <= stop_loss_btc {
            break TradeSignal::StopLoss(btc_price);
        }

        // Проверка тейк-профита
        if btc_price >= take_profit_btc {
            break TradeSignal::TakeProfit(btc_price);
        }

        // Защита от бесконечного цикла
        if tick >= max_ticks {
            break TradeSignal::Timeout(btc_price);
        }
    };

    println!("\n╔════════════════════════════════════╗");
    match result {
        TradeSignal::StopLoss(price) => {
            println!("║  STOP LOSS TRIGGERED              ║");
            println!("║  Exit price: ${:.2}           ║", price);
        }
        TradeSignal::TakeProfit(price) => {
            println!("║  TAKE PROFIT REACHED              ║");
            println!("║  Exit price: ${:.2}           ║", price);
        }
        TradeSignal::Timeout(price) => {
            println!("║  MONITORING TIMEOUT               ║");
            println!("║  Current price: ${:.2}         ║", price);
        }
    }
    println!("╚════════════════════════════════════╝");
}

enum TradeSignal {
    StopLoss(f64),
    TakeProfit(f64),
    Timeout(f64),
}

fn simulate_price_change(price: f64) -> f64 {
    // Простая симуляция: случайное изменение ±2%
    let change_percent = ((price as i64 % 7) as f64 - 3.0) / 100.0;
    price * change_percent
}
```

## Мониторинг нескольких активов

```rust
fn main() {
    let mut portfolio = Portfolio {
        btc: Asset { symbol: "BTC", price: 42000.0, quantity: 0.5 },
        eth: Asset { symbol: "ETH", price: 2200.0, quantity: 5.0 },
        sol: Asset { symbol: "SOL", price: 95.0, quantity: 100.0 },
    };

    let mut tick = 0;

    loop {
        tick += 1;

        // Обновление цен
        update_prices(&mut portfolio, tick);

        // Расчёт общей стоимости
        let total_value = calculate_portfolio_value(&portfolio);

        println!("\n═══ Tick {} ═══", tick);
        print_portfolio(&portfolio);
        println!("Total Value: ${:.2}", total_value);

        // Проверка условий выхода
        if total_value < 30000.0 {
            println!("\n⚠️  Portfolio value dropped below $30,000!");
            break;
        }

        if total_value > 50000.0 {
            println!("\n✓ Target portfolio value reached!");
            break;
        }

        if tick >= 10 {
            println!("\n— Monitoring session ended —");
            break;
        }
    }
}

struct Asset {
    symbol: &'static str,
    price: f64,
    quantity: f64,
}

struct Portfolio {
    btc: Asset,
    eth: Asset,
    sol: Asset,
}

fn update_prices(portfolio: &mut Portfolio, tick: i32) {
    // Симуляция изменения цен
    portfolio.btc.price *= 1.0 + ((tick % 5) as f64 - 2.0) / 100.0;
    portfolio.eth.price *= 1.0 + ((tick % 4) as f64 - 1.5) / 100.0;
    portfolio.sol.price *= 1.0 + ((tick % 6) as f64 - 2.5) / 100.0;
}

fn calculate_portfolio_value(portfolio: &Portfolio) -> f64 {
    portfolio.btc.price * portfolio.btc.quantity
        + portfolio.eth.price * portfolio.eth.quantity
        + portfolio.sol.price * portfolio.sol.quantity
}

fn print_portfolio(portfolio: &Portfolio) {
    println!(
        "  {} ${:.2} x {} = ${:.2}",
        portfolio.btc.symbol,
        portfolio.btc.price,
        portfolio.btc.quantity,
        portfolio.btc.price * portfolio.btc.quantity
    );
    println!(
        "  {} ${:.2} x {} = ${:.2}",
        portfolio.eth.symbol,
        portfolio.eth.price,
        portfolio.eth.quantity,
        portfolio.eth.price * portfolio.eth.quantity
    );
    println!(
        "  {} ${:.2} x {} = ${:.2}",
        portfolio.sol.symbol,
        portfolio.sol.price,
        portfolio.sol.quantity,
        portfolio.sol.price * portfolio.sol.quantity
    );
}
```

## Паттерны использования loop

```rust
fn main() {
    // 1. Ожидание условия
    let found_price = wait_for_price(42500.0);
    println!("Дождались цены: ${:.2}", found_price);

    // 2. Retry-логика
    match fetch_with_retry(3) {
        Some(data) => println!("Данные получены: {}", data),
        None => println!("Не удалось получить данные"),
    }

    // 3. Обработка очереди
    process_order_queue();
}

fn wait_for_price(target: f64) -> f64 {
    let mut current = 42000.0;

    loop {
        current += 50.0;  // Симуляция роста

        if current >= target {
            break current;
        }
    }
}

fn fetch_with_retry(max_attempts: i32) -> Option<String> {
    let mut attempt = 0;

    loop {
        attempt += 1;
        println!("Попытка {} из {}", attempt, max_attempts);

        // Симуляция успеха на 3-й попытке
        if attempt >= 3 {
            break Some(String::from("Market data"));
        }

        if attempt >= max_attempts {
            break None;
        }
    }
}

fn process_order_queue() {
    let mut orders = vec!["BUY BTC", "SELL ETH", "BUY SOL"];

    loop {
        if orders.is_empty() {
            println!("Очередь ордеров пуста");
            break;
        }

        let order = orders.remove(0);
        println!("Обработка: {}", order);
    }
}
```

## Что мы узнали

| Конструкция | Описание | Пример |
|-------------|----------|--------|
| `loop { }` | Бесконечный цикл | Мониторинг в реальном времени |
| `break` | Выход из цикла | Срабатывание условия |
| `break value` | Выход с возвратом | Результат поиска |
| `continue` | Пропуск итерации | Фильтрация данных |
| `'label: loop` | Именованный цикл | Вложенные циклы |

## Упражнения

1. **Симулятор скальпинга:** Создай цикл, который покупает при падении цены на 0.5% и продаёт при росте на 0.3%. Подсчитай количество сделок и итоговый PnL.

2. **Охотник за арбитражем:** Напиши программу, которая в цикле сравнивает цены на двух биржах и выходит, когда находит разницу более 1%.

3. **Накопление позиции:** Симулируй DCA (Dollar Cost Averaging) — каждую итерацию покупай на фиксированную сумму по текущей цене, пока не накопишь целевое количество актива.

4. **Алерт-система:** Создай монитор, который отслеживает несколько ценовых уровней и выводит соответствующие алерты при их достижении.

## Домашнее задание

1. Напиши функцию `monitor_until_signal(prices: &[f64], buy_level: f64, sell_level: f64) -> TradeAction`, которая в цикле перебирает цены и возвращает первый сигнал.

2. Создай систему мониторинга с множественными условиями выхода: стоп-лосс, тейк-профит, таймаут и максимальная просадка.

3. Реализуй симулятор маркет-мейкера, который постоянно обновляет bid/ask спред и выходит при определённом уровне прибыли.

4. Напиши торгового бота с retry-логикой для подключения к бирже: бот пытается подключиться, при неудаче ждёт и пробует снова (максимум N попыток).

## Навигация

[← Предыдущий день](../018-if-else-trading-decisions/ru.md) | [Следующий день →](../020-while-position-holding/ru.md)
