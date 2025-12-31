# День 27: Shadowing — обновление цены с тем же именем

## Аналогия из трейдинга

Представь торговый терминал, где цена актива постоянно обновляется. Каждую секунду приходит **новая цена**, которая **заменяет** предыдущую. При этом мы всё ещё называем её "текущая цена" — имя остаётся прежним, но значение полностью новое.

В Rust это называется **shadowing** (затенение) — мы можем объявить новую переменную с тем же именем, и она "затенит" предыдущую.

## Что такое Shadowing?

Shadowing позволяет переобъявить переменную с тем же именем:

```rust
fn main() {
    let price = 42000;
    println!("Старая цена: {}", price);

    let price = 43500;  // Новая переменная с тем же именем
    println!("Новая цена: {}", price);
}
```

Вывод:
```
Старая цена: 42000
Новая цена: 43500
```

**Важно:** Это НЕ изменение переменной (как с `mut`), а создание **новой** переменной.

## Shadowing vs Mutability

В чём разница между shadowing и `mut`?

### С mut — изменение существующей переменной:

```rust
fn main() {
    let mut price = 42000.0;
    price = 43500.0;  // Меняем значение той же переменной
    // Тип изменить НЕЛЬЗЯ!
    // price = "сорок две тысячи";  // ОШИБКА!
}
```

### С shadowing — создание новой переменной:

```rust
fn main() {
    let price = "42000";         // Строка
    let price = price.len();     // Теперь число (usize)!
    println!("Длина строки: {}", price);
}
```

**Ключевое отличие:** Shadowing позволяет изменить тип переменной!

## Зачем нужен Shadowing в трейдинге?

### 1. Преобразование данных из API

Данные часто приходят в виде строк:

```rust
fn main() {
    // Цена пришла из API как строка
    let btc_price = "42567.89";
    println!("Получена цена (строка): {}", btc_price);

    // Преобразуем в число для расчётов
    let btc_price: f64 = btc_price.parse().unwrap();
    println!("Цена для расчётов: {} USDT", btc_price);

    // Теперь можем считать
    let position_size = 0.1;
    let position_value = btc_price * position_size;
    println!("Стоимость позиции: {} USDT", position_value);
}
```

### 2. Нормализация цен

Биржи могут отдавать цены в разных форматах:

```rust
fn main() {
    // Цена в центах (как на некоторых API)
    let stock_price = 15099;  // $150.99 в центах
    println!("Цена в центах: {}", stock_price);

    // Преобразуем в доллары
    let stock_price = stock_price as f64 / 100.0;
    println!("Цена в долларах: ${:.2}", stock_price);

    // Рассчитываем комиссию
    let commission = stock_price * 0.001;
    println!("Комиссия: ${:.2}", commission);
}
```

### 3. Пошаговая обработка данных

```rust
fn main() {
    // Сырые данные
    let ticker = "  BTC/USDT  ";
    println!("Сырой тикер: '{}'", ticker);

    // Убираем пробелы
    let ticker = ticker.trim();
    println!("После trim: '{}'", ticker);

    // Разделяем на базовую и котируемую валюту
    let parts: Vec<&str> = ticker.split('/').collect();
    let base_currency = parts[0];
    let quote_currency = parts[1];

    println!("Базовая валюта: {}", base_currency);
    println!("Котируемая валюта: {}", quote_currency);
}
```

## Практические примеры

### Пример 1: Обработка биржевых данных

```rust
fn main() {
    println!("=== Обработка данных с биржи ===\n");

    // Данные приходят как строки из JSON
    let open = "42000.50";
    let high = "43500.75";
    let low = "41800.25";
    let close = "43200.00";
    let volume = "1234.56";

    // Преобразуем в числа для анализа
    let open: f64 = open.parse().unwrap();
    let high: f64 = high.parse().unwrap();
    let low: f64 = low.parse().unwrap();
    let close: f64 = close.parse().unwrap();
    let volume: f64 = volume.parse().unwrap();

    // Теперь можем анализировать
    let price_range = high - low;
    let price_change = close - open;
    let change_percent = (price_change / open) * 100.0;

    println!("Open:   ${:.2}", open);
    println!("High:   ${:.2}", high);
    println!("Low:    ${:.2}", low);
    println!("Close:  ${:.2}", close);
    println!("Volume: {:.2} BTC", volume);
    println!("\n--- Анализ ---");
    println!("Диапазон дня: ${:.2}", price_range);
    println!("Изменение: ${:.2} ({:+.2}%)", price_change, change_percent);
}
```

### Пример 2: Расчёт размера позиции

```rust
fn main() {
    println!("=== Расчёт размера позиции ===\n");

    // Входные данные
    let balance = 10000.0;  // USDT
    let risk_percent = 2.0; // 2%
    let entry_price = 42000.0;
    let stop_loss = 41000.0;

    // Рассчитываем риск в долларах
    let risk = balance * (risk_percent / 100.0);
    println!("Риск на сделку: ${:.2}", risk);

    // Рассчитываем размер стопа
    let stop_distance = entry_price - stop_loss;
    println!("Размер стопа: ${:.2}", stop_distance);

    // Размер позиции в BTC
    let position_size = risk / stop_distance;
    println!("Размер позиции: {:.6} BTC", position_size);

    // Преобразуем в стоимость
    let position_size = position_size * entry_price;
    println!("Стоимость позиции: ${:.2}", position_size);

    // Проверяем плечо
    let leverage = position_size / balance;
    println!("Требуемое плечо: {:.2}x", leverage);
}
```

