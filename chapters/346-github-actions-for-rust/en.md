# Day 346: GitHub Actions for Rust

## Trading Analogy

Imagine you're managing a trading team. Every time a trader makes changes to a trading strategy, you need to:

1. **Check the code** — ensure the syntax is correct
2. **Run tests** — verify the strategy works as expected
3. **Run backtest** — test on historical data
4. **Deploy** — push to production servers

**Without automation:**
You manually perform each step, spending time and risking missing an error. It's like manually verifying each order before sending it to the exchange.

**With GitHub Actions:**
A robot automatically performs all checks on every code change. It's like a trading bot that automatically validates and executes orders according to predefined rules.

| Aspect | Manual Process | GitHub Actions |
|--------|----------------|----------------|
| **Time** | Minutes per change | Seconds |
| **Errors** | Easy to miss | Impossible to miss |
| **Repeatability** | Depends on person | Always consistent |
| **Scalability** | Limited | Unlimited |

## GitHub Actions Basics

GitHub Actions is a CI/CD (Continuous Integration / Continuous Deployment) platform built into GitHub. It allows you to automatically run tasks on specific events.

### Key Concepts

```yaml
# .github/workflows/ci.yml

name: CI Pipeline               # Workflow name

on:                             # Triggers
  push:                         # On push to repository
    branches: [main, develop]   # Only for these branches
  pull_request:                 # On PR creation/update
    branches: [main]

jobs:                           # Tasks to execute
  build:                        # Job name
    runs-on: ubuntu-latest      # OS to run on

    steps:                      # Job steps
      - uses: actions/checkout@v4      # Checkout repository
      - name: Build                     # Step name
        run: cargo build --release      # Command to execute
```

### Directory Structure

```
trading-bot/
├── .github/
│   └── workflows/
│       ├── ci.yml           # Main CI pipeline
│       ├── release.yml      # Release pipeline
│       └── security.yml     # Security checks
├── src/
│   └── main.rs
├── Cargo.toml
└── README.md
```

## Basic CI for Rust Project

```yaml
# .github/workflows/ci.yml

name: Rust CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  # Format check
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  # Static analysis
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  # Testing
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features

  # Build
  build:
    name: Build
    runs-on: ubuntu-latest
    needs: [fmt, clippy, test]  # Runs after successful checks
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release
      - uses: actions/upload-artifact@v4
        with:
          name: trading-bot
          path: target/release/trading-bot
```

## CI for Trading Bot

Full CI example for an algorithmic trading system:

```yaml
# .github/workflows/trading-ci.yml

name: Trading Bot CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
  schedule:
    # Daily run at 6:00 UTC for integration checks
    - cron: '0 6 * * *'

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Quick checks
  quick-checks:
    name: Quick Checks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: cargo clippy --all-targets -- -D warnings

  # Unit tests
  unit-tests:
    name: Unit Tests
    runs-on: ubuntu-latest
    needs: quick-checks
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run unit tests
        run: cargo test --lib --all-features
        env:
          RUST_LOG: debug

  # Integration tests
  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest
    needs: unit-tests
    services:
      # Redis for caching market data
      redis:
        image: redis:7
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

      # PostgreSQL for storing trades
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: trading
          POSTGRES_PASSWORD: trading_pass
          POSTGRES_DB: trading_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run integration tests
        run: cargo test --test '*' --all-features
        env:
          DATABASE_URL: postgres://trading:trading_pass@localhost:5432/trading_test
          REDIS_URL: redis://localhost:6379

  # Strategy backtesting
  backtest:
    name: Strategy Backtest
    runs-on: ubuntu-latest
    needs: integration-tests
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Download historical data
        run: |
          mkdir -p data
          curl -L -o data/btcusdt_1h.csv \
            "https://example.com/historical/btcusdt_1h.csv" || \
            echo "symbol,timestamp,open,high,low,close,volume" > data/btcusdt_1h.csv

      - name: Run backtest
        run: cargo run --release --bin backtest -- --data data/btcusdt_1h.csv

      - name: Upload backtest results
        uses: actions/upload-artifact@v4
        with:
          name: backtest-results
          path: results/

  # Release build
  build-release:
    name: Build Release
    runs-on: ${{ matrix.os }}
    needs: [unit-tests, integration-tests]
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: trading-bot-linux
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: trading-bot-macos
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: trading-bot-windows.exe

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2

      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: target/${{ matrix.target }}/release/trading-bot*
```

