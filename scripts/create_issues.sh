#!/bin/bash

# Скрипт для создания GitHub issues для всех ненаписанных глав
# Использование: ./create_issues.sh
# Требования: gh CLI (https://cli.github.com/)

REPO="suenot/rust-learning-for-trading"

# Проверка gh
if ! command -v gh &> /dev/null; then
    echo "gh CLI не найден. Установите: https://cli.github.com/"
    exit 1
fi

# Проверка авторизации
if ! gh auth status &> /dev/null; then
    echo "Требуется авторизация: gh auth login"
    exit 1
fi

echo "Создание issues для глав 16-365..."
echo "Репозиторий: $REPO"
echo ""

# Читаем issues из JSON файла и создаём
while IFS= read -r line; do
    title=$(echo "$line" | jq -r '.title')
    body=$(echo "$line" | jq -r '.body')
    labels=$(echo "$line" | jq -r '.labels | join(",")')

    echo "Создаю: $title"

    gh issue create \
        --repo "$REPO" \
        --title "$title" \
        --body "$body" \
        --label "$labels" \
        2>/dev/null

    # Небольшая пауза чтобы не превысить rate limit
    sleep 0.5
done < <(jq -c '.[]' scripts/issues_data.json)

echo ""
echo "Готово!"
