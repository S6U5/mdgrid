---
type: Change
id: 01M46Y28PHFZQXAMPDWXNHF5AS
title: リストの列の1つの値を、式では1つの要素のリストにする(scalar-list-expr)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# リストの列の1つの値を、式では1つの要素のリストにする(scalar-list-expr)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 不具合の直し(BV-6 の式は Obsidian と同じ結果のはずで、Obsidian はリストの型の1つの値を1つの要素のリストとして扱う) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、Obsidian を使う人の目の点検(docs/todo.md の O-3)が見つけた。types.json で tags がリストの型の保管庫で `tags: project` のノートは、式で `tags.length` が 7(文字の数)、`tags.contains("proj")` が true(文字の一部の一致)になる。Obsidian は1つの要素のリストとして扱うので、絞り込みが Obsidian と違う行を出す。

終わりの条件: 列の型がリスト(CE-2)の列で、1つの値(空でない文字・数・真偽)で書かれたセルは、式では1つの要素のリストになる(`tags.contains("proj")` は false、`tags.contains("project")` は true、`tags.length` は 1)。画面の見せ方と書き戻し(読むだけの理由 CE-16 の1つの値のリスト)は変えない。関係する要件: BV-6・CE-2・CE-16。

## 不明点と仮定

- 仮定: 式に値を渡す道(`--print` の prop、画面の .base・mdgrid のビューの組み立ての prop)だけで包む。設定の絞り込み(NV-14)の値の比べ方は変えない。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 1つの値のリストを式で包む | BV-6 | 上の終わりの条件のとおり | src/source.rs, src/base.rs, tests/test_scalar_list_expr.rs | test_bv_6_scalar_list_*(tests/test_scalar_list_expr.rs) | 済 |

## 実装の気づき

- 画面の .base の普通の列(Prop)の見せ方も同じ prop を通るので、包むのは式を評価するところ(src/base.rs の Ctx を作る2か所)にした。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。小さな不具合の直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-6 | checked | コード: src/source.rs の expr_value、src/base.rs の Ctx を作る2か所 / テスト: tests/test_scalar_list_expr.rs::test_bv_6_scalar_list_contains_is_element_match / 今: 通った(全体 1141 passed) / 前: 落ちた(実装の前に回し、`tags.contains("proj")` に a.md が `a.md,7` で残った) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
