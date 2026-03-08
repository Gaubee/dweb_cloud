#!/bin/sh
set -eu

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ] || [ $# -lt 2 ]; then
  echo "Usage: ./scripts/smoke-public-webdav.sh <server-url> <secret> [app-id]"
  echo "Example: ./scripts/smoke-public-webdav.sh https://cloud.example.com 'your secret' gaubee-2fa"
  exit 0
fi

SERVER_URL="$1"
SECRET_INPUT="$2"
APP_ID="${3:-gaubee-2fa}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

ISSUE_JSON="$TMP_DIR/issue.json"
SMOKE_TEXT="dweb-cloud-smoke-$(date +%s)"

cargo run -q -p dweb-cloud-cli -- token issue \
  --server "$SERVER_URL" \
  --app "$APP_ID" \
  --secret "$SECRET_INPUT" \
  --json > "$ISSUE_JSON"

WEBDAV_BASE_URL="$(jq -r '.webdavBaseUrl' "$ISSUE_JSON")"
USERNAME="$(jq -r '.username' "$ISSUE_JSON")"
PASSWORD="$(jq -r '.password' "$ISSUE_JSON")"

printf '%s' "$SMOKE_TEXT" > "$TMP_DIR/payload.txt"

curl -fsS -u "$USERNAME:$PASSWORD" -T "$TMP_DIR/payload.txt" "$WEBDAV_BASE_URL/.smoke.txt" >/dev/null
curl -fsS -u "$USERNAME:$PASSWORD" "$WEBDAV_BASE_URL/.smoke.txt" > "$TMP_DIR/result.txt"

diff -u "$TMP_DIR/payload.txt" "$TMP_DIR/result.txt" >/dev/null

echo "Smoke OK"
echo "Server: $SERVER_URL"
echo "WebDAV: $WEBDAV_BASE_URL"
echo "Account: $USERNAME"
