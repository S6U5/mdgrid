---
type: Proposal
id: 01M471318CA2E3H9CBSH5EG3MT
title: BV-22 から file.backlinks・file.hasLink を外す(links-this-narrow)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: specs/_changes/2026-10-06-links-this.md
touches: [BV-22]
created: 2026-10-06
updated: 2026-10-06
---

# BV-22 から file.backlinks・file.hasLink を外す(links-this-narrow)

## きっかけ

BV-22(links-this)の実装で、錠のある試験(BV-3・BV-7・CE-22・SR-23・WB-3・WB-5・CE-16 の錠)と見本の タスク.base が、file.backlinks・file.hasLink を「未対応の式」の例に使っていると分かった。対応すると錠のある試験が落ち、掛け直すにはそれらの要件に触る人の決定が要る。人の決定までは、この2つを未対応のまま残し、file.links と this だけを入れる。

## 差分

- 変更: BV-22 旧「式は、ノートのリンクを評価するべき: `file.links`(そのノートの本文とフロントマターにある `[[名前]]`・`[[名前\|表示]]`・`[[名前#見出し]]`・`[文字](相対のパス.md)` の行き先のノートのリスト。行き先は Obsidian と同じく、保管庫の根からのパス、無ければ同じ名前のノート(同じ名前が複数なら根に近い・パスの短いもの)に解き、解けないリンクは文字のまま入れる)・`file.backlinks`(そのノートを行き先に持つノートのリスト)・`file.hasLink(x)`(x はノートの名前・パス・`this.file`)。`.base` を直接開いたときの `this` はその `.base` のファイルとし、`this.file.name`・`this.file.folder` などと `file.hasLink(this.file)` を評価するべき。」→ 新「式は、ノートのリンクを評価するべき: `file.links`(そのノートの本文とフロントマターにある `[[名前]]`・`[[名前\|表示]]`・`[[名前#見出し]]`・`[文字](相対のパス.md)` の行き先のノートのリスト。行き先は Obsidian と同じく、保管庫の根からのパス、無ければ同じ名前のノート(同じ名前が複数なら根に近い・パスの短いもの)に解き、解けないリンクは文字のまま入れる)。`.base` を直接開いたときの `this` はその `.base` のファイルとし、`this.file.name`・`this.file.folder` などと `file.links.contains(this.file)` を評価するべき。」(確かめ方: a.md の本文に `[[b]]` と `[x](sub/c.md)` → a の file.links に b と sub/c。`[[無い]]` は文字 "無い" のまま入る。Projects.base を開き `file.inFolder(this.file.folder)` → .base と同じフォルダのノート。`file.links.contains(this.file)` → .base を指すノート(`test_bv_22_*`))

## 却下した案

| 案 | 理由 |
|---|---|
| 要件の文を変えずに錠を掛け直す決定記録を作る | 錠の掛け直しは、錠の要件に触る人の決定でだけ行う決まり(AGENTS.md)。要件の文を変えない記録では触れない |

## 承認の記録

AI 自己採択(守られる要件に触らない。BV-22 は AI の決定。file.backlinks・file.hasLink は docs/todo.md の O-5 に人の判断待ちとして残す)
