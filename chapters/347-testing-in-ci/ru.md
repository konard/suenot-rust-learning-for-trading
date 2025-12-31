# День 347: Тестирование в CI

## Аналогия из трейдинга

Представь, что ты управляешь торговой фирмой с несколькими трейдерами. Прежде чем новая торговая стратегия попадёт в продакшен, она проходит через **контроль качества**:

1. **Ручная проверка** — опытный трейдер смотрит код стратегии
2. **Бэктест** — стратегия тестируется на исторических данных
3. **Проверка рисков** — риск-менеджер убеждается, что стратегия не нарушает лимиты
4. **Paper trading** — стратегия торгует виртуальными деньгами

CI (Continuous Integration) — это **автоматический контроль качества кода**. Каждый раз, когда ты пушишь изменения, автоматически запускаются проверки: компиляция, тесты, линтеры. Это как иметь армию роботов-контролёров, которые проверяют каждую сделку перед исполнением.

| Этап трейдинга | Этап CI |
|----------------|---------|
| Проверка кода стратегии | `cargo check` — компиляция |
| Бэктест на исторических данных | `cargo test` — запуск тестов |
| Проверка риск-лимитов | `cargo clippy` — линтер |
| Проверка документации | `cargo doc` — генерация документации |
| Аудит зависимостей | `cargo audit` — проверка безопасности |

## Базовая конфигурация тестов в GitHub Actions

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: Run tests
        run: cargo test --all-features --verbose
```

## Структура тестов для торговой системы

```rust
// src/lib.rs
pub mod order;
pub mod risk;
pub mod strategy;

// src/order.rs
#[derive(Debug, Clone, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub status: OrderStatus,
}

impl Order {
    pub fn new(symbol: &str, side: OrderSide, price: f64, quantity: f64) -> Result<Self, OrderError> {
        if price <= 0.0 {
            return Err(OrderError::InvalidPrice(price));
        }
        if quantity <= 0.0 {
            return Err(OrderError::InvalidQuantity(quantity));
        }
        if symbol.is_empty() {
            return Err(OrderError::EmptySymbol);
        }

        Ok(Order {
            id: format!("ORD-{}", chrono::Utc::now().timestamp_millis()),
            symbol: symbol.to_string(),
            side,
            price,
            quantity,
            status: OrderStatus::Pending,
        })
    }

    pub fn value(&self) -> f64 {
        self.price * self.quantity
    }

    pub fn fill(&mut self) {
        self.status = OrderStatus::Filled;
    }

    pub fn cancel(&mut self) {
        if self.status == OrderStatus::Pending {
            self.status = OrderStatus::Cancelled;
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum OrderError {
    InvalidPrice(f64),
    InvalidQuantity(f64),
    EmptySymbol,
    InsufficientBalance { required: f64, available: f64 },
}

impl std::fmt::Display for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderError::InvalidPrice(p) => write!(f, "Некорректная цена: {}", p),
            OrderError::InvalidQuantity(q) => write!(f, "Некорректное количество: {}", q),
            OrderError::EmptySymbol => write!(f, "Пустой символ"),
            OrderError::InsufficientBalance { required, available } => {
                write!(f, "Недостаточно средств: нужно {}, доступно {}", required, available)
            }
        }
    }
}

impl std::error::Error for OrderError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_order() {
        let order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 0.5);
        assert!(order.is_ok());

        let order = order.unwrap();
        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.price, 42000.0);
        assert_eq!(order.quantity, 0.5);
        assert_eq!(order.status, OrderStatus::Pending);
    }

    #[test]
    fn test_order_value() {
        let order = Order::new("ETHUSDT", OrderSide::Buy, 2500.0, 2.0).unwrap();
        assert_eq!(order.value(), 5000.0);
    }

    #[test]
    fn test_invalid_price() {
        let result = Order::new("BTCUSDT", OrderSide::Buy, -100.0, 1.0);
        assert!(matches!(result, Err(OrderError::InvalidPrice(_))));
    }

    #[test]
    fn test_zero_price() {
        let result = Order::new("BTCUSDT", OrderSide::Buy, 0.0, 1.0);
        assert!(matches!(result, Err(OrderError::InvalidPrice(0.0))));
    }

    #[test]
    fn test_invalid_quantity() {
        let result = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, -0.5);
        assert!(matches!(result, Err(OrderError::InvalidQuantity(_))));
    }

    #[test]
    fn test_empty_symbol() {
        let result = Order::new("", OrderSide::Sell, 42000.0, 1.0);
        assert!(matches!(result, Err(OrderError::EmptySymbol)));
    }

    #[test]
    fn test_order_fill() {
        let mut order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 0.5).unwrap();
        assert_eq!(order.status, OrderStatus::Pending);

        order.fill();
        assert_eq!(order.status, OrderStatus::Filled);
    }

    #[test]
    fn test_order_cancel() {
        let mut order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 0.5).unwrap();
        order.cancel();
        assert_eq!(order.status, OrderStatus::Cancelled);
    }

    #[test]
    fn test_cannot_cancel_filled_order() {
        let mut order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 0.5).unwrap();
        order.fill();
        order.cancel();
        assert_eq!(order.status, OrderStatus::Filled); // статус не изменился
    }
}
```

## Риск-менеджмент: тестирование валидации

```rust
// src/risk.rs
use crate::order::{Order, OrderError, OrderSide};

