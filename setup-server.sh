#!/bin/bash
set -e

# =============================================================================
# Typeset — One-time server setup
# Run this locally: ./setup-server.sh <droplet-ip>
# =============================================================================

if [ -z "$1" ]; then
    echo "Usage: ./setup-server.sh <droplet-ip>"
    echo "Example: ./setup-server.sh 164.90.xxx.xxx"
    exit 1
fi

REMOTE_HOST="$1"
REMOTE_USER="root"
SSH_KEY="$HOME/.ssh/keys/chrisgarlick"

echo "Setting up Typeset on ${REMOTE_HOST}..."
echo ""

# --- Step 1: Create user, directories, and install nginx/certbot ---
echo "[1/5] Creating typeset user and directories..."
ssh -i "$SSH_KEY" "${REMOTE_USER}@${REMOTE_HOST}" bash -s <<'SETUP'
set -e

if ! id -u typeset &>/dev/null; then
    useradd --system --no-create-home --shell /usr/sbin/nologin typeset
    echo "Created typeset user"
else
    echo "typeset user already exists"
fi

mkdir -p /opt/typeset/migrations
chown -R typeset:typeset /opt/typeset

apt-get update -qq
apt-get install -y -qq nginx certbot python3-certbot-nginx > /dev/null
echo "nginx and certbot ready"
SETUP

# --- Step 2: Upload systemd service ---
echo ""
echo "[2/5] Installing systemd service..."
scp -i "$SSH_KEY" typeset.service "${REMOTE_USER}@${REMOTE_HOST}:/etc/systemd/system/typeset.service"
ssh -i "$SSH_KEY" "${REMOTE_USER}@${REMOTE_HOST}" bash -s <<'SYSTEMD'
systemctl daemon-reload
systemctl enable typeset
echo "systemd service installed and enabled"
SYSTEMD

# --- Step 3: Upload nginx config ---
echo ""
echo "[3/5] Installing nginx config..."
scp -i "$SSH_KEY" typeset.nginx.conf "${REMOTE_USER}@${REMOTE_HOST}:/etc/nginx/sites-available/typeset"
ssh -i "$SSH_KEY" "${REMOTE_USER}@${REMOTE_HOST}" bash -s <<'NGINX'
set -e

if ! grep -q "typeset_api" /etc/nginx/nginx.conf; then
    sed -i '/http {/a \    limit_req_zone $binary_remote_addr zone=typeset_api:10m rate=30r/m;' /etc/nginx/nginx.conf
    echo "Added rate limit zone"
fi

ln -sf /etc/nginx/sites-available/typeset /etc/nginx/sites-enabled/typeset
nginx -t 2>&1 || true
echo "nginx config installed"
NGINX

# --- Step 4: Upload .env ---
echo ""
echo "[4/5] Setting up environment..."

if [ -f .env ]; then
    scp -i "$SSH_KEY" .env "${REMOTE_USER}@${REMOTE_HOST}:/opt/typeset/.env"
    ssh -i "$SSH_KEY" "${REMOTE_USER}@${REMOTE_HOST}" "chown typeset:typeset /opt/typeset/.env && chmod 600 /opt/typeset/.env"
    echo "Uploaded .env"
else
    echo "No .env file found locally. Creating template on server..."
    scp -i "$SSH_KEY" .env.example "${REMOTE_USER}@${REMOTE_HOST}:/opt/typeset/.env"
    ssh -i "$SSH_KEY" "${REMOTE_USER}@${REMOTE_HOST}" "chown typeset:typeset /opt/typeset/.env && chmod 600 /opt/typeset/.env"
    echo ""
    echo "  WARNING: You need to edit /opt/typeset/.env on the server with real values:"
    echo "    ssh -i ~/.ssh/keys/chrisgarlick root@${REMOTE_HOST} nano /opt/typeset/.env"
fi

# --- Step 5: SSL ---
echo ""
echo "[5/5] SSL certificate..."
echo ""
echo "  Before running certbot, make sure:"
echo "  1. DNS A record for typeset.chrisgarlick.com points to ${REMOTE_HOST}"
echo "  2. The nginx config is using the correct server_name"
echo ""
read -p "  Run certbot now? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    ssh -i "$SSH_KEY" "${REMOTE_USER}@${REMOTE_HOST}" bash -s <<'CERTBOT'
set -e

cat > /etc/nginx/sites-available/typeset <<'TMPCONF'
server {
    listen 80;
    server_name typeset.chrisgarlick.com;

    location /api/ {
        proxy_pass         http://127.0.0.1:3200;
        proxy_http_version 1.1;
        proxy_set_header   Host $host;
        proxy_set_header   X-Real-IP $remote_addr;
        proxy_read_timeout 60s;
        limit_req zone=typeset_api burst=20 nodelay;
    }

    location /health {
        proxy_pass http://127.0.0.1:3200;
    }
}
TMPCONF

nginx -t && systemctl reload nginx
certbot --nginx -d typeset.chrisgarlick.com --non-interactive --agree-tos --redirect -m your-email@example.com || {
    echo "Certbot failed — you can run it manually later:"
    echo "  certbot --nginx -d typeset.chrisgarlick.com"
}
CERTBOT
else
    echo "  Skipped. Run manually later:"
    echo "    ssh -i ~/.ssh/keys/chrisgarlick root@${REMOTE_HOST} certbot --nginx -d typeset.chrisgarlick.com"
fi

echo ""
echo "============================================"
echo "  Setup complete!"
echo ""
echo "  Next steps:"
echo "  1. Edit /opt/typeset/.env on the server"
echo "  2. Create the PostgreSQL database:"
echo "     createdb typeset"
echo "  3. Build and deploy the binary:"
echo "     ./deploy.sh"
echo "============================================"
