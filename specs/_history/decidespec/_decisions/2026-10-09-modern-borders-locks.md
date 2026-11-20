---
type: Proposal
id: 01M4E2HR97ECNZ19NCVYN3Y7M6
title: 窓の枠をつながった罫線にしたのに合わせて、枠の文字を読む試験の錠を掛け直す(modern-borders-locks)
status: accepted
approval-evidence: [〔伏せ字: 会話の記録の場所〕, 〔伏せ字: 会話の記録の場所〕]
approval-hash: [b345587c18cb454a, 48c8a17e4fd13b44]
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-08
touches: [CE-10, CE-20, CE-22, CE-23, CE-26, CE-27, SR-23, SR-24, WB-18, BV-17]
created: 2026-10-09
updated: 2026-10-09
---

# 窓の枠をつながった罫線にしたのに合わせて、枠の文字を読む試験の錠を掛け直す(modern-borders-locks)

## きっかけ

会話 2026-10-08。窓の枠を既定でつながった罫線にした(SR-32。[modern-borders](2026-10-09-modern-borders.md))。錠のある試験のいくつかが、画面の文字の中の ASCII の枠(`|`・`+-`)を手がかりに、候補・カレンダー・操作の一覧の位置や中身を読んでいた。AI が、今ある窓もそろえる案(A)には錠の掛け直しが要ると示し、人が A を選んだ。

試験の確かめる中身(候補の値・選び・カレンダーの曜日と日・下の縁のキー・位置の番号)は変えていない。変えたのは、枠の文字(`|` → `│`、上の縁の `+-` → `╭─`、下の縁の `+-` → `├─`・`╰─`、位置の番号の `+1/` → `╰1/`)と、新しいノートの窓の縁(`│`)と候補の行を見分ける読み方だけ(TL-9)。見本の画像(tests/golden)は作り直した。

## 差分

- 錠: CE-10(src/ui/test_calendar.rs の一括の日付の試験。カレンダーの縁の読み方)
- 錠: CE-20(src/ui/test_calendar.rs。曜日の行と見出しの縁の読み方)
- 錠: CE-22(src/ui/test_calendar.rs。カレンダーの縁の読み方)
- 錠: CE-23(src/ui/test_calendar.rs。下の縁の行の読み方)
- 錠: SR-23(src/ui/test_language.rs。カレンダーの枠の読み方)
- 錠: SR-24(src/ui/test_list_narrow.rs。候補の行の枠と、操作の一覧の位置の番号の縁)
- 錠: CE-26(src/ui/test_new_note_gaps.rs。候補の行の枠と、新しいノートの窓の縁の見分け)
- 錠: CE-27(同じ試験の関数が CE-27 も確かめる)
- 錠: WB-18(src/ui/test_calendar.rs の同じ試験の関数が WB-18 も確かめる)
- 錠: BV-17(src/ui/test_new_note_gaps.rs の同じ試験の関数が BV-17 も確かめる)

## 却下した案

| 案 | 理由 |
|---|---|
| 今ある窓は ASCII のまま(案 B) | 人が A を選んだ |
| 試験の環境だけ ASCII にする | 既定の見た目(罫線)を試験が確かめなくなる |

## 承認の記録

2026-10-09 人の承認(要約: ASCII のモードは残してよいがモダンにする。錠の掛け直しが要る案 A を選んだ)
