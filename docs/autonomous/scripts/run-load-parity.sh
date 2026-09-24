#!/usr/bin/env bash
# Local test-account traffic only. Do not print environment or keys.
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
flavor="${1:?baseline, unified or wrapper}"
mode="${2:-load}"
suffix="${3:-}"
[[ "$suffix" =~ ^[a-zA-Z0-9-]*$ ]]
run="$root/logs/load-$flavor-$mode${suffix:+-$suffix}"
mkdir "$run"
cd "$root/integration"
set +x
source etc/docker/devnet-env
export FUNDER_KEY="$ANVIL_ACCOUNT_1_KEY"
export LOAD_TEST_OUTPUT="$run/summary.json"
export BASE_NODE_METRICS_ENABLED=false
export RUST_LOG=warn,base_load_tests=info
case "$flavor" in
  baseline) command=("$root/artifacts/baseline/base-load-tester") ;;
  unified) command=("$root/integration/target/debug/base" load-test) ;;
  wrapper) command=("$root/integration/target/debug/base-load-tester") ;;
  *) echo 'Unknown test flavor' >&2; exit 64 ;;
esac
case "$mode" in
  load) args=("$root/state/load-parity.yaml") ;;
  skip-drain) args=("$root/state/load-parity.yaml" --skip-drain) ;;
  drain) args=("$root/state/load-parity.yaml" --drain-only) ;;
  continuous) args=("$root/state/load-parity.yaml" --continuous) ;;
  *) echo 'Unknown test mode' >&2; exit 64 ;;
esac
printf 'LOAD_PARITY_START flavor=%s mode=%s time=%s\n' "$flavor" "$mode" "$(date -u +%FT%TZ)"
if [[ "$mode" == continuous ]]; then
  timeout --preserve-status --signal=INT --kill-after=30 8 "${command[@]}" "${args[@]}" > "$run/output.log" 2>&1
else
timeout 180 "${command[@]}" "${args[@]}" > "$run/output.log" 2>&1
fi
if [[ "$mode" != drain ]]; then
  jq -e '.error == null and .throughput.total_submitted > 0 and .throughput.total_confirmed == .throughput.total_submitted and .throughput.total_failed == 0 and .throughput.total_reverted == 0 and .receipt_coverage.blocks_failed == 0 and .receipt_coverage.transactions_missing == 0' "$run/summary.json"
  jq '{throughput,receipt_coverage,measurement_start_block,measurement_end_block}' "$run/summary.json"
else
  grep -E '^=== Drain-Only Mode ===|^Drained ' "$run/output.log"
fi
printf 'LOAD_PARITY_PASS flavor=%s mode=%s time=%s\n' "$flavor" "$mode" "$(date -u +%FT%TZ)"
