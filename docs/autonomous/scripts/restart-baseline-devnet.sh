#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
cd "$root/integration"
export PATH="$HOME/.config/.foundry/bin:$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export COMPOSE_PROJECT_NAME=base-autonomous-20260924
finish() {
  code=$?
  printf '\nBASELINE_RESTART_EXIT_CODE=%s\nBASELINE_RESTART_DONE %s\n' "$code" "$(date -u +%FT%TZ)"
  tmux wait-for -S base-autonomous-restart-done
}
trap finish EXIT
grep -q '^BASELINE_VERIFY_EXIT_CODE=0$' "$root/logs/baseline-verify2.log"
echo "BASELINE_RESTART_START $(date -u +%FT%TZ)"
# Exclude the one-shot genesis generator: restarting it is not a persistence test.
docker compose --env-file etc/docker/devnet-env -f etc/docker/docker-compose.yml \
  -f "$root/state/baseline-compose.override.yml" restart --timeout 30 \
  l1-el l1-cl l1-vc base-bootnode base-builder base-client base-rpc \
  base-shadow-validator op-batcher base-batcher prometheus grafana jaeger
bash "$root/state/verify-baseline-devnet.sh" baseline-restart1
old="$root/logs/baseline-verify2/receipt.json"
new="$root/logs/baseline-restart1/receipt.json"
old_tx=$(jq -r .transactionHash "$old")
for port in 7545 8545; do
  file="$root/logs/baseline-restart1/old-receipt-$port.json"
  timeout 20 cast receipt "$old_tx" --rpc-url "http://127.0.0.1:$port" --json > "$file"
  jq -e --slurpfile old "$old" '.status == "0x1" and .transactionHash == $old[0].transactionHash and .blockHash == $old[0].blockHash' "$file"
done
old_block=$(jq -r .blockNumber "$old")
new_block=$(jq -r .blockNumber "$new")
(( new_block > old_block ))
echo 'PERSISTENCE_OLD_RECEIPT_MATCH=true'
echo 'POST_RESTART_CHAIN_PROGRESS=true'
