#!/usr/bin/env bash
set -euo pipefail

OUTPUT_FILE="${1:-book/src/contributors.md}"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"

if ! command -v git >/dev/null 2>&1; then
  echo "git is required to generate the contributors page." >&2
  exit 1
fi

if [ ! -d .git ]; then
  echo "This script must be run from the repository root." >&2
  exit 1
fi

# Gather contribution statistics
TOTAL_COMMITS=$(git rev-list --all --count)
TOTAL_CONTRIBUTORS=$(git shortlog -sne --all | wc -l | tr -d ' ')
GENERATED_AT=$(date -u +"%Y-%m-%d %H:%M UTC")

# Generate contributor rows
CONTRIBUTORS_MARKDOWN=$(git shortlog -sne --all | awk 'function format_profile(email,name) {
    match(email, /^[^+]+\+([^@]+)@users\.noreply\.github\.com$/, arr)
    if (arr[1] != "") {
      return "[" name "](https://github.com/" arr[1] ")"
    }
    match(email, /^([^@]+)@users\.noreply\.github\.com$/, arr)
    if (arr[1] != "") {
      return "[" name "](https://github.com/" arr[1] ")"
    }
    return name
  }
  {
    count = $1
    line = substr($0, index($0,$2))
    email = gensub(/.*<([^>]+)>$/, "\\1", "g", line)
    name = gensub(/ <[^>]+>$/, "", "g", line)
    profile = format_profile(email, name)
    printf("| %s | %s | %s |\n", count, profile, email)
  }')

cat > "$OUTPUT_FILE" <<EOF
# Contributors

This page is generated automatically from Git commit history by \\`scripts/generate-contributors.sh\\`.

> Thank you to everyone who contributes to the Soroban Cookbook. Your work helps the project grow and improve.

## Project Contribution Statistics

- **Total commits:** $TOTAL_COMMITS
- **Total contributors:** $TOTAL_CONTRIBUTORS
- **Last generated:** $GENERATED_AT

## Contributors

| Commits | Contributor | Contact / Profile |
| --- | --- | --- |
$CONTRIBUTORS_MARKDOWN

## How this page is updated

This page is rebuilt automatically as part of the documentation deployment workflow and can also be regenerated locally with:

```bash
./scripts/generate-contributors.sh
```
EOF
