---
type: Proposal
id: 01M488A5GBE1E9QC09A1T96ZBY
title: チェックボックスの Enter を真と偽の切り替えにする(checkbox-two-state)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 9482395a0d440c01
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-06
touches: [CE-4]
created: 2026-10-06
updated: 2026-10-06
---

# チェックボックスの Enter を真と偽の切り替えにする(checkbox-two-state)

## きっかけ

会話 2026-10-06。改善の繰り返しの点検(docs/todo.md の O-10)が、チェックボックスの Enter が 偽 → 空 と回るのは Obsidian の手触りと違うと指摘し、AI が人の判断待ちの表(docs/todo.md の7節の3)に「true と false だけを切り替える」をおすすめとして並べた。人がおすすめどおりでよいと答えた。人の依頼を AI が要件の文に起こした。

## 差分

- 変更: CE-4 旧「チェックボックスの列は、1回の操作で、元の値が入っているかどうかにかかわらず、空 → 真 → 偽 → 空 の順に3つの状態を回せるべき。空は CE-9 と同じく `key:` と書き、元にキーが無いノートで空に戻ったときは書かない(元に戻すのと同じ)。」→ 新「チェックボックスの列は、1回の操作で、真と偽を切り替えるべき(Obsidian のチェックボックスと同じ手触りにするため)。値が空かキーの無いノートでは真にするべき。空(CE-9 と同じく `key:`)にするのは、空にする操作(BS・Delete)だけにするべき。」(確かめ方: `done: false` のセルで Enter → `done: true`。`done: true` で Enter → `done: false`。キーの無いノートで Enter → `done: true`、もう一度 → `done: false`。`done:` の空のセルで Enter → `done: true`。BS → `done:`(`test_ce_4_two_state_*`))

## 却下した案

| 案 | 理由 |
|---|---|
| 今の3つの状態(空 → 真 → 偽 → 空) | Obsidian のチェックボックスは真と偽の2つで、偽のつもりで Enter を押すと空になる |

## 承認の記録

2026-10-06 人の承認(要約: 人の判断待ちの表の8件を、おすすめどおりでよいと承認した。この提案はその3)
