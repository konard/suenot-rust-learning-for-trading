# Day 289: Results Visualization

## Trading Analogy

Imagine you've spent a month trading with a new strategy and made 15% profit. Great result! But how do you present it to an investor? Simply saying "15% profit" isn't enough. The investor will want to see:

- **Equity growth chart** — how your balance changed day by day
- **Drawdown chart** — were there periods of losses? How deep?
- **Profit distribution** — did you earn on many small trades or a couple of large ones?
- **Time-based statistics** — when does the strategy work best?

This is **results visualization** — transforming numerical metrics into clear charts and diagrams that help quickly assess the effectiveness of a trading strategy.

In real trading, visualization helps to:
- Quickly evaluate strategy quality
- Detect drawdown periods
- Find patterns in results
- Present results to investors

## What is Results Visualization?

**Backtesting results visualization** is the process of creating charts, diagrams, and visual representations of trading strategy metrics for analysis and presentation.

Main types of visualizations:
1. **Equity Curve** — how balance changed over time
2. **Drawdown Chart** — depth and duration of losses
3. **Trade Distribution** — histogram of profits/losses
4. **Monthly Returns** — heatmap of results
5. **Win/Loss Analysis** — ratio of successful trades

In Rust, we can use these libraries for visualization:
- **plotters** — creating charts in pure Rust
- **plotly** — interactive charts via JavaScript
- **textplots** — simple ASCII charts in terminal

## Basic Example: ASCII Chart in Terminal

Let's start simple — create an ASCII equity curve chart right in the terminal:

```rust
use textplots::{Chart, Plot, Shape};

fn main() {
    // Backtesting results: balance by day
    let equity_data: Vec<(f32, f32)> = vec![
        (1.0, 10000.0),
        (2.0, 10150.0),
        (3.0, 10080.0),
        (4.0, 10300.0),
        (5.0, 10450.0),
        (6.0, 10200.0),
        (7.0, 10600.0),
        (8.0, 10750.0),
        (9.0, 10900.0),
        (10.0, 11100.0),
    ];

    println!("=== Equity Curve ===\n");

    Chart::new(180, 60, 0.0, 11.0)
        .lineplot(&Shape::Lines(&equity_data))
        .display();

    // Calculate metrics
    let initial_balance = equity_data.first().unwrap().1;
    let final_balance = equity_data.last().unwrap().1;
    let total_return = ((final_balance - initial_balance) / initial_balance) * 100.0;

    println!("\n=== Metrics ===");
    println!("Initial balance: ${:.2}", initial_balance);
    println!("Final balance:   ${:.2}", final_balance);
    println!("Total return:    {:.2}%", total_return);
}
```

**Dependencies for Cargo.toml:**
```toml
[dependencies]
textplots = "0.8"
```

## Data Structures for Visualization

Let's create structures to store backtesting results:

```rust
use chrono::{DateTime, Utc};

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
    drawdown: f64, // Percentage from peak
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
```

## Creating Charts with plotters

Now let's create real charts in PNG files:

```rust
use plotters::prelude::*;
use chrono::{DateTime, Utc, TimeZone};

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

    // Draw equity line
    chart.draw_series(LineSeries::new(
        results.equity_curve.iter().map(|ep| (ep.timestamp, ep.balance)),
        &BLUE,
    ))?
    .label("Balance")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Draw initial level
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

    // Draw drawdown area
    chart.draw_series(AreaSeries::new(
        results.equity_curve.iter().map(|ep| (ep.timestamp, ep.drawdown)),
        0.0,
        &RED.mix(0.3),
    ))?;

    // Drawdown line
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
```

**Dependencies for Cargo.toml:**
```toml
[dependencies]
plotters = "0.3"
chrono = "0.4"
```

## Profit Distribution Histogram

Let's create a histogram showing the distribution of profits and losses across trades:

```rust
fn plot_profit_distribution(results: &BacktestResults, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    // Collect profit data
    let profits: Vec<f64> = results.trades.iter().map(|t| t.profit).collect();

    if profits.is_empty() {
        return Ok(());
    }

    // Determine range
    let min_profit = profits.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_profit = profits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Create bins for histogram
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

    // Draw bars
    chart.draw_series(
        bins.iter().enumerate().map(|(i, &count)| {
            let x0 = min_profit + i as f64 * bin_width;
            let x1 = x0 + bin_width;
            let color = if x0 + bin_width / 2.0 >= 0.0 { &GREEN } else { &RED };

            Rectangle::new([(x0, 0), (x1, count)], color.mix(0.5).filled())
        })
    )?;

    // Vertical line at zero
    chart.draw_series(LineSeries::new(
        vec![(0.0, 0), (0.0, max_count)],
        &BLACK,
    ))?;

    root.present()?;
    println!("Profit distribution saved to {}", filename);

    Ok(())
}
```

## Complete Example with Visualization

Let's put everything together in a working example:

