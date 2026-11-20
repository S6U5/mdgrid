#!/bin/sh
# Rust 以外の lint と書式を確かめる: Python(ruff)・シェル(shellcheck・shfmt)・GitHub Actions(actionlint)。
# ci.sh と CI の lint-scripts のジョブが回す。道具は `nix develop`(flake.nix の devShells)で入る。
#   sh scripts/lint.sh         確かめるだけ
#   sh scripts/lint.sh --fix   直せるものを直す(ruff の直しと書式、shfmt の書式)
set -eu
cd "$(dirname "$0")/.."

missing=""
for tool in ruff shellcheck shfmt actionlint; do
  command -v "$tool" >/dev/null 2>&1 || missing="$missing $tool"
done
if [ -n "$missing" ]; then
  echo "lint の道具が無い:${missing}。nix develop の中で回す(flake.nix の devShells)。" >&2
  exit 1
fi

# シェルの書式: 字下げは2文字、case の中も字下げする。
shells="ci.sh demos/*.sh scripts/*.sh"
if [ "${1:-}" = "--fix" ]; then
  ruff check --fix .
  ruff format .
  # shellcheck disable=SC2086 # ファイルの並びをわざと語に分ける。
  shfmt -i 2 -ci -w $shells
fi

ruff check .
ruff format --check .
# shellcheck disable=SC2086
shellcheck $shells
# shellcheck disable=SC2086
shfmt -i 2 -ci -d $shells
actionlint
echo "lint ok(Python・シェル・Actions)"
