---
type: Change
id: 01M4FHKDTNNBKGZYYFP6NKHD6D
title: 自動の表に .base を入れない(WS-5・WS-7)
status: done
size: bugfix
created: 2026-10-09
updated: 2026-10-09
---

# 自動の表に .base を入れない(WS-5・WS-7)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 決定の記録のとおり | — |
| 仕様化 | 済 | specs/_decisions/2026-10-09-workspace-auto-folders.md(人) |
| 設計 | 対象外: 1か所 | — |
| タスク | 対象外: 1か所 | — |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した WS-5・WS-7 の変更(自動の表はフォルダだけ。`.base` は保管庫の根を指してほかの表と重なるので入れない)をコードにする。印の雛形のコメントと説明書も合わせる。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| WS-5 | checked | コード: src/workspace.rs(auto_tables) / テスト: src/test_workspace_unit.rs::test_ws_6_resolve_order(検知の表は books・projects・tasks) |
| WS-7 | checked | コード: src/workspace.rs(auto_tables・init の雛形) / テスト: src/test_workspace_unit.rs::test_ws_7_marker_auto_and_listed_tables(Tasks.base を置いても自動の表は projects・tasks。錠をこの決定の記録で掛け直した) |

既存のテストの削除・skip・弱体化: あり(test_ws_7_marker_auto_and_listed_tables の期待から Tasks を外した。要件の変更どおりで、人の決定の記録で錠を掛け直した)

確かめた: 2 / 2
