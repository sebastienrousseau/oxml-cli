#!/usr/bin/env bash
#
# Composing with other tools: standard input, and exit codes a shell
# script can branch on.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OXML="${OXML:-oxml}"
DATA="$HERE/data"
failures=0

# FILE defaults to standard input.
got="$(< "$DATA/catalogue.xml" "$OXML" query -t '//title')"
if [[ "$got" != $'Dune\nGerminal' ]]; then
  echo "FAIL: reading from standard input"; failures=$((failures + 1))
else
  echo "ok: reading from standard input"
fi

# Exit 1 means "no match", so this reads naturally in a conditional.
if "$OXML" query -c '//nonexistent' "$DATA/catalogue.xml" > /dev/null 2>&1; then
  echo "FAIL: a query matching nothing should exit non-zero"
  failures=$((failures + 1))
else
  echo "ok: no match exits non-zero, so \`if\` works"
fi

# One result per line means the usual tools compose.
count="$("$OXML" query -t '//title' "$DATA/catalogue.xml" | wc -l | tr -d ' ')"
if [[ "$count" != "2" ]]; then
  echo "FAIL: expected 2 lines, got $count"; failures=$((failures + 1))
else
  echo "ok: output is one result per line"
fi

# Diagnostics go to stderr, so stdout stays clean for a pipe.
stdout="$("$OXML" check "$DATA/broken.xml" 2>/dev/null || true)"
if [[ -n "$stdout" ]]; then
  echo "FAIL: diagnostics leaked to stdout: ${stdout@Q}"
  failures=$((failures + 1))
else
  echo "ok: diagnostics go to stderr"
fi

[[ "$failures" -eq 0 ]] || { echo "$failures failed"; exit 1; }
echo "all assertions passed"
