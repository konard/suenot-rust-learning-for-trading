# Day 350: Security Audit: cargo audit

## Trading Analogy

Imagine you're managing a hedge fund. Before launching a new trading strategy into production, you conduct an audit:

**Traditional Financial Audit:**
- Check if counterparties have issues (bankruptcy, sanctions)
- Analyze the reliability of exchanges being used
- Verify there are no vulnerabilities in data exchange protocols

**Security Audit in Rust:**
- Check all dependencies for known vulnerabilities
- Analyze if any crates have been yanked
- Get a risk report before deploying to production

| Financial Audit | cargo audit |
|-----------------|-------------|
| Counterparty check | Dependency check |
| Due diligence | CVE analysis |
| Reliability rating | Severity level |
| Audit report | Security report |
| Regulatory requirements | RustSec Advisory Database |

## What is cargo audit?

`cargo audit` is a security tool that checks your `Cargo.lock` file for dependencies with known security vulnerabilities. It uses the [RustSec Advisory Database](https://rustsec.org/) — a vulnerability database maintained by the Rust Secure Code Working Group.

### Installation

```bash
# Basic installation
cargo install cargo-audit --locked

# With auto-fix support (experimental)
cargo install cargo-audit --locked --features=fix
```

### Basic Usage

```bash
# Run audit in project root
cargo audit

# Verbose output
cargo audit -d

# JSON output for CI
cargo audit --json
```

## Practical Example: Auditing a Trading Bot

Let's look at a typical trading bot `Cargo.toml`:

```toml
[package]
name = "trading-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
# HTTP client for exchange APIs
reqwest = { version = "0.11", features = ["json"] }

# Cryptography for request signing
ring = "0.17"
hmac = "0.12"
sha2 = "0.10"

# WebSocket for data streaming
tokio-tungstenite = "0.21"
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database for trade history
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }

# Monitoring
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Running the Audit

```bash
$ cargo audit
    Fetching advisory database from `https://github.com/RustSec/advisory-db`
      Loaded 650 security advisories (from /home/trader/.cargo/advisory-db)
    Scanning Cargo.lock for vulnerabilities (150 crate dependencies)

Crate:     chrono
Version:   0.4.23
Warning:   unmaintained
Title:     chrono is unmaintained
Date:      2024-01-01
ID:        RUSTSEC-2024-0001
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0001
Dependency tree:
chrono 0.4.23
└── trading-bot 0.1.0

warning: 1 allowed warning found
```

## Integration into Trading System

### Automatic Pre-Deploy Check

```rust
use std::process::Command;
use std::fs;

/// Security audit result
#[derive(Debug)]
struct AuditResult {
    vulnerabilities: Vec<Vulnerability>,
    warnings: Vec<Warning>,
    is_safe: bool,
}

#[derive(Debug)]
struct Vulnerability {
    crate_name: String,
    version: String,
    id: String,
    severity: Severity,
    title: String,
}

#[derive(Debug)]
struct Warning {
    crate_name: String,
    kind: WarningKind,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug)]
enum WarningKind {
    Unmaintained,
    Yanked,
    Unsound,
}

