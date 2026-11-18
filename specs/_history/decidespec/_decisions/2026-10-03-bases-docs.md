---
type: Proposal
id: 01M3ZW8S1TVA8FXBKZXV7RDWC3
title: .base の対応範囲の文書(bases-docs)
status: accepted
decided-by: AI
trigger: AI の提案
trigger-link: docs/todo.md
touches: [BV-21]
created: 2026-10-03
updated: 2026-10-03
---

# .base の対応範囲の文書(bases-docs)

## きっかけ

docs/todo.md の D-3(公開の前に要る文書)。mdgrid は Obsidian Bases の `.base` の一部(table ビュー・一部の関数)だけを解釈し、評価できないものは BV-7 で未対応と出す。使う前に何が効くかを知る文書が無い。英・日の文書を置き、文書の関数の一覧と式の評価が受け付ける一覧のずれを試験で落とす。依頼に無い細部を AI が決めるので AI の決定とする(守られる要件に触らない)。

## 差分

base-view の spec.md の「要件: 版1 — 解釈」に足す。

- 追加: BV-21「`.base` のうち mdgrid が解釈するもの(読む項目・ビューの型・演算子・関数とメソッド・`file.*` の値)と、解釈しないもの(BV-7 で未対応と出すもの)を、英語(`docs/obsidian-bases.md`)と日本語(`docs/obsidian-bases.ja.md`)の文書に並べるべき。文書の関数・メソッド・`file.*` の一覧と、式の評価が受け付ける一覧が食い違えば試験で落ちるべき(文書が古くなるのを防ぐため)。」(確かめ方: 式の評価に関数を1つ足して文書を直さない → 試験が落ちる。文書に評価できない関数を書く → 落ちる。文書の対応する関数を使った絞り込みは BV-7 の未対応にならない(`test_BV_21`))

## 却下した案

| 案 | 理由 |
|---|---|
| README に短く書くだけにする | 関数の一覧は長く、ずれを試験で落とせない |
