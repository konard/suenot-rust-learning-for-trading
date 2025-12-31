use std::time::Instant;

#[derive(Debug, Clone)]
struct PriceData {
    timestamp: u64,
    price: f64,
    volume: f64,
}

/// Calculate Simple Moving Average (slow version)
fn calculate_sma_slow(prices: &[f64], window: usize) -> Vec<f64> {
    let mut result = Vec::new();

    for i in window..=prices.len() {
        let sum: f64 = prices[i - window..i].iter().sum();
        result.push(sum / window as f64);
    }

    result
}

/// Calculate Exponential Moving Average (slow version)
fn calculate_ema_slow(prices: &[f64], period: usize) -> Vec<f64> {
    let mut result = Vec::new();
    let multiplier = 2.0 / (period as f64 + 1.0);

    // First EMA is SMA
    let initial_sma: f64 = prices[..period].iter().sum::<f64>() / period as f64;
    result.push(initial_sma);

    for i in period..prices.len() {
        let ema = (prices[i] - result.last().unwrap()) * multiplier + result.last().unwrap();
        result.push(ema);
    }

    result
}

/// Calculate RSI (slow version with repeated allocations)
fn calculate_rsi_slow(prices: &[f64], period: usize) -> Vec<f64> {
    let mut rsi_values = Vec::new();

    for i in period..prices.len() {
        let mut gains = Vec::new();  // Allocation in hot loop!
        let mut losses = Vec::new(); // Allocation in hot loop!

        for j in i - period + 1..=i {
            let change = prices[j] - prices[j - 1];
            if change > 0.0 {
                gains.push(change);
            } else {
                losses.push(change.abs());
            }
        }

        let avg_gain = if !gains.is_empty() {
            gains.iter().sum::<f64>() / gains.len() as f64
        } else {
            0.0
        };

        let avg_loss = if !losses.is_empty() {
            losses.iter().sum::<f64>() / losses.len() as f64
        } else {
            0.0
        };

        let rsi = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - (100.0 / (1.0 + (avg_gain / avg_loss)))
        };

        rsi_values.push(rsi);
    }

    rsi_values
}

/// Process market data with multiple indicators
fn analyze_market(prices: &[f64]) {
    let start = Instant::now();

    println!("Calculating indicators for {} price points...", prices.len());

    // These calculations will show up in flamegraph
    let sma_20 = calculate_sma_slow(prices, 20);
    let sma_50 = calculate_sma_slow(prices, 50);
    let ema_12 = calculate_ema_slow(prices, 12);
    let ema_26 = calculate_ema_slow(prices, 26);
    let rsi_14 = calculate_rsi_slow(prices, 14);

    println!("SMA(20): {} values", sma_20.len());
    println!("SMA(50): {} values", sma_50.len());
    println!("EMA(12): {} values", ema_12.len());
    println!("EMA(26): {} values", ema_26.len());
    println!("RSI(14): {} values", rsi_14.len());

    println!("Analysis completed in {:?}", start.elapsed());
}

/// Generate test price data
fn generate_price_data(count: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(count);
    let mut price = 50000.0;

    for i in 0..count {
        // Simulate price movement
        let change = ((i * 7) % 100) as f64 - 50.0;
        price += change;
        prices.push(price);
    }

    prices
}

fn main() {
    // Generate a large dataset to make profiling visible
    let prices = generate_price_data(1000);

    // Run analysis
    println!("\n=== Testing Chapter 306 Code ===");
    analyze_market(&prices);
}
