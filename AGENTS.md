# AGENTS Instructions for jico

## Version Policy (Required)
Every pull request MUST bump the project version.

### Which version to bump
Use SemVer and bump at least one level per PR:
- Default: patch bump (`x.y.z` -> `x.y.(z+1)`) for fixes and regular changes.
- Minor bump (`x.y.z` -> `x.(y+1).0`) for backward-compatible feature additions.
- Major bump (`x.y.z` -> `(x+1).0.0`) for breaking changes.

For this repository, if unsure, use a patch bump.

### Files that must stay in sync
When bumping version, update these files in the same PR:
- `Cargo.toml` (`[package].version`)
- `Makefile` (`VERSION ?=`)
- `README.md` (`Current version: ...` and RPM example if needed)
- `README_rus.md` (`Текущая версия: ...` and RPM example if needed)
- `packaging/jico.1` (manpage header version)

## Why this is required
RPM upgrades/installations rely on package version/release ordering. Rebuilding an RPM with the same version often does not upgrade cleanly. Bumping version per PR avoids reinstall/upgrade conflicts.
