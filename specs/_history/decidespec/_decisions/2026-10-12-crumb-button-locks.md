---
type: Proposal
id: 01M4P0CK36PKN35DA7HTWWYR0M
title: パンくずのボタンでヘッダーの字が変わる試験の錠を掛け直す(crumb-button-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 7b2c6e4e5f4a77b1
decided-by: 人
trigger: AI の提案
trigger-link: 会話 2026-10-12
touches: [SR-1, BV-13, NV-15, SR-42, WS-6]
created: 2026-10-12
updated: 2026-10-12
---

# パンくずのボタンでヘッダーの字が変わる試験の錠を掛け直す(crumb-button-locks)

## きっかけ

人の決定 2026-10-12-crumb-button で、1行目のパンくずに引き出しのキーと「▾」を添えた(例「w Work › tasks ▾  3行」)。ヘッダーの字を決め打ちで確かめる既存の試験のうち5本は、SR-42 のほかの要件(SR-1・BV-13・NV-15・WS-6)の錠を持ち、その決定記録では掛け直せない。試験の直しは字を新しいヘッダーに合わせただけで、要件の中身は変えない。

## 差分

- 錠: SR-1(test_screen::test_sr_1_five_bands。「notes  3行」→「notes ▾  3行」)
- 錠: BV-13(test_grid::test_sr_1_base_header_and_tabs。「tasks.base  2行」→「tasks.base ▾  2行」と画面の写し sr_1_base)
- 錠: NV-15(test_settings_screen::test_nv_15_group_off_and_by_column。分類を外したとき見出しの「▾」が無いことを、1行目のパンくずを除いた行で確かめる)
- 錠: WS-6(test_workspace::test_ws_6_header_shows_workspace。「Work › 」→「w Work › 」)
- 錠: SR-42(test_view_bar::test_sr_42_crumb_scope_and_table(SR-42・WS-6)。「Work › 」→「w Work › 」)

## 却下した案

| 案 | 理由 |
|---|---|
| 分類の見出しの「▾」とぶつからないように、パンくずの印を別の字にする | 人が「▾」の画面図を選んで承認した。見出しの印とは行で分かれる |

## 承認の記録

2026-10-12 人の承認(要約: パンくずの字に合わせた5本の試験の錠を掛け直す提案に「いいよ」と答えた)
