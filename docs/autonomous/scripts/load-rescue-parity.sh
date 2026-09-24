#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
cd "$root/integration"
export PATH="$HOME/.config/.foundry/bin:$PATH"
set +x
source etc/docker/devnet-env
export FUNDER_KEY="$ANVIL_ACCOUNT_1_KEY"
export BASE_NODE_METRICS_ENABLED=false
export RUST_LOG=warn,base_load_tests=info
test "$(cast chain-id --rpc-url http://127.0.0.1:7545)" = 84538453
finish() {
  code=$?
  printf '\nRESCUE_MATRIX_EXIT_CODE=%s\nRESCUE_MATRIX_DONE %s\n' "$code" "$(date -u +%FT%TZ)"
  tmux wait-for -S base-autonomous-rescue-done
}
trap finish EXIT
# Original baseline rescue is a recorded parser failure; test the corrected entrypoints.
for flavor in unified wrapper; do
  run="$root/logs/rescue-$flavor-attempt3"
  mkdir "$run"
  case "$flavor" in
    baseline) command=("$root/artifacts/baseline/base-load-tester") ;;
    unified) command=("$root/integration/target/debug/base" load-test) ;;
    wrapper) command=("$root/integration/target/debug/base-load-tester") ;;
  esac
  export LOAD_TEST_OUTPUT="$run/preparation.json"
  timeout 180 "${command[@]}" "$root/state/rescue-parity.yaml" --skip-drain > "$run/preparation.log" 2>&1
  jq -e '.error == null and .throughput.total_confirmed > 0 and .throughput.total_failed == 0' "$run/preparation.json"
  before=$(cast balance "$ANVIL_ACCOUNT_1_ADDR" --rpc-url http://127.0.0.1:7545)
  timeout 120 "${command[@]}" rescue --rpc-url http://127.0.0.1:7545 --seed 88011 --count 4 > "$run/rescue.log" 2>&1
  after=$(cast balance "$ANVIL_ACCOUNT_1_ADDR" --rpc-url http://127.0.0.1:7545)
  python3 -c 'import sys; assert int(sys.argv[2]) > int(sys.argv[1]), "rescue did not return funds"' "$before" "$after"
  timeout 120 "${command[@]}" rescue --rpc-url http://127.0.0.1:7545 --seed 88011 --count 4 > "$run/rescue-again.log" 2>&1
  again=$(cast balance "$ANVIL_ACCOUNT_1_ADDR" --rpc-url http://127.0.0.1:7545)
  python3 -c 'import sys; assert int(sys.argv[2]) == int(sys.argv[1]), "repeat rescue changed funder balance"' "$after" "$again"
  printf 'RESCUE_PASS flavor=%s before=%s after=%s repeat=%s\n' "$flavor" "$before" "$after" "$again"
done
