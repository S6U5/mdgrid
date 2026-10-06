---
type: Change
id: 01M47HZ5MAE92NQHA804J9X4HS
title: types.json の型を大文字小文字を問わずに引く(types-case)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# types.json の型を大文字小文字を問わずに引く(types-case)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: CE-2 の「保管庫の型の設定に従う」を Obsidian と同じ引き方にする不具合の直し | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

5回目の点検(docs/todo.md の C-6)。`.obsidian/types.json` に `"rating": "number"` があっても、`Rating: high` の列に型が効かない(`!` が付かない)。Obsidian のプロパティの名前は大文字小文字を区別しない。

終わりの条件: 列の型は、まず同じ名前、無ければ大文字小文字を問わずに types.json から引く。

## 不明点と仮定

- 仮定: `Status` と `status` の2つの列を1つにまとめるか(点検の後半)は、表の列の作り(BV-1)を変える仕様の問いなので、この記録では扱わない。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 型を問わずに引く | CE-2 | 上の終わりの条件 | src/source/markdown.rs, src/source/test_types_case_unit.rs | test_ce_2_types_json_any_case(src/source/test_types_case_unit.rs) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(引き方を1か所にまとめるだけ)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-2 | checked | コード: src/source/markdown.rs の declared_of(kind・typed・declared_lock が使う) / テスト: src/source/test_types_case_unit.rs::test_ce_2_types_json_any_case / 今: 通った(全体 1294 passed) / 前: 落ちた |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
