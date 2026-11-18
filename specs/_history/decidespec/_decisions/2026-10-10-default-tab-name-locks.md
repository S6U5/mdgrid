---
type: Proposal
id: 01M4JKC7TRQ496GHTVNXWZFSKM
title: 英語の既定の表の名前を Default table にし、名前を確かめる試験の錠を掛け直す(default-tab-name-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: e2a183dddc1553b7
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-10
touches: [SR-23]
created: 2026-10-10
updated: 2026-10-10
---

# 英語の既定の表の名前を Default table にし、名前を確かめる試験の錠を掛け直す(default-tab-name-locks)

## きっかけ

会話 2026-10-10。CSV・TSV の表(SC-15〜SC-17)では行がノートでないので、英語の既定の表の名前「All notes」が合わない。人が、影響が広くても今後の足枷にならない方を取るよう指示した。日本語の「既定の表」と同じ意味の、形式によらない名前「Default table」にする。

要件の文(SR-23)は変わらない(既定の表の名前は文言)。錠のある試験 src/ui/test_view_tabs_auto.rs::test_sr_23_default_tab_english_name が、英語の名前を All notes と確かめているので、新しい名前を確かめ、前の名前(Default と All notes)を別名として受けてビューの名前に使えないことを確かめるように直す。

## 差分

- 錠: SR-23(src/ui/test_view_tabs_auto.rs::test_sr_23_default_tab_english_name。名前を Default table に、前の名前 All notes も別名として受ける確かめを足す)

## 却下した案

| 案 | 理由 |
|---|---|
| CSV の表のときだけ英語の名前を変える | 形式ごとの名前の特例が増え、ビューの名前の予約や places.toml の名前の受け方が形式で分かれる(今後の足枷) |
| 名前を変えずに残す | ノートでない表(CSV、これから足す形式)で意味が合わない |

## 承認の記録

2026-10-10 人の承認(要約: 影響が広くても今後の足枷にならない方を取る。それに沿って英語の既定の表の名前を変える)
