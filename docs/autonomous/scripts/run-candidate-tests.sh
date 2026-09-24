#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
cd "$root/worktrees/load-test"
export PATH="$HOME/.local/share/mise/installs/go/1.26.1/bin:$HOME/.cargo/bin:$HOME/.config/.foundry/bin:$HOME/.local/bin:$PATH"
export CARGO_BUILD_JOBS=8
export CARGO_TARGET_DIR="$root/integration/target"
export BASE_SUCCINCT_ELF_STUB=1
finish() {
  code=$?
  printf '\nCANDIDATE_TESTS_EXIT_CODE=%s\nCANDIDATE_TESTS_DONE %s\n' "$code" "$(date -u +%FT%TZ)"
  tmux wait-for -S base-autonomous-candidate-tests-done
}
trap finish EXIT
test "$(git rev-parse HEAD)" = 450b4d7b60a74194c4196283b85d500e1392f946
git rev-parse HEAD HEAD^{tree}
(cd crates/utilities/test-utils/contracts && forge soldeer install && forge build)
test -z "$(git status --porcelain --untracked-files=no)"
cargo nextest run --workspace --all-features --exclude base-system-tests --no-fail-fast --locked --test-threads 8
