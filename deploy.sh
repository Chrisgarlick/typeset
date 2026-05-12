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

# Login to GitHub Container Registry
# echo $GHCR_TOKEN | docker login ghcr.io -u chrisgarlick --password-stdin 2>/dev/null || true

# Pull pre-built images and restart
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d

sleep 3
curl -sf http://localhost:3200/health && echo " ✓ API running"
curl -sf http://localhost:3000 > /dev/null && echo " ✓ Frontend running"
REMOTE

echo "Deploy complete."
