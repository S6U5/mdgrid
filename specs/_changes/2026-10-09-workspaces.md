---
type: Change
id: 01M4E5NGD13X0CAY0V0GNP30CX
title: ワークスペース(アプリの側・フォルダの印・検知・範囲の決め方。WS-1〜WS-7、REL-5・REL-6 の変更)
status: active
size: full
created: 2026-10-09
updated: 2026-10-09
---

# ワークスペース(アプリの側・フォルダの印・検知・範囲の決め方。WS-1〜WS-7、REL-5・REL-6 の変更)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」(人と話して決めた) |
| 仕様化 | 済 | specs/_decisions/2026-10-09-workspaces.md・2026-10-09-workspace-marker.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 進行中 | コミット |
| 照合 | 未 | この記録の「照合」 |

## 要求

採択した WS-1〜WS-7 と、REL-5・REL-6 の範囲の変更。大きな変更なので、人の言葉なしに main へマージしない。

## 不明点と仮定

- 仮定: 表が範囲に入るかは、開いたフォルダ(か .base のフォルダ)が表のフォルダの中か同じか、開いた .base が表の .base と同じか。
- 仮定: 検知と印の自動の表で、ノートのある直下のフォルダが1つも無い根は、範囲にしない(次の順へ)。
- 仮定: ヘッダーに範囲の名前を出すのは、書いた範囲(`-w`・印・workspaces.toml)のとき。検知の範囲は関係マップのヘッダーに出す(既存の画面の文字を変えすぎないため)。
- 仮定: 画面のワークスペースの操作はパレットのコマンド(ノートを書かないので読むだけでも使える)。

## 設計

- src/workspace.rs(ライブラリ): workspaces.toml の読み書き、印(.mdgrid/workspace.toml)の読み取りと init、自動の表、検知(.obsidian/・.git)、範囲の決め方(resolve)。
- src/config.rs: `workspace_detect`。
- src/ui/workspace.rs: パレットのコマンド(作る・足す・外す・開く)。App の範囲(scope)を起動で決め、リレーション・関係マップ・タブの表を範囲から取る。
- src/main.rs・src/ws_cli.rs: 旗 `--workspaces`・`--add-to`(`--as`)・`--remove-from`・`--remove-workspace`・`--init-workspace` と `-w`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | ワークスペースの核 | WS-1, WS-5, WS-6, WS-7 | 読み書き・印・検知・決め方の試験が通る | src/workspace.rs, src/lib.rs, src/config.rs, src/config_items.rs, src/i18n.rs, src/test_workspace_unit.rs | test_ws_6_resolve_order(src/test_workspace_unit.rs) | 済 |
| 2 | 画面とコマンド、範囲の切り替え | WS-2, WS-3, WS-4, WS-6, REL-5, REL-6 | 画面とコマンドの試験が通る | src/ui/workspace.rs, src/ui/relations.rs, src/ui/relmap.rs, src/ui/bands.rs, src/ui/app.rs, src/ui/startup.rs, src/ui/native_io.rs, src/ui/keymap.rs, src/ui/mod.rs, src/main.rs, src/ws_cli.rs, src/test_main.rs, src/i18n.rs, src/ui/test_workspace.rs, tests/, docs/, demos/, README.md, README.ja.md, specs/test-locks.json | test_ws_4_scope_limits_tables_and_relmap(src/ui/test_workspace.rs)、test_ws_3_add_list_remove_on_pipe(tests/test_workspace_cli.rs) | 済 |

## 実装の気づき

- 初めは `mdgrid workspace …` の命令にしたが、`workspace` という名前のフォルダと重なるため、人の決定(specs/_decisions/2026-10-09-workspace-flags.md)で旗(`--workspaces`・`--add-to`・`--remove-from`・`--remove-workspace`・`--init-workspace`)にした。錠のある tests/test_workspace_cli.rs はその記録で掛け直した。
- パレットの `save` のあいまい一致に `workspace_remove` が引っかかるので、ゴールデン sr_14・sr_5(ヘルプの行数)を作り直した(試験のコードは変えていない)。
- ワークスペースを開いて移ったあとは、以後の開き直しも同じワークスペースを範囲にする(main の run_switching が -w を引き継ぐ)。

## 照合
