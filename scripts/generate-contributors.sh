#!/usr/bin/env bash
# Generate the contributor list from Git history (commits reachable from HEAD).
#
# Usage: ./scripts/generate-contributors.sh [OUTPUT_FILE]
#
#   OUTPUT_FILE defaults to book/src/contributors.md.
#
#   If OUTPUT_FILE contains the markers
#       <!-- contributors:start --> ... <!-- contributors:end -->
#   only the text between them is replaced (used for CONTRIBUTORS.md, whose
#   tier tables are maintained by hand). Otherwise the whole file is written.
#
# Run from the repository root. Runs automatically after every merge to main
# via .github/workflows/contributors.yml.
set -euo pipefail

OUTPUT_FILE="${1:-book/src/contributors.md}"
START_MARKER="<!-- contributors:start -->"
END_MARKER="<!-- contributors:end -->"

if ! command -v git >/dev/null 2>&1; then
  echo "git is required to generate the contributors page." >&2
  exit 1
fi

if [ ! -d .git ]; then
  echo "This script must be run from the repository root." >&2
  exit 1
fi

# Portable awk (works with mawk/BSD awk, not only gawk). Bot accounts are skipped.
# Names link to the GitHub profile when the commit email is a GitHub noreply
# address; other email addresses are not published.
CONTRIBUTORS_MARKDOWN=$(git shortlog -sne HEAD | awk '
  {
    count = $1
    line = $0
    sub(/^[ \t]*[0-9]+[ \t]+/, "", line)
    email = line; sub(/^.*</, "", email); sub(/>$/, "", email)
    name = line;  sub(/ <[^>]*>$/, "", name)
    if (name ~ /\[bot\]$/) next
    profile = name
    if (email ~ /@users\.noreply\.github\.com$/) {
      user = email
      sub(/@.*$/, "", user)
      sub(/^[^+]*\+/, "", user)
      profile = "[" name "](https://github.com/" user ")"
    }
    printf("| %s | %s |\n", count, profile)
    total += count; people++
  }
  END { printf("TOTALS %d %d\n", total, people) }')

TOTALS_LINE=$(printf '%s\n' "$CONTRIBUTORS_MARKDOWN" | tail -n 1)
CONTRIBUTORS_MARKDOWN=$(printf '%s\n' "$CONTRIBUTORS_MARKDOWN" | sed '$d')
TOTAL_COMMITS=$(echo "$TOTALS_LINE" | cut -d' ' -f2)
TOTAL_CONTRIBUTORS=$(echo "$TOTALS_LINE" | cut -d' ' -f3)

TABLE=$(cat <<EOF
- **Total commits:** $TOTAL_COMMITS
- **Total contributors:** $TOTAL_CONTRIBUTORS

| Commits | Contributor |
| --- | --- |
$CONTRIBUTORS_MARKDOWN
EOF
)

if [ -f "$OUTPUT_FILE" ] && grep -qF "$START_MARKER" "$OUTPUT_FILE"; then
  if ! grep -qF "$END_MARKER" "$OUTPUT_FILE"; then
    echo "$OUTPUT_FILE has $START_MARKER but no $END_MARKER." >&2
    exit 1
  fi
  TMP_FILE="$(mktemp)"
  TABLE="$TABLE" awk -v start="$START_MARKER" -v end="$END_MARKER" '
    $0 == start { print; print ENVIRON["TABLE"]; skip = 1; next }
    $0 == end   { skip = 0 }
    !skip       { print }
  ' "$OUTPUT_FILE" > "$TMP_FILE"
  mv "$TMP_FILE" "$OUTPUT_FILE"
  exit 0
fi

cat > "$OUTPUT_FILE" <<EOF
# Contributors

This page is generated automatically from Git commit history by \`scripts/generate-contributors.sh\`.

> Thank you to everyone who contributes to the Soroban Cookbook. Your work helps the project grow and improve.

## Project Contribution Statistics

$TABLE

## How this page is updated

This page is regenerated after every merge to \`main\` by the Contributors workflow and can also be regenerated locally with:

\`\`\`bash
./scripts/generate-contributors.sh
\`\`\`
EOF
