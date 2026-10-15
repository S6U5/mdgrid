---
type: Change
id: 01M4YG461MQKBM74JC9KQ87YZS
title: 印でビューと表を決める手引きのスキル mdgrid-views(views-skill)
status: active
size: full
created: 2026-10-15
updated: 2026-10-15
---

# 印でビューと表を決める手引きのスキル mdgrid-views(views-skill)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [2026-10-15-views-skill](../_decisions/2026-10-15-views-skill.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | (実装のコミット) |
| 照合 | 未 | この記録の「照合」 |

## 要求

印 `.mdgrid/workspace.toml` で表・ビュー・型・値の候補・ノートの欄を決めるときに設定が適当にならないよう、確かめ方の手順を持つスキルを置く(会話 2026-10-15。決定 views-skill)。要件: CLI-24。

## 不明点と仮定

- 仮定: 公開リポの CLI-22 のスキルと同じく英語で書き、同じプラグイン(plugins/mdgrid)の2つ目のスキルにする。

## 設計

- 置き場: `plugins/mdgrid/skills/mdgrid-views/SKILL.md`。試験 `tests/test_views_skill.rs` はスキルの最初の toml の区画を見本のフォルダに置いて読む。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | スキルを書く | CLI-24 | 手順と印の例がある | `plugins/mdgrid/skills/mdgrid-views/SKILL.md` | test_cli_24_skill_names_the_checklist | 済 |
| 2 | 印の例が読めて行が出るのを試験にする | CLI-24 | 警告なしに読め、ビューの行が合う | `tests/test_views_skill.rs` | test_cli_24_* | 済 |

## 実装の気づき

- 独立の試験役ではなく、スキルを書いたメインが試験も書いた(独立したテスト役ではない)。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-24 | unassessed | |

既存のテストの削除・skip・弱体化: なし

確かめた: 0 / 1
