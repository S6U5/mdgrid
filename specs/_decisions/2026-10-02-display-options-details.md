---
type: Proposal
id: 01M3Y2C8HE5F64CTAEW1XAXCT2
title: 表の見せ方の設定の名前と既定(display-options-details)
status: accepted
decided-by: AI
trigger: AI の提案
trigger-link: specs/_decisions/2026-10-02-display-options.md
touches: [SR-21]
created: 2026-10-02
updated: 2026-10-02
---

# 表の見せ方の設定の名前と既定(display-options-details)

## きっかけ

display-options(SR-20)の設定の項目の名前と、既にある `search_bar`(NV-23)との関係を AI が決める。

## 差分

screen の spec.md の「要件: 版1 — 設定と見た目の状態」に足す。

- 追加: SR-21「SR-20 の設定は `[display]` の表に `row_numbers`(既定 false)・`zebra`(既定 false)・`column_lines`(既定 true)・`tabs`(既定 true)・`chips`(既定 true)を持ってよく、検索の欄は既にある最上位の `search_bar` をそのまま使ってよい(`[display]` に同じ名前を足さない)。ビューごとの値はビューの設定(settings)の `display` に、設定と違う項目だけを持ってよい。」(確かめ方: `[display] row_numbers = true` → 番号が出る。`--print-config` に `[display]` の5項目(読んで判定))

## 却下した案

| 案 | 理由 |
|---|---|
| `search_bar` を `[display]` に移す | 既にある設定を壊す。移すなら別の提案で移行の警告と一緒に |

## 承認の記録

2026-10-02 AI の自己採択(守られる要件に触れない。人の依頼の細部の補い)
