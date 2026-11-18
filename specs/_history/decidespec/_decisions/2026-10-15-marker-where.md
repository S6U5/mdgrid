---
type: Proposal
id: 01M4XV323Q1AYCQKZY3GD25VPY
title: 印の表の base に、列の値を配列で並べて絞る where を書けるようにする(marker-where)
status: accepted
decided-by: AI
trigger: 人の発言
trigger-link: 会話 2026-10-15
touches: [WS-10]
created: 2026-10-15
updated: 2026-10-15
---

# 印の表の base に、列の値を配列で並べて絞る where を書けるようにする(marker-where)

## きっかけ

人が、台帳のビューを状態ごとのタブにするとき、設定ファイルでは表示する状態を配列の要素として書くべきだと言った(会話 2026-10-15。プロジェクトのチャット)。`.base` の式(`status == "todo"` を `or` で並べる)は値が増えるほど長くなり、手で書きにくい。

印の表の `base`(WS-9)に、列と値の配列で絞る `where` を足す。項目の名前と形は AI が足した細部。

## 差分

- 追加: WS-10(節「版1 — ワークスペース」)「印の表の `base`(WS-9)では、最上位とビューに `where`(列の名前を鍵に、値の配列を値にした表)を書いてよく、その列の値が配列のどれかと同じ行だけを残すべき(リストの列は、要素のどれかが当たれば残す)。鍵を2つ以上書いたときは全部を満たす行だけにし、同じ所の `filters` と両方を書いたときは両方を満たす行だけにするべき。値の配列の要素は文字列・数・真偽で、空の配列や配列でない値は、その鍵を CLI-3 の形で警告して使わないべき。」(確かめ方: `[[table.base.views]]` に `name = "未完了"`・`where = { status = ["todo", "doing"] }` → todo と doing の行だけ。`where = { status = ["done"], repo = ["tasks"] }` → done で repo が tasks の行だけ。最上位の `where = { kind = ["todo"] }` → どのビューも kind が todo の行だけ。tags が `[a, b]` のノートに `where = { tags = ["b"] }` → 残る。`where = { status = [] }` → `.mdgrid/workspace.toml: table[1].base.views[1].where.status:` で始まる警告と、絞らない表(`test_ws_10_*`))

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない(`filters` の式だけ) | 人は配列で書くことを求めた。値が増えるほど式が長くなる |
| `status` だけの専用の項目にする | 台帳の担当・リポなど、ほかの列でも同じ書き方が欲しくなる |

## 承認の記録

2026-10-15 AI 自己採択。新しい要件を足すだけで、守られる要件の文は変えない。人の言葉はプロジェクトのチャットで、会話の記録で照合できる形で残っていないため、人の決定にはしない。