pub struct RiskManager {
    max_position_size: f64,
    max_order_value: f64,
    daily_loss_limit: f64,
    current_daily_loss: f64,
}

impl RiskManager {
    pub fn new(max_position_size: f64, max_order_value: f64, daily_loss_limit: f64) -> Self {
        RiskManager {
            max_position_size,
            max_order_value,
            daily_loss_limit,
            current_daily_loss: 0.0,
        }
    }

    pub fn validate_order(&self, order: &Order, current_position: f64) -> Result<(), RiskError> {
        // Проверка стоимости ордера
        let order_value = order.value();
        if order_value > self.max_order_value {
            return Err(RiskError::OrderValueExceeded {
                value: order_value,
                limit: self.max_order_value,
            });
        }

        // Проверка размера позиции
        let new_position = match order.side {
            OrderSide::Buy => current_position + order.quantity,
            OrderSide::Sell => current_position - order.quantity,
        };

        if new_position.abs() > self.max_position_size {
            return Err(RiskError::PositionSizeExceeded {
                size: new_position.abs(),
                limit: self.max_position_size,
            });
        }

        // Проверка дневного лимита убытков
        if self.current_daily_loss >= self.daily_loss_limit {
            return Err(RiskError::DailyLossLimitReached {
                loss: self.current_daily_loss,
                limit: self.daily_loss_limit,
            });
        }

        Ok(())
    }

    pub fn record_loss(&mut self, loss: f64) {
        if loss > 0.0 {
            self.current_daily_loss += loss;
        }
    }

    pub fn reset_daily_loss(&mut self) {
        self.current_daily_loss = 0.0;
    }
}

#[derive(Debug, PartialEq)]
pub enum RiskError {
    OrderValueExceeded { value: f64, limit: f64 },
    PositionSizeExceeded { size: f64, limit: f64 },
    DailyLossLimitReached { loss: f64, limit: f64 },
}

impl std::fmt::Display for RiskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskError::OrderValueExceeded { value, limit } => {
                write!(f, "Стоимость ордера {} превышает лимит {}", value, limit)
            }
            RiskError::PositionSizeExceeded { size, limit } => {
                write!(f, "Размер позиции {} превышает лимит {}", size, limit)
            }
            RiskError::DailyLossLimitReached { loss, limit } => {
                write!(f, "Дневной лимит убытков исчерпан: {} из {}", loss, limit)
            }
        }
    }
}

