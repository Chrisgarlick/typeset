#!/usr/bin/env bash
#
# render-json.sh — render a Typeset JSON layout document to PDF and download it.
#
# Usage:
#   ./render-json.sh [input.json] [output.pdf]
#
# Defaults: input  = php-basekit.json
#           output = <input basename>.pdf
#
# Token   : read from .env, falling back to .env.example (TYPESET_TOKEN=...)
# Endpoint: $TYPESET_URL or https://typeset.chrisgarlick.com
#
set -euo pipefail
cd "$(dirname "$0")"

INPUT="${1:-php-basekit.json}"
OUTPUT="${2:-${INPUT%.json}.pdf}"
BASE_URL="${TYPESET_URL:-https://typeset.chrisgarlick.com}"

# --- locate the input ------------------------------------------------------
if [[ ! -f "$INPUT" ]]; then
  echo "✗ Input file not found: $INPUT" >&2
  exit 1
fi

# --- resolve the token -----------------------------------------------------
TOKEN_FILE=""
for f in .env .env.example; do
  if [[ -f "$f" ]] && grep -q '^TYPESET_TOKEN=' "$f"; then
    TOKEN_FILE="$f"
    break
  fi
done
if [[ -z "$TOKEN_FILE" ]]; then
  echo "✗ Could not find TYPESET_TOKEN in .env or .env.example" >&2
  exit 1
fi
TOKEN="$(grep '^TYPESET_TOKEN=' "$TOKEN_FILE" | head -n1 | cut -d= -f2- | tr -d '"'"'"' \r')"

# --- build the request payload --------------------------------------------
# The JSON document is sent as a *string* in `content`, with input_format=json.
# document_type is taken from the document's frontmatter when present.
PAYLOAD="$(python3 - "$INPUT" <<'PY'
import json, sys
src = sys.argv[1]
doc = json.load(open(src))
fm = doc.get("frontmatter") or {}
dtype = (fm.get("document_type") or "general").lower()
valid = {"proposal","report","brief","sop","invoice","general"}
if dtype not in valid:
    dtype = "general"
print(json.dumps({
    "document_type": dtype,
    "format": "pdf",
    "input_format": "json",
    "content": json.dumps(doc),
}))
PY
)"

# --- render ----------------------------------------------------------------
echo "→ Rendering $INPUT  ($BASE_URL/api/render)"
HTTP_CODE="$(curl -sS -X POST "$BASE_URL/api/render" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD" \
  -o "$OUTPUT" \
  -w '%{http_code}')"

# --- report ----------------------------------------------------------------
if [[ "$HTTP_CODE" == "200" ]]; then
  echo "✓ Saved $OUTPUT ($(wc -c < "$OUTPUT" | tr -d ' ') bytes)"
else
  echo "✗ Render failed (HTTP $HTTP_CODE):" >&2
  cat "$OUTPUT" >&2; echo >&2
  rm -f "$OUTPUT"
  exit 1
fi
