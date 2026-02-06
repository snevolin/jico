# jico

Консольная утилита на Rust для работы с Jira Cloud: создание, просмотр, редактирование, список задач и переходы статусов. Все ответы от Jira выводятся в JSON (pretty-print), чтобы их легко было парсить/читать.

Текущая версия: v0.0.3.

## Настройка

Задайте переменные окружения напрямую или через файл `.env` в рабочем каталоге (переменные окружения имеют приоритет над `.env`):
```
JIRA_BASE_URL=https://acme.atlassian.net
JIRA_EMAIL=dev@acme.io
JIRA_API_TOKEN=atlassian_api_token_here
# опционально
JIRA_PROJECT_KEY=ACME
JIRA_DEFAULT_JQL=project = ACME ORDER BY created DESC
```
Используйте `env.example` как основу: `cp env.example .env` и отредактируйте под свой Jira-сайт.

## Запуск

```
jico <command> [args]
```

Команды:
- `create <summary> [--description <text>] [--project <KEY>] [--issue-type <name>] [--parent <KEY>] [--labels <a,b>] [--priority <name>] [--assignee <accountId>]` — создать задачу (для сабтаска используйте `--parent`).
- `list [--jql <expr>] [--limit <n>] [--project <KEY>]` — список задач (по умолчанию `JIRA_DEFAULT_JQL` или `project = KEY`).
- `view <ISSUE-KEY> [--subtasks]` — показать задачу или список сабтасков.
- `update <ISSUE-KEY> [--summary <text>] [--description <text>] [--project <KEY>] [--issue-type <name>] [--parent <KEY>] [--labels <a,b>] [--priority <name>] [--assignee <accountId>]` — изменить поля задачи (нужно указать хотя бы одно поле).
- `transition <ISSUE-KEY> --to <status>` — выполнить переход по статусу/transition name (по имени без учета регистра).
- `link <ISSUE-KEY> --to <ISSUE-KEY> [--relation <blocks|blocked-by|clones|is-cloned-by|duplicates|is-duplicated-by|relates-to>]` — создать связь между задачами (по умолчанию: `blocks`).

Если не указан проект, используется `JIRA_PROJECT_KEY` (если задан).

## Пример

```
jico create "Fix login"
jico create "Fix login" --labels bug,ui --priority High --assignee 12345:abcd
jico create "Child issue" --parent PROJ-1
jico list --limit 10
jico view PROJ-123
jico view PROJ-123 --subtasks
jico update PROJ-123 --summary "Уточнить задачу" --description "Подправили текст"
jico transition PROJ-123 --to "In Progress"
jico link PROJ-26 --to PROJ-3 --relation blocked-by
jico link PROJ-26 --to PROJ-3 --relation relates-to
```

## Сборка RPM

- `make rpm VERSION=0.0.3` (требуются `rpmbuild`, `git` и Rust toolchain).
- Спека: `packaging/jico.spec`; пакет устанавливает бинарник, man-страницу (`jico(1)`) и `env.example` в `/usr/share/doc/jico/`.
