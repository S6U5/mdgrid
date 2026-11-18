---
type: Proposal
id: 01M4RAVQK8SVGFG8KEF50V740Y
title: 表示の項目が11になって変わる試験の錠を掛け直す(row-lines-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 5d103c64a39b8ac1
decided-by: 人
trigger: AI の提案
trigger-link: 会話 2026-10-13
touches: [SR-20, SR-21, CLI-11, NV-13, NV-16, NV-23, BV-17]
created: 2026-10-13
updated: 2026-10-13
---

# 表示の項目が11になって変わる試験の錠を掛け直す(row-lines-locks)

## きっかけ

人の決定 2026-10-12-row-lines で、表示の設定に `row_extra_lines`(行を伸ばす数)が増え、`[display]` の項目とビューの設定の画面の「表示」の区画の項目が 10 から 11 になった。項目の数を前提にした既存の試験が2つの所で落ちた。

- `src/ui/test_display_options.rs` の共通の道具 `toggle_display` は、項目を探して下へ10回までしか動かない。11項目では最後の「設定の帯」に届かない。項目の数だけ動くように直した(数を決め打ちしない)。道具はファイルの中で共有なので、このファイルの14本の試験の指紋が変わった。確かめる中身は変えていない。
- `tests/test_display_options.rs` の `--print-config` の試験は、`[display]` が10項目であることを確かめていた。11項目にし、`row_extra_lines = 0` が出ることを足した(確かめることを増やした)。

新しい項目は「設定の帯」の前に置いた。別の錠のある試験(`src/ui/test_display_more.rs::test_sr_20_display_section_scrolls_on_low_screen`)が、最後の項目を「設定の帯」と前提にしているので、そちらは変えずに通る。設定の画面の写し(`tests/golden/nv_18.txt`・`nv_18_values.txt`)は「表示 5/10」が「5/11」になったので作り直した(試験の中身は変えていない)。

直した試験:

| ファイル | 試験 | 直し |
|---|---|---|
| `src/ui/test_display_options.rs` | test_sr_20_chips_hidden_but_f_works・chips_hidden_from_settings_screen・column_lines_default_none_on_draws_bars・row_numbers_follow_sort_and_filter・row_numbers_from_config・row_numbers_skip_group_headings・saved_native_view_keeps_display・search_bar_hidden_from_settings_screen・settings_screen_has_display_section・tabs_hidden_but_brackets_switch・toggle_in_settings_applies_to_that_view_only・zebra_colors_even_rows・zebra_not_without_color・zebra_off_by_default | 共通の道具 `toggle_display` の探す回数を項目の数に(中身は変えない) |
| `tests/test_display_options.rs` | test_sr_21_print_config_has_display | 10項目 → 11項目、`row_extra_lines = 0` を確かめる |

## 差分

- 錠: SR-20(src/ui/test_display_options.rs の14本。共通の道具の探す回数を項目の数に)
- 錠: SR-21(test_display_options の row_numbers_from_config・column_lines_default_none_on_draws_bars・tabs_hidden_but_brackets_switch と、tests/test_display_options.rs::test_sr_21_print_config_has_display。項目が11になった)
- 錠: CLI-11(tests/test_display_options.rs::test_sr_21_print_config_has_display。同じ理由)
- 錠: NV-13(test_display_options の settings_screen_has_display_section・toggle_in_settings_applies_to_that_view_only。共通の道具の直し)
- 錠: NV-16(test_display_options の chips_hidden_but_f_works・chips_hidden_from_settings_screen。同じ理由)
- 錠: NV-23(test_display_options の search_bar_hidden_from_settings_screen。同じ理由)
- 錠: BV-17(test_display_options の saved_native_view_keeps_display。同じ理由)

## 却下した案

| 案 | 理由 |
|---|---|
| 新しい項目を区画の最後に置く | 最後の項目を「設定の帯」と前提にする別の錠のある試験が落ち、どちらにしても1つの試験を直すことになる。探す回数を項目の数にする直しのほうが、これから項目が増えても壊れない |
| 探す回数を 10 から 11 に変えるだけにする | 項目が増えるたびに同じ直しと錠の掛け直しが要る |

## 承認の記録

2026-10-13 人の承認(要約: 表示の項目が11になって変わる試験15本の錠を掛け直す提案に「承認」と答えた)
