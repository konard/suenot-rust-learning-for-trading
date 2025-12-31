# Day 280: Slippage Model

## Trading Analogy

Imagine you're at a busy farmers market trying to buy 100 apples. The sign says "$1 per apple," but when you try to buy all 100, the vendor says: "I only have 30 at $1. The next 40 will cost $1.10, and the remaining 30 will be $1.25." This price movement caused by your own order is **slippage** — the difference between the expected price and the actual execution price.

In real trading, slippage occurs because:
- Your order consumes available liquidity at the best price
- Market conditions change between order placement and execution
- Large orders move the market against you
- Network latency causes price changes

Accurate slippage modeling is crucial for backtesting — without it, your simulated profits may be significantly higher than what you'd achieve in live trading!

## What is Slippage?

Slippage is the difference between the expected price of a trade and the price at which it actually executes. It can be:

1. **Positive slippage** — execution at a better price than expected (rare, but possible)
2. **Negative slippage** — execution at a worse price than expected (common)
3. **Zero slippage** — execution at exactly the expected price (ideal, but unrealistic for large orders)

### Types of Slippage Models

| Model | Description | Use Case |
|-------|-------------|----------|
| Fixed | Constant cost per trade | Simple backtesting |
| Percentage | Cost as % of trade value | General simulations |
| Volume-based | Slippage increases with order size | Realistic modeling |
| Spread-based | Based on bid-ask spread | High-frequency trading |
| Market Impact | Models price impact of large orders | Institutional trading |

## Basic Slippage Model

Let's start with a simple slippage model structure:

```rust
/// Represents different types of slippage models
#[derive(Debug, Clone)]
pub enum SlippageModel {
    /// No slippage (ideal conditions)
    Zero,
    /// Fixed cost per trade in basis points (1 bp = 0.01%)
    Fixed { basis_points: f64 },
    /// Percentage of trade value
    Percentage { rate: f64 },
    /// Volume-dependent slippage
    VolumeImpact {
        base_spread: f64,      // Base bid-ask spread
        impact_factor: f64,    // Price impact per unit of volume ratio
    },
}

/// Order side for determining slippage direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Represents a trade order
#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub expected_price: f64,
}

/// Market data for slippage calculation
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub volume: f64,         // Current market volume
    pub avg_daily_volume: f64, // Average daily volume
}

impl SlippageModel {
    /// Calculate the execution price after slippage
    pub fn calculate_execution_price(
        &self,
        order: &Order,
        market: &MarketData,
    ) -> f64 {
        let slippage = self.calculate_slippage(order, market);

        match order.side {
            OrderSide::Buy => order.expected_price * (1.0 + slippage),
            OrderSide::Sell => order.expected_price * (1.0 - slippage),
        }
    }

    /// Calculate slippage as a decimal (0.01 = 1%)
    pub fn calculate_slippage(
        &self,
        order: &Order,
        market: &MarketData,
    ) -> f64 {
        match self {
            SlippageModel::Zero => 0.0,

            SlippageModel::Fixed { basis_points } => {
                basis_points / 10_000.0 // Convert bp to decimal
            }

            SlippageModel::Percentage { rate } => *rate,

            SlippageModel::VolumeImpact {
                base_spread,
                impact_factor
            } => {
                // Calculate volume ratio (order size vs avg daily volume)
                let volume_ratio = order.quantity / market.avg_daily_volume;

                // Slippage = half spread + market impact
                let half_spread = base_spread / 2.0;
                let market_impact = impact_factor * volume_ratio.sqrt();

                half_spread + market_impact
            }
        }
    }
}

fn main() {
    let order = Order {
        symbol: "BTC-USD".to_string(),
        side: OrderSide::Buy,
        quantity: 10.0,
        expected_price: 42000.0,
    };

    let market = MarketData {
        symbol: "BTC-USD".to_string(),
        bid: 41990.0,
        ask: 42010.0,
        volume: 1000.0,
        avg_daily_volume: 50000.0,
    };

    // Test different slippage models
    let models = vec![
        ("Zero", SlippageModel::Zero),
        ("Fixed 5bp", SlippageModel::Fixed { basis_points: 5.0 }),
        ("0.1%", SlippageModel::Percentage { rate: 0.001 }),
        ("Volume Impact", SlippageModel::VolumeImpact {
            base_spread: 0.0005,  // 5 bp spread
            impact_factor: 0.01,  // 1% impact at 100% ADV
        }),
    ];

    println!("Order: Buy {} {} @ ${:.2}",
        order.quantity, order.symbol, order.expected_price);
    println!("\nSlippage Model Comparison:");
    println!("{:-<60}", "");

    for (name, model) in &models {
        let exec_price = model.calculate_execution_price(&order, &market);
        let slippage_pct = model.calculate_slippage(&order, &market) * 100.0;
        let cost = (exec_price - order.expected_price) * order.quantity;

        println!(
            "{:<15} | Exec: ${:.2} | Slippage: {:.3}% | Cost: ${:.2}",
            name, exec_price, slippage_pct, cost
        );
    }
}
```

