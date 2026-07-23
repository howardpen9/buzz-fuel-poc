#!/usr/bin/env bash
# Fail if publish-blocking patterns appear in the public tree.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail=0

check_absent() {
  local label="$1"
  local pattern="$2"
  # Search tracked-ish source/docs; skip evidence, target, binaries
  if rg -n --glob '!target/**' --glob '!evidence/**' --glob '!**/.git/**' \
      --glob '!**/LICENSE' --glob '!docs/16-OPEN-SOURCE-AND-DISTRIBUTION-AUDIT.md' \
      -e "$pattern" . >/tmp/buzz-fuel-hygiene-hits.txt 2>/dev/null; then
    echo "HYGIENE FAIL: $label"
    head -20 /tmp/buzz-fuel-hygiene-hits.txt
    fail=1
  else
    echo "ok: $label"
  fi
}

check_absent "personal absolute home paths" '/Users/[A-Za-z0-9._-]+/'
check_absent "Windows user paths" 'C:\\\\Users\\\\'
check_absent "likely live share token paths" 'makereel\.xyz/s/[A-Za-z0-9_-]{10,}'
check_absent "Railway-looking UUIDs in ops docs (heuristic)" \
  '(Environment ID|Project ID|Deployment ID|deploy id).*[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}'

if [[ ! -f LICENSE ]]; then
  echo "HYGIENE FAIL: missing LICENSE"
  fail=1
else
  echo "ok: LICENSE present"
fi

if [[ ! -f Cargo.lock ]]; then
  echo "HYGIENE FAIL: missing Cargo.lock (binary apps should commit lockfile)"
  fail=1
else
  echo "ok: Cargo.lock present"
fi

if rg -n '^Cargo\.lock$' .gitignore >/dev/null 2>&1; then
  echo "HYGIENE FAIL: Cargo.lock is gitignored"
  fail=1
else
  echo "ok: Cargo.lock not gitignored"
fi

if [[ $fail -ne 0 ]]; then
  echo "check-public-hygiene: FAILED"
  exit 1
fi
echo "check-public-hygiene: PASSED"
