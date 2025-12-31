# День 103: thiserror — создаём красивые ошибки

## Аналогия из трейдинга

Представь, что ты работаешь в крупной брокерской компании. Когда что-то идёт не так, важно не просто сказать "Ошибка!", а дать чёткое, понятное сообщение:

❌ **Плохо:** "Error 500"
✅ **Хорошо:** "Недостаточно средств для покупки 100 акций AAPL по цене $150.00. Доступно: $10,000. Требуется: $15,000"

В трейдинге чёткие сообщения об ошибках могут спасти миллионы. Библиотека `thiserror` помогает создавать именно такие — информативные и структурированные ошибки.

## Что такое thiserror?

`thiserror` — это библиотека для удобного создания кастомных типов ошибок с помощью derive-макросов. Она автоматически реализует трейт `std::error::Error` и форматирование сообщений.

### Преимущества thiserror

| Без thiserror | С thiserror |
|---------------|-------------|
| Много boilerplate кода | Минимум кода |
| Ручная реализация Display | Автоматический Display |
| Сложная работа с source | Простой атрибут #[source] |
| Трудно поддерживать | Легко читать и изменять |

## Подключение библиотеки

Добавь в `Cargo.toml`:

```toml
[dependencies]
thiserror = "1.0"
```

## Базовый синтаксис

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Недостаточно средств: требуется {required}, доступно {available}")]
    InsufficientFunds {
        required: f64,
        available: f64,
    },

    #[error("Неверный тикер: {0}")]
    InvalidTicker(String),

    #[error("Рынок закрыт")]
    MarketClosed,
}
```

## Форматирование сообщений

`thiserror` поддерживает несколько способов форматирования:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderError {
    // Позиционные аргументы
    #[error("Ордер {0} не найден")]
    NotFound(u64),

    // Именованные поля
    #[error("Превышен лимит: {current}/{max} ордеров")]
    LimitExceeded { current: usize, max: usize },

    // Вызов методов
    #[error("Неверная цена: {price:.2} (мин: {min:.2})")]
    InvalidPrice { price: f64, min: f64 },

    // Использование Display внутреннего типа
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
```

## Практический пример: торговая система

```rust
use thiserror::Error;
use std::collections::HashMap;

// Определяем все возможные ошибки торговой системы
#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Недостаточно средств для покупки {quantity} {ticker}: требуется ${required:.2}, доступно ${available:.2}")]
    InsufficientFunds {
        ticker: String,
        quantity: u32,
        required: f64,
        available: f64,
    },

    #[error("Актив {0} не найден на бирже")]
    AssetNotFound(String),

    #[error("Невозможно продать {quantity} {ticker}: в портфеле только {available}")]
    InsufficientShares {
        ticker: String,
        quantity: u32,
        available: u32,
    },

    #[error("Рынок {0} сейчас закрыт")]
    MarketClosed(String),

    #[error("Ордер отклонён: {reason}")]
    OrderRejected { reason: String },

    #[error("Превышен дневной лимит торговли: ${current:.2} / ${limit:.2}")]
    DailyLimitExceeded { current: f64, limit: f64 },
}

// Структура портфеля
struct Portfolio {
    balance: f64,
    holdings: HashMap<String, u32>,
    daily_traded: f64,
    daily_limit: f64,
}

impl Portfolio {
    fn new(balance: f64, daily_limit: f64) -> Self {
        Portfolio {
            balance,
            holdings: HashMap::new(),
            daily_traded: 0.0,
            daily_limit,
        }
    }

    fn buy(&mut self, ticker: &str, quantity: u32, price: f64) -> Result<(), TradingError> {
        let total_cost = price * quantity as f64;

        // Проверяем дневной лимит
        if self.daily_traded + total_cost > self.daily_limit {
            return Err(TradingError::DailyLimitExceeded {
                current: self.daily_traded + total_cost,
                limit: self.daily_limit,
            });
        }

        // Проверяем баланс
        if total_cost > self.balance {
            return Err(TradingError::InsufficientFunds {
                ticker: ticker.to_string(),
                quantity,
                required: total_cost,
                available: self.balance,
            });
        }

        // Выполняем покупку
        self.balance -= total_cost;
        self.daily_traded += total_cost;
        *self.holdings.entry(ticker.to_string()).or_insert(0) += quantity;

        println!("✅ Куплено {} {} по ${:.2}", quantity, ticker, price);
        Ok(())
    }

    fn sell(&mut self, ticker: &str, quantity: u32, price: f64) -> Result<(), TradingError> {
        let available = *self.holdings.get(ticker).unwrap_or(&0);

        if available < quantity {
            return Err(TradingError::InsufficientShares {
                ticker: ticker.to_string(),
                quantity,
                available,
            });
        }

        // Выполняем продажу
        let total = price * quantity as f64;
        self.balance += total;
        self.daily_traded += total;
        *self.holdings.get_mut(ticker).unwrap() -= quantity;

        println!("✅ Продано {} {} по ${:.2}", quantity, ticker, price);
        Ok(())
    }
}

fn main() {
    let mut portfolio = Portfolio::new(10_000.0, 50_000.0);

    // Успешная покупка
    match portfolio.buy("AAPL", 10, 150.0) {
        Ok(()) => println!("Баланс: ${:.2}", portfolio.balance),
        Err(e) => println!("❌ Ошибка: {}", e),
    }

    // Попытка купить слишком много
    match portfolio.buy("TSLA", 100, 200.0) {
        Ok(()) => println!("Баланс: ${:.2}", portfolio.balance),
        Err(e) => println!("❌ Ошибка: {}", e),
    }

    // Попытка продать то, чего нет
    match portfolio.sell("GOOGL", 5, 140.0) {
        Ok(()) => println!("Баланс: ${:.2}", portfolio.balance),
        Err(e) => println!("❌ Ошибка: {}", e),
    }
}
```

