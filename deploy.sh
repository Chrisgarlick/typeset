#!/bin/bash
set -e

REMOTE_USER="root"
REMOTE_HOST="165.227.226.255"
SSH_KEY="$HOME/.ssh/keys/chrisgarlick"

echo "Deploying Typeset..."
ssh -i "$SSH_KEY" "${REMOTE_USER}@${REMOTE_HOST}" bash -s <<'REMOTE'
set -e
cd /var/www/typeset

git pull
docker compose up -d --build

sleep 3
curl -sf http://localhost:3200/health && echo " ✓ API running"
curl -sf http://localhost:3000 > /dev/null && echo " ✓ Frontend running"
REMOTE

echo "Deploy complete."
