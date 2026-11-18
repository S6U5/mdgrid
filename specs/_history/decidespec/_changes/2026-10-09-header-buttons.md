---
type: Change
id: 01M4GHEP9EJD1KEA5ADQJX21ZC
title: ヘッダーにワークスペースと設定のボタン(SR-42)
status: done
size: full
created: 2026-10-09
updated: 2026-10-09
---

# ヘッダーにワークスペースと設定のボタン(SR-42)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-09-header-buttons.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した SR-42。ワークスペースの一覧(start_ws_open)とビューの設定(ViewSettings)への入口をヘッダーに出す。

## 不明点と仮定

- 仮定: ボタンを出すのは、ヘッダーの左の文字(名前・行の数など)が切れずに収まるときだけ。
- 人に確かめた(2026-10-10): ワークスペースが1つも無くてもボタンは出す(SR-42 のとおり)。WS-6 の試験の「ワークスペース」の字とぶつかるので、WS-6 に「ボタンの字は範囲の表示に数えない」を足した(specs/_decisions/2026-10-10-ws-button.md)。

## 設計

- src/ui/bands.rs: header_text(左の文字)を分け、header_buttons(出すボタンと桁)と header_button_at(クリックの当たり)。ボタンはタブの左。
- src/ui/app.rs: 表の画面と関係マップのクリックで header_button(関係マップは表に戻ってから)。
- src/i18n.rs: ボタンの文字。ゴールデン 18 本の1行目にボタンが増えた(ほかの行は同じ)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | ヘッダーのボタン | SR-42 | 試験が通る | src/ui/, src/i18n.rs, tests/golden/ | test_sr_42_settings_button・test_sr_42_workspace_button・test_sr_42_narrow_hides_buttons(src/ui/test_header_buttons.rs) | 済 |

## 実装の気づき

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-42 | checked | テスト: src/ui/test_header_buttons.rs::test_sr_42_settings_button・test_sr_42_workspace_button・test_sr_42_narrow_hides_buttons |
| WS-6 | checked | テスト: src/ui/test_workspace.rs::test_ws_6_header_shows_workspace(範囲の表示「· ワークスペース 名前」だけを見る形に書き直し、ws-button の決定で掛け直した) |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2

