#!/usr/bin/env bash
# Publish monorepo nested package as GitHub repo root (force-with-lease).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
BRANCH="${1:-main}"
git subtree split -P buzz-fuel-poc -b "tmp/github-root-$$"
git push --force-with-lease origin "tmp/github-root-$$:${BRANCH}"
git branch -D "tmp/github-root-$$"
echo "published buzz-fuel-poc/ as origin/${BRANCH} (package at root)"
