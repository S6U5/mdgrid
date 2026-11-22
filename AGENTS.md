# mdgrid

このリポの要求の正本は `specs/requirements/`(能力ごとに1ファイル。地図は `specs/requirements/README.md`)。作業は要求に従う。要求とテストは trace-gate の形で持つ(下の「要求とテスト」)。2026-11-17 までの decidespec の仕様・変更の記録・決定の記録は `specs/_history/decidespec/` に凍結してあり、書き換えない。コードやコメントに残る旧い ID(`SR-38` など)は、`specs/requirements/renames.md` で新しい名前に引く。

設計の判断(how)は `docs/design.md` に置く。要求(what)と重ねて書かない。

説明書・サイト・README の文は `docs/writing.md` の決まり(読む人の順に並べる、人が読んで温度のある文にする)で書き、書いたあとに読み直す。画面の型や見せ方を足したら、説明書の「見せ方の一覧」(`docs/manual/<言語>/views.md`)に、機能を足したら「できることの一覧」(`docs/manual/<言語>/features.md`)に、画面1枚と1項を足す。

本物の実行ファイルの画面を確かめる試験(e2e)は、疑似端末の道具 `tests/pty/mod.rs`(portable-pty と vt100。起動・キーを送る・画面を待つ・Drop で必ず止める)を `use crate::pty;` で使い(`mod pty;` は束ねる `tests/all.rs` に1回だけ書く)、`cargo test` の中で回す(例: `tests/test_e2e.rs`・`tests/test_pick.rs`)。Microsoft の tui-test は Node が要り `cargo test` で回せないので使わない(specs/_history/decidespec/_changes/2026-10-03-e2e.md)。結合の試験は `tests/all.rs` が `#[path]` で1つの実行ファイルに束ねる(`Cargo.toml` は `autotests = false`)。新しい試験のファイルを足したら `tests/all.rs` に `#[path]` の2行を足し、環境変数や今のフォルダを書き換える試験は別の `[[test]]` にする(理由は docs/design.md「試験の実行ファイル」)。試験の環境の言語は `.cargo/config.toml` の [env] で日本語に固定してある。英語の画面を確かめるときは子に `LANG=en_US.UTF-8` を明示し、`LC_ALL`・`LC_MESSAGES` を外す。

手元のリポ(作業の正本。履歴を公開しない)と公開リポ S6U5/mdgrid は履歴が分かれている。手元から公開リポへの書き出しと、公開リポだけで入った変更(クラウドのセッションの PR・Dependabot)の取り込みは、`scripts/public_sync.py` で `docs/public-sync.md` の手順どおりにする。手元のブランチを公開リポへ直に push しない。公開リポの側の作業は、ふだんどおりブランチと PR でよい。

## コミットと push の前に

- コミットの題は `cliff.toml` の `commit_parsers` の書き出しで始める(`追加:`・`改善:`・`修正:`・`文書:`・`specs:`・`試験:`・`設定:` など)。`tests/test_cliff.rs` が履歴の全部のコミットの題を確かめるので、決まった書き出しでない題が1つでも入ると、CI のテストが落ちる。初めてコミットする前に `git log --oneline -10` で今の書き方を見る。
- 題の検査は履歴を読むので、コミットする前に回しても引っかからない。コミットしたあと、push の前に `cargo test --test all test_cliff` を回す(結合の試験は束ねてあるので、ファイルの名前で絞る)。push したあとに題を直すには履歴の書き換え(強制 push)が要り、人の許可なしにはできない。
- push の前に `git fetch origin main` で main が進んでいないかを見て、進んでいればマージで取り込む(rebase と強制 push はしない)。取り込まないと、CI の `weaken` が main の側の変更を「このブランチが戻した」と数える。
- 手元の Stop のフックが回すのは `tools/trace_gate.py check`(要求の書式とテストの印)だけ。fmt・clippy・テストと、Python・シェル・Actions の lint(`scripts/lint.sh`)は `./ci.sh` で回す。道具は `nix develop`(`flake.nix` の devShells)で入る。lint の決まり(clippy の pedantic と外した理由など)は docs/design.md「lint と書式」。次の検査は CI でだけ回るので、当たりそうな変更をしたら push の前に手で回す。
  - `python3 tools/trace_gate.py weaken --base origin/main --head HEAD`: 足した・変えた要求と、減らしたテストを並べる。要求を足した PR では必ず当たりが出て、人が見る合図になる(直す対象ではない)。
  - main のテストを今のコードで回す検査(`reverse-run`): テストが直に組み立てている構造体(`Rule`・`Vars` など)に項目を足すと、main のテストがコンパイルできなくなって落ちる。前からあるテストの期待を変える変更(未対応だった関数を対応にする、など)も、ここで人に回る。項目は構造体の外に持つか、前のテストの期待を変えることを PR の本文に書いて人に問う。

## 要求とテスト(trace-gate)

- 要求とテストの期待値は人が決める。AI は案を PR に出し、決まっていないことは `tbd~` の行に残す(黙って埋めると人が気づけないため)。
- 期待値は仕様と受け入れの例から取る(実装から取ると、今の誤りを正しいと固めるため)。
- テストが落ちたら実装を直す。仕様の読みが割れたとき(仕様どうし・仕様とテスト・例と要求文)は、どれも直さずに止め、場所と両方の読みと選べる直し方を書いて、人への問いで終える。
- 要求は write-requirement、テストは write-tests のスキルで書く(正本 `specs/requirements/<能力>.md`、印 `[utest -> req~名前~版]`)。
- 終える前に `python3 tools/trace_gate.py check` を走らせる(0 = 通過。誤りには直し方の help が付く。合否は CI が決める)。

このリポに固有の決まり:

- Rust のテストの印は、テストの関数の直前か本体の中のコメントに `// [utest -> req~名前~版]` と書く。本物の実行ファイルを動かすテスト(`CARGO_BIN_EXE_mdgrid` か疑似端末の道具 `pty` を使うファイル)は `stest` にする。
- このリポは trace-gate を CI に載せる設定にした(`trace-gate.json` の `parts.ci` が true。`.github/workflows/trace-gate.yml` は forge-gate が作る写しで手で直さず、`forge-gate.json` を直して `forge_gate.py init` を走らせる)。ほかの CI(ci.yml など)はそのまま別にある。手元でも check を見る(Stop のフックと `./ci.sh` の最後)。要求は 1 要求 1 文・受け入れ 1 つの見出しの区切りで書く。
- 断言をヘルパーの関数に任せたテストは、今の検査器が断言と数えない(`tbd~rust-helper-asserts`)。そういうテストには印を付けず、コメントに要求の名前だけを `[base-view.expr-eval]` の形で書く。
- 検査器の写し `tools/trace_gate.py` と写しの錠 `tools/trace-gate.lock.json`、承認の記録 `specs/approvals.jsonl` は人だけが書く。更新はプラグインから人が `init --force` で取り込む。
