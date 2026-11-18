---
type: Proposal
id: 01M4QB1GH7AQYDCGR7ENQ5911Y
title: 余白の 0.5 刻み・半行の描き方で行の位置が変わる試験の錠を、まとめて掛け直す(half-row-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 43b88f43a9f132c5
decided-by: 人
trigger: AI の提案
trigger-link: 会話 2026-10-12
touches: [BV-7, BV-13, BV-17, CE-25, CLI-1, CLI-3, CLI-11, CLI-19, NV-1, NV-2, NV-13, NV-15, NV-16, NV-23, NV-24, NV-29, REL-8, REL-12, SR-1, SR-18, SR-20, SR-21, SR-23, SR-26, SR-27, SR-33, SR-34, SR-37, SR-42, WS-6, WS-8]
created: 2026-10-12
updated: 2026-10-12
---

# 余白の 0.5 刻み・半行の描き方で行の位置が変わる試験の錠を、まとめて掛け直す(half-row-locks)

## きっかけ

人の決定 2026-10-12-bar-padding-half(余白を 0.5 刻みにし、既定を 0.5 に。ヘッダーの上にも余白)と 2026-10-12-half-row-inverse(半行は帯の色で行ごと塗り、外側の半分を画面の地の色の字で消す。地の色が分からなければ1行に切り上げる)で、ヘッダーが1行目から下がり、表の上の行の数と、余白の行の字が変わった。人は会話 2026-10-12 で、手元の端末で筋が消えたのを確かめ、今の形(地の色が分かるときだけ半行、分からないときは1行の空き行)のままでよいと答えた。

既存の試験 112 本は、その配置を前提にした行の番号・探す行だけを直したが、これらの決定が触れていない要件の錠も持つので、その記録では掛け直せない。直しは位置合わせが主で、確かめる中身を消していない。中身に触れた直しは次の3つで、どれも上の決定の振る舞いをそのまま写したもの:

- `test_look::test_sr_33_text_is_the_same_in_every_look` と `test_themes::test_sr_26_every_theme_keeps_the_text`: 色を使う表示とテーマなし・色なしの表示で文字が同じことを比べるとき、半分の塗りの字(`▀`・`▄`)を空白とみなす(地の色が分かるときだけ半行の字を描き、分からないときは同じ行の数の空き行にするため。文字の行の数と、それ以外の字は前と同じに比べる)。
- `test_themes` の左上の地の色: 左上が半行の `▀` のときは、見えている上の半分の色(字の色)で比べる。
- `test_bar_padding_half` のテーマなしの既定の配置: 半行の字と地の色を確かめていたのを、空き行で地の色を敷かないことの確かめに変えた(テーマなしでは画面の地の色が分からず、1行に切り上げるため)。半行の字の確かめは、新しいファイル `test_half_row_inverse.rs`・`test_half_row_inverse_termbg.rs` に移した。

直した試験のファイル:

| ファイル | 本数 |
|---|---|
| `src/ui/test_bar_padding_half.rs` | 5 |
| `src/ui/test_bar_padding_theme.rs` | 3 |
| `src/ui/test_crumb_button.rs` | 5 |
| `src/ui/test_display_more.rs` | 1 |
| `src/ui/test_display_options.rs` | 14 |
| `src/ui/test_gaps.rs` | 1 |
| `src/ui/test_grid.rs` | 5 |
| `src/ui/test_header_buttons.rs` | 3 |
| `src/ui/test_header_menu.rs` | 10 |
| `src/ui/test_language.rs` | 1 |
| `src/ui/test_look.rs` | 1 |
| `src/ui/test_nav.rs` | 1 |
| `src/ui/test_plural_en.rs` | 3 |
| `src/ui/test_relmap.rs` | 2 |
| `src/ui/test_relmap_click.rs` | 1 |
| `src/ui/test_relmap_new_note.rs` | 2 |
| `src/ui/test_screen_tabs_fixed.rs` | 1 |
| `src/ui/test_search_box.rs` | 3 |
| `src/ui/test_settings_screen.rs` | 1 |
| `src/ui/test_themes.rs` | 13 |
| `src/ui/test_view_bar.rs` | 16 |
| `src/ui/test_view_bar_cases.rs` | 2 |
| `src/ui/test_view_bar_plain.rs` | 1 |
| `src/ui/test_workspace.rs` | 1 |
| `src/ui/test_ws_drawer.rs` | 15 |
| `tests/test_display_options.rs` | 1 |

## 差分

- 錠: BV-7(test_grid::test_bv_7_unevaluable_filters_view_not_opened、test_grid::test_bv_7_unsupported_formula_column)
- 錠: BV-13(test_display_more::test_sr_20_hidden_tabs_header_shows_view_name、test_grid::test_bv_13_switch_view_changes_rows_and_cols、test_grid::test_sr_1_base_header_and_tabs)
- 錠: BV-17(test_display_options::test_sr_20_saved_native_view_keeps_display)
- 錠: CE-25(test_header_menu::test_ce_25_new_in_view_bar_right_end、test_header_menu::test_ce_25_new_same_place_in_relations、test_relmap_new_note::test_ce_25_relmap_new_note、test_relmap_new_note::test_ce_25_relmap_new_note_other_table)
- 錠: CLI-1(test_grid::test_cli_2_view_option_opens_named_view)
- 錠: CLI-3(test_themes::test_sr_27_unknown_theme_warns_and_falls_back)
- 錠: CLI-11(test_display_options::test_sr_21_print_config_has_display)
- 錠: CLI-19(test_ws_drawer::test_ws_8_pending_changes_ask_then_back、test_ws_drawer::test_ws_8_pending_changes_save_then_switch)
- 錠: NV-1(test_gaps::test_nv_1_nv_8_highlight_is_drawn_on_cells)
- 錠: NV-2(test_search_box::test_nv_23_box_typing_shows_word_inside、test_view_bar::test_nv_23_view_bar_search_word_and_count)
- 錠: NV-13(test_display_options::test_sr_20_settings_screen_has_display_section、test_display_options::test_sr_20_toggle_in_settings_applies_to_that_view_only)
- 錠: NV-15(test_settings_screen::test_nv_15_group_off_and_by_column)
- 錠: NV-16(test_display_options::test_sr_20_chips_hidden_but_f_works、test_display_options::test_sr_20_chips_hidden_from_settings_screen、test_view_bar::test_sr_1_view_bar_with_condition_then_chips)
- 錠: NV-23(test_bar_padding_half::test_nv_23_half_ambiguous_wide_rounds_up、test_bar_padding_half::test_nv_23_half_one_and_a_half、test_bar_padding_half::test_sr_1_half_default_layout、test_bar_padding_half::test_sr_1_half_no_color_rounds_up ほか 13 本)
- 錠: NV-24(test_search_box::test_nv_23_box_window_below_bottom_line、test_view_bar::test_nv_24_view_bar_sort_window_below_button、test_view_bar_plain::test_nv_23_plain_window_right_below_bar)
- 錠: NV-29(test_view_bar::test_nv_29_table_edit_column_in_place、test_view_bar::test_nv_29_table_edit_op_in_place、test_view_bar::test_nv_29_table_edit_value_field、test_view_bar::test_nv_29_table_rows_have_column_op_value_remove ほか 1 本)
- 錠: REL-8(test_header_menu::test_rel_8_menu_back_to_table、test_header_menu::test_rel_8_menu_esc_closes_without_change、test_header_menu::test_rel_8_menu_lists_types_and_switches、test_header_menu::test_rel_8_menu_without_links_has_only_table ほか 3 本)
- 錠: REL-12(test_relmap_click::test_rel_12_click_link_and_linked_row)
- 錠: SR-1(test_bar_padding_half::test_sr_1_half_default_layout、test_bar_padding_half::test_sr_1_half_no_color_rounds_up、test_bar_padding_half::test_sr_1_half_zero_paddings、test_bar_padding_theme::test_nv_23_half_theme_background_nord ほか 9 本)
- 錠: SR-18(test_nav::test_nv_8_same_value_highlight_and_filter)
- 錠: SR-20(test_display_more::test_sr_20_hidden_tabs_header_shows_view_name、test_display_options::test_sr_20_chips_hidden_but_f_works、test_display_options::test_sr_20_chips_hidden_from_settings_screen、test_display_options::test_sr_20_column_lines_default_none_on_draws_bars ほか 16 本)
- 錠: SR-21(test_display_options::test_sr_20_column_lines_default_none_on_draws_bars、test_display_options::test_sr_20_row_numbers_from_config、test_display_options::test_sr_20_tabs_hidden_but_brackets_switch、test_display_options::test_sr_21_print_config_has_display)
- 錠: SR-23(test_language::test_sr_23_ui_palette_english、test_plural_en::test_sr_23_plural_header_one_row、test_plural_en::test_sr_23_plural_header_two_rows、test_plural_en::test_sr_23_plural_japanese_unchanged)
- 錠: SR-26(test_bar_padding_theme::test_nv_23_half_theme_background_nord、test_bar_padding_theme::test_nv_23_half_theme_background_solarized_light、test_look::test_sr_33_text_is_the_same_in_every_look、test_themes::test_sr_26_every_theme_but_default_paints ほか 4 本)
- 錠: SR-27(test_themes::test_sr_27_color_false_ignores_theme、test_themes::test_sr_27_default_is_unchanged、test_themes::test_sr_27_default_zebra_is_unchanged、test_themes::test_sr_27_indexed_uses_256_colors ほか 4 本)
- 錠: SR-33(test_look::test_sr_33_text_is_the_same_in_every_look)
- 錠: SR-34(test_view_bar::test_sr_1_view_bar_tabs_hidden_keeps_search_and_buttons、test_view_bar_cases::test_sr_34_auto_single_view_keeps_search_bar)
- 錠: SR-37(test_themes::test_sr_27_theme_names)
- 錠: SR-42(test_crumb_button::test_sr_42_crumb_button_click_key_and_arrow、test_crumb_button::test_sr_42_crumb_button_follows_rebound_key、test_crumb_button::test_sr_42_crumb_button_key_and_arrow、test_crumb_button::test_sr_42_crumb_button_looks_like_a_button ほか 16 本)
- 錠: WS-6(test_view_bar::test_sr_42_crumb_scope_and_table、test_workspace::test_ws_6_header_shows_workspace)
- 錠: WS-8(test_crumb_button::test_sr_42_crumb_button_click_key_and_arrow、test_view_bar::test_sr_42_crumb_click_toggles_drawer、test_view_bar::test_ws_8_crumb_drawer_switches_table、test_ws_drawer::test_ws_8_click_outside_closes ほか 14 本)

## 却下した案

| 案 | 理由 |
|---|---|
| 試験を変えずに、ヘッダーを1行目に戻す | 人が決めた配置(ヘッダーの上の余白・既定 0.5)そのものを戻すことになる |
| 色なしでも半分の塗りの字を描き、試験の比べ方を変えない | 地の色が分からないときに半行を描くと、帯の中に筋が出る(人が手元で確かめて消えた筋を戻すことになる) |

## 承認の記録

2026-10-12 人の承認(要約: 0.5 刻みと半行の描き方で行の位置を合わせた既存の試験112本の錠をまとめて掛け直す提案に「承認します」と答えた)
