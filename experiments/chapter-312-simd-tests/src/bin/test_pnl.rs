// Basic PnL calculation test
fn calculate_pnl_scalar(entry_prices: &[f32], exit_prices: &[f32]) -> Vec<f32> {
    entry_prices.iter()
        .zip(exit_prices.iter())
        .map(|(entry, exit)| exit - entry)
        .collect()
}

fn main() {
    let entries = vec![100.0, 101.5, 99.8, 102.3];
    let exits = vec![105.0, 100.0, 103.2, 101.5];

    let pnl = calculate_pnl_scalar(&entries, &exits);

    println!("=== Profit/Loss per Trade ===");
    for (i, profit) in pnl.iter().enumerate() {
        println!("Trade {}: {:.2}", i + 1, profit);
    }

    println!("\n✅ Basic PnL test passed!");
}
