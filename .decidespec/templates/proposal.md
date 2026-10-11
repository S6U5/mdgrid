---
type: Proposal
id: {ULID}
title: {変更の名前}
status: proposed
decided-by: {人 / AI}
trigger: {人の発言 / 運用で起きた失敗 / 検証の指摘 / AI の提案}
trigger-link: {出典へのリンク。リポの外の会話なら「会話 YYYY-MM-DD」}
touches: [{XX-1}]
created: {YYYY-MM-DD}
updated: {YYYY-MM-DD}
---

# {変更の名前}

## きっかけ

{1文の要約。人の発言は原文で引用せず要約し、会話なら日付で指す}

## 差分

- 追加: {XX-n}「{全文}」(確かめ方: {…})
- 変更: {XX-n} 旧「{今の文言をそのまま}」→ 新「{…}」
- 削除: {XX-n}(理由: {…})

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない | {なぜ今のままではだめか} |

## 承認の記録

{YYYY-MM-DD 人の承認(要約: {…})(check.py --approve <記録> --find <語句> で照合し、approval-evidence と approval-hash を付ける)/ 未承認 / AI 自己採択(理由)}
