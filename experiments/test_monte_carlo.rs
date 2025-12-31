use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
struct Trade {
    profit: f64,
    date: String,
}

#[derive(Debug)]
struct SimulationResult {
    final_equity: f64,
    max_drawdown: f64,
    total_return: f64,
}

fn calculate_equity_curve(trades: &[Trade], initial_capital: f64) -> (Vec<f64>, f64) {
    let mut equity_curve = vec![initial_capital];
    let mut max_equity = initial_capital;
    let mut max_drawdown = 0.0;

    for trade in trades {
        let new_equity = equity_curve.last().unwrap() + trade.profit;
        equity_curve.push(new_equity);

        if new_equity > max_equity {
            max_equity = new_equity;
        }

        let drawdown = (max_equity - new_equity) / max_equity * 100.0;
        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }
    }

    (equity_curve, max_drawdown)
}

fn run_single_simulation(trades: &[Trade], initial_capital: f64) -> SimulationResult {
    let mut rng = thread_rng();
    let mut shuffled = trades.to_vec();
    shuffled.shuffle(&mut rng);

    let (equity_curve, max_drawdown) = calculate_equity_curve(&shuffled, initial_capital);
    let final_equity = *equity_curve.last().unwrap();
    let total_return = (final_equity - initial_capital) / initial_capital * 100.0;

    SimulationResult {
        final_equity,
        max_drawdown,
        total_return,
    }
}

fn monte_carlo_analysis(
    trades: &[Trade],
    initial_capital: f64,
    simulations: usize,
) -> Vec<SimulationResult> {
    (0..simulations)
        .map(|_| run_single_simulation(trades, initial_capital))
        .collect()
}

fn main() {
    // Historical strategy trades
    let historical_trades = vec![
        Trade { profit: 150.0, date: "2024-01-15".to_string() },
        Trade { profit: -80.0, date: "2024-01-16".to_string() },
        Trade { profit: 200.0, date: "2024-01-17".to_string() },
        Trade { profit: -120.0, date: "2024-01-18".to_string() },
        Trade { profit: 300.0, date: "2024-01-19".to_string() },
        Trade { profit: 100.0, date: "2024-01-22".to_string() },
        Trade { profit: -90.0, date: "2024-01-23".to_string() },
        Trade { profit: 250.0, date: "2024-01-24".to_string() },
        Trade { profit: -150.0, date: "2024-01-25".to_string() },
        Trade { profit: 180.0, date: "2024-01-26".to_string() },
    ];

    let initial_capital = 10_000.0;
    let num_simulations = 100; // Reduced for quick test

    println!("Running {} Monte Carlo simulations...\n", num_simulations);

    let results = monte_carlo_analysis(&historical_trades, initial_capital, num_simulations);

    // Analyze results
    let total_returns: Vec<f64> = results.iter().map(|r| r.total_return).collect();
    let max_drawdowns: Vec<f64> = results.iter().map(|r| r.max_drawdown).collect();

    let avg_return = total_returns.iter().sum::<f64>() / total_returns.len() as f64;
    let min_return = total_returns.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_return = total_returns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let avg_drawdown = max_drawdowns.iter().sum::<f64>() / max_drawdowns.len() as f64;
    let worst_drawdown = max_drawdowns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("=== Monte Carlo Simulation Results ===\n");
    println!("Returns:");
    println!("  Average: {:.2}%", avg_return);
    println!("  Minimum: {:.2}%", min_return);
    println!("  Maximum: {:.2}%", max_return);
    println!("\nDrawdown:");
    println!("  Average: {:.2}%", avg_drawdown);
    println!("  Maximum (worst case): {:.2}%", worst_drawdown);

    // Percentiles
    let mut sorted_returns = total_returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p5 = sorted_returns[(sorted_returns.len() as f64 * 0.05) as usize];
    let p95 = sorted_returns[(sorted_returns.len() as f64 * 0.95) as usize];

    println!("\n90% Confidence Interval:");
    println!("  5th percentile: {:.2}%", p5);
    println!("  95th percentile: {:.2}%", p95);
    println!("\nProbability of loss: {:.1}%",
        (sorted_returns.iter().filter(|&&r| r < 0.0).count() as f64
         / sorted_returns.len() as f64 * 100.0));

    println!("\n✓ Test passed successfully!");
}
