#!/usr/bin/env bash
# Render all README mermaid diagrams to PNG (dark bg, 2× scale).
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

export PUPPETEER_EXECUTABLE_PATH="${PUPPETEER_EXECUTABLE_PATH:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
BG="#0B0F19"
SCALE=2

if [[ ! -x "$PUPPETEER_EXECUTABLE_PATH" ]]; then
  echo "Chrome not found at $PUPPETEER_EXECUTABLE_PATH" >&2
  echo "Set PUPPETEER_EXECUTABLE_PATH to a Chromium binary." >&2
  exit 1
fi

shopt -s nullglob
files=(*.mmd)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "no .mmd files" >&2
  exit 1
fi

for mmd in "${files[@]}"; do
  # Skip theme snippet helper
  [[ "$mmd" == _* ]] && continue
  base="${mmd%.mmd}"
  out="${base}.png"
  echo "render $mmd → $out"
  npx --yes @mermaid-js/mermaid-cli@11.4.2 \
    -i "$mmd" \
    -o "$out" \
    -b "$BG" \
    -s "$SCALE"
done

echo "done"
ls -la *.png
