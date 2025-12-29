# День 10: Символы и строки — тикеры AAPL, BTC, ETH

## Аналогия из трейдинга

На бирже всё имеет текстовые названия:
- Тикеры: **BTC**, **ETH**, **AAPL**
- Торговые пары: **BTC/USDT**, **ETH/BTC**
- Названия бирж: **Binance**, **Coinbase**
- Сообщения: **"Order filled"**, **"Insufficient balance"**

В Rust для работы с текстом есть два основных типа: `char` и строки.

## Символы (char)

`char` — это один символ в одинарных кавычках:

```rust
fn main() {
    let currency_sign: char = '$';
    let btc_symbol: char = '₿';
    let up_arrow: char = '↑';
    let down_arrow: char = '↓';
    let check_mark: char = '✓';

    println!("Bitcoin {} {}", btc_symbol, up_arrow);
    println!("Trade executed {}", check_mark);
}
```

`char` в Rust — это Unicode символ (4 байта), поэтому поддерживает эмодзи и любые символы:

```rust
fn main() {
    let rocket: char = '🚀';
    let money: char = '💰';
    let chart: char = '📈';

    println!("To the moon! {} {} {}", rocket, money, chart);
}
```

## Два типа строк

### `&str` — строковый срез (заимствование)

Фиксированный текст, который нельзя изменить:

```rust
fn main() {
    let ticker: &str = "BTC/USDT";
    let exchange: &str = "Binance";
    let status: &str = "Order filled";

    println!("Trading {} on {}", ticker, exchange);
    println!("Status: {}", status);
}
```

**Аналогия:** Это как надпись на табличке — ты можешь читать, но не можешь изменить.

### `String` — владеющая строка

Строка, которую можно изменять:

```rust
fn main() {
    // Создание String
    let mut message = String::from("Price: ");

    // Добавление текста
    message.push_str("42000");
    message.push_str(" USDT");

    println!("{}", message);  // Price: 42000 USDT

    // Добавление символа
    message.push('!');
    println!("{}", message);  // Price: 42000 USDT!
}
```

**Аналогия:** Это как блокнот — ты можешь писать, стирать и добавлять текст.

## Создание String

```rust
fn main() {
    // Разные способы создания
    let s1 = String::from("BTC");
    let s2 = "ETH".to_string();
    let s3 = String::new();  // Пустая строка
    let s4: String = format!("{}/{}", "BTC", "USDT");

    println!("s1: {}", s1);
    println!("s2: {}", s2);
    println!("s3: '{}'", s3);
    println!("s4: {}", s4);
}
```

## Конкатенация строк

```rust
fn main() {
    // Способ 1: format!
    let base = "BTC";
    let quote = "USDT";
    let pair = format!("{}/{}", base, quote);
    println!("Pair: {}", pair);

    // Способ 2: + оператор (забирает владение первой строкой)
    let mut s = String::from("Order #");
    s = s + "12345";
    println!("{}", s);

    // Способ 3: push_str
    let mut log = String::from("[INFO] ");
    log.push_str("Trade executed at ");
    log.push_str("42000 USDT");
    println!("{}", log);
}
```

## Полезные методы String

```rust
fn main() {
    let ticker = String::from("  BTC/USDT  ");

    // Убрать пробелы
    println!("Trimmed: '{}'", ticker.trim());  // 'BTC/USDT'

    // Длина
    println!("Length: {}", ticker.len());  // 12

    // Проверка содержимого
    println!("Contains BTC: {}", ticker.contains("BTC"));  // true
    println!("Starts with BTC: {}", ticker.trim().starts_with("BTC"));  // true
    println!("Ends with USDT: {}", ticker.trim().ends_with("USDT"));  // true

    // Замена
    let new_pair = ticker.trim().replace("USDT", "EUR");
    println!("New pair: {}", new_pair);  // BTC/EUR

    // Верхний/нижний регистр
    println!("Upper: {}", ticker.to_uppercase());
    println!("Lower: {}", ticker.to_lowercase());

    // Пустая ли строка
    let empty = String::new();
    println!("Is empty: {}", empty.is_empty());  // true
}
```

## Разбиение строк

```rust
fn main() {
    let pair = "BTC/USDT";

    // Split по разделителю
    let parts: Vec<&str> = pair.split('/').collect();
    println!("Base: {}", parts[0]);   // BTC
    println!("Quote: {}", parts[1]);  // USDT

    // Split на итератор
    for part in pair.split('/') {
        println!("Part: {}", part);
    }

    // Split на строки
    let log = "Line 1\nLine 2\nLine 3";
    for line in log.lines() {
        println!("Line: {}", line);
    }
}
```

