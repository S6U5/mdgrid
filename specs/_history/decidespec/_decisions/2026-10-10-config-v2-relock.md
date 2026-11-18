---
type: Proposal
id: 01M4JXMCM9DPFM381GZSKEXP85
title: 公開リポで入った設定の形の作り直し(#30)が書き換えた試験の錠を掛け直す(config-v2-relock)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 6fad82c06c574240
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-10
touches: [BV-17, BV-19, BV-20, CE-16, CE-20, CE-22, CE-25, CE-26, CE-27, CE-33, CLI-11, CLI-12, CLI-18, NV-2, NV-18, NV-23, NV-24, REL-5, SR-8, SR-20, SR-21, SR-23, SR-26, SR-27, SR-30, SR-32, SR-33, SR-34, SR-35, SR-36, SR-37, SR-38, SR-39, SR-40, SR-41, SR-42, SR-43, WB-2, WB-3, WB-18, WS-1, WS-2, WS-4, WS-6, WS-7]
created: 2026-10-10
updated: 2026-10-10
---

# 公開リポで入った設定の形の作り直し(#30)が書き換えた試験の錠を掛け直す(config-v2-relock)

## きっかけ

会話 2026-10-10。設定の形の作り直し([config-v2](2026-10-10-config-v2.md)・[scope-overrides](2026-10-10-scope-overrides.md))は、別の作業の場で公開リポに直接入った(#30)。その場には check.py が無く、錠のある試験 196 本を書き換えたまま錠を掛け直していない([config-v2-locks](2026-10-10-config-v2-locks.md) に「まだ掛け直していない」とある)。config-v2-locks の差分の書き方は check.py の形でない(E13)ので、その記録は書き換えず、この記録で掛け直す。

手元に #30 を取り込み、CSV の作業と合わせた(merge/csv-config-v2)うえで、check.py の E19 が挙げた試験だけを掛け直す。どれも #30 が書き換えた試験で、この作業では中身を変えていない。人は、この掛け直しを承認した。

## 差分

- 錠: BV-17(tests/test_new_note.rs)
- 錠: BV-19(src/ui/test_display_more.rs)
- 錠: BV-20(src/test_views_unit.rs)
- 錠: CE-16(tests/test_examples.rs)
- 錠: CE-20(src/ui/test_calendar.rs)
- 錠: CE-22(src/test_config_unit.rs・src/ui/test_calendar.rs・tests/test_date_format.rs・tests/test_examples.rs)
- 錠: CE-25(tests/test_new_note.rs)
- 錠: CE-26(tests/test_new_note.rs)
- 錠: CE-27(src/test_newnote_unit.rs・tests/test_new_note.rs)
- 錠: CE-33(src/ui/test_note_editor.rs)
- 錠: CLI-11(tests/test_config_docs.rs・tests/test_config_docs_values.rs・tests/test_display_options.rs・tests/test_language.rs)
- 錠: CLI-12(src/test_config_unit.rs・src/ui/test_keymap.rs・tests/test_config_docs.rs・tests/test_config_docs_values.rs・tests/test_display_options.rs・tests/test_new_note.rs)
- 錠: CLI-18(src/test_places_unit.rs・src/test_toml_edit_safety_unit.rs)
- 錠: NV-2(src/ui/test_search_bar.rs)
- 錠: NV-18(src/ui/test_display_more.rs)
- 錠: NV-23(src/test_config_unit.rs・src/ui/test_search_bar.rs)
- 錠: NV-24(src/ui/test_sort_more.rs)
- 錠: REL-5(src/ui/test_workspace.rs)
- 錠: SR-8(src/test_config_unit.rs・tests/test_editor_setting.rs)
- 錠: SR-20(src/ui/test_display_more.rs・src/ui/test_display_options.rs・src/ui/test_group_gap.rs・src/ui/test_themes.rs・src/ui/test_themes_more.rs)
- 錠: SR-21(tests/test_display_options.rs)
- 錠: SR-23(src/ui/test_view_tabs_auto.rs・tests/test_language.rs・tests/test_language_lib.rs・tests/test_no_japanese_literals.rs)
- 錠: SR-26(src/ui/test_look.rs・src/ui/test_themes.rs・src/ui/test_themes_more.rs)
- 錠: SR-27(src/ui/test_themes.rs)
- 錠: SR-30(src/ui/test_group_gap.rs・tests/test_display_options.rs)
- 錠: SR-32(src/ui/test_borders.rs)
- 錠: SR-33(src/ui/test_look.rs・src/ui/test_modern_rest.rs・src/ui/test_style_frames.rs)
- 錠: SR-34(src/ui/test_view_tabs_auto.rs)
- 錠: SR-35(src/test_cells_unit.rs・src/ui/test_rich_cells.rs・src/ui/test_rich_cells_more.rs・src/ui/test_style_screen.rs)
- 錠: SR-36(src/test_bounds_unit.rs・src/test_dozy_unit.rs・src/test_nerd_auto_unit.rs・src/test_style_unit.rs・src/ui/test_colors_screen.rs・src/ui/test_style_frames.rs・src/ui/test_style_list_pill.rs・src/ui/test_style_props.rs・src/ui/test_style_screen.rs・src/ui/test_style_selected.rs・tests/test_catalog_nerd.rs)
- 錠: SR-37(src/test_style_unit.rs・src/ui/test_themes.rs)
- 錠: SR-38(src/test_style_unit.rs・tests/test_catalog.rs・tests/test_catalog_nerd.rs)
- 錠: SR-39(src/test_bounds_unit.rs・src/test_style_unit.rs・tests/test_theme_auto_e2e.rs)
- 錠: SR-40(src/test_bounds_unit.rs・src/test_colors_unit.rs・src/ui/test_colors_more.rs・src/ui/test_colors_screen.rs・src/ui/test_style_props.rs)
- 錠: SR-41(src/test_bounds_unit.rs・src/test_colors_unit.rs・src/ui/test_colors_more.rs・src/ui/test_colors_screen.rs・src/ui/test_style_props.rs)
- 錠: SR-42(src/ui/test_header_buttons.rs)
- 錠: SR-43(src/test_look_unit.rs・src/ui/test_look_keep.rs・src/ui/test_look_section.rs)
- 錠: WB-2(tests/test_new_note.rs)
- 錠: WB-3(tests/test_e2e.rs・tests/test_empty_frontmatter.rs・tests/test_new_note.rs)
- 錠: WB-18(src/ui/test_calendar.rs)
- 錠: WS-1(src/test_toml_edit_safety_unit.rs・src/test_workspace_keep_unit.rs・src/test_workspace_more_unit.rs・src/test_workspace_unit.rs)
- 錠: WS-2(src/ui/test_workspace.rs)
- 錠: WS-4(src/ui/test_workspace.rs)
- 錠: WS-6(src/test_workspace_chosen_unit.rs・src/test_workspace_unit.rs・src/ui/test_workspace.rs)
- 錠: WS-7(src/test_toml_edit_safety_unit.rs・src/test_workspace_more_unit.rs)

## 却下した案

| 案 | 理由 |
|---|---|
| config-v2-locks の差分を check.py の形に書き直して、その記録で掛け直す | 採択した決定記録は書き換えない(伏せ字のほか) |

## 承認の記録

2026-10-10 人の承認(要約: #30 が書き換えた錠のある試験 196 本を、#30 の決定に基づいて掛け直してよい)