## Advanced Volume Impact Model

For more realistic backtesting, we need a sophisticated market impact model:

```rust
use std::collections::HashMap;

/// Advanced slippage model with market microstructure considerations
#[derive(Debug, Clone)]
pub struct AdvancedSlippageModel {
    /// Temporary impact coefficient (immediate price impact)
    pub temporary_impact: f64,
    /// Permanent impact coefficient (lasting price impact)
    pub permanent_impact: f64,
    /// Volatility factor
    pub volatility_factor: f64,
    /// Liquidity profiles per symbol
    pub liquidity_profiles: HashMap<String, LiquidityProfile>,
}

#[derive(Debug, Clone)]
pub struct LiquidityProfile {
    pub avg_daily_volume: f64,
    pub avg_spread_bps: f64,
    pub volatility: f64,
    pub depth_at_best: f64, // Average size at best bid/ask
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub order_id: u64,
    pub expected_price: f64,
    pub execution_price: f64,
    pub slippage_bps: f64,
    pub market_impact: f64,
    pub total_cost: f64,
}

impl AdvancedSlippageModel {
    pub fn new() -> Self {
        AdvancedSlippageModel {
            temporary_impact: 0.1,
            permanent_impact: 0.05,
            volatility_factor: 1.0,
            liquidity_profiles: HashMap::new(),
        }
    }

    pub fn with_liquidity_profile(
        mut self,
        symbol: &str,
        profile: LiquidityProfile,
    ) -> Self {
        self.liquidity_profiles.insert(symbol.to_string(), profile);
        self
    }

    /// Calculate execution using the Almgren-Chriss market impact model
    pub fn simulate_execution(
        &self,
        order: &Order,
        market: &MarketData,
        order_id: u64,
    ) -> ExecutionResult {
        let profile = self.liquidity_profiles
            .get(&order.symbol)
            .cloned()
            .unwrap_or(LiquidityProfile {
                avg_daily_volume: market.avg_daily_volume,
                avg_spread_bps: 10.0,
                volatility: 0.02,
                depth_at_best: market.volume * 0.01,
            });

        // Volume participation rate
        let participation = order.quantity / profile.avg_daily_volume;

        // Spread cost (half the spread)
        let spread_cost = profile.avg_spread_bps / 20_000.0; // Convert to decimal

        // Temporary impact: immediate price movement
        let temp_impact = self.temporary_impact
            * profile.volatility
            * (participation).sqrt();

        // Permanent impact: lasting price movement
        let perm_impact = self.permanent_impact
            * profile.volatility
            * participation;

        // Total slippage
        let total_slippage = spread_cost + temp_impact + perm_impact;

        // Calculate execution price based on side
        let execution_price = match order.side {
            OrderSide::Buy => order.expected_price * (1.0 + total_slippage),
            OrderSide::Sell => order.expected_price * (1.0 - total_slippage),
        };

        let slippage_bps = total_slippage * 10_000.0;
        let total_cost = (execution_price - order.expected_price).abs()
            * order.quantity;

        ExecutionResult {
            order_id,
            expected_price: order.expected_price,
            execution_price,
            slippage_bps,
            market_impact: temp_impact + perm_impact,
            total_cost,
        }
    }
}

impl Default for AdvancedSlippageModel {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    // Create model with BTC liquidity profile
    let model = AdvancedSlippageModel::new()
        .with_liquidity_profile("BTC-USD", LiquidityProfile {
            avg_daily_volume: 50_000.0, // 50k BTC daily
            avg_spread_bps: 5.0,        // 5 bp spread
            volatility: 0.03,           // 3% daily volatility
            depth_at_best: 10.0,        // 10 BTC at best price
        });

    let market = MarketData {
        symbol: "BTC-USD".to_string(),
        bid: 41990.0,
        ask: 42010.0,
        volume: 1000.0,
        avg_daily_volume: 50_000.0,
    };

    // Test with different order sizes
    let order_sizes = vec![1.0, 10.0, 100.0, 500.0, 1000.0];

    println!("Advanced Slippage Model - Order Size Impact");
    println!("{:-<70}", "");
    println!(
        "{:<10} | {:<12} | {:<12} | {:<10} | {:<12}",
        "Size", "Exec Price", "Slippage", "Impact", "Cost ($)"
    );
    println!("{:-<70}", "");

    for (i, &size) in order_sizes.iter().enumerate() {
        let order = Order {
            symbol: "BTC-USD".to_string(),
            side: OrderSide::Buy,
            quantity: size,
            expected_price: 42000.0,
        };

        let result = model.simulate_execution(&order, &market, i as u64);

        println!(
            "{:<10.1} | ${:<11.2} | {:<10.2} bp | {:<9.4}% | ${:<11.2}",
            size,
            result.execution_price,
            result.slippage_bps,
            result.market_impact * 100.0,
            result.total_cost
        );
    }
}
```

