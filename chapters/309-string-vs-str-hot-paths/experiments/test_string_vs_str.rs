use std::time::Instant;

#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    price: f64,
    quantity: f64,
}

#[derive(Debug)]
struct OrderRef<'a> {
    symbol: &'a str,
    price: f64,
    quantity: f64,
}

fn process_order_owned(symbol: String, price: f64, qty: f64) -> f64 {
    let order = Order {
        symbol,
        price,
        quantity: qty,
    };
    order.price * order.quantity
}

fn process_order_borrowed(symbol: &str, price: f64, qty: f64) -> f64 {
    let order = OrderRef {
        symbol,
        price,
        quantity: qty,
    };
    order.price * order.quantity
}

fn main() {
    let iterations = 1_000_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let symbol = String::from("BTCUSDT");
        process_order_owned(symbol, 50000.0, 0.5);
    }
    let string_duration = start.elapsed();

    let symbol_ref = "BTCUSDT";
    let start = Instant::now();
    for _ in 0..iterations {
        process_order_borrowed(symbol_ref, 50000.0, 0.5);
    }
    let str_duration = start.elapsed();

    println!("=== Performance Comparison ===");
    println!("String version: {:?}", string_duration);
    println!("&str version:   {:?}", str_duration);
    println!("Speedup: {:.2}x", string_duration.as_nanos() as f64 / str_duration.as_nanos() as f64);
}
