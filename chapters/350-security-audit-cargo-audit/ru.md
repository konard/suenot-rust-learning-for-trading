# День 350: Security Audit: cargo audit

## Аналогия из трейдинга

Представь, что ты управляешь хедж-фондом. Перед тем как запустить новую торговую стратегию в продакшн, ты проводишь аудит:

**Традиционный финансовый аудит:**
- Проверяешь, нет ли у контрагентов проблем (банкротства, санкции)
- Анализируешь надёжность используемых бирж
- Проверяешь, нет ли уязвимостей в протоколах обмена данными

**Security audit в Rust:**
- Проверяешь все зависимости на известные уязвимости
- Анализируешь, не отозваны ли какие-либо crates
- Получаешь отчёт о рисках до деплоя в продакшн

| Финансовый аудит | cargo audit |
|------------------|-------------|
| Проверка контрагентов | Проверка зависимостей |
| Due diligence | CVE анализ |
| Рейтинг надёжности | Severity level |
| Аудиторский отчёт | Security report |
| Регуляторные требования | RustSec Advisory Database |

## Что такое cargo audit?

`cargo audit` — это инструмент безопасности, который проверяет файл `Cargo.lock` на наличие зависимостей с известными уязвимостями. Он использует [RustSec Advisory Database](https://rustsec.org/) — базу данных уязвимостей, поддерживаемую Rust Secure Code Working Group.

### Установка

```bash
# Базовая установка
cargo install cargo-audit --locked

# С поддержкой автоматического исправления (экспериментально)
cargo install cargo-audit --locked --features=fix
```

### Базовое использование

```bash
# Запуск аудита в корне проекта
cargo audit

# Подробный вывод
cargo audit -d

# Вывод в JSON для CI
cargo audit --json
```

## Практический пример: Аудит торгового бота

Рассмотрим типичный `Cargo.toml` торгового бота:

```toml
[package]
name = "trading-bot"
version = "0.1.0"
edition = "2021"

[dependencies]
# HTTP клиент для API бирж
reqwest = { version = "0.11", features = ["json"] }

# Криптография для подписи запросов
ring = "0.17"
hmac = "0.12"
sha2 = "0.10"

# WebSocket для стриминга данных
tokio-tungstenite = "0.21"
tokio = { version = "1", features = ["full"] }

# Сериализация
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# База данных для истории сделок
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }

# Мониторинг
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Запуск аудита

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

## Интеграция в торговую систему

### Автоматическая проверка перед деплоем

```rust
use std::process::Command;
use std::fs;

/// Результат аудита безопасности
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

/// Проверка безопасности перед деплоем торгового бота
fn pre_deploy_security_check() -> Result<AuditResult, String> {
    println!("=== Предварительная проверка безопасности ===\n");

    // Запускаем cargo audit с JSON выводом
    let output = Command::new("cargo")
        .args(["audit", "--json"])
        .output()
        .map_err(|e| format!("Не удалось запустить cargo audit: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Парсим результат (упрощённо)
    let result = parse_audit_output(&stdout);

    if result.is_safe {
        println!("✅ Аудит пройден: уязвимости не обнаружены");
    } else {
        println!("❌ Обнаружены проблемы безопасности:");
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
    // Упрощённый парсинг для демонстрации
    AuditResult {
        vulnerabilities: vec![],
        warnings: vec![],
        is_safe: !json.contains("vulnerability"),
    }
}

fn main() {
    match pre_deploy_security_check() {
        Ok(result) if result.is_safe => {
            println!("\n🚀 Можно деплоить торгового бота!");
        }
        Ok(result) => {
            println!("\n⚠️ Необходимо исправить уязвимости перед деплоем!");
            println!("Найдено уязвимостей: {}", result.vulnerabilities.len());
        }
        Err(e) => {
            println!("\n❌ Ошибка аудита: {}", e);
        }
    }
}
```

### Политика безопасности для торговой платформы

```rust
use std::collections::HashMap;

/// Политика безопасности для зависимостей
#[derive(Debug)]
struct SecurityPolicy {
    /// Максимально допустимый уровень уязвимости
    max_allowed_severity: Severity,
    /// Разрешённые исключения (CVE ID -> причина)
    allowed_exceptions: HashMap<String, String>,
    /// Блокировать yanked crates
    block_yanked: bool,
    /// Блокировать unmaintained crates
    block_unmaintained: bool,
    /// Автоматическое исправление разрешено
    auto_fix_allowed: bool,
}

impl SecurityPolicy {
    /// Строгая политика для продакшн торговых систем
    fn production_trading() -> Self {
        SecurityPolicy {
            max_allowed_severity: Severity::Low,
            allowed_exceptions: HashMap::new(),
            block_yanked: true,
            block_unmaintained: true,
            auto_fix_allowed: false, // Требуется ручная проверка
        }
    }

    /// Политика для development
    fn development() -> Self {
        SecurityPolicy {
            max_allowed_severity: Severity::High,
            allowed_exceptions: HashMap::new(),
            block_yanked: true,
            block_unmaintained: false,
            auto_fix_allowed: true,
        }
    }

    /// Проверка соответствия политике
    fn check_compliance(&self, result: &AuditResult) -> ComplianceResult {
        let mut violations = Vec::new();

        for vuln in &result.vulnerabilities {
            // Проверяем, есть ли исключение
            if self.allowed_exceptions.contains_key(&vuln.id) {
                continue;
            }

            // Проверяем severity
            if self.severity_exceeds(&vuln.severity) {
                violations.push(format!(
                    "Уязвимость {} превышает допустимый уровень: {:?}",
                    vuln.id, vuln.severity
                ));
            }
        }

        for warning in &result.warnings {
            match warning.kind {
                WarningKind::Yanked if self.block_yanked => {
                    violations.push(format!(
                        "Отозванный crate: {}", warning.crate_name
                    ));
                }
                WarningKind::Unmaintained if self.block_unmaintained => {
                    violations.push(format!(
                        "Неподдерживаемый crate: {}", warning.crate_name
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
    println!("=== Проверка соответствия политике безопасности ===\n");

    // Симуляция результата аудита
    let audit_result = AuditResult {
        vulnerabilities: vec![
            Vulnerability {
                crate_name: "old-crypto".to_string(),
                version: "1.0.0".to_string(),
                id: "RUSTSEC-2024-0001".to_string(),
                severity: Severity::High,
                title: "Слабое шифрование".to_string(),
            }
        ],
        warnings: vec![
            Warning {
                crate_name: "unmaintained-lib".to_string(),
                kind: WarningKind::Unmaintained,
                message: "Не обновлялось более года".to_string(),
            }
        ],
        is_safe: false,
    };

    // Проверяем соответствие продакшн политике
    let policy = SecurityPolicy::production_trading();
    let compliance = policy.check_compliance(&audit_result);

    println!("Политика: Production Trading");
    println!("Соответствует: {}", compliance.compliant);

    if !compliance.compliant {
        println!("\nНарушения:");
        for violation in &compliance.violations {
            println!("  ❌ {}", violation);
        }
    }
}
```

## CI/CD интеграция

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
    # Ежедневная проверка в 6:00 UTC
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

### Пример скрипта для локальной проверки

```rust
use std::process::{Command, ExitCode};

/// Скрипт предварительной проверки для торговой системы
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
        print!("Проверка: {} ... ", name);

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
                    println!("Ошибка: {}", stderr);
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
        println!("✅ Все проверки пройдены! Готово к деплою.");
        ExitCode::SUCCESS
    } else {
        println!("❌ Некоторые проверки не пройдены!");
        ExitCode::FAILURE
    }
}
```

## Аудит скомпилированных бинарников

### cargo-auditable

Для аудита уже скомпилированных бинарников используется `cargo-auditable`:

```bash
# Установка
cargo install cargo-auditable

# Сборка с метаданными для аудита
cargo auditable build --release

# Аудит бинарника
cargo audit bin ./target/release/trading-bot
```

### Пример использования в торговой инфраструктуре

```rust
use std::path::Path;
use std::process::Command;

/// Аудит развёрнутого торгового бота
fn audit_deployed_binary(binary_path: &Path) -> Result<bool, String> {
    println!("Аудит бинарника: {}\n", binary_path.display());

    let output = Command::new("cargo")
        .args(["audit", "bin", binary_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Не удалось запустить аудит: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("{}", stdout);

    if output.status.success() {
        println!("✅ Бинарник безопасен");
        Ok(true)
    } else {
        println!("❌ Обнаружены уязвимости");
        println!("{}", stderr);
        Ok(false)
    }
}

fn main() {
    let binary_path = Path::new("./target/release/trading-bot");

    match audit_deployed_binary(binary_path) {
        Ok(true) => println!("\n🚀 Бот может продолжать работу"),
        Ok(false) => println!("\n⚠️ Требуется обновление бота"),
        Err(e) => println!("\n❌ Ошибка: {}", e),
    }
}
```

## Мониторинг безопасности в реальном времени

```rust
use std::time::{Duration, Instant};
use std::process::Command;
use std::thread;

/// Мониторинг безопасности для работающей торговой системы
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
        println!("[{}] Запуск аудита безопасности...",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));

        let output = Command::new("cargo")
            .args(["audit", "--json"])
            .output();

        self.last_check = Some(Instant::now());

        match output {
            Ok(result) if result.status.success() => {
                println!("  ✅ Уязвимости не обнаружены");
                AuditStatus::Clean
            }
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                println!("  ⚠️ Обнаружены проблемы безопасности");

                if self.alert_on_vulnerability {
                    self.send_alert(&stdout);
                }

                AuditStatus::Vulnerabilities(stdout.to_string())
            }
            Err(e) => {
                println!("  ❌ Ошибка аудита: {}", e);
                AuditStatus::Error(e.to_string())
            }
        }
    }

    fn send_alert(&self, details: &str) {
        // В реальной системе здесь была бы отправка в Slack, Telegram, Email
        println!("  🚨 ALERT: Обнаружена уязвимость в торговой системе!");
        println!("  Детали: {}", &details[..details.len().min(200)]);
    }
}

#[derive(Debug)]
enum AuditStatus {
    Clean,
    Vulnerabilities(String),
    Error(String),
}

// Заглушка для chrono
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
    println!("=== Мониторинг безопасности торговой системы ===\n");

    let mut monitor = SecurityMonitor::new(24); // Проверка каждые 24 часа

    // Симуляция работы
    for i in 1..=3 {
        println!("Итерация {}", i);

        if monitor.should_check() {
            let status = monitor.run_audit();

            match status {
                AuditStatus::Clean => {
                    println!("  Система безопасна\n");
                }
                AuditStatus::Vulnerabilities(_) => {
                    println!("  ВНИМАНИЕ: Требуется обновление!\n");
                }
                AuditStatus::Error(e) => {
                    println!("  Ошибка проверки: {}\n", e);
                }
            }
        }

        // В реальной системе здесь был бы sleep
        // thread::sleep(Duration::from_secs(3600));
    }
}
```

## Что мы узнали

| Концепция | Описание |
|-----------|----------|
| **cargo audit** | Инструмент проверки зависимостей на уязвимости |
| **RustSec Database** | База данных уязвимостей Rust crates |
| **Severity levels** | Уровни критичности: Critical, High, Medium, Low |
| **Yanked crates** | Отозванные версии пакетов |
| **cargo-auditable** | Встраивание метаданных в бинарники для аудита |
| **Security Policy** | Правила допустимых уязвимостей для проекта |
| **CI/CD интеграция** | Автоматическая проверка безопасности в пайплайне |

## Практические задания

1. **Базовый аудит**: Создай торговый проект со следующими зависимостями и запусти `cargo audit`:
   - Добавь `reqwest`, `serde`, `tokio`
   - Проанализируй результат
   - Исправь найденные предупреждения

2. **CI Pipeline**: Настрой GitHub Actions workflow:
   - Запускай `cargo audit` при каждом push
   - Блокируй merge при наличии уязвимостей
   - Добавь ежедневную проверку по расписанию

3. **Политика безопасности**: Напиши модуль политики:
   - Определи допустимые уровни severity для dev/staging/prod
   - Реализуй систему исключений для известных проблем
   - Добавь логирование решений

4. **Мониторинг**: Создай систему мониторинга:
   - Периодическая проверка зависимостей
   - Отправка алертов в Telegram/Slack
   - Хранение истории проверок

## Домашнее задание

1. **Полный аудит торговой платформы**: Создай проект с типичными зависимостями торгового бота:
   - HTTP клиент для REST API (reqwest)
   - WebSocket для стриминга (tokio-tungstenite)
   - Криптография для подписи (ring, hmac)
   - База данных (sqlx)
   - Запусти полный аудит и задокументируй все findings
   - Создай план исправления найденных проблем

2. **Автоматизация безопасности**: Разработай систему:
   - Pre-commit hook для локальной проверки
   - GitHub Action для PR проверки
   - Scheduled workflow для ежедневного аудита
   - Отправка отчётов на email

3. **Dashboard безопасности**: Создай dashboard:
   - Визуализация истории аудитов
   - Тренды уязвимостей по времени
   - Статистика по severity
   - Рекомендации по обновлению

4. **Аудит бинарников в продакшне**: Реализуй систему:
   - Сборка с cargo-auditable
   - Аудит развёрнутых бинарников
   - Сравнение версий dev/staging/prod
   - Автоматическое создание tickets для обновлений

5. **Интеграция с SIEM**: Подключи cargo audit к системе мониторинга:
   - Формирование событий безопасности
   - Корреляция с другими источниками
   - Настройка алертов по severity
   - Автоматическое реагирование на Critical

## Навигация

[← Предыдущий день](../349-rustfmt/ru.md) | [Следующий день →](../351-rustdoc/ru.md)