### Пример 3: Валидация и нормализация ордера

```rust
fn main() {
    println!("=== Валидация ордера ===\n");

    // Данные от пользователя (могут быть некорректными)
    let quantity = "  0.001500  ";
    let price = "42000.123456789";
    let side = "BUY";

    println!("Входные данные:");
    println!("  Количество: '{}'", quantity);
    println!("  Цена: '{}'", price);
    println!("  Сторона: '{}'", side);

    // Очищаем и парсим количество
    let quantity = quantity.trim();
    let quantity: f64 = quantity.parse().unwrap();

    // Округляем до шага размера лота (0.0001 BTC)
    let quantity = (quantity * 10000.0).floor() / 10000.0;
    println!("\nОчищенное количество: {:.4} BTC", quantity);

    // Парсим и округляем цену
    let price = price.trim();
    let price: f64 = price.parse().unwrap();

    // Округляем до тика (0.01 USDT)
    let price = (price * 100.0).floor() / 100.0;
    println!("Округлённая цена: ${:.2}", price);

    // Нормализуем сторону
    let side = side.trim().to_uppercase();
    println!("Нормализованная сторона: {}", side);

    // Итоговый ордер
    println!("\n--- Итоговый ордер ---");
    println!("{} {:.4} BTC @ ${:.2}", side, quantity, price);
}
```

### Пример 4: Обработка истории сделок

```rust
fn main() {
    println!("=== Анализ истории сделок ===\n");

    // Результаты сделок (PnL)
    let trades = [150.0, -80.0, 200.0, -50.0, 300.0, -120.0, 180.0];

    // Подсчёт статистики
    let mut total_pnl = 0.0;
    let mut winning_pnl = 0.0;
    let mut losing_pnl = 0.0;
    let mut wins = 0;
    let mut losses = 0;

    for trade in trades {
        total_pnl += trade;
        if trade > 0.0 {
            winning_pnl += trade;
            wins += 1;
        } else {
            losing_pnl += trade;
            losses += 1;
        }
    }

    // Shadowing для расчёта метрик
    let win_rate = wins as f64 / (wins + losses) as f64;
    let win_rate = win_rate * 100.0;  // Преобразуем в проценты

    let avg_win = winning_pnl / wins as f64;
    let avg_loss = (losing_pnl / losses as f64).abs();

    let profit_factor = winning_pnl / losing_pnl.abs();

    println!("Всего сделок: {}", trades.len());
    println!("Прибыльных: {}", wins);
    println!("Убыточных: {}", losses);
    println!("\nОбщий PnL: ${:.2}", total_pnl);
    println!("Win Rate: {:.1}%", win_rate);
    println!("Средняя прибыль: ${:.2}", avg_win);
    println!("Средний убыток: ${:.2}", avg_loss);
    println!("Profit Factor: {:.2}", profit_factor);
}
```

## Shadowing во вложенных областях

Shadowing работает и во вложенных блоках:

```rust
fn main() {
    let price = 42000.0;
    println!("Внешний: {}", price);

    {
        // Это новая переменная только внутри блока
        let price = 43000.0;
        println!("Внутренний: {}", price);
    }

    // Здесь снова видна внешняя переменная
    println!("Снова внешний: {}", price);
}
```

Вывод:
```
Внешний: 42000
Внутренний: 43000
Снова внешний: 42000
```

## Когда использовать Shadowing?

### Используй Shadowing когда:
- Преобразуешь данные из строки в число
- Нормализуешь данные (центы → доллары)
- Пошагово обрабатываешь данные
- Логически это "те же данные, но в другом виде"

### НЕ используй Shadowing когда:
- Нужно обновлять значение в цикле (используй `mut`)
- Нужен доступ к обоим значениям одновременно
- Это может запутать читающего код

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| Shadowing | Переобъявление переменной с тем же именем |
| Отличие от mut | Shadowing создаёт новую переменную |
| Изменение типа | Shadowing позволяет менять тип |
| Области видимости | Shadowing работает в блоках |

## Упражнения

### Упражнение 1: Конвертация валют
Напиши программу, которая:
- Получает сумму в рублях как строку `"85000.50"`
- Конвертирует в число
- Переводит в доллары (курс 85)
- Переводит в BTC (курс $42000)

### Упражнение 2: Обработка тикера
Напиши программу, которая:
- Получает тикер `"  eth_usdt  "`
- Убирает пробелы
- Преобразует в верхний регистр
- Заменяет `_` на `/`
- Выводит результат: `"ETH/USDT"`

### Упражнение 3: Парсинг ордера
Напиши программу, которая обрабатывает ордер:
```
price: "42150.123"
amount: "0.0015678"
side: "sell"
```
И выводит нормализованный ордер:
```
SELL 0.0015 BTC @ $42150.12
```

## Домашнее задание

1. Создай симулятор обработки данных с биржи:
   - Входные данные: строки с ценами OHLCV
   - Парсинг и преобразование в числа
   - Расчёт технических индикаторов (SMA, диапазон)

2. Создай валидатор торгового сигнала:
   - Входные данные: сырой сигнал в виде строк
   - Нормализация (trim, uppercase)
   - Преобразование цен и объёмов
   - Вывод готового сигнала для исполнения

3. Реализуй конвертер между биржами:
   - Binance формат: `"BTCUSDT"`
   - Kraken формат: `"XBT/USD"`
   - Преобразование между форматами

## Навигация

[← Предыдущий день](../026-*/ru.md) | [Следующий день →](../028-*/ru.md)
