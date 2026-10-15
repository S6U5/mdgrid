#!/bin/sh
# CI: 書式・lint・テスト・依存の検査。
set -e
cd "$(dirname "$0")"
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
# 依存の検査。ライセンス・重複と版の指定・出どころは落とし、勧告(advisories)は今は警告だけ。
command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny --locked
cargo deny check licenses bans sources
cargo deny check advisories || echo "警告: cargo deny の勧告(advisories)に当たりがある。今は止めない。" >&2
# 仕様の検査(錠を含む)。check.py の場所は環境変数 DECIDESPEC_CHECK で渡す。
if [ -n "$DECIDESPEC_CHECK" ]; then python3 "$DECIDESPEC_CHECK" .; fi
