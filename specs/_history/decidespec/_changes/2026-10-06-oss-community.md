---
type: Change
id: 01M46ME1AQ2HT97YD10JHW4Y65
title: 貢献の手引き・報告の窓口・テンプレートと、パッケージの情報(oss-community)
status: done
size: tiny
created: 2026-10-06
updated: 2026-10-06
---

# 貢献の手引き・報告の窓口・テンプレートと、パッケージの情報(oss-community)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 製品の振る舞いを変えない(貢献の文書とパッケージの情報) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 で、人が OSS として多くの人に使われる水準まで改善を繰り返すよう頼み、判断を AI に委ねた。docs/todo.md の O-8(CONTRIBUTING・CODE_OF_CONDUCT・SECURITY)と O-4(Cargo.toml の情報)の、リポの URL が無くてもできる部分を入れる。

終わりの条件: CONTRIBUTING.md・SECURITY.md・CODE_OF_CONDUCT.md・Issue と PR のテンプレートがあり、`cargo package --list` に src と README と使用許諾だけが入る。関係する要件: 該当なし。

## 不明点と仮定

- 仮定(O-8 の報告先の判断を委ねられた): 脆弱性の報告は GitHub の Private vulnerability reporting にする。個人のメールアドレスは公開の文書に書かない。
- 仮定: 行動規範は Contributor Covenant 2.1 を採り、本文は写さず公式の文へのリンクと、報告の窓口(リポの持ち主へ GitHub で)だけを書く。
- 仮定(O-4 の crates.io に出すかの判断): 出す前提で keywords・categories・readme・include を入れるが、`publish = false` と `repository` は、リポを公開して URL が決まるまで今のまま(O-10 のあと)。
- 仮定: 使われていない docs/assets/demo-table.svg を消す(README の先頭は demo.gif になった)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 貢献の文書・テンプレート・パッケージの情報 | なし(貢献の文書とパッケージの情報) | 文書とテンプレートがあり、パッケージに要るものだけが入る | CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md, .github/ISSUE_TEMPLATE/, .github/pull_request_template.md, Cargo.toml, docs/assets/, README.md, docs/todo.md | 読んで判定(`cargo package --list`) | 済 |

## 実装の気づき

- Cargo の `include` の型は gitignore と同じで、`README.md` は下のフォルダの README.md にも当たる。先頭に `/` を付けて根だけにした。

## 照合

自分で照合(フレッシュ文脈でない。要件の無い貢献の文書とパッケージの情報のため)。

| 要件 | 結果 | 証拠 |
|---|---|---|

要件の無いタスク1の照合: checked。読んで判定: Issue のテンプレート3つを ruby の YAML で読めた。`cargo package --list` が src の .rs と Cargo.toml・Cargo.lock・README.md・使用許諾の2つだけ(106ファイル、圧縮で 454KiB)。`cargo package` の検証のビルドが通った。CONTRIBUTING の `mdgrid --version`・`./ci.sh`・Rust 1.90・試験の言語の固定が今のリポと合うことを確かめた。公開の文書に個人のメールアドレスを書いていない。

既存のテストの削除・skip・弱体化: なし(試験に触れていない)

確かめた: 0 / 0(要件の無い変更。タスク1は上の文のとおり checked)
