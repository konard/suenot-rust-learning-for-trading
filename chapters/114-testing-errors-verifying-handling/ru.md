# День 114: Тестирование ошибок — проверяем обработку

## Аналогия из трейдинга

Представь, что ты тестируешь торгового бота перед запуском на реальном рынке. Ты **намеренно** создаёшь проблемные ситуации: отключаешь интернет, посылаешь невалидные ордера, симулируешь падение биржи. Если бот правильно обрабатывает эти ситуации, ты можешь доверять ему свои деньги.

Тестирование ошибок в коде работает так же — мы специально вызываем ошибочные сценарии и проверяем, что код реагирует правильно.

## Базовое тестирование Result

```rust
fn validate_order_price(price: f64) -> Result<f64, String> {
    if price <= 0.0 {
        return Err(String::from("Цена должна быть положительной"));
    }
    if price > 1_000_000.0 {
        return Err(String::from("Цена превышает лимит"));
    }
    Ok(price)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_price() {
        let result = validate_order_price(42000.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42000.0);
    }

    #[test]
    fn test_negative_price() {
        let result = validate_order_price(-100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_price() {
        let result = validate_order_price(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_price_exceeds_limit() {
        let result = validate_order_price(2_000_000.0);
        assert!(result.is_err());
    }
}
```

## Проверка конкретного сообщения об ошибке

```rust
fn calculate_position_size(
    balance: f64,
    risk_percent: f64,
    stop_loss_distance: f64,
) -> Result<f64, String> {
    if balance <= 0.0 {
        return Err(String::from("Баланс должен быть положительным"));
    }
    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return Err(String::from("Риск должен быть от 0 до 100%"));
    }
    if stop_loss_distance <= 0.0 {
        return Err(String::from("Расстояние до стоп-лосса должно быть положительным"));
    }

    let risk_amount = balance * (risk_percent / 100.0);
    Ok(risk_amount / stop_loss_distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message_for_zero_balance() {
        let result = calculate_position_size(0.0, 2.0, 100.0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Баланс должен быть положительным"
        );
    }

    #[test]
    fn test_error_message_for_invalid_risk() {
        let result = calculate_position_size(10000.0, 150.0, 100.0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Риск должен быть от 0 до 100%"
        );
    }

    #[test]
    fn test_contains_error_text() {
        let result = calculate_position_size(10000.0, 2.0, 0.0);
        let error = result.unwrap_err();
        assert!(error.contains("стоп-лосс"));
    }
}
```

## Использование matches! для проверки вариантов

```rust
#[derive(Debug, PartialEq)]
enum OrderError {
    InsufficientBalance { required: f64, available: f64 },
    InvalidPrice(f64),
    InvalidQuantity(f64),
    MarketClosed,
    ConnectionLost,
}

fn place_order(
    price: f64,
    quantity: f64,
    balance: f64,
    market_open: bool,
) -> Result<String, OrderError> {
    if !market_open {
        return Err(OrderError::MarketClosed);
    }
    if price <= 0.0 {
        return Err(OrderError::InvalidPrice(price));
    }
    if quantity <= 0.0 {
        return Err(OrderError::InvalidQuantity(quantity));
    }

    let required = price * quantity;
    if required > balance {
        return Err(OrderError::InsufficientBalance { required, available: balance });
    }

    Ok(format!("ORDER-{}", (price * quantity) as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_closed_error() {
        let result = place_order(42000.0, 0.5, 100000.0, false);
        assert!(matches!(result, Err(OrderError::MarketClosed)));
    }

    #[test]
    fn test_invalid_price_error() {
        let result = place_order(-100.0, 0.5, 100000.0, true);
        assert!(matches!(result, Err(OrderError::InvalidPrice(p)) if p < 0.0));
    }

    #[test]
    fn test_insufficient_balance_error() {
        let result = place_order(50000.0, 1.0, 10000.0, true);

        match result {
            Err(OrderError::InsufficientBalance { required, available }) => {
                assert_eq!(required, 50000.0);
                assert_eq!(available, 10000.0);
            }
            _ => panic!("Ожидалась ошибка InsufficientBalance"),
        }
    }
}
```

