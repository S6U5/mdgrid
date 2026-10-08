---
type: Change
id: 01M4DNCBD94409CHY8FWCS4FF3
title: 見本 ai-human の英語版を足し、英語の README の画像を英語で撮り直す
status: done
size: full
created: 2026-10-08
updated: 2026-10-08
---

# 見本 ai-human の英語版を足し、英語の README の画像を英語で撮り直す

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 見本と文書の画像だけで、mdgrid の振る舞いは変えない | なし |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼(会話 2026-10-08 の要約): 英語の README の画像 docs/assets/demo-ai-human.svg が日本語のままで、文字が横に引き伸ばされてサイズがおかしく見える。英語にしたい。

## 不明点と仮定

- 原因: 画像を作る道具(tui_shot)が、全角の文字を半角2つ分の幅に textLength で引き伸ばして描くため、日本語の画像では字が横長になる。英語にすればこの画像では起きない。日本語の画像の見え方は道具の側の直しなので、ここでは扱わない(人に報告する)。
- 仮定: 英語版は examples/ai-human/vault-en/(Tasks.base と Tasks/)。設定(config.toml)の列の名前はもともと英語なので共通にする。
- 仮定: 日本語の README の画像(docs/assets/ja/demo-ai-human.svg)は今のまま。

## 設計

- examples/ai-human/vault-en/: vault/ と同じ 11 のタスクを英語にしたもの。式・ビュー・表示名も英語。
- docs/manual-scenarios.toml: 場面 demo-ai-human-en(英語の見本)を足す。英語で撮って docs/assets/demo-ai-human.svg に写す。
- examples/ai-human/README.md と README.md: 英語版の開き方。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 英語版の見本を作り、README の画像を撮り直す | なし | 英語の画像が日本語の画像と同じ形で撮れる | examples/ai-human/, docs/manual-scenarios.toml, docs/assets/demo-ai-human.svg, README.md | 読んで判定(撮った画像を見る) | 済 |

## 実装の気づき

- 式の名前を小文字の owner にすると、まとまりの見出しが「owner:」と小文字で出た。式の名前を Owner にした。
- 名前の欄に収まらない2つのタスクの名前を短くした(Get a residence certificate・Thank the client by email)。

## 照合

要件を足さない変更なので、撮って確かめた。英語の見本を --print で出し、日本語版と同じ分け方(人 5・AI 4)と並びになることを見た。場面 demo-ai-human-en を英語で撮り、画像を描いて、英語の字が引き伸ばされずに収まり、見出し・列・名前が欠けないことを見た。

確かめた: 0 / 0。

