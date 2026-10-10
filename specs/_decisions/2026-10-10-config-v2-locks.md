---
type: Proposal
id: 01M4JR4MD0JFTYDNJ9B9ZSGSCJ
title: 設定の形の作り直しに合わせて、項目の名前と Config の形に触れる試験の錠を掛け直す(config-v2-locks)
status: accepted
approval-evidence: 会話 2026-10-10(人の決定 Q2「掛け直してよい」と「さいたく」)
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-10
touches: [BV-17, BV-20, CE-16, CE-20, CE-22, CE-25, CE-26, CE-27, CE-33, CLI-11, CLI-12, CLI-18, NV-23, NV-24, SR-8, SR-20, SR-21, SR-23, SR-26, SR-27, SR-30, SR-32, SR-33, SR-34, SR-35, SR-36, SR-37, SR-38, SR-39, SR-40, SR-41, SR-43, WB-2, WB-3, WB-18, WS-1, WS-6]
created: 2026-10-10
updated: 2026-10-10
---

# 設定の形の作り直しに合わせて、項目の名前と Config の形に触れる試験の錠を掛け直す(config-v2-locks)

## きっかけ

設定の形を作り直した([config-v2](2026-10-10-config-v2.md))のと、範囲ごとの上書きを足した([scope-overrides](2026-10-10-scope-overrides.md))のに合わせて、錠のある試験のうち、設定の項目の名前(`theme`・`[style]`・`search_bar` など)を書いた設定の文、`Config` の欄(`c.theme`・`c.search_bar` など)、`look.toml`、`views.toml` の `[[target]]`、警告の文の形を確かめていたものを、新しい形に書き直した。人は不明点 Q2 で、作り直しの決定記録でこれらを掛け直してよいと決めた(会話 2026-10-10)。

確かめる中身(色・形・表示・日付・新しいノート・ワークスペース・関係の振る舞い)は変えていない。変えたのは、試験が書く設定の文を新しい道筋にしたこと(`theme = "nord"` → `[look] theme = "nord"` など)、`Config` の欄を決まった値(`c.resolved()`)から読むようにしたこと、警告の文の形(「ファイル: 道筋: 理由」)、`look.toml` を `ui.toml` にしたこと、見た目の区画の項目の並び(保存先の行が先頭に増えた)。旧い書き方そのものは、新しい試験(tests/test_config_v2.rs・tests/test_display_options.rs の test_cli_20_*)で確かめる。

この記録は、decidespec の `check.py --lock --relock specs/_decisions/2026-10-10-config-v2-locks.md` で錠を掛け直すためのもの(この作業の環境には check.py が無いので、specs/test-locks.json はまだ掛け直していない)。

## 差分

