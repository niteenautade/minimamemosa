#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "→ Killing any process on port 3000..."
lsof -ti:3000 2>/dev/null | xargs kill -9 2>/dev/null || true
sleep 1

echo "→ Resetting database..."
rm -f data/minimamemosa.db data/minimamemosa.db-wal data/minimamemosa.db-shm

echo "→ Starting server..."
nohup bash -c 'SESSION_SECRET=test-e2e-secret DATABASE_PATH=data/minimamemosa.db PORT=3000 exec ./target/release/minimamemosa' \
  > /tmp/minimamemosa.log 2>&1 &
sleep 2

echo "→ Removing old screenshots..."
rm -rf screenshots
mkdir -p screenshots

echo "→ Generating screenshots..."
npx tsx screenshots.ts

echo "→ Done — screenshots/ and screenshots/demo.gif regenerated"
