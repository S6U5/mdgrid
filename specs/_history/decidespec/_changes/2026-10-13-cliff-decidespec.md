---
type: Change
id: 01M4TASDYBFQDAYWRJ5THD5A9D
title: 変更履歴の書き出しで、仕様駆動の道具の更新のコミットを節に出さない
status: done
size: tiny
created: 2026-10-13
updated: 2026-10-13
---

# 変更履歴の書き出しで、仕様駆動の道具の更新のコミットを節に出さない

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: tiny(設定の1行) | |
| 仕様化 | 対象外: tiny(要件の意味を変えない) | |
| 設計 | 対象外: tiny | |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

tests/test_cliff.rs::test_cliff_history_prefixes_all_covered が前から落ちていた。仕様駆動の道具の更新のコミット(件名の頭 `decidespec:`)が、変更履歴の書き出しの設定 cliff.toml の commit_parsers のどれにも当たらないため。利用者に見える変化でないので、節に出さない(skip)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | cliff.toml に `^decidespec[:：]` を skip で足す | なし(作る道具の設定) | test_cliff が全部通る | `cliff.toml` | tests/test_cliff.rs の6本 | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。tiny で、試験が全部 checked のため検証役は起動しない)。

| 要件 | 結果 | 証拠 |
|---|---|---|

既存のテストの削除・skip・弱体化: なし(試験は変えていない)

確かめた: 0 / 0。`cargo test --test test_cliff` の6本が通った(前は test_cliff_history_prefixes_all_covered が落ちていた)。