## Тестирование с помощью #[should_panic]

```rust
fn get_price_or_panic(prices: &[f64], index: usize) -> f64 {
    if index >= prices.len() {
        panic!("Индекс {} выходит за пределы массива цен (длина: {})", index, prices.len());
    }
    prices[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_panics_on_invalid_index() {
        let prices = [42000.0, 42100.0, 42050.0];
        get_price_or_panic(&prices, 10);
    }

    #[test]
    #[should_panic(expected = "выходит за пределы")]
    fn test_panic_message_contains_text() {
        let prices = [42000.0];
        get_price_or_panic(&prices, 5);
    }

    #[test]
    fn test_valid_index_does_not_panic() {
        let prices = [42000.0, 42100.0, 42050.0];
        let result = get_price_or_panic(&prices, 1);
        assert_eq!(result, 42100.0);
    }
}
```

## Тестирование Option

```rust
fn find_best_bid(order_book: &[(f64, f64)]) -> Option<(f64, f64)> {
    if order_book.is_empty() {
        return None;
    }

    order_book
        .iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .copied()
}

fn calculate_spread(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> Option<f64> {
    let best_bid = find_best_bid(bids)?;
    let best_ask = asks.iter().min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())?;

    Some(best_ask.0 - best_bid.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_order_book_returns_none() {
        let result = find_best_bid(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_finds_best_bid() {
        let bids = [(41900.0, 1.0), (42000.0, 0.5), (41800.0, 2.0)];
        let result = find_best_bid(&bids);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), (42000.0, 0.5));
    }

    #[test]
    fn test_spread_with_empty_bids() {
        let bids: [(f64, f64); 0] = [];
        let asks = [(42100.0, 1.0)];

        assert!(calculate_spread(&bids, &asks).is_none());
    }

    #[test]
    fn test_spread_calculation() {
        let bids = [(42000.0, 0.5)];
        let asks = [(42050.0, 1.0)];

        let spread = calculate_spread(&bids, &asks);
        assert_eq!(spread, Some(50.0));
    }
}
```

## Тестирование граничных случаев

```rust
fn calculate_sma(prices: &[f64], period: usize) -> Result<f64, String> {
    if prices.is_empty() {
        return Err(String::from("Массив цен пуст"));
    }
    if period == 0 {
        return Err(String::from("Период должен быть больше нуля"));
    }
    if period > prices.len() {
        return Err(format!(
            "Недостаточно данных: нужно {}, есть {}",
            period,
            prices.len()
        ));
    }

    let slice = &prices[prices.len() - period..];
    let sum: f64 = slice.iter().sum();
    Ok(sum / period as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Тесты граничных случаев ошибок
    #[test]
    fn test_empty_prices() {
        assert!(calculate_sma(&[], 3).is_err());
    }

    #[test]
    fn test_zero_period() {
        assert!(calculate_sma(&[42000.0], 0).is_err());
    }

    #[test]
    fn test_period_exceeds_data() {
        let prices = [42000.0, 42100.0];
        let result = calculate_sma(&prices, 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Недостаточно данных"));
    }

    // Тесты граничных случаев успеха
    #[test]
    fn test_period_equals_data_length() {
        let prices = [42000.0, 42100.0, 42200.0];
        let result = calculate_sma(&prices, 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_element() {
        let prices = [42000.0];
        let result = calculate_sma(&prices, 1);
        assert_eq!(result.unwrap(), 42000.0);
    }
}
```

## Практический пример: полный набор тестов для валидации ордера

