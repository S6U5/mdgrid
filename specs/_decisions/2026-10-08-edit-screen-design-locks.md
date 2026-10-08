---
type: Proposal
id: 01M4CV7494GEK7PCSWTH26HMR5
title: 新しいノートの窓の枠に合わせて、窓の欄を確かめる試験の錠を掛け直す(edit-screen-design-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 4276a4e82d8d7ce5
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-08
touches: [CE-25, CE-26]
created: 2026-10-08
updated: 2026-10-08
---

# 新しいノートの窓の枠に合わせて、窓の欄を確かめる試験の錠を掛け直す(edit-screen-design-locks)

## きっかけ

会話 2026-10-08。新しいノートの窓を枠のある窓にした(SR-31。[edit-screen-design](2026-10-08-edit-screen-design.md))ので、行の頭が窓の左の縁(`│`)になった。錠のある2つの試験が「行の頭が `> status`・`> 名前` で始まる」を確かめていたので、「行に含む」に変える。AI がこの掛け直しを示し、人がよいと答えた。確かめる中身(今の欄と値)は弱めていない(TL-9)。

## 差分

- 錠: CE-26(src/ui/test_new_note_more.rs の、今の欄の行の確かめを「始まる」から「含む」に)
- 錠: CE-25(同じ試験の関数が CE-25 も確かめる)

## 却下した案

| 案 | 理由 |
|---|---|
| 窓の左の縁を付けない | 枠のある窓という SR-31 の形にならない |

## 承認の記録

2026-10-08 人の承認(要約: 錠の掛け直しと、画像の撮り直しの範囲 A でよいと答えた)
