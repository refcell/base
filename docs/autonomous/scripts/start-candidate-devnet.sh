#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
repo="$root/worktrees/load-test"
expected=450b4d7b60a74194c4196283b85d500e1392f946
cd "$repo"
export COMPOSE_PROJECT_NAME=base-autonomous-20260924
test "$(git rev-parse HEAD)" = "$expected"
test -z "$(git status --porcelain)"
grep -q '^CANDIDATE_IMAGE_EXIT_CODE=0$' "$root/logs/candidate-image1.log"
grep -q '^LOAD_AFTER_FIX_EXIT_CODE=0$' "$root/logs/load-parity-after-fix1.log"
test "$(docker image inspect --format '{{index .Config.Labels "org.opencontainers.image.revision"}}' base:base-autonomous-load-20260924)" = "$expected"
test ! -e "$root/state/candidate-load-devnet"
compose=(docker compose --env-file etc/docker/devnet-env -f etc/docker/docker-compose.yml -f "$root/state/baseline-compose.override.yml" -f "$root/state/candidate-load.override.yml")
"${compose[@]}" config --format json > "$root/state/candidate-load.resolved.json"
jq -e '.name == "base-autonomous-20260924" and (.services|length)==14 and all(.services[]; .container_name|startswith("base-autonomous-20260924-")) and all(.services[].ports[]?; .host_ip == "127.0.0.1") and all(.services[].volumes[]? | select(.type=="bind" and .read_only!=true); .source|startswith("/home/refcell/base-autonomous-20260924/state/candidate-load-devnet/"))' "$root/state/candidate-load.resolved.json"
# Stop/remove ONLY this experiment's prior containers/network. Bind data is retained.
# No volumes flag, no remove-orphans, no host/global cleanup.
(cd "$root/integration" && docker compose --env-file etc/docker/devnet-env \
  -f etc/docker/docker-compose.yml -f "$root/state/baseline-compose.override.yml" down --timeout 30)
printf 'CANDIDATE_DEVNET_START %s\n' "$(date -u +%FT%TZ)"
git rev-parse HEAD HEAD^{tree}
"${compose[@]}" up --no-build
