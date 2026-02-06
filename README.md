# jico

Rust CLI for Jira Cloud: create issues, list/search, view, update fields, and transition statuses. All Jira responses are printed as pretty JSON for easy reading/parsing.

Current version: v0.0.3.

## Setup

Set variables directly in your environment or in a `.env` file in the working directory (environment variables override `.env` entries):
```
JIRA_BASE_URL=https://acme.atlassian.net
JIRA_EMAIL=dev@acme.io
JIRA_API_TOKEN=atlassian_api_token_here
# optional defaults
JIRA_PROJECT_KEY=ACME
JIRA_DEFAULT_JQL=project = ACME ORDER BY created DESC
```
Use `env.example` as a starting point: `cp env.example .env` and edit to suit your Jira site.

## Run

```
jico <command> [args]
```

Commands:
- `create <summary> [--description <text>] [--project <KEY>] [--issue-type <name>] [--parent <KEY>] [--labels <a,b>] [--priority <name>] [--assignee <accountId>]` — create an issue (use `--parent` for sub-tasks).
- `list [--jql <expr>] [--limit <n>] [--project <KEY>]` — list issues (defaults to `JIRA_DEFAULT_JQL` or `project = KEY`).
- `view <ISSUE-KEY> [--subtasks]` — show an issue or list its subtasks.
- `update <ISSUE-KEY> [--summary <text>] [--description <text>] [--project <KEY>] [--issue-type <name>] [--parent <KEY>] [--labels <a,b>] [--priority <name>] [--assignee <accountId>]` — update an issue (provide at least one field).
- `transition <ISSUE-KEY> --to <status>` — perform a transition by name (case-insensitive).
- `link <ISSUE-KEY> --to <ISSUE-KEY> [--relation <blocks|blocked-by>]` — create an issue link (default relation: `blocks`).

If no project is provided, `JIRA_PROJECT_KEY` is used (when present).

## Examples

```
jico create "Fix login"
jico create "Fix login" --labels bug,ui --priority High --assignee 12345:abcd
jico create "Child issue" --parent PROJ-1
jico list --limit 10
jico view PROJ-123
jico view PROJ-123 --subtasks
jico update PROJ-123 --summary "Tighten auth" --description "Rotated secrets"
jico transition PROJ-123 --to "In Progress"
jico link PROJ-26 --to PROJ-3 --relation blocked-by
```

## Packaging

- Build RPM (requires `rpmbuild`, `git`, and the Rust toolchain): `make rpm VERSION=0.0.3`
- Spec file lives at `packaging/jico.spec`; package installs the binary, man page (`jico(1)`), and `env.example` under `/usr/share/doc/jico/`.
