#!/bin/sh
# Connects Typeset to Clients (run on the server, in /var/www/typeset):
# reads a Clients service key on stdin, writes CLIENTS_URL and
# CLIENTS_SERVICE_KEY into .env (replacing old values) and restarts the API.
# The key is never printed.
set -eu
cd "$(dirname "$0")/.."
key=$(tr -d '[:space:]')
case "$key" in
  ck_*) ;;
  *) echo "no key on stdin (expected ck_…)" >&2; exit 1 ;;
esac
touch .env
grep -v '^CLIENTS_URL=' .env | grep -v '^CLIENTS_SERVICE_KEY=' > .env.new || true
printf 'CLIENTS_URL=https://clients.kritano.com\nCLIENTS_SERVICE_KEY=%s\n' "$key" >> .env.new
chmod 600 .env.new
mv .env.new .env
docker compose -f docker-compose.prod.yml up -d typeset
sleep 4
docker compose -f docker-compose.prod.yml logs --tail 20 typeset | grep -i "clients\|error" || true
