# День 346: GitHub Actions для Rust

## Аналогия из трейдинга

Представь, что ты управляешь торговой командой. Каждый раз, когда трейдер вносит изменения в торговую стратегию, тебе нужно:

1. **Проверить код** — убедиться, что синтаксис правильный
2. **Запустить тесты** — проверить, что стратегия работает как ожидалось
3. **Провести бэктест** — протестировать на исторических данных
4. **Задеплоить** — отправить на продакшн-серверы

**Без автоматизации:**
Ты вручную выполняешь каждый шаг, тратя время и рискуя пропустить ошибку. Это как если бы ты вручную проверял каждый ордер перед отправкой на биржу.

**С GitHub Actions:**
Робот автоматически выполняет все проверки при каждом изменении кода. Это как торговый бот, который автоматически валидирует и исполняет ордера по заданным правилам.

| Аспект | Ручной процесс | GitHub Actions |
|--------|----------------|----------------|
| **Время** | Минуты на каждое изменение | Секунды |
| **Ошибки** | Легко пропустить | Невозможно пропустить |
| **Повторяемость** | Зависит от человека | Всегда одинаково |
| **Масштабируемость** | Ограничена | Неограничена |

## Основы GitHub Actions

GitHub Actions — это платформа CI/CD (Continuous Integration / Continuous Deployment), встроенная в GitHub. Она позволяет автоматически запускать задачи при определённых событиях.

### Ключевые концепции

```yaml
# .github/workflows/ci.yml

name: CI Pipeline               # Имя workflow

on:                             # Триггеры запуска
  push:                         # При push в репозиторий
    branches: [main, develop]   # Только для этих веток
  pull_request:                 # При создании/обновлении PR
    branches: [main]

jobs:                           # Задачи для выполнения
  build:                        # Имя задачи
    runs-on: ubuntu-latest      # На какой ОС запускать

    steps:                      # Шаги задачи
      - uses: actions/checkout@v4      # Checkout репозитория
      - name: Build                     # Имя шага
        run: cargo build --release      # Команда для выполнения
```

### Структура директорий

```
trading-bot/
├── .github/
│   └── workflows/
│       ├── ci.yml           # Основной CI pipeline
│       ├── release.yml      # Релизный pipeline
│       └── security.yml     # Проверки безопасности
├── src/
│   └── main.rs
├── Cargo.toml
└── README.md
```

## Базовый CI для Rust-проекта

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
  # Проверка форматирования
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  # Статический анализ
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

  # Тестирование
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features

  # Сборка
  build:
    name: Build
    runs-on: ubuntu-latest
    needs: [fmt, clippy, test]  # Запускается после успешных проверок
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

## CI для торгового бота

Пример полноценного CI для алготрейдинговой системы:

```yaml
# .github/workflows/trading-ci.yml

name: Trading Bot CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
  schedule:
    # Ежедневный запуск в 6:00 UTC для проверки интеграций
    - cron: '0 6 * * *'

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Быстрые проверки
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

  # Unit тесты
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

  # Интеграционные тесты
  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest
    needs: unit-tests
    services:
      # Redis для кеширования рыночных данных
      redis:
        image: redis:7
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

      # PostgreSQL для хранения сделок
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

  # Бэктестинг стратегий
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

  # Сборка релиза
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

## Матрица сборки для разных версий Rust

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
      fail-fast: false  # Продолжать даже если один job упал
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta, nightly]
        exclude:
          # Исключаем nightly на Windows (нестабильно)
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

## Секреты и переменные окружения

Для торговых систем часто нужны API ключи и секреты:

```yaml
# .github/workflows/trading-deploy.yml

name: Deploy Trading Bot

on:
  push:
    tags:
      - 'v*'  # Только при создании тега версии

jobs:
  deploy:
    name: Deploy
    runs-on: ubuntu-latest
    environment: production  # Использует окружение production

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Build release
        run: cargo build --release

      - name: Configure trading credentials
        env:
          # Секреты хранятся в настройках репозитория
          BINANCE_API_KEY: ${{ secrets.BINANCE_API_KEY }}
          BINANCE_SECRET_KEY: ${{ secrets.BINANCE_SECRET_KEY }}
        run: |
          echo "Configuring API keys..."
          # Создаём конфигурационный файл (НЕ коммитим!)
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

## Кэширование для быстрой сборки

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

      # Кэширование cargo registry и target директории
      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          # Ключ кэша включает hash Cargo.lock
          shared-key: "trading-bot"
          # Кэшировать только после успешной сборки
          save-if: ${{ github.ref == 'refs/heads/main' }}

      # Альтернативный способ кэширования
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

## Автоматический релиз

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
          # Генерируем changelog из git log
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

## Проверка безопасности

