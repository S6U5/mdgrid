---
type: Change
id: 01M4VVCFRFR64CVK70TKJGKP2D
title: CI の lint(cargo fmt --check と clippy -D warnings)を通す
status: done
size: tiny
created: 2026-10-14
updated: 2026-10-14
---

# CI の lint(cargo fmt --check と clippy -D warnings)を通す

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: tiny(整形と lint の直しだけ) | |
| 仕様化 | 対象外: tiny(要件の意味を変えない) | |
| 設計 | 対象外: tiny | |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

公開リポの CI の lint が落ちる。cargo fmt --check は src/ui/test_view_bar_cases.rs の1つの assert! の折り返しで、cargo clippy --all-targets -- -D warnings は次の4か所で落ちる。

- src/ui/rows.rs:56 type_complexity(戻り値の型が込み入る)
- src/ui/view.rs:892 question_mark(let…else で None を返す)
- src/ui/test_filter_window_more.rs:5 unused_imports(`use super::*;`)
- tests/test_root_folder.rs:100 bool_assert_comparison(`assert_eq!(…, true)`)

錠の指紋はコメントと空白を除いて取る(TL-5)ので、整形では錠は変わらない。錠の指紋はファイルの共有部分(use など)も含むので、錠のある test_filter_window_more.rs と test_root_folder.rs は書き換えず、錠の無い側で抑える(mod の宣言に allow、Cargo.toml の [lints.clippy])。振る舞いは変えない。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | rustfmt で整形する | なし(整形) | cargo fmt --check が通り、check.py に E19 が出ない | `src/ui/test_view_bar_cases.rs` | src/ui/test_view_bar_cases.rs の試験 | 済 |
| 2 | clippy の4か所を、錠の無いファイルで直す・抑える | なし(lint) | cargo clippy --all-targets --locked -- -D warnings が通り、全部の試験が通り、check.py に E19 が出ない | `src/ui/rows.rs`, `src/ui/view.rs`, `src/ui/mod.rs`, `Cargo.toml` | 全部の試験 | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。tiny で、振る舞いを変えないため検証役は起動しない)。

| 要件 | 結果 | 証拠 |
|---|---|---|

既存のテストの削除・skip・弱体化: なし(test_view_bar_cases.rs は空白と改行だけ。錠のある2つのファイルは書き換えていない)

確かめた: 0 / 0。`cargo fmt --check` と `cargo clippy --all-targets --locked -- -D warnings` が通り、`cargo test` は 1839 本が通って落ちは 0 本、check.py に E19 が出ない。
