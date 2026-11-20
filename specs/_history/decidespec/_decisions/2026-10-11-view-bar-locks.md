---
type: Proposal
id: 01M4NBQFYVYRTB2EBP6FC7VQ5X
title: ビューの帯で表の行の位置と検索の欄の字が変わる試験の錠を掛け直す(view-bar-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 565339bc4d33d728
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-11
touches: [BV-7, CE-16, NV-16, NV-17, NV-2, SC-17, SR-12, SR-21]
created: 2026-10-11
updated: 2026-10-11
---

# ビューの帯で表の行の位置と検索の欄の字が変わる試験の錠を掛け直す(view-bar-locks)

## きっかけ

人の決定(2026-10-11-view-bar)で、表の上をパンくずのヘッダーとビューの帯の2行にした(タブの行と検索の欄の行を1行にまとめた)。人がこの配置を残すと言った(会話 2026-10-11 の要約)。その影響で、文言の変わらない別の守られる要件の試験が、表・設定の帯・列の見出しの行の位置(1行上がる)と、検索の欄の字と位置(「\ 検索」でビューの帯の中)を前提にしていて落ちる。要件の文は変えず、試験の位置と字だけを新しい配置に合わせて錠を掛け直す。決定記録 2026-10-11-view-bar の要件(SR-1・SR-42・NV-23・NV-24・NV-29・SR-20・SR-34・WS-6・WS-8)の試験は、その決定記録で掛け直す。

## 差分

- 錠: BV-7(test_grid::test_bv_7_unsupported_formula_column。表が1行上に上がり、見る行の位置が変わるため)
- 錠: CE-16(test_screen::test_sr_18_read_only_cell_reason。表の行の位置(クリックと読む行)が1行上に変わるため)
- 錠: NV-16(test_settings_screen::test_nv_16_band_lists_settings・test_freq::test_nv_9_enter_on_first_filters_to_value。設定の帯の行がビューの帯の下(3行目)に変わるため)
- 錠: NV-17(test_settings_screen::test_nv_17_settings_survive_restart_per_view。帯と表の行の位置が変わるため)
- 錠: NV-2(test_nav::test_nv_2_quick_filter_and_esc・test_search_bar::test_nv_2_backslash_enters_bar_and_esc_restores。検索の欄の字(「\ 検索」)と位置がビューの帯の中に変わるため)
- 錠: SC-17(test_csv_note_only::test_sc_17_no_note_label_column。表の見出しの行の位置が変わるため)
- 錠: SR-12(test_filter_window::test_nv_29_kept_after_restart。検索の欄の行を「検索:」で探していて、欄の字と位置が変わるため)
- 錠: SR-21(test_display_options::test_sr_20_tabs_hidden_but_brackets_switch。タブを隠したとき、検索の欄はビューの帯に残り、表が上がらないため)

## 却下した案

| 案 | 理由 |
|---|---|
| 試験を変えずに、表の行の位置を前と同じにする | ビューの帯で表の上の行を減らすのが、人が残すと言った変更の目的そのもの |

## 承認の記録

2026-10-11 人の承認(要約: 錠を掛け直す提案に OK と答えた)
