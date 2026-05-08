#!/bin/bash
set -e

REMOTE_USER="root"
REMOTE_HOST="165.227.226.255"
SSH_KEY="$HOME/.ssh/keys/chrisgarlick"

echo "Deploying Typeset..."
ssh -i "$SSH_KEY" "${REMOTE_USER}@${REMOTE_HOST}" bash -s <<'REMOTE'
set -e
cd /opt/typeset

git pull
docker compose up -d --build

sleep 3
curl -sf http://localhost:3200/health && echo " ✓ Typeset running" || echo " ✗ Health check failed"
REMOTE

echo "Deploy complete."