```yaml
# .github/workflows/security.yml

name: Security Audit

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    # Еженедельная проверка в понедельник в 9:00 UTC
    - cron: '0 9 * * 1'

jobs:
  # Проверка зависимостей на уязвимости
  audit:
    name: Cargo Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  # Проверка лицензий зависимостей
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

  # Проверка кода на уязвимости
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

## Пример Rust-кода для тестирования в CI

```rust
// src/lib.rs

use std::collections::HashMap;

/// Калькулятор торговых метрик
pub struct TradingMetrics {
    trades: Vec<Trade>,
}

/// Торговая сделка
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

    /// Расчёт P&L для сделки
    pub fn calculate_pnl(trade: &Trade) -> f64 {
        let price_diff = trade.exit_price - trade.entry_price;
        match trade.side {
            TradeSide::Long => price_diff * trade.quantity,
            TradeSide::Short => -price_diff * trade.quantity,
        }
    }

    /// Общий P&L всех сделок
    pub fn total_pnl(&self) -> f64 {
        self.trades.iter().map(Self::calculate_pnl).sum()
    }

    /// Win Rate (процент прибыльных сделок)
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

    /// Максимальная просадка
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

    /// Группировка сделок по символу
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

        // 2 прибыльные сделки
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 51000.0, 0.1));
        metrics.add_trade(create_test_trade("ETHUSDT", TradeSide::Long, 3000.0, 3100.0, 1.0));

        // 1 убыточная сделка
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 49000.0, 0.1));

        let win_rate = metrics.win_rate();
        assert!((win_rate - 66.666).abs() < 0.01);
    }

    #[test]
    fn test_profit_factor() {
        let mut metrics = TradingMetrics::new();

        // Прибыль: 200
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 52000.0, 0.1));

        // Убыток: 100
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 49000.0, 0.1));

        let pf = metrics.profit_factor();
        assert!((pf - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_max_drawdown() {
        let mut metrics = TradingMetrics::new();

        // Рост до 100
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 51000.0, 0.1));
        // Падение на 50 (equity = 50)
        metrics.add_trade(create_test_trade("BTCUSDT", TradeSide::Long, 50000.0, 49500.0, 0.1));
        // Рост на 30 (equity = 80)
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

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **Workflow** | YAML-файл с описанием автоматизированного процесса |
| **Job** | Отдельная задача внутри workflow |
| **Step** | Шаг внутри job |
| **Action** | Переиспользуемый компонент (uses) |
| **Matrix** | Параллельное выполнение с разными параметрами |
| **Cache** | Кэширование для ускорения сборки |
| **Secrets** | Безопасное хранение API ключей |
| **Artifacts** | Сохранение результатов сборки |

## Практические задания

1. **Базовый CI Pipeline**: Создай workflow, который:
   - Проверяет форматирование кода (cargo fmt)
   - Запускает Clippy с предупреждениями как ошибки
   - Выполняет все тесты
   - Собирает проект в release режиме
   - Загружает бинарный файл как артефакт

2. **Мультиплатформенная сборка**: Настрой сборку для:
   - Linux (x86_64 и ARM64)
   - macOS (Intel и Apple Silicon)
   - Windows
   - Автоматическую публикацию бинарников при создании тега

3. **Интеграционные тесты**: Добавь workflow с:
   - Запуском Redis и PostgreSQL контейнеров
   - Миграцией базы данных
   - Выполнением интеграционных тестов
   - Отчётом о покрытии кода

4. **Мониторинг зависимостей**: Настрой:
   - Еженедельную проверку уязвимостей
   - Проверку совместимости лицензий
   - Автоматическое создание Issue при обнаружении проблем

## Домашнее задание

1. **Полный CI/CD Pipeline для торгового бота**:
   - Создай репозиторий с простым торговым ботом
   - Настрой полный CI pipeline (lint, test, build)
   - Добавь бэктестинг как часть CI
   - Реализуй автоматический релиз при создании тега
   - Настрой уведомления в Telegram/Discord о результатах

2. **Оптимизация времени сборки**:
   - Замерь текущее время сборки
   - Настрой кэширование зависимостей
   - Используй sccache для кэширования компиляции
   - Сравни время до и после оптимизации
   - Задокументируй достигнутые улучшения

3. **Security-first Pipeline**:
   - Добавь cargo-audit для проверки уязвимостей
   - Настрой cargo-deny для проверки лицензий
   - Добавь SAST-анализ с помощью clippy
   - Реализуй проверку секретов в коде
   - Настрой отчёт о безопасности

4. **Continuous Deployment**:
   - Создай staging и production окружения
   - Настрой автоматический деплой на staging при push в main
   - Реализуй ручное одобрение для production
   - Добавь откат при неудачном деплое
   - Настрой smoke tests после деплоя

## Навигация

[← Предыдущий день](../345-*/ru.md) | [Следующий день →](../347-*/ru.md)
