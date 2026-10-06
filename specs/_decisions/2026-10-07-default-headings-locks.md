---
type: Proposal
id: 01M48ZX38SE3SDDMHQ7YM5NDDT
title: 既定の見出しに合わせて、見出しを鍵に使う試験の錠を掛け直す(default-headings-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 9482395a0d440c01
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-06
touches: [BV-22, BV-7]
created: 2026-10-07
updated: 2026-10-07
---

# 既定の見出しに合わせて、見出しを鍵に使う試験の錠を掛け直す(default-headings-locks)

## きっかけ

会話 2026-10-06。人の判断待ちの表(docs/todo.md の7節の2)の「Obsidian の既定の見出し」を人がおすすめどおりでよいと答え、BV-24 を足した(default-headings)。表の触るものの欄に「錠のある tests/test_base.rs・tests/test_print.rs」と並べていた錠の掛け直しを、ここで行う。要件の文は変えず、`--print --format json` の鍵と csv の見出し(`file.name` → `file name`、`formula.謎` → `謎`)を見ていた試験の錠だけを掛け直す(TL-9)。

## 差分

- 錠: BV-22(json の鍵を、既定の見出し `file name`・`file links`・式の名前 に替える。同じファイルの共有の手助けの関数が鍵を見る)
- 錠: BV-7(未対応の式の列の見出しと json の鍵を `formula.謎` から式の名前 `謎` に替える。tests/test_links.rs の this の試験は同じファイルの共有の部分が変わる)

## 却下した案

| 案 | 理由 |
|---|---|
| 試験に displayName を足して見出しを元のままにする | 既定の見出しを確かめる試験にならず、.base の文が変わるので錠の掛け直しはどのみち要る |

## 承認の記録

2026-10-06 人の承認(要約: 人の判断待ちの表の8件を、おすすめどおりでよいと承認した。これはその2の錠の掛け直し)
