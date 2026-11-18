---
type: Change
id: 01M4E9581BH4VQ90M9BS842036
title: ビューが1つならタブの行を出さない設定と、既定の表の英語の名前(SR-34)
status: done
size: full
created: 2026-10-09
updated: 2026-10-09
---

# ビューが1つならタブの行を出さない設定と、既定の表の英語の名前(SR-34)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-09-view-tabs-auto.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、フォルダを開いたときのタブの「Default」が分かりにくいと言い、ビューが1つならタブの行を出さない設定と、名前の変更の両方を求めた(会話 2026-10-09)。SR-34 と、既定の表の英語の名前。

## 不明点と仮定

- 仮定: 名前の変更は英語だけ(Default → All notes)。日本語の「既定の表」は、錠のある多くの試験と文書が使っていて、意味も通るので変えない。外れたら: 日本語も変える提案を出す。
- 仮定: 今までの名前「Default」は、書いてある設定(places.toml の view など)が読めるよう、既定の表の別名として受ける。

## 設計

- src/config.rs・src/config_items.rs・docs/config*.md: `view_tabs`("always" / "auto")。
- src/ui/bands.rs: tab_rows で、auto かつビューが1つなら 0。ヘッダーのビューの名前もそのときは出さない。
- src/i18n.rs: DefaultTabName の英語を "All notes"。src/ui/native_views.rs と select_view_named で古い "Default" も受ける。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | view_tabs と既定の表の英語の名前 | SR-34 | 試験が通る | src/config.rs, src/config_items.rs, src/i18n.rs, src/ui/bands.rs, src/ui/app.rs, src/ui/native_views.rs, src/ui/places.rs, src/ui/mod.rs, src/ui/test_view_tabs_auto.rs, docs/, tests/golden/, specs/test-locks.json | test_sr_34_auto_hides_single_tab(src/ui/test_view_tabs_auto.rs) | 済 |

## 実装の気づき

- ビューが既定の表だけのときのタブ(TabDefault)は幅が10桁に決め打ちだったので、名前の幅にした(All notes が切れないように)。
- 名前の検査(name_problem)を試験から呼べるよう pub(crate) にした。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-34 | checked | コード: src/ui/bands.rs(tab_rows・single_tab_hidden)、src/config.rs(view_tabs)、src/ui/places.rs(前の名前 Default) / テスト: src/ui/test_view_tabs_auto.rs::test_sr_34_auto_hides_single_tab・test_sr_34_auto_shows_tabs_with_two_views・test_sr_34_unknown_value_warns・test_sr_23_default_tab_english_name |

既存のテストの削除・skip・弱体化: なし

照合は書込なしの検証役(2026-10-09)。残した点: この変更の前に「All notes」という名前で保存した mdgrid のビューがあると、既定の表と同じ名前のタブが2つ並ぶ(新しく付ける名前は断る)。

確かめた: 1 / 1