## Slippage in a Backtesting Engine

Here's how to integrate slippage into a trading simulation:

```rust
use std::collections::VecDeque;

/// Trade execution with slippage tracking
#[derive(Debug, Clone)]
pub struct Trade {
    pub id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub expected_price: f64,
    pub execution_price: f64,
    pub slippage_cost: f64,
    pub timestamp: u64,
}

/// Backtesting engine with slippage model
pub struct BacktestEngine {
    pub slippage_model: SlippageModel,
    pub trades: Vec<Trade>,
    pub cash: f64,
    pub positions: HashMap<String, f64>,
    pub total_slippage_cost: f64,
    trade_counter: u64,
}

impl BacktestEngine {
    pub fn new(initial_cash: f64, slippage_model: SlippageModel) -> Self {
        BacktestEngine {
            slippage_model,
            trades: Vec::new(),
            cash: initial_cash,
            positions: HashMap::new(),
            total_slippage_cost: 0.0,
            trade_counter: 0,
        }
    }

    /// Execute an order with slippage
    pub fn execute_order(
        &mut self,
        order: Order,
        market: &MarketData,
        timestamp: u64,
    ) -> Result<Trade, String> {
        // Calculate execution price with slippage
        let execution_price = self.slippage_model
            .calculate_execution_price(&order, market);

        let trade_value = execution_price * order.quantity;
        let slippage_cost = (execution_price - order.expected_price).abs()
            * order.quantity;

        // Check if we have enough capital/position
        match order.side {
            OrderSide::Buy => {
                if self.cash < trade_value {
                    return Err(format!(
                        "Insufficient cash: need ${:.2}, have ${:.2}",
                        trade_value, self.cash
                    ));
                }
                self.cash -= trade_value;
                *self.positions.entry(order.symbol.clone()).or_insert(0.0)
                    += order.quantity;
            }
            OrderSide::Sell => {
                let position = self.positions.get(&order.symbol).unwrap_or(&0.0);
                if *position < order.quantity {
                    return Err(format!(
                        "Insufficient position: need {}, have {}",
                        order.quantity, position
                    ));
                }
                self.cash += trade_value;
                *self.positions.entry(order.symbol.clone()).or_insert(0.0)
                    -= order.quantity;
            }
        }

        self.trade_counter += 1;
        self.total_slippage_cost += slippage_cost;

        let trade = Trade {
            id: self.trade_counter,
            symbol: order.symbol,
            side: order.side,
            quantity: order.quantity,
            expected_price: order.expected_price,
            execution_price,
            slippage_cost,
            timestamp,
        };

        self.trades.push(trade.clone());
        Ok(trade)
    }

    /// Get slippage statistics
    pub fn get_slippage_stats(&self) -> SlippageStats {
        if self.trades.is_empty() {
            return SlippageStats::default();
        }

        let slippages: Vec<f64> = self.trades.iter()
            .map(|t| {
                let slippage_pct = (t.execution_price - t.expected_price)
                    / t.expected_price * 100.0;
                match t.side {
                    OrderSide::Buy => slippage_pct,
                    OrderSide::Sell => -slippage_pct,
                }
            })
            .collect();

        let avg_slippage = slippages.iter().sum::<f64>() / slippages.len() as f64;
        let max_slippage = slippages.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_slippage = slippages.iter().cloned().fold(f64::INFINITY, f64::min);

        SlippageStats {
            total_trades: self.trades.len(),
            total_slippage_cost: self.total_slippage_cost,
            avg_slippage_pct: avg_slippage,
            max_slippage_pct: max_slippage,
            min_slippage_pct: min_slippage,
        }
    }
}

#[derive(Debug, Default)]
pub struct SlippageStats {
    pub total_trades: usize,
    pub total_slippage_cost: f64,
    pub avg_slippage_pct: f64,
    pub max_slippage_pct: f64,
    pub min_slippage_pct: f64,
}

fn main() {
    // Compare backtest results with different slippage models
    let initial_cash = 100_000.0;

    let models = vec![
        ("No Slippage", SlippageModel::Zero),
        ("Fixed 10bp", SlippageModel::Fixed { basis_points: 10.0 }),
        ("Volume Impact", SlippageModel::VolumeImpact {
            base_spread: 0.001,
            impact_factor: 0.02,
        }),
    ];

    let market = MarketData {
        symbol: "BTC-USD".to_string(),
        bid: 41990.0,
        ask: 42010.0,
        volume: 1000.0,
        avg_daily_volume: 50_000.0,
    };

    println!("Backtest Slippage Comparison");
    println!("Simulating 100 round-trip trades (buy + sell)\n");

    for (name, model) in models {
        let mut engine = BacktestEngine::new(initial_cash, model);

        // Simulate 100 round-trip trades
        for i in 0..100 {
            let timestamp = i as u64;
            let price = 42000.0 + (i as f64 * 10.0).sin() * 500.0; // Price variation

            // Buy
            let buy_order = Order {
                symbol: "BTC-USD".to_string(),
                side: OrderSide::Buy,
                quantity: 0.5,
                expected_price: price,
            };
            let _ = engine.execute_order(buy_order, &market, timestamp);

            // Sell at slightly higher price
            let sell_order = Order {
                symbol: "BTC-USD".to_string(),
                side: OrderSide::Sell,
                quantity: 0.5,
                expected_price: price * 1.002, // 0.2% profit target
            };
            let _ = engine.execute_order(sell_order, &market, timestamp);
        }

        let stats = engine.get_slippage_stats();
        println!("Model: {}", name);
        println!("  Final Cash: ${:.2}", engine.cash);
        println!("  Total Slippage Cost: ${:.2}", stats.total_slippage_cost);
        println!("  Avg Slippage: {:.4}%", stats.avg_slippage_pct);
        println!("  Net P&L: ${:.2}", engine.cash - initial_cash);
        println!();
    }
}
```

