---
type: Change
id: 01M491JG0RDNHTZMC0JBRJWJMB
title: 新しい欄と見出しで README の画像と動画を撮り直す(reshoot-assets)
status: done
size: tiny
created: 2026-10-07
updated: 2026-10-07
---

# 新しい欄と見出しで README の画像と動画を撮り直す(reshoot-assets)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: tiny | なし |
| 仕様化 | 対象外: 要件を変えない(SR-29・BV-24 の画面を写すだけ) | なし |
| 設計 | 対象外: tiny | なし |
| タスク | 対象外: tiny | なし |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

SR-29(左の欄)と BV-24(見出し)で画面が変わったので、README の docs/assets の画像(demo-*.svg)と demo.gif を、場面の定義(docs/manual-scenarios.toml)から撮り直す。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-29 | checked | 読んで判定: docs/assets/demo-group.svg を PNG にして見た。左の欄が `Fix login redirect bug` のように `Tasks/` と `.md` なしで出る。demo-save.svg は表を写さない場面なので変わらない |

確かめた: 1 / 1
