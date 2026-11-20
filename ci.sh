#!/bin/sh
# CI: 書式・lint・テスト・依存の検査。
set -e
cd "$(dirname "$0")"
# 道具の版は flake.lock でそろえる(CI と同じ clippy・rustfmt・lint の道具)。nix develop の中でなければ入り直す。
if [ -z "${IN_NIX_SHELL:-}" ] && command -v nix >/dev/null 2>&1; then
  exec nix develop -c ./ci.sh "$@"
fi
cargo fmt --check
cargo clippy --all-targets -- -D warnings
# Python・シェル・GitHub Actions の lint と書式(道具は nix develop で入る)。
sh scripts/lint.sh
# テスト。cargo-nextest があれば、ファイルをまたいで並べて走らせ、落ちても残りを全部走らせる(CI と同じ道具)。
# nextest は doc の試験を走らせないので、それだけ cargo test --doc で回す。無ければ cargo test。
if command -v cargo-nextest >/dev/null 2>&1; then
  cargo nextest run --no-fail-fast
  cargo test --doc
else
  cargo test
fi
# 依存の検査。ライセンス・重複と版の指定・出どころは落とし、勧告(advisories)は今は警告だけ。
command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny --locked
cargo deny check licenses bans sources
cargo deny check advisories || echo "警告: cargo deny の勧告(advisories)に当たりがある。今は止めない。" >&2
# 要求とテストの印の検査(trace-gate。要求の正本は specs/requirements/、設定は trace-gate.json)。
python3 tools/trace_gate.py check
