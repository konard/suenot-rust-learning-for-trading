# День 29: Парсинг строк — конвертация ввода в число

## Аналогия из трейдинга

В алготрейдинге данные часто приходят в текстовом формате:
- **API ответы:** `"price": "42150.50"` — цена как строка
- **CSV файлы:** `BTC,42150.50,1.5` — все поля текстовые
- **Пользовательский ввод:** `"Введите количество: 0.5"`
- **WebSocket сообщения:** `{"bid": "42100", "ask": "42200"}`

Прежде чем выполнять вычисления — рассчитать прибыль, проверить лимиты, сравнить цены — нужно преобразовать строки в числа.

## Метод parse()

Основной способ конвертации строки в число в Rust:

```rust
fn main() {
    let price_str = "42150.50";

    // parse() возвращает Result<T, E>
    let price: f64 = price_str.parse().unwrap();

    println!("Цена: {}", price);
    println!("Цена +10%: {}", price * 1.1);
}
```

**Важно:** Нужно указать тип результата — Rust должен знать, во что парсить.

## Разные способы указать тип

```rust
fn main() {
    let amount_str = "100";

    // Способ 1: Аннотация типа переменной
    let amount1: i32 = amount_str.parse().unwrap();

    // Способ 2: Turbofish синтаксис
    let amount2 = amount_str.parse::<i32>().unwrap();

    // Способ 3: Тип выводится из использования
    let amount3 = amount_str.parse().unwrap();
    let _doubled: i32 = amount3 * 2;

    println!("amount1: {}", amount1);
    println!("amount2: {}", amount2);
}
```

## Парсинг разных типов чисел

```rust
fn main() {
    // Целые числа
    let shares: i32 = "1000".parse().unwrap();
    let big_volume: i64 = "1000000000000".parse().unwrap();
    let order_id: u64 = "9876543210".parse().unwrap();

    // Числа с плавающей точкой
    let price: f64 = "42150.50".parse().unwrap();
    let ratio: f32 = "0.95".parse().unwrap();

    println!("Shares: {}", shares);
    println!("Volume: {}", big_volume);
    println!("Order ID: {}", order_id);
    println!("Price: {}", price);
    println!("Ratio: {}", ratio);
}
```

## Обработка ошибок парсинга

`parse()` возвращает `Result`, потому что парсинг может не удаться:

```rust
fn main() {
    let valid = "42150.50";
    let invalid = "not_a_number";
    let empty = "";
    let with_spaces = " 100 ";

    // unwrap() вызовет панику при ошибке
    let price: f64 = valid.parse().unwrap();
    println!("Valid: {}", price);

    // Безопасный способ с match
    match invalid.parse::<f64>() {
        Ok(num) => println!("Parsed: {}", num),
        Err(e) => println!("Error parsing '{}': {}", invalid, e),
    }

    // unwrap_or — значение по умолчанию
    let default_price: f64 = invalid.parse().unwrap_or(0.0);
    println!("Default: {}", default_price);

    // unwrap_or_else — ленивое значение по умолчанию
    let fallback: f64 = empty.parse().unwrap_or_else(|_| {
        println!("Пустая строка, использую рыночную цену");
        42000.0
    });
    println!("Fallback: {}", fallback);

    // trim() помогает с пробелами
    let trimmed: i32 = with_spaces.trim().parse().unwrap();
    println!("Trimmed: {}", trimmed);
}
```

## Практический пример: парсинг цены из API

```rust
fn main() {
    // Симулируем ответ API
    let api_response = r#"{"symbol": "BTCUSDT", "price": "42150.50"}"#;

    // Простой парсинг (в реальности используйте serde_json)
    if let Some(start) = api_response.find("\"price\": \"") {
        let price_start = start + 10;
        if let Some(end) = api_response[price_start..].find("\"") {
            let price_str = &api_response[price_start..price_start + end];

            match price_str.parse::<f64>() {
                Ok(price) => {
                    println!("BTC Price: ${:.2}", price);
                    println!("Buy 0.1 BTC: ${:.2}", price * 0.1);
                }
                Err(e) => println!("Failed to parse price: {}", e),
            }
        }
    }
}
```

## Практический пример: парсинг торговой команды

