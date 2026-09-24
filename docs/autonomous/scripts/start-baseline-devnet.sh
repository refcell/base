#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
cd "$root/integration"
export COMPOSE_PROJECT_NAME=base-autonomous-20260924
# Deliberately no stock `just devnet down`: it targets shared defaults and deletes data.
grep -q '^BASELINE_IMAGES_EXIT_CODE=0$' "$root/logs/baseline-images-attempt1.log"
test "$(git rev-parse HEAD)" = 539605ce58aaf02fe5c0382fcd9139c1ee4207e9
test -z "$(git status --porcelain --untracked-files=no)"
docker compose --env-file etc/docker/devnet-env \
  -f etc/docker/docker-compose.yml -f "$root/state/baseline-compose.override.yml" \
  up --no-build
