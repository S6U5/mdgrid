---
type: Proposal
id: 01M472BFSSFM8HM31Y30BQFX8M
title: .base の組み込みの集計(summaries)を版1に入れる(summaries)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: docs/todo.md
touches: [BV-14]
created: 2026-10-06
updated: 2026-10-06
---

# .base の組み込みの集計(summaries)を版1に入れる(summaries)

## きっかけ

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-13)が、列の集計(数・合計・平均・最早・最遅)が無いのは表の道具で最も惜しまれると指摘した。BV-14 は「次の段」に「してよい」で置いてあった。Obsidian のヘルプ(https://obsidian.md/help/bases/syntax、2026-10-06 に読んだ)の組み込みの集計を、表の下の1行として版1に上げる。

## 差分

base-view の spec.md の BV-14 を「次の段」から「版1 — 解釈」に移し、文を変える。

- 変更: BV-14 旧「`.base` の `summaries`(列の集計: 空・埋まり・種類の数、数の合計・平均など、日付の最古・最新、チェックの数)を、グループの無いときは列の下、グループのあるときは各グループの先頭に出してよい。」→ 新「`.base` のビューの `summaries`(列の id → 集計の名前)のうち、Obsidian の組み込みの集計(Average・Min・Max・Sum・Range・Median・Stddev・Earliest・Latest・Checked・Unchecked・Empty・Filled・Unique)を、今のビューの行(絞り込みのあと)で計算し、表の下に1行で、集計した列の下に「集計の名前 値」を出すべき。集計の名前は英語と日本語で見せ、型の合わない値は数えないべき。式で書いた集計(最上位の `summaries` の formula)とまとまりごとの集計は解釈しないで、未対応の理由を出すべき。」(確かめ方: `summaries: {estimate: Sum, due: Earliest, done: Checked}` → 表の下に「Sum 18」「Earliest 2026-10-05」「Checked 2」。絞り込みで行が減ると値も変わる。`values.mean()` の集計 → 未対応の理由が出て、ビューは開く(`test_bv_14_*`))

## 却下した案

| 案 | 理由 |
|---|---|
| まとまりごとにも集計を出す | 見出しの行の幅と、畳んだまとまりの扱いが要る。まず表全体の1行 |
| 式の集計(values.mean() など)も入れる | 集計の式の評価(values のリスト)を足す仕事が大きい。組み込みで多くの使い方が足りる |

## 承認の記録

AI 自己採択(守られる要件に触らない。BV-14 の決定は AI。人は 2026-10-06 に判断を AI に委ねた)
