---
type: Proposal
id: 01M479MARASKJEPGTVABF06F73
title: 隠しフォルダと node_modules の下のノートを探さない(skip-hidden-dirs)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: docs/todo.md
touches: [BV-23]
created: 2026-10-06
updated: 2026-10-06
---

# 隠しフォルダと node_modules の下のノートを探さない(skip-hidden-dirs)

## きっかけ

会話 2026-10-06 の改善の繰り返しで、4回目の点検(docs/todo.md の B-5)が、コードのリポで `mdgrid` を開くと `node_modules/` の依存の README や `.hidden/` のノートまで行になり、`.obsidian`・`.git`・`.trash` だけを飛ばす今の扱いがばらばらだと指摘した。Obsidian は名前が `.` で始まるフォルダを保管庫に入れない。

## 差分

- 追加: BV-23「ノートを探すときは、名前が `.` で始まるフォルダ(Obsidian が保管庫に入れない隠しフォルダ)と `node_modules` の下を探さないべき(コードのリポを開いたときに依存のパッケージの文書を行にしないため)。渡したフォルダそのものが隠しフォルダでも、その中は探してよい。」(確かめ方: `a.md`・`.hidden/b.md`・`node_modules/p/README.md`・`sub/c.md` のフォルダを --print → a と sub/c の2行。`.hidden` を直接渡す → b の1行(`test_bv_23_*`))

## 却下した案

| 案 | 理由 |
|---|---|
| `.gitignore` を読む | 書き方(否定・`**`・入れ子の .gitignore)を正しく読むには部品(ignore の crate)を足す大きさになる。隠しフォルダと node_modules で点検の例の大半は消える。要るなら別の提案で |
| 何もしない | コードのリポで依存の README が行に混ざる |

## 承認の記録

AI 自己採択(新しい要件を足すだけで、守られる要件の文言を変えない。人は 2026-10-06 に判断を AI に委ねた)
