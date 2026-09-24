#!/usr/bin/env bash
# Disposable local chain only. Never print private keys or shell environment.
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
repo="$root/integration"
run="${1:-baseline-verify1}"
[[ "$run" =~ ^baseline-[a-zA-Z0-9-]+$ ]]
evidence="$root/logs/$run"
mkdir "$evidence" # Fail rather than silently rebroadcast into an existing run.
cd "$repo"
export PATH="$HOME/.config/.foundry/bin:$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export COMPOSE_PROJECT_NAME=base-autonomous-20260924
set +x
source etc/docker/devnet-env
finish() {
  code=$?
  printf '\nBASELINE_VERIFY_EXIT_CODE=%s\nBASELINE_VERIFY_DONE %s\n' "$code" "$(date -u +%FT%TZ)"
  tmux wait-for -S base-autonomous-verify-done
}
trap finish EXIT
printf 'BASELINE_VERIFY_START %s\n' "$(date -u +%FT%TZ)"
git rev-parse HEAD HEAD^{tree}
test -z "$(git status --porcelain --untracked-files=no)"
# Readiness polling is bounded and never broadcasts transactions.
for port in 7545 8545 8645 8845; do
  ready=0
  for attempt in {1..120}; do
    chain=$(timeout 5 cast chain-id --rpc-url "http://127.0.0.1:$port" 2>/dev/null || true)
    height=$(timeout 5 cast block-number --rpc-url "http://127.0.0.1:$port" 2>/dev/null || true)
    if [[ "$chain" == 84538453 && "$height" =~ ^[0-9]+$ ]] && (( height > 0 )); then
      printf 'RPC_READY port=%s chain=%s height=%s\n' "$port" "$chain" "$height"
      ready=1
      break
    fi
    sleep 2
  done
  test "$ready" = 1 || { echo "RPC readiness failed at port $port"; exit 1; }
done
compose=(docker compose --env-file etc/docker/devnet-env -f etc/docker/docker-compose.yml -f "$root/state/baseline-compose.override.yml")
"${compose[@]}" ps --all --format json > "$evidence/containers.json"
jq -s -e '[.[] | if type == "array" then .[] else . end] | length == 14 and all(.[]; (.Service == "setup-devnet" and .ExitCode == 0) or .State == "running")' "$evidence/containers.json"
# No retry if cast fails or times out after broadcast; inspect this run's files first.
timeout 120 cast send "$ANVIL_ACCOUNT_2_ADDR" --value 0.001ether \
  --private-key "$ANVIL_ACCOUNT_1_KEY" --rpc-url http://127.0.0.1:7545 \
  --chain-id 84538453 --json > "$evidence/receipt.json"
jq -e --arg from "$ANVIL_ACCOUNT_1_ADDR" --arg to "$ANVIL_ACCOUNT_2_ADDR" \
  '.status == "0x1" and (.from|ascii_downcase) == ($from|ascii_downcase) and (.to|ascii_downcase) == ($to|ascii_downcase) and (.transactionHash|test("^0x[0-9a-fA-F]{64}$")) and .blockNumber != null and .blockHash != null' "$evidence/receipt.json"
block=$(jq -r .blockNumber "$evidence/receipt.json")
timeout 20 cast block "$block" --rpc-url http://127.0.0.1:7545 --json > "$evidence/block.json"
jq -e --slurpfile receipt "$evidence/receipt.json" \
  '.hash == $receipt[0].blockHash and (.transactions | map(if type == "object" then .hash else . end) | index($receipt[0].transactionHash) != null)' "$evidence/block.json"
# Verify the same transaction reaches the independent validator, not just the sequencer.
tx=$(jq -r .transactionHash "$evidence/receipt.json")
seen=0
for attempt in {1..60}; do
  if timeout 5 cast receipt "$tx" --rpc-url http://127.0.0.1:8545 --json > "$evidence/validator-receipt.json" 2>/dev/null && \
     jq -e --slurpfile receipt "$evidence/receipt.json" '.status == "0x1" and .blockHash == $receipt[0].blockHash and .transactionHash == $receipt[0].transactionHash' "$evidence/validator-receipt.json" >/dev/null; then
    seen=1
    break
  fi
  sleep 2
done
test "$seen" = 1
./etc/scripts/devnet/status.sh > "$evidence/status.txt"
jq '{transactionHash,status,from,to,blockNumber,blockHash}' "$evidence/receipt.json"
printf 'VALIDATOR_RECEIPT_MATCH=true\n'
