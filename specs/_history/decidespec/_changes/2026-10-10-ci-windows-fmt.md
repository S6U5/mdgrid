---
type: Change
id: 01M4J3NC415E1YYDFZJR1Y0B9P
title: 公開リポの CI の直し(疑似端末の試験を unix だけに・整形)
status: done
size: tiny
created: 2026-10-10
updated: 2026-10-10
---

# 公開リポの CI の直し(疑似端末の試験を unix だけに・整形)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: CI の失敗の直し | — |
| 仕様化 | 対象外: 要件の意味を変えない | — |
| 設計 | 対象外: 2か所 | — |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

公開リポの PR の CI で、Windows の test が tests/test_theme_auto_e2e.rs の疑似端末の道具(unix だけ)を組めずに落ち、lint が tests/test_pages.rs の整形で落ちた。疑似端末の試験はほかと同じく unix だけにし、整形を当てる。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | unix だけ・整形 | SR-39 | CI の lint と Windows の組み立てが通る | tests/test_theme_auto_e2e.rs, tests/test_pages.rs | 公開リポの CI | 済 |

## 実装の気づき

- どちらも錠のある試験なので、specs/_decisions/2026-10-10-ci-locks.md(人の承認)で錠を掛け直した。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-39 | checked | テスト: tests/test_theme_auto_e2e.rs::test_sr_39_e2e_auto_theme_without_answer_starts(unix で今までどおり通る)、cargo fmt --check |

既存のテストの削除・skip・弱体化: なし(Windows では疑似端末の試験を組まない。ほかの疑似端末の試験と同じ)

確かめた: 1 / 1
