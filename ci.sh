#!/bin/sh
# CI: 書式・lint・テスト(と、依存を入れたら cargo deny)。
set -e
cd "$(dirname "$0")"
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
if command -v cargo-deny >/dev/null 2>&1; then cargo deny check; fi
# 仕様の検査(錠を含む)。check.py の場所は環境変数 DECIDESPEC_CHECK で渡す。
if [ -n "$DECIDESPEC_CHECK" ]; then python3 "$DECIDESPEC_CHECK" .; fi