impl std::error::Error for RiskError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_risk_manager() -> RiskManager {
        RiskManager::new(
            10.0,      // макс. позиция: 10 BTC
            100000.0,  // макс. стоимость ордера: $100k
            5000.0,    // дневной лимит убытков: $5k
        )
    }

    #[test]
    fn test_valid_order_passes_risk_check() {
        let rm = create_risk_manager();
        let order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 1.0).unwrap();

        let result = rm.validate_order(&order, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_order_value_exceeded() {
        let rm = create_risk_manager();
        let order = Order::new("BTCUSDT", OrderSide::Buy, 50000.0, 3.0).unwrap(); // $150k

        let result = rm.validate_order(&order, 0.0);
        assert!(matches!(
            result,
            Err(RiskError::OrderValueExceeded { .. })
        ));
    }

    #[test]
    fn test_position_size_exceeded_on_buy() {
        let rm = create_risk_manager();
        let order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 5.0).unwrap();

        // Текущая позиция 8 BTC + покупка 5 BTC = 13 BTC > 10 BTC лимит
        let result = rm.validate_order(&order, 8.0);
        assert!(matches!(
            result,
            Err(RiskError::PositionSizeExceeded { .. })
        ));
    }

    #[test]
    fn test_position_size_exceeded_on_sell() {
        let rm = create_risk_manager();
        let order = Order::new("BTCUSDT", OrderSide::Sell, 42000.0, 5.0).unwrap();

        // Текущая позиция -8 BTC + продажа 5 BTC = -13 BTC, abs() > 10 BTC лимит
        let result = rm.validate_order(&order, -8.0);
        assert!(matches!(
            result,
            Err(RiskError::PositionSizeExceeded { .. })
        ));
    }

    #[test]
    fn test_daily_loss_limit() {
        let mut rm = create_risk_manager();
        rm.record_loss(5000.0); // исчерпали лимит

        let order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 0.1).unwrap();
        let result = rm.validate_order(&order, 0.0);

        assert!(matches!(
            result,
            Err(RiskError::DailyLossLimitReached { .. })
        ));
    }

    #[test]
    fn test_reset_daily_loss() {
        let mut rm = create_risk_manager();
        rm.record_loss(5000.0);
        rm.reset_daily_loss();

        let order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 0.1).unwrap();
        let result = rm.validate_order(&order, 0.0);

        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_case_exactly_at_limit() {
        let rm = create_risk_manager();
        // Ордер ровно на $100k — должен пройти
        let order = Order::new("BTCUSDT", OrderSide::Buy, 50000.0, 2.0).unwrap();

        let result = rm.validate_order(&order, 0.0);
        assert!(result.is_ok());
    }
}
```

## Тестирование стратегий

```rust
// src/strategy.rs
use crate::order::{Order, OrderSide};

/// Сигнал торговой стратегии
#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Простая стратегия на основе SMA кроссовера
pub struct SmaCrossover {
    short_period: usize,
    long_period: usize,
}

impl SmaCrossover {
    pub fn new(short_period: usize, long_period: usize) -> Result<Self, StrategyError> {
        if short_period == 0 || long_period == 0 {
            return Err(StrategyError::InvalidPeriod("Период должен быть больше 0".to_string()));
        }
        if short_period >= long_period {
            return Err(StrategyError::InvalidPeriod(
                "Короткий период должен быть меньше длинного".to_string()
            ));
        }

        Ok(SmaCrossover { short_period, long_period })
    }

    pub fn calculate_sma(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let sum: f64 = prices[prices.len() - period..].iter().sum();
        Some(sum / period as f64)
    }

    pub fn generate_signal(&self, prices: &[f64]) -> Result<Signal, StrategyError> {
        if prices.len() < self.long_period {
            return Err(StrategyError::InsufficientData {
                required: self.long_period,
                available: prices.len(),
            });
        }

        let short_sma = self.calculate_sma(prices, self.short_period)
            .ok_or(StrategyError::CalculationError)?;
        let long_sma = self.calculate_sma(prices, self.long_period)
            .ok_or(StrategyError::CalculationError)?;

        // Проверяем предыдущие значения для определения кроссовера
        if prices.len() > self.long_period {
            let prev_prices = &prices[..prices.len() - 1];
            let prev_short = self.calculate_sma(prev_prices, self.short_period);
            let prev_long = self.calculate_sma(prev_prices, self.long_period);

            if let (Some(ps), Some(pl)) = (prev_short, prev_long) {
                // Золотой крест: короткая SMA пересекает длинную снизу вверх
                if ps <= pl && short_sma > long_sma {
                    return Ok(Signal::Buy);
                }
                // Мёртвый крест: короткая SMA пересекает длинную сверху вниз
                if ps >= pl && short_sma < long_sma {
                    return Ok(Signal::Sell);
                }
            }
        }

        Ok(Signal::Hold)
    }
}

#[derive(Debug, PartialEq)]
pub enum StrategyError {
    InvalidPeriod(String),
    InsufficientData { required: usize, available: usize },
    CalculationError,
}