## Matrix Build for Different Rust Versions

```yaml
# .github/workflows/rust-matrix.yml

name: Rust Matrix Build

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-matrix:
    name: Test on ${{ matrix.os }} with Rust ${{ matrix.rust }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false  # Continue even if one job fails
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta, nightly]
        exclude:
          # Exclude nightly on Windows (unstable)
          - os: windows-latest
            rust: nightly

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust ${{ matrix.rust }}
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.os }}-${{ matrix.rust }}

      - name: Build
        run: cargo build --verbose

      - name: Run tests
        run: cargo test --verbose
```

## Secrets and Environment Variables

Trading systems often need API keys and secrets:

```yaml
# .github/workflows/trading-deploy.yml

name: Deploy Trading Bot

on:
  push:
    tags:
      - 'v*'  # Only on version tag creation

jobs:
  deploy:
    name: Deploy
    runs-on: ubuntu-latest
    environment: production  # Uses production environment

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Build release
        run: cargo build --release

      - name: Configure trading credentials
        env:
          # Secrets stored in repository settings
          BINANCE_API_KEY: ${{ secrets.BINANCE_API_KEY }}
          BINANCE_SECRET_KEY: ${{ secrets.BINANCE_SECRET_KEY }}
        run: |
          echo "Configuring API keys..."
          # Create config file (DON'T commit!)
          cat > config/production.toml << EOF
          [exchange.binance]
          api_key = "${BINANCE_API_KEY}"
          secret_key = "${BINANCE_SECRET_KEY}"
          EOF

      - name: Deploy to server
        env:
          DEPLOY_KEY: ${{ secrets.DEPLOY_KEY }}
          DEPLOY_HOST: ${{ vars.DEPLOY_HOST }}
        run: |
          echo "$DEPLOY_KEY" > deploy_key
          chmod 600 deploy_key
          scp -i deploy_key -o StrictHostKeyChecking=no \
            target/release/trading-bot \
            deploy@${DEPLOY_HOST}:/opt/trading/
```

## Caching for Fast Builds

```yaml
# .github/workflows/cached-build.yml

name: Cached Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      # Cache cargo registry and target directory
      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          # Cache key includes Cargo.lock hash
          shared-key: "trading-bot"
          # Only cache after successful build
          save-if: ${{ github.ref == 'refs/heads/main' }}

      # Alternative caching method
      - name: Cache target directory
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: Build
        run: cargo build --release
```

## Automatic Release

```yaml
# .github/workflows/release.yml

name: Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'

permissions:
  contents: write

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}

    steps:
      - uses: actions/checkout@v4

      - name: Generate changelog
        id: changelog
        run: |
          # Generate changelog from git log
          echo "## Changes" > CHANGELOG.md
          git log --pretty=format:"- %s" $(git describe --tags --abbrev=0 HEAD^)..HEAD >> CHANGELOG.md

      - name: Create Release
        id: create_release
        uses: softprops/action-gh-release@v1
        with:
          body_path: CHANGELOG.md
          draft: false
          prerelease: ${{ contains(github.ref, 'alpha') || contains(github.ref, 'beta') }}

  build-and-upload:
    name: Build and Upload
    needs: create-release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: trading-bot-linux-x64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            name: trading-bot-linux-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            name: trading-bot-macos-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            name: trading-bot-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            name: trading-bot-windows-x64.exe

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross-compilation tools
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: |
          sudo apt-get update
          sudo apt-get install -y gcc-aarch64-linux-gnu

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package (Unix)
        if: runner.os != 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          tar -czvf ../../../${{ matrix.name }}.tar.gz trading-bot

      - name: Package (Windows)
        if: runner.os == 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          7z a ../../../${{ matrix.name }}.zip trading-bot.exe

      - name: Upload Release Asset
        uses: softprops/action-gh-release@v1
        with:
          files: |
            *.tar.gz
            *.zip
```

## Security Checks