```rust
fn main() {
    let commands = [
        "buy BTC 0.5 42000",
        "sell ETH 10 2500.50",
        "buy DOGE 10000 0.08",
        "invalid command",
    ];

    for cmd in commands {
        println!("\nКоманда: '{}'", cmd);
        parse_trade_command(cmd);
    }
}

fn parse_trade_command(command: &str) {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.len() != 4 {
        println!("  Ошибка: неверный формат команды");
        return;
    }

    let action = parts[0];
    let symbol = parts[1];

    // Парсинг количества
    let quantity: f64 = match parts[2].parse() {
        Ok(q) => q,
        Err(_) => {
            println!("  Ошибка: неверное количество '{}'", parts[2]);
            return;
        }
    };

    // Парсинг цены
    let price: f64 = match parts[3].parse() {
        Ok(p) => p,
        Err(_) => {
            println!("  Ошибка: неверная цена '{}'", parts[3]);
            return;
        }
    };

    let total = quantity * price;

    println!("  Action: {}", action.to_uppercase());
    println!("  Symbol: {}", symbol);
    println!("  Quantity: {}", quantity);
    println!("  Price: ${:.2}", price);
    println!("  Total: ${:.2}", total);
}
```

## Практический пример: парсинг CSV строки

```rust
fn main() {
    // Данные о сделках в CSV формате
    let csv_data = "BTCUSDT,42150.50,0.5,BUY
ETHUSDT,2500.00,10.0,SELL
DOGEUSDT,0.08,50000,BUY";

    println!("=== Парсинг сделок из CSV ===\n");

    let mut total_volume = 0.0;

    for (i, line) in csv_data.lines().enumerate() {
        println!("Сделка #{}:", i + 1);

        let fields: Vec<&str> = line.split(',').collect();

        if fields.len() != 4 {
            println!("  Ошибка: неверный формат строки\n");
            continue;
        }

        let symbol = fields[0];

        let price: f64 = match fields[1].parse() {
            Ok(p) => p,
            Err(_) => {
                println!("  Ошибка парсинга цены\n");
                continue;
            }
        };

        let quantity: f64 = match fields[2].parse() {
            Ok(q) => q,
            Err(_) => {
                println!("  Ошибка парсинга количества\n");
                continue;
            }
        };

        let side = fields[3];
        let volume = price * quantity;
        total_volume += volume;

        println!("  {} {} {} @ ${:.2}", side, quantity, symbol, price);
        println!("  Объём: ${:.2}\n", volume);
    }

    println!("Общий объём: ${:.2}", total_volume);
}
```

## Практический пример: валидация ордера

```rust
fn main() {
    // Пользовательский ввод (симуляция)
    let inputs = [
        ("1000", "42000.50"),    // Валидный
        ("abc", "42000"),        // Невалидное количество
        ("100", ""),             // Пустая цена
        ("-50", "42000"),        // Отрицательное количество
        ("100", "-100"),         // Отрицательная цена
    ];

    for (qty_str, price_str) in inputs {
        println!("\nВвод: количество='{}', цена='{}'", qty_str, price_str);

        match validate_order(qty_str, price_str) {
            Ok((qty, price)) => {
                println!("  Ордер валиден!");
                println!("  Количество: {}", qty);
                println!("  Цена: ${:.2}", price);
                println!("  Сумма: ${:.2}", qty * price);
            }
            Err(e) => println!("  Ошибка: {}", e),
        }
    }
}

fn validate_order(qty_str: &str, price_str: &str) -> Result<(f64, f64), String> {
    // Проверка на пустоту
    if qty_str.is_empty() {
        return Err("Количество не может быть пустым".to_string());
    }
    if price_str.is_empty() {
        return Err("Цена не может быть пустой".to_string());
    }

    // Парсинг количества
    let quantity: f64 = qty_str
        .trim()
        .parse()
        .map_err(|_| format!("Невозможно распарсить количество: '{}'", qty_str))?;

    // Парсинг цены
    let price: f64 = price_str
        .trim()
        .parse()
        .map_err(|_| format!("Невозможно распарсить цену: '{}'", price_str))?;

    // Валидация значений
    if quantity <= 0.0 {
        return Err("Количество должно быть положительным".to_string());
    }
    if price <= 0.0 {
        return Err("Цена должна быть положительной".to_string());
    }

    Ok((quantity, price))
}
```

## Парсинг с очисткой данных