impl std::fmt::Display for StrategyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyError::InvalidPeriod(msg) => write!(f, "Некорректный период: {}", msg),
            StrategyError::InsufficientData { required, available } => {
                write!(f, "Недостаточно данных: нужно {}, есть {}", required, available)
            }
            StrategyError::CalculationError => write!(f, "Ошибка расчёта"),
        }
    }
}

impl std::error::Error for StrategyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_strategy() {
        let strategy = SmaCrossover::new(5, 20);
        assert!(strategy.is_ok());
    }

    #[test]
    fn test_invalid_period_zero() {
        let result = SmaCrossover::new(0, 20);
        assert!(matches!(result, Err(StrategyError::InvalidPeriod(_))));
    }

    #[test]
    fn test_invalid_period_short_greater_than_long() {
        let result = SmaCrossover::new(20, 5);
        assert!(matches!(result, Err(StrategyError::InvalidPeriod(_))));
    }

    #[test]
    fn test_calculate_sma() {
        let strategy = SmaCrossover::new(3, 5).unwrap();
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];

        let sma3 = strategy.calculate_sma(&prices, 3);
        assert_eq!(sma3, Some(40.0)); // (30 + 40 + 50) / 3

        let sma5 = strategy.calculate_sma(&prices, 5);
        assert_eq!(sma5, Some(30.0)); // (10 + 20 + 30 + 40 + 50) / 5
    }

    #[test]
    fn test_sma_insufficient_data() {
        let strategy = SmaCrossover::new(3, 5).unwrap();
        let prices = vec![10.0, 20.0];

        let result = strategy.calculate_sma(&prices, 3);
        assert_eq!(result, None);
    }

    #[test]
    fn test_signal_insufficient_data() {
        let strategy = SmaCrossover::new(5, 20).unwrap();
        let prices = vec![100.0; 10]; // только 10 цен, нужно 20

        let result = strategy.generate_signal(&prices);
        assert!(matches!(result, Err(StrategyError::InsufficientData { .. })));
    }

    #[test]
    fn test_hold_signal_no_crossover() {
        let strategy = SmaCrossover::new(3, 5).unwrap();
        // Восходящий тренд без кроссовера
        let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0, 105.0];

        let signal = strategy.generate_signal(&prices).unwrap();
        assert_eq!(signal, Signal::Hold);
    }

    #[test]
    fn test_buy_signal_golden_cross() {
        let strategy = SmaCrossover::new(2, 4).unwrap();
        // Создаём ситуацию золотого креста
        // Сначала короткая SMA ниже длинной, потом выше
        let prices = vec![
            100.0, 90.0, 80.0, 70.0,  // короткая ниже длинной
            110.0, 120.0,              // резкий рост, короткая пересекает длинную
        ];

        let signal = strategy.generate_signal(&prices).unwrap();
        assert_eq!(signal, Signal::Buy);
    }

    #[test]
    fn test_sell_signal_death_cross() {
        let strategy = SmaCrossover::new(2, 4).unwrap();
        // Создаём ситуацию мёртвого креста
        let prices = vec![
            100.0, 110.0, 120.0, 130.0, // короткая выше длинной
            80.0, 70.0,                  // резкое падение
        ];

        let signal = strategy.generate_signal(&prices).unwrap();
        assert_eq!(signal, Signal::Sell);
    }
}
```

## Конфигурация CI для полного тестирования

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      - run: cargo check --all-features

  test:
    name: Test Suite
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --lib --verbose

      - name: Run integration tests
        run: cargo test --test '*' --verbose

      - name: Run doc tests
        run: cargo test --doc --verbose

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Generate coverage report
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: true

  test-matrix:
    name: Test on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@${{ matrix.rust }}
      - run: cargo test --all-features
```

## Интеграционные тесты