```rust
#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    side: OrderSide,
    price: f64,
    quantity: f64,
}

#[derive(Debug, Clone, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq)]
enum ValidationError {
    EmptySymbol,
    InvalidPrice { price: f64, reason: &'static str },
    InvalidQuantity { quantity: f64, reason: &'static str },
    OrderTooSmall { value: f64, minimum: f64 },
    OrderTooLarge { value: f64, maximum: f64 },
}

fn validate_order(order: &Order) -> Result<(), ValidationError> {
    // Проверка символа
    if order.symbol.is_empty() {
        return Err(ValidationError::EmptySymbol);
    }

    // Проверка цены
    if order.price <= 0.0 {
        return Err(ValidationError::InvalidPrice {
            price: order.price,
            reason: "Цена должна быть положительной",
        });
    }
    if order.price.is_nan() || order.price.is_infinite() {
        return Err(ValidationError::InvalidPrice {
            price: order.price,
            reason: "Цена должна быть конечным числом",
        });
    }

    // Проверка количества
    if order.quantity <= 0.0 {
        return Err(ValidationError::InvalidQuantity {
            quantity: order.quantity,
            reason: "Количество должно быть положительным",
        });
    }

    // Проверка минимальной стоимости ордера
    let order_value = order.price * order.quantity;
    const MIN_ORDER_VALUE: f64 = 10.0;
    const MAX_ORDER_VALUE: f64 = 1_000_000.0;

    if order_value < MIN_ORDER_VALUE {
        return Err(ValidationError::OrderTooSmall {
            value: order_value,
            minimum: MIN_ORDER_VALUE,
        });
    }

    if order_value > MAX_ORDER_VALUE {
        return Err(ValidationError::OrderTooLarge {
            value: order_value,
            maximum: MAX_ORDER_VALUE,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_order() -> Order {
        Order {
            symbol: String::from("BTC/USDT"),
            side: OrderSide::Buy,
            price: 42000.0,
            quantity: 0.1,
        }
    }

    // Тесты успешной валидации
    #[test]
    fn test_valid_order_passes() {
        let order = valid_order();
        assert!(validate_order(&order).is_ok());
    }

    #[test]
    fn test_minimum_valid_order() {
        let order = Order {
            symbol: String::from("BTC/USDT"),
            side: OrderSide::Buy,
            price: 10.0,
            quantity: 1.0, // value = 10.0, ровно минимум
        };
        assert!(validate_order(&order).is_ok());
    }

    // Тесты ошибок символа
    #[test]
    fn test_empty_symbol() {
        let mut order = valid_order();
        order.symbol = String::new();

        assert_eq!(
            validate_order(&order),
            Err(ValidationError::EmptySymbol)
        );
    }

    // Тесты ошибок цены
    #[test]
    fn test_zero_price() {
        let mut order = valid_order();
        order.price = 0.0;

        assert!(matches!(
            validate_order(&order),
            Err(ValidationError::InvalidPrice { price: 0.0, .. })
        ));
    }

    #[test]
    fn test_negative_price() {
        let mut order = valid_order();
        order.price = -100.0;

        assert!(matches!(
            validate_order(&order),
            Err(ValidationError::InvalidPrice { .. })
        ));
    }

    #[test]
    fn test_nan_price() {
        let mut order = valid_order();
        order.price = f64::NAN;

        let result = validate_order(&order);
        assert!(matches!(result, Err(ValidationError::InvalidPrice { .. })));
    }

    #[test]
    fn test_infinite_price() {
        let mut order = valid_order();
        order.price = f64::INFINITY;

        assert!(matches!(
            validate_order(&order),
            Err(ValidationError::InvalidPrice { .. })
        ));
    }

    // Тесты ошибок количества
    #[test]
    fn test_zero_quantity() {
        let mut order = valid_order();
        order.quantity = 0.0;

        assert!(matches!(
            validate_order(&order),
            Err(ValidationError::InvalidQuantity { .. })
        ));
    }

    #[test]
    fn test_negative_quantity() {
        let mut order = valid_order();
        order.quantity = -1.0;

        assert!(matches!(
            validate_order(&order),
            Err(ValidationError::InvalidQuantity { .. })
        ));
    }

    // Тесты ограничений по размеру ордера
    #[test]
    fn test_order_too_small() {
        let order = Order {
            symbol: String::from("BTC/USDT"),
            side: OrderSide::Buy,
            price: 1.0,
            quantity: 1.0, // value = 1.0 < 10.0
        };

        match validate_order(&order) {
            Err(ValidationError::OrderTooSmall { value, minimum }) => {
                assert_eq!(value, 1.0);
                assert_eq!(minimum, 10.0);
            }
            other => panic!("Ожидалась ошибка OrderTooSmall, получено: {:?}", other),
        }
    }

    #[test]
    fn test_order_too_large() {
        let order = Order {
            symbol: String::from("BTC/USDT"),
            side: OrderSide::Buy,
            price: 50000.0,
            quantity: 100.0, // value = 5_000_000 > 1_000_000
        };

        match validate_order(&order) {
            Err(ValidationError::OrderTooLarge { value, maximum }) => {
                assert_eq!(value, 5_000_000.0);
                assert_eq!(maximum, 1_000_000.0);
            }
            other => panic!("Ожидалась ошибка OrderTooLarge, получено: {:?}", other),
        }
    }
}
```

