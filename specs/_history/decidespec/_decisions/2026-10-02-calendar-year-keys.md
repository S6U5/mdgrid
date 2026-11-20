---
type: Proposal
id: 01M3XXMTYFGJTX4ZM7CX8PVH6V
title: カレンダーで1年を飛ばす(calendar-year-keys)
status: accepted
decided-by: AI
trigger: AI の提案
trigger-link: 会話 2026-10-02
touches: [CE-24]
created: 2026-10-02
updated: 2026-10-02
---

# カレンダーで1年を飛ばす(calendar-year-keys)

## きっかけ

calendar-month-keys で月を飛ばすキーを足すのに合わせて、遠い日(翌年の締め切りなど)へ早く動けるよう、AI が1年を飛ばすキーを足すことを提案する。依頼に無い細部なので、別の提案にする。

## 差分

cell-edit の spec.md の「要件: 版1 — 型ごとの入り方」に足す。

- 追加: CE-24「カレンダーは、Shift+↑・Shift+↓ で1年動かしてよく、そのキーをカレンダーの下の縁に示してよい。」(確かめ方: 2026-10-30 で Shift+↓ → 2027-10-30。2028-02-29 で Shift+↓ → 2029-02-28(`test_CE_24`))

## 却下した案

| 案 | 理由 |
|---|---|
| 年は打ち込みだけ | 打ち込み(CE-20)でも足りるが、カレンダーで選んでいる途中に年を変えるには打ち直しが要る |

## 承認の記録

2026-10-02 AI の自己採択(守られる要件に触れない。依頼の細部の補い)