```rust
// tests/integration_test.rs
use trading_bot::{
    order::{Order, OrderSide, OrderStatus},
    risk::RiskManager,
    strategy::{SmaCrossover, Signal},
};

/// Интеграционный тест: полный цикл от сигнала до исполнения
#[test]
fn test_full_trading_cycle() {
    // 1. Создаём стратегию
    let strategy = SmaCrossover::new(5, 20).unwrap();

    // 2. Подготавливаем данные (восходящий тренд с кроссовером)
    let mut prices: Vec<f64> = (0..25)
        .map(|i| 40000.0 + (i as f64) * 100.0)
        .collect();

    // 3. Генерируем сигнал
    let signal = strategy.generate_signal(&prices);
    assert!(signal.is_ok());

    // 4. Если есть сигнал на покупку — создаём ордер
    if signal.unwrap() == Signal::Buy {
        let current_price = *prices.last().unwrap();
        let order = Order::new("BTCUSDT", OrderSide::Buy, current_price, 0.5);
        assert!(order.is_ok());

        // 5. Проверяем через риск-менеджер
        let rm = RiskManager::new(10.0, 100000.0, 5000.0);
        let risk_check = rm.validate_order(&order.as_ref().unwrap(), 0.0);
        assert!(risk_check.is_ok());
    }
}

/// Тест: риск-менеджер блокирует опасный ордер
#[test]
fn test_risk_manager_blocks_dangerous_order() {
    let rm = RiskManager::new(1.0, 50000.0, 1000.0); // строгие лимиты

    // Ордер слишком большой
    let order = Order::new("BTCUSDT", OrderSide::Buy, 45000.0, 2.0).unwrap();
    let result = rm.validate_order(&order, 0.0);

    assert!(result.is_err());
}

/// Тест: серия ордеров в рамках лимитов
#[test]
fn test_multiple_orders_within_limits() {
    let rm = RiskManager::new(5.0, 100000.0, 10000.0);
    let mut current_position = 0.0;

    // Три успешные покупки
    for i in 1..=3 {
        let order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 1.0).unwrap();
        let result = rm.validate_order(&order, current_position);
        assert!(result.is_ok(), "Ордер {} должен пройти", i);
        current_position += 1.0;
    }

    // Четвёртая покупка превысит лимит позиции
    let order = Order::new("BTCUSDT", OrderSide::Buy, 42000.0, 3.0).unwrap();
    let result = rm.validate_order(&order, current_position);
    assert!(result.is_err());
}

/// Тест: корректная работа стратегии на реальных данных
#[test]
fn test_strategy_with_realistic_data() {
    let strategy = SmaCrossover::new(10, 30).unwrap();

    // Симулируем реальные данные с волатильностью
    let base_price = 42000.0;
    let prices: Vec<f64> = (0..50)
        .map(|i| {
            let trend = (i as f64) * 50.0;
            let noise = ((i as f64) * 0.5).sin() * 200.0;
            base_price + trend + noise
        })
        .collect();

    let signal = strategy.generate_signal(&prices);
    assert!(signal.is_ok());

    // Сигнал должен быть одним из трёх возможных
    let signal = signal.unwrap();
    assert!(
        signal == Signal::Buy || signal == Signal::Sell || signal == Signal::Hold
    );
}
```

## Тестирование с моками

