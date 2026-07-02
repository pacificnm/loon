#!/usr/bin/env bash
# Live end-to-end: POST /api/library/scan → Ollama → TMDB → GET movie detail
#
# Prerequisites:
#   - loon-server running (./build run)
#   - ollama serve + smollm2:360m pulled
#   - TMDB_API_KEY exported (or api_key in config.toml)
#   - [ai] section in config.toml
#
# Usage:
#   ./scripts/e2e-enrichment-live.sh [base_url]
#
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3000}"
NARNIA="the_chronicles_of_narnia_the_lion_the_witch_and_the_wardrobe.mp4"

echo "== Loon live enrichment e2e =="
echo "Server: $BASE_URL"

echo
echo "1. Library status (before)"
curl -s "$BASE_URL/api/library/status" | python3 -m json.tool

echo
echo "2. Trigger full library scan"
curl -s -X POST "$BASE_URL/api/library/scan" \
  -H 'Content-Type: application/json' \
  -d '{"full": true}' | python3 -m json.tool

echo
echo "3. Waiting for scan to finish..."
for i in $(seq 1 120); do
  STATUS=$(curl -s "$BASE_URL/api/library/status")
  STATE=$(echo "$STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['state'])")
  IN_PROGRESS=$(echo "$STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['scan_in_progress'])")
  COUNT=$(echo "$STATUS" | python3 -c "import sys,json; print(json.load(sys.stdin)['movies_count'])")
  echo "   [$i] state=$STATE in_progress=$IN_PROGRESS movies=$COUNT"
  if [ "$STATE" = "idle" ] && [ "$IN_PROGRESS" = "False" ]; then
    break
  fi
  sleep 2
done

echo
echo "4. Search for Narnia in catalog"
curl -s "$BASE_URL/api/search?q=narnia" | python3 -m json.tool

echo
echo "5. Movie detail (first search hit)"
SLUG=$(curl -s "$BASE_URL/api/search?q=narnia" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['movies'][0]['slug'] if d.get('movies') else '')")
if [ -z "$SLUG" ]; then
  echo "ERROR: No Narnia movie found after scan"
  exit 1
fi
echo "   slug=$SLUG"
curl -s "$BASE_URL/api/movies/$SLUG" | python3 -m json.tool

echo
echo "Done. Verify title/year/summary/poster_url came from TMDB (not raw filename)."
