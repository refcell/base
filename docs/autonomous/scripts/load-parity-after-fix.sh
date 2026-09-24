#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
finish() {
  code=$?
  printf '\nLOAD_AFTER_FIX_EXIT_CODE=%s\nLOAD_AFTER_FIX_DONE %s\n' "$code" "$(date -u +%FT%TZ)"
  tmux wait-for -S base-autonomous-load-after-fix-done
}
trap finish EXIT
grep -q '^LOAD_CANDIDATE_EXIT_CODE=0$' "$root/logs/load-candidate-build3.log"
grep -q '^RESCUE_MATRIX_EXIT_CODE=0$' "$root/logs/load-rescue-parity3.log"
for flavor in unified wrapper; do
  for mode in load skip-drain drain continuous; do
    bash "$root/state/run-load-parity.sh" "$flavor" "$mode" after-fix
  done
done
