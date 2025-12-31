use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// Simple order structure
#[derive(Debug, Clone)]
struct Order {
    symbol: String,
    quantity: f64,
    price: f64,
}

// Function to benchmark: calculate total order value
fn calculate_order_value(order: &Order) -> f64 {
    order.quantity * order.price
}

// Basic benchmark
fn bench_order_value(c: &mut Criterion) {
    let order = Order {
        symbol: "BTCUSDT".to_string(),
        quantity: 1.5,
        price: 42000.0,
    };

    c.bench_function("calculate_order_value", |b| {
        b.iter(|| calculate_order_value(black_box(&order)))
    });
}

// Benchmark with different inputs
fn bench_order_value_with_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_value_by_size");

    for quantity in [0.1, 1.0, 10.0, 100.0].iter() {
        let order = Order {
            symbol: "BTCUSDT".to_string(),
            quantity: *quantity,
            price: 42000.0,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(quantity),
            &order,
            |b, order| {
                b.iter(|| calculate_order_value(black_box(order)))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_order_value, bench_order_value_with_sizes);
criterion_main!(benches);