## Realistic Slippage with Order Book Simulation

For the most accurate slippage modeling, we can simulate order book consumption:

```rust
use std::collections::BTreeMap;

/// Represents a price level in the order book
#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
}

/// Simulated order book for realistic slippage
#[derive(Debug, Clone)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: BTreeMap<i64, PriceLevel>, // Price in cents as key (descending)
    pub asks: BTreeMap<i64, PriceLevel>, // Price in cents as key (ascending)
}

impl OrderBook {
    pub fn new(symbol: &str) -> Self {
        OrderBook {
            symbol: symbol.to_string(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Create a sample order book with liquidity
    pub fn with_sample_liquidity(mut self, mid_price: f64) -> Self {
        // Generate bid levels (descending from mid)
        for i in 1..=10 {
            let price = mid_price - (i as f64 * 0.5);
            let quantity = 5.0 * i as f64; // More liquidity further from mid
            self.bids.insert(
                (-price * 100.0) as i64, // Negative for descending order
                PriceLevel { price, quantity },
            );
        }

        // Generate ask levels (ascending from mid)
        for i in 1..=10 {
            let price = mid_price + (i as f64 * 0.5);
            let quantity = 5.0 * i as f64;
            self.asks.insert(
                (price * 100.0) as i64,
                PriceLevel { price, quantity },
            );
        }

        self
    }

    /// Simulate market order execution consuming order book levels
    pub fn execute_market_order(
        &self,
        side: OrderSide,
        quantity: f64,
    ) -> OrderBookExecution {
        let mut remaining = quantity;
        let mut fills: Vec<(f64, f64)> = Vec::new();
        let mut total_cost = 0.0;

        let levels: Vec<&PriceLevel> = match side {
            OrderSide::Buy => self.asks.values().collect(),
            OrderSide::Sell => self.bids.values().collect(),
        };

        for level in levels {
            if remaining <= 0.0 {
                break;
            }

            let fill_qty = remaining.min(level.quantity);
            fills.push((level.price, fill_qty));
            total_cost += level.price * fill_qty;
            remaining -= fill_qty;
        }

        let filled_quantity = quantity - remaining;
        let avg_price = if filled_quantity > 0.0 {
            total_cost / filled_quantity
        } else {
            0.0
        };

        let best_price = match side {
            OrderSide::Buy => self.asks.values().next().map(|l| l.price).unwrap_or(0.0),
            OrderSide::Sell => self.bids.values().next().map(|l| l.price).unwrap_or(0.0),
        };

        let slippage = if best_price > 0.0 {
            match side {
                OrderSide::Buy => (avg_price - best_price) / best_price,
                OrderSide::Sell => (best_price - avg_price) / best_price,
            }
        } else {
            0.0
        };

        OrderBookExecution {
            requested_quantity: quantity,
            filled_quantity,
            unfilled_quantity: remaining,
            avg_execution_price: avg_price,
            best_available_price: best_price,
            slippage_pct: slippage * 100.0,
            fills,
        }
    }
}

#[derive(Debug)]
pub struct OrderBookExecution {
    pub requested_quantity: f64,
    pub filled_quantity: f64,
    pub unfilled_quantity: f64,
    pub avg_execution_price: f64,
    pub best_available_price: f64,
    pub slippage_pct: f64,
    pub fills: Vec<(f64, f64)>, // (price, quantity) pairs
}

fn main() {
    let order_book = OrderBook::new("BTC-USD")
        .with_sample_liquidity(42000.0);

    println!("Order Book Slippage Simulation");
    println!("Mid price: $42,000.00\n");

    // Show order book
    println!("Ask Levels (selling to you):");
    for level in order_book.asks.values().take(5) {
        println!("  ${:.2} x {:.1}", level.price, level.quantity);
    }
    println!("\nBid Levels (buying from you):");
    for level in order_book.bids.values().take(5) {
        println!("  ${:.2} x {:.1}", level.price, level.quantity);
    }

    println!("\n{:-<60}", "");
    println!("Market Order Executions:\n");

    // Test different order sizes
    let order_sizes = vec![5.0, 20.0, 50.0, 100.0];

    for size in order_sizes {
        let execution = order_book.execute_market_order(OrderSide::Buy, size);

        println!("Buy {} BTC:", size);
        println!("  Best Price: ${:.2}", execution.best_available_price);
        println!("  Avg Exec Price: ${:.2}", execution.avg_execution_price);
        println!("  Slippage: {:.3}%", execution.slippage_pct);
        println!("  Filled: {:.1} / Unfilled: {:.1}",
            execution.filled_quantity,
            execution.unfilled_quantity
        );
        println!("  Fills: {:?}", execution.fills.iter()
            .map(|(p, q)| format!("{:.1}@${:.2}", q, p))
            .collect::<Vec<_>>()
        );
        println!();
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| Slippage | Difference between expected and actual execution price |
| Fixed Model | Constant slippage per trade (simple but unrealistic) |
| Volume Impact | Slippage increases with order size |
| Market Impact | Temporary + permanent price effects from trading |
| Order Book Model | Most realistic — simulates consuming liquidity |
| Spread Cost | Half the bid-ask spread adds to execution cost |

## Practical Exercises

1. **Model Comparison**: Using the basic slippage model, calculate the execution price for a $1 million BTC order at $42,000 with each slippage type. Which model gives the most conservative (worst-case) estimate?

2. **Volatility Adjustment**: Modify the `VolumeImpact` model to include a volatility multiplier — during high volatility periods (volatility > 5%), slippage should be 2x higher.

3. **Asymmetric Slippage**: Implement a slippage model where selling has 20% more slippage than buying (reflecting typical market conditions during sell-offs).

4. **Time-of-Day Factor**: Create a slippage model that increases slippage by 50% during market open (first 30 minutes) and market close (last 30 minutes) when liquidity is typically lower.

## Homework

1. **Historical Slippage Analysis**: Create a struct `SlippageAnalyzer` that:
   - Tracks expected vs actual execution prices over time
   - Calculates running statistics (mean, std dev, max slippage)
   - Identifies patterns (higher slippage for larger orders, specific times)
   - Outputs a summary report

2. **Adaptive Slippage Model**: Implement a slippage model that:
   - Learns from historical execution data
   - Adjusts parameters based on recent market conditions
   - Uses a moving average of realized slippage to calibrate predictions
   - Includes a confidence interval for slippage estimates

3. **TWAP Slippage Optimizer**: Create a Time-Weighted Average Price (TWAP) execution simulator that:
   - Splits large orders into smaller chunks
   - Executes over a time period to minimize market impact
   - Compares slippage between single execution and TWAP
   - Finds the optimal number of slices for different order sizes

4. **Full Backtest Integration**: Build a complete backtesting framework that:
   - Uses the order book slippage model
   - Simulates realistic order book regeneration between trades
   - Tracks slippage as a separate P&L component
   - Generates a report showing "ideal P&L" vs "realistic P&L with slippage"

## Navigation

[← Previous day](../279-position-sizing-model/en.md) | [Next day →](../281-commission-model/en.md)
