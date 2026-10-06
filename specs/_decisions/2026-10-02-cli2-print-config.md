---
type: Proposal
id: 01M3Y0WBSB72FB63JAVK6XV770
title: CLI-2 の起動の引数に --print-config を並べる(cli2-print-config)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: specs/_changes/2026-10-02-oss-config.md
touches: [CLI-2]
created: 2026-10-02
updated: 2026-10-02
---

# CLI-2 の起動の引数に --print-config を並べる(cli2-print-config)

## きっかけ

oss-config のレビューで、CLI-11 の `--print-config` が CLI-2 の受け付ける引数の列挙に無いと指摘された。

## 差分

- 変更: CLI-2
  - 旧: 「`--view <名前>`・`--readonly`・`--no-color`・`--config <パス>`・`--help`・`--version` を受け付けるべき。」
  - 新: 「`--view <名前>`・`--readonly`・`--no-color`・`--config <パス>`・`--print-config`(CLI-11)・`--help`・`--version` を受け付けるべき。」

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない | CLI-2 の列挙と実装・試験の札が食い違う |

## 承認の記録

2026-10-02 AI の自己採択(守られる要件に触れない。CLI-11 で採択済みの引数を列挙に足すだけ)