```rust
// tests/mock_tests.rs
use std::collections::HashMap;

/// Мок для API биржи
pub struct MockExchangeApi {
    prices: HashMap<String, f64>,
    orders: Vec<MockOrder>,
    should_fail: bool,
}

#[derive(Debug, Clone)]
pub struct MockOrder {
    pub id: String,
    pub symbol: String,
    pub filled: bool,
}

impl MockExchangeApi {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        prices.insert("BTCUSDT".to_string(), 42000.0);
        prices.insert("ETHUSDT".to_string(), 2500.0);

        MockExchangeApi {
            prices,
            orders: vec![],
            should_fail: false,
        }
    }

    pub fn set_should_fail(&mut self, fail: bool) {
        self.should_fail = fail;
    }

    pub fn get_price(&self, symbol: &str) -> Result<f64, MockApiError> {
        if self.should_fail {
            return Err(MockApiError::ConnectionFailed);
        }

        self.prices
            .get(symbol)
            .copied()
            .ok_or(MockApiError::SymbolNotFound(symbol.to_string()))
    }

    pub fn place_order(&mut self, symbol: &str) -> Result<String, MockApiError> {
        if self.should_fail {
            return Err(MockApiError::OrderRejected);
        }

        let order_id = format!("MOCK-{}", self.orders.len() + 1);
        self.orders.push(MockOrder {
            id: order_id.clone(),
            symbol: symbol.to_string(),
            filled: false,
        });

        Ok(order_id)
    }

    pub fn get_order_status(&self, order_id: &str) -> Result<bool, MockApiError> {
        self.orders
            .iter()
            .find(|o| o.id == order_id)
            .map(|o| o.filled)
            .ok_or(MockApiError::OrderNotFound(order_id.to_string()))
    }
}

#[derive(Debug, PartialEq)]
pub enum MockApiError {
    ConnectionFailed,
    SymbolNotFound(String),
    OrderRejected,
    OrderNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_get_price() {
        let api = MockExchangeApi::new();

        let btc_price = api.get_price("BTCUSDT");
        assert_eq!(btc_price, Ok(42000.0));

        let unknown = api.get_price("UNKNOWN");
        assert!(matches!(unknown, Err(MockApiError::SymbolNotFound(_))));
    }

    #[test]
    fn test_mock_place_order() {
        let mut api = MockExchangeApi::new();

        let order1 = api.place_order("BTCUSDT");
        assert!(order1.is_ok());
        assert_eq!(order1.unwrap(), "MOCK-1");

        let order2 = api.place_order("ETHUSDT");
        assert!(order2.is_ok());
        assert_eq!(order2.unwrap(), "MOCK-2");
    }

    #[test]
    fn test_mock_api_failure() {
        let mut api = MockExchangeApi::new();
        api.set_should_fail(true);

        let price = api.get_price("BTCUSDT");
        assert_eq!(price, Err(MockApiError::ConnectionFailed));

        let order = api.place_order("BTCUSDT");
        assert_eq!(order, Err(MockApiError::OrderRejected));
    }

    #[test]
    fn test_order_lifecycle_with_mock() {
        let mut api = MockExchangeApi::new();

        // 1. Получаем цену
        let price = api.get_price("BTCUSDT").unwrap();
        assert!(price > 0.0);

        // 2. Размещаем ордер
        let order_id = api.place_order("BTCUSDT").unwrap();

        // 3. Проверяем статус
        let is_filled = api.get_order_status(&order_id).unwrap();
        assert!(!is_filled); // изначально не заполнен
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **CI** | Continuous Integration — автоматическая проверка кода при каждом коммите |
| **GitHub Actions** | Платформа для CI/CD от GitHub |
| **cargo test** | Команда для запуска всех тестов |
| **Unit-тесты** | Тестирование отдельных функций и модулей |
| **Интеграционные тесты** | Тестирование взаимодействия компонентов |
| **Покрытие кода** | Метрика показывающая какой % кода покрыт тестами |
| **Моки** | Имитация внешних зависимостей для изолированного тестирования |
| **Матрица тестов** | Запуск тестов на разных ОС и версиях Rust |

## Практические задания

1. **CI для торговой библиотеки**: Настрой GitHub Actions для запуска:
   - Компиляции на stable и beta версиях Rust
   - Unit-тестов для всех модулей
   - Проверки покрытия кода с отчётом
   - Тестов на Linux, macOS и Windows

2. **Тестирование риск-менеджера**: Напиши полный набор тестов для:
   - Всех типов ошибок валидации
   - Граничных случаев (ровно на лимите)
   - Комбинаций нескольких ордеров
   - Сброса дневных лимитов

3. **Мок API биржи**: Создай мок для тестирования:
   - Обработки сетевых ошибок
   - Таймаутов запросов
   - Retry логики
   - Rate limiting

4. **Интеграционные тесты стратегии**: Напиши тесты для:
   - Полного торгового цикла от сигнала до исполнения
   - Работы стратегии на разных рыночных условиях
   - Обработки ошибок на каждом этапе

## Домашнее задание

1. **Полная CI конфигурация**: Создай `.github/workflows/ci.yml`:
   - Запускай тесты при push и pull request
   - Добавь кеширование зависимостей
   - Настрой матрицу для 3 ОС и 2 версий Rust
   - Включи проверку покрытия с загрузкой в Codecov
   - Добавь проверку форматирования (rustfmt) и линтер (clippy)

2. **Тестирование с таймаутами**: Реализуй:
   - Тесты для асинхронных операций с таймаутами
   - Проверку корректной отмены долгих операций
   - Тесты для retry с exponential backoff

3. **Property-based тестирование**: Используй proptest для:
   - Генерации случайных ордеров и проверки инвариантов
   - Тестирования стратегий на случайных данных
   - Проверки риск-менеджера на граничных случаях

4. **Benchmark тесты**: Настрой criterion для:
   - Замера производительности расчёта индикаторов
   - Сравнения разных реализаций стратегий
   - Тестирования производительности риск-менеджера

## Навигация

[← Предыдущий день](../346-*/ru.md) | [Следующий день →](../348-*/ru.md)
