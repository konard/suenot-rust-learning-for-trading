# День 152: Потоки — параллельно следим за биржами

## Аналогия из трейдинга

Представь, что ты хочешь одновременно следить за ценами на нескольких биржах: Binance, Bybit и OKX. Если ты будешь проверять их по очереди, пока ты смотришь на Binance, цена на OKX может уже измениться. Это как если бы у тебя был только один монитор и приходилось постоянно переключаться между вкладками.

**Потоки (threads)** — это как если бы ты нанял трёх помощников-трейдеров: один следит за Binance, другой за Bybit, третий за OKX. Все работают одновременно и сообщают тебе, когда видят интересную цену.

В Rust потоки позволяют выполнять несколько задач **параллельно**, что критически важно для:
- Мониторинга нескольких бирж одновременно
- Обработки большого объёма рыночных данных
- Расчёта сложных индикаторов без блокировки основной программы
- Быстрого реагирования на рыночные события

## Зачем нужны потоки в трейдинге?

```rust
// БЕЗ потоков: последовательная проверка
fn check_prices_sequential() {
    let binance_price = fetch_binance_price(); // 1 секунда
    let bybit_price = fetch_bybit_price();     // 1 секунда
    let okx_price = fetch_okx_price();         // 1 секунда
    // Итого: 3 секунды - слишком медленно для трейдинга!
}

// С потоками: параллельная проверка
fn check_prices_parallel() {
    // Все три запроса выполняются одновременно
    // Итого: ~1 секунда - именно то, что нужно!
}
```

## Создание первого потока

В Rust потоки создаются с помощью `std::thread::spawn`:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("Главный трейдер: Запускаю мониторинг бирж...");

    // Создаём поток для мониторинга Binance
    let binance_handle = thread::spawn(|| {
        for i in 1..=3 {
            println!("Binance: BTC = ${}", 42000 + i * 100);
            thread::sleep(Duration::from_millis(500));
        }
        println!("Binance: Мониторинг завершён");
    });

    // Главный поток продолжает работу
    for i in 1..=3 {
        println!("Главный: Анализирую рынок... шаг {}", i);
        thread::sleep(Duration::from_millis(300));
    }

    // Ждём завершения потока Binance
    binance_handle.join().unwrap();

    println!("Главный трейдер: Все потоки завершены");
}
```

Вывод программы (порядок может меняться — это и есть параллелизм!):
```
Главный трейдер: Запускаю мониторинг бирж...
Binance: BTC = $42100
Главный: Анализирую рынок... шаг 1
Главный: Анализирую рынок... шаг 2
Binance: BTC = $42200
Главный: Анализирую рынок... шаг 3
Binance: BTC = $42300
Binance: Мониторинг завершён
Главный трейдер: Все потоки завершены
```

## Несколько потоков: мониторинг трёх бирж

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("═══════════════════════════════════════");
    println!("    MULTI-EXCHANGE PRICE MONITOR");
    println!("═══════════════════════════════════════\n");

    // Запускаем потоки для каждой биржи
    let binance = thread::spawn(|| {
        monitor_exchange("Binance", 42000.0, 50.0)
    });

    let bybit = thread::spawn(|| {
        monitor_exchange("Bybit", 41980.0, 30.0)
    });

    let okx = thread::spawn(|| {
        monitor_exchange("OKX", 42010.0, 40.0)
    });

    // Ждём завершения всех потоков
    binance.join().unwrap();
    bybit.join().unwrap();
    okx.join().unwrap();

    println!("\nМониторинг всех бирж завершён!");
}

fn monitor_exchange(name: &str, base_price: f64, volatility: f64) {
    for tick in 1..=5 {
        // Симулируем изменение цены
        let price_change = ((tick as f64 * 1.5).sin() * volatility) as f64;
        let current_price = base_price + price_change;

        println!("[{}] BTC: ${:.2}", name, current_price);
        thread::sleep(Duration::from_millis(200));
    }
}
```

## Получение результата из потока

