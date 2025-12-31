use chrono::{DateTime, Utc, Duration};
use plotters::prelude::*;

#[derive(Debug, Clone)]
struct Trade {
    timestamp: DateTime<Utc>,
    symbol: String,
    side: TradeSide,
    quantity: f64,
    price: f64,
    profit: f64,
}

#[derive(Debug, Clone)]
enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
struct EquityPoint {
    timestamp: DateTime<Utc>,
    balance: f64,
    drawdown: f64,
}

#[derive(Debug)]
struct BacktestResults {
    trades: Vec<Trade>,
    equity_curve: Vec<EquityPoint>,
    initial_balance: f64,
    final_balance: f64,
}

impl BacktestResults {
    fn new(initial_balance: f64) -> Self {
        BacktestResults {
            trades: Vec::new(),
            equity_curve: Vec::new(),
            initial_balance,
            final_balance: initial_balance,
        }
    }

    fn add_trade(&mut self, trade: Trade) {
        self.final_balance += trade.profit;
        self.trades.push(trade);
    }

    fn calculate_equity_curve(&mut self) {
        let mut balance = self.initial_balance;
        let mut peak = self.initial_balance;

        for trade in &self.trades {
            balance += trade.profit;

            if balance > peak {
                peak = balance;
            }

            let drawdown = ((peak - balance) / peak) * 100.0;

            self.equity_curve.push(EquityPoint {
                timestamp: trade.timestamp,
                balance,
                drawdown,
            });
        }

        self.final_balance = balance;
    }

    fn total_return(&self) -> f64 {
        ((self.final_balance - self.initial_balance) / self.initial_balance) * 100.0
    }

    fn winning_trades(&self) -> usize {
        self.trades.iter().filter(|t| t.profit > 0.0).count()
    }

    fn losing_trades(&self) -> usize {
        self.trades.iter().filter(|t| t.profit < 0.0).count()
    }

    fn win_rate(&self) -> f64 {
        let total = self.trades.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        (self.winning_trades() as f64 / total) * 100.0
    }

    fn max_drawdown(&self) -> f64 {
        self.equity_curve
            .iter()
            .map(|ep| ep.drawdown)
            .fold(0.0, f64::max)
    }
}

fn plot_equity_curve(results: &BacktestResults, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let min_balance = results.equity_curve.iter().map(|ep| ep.balance).fold(f64::INFINITY, f64::min);
    let max_balance = results.equity_curve.iter().map(|ep| ep.balance).fold(f64::NEG_INFINITY, f64::max);

    let min_time = results.equity_curve.first().unwrap().timestamp;
    let max_time = results.equity_curve.last().unwrap().timestamp;

    let mut chart = ChartBuilder::on(&root)
        .caption("Equity Curve", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            min_time..max_time,
            (min_balance * 0.95)..(max_balance * 1.05)
        )?;

    chart.configure_mesh()
        .x_desc("Time")
        .y_desc("Balance ($)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        results.equity_curve.iter().map(|ep| (ep.timestamp, ep.balance)),
        &BLUE,
    ))?
    .label("Balance")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart.draw_series(LineSeries::new(
        vec![
            (min_time, results.initial_balance),
            (max_time, results.initial_balance)
        ],
        &RED.mix(0.5),
    ))?
    .label("Initial Balance")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Equity curve saved to {}", filename);

    Ok(())
}

fn plot_drawdown(results: &BacktestResults, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_dd = results.max_drawdown();
    let min_time = results.equity_curve.first().unwrap().timestamp;
    let max_time = results.equity_curve.last().unwrap().timestamp;

    let mut chart = ChartBuilder::on(&root)
        .caption("Drawdown", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            min_time..max_time,
            0.0..(max_dd * 1.1)
        )?;

    chart.configure_mesh()
        .x_desc("Time")
        .y_desc("Drawdown (%)")
        .draw()?;

    chart.draw_series(AreaSeries::new(
        results.equity_curve.iter().map(|ep| (ep.timestamp, ep.drawdown)),
        0.0,
        &RED.mix(0.3),
    ))?;

    chart.draw_series(LineSeries::new(
        results.equity_curve.iter().map(|ep| (ep.timestamp, ep.drawdown)),
        &RED,
    ))?
    .label(format!("Max DD: {:.2}%", max_dd))
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Drawdown chart saved to {}", filename);

    Ok(())
}

fn plot_profit_distribution(results: &BacktestResults, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let profits: Vec<f64> = results.trades.iter().map(|t| t.profit).collect();

    if profits.is_empty() {
        return Ok(());
    }

    let min_profit = profits.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_profit = profits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let bin_count = 20;
    let bin_width = (max_profit - min_profit) / bin_count as f64;
    let mut bins = vec![0u32; bin_count];

    for &profit in &profits {
        let bin_index = ((profit - min_profit) / bin_width).floor() as usize;
        let bin_index = bin_index.min(bin_count - 1);
        bins[bin_index] += 1;
    }

    let max_count = *bins.iter().max().unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Profit/Loss Distribution", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            min_profit..max_profit,
            0u32..(max_count + 5)
        )?;

    chart.configure_mesh()
        .x_desc("Profit/Loss ($)")
        .y_desc("Number of Trades")
        .draw()?;

    chart.draw_series(
        bins.iter().enumerate().map(|(i, &count)| {
            let x0 = min_profit + i as f64 * bin_width;
            let x1 = x0 + bin_width;
            let color = if x0 + bin_width / 2.0 >= 0.0 { &GREEN } else { &RED };

            Rectangle::new([(x0, 0), (x1, count)], color.mix(0.5).filled())
        })
    )?;

    chart.draw_series(LineSeries::new(
        vec![(0.0, 0), (0.0, max_count)],
        &BLACK,
    ))?;

    root.present()?;
    println!("Profit distribution saved to {}", filename);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Chapter 289: Results Visualization Test ===\n");

    let mut results = BacktestResults::new(10_000.0);

    let start_date = Utc::now() - Duration::days(30);

    let trade_data = vec![
        (1, 250.0),
        (2, -100.0),
        (3, 300.0),
        (4, 150.0),
        (5, -200.0),
        (6, 400.0),
        (7, 100.0),
        (8, -150.0),
        (9, 500.0),
        (10, 200.0),
        (11, -100.0),
        (12, 350.0),
        (13, 150.0),
        (14, -250.0),
        (15, 600.0),
    ];

    for (day, profit) in trade_data {
        results.add_trade(Trade {
            timestamp: start_date + Duration::days(day),
            symbol: "BTC".to_string(),
            side: if profit > 0.0 { TradeSide::Buy } else { TradeSide::Sell },
            quantity: 0.1,
            price: 42000.0,
            profit,
        });
    }

    results.calculate_equity_curve();

    println!("=== Backtesting Results ===");
    println!("Total trades:      {}", results.trades.len());
    println!("Winning trades:    {}", results.winning_trades());
    println!("Losing trades:     {}", results.losing_trades());
    println!("Win rate:          {:.2}%", results.win_rate());
    println!("Total return:      {:.2}%", results.total_return());
    println!("Max drawdown:      {:.2}%", results.max_drawdown());
    println!("Initial balance:   ${:.2}", results.initial_balance);
    println!("Final balance:     ${:.2}", results.final_balance);

    println!("\n=== Generating Charts ===");
    plot_equity_curve(&results, "equity_curve.png")?;
    plot_drawdown(&results, "drawdown.png")?;
    plot_profit_distribution(&results, "profit_distribution.png")?;

    println!("\nAll charts generated successfully!");
    println!("Test completed - code compiles and runs correctly!");

    Ok(())
}
