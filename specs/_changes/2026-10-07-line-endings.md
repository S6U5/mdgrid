---
type: Change
id: 01M49420V38WW63R7DC1F1WENE
title: Windows の checkout で改行を LF に保つ(line-endings)
status: done
size: tiny
created: 2026-10-07
updated: 2026-10-07
---

# Windows の checkout で改行を LF に保つ(line-endings)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: tiny | なし |
| 仕様化 | 対象外: 要件を変えない | なし |
| 設計 | 対象外: tiny | なし |
| タスク | 対象外: tiny | なし |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

公開リポの最初の CI で、Windows の checkout が tests/golden/*.txt の改行を CRLF に変え、画面の試験が25件落ちた。`.gitattributes` で全部のテキストを LF に保つ。

## 照合

要件に触らない(改行の設定)。`git add --renormalize .` で変わるファイルが無い(全部 LF)ことを確かめた。Windows の CI で確かめる。

確かめた: 0 / 0