- 錠: BV-17(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: BV-20(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CE-16(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CE-20(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CE-22(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CE-25(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CE-26(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CE-27(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CE-33(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CLI-11(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CLI-12(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: CLI-18(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: NV-23(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: NV-24(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-8(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-20(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-21(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-23(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-26(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-27(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-30(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-32(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-33(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-34(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-35(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-36(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-37(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-38(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-39(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-40(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-41(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: SR-43(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: WB-2(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: WB-3(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: WB-18(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: WS-1(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))
- 錠: WS-6(下の試験。掛け直しは [config-v2-relock](2026-10-10-config-v2-relock.md))

掛け直す試験(元の書き方):

- 試験: src/test_bounds_unit.rs の test_sr_36_style_wrong_types_warn, test_sr_40_colors_wrong_types_warn を掛け直す
- 試験: src/test_cells_unit.rs の test_sr_35_cells_bad_values_warn, test_sr_35_cells_string_and_table を掛け直す
- 試験: src/test_colors_unit.rs の test_sr_40_colors, test_sr_41_value_colors_read を掛け直す
- 試験: src/test_config_unit.rs の test_cli_12_every_item_is_read, test_cli_12_items_have_descriptions, test_ce_22_config_date_items_wrong_types_warn, test_nv_23_config_search_bar を掛け直す
- 試験: src/test_dozy_unit.rs の test_sr_36_dozy_pink_preset を掛け直す
- 試験: src/test_nerd_auto_unit.rs の test_sr_36_nerd_font_setting を掛け直す
- 試験: src/test_newnote_unit.rs の test_ce_27_bad_ask_and_set_columns_warn_and_drop を掛け直す
- 試験: src/test_places_unit.rs の test_cli_18_bad_rows_warn_and_skip を掛け直す
- 試験: src/test_style_unit.rs の test_sr_36_style_bad_values_warn, test_sr_36_style_presets_and_overrides, test_sr_37_new_themes_and_sr_39_auto, test_sr_38_names_are_listed を掛け直す
- 試験: src/test_views_unit.rs の test_bv_20_save_keeps_other_targets_and_empty_removes_target を掛け直す
- 試験: src/test_workspace_chosen_unit.rs の test_ws_6_chosen_without_the_table_warns を掛け直す
- 試験: src/test_workspace_more_unit.rs の test_ws_1_home_and_relative_paths を掛け直す
- 試験: src/test_workspace_unit.rs の test_ws_1_read_write_round_trip, test_ws_6_resolve_order を掛け直す
- 試験: src/ui/test_borders.rs の test_sr_32_ascii_by_setting, test_sr_32_ascii_when_ambiguous_wide, test_sr_32_bad_value_warns を掛け直す
- 試験: src/ui/test_calendar.rs の test_ce_22_short_formats_keep_year, test_ce_22_date_format_in_table_and_input を掛け直す
- 試験: src/ui/test_colors_more.rs の test_sr_40_no_color_ignores_colors, test_sr_41_value_color_in_every_chip_shape を掛け直す
- 試験: src/ui/test_colors_screen.rs の test_sr_36_solid, test_sr_40_accent_override, test_sr_41_value_colors を掛け直す
- 試験: src/ui/test_display_more.rs の test_sr_20_column_lines_with_ambiguous_wide_keep_positions を掛け直す
- 試験: src/ui/test_display_options.rs の test_sr_20_zebra_not_without_color を掛け直す
- 試験: src/ui/test_group_gap.rs の test_sr_30_print_config_and_views_key, test_sr_30_view_override_and_row_numbers を掛け直す
- 試験: src/ui/test_keymap.rs の test_cli_12_keys_item_lists_every_mode を掛け直す
- 試験: src/ui/test_look.rs の test_sr_33_bad_value_warns, test_sr_33_classic_and_no_color_keep_reverse, test_sr_33_footer_keys_accent_labels_dim, test_sr_33_table_selection_is_tinted, test_sr_33_text_is_the_same_in_every_look を掛け直す
- 試験: src/ui/test_look_keep.rs の test_sr_43_auto_kept, test_sr_43_only_changed_items_saved を掛け直す
- 試験: src/ui/test_look_section.rs の test_sr_43_choose_apply_and_save, test_sr_43_readonly_applies_without_writing, test_sr_43_templates_save_use_delete_and_reset を掛け直す
- 試験: src/ui/test_modern_rest.rs の test_sr_33_no_reverse_left_in_modern を掛け直す
- 試験: src/ui/test_note_editor.rs の test_ce_33_bad_mode_warns_and_uses_form を掛け直す
- 試験: src/ui/test_rich_cells.rs の test_sr_35_per_part_and_column, test_sr_35_plain_without_color_or_setting を掛け直す
- 試験: src/ui/test_rich_cells_more.rs の test_sr_35_chip_column_wins_over_parts を掛け直す
- 試験: src/ui/test_sort_more.rs の test_nv_24_hidden_bar_and_narrow を掛け直す
- 試験: src/ui/test_style_frames.rs の test_sr_33_frame_accent_in_every_frame_shape を掛け直す
- 試験: src/ui/test_style_list_pill.rs の test_sr_36_picking_cell_keeps_parts, test_sr_36_tint_rounds_with_nerd_font, test_sr_36_value_list_keeps_pill_shape を掛け直す
- 試験: src/ui/test_style_props.rs の test_sr_36_any_style_fits_the_screen を掛け直す
- 試験: src/ui/test_style_screen.rs の test_sr_36_cross_tints_column, test_sr_36_no_color_keeps_text, test_sr_36_overrides_each_part, test_sr_36_pill_needs_nerd_font, test_sr_36_preset_saas_tints_and_segments, test_sr_36_rules_and_frames を掛け直す
- 試験: src/ui/test_style_selected.rs の test_sr_36_selected_cell_keeps_parts を掛け直す
- 試験: src/ui/test_themes.rs の test_sr_26_nord_paints_background_selection_and_band, test_sr_26_every_theme_keeps_the_text, test_sr_26_every_theme_but_default_paints, test_sr_26_zebra_uses_theme_colors, test_sr_27_default_zebra_is_unchanged, test_sr_27_color_false_ignores_theme, test_sr_27_default_is_unchanged, test_sr_27_indexed_uses_256_colors, test_sr_27_no_color_ignores_theme, test_sr_27_theme_names, test_sr_27_unknown_theme_warns_and_falls_back を掛け直す
- 試験: src/ui/test_themes_more.rs の test_sr_26_zebra_row_selection_mark_and_bold, test_sr_26_colhead_only_on_the_column_heading_row, test_sr_26_input_box_is_strong_not_colhead, test_sr_26_search_hit_name_is_strong_not_colhead を掛け直す
- 試験: src/ui/test_view_tabs_auto.rs の test_sr_34_unknown_value_warns を掛け直す
- 試験: tests/test_catalog.rs の test_sr_38_catalog_matches_config, test_sr_38_catalog_presets_match を掛け直す
- 試験: tests/test_catalog_nerd.rs の test_sr_38_catalog_nerd_default を掛け直す
- 試験: tests/test_config_docs.rs の test_cli_11_output_reads_back_without_warnings_as_default, test_cli_11_output_has_every_item_with_comment, test_cli_11_output_values_are_defaults, test_cli_12_keys_cover_known_items, test_cli_12_parse_knows_every_key, test_cli_12_print_config_matches_keys を掛け直す
- 試験: tests/test_config_docs_values.rs の test_cli_12_english_doc_type_default_example, test_cli_12_japanese_doc_type_default_example, test_cli_12_table_defaults_are_real_defaults を掛け直す
- 試験: tests/test_date_format.rs の test_cli_3_config_reads_date_format_and_week_start, test_cli_3_config_date_defaults, test_cli_3_config_bad_date_format_warns_and_uses_default を掛け直す
- 試験: tests/test_display_options.rs の test_sr_21_defaults, test_sr_21_reads_each_item, test_sr_21_display_is_a_known_item, test_sr_21_wrong_type_warns_and_keeps_default, test_sr_21_unknown_item_warns, test_sr_21_print_config_has_display を掛け直す
- 試験: tests/test_e2e.rs の test_wb_3_e2e_add_frontmatter_false_makes_cell_readonly_with_reason を掛け直す
- 試験: tests/test_editor_setting.rs の test_sr_8_config_editor_keeps_other_items を掛け直す
- 試験: tests/test_empty_frontmatter.rs の test_cli_3_add_frontmatter_setting を掛け直す
- 試験: tests/test_examples.rs の test_examples_showcase を掛け直す
- 試験: tests/test_language.rs の test_sr_23_print_config_has_language_auto を掛け直す
- 試験: tests/test_language_lib.rs の test_sr_23_lib_config_warnings_are_english, test_sr_23_lib_config_warnings_stay_japanese_under_ja を掛け直す
- 試験: tests/test_new_note.rs の test_ce_25_whole_flow_writes_new_note_only, test_ce_26_view_rule_replaces_config_rule, test_ce_27_new_note_is_a_documented_config_item, test_ce_27_config_new_note_is_read_without_warnings, test_ce_27_default_config_has_empty_rule を掛け直す
- 試験: tests/test_theme_auto_e2e.rs の test_sr_39_e2e_auto_theme_without_answer_starts を掛け直す
- 試験: src/test_look_unit.rs の test_sr_43_apply_overrides_config, test_sr_43_look_broken_and_bad_values_warn, test_sr_43_look_round_trip_with_templates を外す(ファイルを消した。新しい試験は src/test_uifile_unit.rs)

## 却下した案

| 案 | 理由 |
|---|---|
| 旧い書き方の設定の文と Config の欄を残し、錠のある試験を変えない | 人は綺麗に組み直すことを選び、錠を掛け直してよいと決めた(Q2)。旧い名前を中の形に残すと、作り直しの意味が無い |

## 承認の記録

2026-10-10 人の承認(要約: 作り直しの不明点 Q2 で「掛け直してよい」を選び、提案2本を「さいたく」と採択した)
