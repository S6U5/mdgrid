---
type: Change
id: 01M4972YJ05HB47X1HM4ZM1TKS
title: 日本語の README の画像と動画、説明書の docs への配置、バッジと見出しの絵文字(docs-polish)
status: done
size: tiny
created: 2026-10-07
updated: 2026-10-07
---

# 日本語の README の画像と動画、説明書の docs への配置、バッジと見出しの絵文字(docs-polish)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: tiny(文書と画像だけ) | なし |
| 仕様化 | 対象外: 要件を変えない | なし |
| 設計 | 対象外: tiny | なし |
| タスク | 対象外: tiny | なし |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼(会話 2026-10-07): 日本語の README にも動画と画像を置く、説明書(手元の manual/、git の外)を整えて docs に置く、CI などのバッジ、README の見出しに適度に絵文字。

- 説明書は docs/manual/{en,ja}/ に置き、画面は今の版で docs/manual-scenarios.toml から撮り直す。参照の文書へのリンクを新しい場所に合わせる。
- 日本語の README は英語の README と同じ構成にし、画像と動画は日本語の画面で撮る(docs/assets/ja/)。scripts/demo-gif.sh に言語を選ぶ引数を足す。

## 照合

要件に触らない(文書と画像)。docs/manual/{en,ja}/ の参照の文書へのリンクが全部実在すること、日本語の画面の画像(docs/assets/ja/demo-edit-list.svg)を PNG にして見たこと、試験の全部(1325)が通ることを確かめた。

確かめた: 0 / 0
