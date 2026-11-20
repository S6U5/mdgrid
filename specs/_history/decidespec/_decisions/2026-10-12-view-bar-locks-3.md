---
type: Proposal
id: 01M4NVDEY578A4S8MWHSHRNRX3
title: ヘッダーのワークスペースの名前がパンくずに移ったので、-w で開く試験の錠を掛け直す(view-bar-locks-3)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 5d103c64a39b8ac1
decided-by: 人
trigger: AI の提案
trigger-link: 会話 2026-10-12
touches: [WS-3]
created: 2026-10-12
updated: 2026-10-12
---

# ヘッダーのワークスペースの名前がパンくずに移ったので、-w で開く試験の錠を掛け直す(view-bar-locks-3)

## きっかけ

人の決定 2026-10-11-view-bar で、ヘッダーのボタン「ワークスペース Product」を外し、範囲の名前をパンくず「Product › projects」で示すことにした(WS-6・SR-42)。本物の実行ファイルで `mdgrid -w Product` を確かめる試験 tests/test_workspace_cli.rs::test_ws_3_w_opens_first_table が、前のボタンの字「ワークスペース Product」を待っていて、時間切れで落ちる。この試験は WS-3 の錠で、view-bar の決定は WS-3 に触れていないので、その記録では掛け直せない。WS-3 の中身(`-w <名前>` で最初の表を開く)は変えない。

## 差分

- 錠: WS-3(test_workspace_cli::test_ws_3_w_opens_first_table。待つ字を「ワークスペース Product」から、パンくずの「Product › projects」に変える)

## 却下した案

| 案 | 理由 |
|---|---|
| ヘッダーに「ワークスペース Product」の字を残す | パンくずで範囲を示すと決めた配置と重なり、1行目が狭くなる |

## 承認の記録

2026-10-12 人の承認(要約: -w で開く試験の待つ字をパンくずに変えて錠を掛け直す提案に「承認」と答えた)