## Доступ к символам

```rust
fn main() {
    let ticker = "BTC";

    // Получить символы (итератор)
    for c in ticker.chars() {
        println!("Char: {}", c);
    }

    // Первый символ
    let first = ticker.chars().next();
    println!("First: {:?}", first);  // Some('B')

    // N-й символ
    let second = ticker.chars().nth(1);
    println!("Second: {:?}", second);  // Some('T')

    // НЕЛЬЗЯ обращаться по индексу напрямую!
    // let c = ticker[0];  // ОШИБКА!
}
```

**Важно:** В Rust строки — это UTF-8, и один "символ" может занимать несколько байт. Поэтому индексация запрещена.

## Практический пример: парсинг торговой пары

```rust
fn main() {
    let pair = "ETH/USDT";

    // Разбиваем пару
    let parts: Vec<&str> = pair.split('/').collect();

    if parts.len() == 2 {
        let base = parts[0];
        let quote = parts[1];

        println!("Trading pair: {}", pair);
        println!("Base currency: {}", base);
        println!("Quote currency: {}", quote);

        // Проверяем, это стейблкоин?
        let stablecoins = ["USDT", "USDC", "BUSD", "DAI"];
        let is_stable_pair = stablecoins.contains(&quote);

        println!("Stable pair: {}", is_stable_pair);
    } else {
        println!("Invalid pair format!");
    }
}
```

## Практический пример: форматирование сообщений

```rust
fn main() {
    // Данные сделки
    let symbol = "BTC/USDT";
    let side = "BUY";
    let price = 42000.50;
    let quantity = 0.5;
    let order_id = 123456789;

    // Форматируем сообщение для лога
    let log_message = format!(
        "[TRADE] {} {} {:.8} @ {:.2} | Order #{}",
        side, symbol, quantity, price, order_id
    );
    println!("{}", log_message);

    // Форматируем для пользователя
    let user_message = format!(
        "Bought {} {} at ${:.2} each. Total: ${:.2}",
        quantity, symbol.split('/').next().unwrap_or(""),
        price, price * quantity
    );
    println!("{}", user_message);

    // Форматируем для API
    let api_payload = format!(
        r#"{{"symbol":"{}","side":"{}","price":{},"quantity":{}}}"#,
        symbol, side, price, quantity
    );
    println!("API: {}", api_payload);
}
```

## Практический пример: валидация тикера

```rust
fn main() {
    let tickers = ["BTC", "eth", "XRP123", "DOGE", ""];

    for ticker in tickers {
        let is_valid = validate_ticker(ticker);
        println!("{:10} -> valid: {}", format!("'{}'", ticker), is_valid);
    }
}

fn validate_ticker(ticker: &str) -> bool {
    // Правила:
    // 1. Не пустой
    // 2. Только буквы
    // 3. От 2 до 10 символов
    // 4. Верхний регистр

    if ticker.is_empty() {
        return false;
    }

    if ticker.len() < 2 || ticker.len() > 10 {
        return false;
    }

    // Проверяем, что все символы — заглавные буквы
    ticker.chars().all(|c| c.is_ascii_uppercase())
}
```

## Преобразование &str и String

```rust
fn main() {
    // &str -> String
    let s: &str = "BTC";
    let string1: String = s.to_string();
    let string2: String = String::from(s);
    let string3: String = s.to_owned();

    // String -> &str
    let string = String::from("ETH");
    let slice: &str = &string;
    let slice2: &str = string.as_str();

    println!("String: {}", string);
    println!("Slice: {}", slice);
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| `char` | Один Unicode символ |
| `&str` | Неизменяемый строковый срез |
| `String` | Изменяемая строка |
| `format!` | Форматирование строк |
| `split`, `trim` | Обработка строк |

## Домашнее задание

1. Напиши функцию, которая принимает торговую пару (например, "BTC/USDT") и возвращает кортеж (base, quote)

2. Создай функцию валидации тикера с правилами:
   - Длина 2-6 символов
   - Только буквы
   - Верхний регистр

3. Напиши форматтер для отображения сделки:
   ```
   ═══════════════════════
   TRADE EXECUTED
   Symbol: BTC/USDT
   Side: BUY
   Price: $42,000.50
   Quantity: 0.50000000
   Total: $21,000.25
   ═══════════════════════
   ```

4. Реализуй парсер простой команды: "buy BTC 0.5" -> (action, symbol, amount)

## Навигация

[← Предыдущий день](../009-booleans-market-status/ru.md) | [Следующий день →](../011-tuples-bid-ask/ru.md)
