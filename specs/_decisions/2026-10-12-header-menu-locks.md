---
type: Proposal
id: 01M4P9TRVEV5MFMMSVMTVW44J2
title: 右上のメニュー・3行のビューの帯で行の位置が変わる試験の錠を、まとめて掛け直す(header-menu-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 5d103c64a39b8ac1
decided-by: 人
trigger: AI の提案
trigger-link: 会話 2026-10-12
touches: [BV-3, BV-7, BV-13, BV-17, BV-18, BV-19, BV-20, CE-20, CE-25, CE-26, CE-27, CLI-1, NV-2, NV-13, NV-16, NV-17, NV-23, NV-24, NV-26, NV-29, SR-1, SR-20, SR-21, SR-23, SR-34, SR-42, WB-2, WS-6, WS-8]
created: 2026-10-12
updated: 2026-10-12
---

# 右上のメニュー・3行のビューの帯で行の位置が変わる試験の錠を、まとめて掛け直す(header-menu-locks)

## きっかけ

人の決定 2026-10-12-header-menu(設定を右上の別のボタン・画面の型をメニュー・「+ 新規」をビューの帯に)、2026-10-12-search-box・view-bar-plain・bar-padding(ビューの帯を上下に余白のある3行に)で、表の上の行の数と、タブ・ボタン・「+ 新規」の行が変わった。既存の試験 82 本は、その配置を前提にした行の番号・探す行・描く幅だけを直したが、これらの決定が触れていない要件の錠も持つので、その記録では掛け直せない。直しは位置合わせだけで、確かめる中身を消したり緩めたりしていない(直しの一覧は記録 specs/_changes/2026-10-12-header-menu.md と specs/_changes/2026-10-12-search-box.md の実装の気づき)。

直した試験のファイル:

| ファイル | 本数 |
|---|---|
| `src/ui/test_display_options.rs` | 14 |
| `src/ui/test_filter_window.rs` | 1 |
| `src/ui/test_grid.rs` | 5 |
| `src/ui/test_language.rs` | 6 |
| `src/ui/test_language_more.rs` | 1 |
| `src/ui/test_native_state.rs` | 1 |
| `src/ui/test_native_views.rs` | 14 |
| `src/ui/test_new_note.rs` | 8 |
| `src/ui/test_new_note_gaps.rs` | 6 |
| `src/ui/test_search_box.rs` | 3 |
| `src/ui/test_startup.rs` | 1 |
| `src/ui/test_view_bar.rs` | 13 |
| `src/ui/test_view_bar_cases.rs` | 1 |
| `src/ui/test_view_tabs.rs` | 5 |
| `src/ui/test_view_tabs_auto.rs` | 3 |

## 差分

- 錠: BV-3(test_native_views::test_bv_19_export_refuses_existing_base、test_native_views::test_bv_19_export_writes_new_base_and_reports_dropped)
- 錠: BV-7(test_grid::test_bv_7_unevaluable_filters_view_not_opened、test_grid::test_bv_7_unsupported_formula_column)
- 錠: BV-13(test_grid::test_bv_13_switch_view_changes_rows_and_cols、test_grid::test_sr_1_base_header_and_tabs、test_native_views::test_bv_20_base_views_come_first、test_startup::test_bv_13_startup_opens_first_view)
- 錠: BV-17(test_display_options::test_sr_20_saved_native_view_keeps_display、test_native_views::test_bv_17_hand_written_views_toml_becomes_tabs、test_native_views::test_bv_17_no_new_files_in_notes_folder、test_native_views::test_bv_17_tabs_list_default_and_saved_views ほか 1 本)
- 錠: BV-18(test_native_views::test_bv_18_delete_removes_tab、test_native_views::test_bv_18_overwrite_survives_restart、test_native_views::test_bv_18_rename_changes_tab、test_native_views::test_bv_18_save_as_adds_tab)
- 錠: BV-19(test_native_views::test_bv_19_import_base_view_adds_tab、test_native_views::test_bv_19_export_refuses_existing_base、test_native_views::test_bv_19_export_writes_new_base_and_reports_dropped)
- 錠: BV-20(test_native_views::test_bv_20_base_views_come_first、test_native_views::test_bv_20_broken_views_toml_warns_and_starts_without_tabs、test_native_views::test_bv_20_readonly_writes_nothing、test_native_views::test_bv_20_unknown_key_warns_and_reads)
- 錠: CE-20(test_new_note::test_ce_26_ask_number_then_date_with_calendar)
- 錠: CE-25(test_new_note::test_ce_25_key_a_creates_note、test_new_note::test_ce_25_existing_name_refused_and_retype)
- 錠: CE-26(test_new_note::test_ce_26_ask_number_then_date_with_calendar、test_new_note::test_ce_26_empty_answer_is_not_written、test_new_note::test_ce_26_esc_while_asking_creates_nothing、test_new_note_gaps::test_ce_26_view_rule_overrides_config ほか 5 本)
- 錠: CE-27(test_new_note::test_ce_25_key_a_creates_note、test_new_note::test_ce_27_help_lists_new_note、test_new_note::test_ce_27_name_template_has_today、test_new_note::test_ce_27_rebind_n_to_new_note ほか 2 本)
- 錠: CLI-1(test_grid::test_cli_2_view_option_opens_named_view)
- 錠: NV-2(test_search_box::test_nv_23_box_typing_shows_word_inside、test_view_bar::test_nv_23_view_bar_search_word_and_count)
- 錠: NV-13(test_display_options::test_sr_20_settings_screen_has_display_section、test_display_options::test_sr_20_toggle_in_settings_applies_to_that_view_only、test_native_views::test_bv_18_save_as_adds_tab)
- 錠: NV-16(test_display_options::test_sr_20_chips_hidden_but_f_works、test_display_options::test_sr_20_chips_hidden_from_settings_screen、test_view_bar::test_sr_1_view_bar_with_condition_then_chips)
- 錠: NV-17(test_native_state::test_bv_20_applied_settings_and_columns_survive_tabs_and_restart)
- 錠: NV-23(test_display_options::test_sr_20_search_bar_hidden_from_settings_screen、test_filter_window::test_nv_29_buttons_filter_then_sort_at_right_end、test_search_box::test_nv_23_box_typing_shows_word_inside、test_search_box::test_nv_23_box_window_below_bottom_line ほか 2 本)
- 錠: NV-24(test_filter_window::test_nv_29_buttons_filter_then_sort_at_right_end、test_search_box::test_nv_23_box_window_below_bottom_line、test_view_bar::test_nv_24_view_bar_sort_window_below_button)
- 錠: NV-26(test_view_tabs::test_nv_26_current_and_base_refused、test_view_tabs::test_nv_26_hide_order_default_persist、test_view_tabs::test_nv_26_readonly_shows_but_does_not_write、test_view_tabs::test_nv_26_rename_delete_follow_prefs ほか 1 本)
- 錠: NV-29(test_filter_window::test_nv_29_buttons_filter_then_sort_at_right_end、test_view_bar::test_nv_29_table_edit_column_in_place、test_view_bar::test_nv_29_table_edit_op_in_place、test_view_bar::test_nv_29_table_edit_value_field ほか 2 本)
- 錠: SR-1(test_grid::test_sr_1_base_header_and_tabs、test_view_bar::test_sr_1_view_bar_with_condition_then_chips、test_view_bar::test_sr_1_view_bar_hidden_when_tabs_and_search_hidden、test_view_bar::test_sr_1_view_bar_tabs_hidden_keeps_search_and_buttons ほか 1 本)
- 錠: SR-20(test_display_options::test_sr_20_saved_native_view_keeps_display、test_display_options::test_sr_20_settings_screen_has_display_section、test_display_options::test_sr_20_toggle_in_settings_applies_to_that_view_only、test_display_options::test_sr_20_chips_hidden_but_f_works ほか 12 本)
- 錠: SR-21(test_display_options::test_sr_20_column_lines_default_none_on_draws_bars、test_display_options::test_sr_20_row_numbers_from_config、test_display_options::test_sr_20_tabs_hidden_but_brackets_switch)
- 錠: SR-23(test_language::test_sr_23_ui_band_english、test_language::test_sr_23_ui_calendar_english、test_language::test_sr_23_ui_help_english、test_language::test_sr_23_ui_japanese_unchanged ほか 4 本)
- 錠: SR-34(test_view_bar::test_sr_1_view_bar_tabs_hidden_keeps_search_and_buttons、test_view_bar_cases::test_sr_34_auto_single_view_keeps_search_bar、test_view_tabs_auto::test_sr_34_auto_hides_single_tab、test_view_tabs_auto::test_sr_34_auto_shows_tabs_with_two_views)
- 錠: SR-42(test_view_bar::test_sr_42_crumb_scope_and_table、test_view_bar::test_sr_42_crumb_click_toggles_drawer、test_view_bar::test_ws_8_crumb_drawer_switches_table)
- 錠: WB-2(test_new_note::test_ce_25_existing_name_refused_and_retype)
- 錠: WS-6(test_view_bar::test_sr_42_crumb_scope_and_table)
- 錠: WS-8(test_view_bar::test_sr_42_crumb_click_toggles_drawer、test_view_bar::test_ws_8_crumb_drawer_switches_table)

## 却下した案

| 案 | 理由 |
|---|---|
| 試験を変えずに、表の上の行の数を前と同じにする | 人が決めた配置(余白のある3行の帯・右上の設定)そのものを戻すことになる |

## 承認の記録

2026-10-12 人の承認(要約: 配置の変化で行の位置を合わせた既存の試験82本の錠をまとめて掛け直す提案に「承認」と答えた)
