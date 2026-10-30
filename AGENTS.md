# mdgrid

このリポの仕様の正本は `specs/`(入口: `specs/README.md`)。作業は仕様に従う。仕様を足す・変える・消すときだけ、decidespec の spec-change で提案から入れる。

設計の判断(how)は `docs/design.md` に置く。仕様(what)と重ねて書かない。

説明書・サイト・README の文は `docs/writing.md` の決まり(読む人の順に並べる、人が読んで温度のある文にする)で書き、書いたあとに読み直す。画面の型や見せ方を足したら、説明書の「見せ方の一覧」(`docs/manual/<言語>/views.md`)に画面1枚と1項を足す。

守られる要件の試験のファイルには錠がある(`specs/test-locks.json`)。錠のあるファイルを書き換えない。新しい試験は新しいファイルに置く。人の決定で要件を変えたときだけ、その決定記録で `check.py --lock --relock <記録>` を使って掛け直す。

本物の実行ファイルの画面を確かめる試験(e2e)は、疑似端末の道具 `tests/pty/mod.rs`(portable-pty と vt100。起動・キーを送る・画面を待つ・Drop で必ず止める)を `mod pty;` で使い、`cargo test` の中で回す(例: `tests/test_e2e.rs`・`tests/test_pick.rs`)。Microsoft の tui-test は Node が要り `cargo test` で回せないので使わない(specs/_changes/2026-10-03-e2e.md)。試験の環境の言語は `.cargo/config.toml` の [env] で日本語に固定してある。英語の画面を確かめるときは子に `LANG=en_US.UTF-8` を明示し、`LC_ALL`・`LC_MESSAGES` を外す。

この仕様駆動の手順の本体は `.decidespec/skills/`(コードを変える依頼は `.decidespec/skills/sdd-flow/SKILL.md`、仕様を変えるときは `.decidespec/skills/spec-change/SKILL.md`)。道具の更新は spec-init で行い、`.decidespec/` の中は手で直さない。

手元のリポ(作業の正本。履歴を公開しない)と公開リポ S6U5/mdgrid は履歴が分かれている。手元から公開リポへの書き出しと、公開リポだけで入った変更(クラウドのセッションの PR・Dependabot)の取り込みは、`scripts/public_sync.py` で `docs/public-sync.md` の手順どおりにする。手元のブランチを公開リポへ直に push しない。公開リポの側の作業は、ふだんどおりブランチと PR でよい。