## Паттерн: хелпер-функции для тестов

```rust
#[derive(Debug, PartialEq)]
enum TradeError {
    InsufficientFunds,
    InvalidAmount,
    MarketClosed,
}

fn execute_trade(balance: f64, amount: f64, market_open: bool) -> Result<f64, TradeError> {
    if !market_open {
        return Err(TradeError::MarketClosed);
    }
    if amount <= 0.0 {
        return Err(TradeError::InvalidAmount);
    }
    if amount > balance {
        return Err(TradeError::InsufficientFunds);
    }
    Ok(balance - amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Хелпер для проверки конкретной ошибки
    fn assert_trade_error(
        balance: f64,
        amount: f64,
        market_open: bool,
        expected: TradeError,
    ) {
        let result = execute_trade(balance, amount, market_open);
        assert_eq!(result, Err(expected));
    }

    #[test]
    fn test_all_error_cases() {
        assert_trade_error(1000.0, 100.0, false, TradeError::MarketClosed);
        assert_trade_error(1000.0, -50.0, true, TradeError::InvalidAmount);
        assert_trade_error(1000.0, 2000.0, true, TradeError::InsufficientFunds);
    }

    // Хелпер для проверки успеха
    fn assert_trade_success(balance: f64, amount: f64, expected_balance: f64) {
        let result = execute_trade(balance, amount, true);
        assert_eq!(result, Ok(expected_balance));
    }

    #[test]
    fn test_success_cases() {
        assert_trade_success(1000.0, 100.0, 900.0);
        assert_trade_success(1000.0, 1000.0, 0.0);
        assert_trade_success(1000.0, 0.01, 999.99);
    }
}
```

## Что мы узнали

| Метод | Когда использовать |
|-------|-------------------|
| `is_ok()` / `is_err()` | Проверить только факт успеха/ошибки |
| `unwrap_err()` | Получить значение ошибки |
| `matches!` | Проверить вариант enum |
| `#[should_panic]` | Тестировать панику |
| `is_some()` / `is_none()` | Тестировать Option |
| Хелпер-функции | Уменьшить дублирование в тестах |

## Домашнее задание

1. Напиши функцию `parse_trade_signal(s: &str) -> Result<(String, OrderSide, f64), ParseError>` и покрой её тестами для всех возможных ошибок парсинга.

2. Создай валидатор портфеля `validate_portfolio(positions: &[Position]) -> Result<(), Vec<PositionError>>`, который собирает все ошибки, и напиши тесты для случаев с одной и несколькими ошибками.

3. Реализуй функцию расчёта риска `calculate_var(returns: &[f64], confidence: f64) -> Result<f64, VaRError>` с тестами граничных случаев.

4. Напиши тесты для функции, которая может паниковать, используя `#[should_panic(expected = "...")]` с разными сообщениями.

## Навигация

[← Предыдущий день](../113-builder-pattern-complex-structs/ru.md) | [Следующий день →](../115-mocking-errors-tests/ru.md)
