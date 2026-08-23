#!/usr/bin/env bash
# Shared harness. Every example asserts its output, so the examples are
# tests: a change in behaviour fails CI rather than quietly making the
# README wrong.
set -euo pipefail

OXML="${OXML:-oxml}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA="$HERE/data"
FAILURES=0

# expect <description> <expected-exit> <expected-output> -- <args...>
expect() {
  local description="$1" want_code="$2" want_out="$3"
  shift 4  # description, code, output, and the literal --
  local out code
  set +e
  out="$("$OXML" "$@" 2>&1)"
  code=$?
  set -e
  if [[ "$code" != "$want_code" || "$out" != "$want_out" ]]; then
    echo "FAIL: $description"
    echo "  command : $OXML $*"
    echo "  exit    : got $code, want $want_code"
    echo "  output  : got ${out@Q}"
    echo "            want ${want_out@Q}"
    FAILURES=$((FAILURES + 1))
  else
    echo "ok: $description"
  fi
}

finish() {
  if [[ "$FAILURES" -gt 0 ]]; then
    echo "$FAILURES assertion(s) failed"
    exit 1
  fi
  echo "all assertions passed"
}