```rust
fn main() {
    // Грязные данные из разных источников
    let dirty_prices = [
        "  42150.50  ",     // Пробелы
        "$42,150.50",       // Символ валюты и запятые
        "42150.50 USD",     // Суффикс валюты
        "42_150.50",        // Подчёркивания
        "+42150.50",        // Знак плюс
    ];

    for dirty in dirty_prices {
        let clean = clean_price_string(dirty);
        match clean.parse::<f64>() {
            Ok(price) => println!("'{}' -> {:.2}", dirty, price),
            Err(_) => println!("'{}' -> не удалось распарсить", dirty),
        }
    }
}

fn clean_price_string(s: &str) -> String {
    s.trim()
        .replace("$", "")
        .replace(",", "")
        .replace("_", "")
        .replace(" USD", "")
        .replace(" USDT", "")
        .replace("+", "")
}
```

## Парсинг разных систем счисления

```rust
fn main() {
    // Иногда ID или коды приходят в разных форматах

    // Десятичная (по умолчанию)
    let decimal: i32 = "255".parse().unwrap();

    // Шестнадцатеричная (0x prefix или from_str_radix)
    let hex = i32::from_str_radix("FF", 16).unwrap();
    let hex2 = i32::from_str_radix("ff", 16).unwrap();  // Регистр не важен

    // Двоичная
    let binary = i32::from_str_radix("11111111", 2).unwrap();

    // Восьмеричная
    let octal = i32::from_str_radix("377", 8).unwrap();

    println!("Decimal '255': {}", decimal);
    println!("Hex 'FF': {}", hex);
    println!("Hex 'ff': {}", hex2);
    println!("Binary '11111111': {}", binary);
    println!("Octal '377': {}", octal);

    // Все равны 255
    assert_eq!(decimal, hex);
    assert_eq!(hex, binary);
    assert_eq!(binary, octal);
    println!("\nВсе значения равны 255!");
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `parse()` | Основной метод парсинга строки в число |
| Turbofish `::<T>` | Синтаксис указания типа для parse |
| `Result<T, E>` | Парсинг может не удаться |
| `unwrap()` | Паникует при ошибке |
| `unwrap_or()` | Значение по умолчанию |
| `trim()` | Убирает пробелы перед парсингом |
| `from_str_radix` | Парсинг в разных системах счисления |

## Упражнения

### Упражнение 1: Калькулятор прибыли

Напишите функцию, которая принимает строки с ценой покупки, ценой продажи и количеством, и возвращает прибыль/убыток:

```rust
fn calculate_profit(buy_price: &str, sell_price: &str, quantity: &str) -> Result<f64, String> {
    // Ваш код здесь
}

// Использование:
// calculate_profit("42000", "43000", "0.5") -> Ok(500.0)
// calculate_profit("abc", "43000", "0.5") -> Err("...")
```

### Упражнение 2: Парсер стакана ордеров

Напишите функцию для парсинга строки стакана в формате `"price:quantity"`:

```rust
fn parse_order_book_level(level: &str) -> Result<(f64, f64), String> {
    // Ваш код здесь
}

// Использование:
// parse_order_book_level("42000.50:1.5") -> Ok((42000.50, 1.5))
```

### Упражнение 3: Парсер торговой истории

Напишите функцию для парсинга строки сделки в формате `"timestamp,symbol,side,price,quantity"`:

```rust
struct Trade {
    timestamp: u64,
    symbol: String,
    side: String,
    price: f64,
    quantity: f64,
}

fn parse_trade(line: &str) -> Result<Trade, String> {
    // Ваш код здесь
}
```

### Упражнение 4: Валидатор лимитов

Напишите функцию валидации торговых лимитов:

```rust
fn validate_trade_limits(
    quantity_str: &str,
    price_str: &str,
    min_qty: f64,
    max_qty: f64,
    min_price: f64,
    max_price: f64,
) -> Result<(f64, f64), String> {
    // Ваш код здесь
}
```

## Домашнее задание

1. **Конвертер валют:** Напишите программу, которая парсит строку вида `"100 USD to EUR"` и выполняет конвертацию (используйте фиксированные курсы).

2. **Парсер портфеля:** Создайте функцию для парсинга строки портфеля `"BTC:0.5,ETH:10,DOGE:50000"` в вектор кортежей (symbol, quantity).

3. **Калькулятор позиции:** Напишите функцию, которая парсит строку с несколькими сделками и вычисляет среднюю цену входа:
   ```
   "BUY 0.5 @ 42000, BUY 0.3 @ 41000, BUY 0.2 @ 43000"
   ```

4. **Генератор отчёта:** Создайте программу, которая парсит CSV данные о сделках и выводит статистику: общий объём, средняя цена, количество сделок покупки/продажи.

## Навигация

[← Предыдущий день](../028-user-input/ru.md) | [Следующий день →](../030-result-type/ru.md)
