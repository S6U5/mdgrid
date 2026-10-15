---
type: Change
id: 01M4YT5RCXQ9D6J6CGVT592DFR
title: 疑似端末の試験を Windows で組み立てない(pty-tests-unix)
status: done
size: tiny
created: 2026-10-15
updated: 2026-10-15
---

# 疑似端末の試験を Windows で組み立てない(pty-tests-unix)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 条件に当たらない | なし |
| 仕様化 | 対象外: 要件の意味を変えない(試験を組み立てる OS の条件だけ) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

公開リポの PR #42 の CI で、windows-latest の試験が組み立てで落ちた(`tests/test_marker_base_name.rs` の `use pty::…` が解決できない)。疑似端末の道具 `tests/pty/mod.rs` は `#![cfg(unix)]` なので、それを使う試験のファイルも `#![cfg(unix)]` にする決まり(既存の `tests/test_e2e.rs` など)を、新しく足された4つのファイルにも当てる。関係する要件: 該当なし(試験の中身は変えない)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | pty を使う試験のファイルの頭に `#![cfg(unix)]` | なし | `mod pty;` を持つ追跡中の試験のファイルがすべて `#![cfg(unix)]` で始まる | tests/test_marker_base_name.rs, tests/test_marker_base_switch.rs, tests/test_where_formula.rs, tests/test_views_skill.rs | 読んで判定 | 済 |

## 実装の気づき

tests/test_marker_base_switch.rs は錠があり、ファイルの頭の属性も指紋に入る(check.py の E19)。決定 specs/_decisions/2026-10-15-pty-tests-unix-locks.md(AI 自己採択)で錠を掛け直した。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|

要件の無いタスク1の照合: checked。読んで判定: 4つのファイルの1行目が `#![cfg(unix)]`。macOS で 4つの試験のファイルが今までどおり通る。Windows では公開リポの CI で確かめる。

既存のテストの削除・skip・弱体化: Windows ではこの4つのファイルの試験を組み立てない(疑似端末の道具がもともと unix だけのため。macOS と ubuntu では今までどおり回る)

確かめた: 0 / 0(要件の無い変更)