Вывод программы:
```
✅ Куплено 10 AAPL по $150.00
Баланс: $8500.00
❌ Ошибка: Недостаточно средств для покупки 100 TSLA: требуется $20000.00, доступно $8500.00
❌ Ошибка: Невозможно продать 5 GOOGL: в портфеле только 0
```

## Работа с вложенными ошибками

Атрибут `#[source]` позволяет сохранять цепочку ошибок:

```rust
use thiserror::Error;
use std::num::ParseFloatError;

#[derive(Error, Debug)]
pub enum PriceParseError {
    #[error("Неверный формат цены '{input}'")]
    InvalidFormat {
        input: String,
        #[source]
        source: ParseFloatError,
    },

    #[error("Цена не может быть отрицательной: {0}")]
    NegativePrice(f64),

    #[error("Цена слишком высокая: {price} > {max}")]
    PriceTooHigh { price: f64, max: f64 },
}

fn parse_price(input: &str, max_price: f64) -> Result<f64, PriceParseError> {
    let price: f64 = input.trim().parse().map_err(|e| {
        PriceParseError::InvalidFormat {
            input: input.to_string(),
            source: e,
        }
    })?;

    if price < 0.0 {
        return Err(PriceParseError::NegativePrice(price));
    }

    if price > max_price {
        return Err(PriceParseError::PriceTooHigh {
            price,
            max: max_price,
        });
    }

    Ok(price)
}

fn main() {
    // Валидная цена
    println!("Цена: {:?}", parse_price("150.50", 1000.0));

    // Неверный формат
    println!("Ошибка: {:?}", parse_price("abc", 1000.0));

    // Отрицательная цена
    println!("Ошибка: {:?}", parse_price("-50.0", 1000.0));

    // Слишком высокая цена
    println!("Ошибка: {:?}", parse_price("5000.0", 1000.0));
}
```

## Автоматическое преобразование с #[from]

```rust
use thiserror::Error;
use std::io;
use std::num::ParseIntError;

#[derive(Error, Debug)]
pub enum DataLoadError {
    #[error("Ошибка чтения файла")]
    Io(#[from] io::Error),

    #[error("Ошибка парсинга числа")]
    Parse(#[from] ParseIntError),

    #[error("Файл пуст")]
    EmptyFile,
}

fn load_prices(filename: &str) -> Result<Vec<i32>, DataLoadError> {
    let content = std::fs::read_to_string(filename)?; // Автоматически io::Error -> DataLoadError

    if content.is_empty() {
        return Err(DataLoadError::EmptyFile);
    }

    let prices: Result<Vec<i32>, _> = content
        .lines()
        .map(|line| line.parse())
        .collect();

    Ok(prices?) // Автоматически ParseIntError -> DataLoadError
}
```

## Пример: система риск-менеджмента

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RiskError {
    #[error("Позиция {ticker} превышает лимит: {current:.1}% > {max:.1}%")]
    PositionLimitExceeded {
        ticker: String,
        current: f64,
        max: f64,
    },

    #[error("Общий риск портфеля слишком высок: VaR = ${var:.2} (лимит: ${limit:.2})")]
    VaRExceeded { var: f64, limit: f64 },

    #[error("Слишком высокая корреляция между {asset1} и {asset2}: {correlation:.2}")]
    HighCorrelation {
        asset1: String,
        asset2: String,
        correlation: f64,
    },

    #[error("Недопустимый уровень плеча: {leverage}x (макс: {max_leverage}x)")]
    ExcessiveLeverage { leverage: f64, max_leverage: f64 },
}

struct RiskManager {
    max_position_pct: f64,
    max_var: f64,
    max_leverage: f64,
}

