#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
finish() {
  code=$?
  printf '\nLOAD_MATRIX_EXIT_CODE=%s\nLOAD_MATRIX_DONE %s\n' "$code" "$(date -u +%FT%TZ)"
  tmux wait-for -S base-autonomous-load-matrix-done
}
trap finish EXIT
grep -q '^LOAD_CANDIDATE_EXIT_CODE=0$' "$root/logs/load-candidate-build2.log"
# Baseline normal load already passed; never silently overwrite/rebroadcast that run.
for flavor in unified wrapper; do
  bash "$root/state/run-load-parity.sh" "$flavor" load
done
for flavor in baseline unified wrapper; do
  bash "$root/state/run-load-parity.sh" "$flavor" skip-drain
  bash "$root/state/run-load-parity.sh" "$flavor" drain
  bash "$root/state/run-load-parity.sh" "$flavor" continuous
done
