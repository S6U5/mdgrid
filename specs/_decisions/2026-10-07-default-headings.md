---
type: Proposal
id: 01M48ZBTVD008BKGEPQDT99PNE
title: displayName の無い列の見出しを Obsidian の既定にする(default-headings)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 9482395a0d440c01
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-06
touches: [BV-24]
created: 2026-10-07
updated: 2026-10-07
---

# displayName の無い列の見出しを Obsidian の既定にする(default-headings)

## きっかけ

会話 2026-10-06。displayName の無い `.base` の列は、見出しが `formula.x`・`file.name` のままで、Obsidian で見る表と見出しが違う(O-12)。AI が人の判断待ちの表(docs/todo.md の7節の2)に「Obsidian の既定の見出しにする」をおすすめとして並べ、人がおすすめどおりでよいと答えた。人の依頼を AI が要件の文に起こした。見出しの文字は Obsidian の文書(Bases syntax のファイルの項目の表)の説明の語に合わせた(AI の仮定。Obsidian の画面の言葉の全部は文書に無い)。

## 差分

- 追加: BV-24「`properties` に displayName の無い列の見出しは、Obsidian の既定に合わせるべき: 式の列(`formula.x`)は式の名前(`x`)、ファイルの項目は `file.name` → `file name`・`file.ctime` → `created time`・`file.mtime` → `modified time`・`file.ext` → `file extension`、それ以外の `file.x` → `file x`、ノートのキーの列はキーの名前。displayName があればそれを使うべき。」(確かめ方: order に `file.name`・`formula.価格`・`status` を並べた `.base` → 見出しは `file name`・`価格`・`status`。`file.name` に displayName `名前` → `名前`。`--print --format json` の鍵も同じ見出し(`test_bv_24_*`))

## 却下した案

| 案 | 理由 |
|---|---|
| id のまま | Obsidian で同じ `.base` を見たときと見出しが違い、`formula.` の前置きが列の幅を食う |

## 承認の記録

2026-10-06 人の承認(要約: 人の判断待ちの表の8件を、おすすめどおりでよいと承認した。この提案はその2)
