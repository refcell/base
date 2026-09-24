#!/usr/bin/env bash
# Runtime-only orchestration. Run on gene; no tracked source is changed.
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
repo="$root/integration"
cd "$repo"
export PATH="$HOME/.local/share/mise/installs/go/1.26.1/bin:$HOME/.cargo/bin:$HOME/.config/.foundry/bin:$HOME/.local/bin:$PATH"
export COMPOSE_PROJECT_NAME=base-autonomous-20260924
export GOMAXPROCS=8
export CARGO_BUILD_JOBS=8
export PROFILE=dev
export BASE_SUCCINCT_ELF_REQUIRE=0
finish() {
  code=$?
  printf '\nBASELINE_IMAGES_EXIT_CODE=%s\nBASELINE_IMAGES_DONE %s\n' "$code" "$(date -u +%FT%TZ)"
  tmux wait-for -S base-autonomous-images-done
}
trap finish EXIT
printf 'BASELINE_IMAGES_START %s\n' "$(date -u +%FT%TZ)"
test "$(git rev-parse HEAD)" = 539605ce58aaf02fe5c0382fcd9139c1ee4207e9
test -z "$(git status --porcelain --untracked-files=no)"
grep '^BASELINE_TESTS_EXIT_CODE=' "$root/logs/baseline-tests-attempt1.log"
# Original recipe's setup Dockerfile, with only an experiment-owned image tag.
docker buildx build --load --progress plain \
  -f etc/docker/Dockerfile.devnet \
  -t devnet-setup:base-autonomous-20260924 .
# Sequential targets prevent the Go and Rust builds competing for resources.
docker buildx bake -f etc/docker/docker-bake.hcl op-batcher --load --progress plain \
  --set op-batcher.tags=op-batcher:base-autonomous-20260924
docker buildx bake -f etc/docker/docker-bake.hcl base --load --progress plain \
  --set base.tags=base:base-autonomous-20260924 \
  --set 'base.args.CARGO_FEATURES=--jobs 8' \
  --set base.args.SCCACHE_CACHE_ID=base-autonomous-20260924-sccache
# Record image identity without dumping environment variables or credentials.
docker image inspect --format '{{.Id}} {{join .RepoTags " "}}' \
  devnet-setup:base-autonomous-20260924 \
  op-batcher:base-autonomous-20260924 \
  base:base-autonomous-20260924
