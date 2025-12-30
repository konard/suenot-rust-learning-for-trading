#!/usr/bin/env python3
"""
Генератор GitHub issues для всех глав курса Rust для Алготрейдинга.
Парсит README.md и создаёт issues для каждой ненаписанной главы.

Использование:
    python3 generate_issues.py              # Генерировать JSON
    python3 generate_issues.py --create     # Создать issues (требует gh CLI)
"""

import json
import re
import subprocess
import sys
import time
from pathlib import Path

# Главы, которые уже написаны
WRITTEN_CHAPTERS = set(range(1, 16))  # 1-15

# Определение месяцев и их тем
MONTHS = {
    1: ("basics", "Первые шаги / First Steps"),
    2: ("ownership", "Владение / Ownership"),
    3: ("data-structures", "Структуры данных / Data Structures"),
    4: ("error-handling", "Обработка ошибок / Error Handling"),
    5: ("working-with-data", "Работа с данными / Working with Data"),
    6: ("concurrency", "Многопоточность / Concurrency"),
    7: ("async-networking", "Async и сети / Async and Networking"),
    8: ("databases", "Базы данных / Databases"),
    9: ("trading-algorithms", "Алгоритмы трейдинга / Trading Algorithms"),
    10: ("backtesting", "Бэктестинг / Backtesting"),
    11: ("optimization", "Оптимизация / Optimization"),
    12: ("production", "Продакшн / Production"),
}

def get_month_for_day(day: int) -> int:
    """Определяет месяц по номеру дня."""
    if day <= 31:
        return 1
    elif day <= 59:
        return 2
    elif day <= 90:
        return 3
    elif day <= 120:
        return 4
    elif day <= 151:
        return 5
    elif day <= 181:
        return 6
    elif day <= 212:
        return 7
    elif day <= 243:
        return 8
    elif day <= 274:
        return 9
    elif day <= 304:
        return 10
    elif day <= 334:
        return 11
    else:
        return 12

def parse_readme():
    """Парсит README.md и извлекает информацию о главах."""
    readme_path = Path(__file__).parent.parent / "README.md"

    with open(readme_path, 'r', encoding='utf-8') as f:
        content = f.read()

    chapters = []

    # Паттерн для строки таблицы: | 001 | Тема RU | Topic EN |
    pattern = r'\| (\d{3}) \| (.+?) \| (.+?) \|'

    for match in re.finditer(pattern, content):
        day = int(match.group(1))
        topic_ru = match.group(2).strip()
        topic_en = match.group(3).strip()

        chapters.append({
            'day': day,
            'topic_ru': topic_ru,
            'topic_en': topic_en,
        })

    return chapters

def generate_issue_body(chapter: dict) -> str:
    """Генерирует тело issue для главы."""
    day = chapter['day']
    topic_ru = chapter['topic_ru']
    topic_en = chapter['topic_en']
    month = get_month_for_day(day)
    month_label, month_name = MONTHS[month]

    # Определяем тип главы
    is_project = "Проект" in topic_ru or "Project" in topic_en

    body = f"""## День {day}: {topic_ru}
## Day {day}: {topic_en}

### Месяц / Month
{month_name}

### Цель / Goal
Написать главу с объяснениями на русском и английском языках.

Write a chapter with explanations in Russian and English.

### Требования / Requirements

#### Структура главы / Chapter Structure
- `chapters/{day:03d}-*/ru.md` - Русская версия
- `chapters/{day:03d}-*/en.md` - English version

#### Содержание / Content
1. **Аналогия из трейдинга** - понятное объяснение через торговые примеры
2. **Теория** - основные концепции Rust
3. **Примеры кода** - все примеры связаны с алготрейдингом
4. **Практические задания** - 3-4 упражнения
5. **Домашнее задание** - задачи для самостоятельной работы

#### Trading Analogy Required
All examples must be related to:
- Price analysis
- Order management
- Portfolio tracking
- Risk management
- Trading strategies

### Checklist
- [ ] Создана папка главы / Chapter folder created
- [ ] Написана русская версия / Russian version written
- [ ] Написана английская версия / English version written
- [ ] Добавлены примеры кода / Code examples added
- [ ] Добавлено домашнее задание / Homework added
- [ ] Проверена навигация (ссылки на prev/next) / Navigation links checked
"""

    if is_project:
        body += """
### Это проектная глава / This is a Project Chapter
Глава должна содержать полноценный мини-проект, объединяющий знания месяца.

This chapter should contain a complete mini-project combining the month's knowledge.
"""

    return body

def generate_issues_json(chapters: list) -> list:
    """Генерирует список issues в формате JSON."""
    issues = []

    for chapter in chapters:
        day = chapter['day']

        # Пропускаем уже написанные главы
        if day in WRITTEN_CHAPTERS:
            continue

        month = get_month_for_day(day)
        month_label, _ = MONTHS[month]

        topic_en = chapter['topic_en']
        is_project = "Project" in topic_en

        labels = ["chapter", f"month-{month}", month_label]
        if is_project:
            labels.append("project")

        issue = {
            "title": f"Chapter {day:03d}: {topic_en}",
            "body": generate_issue_body(chapter),
            "labels": labels
        }

        issues.append(issue)

    return issues

def save_json(issues: list, output_path: Path):
    """Сохраняет issues в JSON файл."""
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(issues, f, ensure_ascii=False, indent=2)
    print(f"Сохранено {len(issues)} issues в {output_path}")

def create_issues_with_gh(issues: list, repo: str):
    """Создаёт issues через gh CLI."""
    print(f"Создание {len(issues)} issues в {repo}...")

    for i, issue in enumerate(issues, 1):
        print(f"[{i}/{len(issues)}] {issue['title'][:50]}...")

        labels = ",".join(issue['labels'])

        try:
            subprocess.run([
                "gh", "issue", "create",
                "--repo", repo,
                "--title", issue['title'],
                "--body", issue['body'],
                "--label", labels
            ], check=True, capture_output=True)

            # Пауза для rate limiting
            time.sleep(1)

        except subprocess.CalledProcessError as e:
            print(f"  Ошибка: {e.stderr.decode()}")
        except FileNotFoundError:
            print("gh CLI не найден. Установите: https://cli.github.com/")
            sys.exit(1)

    print("Готово!")

def main():
    create_mode = "--create" in sys.argv
    repo = "suenot/rust-learning-for-trading"

    print("Парсинг README.md...")
    chapters = parse_readme()
    print(f"Найдено {len(chapters)} глав")

    print("Генерация issues...")
    issues = generate_issues_json(chapters)
    print(f"Сгенерировано {len(issues)} issues (пропущено {len(WRITTEN_CHAPTERS)} написанных)")

    # Сохраняем JSON
    output_path = Path(__file__).parent / "all_issues.json"
    save_json(issues, output_path)

    if create_mode:
        create_issues_with_gh(issues, repo)
    else:
        print("\nДля создания issues выполните:")
        print(f"  python3 {sys.argv[0]} --create")
        print("\nИли используйте bash скрипт:")
        print("  ./create_issues.sh")

if __name__ == "__main__":
    main()