```yaml
# .github/workflows/security.yml

name: Security Audit

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    # Weekly check on Monday at 9:00 UTC
    - cron: '0 9 * * 1'

jobs:
  # Check dependencies for vulnerabilities
  audit:
    name: Cargo Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  # Check dependency licenses
  license-check:
    name: License Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-deny
        run: cargo install cargo-deny

      - name: Check licenses
        run: cargo deny check licenses

  # Check code for vulnerabilities
  code-scanning:
    name: Code Scanning
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Run security-focused clippy lints
        run: |
          cargo clippy --all-targets -- \
            -D clippy::unwrap_used \
            -D clippy::expect_used \
            -D clippy::panic \
            -D clippy::todo \
            -D clippy::unimplemented
```

## Code Coverage

```yaml
# .github/workflows/coverage.yml

name: Code Coverage

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  coverage:
    name: Coverage
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install grcov
        run: cargo install grcov

      - name: Run tests with coverage
        env:
          CARGO_INCREMENTAL: 0
          RUSTFLAGS: -Cinstrument-coverage
          LLVM_PROFILE_FILE: cargo-test-%p-%m.profraw
        run: cargo test --all-features

      - name: Generate coverage report
        run: |
          grcov . \
            --binary-path ./target/debug/ \
            -s . \
            -t lcov \
            --branch \
            --ignore-not-existing \
            --ignore "/*" \
            -o coverage.lcov

      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: coverage.lcov
          fail_ci_if_error: true
          token: ${{ secrets.CODECOV_TOKEN }}
```

## Rust Code Example for CI Testing

