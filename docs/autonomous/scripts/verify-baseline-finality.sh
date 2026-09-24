#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
export PATH="$HOME/.config/.foundry/bin:$PATH"
cd "$root/integration"
out="$root/logs/baseline-finality1"
mkdir "$out"
receipt="$root/logs/baseline-verify2/receipt.json"
block=$(jq -r .blockNumber "$receipt")
for pair in '7549 7545' '8549 8545'; do
  read -r consensus execution <<< "$pair"
  status="$out/status-$consensus.json"
  curl --fail --silent --show-error --max-time 10 "http://127.0.0.1:$consensus" \
    -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"2.0","method":"optimism_syncStatus","params":[],"id":1}' > "$status"
  jq -e --argjson included "$((block))" '.error == null and .result.finalized_l2.number >= $included and .result.unsafe_l2.number >= .result.safe_l2.number and .result.safe_l2.number >= .result.finalized_l2.number' "$status"
  finalized=$(jq -r .result.finalized_l2.number "$status")
  origin=$(jq -r .result.finalized_l2.l1origin.number "$status")
  timeout 15 cast block "$finalized" --rpc-url "http://127.0.0.1:$execution" --json > "$out/finalized-$execution.json"
  jq -e --slurpfile status "$status" '.hash == $status[0].result.finalized_l2.hash' "$out/finalized-$execution.json"
  timeout 15 cast block "$origin" --rpc-url http://127.0.0.1:4545 --json > "$out/origin-$execution.json"
  jq -e --slurpfile status "$status" '.hash == $status[0].result.finalized_l2.l1origin.hash' "$out/origin-$execution.json"
  timeout 15 cast block "$block" --rpc-url "http://127.0.0.1:$execution" --json > "$out/included-$execution.json"
  jq -e --slurpfile receipt "$receipt" '.hash == $receipt[0].blockHash and (.transactions | map(if type == "object" then .hash else . end) | index($receipt[0].transactionHash) != null)' "$out/included-$execution.json"
done
l1_finalized=$(timeout 15 cast block finalized --field number --rpc-url http://127.0.0.1:4545)
for port in 7549 8549; do
  jq -e --argjson l1 "$l1_finalized" '.result.finalized_l2.l1origin.number <= $l1' "$out/status-$port.json"
done
printf 'BASELINE_FINALITY_PASS %s L1_FINALIZED=%s\n' "$(date -u +%FT%TZ)" "$l1_finalized"
