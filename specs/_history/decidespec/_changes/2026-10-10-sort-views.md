---
type: Change
id: 01M4GP3D5SP1GVWAX4T608CFCY
title: 並べ替えを選んで覚える(見出しの並べ替えの保存・並べ替えの窓・既定のビュー)(NV-3・NV-20・SR-12・BV-13・BV-20・NV-24・NV-25)
status: done
size: full
created: 2026-10-10
updated: 2026-10-10
---

# 並べ替えを選んで覚える(NV-3・NV-20・SR-12・BV-13・BV-20・NV-24・NV-25)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-10-sort-views.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した sort-views。見出しの並べ替えを設定の並べ替えとして保存、検索の欄の右の「並べ替え」のボタンと窓、既定のビュー。

## 不明点と仮定

- 仮定: 見出しのクリック(と s)は、別の列なら設定の並べ替えをその列の昇順1つに置き換える(ほかの決まりは外す)。同じ列の1つだけなら 昇順 → 降順 → 外す。
- 仮定: 保存先は今ある見た目の状態(SR-12。既定の表も mdgrid のビューもビューの名前ごと)。mdgrid のビューでは定義と違えばタブに * が付く(今の「反映」と同じ)。
- 仮定: 既定のビューは views.toml の対象の表のキー default_view。

## 設計

- src/ui/columns.rs の cycle_sort と head_title を設定の並べ替えに。src/ui/settings.rs に set_sorts(当てて組み直し、persist_state)。一時的な並べ替え app.sort は使わない。
- 並べ替えの窓: 新しいモード。検索の欄の右のボタン(bands.rs)とクリック(app.rs)、操作の一覧とパレット。
- 既定のビュー: src/views.rs に default_view、src/ui/native_views.rs に「既定のビューにする」、起動(startup.rs)で選ぶ。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 見出しの並べ替えの保存 | NV-3, NV-20, SR-12 | 試験が通る | src/ui/columns.rs, src/ui/settings.rs, src/ui/test_nav.rs, src/ui/test_settings_screen.rs, src/ui/test_startup.rs | test_nv_3_temporary_sort_keeps_base・test_nv_20_layers・test_sr_12_temporary_sort_not_kept(書き直し) | 済 |
| 2 | 並べ替えの窓 | NV-24 | 試験が通る | src/ui/, src/i18n.rs, docs/ | test_nv_24_sort_window・test_nv_24_two_rules_and_order(src/ui/test_sort_window.rs) | 済 |
| 3 | 既定のビュー | NV-25, BV-13, BV-20 | 試験が通る | src/views.rs, src/ui/, src/i18n.rs, docs/ | test_nv_25_default_view・test_nv_25_missing_and_readonly(src/ui/test_default_view.rs) | 済 |

## 実装の気づき

- 既定のビューは views.toml の対象の default_view(views::load_default_view・save_default_view)。先頭のタブを既定にすると名前を消す。起動時(startup.rs)に選び、`--view` と登録した表のビューはそのあと main が選び直す。
- 説明書(tasks・getting-started)の「一時的な並べ替え」を書き直し、並べ替えの窓と既定のビューを足した。

- 並べ替えの窓は新しいモード sorts(キーの割り当て直しの [keys.sorts])。表のモードの `S` でも開く。`s` の表示名は「この列で並べ替え」に(一時的ではなくなったため)。ゴールデンの検索の欄の行にボタンが増え、ヘルプの行の数が変わった。

- 並べ替えが効くと設定の帯(NV-16)に「並べ替え」の項目が出て、表が1行下がる(見出しの行は view::data_y で引く)。

## 照合

書込なしの検証役が条件ごとに確かめた(cargo test 1546 通過・0 失敗)。指摘は直した。

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-3 | checked | テスト: src/ui/test_nav.rs::test_nv_3_temporary_sort_keeps_base(昇順・降順・解除、設定の並べ替え、.base のバイト) |
| NV-20 | checked | テスト: src/ui/test_settings_screen.rs::test_nv_20_layers。例が古い意味のままだった → 字句で直した(specs: 字句 NV-20) |
| SR-12 | checked | テスト: src/ui/test_startup.rs::test_sr_12_temporary_sort_not_kept(書き直し)、src/ui/test_sort_more.rs::test_nv_24_window_sort_kept_after_restart |
| NV-24 | checked | テスト: src/ui/test_sort_window.rs の2つ、src/ui/test_sort_remove.rs::test_nv_24_click_x_removes_rule、src/ui/test_sort_more.rs(読むだけで書かない・検索の欄を隠しても開ける・ボタンを省いた幅では当たりにしない(直した))。パレットのコマンドを READ_COMMANDS にも足した(S を外しても開ける) |
| NV-25 | checked | テスト: src/ui/test_default_view.rs の2つ。登録した表のビューが勝つ(読んで判定: src/main.rs の run_switching で開き直したあと select_view_named) |
| BV-13 | checked | テスト: src/ui/test_default_view.rs::test_nv_25_default_view(既定のビュー・無ければ先頭・--view が勝つ) |
| BV-20 | checked | 読んで判定: native_dirty が設定の違いを見るので、mdgrid のビューで並べ替えを変えるとタブに *(検証役の写しの実験で確認) |

既存のテストの削除・skip・弱体化: なし(一時的な並べ替えを確かめていた3つは、採択した文どおりの保存を確かめる形に書き直し、この決定で掛け直した)

確かめた: 7 / 7

