---
type: Change
id: 01M4AZZHM16K2R9V7YBPZN96F0
title: Windows でもノートのパス(--with-path・--pick path)を / でつなぐ(windows-path-slash)
status: done
size: bugfix
created: 2026-10-07
updated: 2026-10-07
---

# Windows でもノートのパス(--with-path・--pick path)を / でつなぐ(windows-path-slash)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | なし |
| 仕様化 | 対象外: bugfix(CLI-14 のパスの形。--pick path(OUT-3)も同じ関数。試験の文書は「/ でつないだもの」) | なし |
| 設計 | 対象外: bugfix | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、CI が落ちているので直すよう頼んだ(会話 2026-10-07)。公開リポの CI の Windows の job が毎回 tests/test_flow_frontmatter.rs::test_wb_5_flow_frontmatter_values_shown で落ちる。`--print --with-path` の path が Windows では `v\\a.md`(OS の区切り)になり、`v/a.md` と違う。CLI-14 の試験の文書のとおり、引数のフォルダとノートの相対のパスを `/` でつなぐ(Windows でも `/` の区切りのパスは読める。CSV を OS をまたいで戻せる)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | pick_path を / でつなぐ | CLI-14 | 公開リポの CI の Windows の job が通る | src/main.rs | test_wb_5_flow_frontmatter_values_shown(tests/test_flow_frontmatter.rs) | 済 |

## 実装の気づき

- 手元(macOS)では区切りがもともと / なので、Windows での結果は公開リポの CI で確かめる。画面の表の書き出し(OUT-5)の path も同じ関数なので / になる。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-14 | checked | コード: src/main.rs の slash / テスト: tests/test_flow_frontmatter.rs::test_wb_5_flow_frontmatter_values_shown / 今: 通った(macOS。全体 1342 passed) / 前: 落ちた(公開リポの CI の Windows で path が v\\a.md) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
