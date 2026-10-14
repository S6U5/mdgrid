---
type: Proposal
id: 01M4QE540WRAN1QJD2ZD8GJZ2K
title: 余白の既定を 1 にして変わる試験の錠を掛け直す(padding-default-one-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 5d103c64a39b8ac1
decided-by: 人
trigger: AI の提案
trigger-link: 会話 2026-10-12
touches: [NV-23, SR-1, SR-26, CLI-11, SR-21]
created: 2026-10-12
updated: 2026-10-12
---

# 余白の既定を 1 にして変わる試験の錠を掛け直す(padding-default-one-locks)

## きっかけ

人の決定 2026-10-12-padding-default-one(上の余白とビューの帯の余白の既定を 0.5 から 1 に)で、既定のままでは半行を描かなくなった。半行の描き方を確かめる既存の試験は既定の 0.5 を前提にしていたので、設定で `top_padding = 0.5`・`bar_padding = 0.5` を明示する形に直した。設定の書き出し(`--print-config`)の試験は、既定の値を 0.5 から 1 にし、整数の `1` も受けるようにした。確かめる中身は消しも緩めもしていない。テーマ(SR-26)と設定の書き出し(CLI-11・SR-21)の錠も持つので、決定の記録だけでは掛け直せない。

直した試験:

| ファイル | 試験 | 直し |
|---|---|---|
| `src/ui/test_bar_padding_theme.rs` | test_nv_23_half_theme_background_nord・solarized_light・none_without_theme | 共有の起動の関数で 0.5 を明示 |
| `src/ui/test_half_row_inverse.rs` | test_nv_23_half_inverse_with_painted_background・top_padding_with_painted_background | 0.5 を明示 |
| `src/ui/test_half_row_inverse_termbg.rs` | test_nv_23_half_termbg_default_layout | 0.5 を明示 |
| `tests/test_display_options.rs` | test_sr_21_print_config_has_display | 既定の値 0.5 → 1(整数も受ける) |
| `src/ui/test_padding_default_one.rs` | test_nv_23_default_one_half_still_selectable | 整形だけ(cargo fmt) |

## 差分

- 錠: NV-23(test_bar_padding_theme・test_half_row_inverse・test_half_row_inverse_termbg・test_padding_default_one の上の試験。半行を設定で選ぶ形に)
- 錠: SR-1(test_bar_padding_theme::test_nv_23_half_theme_background_nord ほか、上の余白の半行を確かめる試験。同じ理由)
- 錠: SR-26(test_bar_padding_theme::test_nv_23_half_theme_background_nord・solarized_light。同じ理由)
- 錠: CLI-11(test_display_options::test_sr_21_print_config_has_display。書き出す既定の値が 1 になった)
- 錠: SR-21(test_display_options::test_sr_21_print_config_has_display。同じ理由)

## 却下した案

| 案 | 理由 |
|---|---|
| 半行の試験を既定のまま残し、既定が 1 なら空き行になることを確かめる形に変える | 半行の描き方(帯の色・地の色の字)を確かめる試験が無くなる |

## 承認の記録

2026-10-12 人の承認(要約: 余白の既定を 1 にして変わる試験7本の錠を掛け直す提案に「承認」と答えた)
