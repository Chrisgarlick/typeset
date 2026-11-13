#!/usr/bin/env bash
# Stages Typeset's brand in Clients: uploads the font, then the logos (Clients
# outlines their text with the font), then saves the brand draft. Publishing
# stays in the Clients admin pages (password and code).
#
# Runs the uploads on the droplet with Hub's Clients key, which never leaves it.
set -euo pipefail
cd "$(dirname "$0")"

HOST=root@167.71.134.177
SSH=(ssh -i ~/.ssh/keys/kritano "$HOST")

"${SSH[@]}" 'mkdir -p /tmp/typeset-brand'
scp -q -i ~/.ssh/keys/kritano \
  BarlowCondensed-Bold.ttf typeset-logo.svg typeset-logo-on-dark.svg typeset-mark.svg typeset-brand.yaml \
  "$HOST:/tmp/typeset-brand/"

"${SSH[@]}" 'bash -s' <<'REMOTE'
set -euo pipefail
cd /tmp/typeset-brand
trap 'rm -rf /tmp/typeset-brand' EXIT
KEY=$(sed -nE 's/^ *clients_service_key *= *"([^"]+)".*/\1/p' /var/lib/hub/hub.toml)
[ -n "$KEY" ] || { echo "No clients_service_key in /var/lib/hub/hub.toml"; exit 1; }
put() {
  echo "== $1"
  curl -sS -w '\n[%{http_code}]\n' -X PUT -H "Authorization: Bearer $KEY" \
    --data-binary @"$2" "http://127.0.0.1:8130/v1/clients/typeset/$1"
}
put library/barlow-condensed-bold BarlowCondensed-Bold.ttf
put library/typeset-logo typeset-logo.svg
put library/typeset-logo-on-dark typeset-logo-on-dark.svg
put library/typeset-mark typeset-mark.svg
put draft typeset-brand.yaml
REMOTE
