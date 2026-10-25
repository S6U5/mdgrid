---
type: Change
id: 01M4ZJF8P4B4NJMGVPGNAF5RS7
title: 折り返すときはタブを全部、検索の欄の行の下に要る行の数だけ並べ、間の空き行を tab_wrap_gap で選ぶ(tab-wrap-below)
status: done
size: full
created: 2026-10-15
updated: 2026-10-15
---

# 折り返すときはタブを全部、検索の欄の行の下に要る行の数だけ並べ、間の空き行を tab_wrap_gap で選ぶ(tab-wrap-below)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [決定](../_decisions/2026-10-15-tab-wrap-below.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 「実装: [NV-31][SR-21] …」のコミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

前の直し(2026-10-15-tab-wrap-adjacent)の画面を見た人が、タブが検索の欄の行に続く2行目になって切れ目が変だと言い、ヘッダーの下に検索や絞り込みの行、その下にタブを要る行の数だけ並べ、間を空けるかは人ごとに設定で選べればよいと言った(会話 2026-10-15)。台帳(タブ11個、幅 100)で、帯の行が検索の欄とボタンだけになり、その下にタブが続けば終わり。関係する要件: NV-31・NV-23・SR-21。

## 不明点と仮定

- 仮定: 入りきるときは今の1行のまま(タブを下に置くのは入りきらないときだけ)。広い端末で行を増やさないため。外れたら: いつも下に置く設定を足す。
- 仮定: 空き行の既定は空けない(false)。帯の下の余白がタブの行と表の間を空けるため。

## 設計

- `src/ui/bands.rs` の `view_bar`: 折り返すときはタブを全部 `wrapped` に移し(左から幅いっぱい)、真ん中の行のタブを空にする(検索の欄は左の端から)。
- `view_bar_lines`: 余白 0 は 真ん中の行・(空き行)・タブの行。囲まない帯は 上の余白・真ん中の行・(空き行)・タブの行・下の余白(端数の地の色はタブの行と空き行にも)。囲む帯は 上の線・真ん中の行・下の線・(空き行)・タブの行・残りの余白。
- `bar_mid` から足した行を外し(帯の上に行を足さない)、`wrap_rows` は タブの行 + 空き行、`wrap_y` は真ん中の行の下(囲むときは線の下、空き行の下)。`tab_at` はタブの行だけを見る。
- `src/display.rs`・`src/schema.rs` に `tab_wrap_gap`(真偽。既定 false。プロファイルの範囲)。文書 docs/config.md・docs/config.ja.md。
- 錠: `[display]` の項目の数の3つの試験を 15 にし、決定の記録で掛け直す。
- 却下: 入りきるときもいつも下に置く案(決定の「却下した案」)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | タブを全部真ん中の行の下へ並べる | NV-31 | 幅 60 で、検索の欄の行にタブが無く、すぐ下から間を空けずにタブの行 | src/ui/bands.rs src/ui/test_tab_wrap.rs src/ui/test_tab_wrap_below.rs | test_nv_31_all_tabs_below_bar・test_nv_31_tabs_right_below_search_by_default | 済 |
| 2 | 空き行の設定 tab_wrap_gap | NV-31 SR-21 | true で空き行が1つ、折り返さないときは何もしない、`[display]` は15項目 | src/display.rs src/schema.rs docs/config.md docs/config.ja.md tests/test_display_options.rs | test_nv_31_gap_puts_one_empty_row・test_nv_31_gap_without_wrap_changes_nothing・test_sr_21_print_config_has_display | 済 |
| 3 | 囲む帯・余白 0・クリック | NV-31 | box では下の線の下、余白 0 では真ん中の行のすぐ下、空き行のあとのタブのクリックでそのビュー | src/ui/test_tab_wrap_below.rs | test_nv_31_box_tabs_below_bottom_line・test_nv_31_zero_padding_tabs_below・test_nv_31_click_tab_below_after_gap | 済 |

## 実装の気づき

前の直しの試験ファイル src/ui/test_tab_wrap_adjacent.rs は、置き場の要件が変わったので消し、test_tab_wrap_below.rs に置き換えた(錠は無い)。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-31 | checked | コード: src/ui/bands.rs:560(wrap_y)・src/ui/bands.rs:1008(タブの行を真ん中の行の下に) / テスト: src/ui/test_tab_wrap_below.rs::test_nv_31_tabs_right_below_search_by_default / テスト: src/ui/test_tab_wrap_below.rs::test_nv_31_gap_puts_one_empty_row / テスト: src/ui/test_tab_wrap_below.rs::test_nv_31_gap_without_wrap_changes_nothing / テスト: src/ui/test_tab_wrap_below.rs::test_nv_31_box_tabs_below_bottom_line / テスト: src/ui/test_tab_wrap_below.rs::test_nv_31_click_tab_below_after_gap / テスト: src/ui/test_tab_wrap.rs::test_nv_31_all_tabs_below_bar / 今: 本物の台帳(タブ11個)を幅 100 で撮り、検索の欄の行の下にタブが2行で続く / 前: 足した行が検索の欄の行の上にあり、残りのタブが検索の欄の行の左に続いていた |
| NV-23 | checked | コード: src/ui/bands.rs:790(折り返したときは今のでないタブを省かない) / テスト: src/ui/test_tab_wrap.rs::test_nv_31_off_keeps_current_tab_only / 今: tab_wrap = false で幅 60 は今のタブだけ / 前: 同じ |
| SR-21 | checked | コード: src/display.rs(tab_wrap_gap) / テスト: tests/test_display_options.rs::test_sr_21_print_config_has_display / 今: --print-config の [display] は15項目 / 前: 14項目 |

既存のテストの削除・skip・弱体化: src/ui/test_tab_wrap_adjacent.rs を消した(前の直しの置き場を確かめる試験。要件が変わったため。新しい置き場は test_tab_wrap_below.rs で確かめる)

確かめた: 3 / 3