Потоки могут возвращать значения:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    println!("Запрашиваем цены с бирж...\n");

    // Каждый поток возвращает цену
    let binance_handle = thread::spawn(|| -> f64 {
        thread::sleep(Duration::from_millis(100));
        42150.50  // Симуляция полученной цены
    });

    let bybit_handle = thread::spawn(|| -> f64 {
        thread::sleep(Duration::from_millis(150));
        42145.25
    });

    let okx_handle = thread::spawn(|| -> f64 {
        thread::sleep(Duration::from_millis(120));
        42155.75
    });

    // Получаем результаты
    let binance_price = binance_handle.join().unwrap();
    let bybit_price = bybit_handle.join().unwrap();
    let okx_price = okx_handle.join().unwrap();

    // Анализируем полученные данные
    println!("Полученные цены:");
    println!("  Binance: ${:.2}", binance_price);
    println!("  Bybit:   ${:.2}", bybit_price);
    println!("  OKX:     ${:.2}", okx_price);

    let avg_price = (binance_price + bybit_price + okx_price) / 3.0;
    let best_buy = binance_price.min(bybit_price).min(okx_price);
    let best_sell = binance_price.max(bybit_price).max(okx_price);
    let spread = best_sell - best_buy;

    println!("\nАнализ:");
    println!("  Средняя цена:     ${:.2}", avg_price);
    println!("  Лучшая покупка:   ${:.2}", best_buy);
    println!("  Лучшая продажа:   ${:.2}", best_sell);
    println!("  Арбитражный спред: ${:.2} ({:.3}%)",
             spread,
             (spread / avg_price) * 100.0);
}
```

## Практический пример: Арбитражный сканер

```rust
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct ExchangePrice {
    exchange: String,
    symbol: String,
    bid: f64,      // Цена покупки
    ask: f64,      // Цена продажи
    timestamp: u64,
}

fn main() {
    println!("╔═════════════════════════════════════════════════╗");
    println!("║         ARBITRAGE SCANNER v1.0                  ║");
    println!("║    Поиск арбитражных возможностей               ║");
    println!("╚═════════════════════════════════════════════════╝\n");

    let start = Instant::now();

    // Параллельно получаем цены со всех бирж
    let binance = thread::spawn(|| fetch_exchange_price("Binance", 42100.0, 42105.0));
    let bybit = thread::spawn(|| fetch_exchange_price("Bybit", 42095.0, 42102.0));
    let okx = thread::spawn(|| fetch_exchange_price("OKX", 42110.0, 42118.0));
    let kraken = thread::spawn(|| fetch_exchange_price("Kraken", 42090.0, 42098.0));

    // Собираем результаты
    let prices = vec![
        binance.join().unwrap(),
        bybit.join().unwrap(),
        okx.join().unwrap(),
        kraken.join().unwrap(),
    ];

    let elapsed = start.elapsed();
    println!("Данные получены за {:?}\n", elapsed);

    // Выводим все цены
    println!("┌─────────────┬────────────┬────────────┬────────────┐");
    println!("│ Биржа       │ Bid        │ Ask        │ Spread     │");
    println!("├─────────────┼────────────┼────────────┼────────────┤");
    for price in &prices {
        let spread = price.ask - price.bid;
        println!("│ {:11} │ ${:9.2} │ ${:9.2} │ ${:9.2} │",
                 price.exchange, price.bid, price.ask, spread);
    }
    println!("└─────────────┴────────────┴────────────┴────────────┘\n");

    // Ищем арбитражные возможности
    find_arbitrage(&prices);
}

fn fetch_exchange_price(exchange: &str, bid: f64, ask: f64) -> ExchangePrice {
    // Симулируем задержку сети
    thread::sleep(Duration::from_millis(50 + (bid as u64 % 50)));

    ExchangePrice {
        exchange: exchange.to_string(),
        symbol: "BTC/USDT".to_string(),
        bid,
        ask,
        timestamp: 1234567890,
    }
}