```rust
use chrono::{DateTime, Utc, Duration};
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create backtesting results
    let mut results = BacktestResults::new(10_000.0);

    // Simulate trades
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

    // Calculate equity curve
    results.calculate_equity_curve();

    // Print metrics
    println!("=== Backtesting Results ===");
    println!("Total trades:      {}", results.trades.len());
    println!("Winning trades:    {}", results.winning_trades());
    println!("Losing trades:     {}", results.losing_trades());
    println!("Win rate:          {:.2}%", results.win_rate());
    println!("Total return:      {:.2}%", results.total_return());
    println!("Max drawdown:      {:.2}%", results.max_drawdown());
    println!("Initial balance:   ${:.2}", results.initial_balance);
    println!("Final balance:     ${:.2}", results.final_balance);

    // Create charts
    plot_equity_curve(&results, "equity_curve.png")?;
    plot_drawdown(&results, "drawdown.png")?;
    plot_profit_distribution(&results, "profit_distribution.png")?;

    println!("\nAll charts generated successfully!");

    Ok(())
}
```

## Interactive HTML Visualization

For more advanced visualization, we can generate HTML with interactive charts:

```rust
use std::fs::File;
use std::io::Write;

fn generate_html_report(results: &BacktestResults, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let equity_data: Vec<String> = results.equity_curve.iter()
        .map(|ep| format!("{{ x: '{}', y: {} }}",
            ep.timestamp.format("%Y-%m-%d"),
            ep.balance))
        .collect();

    let drawdown_data: Vec<String> = results.equity_curve.iter()
        .map(|ep| format!("{{ x: '{}', y: {} }}",
            ep.timestamp.format("%Y-%m-%d"),
            ep.drawdown))
        .collect();

    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Backtesting Results</title>
    <script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .metrics {{ margin: 20px 0; padding: 20px; background: #f0f0f0; border-radius: 5px; }}
        .chart {{ margin: 20px 0; }}
    </style>
</head>
<body>
    <h1>Backtesting Results Report</h1>

    <div class="metrics">
        <h2>Key Metrics</h2>
        <p><strong>Total Trades:</strong> {}</p>
        <p><strong>Win Rate:</strong> {:.2}%</p>
        <p><strong>Total Return:</strong> {:.2}%</p>
        <p><strong>Max Drawdown:</strong> {:.2}%</p>
        <p><strong>Final Balance:</strong> ${:.2}</p>
    </div>

    <div id="equityChart" class="chart"></div>
    <div id="drawdownChart" class="chart"></div>

    <script>
        const equityData = [{}];
        const drawdownData = [{}];

        Plotly.newPlot('equityChart', [{{
            x: equityData.map(d => d.x),
            y: equityData.map(d => d.y),
            type: 'scatter',
            mode: 'lines',
            name: 'Equity',
            line: {{ color: 'blue' }}
        }}], {{
            title: 'Equity Curve',
            xaxis: {{ title: 'Date' }},
            yaxis: {{ title: 'Balance ($)' }}
        }});

        Plotly.newPlot('drawdownChart', [{{
            x: drawdownData.map(d => d.x),
            y: drawdownData.map(d => d.y),
            type: 'scatter',
            mode: 'lines',
            fill: 'tozeroy',
            name: 'Drawdown',
            line: {{ color: 'red' }}
        }}], {{
            title: 'Drawdown',
            xaxis: {{ title: 'Date' }},
            yaxis: {{ title: 'Drawdown (%)' }}
        }});
    </script>
</body>
</html>
    "#,
        results.trades.len(),
        results.win_rate(),
        results.total_return(),
        results.max_drawdown(),
        results.final_balance,
        equity_data.join(",\n        "),
        drawdown_data.join(",\n        ")
    );

    let mut file = File::create(filename)?;
    file.write_all(html.as_bytes())?;

    println!("HTML report saved to {}", filename);

    Ok(())
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Results Visualization | Transforming metrics into charts for analysis |
| Equity Curve | Chart of balance changes over time |
| Drawdown Chart | Chart of capital drawdowns |
| Profit Distribution | P&L histogram across trades |
| `textplots` | ASCII charts in terminal |
| `plotters` | Creating PNG charts |
| HTML Reports | Interactive charts with plotly.js |

## Homework

1. **ASCII Dashboard**: Create a function that displays a beautiful ASCII dashboard in the terminal with:
   - Equity curve
   - Key metrics (win rate, total return, max DD)
   - Last 10 trades

2. **Monthly Returns**: Implement a `plot_monthly_returns()` function that creates a heatmap of returns by month (rows) and year (columns).

3. **Strategy Comparison**: Write a `compare_strategies()` function that takes results from 2-3 strategies and plots their equity curves on one chart for comparison.

4. **Extended HTML Report**: Enhance the HTML report with:
   - Table of all trades
   - Profit distribution chart
   - Day-of-week statistics (which days the strategy performs best)
   - Underwater equity chart (recovery time after drawdowns)

## Navigation

[← Previous day](../288-report-generation/en.md) | [Next day →](../290-walk-forward-analysis/en.md)
