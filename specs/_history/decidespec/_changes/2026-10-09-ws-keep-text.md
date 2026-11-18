---
type: Change
id: 01M4EHXH7BAA72SNQB2E8T2PJH
title: workspaces.toml の書き換えで、手書きのコメント・空行・~・並びを残す(WS-1)
status: done
size: bugfix
created: 2026-10-09
updated: 2026-10-09
---

# workspaces.toml の書き換えで、手書きのコメント・空行・~・並びを残す(WS-1)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | — |
| 仕様化 | 対象外: bugfix(WS-1 の「workspaces.toml に持つ」「手で書いてよい」に合わせる) | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 対象外: bugfix | — |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

places.toml と同じく(2026-10-09-places-keep-comments)、workspaces.toml も足す・外すたびに全部を書き直していて、手で書いたコメント・空行・並びが消えていた。

## 設計

- src/workspace.rs: add・remove は、今までどおり読み直した並びで確かめ(読めない行があれば書かない。錠のある試験のとおり)、書くときはファイルの文字を [[workspace]] と [[workspace.table]] の区画に分けて、その区画だけを書き換える。新しいワークスペースは末尾、新しい表はそのワークスペースの区画の末尾(末尾のコメントの前)。区画の末尾のコメントと空行は次の区画の前置きとして残す。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| WS-1 | checked | コード: src/workspace.rs(edit_add・edit_remove・blocks) / テスト: src/test_workspace_keep_unit.rs::test_ws_1_add_keeps_hand_written_text・test_ws_1_replace_and_remove_only_that_part、錠のある src/test_workspace_unit.rs・src/test_workspace_more_unit.rs |

既存のテストの削除・skip・弱体化: なし

メインが試験で確かめた。

確かめた: 1 / 1
