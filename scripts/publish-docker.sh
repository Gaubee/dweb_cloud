#!/bin/sh
set -eu

IMAGE_NAME="${IMAGE_NAME:-gaubee/dweb-cloud}"
DEFAULT_VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  echo "Usage: ./scripts/publish-docker.sh [version]"
  echo "Default version: ${DEFAULT_VERSION}"
  exit 0
fi
VERSION="${1:-$DEFAULT_VERSION}"
PLATFORMS="${PLATFORMS:-linux/amd64,linux/arm64}"

echo "Publishing ${IMAGE_NAME}:${VERSION} and ${IMAGE_NAME}:latest"
docker buildx build \
  --platform "${PLATFORMS}" \
  -t "${IMAGE_NAME}:${VERSION}" \
  -t "${IMAGE_NAME}:latest" \
  --push \
  .
