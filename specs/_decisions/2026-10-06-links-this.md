---
type: Proposal
id: 01M4700GJB4N4C8Q51HTFJMRJR
title: リンク・被リンク・.base を開いたときの this を評価する(links-this)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: docs/todo.md
touches: [BV-22]
created: 2026-10-06
updated: 2026-10-06
---

# リンク・被リンク・.base を開いたときの this を評価する(links-this)

## きっかけ

会話 2026-10-06 の改善の繰り返しで、Obsidian を使う人の目の点検(docs/todo.md の O-5)が、リンクの多い保管庫でいちばん使われる「ここへのリンク」「被リンク」のビュー(`file.hasLink(this.file)`・`file.backlinks`)が開かないと指摘した。base-view の範囲外の「次の段: this を使う式とノートの中の base のブロック」のうち、`.base` を直接開いたときの this を版1に上げる(ノートの中の base のブロックは次の段のまま)。

## 差分

base-view の spec.md の「要件: 版1 — 解釈」に足し、範囲外の次の段の行を、ノートの中のブロックの this だけに直す。

- 追加: BV-22「式は、ノートのリンクを評価するべき: `file.links`(そのノートの本文とフロントマターにある `[[名前]]`・`[[名前\|表示]]`・`[[名前#見出し]]`・`[文字](相対のパス.md)` の行き先のノートのリスト。行き先は Obsidian と同じく、保管庫の根からのパス、無ければ同じ名前のノート(同じ名前が複数なら根に近い・パスの短いもの)に解き、解けないリンクは文字のまま入れる)・`file.backlinks`(そのノートを行き先に持つノートのリスト)・`file.hasLink(x)`(x はノートの名前・パス・`this.file`)。`.base` を直接開いたときの `this` はその `.base` のファイルとし、`this.file.name`・`this.file.folder` などと `file.hasLink(this.file)` を評価するべき。」(確かめ方: a.md の本文に `[[b]]` と `[x](sub/c.md)` → a の file.links に b と sub/c、b の file.backlinks に a。`file.hasLink("b")` は a で真。`[[無い]]` は文字 "無い" のまま入り、被リンクには出ない。Projects.base を開き `file.inFolder(this.file.folder)` → .base と同じフォルダのノート(`test_bv_22_*`))
- 変更: base-view の範囲外 旧「次の段: `this` を使う式と、ノートの中の ```` ```base ```` のブロックの解釈。」→ 新「次の段: ノートの中の ```` ```base ```` のブロックの解釈と、そのときの `this`(`.base` を直接開いたときの this は BV-22)。」

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない | リンクの多い保管庫で最も使われるビューが開かない |
| 解けないリンクを捨てる | Obsidian も解けないリンクを持つ。行き先の名前で絞る式が働かなくなる |
| 名前の解き方を Obsidian の設定(新しいリンクの形式など)に合わせて変える | まず既定(最短の一致)にそろえる。設定は使う人が出てから |

## 承認の記録

AI 自己採択(守られる要件に触らない。BV-6 と範囲外の次の段の決定は AI。人は 2026-10-06 に判断を AI に委ねた)
