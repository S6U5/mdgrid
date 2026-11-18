---
type: Change
id: 01M4JBQ6PKB3WW9ZV3F1C2CTHE
title: 点検で見つかった不具合の直し(2): 親子の表の留めた印、ためた値の形、英語の切れる文言
status: done
size: bugfix
created: 2026-10-10
updated: 2026-10-10
---

# 点検で見つかった不具合の直し(2)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 原因が分かっている | — |
| 仕様化 | 対象外: NV-12・NV-27 どおりに動かない不具合 | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

点検で見つかった次の3つを直す(ほかに、ためた真偽とリストが生の形で出る(SR-35)、英語の「WBS (numbers and pr…」「Parent/c…」「Fold or unfold the chil…」が切れる(SR-23))。


点検(docs/todo.md の F-10)で、親子で並べた表で子を持つ行のどのセルを直しても、並びが変わっていないのに「~(ここに無い行)」の印が付き、u で戻しても残ると分かった(NV-12 は本来の位置と違う行にだけ印を付ける)。

## 設計

- 直した行の留め(keep_stay)は、前の表の位置と組み立てたままの位置を比べる。前の表は親子の並び、組み立てたままは親子にする前の並びで比べていたので、子を持つ行がいつもずれて見えた。親子の並べ直しを overlay の中の留めの前に移し、両方を親子の並びで比べる。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 親子の並べ直しを留めの前に | NV-12 | 子を持つ行を直しても ~ が付かない | src/ui/grid.rs | test_nv_12_tree_parent_edit_not_held(src/ui/test_tree_held.rs) | 済 |
| 2 | ためた真偽とリストを部品の形で | SR-35 | `*true`・`*[a, b]` と出ない | src/ui/cell.rs | test_sr_35_pending_bool_and_list_use_parts(src/ui/test_pending_shape.rs) | 済 |
| 3 | 英語の文言が切れない長さに | SR-23 | WBS・Tree・Fold child rows | src/i18n.rs, docs/manual/en/tasks.md | 読んで判定 | 済 |

## 実装の気づき

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-12 | checked | テスト: src/ui/test_tree_held.rs::test_nv_12_tree_parent_edit_not_held(直す前の実装では落ちることも確かめた) |
| SR-35 | checked | テスト: src/ui/test_pending_shape.rs::test_sr_35_pending_bool_and_list_use_parts |
| SR-23 | checked | 読んで判定: 英語の文言を短くし、全部の試験が通る |

既存のテストの削除・skip・弱体化: なし

確かめた: 3 / 3
