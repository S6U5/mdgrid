---
type: Change
id: 01M4GTJ7E8XFVWAPXWWHKYE9QG
title: ビューの設定の画面を、左に区画の一覧・右に中身・上に反映の形に作り直す(NV-18)
status: done
size: full
created: 2026-10-10
updated: 2026-10-10
---

# ビューの設定の画面を、左に区画の一覧・右に中身・上に反映の形に作り直す(NV-18)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: 形はカタログの見本で人が確かめた | — |
| 仕様化 | 済 | specs/_decisions/2026-10-10-settings-layout.md・2026-10-10-settings-layout-follow.md・2026-10-10-settings-layout-examples.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が良いと言ったカタログの見本(SR-38)の新しい設定の画面を、本物の画面にする。左に区画(列・フィルター・並べ替え・グループ・表示)と今の状態、右に選んだ区画の説明と項目、上の右に反映・取り消しと未反映の数、表示の項目はスイッチ(● オン / ○ オフ)。

## 設計

- src/ui/settings_view.rs を書き直す。左の幅は画面の 1/4(14〜24)。区画の印(badge)と未反映の数(pending)は Draft と今の設定の差から数える。
- ボタンの並びは上(反映・取り消し)と下(初期化・名前を付けて保存・上書き・名前を変える・削除)に分け、Draft::button_row で上下を行き来する。
- 文言は i18n.rs の Msg に足す(SR-23)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 画面の描き方とクリック | NV-18, NV-13 | 試験が通る | src/ui/settings_view.rs, src/ui/settings.rs, src/i18n.rs | test_nv_18 系(src/ui/test_settings_screen.rs)、tests/golden/nv_18*.txt | 済 |
| 2 | 表示の区画のスイッチ | SR-20, SR-21 | 試験が通る | src/ui/settings_view.rs | src/ui/test_display_options.rs・test_display_more.rs | 済 |

## 実装の気づき

- 下のボタンを右の欄にそろえると狭い画面で幅を超えたため、左の端(x=2)から並べる。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-18 | checked | テスト: src/ui/test_settings_screen.rs(区画の一覧・印・未反映の数・区画ごとの中身・上の反映のクリック)、golden nv_18・nv_18_values |
| NV-13 | checked | テスト: src/ui/test_display_options.rs::toggle_in_settings_applies_to_that_view_only ほか |
| SR-20 | checked | テスト: src/ui/test_display_options.rs・src/ui/test_display_more.rs(表示の区画のスイッチ、低い画面で流れる) |
| SR-21 | checked | テスト: src/ui/test_display_options.rs::row_numbers_from_config・column_lines_default_none_on_draws_bars(設定の既定が表示の区画のスイッチに出る) |
| SR-23 | checked | テスト: src/ui/test_language_more.rs(区画の名前の日英) |

既存のテストの削除・skip・弱体化: なし(表示の確かめを [x] からスイッチの文字に置き換えた)

確かめた: 5 / 5