```rust
// src/lib.rs

use std::collections::HashMap;

/// Trading metrics calculator
pub struct TradingMetrics {
    trades: Vec<Trade>,
}

/// Trade record
#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: String,
    pub side: TradeSide,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradeSide {
    Long,
    Short,
}

impl TradingMetrics {
    pub fn new() -> Self {
        TradingMetrics { trades: Vec::new() }
    }

    pub fn add_trade(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    /// Calculate P&L for a trade
    pub fn calculate_pnl(trade: &Trade) -> f64 {
        let price_diff = trade.exit_price - trade.entry_price;
        match trade.side {
            TradeSide::Long => price_diff * trade.quantity,
            TradeSide::Short => -price_diff * trade.quantity,
        }
    }

    /// Total P&L of all trades
    pub fn total_pnl(&self) -> f64 {
        self.trades.iter().map(Self::calculate_pnl).sum()
    }

    /// Win Rate (percentage of profitable trades)
    pub fn win_rate(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }

        let winning_trades = self.trades
            .iter()
            .filter(|t| Self::calculate_pnl(t) > 0.0)
            .count();

        winning_trades as f64 / self.trades.len() as f64 * 100.0
    }

    /// Profit Factor
    pub fn profit_factor(&self) -> f64 {
        let (gross_profit, gross_loss) = self.trades.iter().fold((0.0, 0.0), |acc, trade| {
            let pnl = Self::calculate_pnl(trade);
            if pnl > 0.0 {
                (acc.0 + pnl, acc.1)
            } else {
                (acc.0, acc.1 + pnl.abs())
            }
        });

        if gross_loss == 0.0 {
            return f64::INFINITY;
        }

        gross_profit / gross_loss
    }

    /// Maximum Drawdown
    pub fn max_drawdown(&self) -> f64 {
        let mut equity = 0.0;
        let mut peak = 0.0;
        let mut max_dd = 0.0;

        for trade in &self.trades {
            equity += Self::calculate_pnl(trade);
            if equity > peak {
                peak = equity;
            }
            let drawdown = peak - equity;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        max_dd
    }

    /// Group trades by symbol
    pub fn trades_by_symbol(&self) -> HashMap<String, Vec<&Trade>> {
        let mut result: HashMap<String, Vec<&Trade>> = HashMap::new();
        for trade in &self.trades {
            result.entry(trade.symbol.clone()).or_default().push(trade);
        }
        result
    }
}

impl Default for TradingMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_trade(symbol: &str, side: TradeSide, entry: f64, exit: f64, qty: f64) -> Trade {
        Trade {
            symbol: symbol.to_string(),
            side,
            entry_price: entry,
            exit_price: exit,
            quantity: qty,
            timestamp: 1234567890,
        }
    }

    #[test]
    fn test_pnl_long_profit() {
        let trade = create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 51000.0, 0.1);
        let pnl = TradingMetrics::calculate_pnl(&trade);
        assert!((pnl - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_pnl_long_loss() {
        let trade = create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 49000.0, 0.1);
        let pnl = TradingMetrics::calculate_pnl(&trade);
        assert!((pnl - (-100.0)).abs() < 0.001);
    }

    #[test]
    fn test_pnl_short_profit() {
        let trade = create_test_trade("BTCUSDT", TradeSide::Short, 50000.0, 49000.0, 0.1);
        let pnl = TradingMetrics::calculate_pnl(&trade);
        assert!((pnl - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_win_rate() {
        let mut metrics = TradingMetrics::new();

        // 2 profitable trades
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 51000.0, 0.1));
        metrics.add_trade(create_test_trade("ETHUSDT", TradeSide::Long, 3000.0, 3100.0, 1.0));

        // 1 losing trade
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 49000.0, 0.1));

        let win_rate = metrics.win_rate();
        assert!((win_rate - 66.666).abs() < 0.01);
    }

    #[test]
    fn test_profit_factor() {
        let mut metrics = TradingMetrics::new();

        // Profit: 200
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 52000.0, 0.1));

        // Loss: 100
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 49000.0, 0.1));

        let pf = metrics.profit_factor();
        assert!((pf - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_max_drawdown() {
        let mut metrics = TradingMetrics::new();

        // Rise to 100
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 51000.0, 0.1));
        // Drop by 50 (equity = 50)
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 49500.0, 0.1));
        // Rise by 30 (equity = 80)
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 50300.0, 0.1));

        let max_dd = metrics.max_drawdown();
        assert!((max_dd - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_empty_metrics() {
        let metrics = TradingMetrics::new();
        assert_eq!(metrics.win_rate(), 0.0);
        assert_eq!(metrics.total_pnl(), 0.0);
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **Workflow** | YAML file describing an automated process |
| **Job** | Individual task within a workflow |
| **Step** | Step within a job |
| **Action** | Reusable component (uses) |
| **Matrix** | Parallel execution with different parameters |
| **Cache** | Caching for faster builds |
| **Secrets** | Secure storage for API keys |
| **Artifacts** | Saving build results |

## Practical Exercises

1. **Basic CI Pipeline**: Create a workflow that:
   - Checks code formatting (cargo fmt)
   - Runs Clippy with warnings as errors
   - Executes all tests
   - Builds the project in release mode
   - Uploads binary file as artifact

2. **Multi-platform Build**: Set up building for:
   - Linux (x86_64 and ARM64)
   - macOS (Intel and Apple Silicon)
   - Windows
   - Automatic binary publishing on tag creation

3. **Integration Tests**: Add workflow with:
   - Running Redis and PostgreSQL containers
   - Database migration
   - Running integration tests
   - Code coverage report

4. **Dependency Monitoring**: Set up:
   - Weekly vulnerability checks
   - License compatibility checks
   - Automatic Issue creation when problems are detected

## Homework

1. **Complete CI/CD Pipeline for Trading Bot**:
   - Create a repository with a simple trading bot
   - Set up complete CI pipeline (lint, test, build)
   - Add backtesting as part of CI
   - Implement automatic release on tag creation
   - Configure Telegram/Discord notifications about results

2. **Build Time Optimization**:
   - Measure current build time
   - Configure dependency caching
   - Use sccache for compilation caching
   - Compare times before and after optimization
   - Document achieved improvements

3. **Security-first Pipeline**:
   - Add cargo-audit for vulnerability checks
   - Configure cargo-deny for license checks
   - Add SAST analysis with clippy
   - Implement secret scanning in code
   - Configure security report

4. **Continuous Deployment**:
   - Create staging and production environments
   - Set up automatic deployment to staging on push to main
   - Implement manual approval for production
   - Add rollback on failed deployment
   - Configure smoke tests after deployment

## Navigation

[← Previous day](../345-*/en.md) | [Next day →](../347-*/en.md)
