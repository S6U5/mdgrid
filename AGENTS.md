# mdgrid

このリポの仕様の正本は `specs/`(入口: `specs/README.md`)。作業は仕様に従う。仕様を足す・変える・消すときだけ、decidespec の spec-change で提案から入れる。

設計の判断(how)は `docs/design.md` に置く。仕様(what)と重ねて書かない。

守られる要件の試験のファイルには錠がある(`specs/test-locks.json`)。錠のあるファイルを書き換えない。新しい試験は新しいファイルに置く。人の決定で要件を変えたときだけ、その決定記録で `check.py --lock --relock <記録>` を使って掛け直す。

本物の実行ファイルの画面を確かめる試験(e2e)は、疑似端末の道具 `tests/pty/mod.rs`(portable-pty と vt100。起動・キーを送る・画面を待つ・Drop で必ず止める)を `mod pty;` で使い、`cargo test` の中で回す(例: `tests/test_e2e.rs`・`tests/test_pick.rs`)。Microsoft の tui-test は Node が要り `cargo test` で回せないので使わない(specs/_changes/2026-10-03-e2e.md)。試験の環境の言語は `.cargo/config.toml` の [env] で日本語に固定してある。英語の画面を確かめるときは子に `LANG=en_US.UTF-8` を明示し、`LC_ALL`・`LC_MESSAGES` を外す。
