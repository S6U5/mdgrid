---
type: Proposal
id: 01M3Y3WWBP38T3XB8541111NQE
title: 列の区切り線は入れると │ を引き、既定は線なし(column-lines-fix)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 8ad612b34c76609f
decided-by: 人
trigger: 検証の指摘
trigger-link: specs/_changes/2026-10-02-display-options.md
touches: [SR-20, SR-21]
created: 2026-10-02
updated: 2026-10-02
---

# 列の区切り線は入れると │ を引き、既定は線なし(column-lines-fix)

## きっかけ

display-options の受け入れの試験を書く役が、今の表の列の間は空白1つで `│` が無いため、SR-20 の「既定は今の見た目」と確かめ方の「区切り線を消す → `│` が無い」が同時に成り立たないと指摘した(SR-21 の `column_lines` の既定 true も同じ食い違い)。会話 2026-10-02 で人に尋ね、人は「入れると │ を引く、既定は線なし」を選んだ。

## 差分

- 変更: SR-20 の確かめ方
  - 旧: 「区切り線を消す → 列の間の `│` が無い。」
  - 新: 「区切り線を入れる → 列の間に `│` が出る。既定(今の見た目)では `│` が無い。」
- 変更: SR-21 の文
  - 旧: 「`column_lines`(既定 true)」
  - 新: 「`column_lines`(既定 false。true で列の間に `│` を引く)」

## 却下した案

| 案 | 理由 |
|---|---|
| 既定で `│` を引く | 人は今の見た目を既定のままにする方を選んだ |

## 承認の記録

2026-10-02 人の承認(要約: 列の区切り線は、入れると │ を引き、既定は線なしにすると選んだ)
