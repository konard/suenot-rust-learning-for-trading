# Инструкция по обновлению PR после реорганизации experiments/

## Контекст

Была проведена реорганизация папки `experiments/`. Теперь эксперименты хранятся в папках глав:
```
chapters/<chapter-name>/experiments/
```

Это устраняет конфликты при мерджах множественных PR.

## Какие PR нужно обновить

Все открытые PR имеют конфликты из-за:
1. Устаревшей версии `chapters/294-overfitting-strategy-optimization/*.md`
2. Старой структуры `experiments/`

Список PR для обновления:
- #507, #506, #505, #503, #502, #501, #500, #499, #496, #495, #494, #490

## Как обновить PR (для каждого)

### Шаг 1: Обновить с main

```bash
# Checkout PR branch
gh pr checkout <PR_NUMBER>

# Merge main
git merge origin/main
```

### Шаг 2: Разрешить конфликты в experiments/

Если PR добавляет файлы в `experiments/chapter-XXX/`, нужно:

1. **Переместить эксперименты в главу:**
```bash
# Найти номер главы из PR
CHAPTER_NUM=$(ls experiments/ | grep chapter- | head -1 | grep -o '[0-9]\+')

# Найти папку главы
CHAPTER_DIR=$(ls chapters/ | grep "^${CHAPTER_NUM}-")

# Создать папку experiments в главе
mkdir -p "chapters/${CHAPTER_DIR}/experiments"

# Переместить файлы
mv experiments/chapter-${CHAPTER_NUM}-* "chapters/${CHAPTER_DIR}/experiments/"
```

2. **Разрешить конфликты:**
```bash
# Принять версию .gitignore и Cargo.toml из main
git checkout --theirs experiments/.gitignore
git checkout --theirs experiments/Cargo.toml

# Если есть experiments/src/main.rs - удалить
git rm experiments/src/main.rs 2>/dev/null || true

# Добавить новые файлы
git add chapters/*/experiments/
git add experiments/

# Завершить merge
git commit
```

### Шаг 3: Push обновленную ветку

```bash
# Push в fork
git push origin HEAD --force-with-lease
```

## Автоматический скрипт для всех PR

```bash
#!/bin/bash

for pr in 507 506 505 503 502 501 500 499 496 495 494 490; do
  echo "=== Updating PR #$pr ==="

  # Checkout PR
  gh pr checkout $pr

  # Try to merge
  if git merge origin/main; then
    echo "✓ PR #$pr merged cleanly"
    git push origin HEAD --force-with-lease
  else
    echo "⚠ PR #$pr has conflicts - needs manual resolution"
    git merge --abort
  fi

  # Return to main
  git checkout main
done
```

## Альтернативный подход - закрыть и пересоздать

Если обновление слишком сложное, можно:
1. Скопировать контент глав из PR
2. Закрыть старый PR
3. Создать новый PR с правильной структурой

```bash
# Для одного PR
gh pr checkout <PR_NUMBER>

# Скопировать нужные файлы
cp -r chapters/<chapter>/ /tmp/

# Вернуться на main
git checkout main
git pull origin main

# Создать новую ветку
git checkout -b chapter-<number>-fixed

# Вернуть файлы
cp -r /tmp/<chapter>/ chapters/

# Если есть experiments, переместить их
mkdir -p chapters/<chapter>/experiments/
# ... скопировать файлы ...

# Commit и создать PR
git add .
git commit -m "Chapter <number>: <title> (adapted to new structure)"
git push origin chapter-<number>-fixed
gh pr create --base main --head chapter-<number>-fixed
```

## После обновления

После того как PR обновлены:
- Конфликты должны исчезнуть
- PR можно будет мерджить без проблем
- Новые PR не будут иметь конфликтов благодаря новой структуре