fn find_arbitrage(prices: &[ExchangePrice]) {
    println!("Поиск арбитражных возможностей...\n");

    let mut opportunities_found = false;

    for buy_from in prices {
        for sell_to in prices {
            if buy_from.exchange == sell_to.exchange {
                continue;
            }

            // Покупаем по ask, продаём по bid
            let buy_price = buy_from.ask;
            let sell_price = sell_to.bid;
            let profit_percent = ((sell_price - buy_price) / buy_price) * 100.0;

            if profit_percent > 0.0 {
                opportunities_found = true;
                println!("АРБИТРАЖ НАЙДЕН!");
                println!("  Купить на {} по ${:.2}", buy_from.exchange, buy_price);
                println!("  Продать на {} по ${:.2}", sell_to.exchange, sell_price);
                println!("  Профит: {:.4}%\n", profit_percent);
            }
        }
    }

    if !opportunities_found {
        println!("Арбитражных возможностей не найдено.");
        println!("(Это нормально — рынки обычно эффективны)");
    }
}
```

## Потоки и владение: ключевой концепт

В Rust важно понимать, что каждый поток получает **владение** над данными, которые ему передаются:

```rust
use std::thread;

fn main() {
    let portfolio = vec!["BTC", "ETH", "SOL"];

    // move — передаём владение вектором в поток
    let handle = thread::spawn(move || {
        println!("Портфель в потоке: {:?}", portfolio);
        // portfolio теперь принадлежит этому потоку
    });

    // Эта строка не скомпилируется:
    // println!("Портфель в main: {:?}", portfolio);
    // ошибка: value borrowed after move

    handle.join().unwrap();
}
```

Для работы с общими данными между потоками нам понадобятся специальные инструменты (`Arc`, `Mutex`), которые мы изучим в следующих главах.

## Обработка паники в потоках

Потоки могут паниковать, и это не завершит основную программу:

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        // Симулируем критическую ошибку
        panic!("Ошибка подключения к бирже!");
    });

    // join() вернёт Err, если поток запаниковал
    match handle.join() {
        Ok(_) => println!("Поток завершился успешно"),
        Err(e) => {
            println!("Поток завершился с ошибкой!");
            // Можно попытаться извлечь сообщение
            if let Some(msg) = e.downcast_ref::<&str>() {
                println!("Причина: {}", msg);
            }
        }
    }

    println!("Главный поток продолжает работу");
}
```

## Что мы узнали

| Концепция | Описание | Аналогия в трейдинге |
|-----------|----------|----------------------|
| `thread::spawn` | Создание нового потока | Нанимаем помощника-трейдера |
| `handle.join()` | Ожидание завершения потока | Ждём отчёт от помощника |
| `move` closure | Передача владения в поток | Даём помощнику свои данные |
| Возврат значения | Поток возвращает результат | Помощник сообщает цену |
| Обработка паники | Поток может упасть безопасно | Помощник может ошибиться |

## Практические задания

### Задание 1: Параллельный расчёт индикаторов
Создайте программу, которая параллельно рассчитывает три индикатора для массива цен:
- Скользящее среднее (SMA)
- Максимум
- Минимум

### Задание 2: Мониторинг нескольких криптовалют
Напишите программу, где каждый поток "мониторит" отдельную криптовалюту (BTC, ETH, SOL, DOT) и возвращает "последнюю цену".

### Задание 3: Таймер исполнения
Создайте функцию, которая запускает задачу в отдельном потоке и измеряет время её выполнения.

### Задание 4: Пул запросов
Напишите программу, которая создаёт 10 потоков, каждый из которых "делает запрос к API" (sleep на случайное время) и возвращает результат.

## Домашнее задание

1. **Арбитражный сканер**: Расширьте пример арбитражного сканера, добавив:
   - Расчёт прибыли в абсолютных числах для позиции $10,000
   - Учёт комиссий бирж (0.1% maker, 0.1% taker)
   - Фильтрацию возможностей с профитом > 0.05%

2. **Параллельный бэктест**: Напишите программу, которая:
   - Загружает исторические данные (можно симулировать)
   - Параллельно тестирует три разные стратегии
   - Возвращает результаты каждой стратегии

3. **Мульти-таймфрейм анализ**: Создайте программу, где:
   - Один поток анализирует 1-минутные свечи
   - Второй поток — 5-минутные
   - Третий — 15-минутные
   - Главный поток собирает и выводит результаты

## Навигация

[← Предыдущий день](../151-project-historical-data-loader/ru.md) | [Следующий день →](../153-thread-spawn/ru.md)