/// Pre-deploy security check for trading bot
fn pre_deploy_security_check() -> Result<AuditResult, String> {
    println!("=== Pre-Deploy Security Check ===\n");

    // Run cargo audit with JSON output
    let output = Command::new("cargo")
        .args(["audit", "--json"])
        .output()
        .map_err(|e| format!("Failed to run cargo audit: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse the result (simplified)
    let result = parse_audit_output(&stdout);

    if result.is_safe {
        println!("✅ Audit passed: no vulnerabilities found");
    } else {
        println!("❌ Security issues detected:");
        for vuln in &result.vulnerabilities {
            println!(
                "  - {} v{}: {} ({})",
                vuln.crate_name, vuln.version, vuln.title, vuln.id
            );
        }
    }

    Ok(result)
}

fn parse_audit_output(json: &str) -> AuditResult {
    // Simplified parsing for demonstration
    AuditResult {
        vulnerabilities: vec![],
        warnings: vec![],
        is_safe: !json.contains("vulnerability"),
    }
}

fn main() {
    match pre_deploy_security_check() {
        Ok(result) if result.is_safe => {
            println!("\n🚀 Trading bot ready for deployment!");
        }
        Ok(result) => {
            println!("\n⚠️ Vulnerabilities must be fixed before deployment!");
            println!("Vulnerabilities found: {}", result.vulnerabilities.len());
        }
        Err(e) => {
            println!("\n❌ Audit error: {}", e);
        }
    }
}
```

### Security Policy for Trading Platform

```rust
use std::collections::HashMap;

/// Security policy for dependencies
#[derive(Debug)]
struct SecurityPolicy {
    /// Maximum allowed vulnerability severity
    max_allowed_severity: Severity,
    /// Allowed exceptions (CVE ID -> reason)
    allowed_exceptions: HashMap<String, String>,
    /// Block yanked crates
    block_yanked: bool,
    /// Block unmaintained crates
    block_unmaintained: bool,
    /// Automatic fixing allowed
    auto_fix_allowed: bool,
}

impl SecurityPolicy {
    /// Strict policy for production trading systems
    fn production_trading() -> Self {
        SecurityPolicy {
            max_allowed_severity: Severity::Low,
            allowed_exceptions: HashMap::new(),
            block_yanked: true,
            block_unmaintained: true,
            auto_fix_allowed: false, // Manual review required
        }
    }

    /// Policy for development
    fn development() -> Self {
        SecurityPolicy {
            max_allowed_severity: Severity::High,
            allowed_exceptions: HashMap::new(),
            block_yanked: true,
            block_unmaintained: false,
            auto_fix_allowed: true,
        }
    }

    /// Check policy compliance
    fn check_compliance(&self, result: &AuditResult) -> ComplianceResult {
        let mut violations = Vec::new();

        for vuln in &result.vulnerabilities {
            // Check if exception exists
            if self.allowed_exceptions.contains_key(&vuln.id) {
                continue;
            }

            // Check severity
            if self.severity_exceeds(&vuln.severity) {
                violations.push(format!(
                    "Vulnerability {} exceeds allowed level: {:?}",
                    vuln.id, vuln.severity
                ));
            }
        }

        for warning in &result.warnings {
            match warning.kind {
                WarningKind::Yanked if self.block_yanked => {
                    violations.push(format!(
                        "Yanked crate: {}", warning.crate_name
                    ));
                }
                WarningKind::Unmaintained if self.block_unmaintained => {
                    violations.push(format!(
                        "Unmaintained crate: {}", warning.crate_name
                    ));
                }
                _ => {}
            }
        }

        ComplianceResult {
            compliant: violations.is_empty(),
            violations,
        }
    }

    fn severity_exceeds(&self, severity: &Severity) -> bool {
        let level = |s: &Severity| match s {
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 3,
            Severity::Critical => 4,
        };
        level(severity) > level(&self.max_allowed_severity)
    }
}

#[derive(Debug)]
struct ComplianceResult {
    compliant: bool,
    violations: Vec<String>,
}

fn main() {
    println!("=== Security Policy Compliance Check ===\n");

    // Simulated audit result
    let audit_result = AuditResult {
        vulnerabilities: vec![
            Vulnerability {
                crate_name: "old-crypto".to_string(),
                version: "1.0.0".to_string(),
                id: "RUSTSEC-2024-0001".to_string(),
                severity: Severity::High,
                title: "Weak encryption".to_string(),
            }
        ],
        warnings: vec![
            Warning {
                crate_name: "unmaintained-lib".to_string(),
                kind: WarningKind::Unmaintained,
                message: "Not updated for over a year".to_string(),
            }
        ],
        is_safe: false,
    };

    // Check production policy compliance
    let policy = SecurityPolicy::production_trading();
    let compliance = policy.check_compliance(&audit_result);

    println!("Policy: Production Trading");
    println!("Compliant: {}", compliance.compliant);

    if !compliance.compliant {
        println!("\nViolations:");
        for violation in &compliance.violations {
            println!("  ❌ {}", violation);
        }
    }
}
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/security.yml
name: Security Audit

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    # Daily check at 6:00 UTC
    - cron: '0 6 * * *'

jobs:
  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install cargo-audit
        run: cargo install cargo-audit --locked

      - name: Run security audit
        run: cargo audit --deny warnings

      - name: Check for yanked dependencies
        run: cargo audit --deny yanked
```

### Local Pre-Check Script Example

```rust
use std::process::{Command, ExitCode};

/// Pre-check script for trading system
fn main() -> ExitCode {
    println!("╔══════════════════════════════════════════╗");
    println!("║  Trading System Security Pre-Check       ║");
    println!("╚══════════════════════════════════════════╝\n");

    let checks = vec![
        ("cargo audit", vec!["audit"]),
        ("cargo clippy", vec!["clippy", "--", "-D", "warnings"]),
        ("cargo test", vec!["test"]),
    ];

    let mut all_passed = true;

    for (name, args) in checks {
        print!("Check: {} ... ", name);

        let status = Command::new("cargo")
            .args(&args)
            .output();

        match status {
            Ok(output) if output.status.success() => {
                println!("✅ PASS");
            }
            Ok(output) => {
                println!("❌ FAIL");
                all_passed = false;

                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    println!("Error: {}", stderr);
                }
            }
            Err(e) => {
                println!("❌ ERROR: {}", e);
                all_passed = false;
            }
        }
    }

    println!("\n═══════════════════════════════════════════");

    if all_passed {
        println!("✅ All checks passed! Ready for deployment.");
        ExitCode::SUCCESS
    } else {
        println!("❌ Some checks failed!");
        ExitCode::FAILURE
    }
}
```

## Auditing Compiled Binaries

### cargo-auditable

For auditing already compiled binaries, use `cargo-auditable`:

```bash
# Installation
cargo install cargo-auditable

# Build with audit metadata
cargo auditable build --release

# Audit the binary
cargo audit bin ./target/release/trading-bot
```

### Usage Example in Trading Infrastructure

```rust
use std::path::Path;
use std::process::Command;

/// Audit deployed trading bot
fn audit_deployed_binary(binary_path: &Path) -> Result<bool, String> {
    println!("Auditing binary: {}\n", binary_path.display());

    let output = Command::new("cargo")
        .args(["audit", "bin", binary_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to run audit: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("{}", stdout);

    if output.status.success() {
        println!("✅ Binary is safe");
        Ok(true)
    } else {
        println!("❌ Vulnerabilities detected");
        println!("{}", stderr);
        Ok(false)
    }
}

fn main() {
    let binary_path = Path::new("./target/release/trading-bot");

    match audit_deployed_binary(binary_path) {
        Ok(true) => println!("\n🚀 Bot can continue running"),
        Ok(false) => println!("\n⚠️ Bot update required"),
        Err(e) => println!("\n❌ Error: {}", e),
    }
}
```

## Real-Time Security Monitoring

```rust
use std::time::{Duration, Instant};
use std::process::Command;
use std::thread;

/// Security monitoring for running trading system
struct SecurityMonitor {
    check_interval: Duration,
    last_check: Option<Instant>,
    alert_on_vulnerability: bool,
}

impl SecurityMonitor {
    fn new(check_interval_hours: u64) -> Self {
        SecurityMonitor {
            check_interval: Duration::from_secs(check_interval_hours * 3600),
            last_check: None,
            alert_on_vulnerability: true,
        }
    }

    fn should_check(&self) -> bool {
        match self.last_check {
            None => true,
            Some(last) => last.elapsed() >= self.check_interval,
        }
    }

    fn run_audit(&mut self) -> AuditStatus {
        println!("[{}] Running security audit...",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));

        let output = Command::new("cargo")
            .args(["audit", "--json"])
            .output();

        self.last_check = Some(Instant::now());

        match output {
            Ok(result) if result.status.success() => {
                println!("  ✅ No vulnerabilities found");
                AuditStatus::Clean
            }
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                println!("  ⚠️ Security issues detected");

                if self.alert_on_vulnerability {
                    self.send_alert(&stdout);
                }

                AuditStatus::Vulnerabilities(stdout.to_string())
            }
            Err(e) => {
                println!("  ❌ Audit error: {}", e);
                AuditStatus::Error(e.to_string())
            }
        }
    }

    fn send_alert(&self, details: &str) {
        // In real system, this would send to Slack, Telegram, Email
        println!("  🚨 ALERT: Vulnerability detected in trading system!");
        println!("  Details: {}", &details[..details.len().min(200)]);
    }
}

#[derive(Debug)]
enum AuditStatus {
    Clean,
    Vulnerabilities(String),
    Error(String),
}

// Stub for chrono
mod chrono {
    pub struct Local;
    impl Local {
        pub fn now() -> DateTime { DateTime }
    }
    pub struct DateTime;
    impl DateTime {
        pub fn format(&self, _: &str) -> &str { "2024-01-01 12:00:00" }
    }
}

fn main() {
    println!("=== Trading System Security Monitoring ===\n");

    let mut monitor = SecurityMonitor::new(24); // Check every 24 hours

    // Simulation
    for i in 1..=3 {
        println!("Iteration {}", i);

        if monitor.should_check() {
            let status = monitor.run_audit();

            match status {
                AuditStatus::Clean => {
                    println!("  System is secure\n");
                }
                AuditStatus::Vulnerabilities(_) => {
                    println!("  WARNING: Update required!\n");
                }
                AuditStatus::Error(e) => {
                    println!("  Check error: {}\n", e);
                }
            }
        }

        // In real system, this would sleep
        // thread::sleep(Duration::from_secs(3600));
    }
}
```

## What We Learned

| Concept | Description |
|---------|-------------|
| **cargo audit** | Tool for checking dependencies for vulnerabilities |
| **RustSec Database** | Rust crates vulnerability database |
| **Severity levels** | Criticality levels: Critical, High, Medium, Low |
| **Yanked crates** | Withdrawn package versions |
| **cargo-auditable** | Embedding metadata in binaries for auditing |
| **Security Policy** | Rules for acceptable vulnerabilities in project |
| **CI/CD integration** | Automatic security checking in pipeline |

## Practical Exercises

1. **Basic Audit**: Create a trading project with the following dependencies and run `cargo audit`:
   - Add `reqwest`, `serde`, `tokio`
   - Analyze the results
   - Fix any warnings found

2. **CI Pipeline**: Set up a GitHub Actions workflow:
   - Run `cargo audit` on every push
   - Block merge if vulnerabilities exist
   - Add daily scheduled checks

3. **Security Policy**: Write a policy module:
   - Define acceptable severity levels for dev/staging/prod
   - Implement exception system for known issues
   - Add decision logging

4. **Monitoring**: Create a monitoring system:
   - Periodic dependency checking
   - Send alerts to Telegram/Slack
   - Store check history

## Homework

1. **Full Trading Platform Audit**: Create a project with typical trading bot dependencies:
   - HTTP client for REST API (reqwest)
   - WebSocket for streaming (tokio-tungstenite)
   - Cryptography for signing (ring, hmac)
   - Database (sqlx)
   - Run full audit and document all findings
   - Create a remediation plan

2. **Security Automation**: Develop a system:
   - Pre-commit hook for local checking
   - GitHub Action for PR verification
   - Scheduled workflow for daily audit
   - Send reports via email

3. **Security Dashboard**: Create a dashboard:
   - Visualize audit history
   - Vulnerability trends over time
   - Statistics by severity
   - Update recommendations

4. **Production Binary Audit**: Implement a system:
   - Build with cargo-auditable
   - Audit deployed binaries
   - Compare dev/staging/prod versions
   - Automatic ticket creation for updates

5. **SIEM Integration**: Connect cargo audit to monitoring system:
   - Generate security events
   - Correlate with other sources
   - Configure alerts by severity
   - Automatic response to Critical

## Navigation

[← Previous day](../349-rustfmt/en.md) | [Next day →](../351-rustdoc/en.md)
