---
type: Change
id: 01M79AB92BXWR0EJWQ29S9W66B
title: 公開リポの PR の題「docs:」を変更履歴の決まりに足す(cliff-docs)
status: done
size: tiny
created: 2026-11-13
updated: 2026-11-13
---

# 公開リポの PR の題「docs:」を変更履歴の決まりに足す(cliff-docs)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: tiny(設定の1行) | — |
| 仕様化 | 対象外: 要件の意味を変えない | — |
| 設計 | 対象外: tiny(設定の1行) | — |
| タスク | 対象外: tiny(設定の1行) | — |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

公開リポの側で作った PR をスカッシュマージした題が「docs: …」の形で main に入り(#51・#52・#57・#58・#59)、変更履歴の決まりの試験 `test_cliff_history_prefixes_all_covered` が落ちている(cliff.toml のどの parser にも当たらない)。人が直すと決めた(会話 2026-11-13)。何を: cliff.toml に `^docs[:：]` を Documentation の節として足す(手元の「文書:」と同じ扱い)。何ができたら終わりか: `cargo test --test test_cliff` が全部通る。

## 不明点と仮定

なし

## 設計

対象外: tiny。関わる束: `cargo test --locked --test test_cliff`。全体: `cargo test`・`cargo clippy --all-targets -- -D warnings`・`cargo fmt --check`

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 「docs:」の parser を足す | — | 履歴の書き出しの漏れが無い | cliff.toml, docs/design.md | test_cliff_history_prefixes_all_covered | 済 |

## 実装の気づき

なし

## レビューの指摘

なし

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|

要件の無いタスク1(設定の1行)の照合: checked。`cargo test --locked --test test_cliff` は直す前に `test_cliff_history_prefixes_all_covered` が落ち(「docs」で始まる題が当たらない)、直したあと 6 passed。

既存のテストの削除・skip・弱体化: なし

セキュリティの回: 対象外(tiny)

全体の走り: 1 分(`cargo fmt --check`・`cargo clippy --all-targets -- -D warnings`・`cargo test`。26 秒。2090 passed, 0 failed)

確かめた: 0 / 0(要件の無い変更。タスク1は上の文のとおり checked)
