---
type: Proposal
id: 01M4NDJXGZBKMS4GDVGEXS1P38
title: 2つの決定にまたがる要件を持つ試験の錠を、まとめて掛け直す(view-bar-locks-2)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 5d103c64a39b8ac1
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-11
touches: [SR-20, SR-21, NV-2, NV-23, NV-1, NV-29, SR-12, CE-16, SR-18]
created: 2026-10-11
updated: 2026-10-12
---

# 2つの決定にまたがる要件を持つ試験の錠を、まとめて掛け直す(view-bar-locks-2)

## きっかけ

人の決定 2026-10-11-view-bar(配置)と 2026-10-11-view-bar-locks(配置の影響で変わる試験)で、試験を新しい配置に合わせて直した。直した試験のうち5本は、1本の試験が2つの決定にまたがる要件(例: SR-20 は view-bar、SR-21 は view-bar-locks)を確かめていて、どちらの決定記録でも錠を掛け直せない。試験の直しの中身は、2つの決定で人が認めた配置の変化に合わせたものだけで、新しく変えるものは無い。人が錠の掛け直しに OK と答えた(会話 2026-10-11 の要約)。

## 差分

- 錠: SR-20(test_display_options::test_sr_20_tabs_hidden_but_brackets_switch(SR-20・SR-21 の試験)。タブを隠しても検索の欄がビューの帯に残るので「表が1行広い」が「表は上がらない」に変わる)
- 錠: SR-21(同じ試験)
- 錠: NV-2(test_filter_window の検索の欄の行の探し方(「検索:」で始まる行 → 表より上で「検索」を含む行)。NV-2・NV-23 の試験と NV-1・NV-2 の試験が、この共通の部分を通る)
- 錠: NV-23(同じ(test_nv_29_english_bar_lead_is_search。英語の欄の頭が「\ Search…」))
- 錠: NV-1(同じ共通の部分を通る test_nv_29_names_find_and_search(試験の本体は変えていない))
- 錠: NV-29(test_nv_29_kept_after_restart(NV-29・SR-12)。共通の部分を通る)
- 錠: SR-12(同じ試験)
- 錠: CE-16(test_screen の表の行の位置(1行上がる)を data_y から求めるようにした試験(CE-16・SR-18))
- 錠: SR-18(同じ試験)

## 却下した案

| 案 | 理由 |
|---|---|
| 試験を2つに分けて、それぞれの決定で掛け直す | 確かめている中身(1つの操作の結果)を割ることになり、試験が読みにくくなる |

## 承認の記録

2026-10-12 人の承認(要約: 2つの決定にまたがる5本の試験の錠を掛け直す提案に「承認」と答えた)
