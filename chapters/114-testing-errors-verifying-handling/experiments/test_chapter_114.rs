// Test file to verify Chapter 114 code examples compile correctly

// Example 1: Basic Result Testing
fn validate_order_price(price: f64) -> Result<f64, String> {
    if price <= 0.0 {
        return Err(String::from("Price must be positive"));
    }
    if price > 1_000_000.0 {
        return Err(String::from("Price exceeds limit"));
    }
    Ok(price)
}

// Example 2: Position size calculation
fn calculate_position_size(
    balance: f64,
    risk_percent: f64,
    stop_loss_distance: f64,
) -> Result<f64, String> {
    if balance <= 0.0 {
        return Err(String::from("Balance must be positive"));
    }
    if risk_percent <= 0.0 || risk_percent > 100.0 {
        return Err(String::from("Risk must be between 0 and 100%"));
    }
    if stop_loss_distance <= 0.0 {
        return Err(String::from("Stop loss distance must be positive"));
    }

    let risk_amount = balance * (risk_percent / 100.0);
    Ok(risk_amount / stop_loss_distance)
}

// Example 3: Order error enum
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

// Example 4: Panic function
fn get_price_or_panic(prices: &[f64], index: usize) -> f64 {
    if index >= prices.len() {
        panic!("Index {} out of bounds for price array (length: {})", index, prices.len());
    }
    prices[index]
}

// Example 5: Option functions
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

// Example 6: SMA calculation
fn calculate_sma(prices: &[f64], period: usize) -> Result<f64, String> {
    if prices.is_empty() {
        return Err(String::from("Price array is empty"));
    }
    if period == 0 {
        return Err(String::from("Period must be greater than zero"));
    }
    if period > prices.len() {
        return Err(format!(
            "Insufficient data: need {}, have {}",
            period,
            prices.len()
        ));
    }

    let slice = &prices[prices.len() - period..];
    let sum: f64 = slice.iter().sum();
    Ok(sum / period as f64)
}

// Example 7: Order validation
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
    if order.symbol.is_empty() {
        return Err(ValidationError::EmptySymbol);
    }

    if order.price <= 0.0 {
        return Err(ValidationError::InvalidPrice {
            price: order.price,
            reason: "Price must be positive",
        });
    }
    if order.price.is_nan() || order.price.is_infinite() {
        return Err(ValidationError::InvalidPrice {
            price: order.price,
            reason: "Price must be a finite number",
        });
    }

    if order.quantity <= 0.0 {
        return Err(ValidationError::InvalidQuantity {
            quantity: order.quantity,
            reason: "Quantity must be positive",
        });
    }

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

// Example 8: Trade error
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

fn main() {
    // Test basic validation
    println!("Testing validate_order_price...");
    assert!(validate_order_price(42000.0).is_ok());
    assert!(validate_order_price(-100.0).is_err());
    assert!(validate_order_price(0.0).is_err());
    println!("  OK");

    // Test position size
    println!("Testing calculate_position_size...");
    assert!(calculate_position_size(10000.0, 2.0, 100.0).is_ok());
    assert!(calculate_position_size(0.0, 2.0, 100.0).is_err());
    println!("  OK");

    // Test place_order
    println!("Testing place_order...");
    assert!(matches!(place_order(42000.0, 0.5, 100000.0, false), Err(OrderError::MarketClosed)));
    assert!(matches!(place_order(-100.0, 0.5, 100000.0, true), Err(OrderError::InvalidPrice(_))));
    assert!(place_order(42000.0, 0.5, 100000.0, true).is_ok());
    println!("  OK");

    // Test get_price_or_panic (valid case)
    println!("Testing get_price_or_panic...");
    let prices = [42000.0, 42100.0, 42050.0];
    assert_eq!(get_price_or_panic(&prices, 1), 42100.0);
    println!("  OK");

    // Test find_best_bid
    println!("Testing find_best_bid...");
    assert!(find_best_bid(&[]).is_none());
    let bids = [(41900.0, 1.0), (42000.0, 0.5), (41800.0, 2.0)];
    assert_eq!(find_best_bid(&bids), Some((42000.0, 0.5)));
    println!("  OK");

    // Test calculate_spread
    println!("Testing calculate_spread...");
    let bids = [(42000.0, 0.5)];
    let asks = [(42050.0, 1.0)];
    assert_eq!(calculate_spread(&bids, &asks), Some(50.0));
    println!("  OK");

    // Test calculate_sma
    println!("Testing calculate_sma...");
    assert!(calculate_sma(&[], 3).is_err());
    assert!(calculate_sma(&[42000.0], 0).is_err());
    let prices = [42000.0, 42100.0, 42200.0];
    assert!(calculate_sma(&prices, 3).is_ok());
    println!("  OK");

    // Test validate_order
    println!("Testing validate_order...");
    let valid_order = Order {
        symbol: String::from("BTC/USDT"),
        side: OrderSide::Buy,
        price: 42000.0,
        quantity: 0.1,
    };
    assert!(validate_order(&valid_order).is_ok());
    
    let invalid_order = Order {
        symbol: String::new(),
        side: OrderSide::Buy,
        price: 42000.0,
        quantity: 0.1,
    };
    assert_eq!(validate_order(&invalid_order), Err(ValidationError::EmptySymbol));
    println!("  OK");

    // Test execute_trade
    println!("Testing execute_trade...");
    assert_eq!(execute_trade(1000.0, 100.0, false), Err(TradeError::MarketClosed));
    assert_eq!(execute_trade(1000.0, -50.0, true), Err(TradeError::InvalidAmount));
    assert_eq!(execute_trade(1000.0, 2000.0, true), Err(TradeError::InsufficientFunds));
    assert_eq!(execute_trade(1000.0, 100.0, true), Ok(900.0));
    println!("  OK");

    println!("\nAll tests passed!");
}
