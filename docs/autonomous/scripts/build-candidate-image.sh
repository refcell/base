#!/usr/bin/env bash
set -euo pipefail
root=/home/refcell/base-autonomous-20260924
cd "$root/worktrees/load-test"
expected=450b4d7b60a74194c4196283b85d500e1392f946
test "$(git rev-parse HEAD)" = "$expected"
test -z "$(git status --porcelain)"
finish() {
  code=$?
  printf '\nCANDIDATE_IMAGE_EXIT_CODE=%s\nCANDIDATE_IMAGE_DONE %s\n' "$code" "$(date -u +%FT%TZ)"
  tmux wait-for -S base-autonomous-candidate-image-done
}
trap finish EXIT
export PROFILE=dev
export BASE_SUCCINCT_ELF_REQUIRE=0
printf 'CANDIDATE_IMAGE_START %s\n' "$(date -u +%FT%TZ)"
git rev-parse HEAD HEAD^{tree}
docker buildx bake -f etc/docker/docker-bake.hcl base --load --progress plain \
  --set base.tags=base:base-autonomous-load-20260924 \
  --set 'base.args.CARGO_FEATURES=--jobs 8 --locked' \
  --set base.args.SCCACHE_CACHE_ID=base-autonomous-20260924-sccache \
  --set "base.labels.org.opencontainers.image.revision=$expected"
docker image inspect --format '{{.Id}} {{index .Config.Labels "org.opencontainers.image.revision"}}' \
  base:base-autonomous-load-20260924