impl RiskManager {
    fn check_position(&self, ticker: &str, position_pct: f64) -> Result<(), RiskError> {
        if position_pct > self.max_position_pct {
            return Err(RiskError::PositionLimitExceeded {
                ticker: ticker.to_string(),
                current: position_pct,
                max: self.max_position_pct,
            });
        }
        Ok(())
    }

    fn check_var(&self, var: f64) -> Result<(), RiskError> {
        if var > self.max_var {
            return Err(RiskError::VaRExceeded {
                var,
                limit: self.max_var,
            });
        }
        Ok(())
    }

    fn check_leverage(&self, leverage: f64) -> Result<(), RiskError> {
        if leverage > self.max_leverage {
            return Err(RiskError::ExcessiveLeverage {
                leverage,
                max_leverage: self.max_leverage,
            });
        }
        Ok(())
    }
}

fn main() {
    let risk_manager = RiskManager {
        max_position_pct: 10.0,
        max_var: 50_000.0,
        max_leverage: 3.0,
    };

    // Проверка позиции
    if let Err(e) = risk_manager.check_position("BTC", 25.0) {
        println!("⚠️ Риск: {}", e);
    }

    // Проверка VaR
    if let Err(e) = risk_manager.check_var(75_000.0) {
        println!("⚠️ Риск: {}", e);
    }

    // Проверка плеча
    if let Err(e) = risk_manager.check_leverage(5.0) {
        println!("⚠️ Риск: {}", e);
    }
}
```

## Упражнения

### Упражнение 1: Ошибки валидации ордера
Создай enum `OrderValidationError` с вариантами:
- `InvalidQuantity` — количество должно быть > 0
- `InvalidPrice` — цена должна быть > 0
- `InvalidSide` — сторона должна быть "buy" или "sell"

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderValidationError {
    // Твой код здесь
}

fn validate_order(quantity: i32, price: f64, side: &str) -> Result<(), OrderValidationError> {
    // Твой код здесь
    Ok(())
}
```

### Упражнение 2: Ошибки API биржи
Создай enum для ошибок при работе с API биржи:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExchangeApiError {
    // Варианты:
    // - RateLimited { retry_after: u64 } — превышен лимит запросов
    // - AuthenticationFailed — неверный API ключ
    // - NetworkError с вложенной std::io::Error
    // - InvalidResponse { status: u16, body: String }
}
```

### Упражнение 3: Комбинирование ошибок
Создай функцию, которая читает цены из файла и валидирует их:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceDataError {
    // Объедини ошибки чтения файла и парсинга
}

fn load_and_validate_prices(filename: &str) -> Result<Vec<f64>, PriceDataError> {
    // Читай файл, парсь строки в f64, проверяй что все цены > 0
    todo!()
}
```

### Упражнение 4: Иерархия ошибок
Создай иерархию ошибок для торгового бота:

```rust
#[derive(Error, Debug)]
pub enum StrategyError { /* ... */ }

#[derive(Error, Debug)]
pub enum ExecutionError { /* ... */ }

#[derive(Error, Debug)]
pub enum TradingBotError {
    #[error("Ошибка стратегии")]
    Strategy(#[from] StrategyError),

    #[error("Ошибка исполнения")]
    Execution(#[from] ExecutionError),

    // Добавь ещё варианты
}
```

## Домашнее задание

1. **Создай полную систему ошибок** для торгового приложения, включающую:
   - Ошибки подключения к бирже
   - Ошибки валидации ордеров
   - Ошибки риск-менеджмента
   - Ошибки работы с портфелем

2. **Реализуй трейт From** для преобразования между уровнями ошибок

3. **Добавь контекст** к ошибкам с помощью методов `.context()` (потребуется `anyhow` из следующей главы)

4. **Напиши тесты**, проверяющие сообщения об ошибках:
   ```rust
   #[test]
   fn test_error_messages() {
       let err = TradingError::InsufficientFunds {
           ticker: "AAPL".to_string(),
           quantity: 10,
           required: 1500.0,
           available: 1000.0,
       };
       assert!(err.to_string().contains("AAPL"));
       assert!(err.to_string().contains("1500"));
   }
   ```

## Ключевые выводы

| Концепция | Описание |
|-----------|----------|
| `#[derive(Error)]` | Автоматически реализует трейт Error |
| `#[error("...")]` | Задаёт сообщение для Display |
| `#[from]` | Автоматическое преобразование из другой ошибки |
| `#[source]` | Указывает на причину ошибки |
| `#[error(transparent)]` | Пробрасывает Display вложенной ошибки |

## Навигация

[← День 102: Box<dyn Error>](../102-box-dyn-error/ru.md) | [День 104: anyhow →](../104-anyhow-simple-errors/ru.md)
