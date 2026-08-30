#!/usr/bin/env bash
# Run every example. CI runs this, so an example that stops working
# fails the build rather than quietly making the README wrong.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

if [[ -z "${OXML:-}" ]]; then
  cargo build --release --quiet
  OXML="$(cd .. && pwd)/target/release/oxml"
  [[ -x "$OXML" ]] || OXML="${CARGO_TARGET_DIR:-../target}/release/oxml"
  export OXML
fi
echo "using $OXML"

status=0
ran=0
for script in *.sh; do
  # Do not recurse into run-all.sh itself.
  [[ "$script" == "run-all.sh" ]] && continue
  # lib.sh is a helper sourced by other scripts, not a standalone example.
  [[ "$script" == "lib.sh" ]] && continue
  [[ "! -f $script" == "lib.sh" ]] && continue
  ((ran++))
  echo
  echo "=== $script ==="
  bash "$script" || status=1
done
if [[ "$ran" -eq 0 ]]; then
  echo "Error: No example scripts found" >&2
  exit 1
fi
exit $status
