---
type: Proposal
id: 01M5SSC5TF2Z3P4594XE7CY0WR
title: 説明書とサイトの入口を、読む人の順に並べ、人が読んで温度のある文で書く(docs-reading-order)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 145b00cd6916f7ff
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-25
touches: [SR-50]
created: 2026-10-26
updated: 2026-10-26
---

# 説明書とサイトの入口を、読む人の順に並べ、人が読んで温度のある文で書く(docs-reading-order)

## きっかけ

人がサイトの説明書を見て、三つを指摘した(会話 2026-10-25): 使う前の人に、自分がどう使えるかが伝わらない(細かく書くより先に、何ができるかが要る)。章の並びが、読む人の順になっていない。文に AI の書き方の癖があるので、世間の知見を参照して直す。あわせて、サイトは docs サイトの形(「mdgrid docs」)に寄せたいと言った。世間の知見(Wikipedia の「Signs of AI writing」、日本語の AI 文の癖の一覧、Diátaxis と docs サイトの入口の決まり)を `docs/writing.md` にまとめ、並べ方と文の決まりを要件にする。

## 差分

- 追加: SR-50「説明書の目次(`docs/manual/<言語>/index.md`)と、サイトの説明書の章の並び(`scripts/pages.py` の `PAGES`)は、読む人の順に並べるべき: 誰のための道具で何がどう見えるか(index)→ 見せ方の一覧(views)→ はじめかた(getting-started。1分で試す道を1本)→ 作業の手引き(tasks)→ 考え方(concepts)→ 見た目(appearance・themes)→ 困ったとき(faq)→ 参照。入口は短く保ち、詳しさは下のページへ送るべき。説明書・サイト・README の文は `docs/writing.md` の決まり(具体から書く、語尾と長さを変える、接続詞を連ねない、箇条書きは並列のものだけ、大げさな語・対比の型・三つ組・太字の強調・締めの定型句を使わない)で書くべき。」(確かめ方: `docs/manual/ja/index.md` と `en/index.md` の目次のリンクが views → getting-started → tasks → concepts → appearance → faq の順に並び、`PAGES` も同じ順である(`test_sr_50_*`)。入口と説明書の文に、大げさな語(強力・シームレス・直感的・画期的・最適・豊富 / seamless・robust・powerful・intuitive・leverage・empower・delve・crucial・pivotal)が無い(`test_sr_50_*`)。文の決まりに合うかは、`docs/writing.md` の一覧に照らして読んで判定)

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない(今の目次のまま文だけ直す) | 今の目次は「はじめかた → 考え方 → 手引き」で、使いたい人が自分の場面を思い浮かべる前に仕組みの説明が来る。人の指摘はまず並びの問題。 |
| 文の決まりを要件にせず、記憶や会話の中だけに置く | 書くのは毎回別のセッションで、決まりがリポに無いと同じ癖が戻る。`docs/writing.md` に置き、AGENTS.md から指す。 |
| 大げさな語の有無を機械で検査しない | 一覧の語だけでも機械で止められ、残りは読んで判定にする方が、読み直しを忘れない。 |

## 承認の記録

2026-10-25 人の承認(要約: 使いたい人がどう使えるかが先に見える順に並べ、文の AI っぽさを世間のナレッジを参照して抜く)
