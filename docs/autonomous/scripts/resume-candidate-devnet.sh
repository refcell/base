#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
repo="$root/worktrees/load-test"
cd "$repo"
export PATH="$HOME/.config/.foundry/bin:$HOME/.cargo/bin:$PATH"
export COMPOSE_PROJECT_NAME=base-autonomous-20260924
export BASE_FACTORY_REPO="$repo"
export BASE_FACTORY_EXPECTED_REV=450b4d7b60a74194c4196283b85d500e1392f946
export BASE_FACTORY_EXTRA_OVERRIDE="$root/state/candidate-load.override.yml"
export BASE_FACTORY_EXPECTED_IMAGE=sha256:18ac180dc65cbbde1784c562f7ab6623c8fdb443f3b47e50ae65be154d03d6bb
finish() {
  code=$?
  printf '\nCANDIDATE_RESTART_EXIT_CODE=%s\nCANDIDATE_RESTART_DONE %s\n' "$code" "$(date -u +%FT%TZ)"
  tmux wait-for -S base-autonomous-candidate-restart-done
}
trap finish EXIT
grep -q '^CANDIDATE_TESTS_EXIT_CODE=' "$root/logs/candidate-tests1.log"
grep -q '^BASELINE_VERIFY_EXIT_CODE=0$' "$root/logs/candidate-verify1.log"
# Services were stopped (data kept) while the workspace suite ran to avoid test-port interference.
docker compose --env-file etc/docker/devnet-env -f etc/docker/docker-compose.yml \
  -f "$root/state/baseline-compose.override.yml" -f "$BASE_FACTORY_EXTRA_OVERRIDE" \
  start l1-el l1-cl l1-vc base-bootnode base-builder base-client base-rpc \
  base-shadow-validator op-batcher base-batcher prometheus grafana jaeger
bash "$root/state/verify-baseline-devnet.sh" baseline-load-candidate-restart
old="$root/logs/baseline-load-candidate/receipt.json"
new="$root/logs/baseline-load-candidate-restart/receipt.json"
old_tx=$(jq -r .transactionHash "$old")
for port in 7545 8545; do
  file="$root/logs/baseline-load-candidate-restart/old-receipt-$port.json"
  timeout 20 cast receipt "$old_tx" --rpc-url "http://127.0.0.1:$port" --json > "$file"
  jq -e --slurpfile old "$old" '.status == "0x1" and .transactionHash == $old[0].transactionHash and .blockHash == $old[0].blockHash' "$file"
done
old_block=$(jq -r .blockNumber "$old")
new_block=$(jq -r .blockNumber "$new")
(( new_block > old_block ))
echo 'CANDIDATE_PERSISTENCE_AND_PROGRESS_PASS=true'
sha256sum "$root/integration/target/debug/base" "$root/integration/target/debug/base-load-tester"
bash "$root/state/run-load-parity.sh" unified load candidate-node
bash "$root/state/run-load-parity.sh" unified continuous candidate-node
