#!/bin/sh
# Writes every old client profile as brand YAML for Clients, with its logos,
# to /tmp/typeset-brands on the server, and prints the YAML (plan P4.1).
set -eu
cd "$(dirname "$0")/.."
docker compose -f docker-compose.prod.yml exec -T typeset typeset export-brands /tmp/typeset-brands
rm -rf /tmp/typeset-brands
docker compose -f docker-compose.prod.yml cp typeset:/tmp/typeset-brands /tmp/typeset-brands
for f in /tmp/typeset-brands/*.yaml; do
  echo "=== $f"
  cat "$f"
done
ls -l /tmp/typeset-brands
