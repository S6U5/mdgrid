---
type: Change
id: 01M4RPEGCGEZD223B77E4SMYXC
title: 「保管庫」をルートフォルダと言い換え、検知の値を目印のフォルダの名前にする
status: done
size: full
created: 2026-10-13
updated: 2026-10-13
---

# 「保管庫」をルートフォルダと言い換え、検知の値を目印のフォルダの名前にする

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [root-folder](../_decisions/2026-10-13-root-folder.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が会話 2026-10-11 で、Markdown のフォルダを扱う道具なのに「保管庫(vault)」という言い方はよくないと言い、2026-10-13 に言い方を「ルートフォルダ」(英語 root folder)に決めた。

終わりの条件:
- 画面・CLI の文言(`--help`・man・設定の説明・警告)と文書・仕様に「保管庫」・語としての vault が残らない(ほかの道具の言い方として説明する所を除く)。
- `[workspace] detect` の値は `".obsidian"`・`".git"`。旧い `"vault"`・`"git"` も同じ意味で読み、移ったと警告する。`--print-config` と文書の既定は `[".obsidian"]`。
- 境界: 旧い値だけの設定でも、検知は前と同じに動く。

関係する要件: WS-5(検知の値)、BV-2・BV-9・BV-12・BV-22・BV-23・CE-2・CE-16・WS-6(言い方だけ)。

## 不明点と仮定

- 仮定: 見本のフォルダの名前(`examples/vault` ほか)とコードの名前(`Detect::Vault`・`mdgrid::vault` など)は変えない。画面と文書の文に出ず、参照と錠のある試験が多い。外れたら: 別の変更で名前を変える。
- 仮定: ほかの道具の言い方を説明する所(docs/obsidian-bases*.md など、その道具の用語として vault と書く所)は残す。
- 仮定: 旧い値の警告は、タブの旧い書き方(CLI-20)と同じ「旧い名前は `…` に移った(そこに書いたものとして読んだ)」。

## 設計

変える場所: `src/workspace.rs`(`Detect::parse` は新しい値 `".obsidian"`・`".git"`、`Detect::parse_old` は旧い値、`name` は新しい値を返す)、`src/config.rs` の `read_workspace`(旧い値なら移ったと警告して読む)、`src/i18n.rs`(`Usage`・`WantDetect`)、`src/main.rs`(clap の long_about)、`src/schema.rs`(既定・例・説明)、`docs/` と README・demos の文書。変えない場所: 見本のフォルダの名前、コードの名前、過去の決定と変更の記録。

判断: 値を目印のフォルダの名前にし、`Detect` の型はそのまま(値と目印の対応は `parse`・`name`・`marker` の3つだけ)。却下: `Detect` を任意の目印の名前の型に作り直す(頼まれていない。公開の型と錠のある試験3本が変わる)。

要件との対応: WS-5 は `workspace.rs`・`config.rs`・`schema.rs`。ほかの要件は文言と文書の言い換えだけ。

検証: `cargo test --test test_root_folder`、`cargo test --no-fail-fast`(裏で)、`python3.12 .decidespec/scripts/check.py .`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 受け入れの試験を置く(新しいファイル) | WS-5, BV-2 | 実装の前に落ちる | `tests/test_root_folder.rs` | test_ws_5_detect_* / test_bv_2_help_says_root_folder | 済 |
| 2 | 検知の値と文言を変える | WS-5, BV-2 | 1 の試験が通る | `src/workspace.rs` `src/config.rs` `src/i18n.rs` `src/main.rs` `src/schema.rs` `docs/config.md` `docs/config.ja.md` | test_ws_5_detect_* / test_bv_2_help_says_root_folder | 済 |
| 3 | 文書の言い換え | BV-2, WS-5 | 文書に「保管庫」・語としての vault が残らない(ほかの道具の用語の説明を除く) | `docs/` `README.md` `README.ja.md` `demos/` `examples/` | 読んで判定 | 済 |
| 4 | 既存の試験を通す | WS-5 | `cargo test` が通る(前からの test_cliff を除く) | `tests/` `src/` | 既存の試験 | 済 |

## 実装の気づき

- `--print-config` の説明に旧い値 `"vault"` を引用する所が残るので、受け入れの試験 test_ws_5_print_config_default_is_obsidian の「vault と書かない」は、引用した `"vault"` を除いて見る形に直した(試験の側が厳しすぎた)。
- 文書の言い換えは 28 ファイル・77 箇所。残したのは、ほかの道具の概念にたとえて案を説明する docs/todo.md の1か所と、パス・画面に出る見本のフォルダの名前(`vault ▾` など)・コードの名前・旧い値の説明。見本の登録の名前(examples/places)は「見本」にした。

## 照合

検証役(フレッシュ文脈のサブエージェント、2026-10-13)が HEAD f36f3af と、試験のコミット c91086c の木(`git archive` で scratchpad に展開し、別の target で実際に走らせた)で確かめた。指摘1・3 は直した(下)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WS-5 | checked | コード: src/workspace.rs:50 `Detect::parse`(目印の名前)・:58 `parse_old`(旧い値)・:67 `name`、src/legacy.rs `detect_values`(旧い値を写して警告)、src/config.rs `read_workspace`、src/schema.rs:164(既定 `[".obsidian"]`)、docs/config.md・docs/config.ja.md の `workspace.detect` / テスト: tests/test_root_folder.rs::test_ws_5_detect_reads_marker_names・test_ws_5_detect_old_names_read_with_warning・test_ws_5_detect_unknown_warns・test_ws_5_print_config_default_is_obsidian・test_ws_5_migrate_config_rewrites_old_values / 今: 通った(`cargo test --test test_root_folder`) / 前: c91086c で落ちた(新しい値が知らない値で警告され、既定が `"vault"`) |
| BV-2 | checked | コード: src/i18n.rs:260 `Usage`(en "the root folder"・ja「ルートフォルダ」)、src/main.rs:126 の long_about / テスト: tests/test_root_folder.rs::test_bv_2_help_says_root_folder / 今: 通った / 前: c91086c で落ちた(「保管庫の根」・"vault root") |
| BV-9 | checked | 読んで判定: specs/base-view/spec.md の BV-9 の行と、画面・CLI の文言(src/i18n.rs・src/main.rs・src/schema.rs)と文書(docs/・README*.md)の grep。言い方だけの変更で、振る舞いは変えていない |
| BV-12 | checked | 読んで判定: BV-12 の行と、同じ grep |
| BV-22 | checked | 読んで判定: BV-22 の行と、同じ grep |
| BV-23 | checked | 読んで判定: BV-23 の行(「Obsidian がノートに数えない隠しフォルダ」)と、同じ grep |
| CE-2 | checked | 読んで判定: CE-2 の行と、同じ grep |
| CE-16 | checked | 読んで判定: CE-16 の例と、同じ grep |
| WS-6 | checked | 読んで判定: WS-6 の例と、同じ grep |

既存のテストの削除・skip・弱体化: なし(試験のファイルで変わったのは新しい tests/test_root_folder.rs だけ。その1つの assert は引用した旧い値 `"vault"` を除いて見る形にした。理由は「実装の気づき」)

確かめた: 9 / 9

指摘と行き先:
- `--migrate-config` が検知の旧い値を書き換えず、写した結果を読み直すと警告が出た(CLI-20 の「読み直すと警告なし」から外れる)。直した: 旧い値の写しを `legacy::lift` に移し、`--migrate-config` も読み込みも同じ所で写す(test_ws_5_migrate_config_rewrites_old_values)。
- 警告の文が「旧い名前は … に移った」で、変わったのは値。タブの旧い書き方と同じ形なので残した。
- コードのコメントに前の言い方が残っていた。直した(コメントの行だけ)。

任意の補足: 残した「保管庫」は docs/todo.md のほかの道具のたとえの1か所。語としての vault は、パス・画面に出る見本のフォルダの名前・コードの名前・旧い値の説明だけ。
