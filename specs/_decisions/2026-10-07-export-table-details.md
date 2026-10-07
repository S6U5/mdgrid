---
type: Proposal
id: 01M4AZEAY95S064TSE3NMH15GM
title: 表の書き出しの細部(export-table-details)
status: accepted
decided-by: AI
trigger: AI の提案
trigger-link: specs/_decisions/2026-10-07-export-table.md
touches: [OUT-5]
created: 2026-10-07
updated: 2026-10-07
---

# 表の書き出しの細部(export-table-details)

## きっかけ

人の依頼(OUT-2・CLI-5。[export-table](2026-10-07-export-table.md))を作るには、依頼に無い細部を決める必要がある。AI が細部を決めて足す。

## 差分

- 追加: OUT-5「画面の表の書き出し(OUT-2)は、先頭の列に各行のノートのパス(`path`。`--print --with-path` と同じ)を入れ、書き出したファイルを `--apply` で戻せるようにするべき。ファイル名は起動したフォルダからの相対か絶対のパスとし、先頭の `~` はホームのフォルダにするべき。同じ名前のファイルが既にあれば、`y` で確かめてから上書きするべき。書いたら、書いた行の数とパスを知らせるべき。読むだけの起動でも書き出せてよい(ノートは書かない)。」(確かめ方: 書き出した CSV の1列目が `path`。`--apply` に渡すと差分なし。既にある `out.csv` → 確かめの問い、`n` で書かない。`~/out.csv` → ホームの下に書く(`test_out_5_*`))

## 却下した案

| 案 | 理由 |
|---|---|
| パスの列を入れない | どの行がどのノートか分からず、`--apply` で戻せない |
| 黙って上書きする | 別のファイルを消しうる |

## 承認の記録

AI 自己採択(人の依頼の細部で、守られる要件の文言には触らない。強さは「するべき」)
