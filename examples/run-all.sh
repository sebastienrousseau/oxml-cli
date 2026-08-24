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
for script in query-basics.sh inspect.sh pipeline.sh validate.sh; do
  echo
  echo "=== $script ==="
  bash "$script" || status=1
done
exit $status
