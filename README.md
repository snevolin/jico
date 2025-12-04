# jico

Rust CLI for Jira Cloud: create issues, list/search, view, and transition statuses. All Jira responses are printed as pretty JSON for easy reading/parsing.

Current version: v0.0.1.

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
- `create <summary> [--description <text>] [--project <KEY>] [--issue-type <name>]` — create an issue.
- `list [--jql <expr>] [--limit <n>] [--project <KEY>]` — list issues (defaults to `JIRA_DEFAULT_JQL` or `project = KEY`).
- `view <ISSUE-KEY>` — show an issue.
- `transition <ISSUE-KEY> --to <status>` — perform a transition by name (case-insensitive).

If no project is provided, `JIRA_PROJECT_KEY` is used (when present).

## Examples

```
jico create "Fix login"
jico list --limit 10
jico view PROJ-123
jico transition PROJ-123 --to "In Progress"
```

## Packaging

- Build RPM (requires `rpmbuild`, `git`, and the Rust toolchain): `make rpm VERSION=0.0.1`
- Spec file lives at `packaging/jico.spec`; package installs the binary, man page (`jico(1)`), and `env.example` under `/usr/share/doc/jico/`.
