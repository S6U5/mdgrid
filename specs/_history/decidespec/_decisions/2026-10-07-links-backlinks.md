---
type: Proposal
id: 01M48X10F6GN0A4QX3T1GZE1RF
title: file.backlinks と file.hasLink() を評価する(links-backlinks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 9482395a0d440c01
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-06
touches: [BV-22, BV-7, SR-23, WB-3, WB-5, CE-22]
created: 2026-10-07
updated: 2026-10-07
---

# file.backlinks と file.hasLink() を評価する(links-backlinks)

## きっかけ

会話 2026-10-06。改善の繰り返しで file.links と this は入れたが(links-this)、file.backlinks・file.hasLink() は、錠のある試験が「未対応の式」の例に使っていたので外していた(links-this-narrow)。AI が人の判断待ちの表(docs/todo.md の7節の1)に「対応する(触るもの: 7つの要件の試験の錠)」をおすすめとして並べ、人がおすすめどおりでよいと答えた。人の依頼を AI が要件の文に起こした。錠の行の要件は文を変えず、例に使っていた試験の錠だけを掛け直す(TL-9)。

## 差分

- 変更: BV-22 旧「式は、ノートのリンクを評価するべき: `file.links`(そのノートの本文とフロントマターにある `[[名前]]`・`[[名前\|表示]]`・`[[名前#見出し]]`・`[文字](相対のパス.md)` の行き先のノートのリスト。行き先は Obsidian と同じく、保管庫の根からのパス、無ければ同じ名前のノート(同じ名前が複数なら根に近い・パスの短いもの)に解き、解けないリンクは文字のまま入れる)。`.base` を直接開いたときの `this` はその `.base` のファイルとし、`this.file.name`・`this.file.folder` などと `file.links.contains(this.file)` を評価するべき。」→ 新「式は、ノートのリンクを評価するべき: `file.links`(そのノートの本文とフロントマターにある `[[名前]]`・`[[名前\|表示]]`・`[[名前#見出し]]`・`[文字](相対のパス.md)` の行き先のノートのリスト。行き先は Obsidian と同じく、保管庫の根からのパス、無ければ同じ名前のノート(同じ名前が複数なら根に近い・パスの短いもの)に解き、解けないリンクは文字のまま入れる)・`file.backlinks`(そのノートを行き先に持つノートのリスト)・`file.hasLink(x)`(x はノートの名前・パス・`this.file`)。`.base` を直接開いたときの `this` はその `.base` のファイルとし、`this.file.name`・`this.file.folder` などと `file.hasLink(this.file)` を評価するべき。」(確かめ方: a.md の本文に `[[b]]` と `[x](sub/c.md)` → a の file.links に b と sub/c。`[[無い]]` は文字 "無い" のまま入る。Projects.base を開き `file.inFolder(this.file.folder)` → .base と同じフォルダのノート。`file.links.contains(this.file)` → .base を指すノート。b.md を a と c が指す → b の file.backlinks に a と c。`file.hasLink("b")` → b を指すノートで真。`file.hasLink(this.file)` → .base を指すノート(`test_bv_22_*`))
- 錠: BV-7(未対応の関数・項目の例(file.hasLink・file.backlinks)を、まだ未対応の link()・file.embeds に差し替える)
- 錠: SR-23(未対応の関数・項目の例(file.hasLink・file.backlinks)を、まだ未対応の link()・file.embeds に差し替える。英語と日本語の理由の試験の共有の .base の文)
- 錠: WB-3(未対応の関数・項目の例(file.hasLink・file.backlinks)を、まだ未対応の link()・file.embeds に差し替える。同じファイルの共有の .base の文が変わる)
- 錠: WB-5(未対応の関数・項目の例(file.hasLink・file.backlinks)を、まだ未対応の link()・file.embeds に差し替える。同じファイルの共有の .base の文が変わる)
- 錠: CE-22(未対応の関数・項目の例(file.hasLink・file.backlinks)を、まだ未対応の link()・file.embeds に差し替える。同じファイルの共有の .base の文が変わる)

## 却下した案

| 案 | 理由 |
|---|---|
| 未対応のまま | Obsidian の .base でよく使う被リンクの列と絞り込みが開けない |

## 承認の記録

2026-10-06 人の承認(要約: 人の判断待ちの表の8件を、おすすめどおりでよいと承認した。この提案はその1)
